# Composable (Hook) Guidelines

> Vue 3 composable conventions in `src/composables/`.

---

## Overview

This project uses Vue 3 composables (not React hooks). All composables live in `src/composables/` and follow the `use*` naming convention. There are ~60 composables covering state management, streaming, markdown, Unity integration, and more.

---

## Key Composables

| Composable | Size | Purpose |
|------------|------|---------|
| `useStreamReducer.ts` | ~34k | Core streaming response reducer for tool call state machine |
| `useKnowledgeState.ts` | ~134k | Knowledge system state management |
| `useSettingsState.ts` | ~53k | Settings state |
| `useCollabState.ts` | ~44k | Collaboration state |
| `useAssetState.ts` | ~35k | Asset state management |
| `useAppBootstrap.ts` | ~24k | App initialization sequence |
| `useEmbeddedChatSession.ts` | ~47k | Embedded chat in Unity window |
| `markdownInject.ts` | ~43k | Markdown content injection/rendering |
| `toolCallBatches.ts` | ~26k | Tool call batching and display |
| `streamingTextChunks.ts` | ~6k | Streaming text buffering |
| `streamingMarkdownBlocks.ts` | ~9k | Streaming markdown block detection |
| `chatInputIntents.ts` | ~11k | Chat input intent parsing |

Reference: `src/composables/`

---

## Composable Pattern

```typescript
// composables/useMyFeature.ts
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { myService } from '@/services/myService'

export function useMyFeature(config: { autoFetch?: boolean } = {}) {
  const data = ref<MyType[]>([])
  const loading = ref(false)

  async function fetch() {
    loading.value = true
    try {
      data.value = await myService.getData()
    } finally {
      loading.value = false
    }
  }

  if (config.autoFetch) {
    onMounted(fetch)
  }

  return { data, loading, fetch }
}
```

---

## Naming Convention

- **All composables**: `use` prefix + PascalCase — `useStreamReducer`, `useChatScrollState`
- **Markdown-specific composables**: no `use` prefix when they're pure transform functions — `markdownMath.ts`, `markdownImages.ts`
- **File name matches export name**: `useStreamReducer.ts` exports `useStreamReducer`

---

## When to Extract a Composable

- Component `<script setup>` exceeds ~200 lines
- Logic is reused by 2+ components
- Logic has its own lifecycle (`onMounted`, `onUnmounted`, watchers)
- Logic manages complex async state (streaming, polling)

---

## Common Mistakes

- **Creating a composable for one component's private logic** — keep it in the component until reuse is needed
- **Not cleaning up in `onUnmounted`** — event listeners, intervals, and subscriptions leak memory
- **Putting IPC calls in composables** — go through `src/services/` instead
- **Forgetting return types** — composables should have clear typed return values
