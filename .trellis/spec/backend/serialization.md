# Serialization

> serde conventions for frontend-facing types in the Rust backend.

---

## Camel Case for Frontend

**All types that cross the IPC boundary** must use camelCase to match the TypeScript frontend:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SomeType {
    pub some_field: String,
    pub another_field: Option<i32>,
}
```

This applies to:
- Tauri command request/response types
- StreamEvent and all its variant payloads
- Types in `session/models.rs`
- `AppError` (serialized to frontend)
- Config types exposed via commands

**Internal types** (not serialized for frontend) don't need `rename_all`.

---

## Tagged Enums for Event Discrimination

The `StreamEvent` enum uses serde's internal tagging:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StreamEvent {
    RunStart { session_id: String },
    TextDelta { session_id: String, text: String, ... },
    Done { session_id: String, ... },
    // ...
}
```

This produces JSON like `{"type": "textDelta", "sessionId": "...", "text": "..."}`.

Reference: `src-tauri/src/commands/mod.rs:86-319`

For other enums that need discrimination, use the same pattern:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ToolConfirmDisplay {
    Basic(BasicToolConfirmDisplay),
    Knowledge(KnowledgeToolConfirmPreview),
    // ...
}
```

Reference: `src-tauri/src/commands/mod.rs:423-430`

---

## Optional Fields

Always use `#[serde(default, skip_serializing_if = "...")]` for optional fields:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub detail: Option<String>,

#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub render_parts: Option<Vec<AssistantRenderPart>>,

#[serde(default)]
pub multiline: bool,  // for non-Option types with a sensible Default
```

This keeps event payloads compact and avoids breaking old frontends when new fields are added.

---

## serde_json::Value for Tool Parameters

Tool parameter schemas are stored as `serde_json::Value` because they're JSON Schema objects passed directly to LLM providers:

```rust
pub struct ToolDef {
    pub parameters: serde_json::Value,  // raw JSON Schema
    // ...
}
```

The `execute` closure also receives `serde_json::Value` as the parsed arguments. Parse it with `serde_json::from_value::<MyArgs>(args)` inside the tool.

---

## Snail Case for Internal Enums

Enum variants exposed to frontend as string values use `snake_case`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeToolConfirmOperation {
    Create,
    Edit,
    Move,
    Delete,
}
```

Reference: `src-tauri/src/commands/mod.rs:370-376`

---

## Type Location

- **Command request/response types:** In the command handler file (e.g., `commands/session.rs`)
- **Session model types:** In `session/models.rs`
- **Event types:** In `commands/mod.rs` (StreamEvent + related enums)
- **Config types:** In `config.rs`
- **Error types:** In `error.rs`

Don't sprinkle serializable types across implementation files — keep them near their IPC boundary or in a dedicated models file.

---

## Anti-Patterns

- **Don't use snake_case for frontend-facing JSON** — TypeScript convention is camelCase.
- **Don't serialize internal implementation details** — use `#[serde(skip)]` for fields like `Arc<Mutex<...>>`.
- **Don't add required fields to existing event variants** — use `Option` + `#[serde(default)]` for backward compatibility.
