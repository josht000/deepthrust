# Price Action Lab: The Five Prompts

Run the prompts in order. The lab converts price-action ideas into explicit rules, tests them on NQ and ES futures, compares strategy versions, evaluates selected results on a later period, and builds an offline presentation dashboard. Use one recorded master seed for every random draw, save decisions and their reasons, and do not change a rule or threshold after seeing its result; record any necessary change as a numbered amendment before continuing.

| # | Prompt | Page | Builds |
|---|--------|------|--------|
| 1 | Data and dials | 2 | Clean datasets, charts, and rule definitions |
| 2 | Engine and level tests | 6 | Trade engine and behavior-test results |
| 3 | The funnel | 10 | Version results, comparisons, and selections |
| 4 | Test-period evaluation | 13 | Later-period results for selected versions |
| 5 | The dashboard | 15 | Offline presentation and source list |

## Prompt 1 of 5: Data and dials

### What this builds
A repeatable research dataset and a rulebook of measurable price-action setups for NQ and ES. Use a single project folder; save study outputs under `results/price_action_lab/`.

### Inputs and data files
- Obtain one-minute, back-adjusted, continuous front-month futures for each market: `data/futures/raw/nq_1m_back_adjusted.parquet` and `data/futures/raw/es_1m_back_adjusted.parquet`. Required columns: `ts_event` (UTC bar-open time), `open`, `high`, `low`, `close`, `volume`, and `contract`.
- Optional unadjusted cross-check files: `data/futures/raw/nq_1m_unadjusted.parquet` and `data/futures/raw/es_1m_unadjusted.parquet`.
- Roll schedules: `data/futures/raw/nq_rolls.csv` and `data/futures/raw/es_rolls.csv`, with `roll_time_utc`, `old_contract`, `new_contract`, and `spread`.
- Event calendar: `data/futures/processed/news_calendar.csv`, listing official CPI, jobs report, and FOMC dates.

### Dates and processing
- Warm-up: 2021-10-01 to 2021-12-31. Build detector state; do not score trades.
- Dev: 2022-01-03 to 2024-07-31. Use for rule development, comparisons, and selection. Era 1 is 2022; era 2 is 2023-01-01 to 2024-07-31.
- Test: 2024-08-01 to 2026-10-16 (or the latest available date). For other studies, use a short warm-up, about two and a half years for dev, and the most recent two years for test.
- Put both splits side by side in `data/futures/processed/`, not in separate directories: `dev_nq_1m.parquet`, `dev_es_1m.parquet`, `test_nq_1m.parquet`, and `test_es_1m.parquet`.
- Convert timestamps to America/New_York. A session is 18:00–17:00 ET and dated by its ending day; Sunday evening belongs to Monday. Drop the 17:00 halt hour.
- Audit duplicates, ordering, gaps over 1/5/60 minutes, OHLC consistency, zero-volume bars, and adjustment steps at rolls. Count gaps; do not fill them. Use adjusted bars for signals and fills. No entries in the last 60 minutes before a roll; close open trades on the last bar before it.
- Build 5m, 15m, 1h, 4h, and daily bars from 1m data, counted from the 18:00 open. The 4h intervals are 18–22, 22–02, 02–06, 06–10, 10–14, and 14–17 ET. Daily bars cover one session. A higher-timeframe bar is available only after it closes. Calculate Wilder 14-bar ATR using earlier bars only.
- Save corresponding bar files beside the 1m files, named `{split}_{market}_{timeframe}.parquet` (for example, `dev_nq_5m.parquet` and `test_es_daily.parquet`).

### Rulebook
Define each setup with fixed, testable dials and save the definitions to `results/price_action_lab/rulebook.md` and `results/price_action_lab/rulebook_spec.json`.

Include ten setups: level bounce, breakout, flip zone, trend pullback, trend line, break-of-structure continuation, change-of-character reversal, sweep and reclaim, supply-and-demand zone, and fair value gap.

Cover setup charts 1m, 5m, 15m, 1h, 4h, and daily. Triggers use the setup chart or one/two steps lower; daily setups trigger on 1h. Define swing strengths 2/3/5/8; trend from the last two swing highs and lows; structure breaks as the first close past the last confirmed swing (with-trend break of structure versus against-trend change of character); level clusters within 0.1/0.25/0.5 ATR and 1–4 touches; line or box levels; trend lines; zones from 2–5 candles spanning 1.5–3 ATR; and three-candle fair value gaps.

Save the processing audit as `results/price_action_lab/data_audit.csv`.

## Prompt 2 of 5: Engine and level tests

### What this builds
A trade simulator and evidence tables measuring whether price behavior around levels, flips, breaks, sweeps, gaps, and zones differs from matched random comparisons.

### Inputs
Use Prompt 1 outputs: the dev and test bar files, roll schedules, news calendar, and rulebook. Develop and calibrate with dev data.

### Engine checks
- Test long and mirrored short cases for tick-through targets, stop/gap slippage, stop-first bars, limit windows and cancels, close stops, time/roll exits, one position at a time, and half size. Save results to `results/price_action_lab/engine_tests.csv`.
- Run 20,000 random dev-market entries with a 1 ATR stop and 2R target; after costs, results should lose modestly (reference: -$9.80 per trade). Test the engine with a temporary 60-minute look-ahead example; it should show obvious inflated performance. Do not include the example in the research engine.
- Verify no look-ahead by cutting data at random bars and comparing detector and signal output up to each cut with the full run.

### Behavior tests
- On NQ dev, replicated on ES dev, evaluate 5m, 15m, 1h, 4h, and daily charts, individually and pooled. Use swing strength 3 and 0.25 ATR level tolerance.
- A touch holds if price moves 1 ATR away before a close more than 0.25 ATR through the level; it breaks if that close comes first; otherwise it is unresolved after 50 bars.
- Compare real levels with displaced random lines (1–3 ATR away, at least 0.5 ATR from real levels, preserving lifetime and touches). Apply corresponding matched comparisons to trend lines, zones, and gaps. Compare breaks, sweeps, and candles with 20 random bars from the same chart, half-hour, and direction.
- Measure hold rate by touch count, whether repeated touches act as a wall or target, shrinking bounces, line versus box, wick versus close breaks, flips, structure-break/change-of-character follow-through at 10/20/50 bars, sweeps versus closes through, trends starting with sweeps, gap holds/fills, first versus later zone returns, pullback depth, and engulfing/rejection candles at levels.
- Use 2,000 whole-session bootstrap resamples for 95% intervals. Label results true when pooled NQ supports the claim with an interval excluding zero, ES has the same sign, and both dev eras agree; split when only some conditions hold; otherwise false.

Save event-level observations to `results/price_action_lab/behavior_events.csv` and summaries/verdicts to `results/price_action_lab/behavior_summary.csv`. Finish when engine tests pass and each behavior test has a result and verdict.

## Prompt 3 of 5: The funnel

### What this builds
A broad comparison of strategy versions on dev data, a staged funnel, and an equal-weight assessment of each setup.

### Inputs and versions
Use the Prompt 1 rulebook and Prompt 2 engine with NQ and ES dev bars. Use the recorded master seed and save each version's configuration before running it. Set the funnel stages and thresholds before the first run.

- Spread set: 40,000 versions, 4,000 per setup, balanced across applicable dial values. Keep versions with at least 40 NQ dev signals. Draw 400 candidates at a time and add the candidate that best improves dial balance.
- Named set: the Bare Minimum, Daily Version, Checklist, Sweep System, Zone System, and Entry Model, each implemented as a complete system with lab defaults for unspecified rules.
- Common set: usual settings (swing strength 3, both directions, trigger on setup chart, hard stop 0.1 ATR beyond reference, hold, exit after 96 bars), all six charts, each setup's usual entries (level limit, close back, rejection/engulfing; breaks at close or limit retest), two stops (beyond level or last swing), four targets (1R/2R/3R/next swing), no filter or one filter (trend agrees, New York only, or skip news days), and each setup's main variants. Reference size: 14,016 versions.
- Trade-style sample: per setup, 40% no filter, 40% one filter, 20% two filters; draw other dials randomly, without duplicates. Reference size: 47,329. Grade only versions with at least 40 NQ dev signals.

Save the complete inventory to `results/price_action_lab/version_catalog.csv` and each configuration as `results/price_action_lab/versions/{version_id}.json`. Save per-version metrics and daily P&L to `results/price_action_lab/version_metrics.csv` and `results/price_action_lab/version_daily_pnl.csv`, keyed by version and market.

### Funnel and setup assessment
Run versions on NQ and ES dev. Apply these stages to NQ: at least 30 trades; profitable after costs; Sharpe at least 0.5; outperform matched random trades (10,000 sets, same direction, half-hour, stop/target ATR distances, holding time, and costs; pass if fewer than 5% match or exceed the version); and profitable on ES with Sharpe at least 0.3. A test-period candidate must pass the first four stages, make money in both dev eras, and meet the ES stage.

For each setup, chart, and market, equally average the daily P&L of its versions at one contract each. A setup works on dev when annual P&L is positive, Sharpe is at least 0.5, and both dev eras are profitable.

Save funnel counts to `results/price_action_lab/funnel_summary.csv`, setup results to `results/price_action_lab/setup_summary.csv`, and selections to `results/price_action_lab/test_candidates.csv` and `results/price_action_lab/working_setups.csv`.

Report stage pass rates by setup, chart, dial, and filter; win rate versus profit; rule effects with other settings held equal; and named-version results. For every trade family, write the established metrics to `results/price_action_lab/all_trade_metrics_{family}.csv`. Include win rate, mean return, profit factor, Sharpe, drawdown, fill, period, days, trades, total log return, trend/beta and skill/baseline statistics. Simulation reports must include partition test (trend, random-pick baseline, skill, bias, skill %), Sharpe, profit factor, trade count, and win rate for dev and test. Plot strategy cumulative log returns against SPY or QQQ.

## Prompt 4 of 5: Test-period evaluation

### What this builds
An evaluation of the versions and setups selected using dev results, using the later test files.

### Inputs and run
Use `test_candidates.csv`, `working_setups.csv`, the saved version configurations, the unchanged engine, and the test bar files.

Add dev bars from 2024-05-01 for detector warm-up, but score only test dates. Use the same costs and fills as dev.

Keep complete sessions, identified by a bar opening at 15:59 ET or later; close open positions before a dropped session. Run selected candidates, working setups, and named versions on NQ and ES.

Run the test in restartable family stages, not as one all-or-nothing batch. A parent runner must invoke one stable trade family at a time (each setup family, plus a named-versions family) and record its status and completed outputs in `results/price_action_lab/test_run_manifest.json`. Each family writes its own `results/price_action_lab/test_all_trade_metrics_{family}.csv`, using the metric columns and definitions in `results/all_trade_metrics.csv` and an explicit market field for NQ versus ES. Publish each completed family's files atomically; on restart, verify and skip completed families and resume only missing or incomplete work using the same inputs and settings. A new run must refuse to overwrite any existing family results; resume only when the manifest and existing outputs validate. Never use test results to alter later families. After all families finish, also write the combined per-version metrics and daily P&L to `results/price_action_lab/test_version_metrics.csv` and `results/price_action_lab/test_version_daily_pnl.csv`, and setup/chart summaries to `results/price_action_lab/test_setup_summary.csv`.

Count a version as held up on a market only if it is profitable after costs and has Sharpe of at least 0.5 on that market's test period. A setup holds up when its equal-weight version average is profitable per year and has Sharpe of at least 0.5. Report profitability after costs and Sharpe per market; the share held up among all versions and among candidates; the share profitable on each chart; every setup/chart result by year and overall; matched-random comparisons for working setups; the best usual-settings version per setup (selected on dev); named-version results; and common dials among versions that held up. Do not change selections based on test results.

## Prompt 5 of 5: The dashboard

### What this builds
An offline, dark presentation dashboard with a page per finding and slides for key concepts. Every displayed number links to a saved result.

### Inputs and files
Read the saved behavior, funnel, setup, test-period, and daily-P&L files in `results/price_action_lab/`. Build:
- `results/price_action_lab/dashboard.html`: a self-contained offline page with inline data/images, a 1920x1080 stage scaled to the window, navy background (#0b0f1a), cyan and amber accents, and an empty 520x368 px bottom-right facecam area.
- `results/price_action_lab/dashboard_provenance.csv`: source file, row, column, format, value, and displayed text for every number; independently re-read values before rendering and stop if any mismatch or unsourced number is found.
- `results/price_action_lab/dashboard_pages.csv`: page title, subtitle, numbers, and conclusion for each page.
- `results/price_action_lab/dashboard_screenshots/page-{page_number}.png`: one screenshot per page.

Use plain-noun titles and descriptive subtitles. Each finding page has a section label, one chart, at most two large statistics, and a short conclusion with a survived/split/did-not-survive badge where applicable. Use green for gains and red for losses. Concept slides use a title, short tagline, numbered badges, rounded cards, and minimal text. Animate content on page open; R replays it.

Support arrow keys, space, and Page Up/Down for navigation; Home/End for first/last; `#N` for a page; M for section list; F for the facecam area. Add a pointer ring and a temporary yellow mark while dragging.

Check headless Chrome layouts at 1920x1080 and 1280x720 for overflow, facecam clearance, one-line subtitles, and scrolling. Finish when provenance, layout checks, page list, and screenshots agree.
