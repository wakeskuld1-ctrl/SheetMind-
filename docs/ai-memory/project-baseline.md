# Project Baseline

## 1. Current Focus

- Current work is focused on foundation capabilities, not application-side features.
- The current foundation target is an ontology-driven knowledge navigation base.
- The primary technical direction is: ontology first, roaming second, retrieval third.

## 2. Hard Boundaries

- Do not touch GUI for the current AI roadmap.
- Do not prioritize GUI-based intelligent Q&A.
- Do not mix application-side flow, page logic, or presentation logic into foundation work.
- Do not hard-code any single business domain into the foundation model.
- Do not treat retrieval as the system entry point; retrieval is an executor inside the navigation flow.
- Do not treat `security/*`, `stock/*`, approval flows, committee flows, or scorecard flows as the current project mainline.
- Domain-specific tracks may exist in the repository, but they are adapter-side work and must not redefine foundation scope.

## 3. Current Technical Direction

- The foundation must remain business-agnostic.
- Metadata is a filtering layer, not the core semantic layer.
- Metadata field management should prefer an explicit registry over ad-hoc key discovery.
- Metadata fields must declare whether they apply to concept scope, node scope, or both.
- Registry mode should reject unregistered metadata fields instead of silently skipping them.
- Registry mode should reject operators that are not declared by the registered metadata field contract.
- Metadata field definitions should include governance metadata such as group, description, applies-to reason, and deprecation note.
- Metadata registry should expose a structured governance validation summary for audit and handoff.
- Metadata field governance can include compatibility attributes such as aliases, replaced-by targets, and compatibility notes.
- Metadata registry should expose compatibility summaries and replacement chains as directory-level governance outputs, not as runtime execution behavior.
- Metadata registry should expose canonical field resolution so explicit field names and aliases can converge to one catalog-level interpretation.
- Metadata registry batch normalization should preserve input order and keep unknown fields visible instead of silently dropping them.
- Metadata registry batch normalization can expose lightweight count summaries before any later unknown-field list or canonical aggregation view is introduced.
- Metadata registry can expose a structured unknown-field list with input positions before any later canonical aggregation view is introduced.
- Metadata registry can expose a canonical aggregation view that groups only resolved inputs, preserves canonical first-seen order, and keeps unknown fields outside the aggregation itself.
- Metadata registry can expose a unified audit/export object that bundles governance summary, compatibility summary, batch normalization detail, unknown-field reporting, and canonical aggregation while remaining outside runtime execution semantics.
- Metadata registry governance should fail catalog review when alias conflicts, replacement cycles, or ambiguous shared replacement targets exist.
- Ontology-lite must be introduced as the semantic backbone.
- Knowledge roaming must be introduced to complete upstream and downstream context.
- Vectorization is an enhancement layer, not the core association mechanism.
- Cloud or local models are optional provider-based enhancers, not architectural prerequisites.

## 4. Mainline Directory Rule

The current mainline should be understood as three layers:

1. `src/ops/foundation/` for ontology, roaming, retrieval, and evidence assembly only.
2. Generic runtime/tooling for reusable table, analysis, report, and delivery capabilities.
3. Domain adapters as upper-layer tracks or examples, not as the project identity.

Concrete guardrails:

- `src/ops/foundation/` is the business-agnostic navigation kernel.
- Reusable non-domain capabilities may grow around runtime, tooling, table processing, analysis, and reporting.
- Domain-specific files such as `security_*`, `stock_*`, approval chains, and scorecard chains are adapter-side work.
- Domain adapter progress must not be described as foundation progress.

## 5. Explicitly Rejected Directions

- GUI-first intelligent Q&A.
- Building application-side chat flows before the foundation is stable.
- Designing the foundation around stock, financial reports, contracts, invoices, or any single domain.
- Treating metadata plus vector retrieval as a sufficient semantic architecture.
- Making any model provider a hard dependency of the core navigation flow.
- Treating any domain adapter branch as evidence that the project mainline has shifted away from foundation.

## 6. Current Priorities

1. Retrieval accuracy.
2. Retrieval execution efficiency.
3. Vectorization efficiency.
4. Ontology-lite design.
5. Knowledge roaming design.
6. Provider-based model enhancement interfaces.

## 7. Read This First

- Read this file first.
- Then read [AI_HANDOFF_MANUAL.md](/D:/Rust/Excel_Skill/docs/ai-handoff/AI_HANDOFF_MANUAL.md).
- Do not start new architecture work until these constraints are understood.
