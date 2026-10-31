# deepthrust
Deep market apha penetration testing in Rust. 

Pure-Rust port of the [trading_framework](https://github.com/josht000/trading_framework) daily-bar study runner.
Currently implemented: **S001 RSI(2) Classic (Connors)** (`configs/s001_rsi2_connors.yaml`).

## Usage

```bash
# run the configured daily-bar inputs, then trades -> table -> rank -> partition -> charts
cargo run --release -- --strategy s001_rsi2_connors --profile

# named variants from the config's `variants:` block, other symbols, custom data dir
cargo run --release -- --variant rsi5,gate21 --symbols AAPL,MSFT --data-dir C:\data\trading\trading_framework\data\cache\daily_10years
cargo run --release -- --check-data          # validate inputs + write a manifest only
cargo test                                   # unit tests + offline end-to-end smoke test (synthetic bars)
```

CLI flags mirror `run.py`: `--strategy --config --stages --test --rebuild --profile --check-data --variant --data-dir`
(plus `--symbols`). Daily input data is read from the configured split-adjusted Parquet files and is never rewritten.
Outputs (trades, partition dev/test CSVs, cost ladder, baselines, `report.md`, and a `summary.svg` nine-panel
dashboard with the same panels as the Python PNG: cumulative returns, key metrics, partition decomposition, drawdown,
calendar years, trade returns, holding period, exit reasons, open positions; a configured benchmark is added to panel A)
land in `results/s001_rsi2_connors/`; stage timings are printed as a tree and written to
`results/timing/*.json` (`--profile` adds per-call mean/min/max).
