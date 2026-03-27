# Findings

## Research Notes
- The current project has `yfinance` and `alpha_vantage` dataflow routing, but no disclosure-event abstraction for A-share or HK announcement data.
- `pyproject.toml` does not declare an ORM dependency even though `uv.lock` contains `sqlalchemy` and `peewee`, so standard-library `sqlite3` is the safest first persistence choice.
- Existing tests are minimal and use `unittest`, while `pytest` is available and can discover those tests.
- A clean M3-1 seam is to add disclosure-specific modules under `tradingagents/dataflows` without touching existing market/news/fundamental routing yet.
- On this machine, some pre-existing test imports transitively load `torch` through `langchain_openai`, which currently fails DLL initialization; targeted verification must avoid conflating that environment issue with the new disclosure changes.
- CNInfo's current search stack still exposes usable JSON endpoints behind the web app: `/new/information/topSearch/detailOfQuery`, `/new/hisAnnouncement/query`, and `/new/announcement/bulletin_detail`.
- The detail page script `notice-detail.js` confirms `bulletin_detail` is the stable path for attachment URL retrieval, which is cleaner than scraping the rendered HTML page.
- A live fetch against ticker `600519` on 2026-03-27 returned two announcements in the 2026-03-01 to 2026-03-27 window, proving the CNInfo-only chain can already ingest real data into SQLite.
- SSE's latest-announcement page currently drives the official query endpoint `https://query.sse.com.cn/security/stock/queryCompanyBulletinNew.do`, returning JSONP rather than plain JSON.
- The SSE payload shape nests each bulletin row inside a one-element list under `result`, so flattening is required before comparison.
- A live SSE verification against stored CNInfo rows for ticker `600519` over `2026-03-01` to `2026-03-27` produced a full match set: 2 matched rows, 0 SSE-only rows, and 0 CNInfo-only rows.
- Typer will collapse a single-command application into that command unless the app is forced to stay as a group; this breaks a future packaged entry when users invoke `disclosure run ...`.
- A stable M3-4 seam is now in place: one runner function owns runtime-path selection, SQLite location, snapshot root, and JSON summary emission, which keeps later no-Python packaging work isolated from fetcher internals.
