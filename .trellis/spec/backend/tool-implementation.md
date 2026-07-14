# Tool Implementation

> Agent tool definitions, registration, and execution in `src-tauri/src/tool/`.

---

## Core Types

Defined in `tool/mod.rs`:

```rust
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema for LLM tool-use
    pub mutates_workspace: bool,        // true → round must be checkpointed for undo
    pub execute: ToolExecuteFn,
}

pub type ToolExecuteFn = Arc<
    dyn Fn(serde_json::Value, ToolExecutionContext)
        -> Pin<Box<dyn Future<Output = ToolResult> + Send>>
        + Send + Sync,
>;

pub struct ToolResult {
    pub output: String,
    pub is_error: bool,
}

pub struct ToolExecutionContext {
    pub app_handle: Option<AppHandle>,
    pub working_dir: Option<String>,
    pub unity_connected: Option<bool>,
    pub runtime_state: Option<Arc<ToolRuntimeState>>,
}
```

Reference: `src-tauri/src/tool/mod.rs:13-29`, `82-100`

---

## Tool Registration

All tools are registered in `tool/builtins/mod.rs::register_all()`. Each tool is a function that returns `ToolDef`:

```rust
pub fn register_all(registry: &mut ToolRegistry) {
    // Simple tools — always available
    registry.register_builtin(filesystem::read());
    registry.register_builtin(filesystem::write());
    registry.register_builtin(shell::bash());
    registry.register_builtin(search::grep());

    // Skill-gated tools — only loaded when the skill is active
    registry.register_builtin_with_load_mode(plugin::plugin_list(), ToolLoadMode::Skill);
    registry.register_builtin_with_load_mode(view::view_create(), ToolLoadMode::Skill);
    // ...
}
```

### Load Modes

| Mode | When tool is available |
|------|----------------------|
| `ToolLoadMode::Always` (default via `register_builtin`) | Always in the LLM's tool list |
| `ToolLoadMode::Skill` | Only when the corresponding skill is loaded |

Use `Skill` mode for tools that require plugin/view system initialization — they'd fail or confuse the LLM otherwise.

---

## Tool Implementation Pattern

Each tool module exports a function that builds a `ToolDef`. The standard pattern:

```rust
// tool/builtins/misc.rs
use crate::prompt::parse_tool_prompt;

pub fn web_fetch() -> ToolDef {
    let prompt = parse_tool_prompt(crate::prompt::tools::WEB_FETCH);
    ToolDef {
        name: "web_fetch".to_string(),
        description: prompt.description,
        parameters: prompt.parameters,
        mutates_workspace: false,
        execute: Arc::new(|args, ctx| {
            Box::pin(async move {
                // 1. Parse args
                // 2. Execute logic with ctx (working_dir, unity_connected, etc.)
                // 3. Return ToolResult { output, is_error }
            })
        }),
    }
}
```

Reference: `src-tauri/src/tool/builtins/mod.rs:108-127` (exit_plan_mode_tool), `129-167` (config_query_tool).

### Prompt Integration

Tool descriptions and parameter schemas come from `crate::prompt::parse_tool_prompt(crate::prompt::tools::<TOOL_NAME>)`. This keeps the LLM-facing text in the `prompt/` directory rather than hardcoded in Rust.

### make_exec Helper

For convenience, `tool/builtins/mod.rs:189-200` provides a `make_exec` helper that wraps an `async fn(Value, ToolExecutionContext) -> ToolResult` into a `ToolExecuteFn`. Use this for cleaner tool code unless the tool needs custom `Arc` handling.

---

## Unity-Specific Tool Considerations

Tools that read Unity YAML assets must handle redirect logic. The `ToolExecutionContext::should_redirect_unity_asset_read()` method in `tool/mod.rs:36-52` checks whether:
1. Unity is connected
2. The file path looks like a Unity YAML asset (`.unity`, `.prefab`, `.asset`, `.mat`, `.anim`, `.controller`)

Reference: `src-tauri/src/tool/mod.rs:68-80` for `is_unity_yaml_candidate_path()`.

Tools that modify Unity state should set `mutates_workspace: true` so the undo system checkpoints the round.

## Scenario: Unity Test Framework Agent Tools

### 1. Scope / Trigger

- Trigger: adding or changing Unity Test Framework discovery/run tools, their Unity bridge pipe messages, or the latest test snapshot IPC command.
- This is a cross-layer contract: agent tool schema -> Rust bridge -> Unity editor pipe -> `Locus/test-results/latest.json` -> frontend snapshot reader.

### 2. Signatures

- Agent tools: `unity_test_find(args)` and `unity_test_run(args)`.
- Unity pipe messages: `find_tests`, `run_tests`, `cancel_tests`, `test_run_progress`.
- Tauri command: `unity_test_latest_snapshot(working_dir: String) -> AppResult<Option<UnityTestSnapshot>>`.

### 3. Contracts

- Request fields use camelCase over the pipe: `testMode`, `assemblyName`, `fixtureName`, `testName`, `search`, and optional explicit `tests[]`.
- Tool schemas expose snake_case names for model ergonomics: `test_mode`, `assembly_name`, `fixture_name`, `test_name`, `search`, `tests`.
- Mixed `all` runs execute `editmode` before `playmode` under one logical `runId`.
- `search` must be resolved through discovery into explicit test targets before execution; do not pass fuzzy search directly to UTF execution if that can broaden the run.
- Terminal snapshots are written to `Locus/test-results/latest.json`; `unity_test_run` is `mutates_workspace: true`.
- The Unity editor plugin must compile without hard asmdef references to `UnityEditor.TestRunner` or `UnityEngine.TestRunner`; detect missing UTF at runtime and return `test_framework_missing`.

### 4. Validation & Error Matrix

- No selected Unity project -> tool error.
- Unity disconnected -> `unity_disconnected`.
- Another run active -> `busy`.
- Test Framework missing -> `test_framework_missing`.
- Compile/preparation failure -> `compile_failed` and terminal snapshot with `prepare_error`.
- User cancellation -> cancel Unity run, write terminal snapshot, exit PlayMode, and return interrupted.

### 5. Good/Base/Bad Cases

- Good: `unity_test_find({ test_mode: "all" })` discovers EditMode and PlayMode tests without changing files.
- Base: `unity_test_run({ test_mode: "editmode", fixture_name: "FooTests" })` prepares code, runs only matching EditMode tests, streams progress, writes latest snapshot.
- Bad: `unity_test_run({ test_mode: "all", search: "Foo" })` passes `search` directly to UTF execution or silently runs every test when no search match exists.

### 6. Tests Required

- Source/registration test: prompts include both tools; builtins register both tools; `unity_test_run` is intercepted in the agent loop for progress/cancel.
- Frontend wiring test: tool block override renders `unity_test_run`; service invokes `unity_test_latest_snapshot`; shared TS types include `UnityTestSnapshot`.
- Backend unit tests: mode ordering is EditMode before PlayMode; phase summaries aggregate total/passed/failed/skipped/duration.
- Manual Unity validation: find tests, run one EditMode test, run one PlayMode test, cancel a long run, force a failed assertion, and verify PlayMode cleanup plus latest snapshot write.

### 7. Wrong vs Correct

#### Wrong

```rust
registry.register_builtin(unity::unity_test_run()); // normal execution path only
```

This loses cancellation and `StreamEvent::ToolCallProgress` because the run needs the live agent session path.

#### Correct

```rust
if tc.name == "unity_test_run" {
    self.execute_unity_test_run(app_handle, &tc.id, args, run_id).await
}
```

Keep discovery as a normal builtin tool, but route test execution through the agent loop so progress events and cancellation use the initiating tool call.

---

## Adding a New Tool

1. Create the tool function in the appropriate `tool/builtins/<domain>.rs` file
2. Export it from the module
3. Register it in `register_all()` with the correct load mode
4. Add the prompt definition in `prompt/`
5. Wire the prompt constant in `prompt.rs`

---

## Anti-Patterns

- **Don't hardcode tool descriptions in Rust** — use `parse_tool_prompt()` and put text in `prompt/`.
- **Don't block the async runtime** — tool execution is async. Use `tokio::task::spawn_blocking` for CPU-heavy or blocking I/O.
- **Don't forget `mutates_workspace`** — if a tool changes files, set it to `true` or undo won't work.
- **Don't return string errors** — use `ToolResult { output: "...", is_error: true }` for tool-level errors. Reserve `AppError` for IPC command failures.
