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

## 3. Current Technical Direction

- The foundation must remain business-agnostic.
- Metadata is a filtering layer, not the core semantic layer.
- Ontology-lite must be introduced as the semantic backbone.
- Knowledge roaming must be introduced to complete upstream and downstream context.
- Vectorization is an enhancement layer, not the core association mechanism.
- Cloud or local models are optional provider-based enhancers, not architectural prerequisites.

## 4. Explicitly Rejected Directions

- GUI-first intelligent Q&A.
- Building application-side chat flows before the foundation is stable.
- Designing the foundation around stock, financial reports, contracts, invoices, or any single domain.
- Treating metadata plus vector retrieval as a sufficient semantic architecture.
- Making any model provider a hard dependency of the core navigation flow.

## 5. Current Priorities

1. Retrieval accuracy.
2. Retrieval execution efficiency.
3. Vectorization efficiency.
4. Ontology-lite design.
5. Knowledge roaming design.
6. Provider-based model enhancement interfaces.

## 6. Read This First

- Read this file first.
- Then read [AI_HANDOFF_MANUAL.md](/D:/Rust/Excel_Skill/docs/ai-handoff/AI_HANDOFF_MANUAL.md).
- Do not start new architecture work until these constraints are understood.
