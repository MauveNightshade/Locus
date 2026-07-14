# Component Guidelines

> Vue 3 component patterns in `src/components/`.

---

## Component Structure

Components use Vue 3 `<script setup lang="ts">` + `<template>` + `<style scoped>` in single-file components:

```
src/components/
├── ChatView.vue              ← Major feature views (~132k largest)
├── CollabView.vue
├── PluginView.vue
├── ...
├── agent/                    ← Agent-specific sub-components
├── chat/                     ← Chat sub-components
├── ui/                       ← Shared UI primitives
├── unity/                    ← Unity-specific UI
└── view/                     ← View system components
```

**Rule:** Top-level `components/` holds full-page or standalone views. Sub-component directories hold pieces specific to one domain.

---

## Props Convention

Props are defined with TypeScript generics in `<script setup>`:

```typescript
const props = defineProps<{
  sessionId: string
  compact?: boolean
}>()
```

Default values use `withDefaults`:
```typescript
const props = withDefaults(defineProps<{
  showHeader?: boolean
  maxHeight?: number
}>(), {
  showHeader: true,
  maxHeight: 600,
})
```

Reference: See `ChatView.vue`, `AgentView.vue` for patterns.

---

## Composition

Complex components delegate logic to composables (`src/composables/`). The component file stays focused on rendering:

```typescript
// Inside ChatView.vue
const { messages, streaming, send } = useChatStore()
const { scrollState, onScroll } = useChatScrollState()
const { renderParts, injectMarkdown } = useMarkdownInject()
```

**Rule:** If a component's `<script setup>` exceeds ~200 lines, extract logic into a composable. The largest composables (e.g., `useKnowledgeState.ts` ~134k, `useStreamReducer.ts` ~34k) are exceptions due to complexity — prefer smaller composables for new code.

---

## Styling

Scoped styles in each `.vue` file:
```vue
<style scoped>
.my-component {
  /* ... */
}
</style>
```

Global styles live in `src/styles/`. No external CSS framework — all UI is custom-built with Lucide icons.

---

## Component Naming

- PascalCase for component files: `ChatView.vue`, `ToolCallBlock.vue`
- kebab-case in templates: `<ToolCallBlock :tool-call="..." />`
- Sub-components grouped by domain directory

---

## Common Mistakes

- **Putting too much logic in the component** — extract to a composable when logic grows
- **Direct DOM manipulation** — use Vue's reactivity system, not `document.querySelector`
- **Not cleaning up event listeners** — use `onUnmounted()` for any manual listeners
- **Applying layout changes during a range drag** — when a slider changes root `zoom`, scale, or another property that moves the slider itself, update only a local preview in `@input` and commit the setting in `@change`. Applying on every `@input` changes the pointer-to-control geometry and makes dragging unstable. Verify that the page does not resize until the drag is committed.
