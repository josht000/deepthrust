# deepthrust
Deep market apha penetration testing in Rust.  On the road to implementing 5/1000 published trading strategies in search of alpha.

Identified: 263/1000 individual strategies
Implemented: 96/263

Method: Pure agentic approach to research and implementation. Python is ~120 seconds per run whereas Rust is less than 1
and still has all the plotting and data processing needed. 

Another goal is to live trade these and publish live results. 

## Usage

```bash
# run the S006 baseline and its configured robustness variants with timing profiling
cargo run --release -- --strategy s006_double_7s --variant lb5,lb10,stop3atr --profile
cargo run --release -- --strategy s021_index_overnight --variant sma50,qqq,iwm
cargo run --release -- --strategy s105_overnight_carry --variant cap2x,daily_resize
# show warnings and per-symbol trade counts
cargo run --release -- --strategy s001_rsi2_connors --debug

# named variants from the config's `variants:` block (baseline is included automatically)
cargo run --release -- --strategy s001_rsi2_connors --variant rsi5,gate21 --symbols AAPL,MSFT --data-dir C:\\data\\trading\\trading_framework\\data\\cache\\daily_10years

# run the full configured matrix for all registered strategies
run_strats.bat
run_strats.bat --cash                 # also run the S041 cash-study variant screen
run_strats.bat --cash-only            # run only the S041 cash-study screen
run_strats.bat --cash --debug         # flags can be combined in either order
cargo run --release -- --check-data          # validate inputs + write a manifest only
cargo test                                   # unit tests + offline end-to-end smoke test (synthetic bars)
```

CLI flags mirror `run.py`: `--strategy --config --stages --test --rebuild --profile --debug --check-data --variant --data-dir`
(plus `--symbols`). Daily input data is read from the configured split-adjusted Parquet files and is never rewritten.
Input-gap and benchmark-fallback warnings, plus the per-symbol trade-count console table, are quiet by default.
Pass `--debug` to print them; `run_strats.bat --debug` forwards the option to every strategy run.
`run_strats.bat --cash` additionally runs the 44-configuration S041 cash screen with one
deterministic random-pick draw per configuration; `--cash-only` builds and runs just that screen,
skipping all other strategies. This is a screening run, not robust placebo evaluation.

After the standard strategy matrix completes, `run_strats.bat` rebuilds
`results/all_trade_metrics.csv` from the saved partition DEV/TEST CSVs.
Each strategy may use the legacy single-source keys (`data_dir`, `data_suffix`, `symbols`) or a `sources:` list
with named Parquet directories, symbol lists, and per-source costs. Sources are aligned by date; duplicate symbols
across sources are rejected. A benchmark can be loaded independently of the traded universe with
`benchmark_data_dir` and `benchmark_data_suffix`; this is useful for stock-only variants that still compare against
an ETF benchmark. For example:

The `stocks_mega`, `stocks_large`, and `stocks_small` variants are hand-picked stock samples, not
point-in-time market-cap universes; their results are subject to survivorship and classification bias.
They use 10 bps per side and compare against SPY from the ETF cache. The `mixed_mega` variants combine
index ETFs with the mega-cap stock sample and apply source-specific costs.

Outputs (trades, partition dev/test CSVs, cost ladder, baselines, `report.md`, and a `summary.svg` nine-panel
dashboard: cumulative returns, key metrics, partition decomposition, drawdown,
calendar years, trade returns, holding period, exit reasons, open positions; a configured benchmark is added to panel A).
Panel A accumulates all series from the evaluation start, excluding warm-up returns.
land in `results/<strategy>/`. When charts are requested, `symbols_overview.svg` compares the baseline, the best
dev-selected variant, each symbol's buy-and-hold, and the configured benchmark in one small-multiples plot;
`symbols_summary.csv` retains metrics for every symbol and requested variant. Running `--variant a,b` also runs
the baseline and writes a combined overview. For universes above 24 symbols, the SVG shows the 12 strongest and
12 weakest symbols by baseline dev excess return; the CSV still includes all symbols. Only total elapsed timing is
printed to the console. The full stage tree is written to `results/<strategy>/timing.txt`; `--profile` adds
per-call mean/min/max columns to that report.

For Family 4, partition trend uses the equal-weight return of the tradable universe selected at the
previous close; S046 uses the mean French industry return. External benchmarks such as SPY remain
separate chart comparisons. Weighted exposure counts the position held into each period instead of
counting both sides of a rebalance. Trend uses signed net exposure, while invested fraction uses
gross exposure; this is a close-to-close approximation for next-open trades.

## Markov regime layer

`src/markov.rs` ports the observable Markov chain from `markov-hedge-fund-method` (Roan/@RohOnChain):
each bar is labelled Bear/Sideways/Bull from a trailing `markov_window`-bar return beyond
±`markov_threshold`, a 3x3 transition matrix is re-estimated walk-forward (no lookahead), and the
signal `s(t) = P(next=Bull|state) - P(next=Bear|state)` drives either a standalone study or a
cross-strategy risk overlay.

Standalone validation study:

```bash
cargo run --release -- --strategy markov_regime
cargo run --release -- --strategy markov_regime --variant w60,qqq_w40,long_short
```

`markov_position` is `long_flat` (default), `long_short`, or `conviction` (weight = clamped signal).

Risk overlay on any registered strategy via config fields (variant overlays work too —
`mk_ge: {markov_mode: gate,exit}`): `markov_mode` is a comma list of `gate` (veto entries/adds while
risk-off), `exit` (force `exit_now`), `size` (rescale `trade_weights` by `markov_blend_form`
tilt|mix|conf). `markov_source` is `symbol` (per-symbol chain, default), `benchmark`, `universe`,
`breadth` (label the daily bear-fraction of the universe), or a `markov_symbol` override.
`markov_regime` strict|hysteresis uses `markov_enter`/`markov_exit` as the on/off band;
`markov_warmup` pass|block covers the `markov_min_train` (252) warm-up.

Chain tuning levers: `markov_vol_window`+`markov_vol_mult` replace the fixed `markov_threshold` with
a per-symbol vol-scaled cutoff (`mult * stdev(daily rets) * sqrt(window)`); `markov_commit` requires
N consecutive labels before a state change; `markov_decay` (<1) recency-weights transition counts;
`markov_shrink` blends per-symbol matrices toward the pooled-universe matrix (Dirichlet prior);
`markov_horizon` raises the signal to an n-step `P^n` forecast; `markov_signal_mode` is `spread`
(default) or `bear_rel` (stationary-relative bear excess). Overlay runs write
`<variant>__markov_summary.csv` (regime mix, persistence, stationary mix, risk-off share, flips).
`tradable`/`pool` are never modified.

`markov_model: hmm` swaps the observable chain for a Gaussian-emission HMM (`src/hmm.rs`, port of
`docs/strategies/markov/markov_regime_models.py`): six OHLCV features (log return, realized vol,
normalized ATR, log volume change, rolling skew, MA spread) standardized on each train window;
Baum-Welch EM (`hmm_states`, `hmm_restarts`, `hmm_max_iter`/`hmm_tol`/`hmm_min_iter`,
`hmm_cov_floor`, `hmm_covariance: diagonal|full` (default `diagonal`, full covariance
models cross-feature correlations), `hmm_trans_prior`) refits every `hmm_refit_every` bars on an expanding or
`hmm_train_window` rolling window; between refits the frozen model filters causally. Hidden states
are ordered by weighted mean return into Bear/Sideways/Bull; `hmm_probs` picks the filtered
(default) or predicted distribution for the same spread/bear_rel signal, so `markov_source`,
`markov_mode`, `markov_horizon`, and `markov_position` all carry over (`breadth` stays chain-only;
the chain labeling/counting levers are ignored with a warning). Overlay runs additionally write
`<variant>__hmm_fits.csv` (per-refit loglik, EM iterations, convergence, state occupancy).
Standalone study: `hmm_regime` (`configs/hmm_regime.yaml`).
Set `hmm_pca_components` to a positive count to fit unwhitened PCA on each scaled
training window; `0` (default) disables PCA. Run `scripts\run_hmm_pca.bat` for
the configured PCA experiment matrix and its DEV-selected report.
S041 and S042 use explicitly weighted rebalance legs, so an outgoing overnight segment and an
incoming intraday segment are not divided by their combined position count on rebalance days.
Their existing cost model charges full entry and exit costs on each rebalance leg, including
unchanged holdings, rather than netting those holdings.
Monthly studies set `periods_per_year: 12` for annualized Sharpe, CAGR, and turnover. The DEV bias
currently adjusts for fill-model correlation, not for the full variant search; DEV rankings are
exploratory, and an already inspected TEST period is not a fresh holdout. When DEV skill ties
exactly, the Family 4 catalog uses the last configured candidate, matching the symbols overview.

## S041 execution research

The existing S041 portfolio study is a research-scale daily-bar simulation; its published
TEST period has already been inspected. A separate `s041_cash_study` runner evaluates a
configurable whole-share cash-account scenario from $10,000 using adjusted total-return bars
for signals and separately configured raw daily bars for fills and marks. It writes daily
portfolio, completed-trade, order-event, corporate-action, missing-quote, manifest, report,
lifecycle-event, cost-ladder, DEV-selection, and plot outputs under `results/s041_cash/`.
Run the baseline with `cargo run --release --bin s041_cash_study -- --random-draws 100`;
run baseline plus named overlays with, for example,
`cargo run --release --bin s041_cash_study -- --variant size25,size50,dca3,stop5,pyramid10 --random-draws 1`.
Use `--check-data` to validate input coverage without running the replay. A comma-separated
variant call includes baseline automatically and writes a DEV-only candidate-selection summary.

The runner models next-open fills using only the preceding completed session's volume,
whole-share no-leverage accounting, configurable participation/costs, dividend entitlement
and payment, split adjustments, scheduled DCA installments, stop exits, one-time winner adds,
and deterministic random-pick baselines. Its 44-configuration baseline-inclusive registry
includes sizing, momentum/breadth, participation, trading costs, delisting-return sensitivity,
and lifecycle overlays. It writes every attempted configuration to `variants_comparison.csv`
and appends trial records; `selection.md` uses positive DEV skill plus the stricter of 25% or
SPY DEV drawdown for eligibility, then equal-weight DEV percentile ranks for skill, Sharpe,
profit factor, and drawdown. It selects from the latest saved result per named configuration,
so an interrupted grid can resume without rerunning completed variants; blocked configurations
are excluded. TEST never ranks candidates.

The corrected one-draw screen completed all 44 named configurations with none blocked;
`mom315` now completes after correcting the UTX/RTX spin-off history. DEV-only selection still
names `size25`, but the single random-pick draw is preliminary and TEST has already been
inspected. This is not a validated winner or a fresh holdout; see
`results/s041_cash/selection.md` and the selected variant report.

After the configured
number of consecutive missing raw closes, it settles a held position using an explicit
assumed delisting return (default -30%); this is a sensitivity assumption, not historical
delisting proceeds. Missing raw symbols are not substituted with adjusted prices. Daily-bar
opens, participation, and synthetic event timestamps are scenarios, not verified intraday
execution. Short availability is assumed when enabled; borrow availability, spreads, market
impact, capacity, and broker-specific execution remain unestablished. The runner is not a
brokerage-certified cash account, and the short scenarios use assumed locate/collateral
availability. Current results remain diagnostic until raw-data coverage, incomplete corporate
actions, the cost sensitivity search, and robust placebo evaluation are resolved; TEST is not
a fresh holdout. The current local universe has 1,399 signal symbols, 1,221 raw files, 178
missing raw files, and 192 held dividend entitlements without a payable date. Split-adjusted
files exist for those 178 names, but are deliberately not used as executable prices. The
missing symbols and per-date quote gaps are listed in the study outputs.

The S041 decile now uses the eligible universe on each rebalance date, not the median
eligible count across the full sample. Results produced before this correction are not
comparable for decile variants. The corrected 33-configuration research grid has no
positive-DEV-skill candidate meeting the stricter of a 25% or SPY DEV drawdown limit.
`tools\s041_dev_screen.py` ranks eligible candidates by equal-weight DEV percentile
ranks for skill, Sharpe, profit factor, and drawdown; it never uses TEST to select.
`tools\s041_frequency.py` reports gross modeled entries, completed legs, and rebalance
dates separately for DEV and TEST. These tools do not add whole-share account realism.

For the optional intraday data environment, install the pinned packages with the
configured Windows interpreter. For example:

```powershell
& 'C:\data\envs\trading_framework\Scripts\python.exe' -m pip install -r requirements-data.txt
& 'C:\data\envs\trading_framework\Scripts\python.exe' tools\fetch_s041_minutes.py --symbols SPY AAPL --start 2026-10-05 --end 2026-10-12 --interval 1m --out-dir data\cache\s041_minute\example
```

The downloader saves unadjusted one- or two-minute snapshots and a manifest with retrieval
time, coverage endpoints, and file hashes. It does not establish historical publication
times, revision history, consolidated-exchange coverage, or the absence of missing bars;
these snapshots are not a verified point-in-time archive. Yahoo's data usage terms and
short historical retention apply. `tools\s041_open_diagnostic.py` compares modeled
rebalance legs with the observed 09:30 and 09:32 bar opens, rejecting missing opening
bars or mismatched hashes. It does not simulate fills, whole shares, cash, netted orders,
or portfolio performance.

An alternative data-only Alpaca IEX snapshot can be obtained with an authorized account:

```powershell
& 'C:\data\envs\trading_framework\Scripts\python.exe' tools\fetch_s041_alpaca_minutes.py --trades results\s041_mom_12_1\mom126_skip10__trades_next_open.csv --start 2023-12-29 --end 2024-01-20 --env-file C:\path\to\alpaca.env --out-dir data\cache\s041_minute\example_iex
```

The env file supplies `ALPACA_API_KEY` and `ALPACA_API_SECRET`. This uses the market-data
endpoint only and downloads symbols from overlapping historical trade records; that candidate
list is not a point-in-time trading signal or coverage for other variants. IEX is one exchange,
not a consolidated feed, and retrospective snapshots do not establish revision-time history.
For example, 72 of 74 downloaded symbols have fewer than 390 regular-session bars on
2024-01-02. Missing bars cannot be treated as fills.

Follow up S041 execution research with S042 sector momentum, S043 multi-asset momentum,
and a tradable ETF implementation of S046 industry rotation. The S046 monthly French
industry study is not an executable ETF backtest.
