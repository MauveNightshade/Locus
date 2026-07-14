# Runtime UI Scaling Implementation Plan

1. Read the frontend component, composable, type-safety, and quality guidelines
   required by `trellis-before-dev`.
2. Add normalized `uiScale` state, initialization, and an immediate setter to
   `useDisplaySettings.ts`; add focused tests for defaults, persistence, and
   invalid stored values.
3. Initialize the saved scale in `App.vue` beside theme and font initialization
   so every Locus renderer applies it on startup.
4. Add the range, decrement/increment controls, current value, reset control,
   styles, and English/Chinese strings to Display settings.
5. Extend the existing display-settings layout tests to cover the setting's
   state contract, UI binding, startup initialization, and translations.
6. Run targeted Vitest coverage, TypeScript checks, and the Trellis quality
   gate. Inspect the UI at 50%, 100%, and 300% in both themes before declaring
   the feature complete.

## Risk Checks

- Validate stored values before applying CSS zoom so corrupted settings cannot
  make the app unusable.
- Verify every standalone route uses `App.vue`; this is the coverage boundary
  for synchronous initialization.
- At 300%, confirm settings controls remain reachable through the existing
  scrollable content areas and no root-level overflow blocks access.
