# Type Safety

> TypeScript patterns for the Vue 3 frontend.

---

## Overview

TypeScript 5.6 with strict mode enabled (`tsconfig.base.json`). All frontend code is in `src/`.

---

## Type Organization

### Central Types File

`src/types.ts` (~75k) is the single source for shared TypeScript interfaces and types. Add new shared types here.

### Local Types

Types used by a single component or composable can stay in that file. If a type is needed by 2+ files, move it to `types.ts`.

### Rust ↔ TypeScript Alignment

Backend uses `#[serde(rename_all = "camelCase")]` — all JSON from the backend uses camelCase. TypeScript interfaces mirror this:

```typescript
// Matches Rust: #[serde(rename_all = "camelCase")]
interface StreamEventEnvelope {
  runId: string
  type: string
  sessionId: string
  // ...
}
```

Reference: `src-tauri/src/commands/mod.rs` for Rust types, `src/types.ts` for TS counterparts.

---

## IPC Type Safety

Service functions must declare return types using the `invoke<T>()` generic:

```typescript
// services/session.ts
import type { SessionDetail } from '@/types'

export async function createSession(workspaceKey: string): Promise<SessionDetail> {
  return invoke<SessionDetail>('create_session', { workspaceKey })
}
```

---

## Optional Fields

Rust `Option<T>` fields become TypeScript `T | undefined` or `T | null`. Always handle the undefined case:

```typescript
// Rust: pub detail: Option<String>
// TypeScript: detail?: string
if (error.detail) {
  showDetail(error.detail)
}
```

---

## Forbidden Patterns

- **`any` type** — use `unknown` and narrow with type guards, or define the proper type
- **`as` casts on backend responses** — the response shape should match the declared type; if it doesn't, fix the type
- **Non-null assertions (`!`)** — handle the null case instead
- **`@ts-ignore` or `@ts-expect-error`** — fix the type error, don't suppress it

---

## Common Mistakes

- **Assuming backend fields are always present** — check the Rust struct for `Option`; mirror as optional in TS
- **Type-only imports not marked** — use `import type { ... }` for cleaner compilation
- **Using `any` for complex generic types** — define the generic constraint properly
