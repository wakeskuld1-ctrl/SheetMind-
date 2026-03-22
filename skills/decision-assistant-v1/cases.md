# 决策助手 Skill V1 场景映射

## 场景 1：用户只想知道下一步该做什么

- 用户说法：这张表现在最该先处理什么？
- 路由：`decision_assistant`
- 说明：先给优先动作，不直接切模型

## 场景 2：用户已经有上游确认态

- 用户说法：前面的表头已经确认过了，你直接告诉我下一步
- 路由：`decision_assistant(table_ref)`
- 说明：优先复用 `table_ref`，不重复追问 `path + sheet`

## 场景 3：存在明显阻塞风险

- 用户说法：我能不能直接开始建模？
- 路由：`decision_assistant`
- 说明：先解释阻塞风险，再说明为什么不建议直接建模

## 场景 4：没有明显阻塞风险

- 用户说法：如果可以继续，建议我先做哪个分析
- 路由：`decision_assistant`
- 说明：解释 ready 的下一步 Tool，但不要替用户直接开跑

## 场景 5：用户开始问模型参数

- 用户说法：那你直接帮我做逻辑回归吧
- 路由：转回 `analysis-modeling-v1`
- 说明：本 Skill 不负责补 `target`、`features`、`positive_label`
