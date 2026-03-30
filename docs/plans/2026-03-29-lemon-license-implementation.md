# Lemon License Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `excel_skill` 增加 Lemon Squeezy 直连授权、SQLite 本地缓存与 EXE 门禁。

**Architecture:** 在 `main.rs` 增加授权门禁，只放行公开授权工具；新增 `license` 模块负责 Lemon API 交互与授权判定；新增 `runtime/license_store.rs` 负责把授权状态持久化到当前 runtime SQLite。

**Tech Stack:** Rust, rusqlite, serde, chrono, ureq, cargo test

---

### Task 1: 先锁定授权对外合同

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\license_cli.rs`

**Step 1: Write the failing test**

覆盖：

- `tool_catalog` 包含 `license_activate / license_status / license_deactivate`
- 开启授权门禁时，未授权工具会被拦截
- 激活后允许执行受保护工具
- 过期缓存会触发 `validate`
- 反激活后重新拦截

**Step 2: Run test to verify it fails**

```powershell
cargo test --test license_cli -- --nocapture
```

预期：

- 因为工具未注册或门禁不存在而失败

### Task 2: 落本地授权缓存

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\runtime\license_store.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\runtime\mod.rs`

**Step 1: Write minimal implementation**

- 建 `license_state` 单例表
- 支持读、写、清空授权状态
- 复用当前 runtime SQLite 路径

**Step 2: Run targeted test**

```powershell
cargo test --test license_cli -- --nocapture
```

### Task 3: 接 Lemon API

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\license\mod.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\license\types.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\license\client.rs`
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\license\service.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\lib.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\Cargo.toml`

**Step 1: Write minimal implementation**

- `activate / validate / deactivate`
- 配置读取
- 返回统一中文错误
- 校验 `store_id / product_id / variant_id`

**Step 2: Run targeted test**

```powershell
cargo test --test license_cli -- --nocapture
```

### Task 4: 接到 EXE 门禁

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\main.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`

**Step 1: Write minimal implementation**

- 公开放行：
  - `tool_catalog`
  - `license_activate`
  - `license_status`
  - `license_deactivate`
- 其他工具在执行前先做授权判定
- 缓存过期时自动 `validate`

**Step 2: Run targeted test**

```powershell
cargo test --test license_cli -- --nocapture
```

### Task 5: 回归验证与任务记录

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\.trae\CHANGELOG_TASK.md`

**Step 1: Run focused verification**

```powershell
cargo test --test license_cli -- --nocapture
```

**Step 2: Run regression**

```powershell
cargo test --test integration_tool_contract -- --nocapture
cargo test --test integration_cli_json -- --nocapture
```

**Step 3: 追加任务日志**

- 只追加，不改历史
