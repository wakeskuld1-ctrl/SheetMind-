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
- Metadata field management must prefer an explicit registry instead of implicit key discovery.
- Metadata fields must declare whether they act on concept scope, node scope, or both.
- When registry mode is enabled, unregistered metadata fields must fail fast instead of being silently ignored.
- When registry mode is enabled, operators unsupported by the registered field contract must fail fast instead of degrading into empty results.
- Metadata field definitions should carry governance attributes such as group, description, applies-to reason, and deprecation state.
- Metadata registry governance quality should be inspectable through a structured validation summary instead of ad-hoc manual review.
- Metadata field governance may also carry compatibility metadata such as aliases, replacement targets, and compatibility notes.
- Metadata registry should be able to summarize compatibility chains and alias mappings without changing runtime execution semantics.
- Metadata registry should expose canonical field resolution outputs so direct hits and alias hits can converge to the same catalog-level field view.
- Metadata registry should support batch field normalization with stable input-order preservation and explicit unknown-field retention.
- Metadata registry batch normalization may expose lightweight batch summaries such as direct-hit, alias-hit, unresolved, and deprecated counts without changing runtime execution semantics.
- Metadata registry batch normalization may expose structured unknown-field lists that preserve input order and duplicate unresolved entries for audit-friendly catalog review.
- Metadata registry batch normalization may expose canonical-field aggregation views that group only resolved inputs, keep canonical first-seen order, and leave unknown fields to explicit unknown-field reporting.
- Metadata registry may expose a unified audit/export view that bundles governance summary, compatibility summary, batch normalization detail, unknown-field reporting, and canonical aggregation as directory-level outputs without changing runtime execution semantics.
- Metadata registry governance should explicitly report alias conflicts, replacement-chain cycles, and ambiguous shared replacement targets instead of leaving them as implicit catalog behavior.
- Retrieval accuracy, retrieval efficiency, and vectorization efficiency remain high priorities.
- Foundation work must avoid accidental spread into application-side files.
- The repository mainline must be described through foundation and generic capabilities, not through domain adapter progress.

## 11A. Mainline Directory Guardrails

The current mainline directory rule is:

1. `src/ops/foundation/` is reserved for the business-agnostic navigation kernel.
2. Generic reusable capabilities belong to runtime/tooling/table-analysis-report lines, not to domain-specific workflow files.
3. Domain-specific tracks belong to adapter-side lines and must not redefine project identity.

For current practical classification:

- Mainline foundation scope:
  - ontology
  - roaming
  - retrieval
  - evidence assembly
- Mainline generic scope:
  - reusable runtime
  - reusable tooling
  - reusable table / analysis / report capability
- Adapter-side scope:
  - `security_*`
  - `stock_*`
  - approval workflow objects
  - committee workflow objects
  - scorecard / refit / training workflow objects

If a future AI sees active `security_*` work in the repository, it must interpret that line as a domain adapter track, not as a foundation architecture change.

## 12. GitHub Handoff Rules

These rules must be followed whenever the project is prepared for GitHub upload or handoff:

1. Review whether the current architecture baseline has changed.
2. Review whether the hard boundaries have changed.
3. Review whether any new rejected direction has emerged.
4. Update this handoff manual if project-level guidance changed.
5. Update the short baseline file if the practical focus changed.
6. Do not push major direction changes without corresponding handoff updates.

## 12A. Version Consistency Closeout Rule

Added on 2026-04-09 to stop repeated merge-cleanup drift across future AI sessions.

The rule is:

1. Version consistency closeout is a one-time recovery task, not a default startup task for every new AI session.
2. Once a worktree has no unresolved merge conflict entries, no merge markers in active handoff files, and at least one documented verification run, future AI must continue from that baseline instead of reopening version-alignment work by default.
3. Future AI may reopen version consistency work only if at least one real trigger exists:
   - the user explicitly asks for sync, merge, pull, or branch comparison
   - the Git index contains unresolved conflicts
   - active project files contain new unresolved merge markers
   - a newly fetched remote branch introduces real divergence that affects the current line
   - fresh verification proves a real contract conflict caused by cross-branch drift
4. The following are not enough, by themselves, to reopen version consistency work:
   - a dirty worktree
   - staged but not yet pushed files
   - parallel adapter-side tracks such as `security_*` or `stock_*`
   - old execution notes that already describe the last successful closeout
5. If a future AI suspects drift, it must first read the latest handoff note and execution note, then point to the exact new conflict before starting another cleanup round.

Current durable memory for this repository:

- The 2026-04-09 `codex/foundation-merge-review` closeout is intended to be the last generic version-consistency cleanup round for this line.
- After this point, AI should default back to feature continuation, diagnostics, or governance work unless a new real trigger appears.
- A fresh combined regression run for foundation plus security closeout targets was rerun on 2026-04-09 and passed after fixture hygiene repairs.
- The most important closeout lesson was not "merge drift broke the branch"; it was "runtime cleanup exposed hidden test-fixture assumptions":
  - two security tests had been depending on untracked local SQLite state
  - one scorecard-training sample had degraded enough to trigger `RSRS` denominator-zero failure
- Future AI should treat similar failures as fixture/data hygiene candidates first, then escalate to version-consistency work only if a real cross-branch conflict is proven.

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

The foundation navigation kernel has completed Tasks 1 through 9 in the current implementation sequence:

1. foundation module entry wiring
2. ontology schema
3. ontology store
4. knowledge record and knowledge graph store
5. capability router
6. roaming engine
7. retrieval engine
8. evidence assembler
9. navigation pipeline integration

The current confirmed implementation order remains:

`ontology-lite -> roaming -> retrieval -> evidence assembly`

This ordering is now a maintenance rule, not a temporary suggestion.

Do not re-open completed Tasks 1-9 for speculative restructuring when a future AI session starts.

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
- scorecard, refit, or training workflow logic

Future AI sessions must also preserve this wording:

- `security_*` is an adapter-side domain track.
- `security_*` is not the current project mainline.
- domain progress must not be written as foundation delivery progress.

If the repo is dirty, stage and commit only the files belonging to this foundation line.

## 18. Default Priority After Task 9

The default priority is still the foundation line, not the business governance line.

Current confirmed baseline:

- `evidence_assembler` is now a real foundation output assembler
- `navigation_pipeline` now connects route, roaming, retrieval, and evidence assembly into one minimal in-memory pipeline
- `navigation_pipeline` now also supports a minimal explicit config contract for roaming scope control
- the pipeline remains inside foundation scope and does not introduce Tool dispatcher, GUI flow, or domain-specific shortcuts

Mandatory memory point:

- if the user says only "continue", "go on", or "develop further", default to the foundation line first
- only return to the security / stock governance line when the user explicitly names that business line

Before continuing foundation work, re-read:

1. [execution-notes-2026-04-08-foundation-navigation-pipeline.md](/D:/Rust/Excel_Skill/docs/execution-notes-2026-04-08-foundation-navigation-pipeline.md)
2. [navigation_pipeline.rs](/D:/Rust/Excel_Skill/src/ops/foundation/navigation_pipeline.rs)
3. [navigation_pipeline_integration.rs](/D:/Rust/Excel_Skill/tests/navigation_pipeline_integration.rs)

The next recommended refinement inside foundation is:

- continue from the new `NavigationPipelineConfig` contract
- keep configuration minimal and explicit
- avoid introducing profile systems or retrieval config splitting too early

Update after the latest boundary verification:

- `max_depth = 0` and `max_concepts = 1` are now both covered by integration tests
- both boundaries already pass under the current A1 implementation
- the next default refinement should move from config-boundary closure to the first retrieval enhancement step

Update after Task 11 retrieval enhancement layer 1:

- `retrieval_engine` no longer relies on plain token-overlap counting only
- current retrieval ranking now includes:
  - title-weighted matches
  - exact phrase bonus
  - seed concept bonus
- `seed concept bonus` must only reorder already-matched nodes and must not create hits from zero text overlap
- do not introduce `RetrievalConfig` yet

The next recommended refinement inside foundation is now:

- keep retrieval enhancement inside `retrieval_engine.rs`
- prefer small scoring improvements or evidence-oriented ranking signals
- continue avoiding config splitting until there is a proven second caller with conflicting needs

## 19. Parallel Security Governance Track Status (2026-04-08)

<<<<<<< HEAD
<!-- 2026-04-08 CST: 调整交接口径，原因是本节记录的是 Task 11 前的最小 Green 快照，而第 20 节已经写入 Task 11 后的新状态；目的：让后续 AI 明确第 19 节是历史背景，第 20 节才是当前合同状态。 -->

There is an active parallel security decision workflow line in this repository. It is not part of the foundation navigation kernel, but future AI sessions must not ignore it when working on the stock governance path.
=======
There is an active parallel security decision workflow line in this repository. It is not part of the foundation navigation kernel and it does not redefine the project mainline. Future AI sessions may continue it only as an adapter-side domain track when explicitly asked.
>>>>>>> origin/codex/foundation-navigation-kernel

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

Important classification rule:

- do not present the security governance line as the current repository mainline
- do not describe security scorecard or approval progress as foundation delivery progress
- if mainline guidance and domain-track progress appear to conflict, mainline guidance wins

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

## 21. Foundation Retrieval Update After Task 11 Layer 2 (2026-04-09)

This section is the latest foundation-side retrieval contract and should be read before any further ranking work.

Current confirmed status:

- `retrieval_engine` now applies `source_ref` priority only as a secondary tie-break
- the effective ranking order is now:
  - text score
  - source priority
  - `node_id`
- the current fixed source tiers are:
  - primary source: default tier when no derived/planning keywords are detected
  - derived / summary source: `summary`, `trend`, `report`, `analysis`, `derived`
  - planning source: `plan`, `forecast`, `scenario`
- source priority does not participate in hit creation
- source priority does not override higher text relevance
- `RetrievalHit` remains unchanged
- `RetrievalConfig` is still intentionally absent

New regression coverage added in this round:

- `retrieval_engine_prefers_primary_source_refs_when_scores_tie`
- `retrieval_engine_prefers_derived_sources_over_planning_sources_when_scores_tie`
- `retrieval_engine_keeps_higher_text_score_ahead_of_better_source_priority`

Verified commands for this state:

- `cargo test --test retrieval_engine_unit -- --nocapture`
- `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

Mandatory boundary memory:

- do not move source priority into the main score
- do not introduce `RetrievalConfig` unless there is a proven second caller with conflicting needs
- do not connect this retrieval enhancement directly to CLI, Tool, or GUI work
- continue foundation-first unless the user explicitly switches to a business line

## 22. Foundation Retrieval Update After Task 11 Layer 3 (2026-04-09)

This section supersedes the narrower Layer 2 ranking note above for future foundation retrieval work.

Current confirmed status:

- `retrieval_engine` now applies explainable multi-step tie-break ranking inside foundation scope only
- the effective ranking order is now:
  - text score
  - source priority
  - evidence reference count
  - locator precision
  - `node_id`
- `source_ref` priority still stays ahead of all evidence-side tie-break signals
- `evidence_refs` count is only used after text score and source priority are tied
- locator precision is only used after text score, source priority, and evidence reference count are tied
- current locator precision heuristics remain intentionally minimal:
  - single-cell locator such as `A1` ranks ahead of ranges
  - smaller Excel/WPS-style ranges such as `A1:B3` rank ahead of larger ranges
  - unrecognized locator strings fall back to the lowest precision tier
- `RetrievalHit` remains unchanged
- `RetrievalConfig` is still intentionally absent

New regression coverage added in this round:

- `retrieval_engine_prefers_more_evidence_refs_when_scores_and_source_priority_tie`
- `retrieval_engine_prefers_more_specific_locator_when_scores_source_and_counts_tie`
- `retrieval_engine_keeps_better_source_priority_ahead_of_more_evidence_refs`

Verified commands for this state:

- `cargo test --test retrieval_engine_unit -- --nocapture`
- `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

Mandatory boundary memory:

- do not let evidence-side signals participate in hit creation
- do not let `evidence_refs` count override better `source_ref` priority
- do not move locator precision into the main score
- do not introduce `RetrievalConfig` unless there is a proven second caller with conflicting needs
- do not connect this retrieval enhancement directly to CLI, Tool, or GUI work
- continue foundation-first unless the user explicitly switches to a business line

## 23. Foundation Retrieval Diagnostics Update After Task 11 Layer 4 (2026-04-09)

This section is the latest retrieval explainability contract and should be read before any further diagnostics or ranking work.

Current confirmed status:

- `retrieval_engine` now exposes a foundation-internal diagnostics path through `retrieve_with_diagnostics()`
- `retrieve()` still keeps the original contract and still returns only `Vec<RetrievalHit>`
- diagnostics currently explain both hit creation signals and tie-break signals for each ranked node
- each `RetrievalDiagnostic` now captures:
  - matched title tokens
  - matched body tokens
  - title overlap
  - body overlap
  - phrase bonus
  - seed bonus
  - text score
  - final score
  - source priority
  - evidence reference count
  - best locator
  - locator priority
- diagnostics are ordered to align with the final ranked `hits`
- diagnostics do not change hit creation
- diagnostics do not change ranking behavior
- diagnostics remain inside foundation retrieval scope only

New regression coverage added in this round:

- `retrieval_engine_returns_diagnostics_aligned_with_ranked_hits`
- `retrieval_engine_diagnostics_expose_text_and_tie_break_signals`

Verified commands for this state:

- `cargo test --test retrieval_engine_unit -- --nocapture`
- `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

Mandatory boundary memory:

- do not add diagnostics fields onto `RetrievalHit`
- do not let diagnostics become a second scoring system
- do not connect diagnostics directly to CLI, Tool, or GUI in this phase
- do not introduce `RetrievalConfig` unless there is a proven second caller with conflicting needs
- continue foundation-first unless the user explicitly switches to a business line

## 24. Foundation Retrieval Hygiene Diagnostics Update After Task 11 Layer 5 (2026-04-09)

This section is the latest evidence-quality diagnostics contract and should be read before any further retrieval hygiene work.

Current confirmed status:

- `RetrievalDiagnostic` now captures minimal evidence hygiene signals in addition to hit and tie-break explanations
- current hygiene fields are:
  - `duplicate_evidence_ref_count`
  - `weak_locator_count`
  - `weak_source_ref_count`
  - `hygiene_flags`
- current hygiene flags are:
  - `DuplicateEvidenceRefs`
  - `WeakLocator`
  - `WeakSourceRef`
- duplicate evidence currently means the same node carries repeated `source_ref + locator` pairs
- weak locator currently means:
  - empty locator
  - unrecognized locator
  - parsed range locator with overly broad area under the current fixed threshold
- weak source ref currently means:
  - empty normalized source ref
  - single-token placeholder source refs such as `sheet`, `data`, `table`, `source`, `file`
- hygiene diagnostics do not change hit creation
- hygiene diagnostics do not change ranking behavior
- hygiene diagnostics remain inside foundation retrieval scope only

New regression coverage added in this round:

- `retrieval_engine_diagnostics_flag_duplicate_evidence_refs`
- `retrieval_engine_diagnostics_flag_weak_locator_refs`
- `retrieval_engine_diagnostics_flag_weak_source_refs`

Verified commands for this state:

- `cargo test --test retrieval_engine_unit -- --nocapture`
- `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

Mandatory boundary memory:

- do not let hygiene diagnostics affect ranking in this phase
- do not convert hygiene flags into hidden penalties inside the main score
- do not move hygiene diagnostics directly into CLI, Tool, or GUI in this phase
- do not introduce `RetrievalConfig` unless there is a proven second caller with conflicting needs
- continue foundation-first unless the user explicitly switches to a business line

## 25. Foundation Retrieval Locator Hygiene Boundary Update After Task 11 Layer 5.1 (2026-04-09)

This section is the latest locator hygiene boundary contract and should be read before any further retrieval locator work.

Current confirmed status:

- `RetrievalDiagnostic` locator hygiene now accepts common sheet-qualified A1-style locators
- current supported locator shapes for hygiene parsing are:
  - `A1`
  - `A1:B3`
  - `Sheet1!A1`
  - `Sheet1!A1:B3`
  - `'Sheet Name'!$A$1`
  - `'Sheet Name'!$A$1:$D$5`
- current unsupported locator shapes still remain weak locators, including:
  - named ranges such as `RevenueNamedRange`
  - broader Excel locator semantics that are not plain A1-style cell or range references
- sheet-qualified large ranges still count as weak locators when the parsed area exceeds the current fixed threshold
- locator normalization in this phase only strips:
  - sheet prefixes before the last `!`
  - absolute reference markers `$`
- locator hygiene still does not affect hit creation
- locator hygiene still does not affect ranking behavior
- locator hygiene still remains inside foundation retrieval scope only

New regression coverage added in this round:

- `retrieval_engine_diagnostics_do_not_flag_sheet_qualified_single_cell_locator_as_weak`
- `retrieval_engine_diagnostics_do_not_flag_sheet_qualified_absolute_range_locator_as_weak`
- `retrieval_engine_diagnostics_flag_sheet_qualified_large_range_locator_as_weak`
- `retrieval_engine_diagnostics_still_flags_named_range_locator_as_weak`

Verified commands for this state:

- `cargo test --test retrieval_engine_unit -- --nocapture`
- `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

Mandatory boundary memory:

- do not expand locator parsing beyond plain A1-style cells and ranges without red tests first
- do not let support for sheet-qualified locators silently broaden into formula parsing
- do not treat named ranges as supported until a dedicated red-green cycle explicitly defines the contract
- do not let locator hygiene signals affect ranking in this phase
- continue foundation-first unless the user explicitly switches to a business line

## 26. Current Dirty Worktree Soft-Isolation Map (2026-04-09)

This section is the latest worktree hygiene map and should be read before any new implementation starts on this branch.

Current branch:

- `codex/security-post-meeting-package-binding`

Current dirty worktree count:

- `52` paths are currently dirty or untracked in this worktree

Current soft-isolation policy:

- do not try to clean the whole repository in one pass
- do not revert unrelated dirty files unless the user explicitly asks
- keep `foundation` as the active implementation line by default
- treat `security` and `stock` changes as parallel business-line dirty work
- treat shared entry files as high-risk files that should only be touched when a task explicitly requires them

Current dirty groups:

### Foundation Group (`14`)

Scope:

- foundation ontology, roaming, retrieval, evidence assembly, navigation pipeline, and foundation-facing unit/integration tests

Files:

- `src/ops/foundation.rs`
- `src/ops/foundation/capability_router.rs`
- `src/ops/foundation/evidence_assembler.rs`
- `src/ops/foundation/knowledge_record.rs`
- `src/ops/foundation/navigation_pipeline.rs`
- `src/ops/foundation/ontology_schema.rs`
- `src/ops/foundation/retrieval_engine.rs`
- `src/ops/foundation/roaming_engine.rs`
- `tests/evidence_assembler_unit.rs`
- `tests/knowledge_graph_store_unit.rs`
- `tests/knowledge_record_unit.rs`
- `tests/navigation_pipeline_integration.rs`
- `tests/ontology_store_unit.rs`
- `tests/retrieval_engine_unit.rs`

Rule:

- new default work should stay inside this group unless the user explicitly switches lines

### Security Group (`17`)

Scope:

- approval package, decision governance, post-meeting conclusion, and related security CLI tests

Files:

- `src/ops/security_analysis_contextual.rs`
- `src/ops/security_analysis_fullstack.rs`
- `src/ops/security_decision_approval_bridge.rs`
- `src/ops/security_decision_card.rs`
- `src/ops/security_decision_committee.rs`
- `src/ops/security_decision_evidence_bundle.rs`
- `src/ops/security_decision_package_revision.rs`
- `src/ops/security_decision_submit_approval.rs`
- `src/ops/security_decision_verify_package.rs`
- `src/ops/security_position_plan.rs`
- `src/ops/security_record_post_meeting_conclusion.rs`
- `tests/security_decision_committee_cli.rs`
- `tests/security_decision_evidence_bundle_cli.rs`
- `tests/security_decision_package_revision_cli.rs`
- `tests/security_decision_submit_approval_cli.rs`
- `tests/security_decision_verify_package_cli.rs`
- `tests/security_post_meeting_conclusion_cli.rs`

Rule:

- do not mix this group into foundation commits unless the user explicitly asks to switch to the security line

### Stock Group (`2`)

Scope:

- stock business entry and stock dispatcher bridge

Files:

- `src/ops/stock.rs`
- `src/tools/dispatcher/stock_ops.rs`

Rule:

- keep isolated from foundation work unless the user explicitly asks to return to the stock line

### Shared Entry Group (`2`)

Scope:

- top-level module exports and central dispatcher wiring

Files:

- `src/ops/mod.rs`
- `src/tools/dispatcher.rs`

Rule:

- treat these files as cross-line risk points
- only touch them when a task truly requires shared registration or export changes

### Docs and Handoff Group (`17`)

Scope:

- execution notes, design/plan files, AI handoff, and task journal

Files:

- `.trae/CHANGELOG_TASK.md`
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`
- `docs/execution-notes-2026-04-08-foundation-navigation-pipeline.md`
- `docs/plans/2026-04-08-foundation-navigation-pipeline-config-design.md`
- `docs/plans/2026-04-08-foundation-navigation-pipeline-config-plan.md`
- `docs/plans/2026-04-08-foundation-navigation-pipeline-design.md`
- `docs/plans/2026-04-08-foundation-navigation-pipeline-plan.md`
- `docs/plans/2026-04-08-foundation-retrieval-enhancement-design.md`
- `docs/plans/2026-04-08-foundation-retrieval-enhancement-plan.md`
- `docs/plans/2026-04-09-foundation-retrieval-diagnostics-design.md`
- `docs/plans/2026-04-09-foundation-retrieval-diagnostics-plan.md`
- `docs/plans/2026-04-09-foundation-retrieval-evidence-tiebreak-design.md`
- `docs/plans/2026-04-09-foundation-retrieval-evidence-tiebreak-plan.md`
- `docs/plans/2026-04-09-foundation-retrieval-hygiene-diagnostics-design.md`
- `docs/plans/2026-04-09-foundation-retrieval-hygiene-diagnostics-plan.md`
- `docs/plans/2026-04-09-foundation-retrieval-source-priority-design.md`
- `docs/plans/2026-04-09-foundation-retrieval-source-priority-plan.md`

Rule:

- doc staging should follow the code theme it belongs to
- do not batch all docs together with unrelated business-line code

Mandatory operating memory:

- if the user only says “continue”, continue the `foundation` group first
- if a task touches `security`, explicitly say that the active line has switched
- if a task touches `stock`, explicitly say that the active line has switched
- if a task touches `src/ops/mod.rs` or `src/tools/dispatcher.rs`, call out the cross-line risk before editing
- do not interpret “整理脏改动” as permission to discard parallel work

## 27. Dirty File Meaning Reference For Future AI Handoffs (2026-04-09)

这一节不是新的开发方案，而是当前脏文件的用途说明表。

目的：

- 以后 AI 接手时，不需要重新猜这些脏文件分别在做什么
- 以后 AI 接手时，能先判断“这是 foundation 继续项，还是 security/stock 并行项”
- 以后 AI 接手时，知道哪些文件不能因为“看起来脏”就直接删掉或混到同一个提交里

使用规则：

- 如果用户没有显式切线，默认继续 `foundation`
- 如果任务碰到 `security` 文件，先说明这是在切到证券治理线
- 如果任务碰到 `stock` 文件，先说明这是在切到证券业务线
- 如果任务碰到 `src/ops/mod.rs` 或 `src/tools/dispatcher.rs`，先说明这是共享入口风险点
- “脏”只表示当前未提交，不表示“没用”或“应该删除”

### Foundation Dirty Files

- `src/ops/foundation.rs`：foundation 总入口与命名空间边界文件，用来把通用分析能力和新导航内核收进 `foundation` 域。不能随便删，因为它承担“底座能力归属”这条架构边界。
- `src/ops/foundation/capability_router.rs`：问题到 seed concept 的路由入口，用来把自然语言问题先收敛到候选概念。不能随便删，因为 `navigation_pipeline` 的第一段就依赖它。
- `src/ops/foundation/evidence_assembler.rs`：把 route、roam、retrieve 的结果装配成统一 `NavigationEvidence` 输出。不能随便删，因为它是 foundation 闭环的最终结构化输出层。
- `src/ops/foundation/knowledge_record.rs`：定义知识节点、知识边、证据引用等基础数据模型。不能随便删，因为 retrieval、graph store、assembler 都依赖这些结构。
- `src/ops/foundation/navigation_pipeline.rs`：foundation 内部最小导航流水线，负责串起 route、roam、retrieve、assemble。不能随便删，因为它是当前底座主线最明确的整合入口。
- `src/ops/foundation/ontology_schema.rs`：概念、别名、关系类型等 ontology-lite 定义入口。不能随便删，因为 capability routing 和 roaming 都依赖它的概念图谱定义。
- `src/ops/foundation/retrieval_engine.rs`：foundation 检索核心，当前承载排序增强、diagnostics、hygiene diagnostics。不能随便删，因为最近几轮底座工作几乎都在这里收口。
- `src/ops/foundation/roaming_engine.rs`：知识漫游引擎，负责从 seed concept 沿允许关系扩出候选域。不能随便删，因为 route 后必须先有 scope 才能 retrieve。
- `tests/evidence_assembler_unit.rs`：`evidence_assembler.rs` 的单测合同，保证 citations、summary、path/hits 保真。不能随便删，因为这就是 assembler 的回归保护网。
- `tests/knowledge_graph_store_unit.rs`：knowledge graph store 的读写与候选节点收集测试。不能随便删，因为 graph store 是 retrieval 的基础依赖。
- `tests/knowledge_record_unit.rs`：知识记录模型的最小结构合同测试。不能随便删，因为一旦模型漂移，底座其他层会一起受影响。
- `tests/navigation_pipeline_integration.rs`：foundation pipeline 的最小集成测试，验证 route -> roam -> retrieve -> assemble 闭环。不能随便删，因为它保护的是“底座能不能跑通”而不是单点函数。
- `tests/ontology_store_unit.rs`：ontology store 邻接关系与概念读取测试。不能随便删，因为 capability routing 与 roaming 的输入都来自这里。
- `tests/retrieval_engine_unit.rs`：retrieval 排序、diagnostics、hygiene diagnostics 的核心回归集。不能随便删，因为这几轮 foundation 主线都靠它做红绿闭环。

### Security Dirty Files

- `src/ops/security_analysis_contextual.rs`：证券分析上下文构建能力，属于证券业务治理链的上游输入。不能和 foundation 混提，因为它不是通用底座。
- `src/ops/security_analysis_fullstack.rs`：证券分析全链路能力，负责较完整的证券分析结果组织。不能和 foundation 混提，因为语义明显属于证券业务域。
- `src/ops/security_decision_approval_bridge.rs`：把证券决策结果桥接成审批工件的适配层。不能随便删，因为 approval brief、approval request、position plan 等工件要靠它衔接。
- `src/ops/security_decision_card.rs`：证券决策卡片结构与产物，属于审批链中的核心对象之一。不能随便删，因为后续 package 与 brief 会引用它。
- `src/ops/security_decision_committee.rs`：证券决策委员会入口，负责把证券分析结果推进到 committee 级别结论。不能随便删，因为 submit approval 流程会先依赖它。
- `src/ops/security_decision_evidence_bundle.rs`：证券决策证据包能力，用来把分析证据打包成审批可消费材料。不能随便删，因为它是治理链的证据层工件。
- `src/ops/security_decision_package_revision.rs`：证券审批包 revision 入口，负责生成新版本 decision package。不能随便删，因为 governance 主线要求 package 可演进。
- `src/ops/security_decision_submit_approval.rs`：证券审批提交总入口，负责 committee、bridge、持久化等总流程。不能随便删，因为它是证券治理主链的关键落点。
- `src/ops/security_decision_verify_package.rs`：证券审批包校验入口，负责对 package 做治理一致性验证。不能随便删，因为 package 没有 verify 就不构成治理闭环。
- `src/ops/security_position_plan.rs`：证券仓位计划对象与相关治理语义。不能随便删，因为 approval request 与 package object graph 会绑定它。
- `src/ops/security_record_post_meeting_conclusion.rs`：会后结论记录入口，负责落盘 post-meeting conclusion 并驱动 revision。不能随便删，因为它是治理链往后半段推进的关键动作。
- `tests/security_decision_committee_cli.rs`：committee CLI/Tool 入口测试。不能随便删，因为它保护的是外部调用合同。
- `tests/security_decision_evidence_bundle_cli.rs`：evidence bundle CLI/Tool 合同测试。不能随便删，因为它约束证券证据包对外行为。
- `tests/security_decision_package_revision_cli.rs`：package revision CLI/Tool 测试。不能随便删，因为它保护 version bump 与 object graph 延续语义。
- `tests/security_decision_submit_approval_cli.rs`：submit approval CLI/Tool 测试。不能随便删，因为它约束证券审批提交主线。
- `tests/security_decision_verify_package_cli.rs`：verify package CLI/Tool 测试。不能随便删，因为治理校验的失败语义都钉在这里。
- `tests/security_post_meeting_conclusion_cli.rs`：post meeting conclusion CLI/Tool 测试。不能随便删，因为它保护的是 Task 3 一类治理链路的对外入口。

### Stock Dirty Files

- `src/ops/stock.rs`：证券业务域的总入口，用来把证券分析、审批、包、会后结论等能力收进 `stock` 域。不能随便删，因为它就是“证券业务和 foundation 分域”的根文件之一。
- `src/tools/dispatcher/stock_ops.rs`：dispatcher 对证券业务域的分发桥。不能随便删，因为 stock 线如果要从工具层暴露，最终都要经过这里。

### Shared Entry Dirty Files

- `src/ops/mod.rs`：`foundation` 与 `stock` 的兼容导出层。不能随便删，因为仓库里还存在大量 `crate::ops::...` 旧路径依赖，它是过渡兼容层。
- `src/tools/dispatcher.rs`：整个工具分发总入口，负责把 workbook、single_table、analysis、stock 等工具挂到统一 dispatcher。不能随便删，因为删它会直接影响对外工具调用面。

### Docs And Handoff Dirty Files

- `.trae/CHANGELOG_TASK.md`：任务级追加日志，记录每次改动的原因、剩余项和风险。不能随便删，因为这是 AI 交接时最直接的“过程记忆”。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`：当前最重要的 AI 交接手册。不能随便删，因为以后接手顺序、主线、边界、脏改动地图都在这里。
- `docs/execution-notes-2026-04-08-foundation-navigation-pipeline.md`：foundation navigation/pipeline 主线执行记录。不能随便删，因为它保存了 Task 8 到 Task 11 的推进轨迹与验证结果。
- `docs/plans/2026-04-08-foundation-navigation-pipeline-config-design.md`：navigation pipeline config 的设计说明。不能随便删，因为它记录了为什么只做最小配置对象。
- `docs/plans/2026-04-08-foundation-navigation-pipeline-config-plan.md`：navigation pipeline config 的执行计划。不能随便删，因为它对应 A1 配置化收口步骤。
- `docs/plans/2026-04-08-foundation-navigation-pipeline-design.md`：foundation navigation pipeline 设计文档。不能随便删，因为它说明了为什么要有 route/roam/retrieve/assemble 闭环。
- `docs/plans/2026-04-08-foundation-navigation-pipeline-plan.md`：foundation navigation pipeline 实施计划。不能随便删，因为它是 pipeline 落地时的步骤依据。
- `docs/plans/2026-04-08-foundation-retrieval-enhancement-design.md`：retrieval 第一层增强设计。不能随便删，因为它定义了标题优先、短语优先、seed concept 优先等边界来源。
- `docs/plans/2026-04-08-foundation-retrieval-enhancement-plan.md`：retrieval 第一层增强执行计划。不能随便删，因为它记录了先红后绿的实施路径。
- `docs/plans/2026-04-09-foundation-retrieval-diagnostics-design.md`：retrieval diagnostics 设计文档。不能随便删，因为它解释了为什么 `retrieve_with_diagnostics()` 存在。
- `docs/plans/2026-04-09-foundation-retrieval-diagnostics-plan.md`：retrieval diagnostics 执行计划。不能随便删，因为它承接了 Layer 4 explainability 的落地步骤。
- `docs/plans/2026-04-09-foundation-retrieval-evidence-tiebreak-design.md`：evidence-side tie-break 设计文档。不能随便删，因为 evidence count 与 locator precision 的层级来自这里。
- `docs/plans/2026-04-09-foundation-retrieval-evidence-tiebreak-plan.md`：evidence-side tie-break 执行计划。不能随便删，因为它对应 Layer 3 的推进顺序。
- `docs/plans/2026-04-09-foundation-retrieval-hygiene-diagnostics-design.md`：hygiene diagnostics 设计文档。不能随便删，因为 duplicate/weak locator/weak source_ref 的语义来源于这里。
- `docs/plans/2026-04-09-foundation-retrieval-hygiene-diagnostics-plan.md`：hygiene diagnostics 执行计划。不能随便删，因为它记录了 Layer 5 的红绿路径。
- `docs/plans/2026-04-09-foundation-retrieval-source-priority-design.md`：source priority 设计文档。不能随便删，因为 primary/derived/planning 的次级排序边界来自这里。
- `docs/plans/2026-04-09-foundation-retrieval-source-priority-plan.md`：source priority 执行计划。不能随便删，因为它承接 Layer 2 的实施步骤。

### Why These Dirty Files Are Not Merged Or Deleted By Default

- `foundation`、`security`、`stock` 是三条不同主线，混成一个提交后几乎无法清楚回滚
- `src/ops/mod.rs` 与 `src/tools/dispatcher.rs` 属于共享入口，一旦混提最容易重新把底座和业务层缠回去
- 很多脏文件并不是垃圾，而是已经完成但尚未单独收口的有效产物
- 文档类脏文件不是噪音，它们是当前架构和任务推进的 durable memory
- 当前只对 foundation 最小回归有明确验证记录，其他组不能假设“直接一起合并也安全”

### Operating Advice For Future AI

- 如果目标是继续底座能力，优先只看 `foundation` 组和对应 docs
- 如果目标是整理提交，按组分批，不要把 `foundation + security + stock + shared` 混成一次提交
- 如果目标是清理仓库，先得到用户显式许可，再做更强的 stash、分支拆分或历史清理动作
- 如果目标只是“继续开发”，不要重复做一遍脏文件分析，直接把本节当成当前工作区的说明索引

## 28. Foundation Weak Source Ref Hygiene Boundary Update After Task 11 Layer 5.2 (2026-04-09)

This section is the latest weak source ref hygiene contract and should be read before any further source-name diagnostics work.

Current confirmed status:

- `RetrievalDiagnostic` weak source ref hygiene no longer only flags single-token placeholder source names
- current weak source ref shapes now include:
  - empty normalized source refs
  - single-token placeholder refs such as `sheet`, `data`, `table`, `source`, `file`
  - multi-token source refs composed only of placeholder tokens such as `source data`
  - placeholder source refs with numeric suffixes such as `table 1`
- current weak source ref shapes still do not include source refs that contain meaningful business tokens, even if they also contain placeholder words
  - example kept non-weak in regression: `sales detail sheet`
- weak source ref hygiene still does not affect hit creation
- weak source ref hygiene still does not affect ranking behavior
- weak source ref hygiene still remains inside foundation retrieval scope only

New regression coverage added in this round:

- `retrieval_engine_diagnostics_flag_multi_token_placeholder_source_refs`
- `retrieval_engine_diagnostics_flag_placeholder_source_refs_with_numeric_suffix`
- `retrieval_engine_diagnostics_do_not_flag_semantic_source_refs_with_placeholder_tokens_as_weak`

Verified commands for this state:

- `cargo test --test retrieval_engine_unit source_ref -- --nocapture`
- `cargo test --test retrieval_engine_unit -- --nocapture`
- `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

Mandatory boundary memory:

- do not expand weak source ref rules by broad keyword matching alone
- do not mark source refs as weak when they contain concrete business semantics
- do not let weak source ref hygiene become a hidden ranking penalty
- continue foundation-first unless the user explicitly switches to a business line

## 29. Foundation Weak Source Ref Hygiene Boundary Update After Task 11 Layer 5.3 (2026-04-09)

This section is the latest weak source ref hygiene contract and should be read before any further compact source-name diagnostics work.

Current confirmed status:

- `RetrievalDiagnostic` weak source ref hygiene now also flags compact placeholder source refs with numeric suffixes
  - regression example now kept weak: `sheet1`
- the current compact rule remains intentionally narrow
  - only `placeholder token + digits` is included in this round
- previously confirmed weak source ref shapes still remain valid:
  - empty normalized source refs
  - single-token placeholder refs such as `sheet`, `data`, `table`, `source`, `file`
  - multi-token placeholder refs such as `source data`
  - placeholder refs with spaced numeric suffixes such as `table 1`
- weak source ref hygiene still does not affect hit creation
- weak source ref hygiene still does not affect ranking behavior
- weak source ref hygiene still remains inside foundation retrieval scope only

New regression coverage added in this round:

- `retrieval_engine_diagnostics_flag_compact_placeholder_source_refs_with_numeric_suffix`

Verified commands for this state:

- `cargo test --test retrieval_engine_unit retrieval_engine_diagnostics_flag_compact_placeholder_source_refs_with_numeric_suffix -- --nocapture`
- `cargo test --test retrieval_engine_unit -- --nocapture`
- `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

Mandatory boundary memory:

- do not broaden compact weak source ref matching into generic prefix matching
- do not let compact weak source ref hygiene become a hidden ranking penalty
- continue foundation-first unless the user explicitly switches to a business line

## 30. Foundation Locator Hygiene Boundary Update After Task 11 Layer 5.4 (2026-04-09)

This section is the latest locator hygiene boundary contract and should be read before any further external-workbook locator work.

Current confirmed status:

- `RetrievalDiagnostic` locator hygiene now also accepts Windows absolute-path external workbook A1-style ranges
  - regression examples now kept non-weak:
    - `C:\Reports\[Budget.xlsx]Sheet1!A1:B3`
    - `C:\Reports\[Budget.xlsx]'Sales Detail'!$A$1:$D$5`
- this round only fixes one narrow parsing boundary
  - the current change handles drive-letter `:` breaking the range split before `A1:B3` is parsed
- previously confirmed locator hygiene boundaries still remain valid:
  - plain A1 cells and ranges remain supported
  - sheet-qualified and `$`-qualified A1 cells and ranges remain supported
  - large ranges over the current fixed threshold still remain weak
  - named ranges still remain weak
  - 3D references still remain unsupported
- locator hygiene still does not affect hit creation
- locator hygiene still does not affect ranking behavior
- locator hygiene still remains inside foundation retrieval scope only

New regression coverage added in this round:

- `retrieval_engine_diagnostics_do_not_flag_windows_path_external_workbook_range_locator_as_weak`
- `retrieval_engine_diagnostics_do_not_flag_windows_path_external_workbook_absolute_range_locator_as_weak`

Verified commands for this state:

- `cargo test --test retrieval_engine_unit windows_path_external_workbook -- --nocapture`
- `cargo test --test retrieval_engine_unit -- --nocapture`
- `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

Mandatory boundary memory:

- do not broaden this fix into generic path or formula parsing
- do not treat 3D references as supported until a dedicated red-green cycle defines that contract
- do not let locator hygiene become a hidden ranking penalty
- continue foundation-first unless the user explicitly switches to a business line

## 31. Foundation Locator Hygiene Boundary Update After Task 11 Layer 5.5 (2026-04-09)

This section is the latest locator hygiene boundary contract and should be read before any further Windows-path large-range locator work.

Current confirmed status:

- `RetrievalDiagnostic` locator hygiene now also has explicit regression protection for Windows absolute-path external workbook large ranges
  - regression examples now kept weak:
    - `C:\Reports\[Budget.xlsx]Sheet1!A1:Z200`
    - `C:\Reports\[Budget.xlsx]'Sales Detail'!$A$1:$Z$200`
- this round does not change production code
- the new large-range protection tests passed immediately
  - this confirms the current implementation already preserves the existing large-range weak threshold after the Task 11 Layer 5.4 path-prefix parsing fix
- previously confirmed locator hygiene boundaries still remain valid:
  - plain A1 cells and ranges remain supported
  - sheet-qualified and `$`-qualified A1 cells and ranges remain supported
  - Windows-path external workbook small ranges remain supported
  - large ranges over the current fixed threshold still remain weak
  - named ranges still remain weak
  - 3D references still remain unsupported
- locator hygiene still does not affect hit creation
- locator hygiene still does not affect ranking behavior
- locator hygiene still remains inside foundation retrieval scope only

New regression coverage added in this round:

- `retrieval_engine_diagnostics_still_flag_windows_path_external_workbook_large_range_locator_as_weak`
- `retrieval_engine_diagnostics_still_flag_windows_path_external_workbook_absolute_large_range_locator_as_weak`

Verified commands for this state:

- `cargo test --test retrieval_engine_unit windows_path_external_workbook_large_range -- --nocapture`
- `cargo test --test retrieval_engine_unit absolute_large_range -- --nocapture`
- `cargo test --test retrieval_engine_unit -- --nocapture`
- `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

Mandatory boundary memory:

- do not misread Windows-path locator support as a blanket exemption from the large-range weak threshold
- do not broaden this contract into generic workbook path parsing or full formula parsing
- do not treat 3D references as supported until a dedicated red-green cycle defines that contract
- do not let locator hygiene become a hidden ranking penalty
- if a protection test passes immediately, record the boundary and move on instead of inventing unnecessary production changes
