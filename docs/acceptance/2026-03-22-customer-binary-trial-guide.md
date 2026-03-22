# 客户侧纯二进制试用说明与真实文件验收流程

## 目标

这份文档给两类人直接用：

- 你自己做 V1 试用验收
- 后续给业务侧或实施侧做“如何试”的最小说明

核心原则只有一条：

- 客户侧正式运行只接受 Rust 二进制
- 不要求用户安装 Python
- 不依赖 pandas、Jupyter、Node 或其他脚本环境

## 当前可试用形态

V1 当前最稳定的试用方式是：

1. 底层计算与读写：`D:/Rust/Excel_Skill/target/debug/excel_skill.exe`
2. 问答入口：总入口 Skill
   - `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/SKILL.md`
3. 子层 Skill：
   - 表处理层：`D:/Rust/Excel_Skill/skills/table-processing-v1/SKILL.md`
   - 分析建模层：`D:/Rust/Excel_Skill/skills/analysis-modeling-v1/SKILL.md`
   - 决策助手层：`D:/Rust/Excel_Skill/skills/decision-assistant-v1/SKILL.md`

如果只是做二进制冒烟验证，直接运行：

```powershell
cd D:\Rust\Excel_Skill
cargo run --quiet
```

预期结果：

- 输出一段 JSON
- `tool_catalog` 中包含 `open_workbook`、`apply_header_schema`、`analyze_table`、`decision_assistant` 等 Tool

## 客户侧最小使用前提

客户机器只需要：

- Excel 文件本身
- Rust 编译产物 `excel_skill.exe`
- 一个承载 Skill 问答的宿主界面

客户不需要：

- Python
- pandas
- Jupyter
- Node
- 手写脚本

## 推荐试用入口

推荐你实际试用时，不要直接从决策助手层开始，而是从总入口 Skill 开始。

推荐入口：

- `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/SKILL.md`

原因：

- 它会先读会话状态
- 会根据当前阶段决定是去表处理、分析建模还是决策助手
- 更接近后续真正的“一体化问答入口”

## 真实文件

本轮真实文件验收统一使用：

- `D:/Excel测试/新疆客户/2026文旅体台账.xlsx`

如果宿主界面或底层库对中文路径有兼容问题，当前 V1 的恢复口径是：

1. 先判断是否为 Windows 路径格式问题
2. 再判断是否为中文路径兼容问题
3. 必要时复制一份只用于分析的 ASCII 临时副本
4. 最后才判断文件当前不可读

## 推荐试用对话流程

### 第 1 步：从总入口打开文件

你可以先这样问：

```text
请先打开 D:\Excel测试\新疆客户\2026文旅体台账.xlsx，先看看这份台账应该从哪里开始。
```

预期行为：

- 总入口先做状态摘要
- 如果没有确认态，切到表处理层
- 优先先读工作簿和 Sheet，而不是直接分析建模

### 第 2 步：确认目标 Sheet 与表头

建议继续这样问：

```text
先确认最像收入台账的 Sheet，并把确认后的结果作为后续复用的确认态。
```

预期行为：

- 先 `open_workbook`
- 再 `apply_header_schema`
- 产出可复用的 `table_ref`

### 第 3 步：进入分析建模层

建议继续这样问：

```text
基于刚才确认好的表，先给我一个统计摘要，再判断这张表现在适不适合继续分析。
```

预期行为：

- 优先复用 `table_ref`
- 调 `stat_summary` / `analyze_table`
- 不再重复追问 `path + sheet`

### 第 4 步：进入决策助手层

建议继续这样问：

```text
不要给我太多技术细节，直接告诉我这张表下一步最该先做什么。
```

预期行为：

- 调 `decision_assistant`
- 先解释阻塞风险，再解释优先动作
- 不直接替用户开始建模

### 第 5 步：如果你想继续验证模型入口

建议继续这样问：

```text
如果现在可以继续，就告诉我更适合先做聚类、线性回归，还是先补数据质量处理。
```

预期行为：

- 仍然优先解释 readiness
- 如果继续建模，切到分析建模层
- 继续复用同一个 `table_ref`

## 本轮真实文件验收证据

现成证据已经落在：

- 验收文档：`D:/Rust/Excel_Skill/docs/acceptance/2026-03-22-v1-final-e2e-real-file.md`
- 工件清单：`D:/Rust/Excel_Skill/docs/acceptance/artifacts/2026-03-22-v1-final-e2e-real-file/manifest.json`

关键工件包括：

- `step_02_apply_header_schema_lvz_response.json`
- `step_03_analyze_by_table_ref_response.json`
- `step_04_stat_summary_by_table_ref_response.json`
- `step_05_decision_assistant_by_table_ref_response.json`

这几份文件已经证明：

- 表处理确认态能落成 `table_ref`
- 分析建模层能复用 `table_ref`
- 决策助手层也能复用 `table_ref`

## 验收通过标准

满足下面 6 条，就可以认为“客户侧最小纯二进制体验”已经可验：

1. 不要求安装 Python
2. 运行主链路不要求脚本环境
3. 能打开真实 Excel
4. 能确认目标 Sheet 和表头
5. 能产出并复用 `table_ref`
6. 能从分析建模层自然切到决策助手层

## 当前 V1 边界

- 中文路径恢复目前仍以规则层和宿主能力降级为主，不是所有环境都自动修复
- 逻辑回归仍需要用户提供真正的二分类目标列
- 决策助手只给“下一步建议”，不替用户做最终经营判断
- Skill 本身不做计算，所有计算仍在 Rust Tool 层

## 给业务侧的一句话版本

如果你后面要给业务侧一个最短解释，可以直接说：

“你只需要这份 Excel 和一个本地可执行文件，不需要装 Python。系统会先帮你确认表，再继续做分析和下一步建议。”
