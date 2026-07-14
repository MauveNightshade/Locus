# Quality Guidelines

> Code quality standards for frontend development.

---

## TypeScript

- Strict mode enabled (`tsconfig.base.json`)
- No `any` without justification comment
- No `as` casts to bypass type errors
- No `@ts-ignore` or `@ts-expect-error`
- Run `bun run typecheck` before committing

Reference: `tsconfig.app.json`, `tsconfig.test.json`

---

## Forbidden Patterns

| Pattern | Why | Alternative |
|---------|-----|-------------|
| Direct `invoke()` in components | Bypasses service layer, hard to test | Use `src/services/` |
| `document.querySelector()` | Bypasses Vue reactivity | Use `ref()` and template refs |
| `any` type | Loses all type safety | Define proper type or use `unknown` |
| `as` cast on API responses | Hides type mismatches | Fix the type definition |
| Inline styles (`style="..."`) | Hard to maintain | Scoped `<style>` block |
| `setTimeout` for waiting on state | Race condition prone | Use `watch` or `nextTick` |
| Mutating props directly | Violates Vue one-way data flow | Emit events to parent |

---

## Required Patterns

- **Service layer**: All IPC goes through `src/services/` — never call `invoke()` elsewhere
- **Error boundaries**: Wrap async operations in try/catch; show user-facing errors via the notification system
- **Loading states**: Every async operation must track and expose a `loading` boolean
- **Cleanup**: `onUnmounted` for any manual listeners, intervals, or subscriptions
- **i18n**: All user-facing strings use the i18n system (`src/i18n.ts`, `src/language/`)

---

## Testing

- Test framework: Vitest 4 + jsdom
- Run: `bun run test` (vitest run) or `bun run test:watch` (vitest)
- Test files in `src/__tests__/`
- Type-check tests separately: `bun run typecheck:test`

---

## Before Commit Checklist

- [ ] `bun run typecheck` passes
- [ ] `bun run test` passes
- [ ] No `console.log` left in production code (use the debug console service instead)
- [ ] No commented-out code without explanation
- [ ] New IPC calls go through services
- [ ] New shared types added to `types.ts`
