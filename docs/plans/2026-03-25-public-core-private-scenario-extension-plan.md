# Public Core + Private Scenario Extension Plan

## 1. Design intent

The right way to understand the new system is:

- `SheetMind-` remains the public foundational project
- the new large analysis system is an extension built on top of it
- open-source stays focused on generic infrastructure and reusable primitives
- private value stays in scenario packs, scene-specific decision logic, and internal workflows

So this is not:

- a separate project that happens to borrow a few files
- or a hard fork that mixes public and private logic into one inseparable tree

It is:

- a layered architecture
- with a public kernel below and a private scenario layer above

This is the key strategic boundary.

## 2. Recommended architecture in one sentence

Keep `SheetMind-` as the public binary-first analysis kernel, then build a private extension layer that adds:

- private Skills for scenario understanding and orchestration
- private composite Tools for scenario execution
- a private Decision Layer for scene-specific judgment, gating, and approval flow

## 3. What should stay public

The public layer should contain only domain-agnostic or widely reusable capabilities.

### 3.1 Runtime kernel

Keep public:

- binary-first Rust runtime
- tool request / response contract
- dispatcher model
- local memory runtime
- handle reference system
- report / chart / export infrastructure

These are product infrastructure, not proprietary edge.

### 3.2 Generic Skills

Keep public:

- orchestrator skill pattern
- table / dataset processing skills
- generic analysis-modeling skills
- generic decision-assistant pattern

The value here is the product shell and user interaction discipline.

### 3.3 Generic Tools

Keep public:

- data loading and normalization
- filtering, joins, groupings, pivots
- generic stats, trend, correlation, clustering, regression
- chart building and workbook delivery
- session state tools

These are primitives. They are your platform layer.

## 4. What should stay private

The private layer should contain your true edge.

### 4.1 Scenario packages

A scenario package is a private business analysis module for a repeatable real-world workflow.

Examples:

- event-driven equity decision package
- earnings preview package
- industry chain shock package
- inventory / pricing signal package
- portfolio rebalance package
- macro regime package

Each scenario package contains:

- one or more private Skills
- one or more private composite Tools
- private decision-card schema extensions
- private gating rules
- private report templates

### 4.2 Proprietary heuristics and judgment rules

Keep private:

- how you define readiness for a scenario
- how you rank evidence
- how you weight conflicting signals
- how you score conviction
- how you define scenario-specific risk gates
- how you decide what is approval-worthy

This is where your real edge lives.

### 4.3 Internal workflow and operating memory

Keep private:

- private watchlists
- portfolio context
- PM review notes
- approval logs
- private event calendars
- scenario-specific memory and feedback loops

## 5. How to interpret “merge a Tool into a Skill” correctly

This phrase is useful conceptually, but I do not recommend literally collapsing Tool code into Skill files.

The better engineering model is:

- `Skill` remains the thinking and orchestration layer
- but the Skill can call a new `composite Tool` that already bundles multiple low-level actions

So what feels like “Tool merged into Skill” in product behavior should actually be:

- a private scenario Skill
- calling a private scenario Tool
- where the scenario Tool internally composes several generic public Tools

### Example

Instead of exposing all these steps to the user:

- fetch market data
- compute valuation band
- check event calendar
- summarize recent news
- compute position impact
- run risk gates

You create one private Tool such as:

- `run_earnings_precheck`

That private Tool internally uses the public kernel and returns a structured result.

Then the private Skill:

- interprets the result
- decides whether more evidence is needed
- writes the approval brief

This gives you two advantages:

- the user experience is simple
- your scene logic is not exposed as a pile of public primitives

## 6. Recommended module structure

I recommend a two-repo or two-layer workspace model.

### Option A: Hard fork mixed together

Structure:

- public and private code in the same repository
- internal modules mixed under the same tree

Pros:

- fastest at the very beginning
- simplest local wiring

Cons:

- private/public boundary gets messy fast
- hard to open-source cleanly
- easy to leak scene logic into the public repo
- long-term maintenance becomes painful

I do not recommend this except as a temporary prototype.

### Option B: Public core repo + private overlay repo (recommended)

Structure:

- public repo: `SheetMind-`
- private repo: something like `SheetMind-Scenes` or `SheetMind-Private`
- private repo depends on public repo as path dependency or git dependency

Pros:

- clean IP boundary
- public repo stays clean and reusable
- private repo can evolve fast without polluting the open layer
- release model is much easier to control

Cons:

- build wiring is slightly more work
- need discipline around version compatibility

This is the best balance.

### Option C: Plugin marketplace style

Structure:

- public binary loads external private plugins dynamically

Pros:

- strongest separation in theory
- future multi-package ecosystem possible

Cons:

- highest engineering complexity
- plugin ABI/versioning/security burden
- overkill for current stage

I do not recommend this for phase 1.

## 7. Recommended repository model

### Public repo: `SheetMind-`

This repo should provide:

- runtime
- dispatcher
- local memory
- handle stores
- generic tools
- generic skills
- generic report delivery
- public schemas and contracts

### Private repo: `SheetMind-Scenes`

This repo should provide:

- finance-specific or scenario-specific Skills
- private composite Tools
- private Decision Layer
- private templates
- internal configs and approval policies
- scene-specific data adapters

The private repo should either:

- build a separate internal binary on top of the public crate
- or extend the public binary via compile-time feature inclusion

My recommendation is a separate internal binary first. That keeps release boundaries clean.

## 8. Public core vs private extension at runtime

### Public core runtime responsibilities

The public core should do only these things:

- execute tools
- persist state
- manage handles and lineage
- export artifacts
- provide generic analysis primitives

### Private extension responsibilities

The private extension should do these things:

- decide which scenario package to activate
- call private scene Tools
- build private decision cards
- run private risk gates
- create approval objects
- route into private reporting flows

This keeps the public core neutral and the private layer opinionated.

## 9. Evolving the current Skill architecture

The current `orchestrator -> child skill -> tool` model is already right.

The next evolution should be:

- `public orchestrator skill`
- `private scenario router skill`
- `private scenario skill`
- `private composite tool`
- `public primitive tools`

So the hierarchy becomes:

1. public entry skill decides generic route
2. if a private scenario is requested, hand off to a private scenario router
3. the private scenario router selects a scenario package
4. the scenario Skill calls one or more private composite Tools
5. those composite Tools use public primitives under the hood
6. the Decision Layer returns a reviewable decision object

## 10. How the Decision Layer fits into the private extension

The Decision Layer should live in the private extension, not the public core.

Reason:

- the generic shape can be public
- but the real gates, thresholds, judgment policy, and approval semantics are proprietary

### Publicly reusable part

You can keep public:

- a generic decision result contract shape
- a generic blocking risk structure
- a generic priority action structure
- a generic next-step structure

This mirrors the existing `decision_assistant` pattern.

### Private part

Keep private:

- finance decision card schema
- scenario-specific readiness rules
- gate thresholds
- confidence scoring logic
- approval and override policy

## 11. How to package scenario logic

I recommend a standard package format for every private scene.

Each scene package should include:

- `SCENE.md` or `SKILL.md` for scene reasoning rules
- `scene_tools.rs` or equivalent module
- `scene_contracts.rs` for inputs/outputs
- `scene_decision.rs` for card and gate logic
- optional `scene_reports.rs` for delivery formatting
- optional `scene_memory.rs` for feedback persistence

A scene package should be installable into the private binary as a bounded module, not spread loosely across the codebase.

## 12. Concrete example of the extension pattern

### Public tools

These stay open:

- `trend_analysis`
- `correlation_analysis`
- `report_delivery`
- `build_chart`
- `get_session_state`
- `update_session_state`

### Private composite tool

Example private tool:

- `scene_equity_event_precheck`

Internally this tool may:

- fetch price and trend state
- fetch valuation snapshot
- fetch event calendar
- fetch news summary
- compute concentration and liquidity checks
- produce one scene result object

### Private scene skill

Example private skill:

- `equity-event-scene-skill`

This skill may:

- inspect user goal
- call `scene_equity_event_precheck`
- decide whether to request more evidence
- produce a draft decision card
- hand off to approval brief generation

This is the right form of “merging Tool into Skill” from a user point of view, while still preserving clean architecture.

## 13. Suggested implementation path

### Phase 1: Protect the public kernel

Do not heavily rewrite `SheetMind-` first.

Instead:

- freeze the public kernel boundary
- define which modules are guaranteed public
- define which extension points private repos may use

### Phase 2: Define extension contracts

Add clear public contracts for:

- handle refs
- session state extension fields
- tool registration hooks or build-time registry inclusion
- report artifact contracts
- generic decision result envelope

### Phase 3: Build the private scenario repo

Create:

- private scene registry
- private scene skills
- first private composite tool
- first private decision card and gate set

### Phase 4: Build one end-to-end scenario

Do only one real scenario first.

That is the fastest way to validate the architecture.

### Phase 5: Generalize the scene package format

Once one scene works end-to-end, standardize:

- naming
- contracts
- approval states
- report outputs
- memory feedback

## 14. Recommended first private scenario

Pick a scenario that is:

- high-frequency in your own workflow
- structurally repeatable
- evidence-heavy
- worth protecting as internal edge

Good examples:

- event-driven single-name review
- portfolio rebalance review
- earnings pre-brief
- macro shock impact brief

Do not start with a vague “universal intelligence layer”. Start with one scene that saves you time every week.

## 15. Final recommendation

Your strategic direction makes sense.

The right structure is:

- `SheetMind-` is the public foundation
- your new project is an upper-layer extension on top of it
- public keeps the generic kernel and reusable primitives
- private keeps scenario reasoning, scene tools, gates, approval policy, and operating workflow

And the right way to handle “Tool merged into Skill” is:

- not by mixing execution code into skill markdown or routing logic
- but by creating private composite Tools that make the Skill feel scene-aware and compact

In short:

- open-source the engine
- keep the scenarios private
- keep Skills as brains
- keep Tools as hands
- let the private Decision Layer be your investment-process moat
