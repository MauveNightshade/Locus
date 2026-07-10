import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import type { UnityTestDiscovery, UnityTestSnapshot } from "../types";
import {
  buildUnityTestRunRequest,
  filterUnityTestTree,
  indexUnityTestDiscovery,
  mapUnityTestResults,
  selectedState,
  unityTestKey,
} from "../utils/unityTestDashboard";

const discovery: UnityTestDiscovery = {
  assemblies: [
    {
      name: "Game.EditMode.Tests",
      testMode: "editmode",
      fixtures: [{
        name: "Game.Tests.InventoryTests",
        tests: [
          { name: "AddsItem(1)", fullName: "Game.Tests.InventoryTests.AddsItem(1)", attributes: [], sourcePath: "Assets/Tests/InventoryTests.cs", line: 12 },
          { name: "AddsItem(2)", fullName: "Game.Tests.InventoryTests.AddsItem(2)", attributes: [], sourcePath: "Assets/Tests/InventoryTests.cs", line: 20 },
        ],
      }],
    },
    {
      name: "Game.PlayMode.Tests",
      testMode: "playmode",
      fixtures: [{
        name: "Game.Tests.InventoryTests",
        tests: [{ name: "AddsItem(1)", fullName: "Game.Tests.InventoryTests.AddsItem(1)", attributes: [] }],
      }],
    },
  ],
};

function snapshot(): UnityTestSnapshot {
  const result = {
    assemblyName: "Game.EditMode.Tests",
    fixtureName: "Game.Tests.InventoryTests",
    testName: "AddsItem(1)",
    fullName: "Game.Tests.InventoryTests.AddsItem(1)",
    outcome: "failed",
    duration: 0.25,
    message: "Expected one item",
    stackTrace: "at InventoryTests.AddsItem",
  };
  return {
    runId: "run-1",
    startedAt: "2026-07-10T00:00:00Z",
    finishedAt: "2026-07-10T00:00:01Z",
    terminalStatus: "completed_failed",
    preparation: { method: "hot_reload", status: "ok" },
    requestedScope: { testMode: "editmode" },
    phaseSummaries: [{
      runId: "run-1",
      testMode: "editmode",
      status: "failed",
      total: 1,
      passed: 0,
      failed: 1,
      skipped: 0,
      duration: 0.25,
      results: [result],
    }],
    totalSummary: { total: 1, passed: 0, failed: 1, skipped: 0, duration: 0.25 },
    results: [result],
  };
}

describe("Unity test dashboard projections", () => {
  it("uses mode, assembly, fixture, and full name in stable keys", () => {
    const edit = unityTestKey("editmode", "Asm", "Fixture", "Fixture.Test(1)");
    const play = unityTestKey("playmode", "Asm", "Fixture", "Fixture.Test(1)");
    const parameter = unityTestKey("editmode", "Asm", "Fixture", "Fixture.Test(2)");
    expect(new Set([edit, play, parameter])).toHaveLength(3);
  });

  it("indexes parameterized and same-named tests without collisions", () => {
    const index = indexUnityTestDiscovery(discovery);
    expect(index.size).toBe(3);
  });

  it("maps results through phase mode and combines filters with AND", () => {
    const results = mapUnityTestResults(snapshot());
    const failedEdit = filterUnityTestTree(discovery, "Inventory", "editmode", "failed", results);
    expect(failedEdit).toHaveLength(1);
    expect(failedEdit[0].fixtures[0].tests.map((leaf) => leaf.test.name)).toEqual(["AddsItem(1)"]);
    expect(filterUnityTestTree(discovery, "PlayMode", "editmode", "failed", results)).toEqual([]);
  });

  it("keeps checked state independent from visible filtering", () => {
    const keys = [...indexUnityTestDiscovery(discovery).keys()];
    const checked = new Set(keys.slice(0, 2));
    expect(selectedState(keys, checked)).toBe("some");
    expect(filterUnityTestTree(discovery, "no match", "all", "all", new Map())).toEqual([]);
    expect(checked.size).toBe(2);
  });

  it("builds exact requests and narrows homogeneous modes", () => {
    const leaves = [...indexUnityTestDiscovery(discovery).values()];
    const editRequest = buildUnityTestRunRequest(leaves.filter((leaf) => leaf.testMode === "editmode"));
    expect(editRequest.testMode).toBe("editmode");
    expect(editRequest.tests).toHaveLength(2);
    expect(buildUnityTestRunRequest(leaves).testMode).toBe("all");
  });
});

describe("Unity test dashboard registration", () => {
  it("registers commands, events, lazy tab, and bilingual labels", () => {
    const read = (path: string) => readFileSync(path, "utf8");
    const lib = read("src-tauri/src/lib.rs");
    const command = read("src-tauri/src/commands/unity_test.rs");
    const app = read("src/App.vue");
    const en = read("src/language/en.json");
    const zh = read("src/language/zh.json");
    for (const name of [
      "unity_test_discover",
      "unity_test_run_dashboard",
      "unity_test_cancel_dashboard",
      "unity_test_active_progress",
      "unity_test_open_source",
    ]) expect(lib).toContain(`commands::${name}`);
    expect(command).toContain('"unity-test-progress"');
    expect(command).toContain('"unity-test-snapshot-changed"');
    expect(app).toContain('import("./components/UnityTestDashboardView.vue")');
    expect(app).toContain('id: "tests"');
    expect(en).toContain('"app.tab.tests": "Tests"');
    expect(zh).toContain('"app.tab.tests": "测试"');
  });
});
