# Capacity Assessment From Inventory Design

**目标**

在现有 `ssh_inventory` 与 `capacity_assessment` 之间补一条正式桥接链路，让用户即使没有完整 Excel 指标，也能通过受限 SSH 盘点拿到部署事实，再直接产出容量评估结果。

**背景**

当前工程已经具备：
- 高层容量评估 Tool：`capacity_assessment`
- 受限只读 SSH 盘点 Tool：`ssh_inventory`
- 三层证据模型：`scenario_profile`、`deployment_profile`、`inventory_evidence`

现在缺的是“采集结果自动映射并直接分析”的收口层。用户不希望每次都手工把 SSH 结果转写成 `inventory_evidence`。

**方案比较**

1. 在 `capacity_assessment` 内部直接接收 `ssh_inventory_request`
- 优点：调用入口少
- 缺点：容量计算 Tool 同时承担采集职责，SRP 变差，安全边界也更难看清

2. 新增桥接 Tool：`capacity_assessment_from_inventory`
- 优点：职责清晰，`ssh_inventory` 只负责采集，`capacity_assessment` 只负责分析，桥接 Tool 只负责编排与映射
- 缺点：会多一个高层 Tool

3. 只提供一个本地转换函数，不暴露新 Tool
- 优点：代码改动面小
- 缺点：CLI 和外部调用方还是要自己串流程，用户体验没有改善

**推荐方案**

采用方案 2，新增 `capacity_assessment_from_inventory`。

理由：
- 最符合单一职责
- 便于后续扩展成“SSH -> 容量评估 -> 报表输出”的稳定主链
- 不会把远程采集逻辑硬塞进容量核心实现

**架构**

新增一个高层桥接 Tool：

`capacity_assessment_from_inventory`

执行流程：
1. 解析桥接请求
2. 调用 `ssh_inventory`
3. 把返回结果映射为 `InventoryEvidence`
4. 合并用户传入的 `scenario_profile`、`deployment_profile`、可选 Excel 数据源
5. 调用现有 `capacity_assessment`
6. 返回容量结论，并附带映射明细

职责边界：
- `ssh_inventory`：只做安全白名单采集
- `capacity_assessment`：只做容量推断
- `capacity_assessment_from_inventory`：只做编排和证据映射

**输入设计**

桥接 Tool 请求包含：
- `inventory_request`
  直接复用 `SshInventoryRequest`
- `service_matchers`
  用于从 `ps -ef` 结果中识别服务实例
- `scenario_profile`
- `deployment_profile`
- 可选 Excel 数据源字段
  直接复用 `capacity_assessment` 现有入参，便于“部分指标 + SSH 事实”混合分析

**实例识别设计**

为了避免误判，实例识别采用保守规则：
- 用户显式提供 `service_matchers` 时才尝试从 `ps -ef` 中提取实例数
- 首版支持：
  - `process_contains`
  - `command_contains`
  - `regex`（如当前实现复杂度偏高，可先不做）
- 匹配到的进程数映射为 `inventory_evidence.discovered_instance_count`
- 未提供 matcher 或匹配不到时，不猜实例数，只保留主机事实

这样可以避免把 SSH 工具做成“自动猜业务”的黑盒。

**映射规则**

首版只做稳定、可解释的映射：
- `hostname` -> 单机主机事实
- 实际采集主机数 -> `host_count`
- `nproc` -> `host_cpu_cores`
- `free -m` -> `host_memory_mb`
- 进程匹配结果 -> `discovered_instance_count`
- `source` 固定为 `ssh_inventory`

如果桥接 Tool 后续支持多主机输入，再扩展为聚合模式。

**输出设计**

桥接 Tool 返回：
- 原有 `capacity_assessment` 的完整输出
- `inventory_mapping`
  展示哪些 SSH 字段被映射成了哪些 `inventory_evidence`
- `mapping_confidence`
  展示哪些事实是直接确定的，哪些字段缺失或未匹配

这样用户能追溯“结论从哪里来”，方便交付和审阅。

**错误处理**

需要保证三类稳定行为：
- SSH 调用失败：直接返回稳定错误，不生成伪造容量结论
- SSH 成功但证据不足：允许降级到 `partial` 或 `guidance_only`
- 进程匹配失败：不报错，不瞎猜，只把实例数留空

**测试策略**

按 TDD 增加 CLI 级失败测试，覆盖：
- tool catalog 能发现 `capacity_assessment_from_inventory`
- SSH 盘点结果能映射成 `inventory_evidence`
- 显式 matcher 能驱动 `discovered_instance_count`
- 没有 matcher 时不会乱填实例数
- 盘点失败时返回稳定错误
- “SSH 事实 + 场景画像 + 部署画像 + 部分 Excel 指标”能产出 `partial` 或更高等级结论

**本轮边界**

本轮不做：
- 任意 SSH 命令扩展
- Windows 远程盘点
- jump host
- K8s API / systemd / docker 专项识别
- Excel 报表输出

本轮只打通“SSH 采集 -> 自动映射 -> 容量评估”主链。
