# Capacity Assessment From Inventory Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a high-level `capacity_assessment_from_inventory` Tool that runs restricted SSH inventory, maps the result into `inventory_evidence`, and then invokes the existing capacity engine.

**Architecture:** Keep `ssh_inventory` and `capacity_assessment` independent, then introduce one bridge layer that orchestrates both. The bridge owns request parsing, service-process matching, inventory-to-evidence mapping, and response enrichment with mapping details.

**Tech Stack:** Rust, serde, existing SheetMind tool dispatcher, cargo test, JSON CLI contract

---

### Task 1: Lock the new bridge Tool contract with failing CLI tests

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\capacity_assessment_from_inventory_cli.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\common\mod.rs`

**Step 1: Write the failing tool-catalog test**

Add a test that asserts `capacity_assessment_from_inventory` appears in the tool catalog.

**Step 2: Write the failing mapping test**

Add a test that sends `validate_only: true` style inventory input through a mockable bridge request or direct normalized inventory payload and asserts:
- `inventory_evidence.source == "ssh_inventory"`
- `host_cpu_cores`
- `host_memory_mb`
- `host_count`

**Step 3: Write the failing process-matcher test**

Add a test that feeds a `ps -ef` sample with an explicit matcher and asserts `discovered_instance_count` is populated.

**Step 4: Write the failing “no matcher, no guessing” test**

Add a test that proves instance count remains empty when process matchers are absent.

**Step 5: Run the focused test file to verify RED**

Run: `cargo test --test capacity_assessment_from_inventory_cli -- --nocapture`
Expected: FAIL because the bridge Tool does not exist yet.

### Task 2: Implement the bridge request/result and inventory mapping core

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\capacity_assessment_from_inventory.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\mod.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\capacity_assessment.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\ssh_inventory.rs`

**Step 1: Define request structs**

Add:
- `CapacityAssessmentFromInventoryRequest`
- `ServiceMatchers`
- `InventoryMappingSummary`

The request should carry:
- `inventory_request: SshInventoryRequest`
- optional `scenario_profile`
- optional `deployment_profile`
- optional passthrough fields for existing capacity inputs

**Step 2: Write the minimal mapping helpers**

Implement helpers to:
- map `SshInventoryResult.inventory.cpu_core_count` -> `host_cpu_cores`
- map `memory_total_mb` -> `host_memory_mb`
- default `host_count` to `1`
- set `source` to `ssh_inventory`

**Step 3: Implement minimal process matching**

Support:
- `process_contains: Vec<String>`
- `command_contains: Vec<String>`

Count matched `ps -ef` lines and map to `discovered_instance_count`.

**Step 4: Re-run the focused bridge tests**

Run: `cargo test --test capacity_assessment_from_inventory_cli -- --nocapture`
Expected: still FAIL on dispatcher/catalog exposure or end-to-end analysis path, but core mapping tests should move forward.

### Task 3: Connect the bridge Tool to the capacity engine

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\capacity_assessment_from_inventory.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\capacity_assessment.rs`

**Step 1: Build a derived `CapacityAssessmentRequest`**

Translate the bridge request into an internal `CapacityAssessmentRequest` by:
- forwarding scenario/deployment inputs
- injecting derived `inventory_evidence`
- forwarding existing capacity-analysis arguments

**Step 2: Call the existing capacity engine**

Use the current capacity-analysis path so the new Tool reuses:
- evidence grading
- risk posture logic
- recommendation logic

**Step 3: Enrich the response**

Return:
- the capacity result
- `inventory_mapping`
- `mapping_confidence`

**Step 4: Run the focused bridge tests**

Run: `cargo test --test capacity_assessment_from_inventory_cli -- --nocapture`
Expected: logic tests should PASS before tool-layer registration.

### Task 4: Expose the bridge Tool through the tool layer

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\analysis_ops.rs`

**Step 1: Register `capacity_assessment_from_inventory`**

Add the tool name to the catalog.

**Step 2: Add dispatcher parsing**

Parse `CapacityAssessmentFromInventoryRequest` in `analysis_ops.rs` and route to the new bridge op.

**Step 3: Re-run the focused bridge tests**

Run: `cargo test --test capacity_assessment_from_inventory_cli -- --nocapture`
Expected: PASS

### Task 5: Add regression coverage for failure and partial-evidence behavior

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\capacity_assessment_from_inventory_cli.rs`

**Step 1: Add a stable SSH failure regression test**

Assert the bridge Tool returns a stable error when the underlying SSH collection fails.

**Step 2: Add a partial-analysis regression test**

Assert that:
- partial host facts
- plus scenario/deployment profile
- without full workbook metrics

still yield `partial` or `guidance_only` rather than crashing.

**Step 3: Run the focused bridge tests**

Run: `cargo test --test capacity_assessment_from_inventory_cli -- --nocapture`
Expected: PASS

### Task 6: Full verification and documentation

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run verification**

Run:
- `cargo test --test capacity_assessment_from_inventory_cli -- --nocapture`
- `cargo test --test ssh_inventory_cli -- --nocapture`
- `cargo test --test capacity_assessment_cli -- --nocapture`
- `cargo test`

Expected: PASS or only unchanged pre-existing warnings.

**Step 2: Update tracking**

Record:
- bridge Tool landed
- SSH results now auto-map into `inventory_evidence`
- instance matching is matcher-driven and conservative
- any remaining environment or parsing limitations
