# 2026-04-13 Multi-Head Live Validation

## Scope
- Objective: push the governed securities stack from single-head replay into a real multi-head live-validation batch.
- Assets included in this wave:
  - `601916.SH` as the governed stock representative
  - `511010.SH` as `treasury_etf`
  - `518880.SH` as `gold_etf`
  - `513500.SH` as `cross_border_etf`
  - `512800.SH` as `equity_etf`

## Runtime Outputs
- Training summary:
  - `.excel_skill_runtime\\multi_head_live_batch_20260413\\training_summary.json`
- Final chair summary:
  - `.excel_skill_runtime\\multi_head_live_batch_20260413\\chair_resolution_summary.json`

## What Landed
- Trained six governed heads for all five assets:
  - `direction_head`
  - `return_head`
  - `drawdown_head`
  - `path_quality_head`
  - `upside_first_head`
  - `stop_first_head`
- Confirmed that every asset now reaches `security_chair_resolution` with:
  - `score_status = ready`
  - multi-head context available inside `master_scorecard`
  - non-empty path-risk constraints in the final chair output

## Key Outcomes
- `601916.SH`
  - multi-head context is live and fully consumable
  - final chair output remains `abstain`
  - main blocker is still committee-side `needs_more_evidence`, not missing quantitative heads
- `511010.SH`
  - treasury ETF final chain is now structurally complete
  - multi-head context shows low expected drawdown and strong upside-first skew
  - final chair output remains `abstain` because ETF information-side governance still degrades to `needs_more_evidence`
- `518880.SH`
  - gold ETF now carries usable return/drawdown/path/upside/stop context
  - final chair output remains `abstain`
- `513500.SH`
  - cross-border ETF now reaches final chair with real multi-head context
  - current path asymmetry is stop-first heavy, which explains the weakest final stance among the ETF batch
- `512800.SH`
  - equity ETF now carries full multi-head governed context
  - final chair output remains `abstain`

## Operational Reading
- The primary blocker has moved:
  - no longer a structural chain break
  - now mainly an evidence-depth and model-quality problem
- The governed stack can now say more than direction:
  - expected return
  - expected drawdown
  - path quality
  - upside-first probability
  - stop-first probability
- `180d` direction training is still unavailable in the current live slices because future rows are insufficient.

## Recommended Next Data Thickening
- ETF information-side evidence:
  - build ETF-native information semantics instead of inheriting stock-only disclosure expectations
- ETF multi-head strengthening:
  - especially `cross_border_etf` and `gold_etf`
- Longer live price windows:
  - needed before any governed `180d` head can be trained responsibly

## 2026-04-13 ETF Balanced-Scorecard Live Closeout
- Objective:
  - finish the ETF governed path from dated real-data slices to the last formal chair layer with all six heads attached and auditable
- Live batch root:
  - `.excel_skill_runtime\\balanced_scorecard_live_complete_20260413`
- Long-window validation slices:
  - `.excel_skill_runtime\\validation_real_data_slices\\511010_SH_real_data_20260413_long`
  - `.excel_skill_runtime\\validation_real_data_slices\\518880_SH_real_data_20260413_long`
  - `.excel_skill_runtime\\validation_real_data_slices\\513500_SH_real_data_20260413_long`
  - `.excel_skill_runtime\\validation_real_data_slices\\512800_SH_real_data_20260413_long`

### What Changed
- Rebuilt ETF-native long-window validation slices through `2026-04-10` so replay can support a later 180-day-capable analysis cut.
- Retrained six governed heads for all four ETF subscopes under the new batch:
  - `direction_head`
  - `return_head`
  - `drawdown_head`
  - `path_quality_head`
  - `upside_first_head`
  - `stop_first_head`
- Re-ran the formal final chair layer twice:
  - `2025-08-08` to confirm the latest live cut still lacks enough future rows for 180-day replay
  - `2025-07-11` as the latest common ETF date where full 180-day replay is available

### Key Verification Result
- The structural blocker is now closed:
  - ETF proxy information reaches `security_scorecard`
  - `master_scorecard` reads multi-head governed artifacts
  - `security_chair_resolution` consumes the full ETF quant context
- The current blocker is no longer missing chain connectivity.
- The current blocker is evidence depth plus conservative committee/risk governance.

### Latest Common 180d-Capable ETF Date
- `2025-07-11`
- Why not `2025-08-08`:
  - even after extending ETF slices to `2026-04-10`, the latest ETF cut still only had `160` future rows available for the 180-day head
  - the formal runtime therefore downgraded `2025-08-08` to `replay_unavailable` for complete 180-day balanced-scorecard output

### Final ETF Balanced-Scorecard Readout (`2025-07-11`)
- `511010.SH` (`treasury_etf`)
  - `score_status = ready`
  - `selected_action = avoid`
  - `aggregation_status = replay_with_multi_head_quant_context`
  - `success_probability = 0.9976`
  - `profitability_effectiveness_score = 29.36`
  - `risk_resilience_score = 43.75`
  - `path_quality_score = 20.23`
  - `master_score = 32.57`
  - `master_signal = weak`
- `518880.SH` (`gold_etf`)
  - `score_status = ready`
  - `selected_action = avoid`
  - `aggregation_status = replay_with_multi_head_quant_context`
  - `success_probability = 0.9999`
  - `profitability_effectiveness_score = 81.14`
  - `risk_resilience_score = 4.22`
  - `path_quality_score = 96.31`
  - `master_score = 57.25`
  - `master_signal = mixed`
- `513500.SH` (`cross_border_etf`)
  - `score_status = ready`
  - `selected_action = avoid`
  - `aggregation_status = replay_with_multi_head_quant_context`
  - `success_probability = 0.0005`
  - `profitability_effectiveness_score = 97.98`
  - `risk_resilience_score = 4.99`
  - `path_quality_score = 96.39`
  - `master_score = 65.12`
  - `master_signal = constructive`
- `512800.SH` (`equity_etf_peer`)
  - `score_status = ready`
  - `selected_action = avoid`
  - `aggregation_status = replay_with_multi_head_quant_context`
  - `success_probability = 1.0000`
  - `profitability_effectiveness_score = 3.12`
  - `risk_resilience_score = 0.00`
  - `path_quality_score = 81.02`
  - `master_score = 17.61`
  - `master_signal = weak`

### Operational Interpretation
- All four ETF pools now have:
  - ETF-native information available from governed proxy history
  - structurally valid six-head live artifacts
  - formal chair outputs produced from the governed multi-head path
- All four ETF pools still ended at `avoid`.
- The current reason is governance, not broken plumbing:
  - committee majority remains `avoid`
  - risk veto remains `blocked`
  - model grade is still `candidate/shadow`, not `champion`

### Remaining Thickening Priorities
- Build thicker ETF-native information semantics so ETF pools do not inherit stock-style evidence expectations where those expectations are not economically appropriate.
- Keep extending live price windows so later governed 180-day heads can be evaluated closer to the latest available date.
- Strengthen ETF multi-head quality, especially:
  - `cross_border_etf`
  - `gold_etf`
