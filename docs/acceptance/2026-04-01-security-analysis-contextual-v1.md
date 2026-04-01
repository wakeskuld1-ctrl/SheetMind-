# 2026-04-01 综合证券分析 V1

## 目标

把 `security_analysis_contextual` 从最小 MVP 推进到可交付 V1：

- 支持 `tailwind`
- 支持 `mixed`
- 支持 `headwind`
- 支持通过 profile 收口默认代理符号

## 当前调用方式

### 方式 1：显式传代理 symbol

```json
{
  "tool": "security_analysis_contextual",
  "args": {
    "symbol": "601916.SH",
    "market_symbol": "510300.SH",
    "sector_symbol": "512800.SH"
  }
}
```

### 方式 2：使用内置 profile

```json
{
  "tool": "security_analysis_contextual",
  "args": {
    "symbol": "601916.SH",
    "market_profile": "a_share_core",
    "sector_profile": "a_share_bank"
  }
}
```

## 当前内置 profile

- `market_profile = "a_share_core"` -> `510300.SH`
- `sector_profile = "a_share_bank"` -> `512800.SH`

## 返回重点

调用结果建议优先看以下字段：

- `stock_analysis.consultation_conclusion`
- `market_analysis.consultation_conclusion`
- `sector_analysis.consultation_conclusion`
- `contextual_conclusion.alignment`

## alignment 语义

- `tailwind`
  - 个股与大盘、板块同向
  - 更适合作为顺风环境理解
- `mixed`
  - 个股尚未完成确认，或三者没有完全共振
  - 更适合作为观察期理解
- `headwind`
  - 个股方向与大盘、板块明显相反
  - 更适合作为逆风环境理解

## 当前边界

- 这是上层综合证券分析 Tool，不修改 `technical_consultation_basic` 的边界。
- 目前只覆盖：
  - 个股技术面
  - 大盘环境
  - 板块环境
- 目前还不覆盖：
  - 新闻面
  - 公告面
  - 资金面
  - 基本面

## 验证

- `cargo test --test security_analysis_contextual_cli -- --nocapture --test-threads=1`
- `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- `cargo fmt --all`
