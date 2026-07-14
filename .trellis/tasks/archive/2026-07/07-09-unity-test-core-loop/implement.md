# Core Test Loop Implementation Plan

## Order

1. Inspect existing Unity bridge pipe message handling, editor state probes, Hot Reload/recompile helpers, and tool cancellation support.
2. Define shared Rust types for discovery filters, run requests, progress events, test results, summaries, latest snapshot, and structured errors.
3. Implement Unity-side `find_tests` handling using UTF APIs and UTF mode classification.
4. Implement Unity-side `run_tests` and `cancel_tests` handling with UTF `ExecutionSettings` / `Filter` and callback collection.
5. Implement Rust bridge send/receive functions and single-active-run state.
6. Implement pre-run preparation: Hot Reload first, fallback to exit PlayMode -> compile/wait stable -> enter PlayMode when needed.
7. Implement mixed-run phase orchestration: EditMode first, then PlayMode, under one logical `run_id`.
8. Route progress to the initiating `tool_call_id` through `StreamEvent::ToolCallProgress`.
9. Implement latest snapshot write on terminal states at `Locus/test-results/latest.json`.
10. Add Tauri command to read the latest snapshot.
11. Implement `unity_test_find` and `unity_test_run` agent tools and register them.
12. Wire existing cancellation path to active test run cancellation; add internal fallback only if needed.
13. Update Chat rendering for Unity test progress/final summary.
14. Add focused tests for request validation, active-run busy behavior, latest snapshot serialization, and frontend rendering logic.

## Validation

Run the standard checks relevant to touched layers:

```powershell
corepack pnpm test
corepack pnpm build
cargo test
cargo check
```

If full Rust checks are too slow during iteration, run targeted tests first, then full build/check before marking implementation complete.

Manual Unity validation:

1. `unity_test_find({ test_mode: "all" })` sees EditMode and PlayMode tests.
2. Run one EditMode test.
3. Run one PlayMode test and confirm Locus exits PlayMode afterward.
4. Run all tests and confirm EditMode phase precedes PlayMode phase.
5. Force an assertion failure and confirm native message/stack trace appears.
6. Cancel a long-running test and confirm partial results plus PlayMode exit.
7. Remove/disable UTF package in a disposable project and confirm `test_framework_missing`.
8. Confirm `Locus/test-results/latest.json` is replaced only on terminal run state.

## Risk Points

- UTF callback thread/main-thread requirements for pipe sends.
- Existing cancellation hooks may not reach a running async tool without additional wiring.
- Hot Reload availability/failure detection may vary by project.
- PlayMode transition waiting must use existing stable-state probes instead of fixed sleeps where possible.
- Source line metadata may be missing or unreliable; path-only navigation must remain valid.

## Rollback Points

- Keep Unity plugin changes isolated to `LocusBridge.TestRunner.cs` where possible.
- Keep latest snapshot writing behind one Rust module function so the path can change if needed.
- Register tools and commands only after the bridge path is functional.
- Frontend rendering should gracefully ignore unknown/missing progress fields.

## Planning Gate

Before `task.py start`, review `prd.md`, `design.md`, and `implement.md` together and confirm the scope is still limited to the core agent loop, not the dashboard or collaboration child tasks.
