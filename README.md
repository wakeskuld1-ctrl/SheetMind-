<!-- 2026-03-23: 新增这份双语 README，原因是用户准备发 GitHub 并开始宣发；目的是把产品价值、能力边界和试用入口整理成适合外部访客阅读的首页文案。 -->
# SheetMind

**Turn messy Excel work into structured analysis through a Rust binary-first, Skill-driven workflow.**  
**通过“Rust 二进制优先 + Skill 编排”的方式，把杂乱 Excel 工作流收口成可复用的数据分析流程。**

## 中文简介

SheetMind 是一个面向普通业务用户的本地 Excel 智能分析系统。

它的目标不是让用户学习 Python、pandas 或复杂数据工具，而是让用户通过自然语言和最少确认动作，就能完成：

- Excel 读取与表头确认
- 单表整理、筛选、汇总
- 多表追加与显性关联
- 统计摘要与质量诊断
- 线性回归、逻辑回归、聚类
- 下一步建议与优先级解释

整个运行主链路坚持一个原则：

- **客户侧正式运行只接受 Rust 二进制**
- **不要求用户安装 Python**
- **不依赖 pandas、Jupyter、Node 或其他脚本环境**

## English Overview

SheetMind is a local, binary-first Excel analysis system designed for non-technical business users.

Instead of forcing users to learn Python, pandas, notebooks, or engineering-style data workflows, it lets them work through natural-language prompts and a small number of confirmation steps to complete:

- Excel ingestion and header confirmation
- Single-table cleanup, filtering, and aggregation
- Multi-table append and explicit joins
- Statistical summaries and quality diagnostics
- Linear regression, logistic regression, and clustering
- Next-step recommendations and decision support

The core runtime rule is simple:

- **Customer-facing execution is Rust binary only**
- **No Python installation is required**
- **No pandas, Jupyter, Node, or external scripting runtime is part of the product path**

<!-- 2026-03-23: 新增这一段，原因是 GitHub 首页仍可能让普通用户误以为需要装 Rust/cargo；目的是把“用户入口”和“开发者构建入口”彻底分开。 -->
## Delivery Rule / 交付规则

### 中文

- 普通用户只使用预编译二进制，不直接接触源码工程。
- 普通用户不需要安装 Rust。
- 普通用户不需要安装 cargo。
- 普通用户也不需要安装 Python、pandas、Jupyter 或 Node。

### English

- Ordinary users should use a prebuilt binary instead of the source workspace.
- Ordinary users do not need to install Rust.
- Ordinary users do not need to install cargo.
- Ordinary users also do not need Python, pandas, Jupyter, or Node.

## Why This Exists / 为什么做这个

### 中文

大量 Excel 用户并不是 IT 人员，但他们每天都在处理真实业务数据。

他们通常面临三个问题：

1. 数据就在 Excel 里，但很难稳定整理
2. 想做更高级分析，却被 Python 环境、脚本、安装过程拦住
3. 即使有模型能力，也很难把“表处理 -> 分析建模 -> 决策建议”串成一条自然链路

Excel Skill 的定位，就是把这条链路做成本地、可交付、低部署成本的产品能力。

### English

Many Excel-heavy users are not technical professionals, yet they deal with high-value business data every day.

They usually face three problems:

1. The data already lives in Excel, but cleanup and structuring are unstable
2. More advanced analysis is blocked by Python environments, scripts, and setup friction
3. Even when modeling exists, the full path from table processing to analysis to decision guidance is fragmented

SheetMind is built to turn that path into a local, shippable, low-friction product workflow.

## Core Capabilities / 核心能力

### V1 Today / 当前 V1

- Table processing / 表处理
  - workbook open
  - header inference and confirmation
  - preview, filtering, column selection
  - sorting, top-N, grouping and aggregation
  - append tables
  - explicit joins
- Analysis and modeling / 分析建模
  - summary profiling
  - statistical summary
  - quality diagnostics
  - linear regression
  - logistic regression
  - k-means clustering
- Decision support / 决策助手
  - blocking risks
  - next-step priorities
  - tool suggestions based on current table state
- Local memory / 本地记忆
  - session state
  - reusable `table_ref`
  - stage-aware routing across Skills

## Why Rust Binary Only / 为什么坚持纯二进制

### 中文

我们选择 Rust，不是为了“炫技术”，而是为了交付形态。

对目标用户来说，最重要的不是模型名词，而是：

- 能不能直接运行
- 会不会要求装环境
- 出问题时能不能稳定落地

所以这个项目明确拒绝把客户侧能力建立在 Python 环境之上。  
Python 可以在研发阶段作为辅助工具存在，但**不能成为客户交付依赖**。

### English

Rust is not a branding choice here. It is a delivery choice.

For the target audience, what matters is not the model vocabulary. What matters is:

- Can it run directly?
- Does it require environment setup?
- Can it be delivered and supported reliably?

That is why this project explicitly avoids making customer-facing capability depend on Python.  
Python may exist as a development helper, but **it is not allowed to become a product runtime dependency**.

## How It Works / 工作方式

### 中文

当前推荐的体验路径是三层：

1. **表处理层**
   - 先读 Excel
   - 识别表头
   - 建立确认态
2. **分析建模层**
   - 复用确认态 `table_ref`
   - 做统计摘要、诊断、回归、聚类
3. **决策助手层**
   - 继续复用 `table_ref`
   - 把当前结果翻译成下一步建议

总入口 Skill 会在三层之间做路由，但不做计算。

### English

The current recommended user path is a three-layer flow:

1. **Table Processing**
   - open the workbook
   - infer and confirm headers
   - create a confirmed table state
2. **Analysis & Modeling**
   - reuse the confirmed `table_ref`
   - run summaries, diagnostics, regressions, and clustering
3. **Decision Assistant**
   - continue reusing the same `table_ref`
   - translate results into prioritized next actions

The top-level orchestrator Skill routes between these layers, but does not perform computation.

## Who It Is For / 适合谁

### 中文

- 以 Excel 为主工作界面的业务团队
- 数据量不一定大，但流程复杂、表关系多的团队
- 不希望部署 Python 环境的客户
- 想把“会做 Excel”升级成“会做数据分析”的团队

### English

- Business teams that already live inside Excel
- Teams with moderate data volume but messy multi-table workflows
- Customers who do not want Python environment deployment
- Organizations that want to upgrade from spreadsheet operations to guided analytics

## Real-File Validation / 真实文件验收

This repository already includes real-file acceptance writeups and artifacts.

- Trial guide / 试用说明: `docs/acceptance/2026-03-22-customer-binary-trial-guide.md`
- Real-file validation / 真实文件走查: `docs/acceptance/2026-03-22-v1-final-e2e-real-file.md`
- Acceptance artifacts / 验收工件: `docs/acceptance/artifacts/2026-03-22-v1-final-e2e-real-file`
- P0 risk-threshold trial guide: `docs/acceptance/2026-03-26-p0-join-risk-threshold-trial-guide.md`
- P0 risk-threshold E2E record: `docs/acceptance/2026-03-26-p0-join-risk-threshold-e2e.md`
- P0 risk-threshold artifacts: `docs/acceptance/artifacts/2026-03-26-p0-join-risk-threshold`
- P1 ingress recovery E2E record: `docs/acceptance/2026-03-26-p1-ingress-recovery-e2e.md`
- P1 ingress recovery regression gates: `docs/acceptance/2026-03-26-p1-ingress-recovery-regression-gates.md`
- P1 ingress recovery artifacts: `docs/acceptance/artifacts/2026-03-26-p1-ingress-recovery`

The current acceptance evidence shows that V1 can already:

- confirm a table structure
- produce a reusable `table_ref`
- continue analysis by `table_ref`
- continue decision guidance by `table_ref`
- recover ingress/runtime stops with deterministic A/B/C/D routing and unified UX wording

## Quick Start / 快速开始

<!-- 2026-03-23: 重写 Quick Start，原因是原先把 cargo 放在首页主入口会让普通用户误解；目的是把“预编译二进制试用”和“开发者构建”明确拆开。 -->
### Option A: Ordinary user trial / 普通用户试用

If you are a business user or a customer-side trial user, start from the prebuilt binary path instead of the source code path.

- Use the prebuilt executable delivered by the maintainer.
- Start from the top-level Skill: `skills/excel-orchestrator-v1/SKILL.md`
- Follow the customer binary trial flow: `docs/acceptance/2026-03-22-customer-binary-trial-guide.md`
- Binary delivery guide: `docs/acceptance/2026-03-23-binary-delivery-guide.md`

### Option B: Developer build / 开发者构建

This path is only for maintainers, contributors, or local development. It is not a required step for ordinary users.

Inspect the current tool catalog:

```powershell
cargo run --quiet
```

Build the release binary:

```powershell
cargo build --release
```

Expected binary:

- `target/release/excel_skill.exe`

### Option C: Skill-first walkthrough / 从总入口 Skill 开始

Recommended top-level Skill:

- `skills/excel-orchestrator-v1/SKILL.md`

Then follow:

- `docs/acceptance/2026-03-22-customer-binary-trial-guide.md`
- `docs/acceptance/2026-03-23-binary-delivery-guide.md`

## Current V1 Boundaries / 当前 V1 边界

### 中文

- 中文路径恢复目前仍以规则层和宿主能力降级为主
- 逻辑回归仍需要用户提供真正的二分类目标列
- 决策助手给“下一步建议”，不替用户做最终经营拍板
- 当前还不是 GUI 成品，更接近“可交付内核 + Skill 编排层”

### English

- Chinese-path recovery is still handled mainly through routing rules and host-level fallback logic
- Logistic regression still requires a true binary target column
- The decision assistant recommends next actions; it does not make final business decisions
- This is not yet a polished GUI product; it is closer to a deliverable engine plus Skill orchestration layer

## Roadmap / 路线图

### Near-term / 近期

- stronger join and multi-table planning
- more robust entry recovery
- cleaner customer-facing trial flow
- binary-first packaging refinement
- binary-first chart generation for common Excel visuals

### Longer-term / 后续

- simpler conversational UI
- stronger local memory and lineage
- richer modeling workflows
- report-ready chart packs and guided visual outputs
- better productized delivery around ordinary Excel users

## Chart Capability Direction / 图表能力方向

### 中文

图表能力会作为下一阶段的重要补充方向，但会继续遵守“计算在 Rust Tool 层、交付以二进制为主”的原则。

优先顺序会是：

- 先支持基于确认后的表和分析结果生成常见图表
- 优先覆盖折线图、饼图、柱状图、散点图等业务常用类型
- 让用户通过问答生成更适合汇报和决策阅读的图形输出
- 暂不把“读取并修改客户原始 Excel 内已有图表”作为近期承诺

这意味着后续的产品链路会从“表处理 -> 分析建模 -> 决策建议”，进一步延伸到“图表表达与结果交付”。

### English

Chart capability is planned as a major next-stage extension, while keeping the same rule: computation stays in the Rust tool layer and delivery stays binary-first.

The likely order is:

- generate common charts from confirmed tables and analysis results first
- prioritize line, pie, column, and scatter charts used in everyday business reporting
- let users create more presentation-ready visual outputs through conversation
- avoid promising near-term support for reading and modifying existing charts inside customer workbooks

This extends the product path from table processing, analytics, and decision guidance into visual expression and deliverable-ready output.

## Next Stage / 下一阶段

### 中文

下一阶段我们会优先推进这五件事：

- 更强的多表计划与显性关联稳定性
- 更稳的中文路径与文件入口恢复
- 更简单的一体化问答入口体验
- 面向汇报和交付的二进制优先图表生成能力
- 更接近客户交付的二进制打包与试用流程

目标不是单纯继续堆 Tool，而是把“表处理 -> 分析建模 -> 图表表达 -> 决策建议”这条链路做得更稳、更顺、更像真正能交付给普通业务团队的产品。

### English

The next stage will focus on five priorities:

- stronger multi-table planning and explicit-join reliability
- more robust Chinese-path and file-entry recovery
- a simpler unified conversational entry experience
- binary-first chart generation for reporting and deliverable output
- cleaner binary packaging and customer trial flow

The goal is not just to add more tools. It is to make the full path from table processing to analytics, visual output, and decision guidance more stable, smoother, and closer to a deliverable product for ordinary business teams.

## Project Positioning / 项目定位

**中文一句话：**  
这是一个让普通 Excel 用户不用装 Python，也能通过问答方式获得高级数据处理与分析能力的本地二进制产品方向。

**English one-liner:**  
This is a local, binary-first product direction that helps ordinary Excel users gain advanced data-processing and analytics capability without installing Python.

## P0 Default Execution Template (Multi-table)

For customer-safe P0 execution, the recommended path is:

1. Build a plan with `suggest_multi_table_plan`
2. Execute with `execute_multi_table_plan`
3. Keep `auto_confirm_join=true` only when risk guard is enabled

### Step 1: plan

```json
{
  "tool": "suggest_multi_table_plan",
  "args": {
    "tables": [
      {
        "path": "tests/fixtures/join-customers.xlsx",
        "sheet": "Customers",
        "alias": "customers"
      },
      {
        "path": "tests/fixtures/join-orders.xlsx",
        "sheet": "Orders",
        "alias": "orders"
      }
    ],
    "max_link_candidates": 3
  }
}
```

### Step 2: execute (default safe mode)

```json
{
  "tool": "execute_multi_table_plan",
  "args": {
    "tables": [
      {
        "path": "tests/fixtures/join-customers.xlsx",
        "sheet": "Customers",
        "alias": "customers"
      },
      {
        "path": "tests/fixtures/join-orders.xlsx",
        "sheet": "Orders",
        "alias": "orders"
      }
    ],
    "max_link_candidates": 3,
    "auto_confirm_join": true
  }
}
```

When `auto_confirm_join=true` and no thresholds are provided, runtime applies defaults:

- `max_left_unmatched_rows = 10`
- `max_right_unmatched_rows = 10`
- `max_left_duplicate_keys = 5`
- `max_right_duplicate_keys = 5`

If a `join_preflight` step exceeds threshold, execution stops with:

- `execution_status = "stopped_join_risk_threshold"`
- `stopped_at_step_id` set to the preflight step
- `executed_steps[n].join_risk_guard_breaches` with breached metrics

### Step 3: execute (strict custom guard)

```json
{
  "tool": "execute_multi_table_plan",
  "args": {
    "tables": [
      {
        "path": "tests/fixtures/join-customers.xlsx",
        "sheet": "Customers",
        "alias": "customers"
      },
      {
        "path": "tests/fixtures/join-orders.xlsx",
        "sheet": "Orders",
        "alias": "orders"
      }
    ],
    "max_link_candidates": 3,
    "auto_confirm_join": true,
    "max_left_unmatched_rows": 0,
    "max_right_unmatched_rows": 0,
    "max_left_duplicate_keys": 0,
    "max_right_duplicate_keys": 0
  }
}
```
