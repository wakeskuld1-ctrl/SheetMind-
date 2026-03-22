# 分析建模层 Skill 真实走查复测（table_ref 桥接完成后）

## 测试说明

- 测试日期：2026-03-22
- 真实文件：`D:\Excel测试\新疆客户\2026文旅体台账.xlsx`
- 复测目标：验证“表处理层确认态 -> 持久化 `table_ref` -> 分析建模层复用”是否真正打通
- 原始工件目录：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2`
- 工件清单：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\manifest.json`

## 这轮和上一轮的核心差异

- 上一轮：`analyze_table`、`stat_summary`、`linear_regression`、`logistic_regression`、`cluster_kmeans` 都会重新推断 schema，因此卡在 `needs_confirmation`。
- 这一轮：先用 `apply_header_schema` 产出持久化 `table_ref`，后续分析建模 Tool 直接消费 `table_ref`，不再重复做 schema 推断。

## 关键结论

- `apply_header_schema` 现在会返回可跨请求复用的 `table_ref`。
- 基于真实 `旅责险` sheet，这一轮已经验证：
  - `stat_summary(table_ref)` -> `ok`
  - `analyze_table(table_ref)` -> `ok`
  - `cluster_kmeans(table_ref)` -> `ok`
  - `linear_regression(table_ref)` -> `ok`
  - `logistic_regression(table_ref)` -> `error`，但错误原因已经从“schema 未确认”变成“目标列只有一个类别”
- 这说明桥接缺口已经关闭；当前逻辑回归没跑通，不再是桥接层问题，而是数据本身不满足建模前提。

## 场景 1：真实确认态升级为 `table_ref`

### 我问了什么

先确认 `旅责险` 的表头，然后把确认结果作为后续分析建模的输入。

### Skill 怎么回

先调用 `apply_header_schema` 完成确认，并返回可继续复用的 `table_ref`，后面所有分析建模 Tool 都优先使用这个句柄。

### Tool 请求 / 响应

- 请求：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_02_apply_header_schema_lvz_request.json`
- 响应：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_02_apply_header_schema_lvz_response.json`

### 结论

确认成功，返回了可复用的 `table_ref`，后续真实复测都基于这个句柄进行。

## 场景 2：`stat_summary(table_ref)` 复测

### Tool 请求 / 响应

- 请求：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_03_stat_summary_by_table_ref_request.json`
- 响应：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_03_stat_summary_by_table_ref_response.json`

### 结论

返回 `ok`，已经不再是 `needs_confirmation`。这证明桥接入口已经真正复用确认态。

## 场景 3：`analyze_table(table_ref)` 复测

### Tool 请求 / 响应

- 请求：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_04_analyze_by_table_ref_request.json`
- 响应：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_04_analyze_by_table_ref_response.json`

### 结论

返回 `ok`，说明观察诊断层已经不再重复卡在 schema 门禁。

## 场景 4：`cluster_kmeans(table_ref)` 复测

### Tool 请求 / 响应

- 请求：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_05_cluster_by_table_ref_request.json`
- 响应：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_05_cluster_by_table_ref_response.json`

### 结论

返回 `ok`，说明聚类链路已经不再因为重复推断 schema 被挡住。

## 场景 5：`linear_regression(table_ref)` 复测

### Tool 请求 / 响应

- 请求：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_06_linear_by_table_ref_request.json`
- 响应：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_06_linear_by_table_ref_response.json`

### 结论

返回 `ok`，而且真实使用了 683 行有效样本。线性回归已经证明桥接层可以支撑真实建模执行。

## 场景 6：`logistic_regression(table_ref)` 复测

### Tool 请求 / 响应

- 请求：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_07_logistic_by_table_ref_request.json`
- 响应：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file-round2\step_07_logistic_by_table_ref_response.json`

### 结论

返回 `error`，但错误信息是：`目标列 \`column_10\` 只有一个类别，暂时不能做逻辑回归`。这说明当前阻断已经转移到真实数据前提校验，而不是 schema 复用问题。

## 最终结论

- 方案 C 已落地：表处理层确认结果现在会被持久化为 `table_ref`，并能跨请求复用到分析建模层。
- 上一轮最核心的桥接缺口已经被关闭。
- 真实文件上的分析建模链路已经从“统一卡在 `needs_confirmation`”变成“能执行就执行，不能执行时给出更贴近数据前提的错误”。
- 下一轮如果继续做，优先建议：
  - 让 Skill 在确认后优先切换到 `table_ref` 路由
  - 补逻辑回归目标列选择/正类选择的真实引导
  - 单独修 `咨询费` 的错误可读性问题
