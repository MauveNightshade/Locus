# Unity Test Dashboard

> Child of [unity-test-framework](../07-09-unity-test-framework/prd.md): the human-facing Unity Test Framework workspace.

## Goal

在 Locus 中提供一个独立、简洁的 Unity 测试工作台，让用户浏览当前项目的测试、明确选择运行范围、观察 agent 或人工发起的实时进度、检查最近结果，并从失败信息直接导航到测试源码。该工作台与 Unity Test Runner 窗口相互独立，不承担测试编写和历史分析。

## Background

- Locus 使用顶栏 Tab 导航，因此测试工作台与 Chat、Knowledge、Asset 等页面同级接入。
- 已实现的 core loop 提供测试发现、单 active run、best-effort cancel、UnityTestProgress、终态 UnityTestSnapshot，以及项目内 Locus/test-results/latest.json。
- 最新快照包含运行范围、准备状态、阶段摘要、逐测试结果、结构化错误、best-effort 源码路径和行号。
- Chat 的 StreamEvent::ToolCallProgress 绑定 agent session/tool call。Dashboard 需要独立的全局生命周期事件，不能伪造 Chat 事件。
- 前端已有 latest snapshot 读取服务，但尚无 dashboard 发现、运行、取消、活动进度查询和源码行导航接口。

## Requirements

### R1. Page And Layout

- 测试工作台作为默认可见、可在显示设置中隐藏的顶栏 Tests Tab。
- 主工作区采用已批准的双栏布局：左栏浏览、筛选和勾选测试；右栏展示最近运行、单测详情或实时进度。
- 页面使用现有 Locus 颜色变量、控件密度、Lucide 图标、懒加载和响应式约束。
- 常规与窄桌面窗口中，工具栏、长测试名、堆栈和按钮不得重叠或改变固定控件尺寸。

### R2. Discovery And Filtering

- 首次进入页面和切换 Unity 项目时，以 all 模式自动发现 assembly -> fixture -> test method 测试树。
- 提供手动刷新；新增、删除或重命名测试不会通过定时轮询自动发现。
- 运行结束只把新快照合并进现有树，不额外触发发现。
- 支持大小写不敏感的名称搜索，以及模式筛选（全部、EditMode、PlayMode）和状态筛选（全部、失败、跳过、未运行）。
- 搜索覆盖 assembly、fixture、test method、full name 和可用 source path；多项筛选按 AND 组合并保留命中后代的祖先分支。

### R3. Selection And Inspection

- 复选框只管理待运行集合；点击测试名称只选择右栏详情；两种状态互不影响。
- 点击 assembly/fixture 名称展开或折叠；其复选框批量选择后代，并支持部分选择的三态显示。
- 筛选只隐藏节点，不取消隐藏测试的勾选；运行按钮始终显示实际选中总数。
- 刷新后按稳定测试标识保留仍存在的勾选和详情选择，并移除已删除或重命名测试。
- 稳定标识必须包含模式、assembly、fixture 和 full name（缺失时才退回 method name），不得仅按方法名关联。

### R4. Explicit Run Controls

- 主按钮运行精确勾选项；无勾选时禁用，并显示选中总数。
- 相邻菜单明确提供“全部测试”“全部 EditMode”“全部 PlayMode”。
- 广范围运行不受当前搜索、模式或状态筛选影响。
- 选中项全部属于 EditMode 或 PlayMode 时发送对应明确模式；混合选择发送 all；始终携带精确测试目标列表。
- 运行中将运行入口切换为停止，调用 core loop 的 best-effort cancel，并展示取消后的部分结果和终态。

### R5. Live Synchronization

- Agent 和 Dashboard 发起的测试都广播 dashboard 专用的项目级进度与终态事件；Agent 同时保留原有 Chat ToolCallProgress。
- Dashboard 只消费当前工作区事件，切换项目时清空旧 discovery、selection、progress 和 snapshot 状态。
- 页面挂载时同时查询活动进度和 latest snapshot，避免错过挂载前事件。
- 运行期间右栏显示来源、阶段、当前测试、完成数/总数、失败数和进度。
- 运行进入任一终态后，右栏回到最近运行并重新读取 latest snapshot。

### R6. Results And Detail

- 非运行时右栏提供“最近运行”和“测试详情”两个页签。
- 默认显示最近运行：时间、终态、准备状态、总计/通过/失败/跳过/耗时、阶段摘要、失败和跳过详情。
- 点击左栏测试名称自动切到测试详情：最近 outcome、耗时、断言消息、stack trace 和源码入口。
- 每个测试树叶节点按 latest snapshot 显示通过、失败、跳过或未运行；无法可靠匹配的旧结果不得关联到错误测试。
- 页面覆盖无项目、Unity 未连接、UTF 缺失、无测试、无快照、无详情选择、busy、取消和结构化错误状态。

### R7. Source Navigation

- 有源码路径和行号时，通过 Unity Editor 配置的外部脚本编辑器打开并定位到行。
- 只有路径时打开经过工作区验证的文件。
- 行导航不可用时允许退回路径打开，并以非阻塞方式说明未定位到行。
- 无源码路径时禁用入口并说明 Unity 未返回源码位置。
- 不按测试名称猜测或搜索文件。

### R8. Isolation And Compatibility

- Dashboard 复用 core-loop 的 find_tests、run_tests、cancel_tests 和 latest snapshot，不复制 UTF 执行逻辑。
- 与 Unity Test Runner 窗口相互独立。
- 兼容目标保持 Unity 2022.3 LTS+。
- UTF 缺失时只报告，不自动安装或修改 Unity package。

## Acceptance Criteria

- [ ] AC1 / R1: Tests 作为懒加载顶栏页面出现，默认可见且可在显示设置中隐藏；双栏布局在常规和窄桌面窗口无重叠。
- [ ] AC2 / R2: 首次进入和切换项目会发现完整测试树；手动刷新可反映新增、删除、重命名，且页面不定时轮询。
- [ ] AC3 / R2: 名称、模式、状态筛选可按 AND 组合，结果保留正确的祖先树分支。
- [ ] AC4 / R3: 勾选、详情选择和展开状态彼此独立；父级三态复选正确，筛选不会丢失隐藏项选择。
- [ ] AC5 / R3: 刷新保留仍存在测试的选择并清理失效选择；同名或参数化测试不会错误合并。
- [ ] AC6 / R4: 可运行选中项、全部、全部 EditMode、全部 PlayMode；请求范围明确且不受可见筛选暗中影响。
- [ ] AC7 / R4: 运行中可停止，最终展示 cancelled/partial snapshot，并完成 core-loop 的 PlayMode 清理。
- [ ] AC8 / R5: Agent 和 Dashboard 发起的测试均实时更新工作台；中途打开页面可恢复活动进度，项目间状态不串线。
- [ ] AC9 / R6: 最近运行完整展示摘要、阶段、失败/跳过和结构化错误；点击测试可查看该测试详情并切回最近运行。
- [ ] AC10 / R6: 树叶状态仅来自可靠的 latest result 匹配，未运行和无法匹配的测试不会显示伪造状态。
- [ ] AC11 / R7: 路径+行号可定位源码；路径-only 可打开文件；无路径时入口禁用；系统不会按名称猜文件。
- [ ] AC12 / R8: Dashboard 与 Unity Test Runner 独立，UTF 缺失不会触发自动安装，现有 agent 工具和 Chat 展示不回归。

## Out Of Scope

- 在 Dashboard 中编写或编辑测试。
- 测试结果历史、趋势、覆盖率或数据库。
- Dashboard 自动触发 agent 分析或修复。
- Console 日志与单测结果关联；等待 recorder/watch 能力增强。
- CI 或 Unity batchmode 测试执行。
- 自动安装 Unity Test Framework 或修改项目 .gitignore。

## Planning Reference

- 可交互布局比较稿：[layout-prototypes.html](./layout-prototypes.html)。
- 实现以其中 A 方案“双栏”为信息架构和视觉基线；B、C 方案不进入实现。
