# 2026-04-13 Direction-First Long Training Execution Notes

<!-- 2026-04-13 CST: Add the formal execution note for the seven-hour direction-first run.
Reason: the user approved direct long-run execution and asked to avoid repeated re-asking once the plan was chosen.
Purpose: keep one operator-facing note that records the request path, launch contract, runtime root, and later the real process evidence. -->

## 1. Goal

- Launch one formal long-running direction-first securities training round for:
  - `a_share_bank`
  - `treasury_etf`
  - `gold_etf`
  - `cross_border_etf`
  - `equity_etf`
- Cover horizons:
  - `5`
  - `10`
  - `15`
  - `30`
- Keep ranking policy fixed as:
  - `direction_test_accuracy`
  - `direction_test_auc`
  - `return_test_directional_hit_rate`
  - `return_test_rmse_improvement_vs_baseline`

## 2. Request Contract

- Request file:
  - `D:\Rust\Excel_Skill\.excel_skill_runtime\direction_first_training_run_20260413_7h_request.json`
- Tool:
  - `security_direction_first_training_run`
- Runtime root:
  - `D:\Rust\Excel_Skill\.excel_skill_runtime\direction_first_live_20260413_7h`
- Candidate count:
  - `40`
- Survivor count:
  - `20`
- Candidate layout:
  - `5 pools x 4 horizons x 2 variants`
- Variant policy:
  - `v1 = 8/4/4`
  - `v2 = 10/5/5`

## 3. Execution Command

```powershell
Start-Process `
  -FilePath "D:\Rust\Excel_Skill\target\debug\excel_skill.exe" `
  -WorkingDirectory "D:\Rust\Excel_Skill" `
  -RedirectStandardInput "D:\Rust\Excel_Skill\.excel_skill_runtime\direction_first_training_run_20260413_7h_request.json" `
  -RedirectStandardOutput "D:\Rust\Excel_Skill\.excel_skill_runtime\direction_first_live_20260413_7h\stdout.log" `
  -RedirectStandardError "D:\Rust\Excel_Skill\.excel_skill_runtime\direction_first_live_20260413_7h\stderr.log" `
  -PassThru
```

## 4. Launch Record

- Launch status:
  - `running`
- Launch time:
  - `2026-04-13 07:51:50`
- PID:
  - `29152`
- Stdout log:
  - `D:\Rust\Excel_Skill\.excel_skill_runtime\direction_first_live_20260413_7h\stdout.log`
- Stderr log:
  - `D:\Rust\Excel_Skill\.excel_skill_runtime\direction_first_live_20260413_7h\stderr.log`
- Expected stage summary root:
  - `D:\Rust\Excel_Skill\.excel_skill_runtime\direction_first_live_20260413_7h\direction_first_training_runs`

## 5. First Stability Check

- Check time:
  - `2026-04-13 07:52-07:53`
- Process state:
  - `alive`
- Current observed candidate:
  - `a_share_bank_05_v1`
- Current observed outputs:
  - `scorecard_artifacts\a_share_equity_5d_direction_head__candidate_2026_04_13T21_00_00_08_00.json`
  - `scorecard_artifacts\a_share_equity_5d_return_head__candidate_2026_04_13T21_00_00_08_00.json`
  - `scorecard_model_registry\a_share_equity_5d_direction_head__candidate_2026_04_13T21_00_00_08_00.json`
  - `scorecard_model_registry\a_share_equity_5d_return_head__candidate_2026_04_13T21_00_00_08_00.json`
  - `scorecard_refit_runs\refit_A_SHARE_EQUITY_2026_04_13T21_00_00_08_00.json`
- Current stdout state:
  - `still empty`
- Current stderr state:
  - `still empty`
- Stage summary state:
  - `not emitted yet`

## 6. First-Check Policy

- The tool is a thin long-run orchestration entry, so it may not stream human-readable progress lines continuously.
- The first stability check should therefore rely on:
  - process alive or exited
  - candidate runtime directories appearing under the runtime root
  - artifact or registry files starting to land
  - final stdout JSON if the process exits

## 7. Known Risks

- This tool is not a wall-clock scheduler, so the real runtime depends on the number of candidate pairs and the live data path cost.
- Survivor ranking is global for the full candidate list, not automatically one-per-pool or one-per-horizon.
- If the process exits early, the first diagnostic source should be the redirected stdout or stderr files instead of guessing from the shell.
