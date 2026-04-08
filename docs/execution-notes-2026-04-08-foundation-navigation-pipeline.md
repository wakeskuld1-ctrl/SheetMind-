# 2026-04-08 Foundation Navigation Pipeline Execution Notes

## Scope

This execution note covers the current foundation-only line from Task 8 through Task 11 Layer 5.

Included:

- `EvidenceAssembler` minimum closure
- `NavigationPipeline` minimum integration closure
- `NavigationPipelineConfig` minimum explicit configuration
- retrieval ranking enhancement layer 1
- retrieval `source_ref` tie-break enhancement layer 2
- retrieval evidence-side tie-break enhancement layer 3
- retrieval diagnostics explainability layer 4
- retrieval evidence hygiene diagnostics layer 5

Not included:

- CLI / Tool / dispatcher integration
- GUI integration
- security / stock / approval business flows
- provider / embedding / vector integration

## Current Foundation Flow

The currently confirmed flow remains:

`question -> ontology positioning -> candidate roaming -> scoped retrieval -> evidence assembly`

This is still a foundation-internal pipeline, not an application entry.

## Confirmed Delivery Status

### Task 8

- `EvidenceAssembler` now assembles:
  - `route`
  - `roaming_path`
  - `hits`
  - `citations`
  - `summary`

### Task 9

- `NavigationPipeline` now connects:
  - route
  - roam
  - retrieve
  - assemble
- the pipeline exposes stage-aware errors for:
  - `Route`
  - `Roam`
  - `Retrieve`

### Task 10 A1

- `NavigationPipelineConfig` is now part of the foundation contract
- `NavigationPipeline::new()` still uses default config
- `NavigationPipeline::new_with_config()` allows explicit overrides for:
  - `allowed_relation_types`
  - `max_depth`
  - `max_concepts`
- both boundary guards are now covered and passing:
  - `max_depth = 0`
  - `max_concepts = 1`

### Task 11 Layer 1

- retrieval ranking now includes:
  - title-weighted matching
  - exact phrase bonus
  - seed concept bonus
- `seed bonus` only reorders already matched nodes
- `seed bonus` does not create hits from zero text overlap

### Task 11 Layer 2

- retrieval ranking now adds `source_ref` priority only as a secondary tie-break
- current ranking order is:
  - text score
  - source priority
  - `node_id`
- current fixed source tiers are:
  - primary source: default when no derived/planning keyword is detected
  - derived / summary source: `summary`, `trend`, `report`, `analysis`, `derived`
  - planning source: `plan`, `forecast`, `scenario`
- `source_ref` does not create hits
- `source_ref` does not override a higher text score
- `RetrievalHit` remains unchanged
- `RetrievalConfig` is still intentionally not introduced

### Task 11 Layer 3

- retrieval ranking now adds two evidence-side tie-break layers after `source_ref` priority
- current ranking order is:
  - text score
  - source priority
  - evidence reference count
  - locator precision
  - `node_id`
- `evidence_refs` count is only used when text score and source priority are tied
- locator precision is only used when text score, source priority, and evidence reference count are tied
- current locator precision heuristics remain intentionally minimal:
  - single-cell locator such as `A1` ranks ahead of ranges
  - smaller Excel/WPS-style ranges such as `A1:B3` rank ahead of larger ranges
  - unrecognized locator strings fall back to the lowest precision tier
- evidence-side tie-break signals do not create hits
- evidence-side tie-break signals do not override better source priority
- `RetrievalHit` remains unchanged
- `RetrievalConfig` is still intentionally not introduced

### Task 11 Layer 4

- retrieval now exposes a foundation-internal diagnostics path through `retrieve_with_diagnostics()`
- `retrieve()` still returns only `Vec<RetrievalHit>`
- diagnostics are ordered to align with final ranked hits
- each diagnostic currently exposes:
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
- diagnostics explain ranking behavior but do not alter ranking behavior
- diagnostics remain inside foundation retrieval and are not wired into CLI / Tool / GUI

### Task 11 Layer 5

- retrieval diagnostics now also expose minimal evidence hygiene signals
- current hygiene fields are:
  - `duplicate_evidence_ref_count`
  - `weak_locator_count`
  - `weak_source_ref_count`
  - `hygiene_flags`
- current hygiene flags are:
  - `DuplicateEvidenceRefs`
  - `WeakLocator`
  - `WeakSourceRef`
- duplicate evidence is currently detected by repeated `source_ref + locator` pairs inside the same node
- weak locator is currently detected for:
  - empty locators
  - unrecognized locators
  - overly broad range locators under the current fixed threshold
- weak source ref is currently detected for:
  - empty normalized source refs
  - single-token placeholder refs such as `sheet`, `data`, `table`, `source`, `file`
- hygiene diagnostics explain evidence quality but do not alter ranking behavior
- hygiene diagnostics remain inside foundation retrieval and are not wired into CLI / Tool / GUI

## Added Tests

### Evidence and Pipeline

- `evidence_assembler_preserves_route_path_hits_and_citations`
- `navigation_pipeline_resolves_question_into_structured_evidence`
- `navigation_pipeline_returns_route_error_for_unknown_question`
- `navigation_pipeline_returns_retrieve_error_when_scope_has_no_matching_evidence`
- `navigation_pipeline_uses_custom_config_to_limit_roaming_scope`
- `navigation_pipeline_stops_roaming_when_max_depth_is_zero`
- `navigation_pipeline_stops_roaming_when_max_concepts_is_one`

### Retrieval Layer 1

- `retrieval_engine_prefers_title_match_over_body_only_match`
- `retrieval_engine_prefers_exact_phrase_match_over_scattered_tokens`
- `retrieval_engine_prefers_seed_concept_nodes_over_roamed_nodes_when_scores_tie`

### Retrieval Layer 2

- `retrieval_engine_prefers_primary_source_refs_when_scores_tie`
- `retrieval_engine_prefers_derived_sources_over_planning_sources_when_scores_tie`
- `retrieval_engine_keeps_higher_text_score_ahead_of_better_source_priority`

### Retrieval Layer 3

- `retrieval_engine_prefers_more_evidence_refs_when_scores_and_source_priority_tie`
- `retrieval_engine_prefers_more_specific_locator_when_scores_source_and_counts_tie`
- `retrieval_engine_keeps_better_source_priority_ahead_of_more_evidence_refs`

### Retrieval Layer 4

- `retrieval_engine_returns_diagnostics_aligned_with_ranked_hits`
- `retrieval_engine_diagnostics_expose_text_and_tie_break_signals`

### Retrieval Layer 5

- `retrieval_engine_diagnostics_flag_duplicate_evidence_refs`
- `retrieval_engine_diagnostics_flag_weak_locator_refs`
- `retrieval_engine_diagnostics_flag_weak_source_refs`
- `retrieval_engine_diagnostics_do_not_flag_sheet_qualified_single_cell_locator_as_weak`
- `retrieval_engine_diagnostics_do_not_flag_sheet_qualified_absolute_range_locator_as_weak`
- `retrieval_engine_diagnostics_flag_sheet_qualified_large_range_locator_as_weak`
- `retrieval_engine_diagnostics_still_flags_named_range_locator_as_weak`
- `retrieval_engine_diagnostics_flag_multi_token_placeholder_source_refs`
- `retrieval_engine_diagnostics_flag_placeholder_source_refs_with_numeric_suffix`
- `retrieval_engine_diagnostics_do_not_flag_semantic_source_refs_with_placeholder_tokens_as_weak`
- `retrieval_engine_diagnostics_flag_compact_placeholder_source_refs_with_numeric_suffix`
- `retrieval_engine_diagnostics_do_not_flag_windows_path_external_workbook_range_locator_as_weak`
- `retrieval_engine_diagnostics_do_not_flag_windows_path_external_workbook_absolute_range_locator_as_weak`

## Verification Commands

- `cargo test --test retrieval_engine_unit -- --nocapture`
- `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

## Verification Results

- `retrieval_engine_unit`: `27 passed`
- foundation minimum regression set: `46 passed`

Notes:

- repository-level `dead_code` warnings still exist
- these warnings are pre-existing and were not introduced by this round

## Boundary Memory

- foundation remains business-agnostic
- retrieval enhancement stays inside `src/ops/foundation/retrieval_engine.rs`
- do not split `RetrievalConfig` yet
- do not let evidence-side tie-break signals override `source_ref` priority
- do not let diagnostics become a second scoring system
- do not let hygiene diagnostics turn into hidden ranking penalties
- do not wire retrieval enhancement into CLI / Tool / GUI at this stage
- do not reopen completed Tasks 8 to 11 for speculative refactoring

### Task 11 Layer 5.1

- locator hygiene now accepts common sheet-qualified and absolute A1-style locators
- supported locator examples now include:
  - `Sheet1!A1`
  - `Sheet1!A1:B3`
  - `'Sheet Name'!$A$1`
  - `'Sheet Name'!$A$1:$D$5`
- named ranges such as `RevenueNamedRange` still remain weak locators in this phase
- sheet-qualified large ranges still remain weak when parsed area exceeds the current fixed threshold
- locator normalization in this round only strips sheet prefixes and `$` markers
- this round still does not change hit creation
- this round still does not change ranking behavior

### Task 11 Layer 5.2

- weak source ref hygiene now also flags:
  - multi-token placeholder source refs such as `source data`
  - placeholder source refs with numeric suffixes such as `table 1`
- weak source ref hygiene still does not flag source names that contain clear business semantics
  - regression example kept non-weak: `sales detail sheet`
- this round still does not change hit creation
- this round still does not change ranking behavior

### Task 11 Layer 5.3

- weak source ref hygiene now also flags compact placeholder source refs with numeric suffixes
  - examples now include `sheet1`
- compact numeric suffix support in this round remains intentionally narrow
- the new shape is limited to `placeholder token + digits`
- this round still does not change hit creation
- this round still does not change ranking behavior

### Task 11 Layer 5.4

- locator hygiene now also accepts Windows absolute-path external workbook A1-style ranges
  - examples now include `C:\Reports\[Budget.xlsx]Sheet1!A1:B3`
  - examples now include `C:\Reports\[Budget.xlsx]'Sales Detail'!$A$1:$D$5`
- this round only fixes the range parsing boundary where drive letter `:` previously broke locator splitting
- named ranges still remain weak locators in this phase
- 3D references still remain outside the current supported locator contract
- this round still does not change hit creation
- this round still does not change ranking behavior

## Recommended Next Step

If the user only says "continue", stay on the foundation line.

The next preferred direction is:

1. continue small retrieval-side enhancements inside `retrieval_engine.rs`
2. prefer diagnostics-friendly refinements such as stronger locator/source hygiene heuristics or stable ranking introspection
3. keep avoiding config splitting until there is a proven second caller with conflicting needs
4. if a newly added protection test passes immediately, record it as a confirmed contract instead of forcing an unnecessary production-code change

## Task 11 Layer 5.5

- locator hygiene now also has explicit regression protection for Windows absolute-path external workbook large ranges
  - examples now explicitly kept weak:
    - `C:\Reports\[Budget.xlsx]Sheet1!A1:Z200`
    - `C:\Reports\[Budget.xlsx]'Sales Detail'!$A$1:$Z$200`
- this round adds protection tests only and does not change production code
- the new tests passed immediately, which confirms the current implementation already preserves the large-range weak threshold after the path-prefix parsing fix
- this round therefore locks in a two-layer boundary:
  - Windows path prefix external workbook locators can be parseable
  - parseable large ranges still remain weak locators when area exceeds the current fixed threshold
- named ranges still remain weak locators in this phase
- 3D references still remain outside the current supported locator contract
- verification updates:
  - `retrieval_engine_unit`: `29 passed`
  - foundation minimum regression set: `48 passed`
