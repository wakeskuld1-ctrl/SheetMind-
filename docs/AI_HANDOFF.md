# AI 交接手册

<!-- 2026-04-13 CST: Rewrite the handoff manual into one clean UTF-8 source of truth.
Reason: the previous file had encoding noise, stale branches, and conflict markers that would mislead the next AI.
Purpose: let the next AI resume from the current Rust stock-training mainline without re-scanning the whole repo. -->

## 1. 当前结论

这个仓库当前最重要的主线，不是继续大重构，也不是回头整理旧 Python 参考工程，而是继续沿 Rust 主链推进证券训练与治理能力。

当前已经明确的共识是：

- 后续默认走现有架构继续做，非必要不重构
- `TradingAgents` 只作为参考，不是当前产品主链
- 当前重点是 `Skill` / `Tool` / Rust CLI 这一层
- 当前最近要推进的是“方向优先”的证券训练能力，而不是再改框架

## 2. 当前分支与工作区

- 仓库根目录：`D:\Rust\Excel_Skill`
- 当前分支：`codex/etf-proxy-import-latest-ready-20260412`
- 当前日期：`2026-04-13`

注意：

- 工作区是脏的，不要随便回滚无关改动
- `docs/`、`progress.md`、`findings.md`、`task_plan.md`、`.trae/CHANGELOG_TASK.md` 里都有历史痕迹
- 后续 AI 应优先做增量追加，不要大面积清洗历史文件

## 3. 当前真正可用的主链

### 3.1 股票/ETF 能力主链

当前已接入正式 Tool / Dispatcher / Catalog 的证券相关主链包括：

- `security_feature_snapshot`
- `security_forward_outcome`
- `security_scorecard_training`
- `security_scorecard_refit`
- `security_shadow_evaluation`
- `security_model_promotion`
- `security_master_scorecard`
- `security_chair_resolution`
- `security_external_proxy_history_import`
- `security_real_data_validation_backfill`
- `security_direction_first_training_run`

### 3.2 当前这轮新增的正式入口

本轮已经完成并验证通过的新入口是：

- `security_direction_first_training_run`

作用：

- 用一个正式 Tool 承接“方向优先、回归次优”的训练编排
- 支持两种输入源：
  - 直接读取已有 `model_registry_path`
  - 直接桥接 `SecurityScorecardTrainingRequest`
- 会输出一份正式 `stage_summary`

固定排序规则已经写死：

1. `direction_test_accuracy`
2. `direction_test_auc`
3. `return_test_directional_hit_rate`
4. `return_test_rmse_improvement_vs_baseline`

## 4. 当前已完成到什么程度

### 4.1 代码能力层

以下事情已经做完：

- 正式接入 `security_direction_first_training_run`
- ETF 训练链已经不是“没做完”，而是“主链可用，精度还要继续做厚”
- `security_scorecard_training -> refit -> shadow -> promotion` 主链已经通
- 回归头治理已经补过一轮
  - 增加 `baseline_rmse`
  - 增加 `rmse_improvement_vs_baseline`
  - 增加 `regression_quality_status`

### 4.2 已通过的关键验证

已经通过的关键测试包括：

- `cargo test --test security_scorecard_training_cli tool_catalog_includes_security_direction_first_training_run -- --nocapture --test-threads=1`
- `cargo test --test security_scorecard_training_cli security_direction_first_training_run_prefers_direction_accuracy_before_regression -- --nocapture --test-threads=1`
- `cargo test security_direction_first_training_run --lib -- --nocapture`
- `cargo test --test security_scorecard_training_cli -- --nocapture --test-threads=1`

### 4.3 训练前数据底座

这部分非常关键，因为它是本轮真正处理过的“运行前 blocker”。

#### 股票价格库

根库路径：

- `D:\Rust\Excel_Skill\.excel_skill_runtime\stock_history.db`

本轮已做的事情：

- 原根库曾是空库
- 已从 `validation_real_data_slices` 下的真实切片库合并回根库

当前结果：

- 共 `25` 个关键符号
- 共 `13700` 行

已覆盖的关键符号包括：

- 银行股：
  - `600036.SH`
  - `600919.SH`
  - `601166.SH`
  - `601169.SH`
  - `601288.SH`
  - `601328.SH`
  - `601398.SH`
  - `601658.SH`
  - `601916.SH`
  - `601939.SH`
  - `601988.SH`
- ETF：
  - `511010.SH`
  - `511060.SH`
  - `511220.SH`
  - `511260.SH`
  - `518800.SH`
  - `518850.SH`
  - `518880.SH`
  - `513100.SH`
  - `513180.SH`
  - `513500.SH`
  - `512000.SH`
  - `512800.SH`
  - `515790.SH`
- 市场锚：
  - `510300.SH`

#### ETF 外部代理库

根库路径：

- `D:\Rust\Excel_Skill\.excel_skill_runtime\security_external_proxy.db`

本轮已做的事情：

- treasury / gold 池原本只有代表符号有代理历史
- 已把池级代理历史复制到同池 peer 符号上，避免多符号 ETF 训练时因为 peer 缺代理而失败

当前结果：

- 共 `13` 个 ETF 相关符号
- 共 `5305` 行

当前已覆盖：

- treasury：
  - `511010.SH`
  - `511060.SH`
  - `511220.SH`
  - `511260.SH`
- gold：
  - `518800.SH`
  - `518850.SH`
  - `518880.SH`
- cross-border：
  - `513100.SH`
  - `513180.SH`
  - `513500.SH`
- equity ETF：
  - `512000.SH`
  - `512800.SH`
  - `515790.SH`

## 5. 当前还没做完的事情

下面这些事还没真正开始执行：

- 7 小时正式长训的请求 JSON 还没落盘
- 执行说明文档还没落盘
- 后台长训进程还没启动
- 因此也还没有 PID、stdout/stderr 日志、最终 `stage_summary`

注意：

- 这不是代码能力缺失
- 是因为上一轮会话被中断在“准备写正式请求并启动”的节点

## 6. 当前推荐的正式长训配置

这是下一位 AI 继续时，默认应采用的正式请求配置。

### 6.1 Market Pools

- `a_share_bank`
  - `600036.SH`
  - `600919.SH`
  - `601166.SH`
  - `601169.SH`
  - `601288.SH`
  - `601328.SH`
  - `601398.SH`
  - `601658.SH`
  - `601916.SH`
  - `601939.SH`
  - `601988.SH`
- `treasury_etf`
  - `511010.SH`
  - `511060.SH`
  - `511220.SH`
  - `511260.SH`
- `gold_etf`
  - `518880.SH`
  - `518800.SH`
  - `518850.SH`
- `cross_border_etf`
  - `513500.SH`
  - `513100.SH`
  - `513180.SH`
- `equity_etf`
  - `512800.SH`
  - `512000.SH`
  - `515790.SH`

### 6.2 Horizons

- `5`
- `10`
- `15`
- `30`

### 6.3 统一窗口

为了和 ETF 代理历史的覆盖上限对齐，默认窗口应先固定为：

- `train_range = 2024-11-01..2025-04-30`
- `valid_range = 2025-05-01..2025-06-20`
- `test_range = 2025-06-21..2025-08-08`

### 6.4 统一样本厚度

- `train_samples_per_symbol = 8`
- `valid_samples_per_symbol = 4`
- `test_samples_per_symbol = 4`

### 6.5 统一基础参数

- `feature_set_version = security_feature_snapshot.v1`
- `label_definition_version = security_forward_outcome.v1`
- `lookback_days = 260`
- `disclosure_limit = 8`

### 6.6 建议的 pool 参数

#### 银行股

- `market_scope = A_SHARE`
- `instrument_scope = EQUITY`
- `market_symbol = 510300.SH`
- `sector_symbol = 512800.SH`
- `market_profile = a_share_core`
- `sector_profile = a_share_bank`
- `stop_loss_pct = 0.03`
- `target_return_pct = 0.05`

#### treasury_etf

- `market_scope = A_SHARE`
- `instrument_scope = ETF`
- `instrument_subscope = treasury_etf`
- `market_symbol = 510300.SH`
- `sector_symbol = 511060.SH`
- `market_profile = a_share_core`
- `sector_profile = bond_etf_peer`
- `stop_loss_pct = 0.003`
- `target_return_pct = 0.005`

#### gold_etf

- `market_scope = A_SHARE`
- `instrument_scope = ETF`
- `instrument_subscope = gold_etf`
- `market_symbol = 510300.SH`
- `sector_symbol = 518800.SH`
- `market_profile = a_share_core`
- `sector_profile = gold_etf_peer`
- `stop_loss_pct = 0.01`
- `target_return_pct = 0.015`

#### cross_border_etf

- `market_scope = A_SHARE`
- `instrument_scope = ETF`
- `instrument_subscope = cross_border_etf`
- `market_symbol = 510300.SH`
- `sector_symbol = 513500.SH`
- `market_profile = a_share_core`
- `sector_profile = cross_border_etf_peer`
- `stop_loss_pct = 0.01`
- `target_return_pct = 0.015`

#### equity_etf

- `market_scope = A_SHARE`
- `instrument_scope = ETF`
- `instrument_subscope = equity_etf`
- `market_symbol = 510300.SH`
- `sector_symbol = 512800.SH`
- `market_profile = a_share_core`
- `sector_profile = equity_etf_peer`
- `stop_loss_pct = 0.01`
- `target_return_pct = 0.015`

## 7. 下一位 AI 的推荐执行顺序

### 7.1 先不要做的事

不要先做这些：

- 不要再提“大重构”
- 不要先改 dispatcher / 架构分层
- 不要回头按 Python 方案重做训练
- 不要先补文档细节再拖慢训练启动

### 7.2 应该立刻做的事

1. 生成正式请求 JSON
   - 建议路径：
     - `D:\Rust\Excel_Skill\.excel_skill_runtime\direction_first_training_run_20260413_7h_request.json`
2. 生成执行说明
   - 建议路径：
     - `D:\Rust\Excel_Skill\docs\execution-notes-2026-04-13-direction-first-training.md`
3. 启动后台进程
   - 优先直接使用：
     - `D:\Rust\Excel_Skill\target\debug\excel_skill.exe`
4. 重定向标准输入/输出/错误
5. 记录：
   - PID
   - 启动时间
   - stdout 日志
   - stderr 日志
   - runtime root
   - stage summary 路径
6. 做一次非中断检查
   - 确认进程还活着
   - 确认训练目录开始生成文件

## 8. 建议的启动方式

建议不要再用临时交互命令手工粘 JSON。

推荐做法：

- 先把完整请求写到 JSON 文件
- 然后用 PowerShell `Start-Process`
- 用 `-RedirectStandardInput` 喂请求文件
- 用 `-RedirectStandardOutput` / `-RedirectStandardError` 落日志

推荐优先使用已编译二进制：

- `D:\Rust\Excel_Skill\target\debug\excel_skill.exe`

如果二进制不存在，再退回：

- `cargo run --quiet --bin excel_skill`

## 9. 关键文件清单

继续接手时优先看这些文件：

### 9.1 本轮最关键

- `D:\Rust\Excel_Skill\src\ops\security_direction_first_training_run.rs`
- `D:\Rust\Excel_Skill\src\ops\security_scorecard_training.rs`
- `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`
- `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- `D:\Rust\Excel_Skill\src\tools\catalog.rs`

### 9.2 本轮计划与背景

- `D:\Rust\Excel_Skill\docs\plans\2026-04-12-direction-first-multi-pool-training-design.md`
- `D:\Rust\Excel_Skill\docs\plans\2026-04-12-direction-first-multi-pool-training-plan.md`
- `D:\Rust\Excel_Skill\progress.md`
- `D:\Rust\Excel_Skill\findings.md`
- `D:\Rust\Excel_Skill\task_plan.md`

### 9.3 可参考的现有 live 产物

- `D:\Rust\Excel_Skill\.excel_skill_runtime\multi_head_live_batch_20260413\training_summary.json`
- `D:\Rust\Excel_Skill\.excel_skill_runtime\balanced_scorecard_live_complete_20260413\training_summary.json`
- `D:\Rust\Excel_Skill\.excel_skill_runtime\analysis_30d_validation_20260412\training_summary_30d.json`

## 10. 关键风险

当前最真实的风险不是代码编译，而是运行数据和时间窗口一致性。

### 10.1 风险点

- ETF 代理历史并没有全覆盖到 2026-04-10
- 因此这轮训练窗口必须先收在 `2025-08-08` 之前
- `security_direction_first_training_run` 当前是“薄编排层”
  - 会正式排序
  - 会正式落 `stage_summary`
  - 但还没有更复杂的自动 stage budget / retry 治理

### 10.2 这轮不要误判的点

- 如果训练没启动，不代表代码不行
- 如果训练启动后提前结束，也不代表失败
- 如果某个 pool 某个 horizon 表现差，优先怀疑样本分离度，不要第一反应再重构

## 11. 对下一位 AI 的硬规则

- 响应语言继续用中文
- 默认继续沿当前架构做，不要再主动提重构
- 非必要不改主分发层
- 不要引回 Python 业务实现
- 手工代码编辑继续用 `apply_patch`
- 每次任务完成后更新：
  - `progress.md`
  - `findings.md`
  - `task_plan.md`
  - `.trae/CHANGELOG_TASK.md`

## 12. 一句话交接

当前主线已经不是“缺训练能力”，而是“训练能力已具备，正式 5 池 x 4 horizon 的方向优先长训还差最后一脚启动”。本轮最关键的隐藏 blocker 也已经处理掉了：根价格库和 ETF peer 代理库都补齐了，下一位 AI 直接写请求、起进程、盯日志即可。
