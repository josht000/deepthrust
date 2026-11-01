# deepthrust
Deep market apha penetration testing in Rust.  On the road to implementing 5/1000 published trading strategies in search of alpha.

Identified: 306/1000

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
cargo run --release -- --check-data          # validate inputs + write a manifest only
cargo test                                   # unit tests + offline end-to-end smoke test (synthetic bars)
```

CLI flags mirror `run.py`: `--strategy --config --stages --test --rebuild --profile --debug --check-data --variant --data-dir`
(plus `--symbols`). Daily input data is read from the configured split-adjusted Parquet files and is never rewritten.
Input-gap and benchmark-fallback warnings, plus the per-symbol trade-count console table, are quiet by default.
Pass `--debug` to print them; `run_strats.bat --debug` forwards the option to every strategy run.
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
S041 and S042 use explicitly weighted rebalance legs, so an outgoing overnight segment and an
incoming intraday segment are not divided by their combined position count on rebalance days.
Their existing cost model charges full entry and exit costs on each rebalance leg, including
unchanged holdings, rather than netting those holdings.
Monthly studies set `periods_per_year: 12` for annualized Sharpe, CAGR, and turnover. The DEV bias
currently adjusts for fill-model correlation, not for the full variant search; DEV rankings are
exploratory, and an already inspected TEST period is not a fresh holdout. When DEV skill ties
exactly, the Family 4 catalog uses the last configured candidate, matching the symbols overview.
