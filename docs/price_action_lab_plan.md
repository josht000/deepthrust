# Price Action Lab — Build Plan

Source spec: `docs/price_action_lab_five_prompts.md` (the "five prompts" document).
This plan covers all five phases. The lab lives in this git worktree; the spec's
"one project folder" is this worktree root.

## Project conventions

- Rust engine for everything that runs trades, detectors, studies, and the
  dashboard builder. Python only for one-off tools in `tools/` (data fetch,
  news calendar, headless-Chrome layout check).
- Windows python: `C:\data\envs\trading_framework\Scripts\python.exe`.
- One master seed (`PAL_MASTER_SEED`, default fixed in `rulebook_spec.json`) feeds
  a deterministic xoshiro256** stream; every random draw records its seed.
- `decisions.md` at repo root: numbered decision log. No threshold moves after a
  result; changes go through numbered amendments written before looking.
- Lab artifacts: `results/pal/` for all tables/reports/figures (AGENTS.md
  convention; spec's "save every result as a file").
- Data: `data/futures/raw/` (untouched inputs + sha256 manifest),
  `data/futures/processed/` (cleaned bars, charts), `data/sealed/` (final window),
  `data/futures/news_calendar.csv`. Bar files gitignored; manifests committed.

## Data plan (Databento)

Spec inputs match Databento's schema (`ts_event`). Pull `GLBX.MDP3` `ohlcv-1m`
per-contract for ES and NQ quarterly contracts spanning 2021-09 → 2026-10
(~21 contracts per market). Build the continuous front-month ourselves:

- Roll rule: volume-flip roll, recorded per market in `rolls` files with
  `roll_time_utc, old_contract, new_contract, spread`.
- Splice contracts at roll, back-adjust all earlier history by the cumulative
  roll spread. Save adjusted + unadjusted (row-for-row) parquets — gives full
  audit control instead of trusting a vendor's adjustment.
- `tools/fetch_databento_futures.py`: needs `DATABENTO_API_KEY`
  (**still required from user**). Writes raw parquets + `manifest.sha256`.
- `tools/fetch_news_calendar.py`: CPI / jobs-report / FOMC dates from official
  calendars → `data/futures/news_calendar.csv` (documented source in file header).

Fallback: if Databento doesn't pan out, same ingest accepts CSV/parquet drops
into `data/futures/raw/` from any vendor.

## Crate layout

One binary target added to the existing package: `src/bin/pal.rs`, modules under
`src/pal/` (pub via `pal` mod in lib.rs). Keeps the daily-bar engine untouched.

```
src/pal/
  types.rs      Bar, Session, Contract, Roll, NewsDay, ChartId, DialSet, Version
  rng.rs        seeded RNG, bootstrap resampler
  store.rs      canonical bar store + THE SEAL GUARD (all reads go through here)
  sessions.rs   UTC→ET, session dating (18:00→17:00 ET, end-day dated, Sun→Mon),
                17:00 halt-hour drop
  ingest.rs     raw file load, normalize, sha256, audit trail
  audit.rs      duplicate ts, ordering, intra-session gaps (>1m/5m/60m, counted
                never filled), OHLC consistency, zero-volume, adj-step-at-rolls
  resample.rs   1m → 5m/15m/1h/4h(18-22..14-17)/daily anchored at 18:00
  atr.rs        Wilder ATR-14 on closed bars only
  windows.rs    warm-up/discovery/era1/era2/sealed date ranges
  detect/
    swings.rs     swing highs/lows, strength {2,3,5,8}
    trend.rs      trend from last two swing highs + lows
    structure.rs  BOS / CHoCH (first close past last confirmed swing)
    levels.rs     swing clusters within {0.1,0.25,0.5} ATR, 1-4 touches, line|box
    trendlines.rs
    zones.rs      supply/demand: runs 2-5 candles covering 1.5-3 ATR
    fvg.rs        three-candle fair value gaps
    candles.rs    engulfing / rejection patterns
  rulebook.rs   dial definitions → rulebook_spec.json + rulebook.md
  setups/       the 10 setups: level_bounce, breakout, flip_zone, trend_pullback,
                trend_line, bos_continuation, choch_reversal, sweep_reclaim,
                supply_demand_zone, fair_value_gap — each maps dials → entry
                signals on the trigger chart
  engine.rs     fills: market/limit; tick-through targets; stop-first on
                ambiguous bar; gap slippage; close stops; time/roll exits;
                one position; half size; per-market tick value + costs
  behavior.rs   event measures, random lines / random moments stand-ins,
                session bootstrap (2000), verdicts (true/split/false)
  funnel.rs     version space, spread set (greedy fill), common grid, human
                draws, 6 named versions, twin-trade test, stage filters
  finaltest.rs  hash checks, rehearsal on shifted discovery tail, unlock
                ceremony, final run + commit
  report.rs     CSV/MD table writers
  dash.rs       prompt-5 dashboard: provenance map, independent re-reader,
                inline-data HTML emit
```

CLI (`pal`): `ingest | audit | seal | guard-test | charts | rulebook |
engine-check | behavior | funnel | final-test | dash`. Every subcommand writes
its outputs under `results/pal/` and appends what it did to `decisions.md`
where a decision was made.

## Key design: detection vs execution split

The funnel runs ~100k versions on ~2×700k discovery minutes. Naive per-version
detection is the bottleneck. Instead:

1. **Detector cache**: for each (chart, swing-strength, tolerance, touches, ...)
   dial-tuple, compute event streams once → memory-mapped cache in
   `results/pal/cache/detect/`. ~hundreds of distinct tuples, not 100k.
2. **Engine per version**: consumes cached events + entry/stop/target/filter
   dials → trades + daily P&L. Target <10ms/version/market → ~1-2 hours
   single-threaded worst case, parallelize by version shard if needed.

This also makes the no-lookahead proof cheap: cached event streams must be
identical when computed on data truncated at a random bar.

## Seal guard

All lab reads route through `store::read_*`. The guard raises an error when a
request touches `data/sealed/` paths OR sessions dated ≥ 2024-08-01, unless
unlock is active: named step env `PAL_STEP=final-test` AND `PAL_UNLOCK_SEALED=1`.
Every unlock/relock appends to `data/sealed/access_log.jsonl`. `guard-test`
subcommand exercises it. (Reader-level enforcement — the spec's "guard on every
file reader in the lab".)

## Cost model (user-specified, decision #8)

- ES: tick 0.25 = $12.50; NQ: tick 0.25 = $5.00.
- Commission $2.00/side ($4.00 round-turn; user range $3.50–$4.50).
- Limit orders: touch = 100% fill at limit price, 0 slippage (user).
- Market/stop orders: 1 tick slippage; gap-through fills at gap price.
- Sanity-checked against the spec's -$9.80/trade random-entry benchmark
  (NQ 1ATR-stop/2R randoms should land near-cost-negative).

## Windows

- Warm-up 2021-10-01 → 2021-12-31 (detector state only)
- Discovery 2022-01-03 → 2024-07-31; era1 = 2022, era2 = 2023→2024-07
- Sealed 2024-08-01 → 2026-10-16 (end of data)

## Phase checklist (spec order)

1. **Data & dials**: fetch tool, ingest+sha256, audits→`results/pal/audit_*`,
   sessions, halt drop, rolls, news calendar, windows, seal+manifest+guard+test,
   resampled charts, ATR, `rulebook.md` + `rulebook_spec.json`.
2. **Engine & level tests**: engine + unit tests on synthetic paths (long &
   mirrored short), 20k random entries sanity, cheat-peeks-60m test (deleted
   after run), no-lookahead truncation tests, behavior suite → per-test tables +
   one-line verdicts.
3. **Funnel**: version registry (hashed configs saved before any run), spread
   set 40k (greedy, ≥40 NQ signals), common set ~14k, human set ~47k, 6 named;
   stages 1-5; era check; per-setup judgment; all report tables.
4. **Final test**: hash verification → rehearsal (shifted last discovery month)
   → single unlock → join 2024-05-01 warm-up tail + sealed, complete sessions
   only → run everything on NQ + ES → save + commit → held-up analysis.
5. **Dashboard**: provenance file (every number → file/row/col/value/text),
   second independent reader, offline HTML on 1920×1080 stage, facecam corner,
   keyboard nav, headless-Chrome layout check @1920×1080 & 1280×720,
   screenshots + page list.

## Dependencies to add

`chrono-tz` (ET sessions), `rand`/`rand_xoshiro` (seeded RNG). Reuse existing
`parquet`, `csv`, `serde_json`, `sha2`, `plotters`. Python: add `databento` to
`requirements-data.txt`.

## Framework bridge (added 2026-11-09)

`sweep_reclaim` also runs inside the daily study framework as
`pal_sweep_reclaim` so its trades get the standard partition test,
variant matrix, and reports (`src/family_pal.rs`,
`configs/pal_sweep_reclaim.yaml`, `results/pal_sweep_reclaim/`).

- Baseline = funnel candidate `h:sweep_reclaim:1169` (NQ, 5m/5m,
  long-only, `limit_at_level`, beyond-level stop, 3R, 96-bar time exit,
  no filters). `pal_market: es` variants replicate on ES.
- The PAL engine (`detect_events` + `run_version`) generates trades on
  the real 5-minute sessions; trades are mapped to session dates for the
  framework's dev/test split (2023-01-01 = PAL era boundary).
- Trade-level stats are exact PAL economics. The framework's daily
  account series is its standard close-to-close approximation;
  minute-level daily P&L remains authoritative in
  `results/pal/funnel/daily_pnl.jsonl`.
- Reads the discovery window only — no sealed access.
- `engine::Trade`/`partition::Leg` carry `dir_sign` (+1/−1) so short
  and `both` variants get correctly signed daily account stats
  (decision #15). Caveat: the `random_timing` baseline null hardcodes
  long legs, so its p-value row is only meaningful for long-only
  variants; the primary random-pick universe/pool tests do propagate
  direction.
- `entry_atclose`/`entry_limitback` variants produce zero trades by
  design: sweep events carry `SWEEP|CLOSEBACK` flags while those entry
  kinds require `BREAK` (reserved for break-style setups in the
  rulebook). They are kept in the matrix as documentation.
- Causality (decision #16): level geometry is point-in-time —
  `Level::as_of(at)` returns line/lo/hi over touches confirmed <= the
  decision bar; `line`/`lo`/`hi` fields remain final-state for
  `behavior.rs` probes only. Guarded by
  `pal::setups::tests::detectors_are_causal_under_truncation` plus the
  swings/ATR truncation tests.

Reproduce:

```
target\release\deepthrust.exe --strategy pal_sweep_reclaim --variant baseline
target\release\deepthrust.exe --strategy pal_sweep_reclaim --variant es,both,short,trig1m,chart15m,chart15m_t5m,chart15m_t1m,chart1h,chart1h_t15m,chart1d,chart1m,swing3,swing5,tol10,tol50,touches2,entry_closeback,entry_rejection,entry_engulfing,entry_atclose,entry_limitback,stop_swing,target1R,target2R,target_swing,filt_trend,filt_ny,filt_news,both_t1m,short_chart15,es_both,swing3_tol50,rej_stop_swing,target2R_filt_trend,chart1d_es,touches2_swing3,filt_ny_news
:: or the full matrix via run_strats.bat
```

## Open items for user

1. `DATABENTO_API_KEY` — needed before `pal ingest` runs (put in
   `C:\data\trading\trading_framework\.env` or pass via env).
2. ~~Cost model confirm~~ resolved 2026-11-03 (decision #8).
