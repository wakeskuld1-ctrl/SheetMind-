# Capacity Assessment Scenario V2 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade the existing `capacity_assessment` tool into a scenario-aware capacity engine that accepts business/deployment context and can optionally pull readonly host facts through a restricted SSH inventory tool.

**Architecture:** Keep `capacity_assessment` as the high-level decision tool, but expand its request model from pure metric columns to three evidence layers: scenario profile, deployment profile, and metric samples. Add a separate `ssh_inventory` tool that never performs arbitrary command execution; it only runs a fixed whitelist of readonly Linux commands and returns normalized inventory facts that `capacity_assessment` can consume.

**Tech Stack:** Rust, Polars, existing SheetMind tool dispatcher, JSON CLI contract, cargo test, optional `ssh2` crate

---

### Task 1: Lock scenario-aware capacity behavior with failing CLI tests

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\capacity_assessment_cli.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\common\mod.rs`

**Step 1: Write failing tests for scenario/deployment inputs**

Add tests that verify:
- scenario profile upgrades guidance output when service criticality and SLA are known
- deployment profile influences the recommended instance count when N+1 / active-active / scaling step exists
- partial evidence can still produce quantified output when deployment facts fill the missing columns

**Step 2: Run the focused test file to verify it fails**

Run: `cargo test --test capacity_assessment_cli -- --nocapture`
Expected: FAIL because the new request fields are not used yet.

### Task 2: Lock SSH safety boundaries with failing tests

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\ssh_inventory_cli.rs`

**Step 1: Write failing tests for the readonly whitelist**

Add tests that verify:
- `ssh_inventory` appears in the tool catalog
- requests with non-whitelisted commands are rejected
- requests with shell separators like `;`, `&&`, `|`, redirection, or custom command strings are rejected
- readonly presets such as `process_snapshot` and `host_snapshot` are accepted
- SSH collection failure returns a stable tool error instead of corrupting the capacity path

**Step 2: Run the focused SSH test file to verify it fails**

Run: `cargo test --test ssh_inventory_cli -- --nocapture`
Expected: FAIL because the tool does not exist yet.

### Task 3: Implement the scenario/deployment-aware capacity model

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\capacity_assessment.rs`

**Step 1: Extend request/output structs**

Add:
- `ScenarioProfile`
- `DeploymentProfile`
- `InventoryEvidence`
- `ServiceRiskProfile`

**Step 2: Implement layered evidence logic**

Rules:
- metric evidence still drives resource pressure
- scenario profile adjusts reserve and risk posture
- deployment profile supplies instance/spec/failover/scaling-step facts when Excel metrics are incomplete
- evidence should degrade in order: quantified -> partial -> guidance_only

**Step 3: Run capacity tests**

Run: `cargo test --test capacity_assessment_cli -- --nocapture`
Expected: fewer failures, but catalog/dispatcher changes may still be pending.

### Task 4: Implement restricted SSH inventory core

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\ssh_inventory.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\Cargo.toml`

**Step 1: Add readonly preset and whitelist model**

Support preset-only execution such as:
- `process_snapshot` -> `ps -ef`
- `host_snapshot` -> `top -b -n 1`, `nproc`, `free -m`, `hostname`
- `procfs_snapshot` -> `cat /proc/cpuinfo`, `cat /proc/meminfo`

**Step 2: Reject unsafe inputs**

Reject:
- custom shell fragments
- separators `;`, `&&`, `||`, `|`, `<`, `>`
- write/destructive commands

**Step 3: Normalize collected facts**

Return:
- host name
- cpu core count
- memory total
- process list raw excerpt
- top snapshot excerpt

**Step 4: Run SSH tests**

Run: `cargo test --test ssh_inventory_cli -- --nocapture`
Expected: PASS

### Task 5: Expose SSH inventory through the tool layer

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\analysis_ops.rs`

**Step 1: Register `ssh_inventory`**

**Step 2: Add dispatcher path**

**Step 3: Re-run focused tests**

Run:
- `cargo test --test ssh_inventory_cli -- --nocapture`
- `cargo test --test capacity_assessment_cli -- --nocapture`
Expected: PASS

### Task 6: Verify and document

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run verification**

Run: `cargo test`
Expected: PASS or only unchanged pre-existing warnings.

**Step 2: Update tracking**

Record:
- scenario-aware capacity model landed
- SSH tool is readonly/whitelist-only
- any dependency or environment constraints

