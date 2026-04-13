# Regression Head Precision Scheme C Design

**Goal:** Improve regression-head precision in one bounded slice by tightening regression governance and adding one lightweight robustness upgrade to the current Rust training path.

**Scope Boundary**
- Keep the existing training architecture, request contract, and runtime integration model unchanged.
- Do not introduce Python, external model services, or large-scale refactors.
- Limit behavior changes to `security_scorecard_training` metrics, regression readiness, and one minimal regression prediction enhancement.

**Problem Statement**
- Classification heads already disclose stronger out-of-sample quality signals.
- Regression heads are still permissive: readiness mainly checks sample thickness plus RMSE presence.
- The current regression predictor averages matched bin means directly, which is simple but can overreact to thin per-bin support.

**Chosen Scheme**
- Scheme C combines two small changes in one governed slice:
  - Tighten regression readiness so a regression head must beat a naive baseline and maintain acceptable directional usefulness before it can leave research-only state.
  - Stabilize regression bin predictions by shrinking thin-bin estimates toward the global training baseline.

**Why This Scheme**
- It improves safety first, so weak regression models stop looking production-like.
- It also makes the regression predictor less brittle without replacing the current architecture.
- The write scope stays local to one Rust module plus training tests.

**Design**
- Add regression quality metrics to the training output:
  - `baseline_mae`
  - `baseline_rmse`
  - `rmse_improvement_vs_baseline`
- Add a regression quality status inside readiness assessment:
  - `regression_quality_status`
- Require regression heads to satisfy all of the following before `shadow_candidate_ready`:
  - `minimum_sample_status == sample_ready`
  - `valid/test rmse_improvement_vs_baseline > 0`
  - `valid/test directional_hit_rate >= configured governance floor`
- Apply shrinkage when building regression prediction bins:
  - Use the bin sample count to blend the raw bin mean back toward the global baseline.
  - Keep the implementation deterministic and local to Rust.

**Testing Strategy**
- Red-green a new regression CLI contract test that requires the new metrics and regression quality status.
- Red-green a unit test proving regression bin predictions shrink toward the baseline when support is thin.
- Run targeted training CLI tests first, then the broader scorecard/chair regression suites.

**Risks**
- Thresholds that are too strict could downgrade current regression fixtures unexpectedly.
- Shrinkage that is too strong could flatten useful signal.
- Existing test fixtures may need small expectation updates if they were implicitly relying on permissive readiness.

**Non-Goals**
- No runtime registry auto-landing in this slice.
- No replacement of the current scorecard artifact format beyond additive regression metrics.
- No broad retraining framework rewrite.
