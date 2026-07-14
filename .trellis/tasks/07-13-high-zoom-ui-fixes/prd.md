# High zoom UI fixes

## Goal

Resolve user-reported frontend rendering and interaction defects that occur at high UI scaling after runtime UI scaling was introduced.

The user will report each affected interface and its observed defect. Do not independently broaden the scope by surveying unrelated screens.

## Confirmed Facts

- Runtime UI scaling has been implemented on branch `feat/runtime-ui-scaling`.
- At high scaling levels, some frontend interfaces render incorrectly.
- The user will provide defects one at a time.

## Requirements

- R1. Investigate and repair only the high-scaling frontend defects reported by the user for this task.
- R2. Preserve the intended behavior of runtime UI scaling and unaffected interface layouts.
- R3. Record each reported defect with its affected screen, reproduction conditions, root cause, and verification evidence before considering it resolved.
- R4. Do not proactively search for or modify unreported high-scaling issues unless a reported defect directly requires it.
- R5. Correct sidebar resizing on the Session, Test, and Plugin screens so drag coordinates remain in the same layout coordinate system as their stored CSS widths at UI scales above 100%.

## Acceptance Criteria

- [ ] AC1. Every user-reported defect in this task has a documented reproduction condition and a targeted fix.
- [ ] AC2. Each fix is verified at the reported high-scaling level and does not regress the normal scaling level relevant to the affected screen.
- [ ] AC3. Runtime UI scaling remains functional after all completed fixes.
- [ ] AC4. No unrelated frontend screens are changed solely as speculative cleanup.
- [ ] AC5. At each supported UI scale above 100%, starting a drag on the Session, Test, or Plugin sidebar handle leaves the sidebar at its existing width until the pointer moves.
- [ ] AC6. At 100% UI scale, sidebar resizing on the Session, Test, and Plugin screens retains its existing behavior.
- [ ] AC7. Sidebar width remains bounded by the existing minimum and maximum constraints on all three screens.

## Reported Defects

### D1: Scaled sidebar resize starts with a rightward jump

- Affected screens: Session, Test, and Plugin.
- Reproduction: Set the runtime UI scale above 100%, press a sidebar resize handle, then begin dragging.
- Actual result: The sidebar jumps rightward at drag start.
- Expected result: The sidebar does not change width until the pointer moves, then tracks the pointer continuously.
- Control cases: Asset and Knowledge sidebars resize normally at the same scale.
- Root cause: Global CSS zoom is applied in `src/composables/useDisplaySettings.ts`. The affected resize paths mix zoom-adjusted visual geometry from `getBoundingClientRect()` or `clientX` with unscaled CSS layout widths. The mismatch writes a scaled width as a CSS pixel value on the first mouse move.
- Affected code: `src/components/ChatView.vue`, `src/components/UnityTestDashboardView.vue`, and `src/components/PluginView.vue` through its resizable-panel helper.

## Out of Scope

- Discovering high-scaling defects that the user has not reported.
- Changing runtime UI scaling behavior except when necessary to correct a reported defect.

## Notes

- Individual defect details will be added as they are reported.
