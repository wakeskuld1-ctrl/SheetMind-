# 证券 PM 助手 Skill 设计

## 背景

当前仓库已经具备证券投前治理链的核心 Tool 与下层 Skill：

- `security-analysis-v1`
- `security-decision-workbench-v1`
- `security_decision_submit_approval`
- `security_decision_verify_package`
- `security_decision_package_revision`

但真实使用入口仍然偏“分段式”：

- 做分析时走分析 Skill
- 做投决时走投决会 Skill
- 做治理时再手工切到提交、校验、修订 Tool

这会导致用户在自然语言问答里很难一次说清楚“我要分析、我要买卖判断、我要提交审批、我要校验包、我要生成审批后新版本包”分别该走哪条链路。

<!-- 2026-04-02 CST: 新增本设计文档，原因是用户明确要求按“证券 PM 助手”方案把问答入口补齐，而不是继续停留在分散 Skill 与 Tool；目的是先把职责边界、路由规则、验证场景写清楚，再创建 Skill 本体。 -->

## 目标

新增一个上层问答 Skill：

- 名称：`security-pm-assistant-v1`
- 定位：证券 PM / 投研经理 / 投决秘书的统一自然语言入口

它的职责不是替代底层 Tool 或下层 Skill，而是统一编排：

1. 研究分析
2. 投决会判断
3. 审批提交
4. package 校验
5. 审批后 package 修订

## 方案对比

### 方案 A：扩写现有 `security-decision-workbench-v1`

优点：

- 变更最少
- 可以最快接住“买不买”类问题

缺点：

- 会把一个原本只负责“投决会”的 Skill 扩成总入口
- 职责边界变差，不符合 SRP
- 后续再接投中、投后时会越来越臃肿

### 方案 B：新增“证券投决治理入口 Skill”

优点：

- 路由职责清楚
- 比直接扩写现有 Skill 更容易维护

缺点：

- 仍然偏流程编排器
- 对真实用户来说不够像“PM 助手”

### 方案 C：新增“证券 PM 助手 Skill”

优点：

- 最符合真实问答入口
- 自然承接研究、投决、审批、治理
- 后续扩展投中、投后最顺

缺点：

- 必须严格约束边界，避免变成大杂烩

## 选型

采用方案 C。

原因：

- 用户明确表示要“问答”的统一入口，而不是再记一组分散 Tool 名称
- 当前底层能力已经足够，缺的是上层编排
- 证券 PM 助手更符合当前产品形态，也更利于后续继续扩到投中、投后

## 职责边界

`security-pm-assistant-v1` 只负责三类事情：

1. 识别用户当前所处阶段
2. 选择正确的下层 Skill 或 Tool
3. 用 PM / 投决视角组织最终输出

它不负责：

- 直接替代底层 Tool 计算
- 篡改下层 Tool 的结构化结论
- 越过投决会直接给买卖建议
- 越过审批链直接伪装成“已审批”

## 路由阶段

### 1. 研究阶段

触发特征：

- “分析一下”
- “看一下这只股票”
- “大盘怎么样”
- “行业怎么走”

路由到：

- `security-analysis-v1`

### 2. 投决阶段

触发特征：

- “值不值得买”
- “怎么配仓”
- “买不买”
- “做不做这笔”

路由到：

- `security-decision-workbench-v1`

### 3. 审批提交阶段

触发特征：

- “提交审批”
- “上会”
- “形成审批包”
- “提交投决”

路由到：

- `security_decision_submit_approval`

### 4. package 校验阶段

触发特征：

- “校验这个包”
- “检查审批包”
- “看这个 package 有没有问题”

路由到：

- `security_decision_verify_package`

### 5. package 修订阶段

触发特征：

- “审批通过了，生成新版包”
- “审批有动作了，更新 package”
- “基于新审批事件生成 v2”

路由到：

- `security_decision_package_revision`

## 上下文复用规则

当上下文里已经存在以下对象时，Skill 必须优先复用，而不是重复新建：

- `decision_ref`
- `approval_ref`
- `package_path`
- `verification_report_path`

规则：

1. 有 `package_path` 且用户问“校验/修订”，优先走 package 治理链
2. 有 `decision_ref / approval_ref` 且用户问“提交审批”，要先提醒已存在审批对象，避免重复提交
3. 只有分析结论但没有治理对象时，用户一旦要求“提交”“审批”“上会”，才进入治理链

## 输出规范

输出必须先说明当前处于哪个阶段，再给行动结果。

建议格式：

1. 当前阶段
2. 使用的 Skill / Tool
3. 结果摘要
4. 下一步建议

## 常见错误

### 错误 1：把研究结论伪装成投决结论

正确做法：

- 研究问题只走分析链
- 买卖判断要走投决会链

### 错误 2：用户只问分析，却直接提交审批

正确做法：

- 只有用户明确要求治理动作时，才进入审批提交

### 错误 3：已有 package_path，却重复新建审批包

正确做法：

- 优先复用现有治理对象

## 验证场景

### 场景 1：研究问答

输入：

- “分析一下宁德时代今天的情况”

预期：

- 路由到 `security-analysis-v1`

### 场景 2：投决问答

输入：

- “这周能不能买招商银行，怎么配仓”

预期：

- 路由到 `security-decision-workbench-v1`

### 场景 3：审批提交问答

输入：

- “把刚才这次判断提交审批”

预期：

- 路由到 `security_decision_submit_approval`

### 场景 4：校验问答

输入：

- “检查这个审批包有没有问题”

预期：

- 路由到 `security_decision_verify_package`

### 场景 5：修订问答

输入：

- “审批通过了，基于最新事件生成 v2 package”

预期：

- 路由到 `security_decision_package_revision`
