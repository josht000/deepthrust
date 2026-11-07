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