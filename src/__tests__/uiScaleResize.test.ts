import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { uiScaleFactor, useDisplaySettings } from "../composables/useDisplaySettings";

const cwd = process.cwd();

function read(relPath: string) {
  return readFileSync(resolve(cwd, relPath), "utf8");
}

describe("UI-scale sidebar resizing", () => {
  afterEach(() => {
    useDisplaySettings().state.uiScale = 100;
  });

  it("converts the configured percentage to a layout-coordinate factor", () => {
    useDisplaySettings().state.uiScale = 150;

    expect(uiScaleFactor()).toBe(1.5);
  });

  it("uses the scale factor in every reported sidebar resize path", () => {
    expect(read("src/components/ChatView.vue"))
      .toContain("(e.clientX - sessionSplitterLayoutLeft) / uiScaleFactor()");
    expect(read("src/components/UnityTestDashboardView.vue"))
      .toContain("(event.clientX - resizeStartX) / uiScaleFactor()");
    expect(read("src/components/UnityTestDashboardView.vue"))
      .toContain('querySelector<HTMLElement>(".browser-pane")?.offsetWidth');
    expect(read("src/composables/useResizablePanel.ts"))
      .toContain("size.value = clampSize(pos / uiScaleFactor());");
  });

  it("lets the asset preview retain space at high UI scales", () => {
    expect(read("src/components/AssetView.vue"))
      .toContain("min-width: min(220px, 22vw);");
    expect(read("src/components/AssetView.vue"))
      .toContain("min-width: min(260px, 26vw);");
    expect(read("src/components/AssetView.vue"))
      .toContain("flex-shrink: 1;");
    expect(read("src/components/asset/AssetStatsView.vue"))
      .toContain("repeat(auto-fit, minmax(min(100%, 260px), 1fr))");
  });
});
