# Lemon License Design

## 背景

当前 `excel_skill` 已经是 `Rust / exe / SQLite` 主链，但还没有一层正式的本地授权门禁。
用户这轮已经明确选择：

- 不做自建云后台
- 直接接 Lemon Squeezy License API
- 本地缓存授权状态
- 尽量限制“一份授权很多人共用”
- 不追求重 DRM，不为了防破解引入复杂体系

因此，这次要补的是一套“够用、稳定、可维护”的授权闭环，而不是大而全的账号系统。

## 目标

新增一套沿现有 Rust 主链运行的本地授权能力，至少覆盖：

- `license_activate`
- `license_status`
- `license_deactivate`
- 受保护工具的启动前门禁
- 本地 SQLite 缓存授权状态
- 缓存过期后的在线校验

## 非目标

- 不接自建 SaaS / Web 后台
- 不引入 Python 授权层
- 不做账号登录系统
- 不做强对抗 DRM
- 不改现有 Excel Tool 主链语义

## 方案对比

### 方案 A：Lemon 直连激活 + 本地 SQLite 缓存 + 定期校验

做法：

- EXE 直接调用 Lemon Squeezy License API
- 首次激活时拿到 `instance_id`
- 本地把授权状态落进 SQLite
- 后续优先读取本地缓存
- 缓存超过阈值后再走在线 `validate`

优点：

- 最符合当前 `Rust / exe / SQLite` 主线
- 不需要你维护云端授权服务
- 能限制普通层面的多人共用
- 用户离线时仍有可用空间

缺点：

- 需要本地处理配置、缓存过期和失败降级

### 方案 B：每次启动都在线校验

优点：

- 逻辑最简单
- 授权一致性最强

缺点：

- 用户离线就不可用
- 和单机 EXE 的交付体验冲突

### 方案 C：只做首次激活，不再校验

优点：

- 实现最轻
- 对用户最省事

缺点：

- 对“买一个一堆人用”的约束太弱

本轮采用 **方案 A**。

## 架构落点

沿现有仓库分层落地：

- `src/main.rs`
  - 负责授权门禁和公开授权工具入口
- `src/license/`
  - 放 Lemon API 客户端、授权服务、授权类型
- `src/runtime/license_store.rs`
  - 负责本地 SQLite 持久化
- `src/tools/catalog.rs`
  - 暴露授权工具名

这样做的原因是：

- 授权属于 EXE 级别的运行门禁
- 不应该把现有 Excel / 分析 Tool 全部塞进授权模块
- 也不需要新开第二套业务分发体系

## 数据流

### 激活

1. 用户调用 `license_activate`
2. EXE 读取 Lemon 配置
3. 调用 `POST /v1/licenses/activate`
4. 校验返回的 `store_id / product_id / variant_id`
5. 把 `license_key / instance_id / validated_at / meta` 写入本地 SQLite
6. 返回激活状态

### 正常执行受保护工具

1. EXE 收到工具请求
2. 如果是公开工具，则直接放行
3. 如果是受保护工具，则先读本地授权状态
4. 若缓存仍新鲜，直接放行
5. 若缓存过期，则调用 `POST /v1/licenses/validate`
6. 校验成功后刷新本地缓存，再放行
7. 校验失败则拦截

### 反激活

1. 用户调用 `license_deactivate`
2. EXE 用本地 `license_key + instance_id` 调用 `POST /v1/licenses/deactivate`
3. 成功后清空本地授权记录

## 配置约定

本轮采用环境变量配置，避免把商店信息硬编码进仓库：

- `EXCEL_SKILL_LICENSE_ENFORCED`
- `EXCEL_SKILL_LEMON_BASE_URL`
- `EXCEL_SKILL_LEMON_STORE_ID`
- `EXCEL_SKILL_LEMON_PRODUCT_ID`
- `EXCEL_SKILL_LEMON_VARIANT_ID`
- `EXCEL_SKILL_LICENSE_VALIDATE_MAX_AGE_HOURS`
- `EXCEL_SKILL_LICENSE_OFFLINE_GRACE_HOURS`

默认策略：

- 开发环境默认不强制拦截
- 只有显式启用授权门禁时才保护工具

这样可以避免把你当前开发流和现有测试一次性全部锁死。

## 错误处理

- Lemon 配置缺失：返回明确中文错误
- 未激活：提示先调用 `license_activate`
- 校验失败：拒绝执行受保护工具
- 网络失败但仍处于离线宽限期：允许继续执行
- 网络失败且超出宽限期：拒绝执行

## 测试重点

至少覆盖：

1. `tool_catalog` 能发现授权工具
2. 启用门禁后，未授权时受保护工具被拦截
3. 激活成功后，状态写入本地 SQLite，受保护工具可执行
4. 缓存过期后会触发在线 `validate`
5. 反激活后，本地记录被清空并重新拦截
