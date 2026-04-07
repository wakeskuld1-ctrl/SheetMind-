# Security Post Meeting Conclusion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为证券审批主链补齐正式会后结论对象、前后配对治理关系与独立 Tool，并让 package revision / verify / Skill 展示层都能稳定消费该对象。

**Architecture:** 新增 `SecurityPostMeetingConclusion` 正式对象与独立 Tool `security_record_post_meeting_conclusion`。该 Tool 回读现有 package 与 approval brief，生成会后结论并落盘，再驱动 package revision，把会后结论正式挂入 `artifact_manifest` 与 `object_graph`。底层合同继续使用稳定英文枚举值，Skill / CLI 再做中文展示翻译。

**Tech Stack:** Rust、Serde JSON、CLI 集成测试、现有 `security_decision_package / security_decision_package_revision / security_decision_verify_package` 主链、文件落盘工件。

---

### Task 1: 锁定会后结论对象与独立 Tool 合同

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_package_revision_cli.rs`
- Create: `D:\Rust\Excel_Skill\tests\security_post_meeting_conclusion_cli.rs`

**Step 1: Write the failing test**

- 新增 `tool_catalog_includes_security_record_post_meeting_conclusion`
- 新增 happy path：
  - 先生成初始 package
  - 再调用 `security_record_post_meeting_conclusion`
  - 断言返回 `post_meeting_conclusion_path`
  - 断言 `final_disposition == "approve"` 等正式合同值
  - 断言新 package 版本递增

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 因 Tool 不存在、对象未定义或输出字段不存在而失败

### Task 2: 最小实现正式会后结论对象

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_post_meeting_conclusion.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`

**Step 1: Write minimal implementation**

- 新增 `SecurityPostMeetingConclusion`
- 新增配套输入结构：
  - `SecurityPostMeetingConclusionBuildInput`
- 新增子结构：
  - `SecurityPostMeetingGovernanceBinding`
  - `SecurityPostMeetingBriefPairing`
- 提供最小 builder：
  - `build_security_post_meeting_conclusion(...)`

**Step 2: Run targeted tests**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 测试从“对象缺失”推进到“Tool 或 package 接线缺失”

### Task 3: 接入独立 Tool 与落盘入口

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`
- Create: `D:\Rust\Excel_Skill\src\ops\security_record_post_meeting_conclusion.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_post_meeting_conclusion_cli.rs`

**Step 1: Write the failing test**

- 在 happy path 中断言：
  - Tool 返回会后结论对象
  - 会后结论文件独立落盘
  - Tool 响应中包含新的 `decision_package_path`

**Step 2: Verify RED**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 因 dispatcher / stock ops 未接线或 Tool 未落盘而失败

**Step 3: Write minimal implementation**

- 新增 `security_record_post_meeting_conclusion` 请求与响应
- 回读旧 package 与 approval brief
- 生成并落盘 `post_meeting_conclusion`

**Step 4: Verify GREEN**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- Tool catalog 与最小 happy path 通过

### Task 4: 扩展 package 与 revision 主链

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package_revision.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_record_post_meeting_conclusion.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_post_meeting_conclusion_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_package_revision_cli.rs`

**Step 1: Write the failing test**

- 断言 revision 后：
  - `artifact_manifest` 包含 `post_meeting_conclusion`
  - `object_graph.post_meeting_conclusion_ref`
  - `object_graph.post_meeting_conclusion_path`
- 断言 `approval_brief` 与会后结论能形成配对引用

**Step 2: Verify RED**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
cargo test --test security_decision_package_revision_cli -- --nocapture
```

Expected:

- 因 object graph / artifact manifest 尚未扩展而失败

**Step 3: Write minimal implementation**

- 给 package object graph 增加：
  - `post_meeting_conclusion_ref`
  - `post_meeting_conclusion_path`
- 给 package artifact manifest 增加：
  - `post_meeting_conclusion`
- revision 逻辑支持把会后结论纳入新版本 package

**Step 4: Verify GREEN**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
cargo test --test security_decision_package_revision_cli -- --nocapture
```

Expected:

- 会后结论对象能进入 revision 后 package

### Task 5: 扩展 verify 校验正式会后结论链

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_verify_package.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_post_meeting_conclusion_cli.rs`

**Step 1: Write the failing test**

- 新增 verify happy path 断言：
  - `post_meeting_conclusion_binding_consistent == true`
  - `post_meeting_conclusion_brief_paired == true`
  - `post_meeting_conclusion_complete == true`
- 新增失败用例：
  - 篡改 `source_brief_ref`
  - 篡改 `source_package_path`
  - 篡改 `final_disposition`

**Step 2: Verify RED**

Run:

```powershell
cargo test --test security_decision_verify_package_cli -- --nocapture
```

Expected:

- 因 verify 尚未识别会后结论对象与绑定关系而失败

**Step 3: Write minimal implementation**

- 在 governance checks 中新增三项校验：
  - `post_meeting_conclusion_binding_consistent`
  - `post_meeting_conclusion_brief_paired`
  - `post_meeting_conclusion_complete`

**Step 4: Verify GREEN**

Run:

```powershell
cargo test --test security_decision_verify_package_cli -- --nocapture
```

Expected:

- verify 包含会后结论场景的主链通过

### Task 6: 补充 approval_brief 的成对治理关系

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_brief.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_record_post_meeting_conclusion.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_post_meeting_conclusion_cli.rs`

**Step 1: Write the failing test**

- 断言 `approval_brief` 中出现轻量配对块：
  - `has_post_meeting_conclusion`
  - `latest_conclusion_ref`
  - `pairing_status`

**Step 2: Verify RED**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 因 brief 尚无配对字段而失败

**Step 3: Write minimal implementation**

- 给 `approval_brief` 增加轻量配对信息
- 仅记录正式引用，不把会后全文塞回 brief

**Step 4: Verify GREEN**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 会前 / 会后配对关系通过

### Task 7: 增加展示翻译层

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_conclusion_i18n.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_post_meeting_conclusion_cli.rs`

**Step 1: Write the failing test**

- 针对 helper 或 CLI 响应中的展示字段断言：
  - `approve -> 通过`
  - `reject -> 驳回`
  - `needs_more_evidence -> 需要补证据`
  - `approve_with_override -> 带保留通过`

**Step 2: Verify RED**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 因展示映射未实现而失败

**Step 3: Write minimal implementation**

- 只新增映射层
- 不把翻译文本写入正式落盘 JSON

**Step 4: Verify GREEN**

Run:

```powershell
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 展示层断言通过

### Task 8: 跑证券治理主链回归

**Files:**
- Test: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_verify_package_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_decision_package_revision_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\security_post_meeting_conclusion_cli.rs`

**Step 1: Run regression**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli -- --nocapture
cargo test --test security_decision_verify_package_cli -- --nocapture
cargo test --test security_decision_package_revision_cli -- --nocapture
cargo test --test security_post_meeting_conclusion_cli -- --nocapture
```

Expected:

- 四条证券治理主链全部通过

### Task 9: 完成任务日志

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Append task journal entry**

- 记录会后结论对象
- 记录独立 Tool
- 记录前后配对治理关系
- 记录展示翻译层边界
- 记录验证命令

**Step 2: Final verification**

Run:

```powershell
git status --short --branch
```

Expected:

- 仅包含本轮 Task 3 相关代码、测试与文档变更
