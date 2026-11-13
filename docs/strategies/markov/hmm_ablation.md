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
- [x] H-R
- [x] S-Y
- [x] Implement full-covariance Gaussian emissions and compare with diagonal on A, B, K, Q, Y
- [ ] Test various gaussian configurations on top 10 winners
- [ ] Test various seeds on top 10 winners

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
and are not a fresh holdout.---|

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

## PCA Exp Results

| Experiment | Dev/Test PF | Dev/Test Sharpe | Dev/Test Trades | Out-of-sample log likelihood | Regime persistence | Transition frequency | State stability | Downstream trading performance |
|---|---|---|---|---|---|---|---|---|---|

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