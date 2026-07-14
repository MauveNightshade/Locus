# Frame-by-Frame Value Recorder

> Child of [unity-test-framework](../07-09-unity-test-framework/prd.md) — 「哪里出问题了？」

## Goal

为 Locus agent 提供运行时插桩记录能力：在 Unity Play Mode 下逐帧记录指定变量的值，帮助 AI 定位 bug、理解运行时行为。

## Scope

首期 MVP：**方案 1（字段监视）+ 方案 2（手动插桩 trace）**。

| 方案 | 做法 | 场景 |
|------|------|------|
| 字段监视 | Agent 指定 `类名.字段名` → Locus 反射读取单例或场景中 Component 的值 | 「看看这个值在哪些帧变了」 |
| 手动插桩 | Agent 往代码里插入 `LocusTrace.Watch(...)` → hot reload 生效 | 「在这个函数被调用时记录入参」 |

**不纳入首期：** 函数调用自动追踪（Harmony patch）、实时表达式评估（Roslyn scripting）。

## Architecture Decisions（已确定）

| 决策 | 选择 |
|------|------|
| 工具模式 | **独立工具** `unity_watch`，单一工具 + action（`start`/`stop`/`status`） |
| 字段目标 | targets 数组只管字段监视；手动 trace 由 C# 侧隐式收录 |
| 实例获取 | 反射读取任意单例 + 场景中任意 Component |
| 帧级 Hook | EditorApplication.update / PlayerLoop 注入 |
| 传输策略 | C# 写文件到 `Library/LocusWatch/`，pipe 只返回路径 |
| 文件格式 | **CSV**，delta-only（只写值发生变化的帧，不变列留空） |
| 数据压缩 | Pipe 返回统计概要（每列 initial/final/min/max/change_count），Agent 自行按需读 CSV |
| 前端展示 | 统计卡片 + 「Open in Sheet」按钮打开 CSV |
| 类型支持 | 首期 built-in 类型 + `ILocusWatchable` 接口（用户可 extension 添加自定义序列化） |

## Requirements

- [ ] R1: Agent 可以反射监视任意单例的字段/属性值
- [ ] R2: Agent 可以反射监视场景中任意 Component 的字段/属性值
- [ ] R3: Agent 可以通过插入 trace 调用来手动埋点
- [ ] R4: 支持限时录制和手动停止
- [ ] R5: Watch 可与 test_run 组合使用（先 watch → 跑 test → stop 拿数据）
- [ ] R6: Watch 可与人工操作组合（start → 人操作 → stop）
- [ ] R7: Delta-only CSV + 统计概要 → 数据压缩合理，不污染 agent 上下文
- [ ] R8: 统计概要在 Chat UI 中以卡片展示

## Out of Scope

- 函数调用自动追踪（Harmony patch）
- 实时表达式评估（Roslyn scripting）
- 录制数据持久化存储

## Open Questions

1. API 参数的 instance 寻址方式 → design 阶段讨论。
2. `ILocusWatchable` 接口的具体签名 → design 阶段讨论。
