# Binary Delivery Docs Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把普通用户入口收口为预编译二进制使用方式，并将 `cargo` 明确降级为开发者构建路径，同时同步 Skill 的对外约束。

**Architecture:** 通过 README 首页重排、单独二进制交付说明文档，以及 4 个 Skill 的运行时约束补充，统一“普通用户不需要 Rust / cargo / Python”的表达。再用一个小型校验脚本做文档回归验证。

**Tech Stack:** Markdown, PowerShell, Python 3（仅研发校验脚本，不属于客户运行时）

---

### Task 1: 建立文档回归校验脚本

**Files:**
- Create: `D:\Rust\Excel_Skill\scripts\check_binary_delivery_docs.py`
- Test: `D:\Rust\Excel_Skill\scripts\check_binary_delivery_docs.py`

**Step 1: Write the failing test**

实现一个最小校验脚本，先断言：
- `README.md` 包含“普通用户”与“预编译二进制”入口语义
- `README.md` 不再把 `cargo run --quiet` 放在普通用户主入口描述下
- 4 个 Skill 包含“不要要求普通用户安装 Rust / cargo”语义

**Step 2: Run test to verify it fails**

Run: `python scripts/check_binary_delivery_docs.py`
Expected: FAIL，指出当前 README/Skill 语义不满足。

**Step 3: Write minimal implementation**

实现简单文本断言与失败信息。

**Step 4: Run test to verify it fails correctly**

Run: `python scripts/check_binary_delivery_docs.py`
Expected: FAIL，且失败点对应当前旧文案。

### Task 2: 修改 README 首页入口

**Files:**
- Modify: `D:\Rust\Excel_Skill\README.md`
- Test: `D:\Rust\Excel_Skill\scripts\check_binary_delivery_docs.py`

**Step 1: Write the failing test**

沿用 Task 1 的失败结果，不新增实现前置。

**Step 2: Run test to verify it fails**

Run: `python scripts/check_binary_delivery_docs.py`
Expected: FAIL。

**Step 3: Write minimal implementation**

调整 README：
- 增加“普通用户不需要 Rust / cargo / Python”
- 把 Quick Start 改成“普通用户 / 开发者”分流
- 将 `cargo` 明确标为开发者构建

**Step 4: Run test to verify it passes**

Run: `python scripts/check_binary_delivery_docs.py`
Expected: README 相关断言通过，Skill 相关可能仍失败。

### Task 3: 新增二进制交付说明文档

**Files:**
- Create: `D:\Rust\Excel_Skill\docs\acceptance\2026-03-23-binary-delivery-guide.md`
- Modify: `D:\Rust\Excel_Skill\README.md`

**Step 1: Write the failing test**

沿用校验脚本，可追加 README 必须引用该文档。

**Step 2: Run test to verify it fails**

Run: `python scripts/check_binary_delivery_docs.py`
Expected: FAIL，如果 README 尚未引用新文档。

**Step 3: Write minimal implementation**

新增双语交付说明，并在 README 中链接。

**Step 4: Run test to verify it passes**

Run: `python scripts/check_binary_delivery_docs.py`
Expected: README 与新文档相关断言通过。

### Task 4: 收口 4 个 Skill 的普通用户运行约束

**Files:**
- Modify: `D:\Rust\Excel_Skill\skills\excel-orchestrator-v1\SKILL.md`
- Modify: `D:\Rust\Excel_Skill\skills\table-processing-v1\SKILL.md`
- Modify: `D:\Rust\Excel_Skill\skills\analysis-modeling-v1\SKILL.md`
- Modify: `D:\Rust\Excel_Skill\skills\decision-assistant-v1\SKILL.md`
- Test: `D:\Rust\Excel_Skill\scripts\check_binary_delivery_docs.py`

**Step 1: Write the failing test**

沿用校验脚本中对 4 个 Skill 的断言。

**Step 2: Run test to verify it fails**

Run: `python scripts/check_binary_delivery_docs.py`
Expected: FAIL。

**Step 3: Write minimal implementation**

在各 Skill 的运行时约束补上：
- 不要要求普通用户安装 Rust 或 cargo
- 不要把 `cargo run` / `cargo build` 当成普通用户试用步骤

**Step 4: Run test to verify it passes**

Run: `python scripts/check_binary_delivery_docs.py`
Expected: PASS。

### Task 5: 最终验证、日志、提交推送

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run verification**

Run:
- `python scripts/check_binary_delivery_docs.py`
- `cargo build --release -v`

Expected: 全部通过。

**Step 2: Update task journal**

按固定模板追加到 `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`。

**Step 3: Commit**

Run:
- `git status --short`
- `git add README.md docs/acceptance/2026-03-23-binary-delivery-guide.md docs/plans/2026-03-23-binary-delivery-docs-design.md docs/plans/2026-03-23-binary-delivery-docs.md scripts/check_binary_delivery_docs.py skills/excel-orchestrator-v1/SKILL.md skills/table-processing-v1/SKILL.md skills/analysis-modeling-v1/SKILL.md skills/decision-assistant-v1/SKILL.md .trae/CHANGELOG_TASK.md`
- `git commit -m "docs: clarify binary-only customer delivery"`

**Step 4: Push**

Run: `git push origin HEAD`
Expected: 推送成功。
