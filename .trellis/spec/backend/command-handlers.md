# Command Handlers

> Tauri IPC command patterns in `src-tauri/src/commands/`.

---

## Command Module Organization

23 command submodules in `commands/`:

```
commands/
├── mod.rs          ← StreamEvent enum, event emission, re-exports
├── session.rs      ← Chat session lifecycle
├── asset.rs        ← Asset operations
├── git.rs          ← Version control
├── knowledge.rs    ← Knowledge system
├── plugin.rs       ← Plugin management
├── view.rs         ← View system
├── unity_embed.rs  ← Unity window embedding
├── csharp_lsp.rs   ← C# LSP commands
├── diff.rs         ← Diff operations
├── undo.rs         ← Undo operations
├── auth.rs         ← Authentication
├── plan.rs         ← Plan mode
├── skill.rs        ← Skill management
├── system.rs       ← System-level commands
├── storage.rs      ← Storage operations
├── workspace.rs    ← Workspace management
├── update.rs       ← App update
├── fonts.rs        ← Font management
├── log.rs          ← Logging
├── agent_graph.rs  ← Agent graph operations
├── ref_graph.rs    ← Reference graph
└── unity_serialized_property.rs ← Unity property access
```

**When adding a new IPC surface:** Create a new `<domain>.rs` file, declare it in `commands/mod.rs` as `mod <domain>;`, add `pub use <domain>::*;`, and register the Tauri command in `lib.rs`'s `.invoke_handler()` chain.

---

## Event System

Events are emitted from backend to frontend via Tauri's event system. The central type is `StreamEvent` in `commands/mod.rs:88-319`.

### StreamEvent Pattern

All stream events are tagged with `#[serde(tag = "type", rename_all = "camelCase")]`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StreamEvent {
    RunStart { session_id: String },
    TextDelta { session_id: String, text: String, order: Option<u32>, ... },
    ToolCallStart { session_id: String, tool_call_id: String, tool_name: String, ... },
    ToolCallDone { session_id: String, tool_call_id: String, output: String, outcome: ToolCallOutcome, ... },
    Done { session_id: String, message_id: String, full_text: String, ... },
    Error { session_id: String, error: AppError },
    // ... 20+ variants
}
```

### Event Envelope

Every event is wrapped in `StreamEventEnvelope` with a `run_id` for filtering stale events:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamEventEnvelope {
    pub run_id: String,
    #[serde(flatten)]
    pub event: StreamEvent,
}
```

Reference: `src-tauri/src/commands/mod.rs:322-328`

### Emitting Events

Use `app_handle.emit(event_name, payload)`. The `AppHandle` comes from either:
- The Tauri command's `app_handle: AppHandle` parameter
- `ToolExecutionContext.app_handle`

```rust
// Example pattern from commands/mod.rs:47-64
pub fn emit_session_content_changed(
    app_handle: &AppHandle,
    working_dir: &str,
    session_id: &str,
    source: &str,
) {
    let event = SessionContentChangedEvent { ... };
    if let Err(error) = app_handle.emit(SESSION_CONTENT_CHANGED_EVENT, event) {
        eprintln!("[Locus] failed to emit ... : {}", error);
    }
}
```

**Rule:** Event emission helpers get their own function. Don't inline `app_handle.emit()` at every call site.

---

## Tauri Command Signature Pattern

Tauri commands use `#[tauri::command]` and take `AppHandle` + `State<T>` for dependency injection:

```rust
#[tauri::command]
async fn my_command(
    app_handle: AppHandle,
    state: State<'_, MyState>,
    // ... parameters
) -> Result<MyResponse, AppError> {
    // ...
}
```

Return types are `Result<T, AppError>` (see [[error-handling]]). String errors are legacy — use `AppError` for new commands.

---

## Request/Response Convention

- All request/response types use `#[serde(rename_all = "camelCase")]`
- Response types are defined in the same file as the command handler, not in a separate types file
- Large event payloads get their own struct (e.g., `SessionContentChangedEvent`, `KnowledgeToolConfirmPreview`)
- Optional fields use `#[serde(default, skip_serializing_if = "Option::is_none")]`

---

## Anti-Patterns

- **Don't catch-all string errors:** Old commands return `Result<T, String>`. Use `AppError` for new code (see `error.rs:80-94` for the migration bridge).
- **Don't emit events without `run_id` filtering:** Stale events from cancelled runs cause UI bugs. Always wrap in `StreamEventEnvelope`.
- **Don't add more variants to `commands/mod.rs` without considering a new submodule file.**

## Scenario: Project-Scoped Long-Running Command Events

### 1. Scope / Trigger

- Trigger: one operation can be started by multiple callers (for example an agent and a dashboard), while a project-level UI must observe the same lifecycle independently of caller-specific events.
- Keep caller-specific events intact. Add a separate project-scoped event contract instead of manufacturing a fake session or tool-call identity.

### 2. Signatures

```rust
pub fn emit_operation_progress(
    app_handle: &AppHandle,
    working_dir: &str,
    source: &str,
    progress: OperationProgress,
);

pub fn emit_operation_snapshot_changed(
    app_handle: &AppHandle,
    working_dir: &str,
    source: &str,
    snapshot: &OperationSnapshot,
);
```

Commands that start or cancel the operation take `State<'_, OperationState>` and return `AppResult<T>`. Register the state with `app.manage(...)` and every command in `lib.rs`.

### 3. Contracts

- Progress payload: `workingDir`, `source`, and a typed progress object containing `active` plus the operation's `runId`.
- Terminal payload: `workingDir`, `source`, `runId`, and `terminalStatus`.
- The frontend must compare normalized `workingDir` before mutating state; a run identifier alone is not a project boundary.
- Emit preparation progress before the first lower-layer progress callback when setup can take noticeable time.
- If cancellation returns completed partial work, carry it internally into the terminal snapshot. Internal partial fields may use `#[serde(skip)]` when the public error shape must remain stable.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Empty workspace | Return a stable `<domain>.workspace_required` `AppError`; do not start work. |
| Active-run lock already held (`busy`) | Return `<domain>.busy`; do not read or broadcast an older terminal snapshot. If preparation progress was already emitted, emit `active: false`. |
| Failure after acquiring the active-run lock | Persist/read the new terminal snapshot, then broadcast its run ID and status. |
| Cancellation with partial lower-layer result | Persist `cancelled` plus the partial phase/results; still run normal cleanup. |
| Cancellation response timeout | Persist a valid empty `cancelled` snapshot and continue cleanup. |
| Event emission failure | Log it; do not replace the command result or skip cleanup. |

### 5. Good/Base/Bad Cases

- Good: an agent run emits its normal tool-call progress and the project event; an open dashboard filters by `workingDir` and reloads the terminal snapshot.
- Base: the dashboard mounts mid-run, queries active progress and latest snapshot, then subscribes to later lifecycle changes.
- Bad: a second caller receives `busy`, reads `latest.json` from a previous run, and broadcasts that old snapshot as if the rejected call completed.

### 6. Tests Required

- Serialize progress and terminal payloads; assert camelCase field names and workspace/run identity.
- Exercise `busy`; assert no terminal event is produced from stale persisted state.
- Exercise cancellation with partial results; assert terminal status is `cancelled`, completed results survive, and the public error JSON shape is unchanged.
- Exercise cancellation timeout; assert cleanup and a valid cancelled snapshot still occur.
- Frontend projection test: an event for another normalized `workingDir` must not update the current view.

### 7. Wrong vs Correct

#### Wrong

```rust
let result = run_operation().await;
if let Ok(Some(snapshot)) = read_latest_snapshot() {
    emit_snapshot_changed(&snapshot); // broadcasts stale data when result is busy
}
```

#### Correct

```rust
let result = run_operation().await;
match &result {
    Ok(snapshot) => emit_snapshot_changed(snapshot),
    Err(error) if error.code != "busy" => {
        if let Ok(Some(snapshot)) = read_latest_snapshot() {
            emit_snapshot_changed(&snapshot);
        }
    }
    Err(_) => {}
}
```
