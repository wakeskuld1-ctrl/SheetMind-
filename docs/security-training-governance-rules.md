# 证券训练优先治理规则

<!-- 2026-04-11 CST: Create formal training governance note, reason: user explicitly required all securities conclusions to prioritize training support and forbid sparse-evidence recommendations; purpose: provide a stable written rule set that future AI and humans can follow without relying on chat memory. -->

## 目标

把“训练优先、拟合可披露、无训练不许冒充负责结论”写成正式规则。

这份文档的目的不是替代现有 `security_decision_committee -> security_scorecard -> security_chair_resolution` 主链，而是补上一个更上层的治理判断：

- 什么时候可以把结论说成“训练支撑”
- 什么时候只能说“研究观察”
- 什么时候必须明确“训练不足，不得放行”

## 核心原则

### 1. 训练优先于快结论

所有触达交易、仓位、调仓、胜率、未来赚钱效益的问题，默认都必须先问：

- 当前有没有可用训练 artifact
- 当前有没有可披露拟合摘要
- 当前有没有足够样本避免“看一点点就下结论”

如果答案是否定的：

- 只能降级为研究观察或治理阻断说明
- 不能把少量公开信息、少量技术信号或一次临时分析包装成负责的可执行建议

### 2. 训练结论必须可披露

只要答复里声称“这是训练支持的结论”，就必须至少披露以下信息中的可用部分：

- 模型或 artifact 身份
- horizon / label 定义
- `sample_count`
- `train / valid / test` 划分
- 当前可用拟合摘要

如果这些信息无法披露：

- 不得使用“训练已经证明”“模型已经验证”这类表达

### 3. 训练会持续回算和重估

训练不是一次性动作，而是持续迭代的治理过程。

默认认识必须是：

- 样本越多，训练越可能更稳
- 回算窗口越长，结论越容易暴露问题
- 新的市场环境会改变旧结论的可靠性

因此：

- 任何训练结论都不是永久结论
- 结论必须允许被新的回算、重估和扩样本修正

## 结论分层

### A. 训练支撑的正式结论

满足条件：

- 有正式训练 artifact 或正式评分卡模型
- 有可披露拟合摘要
- 样本规模与切分信息可说明
- 正式治理链已引用这些对象

允许表达：

- `训练支撑的正式结论`
- `训练与正式治理链一致`

### B. 正式链已运行但训练不可用的治理结论

满足条件：

- `security_decision_committee` / `security_chair_resolution` 已运行
- `security_scorecard` 为 `model_unavailable`，或训练不足

允许表达：

- `正式链已给出治理结论`
- `当前结论以风险控制和证据不足为主`

不允许表达：

- `训练已证明可以买`
- `评分卡已经给出高胜率`

### C. 仅研究观察

满足条件：

- 只有公开数据、技术结构、公告解释或人工补证
- 没有可用训练结论，或没有足够拟合信息

允许表达：

- `研究观察`
- `可能性推演`
- `优先观察顺序`

不允许表达：

- `正式建议买入`
- `大概率必涨`
- `可以重仓`

## 当前项目的现实口径

截至 `2026-04-11`，当前项目在训练与评分卡上已经具备：

- 正式训练入口：`security_scorecard_training`
- 正式 artifact：`security_scorecard`
- 正式 refit / registry：`security_scorecard_refit_run`、`security_scorecard_model_registry`

但当前项目能稳定披露的训练摘要仍偏最小化，主要是：

- `sample_count`
- `train/valid/test accuracy`
- `positive_rate`

这意味着：

- 当前可以说“已有最小训练链”
- 但不能把它夸大成“已经具备完整生产级拟合报告”

后续目标应补齐：

- `AUC`
- `KS`
- `OOS / OOT` 命中率
- 分 horizon 表现
- `hit_upside_first / hit_stop_first`
- 更完整的回撤与路径质量指标

## 对后续 AI 的硬约束

后续 AI 在证券场景中必须遵守：

- 没有训练支撑时，不得把少量内容直接写成负责的执行建议
- 正式评分卡为 `model_unavailable` 时，必须明确说清，而不是补人工分数
- 研究链结论只能叫研究链结论，不能冒充训练结论或正式决议
- 任何“很快得出的结论”都必须先说明它来自哪条链

## 最终要求

一句话版本：

- 能训练，就优先训练后再说
- 不能训练，就老实降级成研究观察
- 训练未披露拟合度，就不能冒充已经被证明
