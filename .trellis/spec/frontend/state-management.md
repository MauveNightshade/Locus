# State Management

> Pinia store conventions in `src/stores/`.

---

## Overview

This project uses **Pinia 3** for global state management. State is organized into domain-specific stores in `src/stores/`.

---

## Store List

| Store | Size | Purpose |
|-------|------|---------|
| `chat.ts` | ~109k | Central chat state: messages, streaming, tool calls, transcript management |
| `model.ts` | ~16k | Model selection and configuration |
| `project.ts` | ~14k | Unity project state |
| `ui.ts` | ~12k | UI state (panels, windows, theme) |
| `auth.ts` | ~3k | Authentication state |
| `agent.ts` | ~2k | Agent configuration |
| `chatChanges.ts` | ~12k | Change tracking within chat |
| `appUpdate.ts` | ~7k | App update checking |
| `notification.ts` | ~7k | Notification state |
| `modelSelection.ts` | ~1k | Model selection helpers |

Reference: `src/stores/`

---

## Store Pattern

Use Pinia's Options API style (`state` + `getters` + `actions`):

```typescript
export const useMyStore = defineStore('myStore', {
  state: () => ({
    items: [] as Item[],
    loading: false,
  }),
  getters: {
    activeItems: (state) => state.items.filter(i => i.active),
  },
  actions: {
    async fetchItems() {
      this.loading = true
      try {
        this.items = await myService.getItems()
      } finally {
        this.loading = false
      }
    },
  },
})
```

Reference: See `stores/auth.ts`, `stores/agent.ts` for clean store examples.

---

## When to Use Global State

| State type | Where |
|------------|-------|
| Cross-component shared state | Pinia store |
| Single-component state | `ref()` in the component |
| Backend data with complex lifecycle (streaming) | Pinia store (`chat.ts`) |
| Backend data with simple CRUD | Service + local `ref` |
| UI ephemeral state (hover, focus) | Local `ref` |

---

## Service Layer

Backend IPC calls go through `src/services/`. Stores call services, never `invoke()` directly:

```typescript
// Correct: store calls service
import { createSession } from '@/services/session'

// Wrong: store calls invoke directly
import { invoke } from '@tauri-apps/api/core'
```

This keeps IPC details out of state management and makes services testable independently.

---

## Common Mistakes

- **Calling `invoke()` from components or stores** — always go through `src/services/`
- **Modifying store state from outside the store** — use actions, not direct property mutation
- **Forgetting `finally` block for `loading` state** — stale loading spinners are a common bug
- **Putting too much in `chat.ts`** — it's already ~109k; prefer new stores for new domains
