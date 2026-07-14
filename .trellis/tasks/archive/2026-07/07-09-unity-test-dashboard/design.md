# Unity Test Dashboard Design

## Architecture

The dashboard is one cross-layer feature. Its UI, commands, lifecycle events, and source navigation share one end-to-end contract and should be verified together.

Layers:

- locus_unity/Editor/: only source-at-line navigation needs a new Unity-side handler.
- src-tauri/src/unity_bridge/test_runner.rs: remains the authority for discovery, the single active run, cancellation, progress, and the latest terminal snapshot.
- src-tauri/src/commands/unity_test.rs: exposes dashboard-safe Tauri commands and broadcasts dashboard lifecycle events.
- src/services/unityTest.ts: is the only frontend IPC/event boundary.
- src/composables/useUnityTestDashboard.ts: owns discovery, filters, selection, result merging, run state, and lifecycle subscriptions.
- src/components/UnityTestDashboardView.vue plus focused src/components/unity-test/ subcomponents render the page.
- src/App.vue, src/stores/ui.ts, and display settings register the lazy top-level tab.

The existing agent tool path remains intact. Agent runs continue emitting session-bound StreamEvent::ToolCallProgress for Chat. The dashboard must not manufacture a fake session or tool call.

## Backend Commands

Extend src-tauri/src/commands/unity_test.rs with:

| Command | Input | Output | Purpose |
|---|---|---|---|
| unity_test_discover | UnityTestFilter | UnityTestDiscovery | Discover the current tree. The dashboard requests testMode all and filters locally. |
| unity_test_run_dashboard | UnityTestRunRequest | UnityTestSnapshot | Start a manual run through the existing core run_tests. |
| unity_test_cancel_dashboard | none | none | Signal manual cancellation and call bridge cancellation as a best-effort fallback. |
| unity_test_active_progress | none | optional UnityTestProgress | Recover an already-running state when the dashboard mounts after the first event. |
| unity_test_latest_snapshot | none | optional UnityTestSnapshot | Existing terminal-state source of truth. |
| unity_test_open_source | path plus optional line | navigation result | Open through Unity at a line when possible, otherwise open the validated file without positioning. |

Every command reads the current workspace path from Workspace; the frontend never sends or assembles the project root.

Errors map UnityTestError.code into stable AppError codes under unity_test.*. Busy, unity_disconnected, test_framework_missing, and cancelled remain distinguishable; the UI does not parse message strings.

### Manual Cancellation

The manual run command creates a tokio watch channel and stores its sender in command-owned state for the duration of the invocation. Cancel:

1. marks the sender cancelled so preparation and Rust orchestration can observe it;
2. calls the existing bridge cancel_tests so an active UTF test is asked to stop;
3. leaves run_tests responsible for terminal cleanup, latest snapshot writing, and leaving PlayMode.

Agent cancellation continues using the agent instance's existing receiver. Calling dashboard cancel while an agent-owned run is active can still request UTF cancellation, but must not claim it cancelled the agent session itself.

## Lifecycle Events

Add two typed global Tauri events.

### unity-test-progress

Payload fields:

- workingDir
- source: agent or dashboard
- progress: UnityTestProgress

### unity-test-snapshot-changed

Payload fields:

- workingDir
- source: agent or dashboard
- runId
- terminalStatus

Both the manual command callback and the existing agent execute_unity_test_run callback broadcast progress. Both callers broadcast snapshot-changed after run_tests reaches a terminal outcome. Because run_tests writes a snapshot even when it returns an error, error paths read the latest snapshot before broadcasting.

Event payloads include workingDir; the composable normalizes and compares it to the active project before mutating state. The dashboard also calls unity_test_active_progress and unity_test_latest_snapshot on mount, so correctness does not depend on observing earlier events.

## Frontend State

useUnityTestDashboard.ts owns:

- discovery and latest snapshot loading/error states;
- current lifecycle progress and whether the run source is agent or dashboard;
- selected right-pane view: latest or detail;
- inspected test key;
- checked test keys;
- expanded assembly/fixture keys;
- text, mode, and status filters;
- derived pruned tree, counts, tri-state parents, and run request;
- listener setup and cleanup.

Do not put domain state in App.vue. The view is lazy-loaded and remains mounted after first visit, following existing top-tab behavior.

### Stable Test Identity

Use a normalized composite key built from testMode, assemblyName, fixtureName, and fullName (falling back to testName). The separator must be unambiguous and the helper must be shared by discovery indexing, checked selection, inspection, and result merging.

After discovery refresh:

- intersect checked keys with the new discovery index;
- retain the inspected key only if it still exists;
- rebuild parent tri-state values from surviving child keys;
- never match by test method name alone.

Latest result matching uses the same identity where the snapshot contains enough data. Results that cannot be matched stay visible in the latest-run failure list but are not assigned to an unrelated tree node.

## Filtering And Selection

The dashboard discovers all tests once and filters locally:

- text contains match, case-insensitive, across assembly, fixture, method, full name, and source path;
- mode: all, editmode, or playmode;
- status: all, failed, skipped, or not_run;
- filters combine with AND;
- ancestor branches remain visible when a descendant matches.

Filtering affects visibility only. Checked tests remain checked while hidden, and the primary action displays the total checked count.

Row behavior:

- checkbox toggles the run selection;
- assembly/fixture checkbox toggles all descendants and is indeterminate for partial selection;
- assembly/fixture name toggles expansion;
- test name changes only the inspected test and right-pane view.

## Run Request Construction

The primary action is disabled with zero checked tests. For selected tests:

- all selected tests are EditMode: send testMode editmode;
- all are PlayMode: send testMode playmode;
- mixed selection: send testMode all;
- always send exact tests targets; never broaden selection through search filters.

The adjacent menu sends explicit broad scopes:

- all tests: testMode all;
- all EditMode: testMode editmode;
- all PlayMode: testMode playmode.

Current search/status/mode filters do not alter these broad requests.

## Two-Column UI

The approved baseline is scheme A in layout-prototypes.html.

Left column:

- discovery refresh;
- text search;
- mode and status filters;
- assembly to fixture to method tree;
- independent tri-state checkboxes and inspected-row highlight;
- selected-count primary run button plus explicit-scope menu.

Right column while idle:

- Recent run and Test detail tabs;
- recent summary, timestamp, phase summaries, failures/skips, terminal or preparation error;
- selected test outcome, duration, assertion message, stack trace, and source action;
- empty states for no snapshot, no selected test, no tests, disconnected Unity, and missing UTF.

Right column while running:

- source indicator (Agent or Dashboard);
- phase, current test, completed/total, failure count, and progress bar;
- Stop action;
- on terminal state, switch back to Recent run and reload the snapshot.

At narrower widths the columns keep stable minimum sizes and may use a constrained split. They do not transform into nested cards. Long names and stack traces wrap or truncate without changing toolbar dimensions.

## Source Navigation

unity_test_open_source validates the returned source path against the current Unity workspace.

- Path plus line: ask Unity Editor to open the asset at that line, using Unity's configured external script editor.
- Path only: reuse the existing native file-open path.
- Unity line navigation unavailable: fall back to opening the validated file and return positioned false, allowing a non-blocking notice.
- Missing path: disable the action and explain that Unity did not return a source location.
- Never guess a file by test name.

The Unity handler should use Unity's asset/script opening API and accept project-relative Assets/ or Packages/ paths. Absolute paths must first be validated and normalized to the current project.

## App Integration

- Add tests to the activeTab union and a testsMounted lazy-mount flag.
- Add a lazy UnityTestDashboardView loader and page branch in App.vue.
- Add showTestsTab to display settings, default true like other feature tabs.
- Add all visible strings to both src/language/en.json and src/language/zh.json.
- Use existing Lucide infrastructure and Locus CSS variables; do not copy the prototype's standalone shell CSS into production.

## Compatibility And Rollback

- Unity compatibility remains 2022.3 LTS+.
- Missing UTF is an explicit state; the dashboard never installs packages.
- The latest snapshot remains one project-local file; no history database or migration is introduced.
- Keep dashboard commands additive so removing the page leaves agent tools unchanged.
- Keep lifecycle emission in small shared helpers so it can be removed without changing the core result format.
- The standalone prototype is planning reference only and is not included in the application build.

## Risks

- Agent and manual callers must emit lifecycle events on success, cancellation, and errors.
- A dashboard opened mid-preparation may have limited progress; show a generic active/busy state instead of stale latest results.
- run_tests currently returns an error after writing an error snapshot; callers must reload the snapshot before broadcasting terminal state.
- Source paths and lines are best-effort UTF metadata and may be missing or use different relative forms.
- Result identity must not collapse parameterized tests or same-named methods from different fixtures.
