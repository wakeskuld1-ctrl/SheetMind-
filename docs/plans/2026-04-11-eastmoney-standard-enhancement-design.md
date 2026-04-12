# EastMoney Standard Enhancement Design

## Goal

在不替换现有本地价格历史与技术分析主链的前提下，为证券分析链新增东财“标准增强版”能力：

- 资金面补充
- 公告/资讯事件面补充
- 调用预算池
- 本地缓存池
- 对 `security_analysis_contextual`、`security_analysis_fullstack`、`security_decision_evidence_bundle` 的稳定接入

## Scope

本轮只做方案 2：

- 做：资金面包 + 事件面包
- 做：预算控制与缓存控制
- 做：统一 provider 边界
- 做：降级返回，不能因为东财接口失败拖垮证券主链
- 不做：多 Token 池
- 不做：基本面快照新字段扩展到评分卡训练特征
- 不做：高频实时订阅
- 不做：替换现有价格历史数据源

## Current State

- `security_analysis_fullstack.rs` 已直接调用东财公开接口拉取财务与公告。
- 当前实现缺少统一 provider 边界，HTTP、解析、业务聚合耦合在同一文件。
- 当前实现缺少预算池，无法严格控制免费额度。
- 当前实现缺少缓存池，重复分析同一股票会浪费调用。
- 当前实现没有资金面补充对象，证券主链仍偏“技术面 + 财务/公告”。

## Recommended Architecture

新增独立的东财 provider 层，证券分析主链只消费结构化补充对象，不直接拼 URL 或解析 JSON。

### Layer 1: Provider

新增 `src/providers/eastmoney/`：

- `client.rs`
  - 统一 HTTP 请求
  - 统一超时、错误转换、基础 headers
- `types.rs`
  - 财务、公告、资金面原始响应结构与标准化结构
- `cache.rs`
  - 缓存 key、缓存策略、缓存命中元信息
- `mod.rs`
  - 暴露 provider 对外 API

### Layer 2: Runtime Stores

新增运行时落盘：

- `src/runtime/eastmoney_budget_store.rs`
  - 记录按自然日统计的调用次数
  - 支持按能力类型分别计数
- `src/runtime/eastmoney_cache_store.rs`
  - 落盘缓存 provider 标准化结果
  - 支持 TTL

### Layer 3: Enrichment

新增证券域统一补充对象：

- `src/ops/eastmoney_enrichment.rs`

统一输出：

- `CapitalFlowContext`
- `EventContext`
- `EastMoneyEnrichment`
- `BudgetStatus`
- `CacheStatus`

### Layer 4: Stock Ops Integration

- `security_analysis_contextual`
  - 接入资金面补充
- `security_analysis_fullstack`
  - 改为消费 enrichment，而不是自己直连东财
- `security_decision_evidence_bundle`
  - 把资金面/事件面补充纳入证据质量与风险摘要

## Call Strategy

### Budget Policy

默认免费额度按 `50 次/天` 设计：

- 资金面：30 次
- 事件面：20 次

预算耗尽时：

- 不报致命错误
- 返回 `status = "budget_exhausted"` 的降级对象
- 在 `risk_flags` 和 `data_gaps` 中显式记录

### Cache Policy

- 资金面：当日缓存
- 事件面：4 小时缓存

缓存命中时：

- 不消耗预算
- 在结果中记录 `cache_hit = true`

## File Changes

### Create

- `src/providers/mod.rs`
- `src/providers/eastmoney/mod.rs`
- `src/providers/eastmoney/client.rs`
- `src/providers/eastmoney/types.rs`
- `src/providers/eastmoney/cache.rs`
- `src/runtime/eastmoney_budget_store.rs`
- `src/runtime/eastmoney_cache_store.rs`
- `src/ops/eastmoney_enrichment.rs`
- `tests/eastmoney_enrichment_cli.rs`

### Modify

- `Cargo.toml`
- `src/lib.rs`
- `src/runtime/mod.rs`
- `src/ops/stock.rs`
- `src/ops/security_analysis_contextual.rs`
- `src/ops/security_analysis_fullstack.rs`
- `src/ops/security_decision_evidence_bundle.rs`
- `README.md`
- `.env.example`

## Error Handling

- HTTP 失败：降级，不打断证券主链
- JSON 结构变化：降级，并保留可诊断错误
- 预算耗尽：降级，并标记 budget exhausted
- 缓存读取失败：回退远端请求，不让缓存层拖垮主链

## TDD Strategy

先写失败测试，再最小实现：

1. 预算未命中缓存时会消耗额度
2. 缓存命中时不会再次消耗额度
3. 资金面能进入 `security_analysis_contextual`
4. 事件面能进入 `security_analysis_fullstack`
5. 预算耗尽时 fullstack/evidence bundle 仍然返回降级结果

## Risks

- 现有依赖 fullstack 的长链测试很多，重构需保证兼容字段不被破坏
- Windows 环境下测试易受进程残留影响
- 东财公开接口字段命名可能波动，需要保守解析

## Verification

- `cargo test --test security_analysis_fullstack_cli -- --nocapture`
- `cargo test --test security_decision_evidence_bundle_cli -- --nocapture`
- `cargo test --test eastmoney_enrichment_cli -- --nocapture`
