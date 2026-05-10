extends Control
## CLLawM Chat — streaming LLM chat interface for the ClawWASM GDExtension.
##
## Drop this scene into Godot 4.6+, ensure the clawasm addon is installed
## (see README.md), and press Play. Set a model path in the Settings panel,
## click "Apply & Reload Model", then type a message.
##
## The CLLawM node is instantiated at runtime so this script does not have a
## hard dependency on the class name at parse time (useful during development
## without the extension loaded).

# ── Unique-name node references ───────────────────────────────────────────────

@onready var model_path_edit: LineEdit       = %ModelPathEdit
@onready var browse_btn: Button              = %BrowseBtn
@onready var system_prompt_edit: TextEdit    = %SystemPromptEdit
@onready var temp_slider: HSlider            = %TempSlider
@onready var temp_label: Label               = %TempLabel
@onready var top_p_slider: HSlider           = %TopPSlider
@onready var top_p_label: Label              = %TopPLabel
@onready var top_k_spin: SpinBox             = %TopKSpin
@onready var n_predict_spin: SpinBox         = %NPredictSpin
@onready var n_threads_spin: SpinBox         = %NThreadsSpin
@onready var ctx_size_spin: SpinBox          = %CtxSizeSpin
@onready var apply_btn: Button               = %ApplyBtn
@onready var chat_log: RichTextLabel         = %ChatLog
@onready var status_label: Label             = %StatusLabel
@onready var prompt_edit: LineEdit           = %PromptEdit
@onready var send_btn: Button                = %SendBtn
@onready var stop_btn: Button                = %StopBtn
@onready var model_file_dialog: FileDialog   = %ModelFileDialog

# ── LLM node ──────────────────────────────────────────────────────────────────

## CLLawM is provided by the clawasm GDExtension. It is created at runtime
## so the script parses cleanly even when the extension is not loaded yet.
var llm: Node  # typed as Node; will be a CLLawM at runtime

# ── Chat state ────────────────────────────────────────────────────────────────

## All finalised messages in BBCode, appended as conversations progress.
var _frozen: String = ""

## Tokens accumulating for the currently-streaming response.
var _streaming: String = ""

## True while the LLM is generating a response.
var _running: bool = false

## Conversation history: Array of {role: String, content: String} dicts.
## Used to build multi-turn prompts so the model remembers the conversation.
var _history: Array = []

## Current system prompt (kept in sync with the settings panel).
var _system_prompt: String = "You are a helpful assistant."

# ── Lifecycle ─────────────────────────────────────────────────────────────────

func _ready() -> void:
	# Instantiate CLLawM from the extension and add it as a child so it
	# receives _process() calls every frame for signal delivery.
	if ClassDB.class_exists("CLLawM"):
		llm = ClassDB.instantiate("CLLawM")
		add_child(llm)
		llm.token_generated.connect(_on_token)
		llm.inference_done.connect(_on_done)
		llm.inference_failed.connect(_on_failed)
	else:
		push_error("CLLawM class not found. Build clawasm with --features with-llama and install the addon.")

	# Wire UI signals.
	send_btn.pressed.connect(_on_send)
	stop_btn.pressed.connect(_on_stop)
	apply_btn.pressed.connect(_on_apply)
	browse_btn.pressed.connect(_on_browse)
	prompt_edit.text_submitted.connect(func(_t: String) -> void: _on_send())
	temp_slider.value_changed.connect(func(v: float) -> void: temp_label.text = "%.2f" % v)
	top_p_slider.value_changed.connect(func(v: float) -> void: top_p_label.text = "%.2f" % v)
	model_file_dialog.file_selected.connect(func(p: String) -> void: model_path_edit.text = p)

	# Initial UI state.
	_set_running(false)
	_append_system(
		"[b]Welcome to CLLawM Chat![/b]\n"
		+ "  1. Set the [b].gguf[/b] model path in the Settings panel.\n"
		+ "  2. Click [b]Apply & Reload Model[/b].\n"
		+ "  3. Type a message and press [b]Enter[/b] or [b]Send[/b].\n\n"
		+ "[color=#ffcc44]Note:[/color] clawasm must be compiled with "
		+ "[code]--features with-llama[/code] for inference to work.\n"
		+ "Download a GGUF model with [code]bash scripts/download-model.sh[/code]."
	)

# ── LLM signal handlers ───────────────────────────────────────────────────────

func _on_token(token: String) -> void:
	_streaming += token
	_render()

func _on_done(_full_text: String, _exit_code: int) -> void:
	# Strip any stop-string leakage just in case (belt and suspenders over the
	# Rust-side stop string detection).
	var reply := _streaming
	for stop in ["<end_of_turn>", "</start_of_turn>", "<start_of_turn>", "<eos>", "<|endoftext|>", "[/INST]"]:
		reply = reply.replace(stop, "")
	reply = reply.strip_edges()
	# Save to history so the next turn includes this exchange.
	if not reply.is_empty():
		_history.append({"role": "model", "content": reply})
	# Finalise the streaming bubble into the frozen log.
	if not _streaming.is_empty():
		_frozen += _bubble("assistant", reply if not reply.is_empty() else _streaming)
		_streaming = ""
	_render()
	_set_running(false)
	_update_status("Done.")

func _on_failed(message: String) -> void:
	_frozen += "[color=#ff6b6b][b]⚠ Inference error:[/b][/color]  %s\n\n" % _esc(message)
	_streaming = ""
	_render()
	_set_running(false)
	_update_status("Inference failed — see chat log for details.")

# ── Send / stop ───────────────────────────────────────────────────────────────

func _on_send() -> void:
	var text := prompt_edit.text.strip_edges()
	if text.is_empty() or _running:
		return
	if llm == null:
		_update_status("⚠ CLLawM node unavailable — see Godot error log.")
		return

	# Auto-apply settings if a model path is set but Apply wasn't clicked.
	var path := model_path_edit.text.strip_edges()
	if not path.is_empty() and llm.model_path().is_empty():
		_on_apply()

	prompt_edit.clear()
	# Save user turn to history.
	_history.append({"role": "user", "content": text})
	_frozen += _bubble("user", text)
	_streaming = ""
	_render()
	_set_running(true)
	_update_status("Generating…")

	var ok: bool
	if _history.size() > 1:
		# Multi-turn: build the full Gemma-4 IT conversation and bypass the
		# single-turn template in the Rust node.
		var full_prompt := _build_gemma_prompt(_system_prompt, _history)
		ok = llm.generate_raw(full_prompt)
	else:
		# First turn: let the Rust node apply the template (or its fallback).
		ok = llm.generate(text)

	if not ok:
		_on_failed(
			"generate() returned false. "
			+ "Is a model loaded? Click \"Apply & Reload Model\" first."
		)

func _on_stop() -> void:
	if llm:
		llm.stop()
	_update_status("Stop requested…")

# ── Settings ──────────────────────────────────────────────────────────────────

func _on_apply() -> void:
	var path := model_path_edit.text.strip_edges()
	if path.is_empty():
		_update_status("⚠ No model path set.")
		return
	if llm == null:
		_update_status("⚠ CLLawM node unavailable.")
		return
	_system_prompt = system_prompt_edit.text
	llm.set_model(path)
	llm.set_system_prompt(_system_prompt)
	llm.set_temperature(temp_slider.value)
	llm.set_top_p(top_p_slider.value)
	llm.set_top_k(int(top_k_spin.value))
	llm.set_n_predict(int(n_predict_spin.value))
	llm.set_n_threads(int(n_threads_spin.value))
	llm.set_ctx_size(int(ctx_size_spin.value))
	# Changing the model resets the conversation context.
	_history.clear()
	_update_status("✔ Settings applied. Conversation history cleared. Model loads on first generate().")

func _on_browse() -> void:
	model_file_dialog.popup_centered_ratio(0.7)

# ── Rendering ─────────────────────────────────────────────────────────────────

# ── Prompt building ──────────────────────────────────────────────────────────────

## Build a complete Gemma-4 IT multi-turn prompt from the conversation history.
## BOS is prepended by the tokeniser (AddBos::Always in generate_raw).
func _build_gemma_prompt(sys: String, history: Array) -> String:
	var parts: PackedStringArray = PackedStringArray()
	if not sys.is_empty():
		parts.append("<start_of_turn>system\n" + sys + "\n<end_of_turn>\n")
	for turn in history:
		var role: String = turn["role"]
		var content: String = turn["content"]
		parts.append("<start_of_turn>" + role + "\n" + content + "\n<end_of_turn>\n")
	# Open assistant prefix so the model continues into its reply.
	parts.append("<start_of_turn>model\n")
	return "".join(parts)

# ── Rendering ────────────────────────────────────────────────────────────────

## Rebuild the chat log from frozen history plus any current streaming bubble.
## Called on every token during generation and on state transitions.
func _render() -> void:
	chat_log.clear()
	var content := _frozen
	if not _streaming.is_empty():
		content += (
			"[color=#a8d8a8][b]Assistant[/b][/color]  "
			+ _esc(_streaming)
			+ "[color=#666666]▌[/color]\n\n"
		)
	chat_log.append_text(content)

## Format a single user or assistant message as a BBCode bubble.
func _bubble(role: String, text: String) -> String:
	match role:
		"user":
			return "[color=#87ceeb][b]You[/b][/color]  " + _esc(text) + "\n\n"
		"assistant":
			return "[color=#a8d8a8][b]Assistant[/b][/color]  " + _esc(text) + "\n\n"
		_:
			return "[color=#aaaaaa][i]" + _esc(text) + "[/i][/color]\n\n"

## Append a pre-formatted BBCode system/info message to the frozen log.
func _append_system(bbcode: String) -> void:
	_frozen += "[color=#999999]" + bbcode + "[/color]\n\n"
	_render()

## Escape characters that would be misinterpreted as BBCode tags.
func _esc(text: String) -> String:
	return text.replace("[", "[lb]")

# ── UI helpers ────────────────────────────────────────────────────────────────

func _set_running(running: bool) -> void:
	_running = running
	send_btn.disabled = running
	stop_btn.disabled = not running
	prompt_edit.editable = not running

func _update_status(text: String) -> void:
	status_label.text = text
