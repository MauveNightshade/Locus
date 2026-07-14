# Unity Test Dashboard Implementation Plan

## Ordered Work

1. **Lock shared contracts**
   - Extend Rust and TypeScript Unity test types for dashboard lifecycle events and source navigation results.
   - Add shared event-name constants.
   - Add pure frontend helpers for stable identity, discovery indexing, result matching, filtering, and run-request construction.

2. **Expose dashboard backend commands**
   - Add discover, manual run, cancel, active progress, and source navigation commands in src-tauri/src/commands/unity_test.rs.
   - Register commands in src-tauri/src/lib.rs.
   - Add command-owned manual cancellation state without weakening the core single-active-run guard.
   - Make the minimum bridge progress query callable by the command layer.

3. **Broadcast lifecycle for both run sources**
   - Add typed emit helpers for unity-test-progress and unity-test-snapshot-changed.
   - Emit from the dashboard run command.
   - Extend execute_unity_test_run so agent runs also broadcast dashboard lifecycle events while preserving existing Chat ToolCallProgress.
   - Cover success, failed assertions, cancellation, preparation error, and runtime error.

4. **Implement source-at-line navigation**
   - Add a Unity bridge handler that opens a validated source with Unity's configured external script editor at the requested line.
   - Fall back to the existing path-only native open and return whether positioning succeeded.
   - Do not add editor configuration or filename guessing.

5. **Build the frontend service and composable**
   - Extend src/services/unityTest.ts; keep all IPC and Tauri event subscription out of components.
   - Add useUnityTestDashboard.ts with load/error state, lifecycle listeners, project-change reset, filters, expansion, independent checked/inspected state, result merge, and actions.
   - On mount or project change, load discovery, latest snapshot, and active progress in a race-safe sequence.

6. **Build the approved two-column view**
   - Add UnityTestDashboardView.vue.
   - Add focused subcomponents under src/components/unity-test/ for the tree, run controls, recent run, test detail, and progress where that keeps scripts and styles bounded.
   - Implement keyboard focus, labels, indeterminate checkboxes, tooltips, wrapping, loading, empty, error, busy, and cancelled states.
   - Use scheme A in the prototype as information-architecture reference, not production source.

7. **Register the top-level page**
   - Add the tests tab and lazy mount/load path in the UI store and App.vue.
   - Add showTestsTab display configuration, default true.
   - Add English and Chinese strings.

8. **Add focused verification**
   - Frontend tests: stable keys, parameterized/name collision cases, filter AND behavior, hidden checked items, tri-state parents, refresh intersection, result merge, and explicit run scopes.
   - Registration tests: command names, event names, lazy tab registration, and bilingual key parity.
   - Rust tests: request/error mapping, manual cancellation state, event payload serialization, terminal snapshot notification, and source-path validation.
   - Manual Unity validation: discovery, all manual scopes, agent-run mirroring, cancel, navigation with and without a line, disconnected Unity, missing UTF, and project switch.

## Validation Commands

Run targeted checks during implementation, then the full relevant gate:

    corepack pnpm test -- src/__tests__/unityTestDashboard.test.ts
    corepack pnpm typecheck
    corepack pnpm typecheck:test
    corepack pnpm test
    corepack pnpm build
    cargo test --manifest-path src-tauri/Cargo.toml unity_test
    cargo check --manifest-path src-tauri/Cargo.toml

Manual app and Unity workflow:

1. Open Tests and confirm first discovery plus latest snapshot.
2. Filter by name, mode, and status; verify hidden checked tests remain selected.
3. Run selected EditMode, selected PlayMode, mixed selected, all, all EditMode, and all PlayMode.
4. Start a run from Chat and confirm the open dashboard mirrors progress and reloads the terminal snapshot.
5. Open the dashboard after a run has started and confirm active-progress recovery.
6. Stop a manual long-running test and confirm partial/cancelled terminal state plus PlayMode cleanup.
7. Refresh after adding, renaming, and deleting tests; verify surviving selections only.
8. Open a source with a line, with path only, and with no source path.
9. Switch projects and confirm stale discovery, selection, progress, and snapshot state do not leak.
10. Check normal and narrow desktop windows for clipping or overlap.

## Risky Files And Rollback Points

- src-tauri/src/agent/instance/mod.rs: limit changes to lifecycle emission around the existing Unity test callback and terminal paths.
- src-tauri/src/unity_bridge/test_runner.rs: do not change run semantics; expose only progress and cancellation hooks needed by the dashboard.
- src/App.vue and src/stores/ui.ts: follow existing lazy tab patterns and keep the new branch isolated.
- src/language/en.json and zh.json: update together and verify key parity.
- Unity source opening: isolate the new handler so path-only fallback cannot affect test execution.

## Pre-Start Gate

- Read frontend and backend Trellis specs again with trellis-before-dev before production edits.
- Confirm the final PRD, design, and implementation plan together.
- Keep inline mode; no JSONL curation is required.
- Do not run task.py start until the user approves these planning artifacts.
