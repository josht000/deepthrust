# Daily-Bar Trading Strategy Catalog for Systematic Backtesting

**Version:** 1.2 (revised after adversarial review R1, `docs/strategies/review/r1.md`) - compiled for handoff to a coding/backtesting agent
**Scope:** 105 original strategy numbers; **90 entries remain** after merges (file names keep the original "105" label; strategy numbers are preserved and a merged strategy keeps its LOWEST number, see the merge map below). Most entries use daily OHLCV; entries requiring sub-daily OHLCV for the primary specification or a registered variant are tagged `NOT_DAILY, DEFER` in the TOC and strategy entry. Composition, tagged per entry in the JSON as `frequency_class`:
- **swing** (54 entries): short swings of 1-10 sessions with daily-to-weekly trade frequency across the declared universe.
- **slow** (25 entries): weekly/monthly-rebalance published systems (rotation, factor, pairs, regime gates) included because they are published ETF strategies or come from the attached papers. Use them as overlays or portfolio sleeves.
- **calendar** (11 entries): calendar/rare-event systems that trade only on known dates.

**Status mix:** 62 active, 23 demoted (control / falsification / overlay only), 4 dropped (documented null or untestable edge; kept for the record and excluded from the backtest queue), 1 forward-test only.

**Data-frequency status:** `NOT_DAILY, DEFER` means the strategy requires sub-daily OHLCV for its primary execution or a registered variant. `NOT_DAILY, DELTA` marks an implemented daily-bar adaptation that differs from that planned specification.

**Determinism convention:** where an entry lists alternative fills, exits, or parameters, the FIRST-listed value is the primary baseline; everything after "or"/"optional"/"variant" is a robustness alternative for the backtester to run second. Each entry's `parameter_variants` field lists the registered alternatives; every variant is a registered trial in `results/trial_registry.jsonl` and must not be counted as an independent discovery.

---

## Table of Contents

- [How to use this catalog](#how-to-use-this-catalog)
- [Shared backtest and execution specification (applies to every strategy)](#shared-backtest-and-execution-specification-applies-to-every-strategy)
- [Review R1 resolution log](#review-r1-resolution-log)
- [Family 1 — ETF Index Mean Reversion (Connors lineage)](#family-1--etf-index-mean-reversion-connors-lineage)
    - [S001 | RSI(2) Classic (Connors) - cluster control](#s001--rsi2-classic-connors---cluster-control) — IMPLEMENTED — NOT_DAILY, DELTA
    - [S003 | RSI(4) 25/75 (Connors)](#s003--rsi4-2575-connors) — IMPLEMENTED — NOT_DAILY, DELTA
    - [S004 | Cumulative RSI(2)](#s004--cumulative-rsi2) — NOT_DAILY, DEFER
    - [S005 | Connors %b (Bollinger Band)](#s005--connors-b-bollinger-band) — NOT_DAILY, DEFER
    - [S006 | Double 7s](#s006--double-7s) — IMPLEMENTED — NOT_DAILY, DELTA
    - [S007 | 3-Day Pullback](#s007--3-day-pullback) — NOT_DAILY, DEFER
    - [S008 | ConnorsRSI(3,2,100) Stock Pullback](#s008--connorsrsi32100-stock-pullback) — NOT_DAILY, DEFER
    - [S009 | 3 Days Down Overnight (SPY)](#s009--3-days-down-overnight-spy) — NOT_DAILY, DEFER
    - [S010 | Turnaround Tuesday](#s010--turnaround-tuesday)
    - [S011 | Z-Score / Bollinger Lower-Band Mean Reversion](#s011--z-score--bollinger-lower-band-mean-reversion) — NOT_DAILY, DEFER
    - [S013 | TLT Mean Reversion (RSI(2))](#s013--tlt-mean-reversion-rsi2)
    - [S014 | Sector ETF Basket Mean Reversion](#s014--sector-etf-basket-mean-reversion) — NOT_DAILY, DEFER
- [Family 2 — Overnight and Gap Strategies](#family-2--overnight-and-gap-strategies)
    - [S015 | Chan Buy-on-Gap](#s015--chan-buy-on-gap) — NOT_DAILY, DEFER
    - [S016 | Short-on-Gap-Up (Chan mirror)](#s016--short-on-gap-up-chan-mirror) — NOT_DAILY, DEFER
    - [S017 | Night Trading Overnight-Bias Stock Selection](#s017--night-trading-overnight-bias-stock-selection) — IMPLEMENTED — DAILY, DELTA
    - [S018 | Gap Fade (Fill the Gap)](#s018--gap-fade-fill-the-gap) — NOT_DAILY, DEFER
    - [S019 | Gap-and-Go Continuation](#s019--gap-and-go-continuation) — IMPLEMENTED
    - [S020 | Earnings-Eve Estimate-Drift Mean Reversion (Chan/Estimize)](#s020--earnings-eve-estimate-drift-mean-reversion-chanestimize) — DROPPED
    - [S021 | Index Overnight Premium (SPY close-to-open only)](#s021--index-overnight-premium-spy-close-to-open-only) — IMPLEMENTED
    - [S023 | Post-Earnings Overnight Drift](#s023--post-earnings-overnight-drift) — IMPLEMENTED
    - [S024 | Ex-Dividend Window Capture](#s024--ex-dividend-window-capture) — IMPLEMENTED
    - [S026 | Exhaustion Gap Fade After 3-Day Run](#s026--exhaustion-gap-fade-after-3-day-run) — NOT_DAILY, DEFER
- [Family 3 — Cross-Sectional Short-Term Reversal](#family-3--cross-sectional-short-term-reversal)
    - [S027 | Lehmann 1-Week Loser Reversal](#s027--lehmann-1-week-loser-reversal) — NOT_DAILY, DEFER
    - [S028 | Jegadeesh 1-Month Reversal](#s028--jegadeesh-1-month-reversal) — NOT_DAILY, DEFER
    - [S029 | Capitulation-Volume Loser Reversal](#s029--capitulation-volume-loser-reversal) — NOT_DAILY, DEFER
    - [S030 | Down-Streak Reversal (5+ Days)](#s030--down-streak-reversal-5-days) — NOT_DAILY, DEFER
    - [S031 | Up-Streak Fade (5+ Days)](#s031--up-streak-fade-5-days)
    - [S032 | 52-Week-High Proximity Portfolio (George-Hwang)](#s032--52-week-high-proximity-portfolio-george-hwang) — IMPLEMENTED
    - [S035 | Low-Volatility Anomaly Portfolio](#s035--low-volatility-anomaly-portfolio) — IMPLEMENTED
- [Family 4 — Momentum, Breakout, and Rotation](#family-4--momentum-breakout-and-rotation)
    - [S033 | 52-Week-High Breakout (Donchian-style)](#s033--52-week-high-breakout-donchian-style) — IMPLEMENTED — DAILY, DELTA
    - [S036 | Donchian Breakout, Turtle Systems 1 and 2](#s036--donchian-breakout-turtle-systems-1-and-2)
    - [S038 | NR7 / Inside-Day Breakout](#s038--nr7--inside-day-breakout)
    - [S040 | Volume-Confirmed 20-Day-High Breakout](#s040--volume-confirmed-20-day-high-breakout) — IMPLEMENTED
    - [S041 | 12-1 Cross-Sectional Momentum](#s041--12-1-cross-sectional-momentum) — IMPLEMENTED
    - [S042 | Sector Momentum Rotation](#s042--sector-momentum-rotation) — IMPLEMENTED
    - [S043 | ETF Multi-Asset Momentum Rotation](#s043--etf-multi-asset-momentum-rotation) — IMPLEMENTED
    - [S044 | Efficiente 5 MPT Replication](#s044--efficiente-5-mpt-replication) — IMPLEMENTED
    - [S045 | Multi-Factor Characteristic Blend (US Adaptation; S045a paper design, S045b catalog design)](#s045--multi-factor-characteristic-blend-us-adaptation-s045a-paper-design-s045b-catalog-design) — IMPLEMENTED
    - [S046 | Industry Rotation via Market-State Similarity](#s046--industry-rotation-via-market-state-similarity) — IMPLEMENTED
    - [S047 | Absolute / Dual Momentum (S047a absolute SPY vs cash; S047b Antonacci GEM)](#s047--absolute--dual-momentum-s047a-absolute-spy-vs-cash-s047b-antonacci-gem) — IMPLEMENTED
    - [S048 | Validea Short-Term Sector Momentum](#s048--validea-short-term-sector-momentum)
    - [S049 | Acceleration Momentum](#s049--acceleration-momentum)
    - [S050 | First Pullback After Breakout](#s050--first-pullback-after-breakout) — IMPLEMENTED
    - [S051 | Darvas Box Breakout](#s051--darvas-box-breakout) — IMPLEMENTED
- [Family 5 — Earnings, News, and LLM Strategies](#family-5--earnings-news-and-llm-strategies)
    - [S052 | Earnings Announcement Premium (Monthly)](#s052--earnings-announcement-premium-monthly)
    - [S053 | Pre-Earnings Run-Up (consensus-free; replaces the Earnings Whispers Score setup)](#s053--pre-earnings-run-up-consensus-free-replaces-the-earnings-whispers-score-setup)
    - [S054 | PEAD Earnings-Surprise Drift (SUE / announcement-return, point-in-time)](#s054--pead-earnings-surprise-drift-sue--announcement-return-point-in-time)
    - [S055 | Post-Earnings-Oversold Bounce](#s055--post-earnings-oversold-bounce) — NOT_DAILY, DEFER
    - [S056 | LLM Daily News Sentiment Long/Short](#s056--llm-daily-news-sentiment-longshort)
    - [S057 | LLM Top-Down Sector Allocation (forward test only)](#s057--llm-top-down-sector-allocation-forward-test-only)
    - [S058 | News-Topic Conditioned Sentiment (feature design for S056)](#s058--news-topic-conditioned-sentiment-feature-design-for-s056)
    - [S059 | Insider Cluster Buying](#s059--insider-cluster-buying)
    - [S060 | News-Intensity Overnight Drift (conditioning feature of S056)](#s060--news-intensity-overnight-drift-conditioning-feature-of-s056) — NOT_DAILY, DEFER
- [Family 6 — Volatility Regime and Market Timing](#family-6--volatility-regime-and-market-timing)
    - [S061 | VIX Spike Mean Reversion (SPY)](#s061--vix-spike-mean-reversion-spy) — NOT_DAILY, DEFER
    - [S062 | VIX/VXV Ratio Spike (Term Structure)](#s062--vixvxv-ratio-spike-term-structure)
    - [S063 | VIX Moving-Average Crossover Regime](#s063--vix-moving-average-crossover-regime)
    - [S064 | Hurst Exponent Regime Switch](#s064--hurst-exponent-regime-switch)
    - [S065 | Realized-Vol Compression Breakout (compression cluster champion candidate)](#s065--realized-vol-compression-breakout-compression-cluster-champion-candidate)
    - [S066 | Zweig Breadth Thrust](#s066--zweig-breadth-thrust)
    - [S067 | Relative Sector Breadth Rotation](#s067--relative-sector-breadth-rotation)
    - [S069 | TLT Crash-Hedge Rotation](#s069--tlt-crash-hedge-rotation)
    - [S070 | QQQ Regime Switches with Real TQQQ History (public rules only)](#s070--qqq-regime-switches-with-real-tqqq-history-public-rules-only)
- [Family 7 — Pairs and Spread Strategies](#family-7--pairs-and-spread-strategies)
    - [S071 | Cointegrated ETF Pair Z-Score](#s071--cointegrated-etf-pair-z-score)
    - [S072 | Gold/Silver Ratio Mean Reversion](#s072--goldsilver-ratio-mean-reversion)
    - [S073 | GDX/GDXJ Miners Spread](#s073--gdxgdxj-miners-spread) — DROPPED
    - [S075 | Stock-vs-Sector Residual Reversal](#s075--stock-vs-sector-residual-reversal) — IMPLEMENTED — DAILY, DELTA
    - [S076 | Twin-ETF Convergence (SPY/IVV/VOO)](#s076--twin-etf-convergence-spyivvvoo) — DROPPED
    - [S077 | Energy Pair: XLE/USO Spread](#s077--energy-pair-xleuso-spread) — DROPPED
    - [S078 | Chan-Style Statistical Arbitrage Basket](#s078--chan-style-statistical-arbitrage-basket)
- [Family 8 — Calendar Strategies](#family-8--calendar-strategies)
    - [S079 | Turn-of-the-Month Effect (window grid)](#s079--turn-of-the-month-effect-window-grid)
    - [S080 | Pre-Holiday Effect](#s080--pre-holiday-effect)
    - [S081 | Pre-FOMC Announcement Drift](#s081--pre-fomc-announcement-drift) — NOT_DAILY, DEFER
    - [S082 | Options-Expiration-Week Drift](#s082--options-expiration-week-drift)
- [Family 9 — Trend and Pullback Systems (Stocks)](#family-9--trend-and-pullback-systems-stocks)
    - [S087 | 20-Day-EMA Pullback in Uptrend](#s087--20-day-ema-pullback-in-uptrend) — NOT_DAILY, DEFER
    - [S088 | MACD Bullish Cross with Volume](#s088--macd-bullish-cross-with-volume)
    - [S089 | Golden Cross with Slope Filter](#s089--golden-cross-with-slope-filter)
    - [S090 | KAMA Breakout](#s090--kama-breakout)
    - [S091 | Donchian 55 with Chandelier Trail](#s091--donchian-55-with-chandelier-trail)
    - [S093 | Higher-Low Reversal Entry](#s093--higher-low-reversal-entry) — NOT_DAILY, DEFER
    - [S094 | Bollinger Squeeze Breakout](#s094--bollinger-squeeze-breakout)
    - [S095 | Keltner Channel Breakout](#s095--keltner-channel-breakout)
- [Family 10 — Candlestick and Bar Patterns](#family-10--candlestick-and-bar-patterns)
    - [S096 | Three-Bar Reversal (Bulkowski)](#s096--three-bar-reversal-bulkowski) — NOT_DAILY, DEFER
    - [S097 | Bullish Engulfing After Down-Streak](#s097--bullish-engulfing-after-down-streak)
    - [S098 | Hammer at Lower Bollinger Band](#s098--hammer-at-lower-bollinger-band) — NOT_DAILY, DEFER
    - [S099 | Outside-Day Key Reversal at 200-SMA](#s099--outside-day-key-reversal-at-200-sma)
    - [S100 | Three White Soldiers Continuation](#s100--three-white-soldiers-continuation)
- [Family 11 — Miscellaneous Structural Edge](#family-11--miscellaneous-structural-edge)
    - [S102 | Copper/Gold Ratio Equity Tilt (CPER/GLD ratio, both in cache)](#s102--coppergold-ratio-equity-tilt-cpergld-ratio-both-in-cache)
    - [S103 | Yield-Curve Financials/Utilities Rotation](#s103--yield-curve-financialsutilities-rotation)
    - [S104 | High-Beta/Momentum Crash Protection (overlay comparison on S041)](#s104--high-betamomentum-crash-protection-overlay-comparison-on-s041)
    - [S105 | Volatility-Targeted Overnight Carry](#s105--volatility-targeted-overnight-carry) — IMPLEMENTED
  - [Appendix - Source index](#appendix--source-index)

---

## How to use this catalog

Each entry is written so a coding agent can implement it without interpretation. Rules are deterministic: every indicator is named with its formula/parameters, and entries and exits specify the session and fill convention. The former "at most 4 indicators" cap is dropped as a hard rule (several entries exceeded or misstated it); each entry lists every indicator it uses.

**Provenance labels:**
- **source-specified** - implements documented rules faithfully (re-labelled where the cited source lacked the rule or numbers).
- **adapted** - the cited source supports the mechanism; horizon, universe, execution, or parameters were changed for daily-bar/overnight-hold constraints (changes noted).
- **research-hypothesis** - the combination is proposed here; cited literature supports the components, not the complete system. Treat as a candidate, not a validated edge.

**Entry fields added by review R1:**
- `status`: `active` (implement), `demoted` (control/falsification/overlay only), `dropped` (documented null; excluded from the queue), `forward_test_only` (cannot be backtested cleanly; live paper capture only).
- `priority`: P1 (high value, feasible now) to P4 (retire or last).
- `data_needs`: required series; `DP-*` codes are defined in the data-prerequisite table below.
- `parameter_variants`: registered alternatives in the trial grid.
- `merged_from`: original strategy numbers represented by this entry.
- `review`: the R1 verdict and the source-check result.

**Performance claims:** almost nothing in this catalog has been verified by an independent backtest. Where a source reports performance it is quoted in the notes and is unverified until reproduced under the specification below. The old blanket "Sharpe > 2 / profit factor > 2" screen is replaced by class gates (see Validation): it would reject every published slow sleeve (Efficiente 5 Sharpe 0.5-0.6) and tells nothing about overlays, while PF > 2 on fewer than 100 trades is routinely produced by noise.

---

## Shared backtest and execution specification (applies to every strategy)

**Data.** Daily OHLCV with TWO canonical series per symbol; each strategy declares which one it consumes: `tr` (split- and dividend-adjusted total return; for ETFs, rotation, timing, anything holding bond/cash ETFs) and `raw` (unadjusted OHLC plus dividends credited once at the ex-date; for close-to-open/overnight decompositions, dividend strategies and ex-date studies; a split-only series understates SPY's overnight return by roughly its dividend yield). Point-in-time universes, delisted securities included; delisting return -30% for performance delistings if unknown, 0% for cash mergers. Window: the MAXIMUM available history plus at least 1 year of warm-up; the stock cache starts 2016-01-04 (Alpaca free-tier floor, split-adjusted only), so ETF/VIX/GFC-regime research needs the data prerequisites below (Yahoo/Stooq/CBOE/FRED for pre-2016). The old "15-year window" is replaced by "maximum available, minimum number of independent events" and a regime-coverage table (2016-19, 2020, 2021, 2022, 2023-26) in every report.

**Universe rule.** "S&P 500 stocks", "SP1500" and "top-500" mean `liquid_top_N`: at each rebalance date rank common stocks by trailing 60-day median dollar volume, take the top N (N=500 S&P-500-like, N=1500 SP1500-like, N=100 for the Borsa-style top-100), require price >= $5 and >= 250 prior bars. No historical index-membership table exists, so this price-only rule is the PIT definition.

**Execution convention (default).**
1. Signals are computed from completed session t data (close, high, low, volume of session t).
2. Entries execute at the **open of session t+1**, unless the entry rule explicitly names the close (close-based systems from the Connors/Chan lineage).
3. **Three fill models for close-based systems:** `moc_proxy` (primary: signal from the 15:50 bar, fill at the official close; closing-auction impact takes 3-5 days to dissipate per Wu & Jegadeesh 2020), `next_open`, and `close` (computed on the 16:00 close itself, reported only as an UPPER BOUND because that close cannot be traded with certainty). Until minute data exist for the ETF universe, `next_open` is the primary and `close` is flagged as an upper bound.
4. Exits evaluated on completed-session data execute at the **next open**, except where the exit rule explicitly names a close.
5. Overnight-only systems (close-to-next-open) enter at the close of session t using only information through session t-1 (or the 15:50 bar), and exit at the open of session t+1.
6. **Open-based signals** (gap strategies, news at the open) cannot be filled AT the open that created the signal: model entry at the close of the first-minute (09:31) bar with the RTH slippage, or the opening print with a fill-probability haircut.
7. **Stops and targets** are evaluated against the session's low/high; if both stop and target are touched in the same bar, assume the stop filled first. **Gaps:** a long stop fills at min(open, stop); a short stop / buy-stop fills at max(open, level); limit orders fill at the limit or better. Stop-entry orders fill at max(open, level) (buys) / min(open, level) (sells).
8. Announcement exits ("sell at the open after the announcement") INCLUDE the announcement reaction for both after-close and pre-market reporters.

**Costs.** Cost ladder, not a single number: SPY/index-ETF overnight and auction fills 0.5 / 1 / 2 / 5 bps per side (report the break-even cost; a daily round trip at 5 bps/side is ~25%/yr and exceeds any plausible premium); liquid ETFs 5 bps per side; stocks spread/ADV-aware (2-4 bps for the top dollar-volume decile up to 20-60 bps for small caps; the catalog default 10-15 bps; Chan's 10 bps round trip is large-cap auction fills). The repo's own constants (10 bps commission + 15 bps slippage) disagree with these and must be reconciled before reporting. A turnover table (annual one-way turnover x cost) is required in every report. Shorts: run every strategy LONG-ONLY first (hedged with an SPY/IWM short where stated) and treat short legs as a separate experiment with borrow scenarios (0.3% GC, 3% and 10% annualised for hard-to-borrow); no borrow/locate data exists.

**Validation.**
- Trial registry: one line in `results/trial_registry.jsonl` per (strategy, variant, parameter set) via `log_trials()`; pre-register primary parameters in `configs/<strategy>.yaml` before the first run; every other listed variant is an additional trial. Report the Deflated Sharpe Ratio with the registry's N and require DSR > 0.95 on the locked test partition.
- Partition: dev 2016-01 to 2023-12, test 2024-01 to the last completed session, locked (the ~4-month test window of the current config is too short for slow/calendar strategies); AGENTS.md's partition table (trend, random-pick baseline, skill, bias, skill %), Sharpe, profit factor, # trades and win rate % are still required for dev and test. Rotation strategies add combinatorial purged CV.
- Minimum evidence: >= 100 independent trades (or >= 8 years of monthly rebalances) in dev and >= 30 in test; otherwise print `insufficient evidence`, not a skill number. Report effective sample size (sample length / holding period for multi-day holds).
- Baselines by class (in addition to trend/random/skill/bias): long-flat timing -> exposure-matched random timing, buy-and-hold and SMA200 timing; overnight-only -> unconditional close-to-open drift of the same instrument; cross-sectional baskets -> vol-matched equal-weight universe with cash; rotation -> equal-weight of the same universe, SPY and 60/40 (S044: the published JPME5 figures); pairs/market-neutral -> hedge-ratio-shuffled pairs / random same-sector pair; event/calendar -> placebo dates matched by weekday and volatility.
- Class gates (replace Sharpe > 2 / PF > 2): rotation/factor sleeves net Sharpe >= matched baseline + 0.2 with DSR > 0.9; swing rules net PF >= 1.3 and net expectancy >= 2x round-trip cost; plus bootstrap CIs excluding the baseline, a parameter surface (not the best cell), a cost ladder (0.5x-3x), a subperiod table with no sign flip, and a rolling 24-month CUSUM decay flag (a strategy whose test-period skill sign differs from the dev sign is marked `DECAYED`).
- Scale: run at research scale (frictionless sizing, costs in bps) and at paper scale ($5,000, 5 positions of $1,000) separately and state which one a number refers to.
- Cluster control: after the first runs compute the return-correlation matrix, cluster at |rho| > 0.6 and carry ONE champion per cluster (by DSR); merged entries and variants are counted in the registry but not reported as independent discoveries.
- Earnings-window flag: single-stock mean-reversion rules (S008, S029, S030, S050, S075, S087, S093) report with and without signals within +/-3 sessions of an earnings release.

**LLM/news strategies (S056-S058, S060; S057 forward test only):** the repo holds ~75k timestamped news rows from 2023-09 (data/pit/news/news.parquet); historical archives of 15 years are not available. Use frozen models (FinBERT 2019), post-training-cutoff windows only, masked-ticker and cross-model leakage checks, and live capture for forward evidence. S052 and S086 (now merged into S052) are earnings-calendar strategies, not LLM/news strategies.

---

## Review R1 resolution log

Source: `docs/strategies/review/r1.md`. Strategy numbers are preserved. **KEEP** entries received minor edits, **FIX** entries were corrected per the review, **MERGE** entries were folded into the lowest-numbered strategy of the pair/group (the other number is retired and is NOT reused; its rule survives as a registered alternative in `parameter_variants` of the survivor), **BLOCKED** entries were resolved by rebuilding them on data that exists or by re-scoping, and **DROP/DEMOTE** entries stay in place with `status` set so the numbering and the record are intact.

### Merge map (retired number -> surviving number)

| Retired | Survivor | How the retired rule is kept |
|---|---|---|
| S002 | S001 | sizing variant `pyramid3` of S001 |
| S012 | S011 | cells of the S011 grid: no SMA200 gate, RSI(2) < 10 filter |
| S022 | S015 | `fixed_pct` gap rule (-1/-2/-3%) of S015 |
| S025 | S019 | index-ETF universe and index gap rule of S019 |
| S034 | S017 | monthly-formation and holding-convention variants of S017 |
| S037 | S036 | speed 55 / System 2 of S036 |
| S039 | S038 | `stop_target` exit variant of S038 |
| S068 | S043 | `ma200_distance` signal variant of S043 |
| S074 | S043 | two-asset SPY/QQQ instance of S043 |
| S083 | S079 | open-to-close window cell of the S079 window grid |
| S084 | S021 | per-weekday / weekend overnight table in S021 |
| S085 | S010 | enhanced Monday-decline + VIX variant of S010 |
| S086 | S052 | weekly cadence variant of S052 |
| S092 | S001 | regime-gate variant `ret21 > 0` of S001 |
| S101 | S024 | hold-through-ex-date window of S024 |

### BLOCKED strategies and how they were resolved

| ID | Resolution |
|---|---|
| S053 | Rebuilt consensus-free: pre-earnings run-up on SimFin Publish Date sorted by past announcement-period volume concentration; exit explicitly includes the reaction. |
| S057 | Cannot be backtested (paper backtest is Jan-Jun 2019 only): `forward_test_only` with frozen model and timestamped live capture. |
| S058 | Re-scoped from a placeholder trading rule to a topic-conditioned sentiment FEATURE for S056, testable on news.parquet. |
| S060 | Re-scoped as a news-intensity conditioning feature of S056 (`variant_of: S056`), testable on the 2023-09+ archive with a minimum-count guard. |
| S067 | Sector breadth computed from our own PIT stock universe mapped to sectors (SimFin industry codes) instead of PIT ETF holdings. |
| S070 | Public rules only (oversold -> TQQQ regardless of SMA; QQQ above SMA; SHY/BIL below), real TQQQ history from 2010-02-11, exposure cap; hidden overheated/momentum-mix legs dropped. |
| S102 | CPER/GLD are both in the stock cache from 2016; CPER/GLD ratio tested against a plain cyclical-minus-defensive relative-momentum control. |

### Dropped (documented null or untestable edge)

S020 (the cited source reports the price-only delta rule as a null benchmark), S073 (structural convergence, not mean reversion), S076 (Springer's own poorest case; edge below cost), S077 (USO roll yield; not a stationary pair). Kept with `status: dropped`; one-off falsification or cost-sanity runs only.

### Data prerequisites referenced by `data_needs`

These are build tasks outside the catalog; strategies whose prerequisites are missing are auto-`BLOCKED` in the registry until the data exists.

| Code | Content | Script / status |
|---|---|---|
| DP-ETF | ETF/index daily bars with separate `close_raw`, `close_tr`, `dividend`, `split_factor` (Alpaca adjustment=all cross-checked with raw + dividends; Yahoo/Stooq for pre-2016). Needed: SPY, QQQ, IWM, DIA, the SPDR sectors, TLT, IEF, SHY, BIL, LQD, HYG, EEM, EFA, EMB, IYR, GSG, GLD, SLV, TIP, a dollar ETF, RSP, AGG, TQQQ. In the stock cache today: only GLD, SLV, USO, CPER, GSG; SPY/QQQ/IWM only from 2023-09 in data/pit/market. | scripts/fetch_etf_daily.py (to build) |
| DP-VIX | CBOE VIX, VIX3M (former VXV) and VIX9D daily history (free CSVs; re-verify URLs). | scripts/fetch_cboe_vol.py (to build) |
| DP-FRED | 3-month T-bill (cash proxy before BIL, 2007) and 10Y/2Y yields from FRED. | scripts/fetch_fred_rates.py (to build) |
| DP-FORM4 | SEC Insider Transactions Data Sets (quarterly bulk TSV, 2006+); no Form 4 in data/pit/edgar today. | scripts/fetch_form4.py (to build) |
| DP-EARN | Earnings-event table from SimFin `Publish Date` (3,705 tickers from 2009), quarterly EPS for SUE, EDGAR 8-K Item 2.02 `acceptance_utc` (2023-09+) for BMO/AMC classification. | scripts/build_earnings_events.py (to build) |
| DP-CAL | Exchange holiday, FOMC, options-expiry and ex-dividend calendars (dividends 2016+ from Alpaca corporate actions or Polygon/Massive; SimFin Dividend only 2020-11 to 2025-10). | calendar builders (to build) |
| D1 | data/cache/daily_10years: 11,024 stock symbols, 2016-01-04 onward, split-adjusted only, delisted names included. | exists |
| DP-NEWS | data/pit/news/news.parquet: 75,262 rows, 1,919 symbols, effectively 2023-09 to 2026-10 (Alpaca/Benzinga); live capture via src/data/news_client.py. | exists (short window) |

**Requirements impact (AGENTS.md rule: notify the owner):** `requirements.txt` omits `scipy`, `scikit-learn` and `yfinance` (imported by repo code) and will need `statsmodels` (ADF, Engle-Granger, Newey-West) and optionally `PyPortfolioOpt` for S044. Not modified here.

---

# Family 1 — ETF Index Mean Reversion (Connors lineage)

*Why this family works: broad equity index ETFs mean-revert at short horizons. Daily index returns are negatively autocorrelated over 1–5 days because of liquidity provision: panicky sellers are compensated by patient buyers, and index-level dips that occur without fundamental news are usually repaired. The 200-day SMA regime filter restricts dip-buying to bull regimes where the drift assists the bounce. Chan's 2008 SP1500 example showed Sharpe 4.8 before costs (3.5 after 10 bps round trip) over a single year (2008): one extreme-volatility year is not evidence of enduring profitability. The Connors evidence base is hypothetical backtests without transaction costs or data-mining corrections (CXO), and the S&P 500 edge is largely a post-2000 phenomenon.*

### S001 | RSI(2) Classic (Connors) - cluster control
- family: ETF index mean reversion
- ohlcv_status: NOT_DAILY, DELTA
- concept: Deep short-term oversold readings inside a bull regime revert to the mean within days; Connors found buying RSI(2) below 5 returned more than below 10.
- universe: SPY, QQQ, IWM, DIA plus the 11 SPDR sector ETFs (per-ETF and pooled results)
- direction: long
- indicators: RSI(2) on closes; SMA(200); SMA(5)
- entry: Gate AND RSI(2)(t) < 10; buy at close of t. Gate: Close(t) > SMA200 (primary). fill: moc_proxy primary (signal from the 15:50 bar, fill at the official close); next_open alternate; close fill reported only as an upper bound. Position size: one full unit (primary).
- exit: Close > 5-day SMA; sell at close of that session (primary). No stop in the primary; disaster-stop variant: close < entry - 3 x ATR(14).
- hold: 1-5 sessions typical (1-7 with pyramiding)
- frequency: several trades per month on SPY
- provenance: source-specified
- status: active
- priority: P2
- sources: [StockCharts ChartSchool — RSI(2)](https://chartschool.stockcharts.com/table-of-contents/trading-strategies-and-models/trading-strategies/rsi-2)
- notes: StockCharts: SMA200 filter, RSI(2) < 10 (below 5 returned more), exit above 5-day SMA, buy before the close, no stops. Independent evidence says the edge is regime-limited and post-2000 (Trading Garden S&P 500: 1963-today CAGR 5.4% vs 10.9% buy-and-hold; 2000-today 8.6% vs 8.3%, 187 trades; Be The Puck: beat buy-and-hold Sharpe on 50/371 stocks). Cluster control for the whole Connors family (S003-S014, S030); report per-ETF and pooled, vs exposure-matched random timing and vs buy-and-hold / SMA200 timing. Every combination below is a registered trial.
- parameter_variants: RSI threshold {10 primary, 5}; exit {close > SMA5 primary, RSI(2) > 70}; fill {moc_proxy primary, next_open, close}; gate {SMA200 primary, 21-day return > 0 (ex-S092: replaces the 200-day gate, faster regime adaptation)}; sizing {single unit primary, pyramid3 (ex-S002: unit = 1/3 of the position budget; buy 1 unit at RSI(2) < 10, add 1 unit on each NEW session with RSI(2) < 5 while holding, max 3 units, all units exit together on close > SMA5)}; disaster stop {off primary, close < entry - 3 x ATR(14)}
- merged_from: S002, S092
- data_needs: ETF total-return daily series (DP-ETF)
- review: KEEP (cluster control) | source OK
- implementation_results: `src/strategies/s001_rsi2_connors`; Alpaca `adjustment=all` ETF TR bars (`data/cache/etf_tr`, tools/fetch_tr_daily.py), pooled 15-ETF book, 5 bps/side, next_open primary + close upper bound (no ETF minute bars for moc_proxy); dev < 2024-01-01 <= test. `rsi5` = `rsi_threshold: 5` (dev-selected of 9 registered variants; every dev skill is still <= 0 after the selection haircut - the positive close-fill column is the closing-auction upper bound, not tradable edge).

  | variant | dev total (log) | dev skill | dev Sharpe | dev max DD | test total (log) | test skill | test Sharpe | test max DD | trades (dev+test) |
  |---|---|---|---|---|---|---|---|---|---|
  | baseline (next_open) | +0.06 | -0.05 (p=0.85) | 0.21 | -11% | +0.13 | -0.02 (p=0.78) | 1.32 | -4% | 754+418 |
  | rsi5 (next_open) | +0.05 | -0.01 (p=0.60) | 0.20 | -8% | +0.06 | -0.02 | 1.05 | -3% | 392+209 |
  Reproduce: baseline `python run.py --strategy s001_rsi2_connors`; variants `python run.py --strategy s001_rsi2_connors --variant rsi5,exit70,gate21,pyramid3,stop3atr,rsi5_exit70,rsi5_gate21,rsi5_pyramid3,rsi5_stop3atr`

### S003 | RSI(4) 25/75 (Connors)
- family: ETF index mean reversion
- ohlcv_status: NOT_DAILY, DELTA
- concept: Slightly slower RSI smooths the oversold signal; Connors' original used exit at RSI 75, but the modified exit at 55 halves drawdowns with similar expectancy.
- universe: QQQ, SPY
- direction: long
- indicators: RSI(4); SMA(200)
- entry: Close(t) > SMA200 AND RSI(4)(t) < 25; buy at close of t (fill: moc_proxy primary (signal from the 15:50 bar, fill at the official close); next_open alternate; close fill reported only as an upper bound)
- exit: RSI(4) > 55; sell at close of that session (primary). Registered alternate: RSI(4) > 75 (Connors' original).
- hold: 2-6 sessions
- frequency: several trades per month
- provenance: source-specified
- status: active
- priority: P3
- sources: [Quantified Strategies — RSI 25 & RSI 75](https://www.quantifiedstrategies.com/larry-connors-rsi-25-rsi-75/)
- notes: Quantified Strategies reports QQQ profit factor 2.5, 76% win, 148 trades (~7 trades/yr), 0.9% avg gain; the same page's table shows PF 2.97 (inception to Nov-2020), so the page is internally inconsistent - retest and report a bootstrap CI for PF. The exit level 55 was chosen with hindsight from drawdown results (a selected parameter): count 55 and 75 as separate trials. The 'add a second unit if RSI(4) < 20' variant is NOT part of the baseline. Same trigger family as S001 (G-10).
- parameter_variants: exit {RSI(4) > 55 primary, RSI(4) > 75}; fill {moc_proxy, next_open}
- data_needs: ETF total-return daily series (DP-ETF)
- review: KEEP (cluster member) | source OK
- implementation_results: `src/strategies/s003_rsi4_2575` (same swing engine as s001); QQQ+SPY TR book, 5 bps/side, next_open primary; dev < 2024-01-01 <= test. `insufficient evidence`: 84 dev / 44 test trades (<100 dev) - the 2-ETF universe cannot reach the trade-count gate; the pooled ETF family numbers live under S001.

  | variant | dev total (log) | dev skill | dev Sharpe | dev max DD | test total (log) | test skill | test Sharpe | test max DD | trades (dev+test) |
  |---|---|---|---|---|---|---|---|---|---|
  | baseline (next_open) | +0.18 | -0.05 (p=0.77) | 0.37 | -9% | +0.11 | -0.12 (p=0.99) | 0.51 | -10% | 84+44 |
  | exit75 (next_open) | +0.47 | -0.12 (p=0.99) | 0.55 | -27% | - | - | - | - | 72 dev |
  Reproduce: baseline `python run.py --strategy s003_rsi4_2575`; variant `python run.py --strategy s003_rsi4_2575 --variant exit75`

- rust_port: `configs/s003_rsi4_2575.yaml`, shared `src/study.rs` swing pipeline. Next-open is primary because the catalog's MOC proxy requires unavailable minute bars; close is retained as an upper bound. Reproduce: `cargo run --release -- --strategy s003_rsi4_2575`; baseline plus registered variant: `cargo run --release -- --strategy s003_rsi4_2575 --variant exit75`. Local run on ETF TR bars through 2026-10-23:

  | variant | dev total (log) | dev skill | dev Sharpe | dev PF (95% bootstrap CI) | dev max DD | dev trades | test total (log) | test skill | test Sharpe | test PF (95% bootstrap CI) | test max DD | test trades |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | +0.182 | -0.046 | 0.37 | 1.81 (1.03-3.79) | -9.2% | 84 | +0.105 | -0.120 | 0.51 | 1.76 (0.81-5.39) | -9.9% | 44 |
  | exit75 | +0.473 | -0.118 | 0.55 | 3.24 (1.56-11.43) | -27.4% | 72 | +0.253 | -0.112 | 0.89 | 3.85 (1.66-17.60) | -10.1% | 34 |
  Both have insufficient dev evidence (<100 trades). PF exceeds 2 for `exit75`; the confidence intervals are wide, so treat that as a small-sample warning, not a result to promote. Entry signals use close(t) and fill next_open(t+1); exits use RSI through the decision close and fill next_open, with no future bars used for signal formation.

### S004 | Cumulative RSI(2)
- family: Connors oversold - cumulative RSI
- ohlcv_status: NOT_DAILY, DEFER
- concept: Summing RSI(2) over 2 days measures sustained capitulation rather than a single extreme print, filtering out one-day spikes.
- universe: liquid stocks (price > $5, 21-day avg volume > 500k, liquid_top_N) and, as a SEPARATE study, liquid ETFs; never pooled
- direction: long
- indicators: Cumulative RSI = RSI(2)(t) + RSI(2)(t-1); SMA(200)
- entry: Close > SMA200 AND cumulative RSI(2,2) = RSI(2)(t) + RSI(2)(t-1) < 35; buy at close of t (fill: moc_proxy primary (signal from the 15:50 bar, fill at the official close); next_open alternate; close fill reported only as an upper bound)
- exit: RSI(2) > 65; sell at close of that session (Connors' original, primary)
- hold: 1-6 sessions
- frequency: weekly across a stock universe
- provenance: adapted
- status: active
- rust_status: IMPLEMENTED
- priority: P2
- sources: [DYR - Cumulative RSI strategy](https://tradingwithdyr.com/cumulative-rsi-strategy/) (page could not be retrieved in review; verify); Connors/Alvarez, Short Term Trading Strategies That Work, chapter on cumulative RSI (rule as transcribed: cumulative RSI(2,2) below 35 above SMA200, exit RSI(2) > 65); Quantitativo replication (blog claim, unverified)
- notes: Primary now follows Connors' original (thresholds 35 / 65), not the earlier threshold-20 selection with an up-close next-open exit. Registered alternates: threshold 20 with exit close > SMA5; threshold 10 with exit at next open (Quantitativo, 3 positions, large caps, ~26%/yr blog claim). Trading Garden S&P 500 (cumulative RSI): 1963-today CAGR 8.3%; 2000-today 10.6% (199 trades). Single-stock runs are earnings-window contaminated: report with and without signals within +/-3 sessions of an earnings release (G-12).
- parameter_variants: threshold/exit {cum RSI(2,2) < 35 & RSI(2) > 65 primary; cum RSI < 20 & close > SMA5 or first up-close; cum RSI < 10 & exit at next open (Quantitativo)}; tested thresholds 2/5/10/15/20/35; fill {moc_proxy, next_open}
- data_needs: stocks: daily_10years (D1) + earnings flag (DP-EARN); ETFs: DP-ETF total-return
- review: RESOLVED

### S005 | Connors %b (Bollinger Band)
- family: ETF index mean reversion
- ohlcv_status: NOT_DAILY, DEFER
- concept: %b < 0.2 for three consecutive days is a persistent stretch below the lower Bollinger Band. With BB(20,2) this is a genuinely rare state; with the 5-day band Quantified Strategies uses, it is close to 'three down days' (a 5-bar band is dominated by the last three closes). They are different strategies sharing a name, so both are registered.
- universe: primary: Connors' ETF list (e.g., DIA, EEM, EFA, EWH, EWJ, EWT, EWZ, FXI, GLD, ILF, IWM, IYR, QQQ, SPY, XHB, XLB, XLE, XLF, XLI, XLV; use those with data); secondary: SPY/QQQ single-position version
- direction: long
- indicators: Bollinger %b, BB(20,2) with POPULATION std (ddof=0) primary; SMA(200)
- entry: Close > SMA200 AND %b < 0.2 for 3 consecutive sessions; buy at close of the third session (fill: moc_proxy primary (signal from the 15:50 bar, fill at the official close); next_open alternate; close fill reported only as an upper bound)
- exit: %b > 0.8; sell at close of that session
- hold: 2-8 sessions
- frequency: several trades per month per ETF
- provenance: adapted
- status: active
- priority: P3
- sources: [Quantified Strategies — Connors %b strategy](https://www.quantifiedstrategies.com/larry-connors-b-strategy-bollinger-band/)
- notes: Connors did not publish the lookback. Primary BB(20,2) matches the book as transcribed (a StockFetcher transcription quotes Connors' ETF results: 1,014 trades, 76.5% win, +0.70%/trade, 4.2-day hold; the same poster reports every trade since 4/16/2010 negative in a later test). QS variant: 5-day lookback, 2 sd. Be The Puck %b beats buy-and-hold Sharpe on only 19% of 371 stocks (invested ~18% of days, worst on big tech). Run the single-position SPY/QQQ version and the multi-ETF portfolio as two separate studies.
- parameter_variants: band {BB(20,2) primary, BB(5,2) (Quantified Strategies)}; universe {Connors ETF list, SPY/QQQ only}
- data_needs: ETF total-return daily series (DP-ETF)
- review: RESOLVED

### S006 | Double 7s
- family: ETF index mean reversion
- ohlcv_status: NOT_DAILY, DELTA
- concept: A 7-day closing low in a bull regime is a statistically favorable dip: no stop loss, exit at the opposite 7-day extreme, letting mean reversion complete.
- universe: SPY or broad ETFs, pooled across ETFs (SPY alone gives ~5 trades/yr, ~50 in a 10-year window)
- direction: long
- indicators: 7-day rolling low of closes; 7-day rolling high of closes; SMA(200)
- entry: Close(t) > SMA200 AND close(t) <= the lowest close of the last 7 sessions INCLUDING session t (inclusive window, ties count as a low); buy at close of t (fill: moc_proxy primary (signal from the 15:50 bar, fill at the official close); next_open alternate; close fill reported only as an upper bound)
- exit: Close >= the highest close of the last 7 sessions including session t; sell at close of that session. No stop in the primary; disaster-stop variant: close < entry x (1 - 3 x ATR%).
- hold: 2-10 sessions
- frequency: ~1-2 trades per month on SPY (154 trades since 1993 per Quantified Strategies)
- provenance: source-specified
- status: active
- priority: P2
- sources: [Quantified Strategies — Double Seven strategy](https://www.quantifiedstrategies.com/larry-connors-double-seven-strategy-does-it-still-work/)
- notes: Best-evidenced Connors rule but regime-dependent: Trading Garden S&P 500: 252 trades since 2000, 76% win, CAGR 11.9% vs 8.3% buy-and-hold, but 1963-today 7.5% with a 33-year longest drawdown; Be The Puck beat buy-and-hold Sharpe on 86/371. Quantified Strategies SPY since 1993: 154 trades; lookback robust from 2 to 15 days. Report the full lookback surface (not the best cell). Window definition must be inclusive with <= / >= ties because libraries differ.
- parameter_variants: lookback {7 primary, 5, 10} (report the whole surface); disaster stop {off primary, close < entry x (1 - 3 x ATR%)}; fill {moc_proxy, next_open}
- data_needs: ETF total-return daily series (DP-ETF)
- review: KEEP (cluster champion candidate) | source OK (rule); QS page unfetchable
- implementation_results: Rust strategy registry in `src/study.rs`, signal logic in `src/signals.rs`, shared execution engine, and `configs/s006_double_7s.yaml`. Pooled 15-ETF TR book, 5 bps/side, next_open primary; dev < 2024-01-01 <= test. `lb10` was selected on dev among six registered variants; skill remains negative net of costs.

  | variant | dev total (log) | dev skill | dev Sharpe | dev max DD | test total (log) | test skill | test Sharpe | test max DD | trades (dev+test) |
  |---|---|---|---|---|---|---|---|---|---|
  | baseline (next_open) | +0.135 | -0.075 | 0.278 | -18.0% | +0.183 | -0.057 | 1.070 | -8.8% | 940+483 |
  | lb10 (next_open) | +0.145 | -0.024 | 0.231 | -33.5% | +0.175 | -0.032 | 1.042 | -7.5% | 649+321 |

  Test PF exceeds 2 on the primary next-open fill for `lb10` (2.09; 95% bootstrap CI 1.54-2.92), `spy_only` (2.89), and `lb5_spy` (2.86); all require a leakage sanity-check. Both single-symbol variants have insufficient dev trades (`spy_only` 62; `lb5_spy` 94). The close upper-bound PF also exceeds 2 for these variants. Signals use only the inclusive close window and SMA through decision-day close, with next-open execution; no lookahead was identified. No >=10% cumulative outperformance versus the configured SPY benchmark was detected. The configured start date is 2005-01-01, but bars begin 2016-01-04, so the expected start-date gap warnings are emitted. The catalog's disaster-stop rule uses entry-relative ATR%; the Python engine instead uses an absolute ATR offset, so Rust follows the reviewed catalog rule.

  Reproduce: baseline `cargo run --release -- --strategy s006_double_7s`; baseline plus all variants `cargo run --release -- --strategy s006_double_7s --variant lb5,lb10,spy_only,stop3atr,lb5_spy,lb10_stop`.
- python_reference_results: Existing Python run reports baseline 940+483 and `lb10` 649+321 next_open trades with matching rounded totals/skills; Python commands remain `python run.py --strategy s006_double_7s` and `python run.py --strategy s006_double_7s --variant lb5,lb10,spy_only,stop3atr,lb5_spy,lb10_stop`.

### S007 | 3-Day Pullback
- family: ETF index mean reversion
- ohlcv_status: NOT_DAILY, DEFER
- concept: Three consecutive lower closes inside an uptrend exhaust short-term sellers; exit when the 5-day mean is recovered.
- universe: SPY and liquid ETFs
- direction: long
- indicators: 3-day close streak; SMA(200); SMA(5)
- entry: Close > SMA200 AND three consecutive sessions with close < prior close; buy at close of the third session (fill: moc_proxy primary (signal from the 15:50 bar, fill at the official close); next_open alternate; close fill reported only as an upper bound)
- exit: Close > 5-day SMA; sell at close of that session
- hold: 2-6 sessions
- frequency: several trades per month
- provenance: adapted
- status: active
- priority: P3
- sources: Connors/Alvarez, Short Term Trading Strategies That Work (multiple-days-down chapter); [CXO Advisory - book review (principles only, no rule or numbers)](https://www.cxoadvisory.com/technical-trading/a-few-notes-on-short-term-trading-strategies-that-work/)
- notes: The CXO page is a book review: Connors' general principles (buy pullbacks, above the 200-day, buy fear, no stops, hold overnight) and a warning that the analyses are 'hypothetical backtests without transaction costs or data mining corrections' (1995-2007). It contains NO 'three lower closes, exit above 5-day SMA' rule and no performance numbers. The rule is plausible but unsourced here. Same trigger family as S001/S009/S030: run it as the N=3 member of the down-streak family (N = 2..7) together with S030.
- parameter_variants: streak length N {3 primary; 2..7 as the family grid shared with S009 and S030}
- data_needs: ETF total-return daily series (DP-ETF)
- review: RESOLVED

### S008 | ConnorsRSI(3,2,100) Stock Pullback
- family: Connors oversold - stock pullback
- ohlcv_status: NOT_DAILY, DEFER
- concept: Buys strong trending stocks (ADX > 30) after a sharp one-day capitulation (ConnorsRSI < 10 with close in bottom 25% of day range), selling on recovery to ConnorsRSI > 70.
- universe: liquid US stocks (guidebook: price > $5, volume > 250k, 200-day history)
- direction: long
- indicators: ConnorsRSI(3,2,100) = (RSI(close,3) + RSI(streak,2) + PercentRank(ROC(1),100))/3 with Wilder smoothing, streak = signed count of consecutive up/down closes, PercentRank over the prior 100 one-day returns; ADX(10); gap-from-close and close-location filters (this entry exceeds the old 4-indicator cap; the cap is dropped, see shared spec)
- entry: ADX(10) > 30 AND low(t) at least 4% below prior close AND close in bottom 25% of day range AND ConnorsRSI < 10; buy at close of t (fill: moc_proxy primary (signal from the 15:50 bar, fill at the official close); next_open alternate; close fill reported only as an upper bound). Alternate: guidebook limit order at next session, 2-10% below signal close, filled only if low(t+1) <= limit (report the fill rate)
- exit: ConnorsRSI > 70; sell at close of that session
- hold: 1-5 sessions
- frequency: daily opportunities across a 500-stock universe
- provenance: source-specified
- status: active
- priority: P2
- sources: [Be The Puck — ConnorsRSI pullback](https://bethepuck.com/strategies/connorsrsi-pullback/)
- notes: Be The Puck matches (ADX(10) > 30, low >= 4% below prior close, close in bottom 25% of the range, CRSI < 10, exit CRSI > 70; $5 / 250k-volume filters; no 200-day filter) but its scorecard is the weakest of the Connors rules: calls right 59% over 8,172 calls, beat buy-and-hold Sharpe on only 42/371. The guidebook's primary fill is the next-day limit 2-10% below the close; a limit fill creates adverse selection (you fill precisely on continuation days), so backtest both with fill-probability reported. A 4% intraday drop with a bottom-quartile close is largely earnings/news: report with and without earnings-window signals (+/-3 sessions, G-12).
- parameter_variants: fill {signal-close (moc_proxy) primary, next_open, guidebook limit 2% / 5% / 10% below close}; earnings-window {included, excluded}
- data_needs: stocks: daily_10years (D1) + earnings flag (DP-EARN)
- review: RESOLVED

### S009 | 3 Days Down Overnight (SPY)
- family: overnight (Connors streak)
- ohlcv_status: NOT_DAILY, DEFER
- concept: The overnight session (close-to-open) captures most of the mean reversion after multi-day selloffs; holding only overnight avoids giving back the bounce intraday.
- universe: SPY
- direction: long
- indicators: 3-day down-close streak
- entry: SPY closes lower for 3 consecutive sessions; buy at close of the third session (moc_proxy; next_open not applicable to the overnight version)
- exit: Sell at next day's open (overnight capture, primary); next day's close (full-day version, alternate)
- hold: 1 session (overnight)
- frequency: ~4-8 trades per year on SPY (pool QQQ, IWM, DIA and sector ETFs to raise the count)
- provenance: source-specified
- status: active
- priority: P3
- sources: [Quantified Strategies — 3 Days Down overnight strategy](https://quantifiedstrategies.substack.com/p/3-days-down-overnight-trading-strategy-942)
- notes: The Quantified Strategies Substack post is paywalled: only the rule sentence is visible (three consecutive down days, SPY since 1993, Amibroker). The earlier '212 trades' figure belonged to the Turnaround Tuesday statistic (S010) and is removed; trade count and performance for THIS rule are unverified. The relevant null is the unconditional close-to-open return of SPY (see S021), which is itself positive: report conditional minus unconditional overnight difference using the dividend-correct overnight series (raw open/close plus dividends at the ex-date), with a placebo of 2- and 4-day streaks.
- parameter_variants: streak length {3 primary; placebo 2 and 4}; exit {next open primary, next close}; universe {SPY, then pooled QQQ/IWM/DIA/sector ETFs}
- data_needs: ETF raw prices + dividends (DP-ETF raw+div)
- review: RESOLVED

### S010 | Turnaround Tuesday
- family: calendar (weekday reversal)
- concept: A weak Monday tends to reverse on Tuesday. The relevant nulls are the unconditional Tuesday and unconditional post-down-day drift; calendar effects are the weakest evidence class (Atlas 9.5) and Be The Puck notes the Monday effect 'vanished after 1975'.
- universe: SPY
- direction: long
- indicators: Monday close-to-close return; weekday filter; optional VIX day-over-day change
- entry: On Monday, if close(Mon) < close(Fri) (primary, Monday close-to-close weakness), buy at Monday close
- exit: Sell at Tuesday open (overnight, primary); Tuesday close (alternate)
- hold: 1 session (overnight Monday-to-Tuesday)
- frequency: ~10-15 signals per year
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [Papers With Backtest - Turnaround Tuesday (generic article; its two code snippets define 'Monday drop' inconsistently and one computes win ratio by conditioning on the outcome, i.e. look-ahead)](https://paperswithbacktest.com/course/turnaround-tuesday)
- notes: Earlier note corrected: the 56% win rate, 212 trades and 0.3% avg gain are Quantified Strategies figures (unverified) and do not appear on the cited page. The old rule 'open(t) < close(t-1)' was a Monday GAP-down, a different event from a weak Monday; both are now explicit variants. Run the identical rule for every weekday as a placebo.
- parameter_variants: Monday weakness {close(Mon) < close(Fri) primary; return <= -0.5% (ex-S085: Monday close-to-close decline > 0.5%); gap-down open(Mon) < close(Fri)}; VIX filter {off primary; VIX(t) > VIX(t-1) (ex-S085)}; exit {Tuesday open, Tuesday close}; placebo weekdays {Mon..Fri}
- merged_from: S085
- data_needs: ETF raw prices (DP-ETF); VIX for the S085 filter (DP-VIX)
- review: RESOLVED

### S011 | Z-Score / Bollinger Lower-Band Mean Reversion
- family: ETF index mean reversion
- ohlcv_status: NOT_DAILY, DEFER
- concept: A close more than 2 standard deviations below its 20-day mean (z < -2, algebraically identical to a close below the lower Bollinger Band(20,2) up to ddof) is a >95th-percentile stretch; the 20-day mean is the magnet. S011 (z-score with SMA200 gate) and S012 (lower-band close with RSI(2) filter, no gate) are cells of one family.
- universe: SPY, QQQ, IWM
- direction: long
- indicators: z = (close - SMA20)/std20 of closes (population std, ddof=0) or equivalently Bollinger Bands(20,2); SMA(200) gate (optional); RSI(2) filter (optional)
- entry: z(t) < -2 (close below BB(20,2) lower band) AND Close > SMA200; buy at close of t (fill: moc_proxy primary (signal from the 15:50 bar, fill at the official close); next_open alternate; close fill reported only as an upper bound)
- exit: z(t) > 0 (close above the 20-day middle band); sell at close of that session
- hold: 2-8 sessions
- frequency: 1-3 trades per month
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [Quantified Strategies - 200 free backtested strategies (index page, no rule)](https://www.quantifiedstrategies.com/trading-strategies-free/); Be The Puck Bollinger Reversion scorecard (cross-check)
- notes: Be The Puck's Bollinger Reversion (close below the lower band, exit at the 20-day mean) beat buy-and-hold Sharpe on 56/372 and returned 5.7%/yr vs 8.9% buy-and-hold on the S&P 500 index: weak as a standalone. The ex-S012 form has no SMA200 gate, which makes it a bear-market catcher; the gate is therefore a registered switch. State ddof=0 (pandas default std is ddof=1). Report the four cells as ONE cluster result.
- parameter_variants: SMA200 gate {on primary, off (ex-S012)} x RSI(2) < 10 filter {off primary, on (ex-S012)} (four trials); universe {SPY, QQQ, IWM}; ddof {0 primary, 1}
- merged_from: S012
- data_needs: ETF total-return daily series (DP-ETF)
- review: RESOLVED

### S013 | TLT Mean Reversion (RSI(2))
- family: ETF index mean reversion
- concept: Long Treasuries are among the most mean-reverting daily assets (safe-haven flows reverse); Connors-style RSI(2) rules transfer to TLT.
- universe: TLT
- direction: long
- indicators: RSI(2); SMA(200); SMA(5)
- entry: Close > SMA200 AND RSI(2) < 10; buy at close of t
- exit: Close > 5-day SMA; sell at close of that session
- hold: 1-5 sessions
- frequency: several trades per month
- provenance: adapted
- status: active
- priority: P3
- sources: [StockCharts ChartSchool — RSI(2)](https://chartschool.stockcharts.com/table-of-contents/trading-strategies-and-models/trading-strategies/rsi-2)
- notes: Connors' rules applied to TLT (application is the catalog's). IndicatorEdge's TLT backtests found few daily setups beating buy-and-hold: the edge must be confirmed, not assumed. TLT/SHY must be total-return series (distributions are most of the return) and results must be split by rate regime (2016-19, 2020, 2022 when TLT sat below its SMA200 most of the year so the filter removes trades).
- data_needs: TLT, SHY total-return daily series (DP-ETF)
- review: NO FINDINGS (strategy-specific) | source PARTIAL

### S014 | Sector ETF Basket Mean Reversion
- family: cross-sectional ETF reversal
- ohlcv_status: NOT_DAILY, DEFER
- concept: Buying the 3 weakest sector ETFs of the past 5 sessions (while above their 200-day SMA) diversifies single-ETF timing risk and harvests cross-sector reversal.
- universe: SPDR sector ETFs defined by a PIT inception-date table (9 sectors until XLRE Oct-2015 and XLC Jun-2018; XLK/XLY/XLC reconstituted Sep-2018, which breaks price continuity - note the break); the same table is used by S042
- direction: long
- indicators: 5-day return minus beta x SPY 5-day return (60-day beta through t-1); raw 5-day return (variant); SMA(200); SMA(5) or SMA(10)
- entry: Each session, rank sector ETFs by 5-day residual return vs SPY (primary); buy the 3 worst that have close > SMA200, at the close of t (fill: moc_proxy primary (signal from the 15:50 bar, fill at the official close); next_open alternate; close fill reported only as an upper bound)
- exit: Per position, close > 5-day SMA (primary, the family convention); sell at close of that session
- hold: 2-8 sessions
- frequency: continuous, 1-3 new entries per week
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [CXO Advisory — Notes on Short Term Trading Strategies That Work](https://www.cxoadvisory.com/technical-trading/a-few-notes-on-short-term-trading-strategies-that-work/)
- notes: Sector ETFs are ~0.8 correlated, so a raw '5-day worst 3 of 9' mostly ranks beta to the day's market move; ranking on the residual vs SPY ties to S075. The CXO page has no basket rule. The earlier 10-day SMA exit differed from the family without justification and is now a registered alternate.
- parameter_variants: rank {residual vs SPY primary, raw 5-day return}; exit {SMA5 primary, SMA10}
- data_needs: sector ETF total-return daily series (DP-ETF) + PIT sector universe table
- review: RESOLVED

---

# Family 2 — Overnight and Gap Strategies

*Why this family works: the overnight session is structurally different from the intraday session. Overnight returns carry the bulk of equity risk premium in index ETFs, yet overnight volatility is lower — a favorable risk/return asymmetry documented in the "Night Trading" SSRN paper (its overnight-bias strategy earned twice the market's return with one-third the beta). Gaps also create forced flows: panic sellers at the open, stop cascades, and overnight-news overreaction that reverts or continues depending on volume context.*

### S015 | Chan Buy-on-Gap
- family: overnight and gap
- ohlcv_status: NOT_DAILY, DEFER
- concept: Panic gap-down opens in stocks still in uptrends (open above 20-day MA) are overreactions; buying the open and exiting at the same day's close harvests intraday recovery. This is Chan's documented strategy from Quantitative Trading.
- universe: liquid_top_N US stocks (N=1500, SP1500-like, PIT price-only rule)
- direction: long
- indicators: 90-day POPULATION std of close-to-close returns; 20-day SMA
- entry: Signal from the open: select stocks where return from prior day's low to today's open < -1 std (std on prior 90 days of close-to-close returns, ddof=0); keep only those with open > 20-day SMA of closes; buy the 10 deepest qualifying drops. Fill: the close of the 09:31 bar (first-minute bar after the signal) with 15 bps RTH slippage (a signal computed from open(t) cannot be filled at open(t))
- exit: Sell all positions at the SAME session's close (Chan's published form, primary); next session's open is the registered alternate
- hold: intraday (same-day close, primary); 1 overnight (alternate)
- frequency: ~0.5 qualifying positions per day on average (the reproduction took 695 positions in 1,500 days, held at least one name on 391 days and the full ten on only ten days)
- provenance: adapted
- status: active
- priority: P2
- sources: [Chan Buy-on-Gap reproduction (l3a0 GitHub)](https://github.com/l3a0/quantitative-trading/blob/main/blog/buy-on-gap-lessons.md)
- notes: l3a0 reproduces Chan's printed APR 8.7% / Sharpe 1.5 on his own survivor-biased file (497 stocks, 2006-05 to 2012-04, no costs; with the other std helper 8.4%/1.7). Chan traded a version that 'suffered from diminishing returns from 2009 onward' and a trader cannot know the official open in time (pre-open quotes pick different stocks). Our 2016+ sample is entirely post-decay: run as a pre-registered falsification test with an expected skill of about zero.
- parameter_variants: gap rule {Chan std-based primary; fixed_pct (ex-S022): open(t)/close(t-1) - 1 < -1% / -2% / -3% AND close(t-1) > SMA200 (S022 used a fixed -2%, S&P 500 stocks, which fires in bulk in March 2020 and 2022)}; exit {same-day close primary, next open}; std {population primary, sample}
- merged_from: S022
- data_needs: daily_10years (D1) + 09:31 minute bars for candidate days
- review: RESOLVED

### S016 | Short-on-Gap-Up (Chan mirror)
- family: overnight and gap
- ohlcv_status: NOT_DAILY, DEFER
- concept: The mirror of Chan's buy-on-gap: stocks that gap up more than 1 std against a downtrend revert intraday.
- universe: SP1500 stocks
- direction: short
- indicators: 90-day std of close-to-close returns; 20-day SMA
- entry: Signal from the open: select stocks where return from prior day's high to today's open > +1 std (ddof=0); keep only those with open < 20-day SMA; short the 10 largest qualifying gap-ups. Fill at the close of the 09:31 bar with 15 bps slippage
- exit: Cover at the SAME session's close (Chan's published form, primary); next session's open alternate
- hold: intraday (primary); 1 overnight (alternate)
- frequency: daily across SP1500
- provenance: adapted
- status: active
- priority: P3
- sources: [Chan Buy-on-Gap reproduction (l3a0 GitHub)](https://github.com/l3a0/quantitative-trading/blob/main/blog/buy-on-gap-lessons.md)
- notes: CORRECTED: the l3a0 reproduction found the declared mirror rule PROFITABLE pre-cost (APR 12.2%, Sharpe 1.79, 725 shorts, survivor-biased, no costs) versus Chan's published 46% / 1.27; what failed was the match to Chan's published figure (a volatility-floor argument shows whatever Chan ran moved >= 4.5x as much as the declared rule). The steeper drawdown claim does reproduce (-6.5% vs -5.2%). Same fill issues, borrow/recall risk and survivorship as S015. Test as the hedge leg of S015 (market-neutral basket) under borrow scenarios (0.3% GC, 3%, 10% annualised); run long-only first for the rest of the catalog.
- data_needs: daily_10years (D1) + 09:31 minute bars + borrow assumptions
- review: RESOLVED

### S017 | Night Trading Overnight-Bias Stock Selection
- family: overnight and gap
- concept: Roughly one-fifth of stocks have persistent, statistically significant positive overnight-return biases; holding only their overnight sessions earned twice the market's return with one-third the beta (SSRN). These stocks tend to have negative intraday returns — the premium is paid for overnight risk-bearing.
- universe: liquid_top_N US stocks with 1+ year of history (liquidity floor, price >= $5)
- direction: long
- indicators: trailing 252-day mean overnight return (close-to-open, from raw prices plus dividends added back at the ex-date) and its t-statistic; unconditional overnight drift of the same stocks as null
- entry: Formation: each year (primary) compute each stock's trailing-252-day mean overnight return and t-stat; select stocks with t > 2 (Benjamini-Hochberg FDR-controlled) AND in the top quintile of mean overnight return. Evaluation: hold the selected set through the NEXT calendar year (formation year selects, next year evaluates; do not roll the screen daily). Each day buy at the close (auction/MOC fill)
- exit: Sell at next open (MOO fill); repeat daily (pure overnight capture)
- hold: 1 session (overnight only)
- frequency: daily
- provenance: source-specified
- status: active
- rust_status: IMPLEMENTED
- priority: P1
- sources: [SSRN — Night Trading: Lower Risk But Higher Returns?](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=2633476)
- notes: Lachance (SSRN 2633476; single author, also Rev. Financial Economics): one fifth of stocks have significant positive overnight biases; investing overnight in them the next year yields twice the market return for a third of the beta, and they average negative returns intraday. The t > 2 screen is the catalog's operationalisation (about 2.3% pass by chance one-sided vs 20% flagged, hence FDR). A daily round trip costs 2 x (half-spread + slippage): at 10-15 bps per side that is 25-38%/yr, comparable to the claimed premium: use auction fills and report the break-even cost. Opening-print bid-ask bounce biases measured overnight returns upward (use first-minute VWAP/mid for exits where minute data exist, plus a liquidity floor). Add Lou-Polk-Skouras (2019 JFE, 'A tug of war', verify citation) persistence as a robustness check.
- parameter_variants: formation cadence {annual primary; quarterly; monthly (ex-S034)}; holding convention {overnight-only primary; full-day; long overnight / short intraday of the same stocks}; extra filter {none primary; negative trailing intraday return (ex-S034, kept only if it improves a hold-out)}; hold {daily overnight primary; one week dropped}
- merged_from: S034
- implementation_results: Rust implementation in `src/signals.rs` / `src/study.rs`; 30 named variants. Supports annual, quarterly, and monthly formation; close-to-open, full-day, and overnight-minus-intraday holdings; liquidity/price/history filters; BH-FDR selection; and dividend/split corporate-action adjustments. Synthetic tests cover monthly selection and dividend adjustment. Full daily-data baseline (1,223 symbols, 2016-01-04 to 2026-10-23; close-to-open, 15 bps/side):

  | variant | dev total (log) | dev skill | dev Sharpe | dev max DD | trades dev | test total (log) | test skill | test Sharpe | test max DD | trades test |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | -3.4005 | -3.9467 | -2.7182 | -96.71% | 29369 | -1.2426 | -1.6440 | -2.4462 | -70.94% | 16240 |

  No variant winner has been selected; only the baseline has been run. The baseline is not profitable after costs. The config excludes the reference ETF-like ticker list; any other non-benchmark input symbols are treated as stocks, so use a common-stock-only universe.
  Reproduce baseline: `cargo run --release -- --strategy s017_night_trading --stages partition`. Run the S017 variant matrix with the strategy-specific command in `run_strats.bat`: `target\release\deepthrust.exe --strategy s017_night_trading --variant quarterly,monthly_s034,full_day,overnight_intraday,negative_intraday,top10,top30,top40,fdr01,fdr10,t15,t25,t30,window126,window189,window315,quarterly_top10,quarterly_top30,monthly_top10,monthly_top30,fdr01_top10,fdr10_top30,t25_fdr01,t15_fdr10,full_day_top10,full_day_top30,spread_top10,spread_top30,negative_top10,negative_quarterly`.
- data_needs: daily_10years raw OHLC (D1) + dividend calendar (DP-CAL) for dividend-correct overnight returns
- review: RESOLVED

### S018 | Gap Fade (Fill the Gap)
- family: overnight and gap
- ohlcv_status: NOT_DAILY, DEFER
- concept: Common gaps (not driven by material news) tend to fill — price returns to the prior close — as overnight overreaction unwinds.
- universe: liquid stocks and index ETFs
- direction: long (down-gaps) / short (up-gaps)
- indicators: gap size vs 20-day average true range; volume filter (gap on below-average volume = common gap)
- entry: Step 1 (descriptive study, before any simulation): measure P(fill by close | gap bucket) and P(fill within 3 sessions) by gap size x trend x catalyst flag. Step 2 (rule): 0.5 x ATR(20) <= |gap| < 1.0 x ATR(20) (band disjoint from S019's > 1 ATR) AND no earnings/FOMC/dividend catalyst in the calendar AND prior-5-session average volume below its 20-day average (quiet-tape proxy, a property of the prior tape not of the gap); buy down-gaps / short up-gaps; fill at the close of the 09:31 bar
- exit: Same-day close (primary); next open and the gap-fill target (prior close) are registered alternates
- hold: intraday (primary); 1 overnight (alternate)
- frequency: daily opportunities across liquid universe
- provenance: adapted
- status: active
- priority: P2
- sources: [Capital.com — Gap trading strategies](https://capital.com/en-int/learn/trading-strategies/gap-trading)
- notes: Capital.com is a qualitative education page (common/breakaway/runaway/exhaustion gaps), no numbers. The gap-fill thesis is an intraday/few-day statement, hence the same-day-close primary. Gap-day volume is unknown at the open, so 'common vs breakaway' cannot be resolved; the quiet-tape proxy is the catalog's. Catalysts: earnings from SimFin Publish Date (2009+) / EDGAR 8-K 2.02 (2023-09+); FOMC and dividend calendars must be built. Report expected signal frequency per 500 stocks before sizing the trial. S018 and S019 partition gap-size/volume space so the two never trade the same name in opposite directions.
- parameter_variants: exit {same-day close primary, next open, gap-fill target}; gap band {0.5-1.0 ATR primary}; catalyst filter {on}
- data_needs: daily_10years (D1) + 09:31 minute bars + earnings/FOMC/dividend calendars (DP-EARN, DP-CAL)
- review: RESOLVED

### S019 | Gap-and-Go Continuation
- family: overnight and gap
- concept: Breakaway gaps with high volume tend to continue in the gap direction — institutional repositioning does not complete in one session.
- universe: liquid stocks (primary); index ETFs SPY, QQQ, IWM, DIA and sector ETFs (ex-S025 index version, same code, different universe)
- direction: long (up-gaps) / short (down-gaps)
- indicators: gap size vs ATR(20); relative volume (gap-session volume vs 20-day median); R defined below
- entry: Open(t) > close(t-1) by more than 1 x ATR(20) AND volume by end of session > 2 x 20-day median; buy at close of t (confirmed breakaway). Mirror short for down-gaps (long-only first)
- exit: Stop = prior close (primary; alternate: gap-day low), so R = entry - stop; target 2R or trail at 2-bar low; max hold 5 sessions
- hold: 1-5 sessions
- frequency: weekly across a liquid universe
- provenance: adapted
- status: active
- priority: P2
- sources: [Capital.com — Gap trading strategies](https://capital.com/en-int/learn/trading-strategies/gap-trading)
- notes: Capital.com's gap-and-go: high-volume gap, enter in gap direction, stop just inside the gap area, 1:2 or 1:3 reward-to-risk on intraday charts; adapted here to daily bars with close confirmation. With a close entry after a >1 ATR gap the stop 'just inside the gap' is the prior close, i.e. >1 ATR away, and a 1:2 target is >2 ATR: hence explicit R. The Atlas (10.4, Mittal & Choudhary 2026) argues quiet-looking moves travel further than visually impressive breakouts, the opposite of the 2x-volume rule: both volume regimes are registered. Index version (ex-S025): open(t) > high(t-1) AND session volume > 1.5 x 20-day median, buy at close, exit close of t+2 or trail 2-bar low, max 3 sessions; SPY alone gives only ~15-25 events a decade, so pool the ETF set; test both signs (long continuation / short fade) as registered variants only if costs allow.
- parameter_variants: volume regime {> 2x median primary, < 1x median}; stop {prior close primary, gap-day low}; universe {stocks, index/sector ETFs (ex-S025: open > prior high, volume > 1.5x median, exit close t+2 / 2-bar low, max 3 sessions)}; direction on indices {long continuation primary; short fade}
- merged_from: S025
- data_needs: daily_10years (D1); ETF daily bars (DP-ETF)
- review: RESOLVED
- implementation_results: `src/signals.rs` / `src/study.rs`; 30 large-cap stocks, 10 bps/side, close-fill proxy, 2016-2026 daily bars. Source is qualitative; this is the catalog's pre-registered quantitative adaptation. `gap125_vol25` was dev-selected among variants meeting trade-count gates; dev skill remains negative.

  | variant | dev total / skill / Sharpe / PF / max DD | test total / skill / Sharpe / PF / max DD | trades (dev+test) |
  |---|---|---|---|
  | baseline | +0.037 / -0.292 / 0.15 / 1.07 / -9.6% | +0.146 / -0.064 / 1.19 / 1.62 / -3.8% | 385+149 |
  | gap125_vol25 | +0.099 / -0.077 / 0.47 / 1.31 / -5.3% | +0.105 / -0.026 / 1.09 / 1.76 / -3.3% | 232+91 |
  `gap150` had test PF 2.06 but Sharpe 1.45 and skill -0.011; it was not dev-selected. Signal inputs use only prior-session ATR/volume plus the current open gap and completed-session volume, but the same-close fill is an optimistic proxy after volume confirmation.
  Reproduce: baseline `cargo run --release -- --strategy s019_gap_and_go`; dev-selected variant `cargo run --release -- --strategy s019_gap_and_go --variant gap125_vol25`

### S020 | Earnings-Eve Estimate-Drift Mean Reversion (Chan/Estimize)
- family: overnight and gap
- concept: Chan's documented price-only benchmark: stocks whose return over the full estimate period exceeds the return over the recent (latest-20%) estimate period — delta = R − Rs > 0 — are the ones to BUY into the announcement; the overnight session after the announcement captures the drift.
- universe: US stocks with quarterly earnings the next session
- direction: long and short
- indicators: delta = R (return over average estimate-reporting period T) − Rs (return over the latest 20% of the estimate period); earnings calendar
- entry: On the close before an earnings announcement: buy stocks with delta > 0 (full-period return exceeds recent-period return); short stocks with delta < 0
- exit: Exit at the open after the announcement (overnight capture of the earnings reaction)
- hold: 1 session (overnight)
- frequency: daily during earnings seasons
- provenance: research-hypothesis
- status: dropped
- priority: P4
- sources: [Ernest Chan — Trading with Estimize and I/B/E/S earnings estimates](http://epchan.blogspot.com/2015/01/trading-with-estimize-and-ibes-earnings.html)
- notes: DOCUMENTED NULL, dropped from the active queue. In the source post (guest author Yang Gao, QTS Capital) the price-only delta rule is a BENCHMARK to show Estimize data carries information: 'This benchmark shows no significant positive return'. The strategies that earned returns were Deltix's Estimize mean-reversion (S&P 100, 2012-2013, Sharpe 0.8, 6%/yr after 5 bps) and an I/B/E/S momentum rule (Sharpe 1.5, 11%) that failed out of sample (2010-2011: -6.4%); the authors conclude 'we cannot have a statistically significant backtest'. The post also hedges net exposure with SPY, which this entry omits, and delta needs estimate-reporting dates a price-only run cannot define. If ever run: falsification test only (expect null) with an SPY hedge.
- data_needs: BLOCKED for the profitable variant (Estimize/IBES estimates not available); price-only version is a null control
- review: DROP (documented null) | source MISMATCH

### S021 | Index Overnight Premium (SPY close-to-open only)
- family: overnight and gap
- concept: Nearly all of SPY's long-term return accrues overnight; holding only the overnight session captures the premium with lower volatility than full-day holding.
- universe: SPY
- direction: long
- indicators: SMA(50) participation filter (optional, registered trial); dividend-correct overnight return = (raw open(t+1) + dividend if ex-date)/raw close(t) - 1
- entry: Buy SPY at every close (unconditional, primary); filtered variant: only when close(t) > SMA(50)
- exit: Sell at every open (cash intraday)
- hold: 1 session (overnight), repeated daily
- frequency: daily
- provenance: adapted
- status: active
- priority: P1
- sources: [SSRN - Night Trading: Lower Risk But Higher Returns? (stock-selection evidence; see S017)](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=2633476); Cooper, Cliff & Gulen, Return Differences between Trading and Non-trading Hours (SSRN 1004081): the US equity premium over their decade is solely due to overnight returns for stocks, indexes and index futures
- notes: Market-level evidence is Cooper-Cliff-Gulen, not Lachance (stock selection); the SMA(50) filter is the catalog's adaptation. At 5 bps/side the strategy loses by arithmetic (504 sides a year = 25%/yr): the result is purely a function of the cost assumption, so run a cost-sensitivity ladder (0.5 / 1 / 2 / 5 bps per side), report the break-even cost, and use auction fills. A split-only series understates SPY's overnight return by roughly its dividend yield (~1.3-1.5%/yr): use raw open/close plus dividends added back at the ex-date. Null = SPY close-to-open buy-and-hold. Report the per-weekday decomposition and subperiods (2016-19, 2020, 2021, 2022, 2023-26) for decay. Be The Puck notes the weekend effect 'vanished after 1975', so the weekend slice is a table in this study (test both signs), not a separate strategy.
- parameter_variants: filter {none primary, SMA(50) gate}; cost per side {0.5, 1, 2, 5 bps}; weekday table {Mon..Fri overnight returns, incl. ex-S084 Friday-close to Monday-open, both signs}; universe {SPY primary, QQQ, IWM}
- merged_from: S084
- data_needs: SPY raw OHLC + dividends (DP-ETF raw+div)
- review: RESOLVED
- implementation_results: Rust port in `src/study.rs` / `src/signals.rs`, using the configured total-return-adjusted ETF Parquet bars from 2016-01-04 through 2026-10-23 (2,718 sessions); 30 registered variants, close-to-open daily-bar proxies, 1 bp/side baseline. This differs from the catalog's preferred raw OHLC plus explicit ex-date dividends and auction fills. `iwm_cost025` (IWM, 0.25 bps/side) was selected on dev skill; both the baseline and selected variant have negative overnight-null skill on dev and test. The selected variant exceeds IWM total-return buy-and-hold by >10% cumulative growth on dev only; it does not exceed that margin on test.

  | variant | dev total / skill / Sharpe / max DD | test total / skill / Sharpe / max DD | trades (dev+test) |
  |---|---|---|---|
  | baseline | +0.17 / -0.40 / 0.24 / -33.0% | +0.24 / -0.14 / 0.93 / -16.9% | 2,011+706 |
  | iwm_cost025 | +0.99 / -0.10 / 0.94 / -31.4% | +0.37 / -0.04 / 1.01 / -19.4% | 2,011+706 |
  Reproduce: `run_strats.bat` runs the full matrix; `target\release\deepthrust.exe --strategy s021_index_overnight --variant iwm_cost025` reruns baseline and the dev-selected variant.

### S023 | Post-Earnings Overnight Drift
- family: overnight and gap
- concept: Earnings surprises that hit pre-market or after-hours create overnight order imbalances that continue into the next open (the overnight portion of PEAD).
- universe: stocks reporting earnings with pre-market/after-hours timing
- direction: long (beats) / short (misses)
- indicators: earnings date/surprise sign; overnight gap vs ATR(20)
- entry: Step 1 (descriptive study): open(t) to close(t+1) return by SUE bucket. Step 2 (rule): if earnings are announced outside regular hours (BMO/AMC classified from EDGAR 8-K 2.02 acceptance time, 2023-09+) AND SUE (point-in-time, from SimFin quarterly EPS) is in the top decile on a trailing cutoff AND the gap is < 3 x ATR(20) (not blow-off), buy at the open of the reaction session; misses: short as a separate leg
- exit: Exit at the close of t+1 (one full day plus overnight) or trail 2-bar low
- hold: 1-2 sessions
- frequency: daily in earnings season
- provenance: research-hypothesis
- status: active
- priority: P2
- sources: Earnings Whispers (announcement premium +1.08% for Score +5, NOT post-announcement drift, vendor data): https://www.earningswhispers.com/about-score; Chan's Estimize post is unrelated to PEAD. No direct source for the overnight continuation claim
- notes: The entry buys the open AFTER the surprise gap, so the overnight reaction is already realised; the claimed continuation from open(t) to close(t+1) needs its own evidence. The gap itself must not be used as the surprise. SUE = (EPS_q - EPS_{q-4}) / std of the last 8 such differences (seasonal random walk) at the Publish Date; consensus data is not in the repo. Shorts are a separate later experiment (G-09).
- data_needs: SimFin quarterly EPS + Publish Date; EDGAR 8-K 2.02 timestamps 2023-09+ (DP-EARN); daily_10years (D1)
- review: RESOLVED
- implementation_results: **IMPLEMENTED; backtest DATA-BLOCKED.** Rust implementation in `src/family2.rs` classifies SEC 8-K Item 2.02 acceptance timestamps as BMO/AMC in US Eastern time; computes point-in-time seasonal EPS differences using only previously published q-4 values; standardizes against the prior eight differences and applies a 90th-percentile cutoff over prior SUE observations in a 756-calendar-day window (minimum 20 observations). It implements the long top-decile rule and open-to-next-close exit; the short-miss leg remains deferred per G-09. Synthetic tests pass. No local SimFin quarterly EPS input was found, so no real-data backtest or performance claim is available.
  Reproduce after setting `s023_eps_file` in the config to a valid local SimFin quarterly EPS file: `cargo run -- --strategy s023_post_earnings_drift --config configs/s023_post_earnings_drift.yaml`

### S024 | Ex-Dividend Window Capture
- family: overnight and gap
- concept: The ex-date price drop is typically 80-100% of the dividend, so gross edge is expected near zero; the study measures the ex-day drop ratio and then P&L over several holding windows. No documented mechanism supports a systematic post-ex recovery.
- universe: liquid high-yield stocks and ETFs; S&P 500 stocks with yield > 2% (~100 names, ~4 ex-dates a year each) for the ex-S101 window
- direction: long
- indicators: ex-dividend calendar; trailing 365-day cash-dividend yield; ex-day drop ratio (diagnostic)
- entry: Buy at the close of the session before the ex-dividend date (primary)
- exit: Sell at the open of the ex-date (overnight over the ex-date, primary); alternates: close of the ex-date; close of ex-date +1 (ex-S101: buy close 1 session before ex, sell close 1 session after, 3 sessions)
- hold: 1-3 sessions
- frequency: weekly across a dividend calendar
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [Quantified Strategies - 200 free backtested strategies (index page, no rule)](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: P&L = ex-date return + dividend; with two spreads, costs and the 61-day qualified-dividend rule the net edge is expected negative for most names. Use UNADJUSTED prices with explicit dividend credits and report the ex-day drop ratio (P_cum_close - P_ex_open)/D by yield bucket before P&L. The local raw daily-bar corporate-action cache supplies dividend events back to 2016.
- parameter_variants: hold window {buy close before ex -> sell ex-date open (primary); -> ex-date close; -> ex-date+1 close (ex-S101)}; trailing-yield threshold {0% baseline; 30 registered thresholds from 0.25% through 20%, including 2%}
- merged_from: S101
- data_needs: raw prices + dividend/ex-date calendar 2016+ (DP-CAL); daily_10years (D1)
- review: RESOLVED
- implementation_results: **IMPLEMENTED — DELTA.** Rust implementation in `src/family2.rs`, using raw daily bars and the configured cash-dividend Parquet input. Point-in-time liquidity ranks restrict the eligible dividend universe to the top 500 names; this is not a historical S&P 500 universe and includes ETFs. The optional `yield_gt2` variant uses trailing 365-day cash dividends divided by the pre-ex-date close; the catalog's 20-day price-change indicator is not part of the implemented signal. The run covered 742 candidate symbols and 2,718 sessions (2016-01-04–2026-10-23), with a 2024-01-01 dev/test split and 5 bps/side. Event-level diagnostics include the ex-day price-drop ratio by trailing-yield bucket.

  | variant / exit | dev total / skill / Sharpe / max DD | test total / skill / Sharpe / max DD | trades (dev+test) |
  |---|---|---|---|
  | baseline / ex-date open | -0.207 / -1.020 / -0.358 / -25.6% | -0.052 / -0.366 / -0.340 / -10.8% | 12,139+5,162 |
  | yield > 9% / ex-date+1 close (dev-selected; evidence-qualified) | +0.183 / +0.175 / 0.268 / -10.6% | +0.025 / +0.006 / 0.289 / -2.1% | 310+116 |

  The all-events ex-day drop ratio (mean / median) was 0.708 / 0.693 below 2% yield, 0.941 / 0.848 at 2–4%, and 0.312 / 0.901 at 4%+ (23,795 events). The complete 30-variant yield-threshold grid was evaluated. The dev-selected row is selected only among candidates meeting the 100-dev / 30-test trade minimum; the 15% and 20% yield thresholds are `insufficient evidence`. Across 93 variant/fill trials, no locked-test DSR exceeded 0.95; per-variant DSR and regime coverage are in `results/s024_ex_dividend_capture/s024_deflated_sharpe.csv` and `s024_regime_coverage.csv`. No tested row met the joint PF > 1.5, Sharpe > 1.5, and skill > 1 threshold. The test PF > 2 outlier is yield > 20% / ex-date close (PF 3.65, 19 trades), below the minimum evidence gate; no Sharpe exceeded 2. The signal uses the ex-date cash amount to qualify the prior-close entry and assumes that amount was publicly known by then; the cached event record has no announcement timestamp to independently verify that timing. Five-bps-per-side costs are not spread/ADV-aware for stocks; the daily-bar fills are proxies, and taxes and the qualified-dividend holding rule are not modeled.
  Reproduce: `cargo run -- --strategy s024_ex_dividend_capture --variant yield_gt0_25,yield_gt0_5,yield_gt0_75,yield_gt1,yield_gt1_25,yield_gt1_5,yield_gt1_75,yield_gt2,yield_gt2_25,yield_gt2_5,yield_gt2_75,yield_gt3,yield_gt3_25,yield_gt3_5,yield_gt3_75,yield_gt4,yield_gt4_5,yield_gt5,yield_gt5_5,yield_gt6,yield_gt6_5,yield_gt7,yield_gt7_5,yield_gt8,yield_gt8_5,yield_gt9,yield_gt10,yield_gt12_5,yield_gt15,yield_gt20`

### S026 | Exhaustion Gap Fade After 3-Day Run
- family: overnight and gap
- ohlcv_status: NOT_DAILY, DEFER
- concept: A stock that gaps up after 3+ consecutive up-closes shows climax behavior; late-stage gaps revert as the last buyers are exhausted.
- universe: liquid stocks
- direction: short
- indicators: 3-day up-close streak; gap size vs ATR(20); RSI(2)
- entry: 3 consecutive up closes AND open(t) > close(t-1) by more than 1 × ATR(20) AND RSI(2) > 95; short at open of t
- exit: Cover at the open of t+1 (overnight hold — primary); trail 2-bar high alternate; max 3 sessions
- hold: 1-3 sessions
- frequency: weekly across a liquid universe
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [Capital.com — Gap trading strategies](https://capital.com/en-int/learn/trading-strategies/gap-trading)
- notes: Falsification only. '3 consecutive up closes' already implies a high RSI(2) (the > 95 test is largely redundant); the trade is the opposite side of S019/S100 on similar states; the short is entered at the open on an open-derived signal (fill at the 09:31 bar, G-03.2) with borrow/squeeze risk (G-09). The only source (Capital.com) describes the exhaustion gap as followed by very high volume and appearing near the end of a long trend; this rule has no volume condition and only a 3-day run. No numbers anywhere. Run once as a pre-registered falsification; drop if skill <= 0 at the 5% level with FDR over the gap cluster.
- data_needs: daily_10years (D1) + borrow assumptions
- review: DEMOTE (falsification only) | source GENERIC

---

# Family 3 — Cross-Sectional Short-Term Reversal

*Why this family works: stocks that fall hardest over 1 day to 1 month bounce hardest over the following days (Jegadeesh 1990; Lehmann 1990). The mechanism is liquidity provision: market orders from impatient sellers push prices below fundamental value, and the compensation for providing that liquidity is the reversal premium. The effect is strongest in less liquid, higher-turnover names and after controlling for industry momentum. NY Fed SR 513 shows the monthly reversal profit comes from the within-industry residual (see S075). The attached Borsa Istanbul paper does NOT confirm 1-month reversal as robust: it describes reversal as typically strong in thinly traded emerging markets and finds it among its weakest single characteristics.*

### S027 | Lehmann 1-Week Loser Reversal
- family: cross-sectional short-term reversal
- ohlcv_status: NOT_DAILY, DEFER
- concept: The classic academic reversal: buy the worst performers over the prior week, hold one week. NY Fed SR 513 (Da, Liu & Schaumburg) decomposes reversal profits into across-industry momentum, within-industry variation in expected returns, under-reaction to within-industry cash-flow news and a residual: only the RESIDUAL ('nonfundamental' price changes) is significant and positive, and a strategy built to capture it earns about 3x the risk-adjusted return of the standard sort (1982-2009, monthly sorts). The best-supported implementation is therefore within-industry (peer-adjusted) reversal, see S075.
- universe: liquid_top_N US stocks (PIT rule: top N by trailing 60-day median dollar volume, price >= $5, >= 250 prior bars). Research scale: bottom quintile within the top-1000; paper scale ($5,000, 5 positions): top-10 names
- direction: long (and optional short leg)
- indicators: 5-session trailing return rank; industry-peer and market adjustment (SimFin industries / SIC); optional 1-day skip between formation and holding
- entry: Each week, rank the universe by trailing 5-session return (raw primary; industry-adjusted registered side by side); buy the bottom quintile (research) / worst 10 (paper), equal-weight, at the close of the ranking session (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound)
- exit: Hold 5 sessions; rebalance weekly
- hold: 5 sessions
- frequency: weekly rebalance, always in the market
- provenance: adapted
- status: active
- priority: P1
- sources: [NY Fed Staff Report 513 - Decomposing Short-Term Return Reversal](https://www.newyorkfed.org/medialibrary/media/research/staff_reports/sr513.html) (monthly sorts, not weekly); Lehmann (1990) weekly reversal; Wu & Jegadeesh (2020): closing-auction impact dissipates over 3-5 days
- notes: CORRECTED: the earlier attribution of profits to across-industry momentum and within-industry expected-return variation is wrong (those components are the ones that are NOT significant). Weekly closes carry a bid-ask-bounce component, so report both no-skip and one-day-skip. Weekly turnover is ~100%: at 10 bps/side the drag is ~10%/yr. Long-only beta is ~1.2-1.5 in selloffs, so the null is the vol-matched equal-weight universe with cash. A decile of the top-500 = 50 positions is infeasible at $1,000 per position. Control for S075.
- parameter_variants: adjustment {raw primary, industry-adjusted, industry+market-adjusted}; skip {0 days primary, 1 day}; sort breadth {quintile of top-1000 research, top-10 paper}
- data_needs: daily_10years (D1) + SimFin industry map
- review: RESOLVED

### S028 | Jegadeesh 1-Month Reversal
- family: cross-sectional short-term reversal
- ohlcv_status: NOT_DAILY, DEFER
- concept: Monthly rebalanced reversal: bottom-decile stocks by 1-month return outperform over the next month (Jegadeesh 1990). The NY Fed decomposition (S027) says the monthly strategy works through the within-industry residual, so raw and industry-adjusted versions are both run.
- universe: liquid_top_N US stocks (PIT price-only rule), price >= $5
- direction: long
- indicators: 21-session trailing return rank
- entry: At each month-end, buy the bottom decile by trailing 21-session return (raw primary; industry-adjusted registered), equal-weight, at the close (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound)
- exit: Rebalance monthly (hold 1 month)
- hold: ~21 sessions
- frequency: monthly rebalance
- provenance: adapted
- status: active
- priority: P2
- sources: Jegadeesh (1990), Evidence of Predictable Behavior of Security Returns, J. Finance; [NY Fed Staff Report 513](https://www.newyorkfed.org/medialibrary/media/research/staff_reports/sr513.html); Be The Puck 1-month reversal scorecard (cross-check)
- notes: CORRECTED: the attached Borsa Istanbul paper does NOT say reversal is strongest in liquid universes: its literature review says one-month reversal captures microstructure/liquidity effects 'typically strong in thinly traded emerging markets'; its pure reversal portfolio has net Sharpe 0.93 (CAGR 20.0%) vs 1.35 for the 52-week-high anchor and 1.16 for the equal-weight universe, and removing reversal from the blend RAISES net Sharpe by +0.11 (p = 0.12). The paper defines reversal as minus last month's return inside a z-score composite, not a bottom-decile buy. Be The Puck (stock below SPY last month -> hold a month): beat buy-and-hold Sharpe on 82/372. Benchmark: vol-matched equal-weight universe. Slower than S027; the monthly member of the reversal family.
- parameter_variants: adjustment {raw primary, industry-adjusted}
- data_needs: daily_10years (D1) + SimFin industry map
- review: RESOLVED

### S029 | Capitulation-Volume Loser Reversal
- family: cross-sectional short-term reversal
- ohlcv_status: NOT_DAILY, DEFER
- concept: Reversal is strongest when the selloff came on extreme volume — panic liquidation (mutual fund redemptions, margin calls) rather than informed selling is most likely to revert.
- universe: S&P 500 + mid-cap liquid stocks
- direction: long
- indicators: 5-day return rank; volume z-score (20-day); SPY SMA(200) regime (reported both ways); earnings-window flag
- entry: 5-day return in bottom decile AND volume(t) > 3 x 20-day average volume; buy at close of t (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound); rank by volume z-score and cap positions per day (the partition account caps at 5). Regime: run with the SPY > SMA200 skip filter AND without it, and report conditional on regime; report with and without signals within +/-3 sessions of an earnings release
- exit: Close > 5-day SMA; sell at close of that session
- hold: 2-8 sessions
- frequency: 2-5 signals per week across the universe
- provenance: research-hypothesis
- status: active
- priority: P2
- sources: [Quantified Strategies — 200 free backtested strategies](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: The regime filter is justified as avoiding falling knives, but the fire-sale mechanism (Coval & Stafford 2005; Atlas 12.5) is concentrated in crashes, so the filter may remove the best events (Mar-2020, Dec-2018). Volume > 3x with a bottom-decile 5-day return is dominated by earnings/news-driven drops: without the earnings flag this is S054/S055 in disguise. Signals fire in clusters on market-wide selloff days, so events are not independent: effective sample size per DAY, cluster-robust standard errors by date.
- parameter_variants: regime filter {SPY > SMA200 on, off}; earnings window {included, excluded}
- data_needs: daily_10years (D1) + earnings flag (DP-EARN)
- review: RESOLVED

### S030 | Down-Streak Reversal (5+ Days)
- family: cross-sectional short-term reversal
- ohlcv_status: NOT_DAILY, DEFER
- concept: Consecutive down days are serially overpredicted by panic sellers; Connors' dip-buying rules show that within uptrends, streaks of down closes are bought profitably.
- universe: liquid stocks above $5 with 200-day history
- direction: long
- indicators: down-close streak count; SMA(200); SMA(5)
- entry: 5+ consecutive down closes AND close > SMA200; buy at close of the 5th down day (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound)
- exit: Close > 5-day SMA; sell at close of that session
- hold: 2-6 sessions
- frequency: daily opportunities across 500+ stocks
- provenance: adapted
- status: active
- priority: P3
- sources: [CXO Advisory — Connors rules (buy pullbacks, above 200MA)](https://www.cxoadvisory.com/technical-trading/a-few-notes-on-short-term-trading-strategies-that-work/)
- notes: Streaks are the raw material of Connors-style pullback systems; this is the pure streak version without oscillators. Run as the stock-universe member of the down-streak family N = 2..7 together with S007 and S009 (G-10); report with and without signals within +/-3 sessions of an earnings release (G-12). CXO page has general Connors principles only.
- parameter_variants: streak length N {5 primary; 2..7 family grid shared with S007 and S009}; earnings window {included, excluded}
- data_needs: daily_10years (D1) + earnings flag (DP-EARN)
- review: NO FINDINGS (strategy-specific) | source GENERIC

### S031 | Up-Streak Fade (5+ Days)
- family: cross-sectional short-term reversal
- concept: Mirror of S030: extended up-streaks in stocks below their 200-day SMA (weak regime) exhibit short-term exhaustion.
- universe: liquid stocks
- direction: short
- indicators: up-close streak count; SMA(200); SMA(5)
- entry: 5+ consecutive up closes AND close < SMA200; short at close of the 5th up day
- exit: Close < 5-day SMA OR RSI(2) < 10; cover at close of that session; max 5 sessions
- hold: 2-5 sessions
- frequency: weekly across a liquid universe
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [CXO Advisory — Connors rules](https://www.cxoadvisory.com/technical-trading/a-few-notes-on-short-term-trading-strategies-that-work/)
- notes: CORRECTED: the earlier note said Chan's mirror short-on-gap 'failed to reproduce'; per the l3a0 page the mirror was profitable pre-cost (APR 12.2%, Sharpe 1.79) but did not match the published 46%/1.27 (see S016). Five up closes below the 200-day SMA is a bear-market rally where short-term momentum (S100) and squeeze risk dominate; borrow data is absent (G-09). Run once as the short counterpart to S030 under borrow scenarios (0.3% / 3% / 10% annualised); drop if not significant after FDR. Falsification only.
- data_needs: daily_10years (D1) + borrow assumptions
- review: DEMOTE | source MISMATCH (note)

### S032 | 52-Week-High Proximity Portfolio (George-Hwang)
- family: cross-sectional short-term reversal
- concept: Nearness to the 52-week high is a stronger predictor of future returns than conventional momentum (George & Hwang 2004): the anchor biases sellers into trading too early, so stocks near their high keep grinding up.
- universe: liquid_top_N US stocks (PIT price-only rule)
- direction: long
- indicators: close / 252-day high ratio
- entry: Each month, rank stocks by close / 252-day-high ratio and buy the top decile (rank rule, primary), equal-weight, with a minimum ratio of 0.8 as a registered variant (a fixed 0.95 level selects far more than 10% of names in bull markets and far fewer in bear markets)
- exit: Rebalance monthly; George-Hwang holding variant: skip a month, hold six months with overlapping cohorts
- hold: ~21 sessions (primary); ~126 sessions overlapping (George-Hwang form)
- frequency: monthly rebalance
- provenance: adapted
- status: active
- priority: P2
- sources: George & Hwang (2004), The 52-Week High and Momentum Investing, J. Finance (not retrievable in review); Attached paper (user upload) - Borsa Istanbul (52-week-high characteristic: best single-characteristic net Sharpe 1.35 / CAGR 29.2%, but the paper stresses the ranking is noise, p = 0.12, and warns about retrospective discovery); Be The Puck stock-level 52-week-high scorecard (69/370 beat buy-and-hold Sharpe)
- notes: The Trading Garden page cited earlier tests a 252-day-high BREAKOUT on the S&P cash index, i.e. S033, not a decile portfolio, and is no longer cited here. Highly collinear with S041 momentum: report the correlation (G-10). Benchmark: vol-matched equal-weight universe.
- parameter_variants: selection {top decile by rank primary; ratio >= 0.8 floor}; holding {1 month primary; 6 months overlapping with 1-month skip}
- data_needs: daily_10years (D1)
- review: RESOLVED
- rust_implementation: Registered as `s032_52w_high_proximity` with PIT liquidity-rank filtering, monthly high-proximity ranking, ratio-floor selection, and overlapping holding cohorts; 30 named variants.
- implementation_results: `src/signals.rs` / `src/study.rs` and `configs/s032_52w_high_proximity.yaml`; 1,400 stock TR series, 15 bps/side, next_open primary, 2016-12-30 through 2026-10-23; dev < 2024-01-01 <= test.

  | variant | dev total (log) | dev skill | dev Sharpe | dev max DD | test total (log) | test skill | test Sharpe | test max DD | trades (dev+test) |
  |---|---|---|---|---|---|---|---|---|---|
  | baseline | +0.259 | -0.501 | 0.306 | -30.4% | +0.298 | -0.092 | 0.757 | -17.0% | 4113+1665 |
  | top15_n100 (dev-selected) | +0.678 | -0.164 | 0.612 | -30.3% | +0.443 | -0.144 | 0.836 | -28.0% | 1260+510 |
  `top15_n100` ranked highest on dev skill among the 30 variants, but both it and the baseline remain below the random-pick skill benchmark. `hold6_skip1_top15` had test PF 2.17 but dev skill -0.172 and test skill +0.021 (DECAYED); no look-ahead was found in the close-signal/next-open cohort logic. Reproduce: `target\release\deepthrust.exe --strategy s032_52w_high_proximity --stages partition`; selected: add `--variant top15_n100`.

### S035 | Low-Volatility Anomaly Portfolio
- family: cross-sectional short-term reversal
- concept: Low-volatility stocks earn risk-adjusted returns at least as high as high-volatility stocks (leverage-constraint effect, Frazzini-Pedersen; Novy-Marx & Velikov attribute much of the anomaly to non-transparent equal-weighting, so the benchmark choice decides the result). The attached Borsa paper's finding is that ALL characteristics are redundant against an equal-weight, vol-matched universe, not that low-vol specifically is.
- universe: liquid_top_N US stocks: N=500 primary; N=100 (the paper's top-100-by-60-day-median-traded-value universe) as a variant
- direction: long
- indicators: 252-trading-day annualized std of daily returns (primary); 60-day std (variant); beta vs SPY (variant)
- entry: Each month, buy the lowest-volatility decile (lowest 252-day vol), equal-weight
- exit: Rebalance monthly
- hold: ~21 sessions
- frequency: monthly rebalance
- provenance: adapted
- status: active
- rust_status: IMPLEMENTED
- priority: P2
- sources: Attached paper (user upload) - Multi-Factor Investing in Borsa Istanbul (low-vol = minus the annualised std over the last 252 trading days, eq. 1); Frazzini & Pedersen (2014); Novy-Marx & Velikov
- notes: CORRECTED: the paper uses 252-day vol on a top-100 liquid universe (ranked by 60-day median traded value); the 60-day vol appears only as the vol-targeting estimator for the whole book. Paper numbers: pure low-vol net Sharpe 1.07 / CAGR 21.8% / max drawdown -66.8% in lira terms, USD Sharpe 0.13; the sector-neutral run adds only +1.4% alpha (t = 0.48). Low-vol deciles are utilities/staples/REIT heavy (2022 rate sensitivity, sector concentration). Benchmark: vol-matched equal-weight universe, not just the cap-weighted index.
- parameter_variants: vol window {252-day primary, 60-day}; universe N {500 primary, 100}; sector-neutral {off primary, on}; low-beta decile (shrunk 252-day beta = 0.6 x beta_ts + 0.4, beta-scaled to 1) as a variant
- data_needs: daily_10years (D1); SPY (DP-ETF) for low-beta; SimFin company/sector mapping for sector-neutral
- review: RESOLVED
- rust_implementation: Registered as `s035_low_volatility` with annualized rolling stock volatility, shrunk-SPY-beta selection and scaling, sector-neutral selection, and 31 named variants. Sector-neutral mode uses the available SimFin company/industry mapping, which is not point-in-time.
- implementation_results: `src/signals.rs` / `src/study.rs` and `configs/s035_low_volatility.yaml`; 1,400 stocks plus SPY TR bars, 15 bps/side stocks, next_open primary, 2016-12-29 through 2026-10-23; dev < 2024-01-01 <= test.

  | variant | dev total (log) | dev skill | dev Sharpe | dev max DD | test total (log) | test skill | test Sharpe | test max DD | trades (dev+test) |
  |---|---|---|---|---|---|---|---|---|---|
  | baseline | +0.397 | -0.333 | 0.458 | -35.3% | +0.288 | -0.101 | 1.017 | -9.6% | 4067+1666 |
  | vol504 (dev-selected) | +0.274 | -0.276 | 0.367 | -35.0% | +0.267 | -0.121 | 0.928 | -8.4% | 3479+1666 |
  `vol504` ranked highest on dev skill among the 31 variants, but its skill remains negative on both partitions; neither result supports an anomaly after the vol-matched random-pick comparison. Reproduce: `target\release\deepthrust.exe --strategy s035_low_volatility --stages partition`; selected: add `--variant vol504`.

---

# Family 4 — Momentum, Breakout, and Rotation

**Recalculated:** The tables below use prior-session tradable-universe trend, corrected rebalance exposure and monthly annualization. DEV rankings do not include a full variant-search bias adjustment; TEST has already been inspected and is not a fresh holdout. Next-open trend uses a close-to-close exposure approximation. Exact DEV ties use the last configured candidate, matching the symbols overview.

Across the complete grids, PF or Sharpe exceeded 2 in 14 S042, 45 S043, 33 S044,
and 65 S047 variant/fill/partition rows; none of the S046 rows exceeded 2 after
correcting annualization and short trade returns. Review timing and evidence
before interpreting these figures. S042's `top1` exceeded SPY by at least 10%
in DEV but failed the 100-trade DEV gate. S041's baseline and 15 variants,
and S043's `mom189_top1`, `mom252_top1`, `mom315_top1`, and `mom378_top1`,
exceeded SPY's cumulative growth by at least 10% in TEST. SPY is close-to-close
and is not fill-timing-matched; the inspected TEST results are not fresh evidence.

*Why this family works: momentum is the most replicated anomaly in finance — 3-12 month winners keep winning (Jegadeesh & Titman 1993). At daily granularity, breakouts from consolidation (Donchian channels, NR7 compression) exploit the same underreaction. Rotation systems (sector, factor, asset-class) harvest momentum with diversification. Chan notes the standard lookback is roughly 11 of the most recent 12 months, skipping the most recent month (the reversal window).*

### S033 | 52-Week-High Breakout (Donchian-style)
- family: trend / slow breakout
- concept: Buy at a new 252-day high, exit at a 126-day low — a slow breakout system; documented max single-trade loss 12.9% (max MAE) in the Trading Garden test.
- universe: primary: sector/asset ETF set and liquid_top_N stocks (pooled trades and effective sample size); reference: SPY / S&P index (via MES in the source)
- direction: long
- indicators: 252-day rolling high; 126-day rolling low
- entry: Signal when Close(t) = 252-day closing high; enter at the next open (primary), with close fill as a sensitivity
- exit: Signal when Close(t) is at or below the 126-day closing low; exit at the next open (primary), with close fill as a sensitivity
- hold: weeks to months
- frequency: ~1-3 new positions per year on the index; run across sector ETFs and stocks for frequency
- provenance: source-specified
- status: active
- rust_status: IMPLEMENTED
- priority: P3
- sources: [Trading Garden — 16 S&P 500 trading strategies](https://trading-garden.com/sp500-trading-strategies.html)
- notes: Trading Garden (vendor AVALONPLUS comparison, S&P cash index/MES, sized by max MAE): 31 trades 1963-today, 71% win, CAGR 11.0% vs 10.9% buy-and-hold, worst single-trade loss 12.9% (max MAE), but only 14 trades since 2000 (CAGR 16.0% reinvested on MES): no statistical power on an index alone; it adds value as a regime overlay or when run across many ETFs/stocks. Compare with 200-day SMA timing as the null. The 63-day-high / 21-day-low fast variant is a second registered trial.
- parameter_variants: speed {252-day high / 126-day low primary; 63-day high / 21-day low}
- data_needs: ETF total-return daily series (DP-ETF); daily_10years (D1)
- review: KEEP (overlay/anchor) | source OK
- rust_implementation: Registered as `s033_52w_breakout` in `src/study.rs`, with rolling 252/126-day close signals, next-open execution, ETF plus PIT-liquidity-ranked stock sources, and 30 named variants.
- implementation_results: `src/signals.rs` / `src/study.rs` and `configs/s033_52w_breakout.yaml`; 20 ETFs plus liquid-top-500 stocks (1,420 input symbols), 5 bps/side ETFs and 15 bps/side stocks, next_open primary, 2017-01-12 through 2026-10-23; dev < 2024-01-01 <= test.

  | variant | dev total (log) | dev skill | dev Sharpe | dev max DD | test total (log) | test skill | test Sharpe | test max DD | trades (dev+test) |
  |---|---|---|---|---|---|---|---|---|---|
  | baseline | +0.748 | -0.007 | 0.646 | -33.6% | +0.631 | +0.227 | 1.040 | -22.1% | 2439+841 |
  | top250 (dev-selected) | +0.832 | +0.069 | 0.691 | -31.8% | +0.390 | -0.079 | 0.878 | -23.0% | 1287+462 |
  `top250` had positive but statistically weak dev skill (p=0.394); test skill flips negative (`DECAYED`). Its PF is >2 on dev (2.44) and test (2.28), so treat those trade-level figures cautiously; the index/MES source differs from this pooled ETF/stock adaptation. Code inspection found signals use completed closes and primary entries/exits occur at the following open. Reproduce: `target\release\deepthrust.exe --strategy s033_52w_breakout --stages partition`; selected: add `--variant top250`.

### S036 | Donchian Breakout, Turtle Systems 1 and 2
- family: momentum and breakout
- concept: Entering on an N-day high breakout captures the start of multi-week trends; the shorter opposite-extreme exit cuts losers fast; unit sizing normalizes volatility. Turtle returns came from a diversified futures book (rates, FX, commodities); on equity-only ETFs this is mostly whipsaw long-equity beta, so the universe must be diversified. Run as ONE family with speeds {20, 55}.
- universe: diversified ETF set (equity SPY/QQQ/IWM, rates TLT/IEF, gold GLD, oil USO, USD UUP/dollar ETF, commodities GSG) plus sector ETFs; long-only and long-short reported separately
- direction: long and short
- indicators: Donchian high/low; N = Wilder-smoothed ATR, N_t = (19 x N_{t-1} + TR_t)/20 (alpha = 1/20, not an EMA with span 20); shadow-trade ledger for the System 1 filter
- entry: System 1 (speed 20): close(t) > max high over sessions t-20..t-1 (EXCLUDING the current session); skip the breakout if the last breakout (real or shadow) would have been a winner. System 2 (speed 55): close(t) > max high over t-55..t-1, no last-trade filter. Unit size = 1% equity / (N x point value), capped at the per-position limit of the account; fill at next open
- exit: System 1: close < 10-day low; System 2: close < 20-day low; mandatory 2N stop from entry (fill = min(open, stop)); mirror for shorts
- hold: days to weeks
- frequency: ~2-6 trades per year per instrument; run across 10-20 ETFs for weekly frequency
- provenance: source-specified
- status: demoted
- priority: P3
- sources: [Kagels Trading — Original Turtle rules](https://kagels-trading.com/blog/turtle-trading/); [Investopedia — Donchian Channels](https://www.investopedia.com/terms/d/donchianchannels.asp)
- notes: Kagels and Be The Puck agree with the rules (20-day entry / 10-day exit, N = 20-day ATR average, 1% risk per N, skip the breakout if the last would have won, out on a close below the prior 10-day low or 2N below entry). Be The Puck results: single stocks 62/372 (S1) and 61/372 (S2) beat buy-and-hold Sharpe; S&P 500 index CAGR +2.5% / +2.8% vs +8.9% (Sharpe 0.13 / 0.16 vs 0.45), long only. The Atlas (10.1, Kurth et al. 2026) finds short-term trend in ~100 futures stopped delivering since roughly 2009. Sizing at 1% equity / (N x point value) does not fit a $5,000 account with $1,000 positions. Trend-cluster control (with S033, S051, S091). Report the correlation between speeds.
- parameter_variants: speed {20/10 System 1 primary; 55/20 System 2 (ex-S037)}; last-trade filter {on for speed 20, off for speed 55}; exit {channel exit primary; chandelier 3 x ATR(22) (see S091)}; direction {long-only, long-short}
- merged_from: S037
- data_needs: ETF total-return daily series incl. TLT, GLD, USO, GSG, dollar ETF (DP-ETF)
- review: RESOLVED

### S038 | NR7 / Inside-Day Breakout
- family: momentum and breakout
- concept: Toby Crabel's compression idea: the narrowest range of 7 sessions (or an inside day) precedes expansion; buying the break of the setup bar's high (or shorting the low break) trades the direction of the resolution.
- universe: liquid stocks and ETFs
- direction: long and short
- indicators: 7-day rolling range (NR7 test); inside-day test (high < prior high, low > prior low)
- entry: On a setup session (NR7 or inside day), place a buy stop at the setup bar's high and a sell stop at its low for the next session; the first stop touched enters; cancel both if neither is touched. Stop-entry fills follow max(open, level) for buys and min(open, level) for sells (gap-aware). If BOTH levels trade in the session, the direction is the one whose stop is nearer the session OPEN; if exactly equidistant, skip
- exit: Flatten on the close after 2 sessions (primary baseline). Registered exits: first profitable close (Crabel); opposite break; stop_target (ex-S039: stop at the opposite side of the NR7 bar, target 2 x initial risk, notional capped at the account's per-position limit)
- hold: 2-5 sessions
- frequency: several setups per week across a liquid universe
- provenance: adapted
- status: demoted
- priority: P3
- sources: [Fractiz - NR7 / inside day breakout rules](https://www.fractiz.com/strategies/nr7-inside-breakout/); StockCharts ChartSchool (Crabel's setup is an opening-range breakout with profits taken at the close of the first day or first profitable close); Validated Strategies NR7 page (https://validatedstrategies.com/strategy/nr7-xauusd) returned HTTP 404 in review, so the PF 0.64 gold claim is unverifiable
- notes: Fractiz's sample runs (2020-2024, daily futures) show weak edges and ~40% drawdowns: NQ win 44%, PF 1.04, max drawdown 40.6%; ES PF 1.21, DD 39.3%; GC PF 1.39, DD 38.4% (154-163 trades). Be The Puck NR7: S&P 500 index -2.1%/yr vs +8.9%, beat buy-and-hold Sharpe on 52/372. Holding two sessions differs from Crabel's quick exit. NR7 or inside day, both sides, doubles the trial count. Falsification test: require a positive net PF with the bootstrap CI lower bound > 1. The stop and 2R target are an exit variant; a stop at the opposite side of an NR7 bar is typically 0.3-0.8% of price, so '1% equity risk' sizing would imply 1.3-3x notional and the cap makes the 1% risk fictional.
- parameter_variants: setup {NR7 or inside day; NR7 only; inside day only}; exit {2-session flatten primary; first profitable close; opposite break; stop + 2R target (ex-S039)}
- merged_from: S039
- data_needs: daily_10years (D1); ETF daily bars (DP-ETF)
- review: RESOLVED

### S040 | Volume-Confirmed 20-Day-High Breakout
- family: momentum and breakout
- concept: New highs on above-median volume are institutional accumulation; volume separates genuine breakouts from stop-hunts.
- universe: S&P 500 stocks
- direction: long
- indicators: 20-day rolling high of highs; ATR(14); 20-day median volume
- entry: Close(t) > 20-day high AND close(t) - 20-day high > 0.1 x ATR(14) AND volume(t) vs 20-day median volume in one of two registered regimes (> 1.5x primary; < 1.0x quiet breakout); gap guard: skip if open(t+1) > close(t) + 1 x ATR(14), or enter by limit at close(t); buy at open of t+1
- exit: Initial stop below breakout bar's low (fill = min(open, stop)); target 2 x initial risk or 8 sessions, whichever first
- hold: 1-8 sessions
- frequency: several per week across the universe
- provenance: research-hypothesis
- status: active
- priority: P2
- sources: [Kvants - Volume breakout strategy (generic framework article, no daily rules or numbers)](https://studio.kvants.ai/blog/volume-breakout-strategy)
- notes: The Kvants page is a framework article ('a testable version must define the level, breakout distance, volume baseline...'); the parameters here (0.1 ATR, 1.5x median volume, 2R, 8 sessions) are the catalog's own, hence research-hypothesis. The Atlas (10.4, Mittal & Choudhary 2026) argues 'quiet-looking moves travel further than visually impressive breakouts', so both volume regimes are tested. Entering at the next open after a high-volume breakout carries gap risk. Be The Puck's 20-day-style rules (Four-Week Rule 74/372; Bollinger Breakout 54/372) are mediocre vs buy-and-hold.
- parameter_variants: lookback {10, 20 primary, 30, 40 sessions}; breakout distance {0.05, 0.10 primary, 0.20 ATR}; volume threshold {1.25x, 1.5x primary, 2.0x}; quiet-volume regime and limit-at-close execution are separate variants.
- data_needs: daily_10years (D1)
- rust_status: IMPLEMENTED
- implementation_delta: DELTA — the S&P universe comes from a public GitHub-aggregated ticker interval CSV, not an official S&P source; available stock bars start 2016-01-04 despite the config warm-up start of 2008.
- implementation_results: Rust in `src/family4.rs` / `src/study.rs`, config `configs/s040_volume_breakout.yaml`; Universe: A, AABA, AAL, AAP, AAPL, ABBV, ABMD, ABNB, ABT, ACGL, ACN, ADBE, ADI, ADM, ADP, ADSK, ADT, AEE, AEP, AES, AET, AFL, AGN, AIG, AIZ, AJG, AKAM, AKS, ALB, ALGN, ALK, ALL, ALLE, ALXN, AMAT, AMCR, AMD, AME, AMG, AMGN, AMP, AMT, AMZN, AN, ANDV, ANET, ANF, ANSS, AON, AOS, APA, APC, APD, APH, APO, APP, APTV, AR, ARE, ARES, ARNC, ATI, ATO, ATVI, AVB, AVGO, AVY, AWK, AXON, AXP, AYI, AZO, BA, BAC, BALL, BAX, BBBY, BBWI, BBY, BCR, BDX, BEN, BF.B, BG, BIIB, BIO, BK, BKNG, BKR, BLDR, BLK, BMY, BNY, BR, BRK.B, BRO, BSX, BUD, BWA, BX, BXP, C, CAG, CAH, CAR, CARR, CASY, CAT, CB, CBOE, CBRE, CBS, CCEP, CCI, CCK, CCL, CDNS, CDW, CE, CEG, CELG, CERN, CF, CFG, CG, CHD, CHRW, CHTR, CI, CIEN, CINF, CL, CLF, CLX, CMA, CMCSA, CME, CMG, CMI, CMS, CNC, CNP, CNX, COF, COHR, COIN, COL, COO, COP, COR, COST, COTY, CPAY, CPB, CPRI, CPRT, CPT, CRH, CRL, CRM, CRWD, CSCO, CSGP, CSX, CTAS, CTL, CTLT, CTRA, CTSH, CTVA, CTXS, CVNA, CVS, CVX, CXO, CZR, D, DAL, DASH, DAY, DD, DDOG, DE, DECK, DELL, DFS, DG, DGX, DHI, DHR, DIS, DISCA, DISCK, DISH, DLR, DLTR, DOC, DOV, DOW, DPZ, DRE, DRI, DTE, DUK, DVA, DVN, DWDP, DXC, DXCM, EA, EBAY, ECHO, ECL, ED, EFX, EG, EIX, EL, ELV, EME, EMN, EMR, ENDP, ENPH, EOG, EPAM, EQIX, EQR, EQT, ES, ESRX, ESS, ESV, ETFC, ETN, ETR, ETSY, EVHC, EVRG, EW, EXC, EXE, EXPD, EXPE, EXR, F, FANG, FAST, FCX, FDC, FDS, FDX, FE, FERG, FFIV, FHN, FI, FICO, FIS, FISV, FITB, FIX, FL, FLEX, FLR, FLS, FLT, FMC, FOX, FOXA, FRC, FSLR, FTI, FTNT, FTR, FTV, GD, GDDY, GE, GEHC, GEN, GEV, GGP, GILD, GIS, GL, GLW, GM, GME, GNRC, GOOG, GOOGL, GPC, GPN, GRMN, GS, GT, GWW, HAL, HAR, HAS, HBAN, HBI, HCA, HCP, HD, HES, HFC, HIG, HII, HLT, HOG, HOLX, HON, HOOD, HP, HPE, HPQ, HRB, HRL, HRS, HSIC, HST, HSY, HUBB, HUM, HWM, IAC, IBKR, IBM, ICE, IDXX, IEX, IFF, ILMN, INCY, INTC, INTU, INVH, IP, IPG, IPGP, IQV, IR, IRM, ISRG, IT, ITW, IVZ, J, JBHT, JBL, JCI, JCP, JEC, JKHY, JNJ, JNPR, JPM, JWN, K, KATE, KDP, KEY, KEYS, KHC, KIM, KKR, KLAC, KMB, KMI, KMX, KO, KORS, KR, KSS, KSU, KVUE, LB, LDOS, LEN, LH, LHX, LII, LIN, LITE, LKQ, LLL, LLTC, LLY, LM, LMT, LNC, LNT, LOW, LPX, LRCX, LSI, LULU, LUMN, LUV, LVLT, LVS, LW, LYB, LYV, M, MA, MAA, MAC, MAR, MAS, MAT, MCD, MCHP, MCK, MCO, MDLZ, MDT, MET, META, MGM, MHK, MJN, MKC, MKTX, MLM, MMC, MMM, MNK, MNST, MO, MOH, MON, MOS, MPC, MPWR, MRK, MRNA, MRO, MRSH, MRVL, MS, MSCI, MSFT, MSI, MTB, MTCH, MTD, MU, MUR, MXIM, MYL, NBL, NBR, NCLH, NDAQ, NEE, NEM, NFLX, NI, NKE, NKTR, NLSN, NOC, NOV, NOW, NRG, NSC, NTAP, NTRS, NUE, NVDA, NVR, NWL, NXPI, O, ODFL, OKE, OMC, ON, ORCL, ORLY, OTIS, OXY, PANW, PAYC, PAYX, PBCT, PCAR, PCG, PEG, PENN, PEP, PFE, PFG, PG, PGR, PH, PHM, PKG, PLD, PLTR, PM, PNC, PNR, PNW, PODD, POOL, PPG, PPL, PRGO, PRU, PSA, PSX, PTC, PVH, PWR, PX, PXD, PYPL, Q, QCOM, QRVO, RAI, RCL, RDDT, RDS.A, REG, REGN, RF, RHI, RHT, RIG, RJF, RL, RMD, ROK, ROL, ROP, ROST, RRC, RSG, RTN, RTX, RVTY, S, SATS, SBAC, SBNY, SBUX, SCG, SCHW, SE, SEDG, SEE, SHW, SIG, SIVB, SJM, SLB, SLG, SMCI, SNA, SNDK, SNI, SNPS, SNV, SO, SPG, SPGI, SRCL, SRE, STE, STJ, STLD, STT, STX, STZ, SW, SWK, SWKS, SWN, SYF, SYK, SYMC, SYY, T, TAP, TDG, TDY, TE, TECH, TEL, TER, TFC, TFX, TGT, THC, TIF, TJX, TKO, TMO, TMUS, TPL, TPR, TRGP, TRIP, TROW, TRV, TSCO, TSLA, TSN, TSS, TT, TTD, TTWO, TWTR, TWX, TXN, TXT, TYL, UA, UAA, UAL, UBER, UDR, UHS, ULTA, UNH, UNM, UNP, UPS, URBN, URI, USB, UTX, V, VAR, VEEV, VFC, VIAB, VIAV, VICI, VLO, VLTO, VMC, VMRK, VNO, VRSK, VRSN, VRT, VRTX, VST, VTR, VTRS, VZ, WAB, WAT, WB, WBA, WBD, WCG, WDAY, WDC, WEC, WELL, WFC, WFM, WFT, WHR, WLL, WM, WMB, WMT, WPX, WRB, WRK, WSM, WST, WTW, WU, WY, WYNN, XEC, XEL, XL, XLNX, XOM, XRAY, XRX, XYL, XYZ, YUM, ZBH, ZBRA, ZION, ZTS (daily bars from 1 sources); each symbol at most one position, book slots 1.; Data: `C:\data\trading\trading_framework\data\cache\daily_10years_tr`; sessions 2016-01-04 .. 2026-10-23; evaluation starts 2016-01-04.. Full grid: 30 settings including baseline, 30 variant/fill rows; primary fill `next_open`. DEV-only selection `w40_d020_v125`; baseline DEV/TEST skill -1.787/-0.671; selected DEV/TEST skill -1.345/-0.720. Selected TEST Sharpe -0.731, PF +0.882, win +39.5%, trades 4509. no selected-result evidence or high-metric flag. Variant-search bias is not included; TEST has been inspected.
  | portfolio | variant | partition | total log | trend log | random-pick baselines (log; p) | skill (log) | skill vs random U / pool (log) | bias (log) | skill % | skill p | Sharpe | PF | max DD | win % | trades |
  |---|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | `baseline` | DEV | -1.106 | +0.681 | U -0.330 (p=+0.999); pool -0.357 (p=+0.999) | -1.787 | U -0.776; pool -0.749 | +0.000 | n/a | +1.000 | -0.776 | +0.860 | -73.2% | +40.0% | 12203 |
  | baseline | `baseline` | TEST | -0.292 | +0.378 | U -0.128 (p=+0.934); pool -0.125 (p=+0.939) | -0.671 | U -0.164; pool -0.167 | +0.000 | n/a | +0.998 | -0.636 | +0.855 | -30.4% | +39.1% | 4402 |
  | DEV-selected | `w40_d020_v125` | DEV | -0.644 | +0.701 | U -0.236 (p=+0.936); pool -0.258 (p=+0.934) | -1.345 | U -0.409; pool -0.386 | +0.000 | n/a | +1.000 | -0.452 | +0.830 | -55.2% | +39.7% | 12491 |
  | DEV-selected | `w40_d020_v125` | TEST | -0.316 | +0.404 | U -0.064 (p=+0.985); pool -0.069 (p=+0.989) | -0.720 | U -0.252; pool -0.247 | +0.000 | n/a | +0.998 | -0.731 | +0.882 | -30.6% | +39.5% | 4509 |
  Reproduce: full matrix `cargo run --release -- --strategy s040_volume_breakout --variant quiet_breakout,limit_close,w10_d005_v125,w10_d005_v150,w10_d005_v200,w10_d010_v125,w10_d010_v150,w10_d010_v200,w10_d020_v125,w10_d020_v150,w10_d020_v200,w20_d005_v125,w20_d005_v150,w20_d005_v200,w20_d010_v125,w20_d010_v200,w20_d020_v125,w20_d020_v150,w20_d020_v200,w40_d005_v125,w40_d005_v150,w40_d005_v200,w40_d010_v125,w40_d010_v150,w40_d010_v200,w40_d020_v125,w40_d020_v150,w40_d020_v200,w30_d010_v150`; DEV-selected winner `cargo run --release -- --strategy s040_volume_breakout --variant w40_d020_v125`.
- review: RESOLVED

### S041 | 12-1 Cross-Sectional Momentum
- family: momentum and breakout
- concept: The canonical academic momentum: rank stocks on returns from t-12 months to t-1 month (skipping the recent month to avoid the reversal window); buy winners. Chan confirms the standard lookback of ~11 of the last 12 months.
- universe: liquid_top_N US stocks (PIT rule, price >= $5, delisting returns applied); research scale: top decile; paper scale ($5,000): top-10
- direction: long (and optional short leg)
- indicators: 12-1 month total-return rank (return from t-252 to t-21 sessions)
- entry: At each month-end close, buy the top-decile 12-1 return stocks (top-10 at paper scale), equal-weight
- exit: Rebalance monthly
- hold: ~21 sessions
- frequency: monthly rebalance
- provenance: source-specified
- status: active
- rust_status: IMPLEMENTED
- priority: P1
- sources: [Better System Trader — Ernie Chan interview (momentum lookback)](https://bettersystemtrader.com/012-ernest-chan/); [Jegadeesh-Titman 1993 (via Bauer College)](https://www.bauer.uh.edu/rsusmel/phd/jegadeesh-titman93.pdf)
- notes: Chan interview: standard momentum lookback is roughly 11 of the most recent 12 months; Jegadeesh & Titman 1993 (formation J and holding K of 3-12 months; one-week gap between formation and holding in their main tests). Be The Puck 12-1 momentum beat buy-and-hold Sharpe on 120/369, the best of its momentum family. Crash exposure: a long-only top decile is far less crash-prone than long-short but 2009 and 2020 rebounds hit it (overlays: S104). Required comparisons: vs random 10-stock placebo and vs the vol-matched equal-weight universe (the Borsa appendix: momentum as a negative control, random placebo beat residual momentum in 99.3% of draws, winners were high-vol stocks); industry-neutral variant (industry momentum drives much of stock momentum, Moskowitz-Grinblatt).
- parameter_variants: lookback {126, 189, 252 primary, 315, 378 sessions}; skip {0, 10, 21 primary, 42 sessions}; breadth {5%, 10% primary, 15%, top 10/20/30/40/50/75/100/150/200}; industry-neutral {off primary, on}. S104-gate and volatility-scaling overlays are not included in the tested matrix.
- data_needs: daily_10years (D1) total-return-adjusted incl. delistings
- rust_implementation: Registered as `s041_mom_12_1` with daily 12-1 momentum, monthly reconstitution, PIT liquidity ranks, and named lookback/breadth variants. Industry-neutral processing is available using the latest SimFin company/industry snapshot; this mapping is not historical point-in-time.
- implementation_delta: DELTA — next-open is the executable primary fill; same-close is reported only as an upper bound. The Rust book is long-only; the optional short leg and S104/volatility-scaling overlays are not implemented. PIT liquidity ranks are used, but industry labels are the latest SimFin snapshot rather than historical PIT assignments.
- review: KEEP | source OK
- implementation_results: Rust in `src/family4.rs` / `src/study.rs`, config `configs/s041_mom_12_1.yaml`; Universe: A, AA, AABA, AAL, AAOI, AAP, AAPL, ABBV, ABCM, ABEV, ABMD, ABNB, ABT, ACB, ACC, ACGL, ACHR, ACIA, ACN, ADBE, ADI, ADM, ADP, ADSK, ADT, AEE, AEHR, AEIS, AEM, AEO, AEP, AER, AES, AET, AFL, AFRM, AG, AGN, AGNC, AGQ, AHT, AI, AIG, AIRC, AIZ, AJG, AKAM, AKRX, AKS, ALAB, ALB, ALC, ALGN, ALK, ALL, ALLE, ALLY, ALNY, ALXN, AMAT, AMBA, AMC, AMCR, AMD, AME, AMG, AMGN, AMKR, AMP, AMRN, AMT, AMTD, AMZN, AN, ANDV, ANET, ANF, ANSS, AON, AOS, APA, APC, APD, APH, APHA, APLD, APLS, APO, APP, APPN, APPS, APTV, AR, ARCH, ARCT, ARE, ARES, ARGX, ARM, ARMK, ARNC, ARRS, ARRY, ARWR, ASAN, ASML, ASST, ASTS, ASX, ATHM, ATHN, ATI, ATO, ATUS, ATVI, AU, AUY, AVAV, AVB, AVGO, AVLR, AVTR, AVY, AWK, AXON, AXP, AXTA, AXTI, AYI, AYX, AZEK, AZN, AZO, B, BA, BABA, BAC, BAH, BALL, BAM, BAP, BAX, BB, BBAI, BBBY, BBD, BBIG, BBL, BBWI, BBY, BCR, BCS, BDX, BE, BEAV, BEKE, BEN, BF.B, BFH, BG, BHC, BHI, BHP, BHVN, BIDU, BIIB, BILI, BILL, BIO, BIOR, BIVV, BJ, BK, BKI, BKNG, BKR, BLD, BLDP, BLDR, BLK, BLNK, BLUE, BMNR, BMRN, BMY, BN, BNGO, BNO, BNTX, BNY, BOIL, BOX, BP, BPL, BPMC, BR, BRCD, BRK.A, BRK.B, BRO, BROS, BSX, BTBT, BTDR, BTG, BTI, BTU, BUD, BULL, BURL, BWA, BWLD, BWXT, BX, BXP, BYND, BZUN, C, CAA, CACI, CAG, CAH, CAN, CAR, CARR, CART, CASY, CAT, CAVA, CAVM, CB, CBI, CBOE, CBRE, CBS, CC, CCEP, CCI, CCJ, CCK, CCL, CDE, CDNS, CDW, CE, CEG, CEI, CELG, CELH, CERN, CF, CFG, CFLT, CG, CGC, CGNX, CHD, CHGG, CHKP, CHPT, CHRD, CHRW, CHTR, CHWY, CI, CIEN, CIFR, CINF, CIT, CL, CLF, CLOV, CLR, CLS, CLSK, CLVS, CLX, CM, CMA, CMCSA, CME, CMG, CMI, CMS, CNC, CNH, CNI, CNK, CNP, CNQ, CNX, COF, COH, COHR, COIN, COL, COMM, CONE, COO, COP, COR, CORZ, COST, COTY, COUP, CP, CPAY, CPB, CPN, CPNG, CPRI, CPRT, CPT, CRCL, CRDO, CREE, CRH, CRI, CRL, CRM, CRON, CROX, CRS, CRSP, CRUS, CRWD, CRWV, CSC, CSCO, CSGP, CSIQ, CSL, CSX, CTAS, CTL, CTLT, CTRA, CTRP, CTSH, CTVA, CTXS, CVE, CVNA, CVS, CVX, CW, CWAN, CWH, CX, CXO, CY, CYBR, CYTK, CZR, D, DAL, DAR, DASH, DATA, DAY, DB, DBX, DD, DDD, DDOG, DE, DECK, DELL, DEN, DEO, DFS, DG, DGX, DHI, DHR, DINO, DIS, DISCA, DISCK, DISH, DJT, DK, DKNG, DKS, DLPH, DLR, DLTR, DNKN, DOC, DOCN, DOCU, DOV, DOW, DPS, DPZ, DQ, DRE, DRI, DT, DTE, DUK, DUOL, DVA, DVMT, DVN, DWDP, DXC, DXCM, EA, EAT, EBAY, ECA, ECHO, ECL, ED, EDIT, EDU, EFX, EG, EGN, EH, EIX, EL, ELAN, ELF, ELLI, ELV, EME, EMN, EMR, ENB, ENDP, ENPH, ENTG, ENVX, EOG, EOSE, EPAM, EPD, EQH, EQIX, EQNR, EQR, EQT, ERI, ERIC, ES, ESRX, ESS, ESTC, ESV, ET, ETFC, ETH, ETHE, ETN, ETP, ETR, ETSY, EVHC, EVRG, EW, EXAS, EXC, EXE, EXEL, EXPD, EXPE, EXR, F, FANG, FAST, FBIN, FCAU, FCEL, FCNCA, FCX, FDC, FDS, FDX, FE, FERG, FEYE, FFIV, FHN, FI, FICO, FIG, FIRY, FIS, FISV, FIT, FITB, FIVE, FIVN, FIX, FL, FLEX, FLG, FLGT, FLNA, FLR, FLS, FLT, FLUT, FLY, FMC, FN, FND, FNF, FNSR, FNV, FORM, FOUR, FOX, FOXA, FRC, FSLR, FSLY, FSR, FTAI, FTCH, FTI, FTNT, FTR, FTV, FUBO, FUTU, FVRR, FWONK, FXE, GAP, GBT, GBTC, GD, GDDY, GDS, GE, GEHC, GEN, GEV, GEVO, GFI, GFS, GG, GGB, GGP, GH, GILD, GIS, GL, GLDM, GLOB, GLW, GM, GME, GNRC, GOOG, GOOGL, GOOS, GOTU, GPC, GPN, GRAB, GRMN, GRWG, GS, GSK, GT, GTLB, GTLS, GTM, GWPH, GWR, GWRE, GWW, GXP, HAIN, HAL, HALO, HAR, HAS, HBAN, HBI, HCA, HCP, HD, HDB, HDS, HES, HFC, HIG, HII, HIMS, HL, HLF, HLN, HLT, HOG, HOLX, HON, HOOD, HP, HPE, HPQ, HR, HRB, HRC, HRL, HRS, HSBC, HSIC, HST, HSY, HTHT, HUBB, HUBS, HUM, HUN, HUT, HWM, HYLN, HZNP, IAC, IAU, IAUM, IBKR, IBM, IBN, ICE, ICLR, IDEX, IDTI, IDXX, IEX, IFF, ILMN, IMGN, IMMU, INCY, INFY, INMD, INO, INSM, INSP, INTC, INTU, INVH, INXN, IONQ, IONS, IOT, IP, IPG, IPGP, IPHI, IQ, IQV, IR, IRBT, IREN, IRM, ISEE, ISRG, IT, ITCI, ITUB, ITW, IVZ, J, JAZZ, JBHT, JBL, JBLU, JCI, JCP, JD, JEC, JHX, JKHY, JKS, JMIA, JNJ, JNPR, JOBY, JOYY, JPM, JUNO, JWN, K, KATE, KDP, KEY, KEYS, KGC, KHC, KIM, KITE, KKR, KL, KLAC, KMB, KMI, KMX, KNX, KO, KODK, KOLD, KORS, KR, KRTX, KSS, KSU, KTOS, KVUE, LAC, LAD, LB, LBRDK, LBTYA, LBTYK, LCID, LDOS, LEA, LEN, LEU, LGND, LH, LHX, LI, LII, LIN, LITE, LKQ, LLL, LLTC, LLY, LM, LMND, LMT, LNC, LNG, LNT, LOGC, LOGM, LOW, LOXO, LPLA, LPT, LPX, LRCX, LSCC, LSI, LSPD, LTHM, LULU, LUMN, LUNR, LUV, LVGO, LVLT, LVS, LW, LYB, LYFT, LYV, M, MA, MAA, MAC, MAN, MAR, MARA, MAS, MASI, MAT, MBLY, MCD, MCHP, MCK, MCO, MDB, MDCO, MDGL, MDLZ, MDSO, MDT, MELI, MENT, MET, META, MGA, MGM, MGNI, MHK, MIDD, MIME, MJN, MKC, MKSI, MKTX, MLCO, MLM, MLNX, MMC, MMM, MNDY, MNK, MNST, MO, MOD, MOH, MOMO, MON, MOS, MP, MPC, MPLX, MPT, MPW, MPWR, MRK, MRNA, MRO, MRSH, MRVL, MS, MSCC, MSCI, MSFT, MSI, MSTR, MT, MTB, MTCH, MTD, MTN, MTSI, MTZ, MU, MULE, MUR, MVIS, MXIM, MXL, MYL, MYOK, NBIS, NBIX, NBL, NBR, NCLH, NDAQ, NEE, NEM, NET, NEWR, NFLX, NI, NIO, NKE, NKLA, NKTR, NLSN, NLY, NNDM, NOC, NOK, NOV, NOW, NRG, NSC, NTAP, NTES, NTLA, NTNX, NTR, NTRA, NTRS, NU, NUAN, NUE, NVAX, NVCR, NVDA, NVMI, NVO, NVR, NVS, NVT, NVTA, NVTS, NWL, NXH, NXPI, NXT, O, OA, OC, OCGN, OCLR, ODFL, OKE, OKLO, OKTA, OLED, OLLI, OMC, ON, ONCE, ONDS, ONON, ONTO, OPEN, OPTU, ORCL, ORLY, OSCR, OSH, OTIS, OVV, OWL, OXY, P, PAA, PAAS, PACB, PACW, PAGS, PANW, PATH, PAYC, PAYX, PBCT, PBF, PBR, PBR.A, PBYI, PCAR, PCG, PCOR, PCTY, PDD, PE, PEG, PEN, PENN, PEP, PF, PFE, PFG, PFPT, PG, PGR, PH, PHM, PHYS, PII, PINS, PKG, PL, PLAN, PLD, PLNT, PLTR, PLUG, PM, PNC, PNR, PNRA, PNW, PODD, POET, POOL, POT, PPG, PPL, PPLI, PR, PRGO, PRMB, PRU, PRXL, PSA, PSLV, PSTG, PSX, PTC, PTEN, PTON, PVH, PWR, PX, PXD, PYPL, Q, QBTS, QCOM, QDEL, QFIN, QGEN, QRVO, QS, QSR, QTS, QUBT, QVCA, QXO, RACE, RAI, RBA, RBLX, RBRK, RCL, RDDT, RDFN, RDS.A, RDS.B, RDW, REG, REGN, RETA, RF, RGEN, RGLD, RGTI, RH, RHI, RHT, RICE, RIG, RIO, RIOT, RIVN, RJF, RKLB, RKT, RL, RMBS, RMD, RNA, RNG, ROIV, ROK, ROKU, ROL, ROP, ROST, RP, RR, RRC, RRX, RS, RSG, RSPP, RTN, RTX, RUN, RVMD, RVTY, RXDX, RXRX, RY, S, SABR, SAGE, SAIA, SAIL, SAM, SAP, SATS, SAVA, SAVE, SBAC, SBE, SBET, SBNY, SBUX, SCCO, SCG, SCHW, SCO, SE, SEDG, SEE, SERV, SFIX, SFM, SGEN, SGI, SGOL, SHAK, SHEL, SHLL, SHOP, SHPG, SHW, SI, SIG, SIGA, SIMO, SINA, SIRI, SITM, SIVB, SIVR, SIX, SJM, SKLZ, SKX, SLB, SLCA, SLG, SLW, SM, SMAR, SMCI, SMR, SMTC, SN, SNA, SNAP, SNDK, SNDL, SNI, SNOW, SNPS, SNV, SNY, SO, SOFI, SOLO, SONY, SOS, SOUN, SPCE, SPCX, SPG, SPGI, SPLK, SPOT, SPR, SPRU, SQM, SRC, SRCL, SRE, SRNE, SRPT, SSNC, STE, STJ, STLA, STLD, STM, STNE, STOR, STRL, STT, STX, STZ, SU, SUI, SUM, SW, SWAV, SWK, SWKS, SWN, SWTX, SXL, SYF, SYK, SYMC, SYY, T, TAL, TAP, TCO, TCOM, TD, TDG, TDOC, TDY, TE, TEAM, TECD, TECH, TECK, TEL, TEM, TER, TEVA, TFC, TFX, TGT, THC, THO, TIF, TIGR, TJX, TKO, TLN, TLRY, TMDX, TME, TMO, TMUS, TNDM, TNL, TOL, TOST, TPL, TPR, TREX, TRGP, TRI, TRIP, TROW, TRP, TRU, TRV, TS, TSCO, TSEM, TSLA, TSM, TSN, TSO, TSRO, TSS, TT, TTD, TTE, TTMI, TTWO, TW, TWLO, TWST, TWTR, TWX, TXG, TXN, TXRH, TXT, TYL, U, UA, UAA, UAL, UBER, UCO, UDR, UEC, UGI, UGL, UHS, UL, ULTA, UMC, UNG, UNH, UNM, UNP, UPS, UPST, URBN, URI, USAR, USB, USFD, UTHR, UTX, UUUU, V, VAL, VALE, VALE.P, VAR, VEEV, VER, VFC, VG, VIAB, VIAV, VICI, VICR, VIK, VIPS, VISN, VKTX, VLO, VLTO, VMC, VMRK, VMW, VNO, VNTV, VOD, VOYA, VRNA, VRSK, VRSN, VRT, VRTX, VRX, VSH, VST, VTR, VTRS, VUZI, VXRT, VZ, W, WAB, WAL, WAT, WB, WBA, WBC, WBD, WBS, WCC, WCG, WCN, WDAY, WDC, WEC, WELL, WEX, WFC, WFM, WFRD, WFT, WHR, WING, WISH, WIX, WKHS, WLK, WLL, WM, WMB, WMGI, WMT, WNR, WOLF, WORK, WP, WPC, WPM, WPX, WRB, WRK, WSM, WSO, WST, WTW, WU, WUBA, WULF, WWAV, WWD, WWE, WY, WYN, WYNN, XEC, XEL, XL, XLNX, XNET, XOM, XP, XPEV, XPO, XRAY, XRX, XYL, XYZ, Y, YELP, YHOO, YUM, YUMC, Z, ZAYO, ZBH, ZBRA, ZEN, ZETA, ZG, ZIM, ZION, ZM, ZNGA, ZS, ZSL, ZTO, ZTS (daily bars from 1 sources); each symbol at most one position, book slots 50.; Data: `C:\data\trading\trading_framework\data\cache\daily_10years_tr`; sessions 2016-01-04 .. 2026-10-23; evaluation starts 2016-12-29.. Full grid: 33 settings including baseline, 66 variant/fill rows; primary fill `next_open`. DEV-only selection `mom126_skip10`; baseline DEV/TEST skill -0.070/+0.342; selected DEV/TEST skill +0.021/+0.168. Selected TEST Sharpe +0.697, PF +1.355, win +52.0%, trades 1665. no selected-result evidence or high-metric flag. Variant-search bias is not included; TEST has been inspected.
  | portfolio | variant | partition | total log | trend log | random-pick baselines (log; p) | skill (log) | skill vs random U / pool (log) | bias (log) | skill % | skill p | Sharpe | PF | max DD | win % | trades |
  |---|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | `baseline` | DEV | +0.656 | +0.726 | U +0.469 (p=+0.063); pool +0.465 (p=+0.055) | -0.070 | U +0.187; pool +0.191 | +0.000 | n/a | +0.546 | +0.460 | +1.222 | -46.2% | +51.7% | 4063 |
  | baseline | `baseline` | TEST | +0.735 | +0.394 | U +0.303 (p=+0.001); pool +0.301 (p=+0.001) | +0.342 | U +0.432; pool +0.435 | +0.000 | +46.5% | +0.263 | +0.820 | +1.441 | -38.5% | +51.5% | 1666 |
  | DEV-selected | `mom126_skip10` | DEV | +0.766 | +0.745 | U +0.491 (p=+0.017); pool +0.496 (p=+0.012) | +0.021 | U +0.275; pool +0.270 | +0.000 | +2.7% | +0.486 | +0.526 | +1.252 | -39.1% | +51.9% | 4114 |
  | DEV-selected | `mom126_skip10` | TEST | +0.561 | +0.393 | U +0.309 (p=+0.002); pool +0.304 (p=+0.006) | +0.168 | U +0.252; pool +0.257 | +0.000 | +30.0% | +0.364 | +0.697 | +1.355 | -38.9% | +52.0% | 1665 |
  Reproduce: full matrix `cargo run --release -- --strategy s041_mom_12_1 --variant indneutral,top10_ind,mom126_skip0,mom126_skip10,mom126_skip21,mom126_skip42,mom189_skip0,mom189_skip10,mom189_skip21,mom189_skip42,mom252_skip0,mom252_skip10,mom252_skip42,mom315_skip0,mom315_skip10,mom315_skip21,mom315_skip42,mom378_skip0,mom378_skip10,mom378_skip21,mom378_skip42,top10,top20,top30,top40,top50,top75,top100,top150,top200,decile5,decile15`; DEV-selected winner `cargo run --release -- --strategy s041_mom_12_1 --variant mom126_skip10`.
### S042 | Sector Momentum Rotation
- family: momentum and breakout
- concept: Momentum works at the sector level with less idiosyncratic noise: hold the 3 strongest of 10 US equity sectors by 12-month momentum, rebalance monthly.
- universe: SPDR sector ETFs from the PIT sector universe table (9/10/11 ETFs by date, XLK/XLC reconstitution noted)
- direction: long
- indicators: 12-month trailing return rank
- entry: At month-end, equal-weight the top 3 sector ETFs by 12-month total return (primary)
- exit: Rebalance monthly; replace any sector that exits the top 3
- hold: ~21 sessions
- frequency: monthly rebalance
- provenance: source-specified
- status: active
- rust_status: IMPLEMENTED
- priority: P2
- sources: [Quantpedia — Sector Momentum rotational system](https://quantpedia.com/strategies/sector-momentum-rotational-system)
- notes: Quantpedia: ten sector ETFs, top 3 by 12-month momentum, monthly; its backtests (Faber, French-Fama sector data back to the 1920s) 'explicitly exclude taxes, commissions and slippage' and quote 100-400% annual turnover; the 1/3/6/9/12 grid with breadth 1-9 is dozens of trials in the source itself. Faber's data were French sectors, not ETFs. Benchmarks: equal-weight sector basket and SPY. Costs per the shared spec. S048 is registered as a variant of this entry.
- parameter_variants: signal {12-month primary; rank-average and 1/3/6/9/12 blend}; breadth {top 1/2/3 primary/5/7/9}; rebalance {monthly primary; weekly}.
- data_needs: sector ETF total-return daily series (DP-ETF) + PIT sector universe table
- rust_implementation: Registered as `s042_sector_momentum` with daily ETF momentum scores, monthly top-k rotation, and named signal/breadth variants.
- implementation_delta: DELTA — next-open is the executable primary fill; same-close is an upper bound. The cached implementation uses 11 sector ETFs with total-return history from 2016; XLC becomes eligible only after its available/listing history.
- review: KEEP | source OK
- implementation_results: Rust in `src/family4.rs` / `src/study.rs`, config `configs/s042_sector_momentum.yaml`; Universe: XLB, XLC, XLE, XLF, XLI, XLK, XLP, XLRE, XLU, XLV, XLY (daily bars from 1 sources); each symbol at most one position, book slots 3.; Data: `C:\data\trading\trading_framework\data\cache\etf_tr`; sessions 2016-01-04 .. 2026-10-23; evaluation starts 2016-12-30.. Full grid: 31 settings including baseline, 62 variant/fill rows; primary fill `next_open`. DEV-only selection `top1`; baseline DEV/TEST skill +0.005/+0.057; selected DEV/TEST skill +0.633/-0.254. Selected TEST Sharpe +0.355, PF +1.334, win +50.0%, trades 34. insufficient trade-count evidence; DECAYED: skill sign changes; PF or Sharpe > 2: leakage and timing review required. Variant-search bias is not included; TEST has been inspected.
  | portfolio | variant | partition | total log | trend log | random-pick baselines (log; p) | skill (log) | skill vs random U / pool (log) | bias (log) | skill % | skill p | Sharpe | PF | max DD | win % | trades |
  |---|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | `baseline` | DEV | +0.760 | +0.755 | U +0.647 (p=+0.270); pool +0.653 (p=+0.275) | +0.005 | U +0.113; pool +0.107 | +0.000 | +0.7% | +0.493 | +0.657 | +1.629 | -31.7% | +61.8% | 249 |
  | baseline | `baseline` | TEST | +0.456 | +0.399 | U +0.361 (p=+0.217); pool +0.356 (p=+0.201) | +0.057 | U +0.095; pool +0.100 | +0.000 | +12.5% | +0.345 | +1.190 | +2.021 | -14.8% | +56.9% | 102 |
  | DEV-selected | `top1` | DEV | +1.388 | +0.755 | U +0.615 (p=+0.014); pool +0.594 (p=+0.006) | +0.633 | U +0.773; pool +0.793 | +0.000 | +45.6% | +0.086 | +0.908 | +2.139 | -31.2% | +66.3% | 83 |
  | DEV-selected | `top1` | TEST | +0.145 | +0.399 | U +0.346 (p=+0.834); pool +0.335 (p=+0.786) | -0.254 | U -0.201; pool -0.190 | +0.000 | n/a | +0.799 | +0.355 | +1.334 | -25.8% | +50.0% | 34 |
  Reproduce: full matrix `cargo run --release -- --strategy s042_sector_momentum --variant rankavg,blend13,top1,top2,top5,top7,top9,weekly,rankavg_top1,rankavg_top2,rankavg_top5,rankavg_top7,rankavg_top9,rankavg_weekly,blend13_top1,blend13_top2,blend13_top5,blend13_top7,blend13_top9,blend13_weekly,mom12_top1_weekly,mom12_top2_weekly,mom12_top5_weekly,mom12_top7_weekly,mom12_top9_weekly,rankavg_top1_weekly,rankavg_top2_weekly,rankavg_top5_weekly,rankavg_top7_weekly,rankavg_top9_weekly`; DEV-selected winner `cargo run --release -- --strategy s042_sector_momentum --variant top1`.
### S043 | ETF Multi-Asset Momentum Rotation
- family: momentum and breakout
- concept: Rank a fixed multi-asset ETF universe by 12-month return; hold the top funds; rotate to short-term Treasuries when an ETF's 12-month return is negative (absolute momentum overlay).
- universe: one ETF per asset class (primary): SPY, EFA, EEM, IWM or QQQ (single extra equity), TLT or IEF, LQD or HYG, GLD, GSG/DBC, IYR; SHY as cash floor. The earlier 13-ETF list had five US-equity-beta ETFs (SPY, QQQ, IWM, XLK, XLE) and was mostly equity rotation
- direction: long
- indicators: 12-month total return rank (primary); close/SMA(200) - 1 (variant); SHY/BIL total return as cash floor
- entry: At month-end, hold the top 5 ETFs by 12-month total return, equal-weight; replace any pick with a negative 12-month return by SHY
- exit: Rebalance monthly
- hold: ~21 sessions
- frequency: monthly rebalance
- provenance: adapted
- status: active
- priority: P2
- sources: [PortfoliosLab - Momentum & sector rotation strategies](https://portfolioslab.com/strategies/momentum-rotation) (describes sector/industry/factor rotation only; the top-ranked-funds/Treasury-replacement mechanics are supported, the 13-ETF multi-asset universe and Antonacci framing are the catalog's); CXO SACEMS and GEM are the established multi-asset comparators; phuazz breadth-thrust-etf (sleeve B, for the MA-distance signal)
- notes: Benchmarks: equal-weight of the same ETFs and 60/40; SHY must be total return.
- parameter_variants: tested grid: momentum lookback {63, 126, 189, 252 primary, 315, 378, 504 days}; breadth {top 1/2/3/5 primary/7}. The merged S068 SMA-distance and S074 weekly SPY/QQQ modes remain unimplemented.
- merged_from: S068, S074
- data_needs: ETF total-return daily series (DP-ETF)
- rust_status: IMPLEMENTED
- implementation_delta: DELTA — next-open is the executable primary fill; same-close is an upper bound. The registered baseline uses the 10-ETF asset-class universe, with ETF total-return history from 2016; the 60/40 comparator and merged S068/S074 modes are not implemented; SPY benchmark outperformance uses close-to-close returns, not fill-timing-matched to next-open execution.
- implementation_results: Rust in `src/family4.rs` / `src/study.rs`, config `configs/s043_etf_multi_asset_momentum.yaml`; Universe: EEM, EFA, GLD, GSG, IWM, IYR, LQD, SHY, SPY, TLT (daily bars from 1 sources); each symbol at most one position, book slots 5.; Data: `C:\data\trading\trading_framework\data\cache\etf_tr`; sessions 2016-01-04 .. 2026-10-23; evaluation starts 2016-01-04.. Full grid: 31 settings including baseline, 62 variant/fill rows; primary fill `next_open`. DEV-only selection `mom189_top2`; baseline DEV/TEST skill -0.081/+0.075; selected DEV/TEST skill +0.283/+0.127. Selected TEST Sharpe +1.116, PF +2.170, win +63.2%, trades 68. PF or Sharpe > 2: leakage and timing review required. Variant-search bias is not included; TEST has been inspected.
  | portfolio | variant | partition | total log | trend log | random-pick baselines (log; p) | skill (log) | skill vs random U / pool (log) | bias (log) | skill % | skill p | Sharpe | PF | max DD | win % | trades |
  |---|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | `baseline` | DEV | +0.324 | +0.405 | U +0.373 (p=+0.623); pool +0.380 (p=+0.633) | -0.081 | U -0.049; pool -0.056 | +0.000 | n/a | +0.675 | +0.448 | +1.314 | -21.6% | +57.4% | 376 |
  | baseline | `baseline` | TEST | +0.422 | +0.347 | U +0.342 (p=+0.189); pool +0.337 (p=+0.159) | +0.075 | U +0.080; pool +0.085 | +0.000 | +17.7% | +0.209 | +1.175 | +2.100 | -12.1% | +62.4% | 170 |
  | DEV-selected | `mom189_top2` | DEV | +0.713 | +0.430 | U +0.358 (p=+0.072); pool +0.367 (p=+0.090) | +0.283 | U +0.355; pool +0.346 | +0.000 | +39.7% | +0.204 | +0.730 | +1.692 | -21.8% | +56.5% | 170 |
  | DEV-selected | `mom189_top2` | TEST | +0.474 | +0.347 | U +0.334 (p=+0.160); pool +0.331 (p=+0.146) | +0.127 | U +0.140; pool +0.144 | +0.000 | +26.7% | +0.244 | +1.116 | +2.170 | -22.8% | +63.2% | 68 |
  Reproduce: full matrix `cargo run --release -- --strategy s043_etf_multi_asset_momentum --variant mom63_top1,mom63_top2,mom63_top3,mom63_top5,mom63_top7,mom126_top1,mom126_top2,mom126_top3,mom126_top5,mom126_top7,mom189_top1,mom189_top2,mom189_top3,mom189_top5,mom189_top7,mom252_top1,mom252_top2,mom252_top3,mom252_top7,mom315_top1,mom315_top2,mom315_top3,mom315_top5,mom315_top7,mom378_top1,mom378_top2,mom378_top3,mom378_top5,mom378_top7,mom504_top5`; DEV-selected winner `cargo run --release -- --strategy s043_etf_multi_asset_momentum --variant mom189_top2`.
- review: RESOLVED

### S044 | Efficiente 5 MPT Replication
- family: momentum and breakout
- concept: JPMorgan's rules-based tactical ETF strategy: monthly, compute 6-month (126-day) mean returns and covariance of 13 ETFs, select the efficient portfolio targeting 5% volatility, subject to per-asset and asset-class caps and 5%-multiple weights. Short lookback makes it a momentum-flavored MPT rotation.
- universe: SPY, IWM, EFA, TLT, LQD, HYG, EEM, EMB, IYR, GSG, GLD, TIP, BIL
- direction: long
- indicators: 126-day mean return vector; 126-day covariance matrix; 5% volatility target; asset-class caps (equity 50%, bonds 50%, EM 25%, alternatives 25%, TIP/cash 50%)
- entry: At each month-end, solve the constrained optimization: maximize return s.t. portfolio vol = 5% (escalate the target by 1% steps if infeasible; else minimum variance), weights in 5% multiples, per-asset caps (SPY 20, IWM 10, EFA 20, TLT 20, LQD 20, HYG 20, EEM 20, EMB 20, IYR 20, GSG 10, GLD 10, TIP 50, BIL 50) and class caps (developed equity 50%, bonds 50%, EM 25%, alternatives 25%); implement as a continuous solve followed by constrained rounding/local search (an integer program, not scipy-plus-rounding); portfolio is an excess return over BIL (funded by a short cash position)
- exit: Rebalance monthly to the new solution
- hold: ~21 sessions
- frequency: monthly rebalance
- provenance: source-specified
- status: active
- priority: P1
- sources: Attached paper (user upload) — Institutional Asset Allocation meets the Efficient Frontier (JPM Efficiente 5 replication)] — attached PDF (user upload), no public URL verified
- notes: Verified against the attached PDF (Jungle Rock Capital, Oct 2025): universe and caps match Table 1; monthly rebalance, 126-day average return and covariance. Published acceptance targets (replication Aug-2008 to Aug-2025, Yahoo data): average excess return 3.49% vs 2.89% actual, vol 6.23%, Sharpe 0.58 (actual 0.50), Sortino 0.79, max drawdown -19.07% (actual -16.56%), average monthly turnover 42.48%, correlation with the real index 0.94, tracking error 1.48%. Table 3 (5% target): weekly rebalance with 6-month lookback Sharpe 0.52, monthly 6-month 0.61, monthly 12-month 0.55; at a 10% target the caps bind and realised vol stays ~8%. Sharpe 0.5-0.6 is the PUBLISHED performance (a Sharpe > 2 gate would reject it). Acceptance on the framework's 2016+ data: constraints satisfied every month, subperiod stats consistent with Table 2, qualitative average-weights-by-year pattern; full numeric replication only in 'replication' mode on 2008-2025 Yahoo data. Monthly turnover of 42% x 5-10 bps is a ~2-5%/yr drag: report net. Use as the first end-to-end golden of the slow-sleeve pipeline; unit tests for the cap constraints and the 1%-step escalation. The '(pypfopt)' remark is from the paper's footnote 3, not a requirement.
- parameter_variants: tested grid: lookback {63, 84, 126 primary, 189, 252, 315 days}; target volatility {5% primary, 6%, 7%, 8%, 10%}. Published 2008-2025 replication mode remains blocked by unavailable historical ETF data.
- data_needs: 13 ETFs with dividends from 2008 for replication (Yahoo/Stooq one-off); DP-ETF for the framework window; BIL (FRED T-bill before 2007)
- rust_status: IMPLEMENTED
- implementation_delta: DELTA — next-open is the executable primary fill; same-close is an upper bound. The optimizer exhaustively scans the validated discrete 5%-unit grid (23,816,422 feasible rows), rather than continuous optimization plus rounding. It is long-only, holds BIL as an asset, and does not short BIL. Cached ETF history starts in 2016; Yahoo returned HTTP 429 and Stooq served a verification challenge, so 2008–2025 replication remains unavailable.
- implementation_results: Rust in `src/family4.rs` / `src/study.rs`, config `configs/s044_efficiente5.yaml`; Universe: BIL, EEM, EFA, EMB, GLD, GSG, HYG, IWM, IYR, LQD, SPY, TIP, TLT (daily bars from 1 sources); each symbol at most one position, book slots 1.; Data: `C:\data\trading\trading_framework\data\cache\etf_tr`; sessions 2016-01-04 .. 2026-10-23; evaluation starts 2016-01-04.. Full grid: 30 settings including baseline, 60 variant/fill rows; primary fill `next_open`. DEV-only selection `lb189_t10`; baseline DEV/TEST skill -0.147/-0.095; selected DEV/TEST skill -0.041/+0.026. Selected TEST Sharpe +1.223, PF +2.216, win +67.8%, trades 245. DECAYED: skill sign changes; PF or Sharpe > 2: leakage and timing review required. Variant-search bias is not included; TEST has been inspected.
  | portfolio | variant | partition | total log | trend log | random-pick baselines (log; p) | skill (log) | skill vs random U / pool (log) | bias (log) | skill % | skill p | Sharpe | PF | max DD | win % | trades |
  |---|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | `baseline` | DEV | +0.215 | +0.362 | U +0.326 (p=+0.751); pool +0.319 (p=+0.727) | -0.147 | U -0.111; pool -0.105 | +0.000 | n/a | +0.800 | +0.450 | +1.290 | -16.3% | +56.2% | 521 |
  | baseline | `baseline` | TEST | +0.207 | +0.302 | U +0.296 (p=+0.841); pool +0.283 (p=+0.805) | -0.095 | U -0.089; pool -0.076 | +0.000 | n/a | +0.900 | +1.350 | +1.884 | -4.9% | +67.3% | 196 |
  | DEV-selected | `lb189_t10` | DEV | +0.335 | +0.376 | U +0.339 (p=+0.513); pool +0.347 (p=+0.537) | -0.041 | U -0.004; pool -0.012 | +0.000 | n/a | +0.599 | +0.563 | +1.361 | -17.9% | +57.8% | 512 |
  | DEV-selected | `lb189_t10` | TEST | +0.328 | +0.302 | U +0.291 (p=+0.319); pool +0.294 (p=+0.324) | +0.026 | U +0.037; pool +0.035 | +0.000 | +8.0% | +0.343 | +1.223 | +2.216 | -9.1% | +67.8% | 245 |
  Reproduce: full matrix `cargo run --release -- --strategy s044_efficiente5 --variant target10,lookback252,lb63_t5,lb63_t6,lb63_t7,lb63_t8,lb63_t10,lb84_t5,lb84_t6,lb84_t7,lb84_t8,lb84_t10,lb126_t6,lb126_t7,lb126_t8,lb189_t5,lb189_t6,lb189_t7,lb189_t8,lb189_t10,lb252_t5,lb252_t6,lb252_t7,lb252_t8,lb315_t5,lb315_t6,lb315_t7,lb315_t8,lb315_t10`; DEV-selected winner `cargo run --release -- --strategy s044_efficiente5 --variant lb189_t10`.
- review: KEEP (pipeline replication test) | source OK (verified against attached PDF)

### S045 | Multi-Factor Characteristic Blend (US Adaptation; S045a paper design, S045b catalog design)
- family: momentum and breakout
- concept: The attached Borsa Istanbul paper blends 5 characteristics — low volatility, value (5-year reversal proxy), 52-week-high proximity, 1-month reversal, and quality — into a 10-stock portfolio with volatility targeting and a trend gate. Adapting the price-only characteristics to US stocks is a direct implementation of the paper's design minus the fundamentals.
- universe: top-500 liquid US stocks
- direction: long
- indicators: S045a: composite of z-scores of low volatility (252-day), value (5-year reversal proxy), 52-week-high proximity, 1-month reversal (quality omitted: fundamentals); 60-day realized vol for vol targeting; SMA(200) and vol quintile for the regime gate. S045b: 60-day volatility rank; close/252-day-high rank; 21-day reversal rank; SMA(200) market gate
- entry: S045a (the paper's design): each month, composite-rank the universe on the price characteristics, take the top 20 by composite, then the lower-volatility half = 10 names; rank buffer (incumbents kept while in the top 30); scale gross exposure to a 20% vol target from 60-day realised vol, capped at 100%; three-state trend/regime gate with multipliers 1.00 / 0.70 / 0.40 (SPY vs its 200-day average and vol quintile); 25 bps costs. S045b (the catalog's simplified version): composite of the 3 price characteristics, buy the top 10 equal-weight only when SPY > SMA200 (else cash), 20% target vol
- exit: Rebalance monthly
- hold: ~21 sessions
- frequency: monthly rebalance
- provenance: adapted
- status: active
- priority: P2
- sources: Attached paper (user upload) — Multi-Factor Investing in Borsa Istanbul] — attached PDF (user upload), no public URL verified
- notes: The attached paper's own conclusion: Sharpe 1.19 vs 0.89 for the cap-weighted index but alpha -0.4%/yr (95% CI [-6.3%, +5.5%]) against a vol-matched equal-weight universe; levels are 'in-sample-flattered'; ~57% monthly turnover. The vol-matched equal-weight benchmark is a REQUIRED output column; interpret any result below benchmark + 0.15 Sharpe as 'no selection skill'. S045b deviates from the paper (drops value, replaces the three-state gate with a binary one, drops the buffer and the low-vol tilt, uses top-10 directly); the old 4-indicator cap is no longer a constraint. The Efficiente 5 paper is NOT a source for this entry.
- parameter_variants: design {S045a paper design primary; S045b catalog simplification}; universe {top 250/500 primary/750/1,000}; target volatility {10%, 15%, 20% baseline, 25%, 30% for S045a; 10%, 15%, 20% baseline, 25% for S045b}.
- data_needs: daily_10years (D1) + SPY (DP-ETF)
- rust_status: IMPLEMENTED
- implementation_delta: DELTA — next-open is the executable primary fill; same-close is an upper bound. S045a uses four price-only factors (quality fundamentals omitted), 60-day realized-vol targeting capped at 1x, an incumbent buffer, and the 1.00/0.70/0.40 market/vol gate; S045b is also registered. Cached equity history starts in 2016.
- implementation_results: Rust in `src/family4.rs` / `src/study.rs`, config `configs/s045_characteristic_blend.yaml`; Universe: A, AA, AABA, AAL, AAOI, AAP, AAPL, ABBV, ABCM, ABEV, ABMD, ABNB, ABT, ACB, ACC, ACGL, ACHR, ACIA, ACN, ADBE, ADI, ADM, ADP, ADSK, ADT, AEE, AEHR, AEIS, AEM, AEO, AEP, AER, AES, AET, AFL, AFRM, AG, AGN, AGNC, AGQ, AHT, AI, AIG, AIRC, AIZ, AJG, AKAM, AKRX, AKS, ALAB, ALB, ALC, ALGN, ALK, ALL, ALLE, ALLY, ALNY, ALXN, AMAT, AMBA, AMC, AMCR, AMD, AME, AMG, AMGN, AMKR, AMP, AMRN, AMT, AMTD, AMZN, AN, ANDV, ANET, ANF, ANSS, AON, AOS, APA, APC, APD, APH, APHA, APLD, APLS, APO, APP, APPN, APPS, APTV, AR, ARCH, ARCT, ARE, ARES, ARGX, ARM, ARMK, ARNC, ARRS, ARRY, ARWR, ASAN, ASML, ASST, ASTS, ASX, ATHM, ATHN, ATI, ATO, ATUS, ATVI, AU, AUY, AVAV, AVB, AVGO, AVLR, AVTR, AVY, AWK, AXON, AXP, AXTA, AXTI, AYI, AYX, AZEK, AZN, AZO, B, BA, BABA, BAC, BAH, BALL, BAM, BAP, BAX, BB, BBAI, BBBY, BBD, BBIG, BBL, BBWI, BBY, BCR, BCS, BDX, BE, BEAV, BEKE, BEN, BF.B, BFH, BG, BHC, BHI, BHP, BHVN, BIDU, BIIB, BILI, BILL, BIO, BIOR, BIVV, BJ, BK, BKI, BKNG, BKR, BLD, BLDP, BLDR, BLK, BLNK, BLUE, BMNR, BMRN, BMY, BN, BNGO, BNO, BNTX, BNY, BOIL, BOX, BP, BPL, BPMC, BR, BRCD, BRK.A, BRK.B, BRO, BROS, BSX, BTBT, BTDR, BTG, BTI, BTU, BUD, BULL, BURL, BWA, BWLD, BWXT, BX, BXP, BYND, BZUN, C, CAA, CACI, CAG, CAH, CAN, CAR, CARR, CART, CASY, CAT, CAVA, CAVM, CB, CBI, CBOE, CBRE, CBS, CC, CCEP, CCI, CCJ, CCK, CCL, CDE, CDNS, CDW, CE, CEG, CEI, CELG, CELH, CERN, CF, CFG, CFLT, CG, CGC, CGNX, CHD, CHGG, CHKP, CHPT, CHRD, CHRW, CHTR, CHWY, CI, CIEN, CIFR, CINF, CIT, CL, CLF, CLOV, CLR, CLS, CLSK, CLVS, CLX, CM, CMA, CMCSA, CME, CMG, CMI, CMS, CNC, CNH, CNI, CNK, CNP, CNQ, CNX, COF, COH, COHR, COIN, COL, COMM, CONE, COO, COP, COR, CORZ, COST, COTY, COUP, CP, CPAY, CPB, CPN, CPNG, CPRI, CPRT, CPT, CRCL, CRDO, CREE, CRH, CRI, CRL, CRM, CRON, CROX, CRS, CRSP, CRUS, CRWD, CRWV, CSC, CSCO, CSGP, CSIQ, CSL, CSX, CTAS, CTL, CTLT, CTRA, CTRP, CTSH, CTVA, CTXS, CVE, CVNA, CVS, CVX, CW, CWAN, CWH, CX, CXO, CY, CYBR, CYTK, CZR, D, DAL, DAR, DASH, DATA, DAY, DB, DBX, DD, DDD, DDOG, DE, DECK, DELL, DEN, DEO, DFS, DG, DGX, DHI, DHR, DINO, DIS, DISCA, DISCK, DISH, DJT, DK, DKNG, DKS, DLPH, DLR, DLTR, DNKN, DOC, DOCN, DOCU, DOV, DOW, DPS, DPZ, DQ, DRE, DRI, DT, DTE, DUK, DUOL, DVA, DVMT, DVN, DWDP, DXC, DXCM, EA, EAT, EBAY, ECA, ECHO, ECL, ED, EDIT, EDU, EFX, EG, EGN, EH, EIX, EL, ELAN, ELF, ELLI, ELV, EME, EMN, EMR, ENB, ENDP, ENPH, ENTG, ENVX, EOG, EOSE, EPAM, EPD, EQH, EQIX, EQNR, EQR, EQT, ERI, ERIC, ES, ESRX, ESS, ESTC, ESV, ET, ETFC, ETH, ETHE, ETN, ETP, ETR, ETSY, EVHC, EVRG, EW, EXAS, EXC, EXE, EXEL, EXPD, EXPE, EXR, F, FANG, FAST, FBIN, FCAU, FCEL, FCNCA, FCX, FDC, FDS, FDX, FE, FERG, FEYE, FFIV, FHN, FI, FICO, FIG, FIRY, FIS, FISV, FIT, FITB, FIVE, FIVN, FIX, FL, FLEX, FLG, FLGT, FLNA, FLR, FLS, FLT, FLUT, FLY, FMC, FN, FND, FNF, FNSR, FNV, FORM, FOUR, FOX, FOXA, FRC, FSLR, FSLY, FSR, FTAI, FTCH, FTI, FTNT, FTR, FTV, FUBO, FUTU, FVRR, FWONK, FXE, GAP, GBT, GBTC, GD, GDDY, GDS, GE, GEHC, GEN, GEV, GEVO, GFI, GFS, GG, GGB, GGP, GH, GILD, GIS, GL, GLDM, GLOB, GLW, GM, GME, GNRC, GOOG, GOOGL, GOOS, GOTU, GPC, GPN, GRAB, GRMN, GRWG, GS, GSK, GT, GTLB, GTLS, GTM, GWPH, GWR, GWRE, GWW, GXP, HAIN, HAL, HALO, HAR, HAS, HBAN, HBI, HCA, HCP, HD, HDB, HDS, HES, HFC, HIG, HII, HIMS, HL, HLF, HLN, HLT, HOG, HOLX, HON, HOOD, HP, HPE, HPQ, HR, HRB, HRC, HRL, HRS, HSBC, HSIC, HST, HSY, HTHT, HUBB, HUBS, HUM, HUN, HUT, HWM, HYLN, HZNP, IAC, IAU, IAUM, IBKR, IBM, IBN, ICE, ICLR, IDEX, IDTI, IDXX, IEX, IFF, ILMN, IMGN, IMMU, INCY, INFY, INMD, INO, INSM, INSP, INTC, INTU, INVH, INXN, IONQ, IONS, IOT, IP, IPG, IPGP, IPHI, IQ, IQV, IR, IRBT, IREN, IRM, ISEE, ISRG, IT, ITCI, ITUB, ITW, IVZ, J, JAZZ, JBHT, JBL, JBLU, JCI, JCP, JD, JEC, JHX, JKHY, JKS, JMIA, JNJ, JNPR, JOBY, JOYY, JPM, JUNO, JWN, K, KATE, KDP, KEY, KEYS, KGC, KHC, KIM, KITE, KKR, KL, KLAC, KMB, KMI, KMX, KNX, KO, KODK, KOLD, KORS, KR, KRTX, KSS, KSU, KTOS, KVUE, LAC, LAD, LB, LBRDK, LBTYA, LBTYK, LCID, LDOS, LEA, LEN, LEU, LGND, LH, LHX, LI, LII, LIN, LITE, LKQ, LLL, LLTC, LLY, LM, LMND, LMT, LNC, LNG, LNT, LOGC, LOGM, LOW, LOXO, LPLA, LPT, LPX, LRCX, LSCC, LSI, LSPD, LTHM, LULU, LUMN, LUNR, LUV, LVGO, LVLT, LVS, LW, LYB, LYFT, LYV, M, MA, MAA, MAC, MAN, MAR, MARA, MAS, MASI, MAT, MBLY, MCD, MCHP, MCK, MCO, MDB, MDCO, MDGL, MDLZ, MDSO, MDT, MELI, MENT, MET, META, MGA, MGM, MGNI, MHK, MIDD, MIME, MJN, MKC, MKSI, MKTX, MLCO, MLM, MLNX, MMC, MMM, MNDY, MNK, MNST, MO, MOD, MOH, MOMO, MON, MOS, MP, MPC, MPLX, MPT, MPW, MPWR, MRK, MRNA, MRO, MRSH, MRVL, MS, MSCC, MSCI, MSFT, MSI, MSTR, MT, MTB, MTCH, MTD, MTN, MTSI, MTZ, MU, MULE, MUR, MVIS, MXIM, MXL, MYL, MYOK, NBIS, NBIX, NBL, NBR, NCLH, NDAQ, NEE, NEM, NET, NEWR, NFLX, NI, NIO, NKE, NKLA, NKTR, NLSN, NLY, NNDM, NOC, NOK, NOV, NOW, NRG, NSC, NTAP, NTES, NTLA, NTNX, NTR, NTRA, NTRS, NU, NUAN, NUE, NVAX, NVCR, NVDA, NVMI, NVO, NVR, NVS, NVT, NVTA, NVTS, NWL, NXH, NXPI, NXT, O, OA, OC, OCGN, OCLR, ODFL, OKE, OKLO, OKTA, OLED, OLLI, OMC, ON, ONCE, ONDS, ONON, ONTO, OPEN, OPTU, ORCL, ORLY, OSCR, OSH, OTIS, OVV, OWL, OXY, P, PAA, PAAS, PACB, PACW, PAGS, PANW, PATH, PAYC, PAYX, PBCT, PBF, PBR, PBR.A, PBYI, PCAR, PCG, PCOR, PCTY, PDD, PE, PEG, PEN, PENN, PEP, PF, PFE, PFG, PFPT, PG, PGR, PH, PHM, PHYS, PII, PINS, PKG, PL, PLAN, PLD, PLNT, PLTR, PLUG, PM, PNC, PNR, PNRA, PNW, PODD, POET, POOL, POT, PPG, PPL, PPLI, PR, PRGO, PRMB, PRU, PRXL, PSA, PSLV, PSTG, PSX, PTC, PTEN, PTON, PVH, PWR, PX, PXD, PYPL, Q, QBTS, QCOM, QDEL, QFIN, QGEN, QRVO, QS, QSR, QTS, QUBT, QVCA, QXO, RACE, RAI, RBA, RBLX, RBRK, RCL, RDDT, RDFN, RDS.A, RDS.B, RDW, REG, REGN, RETA, RF, RGEN, RGLD, RGTI, RH, RHI, RHT, RICE, RIG, RIO, RIOT, RIVN, RJF, RKLB, RKT, RL, RMBS, RMD, RNA, RNG, ROIV, ROK, ROKU, ROL, ROP, ROST, RP, RR, RRC, RRX, RS, RSG, RSPP, RTN, RTX, RUN, RVMD, RVTY, RXDX, RXRX, RY, S, SABR, SAGE, SAIA, SAIL, SAM, SAP, SATS, SAVA, SAVE, SBAC, SBE, SBET, SBNY, SBUX, SCCO, SCG, SCHW, SCO, SE, SEDG, SEE, SERV, SFIX, SFM, SGEN, SGI, SGOL, SHAK, SHEL, SHLL, SHOP, SHPG, SHW, SI, SIG, SIGA, SIMO, SINA, SIRI, SITM, SIVB, SIVR, SIX, SJM, SKLZ, SKX, SLB, SLCA, SLG, SLW, SM, SMAR, SMCI, SMR, SMTC, SN, SNA, SNAP, SNDK, SNDL, SNI, SNOW, SNPS, SNV, SNY, SO, SOFI, SOLO, SONY, SOS, SOUN, SPCE, SPCX, SPG, SPGI, SPLK, SPOT, SPR, SPRU, SPY, SQM, SRC, SRCL, SRE, SRNE, SRPT, SSNC, STE, STJ, STLA, STLD, STM, STNE, STOR, STRL, STT, STX, STZ, SU, SUI, SUM, SW, SWAV, SWK, SWKS, SWN, SWTX, SXL, SYF, SYK, SYMC, SYY, T, TAL, TAP, TCO, TCOM, TD, TDG, TDOC, TDY, TE, TEAM, TECD, TECH, TECK, TEL, TEM, TER, TEVA, TFC, TFX, TGT, THC, THO, TIF, TIGR, TJX, TKO, TLN, TLRY, TMDX, TME, TMO, TMUS, TNDM, TNL, TOL, TOST, TPL, TPR, TREX, TRGP, TRI, TRIP, TROW, TRP, TRU, TRV, TS, TSCO, TSEM, TSLA, TSM, TSN, TSO, TSRO, TSS, TT, TTD, TTE, TTMI, TTWO, TW, TWLO, TWST, TWTR, TWX, TXG, TXN, TXRH, TXT, TYL, U, UA, UAA, UAL, UBER, UCO, UDR, UEC, UGI, UGL, UHS, UL, ULTA, UMC, UNG, UNH, UNM, UNP, UPS, UPST, URBN, URI, USAR, USB, USFD, UTHR, UTX, UUUU, V, VAL, VALE, VALE.P, VAR, VEEV, VER, VFC, VG, VIAB, VIAV, VICI, VICR, VIK, VIPS, VISN, VKTX, VLO, VLTO, VMC, VMRK, VMW, VNO, VNTV, VOD, VOYA, VRNA, VRSK, VRSN, VRT, VRTX, VRX, VSH, VST, VTR, VTRS, VUZI, VXRT, VZ, W, WAB, WAL, WAT, WB, WBA, WBC, WBD, WBS, WCC, WCG, WCN, WDAY, WDC, WEC, WELL, WEX, WFC, WFM, WFRD, WFT, WHR, WING, WISH, WIX, WKHS, WLK, WLL, WM, WMB, WMGI, WMT, WNR, WOLF, WORK, WP, WPC, WPM, WPX, WRB, WRK, WSM, WSO, WST, WTW, WU, WUBA, WULF, WWAV, WWD, WWE, WY, WYN, WYNN, XEC, XEL, XL, XLNX, XNET, XOM, XP, XPEV, XPO, XRAY, XRX, XYL, XYZ, Y, YELP, YHOO, YUM, YUMC, Z, ZAYO, ZBH, ZBRA, ZEN, ZETA, ZG, ZIM, ZION, ZM, ZNGA, ZS, ZSL, ZTO, ZTS (daily bars from 2 sources); each symbol at most one position, book slots 10.; Data: `C:\data\trading\trading_framework\data\cache\daily_10years_tr, C:\data\trading\trading_framework\data\cache\etf_tr`; sessions 2016-01-04 .. 2026-10-23; evaluation starts 2016-01-04.. Full grid: 31 settings including baseline, 62 variant/fill rows; primary fill `next_open`. DEV-only selection `a_n750_v30`; baseline DEV/TEST skill -0.183/-0.165; selected DEV/TEST skill -0.133/-0.295. Selected TEST Sharpe +0.528, PF +1.253, win +49.9%, trades 337. no selected-result evidence or high-metric flag. Variant-search bias is not included; TEST has been inspected.
  | portfolio | variant | partition | total log | trend log | random-pick baselines (log; p) | skill (log) | skill vs random U / pool (log) | bias (log) | skill % | skill p | Sharpe | PF | max DD | win % | trades |
  |---|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | `baseline` | DEV | +0.054 | +0.236 | U +0.095 (p=+0.487); pool +0.089 (p=+0.500) | -0.183 | U -0.042; pool -0.035 | +0.000 | n/a | +0.743 | +0.143 | +1.185 | -15.9% | +53.7% | 350 |
  | baseline | `baseline` | TEST | +0.287 | +0.452 | U +0.335 (p=+0.543); pool +0.336 (p=+0.569) | -0.165 | U -0.049; pool -0.049 | +0.000 | n/a | +0.731 | +0.970 | +1.424 | -10.3% | +55.6% | 340 |
  | DEV-selected | `a_n750_v30` | DEV | +0.103 | +0.236 | U +0.102 (p=+0.378); pool +0.100 (p=+0.419) | -0.133 | U +0.001; pool +0.003 | +0.000 | n/a | +0.679 | +0.242 | +1.264 | -13.6% | +54.3% | 350 |
  | DEV-selected | `a_n750_v30` | TEST | +0.153 | +0.448 | U +0.332 (p=+0.751); pool +0.329 (p=+0.727) | -0.295 | U -0.179; pool -0.176 | +0.000 | n/a | +0.864 | +0.528 | +1.253 | -12.5% | +49.9% | 337 |
  Reproduce: full matrix `cargo run --release -- --strategy s045_characteristic_blend --variant paper_design,catalog_simplification,a_n250_v10,a_n250_v15,a_n250_v25,a_n250_v30,a_n500_v10,a_n500_v15,a_n500_v25,a_n500_v30,a_n750_v10,a_n750_v15,a_n750_v25,a_n750_v30,a_n1000_v10,a_n1000_v15,a_n1000_v25,a_n1000_v30,b_n250_v10,b_n250_v15,b_n250_v25,b_n500_v10,b_n500_v15,b_n500_v25,b_n750_v10,b_n750_v15,b_n750_v25,b_n1000_v10,b_n1000_v15,b_n1000_v25`; DEV-selected winner `cargo run --release -- --strategy s045_characteristic_blend --variant a_n750_v30`.
- review: RESOLVED

### S046 | Industry Rotation via Market-State Similarity
- family: momentum and breakout
- concept: Zakamulin: find historical months whose standardized market return is closest to the current month; long industries that historically rose after similar states, short those that fell. Predictability concentrates in the tails (down markets).
- universe: Ken French 49-industry monthly portfolios (free, 1926+; re-verify the URL) for the replication; then sector ETFs (ETF history since 1998 gives ~330 months, so analog counts are a large share of history)
- direction: long and short
- indicators: standardized market excess return; historical analog matching
- entry: At each month-end, find the K closest historical analog months by standardized market return (K = 50 primary; registered {25, 50, 100}); long the top-tercile industries by subsequent historical return, short the bottom tercile (shorting is required for the tail effect the paper cites)
- exit: Rebalance monthly
- hold: ~21 sessions
- frequency: monthly rebalance
- provenance: adapted
- status: active
- priority: P3
- sources: [Papers With Backtest — Industry Rotation Using Market-State Similarity](https://paperswithbacktest.com/strategies/industry-rotation-using-market-state-similarity)
- notes: The cited page is Zakamulin's abstract (May 2026): monthly French industry portfolios, quantile predictability concentrated in the tails, a market-state similarity strategy with 'higher Sharpe, lower volatility, substantially smaller drawdowns than the passive equal-weighted industry benchmark'. The '50 closest analogs' and 'top/bottom tercile' parameters are NOT in the abstract (catalog's own). Replicate on French data first, then port to ETFs.
- parameter_variants: analog count K {25/50 primary/75/100/150}; forward outcome horizon {36/60 primary/84 months}; legs {long-short primary; long-only}.
- data_needs: Ken French 49-industry data (external one-off); sector ETF series (DP-ETF)
- rust_status: IMPLEMENTED
- implementation_delta: DELTA — implementation is the French 49-industry monthly replication only; the sector-ETF port remains future work. Monthly French returns are used (no daily open-fill series); the state uses 60-month standardized market factors, the configured analog/horizon grid, and next-month industry outcomes; short borrow is 300 bps annualized; the close-aligned monthly backtest is an optimistic upper-bound, not an executable next-open replication.
- implementation_results: Rust in `src/family4.rs` / `src/study.rs`, config `configs/s046_market_state_industry_rotation.yaml`; Universe: French 49 industry portfolios with monthly returns; Data: French monthly industry/factor CSVs `C:\data\trading\deepthrust\data\cache\ff49\industries\49_Industry_Portfolios.csv` and `C:\data\trading\deepthrust\data\cache\ff49\factors\F-F_Research_Data_Factors.csv`; months 1926-07-31 .. 2026-09-30; evaluation starts 1926-07-31.. Full grid: 30 settings including baseline, 30 variant/fill rows; primary fill `close`. DEV-only selection `k75_f36_lo`; baseline DEV/TEST skill -1.624/-0.313; selected DEV/TEST skill +0.644/-0.279. Selected TEST Sharpe +0.563, PF +1.391, win +58.7%, trades 3567. DECAYED: skill sign changes. Variant-search bias is not included; TEST has been inspected.
  | portfolio | variant | partition | total log | trend log | random-pick baselines (log; p) | skill (log) | skill vs random U / pool (log) | bias (log) | skill % | skill p | Sharpe | PF | max DD | win % | trades |
  |---|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | `baseline` | DEV | -1.620 | +0.004 | U -3.286 (p=+0.001); pool -3.331 (p=+0.001) | -1.624 | U +1.666; pool +1.711 | +0.000 | n/a | +0.995 | -0.265 | +0.968 | -85.7% | +48.8% | 24835 |
  | baseline | `baseline` | TEST | -0.313 | -0.000 | U -0.883 (p=+0.015); pool -0.880 (p=+0.019) | -0.313 | U +0.570; pool +0.567 | +0.000 | n/a | +0.831 | -0.188 | +0.979 | -33.7% | +49.7% | 7128 |
  | DEV-selected | `k75_f36_lo` | DEV | +8.882 | +8.238 | U +7.694 (p=+0.001); pool +7.701 (p=+0.001) | +0.644 | U +1.188; pool +1.181 | +0.000 | +7.3% | +0.022 | +0.749 | +1.615 | -51.5% | +58.2% | 12317 |
  | DEV-selected | `k75_f36_lo` | TEST | +1.559 | +1.838 | U +1.682 (p=+0.747); pool +1.678 (p=+0.733) | -0.279 | U -0.123; pool -0.119 | +0.000 | n/a | +0.921 | +0.563 | +1.391 | -49.9% | +58.7% | 3567 |
  Reproduce: full matrix `cargo run --release -- --strategy s046_market_state_industry_rotation --variant analog25,analog100,long_only,k25_f36_ls,k25_f84_ls,k50_f36_ls,k50_f84_ls,k75_f36_ls,k75_f60_ls,k75_f84_ls,k100_f36_ls,k100_f84_ls,k150_f36_ls,k150_f60_ls,k150_f84_ls,k25_f36_lo,k25_f60_lo,k25_f84_lo,k50_f36_lo,k50_f84_lo,k75_f36_lo,k75_f60_lo,k75_f84_lo,k100_f36_lo,k100_f60_lo,k100_f84_lo,k150_f36_lo,k150_f60_lo,k150_f84_lo`; DEV-selected winner `cargo run --release -- --strategy s046_market_state_industry_rotation --variant k75_f36_lo`.
- review: RESOLVED

### S047 | Absolute / Dual Momentum (S047a absolute SPY vs cash; S047b Antonacci GEM)
- family: momentum and breakout
- concept: S047a: if SPY's 12-month return beats the 12-month cash (T-bill) return, hold SPY for the next month, else cash (Trading Garden / Antonacci absolute momentum). S047b: Antonacci's GEM (dual momentum): relative step US vs ex-US equities, then absolute step vs T-bills, else aggregate bonds (AGG) / BIL. A single-asset trend/momentum gate that avoids bear markets.
- universe: S047a: SPY, cash (BIL; FRED 3-month T-bill before 2007). S047b: SPY, EFA or ACWX, AGG, BIL
- direction: long
- indicators: 12-month total return of SPY (and EFA/ACWX); 12-month T-bill return
- entry: S047a: at month-end, if SPY 12-month return > 12-month T-bill return hold SPY the next month, else hold cash. S047b: at month-end pick the higher of SPY and EFA/ACWX by 12-month return; if it also beats T-bills hold it, else hold AGG (BIL as alternate)
- exit: Rebalance monthly
- hold: ~21 sessions
- frequency: monthly rebalance
- provenance: source-specified
- status: active
- priority: P2
- sources: [Trading Garden — 16 S&P 500 strategies (Absolute Momentum, Antonacci)](https://trading-garden.com/sp500-trading-strategies.html)
- notes: The earlier rule (SPY -> TLT -> cash) was neither the Trading Garden rule ('long for the whole next month if the 12-month return beats the 12-month cash rate') nor GEM; the TLT fallback adds duration risk (2022) and is now a registered variant of S047a. Trading Garden results (S&P/MES): 1963-today 35 trades, 68.6% win, CAGR 11.5% vs 10.9%; only 15 trades since 2000. Trade count is tiny (G-05): use as the most robust crash filter/overlay for faster strategies. Comparator: Faber 10-month SMA gate (price vs 10-month SMA, total return, monthly).
- parameter_variants: mode {absolute SPY-vs-cash primary; GEM; SMA10 gate}; momentum lookback {63/126/189/252 primary/315/378/504/630/756/1,008 days}; defensive asset {BIL primary; TLT}; SMA gate lookback {126/168/210/252/315 days}.
- data_needs: SPY, EFA/ACWX, AGG, BIL total-return series + FRED T-bill (DP-ETF, DP-FRED)
- rust_status: IMPLEMENTED
- implementation_delta: DELTA — next-open is the executable primary fill; same-close is an upper bound. Absolute, GEM, and SMA-gate modes are registered; ETF TR history starts in 2016 and BIL is used as the cash proxy, with no pre-2007 FRED splice.
- implementation_results: Rust in `src/family4.rs` / `src/study.rs`, config `configs/s047_absolute_dual_momentum.yaml`; Universe: AGG, BIL, EFA, SPY, TLT (daily bars from 1 sources); each symbol at most one position, book slots 1.; Data: `C:\data\trading\trading_framework\data\cache\etf_tr`; sessions 2016-01-04 .. 2026-10-23; evaluation starts 2016-01-04.. Full grid: 31 settings including baseline, 62 variant/fill rows; primary fill `next_open`. DEV-only selection `gem756`; baseline DEV/TEST skill +0.126/+0.289; selected DEV/TEST skill +0.404/+0.289. Selected TEST Sharpe +1.236, PF +3.577, win +73.5%, trades 34. insufficient trade-count evidence; PF or Sharpe > 2: leakage and timing review required. Variant-search bias is not included; TEST has been inspected.
  | portfolio | variant | partition | total log | trend log | random-pick baselines (log; p) | skill (log) | skill vs random U / pool (log) | bias (log) | skill % | skill p | Sharpe | PF | max DD | win % | trades |
  |---|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | `baseline` | DEV | +0.462 | +0.336 | U +0.283 (p=+0.248); pool +0.290 (p=+0.270) | +0.126 | U +0.179; pool +0.172 | +0.000 | +27.3% | +0.362 | +0.462 | +1.552 | -33.8% | +72.3% | 83 |
  | baseline | `baseline` | TEST | +0.501 | +0.212 | U +0.207 (p=+0.005); pool +0.204 (p=+0.003) | +0.289 | U +0.295; pool +0.298 | +0.000 | +57.7% | +0.050 | +1.236 | +3.577 | -18.7% | +73.5% | 34 |
  | DEV-selected | `gem756` | DEV | +0.646 | +0.242 | U +0.197 (p=+0.033); pool +0.194 (p=+0.033) | +0.404 | U +0.449; pool +0.452 | +0.000 | +62.6% | +0.111 | +0.582 | +1.749 | -33.8% | +66.1% | 59 |
  | DEV-selected | `gem756` | TEST | +0.501 | +0.212 | U +0.206 (p=+0.009); pool +0.210 (p=+0.009) | +0.289 | U +0.296; pool +0.292 | +0.000 | +57.7% | +0.050 | +1.236 | +3.577 | -18.7% | +73.5% | 34 |
  Reproduce: full matrix `cargo run --release -- --strategy s047_absolute_dual_momentum --variant gem,absolute_tlt,sma10_gate,abs63_bil,abs63_tlt,abs126_bil,abs126_tlt,abs189_bil,abs189_tlt,abs315_bil,abs315_tlt,abs378_bil,gem63,gem126,gem189,gem315,gem378,gem504,gem630,gem756,gem1008,sma126_bil,sma126_tlt,sma168_bil,sma168_tlt,sma210_tlt,sma252_bil,sma252_tlt,sma315_bil,sma315_tlt`; DEV-selected winner `cargo run --release -- --strategy s047_absolute_dual_momentum --variant gem756`.
- review: RESOLVED

### S048 | Validea Short-Term Sector Momentum
- family: momentum and breakout
- concept: Hold the 5 strongest of 10 sector ETFs ranked by multiple momentum metrics; documented 11.2% annual return since 2006 vs 10.9% for the S&P 500, beta 0.88.
- universe: 10 SPDR sector ETFs
- direction: long
- indicators: multi-metric momentum rank (e.g., 1-month and 3-month return blend)
- entry: At each rebalance (test weekly and monthly), buy the top 5 sector ETFs by blended 1M+3M return, equal-weight
- exit: Rebalance on the fixed schedule
- hold: 5-21 sessions
- frequency: weekly or monthly rebalance
- provenance: adapted
- variant_of: S042
- status: demoted
- priority: P3
- sources: [Validea — Sector Rotation Momentum Portfolio](https://www.validea.com/sector-rotation-momentum-portfolio)
- notes: Implement as a variant of S042 (lookbacks 1M+3M blend, top 5 of 10, monthly), not a separate strategy. Validea's page confirms 'Since 2006... 11.2%' vs S&P 500 10.9%, beta 0.88, standard deviation 19.0%, but its disclaimer says the optimal portfolios 'represent the rebalancing period that has led to the best historical performance... determined after the fact' and 'model returns... do not reflect actual trading': the headline is selected with hindsight, and 11.2% at beta 0.88 against 10.9% is only ~+1.6%/yr of alpha before costs. The 1M+3M blend and the weekly/monthly schedule are the catalog's choices. Benchmark vs equal-weight sectors.
- data_needs: sector ETF total-return daily series (DP-ETF)
- review: DEMOTE | source PARTIAL

### S049 | Acceleration Momentum
- family: momentum and breakout
- concept: Second-order momentum: stocks whose recent 1-month return exceeds their average monthly pace over the prior year are accelerating; acceleration adds cross-sectional information beyond level momentum.
- universe: S&P 500 stocks
- direction: long
- indicators: 21-day return; 252-day return; acceleration = r21 - r252/12 (difference, or cross-sectional ranks); the ratio r21/(r252/12) explodes when r252 is near zero and is not used
- entry: At each week's end, buy the decile with the highest acceleration (r21 - r252/12) among stocks with positive 252-day return; run weekly and monthly
- exit: Rebalance weekly
- hold: 5 sessions
- frequency: weekly rebalance
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: Chan interview (momentum lookbacks only, no acceleration concept): https://bettersystemtrader.com/012-ernest-chan/
- notes: Falsification: the numerator is the last 21-day return, i.e. a 1-month WINNER screen, the sign S027/S028 (and the NY Fed paper) say reverses; the two effects point in opposite directions and the hypothesis needs a theory for when acceleration dominates reversal. Pre-register S028 as the control; drop if it does not beat the inverted S028 reversal. A weekly decile rebalance of liquid names is expensive (G-08). The Chan interview does not discuss acceleration.
- parameter_variants: measure {r21 - r252/12 primary; cross-sectional rank difference}; rebalance {weekly, monthly}
- data_needs: daily_10years (D1)
- review: DEMOTE | source MISMATCH

### S050 | First Pullback After Breakout
- family: momentum and breakout
- concept: After a new 63-day high, the first 3-day pullback to the 10-day SMA is the highest-probability continuation entry — momentum pauses, profit-takers exit to momentum buyers.
- universe: S&P 500 stocks
- direction: long
- indicators: 63-day rolling high; 10-day SMA; 3-day pullback count
- entry: Stock made a 63-day high within the last 10 sessions, then closes 3 consecutive sessions lower with each close within 2% of the 10-day SMA; buy at close of the third session
- exit: Close > breakout bar's high (the 63-day-high bar's high, fixed at breakout; continuation confirmed) or close < 20-day SMA; max hold 10 sessions; report the exit-type distribution (most trades otherwise exit at the time stop)
- hold: 3-10 sessions
- frequency: several per week across the universe
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [Kvants — Volume breakout strategy (breakout context)](https://studio.kvants.ai/blog/volume-breakout-strategy)
- notes: Parameters 63/10/3/2% are all free: run a coarse grid with full trial accounting, not the single (63, 10, 3, 2%) point. A price within 2% of its 10-day SMA after 3 down closes can only exceed the breakout high within 10 sessions if the pullback was shallow. The cited Kvants page has no rules or numbers (decorative citation, see S040). Report with and without earnings-window signals (G-12).
- parameter_variants: grid {breakout lookback 42/63/126; pullback days 2/3/4; SMA distance 1%/2%/3%}; earnings avoidance {off primary; on}. All grid combinations are registered; next-open is primary and same-close is an upper bound.
- data_needs: daily_10years (D1) + earnings flag (DP-EARN)
- rust_status: IMPLEMENTED
- implementation_delta: DELTA — next-open is the executable primary fill; same-close is an upper bound. The full breakout/pullback/distance grid and earnings-avoidance variants are registered. Earnings filtering uses realized SimFin Publish Dates, not PIT scheduled announcements; stock history starts in 2016 and the S&P interval file is a public GitHub aggregation.
- implementation_results: Rust in `src/family4.rs` / `src/study.rs`, config `configs/s050_first_pullback.yaml`; Universe: A, AA, AABA, AAL, AAOI, AAP, AAPL, ABBV, ABCM, ABEV, ABMD, ABNB, ABT, ACB, ACC, ACGL, ACHR, ACIA, ACN, ADBE, ADI, ADM, ADP, ADSK, ADT, AEE, AEHR, AEIS, AEM, AEO, AEP, AER, AES, AET, AFL, AFRM, AG, AGN, AGNC, AGQ, AHT, AI, AIG, AIRC, AIZ, AJG, AKAM, AKRX, AKS, ALAB, ALB, ALC, ALGN, ALK, ALL, ALLE, ALLY, ALNY, ALXN, AMAT, AMBA, AMC, AMCR, AMD, AME, AMG, AMGN, AMKR, AMP, AMRN, AMT, AMTD, AMZN, AN, ANDV, ANET, ANF, ANSS, AON, AOS, APA, APC, APD, APH, APHA, APLD, APLS, APO, APP, APPN, APPS, APTV, AR, ARCH, ARCT, ARE, ARES, ARGX, ARM, ARMK, ARNC, ARRS, ARRY, ARWR, ASAN, ASML, ASST, ASTS, ASX, ATHM, ATHN, ATI, ATO, ATUS, ATVI, AU, AUY, AVAV, AVB, AVGO, AVLR, AVTR, AVY, AWK, AXON, AXP, AXTA, AXTI, AYI, AYX, AZEK, AZN, AZO, B, BA, BABA, BAC, BAH, BALL, BAM, BAP, BAX, BB, BBAI, BBBY, BBD, BBIG, BBL, BBWI, BBY, BCR, BCS, BDX, BE, BEAV, BEKE, BEN, BF.B, BFH, BG, BHC, BHI, BHP, BHVN, BIDU, BIIB, BILI, BILL, BIO, BIOR, BIVV, BJ, BK, BKI, BKNG, BKR, BLD, BLDP, BLDR, BLK, BLNK, BLUE, BMNR, BMRN, BMY, BN, BNGO, BNO, BNTX, BNY, BOIL, BOX, BP, BPL, BPMC, BR, BRCD, BRK.A, BRK.B, BRO, BROS, BSX, BTBT, BTDR, BTG, BTI, BTU, BUD, BULL, BURL, BWA, BWLD, BWXT, BX, BXP, BYND, BZUN, C, CAA, CACI, CAG, CAH, CAN, CAR, CARR, CART, CASY, CAT, CAVA, CAVM, CB, CBI, CBOE, CBRE, CBS, CC, CCEP, CCI, CCJ, CCK, CCL, CDE, CDNS, CDW, CE, CEG, CEI, CELG, CELH, CERN, CF, CFG, CFLT, CG, CGC, CGNX, CHD, CHGG, CHKP, CHPT, CHRD, CHRW, CHTR, CHWY, CI, CIEN, CIFR, CINF, CIT, CL, CLF, CLOV, CLR, CLS, CLSK, CLVS, CLX, CM, CMA, CMCSA, CME, CMG, CMI, CMS, CNC, CNH, CNI, CNK, CNP, CNQ, CNX, COF, COH, COHR, COIN, COL, COMM, CONE, COO, COP, COR, CORZ, COST, COTY, COUP, CP, CPAY, CPB, CPN, CPNG, CPRI, CPRT, CPT, CRCL, CRDO, CREE, CRH, CRI, CRL, CRM, CRON, CROX, CRS, CRSP, CRUS, CRWD, CRWV, CSC, CSCO, CSGP, CSIQ, CSL, CSX, CTAS, CTL, CTLT, CTRA, CTRP, CTSH, CTVA, CTXS, CVE, CVNA, CVS, CVX, CW, CWAN, CWH, CX, CXO, CY, CYBR, CYTK, CZR, D, DAL, DAR, DASH, DATA, DAY, DB, DBX, DD, DDD, DDOG, DE, DECK, DELL, DEN, DEO, DFS, DG, DGX, DHI, DHR, DINO, DIS, DISCA, DISCK, DISH, DJT, DK, DKNG, DKS, DLPH, DLR, DLTR, DNKN, DOC, DOCN, DOCU, DOV, DOW, DPS, DPZ, DQ, DRE, DRI, DT, DTE, DUK, DUOL, DVA, DVMT, DVN, DWDP, DXC, DXCM, EA, EAT, EBAY, ECA, ECHO, ECL, ED, EDIT, EDU, EFX, EG, EGN, EH, EIX, EL, ELAN, ELF, ELLI, ELV, EME, EMN, EMR, ENB, ENDP, ENPH, ENTG, ENVX, EOG, EOSE, EPAM, EPD, EQH, EQIX, EQNR, EQR, EQT, ERI, ERIC, ES, ESRX, ESS, ESTC, ESV, ET, ETFC, ETH, ETHE, ETN, ETP, ETR, ETSY, EVHC, EVRG, EW, EXAS, EXC, EXE, EXEL, EXPD, EXPE, EXR, F, FANG, FAST, FBIN, FCAU, FCEL, FCNCA, FCX, FDC, FDS, FDX, FE, FERG, FEYE, FFIV, FHN, FI, FICO, FIG, FIRY, FIS, FISV, FIT, FITB, FIVE, FIVN, FIX, FL, FLEX, FLG, FLGT, FLNA, FLR, FLS, FLT, FLUT, FLY, FMC, FN, FND, FNF, FNSR, FNV, FORM, FOUR, FOX, FOXA, FRC, FSLR, FSLY, FSR, FTAI, FTCH, FTI, FTNT, FTR, FTV, FUBO, FUTU, FVRR, FWONK, FXE, GAP, GBT, GBTC, GD, GDDY, GDS, GE, GEHC, GEN, GEV, GEVO, GFI, GFS, GG, GGB, GGP, GH, GILD, GIS, GL, GLDM, GLOB, GLW, GM, GME, GNRC, GOOG, GOOGL, GOOS, GOTU, GPC, GPN, GRAB, GRMN, GRWG, GS, GSK, GT, GTLB, GTLS, GTM, GWPH, GWR, GWRE, GWW, GXP, HAIN, HAL, HALO, HAR, HAS, HBAN, HBI, HCA, HCP, HD, HDB, HDS, HES, HFC, HIG, HII, HIMS, HL, HLF, HLN, HLT, HOG, HOLX, HON, HOOD, HP, HPE, HPQ, HR, HRB, HRC, HRL, HRS, HSBC, HSIC, HST, HSY, HTHT, HUBB, HUBS, HUM, HUN, HUT, HWM, HYLN, HZNP, IAC, IAU, IAUM, IBKR, IBM, IBN, ICE, ICLR, IDEX, IDTI, IDXX, IEX, IFF, ILMN, IMGN, IMMU, INCY, INFY, INMD, INO, INSM, INSP, INTC, INTU, INVH, INXN, IONQ, IONS, IOT, IP, IPG, IPGP, IPHI, IQ, IQV, IR, IRBT, IREN, IRM, ISEE, ISRG, IT, ITCI, ITUB, ITW, IVZ, J, JAZZ, JBHT, JBL, JBLU, JCI, JCP, JD, JEC, JHX, JKHY, JKS, JMIA, JNJ, JNPR, JOBY, JOYY, JPM, JUNO, JWN, K, KATE, KDP, KEY, KEYS, KGC, KHC, KIM, KITE, KKR, KL, KLAC, KMB, KMI, KMX, KNX, KO, KODK, KOLD, KORS, KR, KRTX, KSS, KSU, KTOS, KVUE, LAC, LAD, LB, LBRDK, LBTYA, LBTYK, LCID, LDOS, LEA, LEN, LEU, LGND, LH, LHX, LI, LII, LIN, LITE, LKQ, LLL, LLTC, LLY, LM, LMND, LMT, LNC, LNG, LNT, LOGC, LOGM, LOW, LOXO, LPLA, LPT, LPX, LRCX, LSCC, LSI, LSPD, LTHM, LULU, LUMN, LUNR, LUV, LVGO, LVLT, LVS, LW, LYB, LYFT, LYV, M, MA, MAA, MAC, MAN, MAR, MARA, MAS, MASI, MAT, MBLY, MCD, MCHP, MCK, MCO, MDB, MDCO, MDGL, MDLZ, MDSO, MDT, MELI, MENT, MET, META, MGA, MGM, MGNI, MHK, MIDD, MIME, MJN, MKC, MKSI, MKTX, MLCO, MLM, MLNX, MMC, MMM, MNDY, MNK, MNST, MO, MOD, MOH, MOMO, MON, MOS, MP, MPC, MPLX, MPT, MPW, MPWR, MRK, MRNA, MRO, MRSH, MRVL, MS, MSCC, MSCI, MSFT, MSI, MSTR, MT, MTB, MTCH, MTD, MTN, MTSI, MTZ, MU, MULE, MUR, MVIS, MXIM, MXL, MYL, MYOK, NBIS, NBIX, NBL, NBR, NCLH, NDAQ, NEE, NEM, NET, NEWR, NFLX, NI, NIO, NKE, NKLA, NKTR, NLSN, NLY, NNDM, NOC, NOK, NOV, NOW, NRG, NSC, NTAP, NTES, NTLA, NTNX, NTR, NTRA, NTRS, NU, NUAN, NUE, NVAX, NVCR, NVDA, NVMI, NVO, NVR, NVS, NVT, NVTA, NVTS, NWL, NXH, NXPI, NXT, O, OA, OC, OCGN, OCLR, ODFL, OKE, OKLO, OKTA, OLED, OLLI, OMC, ON, ONCE, ONDS, ONON, ONTO, OPEN, OPTU, ORCL, ORLY, OSCR, OSH, OTIS, OVV, OWL, OXY, P, PAA, PAAS, PACB, PACW, PAGS, PANW, PATH, PAYC, PAYX, PBCT, PBF, PBR, PBR.A, PBYI, PCAR, PCG, PCOR, PCTY, PDD, PE, PEG, PEN, PENN, PEP, PF, PFE, PFG, PFPT, PG, PGR, PH, PHM, PHYS, PII, PINS, PKG, PL, PLAN, PLD, PLNT, PLTR, PLUG, PM, PNC, PNR, PNRA, PNW, PODD, POET, POOL, POT, PPG, PPL, PPLI, PR, PRGO, PRMB, PRU, PRXL, PSA, PSLV, PSTG, PSX, PTC, PTEN, PTON, PVH, PWR, PX, PXD, PYPL, Q, QBTS, QCOM, QDEL, QFIN, QGEN, QRVO, QS, QSR, QTS, QUBT, QVCA, QXO, RACE, RAI, RBA, RBLX, RBRK, RCL, RDDT, RDFN, RDS.A, RDS.B, RDW, REG, REGN, RETA, RF, RGEN, RGLD, RGTI, RH, RHI, RHT, RICE, RIG, RIO, RIOT, RIVN, RJF, RKLB, RKT, RL, RMBS, RMD, RNA, RNG, ROIV, ROK, ROKU, ROL, ROP, ROST, RP, RR, RRC, RRX, RS, RSG, RSPP, RTN, RTX, RUN, RVMD, RVTY, RXDX, RXRX, RY, S, SABR, SAGE, SAIA, SAIL, SAM, SAP, SATS, SAVA, SAVE, SBAC, SBE, SBET, SBNY, SBUX, SCCO, SCG, SCHW, SCO, SE, SEDG, SEE, SERV, SFIX, SFM, SGEN, SGI, SGOL, SHAK, SHEL, SHLL, SHOP, SHPG, SHW, SI, SIG, SIGA, SIMO, SINA, SIRI, SITM, SIVB, SIVR, SIX, SJM, SKLZ, SKX, SLB, SLCA, SLG, SLW, SM, SMAR, SMCI, SMR, SMTC, SN, SNA, SNAP, SNDK, SNDL, SNI, SNOW, SNPS, SNV, SNY, SO, SOFI, SOLO, SONY, SOS, SOUN, SPCE, SPCX, SPG, SPGI, SPLK, SPOT, SPR, SPRU, SQM, SRC, SRCL, SRE, SRNE, SRPT, SSNC, STE, STJ, STLA, STLD, STM, STNE, STOR, STRL, STT, STX, STZ, SU, SUI, SUM, SW, SWAV, SWK, SWKS, SWN, SWTX, SXL, SYF, SYK, SYMC, SYY, T, TAL, TAP, TCO, TCOM, TD, TDG, TDOC, TDY, TE, TEAM, TECD, TECH, TECK, TEL, TEM, TER, TEVA, TFC, TFX, TGT, THC, THO, TIF, TIGR, TJX, TKO, TLN, TLRY, TMDX, TME, TMO, TMUS, TNDM, TNL, TOL, TOST, TPL, TPR, TREX, TRGP, TRI, TRIP, TROW, TRP, TRU, TRV, TS, TSCO, TSEM, TSLA, TSM, TSN, TSO, TSRO, TSS, TT, TTD, TTE, TTMI, TTWO, TW, TWLO, TWST, TWTR, TWX, TXG, TXN, TXRH, TXT, TYL, U, UA, UAA, UAL, UBER, UCO, UDR, UEC, UGI, UGL, UHS, UL, ULTA, UMC, UNG, UNH, UNM, UNP, UPS, UPST, URBN, URI, USAR, USB, USFD, UTHR, UTX, UUUU, V, VAL, VALE, VALE.P, VAR, VEEV, VER, VFC, VG, VIAB, VIAV, VICI, VICR, VIK, VIPS, VISN, VKTX, VLO, VLTO, VMC, VMRK, VMW, VNO, VNTV, VOD, VOYA, VRNA, VRSK, VRSN, VRT, VRTX, VRX, VSH, VST, VTR, VTRS, VUZI, VXRT, VZ, W, WAB, WAL, WAT, WB, WBA, WBC, WBD, WBS, WCC, WCG, WCN, WDAY, WDC, WEC, WELL, WEX, WFC, WFM, WFRD, WFT, WHR, WING, WISH, WIX, WKHS, WLK, WLL, WM, WMB, WMGI, WMT, WNR, WOLF, WORK, WP, WPC, WPM, WPX, WRB, WRK, WSM, WSO, WST, WTW, WU, WUBA, WULF, WWAV, WWD, WWE, WY, WYN, WYNN, XEC, XEL, XL, XLNX, XNET, XOM, XP, XPEV, XPO, XRAY, XRX, XYL, XYZ, Y, YELP, YHOO, YUM, YUMC, Z, ZAYO, ZBH, ZBRA, ZEN, ZETA, ZG, ZIM, ZION, ZM, ZNGA, ZS, ZSL, ZTO, ZTS (daily bars from 1 sources); each symbol at most one position, book slots 1.; Data: `C:\data\trading\trading_framework\data\cache\daily_10years_tr`; sessions 2016-01-04 .. 2026-10-23; evaluation starts 2016-01-04.. Full grid: 30 settings including baseline, 60 variant/fill rows; primary fill `next_open`. DEV-only selection `breakout63_pullback4_dist1`; baseline DEV/TEST skill -1.698/-0.458; selected DEV/TEST skill +0.027/-0.156. Selected TEST Sharpe -0.191, PF +0.719, win +33.3%, trades 45. DECAYED: skill sign changes. Variant-search bias is not included; TEST has been inspected.
  | portfolio | variant | partition | total log | trend log | random-pick baselines (log; p) | skill (log) | skill vs random U / pool (log) | bias (log) | skill % | skill p | Sharpe | PF | max DD | win % | trades |
  |---|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | `baseline` | DEV | -0.925 | +0.773 | U -0.441 (p=+0.951); pool -0.441 (p=+0.929) | -1.698 | U -0.484; pool -0.484 | +0.000 | n/a | +1.000 | -0.830 | +0.797 | -63.9% | +40.6% | 4848 |
  | baseline | `baseline` | TEST | -0.180 | +0.278 | U -0.154 (p=+0.565); pool -0.172 (p=+0.526) | -0.458 | U -0.026; pool -0.009 | +0.000 | n/a | +0.975 | -0.411 | +1.082 | -29.1% | +42.9% | 1571 |
  | DEV-selected | `breakout63_pullback4_dist1` | DEV | -0.066 | -0.093 | U -0.424 (p=+0.103); pool -0.430 (p=+0.119) | +0.027 | U +0.358; pool +0.364 | +0.000 | n/a | +0.458 | -0.061 | +0.959 | -34.8% | +41.7% | 259 |
  | DEV-selected | `breakout63_pullback4_dist1` | TEST | -0.056 | +0.101 | U -0.022 (p=+0.591); pool -0.027 (p=+0.578) | -0.156 | U -0.034; pool -0.029 | +0.000 | n/a | +0.886 | -0.191 | +0.719 | -21.4% | +33.3% | 45 |
  Reproduce: full matrix `cargo run --release -- --strategy s050_first_pullback --variant avoid_earnings,breakout42_pullback2_dist1,breakout42_pullback2_dist2,breakout42_pullback2_dist3,breakout42_pullback3_dist1,breakout42_pullback3_dist2,breakout42_pullback3_dist3,breakout42_pullback4_dist1,breakout42_pullback4_dist2,breakout42_pullback4_dist3,breakout63_pullback2_dist1,breakout63_pullback2_dist2,breakout63_pullback2_dist3,breakout63_pullback3_dist1,breakout63_pullback3_dist2,breakout63_pullback3_dist3,breakout63_pullback4_dist1,breakout63_pullback4_dist2,breakout63_pullback4_dist3,breakout126_pullback2_dist1,breakout126_pullback2_dist2,breakout126_pullback2_dist3,breakout126_pullback3_dist1,breakout126_pullback3_dist2,breakout126_pullback3_dist3,breakout126_pullback4_dist1,breakout126_pullback4_dist2,breakout126_pullback4_dist3,breakout126_pullback4_dist2_avoid_earnings`; DEV-selected winner `cargo run --release -- --strategy s050_first_pullback --variant breakout63_pullback4_dist1`.
- review: RESOLVED

### S051 | Darvas Box Breakout
- family: momentum and breakout
- concept: Nicolas Darvas' 1950s system, in its usual mechanical form: a 'box' top is a high that is not beaten for three days; the box bottom is a low that holds for the three days after that; buy a close above the box top.
- universe: liquid stocks
- direction: long
- indicators: Darvas box top (high unbeaten for 3 sessions) and box bottom (low unbeaten for the following 3 sessions)
- entry: Close(t) > current box top; buy at close of t (fill at next open as alternate)
- exit: Close < the latest box bottom (which ratchets up as new boxes form); no profit target
- hold: days to weeks
- frequency: weekly across a liquid universe
- provenance: adapted
- status: active
- priority: P3
- sources: Be The Puck - Darvas box scorecard (mechanical definition above; beat buy-and-hold Sharpe on 44/372; S&P 500 index +4.3%/yr vs +8.9%); the Kagels Turtle page cited earlier contains no Darvas rule and is no longer cited
- notes: REPLACED: the earlier '55-day range unexceeded for 15+ sessions' mixed two windows and was an arbitrary range breakout (a close cousin of S036), with a stray non-English typo in the notes (fixed). The three-day box is the Be The Puck mechanical definition; Darvas' original rules were discretionary.
- parameter_variants: PIT universe top-N {100/250/500 primary/750/1,000}; minimum history {126/250 primary/500 sessions}; fill {next-open primary; same-close upper bound}.
- data_needs: daily_10years (D1)
- rust_status: IMPLEMENTED
- implementation_delta: DELTA — next-open is the executable primary fill; same-close is an upper bound. Entries use PIT liquid-stock ranks, and each three-session pivot is confirmed only with data available then. Cached stock history starts in 2016. DEV skill ranks are not fully like-for-like: the 500-session minimum-history variants start evaluation in December 2017, versus December 2016 for the 126/250-session variants.
- implementation_results: Rust in `src/family4.rs` / `src/study.rs`, config `configs/s051_darvas_box.yaml`; Universe: A, AA, AABA, AAL, AAOI, AAP, AAPL, ABBV, ABCM, ABEV, ABMD, ABNB, ABT, ACB, ACC, ACGL, ACHR, ACIA, ACN, ADBE, ADI, ADM, ADP, ADSK, ADT, AEE, AEHR, AEIS, AEM, AEO, AEP, AER, AES, AET, AFL, AFRM, AG, AGN, AGNC, AGQ, AHT, AI, AIG, AIRC, AIZ, AJG, AKAM, AKRX, AKS, ALAB, ALB, ALC, ALGN, ALK, ALL, ALLE, ALLY, ALNY, ALXN, AMAT, AMBA, AMC, AMCR, AMD, AME, AMG, AMGN, AMKR, AMP, AMRN, AMT, AMTD, AMZN, AN, ANDV, ANET, ANF, ANSS, AON, AOS, APA, APC, APD, APH, APHA, APLD, APLS, APO, APP, APPN, APPS, APTV, AR, ARCH, ARCT, ARE, ARES, ARGX, ARM, ARMK, ARNC, ARRS, ARRY, ARWR, ASAN, ASML, ASST, ASTS, ASX, ATHM, ATHN, ATI, ATO, ATUS, ATVI, AU, AUY, AVAV, AVB, AVGO, AVLR, AVTR, AVY, AWK, AXON, AXP, AXTA, AXTI, AYI, AYX, AZEK, AZN, AZO, B, BA, BABA, BAC, BAH, BALL, BAM, BAP, BAX, BB, BBAI, BBBY, BBD, BBIG, BBL, BBWI, BBY, BCR, BCS, BDX, BE, BEAV, BEKE, BEN, BF.B, BFH, BG, BHC, BHI, BHP, BHVN, BIDU, BIIB, BILI, BILL, BIO, BIOR, BIVV, BJ, BK, BKI, BKNG, BKR, BLD, BLDP, BLDR, BLK, BLNK, BLUE, BMNR, BMRN, BMY, BN, BNGO, BNO, BNTX, BNY, BOIL, BOX, BP, BPL, BPMC, BR, BRCD, BRK.A, BRK.B, BRO, BROS, BSX, BTBT, BTDR, BTG, BTI, BTU, BUD, BULL, BURL, BWA, BWLD, BWXT, BX, BXP, BYND, BZUN, C, CAA, CACI, CAG, CAH, CAN, CAR, CARR, CART, CASY, CAT, CAVA, CAVM, CB, CBI, CBOE, CBRE, CBS, CC, CCEP, CCI, CCJ, CCK, CCL, CDE, CDNS, CDW, CE, CEG, CEI, CELG, CELH, CERN, CF, CFG, CFLT, CG, CGC, CGNX, CHD, CHGG, CHKP, CHPT, CHRD, CHRW, CHTR, CHWY, CI, CIEN, CIFR, CINF, CIT, CL, CLF, CLOV, CLR, CLS, CLSK, CLVS, CLX, CM, CMA, CMCSA, CME, CMG, CMI, CMS, CNC, CNH, CNI, CNK, CNP, CNQ, CNX, COF, COH, COHR, COIN, COL, COMM, CONE, COO, COP, COR, CORZ, COST, COTY, COUP, CP, CPAY, CPB, CPN, CPNG, CPRI, CPRT, CPT, CRCL, CRDO, CREE, CRH, CRI, CRL, CRM, CRON, CROX, CRS, CRSP, CRUS, CRWD, CRWV, CSC, CSCO, CSGP, CSIQ, CSL, CSX, CTAS, CTL, CTLT, CTRA, CTRP, CTSH, CTVA, CTXS, CVE, CVNA, CVS, CVX, CW, CWAN, CWH, CX, CXO, CY, CYBR, CYTK, CZR, D, DAL, DAR, DASH, DATA, DAY, DB, DBX, DD, DDD, DDOG, DE, DECK, DELL, DEN, DEO, DFS, DG, DGX, DHI, DHR, DINO, DIS, DISCA, DISCK, DISH, DJT, DK, DKNG, DKS, DLPH, DLR, DLTR, DNKN, DOC, DOCN, DOCU, DOV, DOW, DPS, DPZ, DQ, DRE, DRI, DT, DTE, DUK, DUOL, DVA, DVMT, DVN, DWDP, DXC, DXCM, EA, EAT, EBAY, ECA, ECHO, ECL, ED, EDIT, EDU, EFX, EG, EGN, EH, EIX, EL, ELAN, ELF, ELLI, ELV, EME, EMN, EMR, ENB, ENDP, ENPH, ENTG, ENVX, EOG, EOSE, EPAM, EPD, EQH, EQIX, EQNR, EQR, EQT, ERI, ERIC, ES, ESRX, ESS, ESTC, ESV, ET, ETFC, ETH, ETHE, ETN, ETP, ETR, ETSY, EVHC, EVRG, EW, EXAS, EXC, EXE, EXEL, EXPD, EXPE, EXR, F, FANG, FAST, FBIN, FCAU, FCEL, FCNCA, FCX, FDC, FDS, FDX, FE, FERG, FEYE, FFIV, FHN, FI, FICO, FIG, FIRY, FIS, FISV, FIT, FITB, FIVE, FIVN, FIX, FL, FLEX, FLG, FLGT, FLNA, FLR, FLS, FLT, FLUT, FLY, FMC, FN, FND, FNF, FNSR, FNV, FORM, FOUR, FOX, FOXA, FRC, FSLR, FSLY, FSR, FTAI, FTCH, FTI, FTNT, FTR, FTV, FUBO, FUTU, FVRR, FWONK, FXE, GAP, GBT, GBTC, GD, GDDY, GDS, GE, GEHC, GEN, GEV, GEVO, GFI, GFS, GG, GGB, GGP, GH, GILD, GIS, GL, GLDM, GLOB, GLW, GM, GME, GNRC, GOOG, GOOGL, GOOS, GOTU, GPC, GPN, GRAB, GRMN, GRWG, GS, GSK, GT, GTLB, GTLS, GTM, GWPH, GWR, GWRE, GWW, GXP, HAIN, HAL, HALO, HAR, HAS, HBAN, HBI, HCA, HCP, HD, HDB, HDS, HES, HFC, HIG, HII, HIMS, HL, HLF, HLN, HLT, HOG, HOLX, HON, HOOD, HP, HPE, HPQ, HR, HRB, HRC, HRL, HRS, HSBC, HSIC, HST, HSY, HTHT, HUBB, HUBS, HUM, HUN, HUT, HWM, HYLN, HZNP, IAC, IAU, IAUM, IBKR, IBM, IBN, ICE, ICLR, IDEX, IDTI, IDXX, IEX, IFF, ILMN, IMGN, IMMU, INCY, INFY, INMD, INO, INSM, INSP, INTC, INTU, INVH, INXN, IONQ, IONS, IOT, IP, IPG, IPGP, IPHI, IQ, IQV, IR, IRBT, IREN, IRM, ISEE, ISRG, IT, ITCI, ITUB, ITW, IVZ, J, JAZZ, JBHT, JBL, JBLU, JCI, JCP, JD, JEC, JHX, JKHY, JKS, JMIA, JNJ, JNPR, JOBY, JOYY, JPM, JUNO, JWN, K, KATE, KDP, KEY, KEYS, KGC, KHC, KIM, KITE, KKR, KL, KLAC, KMB, KMI, KMX, KNX, KO, KODK, KOLD, KORS, KR, KRTX, KSS, KSU, KTOS, KVUE, LAC, LAD, LB, LBRDK, LBTYA, LBTYK, LCID, LDOS, LEA, LEN, LEU, LGND, LH, LHX, LI, LII, LIN, LITE, LKQ, LLL, LLTC, LLY, LM, LMND, LMT, LNC, LNG, LNT, LOGC, LOGM, LOW, LOXO, LPLA, LPT, LPX, LRCX, LSCC, LSI, LSPD, LTHM, LULU, LUMN, LUNR, LUV, LVGO, LVLT, LVS, LW, LYB, LYFT, LYV, M, MA, MAA, MAC, MAN, MAR, MARA, MAS, MASI, MAT, MBLY, MCD, MCHP, MCK, MCO, MDB, MDCO, MDGL, MDLZ, MDSO, MDT, MELI, MENT, MET, META, MGA, MGM, MGNI, MHK, MIDD, MIME, MJN, MKC, MKSI, MKTX, MLCO, MLM, MLNX, MMC, MMM, MNDY, MNK, MNST, MO, MOD, MOH, MOMO, MON, MOS, MP, MPC, MPLX, MPT, MPW, MPWR, MRK, MRNA, MRO, MRSH, MRVL, MS, MSCC, MSCI, MSFT, MSI, MSTR, MT, MTB, MTCH, MTD, MTN, MTSI, MTZ, MU, MULE, MUR, MVIS, MXIM, MXL, MYL, MYOK, NBIS, NBIX, NBL, NBR, NCLH, NDAQ, NEE, NEM, NET, NEWR, NFLX, NI, NIO, NKE, NKLA, NKTR, NLSN, NLY, NNDM, NOC, NOK, NOV, NOW, NRG, NSC, NTAP, NTES, NTLA, NTNX, NTR, NTRA, NTRS, NU, NUAN, NUE, NVAX, NVCR, NVDA, NVMI, NVO, NVR, NVS, NVT, NVTA, NVTS, NWL, NXH, NXPI, NXT, O, OA, OC, OCGN, OCLR, ODFL, OKE, OKLO, OKTA, OLED, OLLI, OMC, ON, ONCE, ONDS, ONON, ONTO, OPEN, OPTU, ORCL, ORLY, OSCR, OSH, OTIS, OVV, OWL, OXY, P, PAA, PAAS, PACB, PACW, PAGS, PANW, PATH, PAYC, PAYX, PBCT, PBF, PBR, PBR.A, PBYI, PCAR, PCG, PCOR, PCTY, PDD, PE, PEG, PEN, PENN, PEP, PF, PFE, PFG, PFPT, PG, PGR, PH, PHM, PHYS, PII, PINS, PKG, PL, PLAN, PLD, PLNT, PLTR, PLUG, PM, PNC, PNR, PNRA, PNW, PODD, POET, POOL, POT, PPG, PPL, PPLI, PR, PRGO, PRMB, PRU, PRXL, PSA, PSLV, PSTG, PSX, PTC, PTEN, PTON, PVH, PWR, PX, PXD, PYPL, Q, QBTS, QCOM, QDEL, QFIN, QGEN, QRVO, QS, QSR, QTS, QUBT, QVCA, QXO, RACE, RAI, RBA, RBLX, RBRK, RCL, RDDT, RDFN, RDS.A, RDS.B, RDW, REG, REGN, RETA, RF, RGEN, RGLD, RGTI, RH, RHI, RHT, RICE, RIG, RIO, RIOT, RIVN, RJF, RKLB, RKT, RL, RMBS, RMD, RNA, RNG, ROIV, ROK, ROKU, ROL, ROP, ROST, RP, RR, RRC, RRX, RS, RSG, RSPP, RTN, RTX, RUN, RVMD, RVTY, RXDX, RXRX, RY, S, SABR, SAGE, SAIA, SAIL, SAM, SAP, SATS, SAVA, SAVE, SBAC, SBE, SBET, SBNY, SBUX, SCCO, SCG, SCHW, SCO, SE, SEDG, SEE, SERV, SFIX, SFM, SGEN, SGI, SGOL, SHAK, SHEL, SHLL, SHOP, SHPG, SHW, SI, SIG, SIGA, SIMO, SINA, SIRI, SITM, SIVB, SIVR, SIX, SJM, SKLZ, SKX, SLB, SLCA, SLG, SLW, SM, SMAR, SMCI, SMR, SMTC, SN, SNA, SNAP, SNDK, SNDL, SNI, SNOW, SNPS, SNV, SNY, SO, SOFI, SOLO, SONY, SOS, SOUN, SPCE, SPCX, SPG, SPGI, SPLK, SPOT, SPR, SPRU, SQM, SRC, SRCL, SRE, SRNE, SRPT, SSNC, STE, STJ, STLA, STLD, STM, STNE, STOR, STRL, STT, STX, STZ, SU, SUI, SUM, SW, SWAV, SWK, SWKS, SWN, SWTX, SXL, SYF, SYK, SYMC, SYY, T, TAL, TAP, TCO, TCOM, TD, TDG, TDOC, TDY, TE, TEAM, TECD, TECH, TECK, TEL, TEM, TER, TEVA, TFC, TFX, TGT, THC, THO, TIF, TIGR, TJX, TKO, TLN, TLRY, TMDX, TME, TMO, TMUS, TNDM, TNL, TOL, TOST, TPL, TPR, TREX, TRGP, TRI, TRIP, TROW, TRP, TRU, TRV, TS, TSCO, TSEM, TSLA, TSM, TSN, TSO, TSRO, TSS, TT, TTD, TTE, TTMI, TTWO, TW, TWLO, TWST, TWTR, TWX, TXG, TXN, TXRH, TXT, TYL, U, UA, UAA, UAL, UBER, UCO, UDR, UEC, UGI, UGL, UHS, UL, ULTA, UMC, UNG, UNH, UNM, UNP, UPS, UPST, URBN, URI, USAR, USB, USFD, UTHR, UTX, UUUU, V, VAL, VALE, VALE.P, VAR, VEEV, VER, VFC, VG, VIAB, VIAV, VICI, VICR, VIK, VIPS, VISN, VKTX, VLO, VLTO, VMC, VMRK, VMW, VNO, VNTV, VOD, VOYA, VRNA, VRSK, VRSN, VRT, VRTX, VRX, VSH, VST, VTR, VTRS, VUZI, VXRT, VZ, W, WAB, WAL, WAT, WB, WBA, WBC, WBD, WBS, WCC, WCG, WCN, WDAY, WDC, WEC, WELL, WEX, WFC, WFM, WFRD, WFT, WHR, WING, WISH, WIX, WKHS, WLK, WLL, WM, WMB, WMGI, WMT, WNR, WOLF, WORK, WP, WPC, WPM, WPX, WRB, WRK, WSM, WSO, WST, WTW, WU, WUBA, WULF, WWAV, WWD, WWE, WY, WYN, WYNN, XEC, XEL, XL, XLNX, XNET, XOM, XP, XPEV, XPO, XRAY, XRX, XYL, XYZ, Y, YELP, YHOO, YUM, YUMC, Z, ZAYO, ZBH, ZBRA, ZEN, ZETA, ZG, ZIM, ZION, ZM, ZNGA, ZS, ZSL, ZTO, ZTS (daily bars from 1 sources); each symbol at most one position, book slots 1.; Data: `C:\data\trading\trading_framework\data\cache\daily_10years_tr`; sessions 2016-01-04 .. 2026-10-23; evaluation starts 2016-12-29.. Full grid: 31 settings including baseline, 32 variant/fill rows; primary fill `next_open`. DEV-only selection `u100_h250_next`; baseline DEV/TEST skill -0.696/-0.083; selected DEV/TEST skill -0.412/-0.174. Selected TEST Sharpe +0.880, PF +1.493, win +40.7%, trades 1549. no selected-result evidence or high-metric flag. Variant-search bias is not included; TEST has been inspected.
  | portfolio | variant | partition | total log | trend log | random-pick baselines (log; p) | skill (log) | skill vs random U / pool (log) | bias (log) | skill % | skill p | Sharpe | PF | max DD | win % | trades |
  |---|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
  | baseline | `baseline` | DEV | +0.067 | +0.763 | U +0.202 (p=+0.965); pool +0.207 (p=+0.973) | -0.696 | U -0.135; pool -0.140 | +0.000 | n/a | +0.999 | +0.143 | +1.056 | -36.2% | +40.8% | 21083 |
  | baseline | `baseline` | TEST | +0.319 | +0.402 | U +0.216 (p=+0.028); pool +0.216 (p=+0.031) | -0.083 | U +0.104; pool +0.103 | +0.000 | n/a | +0.732 | +0.846 | +1.240 | -17.5% | +41.9% | 8117 |
  | DEV-selected | `u100_h250_next` | DEV | +0.407 | +0.820 | U +0.384 (p=+0.442); pool +0.393 (p=+0.469) | -0.412 | U +0.023; pool +0.014 | +0.000 | n/a | +0.974 | +0.405 | +1.187 | -33.4% | +39.3% | 4106 |
  | DEV-selected | `u100_h250_next` | TEST | +0.422 | +0.596 | U +0.379 (p=+0.333); pool +0.381 (p=+0.339) | -0.174 | U +0.043; pool +0.042 | +0.000 | n/a | +0.850 | +0.880 | +1.493 | -18.6% | +40.7% | 1549 |
  Reproduce: full matrix `cargo run --release -- --strategy s051_darvas_box --variant u100_h126_close,u100_h126_next,u100_h250_close,u100_h250_next,u100_h500_close,u100_h500_next,u250_h126_close,u250_h126_next,u250_h250_close,u250_h250_next,u250_h500_close,u250_h500_next,u500_h126_close,u500_h126_next,u500_h250_close,u500_h250_next,u500_h500_close,u500_h500_next,u750_h126_close,u750_h126_next,u750_h250_close,u750_h250_next,u750_h500_close,u750_h500_next,u1000_h126_close,u1000_h126_next,u1000_h250_close,u1000_h250_next,u1000_h500_close,u1000_h500_next`; DEV-selected winner `cargo run --release -- --strategy s051_darvas_box --variant u100_h250_next`.
- review: RESOLVED

---

# Family 5 — Earnings, News, and LLM Strategies

*Why this family works: information diffuses slowly. Post-earnings-announcement drift (PEAD) is among the longest-documented anomalies; stocks that beat estimates keep drifting up for weeks. The earnings announcement premium (NBER w13090): stocks announcing earnings in a month outperform non-announcers in that month — a calendar-holdable premium. LLM-based sentiment (arXiv 2412.19245) demonstrates daily news sentiment L/S portfolios; Kiszczuk's SSRN paper (attached; a single title page, abstract only) is an explainability study with no trading rule.*

### S052 | Earnings Announcement Premium (Monthly)
- family: earnings (calendar-driven)
- concept: Frazzini-Lamont (NBER w13090): stocks expected to announce earnings in month t outperform stocks not announcing; the strategy buys every expected announcer at the last close of month t-1 and shorts non-announcers.
- universe: US stocks with exactly 4 announcements in the prior 12 months (expected dates from SimFin Publish Date, 2009+; ~2/3 of stocks announce in any month)
- direction: long and short (long-only version: just hold announcers)
- indicators: expected earnings calendar for month t (prior-year pattern); past announcement-period volume concentration (Frazzini-Lamont main cross-sectional result)
- entry: On the last trading day of month t-1, buy all stocks expected to announce in month t (equal-weight at paper scale, value-weight at research scale), sorted by past announcement-period volume concentration (high-volume names earn the highest premium); short non-announcers (long-only variant: long announcers only, market-neutralized with an SPY short)
- exit: Hold through the last trading day of month t; reform monthly. Weekly version (ex-S086): single exit rule = hold the weekly basket for one week
- hold: ~21 sessions
- frequency: monthly rebalance
- provenance: source-specified
- status: active
- priority: P2
- sources: [NBER w13090 — The Earnings Announcement Premium and Trading Volume](https://www.nber.org/system/files/working_papers/w13090/w13090.pdf)
- notes: NBER w13090 (Frazzini & Lamont) verified: 'exactly four earnings announcements' restriction, long expected announcers / short non-announcers, value-weighted monthly, 61 bps/month long-short, Sharpe 0.94 vs 0.70 for momentum; stocks with high past announcement-period volume earn the highest premium. The sample ends in the early 2000s; the premium after publication is untested in the cited paper, so report subperiods. This is an earnings-calendar strategy only: no LLM or news input.
- parameter_variants: cadence {monthly primary; weekly ex-S086 (weeks 3-4 of Jan/Apr/Jul/Oct)}; weighting {equal paper, value research}; sort {volume concentration primary; none}; leg {long announcers - SPY hedge primary; long-short}
- merged_from: S086
- data_needs: SimFin Publish Date 2009+ (DP-EARN); daily_10years (D1)
- review: RESOLVED

### S053 | Pre-Earnings Run-Up (consensus-free; replaces the Earnings Whispers Score setup)
- family: earnings (calendar-driven)
- concept: Frazzini-Lamont's announcement premium accrues in the days before and at the announcement and is largest for stocks with high past announcement-period volume. A consensus-free pre-earnings run-up replaces the proprietary Earnings Whispers Score (no free history), and the exit explicitly INCLUDES the earnings reaction.
- universe: liquid_top_N US stocks with SimFin Publish Date history
- direction: long
- indicators: days to the expected announcement (prior-year Publish Date pattern); past announcement-period volume concentration (top tercile)
- entry: Buy a stock 5-7 sessions before its expected earnings announcement when it is in the top tercile of past announcement-period volume concentration
- exit: Sell at the open after the announcement (this INCLUDES the earnings gap for both after-close and pre-market reporters); the window ends at that open
- hold: 5-8 sessions
- frequency: daily during earnings season
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [Earnings Whispers - Scoring Expectations to Capture the Earnings Announcement Premium (vendor context: +1.08% for Score +5 vs +0.47% all others, about a week before to the open after, 11,385 events, Apr-2014 to Oct-2026)](https://www.earningswhispers.com/about-score); [NBER w13090 - Frazzini & Lamont](https://www.nber.org/system/files/working_papers/w13090/w13090.pdf)
- notes: RESOLVED (was BLOCKED): the proprietary Score has no free history and the 'Score > +2' threshold is not even on the vendor page, so the entry is rebuilt on public data (SimFin Publish Date from 2009, prices from daily_10years). Even the vendor's Score +5 edge is only ~0.6% over six sessions before costs on a vendor-selected sample. Weekly version of S052, run in parallel with it; the earlier claim of exiting 'before the reaction risk' was wrong and is removed.
- parameter_variants: entry offset {5-7 sessions before primary; 1 week before}; sort {volume concentration top tercile primary; none}
- data_needs: SimFin Publish Date 2009+ (DP-EARN); daily_10years (D1)
- review: RESOLVED

### S054 | PEAD Earnings-Surprise Drift (SUE / announcement-return, point-in-time)
- family: earnings and news
- concept: Stocks with the largest positive earnings surprises keep drifting for weeks as investors underreact. Surprise is measured consensus-free, as in Chan-Jegadeesh-Lakonishok (1996, verify): SUE from reported EPS and the announcement-window abnormal return (EAR).
- universe: liquid US stocks
- direction: long (and optional short leg for big misses)
- indicators: SUE = (EPS_q - EPS_{q-4}) / std of the last 8 such differences (seasonal random walk, SimFin quarterly EPS); EAR = cumulative return t-1..t+1 around the release minus SPY; trailing-cutoff percentile
- entry: After a positive surprise whose SUE is above the top-decile cutoff computed from the PREVIOUS four quarters' pooled surprises (trailing/expanding distribution; never the same-season cross-section, which contains announcements that have not happened yet), buy at the open of the first session after the reaction session (t+1 after a BMO report, t+2 after an AMC report; BMO/AMC from EDGAR 8-K 2.02 acceptance time, 2023-09+)
- exit: Hold 10 sessions (primary) or until the next earnings announcement; 21 sessions registered alternate
- hold: 10-21 sessions
- frequency: concentrated in earnings seasons (Jan, Apr, Jul, Oct)
- provenance: research-hypothesis
- status: active
- priority: P1
- sources: Chan, Jegadeesh & Lakonishok (1996) (verify); [NBER w13090 - announcement premium (about announcement-period returns, not drift)](https://www.nber.org/system/files/working_papers/w13090/w13090.pdf); Earnings Whispers page (announcement premium, vendor); Martineau 'Rest in Peace PEAD' (verify) on the large-cap decay after 2000
- notes: FIXED: the original 'top decile of the quarter's surprises' ranked announcements against all announcements in the season, including future ones (look-ahead). Neither the Earnings Whispers page nor NBER w13090 is about post-announcement drift. No consensus estimates are available, so SUE + EAR are the surprise measures. PEAD in large caps is thought to have weakened sharply after 2000: test the post-2016 sample before relying on it. Long-only first; the short leg for big misses is a separate experiment (G-09). Matched size/industry controls and subperiods required.
- parameter_variants: surprise measure {SUE top decile primary; EAR top decile; both (SUE and EAR top decile)}; hold {10 primary, 21, 42}; leg {long primary; short bottom decile separate}
- data_needs: SimFin quarterly EPS + Publish Date; EDGAR 8-K 2.02 timestamps 2023-09+ (DP-EARN); daily_10years (D1)
- review: RESOLVED

### S055 | Post-Earnings-Oversold Bounce
- family: earnings and news
- ohlcv_status: NOT_DAILY, DEFER
- concept: A stock that drops sharply on earnings but remains in a long-term uptrend (above 200-day SMA) typically recovers part of the drop within days — earnings-gap dislocation + regime filter.
- universe: S&P 500 stocks
- direction: long
- indicators: earnings-day return; SMA(200); RSI(2)
- entry: Reaction session defined from the filing timestamp (BMO: same day; AMC: next day). On the first session after the reaction session, if the stock fell more than 7% on the reaction AND close > SMA200 AND RSI(2) < 10; buy at close of that session (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound)
- exit: Close > 5-day SMA; sell at close of that session; max 5 sessions
- hold: 2-5 sessions
- frequency: several per earnings season
- provenance: research-hypothesis
- status: demoted
- priority: P3
- sources: [Be The Puck - ConnorsRSI pullback (plain ConnorsRSI rule, not an earnings strategy)](https://bethepuck.com/strategies/connorsrsi-pullback/); Chan's Estimize post is not about earnings reversion
- notes: Counter-hypothesis to S054, expected null: after a large negative surprise the literature expects drift DOWN, and S054's own premise is that surprises persist for weeks. Pair the long test with S054's short leg on the SAME events so the two hypotheses are tested against each other. Reaction day depends on BMO/AMC timing (2.02 timestamps 2023-09+ only, or the Alpaca news timestamp). A 7% drop with RSI(2) < 10 is many conditions for a handful of events per season.
- data_needs: daily_10years (D1) + EDGAR 8-K 2.02 timestamps / SimFin Publish Date (DP-EARN)
- review: DEMOTE (counter-hypothesis to S054) | source MISMATCH

### S056 | LLM Daily News Sentiment Long/Short
- family: earnings and news
- concept: arXiv 2412.19245 (Kirtac & Germano): daily LLM sentiment scores per stock from news; long the top-20% sentiment stocks, short the bottom-20%, rebalanced daily. The paper's best model is OPT (GPT-3-based; accuracy 74.4%, L-S Sharpe 3.05), then BERT 2.11 and FinBERT 2.07; the LM dictionary is lowest.
- universe: US stocks with daily news coverage in data/pit/news/news.parquet (Alpaca/Benzinga, 2023-09 onward, ~75k rows, ~1,900 symbols); live capture for forward evidence
- direction: long and short
- indicators: frozen FinBERT sentiment score (no fine-tuning on our window); quintile assignment; news-intensity z-score (see S060)
- entry: Each day, score each stock's news with FinBERT and trade long the top quintile / short the bottom quintile, value-weighted and size-neutralised, costs 10 bps per trade. Timing: news before 6am -> trade at that day's open and CLOSE THE SAME DAY (the paper's branch); 6am-4pm news -> trade at that day's close and exit the next trading day (primary overnight convention); after 4pm -> next open. Long-only first
- exit: Primary: next-open exit (overnight convention: close entry, next-open exit); the paper's same-day-close exit for pre-6am news is a registered branch
- hold: overnight to 1 session
- frequency: daily rebalance
- provenance: adapted
- status: active
- priority: P2
- sources: [arXiv 2412.19245 - Sentiment trading with large language models](https://arxiv.org/html/2412.19245); Lopez-Lira & Tang 2026 (Atlas 15.1); Roy & Roy 2026 (leakage audits)
- notes: CORRECTED: the earlier 'FINBERT tested best' is wrong (OPT is best). LEAKAGE: the models were fine-tuned on an 80% random split of articles across 2010-2023, so the held-out 20% is interleaved in time with training data, and OPT is a GPT-3-class model whose pretraining covers most of the sample: Sharpe 3.05 (mean daily L-S return 0.55%, ~140%/yr) is not credible out of sample. Data: Refinitiv news + CRSP (commercial), 965,375 articles after a 0.8 cosine novelty filter. GPT-4 headline scores predict drift only AFTER the model cutoff, concentrated in small stocks and negative news, with returns declining as LLM adoption rises (Lopez-Lira & Tang 2026). Therefore: frozen 2019 FinBERT, evaluate ONLY on post-cutoff data (news.parquet from 2023-09 is entirely post-cutoff for FinBERT) or live capture; masked-ticker and cross-model disagreement checks; report IC decay by month.
- parameter_variants: model {FinBERT frozen primary; LM dictionary baseline}; timing {close entry / next-open exit primary; pre-6am same-day branch}; legs {long-only primary; long-short}
- data_needs: data/pit/news/news.parquet (DP-NEWS, 2023-09+) + daily_10years (D1); 15-year archive NOT available
- review: RESOLVED

### S057 | LLM Top-Down Sector Allocation (forward test only)
- family: earnings, news and LLM (forward test)
- concept: arXiv 2503.09647: an LLM Ranking Agent assesses macro conditions, sets overweight/underweight targets across the 11 GICS sectors, then selects stocks within targeted sectors using aspect-based sentiment.
- universe: S&P 500 constituents (dynamic, point-in-time)
- direction: long
- indicators: LLM macro assessment (11-sector target weights); aspect sentiment scores
- entry: FORWARD TEST ONLY (no backtest): weekly, prompt a FROZEN LLM with current macro data and news to produce sector weights across the 11 GICS sectors; rebalance a paper portfolio to those weights, selecting the highest aspect-sentiment stocks within each overweighted sector; capture raw model outputs with vendor and ingestion timestamps
- exit: Rebalance on the fixed weekly schedule; evaluate after >= 6 months of paper trading before any capital
- hold: 5-21 sessions
- frequency: weekly rebalance
- provenance: research-hypothesis
- status: forward_test_only
- priority: P4
- sources: [arXiv 2503.09647 — Leveraging LLMs for Top-Down Sector Allocation](https://arxiv.org/html/2503.09647v5)
- notes: RESOLVED as a forward test: the paper's backtest covers only January 2019 through June 2019 ('due to constraints on computational resources and cost'; 300,000 articles; 8.79% return, Sharpe 2.51 vs cross-momentum -1.39% / -0.61). The earlier '2019-2024' was wrong. Six months, one regime, and a frontier LLM that knows 2019 macro is no evidence, and every rebalance costs LLM calls, so the entry is removed from the backtest queue and run only as a live paper experiment with timestamped capture (existing Alpaca news client in src/data/news_client.py), score IC decay, and costs.
- data_needs: live news capture (DP-NEWS live) + frozen model; no historical backtest possible
- review: RESOLVED

### S058 | News-Topic Conditioned Sentiment (feature design for S056)
- family: earnings, news and LLM (feature)
- concept: Re-scoped: Kiszczuk (SSRN 7665703) is an explainability study (291,260 news summaries for S&P 500 firms; topics via FinBERT embeddings, PaCMAP and hierarchical clustering; topics more than double the within-R2 from 7.17% to 15.17% beyond sentiment; firm-specific and fundamental topics add most; 'reproduced financial intermediary information' adds little). It has NO trading rule or return forecast. The usable idea is a FEATURE for S056: sentiment conditioned on news topic.
- universe: rated stock universe (paper uses news-covered stocks)
- direction: n/a (feature evaluation; feeds S056)
- indicators: FinBERT embeddings of headline/summary; hierarchical clustering into topics; topic-conditioned FinBERT sentiment
- entry: No standalone trading rule. Evaluation spec: cluster news items into topics, keep firm-specific/fundamental topics, drop analyst-rating-repeat topics, and test whether topic-conditioned sentiment has a higher information coefficient (IC) for next-session returns than raw sentiment on news.parquet (2023-09+); if yes, feed it into S056's score
- exit: n/a
- hold: n/a
- frequency: n/a (feature)
- provenance: research-hypothesis
- variant_of: S056
- status: active
- priority: P3
- sources: Attached paper (user upload) - Opening the Black Box: Which News Explain LLM-Based Stock Ratings? (SSRN 7665703); the attached PDF is a single title page and SSRN also lists 1 page, so only the abstract was used
- notes: RESOLVED by re-scope: the earlier 'rating upgrade + positive news -> buy' was the catalog's own placeholder, not derived from the paper. The full paper (if rules are wanted) must be requested from the author/SSRN.
- data_needs: data/pit/news/news.parquet (DP-NEWS, 2023-09+)
- review: RESOLVED

### S059 | Insider Cluster Buying
- family: earnings and news
- concept: Open-market insider buying by several distinct insiders signals undervaluation. Evidence base: a Northwestern thesis by John Ryle, CFA, published as a guest post on Chan's blog (~650,000 insider open-market buys, a random-forest model, 3-month return relative to the Russell 3000).
- universe: US stocks with Form 4 filings (SEC Insider Transactions Data Sets, 2006+)
- direction: long
- indicators: count of distinct insiders with open-market buys (transaction code P) in trailing 21 sessions, excluding routine (same-month-each-year) and 10b5-1 trades; filing acceptance time
- entry: 3+ distinct insiders made open-market purchases (code P, non-routine, non-10b5-1) in the last 21 sessions, timed by Form 4 filing acceptance time (not trade date); buy at the close of the trigger session after the filing. Registered variant: additionally zero insider sells in the 21 sessions (very restrictive)
- exit: Hold 63 sessions or exit on 20-day low; evaluate on relative return vs matched size/industry; effective sample size = sample length / 63
- hold: ~63 sessions (longer swing; flag)
- frequency: weekly across the market
- provenance: research-hypothesis
- status: active
- priority: P2
- sources: [John Ryle (guest post on Ernest Chan's blog) - Building an Insider Trading Database and Predicting Future Equity Returns](http://epchan.blogspot.com/2017/07/building-insider-trading-database-and.html); Cohen, Malloy & Pomorski (2012) opportunistic vs routine insider trades (verify)
- notes: CORRECTED: this is not Chan's research. The post documents a guest thesis whose random forest looked good in validation but in the 2014-2015 test 'the relationship broke down substantially... the actual returns on all remaining prediction deciles appeared no better than random; relative returns were negative for every decile'. No cluster rule is tested there, so the cluster rule, thresholds and exits are entirely the catalog's; the only documented evidence points to NO edge out of sample. Form 4 is not in the repo's EDGAR table (8-K, 10-Q, 13G, S-x only).
- parameter_variants: cluster {>= 3 insiders primary; >= 3 and zero sells}; hold {63 sessions primary; 20-day-low exit}
- data_needs: SEC Form 4 insider transactions bulk TSV 2006+ (DP-FORM4); daily_10years (D1)
- review: RESOLVED

### S060 | News-Intensity Overnight Drift (conditioning feature of S056)
- family: earnings and news
- ohlcv_status: NOT_DAILY, DEFER
- concept: Stocks with unusually high pre-market news flow and positive sentiment gap up and keep drifting for 1-3 days. Re-scoped as a news-intensity z-score conditioning feature of S056 and tested on the available archive.
- universe: stocks with pre-market news coverage
- direction: long
- indicators: pre-market news-item count vs trailing 20-day average (z-score; require a minimum count of 3 items given the sparse archive); frozen FinBERT polarity; gap vs ATR(20)
- entry: Pre-market news count > 3 x its 20-day average (and >= 3 items) AND net frozen-FinBERT sentiment positive AND open gap < 2 x ATR (not exhausted); buy at the close of the 09:31 bar of t (an open-derived signal cannot fill at the open)
- exit: Sell at close of t+2 or trail 2-bar low; max 3 sessions
- hold: 1-3 sessions
- frequency: daily opportunities
- provenance: research-hypothesis
- variant_of: S056
- status: active
- priority: P4
- sources: [arXiv 2412.19245 — Sentiment trading with LLMs](https://arxiv.org/html/2412.19245)
- notes: RESOLVED as a conditioning feature: no evidence exists for the standalone rule and the cited paper has no overnight-news result. The archive in the repo (news.parquet, ~75k rows from 2023-09, ~1,900 symbols) is too sparse for 15-year counts but is enough for a post-cutoff research test; item-count z-scores are noisy at ~40 items per symbol, hence the minimum-count requirement. Overlaps S056/S023. Same look-ahead hygiene as S056 (frozen FinBERT, post-cutoff windows).
- data_needs: data/pit/news/news.parquet with timestamps (DP-NEWS, 2023-09+); daily_10years (D1) + 09:31 minute bars
- review: RESOLVED

---

# Family 6 — Volatility Regime and Market Timing

*Why this family works: volatility is mean-reverting and regime-persistent. VIX spikes mark panic bottoms (buy when fear peaks), volatility-compression precedes expansion (trade the breakout), and the vol term structure signals risk appetite. Chan's volatility term-structure post shows how variance-scaling across intervals (Hurst exponent) diagnoses whether a market is trending or mean-reverting — regime-switching the strategy type accordingly.*

### S061 | VIX Spike Mean Reversion (SPY)
- family: volatility regime
- ohlcv_status: NOT_DAILY, DEFER
- concept: VIX is strongly mean-reverting; extreme spikes (VIX > 30, or > 1.5 × its 20-day average) coincide with panic selling that reverts as vol normalizes. Buying SPY at extreme VIX buys maximum fear.
- universe: SPY
- direction: long
- indicators: VIX level; VIX 20-day average; RSI(2); SMA(5)
- entry: VIX(t) > T AND VIX(t) > 1.5 x 20-day average of VIX AND SPY RSI(2) < 10; buy at close of t (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound). T = 30 primary; registered {25, 35}. Registered confirmation: VIX has turned down (VIX close < prior VIX close)
- exit: VIX < 20 OR close > 5-day SMA (the SMA5 leg will dominate; the VIX leg rarely triggers); sell at close of that session; max 10 sessions
- hold: 2-10 sessions
- frequency: 2-6 trades per year (raise frequency by running on QQQ, IWM, and sector ETFs)
- provenance: research-hypothesis
- status: active
- priority: P2
- sources: [Papers With Backtest - Using VIX to trade SPY (general article: buy SPY when VIX is high, sell when low; no thresholds or results)](https://paperswithbacktest.com/strategies/using-vix-to-trade-spy-and-sp-500)
- notes: The thresholds (VIX > 30, 1.5x 20-day average, RSI(2) < 10) are the catalog's own. Independent episodes with VIX > 30 in 2016-2026 are about 4-6 (Feb-2018, Dec-2018, Mar-Jun 2020, 2022, Apr-2025): a triple-conditioned rule has a handful of events, so report an event table with dates and make no significance claim. In sustained crises (2008-style) buying the first VIX > 30 close loses for months and the 2016+ sample omits that failure case, so validate pre-2016 with CBOE history (VIX from 1990, SPY 1993+) under the same rules. The 'VIX turned down' confirmation follows the Connors CVR3 idea (VIX bar rules: low above the 10-day MA, close >= 10% above it, black candle; sell on the mirror).
- parameter_variants: VIX threshold {30 primary, 25, 35}; confirmation {none primary; VIX close < prior close}; universe {SPY primary; QQQ, IWM, sector ETFs}
- data_needs: CBOE VIX daily 1990+ (DP-VIX) + SPY total-return series (DP-ETF)
- review: RESOLVED

### S062 | VIX/VXV Ratio Spike (Term Structure)
- family: volatility regime
- concept: When the 30-day VIX exceeds the 3-month VIX3M (formerly VXV; ratio >= 1.10), near-term fear is extreme relative to the quarter view. Schaeffer's: 25 de-overlapped signals over nine years, average +1.94% (median 2.90%, 17 wins) per one-month hold, one 16.37% drawdown in 2008.
- universe: SPY
- direction: long
- indicators: VIX / VIX3M ratio (VIX3M is the 2017 rename of VXV: splice the series; VIX9D/VIX is a DIFFERENT quantity and is not a substitute)
- entry: VIX/VXV ratio closes at or above 1.10; buy SPY at close of t
- exit: Hold one month (21 sessions); sell at close
- hold: 21 sessions
- frequency: ~3 signals per year
- provenance: source-specified
- status: active
- priority: P3
- sources: [Schaeffer's Research — Another Way to Trade Volatility (VXV)](https://www.schaeffersresearch.com/content/analysis/2015/12/03/another-way-to-trade-volatility)
- notes: Selectively quoted earlier: across ALL 113 days with ratio >= 1.10 the average one-month forward return was 0.22% vs 0.75% for randomly timed holds (median 2.69% vs 1.49%), the correlation of the ratio with forward returns is 'an uninspiring 0.0324', and the author attributes the pattern to data overlaps intersecting with 2008; data begin mid-2006. 25 de-overlapped incidents dominated by 2008 is not a validated edge; since 2016 the incidents are likely under 10. Report BOTH all-days and incident-based statistics, include 2008 through CBOE history so the drawdown is in sample, and trial-count the 1.10 threshold. Signal counts must not overlap (Schaeffer's incident convention); run in parallel with S061 as one vol-timing basket.
- parameter_variants: ratio threshold {1.10 primary; 1.05, 1.15}; statistics {all signal days and de-overlapped incidents}
- data_needs: CBOE VIX + VIX3M/VXV history 2006+ (DP-VIX) + SPY total-return series
- review: RESOLVED

### S063 | VIX Moving-Average Crossover Regime
- family: volatility regime
- concept: A short-term VIX MA crossing a long-term VIX MA marks vol-regime turns: falling short-term vol (risk appetite returning) → hold equities; rising → de-risk.
- universe: SPY (or QQQ for higher beta)
- direction: long / flat
- indicators: 20-day VIX MA; 50-day VIX MA
- entry: 20-day VIX MA < 50-day VIX MA AND VIX falling (VIX(t) < VIX(t-5)); hold SPY
- exit: 20-day VIX MA > 50-day VIX MA; move to cash
- hold: weeks to months
- frequency: regime switches ~4-8 per year
- provenance: research-hypothesis
- status: demoted
- priority: P3
- sources: [Papers With Backtest - Using VIX to trade SPY (shows a 20/50 VIX moving-average crossover as an example with sample code; no backtest results)](https://paperswithbacktest.com/strategies/using-vix-to-trade-spy-and-sp-500)
- notes: The 'VIX falling over 5 days' condition is the catalog's. A VIX trend signal is mostly a trend-of-SPY signal and will be highly correlated with the 200-day SMA regime (S069/S089): compare against SMA200 timing and S069, and report incremental information by regressing the VIX-MA signal returns on the SMA200 signal. Combine with S047 as a dual-gate overlay only if incremental.
- data_needs: CBOE VIX history (DP-VIX) + SPY (DP-ETF)
- review: DEMOTE | source GENERIC

### S064 | Hurst Exponent Regime Switch
- family: volatility regime
- concept: Chan's volatility term-structure method: compute variance of log returns at multiple horizons; the slope of log-variance vs log-horizon gives the Hurst exponent H. H < 0.5 = mean-reverting market (run mean-reversion strategies); H > 0.5 = trending (run momentum strategies).
- universe: SPY, QQQ, and any traded ETF
- direction: long
- indicators: Hurst exponent via a variance-ratio regression (log-variance vs log-horizon, horizons 1-64 days) with block-bootstrap confidence intervals; strategy-selection gate (inherits the indicators of S001 and S036)
- entry: Each week, estimate H on the trailing 504 daily log returns (variance at horizons 1, 2, 4, 8, 16, 32, 64 days) with a block-bootstrap CI; switch ONLY when the CI excludes 0.5: H CI entirely < 0.5 -> run RSI(2) mean reversion (S001) on the ETF; H CI entirely > 0.5 -> run Donchian 20 breakout (S036); otherwise hold cash. First evaluate regime persistence (autocorrelation of the regime label)
- exit: Managed by the underlying strategy; re-evaluate regime weekly
- hold: 1-10 sessions
- frequency: weekly regime decision; trades from the underlying strategy
- provenance: adapted
- status: demoted
- priority: P3
- sources: [Ernest Chan — Mean reversion, momentum, and volatility term structure](http://epchan.blogspot.com/2016/04/mean-reversion-momentum-and-volatility.html)
- notes: Chan's post (April 2016): SPY intraday H = 0.494 +/- 0.003, daily H (1-256 day variance, 2013-2016) = 0.469 +/- 0.007 ('significantly mean reverting'); USO daily H 0.56 (trending); GLD daily H 0.469; transition from momentum to mean reversion near 16-32 days. The post classifies INSTRUMENTS and does not propose a time-varying gate. With SPY's H near 0.47 the old fixed bands (< 0.45 / > 0.55) mostly output cash, and on a rolling 504-day window with horizons <= 64 the standard error is several times larger than the post's 0.007 (overlapping returns leave ~8 independent samples at the longest horizon). The realistic use is instrument-level classification (USO trending; GLD/SPY mean-reverting). The entry mixes two strategies whose own edges are unproven, adding trials multiplicatively.
- parameter_variants: gate {CI excludes 0.5 primary; fixed bands 0.45/0.55}; use {rolling regime gate; static instrument-level classification}
- data_needs: ETF total-return daily series (DP-ETF)
- review: DEMOTE | source PARTIAL

### S065 | Realized-Vol Compression Breakout (compression cluster champion candidate)
- family: volatility regime
- concept: When 5-day realized vol falls below half the 20-day realized vol, the market is compressing; compression resolves into directional moves — position for expansion via straddle-like longs (buy the breakout in either direction).
- universe: SPY, QQQ
- direction: long and short
- indicators: 5-day realized volatility; 20-day realized volatility; 20-day Donchian high/low
- entry: RealizedVol(5) < 0.5 x RealizedVol(20); place a buy stop at the 20-day high and a sell stop at the 20-day low, valid for the NEXT 5 sessions (GTC for 5 sessions, then cancel); first triggered enters; if both trade in the same session take the one nearer the session open, if equidistant skip. Stop fills follow max(open, level) for buys / min(open, level) for sells
- exit: Opposite 10-day extreme or after 5 sessions
- hold: 1-5 sessions
- frequency: ~1-2 trades per month per instrument
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [Ernest Chan — volatility term structure post](http://epchan.blogspot.com/2016/04/mean-reversion-momentum-and-volatility.html)
- notes: Chan's post is about Hurst/variance scaling, not compression breakouts. RV(5) < 0.5 x RV(20) is frequent (roughly 10-15% of days) so signals cluster inside quiet regimes. Same mechanism as S038/S094/S095 (G-10); the Bollinger squeeze scored worst of all rules on Be The Puck (30/371). Run as the volatility-compression cluster champion candidate with S094 and S095 as variants.
- parameter_variants: order validity {5 sessions primary; 1, 3}; tie-break {nearer to open primary}
- data_needs: ETF daily bars (DP-ETF)
- review: RESOLVED

### S066 | Zweig Breadth Thrust
- family: volatility regime
- concept: Martin Zweig's thrust: when the 10-day EMA of NYSE advance ratio (advances / (advances + declines)) rises rapidly from below 0.40 to above 0.615, a new bull phase is starting — buy immediately.
- universe: SPY; breadth built from our own universe (advancers/decliners among liquid_top_N and all listed common stocks), NYSE A/D data are not in the repo
- direction: long
- indicators: 10-day EMA of NYSE advance ratio; threshold pair (0.40 → 0.615)
- entry: 10-day EMA of advance ratio < 0.40 within the last 10 sessions AND crosses above 0.615 today; buy SPY at close of t
- exit: Close < 50-day SMA or new thrust signal refreshes
- hold: weeks to months
- frequency: rare (a handful per decade) — combine with other breadth signals
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [GitHub - phuazz/breadth-thrust-etf (a constituent-breadth thrust, NOT the classic NYSE advance/decline Zweig thrust; its author concludes the S&P 500 is the wrong universe and the signal does not generalise)](https://github.com/phuazz/breadth-thrust-etf); Zweig / Morris classic thresholds (StockCharts; not in the cited repo)
- notes: The phuazz repo computes '% of constituents above their 50-day MA crossing from below 50% to above 80% within 20 trading days' from point-in-time iShares holdings and Norgate delisted data; on the S&P 500 (CSP1) its tuned config gives Sharpe +0.59 at the 43rd percentile of the random-entry null; energy and financials Sharpe <= 0.09. The catalog's classic Zweig thresholds are not in `sources`. Zweig thrusts are rare (N ~ 3-5 in 2016+). If kept: build breadth from our own universe, test the classic thresholds on pre-2016 data where available; otherwise drop.
- data_needs: breadth from own universe (D1); pre-2016 NYSE A/D history (external, optional)
- review: DEMOTE | source MISMATCH

### S067 | Relative Sector Breadth Rotation
- family: volatility regime
- concept: phuazz sleeve A: each sector's breadth (share of constituent stocks above their own short-term trend) minus the cross-sectional mean isolates sectors with unusual internal strength; overweight them.
- universe: sector ETFs on the PIT sector universe table (11 GICS sectors); breadth computed from our own PIT stock universe mapped to sectors
- direction: long
- indicators: per-sector breadth = share of liquid_top_N stocks (SimFin industry code mapped to sector) above their own 50-day SMA; cross-sectional z-score
- entry: Weekly (Friday close, Monday execution), rank sectors by breadth z-score; buy the top half of sectors (7 of 14 in the source; ~5-6 of 11 here), weighted by positive relative share
- exit: Rebalance weekly; positions exit when they leave the top 7
- hold: 5+ sessions
- frequency: weekly rebalance
- provenance: adapted
- status: active
- priority: P3
- sources: [GitHub — phuazz/breadth-thrust-etf (sleeve A)](https://github.com/phuazz/breadth-thrust-etf)
- notes: RESOLVED (was BLOCKED): the source breadth feature needs point-in-time ETF constituent holdings (the US iShares endpoint has been blocked since 2026-05-16; the repo moved to UK UCITS endpoints and Norgate). It is approximated with our own PIT stock universe mapped to sectors via SimFin industry codes; using today's holdings would be look-ahead. The repo's reported numbers are in-sample (walk-forward Sharpe for sleeve A +1.00; blend Sharpe standard error +/-0.4 over 7.5 years of weekly data; weighting, universe and cost calibration applied across the entire backtest): do NOT use them as a target. 'Weighted by positive relative share' is the source's phrase and is specified here as weight proportional to max(z, 0). Fill at next open per the shared spec.
- data_needs: daily_10years (D1) + SimFin industry-to-sector map; sector ETFs (DP-ETF)
- review: RESOLVED

### S069 | TLT Crash-Hedge Rotation
- family: volatility regime
- concept: When equities lose their 200-day SMA (bear regime), Treasuries historically rally on flight-to-safety; rotating SPY → TLT on the regime break monetizes the negative correlation.
- universe: SPY, TLT
- direction: long
- indicators: SMA(200) on SPY
- entry: If SPY close > 200-day SMA, hold SPY; else hold TLT
- exit: Switches at each month-end (or daily) when the regime flips
- hold: weeks to months
- frequency: regime switches ~1-3 per year
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [BestFolio — Buy the Dip (regime-switch design)](https://bestfolio.app/strategies/buy-the-dip)
- notes: Simple regime rotation; known failure modes are 2013-2015 and 2022 (rates and stocks falling together; report the 2022 stock-bond correlation sign flip by subperiod). TLT must be a total-return series; test a cash alternative to TLT. Highly overlapping with S047 (G-10).
- data_needs: SPY, TLT, BIL total-return daily series (DP-ETF)
- review: NO FINDINGS (strategy-specific) | source GENERIC

### S070 | QQQ Regime Switches with Real TQQQ History (public rules only)
- family: volatility regime
- concept: The BestFolio 'Buy the Dip' strategy (Reddit community) uses hysteresis switches on QQQ: oversold (RSI(14) < 29.7 -> 100% TQQQ regardless of the 200-SMA). Only the oversold rule and the 200-SMA regime are public; the overheated/inverse and momentum-mix rules are Pro-only, so this entry implements only the public part and restricts itself to the real TQQQ history.
- universe: QQQ (signal); TQQQ (execution, real history from 2010-02-11 only), QQQ, SHY/BIL
- direction: long
- indicators: RSI(14) on QQQ with hysteresis (29.7 in / 30.3 out); SMA(200) on QQQ
- entry: Oversold switch on (RSI(14) < 29.7) -> buy TQQQ REGARDLESS of the 200-SMA (as on the source page), with TQQQ exposure capped at a registered fraction (1.0, 0.5) of the account; otherwise above SMA(200) -> hold QQQ (unlevered); below SMA(200) -> SHY/BIL. The source's overheated (SQQQ) leg and momentum mix (USMV/UGL/ZROZ/IEI) are NOT implemented
- exit: Oversold switch off (RSI(14) > 30.3) -> exit TQQQ to the regime-appropriate asset (QQQ or SHY/BIL)
- hold: days to weeks
- frequency: several switches per year
- provenance: research-hypothesis
- status: active
- priority: P4
- sources: [BestFolio — Buy the Dip Strategy by Reddit community](https://bestfolio.app/strategies/buy-the-dip)
- notes: CORRECTED numbers: BestFolio reports 26.8-26.9%/yr over the full simulated history since 1985 with a worst fall of -76.2% on 26 Oct 1987 (a drawdown measure on SIMULATED 3x QQQ, not a single-day loss), and 30.0%/yr with worst fall -20.4% (25 Aug 2015) since TQQQ/SQQQ launched on 11 Feb 2010; 10 bps one-way costs; stand-in simulations before fund inception (TQQQ = simulated 3x QQQ before 2010-02-11). The earlier entry ('oversold AND above 200-SMA', 24.9%/yr, '-76% single-day tail risk') contradicted the page. Most of the published history is a simulation of 3x products: leveraged-ETF path dependence, volatility decay and financing dominate, so use REAL TQQQ history only. With a $5,000 paper account a -70% drawdown is existential: cap exposure.
- parameter_variants: TQQQ cap {1.0, 0.5 of account}; oversold exit {RSI > 30.3 primary}; non-oversold asset {QQQ above SMA / SHY below primary}
- data_needs: TQQQ and QQQ daily bars from 2010-02-11 (DP-ETF fetch list must add TQQQ); SHY/BIL
- review: RESOLVED

---

# Family 7 — Pairs and Spread Strategies

*Why this family works: economically linked instruments (twin ETFs, commodity-equity pairs, gold/silver) have stationary spreads; deviations from the spread's mean are traded with defined risk because the hedge removes market beta. The Springer ETF pairs study (2000-2024) found cointegration-based pairs profitable when the relationship is re-verified monthly and traded within windows where cointegration holds.*

### S071 | Cointegrated ETF Pair Z-Score
- family: pairs and spreads
- concept: The Springer study's method: form ETF pairs (price ratio), test stationarity monthly, compute a rolling z-score, enter at +/-2, exit at the paper's thresholds; re-form pairs when cointegration breaks. The paper's own verdict is modest: short trading windows where cointegration holds limit long-term profitability.
- universe: pre-registered pair list (identify the paper's 30-pair list from its appendix, or fix a list before the first run); ETFs across equities, bonds, commodities, real estate, international
- direction: long and short (pair)
- indicators: Engle-Granger on log prices with an estimated hedge ratio (the paper's ADF is on the price ratio); 12-month rolling mean and std of the spread; z-score
- entry: Pair passes the cointegration test (p < 0.05) on the formation window; |z| > 2 -> short the rich leg / long the cheap leg. Equal-SHARE counts (paper) and dollar-neutral hedge-ratio sizing both registered. Pair selection and trading windows are disjoint: formation on months t-24..t-12, test on t-12..t
- exit: Close shorts when z falls to 0.75 and longs when z rises to -0.5 (Avellaneda & Lee, as in the paper); monthly re-verification closes the pair if cointegration breaks; +/-3 stop is a registered variant; costs on both legs
- hold: 5-40 sessions
- frequency: several open pairs at any time; new entries weekly
- provenance: source-specified
- status: active
- priority: P3
- sources: [Springer — Cointegration-based pairs trading: ETFs 2000-2024](https://link.springer.com/article/10.1057/s41260-025-00416-0)
- notes: Springer (2000-2024, 30 ETF pairs, 12-month formation, monthly ADF on the ratio, rolling z-score) fetched. The SPY/IVV example (cointegrated for 24 years) yielded 'a modest total profit of 1.01% and a Sharpe ratio of 0.28' at z = 2; lowering the threshold raises profit and Sharpe but also volatility and kurtosis. The earlier z = 0 exit and +/-3 stop differed from the paper. Selecting pairs and testing on the same 12 months with 30 pairs x monthly 5% tests produces ~1.5 false positives per month.
- parameter_variants: exits {z 0.75 / -0.5 primary (paper); z = 0 with +/-3 stop}; sizing {equal-share primary (paper); dollar-neutral with hedge ratio}; entry threshold {2.0 primary; lower thresholds}
- data_needs: ETF total-return daily series for the pair list (DP-ETF)
- review: RESOLVED

### S072 | Gold/Silver Ratio Mean Reversion
- family: pairs and spreads
- concept: The gold/silver price ratio is mean-reverting over decades; Quantified Strategies tested ratio-based switching on GLD/SLV (buy on strength signal).
- universe: GLD, SLV
- direction: long and short (pair)
- indicators: GLD/SLV price ratio; 60-day rolling mean and std; z-score
- entry: Hedge ratio from a rolling regression of log GLD on log SLV (total-return series); spread z-score (60-day) > +2 -> long SLV / short GLD; z < -2 -> reverse
- exit: z crosses 0; stop at ±3
- hold: 10-60 sessions
- frequency: a few trades per year
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [Quantified Strategies — Gold Silver chart ratio strategy](https://www.quantifiedstrategies.com/gold-silver-chart-ratio-strategy/)
- notes: Quantified Strategies' exact parameters are members-only and its own verdict ('erratic equity curve, nowhere near a tradable strategy') already argues against it. The ratio trends for months (2018-2020 moved ~80 -> 125 -> 65), so +/-3 stops get hit. GLD and SLV are in daily_10years already. Run once with a regression hedge ratio and accept the null.
- data_needs: GLD, SLV in daily_10years (D1)
- review: DEMOTE | source UNVERIFIED (QS members-only)

### S073 | GDX/GDXJ Miners Spread
- family: pairs and spreads
- concept: Senior vs junior gold miners (GDX vs GDXJ) share the gold driver but differ in leverage/liquidity; the spread between them mean-reverts as risk appetite in the gold complex normalizes.
- universe: GDX, GDXJ
- direction: long and short (pair)
- indicators: GDX/GDXJ price ratio; 60-day rolling z-score
- entry: z > +2 → long GDXJ, short GDX; z < −2 → reverse
- exit: z crosses 0; stop ±3
- hold: 10-60 sessions
- frequency: a few trades per year
- provenance: research-hypothesis
- status: dropped
- priority: P4
- sources: [Quantified Strategies — Gold Silver chart ratio strategy (pair method)](https://www.quantifiedstrategies.com/gold-silver-chart-ratio-strategy/)
- notes: DROPPED: GDXJ holds companies that graduate into GDX and has had methodology changes (the earlier '2009 index change' claim could not be verified; GDXJ launched Nov 2009), so the ratio drifts structurally: convergence is structural, not mean reversion. Only the z-score machinery is shared with S072, so `variant_of` is removed. Neither ETF is in the cache. May be included in the S071 pair-universe study if desired.
- data_needs: GDX/GDXJ not in cache
- review: DROP | source GENERIC

### S075 | Stock-vs-Sector Residual Reversal
- family: pairs and spreads
- ohlcv_status: DAILY, DELTA
- concept: For each stock compute its residual 5-day return versus its industry peers (stock return minus beta x peer return); extreme negative residuals revert because nonfundamental, sector-wide or sentiment-driven selling is misattributed to the name. NY Fed SR 513: only the within-industry residual component of reversal is significant and a strategy built on it earns ~3x the risk-adjusted return of the standard sort.
- universe: liquid_top_N US stocks with PIT industry/sector mapping (SimFin industries or SIC) and equal-weight peer returns (sector ETF where available)
- direction: long
- indicators: 5-day residual return (stock - beta x peer return; 60-day beta estimated through t-1); SMA(200) on the stock (on/off)
- entry: Residual 5-day return in the bottom decile AND stock close > SMA200 (registered on/off); buy at close of t (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound); skip-day variant registered
- exit: Time stop: sell after 5 sessions (primary, fixed horizon); or when the 5-day residual > 0 (noisy), max 10 sessions
- hold: 2-10 sessions
- frequency: daily opportunities across the universe
- provenance: adapted
- status: active
- rust_status: IMPLEMENTED
- priority: P1
- sources: [NY Fed Staff Report 513 - Decomposing Short-Term Return Reversal (within-industry residual)](https://www.newyorkfed.org/medialibrary/media/research/staff_reports/sr513.html); the Springer ETF-pairs paper cited earlier is about ETF pairs, not stock residuals, and is no longer cited
- notes: Best-supported reversal construct. Long-only is a slice of a market-neutral idea, so the beta of a bottom-decile residual basket is not zero: the null is the vol-matched equal-weight universe. Report raw reversal (S027) next to it as the control and exclude/report earnings-window signals (+/-3 sessions, G-12). The market-neutral flavour of the reversal family.
- parameter_variants: exit {5-session time stop primary; residual > 0 with max 10 sessions}; SMA200 filter {on primary, off}; skip {0 days, 1 day}; peers {SimFin industry equal-weight primary; sector ETF}
- data_needs: daily_10years (D1) + SimFin industry map + earnings flag (DP-EARN)
- review: KEEP | source PARTIAL (cited paper was wrong; NY Fed SR 513 now cited)
- rust_implementation: Registered as `s075_residual_reversal` with 30 named variants and daily OHLCV signal/trade tests. Peer groups use the available current SimFin snapshot, not point-in-time historical industry membership; this is a known limitation. No Rust real-data performance results have been run.
- implementation_results: `src/strategies/s075_residual_reversal`; SimFin sector-level (12 groups, matching the paper's 11 SIGC industries) equal-weight leave-one-out peers (no sector ETFs in the cache; `peer_level: industry` reproduces the original 74-industry version), liquid top 500 universe, long-only, research scale, 15 bps/side; dev < 2024-01-01 <= test. `big10` = `min_abs_resid: 0.10` (only residual <= -10% entries; dev-selected winner of 15 variants; dev selection haircut ~0.34 log; test skill not significant).

  | variant | dev total (log) | dev skill | dev Sharpe | dev max DD | test total (log) | test skill | test Sharpe | test max DD | trades (dev+test) | avg positions |
  |---|---|---|---|---|---|---|---|---|---|---|
  | baseline | -0.21 | -0.62 | -0.11 | -48% | +0.09 | -0.17 (p=0.74) | 0.25 | -25% | 18,299 | 49-53 |
  | big10 | +0.04 | -0.09 | +0.11 | -23% | +0.13 | +0.07 (p=0.38) | 0.35 | -19% | 3,163 | 7-15 |

  Reproduce: `python run.py --strategy s075_residual_reversal --data-dir data/cache/daily_10years --variant baseline,big10`

### S076 | Twin-ETF Convergence (SPY/IVV/VOO)
- family: pairs and spreads
- concept: SPY, IVV, and VOO track the same index; deviations between them are pure supply/demand noise that converges — the cleanest statistical arbitrage available to daily-bar traders.
- universe: SPY, IVV, VOO
- direction: long and short
- indicators: SPY/IVV price ratio; 20-day rolling z-score
- entry: z > +2 → long IVV, short SPY; z < −2 → reverse
- exit: z crosses 0; max 10 sessions
- hold: 1-10 sessions
- frequency: occasional; small edge per trade
- provenance: research-hypothesis
- status: dropped
- priority: P4
- sources: [Springer — ETF pairs trading (method)](https://link.springer.com/article/10.1057/s41260-025-00416-0)
- notes: DROPPED (keep only as a one-off cost sanity check): the Springer SPY/IVV example is the cited study's own poorest case (total profit 1.01%, Sharpe 0.28); the SPY/IVV/VOO spread is a few basis points of noise that authorised participants arbitrage within the day, two legs each with a spread cost more than the edge on daily bars, and ex-dividend timing differences add ratio jumps. Run once on 2016+ data as an assertion that the cost model produces a loss.
- data_needs: IVV, VOO, SPY not in cache (DP-ETF)
- review: DROP (one-off cost sanity check) | source OK

### S077 | Energy Pair: XLE/USO Spread
- family: pairs and spreads
- concept: Energy equity ETFs (XLE) and crude oil (USO) are linked by the oil price; the equity leg leads/lags on equity-market sentiment, creating a mean-reverting spread.
- universe: XLE, USO
- direction: long and short
- indicators: XLE/USO ratio; 60-day rolling z-score
- entry: z > +2 → long USO, short XLE; z < −2 → reverse
- exit: z crosses 0; stop ±3
- hold: 10-40 sessions
- frequency: a few trades per year
- provenance: research-hypothesis
- status: dropped
- priority: P4
- sources: [Springer — ETF pairs trading (method)](https://link.springer.com/article/10.1057/s41260-025-00416-0)
- notes: DROPPED: an equity sector ETF and a futures-roll ETF are not tied by a stationary ratio: USO suffers contango roll decay and changed its futures-roll methodology in 2020 (negative WTI, April 2020), so the ratio has a drift a 60-day z-score cannot track, and XLE loads on equity beta so the hedge ratio must be beta-adjusted, not 1:1. If ever run: regression hedge ratio and exclude Mar-Jun 2020.
- data_needs: XLE not in cache; USO in cache
- review: DROP | source GENERIC

### S078 | Chan-Style Statistical Arbitrage Basket
- family: pairs and spreads
- concept: Chan's mean-reversion work: on the most recent one-year SP1500 data (2008) his Example 3.8 strategy showed Sharpe 4.8 without cost and 3.5 after 10 bps round trip, on data unseen when the book was written. A market-neutral basket of mean-reverting stock positions is the research-scale idea; ONE year of an extreme-volatility regime is not 'enduring profitability'.
- universe: SP1500 stocks
- direction: long and short
- indicators: 90-day mean-reversion score (e.g., Ornstein-Uhlenbeck half-life from daily closes); beta to SPY; position count
- entry: Stage 1 (simple version first): daily cross-sectional reversal, long-short, dollar- and beta-neutral, opening/closing-auction execution. Stage 2 (only then): each week compute each stock's mean-reversion half-life (Ornstein-Uhlenbeck over 90 days), take the fastest 50, long the bottom-quintile residual vs beta-adjusted market and short the top quintile, weights proportional to 1/half-life, dollar- and beta-neutral
- exit: Positions exit when residuals cross 0; weekly rebalance
- hold: 5-15 sessions
- frequency: continuous portfolio, daily position turnover
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [Ernest Chan — The enduring profitability of mean-reversion strategies](http://epchan.blogspot.com/2008/12/enduring-profitability-of-mean.html)
- notes: CORRECTED: Chan's post (9 Dec 2008) reports Sharpe 4.8 (no cost) / 3.5 (10 bps) on one year (2008), not '> 5 before costs', and Example 3.8 belongs to the 2008 book (Quantitative Trading), not Algorithmic Trading (2013). Chan's example is a simple long-short reversal portfolio (verify in the 2008 book); the OU half-life / 50-stock weekly basket is the catalog's design. A 50-name dollar-neutral, beta-neutral basket is not implementable at $5,000 (research scale only) and needs borrow (G-09). Chan stresses opening-auction execution. Chan's stated costs: 10 bps round-trip for SP1500 large caps (try 15 bps).
- parameter_variants: stage {simple daily reversal long-short primary; OU half-life version}; cost {10 bps round trip, 15 bps}
- data_needs: daily_10years (D1) + borrow assumptions; research scale only
- review: RESOLVED

---

# Family 8 — Calendar Strategies

*Why this family works: predictable flows cluster on the calendar — month-end pension/payroll contributions, pre-holiday risk reduction, pre-FOMC positioning. These flows create repeatable return patterns at known dates, which is ideal for daily-bar backtesting (no signal estimation, only dates).*

### S079 | Turn-of-the-Month Effect (window grid)
- family: calendar
- concept: All excess US equity market return over 1987-2005 (and 1897-2005) concentrated in the 4-day turn-of-the-month window (last trading day + first three); the other 16 days earned nothing on average (SSRN 917884).
- universe: SPY, IWM and RSP (the effect is documented on equal-weight/CRSP indexes); sector ETFs to raise trade count
- direction: long
- indicators: calendar window grid (entry day -2..-1 relative to the last trading day of the month, exit day +2..+5 of the new month); random-window placebo
- entry: Buy SPY at the close of the SECOND-TO-LAST trading day of the month (so the 4-day window includes the last trading day's own return)
- exit: Sell at the close of the 3rd trading day of the new month (4 daily returns: last day + days 1-3)
- hold: 4 sessions
- frequency: monthly
- provenance: source-specified
- status: active
- priority: P1
- sources: [SSRN 917884 — Equity Returns at the Turn of the Month](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=917884)
- notes: FIXED off-by-one: McConnell & Xu (SSRN 917884, FAJ 2008): returns over the interval beginning the last trading day of the month and ending three days later are significantly higher (all excess market return in 1987-2005 and 1897-2005; no higher standard deviation; present in 31 of 35 countries). Buying the close of the LAST trading day and selling the close of day 3 earns only three daily returns; the four-day window needs entry at the close of the second-to-last day (as Be The Puck). Erosion: Trading Garden S&P 500 (MES): 1963-today 765 trades, 58% win, CAGR 11.2% vs 10.9% buy-and-hold, but since 2000 only 321 trades with CAGR 3.0%; Be The Puck beat buy-and-hold Sharpe on 63/372. Time in market is ~19%, so judge per unit of exposure against a random-window placebo; report a subperiod table.
- parameter_variants: window {entry close of day -2, exit close of day +3 primary; entry day -2 or -1, exit day +2..+5 grid; ex-S083: entry OPEN of day -2, exit close of day +1}; split by prior-month return sign; placebo {random 4-day windows}
- merged_from: S083
- data_needs: SPY, IWM, RSP total-return daily series (DP-ETF) + trading calendar (DP-CAL)
- review: RESOLVED

### S080 | Pre-Holiday Effect
- family: calendar
- concept: The documented anomaly (Lakonishok-Smidt 1988; Ariel 1990) is a high return ON the last trading day before a holiday. The StockCharts page describes the mechanism as traders lightening up before long weekends; the earlier concept text claimed the opposite price pressure and is corrected.
- universe: SPY or liquid stocks
- direction: long
- indicators: exchange holiday calendar
- entry: Buy at the close two sessions before a market holiday (i.e., the close of the second-to-last session before the holiday)
- exit: Sell at the close of the last session before the holiday (primary: captures the documented pre-holiday-day return); alternate: exit the first session after the holiday. Exclude early-close days (July 3, day after Thanksgiving) or report them separately
- hold: 1 session (primary); 2-3 sessions (alternate)
- frequency: ~9 holidays per year
- provenance: adapted
- status: active
- priority: P3
- sources: [StockCharts ChartSchool — The Pre-Holiday Effect](https://chartschool.stockcharts.com/table-of-contents/trading-strategies-and-models/trading-strategies/the-pre-holiday-effect)
- notes: ChartSchool describes: purchase equities one or two days before a holiday (selling pressure before a three-day holiday) with results by holiday and an original study of 419 closings 1928-1975; documented sample ends 1975. Holiday count is ~9-10 per year (~90 events in 2016+; Juneteenth since 2022). Be The Puck: beat buy-and-hold Sharpe on 91/372. Volume is very light on pre-holiday sessions: use conservative fills.
- parameter_variants: exit {close of last session before holiday primary; first session after holiday}; early-close days {excluded primary; separate}
- data_needs: SPY total-return daily series (DP-ETF) + exchange holiday calendar (DP-CAL)
- review: RESOLVED

### S081 | Pre-FOMC Announcement Drift
- family: calendar
- ohlcv_status: NOT_DAILY, DEFER
- concept: Lucca-Moench (JF 2015; NY Fed SR 512): SPX rose 49 bp on average in the 24 hours before scheduled FOMC announcements (Sept 1994 - Mar 2011), about 80% of annual realized excess stock returns, with a holding-only-those-windows strategy at an annualised Sharpe of 1.1 or higher. Kurov-Wolfe-Gilbert (FRL 2021): the drift 'essentially disappeared after 2015'.
- universe: SPY
- direction: long
- indicators: FOMC meeting calendar (scheduled announcements)
- entry: Buy SPY at the close of the session BEFORE the scheduled announcement day t (announcement at 14:00 ET on day t; use federalreserve.gov dates; the announcement day is the second day of two-day meetings; exclude unscheduled meetings)
- exit: Primary: sell at the OPEN of announcement day t (strictly pre-announcement). Alternates: close of day t (includes the 2 pm reaction); with SPY minute bars (2023-09+): 14:00(t-1) to 14:00(t), the faithful window
- hold: 1 session (overnight) primary
- frequency: 8 meetings per year
- provenance: source-specified
- status: active
- priority: P2
- sources: [Fed in Print — The pre-FOMC announcement drift](https://fedinprint.org/item/fednsr/12655)
- notes: FIXED: the window is ~14:00 on the day before to ~14:00 on the announcement day; the earlier 'sell at the open AFTER the announcement' held through the 2 pm release and the following night, contradicting its own 'exit before reaction risk' justification. The '>30x other days' figure is not in the abstract or introduction (which say 'orders of magnitude larger') and is removed. Expect near zero post-2015: split subperiods at 2015. Placebo = the same windows before other macro announcements. Be The Puck: beat buy-and-hold Sharpe on 43/372 ('kept to show that'). Eight scheduled meetings a year; pair with S079/S080 in one calendar basket.
- parameter_variants: exit {open of announcement day primary; close of announcement day; 14:00 to 14:00 minute version}; subperiods {to 2015, after 2015}; placebo {other macro announcements}
- data_needs: SPY daily bars (DP-ETF); SPY minute bars 2023-09+; FOMC calendar (DP-CAL)
- review: RESOLVED

### S082 | Options-Expiration-Week Drift
- family: calendar
- concept: Monthly options expiration (3rd Friday) forces gamma-driven flows into the cash market in expiration week; pinning and post-expiration drift are documented market-microstructure effects.
- universe: SPY, QQQ, and high-open-interest stocks
- direction: long
- indicators: monthly options expiration calendar (3rd Friday); week-of-month counter
- entry: Buy SPY at the close of the session before expiration week begins (typically the 2nd Thursday/Friday of the month)
- exit: Sell at the close of the Monday after expiration (post-expiration drift)
- hold: ~6 sessions (entry the session before expiration week to the close of the Monday after expiration)
- frequency: monthly
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [Quantified Strategies — 200 free backtested strategies (calendar family)](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: Placebo test only. The window is ~6 sessions, not the earlier '3-5'. Option-expiry literature concerns pinning and volatility effects, not drift; the Atlas (11.4) treats expiry as a calendar FEATURE to avoid mistaking settlement mechanics for information; daily/0DTE SPX options since 2022 weaken any monthly-gamma narrative. Use expiry-week dummies as a control variable in S079/S081 studies; run the simple drift once as a placebo test, otherwise drop. The cited page is an index (no rule).
- data_needs: SPY total-return daily series (DP-ETF) + options-expiry calendar (DP-CAL)
- review: DEMOTE | source GENERIC

---

# Family 9 — Trend and Pullback Systems (Stocks)

*Why this family works: in strongly trending stocks, pullbacks are bought by institutional accumulation programs; entering on pullbacks rather than breakouts gets better fill prices and rides the same trend. Connors' rules — buy pullbacks, above the 200-day MA, hold overnight, no stops — formalize this. Trend-following exits (trailing stops, channel breaks) create the asymmetric payoff profile (small losses, large wins) that trend systems need.*

### S087 | 20-Day-EMA Pullback in Uptrend
- family: trend and pullback
- ohlcv_status: NOT_DAILY, DEFER
- concept: A liquid stock in a long-term uptrend (above 200-day SMA) that pulls back to its 20-day EMA is at the institutional buy zone; entries at the EMA get trend exposure at discount prices.
- universe: S&P 500 stocks
- direction: long
- indicators: EMA(20); SMA(200); SMA(10); ATR(14)
- entry: Close(t) > SMA200 AND low(t) <= EMA20(t) (touch or cross below) AND a pullback-depth condition (3-day return <= -x%, registered x in {2, 3, 5}, or rank by distance below the EMA); buy at close of t (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound). Report signal frequency per 500 stocks before sizing
- exit: Close > 10-day SMA; sell at close of that session; stop: close < EMA(20) - 2 x ATR(14); max 8 sessions (stop sits ~2 ATR below entry vs a 10-day SMA reclaim exit: the risk/reward is skewed against the trade, report it)
- hold: 2-8 sessions
- frequency: several per week across the universe
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [CXO Advisory — Connors pullback rules](https://www.cxoadvisory.com/technical-trading/a-few-notes-on-short-term-trading-strategies-that-work/)
- notes: The old condition repeated close > SMA200 twice. In stocks above the 200-day SMA the low touches EMA(20) on a large fraction of days, so 'several per week' signals are unselective: the pullback-depth condition or distance ranking is added. The 20-EMA pullback is standard swing practice; Connors' family uses shallower pullbacks (5-day MA exits), see S001. Report with and without earnings-window signals (+/-3 sessions, G-12).
- parameter_variants: pullback depth x {none, 2%, 3%, 5%}; earnings window {included, excluded}
- data_needs: daily_10years (D1) + earnings flag (DP-EARN)
- review: RESOLVED

### S088 | MACD Bullish Cross with Volume
- family: trend and pullback
- concept: MACD(12,26,9) crossing above zero after a pullback marks momentum re-acceleration; volume confirmation separates genuine re-acceleration from dead-cat bounces.
- universe: S&P 500 stocks
- direction: long
- indicators: MACD(12,26,9); 20-day median volume; SMA(200)
- entry: MACD line crosses above signal line AND MACD > 0 AND close > SMA200 AND volume(t) > 1.3 × 20-day median; buy at open of t+1
- exit: MACD crosses below signal line; sell at close of that session; max 10 sessions
- hold: 3-10 sessions
- frequency: several per week across the universe
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [Quantified Strategies — 200 free backtested strategies](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: Rule-zoo audit member, not an independent strategy. Atlas 10.6 (Rink 2023; Chuang 2024; Yang 2020b): MACD/MA-type rules show no out-of-sample persistence after data-snooping correction; Be The Puck MACD Signal Cross beats buy-and-hold Sharpe on 59/372. 'MACD > 0 AND signal cross' is a late signal and 1.3x median volume is arbitrary; hold the parameterization fixed and report sensitivity. Include in the rule-zoo audit (G-05).
- data_needs: daily_10years (D1)
- review: DEMOTE | source GENERIC

### S089 | Golden Cross with Slope Filter
- family: trend and pullback
- concept: The 50/200 golden cross is the classic regime signal; requiring a rising 200-day SMA slope filters out crosses in sideways markets (whipsaw source).
- universe: SPY, sector ETFs, liquid stocks
- direction: long
- indicators: SMA(50); SMA(200); SMA(200) 20-day slope
- entry: SMA50 crosses above SMA200 AND SMA200(t) > SMA200(t-20); buy at close of t
- exit: SMA50 crosses below SMA200; sell at close of that session
- hold: months
- frequency: rare per instrument (~1 per 2 years); run across 20+ instruments for a live pipeline
- provenance: research-hypothesis
- status: demoted
- priority: P3
- sources: [Quantified Strategies — trend following with moving averages](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: Use as a regime OVERLAY (long-only gate on/off) in the champions' test rather than a standalone strategy. Trading Garden golden cross (S&P): 33 trades 1963-today, 78.8% win, CAGR 11.8%, max drawdown 19.9%/25.5% (best risk profile in its table) but only 13 trades since 2000 (~4-5 SPY events in 2016+); Be The Puck beat buy-and-hold Sharpe on 106/371 (best of the trend rules). The slope filter is a second parameter. As a standalone it has too few events; as a regime gate it is credible.
- data_needs: ETF total-return daily series (DP-ETF); daily_10years (D1)
- review: DEMOTE (overlay) | source GENERIC

### S090 | KAMA Breakout
- family: trend and pullback
- concept: Kaufman's Adaptive Moving Average (KAMA) tightens in sideways markets and loosens in trends; price crossing KAMA after a quiet period is an adaptive breakout signal that self-tunes to volatility.
- universe: QQQ, SPY, liquid stocks
- direction: long and short
- indicators: KAMA(10, 2, 30); 20-day realized volatility
- entry: Close(t) > KAMA(t) AND KAMA flat-to-rising (KAMA(t) > KAMA(t-5) × 0.999); buy at close of t
- exit: Close < KAMA; sell at close of that session
- hold: days to weeks
- frequency: ~1-2 trades per month per instrument
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [YouTube — KAMA indicator swing trading (backtested)](https://www.youtube.com/watch?v=w0oVbDRMmEE)
- notes: Run once with the standard KAMA(10,2,30) formula and accept the null. The only source is a YouTube marketing claim ($124k on NQ; page empty in review); Be The Puck's Kaufman Adaptive MA beat buy-and-hold Sharpe on 44/372; `KAMA(t) > KAMA(t-5) x 0.999` is an arbitrary slope test.
- data_needs: daily_10years (D1); ETF daily bars (DP-ETF)
- review: DEMOTE | source UNVERIFIED

### S091 | Donchian 55 with Chandelier Trail
- family: trend and pullback
- concept: Turtle System 2 entries (55-day breakout) with a 3×ATR(22) chandelier trailing stop from the highest high since entry — the trailing exit lets winners run far beyond the channel exit.
- universe: liquid ETFs
- direction: long and short
- indicators: 55-day Donchian high/low; ATR(22); chandelier stop (highest high − 3 × ATR)
- entry: Close(t) > 55-day high; buy at close of t
- exit: Close < highest high since entry − 3 × ATR(22); mirror for shorts
- hold: weeks to months
- frequency: 1-3 trades per year per instrument across 10-20 ETFs
- provenance: adapted
- variant_of: S036
- status: demoted
- priority: P4
- sources: [Kagels Trading — Turtle rules](https://kagels-trading.com/blog/turtle-trading/)
- notes: Exit variant of the Donchian family (S036, speed 55): the 3 x ATR(22) chandelier is ~6-9% below the high for equities, tight relative to the volatility of 55-day swings, and induces churn; Be The Puck's Chandelier Exit beat buy-and-hold on 74/372. Run as `S036 --speed 55 --exit chandelier` and compare with the 20-day-low and 2N exits. Kagels has no chandelier rule (catalog's own).
- data_needs: ETF total-return daily series (DP-ETF)
- review: RESOLVED

### S093 | Higher-Low Reversal Entry
- family: trend and pullback
- ohlcv_status: NOT_DAILY, DEFER
- concept: After a decline, two consecutive sessions with higher lows signal sellers are exhausted; volume confirmation marks accumulation (Wyckoff-style daily-bar pattern). As originally written the rule selected an ongoing rise, so a prior-decline precondition is now explicit.
- universe: liquid stocks
- direction: long
- indicators: 2-day higher-low sequence; prior-decline condition; volume vs 20-day median; SMA(200)
- entry: Prior decline (definition A: 5-10 day drawdown >= x% from the recent high; definition B: RSI(5) < 40 within the last 5 sessions; both registered) AND close(t) > SMA200 AND low(t) > low(t-1) > low(t-2) AND volume(t) > 1.2 x 20-day median AND close(t) > open(t); buy at close of t (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound)
- exit: Close < low(t-1) (structure broken) or close > 10-day SMA; max 8 sessions
- hold: 2-8 sessions
- frequency: several per week across the universe
- provenance: research-hypothesis
- status: active
- priority: P3
- sources: [Quantified Strategies — 200 free backtested strategies](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: Pattern definitions are unambiguous on daily bars. Without a decline precondition the original entry (above SMA200, rising lows, bullish bar, volume) is a continuation entry, not a reversal; report both decline definitions and the version without any precondition as the control. Report with and without earnings-window signals (G-12).
- parameter_variants: decline definition {A: drawdown >= x%, B: RSI(5) < 40}; control {no precondition}
- data_needs: daily_10years (D1) + earnings flag (DP-EARN)
- review: RESOLVED

### S094 | Bollinger Squeeze Breakout
- family: trend and pullback
- concept: When Bollinger Band width (20-day, 2 sd) falls to a 6-month low, volatility is compressed; the squeeze resolves directionally — trade the first close outside the bands.
- universe: QQQ, SPY, liquid stocks
- direction: long and short
- indicators: Bollinger Bands (20-day, 2 sd); band width percentile (120-day)
- entry: Band width in bottom decile of last 120 sessions AND close(t) > upper band; buy at close of t (mirror for lower band)
- exit: Close crosses the 20-day middle band against the position; max 10 sessions
- hold: 2-10 sessions
- frequency: ~1-2 per month per instrument
- provenance: research-hypothesis
- variant_of: S065
- status: demoted
- priority: P4
- sources: [Quantified Strategies — 200 free backtested strategies](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: Compression cluster member (variant of S065), executed via band-close rather than stop orders. The Be The Puck squeeze rule (BandWidth at its lowest of 125 days, close above the upper band, exit below the 20-day average) beat buy-and-hold Sharpe on only 30/371, the worst of its breakout family. The unused '20-day Donchian confirmation' indicator is removed from the list.
- data_needs: ETF daily bars (DP-ETF); daily_10years (D1)
- review: DEMOTE | source GENERIC

### S095 | Keltner Channel Breakout
- family: trend and pullback
- concept: Keltner Channels (20-day EMA ± 2 × ATR(10)) are trend-following bands; a close above the upper channel marks trend initiation with volatility-based (not standard-deviation-based) bands.
- universe: liquid ETFs and stocks
- direction: long and short
- indicators: Keltner Channel (EMA(20), ATR(10), multiplier 2); 20-day median volume
- entry: Close(t) > upper Keltner band AND volume > 1.3 × 20-day median; buy at close of t
- exit: Close < middle band (EMA20); sell at close of that session; max 10 sessions
- hold: 2-10 sessions
- frequency: several per month across a universe
- provenance: research-hypothesis
- variant_of: S094
- status: demoted
- priority: P4
- sources: [TradingView — Keltner Channels strategies](https://www.tradingview.com/scripts/keltnerchannel/)
- notes: Compression/channel cluster member only (correlated with S094). Be The Puck's Keltner rule (EMA20 + 2 x ATR10; exit below the EMA) beat buy-and-hold Sharpe on 46/372 and returned +2.1%/yr on the S&P 500 index vs +8.9%; the cited TradingView page is a script listing, not a source. Report the correlation with S094.
- data_needs: ETF daily bars (DP-ETF); daily_10years (D1)
- review: DEMOTE | source GENERIC

---

# Family 10 — Candlestick and Bar Patterns

*Why this family works: bar patterns encode the session's order-flow outcome (who won, buyers or sellers) in four numbers. Bulkowski's Stocks & Commodities work (e.g., the 3-bar reversal study) and the Connors/Crabel pattern families are codable without ambiguity on daily bars. The edge, where it exists, comes from pattern frequency asymmetries after conditioning (prior streak, trend regime, location relative to moving averages).*

### S096 | Three-Bar Reversal (Bulkowski)
- family: candlestick patterns
- ohlcv_status: NOT_DAILY, DEFER
- concept: Bulkowski's three-bar reversal as defined on the cited TradingView (HPotter) page: daily bars; the middle day has the lowest low of the three days (no ties); the last day closes above the prior day's high (no ties); each day has a nonzero trading range. A short-term reversal signature.
- universe: liquid stocks
- direction: long (mirror short)
- indicators: 3-bar pattern per the source definition; SMA(200) (catalog addition)
- entry: Pattern completes: low(t-1) < low(t-2) AND low(t-1) < low(t) AND close(t) > high(t-1), no ties, high != low on each of the three bars; buy at close of bar 3 (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound). Registered variant: AND close > SMA200 (catalog addition)
- exit: Close < pattern low (the middle bar's low) or after 5 sessions; sell at close (the exit is the catalog's addition, registered)
- hold: 1-5 sessions
- frequency: daily across a liquid universe
- provenance: adapted
- status: active
- priority: P2
- sources: [TradingView — 3-Bar-Reversal-Pattern strategy backtest (HPotter)](https://br.tradingview.com/script/IPLkvrzq-3-Bar-Reversal-Pattern-Strategy-Backtest/)
- notes: CORRECTED: the earlier 'bar 2 range inside bar 1's range, bar 3 close above bar 1's close' was a different pattern and 'inside' contradicted 'lowest low of the three'. The SMA200 gate and the 5-session/pattern-low exit are the catalog's additions. Compare against random bars of equal frequency; register pattern, length, distance threshold and holding rule before testing (Atlas pattern registry). Based on Bulkowski's Stocks & Commodities article (Jan 2000).
- parameter_variants: gate {none primary; SMA200}; exit {pattern low / 5 sessions primary}; mirror short {long-only first}
- data_needs: daily_10years (D1)
- review: RESOLVED

### S097 | Bullish Engulfing After Down-Streak
- family: candlestick patterns
- concept: A bullish engulfing candle (session body engulfs the prior bearish body) after 3+ down closes marks capitulation followed by aggressive buying — conditioned on the streak to avoid random engulfing noise.
- universe: liquid stocks
- direction: long
- indicators: engulfing pattern test; 3-day down-close streak; SMA(200)
- entry: 3+ consecutive down closes (close(t-1) < open(t-1) bearish prior bar), then session t with close(t) > open(t) AND open(t) < close(t-1) AND close(t) > open(t-1) (current body fully engulfs prior body) AND close(t) > SMA200; buy at close of t
- exit: Close > 5-day SMA or first up-close target; max 5 sessions
- hold: 1-5 sessions
- frequency: several per week across the universe
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [Quantified Strategies — 200 free backtested strategies](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: Candlestick registry member (S096-S100), tested under a family-wise (FDR) correction; compare with S030 (streak) without the pattern ('the desk question is always whether the pattern label adds information beyond the raw window'). Be The Puck's engulfing switch beat buy-and-hold Sharpe on 45/372 and returned +4.4%/yr on the S&P 500 index vs +8.9%. The 3-down-closes-then-engulf condition is mostly S030/S007 with a candlestick overlay. The strict engulfing definition is primary; test the loose variant (close above prior close, open below prior open).
- data_needs: daily_10years (D1)
- review: DEMOTE | source GENERIC

### S098 | Hammer at Lower Bollinger Band
- family: candlestick patterns
- ohlcv_status: NOT_DAILY, DEFER
- concept: A hammer (long lower wick, small body near the session high) at the lower Bollinger Band is a rejection of extreme prices — buyers defended the lower extreme within the session.
- universe: liquid stocks and ETFs
- direction: long
- indicators: hammer defined on the range (lower wick >= 60% of the high-low range AND close in the top third of the range, no zero-body failure); Bollinger Band (20-day, 2 sd); SMA(200)
- entry: Hammer (range-based definition) forms AND low(t) < lower Bollinger band AND close(t) > SMA200; buy at close of t (fill: moc_proxy primary; next_open alternate; close fill only as an upper bound)
- exit: Close > 20-day middle band; max 5 sessions
- hold: 2-5 sessions
- frequency: weekly across a universe
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [Quantified Strategies — 200 free backtested strategies](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: The earlier definition (lower wick > 2 x body AND upper wick < body) is unsatisfiable for doji bars (body = 0) and noisy for tiny bodies; the range-based definition replaces it. `low(t) < lower band` (an intraday touch) differs from S011's close below the band, and this is the same cluster as S011. Candlestick registry member.
- data_needs: daily_10years (D1)
- review: DEMOTE | source GENERIC

### S099 | Outside-Day Key Reversal at 200-SMA
- family: candlestick patterns
- concept: An outside day (higher high, lower low) that closes green against a prior down-close, landing on the 200-day SMA, is a battle won by buyers at the most-watched level in the market.
- universe: liquid stocks
- direction: long
- indicators: outside-day test; prior down-close; SMA(200)
- entry: high(t) > high(t-1) AND low(t) < low(t-1) AND close(t) > open(t) AND close(t-1) < open(t-1) AND |close(t) − SMA200| / close(t) < 1%; buy at close of t
- exit: Close < low(t) (reversal low broken) or close > 10-day SMA; max 8 sessions
- hold: 2-8 sessions
- frequency: several per month across a universe
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [Quantified Strategies — 200 free backtested strategies](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: Candlestick registry member. Classical key reversal is a new low followed by a close above the prior close; this outside-day variant plus 'within 1% of SMA200' is rare and arbitrary. Measure the signal frequency first (the earlier 'several per month across a universe' is unverified) and drop if there are fewer than 100 events in the dev period. 200-SMA proximity is a free parameter.
- data_needs: daily_10years (D1)
- review: DEMOTE | source GENERIC

### S100 | Three White Soldiers Continuation
- family: candlestick patterns
- concept: Three consecutive long green candles each closing above the prior close signals sustained aggressive buying; in an established uptrend it predicts continuation, not exhaustion.
- universe: liquid stocks
- direction: long
- indicators: 3-session pattern (each close > prior close, each close > open, each open within prior body); SMA(50); volume trend
- entry: Pattern completes AND close > SMA50 AND volume rising across the 3 sessions; buy at close of the third session
- exit: First close below the pattern's first session low; max 8 sessions
- hold: 2-8 sessions
- frequency: several per month across a universe
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [Quantified Strategies — 200 free backtested strategies](https://www.quantifiedstrategies.com/trading-strategies-free/)
- notes: Falsification against S027's sign: three strong up days put the stock in the top of its 3-day return distribution, the state where the short-term reversal literature (S027, NY Fed SR 513) expects give-back, and S100 bets on continuation without a reason to expect it to win. It is the mirror of S030/S031 logic. Register as a falsification test; candlestick registry member.
- data_needs: daily_10years (D1)
- review: DEMOTE | source GENERIC

---

# Family 11 — Miscellaneous Structural Edge

*Why this family works: recurring market structure — dividend mechanics, macro linkages between asset classes, and yield-curve-driven sector flows — creates predictable return patterns at daily granularity that are independent of price-based signals.*

### S102 | Copper/Gold Ratio Equity Tilt (CPER/GLD ratio, both in cache)
- family: structural edge
- concept: The copper/gold ratio is a real-time growth-vs-fear indicator (copper = industrial demand, gold = fear hedge); a rising ratio favors cyclical stocks, a falling ratio favors defensives — rotating between them monetizes the macro signal.
- universe: XLB/XLI (cyclicals) vs XLP/XLU (defensives); signal ratio CPER/GLD (both in daily_10years, 2016+)
- direction: long
- indicators: CPER/GLD price ratio; 63-day z-score of the ratio
- entry: Each week, if the CPER/GLD 21-day change > 0 AND 63-day z > 0, hold the cyclical basket; else hold the defensive basket. Registered control: plain relative momentum of (XLB+XLI) minus (XLP+XLU) with no copper input
- exit: Weekly re-evaluation; switch on signal flip
- hold: weeks
- frequency: switches ~6-10 per year
- provenance: research-hypothesis
- status: active
- priority: P4
- sources: [PortfoliosLab - Momentum & sector rotation (generic; does not describe a copper/gold idea)](https://portfolioslab.com/strategies/momentum-rotation)
- notes: RESOLVED (was BLOCKED): CPER launched in late 2011 and is already in the daily_10years cache from 2016-01-04 (2,718 rows; thinly traded with zero-volume days: apply the data-hygiene mask), as are GLD and USO, so the CPER/GLD ratio is testable on 2016+ without futures data; no pre-2016 regime coverage is claimed. The ratio's relationship to cyclicals vs defensives flips with the rate regime and `21-day change > 0 AND 63-day z > 0` flips often, so test against the simple relative-momentum control and accept the null if it adds nothing. Sector ETFs XLB/XLI/XLP/XLU need DP-ETF.
- parameter_variants: signal {CPER/GLD ratio primary; relative momentum of cyclical minus defensive baskets (no copper)}; cadence {weekly primary; monthly}
- data_needs: CPER and GLD in daily_10years (D1); XLB, XLI, XLP, XLU (DP-ETF)
- review: RESOLVED

### S103 | Yield-Curve Financials/Utilities Rotation
- family: structural edge
- concept: A steepening yield curve (10Y−2Y slope rising) compresses bank net-interest margins favorably and hurts utilities (bond proxies); a flattening curve reverses it. Rotating between XLF and XLU on curve-momentum harvests rate-regime flows.
- universe: XLF, XLU
- direction: long
- indicators: 10Y yield level change and 2Y yield change (separately); optional 10Y-2Y slope
- entry: Monthly decision (dead band): use the 21-day change in the 10Y yield and in the 2Y yield separately (bull steepening via falling short rates vs bear steepening via rising long rates have opposite implications for utilities); hold XLF when the signal favours steepening by bank NIM logic beyond the dead band, else XLU
- exit: Weekly re-evaluation
- hold: weeks to months
- frequency: switches ~4-8 per year
- provenance: research-hypothesis
- status: demoted
- priority: P4
- sources: [PortfoliosLab — Momentum & sector rotation (rotation machinery)](https://portfolioslab.com/strategies/momentum-rotation)
- notes: The 21-day change in the 10Y-2Y slope switches sign about weekly in a flat curve, and the mechanism is ambiguous (bull vs bear steepening). Only 2016-2026 episodes (2019 inversion, 2022-23 inversion) give independent regimes. Accept the null if not significant after FDR. The PortfoliosLab citation does not describe curve rotation.
- parameter_variants: decision cadence {monthly with dead band primary; weekly}; signal {10Y and 2Y changes separately primary; slope change}
- data_needs: FRED T10Y2Y, DGS10, DGS2 (DP-FRED); XLF, XLU (DP-ETF)
- review: DEMOTE | source GENERIC

### S104 | High-Beta/Momentum Crash Protection (overlay comparison on S041)
- family: structural edge
- concept: Momentum crashes after market declines (Chan: after a crisis momentum strategies get hurt in the rebound; one remedy is to stop shorting after a market fall and trade only the long side post-crisis; his preferred approach is running momentum and mean reversion together). The evidenced mitigation is scaling momentum by its own trailing realised volatility (Barroso & Santa-Clara 2015: it 'nearly doubles the Sharpe ratio while virtually eliminating crashes'). This entry compares three overlays on S041.
- universe: liquid_top_N momentum decile from S041
- direction: long / cash
- indicators: market drawdown from 252-day high; strategy's own 126-day realised volatility; momentum decile rank
- entry: Hold the S041 top-decile 12-1 portfolio under three registered overlays, compared out of sample: (a) catalog gate: hold while SPY is within 10% of its 252-day high; (b) vol-scaling: exposure w_t = sigma_target / sigma_hat_t with sigma_hat = realised daily vol of the strategy's own return over the previous 126 days annualised, sigma_target = 12% (long-short) or the unscaled strategy's full-sample vol (long-only), capped at 1.0 (optionally 1.5-2.0), re-sized monthly; (c) none
- exit: (a) move to cash when SPY falls more than 10% below its 252-day high, re-enter on a new 63-day high (late both ways); (b) monthly re-scaling; (c) none
- hold: months
- frequency: rebalance monthly, gate checks daily
- provenance: adapted
- status: active
- priority: P2
- sources: [Better System Trader - Ernie Chan (momentum-crash discussion)](https://bettersystemtrader.com/012-ernest-chan/); Barroso & Santa-Clara (2015) (Atlas 9.6, 18.4, 18.6)
- notes: CORRECTED: Chan advises stopping SHORTS after a crisis; a long-only de-risk-to-cash gate is not Chan's advice, and the 10%-from-high trigger with 'new 63-day high' re-entry is a drawdown gate that is late both ways. Test crash months (Apr-May 2020 analogue) and overlay turnover; the Atlas's honest benchmark asks whether a change-point gate beats plain vol scaling.
- parameter_variants: overlay {(a) catalog gate; (b) vol-scaling primary candidate; (c) none}; vol target {12% L/S; unscaled full-sample vol long-only}; cap {1.0 primary; 1.5-2.0}
- data_needs: daily_10years (D1) + SPY (DP-ETF)
- review: RESOLVED

### S105 | Volatility-Targeted Overnight Carry
- family: structural edge
- concept: Combine the overnight premium (S021) with volatility scaling: hold the overnight session sized inversely to current volatility — more exposure in calm markets, less in stressed ones (Barroso/Santa-Clara vol-scaling improves Sharpe for momentum strategies; applied here to the overnight premium).
- universe: SPY
- direction: long
- indicators: 20-day realized volatility of the OVERNIGHT (close-to-open) return series, dividend-correct; target 10% annualized; overnight holding schedule
- entry: Each day, size overnight SPY exposure = 10% / (annualized realized vol of the overnight return series, 20 days), capped at 1x (the $5,000 unlevered account) and, separately, 2x (needs margin); buy at close; re-size only when the target exposure moves outside a 20% band
- exit: Sell at open; re-size daily
- hold: 1 session (overnight), repeated daily
- frequency: daily
- provenance: research-hypothesis
- status: active
- priority: P2
- sources: [SSRN — Night Trading (overnight premium)](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=2633476); Attached paper (user upload) — Borsa Istanbul (volatility scaling, Barroso & Santa-Clara)] — attached PDF (user upload), no public URL verified
- notes: Sizing vol must come from the overnight return series (what is actually held), not close-to-close vol, otherwise the 10% target is misdefined (overnight vol is a fraction of total vol). The null is S021 unscaled on identical dividend-correct data: the question is whether scaling improves Sharpe/drawdown, not whether the premium exists; Barroso-Santa-Clara scaling is documented for momentum, not for the overnight premium. The attached Borsa paper credits breadth (equal-weighting, p = 0.034 vs cap-weighted) plus risk scaling for its whole improvement, not volatility scaling alone. Report turnover of the resizing and costs per the S021 ladder. The Efficiente 5 paper is not a source for this entry.
- parameter_variants: leverage cap {1x primary; 2x}; resizing {20% band primary; daily}; vol window {20-day overnight primary}
- data_needs: SPY raw OHLC + dividends (DP-ETF raw+div)
- review: KEEP | source OK
- implementation_results: Rust port in `src/study.rs` / `src/signals.rs`, using the same total-return-adjusted ETF Parquet bars and close-to-open proxy fills as S021; 30 registered variants. This differs from the catalog's preferred raw OHLC plus explicit ex-date dividends and true auction fills. `target25` (2.5% volatility target) was selected on dev skill; its lower exposure reduces drawdown but does not produce positive overnight-null skill or improve Sharpe over the unscaled baseline.

  | variant | dev total / skill / Sharpe / max DD | test total / skill / Sharpe / max DD | trades (dev+test) |
  |---|---|---|---|
  | baseline | +0.24 / -0.36 / 0.42 / -22.3% | +0.22 / -0.13 / 0.97 / -13.8% | 1,991+706 |
  | target25 | +0.08 / -0.14 / 0.39 / -6.6% | +0.08 / -0.05 / 0.93 / -4.1% | 1,991+706 |
  Reproduce: `run_strats.bat` runs the full matrix; `target\release\deepthrust.exe --strategy s105_overnight_carry --variant target25` reruns baseline and the dev-selected variant.

---

## Appendix — Source index

**Attached papers (user-provided):**
- [Institutional Asset Allocation meets the Efficient Frontier: Replicating the JPMorgan Efficiente 5 Strategy — Jungle Rock Capital, Oct 2025] — attached PDF (user upload), no public URL verified (S044)
- [Multi-Factor Investing in Borsa Istanbul — Kemal Onat Dursun, Oct 2026] — attached PDF (user upload), no public URL verified (S028, S032, S035, S045, S105)
- [Opening the Black Box: Which News Explain LLM-Based Stock Ratings? — Victoria Kiszczuk, SSRN 7665703, Oct 2026] — attached PDF (user upload), no public URL verified (S058)

- [Trading Literature Atlas for Semi-Automated Systematic Trading - S. Bandyopadhyay, Oct 2026] - attached PDF (user upload); a ~140-page survey (1,065 papers) used by the review for validation guidance (G-05, G-06, G-15); not a direct source of any entry
- Verified identifiers: Night Trading SSRN 2633476; Turn of the Month SSRN 917884; Kiszczuk SSRN 7665703; Cooper-Cliff-Gulen SSRN 1004081 (S021)

**Ernest Chan (blog/books/interviews):** epchan.blogspot.com posts on [mean-reversion profitability](http://epchan.blogspot.com/2008/12/enduring-profitability-of-mean.html) (S078), [volatility term structure](http://epchan.blogspot.com/2016/04/mean-reversion-momentum-and-volatility.html) (S064, S065), [Estimize earnings estimates](http://epchan.blogspot.com/2015/01/trading-with-estimize-and-ibes-earnings.html) (S020, S023, S055), [insider trading](http://epchan.blogspot.com/2017/07/building-insider-trading-database-and.html) (S059); [Buy-on-Gap reproduction](https://github.com/l3a0/quantitative-trading/blob/main/blog/buy-on-gap-lessons.md) (S015, S016); [Better System Trader interview](https://bettersystemtrader.com/012-ernest-chan/) (S041, S049, S104).

**Connors/Alvarez:** [StockCharts RSI(2)](https://chartschool.stockcharts.com/table-of-contents/trading-strategies-and-models/trading-strategies/rsi-2) (S001, S003, S013), [Quantified Strategies %b](https://www.quantifiedstrategies.com/larry-connors-b-strategy-bollinger-band/) (S005), [Double 7s](https://www.quantifiedstrategies.com/larry-connors-double-seven-strategy-does-it-still-work/) (S006), [RSI 25/75](https://www.quantifiedstrategies.com/larry-connors-rsi-25-rsi-75/) (S003), [Be The Puck ConnorsRSI](https://bethepuck.com/strategies/connorsrsi-pullback/) (S008), [DYR cumulative RSI](https://tradingwithdyr.com/cumulative-rsi-strategy/) (S004), [CXO Advisory notes](https://www.cxoadvisory.com/technical-trading/a-few-notes-on-short-term-trading-strategies-that-work/) (S007, S014, S030, S087, S001).

**Academic (SSRN/NY Fed/NBER/arXiv):** [Night Trading (SSRN 2633476)](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=2633476) (S017, S021, S105), [Turn of Month (SSRN 917884)](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=917884) (S079), [NY Fed reversal decomposition](https://www.newyorkfed.org/medialibrary/media/research/staff_reports/sr513.html) (S027), [NBER earnings announcement premium](https://www.nber.org/system/files/working_papers/w13090/w13090.pdf) (S052, S054), [Pre-FOMC drift](https://fedinprint.org/item/fednsr/12655) (S081), [arXiv LLM sentiment](https://arxiv.org/html/2412.19245) (S056, S060), [arXiv LLM sector allocation](https://arxiv.org/html/2503.09647v5) (S057), [Jegadeesh-Titman 1993](https://www.bauer.uh.edu/rsusmel/phd/jegadeesh-titman93.pdf) (S041).

**Practitioner sites:** [Quantified Strategies](https://www.quantifiedstrategies.com/trading-strategies-free/) (many), [Quantpedia sector momentum](https://quantpedia.com/strategies/sector-momentum-rotational-system) (S042), [PortfoliosLab](https://portfolioslab.com/strategies/momentum-rotation) (S043, S102, S103), [Trading Garden](https://trading-garden.com/sp500-trading-strategies.html) (S032, S033, S047), [Validea](https://www.validea.com/sector-rotation-momentum-portfolio) (S048), [Kagels Turtle rules](https://kagels-trading.com/blog/turtle-trading/) (S036, S051, S091), [Fractiz NR7](https://www.fractiz.com/strategies/nr7-inside-breakout/) (S038), [Validated Strategies NR7](https://validatedstrategies.com/strategy/nr7-xauusd) (S038), [Kvants volume breakout](https://studio.kvants.ai/blog/volume-breakout-strategy) (S040, S050), [Capital.com gap trading](https://capital.com/en-int/learn/trading-strategies/gap-trading) (S018, S019, S026), [Schaeffer's VXV](https://www.schaeffersresearch.com/content/analysis/2015/12/03/another-way-to-trade-volatility) (S062), [Papers With Backtest](https://paperswithbacktest.com/strategies/using-vix-to-trade-spy-and-sp-500) (S061, S063, S010, S046), [BestFolio Reddit strategy](https://bestfolio.app/strategies/buy-the-dip) (S069, S070), [Earnings Whispers](https://www.earningswhispers.com/about-score) (S053, S054), [StockCharts pre-holiday](https://chartschool.stockcharts.com/table-of-contents/trading-strategies-and-models/trading-strategies/the-pre-holiday-effect) (S080), [phuazz breadth thrust](https://github.com/phuazz/breadth-thrust-etf) (S066, S067), [Springer ETF pairs](https://link.springer.com/article/10.1057/s41260-025-00416-0) (S071, S075, S076, S077), [Quantified Strategies gold/silver](https://www.quantifiedstrategies.com/gold-silver-chart-ratio-strategy/) (S072, S073), [Quantified Strategies 3-days-down](https://quantifiedstrategies.substack.com/p/3-days-down-overnight-trading-strategy-942) (S009), [TradingView 3-bar reversal](https://br.tradingview.com/script/IPLkvrzq-3-Bar-Reversal-Pattern-Strategy-Backtest/) (S096), [YouTube KAMA](https://www.youtube.com/watch?v=w0oVbDRMmEE) (S090), [Investopedia Donchian](https://www.investopedia.com/terms/d/donchianchannels.asp) (S036).

**Caveats carried through the catalog:** NR7 compression breakout is a falsification test (Fractiz sample runs show PF 1.04-1.39 with ~40% drawdowns; the Validated Strategies gold page is a 404 so its PF 0.64 claim is unverifiable) (S038); Chan's short-on-gap mirror was profitable pre-cost in the l3a0 reproduction (APR 12.2%, Sharpe 1.79) but did not match the published 46%/1.27 (S016); the earnings-eve delta rule (S020) is a documented null benchmark and is dropped; LLM strategies cannot be cleanly backtested over long windows without timestamped news archives and frozen models (S056-S058, S060; S057 forward test only); close-based entries cannot be traded with certainty at that close (see the three fill models); the Borsa paper's multi-factor blend produced no alpha vs a volatility-matched equal-weight benchmark (S045); BestFolio's 3x-leveraged regime switch carries a -76.2% worst drawdown in simulated history, so only real TQQQ history with an exposure cap is used (S070).
