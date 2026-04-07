# Task Plan

## Goal
Build the first M3 disclosure-data foundation for TradingAgents: a unified disclosure contract plus SQLite persistence for future A-share and HK disclosure ingestion, then land the first real CNInfo fetch pipeline and the first SSE cross-check path without using LLMs or tokens in the data pipeline.

## Phases
- [x] Phase 1: Inspect current dataflow structure, test entrypoints, and local persistence options
- [x] Phase 2: Add failing tests for disclosure normalization, dedupe key generation, and SQLite round-trip storage
- [x] Phase 3: Implement the minimal disclosure model and SQLite store to satisfy the tests
- [x] Phase 4: Implement the first CNInfo list/detail fetch pipeline with snapshot persistence
- [x] Phase 5: Run targeted verification, execute one live fetch, and update task records
- [x] Phase 6: Implement the first SSE bulletin verification path against stored CNInfo events
- [x] Phase 7: Add the M3-4 unified disclosure runner and CLI entry oriented to future environment-light packaging
- [x] Phase 8: Add market-aware routing so the unified runner no longer assumes every disclosure ticker is CN-SH

## Notes
- This slice intentionally excludes HKEXnews live crawling and SSE formal event ingestion.
- Persistence uses Python standard-library `sqlite3` first to avoid unnecessary ORM expansion in M3-1.
- Cross-source dedupe is currently conservative: ticker + normalized title + hourly bucket + document filename.
- The first live fetch path is now CNInfo-only: security lookup -> historical announcement list -> bulletin detail -> snapshot JSON -> SQLite event row.
- The first SSE path is verification-only: SSE bulletin list -> normalized comparable rows -> compare with stored CNInfo events.
- The current handoff roadmap for the next slices is documented in `docs/plans/2026-03-27-m3-next-steps.md`.

## Errors Encountered
| Error | Attempt | Resolution |
|---|---:|---|
| `sqlite3.Connection` remained open on Windows temporary databases | 1 | Replaced transaction-only context manager usage with `contextlib.closing(sqlite3.connect(...))` so handles are explicitly closed |
| `normalize_disclosure_ticker()` did not support `CN-BJ` when the CNInfo client introduced Beijing exchange mapping | 1 | Added a failing test first, then extended ticker normalization to emit `.BJ` |
| SSE bulletin API returns JSONP and nests each row inside a one-element list | 1 | Added a failing parser/normalizer test first, then implemented JSONP unwrapping and row flattening |
| Existing `tests/test_ticker_symbol_handling.py` import path triggers `torch` DLL initialization failure through `langchain_openai` in this machine environment | 1 | Logged as pre-existing environment issue; scoped verification to the new disclosure tests for this task |
| Typer collapsed the disclosure CLI into a single-command app, so invoking `run` was treated as an unexpected extra argument | 1 | Reproduced with the existing failing CLI test first, then added an explicit root callback to keep the packaged interface in command-group form |
| The unified runner hard-coded `CN-SH` during verification, so Shenzhen and Beijing tickers would be normalized against the wrong market and could incorrectly hit the SSE verifier | 1 | Added failing runner tests first, then introduced explicit market-route resolution and limited SSE verification to SSE-routed tickers |
