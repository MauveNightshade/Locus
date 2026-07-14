# Runtime UI scaling

## Goal

Allow a user to adjust the Locus desktop application's overall UI scale while
it is running.

## Confirmed Facts

- The repository is the Locus desktop application, with Vue UI code under `src/`.
- Its existing Display settings are persisted in browser storage under
  `locus-display-settings` by `src/composables/useDisplaySettings.ts`.
- `src/components/settings/DisplaySettings.vue` is the existing display-settings
  surface and already consumes that composable.
- No existing application-wide scale setting or CSS zoom/transform mechanism was
  found in the relevant display-settings implementation.

## Requirements

- Scale the Locus desktop application's UI, not the UI of Unity projects
  operated through Locus.
- Apply the selected scale consistently to the main Locus window and all
  Locus-owned standalone windows, including View, Inspector, diff-review, and
  progress windows, as well as the Unity-embedded Locus UI.
- Adjust the scale in 10 percentage-point increments.
- Support values from 50% through 300%, inclusive. The default is 100%.
- The selected scale must persist across restarts.
- A changed value must take effect immediately in the current window; every
  newly opened Locus window must initialize with the saved value.
- The Display settings UI must expose the current percentage, a range slider,
  decrement and increment controls, and a reset action that restores 100%.
- Invalid persisted values must fall back to a valid value in the supported
  range and 10% increment.

## Acceptance Criteria

- [ ] Scope and target of scaling are explicitly defined before implementation.
- [ ] The selected behavior can be adjusted at runtime without restarting the
  affected user experience.
- [ ] The selected scale is restored when Locus is restarted and applied to
  all Locus windows.
- [ ] The Display settings control permits only 50% through 300% in 10%
  increments, displays the chosen percentage, and can reset it to 100%.
- [ ] The application remains usable in both themes at the 50%, 100%, and 300%
  scale boundaries.

## Out of Scope

- Independent per-component typography or layout customization, unless required
  by the selected overall-scaling behavior.
