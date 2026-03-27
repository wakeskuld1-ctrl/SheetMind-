# 2026-03-27 M3 Next Steps

## Purpose

This document is the handoff roadmap for the current disclosure-data track in `TradingAgents`.
It is intended to let another AI or engineer continue the work without re-discovering the current scope, constraints, and priorities.

## Current Status

As of 2026-03-27, the following slices are complete:

1. Unified disclosure contract and SQLite persistence
2. Live CNInfo A-share ingestion pipeline
3. SSE verification path against stored CNInfo events
4. Unified disclosure runner and CLI entry
5. Market-aware runner routing for SSE / SZSE / BSE symbols

## Implemented Files

- `tradingagents/dataflows/disclosure_models.py`
- `tradingagents/dataflows/disclosure_store.py`
- `tradingagents/dataflows/disclosure_cninfo.py`
- `tradingagents/dataflows/disclosure_sse_verifier.py`
- `tradingagents/disclosure_runner.py`
- `cli/disclosure.py`
- `tests/test_disclosure_store.py`
- `tests/test_cninfo_disclosure.py`
- `tests/test_sse_disclosure_verifier.py`
- `tests/test_disclosure_runner.py`

## Constraints

- Data collection must not use LLMs
- Data collection must not require tokens
- Prefer free official sources for A-share and HK disclosures
- User-facing delivery should move toward a low-barrier setup, ideally without requiring users to manage a Python environment
- Code changes should continue to follow TDD
- Every completed change must continue to update `CHANGELOG_TASK.MD`

## What Is Stable Now

- CNInfo security lookup, list paging, detail fetch, snapshot persistence, and SQLite ingestion are already working
- SSE official bulletin verification is working for SSE-routed tickers
- The runner now resolves ticker market route explicitly and no longer assumes `CN-SH`
- `run_summary.json` is stable enough to serve as the first fixed machine-readable run artifact

## What Is Not Done Yet

- No SZSE official verification source is connected yet
- No BSE official verification source is connected yet
- No HKEXnews live ingestion is connected yet
- No detailed mismatch artifact exists yet beyond aggregate summary counts
- No environment-free packaged desktop or executable distribution is produced yet
- The disclosure layer is not yet wired into the broader `tradingagents.dataflows.interface` path

## Recommended Next Priority Order

### M3-6

Add detailed verification artifacts for each run.

Recommended output:

- `run_summary.json` keeps aggregate counts
- add a second report for `matched`
- add a second report for `source_only`
- add a second report for `store_only`
- add SSE raw response snapshots for replay

Reason:

- This gives immediate debugging value without opening a new market
- It turns the current A-share path into something auditable and easier for later AI continuation

### M3-7

Add the first SZSE official verification source.

Reason:

- The runner can already route `CN-SZ`
- The biggest structural gap after M3-5 is that Shenzhen symbols still skip official verification

### M3-8

Decide whether BSE verification should be added before HKEXnews, or deferred.

Recommendation:

- If the user still prioritizes A-share completeness first, do BSE verification before HKEXnews
- If the user prioritizes market expansion, start HKEXnews after SZSE is stable

### M4

Move from “developer-first Python CLI” to “user-friendly packaging”.

Suggested scope:

- finalize fixed runtime directory policy
- expose a single launch entry
- test packaging candidates such as PyInstaller or Nuitka
- confirm SQLite, snapshots, and reports remain writable after packaging

## Testing Recommendations

For the next slice, prefer this order:

1. Unit test the route or parser behavior first
2. Add one integration test for runner output shape
3. Only then run live verification against the official source

Important existing regression command:

```bash
python -m pytest tests/test_disclosure_store.py tests/test_cninfo_disclosure.py tests/test_sse_disclosure_verifier.py tests/test_disclosure_runner.py
```

## Known Risks

- Market routing currently relies on ticker suffixes and common prefix rules; unusual symbol types may need source-confirmed routing later
- `verification_source=None` currently means “no supported verifier exists yet”; later this may need a more explicit status model
- Existing unrelated environment failures still exist in some repository tests due to `torch` DLL initialization on this machine

## Handoff Notes

- Current working branch used for this line is `codex/m3-disclosure-foundation`
- Important local commits already created in this branch:
  - `00e358b feat(disclosure): add M3 disclosure ingestion and runner foundation`
  - `ac92fff fix(disclosure): add market-aware runner routing`
- There is one unrelated local untracked note file not part of the disclosure handoff:
  - `docs_tradingagents_agent_prompt_data_notes.md`

## Suggested Next Session Opening Prompt

Use the roadmap plus these records first:

- `task_plan.md`
- `progress.md`
- `findings.md`
- `CHANGELOG_TASK.MD`
- `docs/plans/2026-03-27-m3-next-steps.md`

Then continue with:

1. Confirm the next approved slice with the user
2. Add a failing test first
3. Keep the change set narrow
4. Update the handoff files again before stopping
