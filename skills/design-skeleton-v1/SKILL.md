---
name: design-skeleton-v1
description: Use when starting a new feature, major refactor, or architecture change that must begin from a design skeleton and later be checked against graphify implementation maps.
---

# /design-skeleton-v1

Turn a feature idea into a reusable design skeleton first, then compare that target design with the current implementation graph before and after coding.

## Usage

```text
/design-skeleton-v1
/design-skeleton-v1 <feature-name>
/design-skeleton-v1 <feature-name> --gap-audit
/design-skeleton-v1 <feature-name> --graph E:\TradingAgents\TradingAgents\graphify-out\src-map-2026-04-13\graph.json
```

## What this skill is for

This skill is the design-side companion to `graphify`.

- `graphify` answers: the codebase looks like this now.
- `design-skeleton-v1` answers: the feature should look like this before we write code.
- `foundation_design_gap_audit` answers: how far the implementation still is from that target.

Use it for:

- 新功能开发前先收口层级、模块、接口、方法
- 重大重构前先确认边界，不直接跳进代码
- 实现完成后做“设计 vs 成品”差距回看
- 交接时把目标骨架和现状图一起沉淀

## What You Must Do When Invoked

If the user did not provide a feature name, infer one from the task. Do not block on naming.

Follow these steps in order.

### Step 1 - Confirm the design scope

Collect or infer the minimum design skeleton:

- `feature_name`
- `objective`
- `success_criteria`
- `layers`
- `modules`
- `interfaces`
- `methods`
- `test_scenarios`

If information is missing, make the smallest reasonable assumption and mark it in the design notes.

### Step 2 - Generate the target design skeleton

Call `foundation_design_skeleton` with the full skeleton object.

You must first check the JSON result:

- `summary`
- `warnings`
- `layer_count`
- `module_count`
- `interface_count`
- `method_count`
- `test_scenario_count`

If present, `visuals.*_mermaid` is only auxiliary output for quick reading.

Do not skip this step and go directly to coding.

### Step 3 - Review the warnings before coding

If warnings exist, surface them clearly:

- empty `success_criteria`
- empty `test_scenarios`
- module missing `source_files`

Warnings do not block by default, but they must be acknowledged before implementation.

### Step 4 - Use graphify as the current-state map

Before writing code, check the latest `src-map-*` graph or run the repo graph update flow if needed.

Use `graphify` for:

- 当前真实上下游
- 模块归属
- 已有入口和可复用对象
- 是否会误入业务化支线

Do not treat the graph as a perfect runtime truth. It is a navigation map, not a substitute for source reading.

### Step 5 - Run the gap audit when needed

When the user asks for design-vs-implementation comparison, or after code changes, call:

- `foundation_design_gap_audit`

Inputs:

- the same design skeleton
- optional `graph_path`

If `graph_path` is omitted, the tool should use the latest `graphify-out/src-map-*/graph.json`.

### Step 6 - Report the result in this order

1. design summary
2. key JSON boundary summary
3. warnings
4. gap summary
5. missing modules / interfaces / methods

Only add Mermaid snippets when they help human review. Do not treat them as the primary contract.

## Required Rules

- No new feature or major refactor should start without a design skeleton.
- Do not replace `graphify`; use it as the implementation-side map.
- Do not collapse generic foundation work into stock business modules.
- After implementation, rerun graph update and compare again.

## Output Shape

When summarizing for the user, keep the output compact and stable:

- `目标骨架`
- `核心边界图`
- `主要 warning`
- `实现差距`
- `下一步`

## Common Mistakes

- 只写文档方案，不生成正式 skeleton Tool 输出
- 只看 graphify 现状图，不补目标设计图
- 把 module 层比较误当成 method 层已完成
- 开发后不更新 `src-map-*`，导致账实不一致
