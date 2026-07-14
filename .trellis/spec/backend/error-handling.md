# Error Handling

> Error types, propagation, and conversion patterns in `src-tauri/src/error.rs`.

---

## Core Types

### AppError

The canonical error type for all Tauri commands and IPC boundaries:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,           // machine-readable, e.g. "session.not_found"
    pub message: String,        // human-readable summary
    pub detail: Option<String>,  // optional technical detail
    pub operation: Option<String>, // what was being attempted
    pub retryable: bool,
    pub severity: ErrorSeverity, // Error | Warning | Info
}
```

Reference: `src-tauri/src/error.rs:16-27`

### Builder Pattern

`AppError` uses a builder pattern — NOT a struct literal with all fields:

```rust
AppError::new("session.not_found", "Session not found")
    .detail(format!("session_id: {}", id))
    .operation("load_session")
    .retryable(false)
    .severity(ErrorSeverity::Error)
```

Reference: `src-tauri/src/error.rs:29-59`

### AppResult

Convenience type alias for command return types:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

Reference: `src-tauri/src/error.rs:124`

**Always use `AppResult<T>` for Tauri command return types.**

---

## Error Conversion

### From<anyhow::Error>

Internal code uses `anyhow::Error` for flexibility. At command boundaries, convert to `AppError`:

```rust
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        let message = format!("{}", err);
        let detail = format!("{:#}", err);
        Self::new("internal.unknown", message).detail(detail)
    }
}
```

Reference: `src-tauri/src/error.rs:98-104`

### IntoAppError Trait

For domain-specific error codes (preferred over the generic `From<anyhow>`):

```rust
pub trait IntoAppError {
    fn app_err(self, code: &str, operation: &str) -> AppError;
}
```

Usage at call sites:
```rust
some_fallible_operation()
    .map_err(|e| e.app_err("unity.compile_failed", "compile_csharp"))?;
```

Reference: `src-tauri/src/error.rs:109-121`

### Migration Bridge (String → AppError)

Legacy commands return `Result<T, String>`. The migration bridge allows incremental adoption:

```rust
impl From<String> for AppError { ... }
impl From<&str> for AppError { ... }
```

Reference: `src-tauri/src/error.rs:84-94`

**For new code:** use `AppError`, not `String`. The bridge is for migration only.

---

## Background Error Emission

Errors that occur outside a request/response cycle (background tasks, watchers) are emitted as Tauri events:

```rust
AppError::emit_background(&app_handle, &error);
```

Reference: `src-tauri/src/error.rs:62-69`

This sends an `"app-error"` event to the frontend. The frontend listens for these to show toast notifications.

---

## Error Code Convention

Error codes use dot-separated `domain.specific_error` format:

| Pattern | Example |
|---------|---------|
| `session.*` | `session.not_found`, `session.duplicate_id` |
| `unity.*` | `unity.compile_failed`, `unity.bridge_disconnected` |
| `knowledge.*` | `knowledge.index_locked` |
| `auth.*` | `auth.token_expired` |
| `git.*` | `git.merge_conflict` |
| `internal.*` | `internal.unknown` (fallback from anyhow) |

---

## Anti-Patterns

- **Don't `unwrap()` or `expect()` in command handlers** — propagate via `?` or `AppError`.
- **Don't use `anyhow::Error` as a command return type** — always convert to `AppError` at the boundary.
- **Don't lose detail in conversion** — always include the anyhow `{:#}` format (shows chain) as `detail`.
- **Don't create new error types without a new domain** — check if an existing error code covers it first.
