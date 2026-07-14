# Locus Project Specs

> Coding conventions and design constraints for the Locus codebase.
> Loaded by AI agents before implementation work.

---

## Spec Directories

| Directory | Scope | Status |
|-----------|-------|--------|
| [backend/](./backend/) | Rust backend (src-tauri/src/): ~185k LOC Tauri 2 crate | ✅ Filled |
| [frontend/](./frontend/) | Vue 3 + TypeScript frontend (src/): components, stores, composables | ⚠️ Partial — needs team review |
| [guides/](./guides/) | Cross-cutting thinking guides: code reuse, layer boundaries | ✅ Filled |

---

## When Specs Are Loaded

- **All specs** are loaded into context when `trellis-before-dev` runs
- **Per-package specs** are loaded when working in that package's directory
- **Guides** are always available as thinking aids

---

## How to Update

1. After learning something valuable from debugging or implementation → use `trellis-update-spec`
2. When conventions change → edit the relevant file directly
3. When adding a new package/layer → create a new directory with `index.md`

---

**Last updated:** 2026-07-09 — Initial backend spec bootstrap
