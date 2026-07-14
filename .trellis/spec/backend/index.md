# Backend Development Guidelines

> Rust backend coding conventions for `src-tauri/src/` — a single ~185k LOC Tauri 2 crate.

---

## Overview

The backend is a **single Rust crate** (`locus_lib` rlib + `locus` bin) with module declarations in `lib.rs`. Modules are organized by domain, each owning its own subdirectory when it has multiple files.

All code under `src-tauri/src/`. The binary entry point `main.rs` is 6 lines and just calls `locus_lib::run()`.

---

## Guidelines Index

| Guide | Description |
|-------|-------------|
| [Module Structure](./module-structure.md) | Module visibility, single-crate layout, subdirectory conventions |
| [Command Handlers](./command-handlers.md) | Tauri IPC command patterns, event emission, request/response conventions |
| [Tool Implementation](./tool-implementation.md) | Agent tool definitions, registration, execution pattern |
| [Error Handling](./error-handling.md) | AppError, IntoAppError, AppResult, and error propagation |
| [Serialization](./serialization.md) | serde conventions, event types, camelCase for frontend |
| [Testing](./testing.md) | Test organization, selftest modules, tempfile patterns |

---

## Quick Reference: Where to Put Code

| What you're adding | Where it goes |
|--------------------|---------------|
| New Tauri IPC command | `commands/<domain>.rs` → declare in `commands/mod.rs` |
| New agent tool | `tool/builtins/<domain>.rs` → register in `tool/builtins/mod.rs` → `register_all()` |
| New LLM provider | `llm/<provider>.rs` → wire into `chat_completions.rs` |
| New Unity bridge feature | `unity_bridge/<feature>.rs` or new subdirectory |
| New knowledge operation | `knowledge_store.rs` for core, `knowledge_index/` for search |
| New session feature | `session/<feature>.rs` |
| Cross-cutting config | `config.rs` or `config_registry.rs` |

---

**Language**: All documentation in **English**.
