# Financial Decision Workbench Design on Top of SheetMind

## Goal

Design a lightweight but serious financial decision workbench where:

- `Skill` is responsible for thinking, routing, explanation, and challenge
- `Tool` is responsible for execution, retrieval, calculation, and artifact generation
- `Decision Layer` is responsible for structured decision cards, risk gates, and human approval

The key question is not whether SheetMind is already a finance product. It is not. The key question is whether its product skeleton can be reused. The answer is yes.

## 1. Product thesis

The target product should behave more like a standard investment firm's operating workflow than like a free-form multi-agent chatbot.

A practical operating chain is:

1. ingest research inputs and market context
2. build or refresh structured evidence
3. generate a draft view
4. test the view against risk and portfolio constraints
5. require explicit human approval before execution or publication
6. deliver a report, watchlist update, or execution candidate

This is exactly where a `Skill + Tool + Decision Layer` architecture fits.

## 2. Target system architecture

### Layer A: User Workflow Layer

This is the product surface.

Typical user requests:

- analyze a stock or theme
- explain what changed since yesterday
- tell me whether this idea is actionable
- show the main risks blocking action
- prepare an investment memo or PM summary

This layer should not expose raw tool JSON. It should show:

- current decision status
- evidence summary
- blocked / ready state
- required next action
- final approval status

### Layer B: Skills Layer

This layer should contain finance-specific thinking modules, not personified agents.

Recommended first skills:

- `research-orchestrator-skill`: routes between evidence collection, synthesis, risk review, and approval prep
- `thesis-skill`: turns a user request into one or more hypotheses and required evidence asks
- `evidence-synthesis-skill`: merges market, news, fundamental, and portfolio evidence into a draft decision card
- `risk-challenge-skill`: identifies invalidation conditions, concentration issues, event risks, and hidden assumptions
- `approval-brief-skill`: translates the final card into a short human-facing approval brief

These should follow the same discipline SheetMind already uses: Skills do not compute; they orchestrate and explain.

### Layer C: Tools Layer

This layer should do all real work.

Finance-native tools should include:

- market data tool
- fundamentals tool
- earnings / estimate revision tool
- news / filings / event extraction tool
- valuation tool
- factor exposure tool
- risk snapshot tool
- portfolio constraint tool
- scenario stress tool
- trade cost / liquidity tool
- report delivery tool

This is the layer where SheetMind is already strong architecturally.

### Layer D: Decision Layer

This is the missing layer that makes the product feel like an investment workflow instead of a chat experience.

It contains three parts:

- `structured decision card`
- `risk gates`
- `human approval`

The Decision Layer should be deterministic wherever possible. It should consume structured tool outputs and structured skill outputs, then decide:

- is this trade only an idea?
- is it research-ready?
- is it decision-ready?
- is it blocked?
- can it be approved for watchlist / paper-trade / live-trade candidate?

## 3. What SheetMind already gives you

SheetMind already contains several reusable product capabilities.

### 3.1 Reusable orchestration pattern

The repo already has a strong product pattern:

- top-level orchestrator Skill
- specialized child Skills
- local session memory
- tool dispatch through a single Rust runtime

Evidence:

- top-level orchestrator routes between sub-skills in `E:\Excel\SheetMind-\skills\excel-orchestrator-v1\SKILL.md:41`
- orchestrator explicitly reads and writes session state through `get_session_state` and `update_session_state` in `E:\Excel\SheetMind-\skills\excel-orchestrator-v1\SKILL.md:76`
- Rust binary entrypoint reads JSON from stdin and dispatches tools in `E:\Excel\SheetMind-\src\main.rs:10` and `E:\Excel\SheetMind-\src\main.rs:35`

This pattern maps cleanly to a finance workbench.

### 3.2 Reusable local memory and handle model

SheetMind already has a real local state model, not just prompt memory.

Useful pieces:

- stage tracking
- schema/status tracking
- active handle tracking
- session patch updates
- SQLite-based persistence

Evidence:

- session stage enum in `E:\Excel\SheetMind-\src\runtime\local_memory.rs:15`
- session state object in `E:\Excel\SheetMind-\src\runtime\local_memory.rs:79`
- active handle fields in `E:\Excel\SheetMind-\src\runtime\local_memory.rs:91`
- state read/write in `E:\Excel\SheetMind-\src\runtime\local_memory.rs:201` and `E:\Excel\SheetMind-\src\runtime\local_memory.rs:245`

For finance, `table_ref` should be generalized into something like:

- `dataset_ref`
- `signal_ref`
- `decision_ref`
- `report_ref`
- `approval_ref`
- optionally `order_candidate_ref`

This is a major reuse opportunity.

### 3.3 Reusable decision-assistant contract

This is the single most important reusable concept.

SheetMind already has a structured decision assistant output with:

- blocking risks
- priority actions
- next tool suggestions
- human summary

Evidence:

- `BlockingRisk` in `E:\Excel\SheetMind-\src\ops\decision_assistant.rs:11`
- `PriorityAction` in `E:\Excel\SheetMind-\src\ops\decision_assistant.rs:22`
- `NextToolSuggestion` in `E:\Excel\SheetMind-\src\ops\decision_assistant.rs:32`
- `DecisionAssistantResult` in `E:\Excel\SheetMind-\src\ops\decision_assistant.rs:49`
- builder entrypoint in `E:\Excel\SheetMind-\src\ops\decision_assistant.rs:71`

This can evolve almost directly into a finance Decision Layer.

Instead of:

- missing columns
- duplicate rows
- table quality risks

You would check:

- missing critical market or event data
- stale price or fundamental inputs
- concentration violations
- liquidity limits
- macro event blackout windows
- confidence below threshold
- thesis/evidence contradiction

### 3.4 Reusable delivery layer

SheetMind already has report assembly, chart building, and workbook export.

Evidence:

- tool catalog exposes `report_delivery`, `build_chart`, and `export_excel_workbook` in `E:\Excel\SheetMind-\src\tools\contracts.rs:93`
- report delivery request and chart specs are in `E:\Excel\SheetMind-\src\ops\report_delivery.rs:45` and `E:\Excel\SheetMind-\src\ops\report_delivery.rs:106`
- workbook draft construction is in `E:\Excel\SheetMind-\src\ops\report_delivery.rs:134`
- dispatcher integration is in `E:\Excel\SheetMind-\src\tools\dispatcher.rs:870`

For finance, this is ideal for generating:

- PM briefing packs
- daily risk review workbooks
- watchlist update workbooks
- committee-style decision packs

### 3.5 Reusable lightweight analytics tools

SheetMind already includes generic analytics modules such as:

- correlation analysis
- distribution analysis
- outlier detection
- trend analysis
- regression and clustering

Evidence:

- correlation tool exposed in `E:\Excel\SheetMind-\src\tools\contracts.rs:114` and implemented through dispatcher at `E:\Excel\SheetMind-\src\tools\dispatcher.rs:2029`
- trend tool exposed in `E:\Excel\SheetMind-\src\tools\contracts.rs:120` and implemented through dispatcher at `E:\Excel\SheetMind-\src\tools\dispatcher.rs:2160`

These are not enough for production finance, but they are useful as early-stage research helpers.

## 4. What should be reused directly vs adapted vs replaced

### Reuse directly

These product patterns are already good and should be reused with minimal structural change:

- binary-first tool runtime
- JSON request/response tool contract
- single dispatcher model
- local session state and handle persistence
- orchestrator Skill pattern
- decision-assistant output shape
- report / chart / workbook delivery layer

### Adapt heavily

These concepts are good but need finance-specific semantics:

- `table_ref` -> `dataset_ref`, `signal_ref`, `decision_ref`
- `table_health` -> `research_readiness` or `decision_readiness`
- `blocking_risks` -> `risk_gates`
- `priority_actions` -> `required_next_actions`
- `next_tool_suggestions` -> `recommended_evidence_or_checks`

### Replace or deprioritize

These are not central to the finance product's first serious version:

- Excel-specific header confirmation flow
- sheet range inspection as a core product path
- generic clustering as a primary value proposition
- generic table cleaning as the center of user experience

They may still exist, but they should not define the finance architecture.

## 5. Finance Decision Layer design

### 5.1 Structured decision card

Every candidate idea should become a standard decision object.

Recommended fields:

- `decision_id`
- `asset_id`
- `instrument_type`
- `timestamp`
- `strategy_type`
- `horizon`
- `direction` (`long`, `short`, `hedge`, `no_trade`)
- `expected_return_range`
- `downside_risk`
- `confidence_score`
- `position_size_suggestion`
- `evidence_refs`
- `key_supporting_points`
- `key_risks`
- `invalidation_conditions`
- `portfolio_impact`
- `current_status`

Recommended statuses:

- `draft`
- `needs_more_evidence`
- `blocked`
- `ready_for_review`
- `approved`
- `rejected`
- `approved_with_override`

### 5.2 Risk gates

The first finance version should have explicit, rules-based gates.

Recommended gates:

- `data_freshness_gate`
- `minimum_evidence_gate`
- `event_blackout_gate`
- `liquidity_gate`
- `position_limit_gate`
- `sector_or_theme_concentration_gate`
- `correlation_exposure_gate`
- `confidence_gate`
- `valuation_extreme_gate`
- `portfolio_drawdown_protection_gate`

Each gate should output:

- gate name
- pass / warn / fail
- reason
- blocking or non-blocking
- possible remediation

This is the finance equivalent of SheetMind's `blocking_risks` plus `priority_actions`.

### 5.3 Human confirmation

The human review step should not be a single yes/no button. It should be a tracked decision event.

Recommended review actions:

- `approve`
- `reject`
- `request_more_evidence`
- `approve_with_override`

For every approval event, persist:

- approver
- timestamp
- reviewed decision card version
- override reason if any
- notes

This fits naturally with SheetMind's local-memory and handle architecture.

## 6. Suggested finance-native tool set

The first serious tool catalog should include at least these groups.

### Evidence tools

- `get_market_snapshot`
- `get_price_history`
- `get_returns_regime`
- `get_fundamental_snapshot`
- `get_estimate_revision_signal`
- `get_news_event_summary`
- `get_filing_summary`
- `get_sentiment_snapshot`

### Analysis tools

- `compute_valuation_band`
- `compute_factor_exposure`
- `compute_risk_snapshot`
- `compute_drawdown_profile`
- `compute_scenario_stress`
- `compute_portfolio_impact`
- `compute_trade_cost`
- `compute_liquidity_limit`

### Decision tools

- `build_decision_card`
- `run_risk_gates`
- `build_approval_brief`
- `record_approval_event`
- `promote_to_watchlist`
- `promote_to_order_candidate`

### Delivery tools

- `build_pm_pack`
- `build_risk_review_pack`
- `export_decision_workbook`
- `build_decision_chart`

## 7. Best reuse path from SheetMind to finance

The fastest path is not to rewrite everything. It is to preserve the product kernel and swap the domain.

### Step 1: keep the runtime kernel

Keep:

- stdin JSON request -> dispatcher -> structured JSON response
- local SQLite session state
- handle refs
- report delivery

### Step 2: replace the domain vocabulary

Replace Excel vocabulary with finance vocabulary.

Examples:

- `table_ref` -> `dataset_ref`
- `analysis_modeling` -> `research_analysis`
- `decision_assistant` -> `decision_layer`
- `table_health` -> `decision_readiness`

### Step 3: add finance tool families

Introduce finance-native evidence and risk tools while keeping the existing tool-catalog and dispatch model.

### Step 4: upgrade the decision assistant into a real decision layer

The current `decision_assistant` already has the right shape, but it needs to evolve from:

- data-quality guidance

to:

- investability and execution-readiness guidance

### Step 5: reuse report delivery for investment memos

Instead of table-analysis outputs, feed:

- decision card summary sheet
- evidence summary sheet
- risk gate sheet
- chart sheet
- approval log sheet

This can become a very strong lightweight product artifact.

## 8. Recommended MVP

If the goal is a lightweight product, the first MVP should not try to auto-trade.

Recommended MVP scope:

- single-instrument decision workspace
- research-orchestrator Skill
- thesis-skill
- evidence-synthesis-skill
- risk-challenge-skill
- finance tool catalog for market snapshot, event summary, valuation snapshot, risk snapshot, and portfolio impact
- structured decision card
- 5 to 7 rules-based risk gates
- human review state machine
- workbook / chart report delivery

This would already be far more serious than a generic LLM trading bot, while staying much lighter than a full institutional stack.

## 9. Final recommendation

Yes, some of the capabilities in `E:\Excel\SheetMind-` are not only reusable, they are strategically valuable.

The most reusable assets are not the Excel-specific tools themselves. They are:

- the binary-first product runtime
- the Skill-vs-Tool boundary discipline
- the local session memory model
- the reference/handle lineage model
- the structured decision-assistant output pattern
- the report delivery system

My recommendation is:

- do **not** build the finance product as another free-form multi-agent repo
- do **reuse** SheetMind's runtime kernel and product conventions
- do **evolve** the current decision-assistant shape into a finance Decision Layer
- do **add** finance-native evidence, valuation, risk, portfolio, and approval tools

In short:

SheetMind already gives you the skeleton of a serious local decision product. What it lacks is the finance domain layer, not the product architecture.
