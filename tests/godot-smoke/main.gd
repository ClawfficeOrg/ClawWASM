extends Node
# Reference main.gd for the manual ClawEngine smoke-test scene.
#
# See `tests/godot-smoke/README.md` for the surrounding project layout.
# This script is checked in as a copy-pastable reference rather than
# part of a runnable Godot project (no CI runner yet).

@onready var engine := ClawEngine.new()

func _ready() -> void:
	add_child(engine)
	engine.register_module("res://modules/hello-wasm.wasm")
	engine.stdout_line.connect(_on_stdout)
	engine.stderr_line.connect(_on_stderr)
	engine.finished.connect(_on_finished)
	engine.failed.connect(_on_failed)
	if not engine.start(PackedStringArray()):
		push_error("ClawEngine.start() returned false; see Godot log for cause.")

func _on_stdout(line: String) -> void:
	print("[wasm] ", line)

func _on_stderr(line: String) -> void:
	push_warning("[wasm err] " + line)

func _on_finished(code: int) -> void:
	print("[wasm] exit ", code)

func _on_failed(msg: String) -> void:
	push_error("[wasm fail] " + msg)
