# 2026-04-01 综合证券分析 Fullstack V1

## 目标

把 `security_analysis_contextual` 继续上探成可直接面向产品的总入口：

- 保留技术主链：
  - 个股技术面
  - 大盘代理环境
  - 板块代理环境
- 新增免费信息面：
  - 最新财报快照
  - 最近公告摘要
- 输出统一综合结论：
  - `constructive`
  - `watchful_positive`
  - `mixed_watch`
  - `cautious`
  - `technical_only`

## 当前调用方式

```json
{
  "tool": "security_analysis_fullstack",
  "args": {
    "symbol": "002352.SZ",
    "market_symbol": "510300.SH",
    "sector_symbol": "516530.SH",
    "disclosure_limit": 5
  }
}
```

也支持沿用 `security_analysis_contextual` 的 profile 入口：

```json
{
  "tool": "security_analysis_fullstack",
  "args": {
    "symbol": "601916.SH",
    "market_profile": "a_share_core",
    "sector_profile": "a_share_bank"
  }
}
```

## 返回重点

建议优先看以下字段：

- `technical_context.contextual_conclusion.alignment`
- `fundamental_context.status`
- `fundamental_context.profit_signal`
- `disclosure_context.status`
- `disclosure_context.risk_flags`
- `industry_context.headline`
- `integrated_conclusion.stance`

## 信息源边界

首版只使用免费公开源，不使用大模型抓取，不使用 Token：

- 财报快照：
  - 默认使用东财公开财务接口
- 公告摘要：
  - 默认使用东财公告列表接口
- 技术环境：
  - 继续复用本地 `stock_history.db` + `security_analysis_contextual`

## 当前降级规则

- 财报源失败：
  - `fundamental_context.status = "unavailable"`
- 公告源失败：
  - `disclosure_context.status = "unavailable"`
- 只要任一信息源失败：
  - `integrated_conclusion.stance = "technical_only"`

这意味着首版优先保证产品“不断链”，而不是为了信息面完整度让整个 Tool 直接报错。

## 当前边界

- 已覆盖：
  - 技术面聚合
  - 最新财报快照
  - 最近公告摘要
  - 行业代理独立摘要
  - 综合 stance
- 尚未覆盖：
  - 新闻面
  - 资金面
  - 研报一致预期
  - 全行业自动映射
  - 财报/公告本地持久化

## 验证

- `cargo test --test security_analysis_fullstack_cli -- --nocapture --test-threads=1`
- `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- `cargo fmt --all`
