//! Streaming subprocess runner.
//!
//! [`Runner`] spawns a child process with piped stdout/stderr, ferries
//! output line-by-line into an [`Event`] channel, and exposes a
//! non-blocking [`Runner::poll`] suitable for being drained on each
//! Godot `_process` tick. This is the v0.3.0 building block that lets
//! the [`crate::Engine`] feed live wasm-module output into Godot
//! signals without blocking the main thread.
//!
//! Lifecycle:
//!
//! 1. `Runner::spawn(cmd)` starts the child and three helper threads:
//!    a stdout reader, a stderr reader, and a waiter that joins the
//!    readers and reaps the child. The waiter is the only thread that
//!    sends [`Event::Finished`] / [`Event::Failed`], guaranteeing those
//!    events arrive *after* every line of output.
//! 2. The caller polls [`Runner::poll`] to drain queued events.
//! 3. [`Runner::stop`] (or `Drop`) kills the child if still running.
//!    The waiter thread is then joined so no orphan threads remain.
//!
//! Ordering note: stdout and stderr are interleaved in delivery order,
//! but each individual stream preserves its own line order.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result};

/// One event in a [`Runner`]'s output stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A complete line from the child's stdout (no trailing newline).
    Stdout(String),
    /// A complete line from the child's stderr (no trailing newline).
    Stderr(String),
    /// The child exited with the given status code. `-1` indicates the
    /// process was terminated by a signal (no exit code available).
    Finished(i32),
    /// The runner failed to wait/reap the child. Contains a
    /// human-readable description.
    Failed(String),
}

/// A running subprocess whose stdout/stderr are streamed as [`Event`]s.
pub struct Runner {
    rx: Receiver<Event>,
    child: Arc<Mutex<Option<Child>>>,
    running: Arc<AtomicBool>,
    waiter: Option<JoinHandle<()>>,
}

impl Runner {
    /// Spawn `cmd` with piped stdout/stderr and start the reader and
    /// waiter threads. The supplied [`Command`]'s stdio settings are
    /// overridden — stdin is closed, stdout/stderr are captured.
    pub fn spawn(mut cmd: Command) -> Result<Self> {
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().context("failed to spawn child process")?;

        let stdout = child.stdout.take().expect("stdout was piped above");
        let stderr = child.stderr.take().expect("stderr was piped above");

        let (tx, rx) = mpsc::channel::<Event>();

        let tx_out = tx.clone();
        let stdout_thread = thread::Builder::new()
            .name("clawasm-engine-stdout".into())
            .spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    if tx_out.send(Event::Stdout(line)).is_err() {
                        break;
                    }
                }
            })
            .context("spawning stdout reader thread")?;

        let tx_err = tx.clone();
        let stderr_thread = thread::Builder::new()
            .name("clawasm-engine-stderr".into())
            .spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    if tx_err.send(Event::Stderr(line)).is_err() {
                        break;
                    }
                }
            })
            .context("spawning stderr reader thread")?;

        let child = Arc::new(Mutex::new(Some(child)));
        let running = Arc::new(AtomicBool::new(true));

        let child_for_wait = Arc::clone(&child);
        let running_for_wait = Arc::clone(&running);
        let waiter = thread::Builder::new()
            .name("clawasm-engine-waiter".into())
            .spawn(move || {
                // Drain readers first so all output arrives before
                // Finished. Both threads exit naturally on EOF, which
                // happens when the child closes the pipes (typically at
                // exit or when killed).
                let _ = stdout_thread.join();
                let _ = stderr_thread.join();

                let event = {
                    let mut guard = child_for_wait.lock().expect("child mutex");
                    match guard.as_mut() {
                        Some(child) => match child.wait() {
                            Ok(status) => Event::Finished(status.code().unwrap_or(-1)),
                            Err(e) => Event::Failed(format!("wait failed: {e}")),
                        },
                        // Already reaped (e.g. by stop() racing wait()).
                        None => Event::Finished(-1),
                    }
                };
                // Clear the slot so Drop doesn't try to wait again.
                child_for_wait.lock().expect("child mutex").take();
                running_for_wait.store(false, Ordering::SeqCst);
                let _ = tx.send(event);
            })
            .context("spawning waiter thread")?;

        Ok(Self {
            rx,
            child,
            running,
            waiter: Some(waiter),
        })
    }

    /// Drain all currently-queued events without blocking.
    pub fn poll(&self) -> Vec<Event> {
        let mut out = Vec::new();
        while let Ok(ev) = self.rx.try_recv() {
            out.push(ev);
        }
        out
    }

    /// Block until the next event is available (test/debug helper).
    pub fn recv_blocking(&self) -> Option<Event> {
        self.rx.recv().ok()
    }

    /// Returns `true` until the waiter thread has reaped the child.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Kill the child if still running. The waiter thread will then
    /// reap it and emit [`Event::Finished`]. Idempotent.
    pub fn stop(&mut self) {
        let mut guard = self.child.lock().expect("child mutex");
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
        }
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        self.stop();
        if let Some(h) = self.waiter.take() {
            let _ = h.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// Drain events with a timeout, used by the streaming tests so a
    /// hung child can't wedge CI.
    fn drain_until_finished(runner: &Runner, timeout: Duration) -> Vec<Event> {
        let mut events = Vec::new();
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            for ev in runner.poll() {
                let is_terminal = matches!(ev, Event::Finished(_) | Event::Failed(_));
                events.push(ev);
                if is_terminal {
                    return events;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("timed out waiting for terminal event; got: {events:?}");
    }

    #[test]
    fn streams_stdout_lines_and_exit_code() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg("printf 'alpha\\nbeta\\n'; printf 'oops\\n' >&2; exit 7");
        let runner = Runner::spawn(cmd).expect("spawn sh");
        let events = drain_until_finished(&runner, Duration::from_secs(5));

        assert!(events.contains(&Event::Stdout("alpha".into())));
        assert!(events.contains(&Event::Stdout("beta".into())));
        assert!(events.contains(&Event::Stderr("oops".into())));
        // Finished must be the last event in the stream.
        assert!(matches!(events.last(), Some(Event::Finished(7))));
    }

    #[test]
    fn finished_arrives_after_all_output() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("printf 'one\\ntwo\\nthree\\n'");
        let runner = Runner::spawn(cmd).expect("spawn sh");
        let events = drain_until_finished(&runner, Duration::from_secs(5));

        // Every Stdout event must come before the terminal Finished.
        let finished_idx = events
            .iter()
            .position(|e| matches!(e, Event::Finished(_)))
            .expect("Finished present");
        for (i, ev) in events.iter().enumerate() {
            if matches!(ev, Event::Stdout(_)) {
                assert!(i < finished_idx, "Stdout after Finished: {events:?}");
            }
        }
    }

    #[test]
    fn stop_kills_long_running_process() {
        // Invoke `sleep` directly. Going via `sh -c "sleep 30"` on
        // Linux forks `sleep` as a child of the shell; killing the
        // shell's PID can leave `sleep` orphaned holding the stdout
        // pipe open, which deadlocks the reader threads. macOS's
        // /bin/sh happens to optimise this into an exec, so the test
        // passed there. Going direct sidesteps the difference.
        let mut cmd = Command::new("sleep");
        cmd.arg("30");
        let mut runner = Runner::spawn(cmd).expect("spawn sleep");
        assert!(runner.is_running());
        runner.stop();

        let events = drain_until_finished(&runner, Duration::from_secs(5));
        assert!(matches!(
            events.last(),
            Some(Event::Finished(_)) | Some(Event::Failed(_))
        ));
        assert!(!runner.is_running());
    }

    #[test]
    fn spawn_missing_binary_errors() {
        let cmd = Command::new("/definitely/does/not/exist/clawasm-test");
        let result = Runner::spawn(cmd);
        let err = match result {
            Ok(_) => panic!("missing binary should error"),
            Err(e) => e,
        };
        assert!(format!("{err:#}").contains("failed to spawn"));
    }
}
