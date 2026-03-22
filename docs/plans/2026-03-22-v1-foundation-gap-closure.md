# V1 Foundation Gap Closure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the remaining V1 productization gaps by building the missing foundation for reusable result datasets, rule-based derived fields, customer-facing analytical outputs, and report export.

**Architecture:** Follow the approved foundation-first order. First make intermediate DataFrame results durable as `result_ref`, then add a reusable rule engine for derived columns and labels, then package those capabilities into customer-facing analysis tools, and finally add binary-first report export.

**Tech Stack:** Rust, Polars, calamine, rusqlite, serde_json, Markdown

---

### Task 1: Build `result_ref` runtime foundation

**Files:**
- Create: `D:/Rust/Excel_Skill/src/frame/result_ref_store.rs`
- Modify: `D:/Rust/Excel_Skill/src/frame/mod.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_registry.rs`

**Step 1: Write the failing test**

Add a test that persists a mixed-type DataFrame and loads it back by `result_ref`.

**Step 2: Run test to verify it fails**

Run: `cargo test stored_result_dataset_round_trips_through_disk --test integration_registry -v`
Expected: FAIL because `result_ref_store` does not exist yet.

**Step 3: Write minimal implementation**

Implement:
- `PersistedResultDataset`
- `PersistedResultColumn`
- `ResultRefStore`
- `from_dataframe()` / `to_dataframe()`
- `create_result_ref()`

**Step 4: Run focused regression**

Run: `cargo test --test integration_registry -v`
Expected: PASS with the new round-trip test included.

### Task 2: Add generic result loading into the frame layer

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/frame/loader.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**

Add CLI tests that save an operation result and reuse `result_ref` in the next request.

**Step 2: Run test to verify it fails**

Run the targeted CLI tests and confirm dispatcher cannot yet resolve `result_ref`.

**Step 3: Write minimal implementation**

Support loading from:
- `table_ref`
- `result_ref`
- direct `path + sheet`

Add a small resolver so table tools can consume a unified source handle.

**Step 4: Run focused regression**

Run the relevant CLI tests and confirm chained operations now work across requests.

### Task 3: Add derived-column and labeling primitives

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/derive.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write failing tests**

Cover:
- conditional label generation
- bucket/range classification
- score accumulation across rules

**Step 2: Run tests to verify they fail**

Run targeted tests and confirm the new tool is not registered yet.

**Step 3: Write minimal implementation**

Expose one reusable primitive tool, likely `derive_columns`, that supports:
- `case_when`
- `bucketize`
- `score_rules`
- optional human-readable reason output

**Step 4: Run focused regression**

Run the derive tests and ensure existing table-processing tests still pass.

### Task 4: Add the first customer-facing analysis tool

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/customer_product_match.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write failing test**

Model the smallest realistic customer-product match output with:
- customer id
- historical primary product
- suggested product
- priority label
- recommendation reason

**Step 2: Run test to verify it fails**

Run the targeted test and confirm the tool is missing.

**Step 3: Write minimal implementation**

Build the first topical tool on top of the generic foundation from Tasks 1-3.

**Step 4: Run focused regression**

Confirm the new tool passes with deterministic output.

### Task 5: Add binary-first report export

**Files:**
- Modify: `D:/Rust/Excel_Skill/Cargo.toml`
- Create: `D:/Rust/Excel_Skill/src/ops/export.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Docs: `D:/Rust/Excel_Skill/README.md`

**Step 1: Write failing tests**

Cover:
- export result dataset to CSV
- export result dataset to XLSX
- reject unsupported output target with clear error

**Step 2: Run tests to verify they fail**

Run targeted export tests and confirm no export tool exists yet.

**Step 3: Write minimal implementation**

Add binary-first export capabilities without introducing Python runtime dependency.

**Step 4: Run focused regression**

Run export tests and a wider regression subset.
