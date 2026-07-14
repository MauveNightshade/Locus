# Core Test Loop - Discover, Run, Analyze, Act

> Child of [unity-test-framework](../07-09-unity-test-framework/prd.md) - Batch 1

## Goal

Give Locus agents a Unity Test Framework core loop:

1. Discover project tests.
2. Run an explicit test scope on demand.
3. Receive structured per-test results with native failure details.
4. Use those results to fix code and rerun tests without human orchestration.

## Background

The parent task defines the Unity testing capability as agent-driven infrastructure. This child owns the core agent path only: Unity test discovery, Unity test execution, structured result delivery, Chat progress display, and a latest result snapshot for later dashboard use.

Unity Test Framework distinguishes EditMode and PlayMode through its own `TestMode` / test assembly classification. Locus must use UTF's classification instead of guessing from `[Test]` or `[UnityTest]` attributes.

## Requirements

- R1: `unity_test_find` discovers Unity Test Framework tests by explicit scope: `all`, `editmode`, or `playmode`.
- R2: `unity_test_find` supports filtering by assembly, fixture/class, test method, and text search.
- R3: `unity_test_find` returns each discovered test's source path when available and best-effort line number when reliable.
- R4: `unity_test_run` supports explicit run scopes: one test, a list of tests, all EditMode tests, all PlayMode tests, all tests, and filtered subsets with explicit `test_mode`.
- R5: `unity_test_run` allows mixed EditMode + PlayMode logical runs, executed as phases under one `run_id`: EditMode first, then PlayMode.
- R6: EditMode assertion failures do not stop later PlayMode execution. Continue collecting results unless preparation fails, Unity crashes/disconnects, or the run is cancelled.
- R7: Only one active test run is allowed. A second run request while active returns a structured `busy` error.
- R8: Before each logical run, Locus performs one code-sync preparation pass: try Hot Reload first; if unavailable or unsafe, exit PlayMode, compile/wait for stable editor state, then enter PlayMode only if selected tests require it.
- R9: Preparation failure is reported separately from test failure and produces zero test results.
- R10: At terminal cleanup, Locus exits PlayMode and leaves Unity in EditMode. It does not restore the pre-run PlayMode state.
- R11: Test failure payloads use native UTF/NUnit details only: test identity, outcome, duration, failure message, and stack trace. Unity Console log correlation is deferred.
- R12: Active runs support best-effort cancellation through the existing session/tool cancellation path. Cancellation returns partial results and still exits PlayMode.
- R13: Core imposes no fixed overall timeout. UTF/NUnit per-test `[Timeout]` remains respected; suspiciously long runs are cancelled by the user.
- R14: Rust/Tauri saves only the latest terminal result snapshot at `Locus/test-results/latest.json` inside the Unity project. It does not keep historical runs and does not auto-edit `.gitignore`.
- R15: Dashboard/frontend access to the latest snapshot goes through a Tauri command, not a hard-coded frontend file path.
- R16: Chat progress is attached to the initiating agent tool call and shows phase, current test, completed/total count, and current failure count.
- R17: Chat final display shows summary and failed/skipped details without dumping every passing test.
- R18: Tool and command failures expose stable structured error codes plus human-readable messages.

## Acceptance Criteria

- [ ] `unity_test_find({ test_mode: "all" })` returns all UTF test assemblies, fixtures, and test methods.
- [ ] `unity_test_find` filters by explicit `test_mode`, exact assembly/fixture/test filters, and fuzzy `search` using the documented AND semantics.
- [ ] `unity_test_find` returns an empty tree (`assemblies: []`) for no matches instead of an error.
- [ ] Discovery returns Unity-project-relative source paths and best-effort line numbers for test methods.
- [ ] `unity_test_run` runs explicit single-test and multi-test target lists.
- [ ] `unity_test_run` runs all EditMode tests, all PlayMode tests, and all tests when explicitly requested.
- [ ] Mixed all-tests runs execute EditMode then PlayMode under one logical `run_id`.
- [ ] EditMode assertion failures do not prevent PlayMode phase execution.
- [ ] PlayMode tests are run with automatic preparation, PlayMode entry, and terminal PlayMode exit.
- [ ] Preparation compile failure reports a preparation error and no test results.
- [ ] Failed tests include native UTF/NUnit message and stack trace.
- [ ] Missing UTF package reports a clear `test_framework_missing` error without modifying project dependencies.
- [ ] Unity disconnect/crash is reported with surviving partial results when available.
- [ ] A second run while one is active reports `busy` and does not start another run.
- [ ] User/session cancellation sends `cancel_tests`, returns partial results, and exits PlayMode.
- [ ] Latest terminal result replaces `Locus/test-results/latest.json`.
- [ ] A Tauri command reads the latest result snapshot for dashboard use.
- [ ] Chat progress updates the initiating tool call block.
- [ ] Chat final output shows summary and failed/skipped details without dumping all passing tests.
- [ ] Tool/command failures expose stable error codes: `unity_disconnected`, `plugin_missing_or_outdated`, `test_framework_missing`, `compile_failed`, `busy`, `cancelled`, `unity_crashed`, `unknown`.
- [ ] Agent can complete a fix -> rerun -> pass cycle for logic-only tests without human intervention.

## Out Of Scope

- Human-AI collaboration layer (covered by `07-09-unity-test-collaboration`).
- Frame/value runtime evidence and richer diagnosis attachment (covered by `07-09-unity-test-recorder` / watch work).
- Unity Console log-to-test correlation.
- Test result history, result database, and trend charts.
- Code coverage collection.
- CI or batchmode execution.
- Performance benchmarks.
- Automatically installing `com.unity.test-framework` or editing Unity package dependencies.
- Automatically editing project `.gitignore`.

## Open Questions

- Does the current tool runtime expose a cancellation hook that can propagate user/session cancellation into an active Rust async tool execution, or is an internal command fallback needed?
- Which existing editor-state probe should be reused for "compilation/domain reload stable" and PlayMode transition waiting?
