# AI Handoff Manual

## 1. Purpose

This document preserves project-level AI constraints, architectural direction, rejected paths, and maintenance rules for future AI sessions and GitHub handoff. It exists to prevent repeated drift, repeated re-alignment, and repeated token waste.

## 2. Current Project Goal

The current goal is not to expand application-side features. The current goal is to build a business-agnostic foundation capability that can support multiple domains through a common semantic and navigation base.

The foundation target is:

- ontology-driven knowledge navigation
- controlled knowledge roaming
- retrieval as evidence execution
- optional model enhancement through providers

## 3. Architectural Baseline

The baseline architecture is:

1. Ontology layer
2. Knowledge roaming layer
3. Retrieval layer
4. Optional model enhancement layer

The intended flow is:

`question -> ontology positioning -> capability discovery -> knowledge roaming -> candidate scope convergence -> retrieval execution -> evidence assembly`

This means retrieval is not the system origin. Retrieval is only one stage inside a larger semantic navigation workflow.

## 4. Boundary Between Foundation and Application

Foundation responsibilities include:

- ontology concepts, capabilities, relation types, and constraints
- knowledge nodes, edges, and evidence references
- knowledge roaming policies and navigation rules
- metadata filtering
- keyword retrieval
- vector retrieval
- hybrid ranking
- citation assembly
- optional provider-based model enhancement

Application-side responsibilities include:

- GUI pages
- interactive presentation flows
- application-side orchestration for user experience
- page-level display logic
- chat UI behavior

The current roadmap is explicitly limited to foundation work. Do not move foundation scope into application-side code.

## 5. Why GUI Is Not the Current Priority

GUI-based intelligent Q&A is currently deprioritized for practical reasons:

- it consumes a large amount of tokens during design and implementation
- output quality is unstable relative to cost
- it pulls effort away from the core foundation
- it does not materially improve the current highest-priority capabilities

The preferred direction is CLI-first interaction. Future application layers may consume the same foundation later, but they must not drive the current architecture.

## 6. CLI-First Direction

The current interaction preference is CLI-first, Tool-first, Skill-first, and API-friendly.

This means:

- foundation outputs should remain structured
- evidence and paths should remain inspectable
- future intelligent Q&A should first land through CLI-style interaction
- GUI should be treated as a later consumer, not a current design target

## 7. Ontology-Driven Knowledge Navigation Baseline

The foundation must not be modeled as a domain-specific retrieval engine. It must be modeled as a business-agnostic ontology-driven knowledge navigation base.

Core principles:

- ontology provides semantic structure
- roaming provides upstream and downstream completion
- retrieval finds concrete evidence inside the constrained candidate scope

This architecture is intended to support multiple domains, including but not limited to contracts, invoices, reports, records, disclosures, and other structured or semi-structured business knowledge.

## 8. Roles of Metadata, Ontology, Roaming, Retrieval

### Metadata

Metadata is necessary, but insufficient. It is used for:

- filtering
- partitioning
- source restriction
- time restriction
- type restriction

Metadata must not be treated as the core semantic mechanism.

### Ontology-lite

Ontology-lite is required because metadata alone cannot provide:

- concept normalization
- alias handling
- upstream and downstream semantic relationships
- capability-centered routing
- relation-aware expansion

Ontology-lite is the semantic backbone of the current direction.

### Knowledge Roaming

Knowledge roaming is required because the system must do more than direct lookup. It must expand within a constrained semantic graph to complete surrounding context, especially for upstream, downstream, and adjacent-impact understanding.

Roaming must be controlled by:

- allowed relation types
- direction mode
- depth limits
- capability-specific constraints

### Retrieval

Retrieval is an executor. It works inside the candidate space constrained by ontology positioning and roaming.

Retrieval may include:

- metadata filter
- keyword retrieval
- vector retrieval
- hybrid ranking

But retrieval must not replace ontology or roaming.

## 9. Role of Vectorization and Model Providers

### Vectorization

Vectorization is useful, but it is not the main association mechanism.

It should be used for:

- semantic similarity
- hidden textual association
- candidate enrichment inside a constrained scope

It should not replace:

- ontology positioning
- relation-aware roaming
- foundation semantics

### Model Providers

Cloud or local models may be introduced only as provider-based enhancers.

Allowed roles include:

- query normalization
- ontology alias expansion
- embedding generation
- optional rerank
- answer synthesis

Models must not be hard dependencies of the core foundation. The system must remain runnable in disabled mode.

Key memory point:

- the model is an enhancer, not the foundation

## 10. Rejected or Deprecated Directions

The following directions have already been discussed and rejected for the current phase:

- GUI-first intelligent Q&A
- mixing application flow logic into foundation work
- binding the foundation to a single business domain
- reducing the architecture to metadata plus vector retrieval only
- treating financial reports, disclosures, contracts, invoices, or any single domain object as the foundation model
- making cloud LLMs a hard dependency of the main semantic navigation flow

## 11. Implementation Constraints

- Foundation code must remain business-agnostic.
- Domain-specific objects must only appear as upper-layer adapters or examples.
- The architecture must prefer ontology + roaming + retrieval over domain-specific shortcut logic.
- Retrieval accuracy, retrieval efficiency, and vectorization efficiency remain high priorities.
- Foundation work must avoid accidental spread into application-side files.

## 12. GitHub Handoff Rules

These rules must be followed whenever the project is prepared for GitHub upload or handoff:

1. Review whether the current architecture baseline has changed.
2. Review whether the hard boundaries have changed.
3. Review whether any new rejected direction has emerged.
4. Update this handoff manual if project-level guidance changed.
5. Update the short baseline file if the practical focus changed.
6. Do not push major direction changes without corresponding handoff updates.

## 13. Maintenance Rules

- If a major architectural correction happens, update this document.
- If a new hard boundary is agreed, update both documents.
- If priorities change, update the short baseline file.
- Do not rely on chat history alone as a durable memory source.
- Durable project constraints must be written into project documents.

## 14. Recommended Next Steps

The current recommended next steps remain:

1. Define ontology-lite structures and constraints.
2. Define business-agnostic knowledge node and edge structures.
3. Define roaming plans and controlled navigation behavior.
4. Add retrieval only as the evidence execution stage.
5. Add provider-based model enhancement only after the foundation path is stable.

## 15. Mandatory Read Order for Future AI Sessions

Any future AI session should follow this read order before doing architecture work:

1. [project-baseline.md](/D:/Rust/Excel_Skill/docs/ai-memory/project-baseline.md)
2. This handoff manual
3. Existing project dynamic records such as `task_plan.md`, `progress.md`, and `findings.md`

If a future session conflicts with these documents, it should stop and re-align before continuing.

## 16. Current Foundation Delivery Status (2026-04-08)

The foundation navigation kernel has completed Tasks 1 through 8 in the current implementation sequence:

1. foundation module entry wiring
2. ontology schema
3. ontology store
4. knowledge record and knowledge graph store
5. capability router
6. roaming engine
7. retrieval engine
8. evidence assembler

The current confirmed implementation order remains:

`ontology-lite -> roaming -> retrieval -> evidence assembly`

This ordering is now a maintenance rule, not a temporary suggestion.

Do not re-open completed Tasks 1-8 for speculative restructuring when a future AI session starts.

## 17. Module Scope Guardrails For This Line

The current `src/ops/foundation/` line is only for the business-agnostic navigation kernel.

It currently includes:

- ontology structures and relation lookup
- knowledge node / edge / evidence data structures
- question-to-concept routing
- candidate-scope roaming
- scoped retrieval inside candidate concepts
- structured evidence assembly

It must not absorb:

- security decision workflow logic
- stock analysis workflow logic
- GUI interaction flow logic
- dispatcher-side application orchestration

If the repo is dirty, stage and commit only the files belonging to this foundation line.

## 18. Next Step After Task 8

The next implementation target is Task 9: `navigation_pipeline_integration`.

The expected direction is:

- connect route, roaming, retrieval, and evidence assembly into one minimal in-memory pipeline
- keep the pipeline inside foundation scope
- avoid introducing Tool dispatcher, GUI flow, or domain-specific shortcuts

Before moving on, re-read:

1. [execution-notes-2026-04-08-evidence-assembler.md](/D:/Rust/Excel_Skill/docs/execution-notes-2026-04-08-evidence-assembler.md)
2. [evidence_assembler.rs](/D:/Rust/Excel_Skill/src/ops/foundation/evidence_assembler.rs)
3. [evidence_assembler_unit.rs](/D:/Rust/Excel_Skill/tests/evidence_assembler_unit.rs)

## 19. Parallel Security Governance Track Status (2026-04-08)

<!-- 2026-04-08 CST: 调整交接口径，原因是本节记录的是 Task 11 前的最小 Green 快照，而第 20 节已经写入 Task 11 后的新状态；目的：让后续 AI 明确第 19 节是历史背景，第 20 节才是当前合同状态。 -->

There is an active parallel security decision workflow line in this repository. It is not part of the foundation navigation kernel, but future AI sessions must not ignore it when working on the stock governance path.

This section is a historical snapshot before Task 11 package binding landed.

For the current contract state, always read Section 20 after this section.

The following was the confirmed pre-Task-11 status on branch `codex/foundation-navigation-kernel`:

- Task 3 minimum Green is complete for post-meeting conclusion recording
- a formal `SecurityPostMeetingConclusion` object now exists
- a formal `security_record_post_meeting_conclusion` Tool now exists
- the Tool is wired into stock catalog and dispatcher entry points
- the Tool can persist a post-meeting conclusion and then reuse package revision to produce the next package version

Key files for this line are:

- [security_post_meeting_conclusion.rs](/D:/Rust/Excel_Skill/src/ops/security_post_meeting_conclusion.rs)
- [security_record_post_meeting_conclusion.rs](/D:/Rust/Excel_Skill/src/ops/security_record_post_meeting_conclusion.rs)
- [security_post_meeting_conclusion_cli.rs](/D:/Rust/Excel_Skill/tests/security_post_meeting_conclusion_cli.rs)
- [2026-04-08-security-post-meeting-conclusion-design.md](/D:/Rust/Excel_Skill/docs/plans/2026-04-08-security-post-meeting-conclusion-design.md)
- [2026-04-08-security-post-meeting-conclusion-plan.md](/D:/Rust/Excel_Skill/docs/plans/2026-04-08-security-post-meeting-conclusion-plan.md)
- [execution-notes-2026-04-08-security-post-meeting-conclusion.md](/D:/Rust/Excel_Skill/docs/execution-notes-2026-04-08-security-post-meeting-conclusion.md)

Verified commands for this slice:

- `cargo test --test security_post_meeting_conclusion_cli -- --nocapture`
- `cargo test --test security_decision_submit_approval_cli -- --nocapture`
- `cargo test --test security_decision_verify_package_cli -- --nocapture`
- `cargo test --test security_decision_package_revision_cli -- --nocapture`

Historical limitation before Task 11:

- Task 3 is not fully closed yet
- the revised package does not yet formally carry `post_meeting_conclusion` inside `object_graph` or `artifact_manifest`
- verify has not yet been extended to enforce post-meeting conclusion binding and integrity

If a future AI continues the security governance line, read the security-specific handoff first:

1. [交接摘要_证券分析_给后续AI.md](/D:/Rust/Excel_Skill/docs/交接摘要_证券分析_给后续AI.md)
2. [execution-notes-2026-04-08-security-post-meeting-conclusion.md](/D:/Rust/Excel_Skill/docs/execution-notes-2026-04-08-security-post-meeting-conclusion.md)
3. [security_record_post_meeting_conclusion.rs](/D:/Rust/Excel_Skill/src/ops/security_record_post_meeting_conclusion.rs)
4. [security_post_meeting_conclusion_cli.rs](/D:/Rust/Excel_Skill/tests/security_post_meeting_conclusion_cli.rs)

## 20. Security Governance Update After Task 11 (2026-04-08)

This section supersedes the outdated limitation notes in Section 19 and should be treated as the current contract baseline for this line.

The security governance line has now moved beyond the earlier "minimum Green only" state.

Current confirmed status:

- `post_meeting_conclusion` is formally registered in `artifact_manifest`
- `post_meeting_conclusion_ref/path` are formally registered in `decision_package.object_graph`
- `security_decision_submit_approval` now produces a stable `object_graph` baseline from v1 package onward
- `security_decision_package_revision` now carries forward existing object graph bindings and can attach a new post-meeting conclusion
- `security_record_post_meeting_conclusion` now records the standalone conclusion and immediately binds it into the revised package
- `security_decision_verify_package` now validates:
  - post-meeting conclusion binding consistency
  - approval brief pairing consistency
  - post-meeting conclusion completeness

Verified commands for this updated state:

- `cargo test --test security_post_meeting_conclusion_cli -- --nocapture`
- `cargo test --test security_decision_package_revision_cli -- --nocapture`
- `cargo test --test security_decision_verify_package_cli -- --nocapture`
- `cargo test --test security_decision_submit_approval_cli -- --nocapture`

Important scope note:

- do not re-open this line for generic refactoring
- continue from the established package/object_graph/verify contract
- treat `dispatch_security_committee_member_agent` as a temporary compile-safe placeholder on this branch, not as a completed committee capability

If a future AI continues this security line, it should extend from the current contract instead of undoing it.
