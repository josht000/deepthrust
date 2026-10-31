# deepthrust
Deep market apha penetration testing in Rust. 

Pure-Rust port of the [trading_framework](https://github.com/josht000/trading_framework) daily-bar study runner.
Currently implemented: **S001 RSI(2) Classic (Connors)** (`configs/s001_rsi2_connors.yaml`).

## Usage

```bash
# smoke test: pulls AAPL (and SPY as the chart benchmark) daily bars from Yahoo Finance, caches them as
# total-return adjusted CSVs in data/cache/yf, then runs trades -> table -> rank -> partition -> charts
cargo run --release -- --strategy s001_rsi2_connors --profile

# named variants from the config's `variants:` block, other symbols, custom data dir
cargo run --release -- --variant rsi5,gate21 --symbols AAPL,MSFT --data-dir data/cache/yf
cargo run --release -- --check-data          # validate inputs + write a manifest only
cargo run --release -- --rebuild-cache       # re-download from Yahoo
cargo test                                   # unit tests + offline end-to-end smoke test (synthetic bars)
```

CLI flags mirror `run.py`: `--strategy --config --stages --test --rebuild --rebuild-cache --profile --check-data --variant --data-dir`
(plus `--symbols`). Outputs (trades, partition dev/test CSVs, cost ladder, baselines, `report.md`, `summary.svg` chart with the
benchmark's cumulative log return) land in `results/s001_rsi2_connors/`; stage timings are printed as a tree and written to
`results/timing/*.json` (`--profile` adds per-call mean/min/max).
