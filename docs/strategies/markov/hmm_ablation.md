# HMM Feature Engineering Experiment Plan

## Objective

Implement a controlled feature-ablation and feature-expansion experiment suite for a Gaussian Hidden Markov Model (HMM) used for financial market-regime detection.

The objective is to determine which engineered features improve regime identification, stability, and out-of-sample usefulness.

## Baseline features

- `daily_log_return`: ln(close[t] / close[t-1])
- `log_volume_change`: ln(volume[t] / volume[t-5])
- `moving_average_spread`: SMA(10)[t] / SMA(50)[t] - 1
- `normalized_atr`: mean(true_range[t-13:t]) / close[t], using a simple rolling mean rather than Wilder's smoothing
- `rolling_realized_volatility`: sample standard deviation (ddof=1) of daily log returns over 20 observations
- `rolling_return_skewness`: biased third central moment divided by standard deviation cubed, over 20 observations

## Experiments A-G: Baseline and ablations

| Experiment | Feature configuration |
|---|---|
| A | All six original features |
| B | Original features minus rolling skewness |
| C | Original features minus normalized ATR |
| D | Original features minus realized volatility |
| E | Original features minus log volume change |
| F | Original features plus 5-day log return |
| G | Original features plus downside volatility |

## Experiments H-R: Individual feature additions

Use Experiment A as the baseline and add one feature at a time.

| Experiment | Additional feature |
|---|---|
| H | 5-day log return |
| I | 10 EMA distance |
| J | 10/20 EMA spread |
| K | 200 SMA distance |
| L | VWAP distance |
| M | 20-day realized volatility alternative |
| N | Normalized ATR alternative |
| O | Rolling skewness alternative |
| P | Log volume change alternative |
| Q | SPY log return |
| R | Relative return to SPY |

Do not run duplicate configurations. If an additional feature is mathematically identical to an existing baseline feature, either skip it or explicitly configure a different horizon or representation.

## Experiments S-Y: Feature combinations

| Experiment | Additional features |
|---|---|
| S | 10 EMA distance + 10/20 EMA spread |
| T | 10 EMA distance + 200 SMA distance |
| U | VWAP distance + 10 EMA distance |
| V | SPY log return + relative return to SPY |
| W | 5-day log return + 10/20 EMA spread |
| X | 10 EMA distance + 10/20 EMA spread + 200 SMA distance |
| Y | VWAP distance + SPY log return |

## Feature formulas

- 5-day momentum: ln(close[t] / close[t-5])
- 10 EMA distance: ln(close[t] / EMA(10)[t])
- 10/20 EMA spread: ln(EMA(10)[t] / EMA(20)[t])
- 200 SMA distance: ln(close[t] / SMA(200)[t])
- VWAP distance: ln(close[t] / VWAP[t])
- SPY return: ln(SPY[t] / SPY[t-1])
- Relative return: stock_return[t] - rolling_beta[t] * SPY_return[t]
- Rolling beta: rolling covariance(stock returns, SPY returns) / rolling variance(SPY returns)
- Downside volatility: sqrt(mean(min(daily_log_return, 0)^2)) over 20 observations, using all 20 observations in the denominator.

## Evaluation requirements

1. Use chronological train/validation/test splits and walk-forward validation.
2. Fit all scalers and learned transformations on training data only.
3. Use identical HMM configurations across experiments.
4. Test multiple random seeds.
5. Compare diagonal and full covariance HMMs.
6. Evaluate out-of-sample log likelihood, regime persistence, transition frequency, state stability, and downstream trading performance.
7. Include transaction costs when evaluating trading performance.
8. Do not use smoothed probabilities as historical live trading signals. Use filtered probabilities for current-state inference.
9. Preserve feature names and experiment configurations in saved model metadata.
10. Produce a comparison report ranking experiments by out-of-sample performance and regime stability.

## Important implementation constraints

- All features must be backward-looking and timestamp-aligned.
- Avoid look-ahead bias in VWAP, beta, and benchmark calculations.
- Do not fill unavailable rolling features with zero.
- Ensure adjusted-price consistency for equity data.
- Avoid duplicate or highly redundant features without explicitly testing their effect.
- Keep the original baseline unchanged so that all experiments can be compared against it.

## TODO
- [x] A-G
- [x] H-R (H skipped: duplicate of F)
- [x] S-Y
- [x] Refit-cadence sensitivity (A-G, I-Y at 14/28/35 bars)
- [x] Selected full-covariance comparisons (A, B, K, Q, Y)
- [x] PCA Gaussian comparison (34 configurations)
- [x] XGBoost next-state classification (5 configurations)
- [ ] Z-AH post-review feature variants (not run)
- [ ] Additional Gaussian configurations and seed sweeps on top winners

Winners means we want Sharpe and PF to be above 1 with a reasonable number of trades (>= 100 on Dev).
None of A-Y met the dev trade-count gate; the covariance comparison therefore uses A (baseline),
B (reduced features), K/Q (strongest dev-skill single additions), and Y (strongest dev-skill
combination), without selecting on test results.

## Results

### A-G

Run on QQQ vs SPY benchmark (`hmm_regime` study, `configs/hmm_ablation.yaml`), K=3 diagonal
Gaussian, filtered probs, 4 restarts, refit every 21 bars expanding, min_train 252. Dev/Test
metrics are the `next_open` fill. OOS LL = held-out log-likelihood per bar accumulated over
frozen-model segments (higher is better). Persistence = occupancy-weighted mean P[i][i];
stability = fraction of 116 refits whose state->bucket map changed + mean winning-state
confidence. Trading = dev/test skill (total minus exposure-weighted benchmark trend).

| Experiment | Feature configuration | Dev/Test PF | Dev/Test Sharpe | Dev/Test Trades | Out-of-sample log likelihood | Regime persistence | Transition frequency | State stability | Downstream trading performance |
|---|---|---|---|---|---|---|---|---|---|
| A | All six original features | 1.87 / 3.00 | 0.24 / 0.76 | 31 / 5 | -7.41 | 0.963 (30d) | 0.057 | 82% churn, conf 0.93 | -0.177 / +0.002 |
| B | Original features minus rolling skewness | 1.70 / 3.52 | 0.28 / 0.74 | 26 / 8 | -5.97 | 0.962 (30d) | 0.052 | 87% churn, conf 0.93 | -0.157 / -0.014 |
| C | Original features minus normalized ATR | 1.42 / 2.06 | 0.13 / 0.57 | 33 / 6 | -6.52 | 0.968 (34d) | 0.055 | 84% churn, conf 0.93 | -0.069 / +0.045 |
| D | Original features minus realized volatility | 1.66 / 2.12 | 0.26 / 0.73 | 29 / 7 | -6.40 | 0.964 (29d) | 0.051 | 84% churn, conf 0.93 | -0.000 / -0.020 |
| E | Original features minus log volume change | 2.41 / 2.99 | 0.47 / 0.77 | 28 / 6 | -6.11 | 0.963 (30d) | 0.062 | 86% churn, conf 0.93 | -0.252 / +0.008 |
| F | Original features plus 5-day log return | 2.09 / 1.71 | 0.37 / 0.66 | 30 / 8 | -8.87 | 0.963 (30d) | 0.058 | 80% churn, conf 0.93 | -0.061 / +0.043 |
| G | Original features plus downside volatility | 2.35 / 2.55 | 0.57 / 0.80 | 28 / 10 | -8.33 | 0.961 (28d) | 0.046 | 85% churn, conf 0.94 | -0.091 / +0.027 |

Notes: every ablation beats A on held-out likelihood (B best, -5.97/bar) while F and G's extra
features make fit worse (-8.87/-8.33) — mild overfitting signature from added columns, though
all dev skills sit at or below zero on QQQ (insufficient evidence at 25-33 dev trades).
### H-R

Same run setup as A-G (QQQ vs SPY). H skipped: identical to F (both add the 5-day log return).
K's sma200 warm-up shortens the series (108 refits vs ~116 elsewhere).

| Experiment | Additional feature | Dev/Test PF | Dev/Test Sharpe | Dev/Test Trades | Out-of-sample log likelihood | Regime persistence | Transition frequency | State stability | Downstream trading performance |
|---|---|---|---|---|---|---|---|---|---|
| H | 5-day log return | skipped (identical to F) | - | - | - | - | - | - | - |
| I | 10 EMA distance | 1.93 / 2.03 | 0.32 / 0.69 | 30 / 6 | -8.85 | 0.963 (30d) | 0.059 | 82% churn, conf 0.93 | +0.019 / -0.042 |
| J | 10/20 EMA spread | 1.86 / 3.39 | 0.32 / 0.87 | 31 / 6 | -8.80 | 0.963 (29d) | 0.055 | 83% churn, conf 0.93 | -0.196 / -0.003 |
| K | 200 SMA distance | 4.27 / 1.94 | 0.67 / 0.71 | 18 / 9 | -8.70 | 0.967 (33d) | 0.043 | 86% churn, conf 0.94 | +0.144 / +0.018 |
| L | VWAP distance | 2.05 / 2.22 | 0.45 / 0.71 | 31 / 7 | -8.84 | 0.964 (30d) | 0.055 | 86% churn, conf 0.93 | -0.105 / -0.067 |
| M | 20-day realized vol alternative (Parkinson) | 1.49 / 2.13 | 0.16 / 0.77 | 29 / 8 | -8.38 | 0.963 (31d) | 0.053 | 91% churn, conf 0.94 | +0.020 / -0.015 |
| N | Normalized ATR alternative (Wilder) | 2.93 / 1.80 | 0.64 / 0.70 | 24 / 11 | -8.18 | 0.966 (33d) | 0.039 | 90% churn, conf 0.95 | +0.063 / -0.038 |
| O | Rolling skewness alternative (Pearson) | 1.91 / 2.45 | 0.35 / 0.67 | 26 / 9 | -8.86 | 0.964 (31d) | 0.061 | 85% churn, conf 0.93 | +0.040 / +0.039 |
| P | Log volume change alternative (vol surprise) | 2.22 / 3.01 | 0.39 / 0.89 | 29 / 7 | -8.70 | 0.963 (30d) | 0.059 | 80% churn, conf 0.93 | -0.095 / -0.009 |
| Q | SPY log return | 2.21 / 2.49 | 0.55 / 0.91 | 40 / 11 | -8.81 | 0.962 (29d) | 0.065 | 86% churn, conf 0.93 | +0.098 / +0.002 |
| R | Relative return to SPY | 1.77 / 2.56 | 0.22 / 0.81 | 27 / 7 | -8.94 | 0.961 (29d) | 0.057 | 83% churn, conf 0.93 | -0.150 / -0.020 |

Notes: every single-feature addition still trails the leanest ablations on held-out likelihood
(N wilder_atr best of the additions, -8.18/bar, vs B's -5.97). Best dev skills: K +0.144
(sma200), Q +0.098 (spy_ret), N +0.063 (wilder_atr). Q produces the most trades (40 dev) with
positive dev skill. Nothing clears the winner bar (>= 100 dev trades).

### S-Y

Same run setup (QQQ vs SPY). Combinations carry 8-9 features; T and X inherit K's
sma200 warm-up (108 refits).

| Experiment | Additional features | Dev/Test PF | Dev/Test Sharpe | Dev/Test Trades | Out-of-sample log likelihood | Regime persistence | Transition frequency | State stability | Downstream trading performance |
|---|---|---|---|---|---|---|---|---|---|
| S | 10 EMA distance + 10/20 EMA spread | 2.50 / 2.68 | 0.56 / 0.77 | 45 / 7 | -10.13 | 0.962 (28d) | 0.055 | 84% churn, conf 0.94 | +0.040 / +0.029 |
| T | 10 EMA distance + 200 SMA distance | 3.22 / 3.41 | 0.76 / 0.62 | 26 / 8 | -10.09 | 0.966 (32d) | 0.043 | 78% churn, conf 0.94 | +0.017 / -0.029 |
| U | VWAP distance + 10 EMA distance | 2.66 / 2.12 | 0.84 / 0.69 | 48 / 14 | -10.27 | 0.957 (27d) | 0.077 | 80% churn, conf 0.92 | +0.039 / -0.039 |
| V | SPY log return + relative return to SPY | 1.81 / 1.90 | 0.54 / 0.52 | 43 / 13 | -10.25 | 0.958 (27d) | 0.072 | 79% churn, conf 0.92 | -0.162 / -0.037 |
| W | 5-day log return + 10/20 EMA spread | 3.20 / 2.90 | 0.64 / 0.83 | 35 / 6 | -10.22 | 0.963 (29d) | 0.055 | 86% churn, conf 0.94 | +0.046 / +0.010 |
| X | 10 EMA dist + 10/20 EMA spread + 200 SMA dist | 3.24 / 2.73 | 0.69 / 0.74 | 25 / 6 | -11.52 | 0.964 (30d) | 0.049 | 83% churn, conf 0.94 | +0.035 / +0.044 |
| Y | VWAP distance + SPY log return | 2.18 / 1.96 | 0.51 / 0.69 | 39 / 10 | -10.15 | 0.960 (28d) | 0.077 | 83% churn, conf 0.92 | +0.102 / +0.042 |

Notes: every combination costs another ~1.3-1.5 LL/bar versus the single additions (all land
at -10.1 to -11.5; X worst at -11.52 with 9 features). Trade counts rise (U 48, S 45, V 43
dev) but dev skills stay small-positive at best: Y +0.102 (spy_ret pair), W +0.046, S +0.040.
Clear pattern across A-Y: held-out likelihood favors FEWER features (B -5.97/bar at 5 cols)
while trading skill mildly favors trend context (sma200_dist, spy_ret) — the emissions pay a
fit penalty for every extra column. Nothing clears the winner bar (>= 100 dev trades).

## Refit-cadence sensitivity

The A-Y feature configurations (H omitted because it duplicates F) were rerun with refit intervals
of 14, 28, and 35 bars. All other settings match the A-Y setup above. Each row reports the
primary `next_open` fill with 1 bp/side costs. DEV/TEST cells show PF / Sharpe / trades / win%
followed by trend / exposure-matched random-pick baseline / skill / bias / skill%. OOS LL is
held-out log likelihood per bar; persistence and flip rate are HMM diagnostics; map churn is the
fraction of refits whose state-to-bucket mapping changed. `n/a` means the statistic is undefined.

| Refit | Exp | OOS LL/bar | Persistence | Flip rate | Map churn / confidence | DEV partition | TEST partition |
|---:|---|---:|---:|---:|---|---|---|
| 14 | A | -7.31 | 0.963 | 0.061 | 84.4% / 0.928 | 2.22 / 0.34 / 31 / 45.2%; 0.492 / 0.236 / -0.257 / 0.000 / n/a | 3.24 / 0.86 / 6 / 50.0%; 0.315 / 0.328 / 0.016 / 0.000 / 4.9% |
| 14 | B | -5.95 | 0.962 | 0.052 | 80.3% / 0.932 | 1.55 / 0.28 / 30 / 46.7%; 0.315 / 0.185 / -0.130 / 0.000 / n/a | 3.24 / 0.70 / 9 / 55.6%; 0.288 / 0.262 / -0.023 / 0.000 / n/a |
| 14 | C | -6.49 | 0.968 | 0.061 | 83.8% / 0.932 | 1.30 / 0.15 / 33 / 48.5%; 0.106 / 0.082 / -0.023 / 0.000 / n/a | 3.60 / 0.70 / 7 / 71.4%; 0.275 / 0.260 / -0.013 / 0.000 / n/a |
| 14 | D | -6.39 | 0.964 | 0.055 | 81.5% / 0.926 | 2.54 / 0.57 / 33 / 63.6%; 0.421 / 0.439 / 0.018 / 0.000 / 4.0% | 2.00 / 0.74 / 7 / 42.9%; 0.300 / 0.297 / 0.001 / 0.000 / 0.2% |
| 14 | E | -5.97 | 0.963 | 0.062 | 80.3% / 0.930 | 2.51 / 0.46 / 28 / 46.4%; 0.408 / 0.323 / -0.086 / 0.000 / n/a | 3.16 / 0.78 / 6 / 50.0%; 0.278 / 0.294 / 0.020 / 0.000 / 6.7% |
| 14 | F | -8.77 | 0.964 | 0.061 | 80.3% / 0.930 | 3.56 / 0.73 / 32 / 56.2%; 0.523 / 0.571 / 0.048 / 0.000 / 8.4% | 2.31 / 0.75 / 7 / 57.1%; 0.302 / 0.276 / -0.023 / 0.000 / n/a |
| 14 | G | -8.18 | 0.961 | 0.048 | 80.9% / 0.944 | 2.26 / 0.53 / 27 / 44.4%; 0.450 / 0.393 / -0.058 / 0.000 / n/a | 2.76 / 0.89 / 12 / 58.3%; 0.318 / 0.351 / 0.036 / 0.000 / 10.1% |
| 14 | I | -8.73 | 0.963 | 0.060 | 84.4% / 0.934 | 2.16 / 0.55 / 39 / 43.6%; 0.335 / 0.396 / 0.061 / 0.000 / 15.4% | 2.02 / 0.72 / 8 / 50.0%; 0.291 / 0.264 / -0.027 / 0.000 / n/a |
| 14 | J | -8.67 | 0.964 | 0.062 | 83.2% / 0.934 | 1.78 / 0.27 / 35 / 51.4%; 0.337 / 0.198 / -0.139 / 0.000 / n/a | 3.65 / 0.93 / 6 / 50.0%; 0.391 / 0.381 / -0.006 / 0.000 / n/a |
| 14 | K | -8.65 | 0.967 | 0.046 | 84.0% / 0.946 | 3.10 / 0.52 / 20 / 65.0%; 0.423 / 0.416 / -0.007 / 0.000 / n/a | 2.16 / 0.74 / 9 / 33.3%; 0.277 / 0.279 / 0.005 / 0.000 / 1.7% |
| 14 | L | -8.77 | 0.964 | 0.062 | 84.4% / 0.935 | 1.80 / 0.43 / 38 / 42.1%; 0.441 / 0.322 / -0.118 / 0.000 / n/a | 1.75 / 0.54 / 9 / 44.4%; 0.249 / 0.197 / -0.049 / 0.000 / n/a |
| 14 | M | -8.28 | 0.963 | 0.053 | 75.7% / 0.940 | 1.91 / 0.33 / 26 / 50.0%; 0.170 / 0.214 / 0.044 / 0.000 / 20.8% | 2.04 / 0.75 / 8 / 50.0%; 0.306 / 0.275 / -0.028 / 0.000 / n/a |
| 14 | N | -8.04 | 0.966 | 0.040 | 79.8% / 0.948 | 4.45 / 0.82 / 18 / 66.7%; 0.446 / 0.545 / 0.099 / 0.000 / 18.1% | 1.85 / 0.68 / 10 / 60.0%; 0.276 / 0.244 / -0.029 / 0.000 / n/a |
| 14 | O | -8.77 | 0.964 | 0.060 | 80.9% / 0.928 | 3.29 / 0.67 / 31 / 61.3%; 0.557 / 0.468 / -0.089 / 0.000 / n/a | 3.03 / 0.81 / 6 / 50.0%; 0.314 / 0.316 / 0.005 / 0.000 / 1.6% |
| 14 | P | -8.61 | 0.962 | 0.056 | 83.8% / 0.930 | 2.12 / 0.38 / 29 / 44.8%; 0.464 / 0.267 / -0.198 / 0.000 / n/a | 3.08 / 0.91 / 7 / 57.1%; 0.405 / 0.363 / -0.039 / 0.000 / n/a |
| 14 | Q | -8.69 | 0.962 | 0.074 | 79.8% / 0.926 | 2.20 / 0.58 / 44 / 47.7%; 0.510 / 0.472 / -0.038 / 0.000 / n/a | 2.77 / 0.91 / 10 / 50.0%; 0.344 / 0.361 / 0.021 / 0.000 / 5.6% |
| 14 | R | -8.80 | 0.961 | 0.061 | 82.0% / 0.929 | 2.70 / 0.56 / 30 / 46.7%; 0.421 / 0.407 / -0.014 / 0.000 / n/a | 2.21 / 0.77 / 7 / 57.1%; 0.325 / 0.289 / -0.032 / 0.000 / n/a |
| 14 | S | -10.09 | 0.962 | 0.063 | 84.4% / 0.935 | 2.00 / 0.36 / 39 / 56.4%; 0.321 / 0.266 / -0.055 / 0.000 / n/a | 2.76 / 0.78 / 7 / 42.9%; 0.286 / 0.311 / 0.028 / 0.000 / 8.9% |
| 14 | T | -10.19 | 0.966 | 0.046 | 82.7% / 0.942 | 2.35 / 0.56 / 25 / 56.0%; 0.393 / 0.431 / 0.038 / 0.000 / 8.9% | 2.08 / 0.56 / 12 / 50.0%; 0.292 / 0.205 / -0.084 / 0.000 / n/a |
| 14 | U | -10.14 | 0.957 | 0.076 | 86.1% / 0.926 | 2.07 / 0.59 / 46 / 50.0%; 0.412 / 0.417 / 0.005 / 0.000 / 1.2% | 1.76 / 0.60 / 17 / 35.3%; 0.254 / 0.222 / -0.032 / 0.000 / n/a |
| 14 | V | -10.17 | 0.958 | 0.076 | 84.3% / 0.923 | 2.35 / 0.79 / 51 / 56.9%; 0.665 / 0.584 / -0.081 / 0.000 / n/a | 2.55 / 0.77 / 12 / 58.3%; 0.296 / 0.289 / -0.004 / 0.000 / n/a |
| 14 | W | -10.12 | 0.963 | 0.064 | 87.3% / 0.936 | 1.92 / 0.32 / 39 / 41.0%; 0.403 / 0.230 / -0.173 / 0.000 / n/a | 2.90 / 0.83 / 6 / 50.0%; 0.328 / 0.336 / 0.010 / 0.000 / 3.1% |
| 14 | X | -11.48 | 0.964 | 0.050 | 80.2% / 0.942 | 2.93 / 0.67 / 31 / 48.4%; 0.430 / 0.541 / 0.112 / 0.000 / 20.6% | 2.82 / 0.65 / 7 / 57.1%; 0.222 / 0.255 / 0.036 / 0.000 / 14.1% |
| 14 | Y | -10.10 | 0.960 | 0.076 | 81.5% / 0.929 | 2.04 / 0.49 / 48 / 43.8%; 0.348 / 0.379 / 0.031 / 0.000 / 8.2% | 1.88 / 0.73 / 9 / 33.3%; 0.249 / 0.274 / 0.029 / 0.000 / 10.3% |
| 28 | A | -7.51 | 0.963 | 0.053 | 82.8% / 0.928 | 1.86 / 0.22 / 28 / 39.3%; 0.285 / 0.138 / -0.148 / 0.000 / n/a | 1.67 / 0.62 / 9 / 33.3%; 0.269 / 0.225 / -0.042 / 0.000 / n/a |
| 28 | B | -6.06 | 0.962 | 0.050 | 80.5% / 0.933 | 1.83 / 0.36 / 26 / 53.8%; 0.333 / 0.250 / -0.083 / 0.000 / n/a | 3.52 / 0.74 / 8 / 62.5%; 0.296 / 0.280 / -0.014 / 0.000 / n/a |
| 28 | C | -6.64 | 0.968 | 0.048 | 81.6% / 0.931 | 2.09 / 0.45 / 29 / 58.6%; 0.388 / 0.327 / -0.061 / 0.000 / n/a | 4.90 / 0.83 / 7 / 57.1%; 0.332 / 0.314 / -0.014 / 0.000 / n/a |
| 28 | D | -6.43 | 0.963 | 0.045 | 85.1% / 0.929 | 1.48 / 0.14 / 32 / 43.8%; 0.195 / 0.078 / -0.117 / 0.000 / n/a | 2.12 / 0.77 / 7 / 57.1%; 0.319 / 0.310 / -0.006 / 0.000 / n/a |
| 28 | E | -6.15 | 0.963 | 0.055 | 75.9% / 0.929 | 2.22 / 0.34 / 29 / 44.8%; 0.405 / 0.237 / -0.168 / 0.000 / n/a | 2.77 / 0.70 / 7 / 28.6%; 0.265 / 0.260 / -0.002 / 0.000 / n/a |
| 28 | F | -8.93 | 0.963 | 0.054 | 85.1% / 0.930 | 2.70 / 0.54 / 31 / 54.8%; 0.416 / 0.421 / 0.005 / 0.000 / 1.1% | 2.57 / 0.78 / 8 / 50.0%; 0.334 / 0.292 / -0.040 / 0.000 / n/a |
| 28 | G | -8.48 | 0.961 | 0.042 | 94.3% / 0.944 | 2.72 / 0.67 / 27 / 51.9%; 0.554 / 0.510 / -0.044 / 0.000 / n/a | 4.09 / 1.04 / 10 / 60.0%; 0.381 / 0.417 / 0.039 / 0.000 / 9.4% |
| 28 | I | -8.90 | 0.962 | 0.058 | 88.5% / 0.929 | 2.02 / 0.34 / 35 / 31.4%; 0.228 / 0.240 / 0.012 / 0.000 / 5.1% | 1.52 / 0.60 / 9 / 44.4%; 0.321 / 0.216 / -0.105 / 0.000 / n/a |
| 28 | J | -8.85 | 0.963 | 0.054 | 86.2% / 0.936 | 2.32 / 0.37 / 26 / 53.8%; 0.447 / 0.285 / -0.162 / 0.000 / n/a | 3.50 / 0.88 / 6 / 50.0%; 0.370 / 0.363 / -0.004 / 0.000 / n/a |
| 28 | K | -8.85 | 0.967 | 0.046 | 79.0% / 0.942 | 4.38 / 0.70 / 17 / 52.9%; 0.585 / 0.552 / -0.033 / 0.000 / n/a | 2.08 / 0.73 / 9 / 33.3%; 0.267 / 0.275 / 0.011 / 0.000 / 3.9% |
| 28 | L | -8.88 | 0.963 | 0.059 | 85.1% / 0.931 | 2.17 / 0.46 / 29 / 44.8%; 0.348 / 0.343 / -0.005 / 0.000 / n/a | 1.94 / 0.68 / 9 / 44.4%; 0.315 / 0.252 / -0.059 / 0.000 / n/a |
| 28 | M | -8.72 | 0.963 | 0.049 | 82.8% / 0.940 | 1.95 / 0.36 / 23 / 47.8%; 0.297 / 0.270 / -0.027 / 0.000 / n/a | 1.85 / 0.72 / 9 / 44.4%; 0.277 / 0.266 / -0.008 / 0.000 / n/a |
| 28 | N | -8.28 | 0.966 | 0.036 | 79.3% / 0.947 | 3.38 / 0.74 / 20 / 60.0%; 0.416 / 0.493 / 0.077 / 0.000 / 15.7% | 1.83 / 0.67 / 10 / 60.0%; 0.272 / 0.242 / -0.027 / 0.000 / n/a |
| 28 | O | -9.00 | 0.963 | 0.059 | 81.6% / 0.930 | 2.31 / 0.41 / 34 / 44.1%; 0.335 / 0.293 / -0.042 / 0.000 / n/a | 1.65 / 0.62 / 9 / 33.3%; 0.270 / 0.223 / -0.043 / 0.000 / n/a |
| 28 | P | -8.84 | 0.963 | 0.051 | 80.5% / 0.931 | 2.45 / 0.44 / 31 / 48.4%; 0.460 / 0.317 / -0.143 / 0.000 / n/a | 2.55 / 0.80 / 7 / 57.1%; 0.335 / 0.308 / -0.024 / 0.000 / n/a |
| 28 | Q | -8.84 | 0.962 | 0.061 | 85.1% / 0.928 | 1.88 / 0.40 / 41 / 43.9%; 0.317 / 0.309 / -0.008 / 0.000 / n/a | 2.50 / 0.92 / 11 / 45.5%; 0.382 / 0.361 / -0.018 / 0.000 / n/a |
| 28 | R | -8.97 | 0.962 | 0.062 | 82.6% / 0.923 | 3.68 / 0.72 / 24 / 45.8%; 0.610 / 0.545 / -0.065 / 0.000 / n/a | 2.40 / 0.81 / 10 / 60.0%; 0.363 / 0.304 / -0.056 / 0.000 / n/a |
| 28 | S | -10.28 | 0.962 | 0.056 | 85.1% / 0.937 | 2.55 / 0.52 / 31 / 54.8%; 0.422 / 0.419 / -0.004 / 0.000 / n/a | 2.68 / 0.77 / 7 / 42.9%; 0.281 / 0.307 / 0.029 / 0.000 / 9.3% |
| 28 | T | -10.37 | 0.966 | 0.043 | 87.7% / 0.942 | 2.16 / 0.51 / 24 / 45.8%; 0.488 / 0.406 / -0.082 / 0.000 / n/a | 1.95 / 0.48 / 8 / 37.5%; 0.212 / 0.179 / -0.030 / 0.000 / n/a |
| 28 | U | -10.31 | 0.957 | 0.074 | 83.9% / 0.924 | 3.21 / 0.96 / 43 / 53.5%; 0.723 / 0.758 / 0.034 / 0.000 / 4.5% | 1.51 / 0.57 / 14 / 42.9%; 0.327 / 0.210 / -0.117 / 0.000 / n/a |
| 28 | V | -10.41 | 0.959 | 0.073 | 82.6% / 0.921 | 2.24 / 0.81 / 55 / 56.4%; 0.629 / 0.615 / -0.014 / 0.000 / n/a | 2.36 / 0.74 / 12 / 50.0%; 0.300 / 0.277 / -0.020 / 0.000 / n/a |
| 28 | W | -10.27 | 0.963 | 0.052 | 86.2% / 0.937 | 2.70 / 0.48 / 31 / 38.7%; 0.360 / 0.363 / 0.003 / 0.000 / 0.9% | 2.90 / 0.83 / 6 / 50.0%; 0.328 / 0.336 / 0.010 / 0.000 / 3.1% |
| 28 | X | -11.81 | 0.964 | 0.049 | 80.2% / 0.940 | 2.85 / 0.58 / 26 / 53.8%; 0.318 / 0.451 / 0.134 / 0.000 / 29.6% | 2.74 / 0.66 / 6 / 50.0%; 0.203 / 0.253 / 0.053 / 0.000 / 20.7% |
| 28 | Y | -10.30 | 0.959 | 0.073 | 79.3% / 0.923 | 2.21 / 0.60 / 49 / 46.9%; 0.505 / 0.462 / -0.043 / 0.000 / n/a | 1.90 / 0.76 / 12 / 33.3%; 0.214 / 0.286 / 0.074 / 0.000 / 25.8% |
| 35 | A | -7.72 | 0.964 | 0.061 | 88.6% / 0.928 | 1.82 / 0.31 / 38 / 36.8%; 0.410 / 0.223 / -0.187 / 0.000 / n/a | 3.93 / 0.87 / 6 / 50.0%; 0.351 / 0.337 / -0.011 / 0.000 / n/a |
| 35 | B | -6.27 | 0.962 | 0.049 | 81.4% / 0.929 | 1.68 / 0.30 / 27 / 48.1%; 0.329 / 0.206 / -0.123 / 0.000 / n/a | 3.78 / 0.80 / 8 / 62.5%; 0.332 / 0.306 / -0.023 / 0.000 / n/a |
| 35 | C | -6.81 | 0.968 | 0.054 | 85.7% / 0.929 | 1.25 / 0.13 / 36 / 50.0%; 0.130 / 0.070 / -0.060 / 0.000 / n/a | 8.01 / 0.72 / 7 / 71.4%; 0.283 / 0.278 / -0.002 / 0.000 / n/a |
| 35 | D | -6.53 | 0.964 | 0.049 | 81.4% / 0.928 | 2.39 / 0.49 / 26 / 57.7%; 0.414 / 0.406 / -0.008 / 0.000 / n/a | 2.22 / 0.80 / 7 / 57.1%; 0.297 / 0.324 / 0.030 / 0.000 / 9.3% |
| 35 | E | -6.38 | 0.964 | 0.065 | 87.1% / 0.923 | 1.60 / 0.16 / 36 / 30.6%; 0.421 / 0.089 / -0.332 / 0.000 / n/a | 2.03 / 0.78 / 9 / 44.4%; 0.268 / 0.292 / 0.027 / 0.000 / 9.1% |
| 35 | F | -9.18 | 0.964 | 0.058 | 78.6% / 0.926 | 2.49 / 0.59 / 36 / 50.0%; 0.715 / 0.483 / -0.232 / 0.000 / n/a | 1.54 / 0.61 / 9 / 44.4%; 0.253 / 0.220 / -0.030 / 0.000 / n/a |
| 35 | G | -8.94 | 0.961 | 0.050 | 87.1% / 0.940 | 2.43 / 0.59 / 26 / 53.8%; 0.557 / 0.444 / -0.113 / 0.000 / n/a | 2.38 / 0.83 / 12 / 50.0%; 0.279 / 0.316 / 0.041 / 0.000 / 12.8% |
| 35 | I | -9.16 | 0.964 | 0.061 | 85.7% / 0.929 | 2.48 / 0.55 / 33 / 45.5%; 0.583 / 0.435 / -0.148 / 0.000 / n/a | 1.79 / 0.69 / 8 / 50.0%; 0.321 / 0.251 / -0.069 / 0.000 / n/a |
| 35 | J | -9.07 | 0.964 | 0.054 | 88.6% / 0.934 | 2.12 / 0.41 / 32 / 53.1%; 0.500 / 0.314 / -0.186 / 0.000 / n/a | 3.10 / 0.85 / 6 / 50.0%; 0.377 / 0.346 / -0.028 / 0.000 / n/a |
| 35 | K | -9.34 | 0.967 | 0.042 | 83.1% / 0.944 | 5.49 / 0.88 / 18 / 77.8%; 0.603 / 0.717 / 0.115 / 0.000 / 16.0% | 2.62 / 0.69 / 10 / 40.0%; 0.244 / 0.259 / 0.018 / 0.000 / 7.0% |
| 35 | L | -9.14 | 0.964 | 0.055 | 80.0% / 0.929 | 2.29 / 0.60 / 40 / 45.0%; 0.636 / 0.462 / -0.174 / 0.000 / n/a | 2.26 / 0.72 / 8 / 50.0%; 0.351 / 0.266 / -0.083 / 0.000 / n/a |
| 35 | M | -8.87 | 0.964 | 0.053 | 82.9% / 0.938 | 1.56 / 0.20 / 31 / 48.4%; 0.018 / 0.131 / 0.113 / 0.000 / 86.5% | 2.31 / 0.87 / 9 / 66.7%; 0.378 / 0.335 / -0.040 / 0.000 / n/a |
| 35 | N | -8.48 | 0.966 | 0.039 | 78.6% / 0.944 | 3.30 / 0.73 / 23 / 47.8%; 0.429 / 0.479 / 0.050 / 0.000 / 10.5% | 1.65 / 0.63 / 10 / 60.0%; 0.234 / 0.223 / -0.007 / 0.000 / n/a |
| 35 | O | -9.23 | 0.964 | 0.066 | 85.7% / 0.923 | 2.08 / 0.45 / 38 / 42.1%; 0.399 / 0.352 / -0.046 / 0.000 / n/a | 2.76 / 0.77 / 5 / 40.0%; 0.331 / 0.288 / -0.039 / 0.000 / n/a |
| 35 | P | -9.03 | 0.963 | 0.060 | 81.4% / 0.926 | 2.43 / 0.51 / 35 / 42.9%; 0.606 / 0.369 / -0.238 / 0.000 / n/a | 2.95 / 0.89 / 7 / 57.1%; 0.408 / 0.350 / -0.054 / 0.000 / n/a |
| 35 | Q | -9.06 | 0.962 | 0.067 | 88.6% / 0.926 | 2.34 / 0.61 / 40 / 42.5%; 0.705 / 0.491 / -0.214 / 0.000 / n/a | 2.43 / 0.90 / 11 / 45.5%; 0.331 / 0.357 / 0.029 / 0.000 / 8.0% |
| 35 | R | -9.33 | 0.962 | 0.056 | 84.1% / 0.929 | 3.55 / 0.71 / 23 / 47.8%; 0.570 / 0.534 / -0.036 / 0.000 / n/a | 2.68 / 0.84 / 8 / 62.5%; 0.353 / 0.317 / -0.033 / 0.000 / n/a |
| 35 | S | -10.53 | 0.962 | 0.059 | 80.0% / 0.935 | 3.60 / 0.90 / 40 / 62.5%; 0.716 / 0.738 / 0.022 / 0.000 / 3.0% | 3.16 / 0.82 / 8 / 37.5%; 0.324 / 0.330 / 0.010 / 0.000 / 2.9% |
| 35 | T | -10.74 | 0.966 | 0.041 | 80.0% / 0.942 | 2.81 / 0.69 / 22 / 45.5%; 0.610 / 0.541 / -0.069 / 0.000 / n/a | 2.29 / 0.62 / 11 / 45.5%; 0.313 / 0.227 / -0.083 / 0.000 / n/a |
| 35 | U | -10.58 | 0.958 | 0.066 | 85.7% / 0.923 | 1.94 / 0.59 / 39 / 48.7%; 0.424 / 0.422 / -0.002 / 0.000 / n/a | 1.76 / 0.47 / 13 / 46.2%; 0.208 / 0.164 / -0.044 / 0.000 / n/a |
| 35 | V | -10.66 | 0.959 | 0.072 | 78.3% / 0.923 | 2.68 / 0.66 / 41 / 43.9%; 0.570 / 0.513 / -0.056 / 0.000 / n/a | 1.91 / 0.76 / 13 / 53.8%; 0.293 / 0.287 / -0.003 / 0.000 / n/a |
| 35 | W | -10.51 | 0.963 | 0.058 | 82.9% / 0.934 | 1.94 / 0.34 / 38 / 44.7%; 0.333 / 0.263 / -0.069 / 0.000 / n/a | 2.88 / 0.83 / 7 / 42.9%; 0.324 / 0.335 / 0.014 / 0.000 / 4.1% |
| 35 | X | -12.27 | 0.964 | 0.047 | 81.5% / 0.941 | 3.70 / 0.68 / 25 / 52.0%; 0.468 / 0.539 / 0.071 / 0.000 / 13.1% | 2.73 / 0.74 / 6 / 50.0%; 0.257 / 0.298 / 0.044 / 0.000 / 14.6% |
| 35 | Y | -10.52 | 0.960 | 0.074 | 87.1% / 0.921 | 1.80 / 0.36 / 51 / 39.2%; 0.346 / 0.260 / -0.086 / 0.000 / n/a | 1.87 / 0.75 / 11 / 36.4%; 0.238 / 0.287 / 0.049 / 0.000 / 17.1% |

None of these cadence variants met the predeclared >=100 DEV trades gate. The cadence run
produced 72 complete experiment records; per-variant reports, fit diagnostics, and partition
CSVs are under `results/hmm_regime/`.

### additional post review
| Experiment | Additional feature | Purpose |
|---|---|---|
| Z | Original features minus skewness and volume | Test whether both features are unnecessary |
| AA | B + Wilder ATR | Test alternative ATR with reduced baseline |
| AB | B + SPY return | Test broad-market information without skewness |
| AC | B + SMA200 distance | Test long-term trend information without skewness |
| AD | B + volume surprise | Test alternative volume representation |
| AE | B + downside volatility | Test downside risk without skewness |
| AF | E + SPY return | Test broad-market information without volume change |
| AG | B + SPY return + SMA200 distance | Combine two promising additions |
| AH | B + Wilder ATR + SPY return | Combine volatility and market context |

## Gaussian configurations

| Experiment | Model | Purpose |
|---|---|---|
| 1 | Gaussian HMM, diagonal covariance | Baseline |
| 2 | Gaussian HMM, full covariance | Capture feature correlations |
| 3 | Student-t HMM | Better accommodate fat-tailed observations |
| 4 | GMM-HMM | Model more complex distributions within each regime

The Rust HMM supports `hmm_covariance: diagonal` (default) and `hmm_covariance: full`.
Full covariance estimates one regularized feature covariance matrix per state; Cholesky
factors are used for Gaussian log densities. The same seeds, restarts, split, refit cadence,
scaler, and trading logic are used for the paired comparison.

## Gaussian Exp results

Paired QQQ vs SPY runs from `configs/hmm_ablation.yaml`; K=3, seed 20261106, 4 restarts,
expanding train window, 252-bar minimum, 21-bar refit, train-window scaling, filtered
probabilities, and `next_open` primary fill with 1 bp/side costs. Diagonal values are the
recorded A-Y runs; full covariance was run for A, B, K, Q, and Y. PF, Sharpe, trade count, and
win rate are dev/test pairs. OOS LL is held-out log-likelihood per bar (higher is better).
Map churn is the fraction of refits whose state-to-bucket mapping changed; confidence is the
mean winning-state probability.

| Exp | Covariance | Dev PF / Sharpe / trades / win% | Test PF / Sharpe / trades / win% | OOS LL/bar | Persistence | Transition freq | Map churn / confidence |
|---|---|---|---|---:|---:|---:|---|
| A | Diagonal | 1.87 / 0.24 / 31 / 48.4% | 3.00 / 0.76 / 5 / 60.0% | -7.4082 | 0.9629 | 0.0571 | 81.9% / 0.9277 |
| A | Full | 1.64 / 0.36 / 28 / 50.0% | 3.05 / 0.84 / 18 / 72.2% | -7.3176 | 0.9680 | 0.0645 | 84.5% / 0.9346 |
| B | Diagonal | 1.70 / 0.28 / 26 / 50.0% | 3.52 / 0.74 / 8 / 62.5% | -5.9738 | 0.9617 | 0.0521 | 87.1% / 0.9320 |
| B | Full | 2.91 / 0.76 / 35 / 60.0% | 2.61 / 0.91 / 15 / 60.0% | -5.7423 | 0.9679 | 0.0662 | 83.6% / 0.9294 |
| K | Diagonal | 4.27 / 0.67 / 18 / 55.6% | 1.94 / 0.71 / 9 / 33.3% | -8.6983 | 0.9667 | 0.0432 | 86.1% / 0.9439 |
| K | Full | 3.82 / 0.75 / 26 / 61.5% | 2.46 / 0.61 / 8 / 50.0% | -8.7858 | 0.9744 | 0.0507 | 83.3% / 0.9496 |
| Q | Diagonal | 2.21 / 0.55 / 40 / 45.0% | 2.49 / 0.91 / 11 / 45.5% | -8.8083 | 0.9622 | 0.0650 | 86.2% / 0.9261 |
| Q | Full | 2.39 / 0.77 / 43 / 62.8% | 1.99 / 0.72 / 21 / 61.9% | -7.9133 | 0.9680 | 0.0741 | 85.3% / 0.9333 |
| Y | Diagonal | 2.18 / 0.51 / 39 / 51.3% | 1.96 / 0.69 / 10 / 40.0% | -10.1541 | 0.9602 | 0.0774 | 82.8% / 0.9222 |
| Y | Full | 1.92 / 0.47 / 39 / 56.4% | 1.51 / 0.52 / 15 / 46.7% | -9.1292 | 0.9655 | 0.0716 | 83.6% / 0.9335 |

Full covariance improved held-out likelihood for A, B, Q, and Y, but slightly worsened K.
Trading metrics were mixed: B's dev metrics improved while its test PF fell; Q had more trades
but lower test PF/Sharpe; Y weakened on both PF and Sharpe. None reached 100 dev trades. The
full dev/test partition fields (trend, random-pick baselines, skill, bias, skill%, Sharpe, PF,
trades, and win rate) are preserved in
[`results/hmm_regime/gaussian_covariance_comparison.csv`](../../../results/hmm_regime/gaussian_covariance_comparison.csv);
each variant report also contains the full partition tables. Test results are already inspected
and are not a fresh holdout.

## PCA experiments

| Experiment | Input feature set                 | PCA components  | Covariance type |
| ---------- | --------------------------------- | --------------- | --------------- |
| PCA-A      | Original 6 features               | None (baseline) | Diagonal, Full  |
| PCA-B      | Original 6 features               | 2, 3, 4, 5, 6   | Diagonal, Full  |
| PCA-C      | Reduced set B (minus skewness)    | None (baseline) | Diagonal, Full  |
| PCA-D      | Reduced set B                     | 2, 3, 4, 5      | Diagonal, Full  |
| PCA-E      | Promising expanded set (e.g., AB) | None (baseline) | Diagonal, Full  |
| PCA-F      | Promising expanded set (e.g., AB) | 2, 3, 4, 5, 6   | Diagonal, Full  |

Controls: Fit scaler and PCA on training data only; whitening off; keep state count, seeds, trading logic, and walk-forward splits fixed.

PCA-E/F use AB (`ret,rv,atr,volchg,ma,spy_ret`). Set
`hmm_pca_components: 0` to disable PCA, or a positive component count to fit
unwhitened PCA on each scaled training window and freeze it until the next
refit. Both covariance modes use the same fixed restart seed and trading setup.
The existing HMM baseline uses `ln(mean(true_range)/close)` for `atr`, rather
than the unlogged ratio specified above; it is retained here for comparability.
The existing partition trend/skill calculation uses QQQ, not SPY: SPY is not
in the traded panel, so the risk-symbol calculation falls back to QQQ. SPY
remains the external feature and chart benchmark.
Likelihoods in different dimensions are densities on different spaces and
should not be ranked against one another.

## PCA Exp Results

QQQ, next-open, 1 bp per side. SPY is used for market features and charts;
the partition trend/skill benchmark is QQQ buy-and-hold, not SPY, because
SPY is not in the tradable panel. All 34 configurations use the fixed
three-state walk-forward HMM and four restarts. PCA-A diagonal is `baseline`.
The table is ranked by DEV Sharpe; eligibility requires DEV Sharpe > 1,
profit factor > 1, and at least 100 trades. TEST is shown only for the
baseline and the DEV-selected candidate. TEST has been inspected in earlier
experiments and is not a fresh holdout.

**Best DEV Sharpe, exploratory (no eligible winner): `pca_d5_full`.**

| DEV rank | Experiment | PF | Sharpe | Trades | Win % | Trend | Random pick | Bias | Skill | Skill % | OOS LL/bar | Persistence | Flip rate | Map churn / confidence |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 1 | pca_d5_full | 2.74 | 0.97 | 49 | 61.2 | 0.651 | 0.740 | 0.000 | 0.089 | 12.1 | -5.70 | 0.969 | 0.076 | 78% / 0.92 |
| 2 | pca_f3_diag | 6.03 | 0.96 | 30 | 76.7 | 0.750 | 0.813 | 0.000 | 0.063 | 7.8 | -4.32 | 0.956 | 0.053 | 82% / 0.91 |
| 3 | pca_f2_diag | 4.25 | 0.82 | 30 | 50.0 | 0.770 | 0.698 | 0.000 | -0.072 | n/a | -3.00 | 0.957 | 0.052 | 84% / 0.91 |
| 4 | pca_b4_diag | 3.54 | 0.76 | 33 | 72.7 | 0.482 | 0.602 | 0.000 | 0.120 | 20.0 | -5.47 | 0.957 | 0.061 | 84% / 0.91 |
| 5 | pca_c_full | 2.91 | 0.76 | 35 | 60.0 | 0.539 | 0.551 | 0.000 | 0.012 | 2.2 | -5.74 | 0.968 | 0.066 | 84% / 0.93 |
| 6 | pca_f2_full | 2.51 | 0.71 | 35 | 48.6 | 0.453 | 0.539 | 0.000 | 0.086 | 15.9 | -3.06 | 0.958 | 0.047 | 86% / 0.92 |
| 7 | pca_e_full | 2.31 | 0.66 | 40 | 60.0 | 0.444 | 0.478 | 0.000 | 0.034 | 7.1 | -6.22 | 0.968 | 0.071 | 78% / 0.93 |
| 8 | pca_e_diag | 2.72 | 0.65 | 33 | 42.4 | 0.495 | 0.500 | 0.000 | 0.005 | 1.0 | -7.29 | 0.960 | 0.062 | 82% / 0.93 |
| 9 | pca_b3_full | 2.32 | 0.62 | 37 | 54.1 | 0.524 | 0.474 | 0.000 | -0.050 | n/a | -4.23 | 0.960 | 0.058 | 78% / 0.92 |
| 10 | pca_b3_diag | 2.17 | 0.61 | 34 | 55.9 | 0.453 | 0.439 | 0.000 | -0.014 | n/a | -4.22 | 0.958 | 0.058 | 84% / 0.91 |
| 11 | pca_b2_diag | 3.00 | 0.57 | 24 | 62.5 | 0.360 | 0.438 | 0.000 | 0.078 | 17.8 | -2.83 | 0.957 | 0.048 | 82% / 0.92 |
| 12 | pca_f6_diag | 1.85 | 0.57 | 49 | 59.2 | 0.575 | 0.527 | 0.000 | -0.048 | n/a | -6.48 | 0.960 | 0.059 | 85% / 0.92 |
| 13 | pca_d3_full | 2.08 | 0.57 | 40 | 52.5 | 0.405 | 0.361 | 0.000 | -0.044 | n/a | -4.04 | 0.958 | 0.050 | 87% / 0.92 |
| 14 | pca_d2_full | 2.23 | 0.56 | 31 | 45.2 | 0.427 | 0.380 | 0.000 | -0.047 | n/a | -2.82 | 0.958 | 0.048 | 84% / 0.93 |
| 15 | pca_b6_full | 2.30 | 0.54 | 38 | 52.6 | 0.306 | 0.374 | 0.000 | 0.068 | 18.2 | -7.28 | 0.968 | 0.060 | 83% / 0.94 |
| 16 | pca_b5_full | 1.83 | 0.52 | 37 | 54.1 | 0.246 | 0.293 | 0.000 | 0.047 | 16.0 | -6.98 | 0.969 | 0.070 | 84% / 0.93 |
| 17 | pca_f5_diag | 2.19 | 0.49 | 40 | 62.5 | 0.494 | 0.412 | 0.000 | -0.083 | n/a | -6.20 | 0.964 | 0.053 | 87% / 0.92 |
| 18 | pca_b4_full | 1.76 | 0.49 | 42 | 47.6 | 0.376 | 0.358 | 0.000 | -0.018 | n/a | -5.52 | 0.960 | 0.065 | 84% / 0.92 |
| 19 | pca_f5_full | 1.61 | 0.46 | 42 | 54.8 | 0.429 | 0.343 | 0.000 | -0.086 | n/a | -5.96 | 0.969 | 0.062 | 86% / 0.93 |
| 20 | pca_d5_diag | 2.50 | 0.44 | 28 | 60.7 | 0.410 | 0.365 | 0.000 | -0.045 | n/a | -5.98 | 0.963 | 0.051 | 82% / 0.93 |
| 21 | pca_b2_full | 2.06 | 0.40 | 29 | 41.4 | 0.245 | 0.277 | 0.000 | 0.032 | 11.7 | -2.83 | 0.959 | 0.051 | 89% / 0.92 |
| 22 | pca_f3_full | 1.64 | 0.39 | 45 | 51.1 | 0.304 | 0.272 | 0.000 | -0.032 | n/a | -4.34 | 0.958 | 0.054 | 85% / 0.92 |
| 23 | pca_a_full | 1.64 | 0.36 | 28 | 50.0 | 0.168 | 0.235 | 0.000 | 0.067 | 28.4 | -7.32 | 0.968 | 0.065 | 84% / 0.93 |
| 24 | pca_d4_diag | 1.51 | 0.31 | 44 | 56.8 | 0.252 | 0.242 | 0.000 | -0.010 | n/a | -5.54 | 0.968 | 0.060 | 83% / 0.92 |
| 25 | pca_f6_full | 1.50 | 0.30 | 44 | 52.3 | 0.262 | 0.213 | 0.000 | -0.049 | n/a | -6.29 | 0.968 | 0.065 | 84% / 0.93 |
| 26 | pca_d3_diag | 1.94 | 0.29 | 26 | 57.7 | 0.251 | 0.244 | 0.000 | -0.007 | n/a | -4.07 | 0.956 | 0.044 | 84% / 0.92 |
| 27 | pca_c_diag | 1.70 | 0.28 | 26 | 50.0 | 0.339 | 0.183 | 0.000 | -0.157 | n/a | -5.97 | 0.962 | 0.052 | 87% / 0.93 |
| 28 | baseline | 1.87 | 0.24 | 31 | 48.4 | 0.336 | 0.159 | 0.000 | -0.177 | n/a | -7.41 | 0.963 | 0.057 | 82% / 0.93 |
| 29 | pca_f4_full | 1.29 | 0.18 | 43 | 46.5 | 0.095 | 0.097 | 0.000 | 0.002 | 1.8 | -5.69 | 0.970 | 0.056 | 81% / 0.93 |
| 30 | pca_b6_diag | 1.31 | 0.14 | 32 | 53.1 | 0.253 | 0.080 | 0.000 | -0.173 | n/a | -7.31 | 0.964 | 0.054 | 78% / 0.93 |
| 31 | pca_f4_diag | 1.21 | 0.12 | 43 | 46.5 | 0.157 | 0.054 | 0.000 | -0.103 | n/a | -5.78 | 0.969 | 0.057 | 80% / 0.92 |
| 32 | pca_d4_full | 1.07 | 0.03 | 38 | 50.0 | 0.067 | -0.004 | 0.000 | -0.071 | n/a | -5.38 | 0.970 | 0.053 | 86% / 0.94 |
| 33 | pca_d2_diag | 1.12 | 0.02 | 27 | 48.1 | -0.064 | -0.056 | 0.000 | 0.008 | n/a | -2.82 | 0.957 | 0.049 | 86% / 0.92 |
| 34 | pca_b5_diag | 1.04 | -0.02 | 38 | 42.1 | -0.060 | -0.082 | 0.000 | -0.021 | n/a | -6.94 | 0.967 | 0.060 | 78% / 0.92 |

OOS likelihood and regime diagnostics summarize frozen-model segments across
the full timeline; they were not used for DEV selection. Map churn compares
arbitrary state IDs across refits and is not label-invariant. `n/a` denotes
an undefined statistic; skill % requires positive total and nonnegative skill.

| Candidate | Period | Total log | Trend | Random pick | Skill | Bias | Skill % | Sharpe | PF | Trades | Win % | Max DD % |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| baseline | dev | 0.159 | 0.336 | 0.159 | -0.177 | 0.000 | n/a | 0.24 | 1.87 | 31 | 48.4 | -27.0 |
| baseline | test | 0.291 | 0.289 | 0.288 | 0.002 | 0.000 | 0.7 | 0.76 | 3.00 | 5 | 60.0 | -17.6 |
| pca_d5_full | dev | 0.740 | 0.651 | 0.740 | 0.089 | 0.000 | 12.1 | 0.97 | 2.74 | 49 | 61.2 | -17.0 |
| pca_d5_full | test | 0.289 | 0.313 | 0.286 | -0.024 | 0.000 | n/a | 0.77 | 2.06 | 15 | 53.3 | -17.2 |
- `baseline`: insufficient evidence, DECAYED, Sharpe or PF > 2: review for leakage.
- `pca_d5_full`: insufficient evidence, DECAYED, Sharpe or PF > 2: review for leakage.

Evidence is insufficient for any candidate with fewer than 100 DEV or 30 TEST
trades. See the candidate reports for cost ladders and class gates; a change
in the sign of skill between DEV and TEST is DECAYED.

Reproduce: `scripts\run_hmm_pca.bat`

## XGBoost state-classification experiments

Use XGBoost as a supervised classifier to predict the HMM regime label from the engineered market features. Compare 5-state and 7-state classifications to determine whether additional regime granularity improves predictive performance.

Important: Generate labels using the HMM on training data only, then freeze the fitted HMM and use it to label validation/test observations. Avoid using smoothed probabilities or future information to create labels for a live-prediction experiment.

| Experiment | Features                                       | Classes | Purpose                            |
| ---------- | ---------------------------------------------- | ------- | ---------------------------------- |
| XGB-A      | Original 6 features                            | 5       | Baseline                           |
| XGB-B      | Reduced set: remove skewness and volume change | 5       | Test leaner feature set            |
| XGB-C      | Reduced set + SPY log return                   | 5       | Test market context                |
| XGB-D      | Reduced set + Wilder ATR + SPY return          | 5       | Test volatility and market context |
| XGB-E      | Reduced set + Wilder ATR + SPY return          | 7       | Test higher regime granularity     |

### Training instructions

1. Target: Predict the next trading day's HMM state: \\(X_t \rightarrow S\_{t+1}\\). This tests actual predictive value rather than simply reproducing the current state.
2. Labels: Generate labels using a causal, filtered HMM. Fit the HMM on training data only; freeze it before generating validation/test labels.
3. Model: Use `XGBClassifier` with:
   - `objective="multi:softprob"`
   - `num_class=5` or `7`
   - `eval_metric="mlogloss"`
   - Early stopping using chronological validation data.
4. Splits: Preserve chronological walk-forward splits. Never randomly shuffle observations.
5. Evaluation: Report macro F1, balanced accuracy, per-class recall, confusion matrix, and log loss. Compare against a persistence baseline that predicts tomorrow's state will equal today's state.

Important: HMM state IDs are arbitrary. Keep each experiment's HMM and its state-label mapping fixed across its training, validation, and test periods. XGBoost does not require feature standardization.

### Results (run: `tools/hmm_xgb.py`, QQQ vs SPY, `etf_tr` parquet)

Labeler: canonical 6-feature diagonal Gaussian HMM (numpy port of `src/hmm.rs`),
fit once on the train slice (2016-03-15..2022-06-07, n=1570) and frozen;
val (2022-06-08..2023-12-29, n=393) and test (2024-01-02..2026-10-23, n=705)
labels produced by the frozen causal filter. Class IDs ordered by fitted
mean return. K=5 fit: loglik -8010, 66 iters, P_ii 0.92-0.95.
K=7 fit: loglik -7259, 42 iters, P_ii 0.87-0.94.

| Exp    | K | best_iter | XGB macroF1 (val/test) | XGB balAcc (val/test) | persist balAcc (val/test) | test logloss |
| ------ | - | --------- | -------------------- | ------------------- | ------------------------ | ------------ |
| XGB-A  | 5 | 187       | 0.913 / 0.864        | 0.928 / 0.863       | 0.956 / 0.916            | 0.240        |
| XGB-B  | 5 | 97        | 0.896 / 0.872        | 0.914 / 0.868       | 0.956 / 0.916            | 0.242        |
| XGB-C  | 5 | 96        | 0.892 / 0.860        | 0.911 / 0.853       | 0.956 / 0.916            | 0.245        |
| XGB-D  | 5 | 115       | 0.913 / 0.876        | 0.923 / 0.869       | 0.956 / 0.916            | 0.231        |
| XGB-E  | 7 | 262       | 0.766 / 0.886        | 0.929 / 0.888       | 0.947 / 0.916            | 0.284        |

**Finding: the persistence baseline beats every XGBoost variant on both
splits.** With P_ii ~0.92-0.95 the filtered state is overwhelmingly
self-propagating - predicting "tomorrow = today" is right ~92% of the time,
and no feature set gives XGBoost extra information about *when* a transition
will occur. Every transition bet it makes costs more than it earns
(macroF1 0.86-0.89 vs 0.92). XGB-E (7 classes) is the best XGB variant on
test but collapses on rare classes during val (two classes at 0 recall).
Practical conclusion: regime *identification* is feature-driven, but regime
*persistence* is not - the transition matrix is the predictor. A supervised
classifier would need labels with more transition events (higher-frequency
data or a noisier state definition) to have anything to learn.

Artifacts: `results/hmm_regime/xgb_summary.csv`, `xgb_confusion_xgb-*.csv`,
`xgb_labels_k{5,7}.csv`, `xgb_report.md`.

## Standalone HMM robustness variants

This separate screen uses named overlays from `configs/hmm_regime.yaml` (default universe SPY,
except overlays that change the symbol/universe). All 47 configured variants, including the
preserved `spy_base` baseline, have per-variant reports and DEV/TEST partition files. The
table reports the configured primary fill (`next_open`, except `close`); cells are PF / Sharpe /
trades / win%, followed by trend / random-pick baseline / skill / bias / skill%. Overlays vary
model parameters, symbols, position rules, fill, or costs; rows are therefore not one comparable
feature leaderboard. TEST periods have been inspected and are not fresh holdouts.

| Variant | Config overlay | Fill | DEV partition | TEST partition |
|---|---|---|---|---|
| spy_base | `hmm_states: 3` | next_open | 2.10 / 0.46 / 27 / 51.9%; 0.225 / 0.240 / 0.015 / 0.000 / 6.1% | 2.03 / 0.64 / 10 / 30.0%; 0.123 / 0.158 / 0.035 / 0.000 / 22.3% |
| k2 | `hmm_states: 2` | next_open | 6.54 / 0.93 / 24 / 62.5%; 0.449 / 0.458 / 0.009 / 0.000 / 2.0% | 4.68 / 1.03 / 10 / 70.0%; 0.227 / 0.273 / 0.046 / 0.000 / 16.9% |
| k4 | `hmm_states: 4` | next_open | 2.15 / 0.33 / 19 / 57.9%; 0.102 / 0.136 / 0.034 / 0.000 / 25.2% | 5.07 / 0.58 / 8 / 50.0%; 0.081 / 0.144 / 0.063 / 0.000 / 43.7% |
| k5 | `hmm_states: 5` | next_open | 3.83 / 0.82 / 28 / 64.3%; 0.252 / 0.383 / 0.132 / 0.000 / 34.3% | n/a / 0.73 / 4 / 100.0%; 0.104 / 0.115 / 0.011 / 0.000 / 9.4% |
| refit5 | `hmm_refit_every: 5` | next_open | 1.93 / 0.44 / 40 / 57.5%; 0.122 / 0.232 / 0.110 / 0.000 / 47.4% | 6.61 / 1.17 / 12 / 75.0%; 0.251 / 0.312 / 0.061 / 0.000 / 19.6% |
| refit63 | `hmm_refit_every: 63` | next_open | 1.89 / 0.38 / 19 / 52.6%; 0.155 / 0.197 / 0.042 / 0.000 / 21.1% | 3.54 / 0.74 / 9 / 66.7%; 0.153 / 0.190 / 0.037 / 0.000 / 19.6% |
| tw252 | `hmm_train_window: 252` | next_open | 1.88 / 0.44 / 30 / 40.0%; 0.315 / 0.286 / -0.029 / 0.000 / n/a | 3.83 / 0.93 / 20 / 60.0%; 0.129 / 0.183 / 0.053 / 0.000 / 29.1% |
| tw504 | `hmm_train_window: 504` | next_open | 2.92 / 0.56 / 26 / 69.2%; 0.416 / 0.370 / -0.047 / 0.000 / n/a | 3.12 / 0.87 / 9 / 66.7%; 0.225 / 0.272 / 0.047 / 0.000 / 17.3% |
| tw756 | `hmm_train_window: 756` | next_open | 2.15 / 0.38 / 30 / 70.0%; 0.189 / 0.270 / 0.081 / 0.000 / 29.9% | 2.09 / 0.83 / 7 / 71.4%; 0.189 / 0.188 / -0.001 / 0.000 / n/a |
| pred | `hmm_probs: predicted` | next_open | 1.73 / 0.23 / 25 / 64.0%; 0.148 / 0.113 / -0.035 / 0.000 / n/a | 3.36 / 0.84 / 10 / 70.0%; 0.210 / 0.225 / 0.016 / 0.000 / 7.0% |
| noscale | `hmm_scale: false` | next_open | 2.01 / 0.35 / 24 / 62.5%; 0.204 / 0.178 / -0.026 / 0.000 / n/a | 2.78 / 0.75 / 11 / 63.6%; 0.151 / 0.202 / 0.052 / 0.000 / 25.6% |
| r1 | `hmm_restarts: 1` | next_open | 2.27 / 0.65 / 30 / 63.3%; 0.266 / 0.387 / 0.121 / 0.000 / 31.3% | 8.14 / 1.15 / 11 / 72.7%; 0.257 / 0.304 / 0.047 / 0.000 / 15.6% |
| r8 | `hmm_restarts: 8` | next_open | 1.44 / 0.21 / 25 / 60.0%; 0.114 / 0.100 / -0.015 / 0.000 / n/a | 5.76 / 0.78 / 10 / 80.0%; 0.143 / 0.201 / 0.057 / 0.000 / 28.6% |
| prior0 | `hmm_trans_prior: 0.0` | next_open | 2.41 / 0.41 / 24 / 62.5%; 0.245 / 0.216 / -0.029 / 0.000 / n/a | 3.59 / 0.94 / 9 / 66.7%; 0.266 / 0.247 / -0.020 / 0.000 / n/a |
| prior10 | `hmm_trans_prior: 10.0` | next_open | 2.41 / 0.55 / 20 / 65.0%; 0.265 / 0.288 / 0.023 / 0.000 / 8.0% | 4.14 / 0.91 / 10 / 70.0%; 0.222 / 0.272 / 0.050 / 0.000 / 18.5% |
| train126 | `markov_min_train: 126` | next_open | 1.45 / 0.23 / 23 / 56.5%; 0.150 / 0.114 / -0.036 / 0.000 / n/a | 5.80 / 0.78 / 10 / 80.0%; 0.144 / 0.202 / 0.058 / 0.000 / 28.5% |
| train504 | `markov_min_train: 504` | next_open | 2.44 / 0.40 / 17 / 52.9%; 0.206 / 0.192 / -0.014 / 0.000 / n/a | 5.74 / 0.77 / 10 / 80.0%; 0.154 / 0.200 / 0.045 / 0.000 / 22.7% |
| train756 | `markov_min_train: 756` | next_open | 1.61 / 0.23 / 18 / 38.9%; 0.163 / 0.098 / -0.065 / 0.000 / n/a | 5.76 / 0.78 / 10 / 80.0%; 0.143 / 0.201 / 0.057 / 0.000 / 28.6% |
| hyst | `markov_regime: hysteresis, markov_enter: 0.05, markov_exit: -0.05` | next_open | 1.76 / 0.18 / 17 / 47.1%; 0.100 / 0.075 / -0.024 / 0.000 / n/a | 3.79 / 0.80 / 9 / 66.7%; 0.157 / 0.210 / 0.052 / 0.000 / 25.0% |
| hyst_wide | `markov_regime: hysteresis, markov_enter: 0.10, markov_exit: -0.10` | next_open | 1.66 / 0.16 / 17 / 47.1%; 0.105 / 0.068 / -0.037 / 0.000 / n/a | 3.34 / 0.81 / 8 / 75.0%; 0.219 / 0.214 / -0.005 / 0.000 / n/a |
| strict_up | `markov_enter: 0.10` | next_open | 1.82 / 0.21 / 23 / 56.5%; 0.078 / 0.071 / -0.007 / 0.000 / n/a | 2.61 / 0.88 / 14 / 64.3%; 0.185 / 0.197 / 0.012 / 0.000 / 6.1% |
| strict_dn | `markov_exit: -0.10` | next_open | 1.76 / 0.26 / 25 / 60.0%; 0.155 / 0.125 / -0.030 / 0.000 / n/a | 4.29 / 0.88 / 10 / 70.0%; 0.193 / 0.234 / 0.041 / 0.000 / 17.6% |
| long_short | `markov_position: long_short` | next_open | 0.45 / -0.54 / 49 / 40.8%; -0.495 / -0.881 / -0.386 / 0.000 / n/a | 0.71 / -0.15 / 20 / 45.0%; -0.115 / -0.100 / 0.015 / 0.000 / n/a |
| ls_hyst | `markov_position: long_short, markov_regime: hysteresis, markov_enter: 0.05, markov_exit: -0.05` | next_open | 0.31 / -0.63 / 33 / 30.3%; -0.606 / -1.000 / -0.394 / 0.000 / n/a | 0.60 / -0.27 / 18 / 38.9%; -0.187 / -0.153 / 0.034 / 0.000 / n/a |
| conv | `markov_position: conviction` | next_open | 0.83 / -0.63 / 1499 / 49.2%; -0.357 / -0.837 / -0.480 / 0.000 / n/a | 0.92 / -0.51 / 685 / 50.4%; -0.004 / -0.172 / -0.167 / 0.000 / n/a |
| conv_cap50 | `markov_position: conviction, markov_cap: 0.5` | next_open | 0.77 / -0.45 / 596 / 49.7%; -0.193 / -0.308 / -0.115 / 0.000 / n/a | 0.88 / -0.16 / 188 / 49.5%; -0.006 / -0.030 / -0.024 / 0.000 / n/a |
| conv_cap25 | `markov_position: conviction, markov_cap: 0.25` | next_open | 0.77 / -0.47 / 575 / 50.1%; -0.102 / -0.155 / -0.053 / 0.000 / n/a | 0.88 / -0.16 / 177 / 50.8%; -0.010 / -0.015 / -0.005 / 0.000 / n/a |
| bearrel | `markov_signal_mode: bear_rel` | next_open | 2.61 / 0.65 / 31 / 58.1%; 0.411 / 0.421 / 0.010 / 0.000 / 2.3% | 3.20 / 0.78 / 11 / 63.6%; 0.296 / 0.261 / -0.035 / 0.000 / n/a |
| h3 | `markov_horizon: 3` | next_open | 1.96 / 0.31 / 23 / 60.9%; 0.167 / 0.155 / -0.012 / 0.000 / n/a | 4.26 / 0.88 / 10 / 70.0%; 0.182 / 0.235 / 0.053 / 0.000 / 22.6% |
| h5 | `markov_horizon: 5` | next_open | 1.93 / 0.30 / 22 / 59.1%; 0.174 / 0.153 / -0.021 / 0.000 / n/a | 4.27 / 0.88 / 10 / 70.0%; 0.171 / 0.236 / 0.065 / 0.000 / 27.6% |
| qqq | `symbols: QQQ, benchmark: QQQ, risk_symbol: QQQ` | next_open | 2.36 / 0.40 / 28 / 53.6%; 0.299 / 0.340 / 0.041 / 0.000 / 12.1% | 2.22 / 0.72 / 7 / 57.1%; 0.281 / 0.280 / 0.002 / 0.000 / 0.8% |
| iwm | `symbols: IWM, benchmark: IWM, risk_symbol: IWM` | next_open | 2.02 / 0.37 / 30 / 70.0%; 0.364 / 0.363 / -0.002 / 0.000 / n/a | n/a / 1.07 / 6 / 100.0%; 0.241 / 0.261 / 0.020 / 0.000 / 7.6% |
| dia | `symbols: DIA, benchmark: DIA, risk_symbol: DIA` | next_open | 1.75 / 0.35 / 29 / 41.4%; 0.178 / 0.231 / 0.053 / 0.000 / 23.1% | 2.48 / 0.68 / 8 / 50.0%; 0.119 / 0.203 / 0.081 / 0.000 / 40.6% |
| tlt | `symbols: TLT, benchmark: TLT, risk_symbol: TLT` | next_open | 1.14 / 0.08 / 26 / 46.2%; 0.023 / 0.028 / 0.005 / 0.000 / 18.5% | 1.42 / 0.17 / 6 / 50.0%; -0.007 / 0.041 / 0.043 / 0.000 / 120.1% |
| gld | `symbols: GLD, benchmark: GLD, risk_symbol: GLD` | next_open | 1.94 / 0.32 / 31 / 61.3%; 0.191 / 0.210 / 0.019 / 0.000 / 9.1% | 8.97 / 1.96 / 10 / 60.0%; 0.548 / 0.554 / 0.006 / 0.000 / 1.2% |
| qqq_ls | `symbols: QQQ, benchmark: QQQ, risk_symbol: QQQ, markov_position: long_short` | next_open | 0.82 / -0.39 / 55 / 40.0%; -0.588 / -0.870 / -0.282 / 0.000 / n/a | 0.70 / 0.00 / 14 / 35.7%; 0.005 / -0.061 / -0.063 / 0.000 / n/a |
| src_bench | `markov_source: benchmark` | next_open | 1.76 / 0.26 / 25 / 60.0%; 0.155 / 0.125 / -0.030 / 0.000 / n/a | 4.29 / 0.88 / 10 / 70.0%; 0.193 / 0.234 / 0.041 / 0.000 / 17.6% |
| src_univ | `symbols: "SPY,QQQ,IWM,DIA", markov_source: universe` | next_open | 1.95 / 0.47 / 103 / 55.3%; 1.095 / 0.680 / -0.326 / 0.000 / n/a | 4.52 / 1.16 / 30 / 70.0%; 0.947 / 0.761 / -0.013 / 0.000 / n/a |
| bench_qqq | `symbols: "SPY,QQQ,IWM,DIA", markov_source: benchmark, benchmark: QQQ, risk_symbol: QQQ` | next_open | 1.95 / 0.47 / 103 / 55.3%; 1.465 / 0.680 / -0.696 / 0.000 / n/a | 4.52 / 1.16 / 30 / 70.0%; 1.053 / 0.761 / -0.119 / 0.000 / n/a |
| close | `fills: close` | close | 1.71 / 0.25 / 25 / 64.0%; 0.127 / 0.122 / -0.005 / 0.000 / n/a | 3.08 / 0.72 / 10 / 50.0%; 0.192 / 0.190 / -0.002 / 0.000 / n/a |
| cost05 | `cost_bps_per_side: 0.5` | next_open | 1.77 / 0.26 / 25 / 60.0%; 0.155 / 0.127 / -0.028 / 0.000 / n/a | 4.32 / 0.88 / 10 / 70.0%; 0.193 / 0.235 / 0.042 / 0.000 / 17.9% |
| cost5 | `cost_bps_per_side: 5.0` | next_open | 1.66 / 0.22 / 25 / 60.0%; 0.155 / 0.105 / -0.050 / 0.000 / n/a | 4.02 / 0.85 / 10 / 70.0%; 0.193 / 0.226 / 0.033 / 0.000 / 14.5% |
| rv10 | `hmm_rv_window: 10` | next_open | 3.89 / 0.81 / 28 / 64.3%; 0.461 / 0.468 / 0.007 / 0.000 / 1.4% | 2.52 / 0.73 / 8 / 62.5%; 0.122 / 0.230 / 0.108 / 0.000 / 47.1% |
| rv40 | `hmm_rv_window: 40` | next_open | 1.00 / -0.00 / 14 / 50.0%; 0.014 / -0.017 / -0.031 / 0.000 / n/a | 9.29 / 0.80 / 7 / 85.7%; 0.134 / 0.207 / 0.073 / 0.000 / 35.3% |
| ma20_100 | `hmm_ma_fast: 20, hmm_ma_slow: 100` | next_open | 3.80 / 0.66 / 18 / 66.7%; 0.287 / 0.306 / 0.019 / 0.000 / 6.3% | 14.85 / 1.10 / 6 / 66.7%; 0.253 / 0.299 / 0.046 / 0.000 / 15.5% |
| skew10 | `hmm_skew_window: 10` | next_open | 2.37 / 0.43 / 25 / 48.0%; 0.310 / 0.275 / -0.036 / 0.000 / n/a | 3.77 / 0.84 / 9 / 55.6%; 0.229 / 0.226 / -0.003 / 0.000 / n/a |
| iter200 | `hmm_max_iter: 200` | next_open | 1.76 / 0.26 / 25 / 60.0%; 0.155 / 0.125 / -0.030 / 0.000 / n/a | 4.29 / 0.88 / 10 / 70.0%; 0.193 / 0.234 / 0.041 / 0.000 / 17.6% |

## Appendix

### reference for regime metrics calculation
```python
import numpy as np

def calculate_regime_metrics(states, transition_matrix):
    """
    states: decoded state sequence (e.g. argmax of filtered probabilities)
    transition_matrix: HMM transition matrix, shape (n_states, n_states)
    """
    # Empirical churn: fraction of consecutive observations with a state change
    churn = np.mean(np.asarray(states)[1:] != np.asarray(states)[:-1])

    # Transition-matrix-implied persistence
    self_transition = np.diag(transition_matrix)
    expected_duration = 1.0 / (1.0 - self_transition)

    # Stationary distribution, satisfying pi @ A = pi
    eigenvalues, eigenvectors = np.linalg.eig(transition_matrix.T)
    stationary = np.real(eigenvectors[:, np.argmin(np.abs(eigenvalues - 1))])
    stationary /= stationary.sum()

    weighted_persistence = np.sum(stationary * expected_duration)

    return {
        "churn": churn,
        "mean_self_transition": self_transition.mean(),
        "expected_duration_per_state": expected_duration,
        "stationary_weighted_persistence": weighted_persistence,
    }
```

### Gaussian config sweep: refit windows 14/28/35

All 24 experiments re-run at hmm_refit_every = 14, 28, 35 (72 variants, ~55 min).

| Refit | Best dev skill | Runner-up | Best OOS LL/bar | Notes |
|---|---|---|---|---|
| 14 | r14_X +0.112 (31 tr) | r14_N +0.099 | r14_B -5.95 | most refits, most trades |
| 28 | r28_X +0.134 (26 tr) | r28_N +0.077 | r28_B -6.06 | |
| 35 | r35_K +0.115 (18 tr) | r35_M +0.113 | r35_B -6.27 | stalest fits, fewest trades |

Conclusions: (1) B (-skew, 5 cols) wins held-out likelihood at every window; OOS LL mildly
degrades as refits get sparser (-5.95 / -6.06 / -6.27). (2) Skill leaders are always
trend/vol features — X (ema+sma200) and N (wilder_atr) repeat across windows; K (sma200)
tops refit-35. (3) Test skills stay near zero on 6-14 trades; nothing approaches 100 dev
trades.