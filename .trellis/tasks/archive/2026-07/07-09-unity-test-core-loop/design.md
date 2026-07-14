# Core Test Loop Design

## Architecture

The core loop spans four layers:

- Unity Editor plugin (`locus_unity/Editor/`): discovers and runs UTF tests through Unity Test Framework APIs.
- Rust Unity bridge (`src-tauri/src/unity_bridge/`): sends pipe messages, owns active run state, routes progress, saves latest results.
- Agent tool layer (`src-tauri/src/tool/builtins/`): exposes `unity_test_find` and `unity_test_run`.
- Frontend Chat (`src/`): renders progress and final summaries on the initiating tool call block.

The Unity plugin is an executor only. Rust/Tauri owns Locus state such as active run routing and the latest dashboard snapshot.

## Contracts

### Discovery

`unity_test_find` is synchronous request/reply.

Input:

```json
{
  "test_mode": "all | editmode | playmode",
  "assembly_name": "optional",
  "fixture_name": "optional",
  "test_name": "optional",
  "search": "optional"
}
```

Output:

```json
{
  "assemblies": [
    {
      "name": "Tests-EditMode",
      "test_mode": "editmode",
      "fixtures": [
        {
          "name": "HealthSystemTests",
          "tests": [
            {
              "name": "TakeDamage_ReducesHealth",
              "attributes": ["Test"],
              "source_path": "Assets/Tests/HealthSystemTests.cs",
              "line": 42
            }
          ]
        }
      ]
    }
  ]
}
```

Discovery waits for Unity to leave compiling/domain-reload state, but it does not trigger Hot Reload, compile, enter PlayMode, or otherwise change editor state. It does not cache results in the MVP.

Mode classification must come from UTF `TestMode` / test assembly classification. Do not infer mode from `[Test]` or `[UnityTest]`.

Search and filtering semantics:

- `test_mode` is an explicit scope: `all`, `editmode`, or `playmode`.
- `assembly_name`, `fixture_name`, and `test_name` are exact matches, case-insensitive.
- `search` is fuzzy contains matching, case-insensitive.
- `search` matches assembly name, fixture/class name, test method name, full test name, and source path when available.
- Multiple conditions are ANDed: apply `test_mode`, then exact filters, then `search`.
- Output preserves the assembly -> fixture -> test tree but prunes non-matching branches.
- If an assembly or fixture matches `search`, include all descendant fixtures/tests inside that matched branch after exact filters have been applied.
- If only specific tests match `search`, include only those tests.
- No matches returns `assemblies: []`, not an error.

### Run

`unity_test_run` is asynchronous and emits progress through `StreamEvent::ToolCallProgress`.

Input:

```json
{
  "test_mode": "all | editmode | playmode",
  "assembly_name": "optional",
  "fixture_name": "optional",
  "test_name": "optional",
  "search": "optional",
  "tests": [
    { "assembly_name": "Tests", "fixture_name": "HealthSystemTests", "test_name": "TakeDamage_ReducesHealth" }
  ]
}
```

There is no broad implicit default. Callers must specify an explicit scope or explicit target list.

Filtering rules:

- Simple filters run every matching test within the explicit `test_mode`.
- `tests` is the primary explicit target list and must not be broadened by simple filters.
- Mixed EditMode + PlayMode requests are allowed and split into phases if UTF requires separate runs.
- Logical phase order is EditMode, then PlayMode.
- Assertion failures do not short-circuit later phases.

### Progress

Each `unity_test_run` is owned by the initiating agent `tool_call_id`. Rust keeps active `run_id -> session_id/tool_call_id` routing state while the run is active.

Progress display fields:

- phase: `Preparing`, `EditMode`, `PlayMode`, `Cleaning up`
- current test name when available
- completed count / total count
- current failure count

Final Chat output shows summary plus failed/skipped details. Passing tests are not dumped into Chat for large runs.

### Latest Snapshot

Rust/Tauri writes `Locus/test-results/latest.json` inside the Unity project only when a logical run reaches a terminal state:

- completed
- cancelled/interrupted
- preparation error
- runtime error

Running progress is event-only and is not persisted. Each terminal run replaces the previous snapshot. The MVP does not keep history.

Snapshot fields should include:

- run id
- started/finished timestamps
- terminal status
- preparation method/status
- requested scope
- phase summaries
- total summary
- per-test results
- native failure details
- structured error, if any

Dashboard/frontend reads this through a Tauri command, not by hard-coding the file path.

## State Machine

High-level run states:

1. `idle`
2. `preparing`
3. `running_editmode`
4. `running_playmode`
5. `cleaning_up`
6. terminal: `completed`, `cancelled`, `prepare_error`, `runtime_error`

Only one active run is allowed. Any new run request outside `idle` returns `busy`.

Preparation happens once per logical run:

1. Try Hot Reload when available.
2. If Hot Reload is unavailable, rejected, or unsafe, exit PlayMode if needed.
3. Trigger or wait for Unity compilation.
4. Wait for editor stable.
5. Enter PlayMode only when selected tests require PlayMode.

Terminal cleanup exits PlayMode whenever Unity is in or entering PlayMode. Core does not restore the pre-run PlayMode state.

## Errors

Tool/command errors include stable codes plus display messages:

- `unity_disconnected`
- `plugin_missing_or_outdated`
- `test_framework_missing`
- `compile_failed`
- `busy`
- `cancelled`
- `unity_crashed`
- `unknown`

Preparation errors are not counted as test failures and produce zero test results.

## Cancellation

Cancellation is best-effort and should be reached through the existing session/tool cancellation path:

- user stop
- session cancel
- tool cancellation hook, if available

Rust sends `cancel_tests` to Unity. Unity calls `TestRunnerApi.Cancel()`. UTF may wait for the current test to yield or finish. Final output includes partial results collected so far, an interrupted/cancelled outcome, and PlayMode cleanup.

Do not expose a separate `unity_test_cancel` agent tool unless the existing runtime cannot propagate cancellation into the active tool.

## File Boundaries

Unity side:

- Add `locus_unity/Editor/LocusBridge.TestRunner.cs`.
- Modify `LocusBridge.ExecuteCode.cs` only if compiling/debugging test code requires an assemblies mode that includes test assemblies.

Rust side:

- Add `src-tauri/src/unity_bridge/test_runner.rs`.
- Add `src-tauri/src/tool/builtins/unity_test.rs`.
- Register builtins in `tool/builtins/mod.rs`.
- Add commands for latest snapshot read and any required dashboard/event access.
- Register new commands in `src-tauri/src/lib.rs`.

Frontend:

- Extend existing tool call progress/final rendering, likely near `ToolCallBlock.vue` and existing tool preview patterns.
- Do not add a dashboard page in this task.

## Compatibility

- Target Unity version remains 2022.3 LTS+ from the parent task.
- Missing `com.unity.test-framework` returns `test_framework_missing`; core does not install or edit package dependencies.
- `Locus/test-results/latest.json` is project-local and intentionally not auto-ignored by `.gitignore`.

## Deferred Enhancements

- Console log correlation.
- Watch/recorder evidence attachment.
- Result history and trend views.
- Dashboard live-running persistence.
- CI/batchmode execution.
