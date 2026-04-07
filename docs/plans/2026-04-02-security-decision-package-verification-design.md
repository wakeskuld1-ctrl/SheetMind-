# Security Decision Package Verification Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为正式 `decision package` 增加可执行的校验入口，使证券审批包从“可生成”升级到“可核验”。

**Architecture:** 保持现有 `security_decision_submit_approval` 与 `SecurityDecisionPackageDocument` 不变，本次新增独立 `security_decision_verify_package` Tool。该 Tool 负责读取 package 文档、逐项校验 `artifact_manifest`、重算工件哈希、验证治理绑定一致性，并在存在 `approval_brief_signature` 时验证 detached signature，最终返回并落盘正式 `verification_report`。

**Tech Stack:** Rust、Cargo、serde/serde_json、sha2、hmac、文件型 runtime 持久化、CLI 合同测试

---

## 1. 背景

当前证券审批链已经具备：

- `security_decision_submit_approval`
- `decision_package`
- `approval_brief_signature`

这意味着我们已经能生成正式审批包，但还存在一个明显缺口：

- package 还不能被系统主动校验
- manifest 哈希还没有真正回读文件复核
- approval brief 的 detached signature 还没有验证入口
- 后续审批、归档、复盘仍缺一个可信的检查点

因此，当前 package 更像“可打包”而不是“可核验”。

## 2. 设计目标

本次 `P0-5` 只解决以下问题：

1. 新增 `security_decision_verify_package` Tool
2. 读取 package 文档并校验 manifest 路径存在性
3. 重算 artifact 哈希并与 manifest 对比
4. 在有 `approval_brief_signature` 时验证 detached signature
5. 校验 `decision_ref / approval_ref / evidence_hash / governance_hash` 一致性
6. 输出并落盘正式 `verification_report`

本次明确不做：

1. 不做 package 自身签名
2. 不做人工审批后 package 自动版本化
3. 不做 package 导出为 PDF / XLSX
4. 不做多算法签名框架，只先支持当前 HMAC-SHA256

## 3. 三种做法对比

### 方案 A：轻量存在性校验

优点：

- 开发最快
- 风险最低

缺点：

- 只能确认“文件在不在”
- 不能证明 package 内容可信

### 方案 B：正式治理校验

优点：

- 能校验路径、哈希、治理绑定和 detached signature
- 最符合当前审批治理主线
- 可直接作为后续审计锚点

缺点：

- 比存在性校验多一层报告对象和哈希重算

### 方案 C：版本化 package 校验器

优点：

- 更完整

缺点：

- scope 过大
- 会把本轮拖进投中/审批后链路

本次采用：**方案 B**

## 4. 新增 Tool

建议新增：

- `security_decision_verify_package`

请求字段建议：

- `package_path`
- `approval_brief_signing_key_secret`
- `approval_brief_signing_key_secret_env`
- `write_report`

说明：

- `package_path` 为必填
- 签名 secret 在 package 存在签名工件时才需要
- `write_report` 默认 `true`

## 5. 正式校验报告对象

建议新增：

- `SecurityDecisionPackageVerificationReport`

建议字段：

- `report_id`
- `contract_version`
- `generated_at`
- `package_path`
- `package_id`
- `decision_ref`
- `approval_ref`
- `package_valid`
- `artifact_checks`
- `hash_checks`
- `signature_checks`
- `governance_checks`
- `issues`
- `recommended_action`

## 6. 嵌套对象设计

### 6.1 `artifact_checks`

每项建议包含：

- `artifact_role`
- `path`
- `required`
- `present`
- `exists_on_disk`
- `status`
- `message`

### 6.2 `hash_checks`

每项建议包含：

- `artifact_role`
- `manifest_sha256`
- `actual_sha256`
- `matched`

### 6.3 `signature_checks`

建议包含：

- `artifact_role`
- `algorithm`
- `key_id`
- `payload_sha256_matched`
- `signature_valid`
- `message`

### 6.4 `governance_checks`

建议包含：

- `decision_ref_matched`
- `approval_ref_matched`
- `evidence_hash_matched`
- `governance_hash_matched`

## 7. 落盘策略

默认新增目录：

- `decision_packages_verification`

建议落盘文件：

- `decision_packages_verification/<decision_id>.verification.json`

## 8. 校验顺序

建议按以下顺序执行：

1. 读取 package 文档
2. 遍历 manifest 检查工件是否存在
3. 对存在工件重算 sha256
4. 如果存在 `approval_brief_signature`，则读取：
   - `approval_brief`
   - `approval_brief_signature`
   并验证：
   - payload sha256
   - HMAC signature
5. 校验 package 中治理绑定与正文工件的一致性
6. 汇总 issues 与 `recommended_action`

## 9. 推荐结果规则

建议规则：

- 所有 required 工件存在、哈希一致、签名通过、治理绑定一致
  - `package_valid = true`
  - `recommended_action = proceed_with_review`
- 缺少 required 工件、哈希不一致、签名失败或治理绑定不一致
  - `package_valid = false`
  - `recommended_action = quarantine_and_rebuild`
- 仅缺少 optional 工件
  - `package_valid = true`
  - `recommended_action = review_with_warning`

## 10. 测试策略

必须坚持 TDD。

优先新增：

1. tool catalog 能发现 `security_decision_verify_package`
2. 针对已签名 package，verify 能返回 `package_valid = true`
3. `verification_report_path` 会返回并落盘
4. 篡改 `approval_brief` 后再次 verify 会失败
5. 未签名 package 在 optional signature 缺失时仍可通过，但要有 warning / present=false

## 11. 风险

### 11.1 secret 依赖风险

HMAC 验签依赖 secret，如果调用方不给 secret，就无法完成签名验证。  
因此报告里要明确区分“没有签名工件”和“有签名工件但缺 secret”。

### 11.2 哈希来源差异风险

manifest 哈希来自提交时 payload，而 verify 是回读文件重算。  
这是设计目标，但如果中间文件格式化策略变了，可能带来误报，因此 verify 必须按原始字节重算。

### 11.3 package 演进风险

后续 package 新增工件时，如果 verify 没同步更新，可能会出现清单漏校验。  
因此 manifest 驱动校验，不应把角色列表写死得过重。

## 12. 完成定义

满足以下条件即可认为 `P0-5` 完成：

1. `security_decision_verify_package` Tool 可调用
2. 正式 `verification_report` 合同落地
3. 已签名 package 可验证通过
4. 篡改场景会验证失败
5. CLI 合同测试通过

## 13. 一句话总结

`P0-5` 的目标不是再多一个工具，而是让正式 `decision package` 第一次具备“系统可核验”的治理能力。
