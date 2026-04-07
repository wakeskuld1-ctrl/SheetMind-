# 股票历史 HTTP 同步设计

## 背景

当前 `SheetMind` 已经有稳定的 `CSV -> SQLite -> technical_consultation_basic` 主线，但如果只依赖手工 CSV，后续技术面能力会受制于数据导入效率。用户已经确认两点约束：

- 保留 `CSV -> SQLite` 主线，不要推翻已有导入路径。
- 新增“腾讯 + 新浪”双 HTTP 数据源，但不要把系统主依赖绑死在任何单一外部老接口上。

## 目标

在不改现有 `technical_consultation_basic` 业务结构的前提下，新增一个并列 Tool：

- 从腾讯 `fqkline` 拉取 A 股日线。
- 腾讯失败时按请求允许降级到新浪 KLine。
- 将结果写入现有 `stock_price_history` SQLite 主表。
- 保留原 `import_stock_price_history` 的 CSV 导入能力不变。

## 方案选择

### 方案 1：扩展 `import_stock_price_history`

优点：
- Tool 数量不增加。

缺点：
- 会把“文件导入”和“HTTP 同步”两种完全不同的错误模型、参数模型、测试模型混在一起。
- 现有 CSV 用户路径会被迫共享网络分支，风险更高。

### 方案 2：新增并列 Tool `sync_stock_price_history`

优点：
- 保持 `CSV` 与 `HTTP` 两条入口职责分离。
- 复用同一 `StockHistoryStore`，但不污染现有导入合同。
- 最符合用户“继续沿既有架构渐进推进，非必要不重构”的要求。

缺点：
- catalog / dispatcher / 测试会多一个 Tool 暴露点。

## 最终设计

采用方案 2。

### 输入合同

新增 `SyncStockPriceHistoryRequest`：

- `symbol`: 目标证券代码，支持如 `600519.SH` / `000001.SZ`
- `start_date`: 起始日期，`YYYY-MM-DD`
- `end_date`: 结束日期，`YYYY-MM-DD`
- `adjustment`: 复权方式，第一版只支持 `qfq`
- `providers`: 数据源优先级数组，默认 `["tencent", "sina"]`
- `persist_source`: 可选；落库时写入的 source 前缀，默认按命中的 provider 自动生成

### 输出合同

新增 `SyncStockPriceHistoryResult`：

- `symbol`
- `provider_used`
- `imported_row_count`
- `database_path`
- `table_name`
- `date_range`

### 数据流

1. dispatcher 解析 HTTP 同步请求。
2. 业务层按 `providers` 顺序尝试 HTTP 拉取。
3. 将各 provider 返回统一映射成 `StockHistoryRow`。
4. 复用现有 `StockHistoryStore::import_rows()` 落到 `stock_price_history`。
5. 返回稳定 JSON 回执。

### 边界和降级

- 腾讯失败时，如果请求里还有新浪，则继续尝试新浪。
- 若全部 provider 都失败，返回聚合中文错误。
- 日期区间为空、symbol 不可识别、返回体缺字段时直接报错，不写脏数据。
- 第一版只做“日线 + 前复权 + A 股”，不提前扩到分钟线、港股、美股。

### 为什么这次不直接做技术指标 API

这轮已经确认：

- 腾讯 / 新浪当前可作为“原始行情源”。
- 但没有足够稳定、可公开依赖的“免费现成技术指标 API”证据。

因此系统主线继续保持：

`原始 OHLCV -> Rust 自算指标 -> technical_consultation_basic`

这样后续补 `MFI / CCI / Williams %R` 时不会被外部技术指标口径锁死。

