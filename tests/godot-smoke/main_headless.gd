extends Node
# Headless-CI variant of the ClawEngine smoke script.
# Identical to main.gd but calls get_tree().quit() so Godot exits cleanly
# when run with --headless (no editor loop to stop it otherwise).
#
# Used by .github/workflows/ci.yml godot-smoke job.
# Manual interactive use: copy main.gd instead (no quit).

@onready var engine := ClawEngine.new()

func _ready() -> void:
	add_child(engine)
	engine.register_module("res://modules/hello-wasm.wasm")
	engine.stdout_line.connect(_on_stdout)
	engine.stderr_line.connect(_on_stderr)
	engine.finished.connect(_on_finished)
	engine.failed.connect(_on_failed)
	if not engine.start(PackedStringArray()):
		push_error("ClawEngine.start() returned false; see Godot log.")
		get_tree().quit(1)

func _on_stdout(line: String) -> void:
	print("[wasm] ", line)

func _on_stderr(line: String) -> void:
	push_warning("[wasm err] " + line)

func _on_finished(code: int) -> void:
	print("[wasm] exit ", code)
	get_tree().quit(code)

func _on_failed(msg: String) -> void:
	push_error("[wasm fail] " + msg)
	get_tree().quit(1)
