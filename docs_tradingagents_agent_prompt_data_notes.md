# TradingAgents: Agent, Prompt, and Data Handling Notes

## 1. Overall architecture

TradingAgents is a LangGraph-based multi-agent pipeline, not a free-form chat swarm.
The real execution order is:

1. Selected analyst nodes run one by one.
2. Bull and Bear researchers debate.
3. Research Manager produces an investment plan.
4. Trader turns that plan into a transaction proposal.
5. Aggressive / Conservative / Neutral risk agents debate.
6. Portfolio Manager emits the final rating and decision.

This is assembled in `tradingagents/graph/setup.py` and orchestrated by `TradingAgentsGraph` in `tradingagents/graph/trading_graph.py`.

## 2. How the agents are organized

### 2.1 Graph entrypoint

`TradingAgentsGraph` is the top-level orchestrator.
It does four important things:

- loads runtime config and pushes it into the dataflow layer;
- creates two LLM clients: a `deep_thinking_llm` and a `quick_thinking_llm`;
- creates per-role memory objects;
- builds LangGraph nodes and executes the graph.

By default:

- provider = `openai`
- deep model = `gpt-5.2`
- quick model = `gpt-5-mini`
- debate rounds = 1
- risk debate rounds = 1
- data vendors = `yfinance`

So the shipped default is actually a fairly short pipeline with one round of investment debate and one round of risk debate.

### 2.2 Shared state model

The whole workflow passes around one `AgentState` object.
Important state fields:

- `messages`: LangGraph message history for the active tool-calling analyst;
- `market_report`, `sentiment_report`, `news_report`, `fundamentals_report`: analyst outputs;
- `investment_debate_state`: bull/bear debate transcript, last reply, count, judge result;
- `trader_investment_plan`: trader output;
- `risk_debate_state`: aggressive/conservative/neutral debate transcript and count;
- `final_trade_decision`: final portfolio manager result.

The system does not use structured JSON outputs between agents.
Almost everything is passed as plain text reports embedded in state.

### 2.3 Analyst layer

The first layer contains up to four analyst agents:

- Market Analyst
- Social Media Analyst
- News Analyst
- Fundamentals Analyst

These are the only agents that use LangChain tool binding directly.
Each analyst is implemented as a node function that:

1. builds an inline prompt;
2. binds a small set of tools;
3. invokes the LLM against `state["messages"]`;
4. if the result has tool calls, LangGraph routes to the tool node;
5. if there are no tool calls, the content becomes that analyst's final report.

So the analysts behave like ReAct-style tool users inside LangGraph.

### 2.4 Debate / decision layer

After the analysts finish, the system switches from tool-using agents to pure prompt-to-text agents:

- Bull Researcher and Bear Researcher debate using the four analyst reports as evidence.
- Research Manager reads the debate and writes a combined investment plan.
- Trader converts that plan into a final transaction proposal.
- Risk debators challenge the trader plan from aggressive, conservative, and neutral viewpoints.
- Portfolio Manager synthesizes the risk debate and emits the final rating.

These later agents do not call tools. They only consume prior text outputs from state.

## 3. How prompts are handled

### 3.1 Prompts are hardcoded in Python

There is no external prompt registry, YAML prompt store, or template directory.
Prompts live inline inside the agent source files as either:

- `ChatPromptTemplate` + `MessagesPlaceholder` for analyst agents, or
- raw Python f-strings for debate / manager / trader / reflection agents.

That means prompt changes require code edits.

### 3.2 Analyst prompt structure

The analyst prompts follow a consistent pattern:

- a generic collaboration/tool-use system preamble;
- a role-specific `system_message` describing the task;
- current date;
- instrument context telling the model to preserve the exact ticker;
- tool names;
- `MessagesPlaceholder` so prior tool-call exchanges stay in the loop.

Important detail: each analyst prompt is designed to drive tool use first, then produce a long markdown report.
Examples:

- Market Analyst asks the model to choose relevant indicators, call `get_stock_data` first, then call `get_indicators`, then write a detailed report with a markdown table.
- Social Analyst asks for company-specific news/social sentiment over the last week.
- News Analyst asks for macro + company-relevant news.
- Fundamentals Analyst asks for company overview plus statements.

### 3.3 Debate and manager prompts

The later prompts are much simpler architecturally:

- Bull / Bear prompts take the four analyst reports, prior debate history, opponent's last reply, and retrieved memory snippets.
- Research Manager prompt asks for a decisive recommendation and investment plan.
- Trader prompt explicitly requires `FINAL TRANSACTION PROPOSAL: **BUY/HOLD/SELL**` at the end.
- Risk prompts tell each risk persona to argue against the others using the trader decision plus analyst reports.
- Portfolio Manager prompt requires one of five ratings: Buy / Overweight / Hold / Underweight / Sell.

These prompts are plain-text composition prompts, not tool-using prompts.

### 3.4 Reflection prompt

There is also a separate reflection prompt in `tradingagents/graph/reflection.py`.
It is only used if `reflect_and_remember()` is called after a run.
That reflection prompt asks the LLM to review whether a decision was correct, explain mistakes, propose improvements, summarize lessons, and create a condensed insight for memory.

## 4. How data is handled

### 4.1 Tool wrappers are thin adapters

All analyst tools are defined in `tradingagents/agents/utils/*.py`.
These wrappers are very thin:

- they define LangChain `@tool` signatures;
- then forward to `route_to_vendor()` in `tradingagents/dataflows/interface.py`.

So the real data abstraction is:

Agent -> tool wrapper -> vendor router -> vendor-specific implementation.

### 4.2 Vendor routing layer

The routing layer groups tools into categories:

- `core_stock_apis`
- `technical_indicators`
- `fundamental_data`
- `news_data`

Config can choose a vendor at:

- category level via `data_vendors`, or
- individual tool level via `tool_vendors`.

The router builds a fallback chain. If Alpha Vantage throws `AlphaVantageRateLimitError`, it falls through to another available vendor.

In practice, the default config points everything to `yfinance`.

### 4.3 Data formats passed to the LLM

A key implementation choice: tool results are mostly returned as strings, not typed data objects.
Examples:

- stock price history is returned as CSV text with a short header;
- balance sheet / cash flow / income statement are returned as CSV text;
- fundamentals are returned as a labeled text block;
- news is returned as a markdown-ish text list with title, summary, source, and link;
- technical indicators are returned as formatted text over a date window.

So the LLM is expected to read semi-structured text, not structured tables or JSON objects.
This makes the system simple, but also means every downstream reasoning step depends on the model parsing text correctly.

### 4.4 yfinance is the real default data backend

With the shipped defaults, data comes from yfinance-based implementations:

- price history via `Ticker.history()` or `yf.download()`;
- fundamentals via `Ticker.info`;
- statements via `quarterly_balance_sheet`, `quarterly_cashflow`, `quarterly_income_stmt`, etc.;
- ticker news via `stock.get_news(count=20)`;
- global news via `yf.Search(...)` with a small fixed set of macro queries.

This means the data side is heavily dependent on Yahoo Finance response formats.

### 4.5 Indicator caching

Technical indicator calculation has a notable optimization:

- `_get_stock_stats_bulk()` downloads or loads a long historical CSV;
- stores it under `dataflows/data_cache`;
- computes the requested stockstats indicator in bulk;
- then formats a date-by-date string for the requested lookback window.

So among the data paths, indicator calculation is the most explicitly cached.

### 4.6 Date handling

Dates are passed around as strings such as `YYYY-MM-DD`.
Analyst prompts receive `trade_date`, and tools typically accept either:

- `start_date` / `end_date`, or
- `curr_date` + `look_back_days`.

The code does not maintain a unified market calendar abstraction.
Missing trading days are just rendered as `N/A: Not a trading day (weekend or holiday)` in indicator output.

## 5. How memory is handled

### 5.1 Memory is lexical BM25, not vector DB

The project uses `FinancialSituationMemory`, which stores past situations and recommendations in memory and retrieves them with BM25 lexical matching.
There is:

- no embedding model;
- no external vector store;
- no persistence layer by default;
- no cross-session storage.

So this is lightweight retrieval over plain text snapshots.

### 5.2 What gets stored in memory

The memory key is effectively the concatenation of:

- market report
- sentiment report
- news report
- fundamentals report

That combined string is treated as the current situation.
Each agent retrieves the top 1-2 similar prior situations and injects only the recommendation text back into its prompt.

### 5.3 Reflection is opt-in and in-process

Memories are only updated when `reflect_and_remember(returns_losses)` is called.
That means:

- a normal `propagate()` run does not automatically learn;
- reflection requires an external returns/losses signal;
- memory lives only in the Python process unless the caller builds persistence around it.

This is best understood as ephemeral post-trade learning, not a durable long-term memory system.

## 6. How message flow is controlled

### 6.1 Analyst loop

For each analyst:

- if the last LLM message includes tool calls, LangGraph routes to that analyst's tool node;
- tool results are fed back into the same analyst;
- once the LLM stops requesting tools, the report is considered done.

After an analyst finishes, `create_msg_delete()` clears the message list and inserts a placeholder human message `Continue`.
This prevents the next analyst from inheriting the full previous analyst tool transcript.
The comment explicitly says this is for Anthropic compatibility.

### 6.2 Debate loop

Debate progression is deterministic:

- bull/bear debate continues until `count >= 2 * max_debate_rounds`;
- risk debate continues until `count >= 3 * max_risk_discuss_rounds`.

With the default config of 1, that means:

- one bull turn + one bear turn before Research Manager decides;
- one aggressive + one conservative + one neutral turn before Portfolio Manager decides.

So the default debate depth is shallow unless the caller increases config.

## 7. How outputs are finalized

There are actually two finalization steps:

1. Portfolio Manager writes the full final investment decision and rating narrative.
2. `SignalProcessor.process_signal()` sends that final text to another LLM call whose only job is to extract exactly one label: `BUY`, `OVERWEIGHT`, `HOLD`, `UNDERWEIGHT`, or `SELL`.

So even the rating extraction is LLM-based, not regex- or schema-based.

## 8. Logging and artifacts

`TradingAgentsGraph._log_state()` writes a JSON snapshot under:

- `eval_results/<ticker>/TradingAgentsStrategy_logs/full_states_log_<trade_date>.json`

That log includes:

- analyst reports;
- full debate histories;
- investment plan;
- trader plan;
- risk debate state;
- final trade decision.

This is the most complete persisted artifact the framework generates by default.

## 9. Practical conclusions

### 9.1 What this project really is

This is best viewed as a staged LLM workflow with debate prompts and vendor-routed market data tools, not a deeply autonomous trading system.
The strongest architectural ideas are:

- explicit graph stages;
- role-separated prompts;
- thin vendor abstraction for market data;
- lightweight reflective memory;
- loggable end-to-end state.

### 9.2 What is simple vs. sophisticated

Relatively sophisticated:

- LangGraph stage orchestration;
- dual-model setup for fast vs. deep reasoning;
- vendor routing and fallback;
- reflection + BM25 retrieval loop;
- CLI progress and report aggregation.

Relatively simple / fragile:

- prompts are all inline strings;
- almost all inter-agent handoff is plain text;
- tool outputs are mostly CSV/text blobs for LLM re-interpretation;
- default debate depth is only one cycle;
- learning is not persistent unless the caller adds storage around it;
- final rating extraction still depends on another LLM call.

### 9.3 My read on the design philosophy

My inference from the code is that the authors optimize for:

- ease of hacking on prompts;
- easy swapping of LLM providers;
- minimal infrastructure;
- explainable intermediate reports.

They do **not** optimize for:

- strict structured outputs;
- deterministic post-processing;
- persistent memory infrastructure;
- production-grade execution safeguards.

That makes sense for a research/demo framework, which is exactly how the repo presents itself.

## 10. Key files to read

If you want the fastest path through the codebase, read these in order:

1. `tradingagents/graph/trading_graph.py`
2. `tradingagents/graph/setup.py`
3. `tradingagents/graph/conditional_logic.py`
4. `tradingagents/agents/utils/agent_states.py`
5. `tradingagents/agents/analysts/*.py`
6. `tradingagents/agents/researchers/*.py`
7. `tradingagents/agents/managers/*.py`
8. `tradingagents/agents/risk_mgmt/*.py`
9. `tradingagents/agents/utils/memory.py`
10. `tradingagents/dataflows/interface.py`
11. `tradingagents/dataflows/y_finance.py`
12. `tradingagents/dataflows/yfinance_news.py`
13. `tradingagents/llm_clients/*.py`

## 11. Evidence map

Representative code references:

- Graph orchestration: `tradingagents/graph/trading_graph.py`, `tradingagents/graph/setup.py`
- State model: `tradingagents/agents/utils/agent_states.py`
- Debate routing: `tradingagents/graph/conditional_logic.py`
- Analyst prompts and tool binding: `tradingagents/agents/analysts/market_analyst.py`, `tradingagents/agents/analysts/social_media_analyst.py`, `tradingagents/agents/analysts/news_analyst.py`, `tradingagents/agents/analysts/fundamentals_analyst.py`
- Research / trader / portfolio prompts: `tradingagents/agents/researchers/bull_researcher.py`, `tradingagents/agents/researchers/bear_researcher.py`, `tradingagents/agents/managers/research_manager.py`, `tradingagents/agents/trader/trader.py`, `tradingagents/agents/managers/portfolio_manager.py`
- Memory and reflection: `tradingagents/agents/utils/memory.py`, `tradingagents/graph/reflection.py`
- Data routing and backends: `tradingagents/dataflows/interface.py`, `tradingagents/dataflows/y_finance.py`, `tradingagents/dataflows/yfinance_news.py`
- Provider abstraction: `tradingagents/llm_clients/factory.py`, `tradingagents/llm_clients/openai_client.py`, `tradingagents/llm_clients/base_client.py`
<!-- 2026-04-01 CST: 新增这段仓库边界提醒，原因是当前工作区同时存在 Python TradingAgents 与本地 SheetMind 证券分析主链；目的是防止后续 AI 把两套系统混成同一条产品链路。 -->
## 0. Workspace Boundary Reminder

This workspace now contains two different layers that must not be mixed:

- `tradingagents/`
  - the Python LangGraph multi-agent research pipeline described in this document
- `src/ops/*` + `skills/*`
  - the local SheetMind Rust binary-first product path, including the newer securities analysis chain

For securities analysis continuation work, prefer the local SheetMind chain:

- `technical_consultation_basic`
- `security_analysis_contextual`
- `security_analysis_fullstack`
- `skills/security-analysis-v1/SKILL.md`

Do not assume the Python `tradingagents/` architecture is the user-facing securities product path for the recent 2026-04-01 workstream.
