# Binary Only Runtime Design

## 背景

这个项目面向的主要用户是普通业务人员，而不是开发人员。

因此，客户侧体验的关键不是“本地运行”本身，而是“免环境部署”：

- 可以在客户本机处理本地 Excel
- 但不能要求客户安装 Python、pandas、Jupyter、Node 等外部脚本环境
- 最终交付物必须以 Rust 二进制为中心

## 本轮审计结论

本轮对仓库运行时代码做了定向审计，结论如下：

- `Cargo.toml` 当前没有 Python 桥接依赖
- `src/` 运行时代码中没有发现 `python`、`pandas`、`jupyter`、`openpyxl`、`pyo3`、`cpython`
- `src/` 中没有发现通过 `std::process::Command` 拉起 Python 的运行时链路
- 当前仓库没有落地的 `.py` 或 `.ipynb` 业务脚本
- 现有 `.xlsx` 测试夹具已经是静态文件，不依赖 Python 动态生成

也就是说，当前真正的产品运行链路已经是 Rust 主导，问题主要在于：

1. Skill 文档没有把“二进制唯一运行时”写成硬约束
2. 开发期校验脚本和临时验证方式容易让人误以为 Python 是产品依赖

## 设计目标

- 把“客户正式运行只接受 Rust 二进制”写成明确规则
- 在四层 Skill 中统一口径，避免后续建议用户安装 Python
- 通过测试守护，防止运行时代码回退到 Python 依赖
- 明确区分“开发辅助工具”和“客户运行依赖”

## 非目标

- 本轮不移除 Codex 自带的外部 Skill 校验工具
- 本轮不清洗所有历史计划文档里的 Python 痕迹
- 本轮不新增新的安装器或 GUI 包装层

## 运行时约束

客户侧正式运行必须满足以下约束：

- 只允许依赖 Rust 二进制
- 不依赖 Python、pandas、Jupyter、Node 或其他脚本运行时
- 不要求用户安装 Python
- Skill 不允许把脚本环境当成业务主链路的一部分

可以保留但必须降级表述的内容：

- 研发阶段的校验脚本
- 文档结构校验脚本
- 研发自测时的临时对照脚本

这些能力只能被归类为“开发辅助”，不能被描述成客户交付的一部分。

## Skill 落点

### `excel-orchestrator-v1`

- 作为总入口，必须先明确客户运行时只接受 Rust 二进制
- 不允许把 Python 环境作为入口恢复或后续分析建议的一部分

### `table-processing-v1`

- 作为表处理层，必须明确读取、清洗、追加、关联都走 Rust Tool
- 不允许建议用户改用 Python 脚本做表处理

### `analysis-modeling-v1`

- 作为分析建模层，必须明确统计摘要、回归、聚类都走 Rust Tool
- 不允许建议用户改用 pandas / notebook 做建模

### `decision-assistant-v1`

- 作为决策建议层，必须明确任何下一步建议都不能把 Python 环境部署转嫁给用户

## 测试策略

### 守护测试 1：运行时代码禁用 Python 栈标记

检查对象：

- `Cargo.toml`
- `src/**/*.rs`

禁止标记：

- `python`
- `pandas`
- `jupyter`
- `openpyxl`
- `pyo3`
- `cpython`

### 守护测试 2：四层 Skill 必须显式声明二进制运行约束

检查对象：

- `skills/excel-orchestrator-v1/SKILL.md`
- `skills/table-processing-v1/SKILL.md`
- `skills/analysis-modeling-v1/SKILL.md`
- `skills/decision-assistant-v1/SKILL.md`

必须出现的关键词：

- `Rust 二进制`
- `不依赖 Python`
- `不要求用户安装 Python`

## 风险

### 风险 1：未来有人为了提速，用 Python 做临时实现再混进主链路

应对方式：

- 用守护测试直接卡住
- 在 Skill 中写死客户运行约束

### 风险 2：开发辅助脚本被误解为客户依赖

应对方式：

- 在文档中明确区分“开发辅助”和“客户交付”

### 风险 3：后续如果引入 GUI 壳层，重新把外部运行时带回来

应对方式：

- 继续坚持“壳层可变，计算内核必须是 Rust 二进制”的原则

## 结论

本轮不需要把现有主链路“从 Python 改成 Rust”，因为当前主链路本来就是 Rust。

本轮真正要做的，是把这个事实升级成制度：

- Skill 层写死
- 测试层锁死
- 文档层讲清楚
