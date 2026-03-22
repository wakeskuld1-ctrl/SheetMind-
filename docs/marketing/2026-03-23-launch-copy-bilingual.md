<!-- 2026-03-23: 新增这份双语宣发文案包，原因是用户准备发 GitHub 并开始宣发；目的是提供可直接复用的仓库描述、短文案、中长文案与发布口径。 -->
# Excel Skill Launch Copy Pack

## GitHub Repository Description

### 中文

一个面向普通业务用户的 Excel 智能分析系统：Rust 二进制优先，不要求安装 Python，通过 Skill 编排完成表处理、分析建模和决策建议。

### English

A binary-first Excel intelligence system for non-technical business users: no Python installation required, with Skill-driven table processing, analytics, modeling, and decision guidance.

## Short Post

### 中文短版

我在做一个 Excel Skill 项目。  
目标很简单：让普通业务用户不用装 Python，也能通过问答方式完成 Excel 表处理、统计分析、回归、聚类和下一步建议。  
底层坚持 Rust 二进制交付，尽量把“环境部署”这件事从用户侧拿掉。

### English Short Version

I’m building an Excel Skill project for non-technical business users.  
The goal is simple: let people work through Excel cleanup, analysis, regression, clustering, and next-step guidance through conversation, without installing Python.  
The runtime is binary-first and built around Rust delivery.

## Medium Post

### 中文中版

很多 Excel 用户并不缺数据，他们缺的是一条稳定、低门槛的数据工作流。

现实里经常是这样：

- 数据就在 Excel 里
- 表很复杂，表头不标准，多表还会关联
- 想进一步做分析建模，却先被 Python 环境和脚本挡住

我在做的 Excel Skill，核心方向就是把这条链路收口：

- 先做表处理
- 再做分析建模
- 最后给出决策建议

而且坚持一个原则：

- 客户侧正式运行只接受 Rust 二进制
- 不要求用户安装 Python

V1 目前已经能覆盖：

- Excel 读取与表头确认
- 预览、筛选、汇总
- 追加、显性 Join
- 统计摘要、质量诊断
- 线性回归、逻辑回归、聚类
- 基于当前结果给下一步建议

如果你也在关注“面向普通业务用户的数据产品”这条路线，这个方向应该会有意思。

### English Medium Version

Many Excel-heavy users do not lack data. They lack a stable, low-friction workflow.

In practice, the story is often the same:

- the data already lives in Excel
- headers are messy, structures vary, and tables need to be joined
- once people want deeper analysis, they hit Python setup and scripting friction

Excel Skill is my attempt to close that gap:

- first table processing
- then analysis and modeling
- then decision guidance

And it follows one hard rule:

- customer-facing execution is Rust binary only
- no Python installation is required

V1 already covers:

- workbook reading and header confirmation
- preview, filtering, and aggregation
- append and explicit joins
- statistical summary and quality diagnostics
- linear regression, logistic regression, and clustering
- next-step guidance based on current table state

If you care about data products for ordinary business users, this direction may be worth watching.

## Long Post

### 中文长版

我一直觉得，真正有价值的数据产品，不只是“模型更强”，而是“让本来用不上模型的人也能用起来”。

Excel 用户就是一个非常典型的群体。

他们每天面对的都是现实业务问题：

- 台账怎么整理
- 多张表怎么拼
- 这张表到底能不能分析
- 该先看统计还是先做模型
- 下一步到底先处理数据质量，还是可以直接进入建模

但大多数这类用户都不会接受复杂环境部署。

所以我在做的 Excel Skill，从一开始就把方向定成：

- 本地运行
- Rust 二进制优先
- 不要求 Python
- Skill 负责编排，Rust Tool 负责真实计算

这背后的逻辑不是“反对 Python”，而是面向客户交付时，环境摩擦会直接毁掉体验。

V1 当前已经形成了一个三层链路：

1. 表处理层
   - 读 Excel
   - 判断表头
   - 形成确认态
2. 分析建模层
   - 统计摘要
   - 质量诊断
   - 线性回归 / 逻辑回归 / 聚类
3. 决策助手层
   - 把当前状态翻译成下一步建议

中间用 `table_ref` 来复用确认态，避免用户在每一层都重复解释文件和表结构。

这条路线我觉得真正有意思的地方，不只是“做出一些 Tool”，而是把 Tool 组织成一个普通业务用户也能用得动的液态软件形态。

如果你也在关注：

- Excel 智能化
- 本地二进制产品
- 面向非技术用户的数据产品
- Skill / Tool 协作式软件

欢迎交流。

### English Long Version

I think the most valuable data products are not only about stronger models.  
They are about making advanced capability usable by people who would otherwise never get there.

Excel users are a perfect example.

Their daily questions are concrete and operational:

- how do I clean this ledger?
- how do I combine these tables?
- is this table even analysis-ready?
- should I summarize first or model first?
- should I fix data quality first, or can I move into modeling now?

Most users in this category will not tolerate complex environment setup.

That is why Excel Skill is built around a very explicit direction:

- local execution
- Rust binary first
- no Python requirement for the customer
- Skills for orchestration, Rust Tools for actual computation

This is not about being “anti-Python.”  
It is about the reality that deployment friction destroys product adoption.

V1 already forms a three-layer path:

1. Table Processing
   - open Excel
   - infer headers
   - create a confirmed table state
2. Analysis & Modeling
   - statistical summary
   - quality diagnostics
   - linear regression / logistic regression / clustering
3. Decision Assistant
   - translate the current state into next actions

The reusable `table_ref` acts as the bridge so users do not have to re-explain file structure at every step.

What makes this direction interesting is not only the tools themselves.  
It is the attempt to organize those tools into a liquid-software style workflow that ordinary business users can actually operate.

If you are interested in:

- Excel intelligence
- binary-first local products
- data products for non-technical users
- Skill/Tool cooperative software

I would love to connect.

## Suggested Launch Headlines

### 中文标题候选

1. 让 Excel 用户不用装 Python，也能做高级分析
2. 一个面向普通业务用户的 Rust 二进制 Excel Skill
3. 把表处理、分析建模、决策建议收口成一个 Excel 问答系统

### English Title Options

1. Advanced Excel analytics without installing Python
2. A Rust binary-first Excel Skill for non-technical business users
3. Turning table processing, modeling, and decision guidance into one Excel conversation flow

## Suggested Hashtags

### 中文

- #Excel
- #数据分析
- #Rust
- #本地软件
- #AI应用

### English

- #Excel
- #DataAnalytics
- #RustLang
- #LocalFirst
- #AIProducts
