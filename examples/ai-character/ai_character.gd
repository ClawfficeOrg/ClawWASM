extends Control
## CLLawM AI Character — tool-calling demo for the ClawWASM GDExtension.
##
## A blue square character lives in a 2D game world (left pane). The AI
## controls it by emitting JSON tool calls on their own lines. The right
## pane shows a compact chat log plus a ⚙ button that opens a settings
## sub-window for the model path and sampling parameters.
##
## Conversation flow:
##   User sends a command → AI replies with tool calls (JSON lines) + narration
##   → tools execute, results injected as user turn → AI continues (looped up
##   to MAX_TOOL_LOOPS) → streaming stops when AI produces plain text.

# ── Unique-name node references ───────────────────────────────────────────────

@onready var game_pane: Control              = $HBoxContainer/GamePane
@onready var character: Control              = %Character
@onready var speech_panel: PanelContainer    = %SpeechPanel
@onready var speech_label: Label             = %SpeechLabel
@onready var coord_label: Label              = %CoordLabel
@onready var chat_log: RichTextLabel         = %ChatLog
@onready var status_label: Label             = %StatusLabel
@onready var prompt_edit: LineEdit           = %PromptEdit
@onready var send_btn: Button                = %SendBtn
@onready var stop_btn: Button                = %StopBtn
@onready var settings_btn: Button            = %SettingsBtn
@onready var settings_window: Window         = %SettingsWindow
@onready var model_path_edit: LineEdit       = %ModelPathEdit
@onready var browse_btn: Button              = %BrowseBtn
@onready var temp_slider: HSlider            = %TempSlider
@onready var temp_label: Label               = %TempLabel
@onready var top_p_slider: HSlider           = %TopPSlider
@onready var top_p_label: Label              = %TopPLabel
@onready var top_k_spin: SpinBox             = %TopKSpin
@onready var n_predict_spin: SpinBox         = %NPredictSpin
@onready var n_threads_spin: SpinBox         = %NThreadsSpin
@onready var ctx_size_spin: SpinBox          = %CtxSizeSpin
@onready var apply_btn: Button               = %ApplyBtn
@onready var model_file_dialog: FileDialog   = %ModelFileDialog

# ── Constants ─────────────────────────────────────────────────────────────────

## Pixels the character moves per step command.
const STEP: float         = 60.0
## Character sprite size in pixels.
const CHAR_SIZE: Vector2  = Vector2(48.0, 48.0)
## Seconds the speech bubble stays visible.
const SPEECH_SECS: float  = 4.0
## Maximum tool-call loops per user turn (prevents runaway generation).
const MAX_TOOL_LOOPS: int = 8

# ── LLM node ──────────────────────────────────────────────────────────────────

## CLLawM is provided by the clawasm GDExtension. Created at runtime so the
## script parses even when the extension is not yet installed.
var llm: Node  # will be CLLawM at runtime

# ── State ─────────────────────────────────────────────────────────────────────

## Finalised BBCode log — messages that have completed streaming.
var _frozen: String     = ""
## Tokens accumulating during the current streaming response.
var _streaming: String  = ""
## True while inference is running.
var _running: bool      = false
## Multi-turn conversation history: [{role, content}].
var _history: Array     = []
## Countdown timer for the speech bubble; hides at 0.
var _speech_timer: float = 0.0
## Number of tool-call loops executed in the current user turn.
var _tool_loops: int    = 0

# ── Lifecycle ─────────────────────────────────────────────────────────────────

func _ready() -> void:
	# Wait one frame so the layout engine has computed all Control sizes.
	await get_tree().process_frame
	_center_character()

	# Instantiate CLLawM from the extension (deferred class lookup keeps the
	# script parseable even without the cdylib installed).
	if ClassDB.class_exists("CLLawM"):
		llm = ClassDB.instantiate("CLLawM")
		add_child(llm)
		llm.token_generated.connect(_on_token)
		llm.inference_done.connect(_on_done)
		llm.inference_failed.connect(_on_failed)
	else:
		push_error(
			"CLLawM class not found. "
			+ "Build clawasm with --features with-llama and install the addon."
		)

	# Wire UI signals.
	send_btn.pressed.connect(_on_send)
	stop_btn.pressed.connect(_on_stop)
	settings_btn.pressed.connect(_on_settings)
	apply_btn.pressed.connect(_on_apply)
	browse_btn.pressed.connect(_on_browse)
	prompt_edit.text_submitted.connect(func(_t: String) -> void: _on_send())
	temp_slider.value_changed.connect(func(v: float) -> void: temp_label.text = "%.2f" % v)
	top_p_slider.value_changed.connect(func(v: float) -> void: top_p_label.text = "%.2f" % v)
	model_file_dialog.file_selected.connect(func(p: String) -> void: model_path_edit.text = p)
	settings_window.close_requested.connect(func() -> void: settings_window.hide())

	# Initial UI state.
	_set_running(false)
	_append_system(
		"[b]Welcome to CLLawM AI Character![/b]\n"
		+ "  1. Click [b]⚙[/b] (top-right) to open Settings.\n"
		+ "  2. Set your [b].gguf[/b] model path and click [b]Apply & Reload Model[/b].\n"
		+ "  3. Tell the AI what to do — it will move the blue square!\n\n"
		+ "[color=#ffcc44]Tip:[/color] Try \"Go to the top-right corner and say hello!\" "
		+ "or \"Draw a square path.\"\n\n"
		+ "[color=#ffcc44]Note:[/color] clawasm must be compiled with "
		+ "[code]--features with-llama[/code]. "
		+ "Download a model: [code]bash scripts/download-model.sh[/code]"
	)

func _process(delta: float) -> void:
	# Hide the speech bubble after its timer expires.
	if _speech_timer > 0.0:
		_speech_timer -= delta
		if _speech_timer <= 0.0:
			speech_panel.visible = false

	# Keep the position label in sync with the character's location.
	coord_label.text = "Pos: (%d, %d)" % [
		int(character.position.x + CHAR_SIZE.x * 0.5),
		int(character.position.y + CHAR_SIZE.y * 0.5),
	]

# ── Character helpers ─────────────────────────────────────────────────────────

func _center_character() -> void:
	character.position = (game_pane.size - CHAR_SIZE) * 0.5

func _clamp_character() -> void:
	character.position = character.position.clamp(
		Vector2.ZERO,
		game_pane.size - CHAR_SIZE
	)

# ── LLM signal handlers ───────────────────────────────────────────────────────

func _on_token(token: String) -> void:
	_streaming += token
	_render()

func _on_done(_full_text: String, _exit_code: int) -> void:
	# ── Truncate at the earliest turn boundary ──────────────────────────────────
	# Gemma-4 sometimes generates multiple turns before the Rust stop-string
	# filter applies — the model keeps producing `<end_of_turn>\n<start_of_turn>model\n`
	# pairs until max_tokens is hit.  We only want the first reply.
	var raw := _streaming
	_streaming = ""
	var cut_at := -1
	for boundary: String in ["<end_of_turn>", "<start_of_turn>", "<eos>"]:
		var idx := raw.find(boundary)
		if idx >= 0 and (cut_at < 0 or idx < cut_at):
			cut_at = idx
	if cut_at >= 0:
		raw = raw.left(cut_at)

	var reply := _clean(raw)

	var tool_calls := _parse_tool_calls(reply)

	if tool_calls.is_empty():
		# ── Plain text reply — save and display, end the turn. ────────────────
		if not reply.is_empty():
			_history.append({"role": "model", "content": reply})
			_frozen += _bubble("assistant", reply)
		_render()
		_set_running(false)
		_update_status("Done.")
		_tool_loops = 0
	else:
		# ── Tool calls detected — execute them and continue generation. ───────
		# Save the full model turn (includes both JSON and narration).
		_history.append({"role": "model", "content": reply})

		# Show narration text (non-JSON lines) in the chat log.
		var narration := _strip_json_lines(reply)
		if not narration.is_empty():
			_frozen += _bubble("assistant", narration)

		# Execute each tool and collect results.
		var results: Array = []
		for call: Dictionary in tool_calls:
			var name: String  = call.get("name", "")
			var args: Dictionary = call.get("arguments", {})
			var result: Dictionary = _execute_tool(name, args)
			results.append({"tool": name, "result": result})
			_append_tool_event(name, result)

		# Inject tool results as the next user turn.
		_history.append({
			"role": "user",
			"content": "<tool_response>\n" + JSON.stringify(results, "  ") + "\n</tool_response>",
		})

		_render()

		_tool_loops += 1
		if _tool_loops >= MAX_TOOL_LOOPS:
			_append_system("⚠ Tool call limit (%d) reached — stopping." % MAX_TOOL_LOOPS)
			_set_running(false)
			_tool_loops = 0
			return

		# Continue generation with the updated history.
		# Wait one frame so the Rust "running" flag is fully cleared before we
		# call generate_raw again — inference_done fires before the flag resets.
		_update_status("Running tools… loop %d/%d" % [_tool_loops, MAX_TOOL_LOOPS])
		await get_tree().process_frame
		if not _running:
			return  # Stop was pressed during the frame gap
		if not llm.generate_raw(_build_prompt()):
			_on_failed("generate_raw() returned false in tool loop %d" % _tool_loops)

func _on_failed(message: String) -> void:
	_frozen += "[color=#ff6b6b][b]⚠ Inference error:[/b][/color]  %s\n\n" % _esc(message)
	_streaming = ""
	_render()
	_set_running(false)
	_tool_loops = 0
	_update_status("Inference failed — see chat log for details.")

# ── Send / stop ───────────────────────────────────────────────────────────────

func _on_send() -> void:
	var text := prompt_edit.text.strip_edges()
	if text.is_empty() or _running:
		return
	if llm == null:
		_update_status("⚠ CLLawM node unavailable — see Godot error log.")
		return

	# Auto-apply settings if model path is filled but Apply hasn't been clicked.
	var path := model_path_edit.text.strip_edges()
	if not path.is_empty() and llm.model_path().is_empty():
		_on_apply()

	prompt_edit.clear()
	_history.append({"role": "user", "content": text})
	_frozen += _bubble("user", text)
	_streaming = ""
	_tool_loops = 0
	_render()
	_set_running(true)
	_update_status("Generating…")

	# Always use generate_raw so the system prompt (with live world dimensions)
	# is rebuilt on every turn.
	if not llm.generate_raw(_build_prompt()):
		_on_failed(
			"generate_raw() returned false. "
			+ "Is a model loaded? Open ⚙ Settings and click Apply."
		)

func _on_stop() -> void:
	if llm:
		llm.stop()
	_tool_loops = 0
	_update_status("Stop requested…")

# ── Settings window ───────────────────────────────────────────────────────────

func _on_settings() -> void:
	settings_window.popup_centered(Vector2i(490, 560))

func _on_apply() -> void:
	var path := model_path_edit.text.strip_edges()
	if path.is_empty():
		_update_status("⚠ No model path set — open ⚙ Settings first.")
		return
	if llm == null:
		_update_status("⚠ CLLawM node unavailable.")
		return
	llm.set_model(path)
	llm.set_temperature(temp_slider.value)
	llm.set_top_p(top_p_slider.value)
	llm.set_top_k(int(top_k_spin.value))
	llm.set_n_predict(int(n_predict_spin.value))
	llm.set_n_threads(int(n_threads_spin.value))
	llm.set_ctx_size(int(ctx_size_spin.value))
	# Changing the model resets history so the new context is clean.
	_history.clear()
	settings_window.hide()
	_update_status("✔ Model applied. History cleared. Ready.")

func _on_browse() -> void:
	model_file_dialog.popup_centered_ratio(0.7)

# ── Tool execution ─────────────────────────────────────────────────────────────

func _execute_tool(name: String, args: Dictionary) -> Dictionary:
	match name:
		"move_up":
			character.position.y -= STEP
			_clamp_character()
			return {"moved": true, "direction": "up", "position": _pos_dict()}

		"move_down":
			character.position.y += STEP
			_clamp_character()
			return {"moved": true, "direction": "down", "position": _pos_dict()}

		"move_left":
			character.position.x -= STEP
			_clamp_character()
			return {"moved": true, "direction": "left", "position": _pos_dict()}

		"move_right":
			character.position.x += STEP
			_clamp_character()
			return {"moved": true, "direction": "right", "position": _pos_dict()}

		"get_position":
			return {
				"position": _pos_dict(),
				"bounds": {
					"width":  int(game_pane.size.x),
					"height": int(game_pane.size.y),
				},
			}

		"speak":
			var txt: String = str(args.get("text", ""))
			speech_label.text = txt
			speech_panel.visible = true
			_speech_timer = SPEECH_SECS
			return {"spoken": txt}

		_:
			return {"error": "unknown tool: " + name}

func _pos_dict() -> Dictionary:
	## Returns the character's centre position in game-pane local coordinates.
	return {
		"x": int(character.position.x + CHAR_SIZE.x * 0.5),
		"y": int(character.position.y + CHAR_SIZE.y * 0.5),
	}

# ── Prompt building ───────────────────────────────────────────────────────────

func _make_system_prompt() -> String:
	## Builds the system prompt with live world dimensions baked in.
	## Called on every turn so the world size is always current.
	var w := int(game_pane.size.x)
	var h := int(game_pane.size.y)
	return (
		"You are an AI controlling a character in a 2D game world in Godot Engine.\n"
		+ "The world is %dx%d pixels. The character is a blue square (48x48 pixels).\n\n" % [w, h]
		+ "To call a tool, output a JSON object ALONE on its own line — nothing else on that line.\n"
		+ "Use EXACTLY this format (the key is always \"name\"):\n"
		+ "{\"name\": \"move_up\"}\n"
		+ "{\"name\": \"move_right\"}\n"
		+ "{\"name\": \"speak\", \"arguments\": {\"text\": \"Hello!\"}}\n\n"
		+ "Available tools:\n"
		+ "- move_up: Move character up by 60 pixels\n"
		+ "- move_down: Move character down by 60 pixels\n"
		+ "- move_left: Move character left by 60 pixels\n"
		+ "- move_right: Move character right by 60 pixels\n"
		+ "- get_position: Get current x,y position and world bounds\n"
		+ "- speak: Show speech bubble (arguments: {\"text\": \"...\"}\n\n"
		+ "Rules:\n"
		+ "- Each tool call must be on its own line with ONLY the JSON — no extra words.\n"
		+ "- Call multiple tools in sequence if needed (one JSON object per line).\n"
		+ "- After ALL tool calls, write a short plain-text narration of what you did.\n"
		+ "- Do NOT repeat the JSON in your narration. Be playful and expressive!"
	)

func _build_prompt() -> String:
	## Builds the complete Gemma-4 IT multi-turn prompt from _history.
	## The system prompt is re-generated each call so world dimensions stay live.
	var parts: PackedStringArray = PackedStringArray()
	var sys := _make_system_prompt()
	if not sys.is_empty():
		parts.append("<start_of_turn>system\n" + sys + "\n<end_of_turn>\n")
	for turn: Dictionary in _history:
		var role: String    = turn["role"]
		var content: String = turn["content"]
		parts.append("<start_of_turn>" + role + "\n" + content + "\n<end_of_turn>\n")
	# Open the model's reply prefix so generation continues into the response.
	parts.append("<start_of_turn>model\n")
	return "".join(parts)

# ── Tool call parsing ─────────────────────────────────────────────────────────

func _parse_tool_calls(text: String) -> Array:
	## Extracts JSON tool-call objects from lines of the model's response.
	## Accepts both the canonical "name" key and the common "tool_name" mistake
	## (Gemma models sometimes use "tool_name" when shown a generic template).
	var calls: Array = []
	for line: String in text.split("\n"):
		var trimmed := line.strip_edges()
		if trimmed.begins_with("{") and trimmed.ends_with("}"):
			var parsed = JSON.parse_string(trimmed)
			if parsed is Dictionary:
				var tname: String = str(parsed.get("name", parsed.get("tool_name", "")))
				if not tname.is_empty():
					calls.append({
						"name": tname,
						"arguments": parsed.get("arguments", parsed.get("args", {})),
					})
	return calls

func _strip_json_lines(text: String) -> String:
	## Returns the text with all tool-call JSON lines removed.
	## Handles both "name" and "tool_name" key variants.
	var kept: PackedStringArray = PackedStringArray()
	for line: String in text.split("\n"):
		var trimmed := line.strip_edges()
		if trimmed.begins_with("{") and trimmed.ends_with("}"):
			var parsed = JSON.parse_string(trimmed)
			if parsed is Dictionary:
				var tname: String = str(parsed.get("name", parsed.get("tool_name", "")))
				if not tname.is_empty():
					continue  # drop tool-call line
		kept.append(line)
	return "\n".join(kept).strip_edges()

# ── Rendering ─────────────────────────────────────────────────────────────────

func _render() -> void:
	## Rebuilds the chat log: finalised history + any in-progress streaming.
	chat_log.clear()
	var content := _frozen
	if not _streaming.is_empty():
		content += (
			"[color=#a8d8a8][b]Assistant[/b][/color]  "
			+ _esc(_streaming)
			+ "[color=#666666]▌[/color]\n\n"
		)
	chat_log.append_text(content)

func _bubble(role: String, text: String) -> String:
	## Formats a single message as a BBCode chat bubble.
	match role:
		"user":
			return "[color=#87ceeb][b]You[/b][/color]  " + _esc(text) + "\n\n"
		"assistant":
			return "[color=#a8d8a8][b]AI[/b][/color]  " + _esc(text) + "\n\n"
		_:
			return "[color=#aaaaaa][i]" + _esc(text) + "[/i][/color]\n\n"

func _append_system(bbcode: String) -> void:
	## Appends a pre-formatted BBCode info message to the frozen log.
	_frozen += "[color=#999999]" + bbcode + "[/color]\n\n"
	_render()

func _append_tool_event(name: String, result: Dictionary) -> void:
	## Appends a yellow tool-call event line to the frozen log.
	var result_str := JSON.stringify(result)
	_frozen += (
		"[color=#d4a843][b]🔧 %s[/b][/color]  → [color=#cccccc]%s[/color]\n\n"
		% [_esc(name), _esc(result_str)]
	)
	_render()

func _esc(text: String) -> String:
	## Escapes BBCode opening brackets so text renders as plain text.
	return text.replace("[", "[lb]")

func _clean(text: String) -> String:
	## Strips stop-string leakage that sometimes survives the Rust-side filter.
	## Also removes bare role-label lines ("model", "user", "system") that remain
	## after <start_of_turn> tags are stripped.
	var result := text
	for stop: String in [
		"<end_of_turn>", "</start_of_turn>", "<start_of_turn>",
		"<eos>", "<|endoftext|>", "[/INST]",
	]:
		result = result.replace(stop, "")
	# Remove bare role labels left over after tag stripping.
	var kept: PackedStringArray = PackedStringArray()
	for line: String in result.split("\n"):
		if line.strip_edges() in ["model", "user", "system"]:
			continue
		kept.append(line)
	return "\n".join(kept).strip_edges()

# ── UI helpers ────────────────────────────────────────────────────────────────

func _set_running(running: bool) -> void:
	_running = running
	send_btn.disabled  = running
	stop_btn.disabled  = not running
	prompt_edit.editable = not running

func _update_status(text: String) -> void:
	status_label.text = text
