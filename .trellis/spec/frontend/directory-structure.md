# Directory Structure

> Frontend code organization in `src/`.

---

## Directory Layout

```
src/
├── main.ts                  ← App bootstrap, Pinia setup, i18n init
├── App.vue                  ← Root component (~67k)
├── types.ts                 ← Central type definitions (~75k)
├── i18n.ts                  ← Internationalization setup
├── hljs.ts                  ← highlight.js configuration
├── vite-env.d.ts            ← Vite type declarations
├── assets/                  ← Static assets (images, fonts)
├── components/              ← Vue components
│   ├── ChatView.vue         ← Major feature views
│   ├── SettingsView.vue
│   ├── ...
│   ├── agent/               ← Agent-specific sub-components
│   ├── chat/                ← Chat sub-components
│   ├── diff/                ← Diff display components
│   ├── git/                 ← Git-specific UI
│   ├── knowledge/           ← Knowledge sub-components
│   ├── ui/                  ← Shared UI primitives
│   ├── unity/               ← Unity-specific UI
│   ├── view/                ← View system components
│   └── ...
├── composables/             ← Reusable Vue composables (~60 files)
│   ├── useStreamReducer.ts  ← Core streaming reducer
│   ├── useKnowledgeState.ts ← Knowledge state (~134k)
│   ├── useSettingsState.ts  ← Settings state
│   └── ...
├── config/                  ← Frontend configuration
├── language/                ← i18n translation files
├── services/                ← Backend IPC service layer (~55 files)
├── stores/                  ← Pinia stores (~10 files)
├── styles/                  ← Global CSS/SCSS
└── utils/                   ← Pure utility functions
```

---

## Where to Put New Code

| What you're adding | Where |
|--------------------|-------|
| New page/feature view | `components/NewFeatureView.vue` |
| Reusable UI widget | `components/ui/NewWidget.vue` |
| Feature-specific sub-component | `components/<domain>/NewThing.vue` |
| Shared behavior/logic | `composables/useNewFeature.ts` |
| Backend IPC calls | `services/newFeature.ts` |
| Global state | `stores/newFeature.ts` |
| Shared TypeScript types | `types.ts` |
| Pure utility (no Vue/state) | `utils/newUtil.ts` |

---

## Naming Conventions

- **Components**: PascalCase, `.vue` extension — `ChatView.vue`
- **Composables**: camelCase, `use` prefix — `useStreamReducer.ts`
- **Services**: camelCase, domain name — `knowledge.ts`, `git.ts`
- **Stores**: camelCase, domain name — `chat.ts`, `project.ts`
- **Types file**: `types.ts` (singular, not `types/`)
- **Sub-directories**: lowercase, domain name — `agent/`, `chat/`, `unity/`

---

## Examples

- Clean component structure: `LoginView.vue` (~12k, focused on login flow)
- Well-organized sub-components: `components/chat/` directory
- Good composable separation: `composables/` — each file handles one concern
