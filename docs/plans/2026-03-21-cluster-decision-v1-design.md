# 聚类与决策助手 V1 设计

> 时间：2026-03-21
> 范围：聚类 Tool、分析建模层统一收口、决策助手层 V1、V1 验收收口

## 目标

在现有表处理层、统计桥接层、线性回归与逻辑回归基础上，补齐分析建模层最后一块聚类能力，并把三类建模 Tool 的输出协议统一收口，随后新增决策助手层 V1，让上层 Skill 可以只做编排，不做计算。

## 边界

- 继续保持单二进制、免部署。
- Skill 只负责调用 Tool，不承载任何计算逻辑。
- 聚类 V1 只做数值特征的 KMeans，不做自动选 K、轮廓系数、可视化、复杂初始化对比。
- 逻辑回归仍然维持既定边界，不新增 AUC、混淆矩阵全展开、阈值调优、正则化、多分类 softmax。
- 决策助手 V1 以“质量诊断优先、少量业务统计辅助”为主，不做聊天式自由推理，而是走规则化传统计算。

## 方案对比

### 方案 A：最小独立 Tool 叠加（推荐）
- 做法：新增 `cluster_kmeans`，再抽出共享建模输出结构，最后新增 `decision_assistant` 复用 `analyze_table` 与 `stat_summary`。
- 优点：改动边界清晰，和现在的分层最一致；对 Skill 编排最友好；测试最容易按 TDD 推进。
- 缺点：需要补一层共享输出结构，涉及现有线性/逻辑回归返回体扩展。

### 方案 B：只补聚类，不做统一输出抽象
- 做法：新增聚类 Tool，维持各建模 Tool 各自风格，决策助手单独拼接。
- 优点：实现速度快。
- 缺点：后续 Skill 编排要分别适配三套建模 JSON；决策助手也会堆很多字段映射，后期维护差。

### 方案 C：直接把决策助手做成大一统入口
- 做法：把聚类、分析、建模建议都塞进决策助手内部，只暴露一个高层入口。
- 优点：终端体验最简。
- 缺点：不利于复用和调试；和“Skill 只编排、Tool 负责计算”的既定方向不一致；V1 风险最高。

## 采用设计

采用方案 A。

### 1. 聚类 Tool
- 名称：`cluster_kmeans`
- 输入：`path`、`sheet`、`features`、`casts`、`cluster_count`、`max_iterations`、`missing_strategy`
- 前处理：复用 `model_prep`，新增聚类样本准备函数，只允许数值特征列，缺失值按 `drop_rows` 处理。
- 算法：使用确定性 farthest-point 初始化 + KMeans 迭代，避免引入随机依赖，保证测试稳定。
- 输出：
  - `model_kind` / `problem_type`
  - `features`
  - `cluster_count`
  - `assignments`
  - `cluster_sizes`
  - `cluster_centers`
  - `row_count_used` / `dropped_rows`
  - `data_summary` / `quality_summary`
  - `assumptions`
  - `human_summary`

### 2. 分析建模层统一收口
- 新增共享建模输出模块，统一三类模型最外层协议：
  - `model_kind`
  - `problem_type`
  - `data_summary`
  - `quality_summary`
  - `human_summary`
- 线性回归、逻辑回归、聚类三者都保留各自专有字段，但共享统一总览结构。
- 这样 Skill 和决策助手读取模型结果时，只需要先读统一字段，再按 `model_kind` 处理专有信息。

### 3. 决策助手层 V1
- 名称：`decision_assistant`
- 输入：`path`、`sheet`、`columns`、`casts`、`top_k`
- 内部流程：
  1. 调 `analyze_table` 取质量诊断
  2. 调 `stat_summary` 取统计桥接
  3. 用规则引擎生成优先动作、阻塞风险、可选下一步 Tool 建议
- 输出：
  - `assistant_kind`
  - `table_health`
  - `blocking_risks`
  - `priority_actions`
  - `business_highlights`
  - `next_tool_suggestions`
  - `human_summary`
- 规则优先级：
  1. 全空列 / 高缺失 / 重复行 / 键风险
  2. 分布风险（失衡、零值堆积、异常值）
  3. 可进入的下一步分析动作（统计摘要、回归、逻辑回归、聚类）

### 4. Tool 套用关系
- `cluster_kmeans` -> 复用 `model_prep`
- `linear_regression` / `logistic_regression` -> 复用 `model_prep` + 共享建模输出结构
- `decision_assistant` -> 复用 `analyze_table` + `stat_summary`
- Skill 层只需要根据场景串联 Tool，不需要自己做任何计算或重算。

## 错误处理

- 聚类：
  - 特征列为空
  - 特征列不是数值列
  - 删除缺失值后样本不足
  - `cluster_count` 小于 2 或大于有效样本数
  - 有效样本的不同点数不足以分成指定组数
- 决策助手：
  - 继续沿用底层 Tool 的直白错误文案
  - 不隐藏数据质量阻塞项

## 测试策略

严格走 TDD：
1. 先补聚类内存层测试
2. 再补聚类 CLI 测试
3. 再补统一输出结构测试
4. 再补决策助手行为测试
5. 最后跑全量 `cargo test` 与 `cargo build --release`
