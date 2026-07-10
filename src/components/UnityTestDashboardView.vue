<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  ChevronRight,
  ExternalLink,
  Play,
  RefreshCw,
  Search,
  Square,
} from "lucide";
import { t } from "../i18n";
import { useUnityTestDashboard } from "../composables/useUnityTestDashboard";
import type { UnityTestAssemblyView, UnityTestFixtureView } from "../utils/unityTestDashboard";
import LucideIcon from "./icons/LucideIcon.vue";
import BaseButton from "./ui/BaseButton.vue";
import BaseCheckbox from "./ui/BaseCheckbox.vue";
import BaseDropdown, { type DropdownOption } from "./ui/BaseDropdown.vue";

const props = defineProps<{ workingDir: string }>();
const dashboard = useUnityTestDashboard(() => props.workingDir);
const runScope = ref("");

const modeOptions = computed<DropdownOption[]>(() => [
  { value: "all", label: t("unityTest.filter.modeAll") },
  { value: "editmode", label: "EditMode" },
  { value: "playmode", label: "PlayMode" },
]);
const statusOptions = computed<DropdownOption[]>(() => [
  { value: "all", label: t("unityTest.filter.statusAll") },
  { value: "failed", label: t("unityTest.status.failed") },
  { value: "skipped", label: t("unityTest.status.skipped") },
  { value: "not_run", label: t("unityTest.status.notRun") },
]);
const runScopeOptions = computed<DropdownOption[]>(() => [
  { value: "all", label: t("unityTest.run.all") },
  { value: "editmode", label: t("unityTest.run.allEditMode") },
  { value: "playmode", label: t("unityTest.run.allPlayMode") },
]);

watch(runScope, (scope) => {
  if (scope !== "all" && scope !== "editmode" && scope !== "playmode") return;
  runScope.value = "";
  void dashboard.runBroad(scope);
});

function assemblyKeys(assembly: UnityTestAssemblyView): string[] {
  return [...dashboard.testIndex.value.values()]
    .filter((leaf) => leaf.testMode === assembly.testMode && leaf.assemblyName === assembly.name)
    .map((leaf) => leaf.key);
}

function fixtureKeys(assembly: UnityTestAssemblyView, fixture: UnityTestFixtureView): string[] {
  return [...dashboard.testIndex.value.values()]
    .filter((leaf) => leaf.testMode === assembly.testMode
      && leaf.assemblyName === assembly.name
      && leaf.fixtureName === fixture.name)
    .map((leaf) => leaf.key);
}

function setBranch(keys: string[]) {
  dashboard.setBranchChecked(keys, dashboard.branchState(keys) !== "all");
}

function outcomeLabel(outcome?: string): string {
  const normalized = outcome?.toLowerCase();
  if (normalized === "passed") return t("unityTest.status.passed");
  if (normalized === "failed") return t("unityTest.status.failed");
  if (normalized === "skipped") return t("unityTest.status.skipped");
  return t("unityTest.status.notRun");
}

function formatDuration(seconds?: number): string {
  if (seconds == null) return "-";
  if (seconds < 1) return `${Math.round(seconds * 1000)} ms`;
  return `${seconds.toFixed(2)} s`;
}

function formatDate(value?: string): string {
  if (!value) return "-";
  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime()) ? value : parsed.toLocaleString();
}

const progressPercent = computed(() => {
  const progress = dashboard.activeProgress.value;
  if (!progress?.total) return 0;
  return Math.min(100, Math.round((progress.completed / progress.total) * 100));
});
</script>

<template>
  <main class="test-dashboard">
    <section class="browser-pane" aria-labelledby="unity-test-browser-title">
      <header class="pane-header">
        <div>
          <h1 id="unity-test-browser-title">{{ t("unityTest.title") }}</h1>
          <span>{{ dashboard.totalTests.value }} {{ t("unityTest.tests") }}</span>
        </div>
        <BaseButton
          :title="t('unityTest.refresh')"
          :disabled="dashboard.loading.value || !workingDir"
          :aria-label="t('unityTest.refresh')"
          @click="dashboard.refresh"
        >
          <LucideIcon :icon="RefreshCw" :size="13" :class="{ spinning: dashboard.loading.value }" />
        </BaseButton>
      </header>

      <div class="filters">
        <label class="search-field">
          <LucideIcon :icon="Search" :size="14" />
          <input v-model="dashboard.search.value" :placeholder="t('unityTest.search')" />
        </label>
        <BaseDropdown
          v-model="dashboard.modeFilter.value"
          :options="modeOptions"
          menu-align="start"
          :aria-label="t('unityTest.filter.mode')"
        />
        <BaseDropdown
          v-model="dashboard.statusFilter.value"
          :options="statusOptions"
          menu-align="start"
          :aria-label="t('unityTest.filter.status')"
        />
      </div>

      <div class="tree-scroll" role="tree" :aria-busy="dashboard.loading.value">
        <div v-if="!workingDir" class="empty-state">{{ t("unityTest.empty.workspace") }}</div>
        <div v-else-if="dashboard.error.value && !dashboard.discovery.value" class="empty-state error-state">
          <strong>{{ t("unityTest.empty.unavailable") }}</strong>
          <span>{{ dashboard.error.value }}</span>
          <BaseButton @click="dashboard.refresh">{{ t("common.retry") }}</BaseButton>
        </div>
        <div v-else-if="dashboard.loading.value && !dashboard.discovery.value" class="empty-state">
          {{ t("common.loading") }}
        </div>
        <div v-else-if="!dashboard.filteredTree.value.length" class="empty-state">
          {{ dashboard.totalTests.value ? t("unityTest.empty.filtered") : t("unityTest.empty.tests") }}
        </div>

        <div v-for="assembly in dashboard.filteredTree.value" v-else :key="assembly.key" class="tree-group">
          <div class="tree-row assembly-row" role="treeitem" :aria-expanded="dashboard.expandedKeys.value.has(assembly.key)">
            <BaseCheckbox
              :model-value="dashboard.branchState(assemblyKeys(assembly)) === 'all'"
              :indeterminate="dashboard.branchState(assemblyKeys(assembly)) === 'some'"
              :aria-label="assembly.name"
              @update:model-value="setBranch(assemblyKeys(assembly))"
            />
            <button class="tree-name branch-name" type="button" @click="dashboard.toggleExpanded(assembly.key)">
              <LucideIcon :icon="ChevronRight" :size="13" :class="{ open: dashboard.expandedKeys.value.has(assembly.key) }" />
              <span>{{ assembly.name }}</span>
              <small>{{ assembly.testMode === "editmode" ? "EditMode" : "PlayMode" }}</small>
            </button>
          </div>

          <div v-if="dashboard.expandedKeys.value.has(assembly.key)" role="group">
            <div v-for="fixture in assembly.fixtures" :key="fixture.key" class="fixture-group">
              <div class="tree-row fixture-row" role="treeitem" :aria-expanded="dashboard.expandedKeys.value.has(fixture.key)">
                <BaseCheckbox
                  :model-value="dashboard.branchState(fixtureKeys(assembly, fixture)) === 'all'"
                  :indeterminate="dashboard.branchState(fixtureKeys(assembly, fixture)) === 'some'"
                  :aria-label="fixture.name"
                  @update:model-value="setBranch(fixtureKeys(assembly, fixture))"
                />
                <button class="tree-name branch-name" type="button" @click="dashboard.toggleExpanded(fixture.key)">
                  <LucideIcon :icon="ChevronRight" :size="12" :class="{ open: dashboard.expandedKeys.value.has(fixture.key) }" />
                  <span>{{ fixture.name }}</span>
                </button>
              </div>

              <div v-if="dashboard.expandedKeys.value.has(fixture.key)" role="group">
                <div
                  v-for="leaf in fixture.tests"
                  :key="leaf.key"
                  class="tree-row test-row"
                  :class="{ inspected: dashboard.inspectedKey.value === leaf.key }"
                  role="treeitem"
                >
                  <BaseCheckbox
                    :model-value="dashboard.checkedKeys.value.has(leaf.key)"
                    :aria-label="leaf.test.fullName || leaf.test.name"
                    @update:model-value="dashboard.toggleChecked(leaf.key)"
                  />
                  <button class="tree-name test-name" type="button" @click="dashboard.inspect(leaf.key)">
                    <span class="status-dot" :class="dashboard.resultByKey.value.get(leaf.key)?.outcome?.toLowerCase() || 'not-run'"></span>
                    <span>{{ leaf.test.name }}</span>
                    <small>{{ outcomeLabel(dashboard.resultByKey.value.get(leaf.key)?.outcome) }}</small>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <footer class="run-bar">
        <template v-if="dashboard.isActive.value">
          <BaseButton variant="danger" block :disabled="dashboard.cancelling.value" @click="dashboard.cancel">
            <LucideIcon :icon="Square" :size="12" />
            {{ dashboard.cancelling.value ? t("unityTest.stopping") : t("unityTest.stop") }}
          </BaseButton>
        </template>
        <template v-else>
          <BaseButton
            variant="primary"
            block
            :disabled="!dashboard.checkedTests.value.length"
            @click="dashboard.runSelected"
          >
            <LucideIcon :icon="Play" :size="13" />
            {{ t("unityTest.run.selected", dashboard.checkedTests.value.length) }}
          </BaseButton>
          <BaseDropdown
            v-model="runScope"
            class="scope-menu"
            :options="runScopeOptions"
            :placeholder="t('unityTest.run.scope')"
            :disabled="!workingDir"
            :aria-label="t('unityTest.run.scope')"
          />
        </template>
      </footer>
    </section>

    <section class="details-pane" aria-live="polite">
      <template v-if="dashboard.isActive.value">
        <header class="detail-header">
          <div>
            <span class="eyebrow">{{ dashboard.runSource.value === "agent" ? t("unityTest.source.agent") : t("unityTest.source.dashboard") }}</span>
            <h2>{{ t("unityTest.running") }}</h2>
          </div>
          <span class="progress-count">{{ dashboard.activeProgress.value?.completed ?? 0 }} / {{ dashboard.activeProgress.value?.total ?? 0 }}</span>
        </header>
        <div class="progress-view">
          <progress class="progress-track" :value="progressPercent" max="100"></progress>
          <div class="metric-grid">
            <div><span>{{ t("unityTest.progress.phase") }}</span><strong>{{ dashboard.activeProgress.value?.phase || t("unityTest.progress.preparing") }}</strong></div>
            <div><span>{{ t("unityTest.status.failed") }}</span><strong>{{ dashboard.activeProgress.value?.failed ?? 0 }}</strong></div>
          </div>
          <div class="current-test">
            <span>{{ t("unityTest.progress.current") }}</span>
            <code>{{ dashboard.activeProgress.value?.currentTest || t("unityTest.progress.preparing") }}</code>
          </div>
        </div>
      </template>

      <template v-else>
        <div v-if="dashboard.error.value" class="runtime-error-banner" role="alert">
          {{ dashboard.error.value }}
        </div>
        <nav class="detail-tabs" :aria-label="t('unityTest.details')">
          <button :class="{ active: dashboard.detailTab.value === 'latest' }" @click="dashboard.detailTab.value = 'latest'">
            {{ t("unityTest.latest") }}
          </button>
          <button :class="{ active: dashboard.detailTab.value === 'detail' }" @click="dashboard.detailTab.value = 'detail'">
            {{ t("unityTest.testDetail") }}
          </button>
        </nav>

        <div v-if="dashboard.detailTab.value === 'latest'" class="detail-scroll">
          <div v-if="!dashboard.snapshot.value" class="empty-state">{{ t("unityTest.empty.snapshot") }}</div>
          <template v-else>
            <header class="run-summary-header">
              <div>
                <span class="eyebrow">{{ formatDate(dashboard.snapshot.value.finishedAt) }}</span>
                <h2>{{ dashboard.snapshot.value.terminalStatus }}</h2>
              </div>
              <strong>{{ formatDuration(dashboard.snapshot.value.totalSummary.duration) }}</strong>
            </header>
            <div class="summary-grid">
              <div><span>{{ t("unityTest.summary.total") }}</span><strong>{{ dashboard.snapshot.value.totalSummary.total }}</strong></div>
              <div class="passed"><span>{{ t("unityTest.status.passed") }}</span><strong>{{ dashboard.snapshot.value.totalSummary.passed }}</strong></div>
              <div class="failed"><span>{{ t("unityTest.status.failed") }}</span><strong>{{ dashboard.snapshot.value.totalSummary.failed }}</strong></div>
              <div><span>{{ t("unityTest.status.skipped") }}</span><strong>{{ dashboard.snapshot.value.totalSummary.skipped }}</strong></div>
            </div>
            <section class="detail-section">
              <h3>{{ t("unityTest.preparation") }}</h3>
              <p>{{ dashboard.snapshot.value.preparation.method }} · {{ dashboard.snapshot.value.preparation.status }}</p>
              <p v-if="dashboard.snapshot.value.preparation.message" class="muted">{{ dashboard.snapshot.value.preparation.message }}</p>
            </section>
            <section v-if="dashboard.snapshot.value.phaseSummaries.length" class="detail-section">
              <h3>{{ t("unityTest.phases") }}</h3>
              <div v-for="phase in dashboard.snapshot.value.phaseSummaries" :key="`${phase.runId}-${phase.testMode}`" class="phase-row">
                <strong>{{ phase.testMode }}</strong>
                <span>{{ phase.passed }} / {{ phase.total }} · {{ formatDuration(phase.duration) }}</span>
              </div>
            </section>
            <section v-if="dashboard.snapshot.value.results.some(result => result.outcome !== 'passed')" class="detail-section">
              <h3>{{ t("unityTest.failuresAndSkips") }}</h3>
              <article v-for="result in dashboard.snapshot.value.results.filter(item => item.outcome !== 'passed')" :key="`${result.fixtureName}-${result.fullName}`" class="result-item">
                <div><span class="status-dot" :class="result.outcome"></span><strong>{{ result.fullName || result.testName }}</strong></div>
                <p v-if="result.message">{{ result.message }}</p>
                <pre v-if="result.stackTrace">{{ result.stackTrace }}</pre>
              </article>
            </section>
            <section v-if="dashboard.snapshot.value.error" class="detail-section error-state">
              <h3>{{ dashboard.snapshot.value.error.code }}</h3>
              <p>{{ dashboard.snapshot.value.error.message }}</p>
            </section>
          </template>
        </div>

        <div v-else class="detail-scroll">
          <div v-if="!dashboard.inspectedTest.value" class="empty-state">{{ t("unityTest.empty.detail") }}</div>
          <template v-else>
            <header class="run-summary-header">
              <div>
                <span class="eyebrow">{{ dashboard.inspectedTest.value.fixtureName }}</span>
                <h2>{{ dashboard.inspectedTest.value.test.name }}</h2>
              </div>
              <span class="outcome-pill" :class="dashboard.inspectedResult.value?.outcome || 'not-run'">
                {{ outcomeLabel(dashboard.inspectedResult.value?.outcome) }}
              </span>
            </header>
            <div class="detail-section">
              <dl class="test-metadata">
                <dt>{{ t("unityTest.summary.duration") }}</dt><dd>{{ formatDuration(dashboard.inspectedResult.value?.duration) }}</dd>
                <dt>{{ t("unityTest.mode") }}</dt><dd>{{ dashboard.inspectedTest.value.testMode }}</dd>
                <dt>{{ t("unityTest.fullName") }}</dt><dd>{{ dashboard.inspectedTest.value.test.fullName }}</dd>
              </dl>
              <BaseButton
                :disabled="!(dashboard.inspectedResult.value?.sourcePath || dashboard.inspectedTest.value.test.sourcePath)"
                :title="(dashboard.inspectedResult.value?.sourcePath || dashboard.inspectedTest.value.test.sourcePath) ? t('unityTest.openSource') : t('unityTest.sourceUnavailable')"
                @click="dashboard.openSource(dashboard.inspectedTest.value, dashboard.inspectedResult.value)"
              >
                <LucideIcon :icon="ExternalLink" :size="13" />
                {{ t("unityTest.openSource") }}
              </BaseButton>
            </div>
            <section v-if="dashboard.inspectedResult.value?.message" class="detail-section">
              <h3>{{ t("unityTest.assertion") }}</h3>
              <p>{{ dashboard.inspectedResult.value.message }}</p>
            </section>
            <section v-if="dashboard.inspectedResult.value?.stackTrace" class="detail-section">
              <h3>{{ t("unityTest.stackTrace") }}</h3>
              <pre>{{ dashboard.inspectedResult.value.stackTrace }}</pre>
            </section>
          </template>
        </div>
      </template>
    </section>
  </main>
</template>

<style scoped>
.test-dashboard { display: grid; grid-template-columns: minmax(340px, 42%) minmax(420px, 1fr); width: 100%; height: 100%; min-height: 0; background: var(--bg-color); color: var(--text-color); }
.browser-pane, .details-pane { min-width: 0; min-height: 0; display: flex; flex-direction: column; }
.browser-pane { border-right: 1px solid var(--border-color); background: var(--panel-bg); }
.pane-header, .detail-header, .run-summary-header { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
.pane-header { min-height: 58px; padding: 10px 14px; border-bottom: 1px solid var(--border-color); }
h1, h2, h3, p { margin: 0; }
h1 { font-size: 15px; line-height: 1.3; letter-spacing: 0; }
h2 { font-size: 17px; line-height: 1.4; letter-spacing: 0; overflow-wrap: anywhere; }
h3 { font-size: 12px; letter-spacing: 0; color: var(--text-secondary); }
.pane-header span, .eyebrow, .muted { color: var(--text-secondary); font-size: 11px; }
.filters { display: grid; grid-template-columns: minmax(140px, 1fr) 112px 112px; gap: 7px; padding: 9px 12px; border-bottom: 1px solid var(--border-color); }
.search-field { display: flex; align-items: center; gap: 7px; height: 28px; padding: 0 9px; border: 1px solid var(--border-color); border-radius: 6px; background: var(--input-bg); color: var(--text-secondary); }
.search-field:focus-within { border-color: var(--accent-color); }
.search-field input { min-width: 0; width: 100%; border: 0; outline: 0; background: transparent; color: var(--text-color); font-size: 12px; }
.tree-scroll, .detail-scroll { flex: 1; min-height: 0; overflow: auto; }
.tree-scroll { padding: 6px; }
.tree-row { display: flex; align-items: center; gap: 5px; min-height: 29px; border-radius: 5px; }
.tree-row:hover, .test-row.inspected { background: var(--hover-bg); }
.fixture-row { padding-left: 18px; }
.test-row { padding-left: 38px; }
.tree-name { display: flex; align-items: center; gap: 6px; min-width: 0; flex: 1; height: 28px; padding: 0 7px 0 0; border: 0; background: transparent; color: var(--text-color); text-align: left; cursor: pointer; }
.tree-name > span:not(.status-dot) { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; }
.tree-name small { margin-left: auto; flex: none; color: var(--text-secondary); font-size: 10px; }
.branch-name :deep(svg) { transition: transform 0.12s ease; }
.branch-name :deep(svg.open) { transform: rotate(90deg); }
.status-dot { flex: none; width: 7px; height: 7px; border-radius: 50%; background: var(--text-secondary); }
.status-dot.passed, .passed strong { background: #2ca66f; color: #2ca66f; }
.status-dot.failed, .failed strong { background: var(--status-danger-fg); color: var(--status-danger-fg); }
.status-dot.skipped { background: #c79434; }
.run-bar { display: flex; gap: 7px; min-height: 48px; padding: 9px 12px; border-top: 1px solid var(--border-color); }
.scope-menu { width: 132px; flex: none; }
.detail-tabs { display: flex; gap: 4px; min-height: 48px; padding: 9px 16px 0; border-bottom: 1px solid var(--border-color); }
.runtime-error-banner { margin: 10px 14px 0; padding: 8px 10px; border: 1px solid var(--status-danger-border); border-radius: 6px; background: var(--status-danger-bg); color: var(--status-danger-fg); font-size: 11px; overflow-wrap: anywhere; }
.detail-tabs button { padding: 0 12px; border: 0; border-bottom: 2px solid transparent; background: transparent; color: var(--text-secondary); cursor: pointer; font-size: 12px; }
.detail-tabs button.active { border-color: var(--accent-color); color: var(--text-color); }
.detail-scroll, .progress-view { padding: 18px; }
.run-summary-header, .detail-header { padding-bottom: 16px; border-bottom: 1px solid var(--border-color); }
.detail-header { min-height: 74px; padding: 14px 18px; }
.summary-grid, .metric-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); border-bottom: 1px solid var(--border-color); }
.summary-grid > div, .metric-grid > div { display: flex; flex-direction: column; gap: 4px; padding: 14px 10px; }
.summary-grid span, .metric-grid span, .current-test span { font-size: 11px; color: var(--text-secondary); }
.summary-grid strong { font-size: 20px; }
.detail-section { display: grid; gap: 9px; padding: 16px 0; border-bottom: 1px solid var(--border-color); }
.phase-row { display: flex; justify-content: space-between; gap: 12px; font-size: 12px; }
.result-item { display: grid; gap: 7px; padding: 10px; border-left: 2px solid var(--border-strong); background: var(--hover-bg); }
.result-item > div { display: flex; align-items: center; gap: 8px; min-width: 0; overflow-wrap: anywhere; }
pre, code { font-family: var(--font-mono-block); white-space: pre-wrap; overflow-wrap: anywhere; }
pre { max-height: 260px; overflow: auto; margin: 0; padding: 10px; background: var(--input-bg); font-size: 11px; line-height: 1.55; }
.empty-state { min-height: 180px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; padding: 24px; color: var(--text-secondary); text-align: center; font-size: 12px; }
.error-state { color: var(--status-danger-fg); }
.progress-view { display: grid; gap: 20px; }
.progress-track { width: 100%; height: 5px; overflow: hidden; border: 0; border-radius: 3px; background: var(--border-color); appearance: none; }
.progress-track::-webkit-progress-bar { background: var(--border-color); }
.progress-track::-webkit-progress-value { background: var(--accent-color); transition: width 0.2s ease; }
.progress-track::-moz-progress-bar { background: var(--accent-color); }
.metric-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.current-test { display: grid; gap: 8px; }
.progress-count { font-variant-numeric: tabular-nums; color: var(--text-secondary); }
.outcome-pill { padding: 4px 8px; border-radius: 5px; background: var(--hover-bg); font-size: 11px; }
.outcome-pill.failed { color: var(--status-danger-fg); background: var(--status-danger-bg); }
.outcome-pill.passed { color: #2ca66f; }
.test-metadata { display: grid; grid-template-columns: 92px minmax(0, 1fr); gap: 8px 12px; margin: 0; font-size: 12px; }
.test-metadata dt { color: var(--text-secondary); }
.test-metadata dd { min-width: 0; margin: 0; overflow-wrap: anywhere; }
.spinning { animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 860px) { .test-dashboard { grid-template-columns: minmax(300px, 46%) minmax(360px, 1fr); } .filters { grid-template-columns: 1fr 100px; } .filters > :last-child { grid-column: 1 / -1; } }
</style>
