# Module Structure

> How Rust modules are organized in `src-tauri/src/`.

---

## Single-Crate Layout

This is **one crate** with two targets defined in `Cargo.toml`:

```toml
[lib]
name = "locus_lib"
crate-type = ["rlib"]
```

`main.rs` is 6 lines: `fn main() { locus_lib::run(); }`

All module declarations live in `lib.rs`. There is no workspace or multi-crate split — everything from asset scanning to view rendering shares one compilation unit.

**Why:** The `_lib` suffix avoids a Windows linker name collision when the bin and lib share the name `locus`. Do not rename without testing on Windows.

---

## Module Visibility Rules

Three visibility levels used consistently:

| Visibility | When to use | Examples |
|------------|-------------|----------|
| `pub mod` | Module used by other crates, integration tests, or Tauri commands | `pub mod asset_db`, `pub mod knowledge_store`, `pub mod unity_bridge` |
| `pub(crate) mod` | Module used internally across the crate but not part of public API | `pub(crate) mod diff`, `pub(crate) mod merge`, `pub(crate) mod eol` |
| `mod` | Module private to its parent — implementation detail | `mod agent`, `mod llm`, `mod session`, `mod tool`, `mod commands` |

Reference: `src-tauri/src/lib.rs:21-68` — module declarations.

**Rule of thumb:** Start with `mod` (private). Upgrade to `pub(crate)` when another module needs it. Upgrade to `pub` only for integration tests or when the Tauri app builder pattern requires it.

---

## Subdirectory Conventions

When a module grows beyond ~500 lines, split it into a subdirectory with `mod.rs`:

```
tool/
├── mod.rs          ← ToolDef, ToolExecuteFn, ToolRegistry, ToolResult
└── builtins/
    ├── mod.rs      ← register_all()
    ├── code.rs
    ├── unity.rs
    ├── filesystem.rs
    └── ...
```

```
session/
├── mod.rs          ← submodule declarations only
├── gateway.rs
├── runtime.rs
├── history.rs
├── models.rs
├── store.rs
└── pending_inputs.rs
```

Other examples: `agent/instance/`, `knowledge_index/`, `diff/semantic/`, `view/templates/`, `unity_bridge/state_probe/`, `merge/`.

**Anti-pattern:** A single `mod.rs` that mixes submodule declarations with substantial implementation. The `commands/mod.rs` file does this (declares ~23 submodules alongside `StreamEvent`, event emission helpers, and re-exports) — new command domains should go into separate files, not inflate `mod.rs` further.

---

## Re-Export Pattern

For `commands/`, all public items from submodules are glob-re-exported from `commands/mod.rs`:

```rust
// commands/mod.rs
mod session;
mod asset;
mod git;
// ...
pub use session::*;
pub use asset::*;
pub use git::*;
```

This lets callers use `crate::commands::some_fn()` without knowing which submodule owns it. Follow this pattern for any module with many submodules that form a single logical API surface.

---

## File Size Guidelines

Several files exceed 100k lines and are modules unto themselves (not split into subdirectories):

| File | Lines | Topic |
|------|-------|-------|
| `knowledge_store.rs` | ~8.5k | Single-domain logic with many CRUD variants |
| `view.rs` | ~8.3k | View engine — all view-related logic in one file |
| `compact.rs` | ~3.4k | Context compaction |
| `cli_driver.rs` | ~2.8k | CLI tool execution |
| `unity_docs.rs` | ~5.7k | Unity documentation |
| `feishu_docs.rs` | ~5.0k | Feishu integration |

**Before adding to these files,** consider whether the new logic could live in its own module instead. The existing size is technical debt — don't make it worse.

---

## Allocator

`lib.rs:9-10` sets mimalloc as the `#[global_allocator]`. This is NOT optional on Windows — the system heap degrades under multi-threaded small-object churn from rayon asset scans, tantivy indexing, and tree-sitter parsing.

Reference: `src-tauri/src/lib.rs:4-10`

**Do not remove or change the allocator without benchmarking on Windows.**
