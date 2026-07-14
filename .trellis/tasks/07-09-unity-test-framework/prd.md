# Unity Test Framework — Agent-driven Unity testing capability

> Parent task — requirement overview + cross-batch acceptance

## Goal

为 Locus agent 提供**测试基础设施（原语）**，使 AI 能够将测试作为工作流的一部分：用测试定位 bug、验证修复、自查代码质量。同时提供**人机协同测试**能力（Batch 2）：AI 准备测试场景，人工观察/操作，结果反馈给 AI 继续迭代。

## Vision

| 批次 | 目标 | 核心能力 |
|------|------|----------|
| **Batch 1: 核心闭环** | AI 自主完成纯逻辑测试全流程 | 发现/搜索测试 → 运行测试 → 结构化结果 → 据此行动 |
| **Batch 2: 协同层** | 人机交接 | AI 准备测试场景 → 标记人工验证项 → 人执行后 AI 收到反馈 |
| **Batch 3: 高级特性** | TBD | 覆盖率、性能回归等 |

本 parent task 管理整体需求，实现工作在 child tasks。

## Task Map

```
07-09-unity-test-framework (parent) ← 本 task
├── 07-09-unity-test-core-loop      ← 跑测试看结果（agent 工具）
├── 07-09-unity-test-recorder       ← 插桩逐帧记录（定位/调试）
├── 07-09-unity-test-dashboard      ← 测试面板（给人用的 UI）
└── 07-09-unity-test-collaboration   ← 人机协同（AI 准备场景 → 人验证）
```

| Child | 面向 | 解决的问题 |
|-------|------|-----------|
| core-loop | Agent | 「这个功能对吗？」验证修复、回归检查 |
| recorder | Agent | 「哪里出问题了？」定位 bug、理解运行时 |
| dashboard | 用户 | 「项目里的测试现在什么状态？」浏览、管理、手动运行 |
| collaboration | 用户 + Agent | 「画面/手感对吗？」人机交接验证 |

## Key Architecture Decisions

| 决策 | 选择 | 理由 |
|------|------|------|
| 通信方式 | 扩展 named pipe 消息类型 | 需要结构化 JSON、进度推送、取消 — unity_execute 文本通道不够 |
| PlayMode 生命周期 | Locus 自动管理 | Agent 只给指令，handler 内部处理 enter → run → exit |
| Unity 版本 | 2022.3 LTS+ | UTF API 在 2022.3+ 稳定，缩小兼容矩阵 |
| Agent 工具 | `unity_test_find` + `unity_test_run` | 发现和运行是不同频率的操作，分离避免污染上下文 |

## Acceptance Criteria (Cross-Batch)

- [ ] Agent 可以通过测试工作流定位 bug、验证修复、自查代码质量
- [ ] 纯逻辑测试可实现全自动闭环
- [ ] 视觉/交互类测试支持人机协同（Batch 2）
- [ ] Batch 1 + Batch 2 可独立验收和归档

## Follow-up TODO

- [ ] 整个 Unity Test Framework 能力完成后，补一份 Locus/Unity testing skill 文档，教 agent 何时 discover、如何选择测试范围、如何处理失败/取消/编译错误、如何逐步扩大回归范围，以及如何和 watch/recorder/dashboard 协同。Tool description 只保留参数和返回结构说明，不承载完整测试工作流教学。
