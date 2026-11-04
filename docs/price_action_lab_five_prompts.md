# Price Action Lab · The Five Prompts

Screenshot these and run them in order, one at a time.

| # | Prompt | Page |
|---|--------|------|
| 1 | Data and dials | page 2 |
| 2 | Engine and level tests | page 6 |
| 3 | The funnel | page 10 |
| 4 | The final test | page 13 |
| 5 | The dashboard | page 15 |

---

## Prompt 1 of 5: Data and dials

### WHAT THIS BUILDS
A research lab that turns price action trading rules into exact rules a computer can test on NQ and ES futures. This prompt sets up the data and the dials. Later prompts add the engine and the tests, the funnel, the final test and the dashboard, and each one says what it needs from this one.

### RULES
- Work in one project folder your own git worktree. Save every result as a file, and log each decision and its reason in decisions.md.
- Use one master seed for every random draw, so any result can be rebuilt exactly.
- The final window stays sealed. Only the ingest writes it, and nothing reads it until the final test.
- Never move a threshold after seeing a result. Any change goes in a numbered amendment, written before you look.

### INPUTS
You will need to obtain 1-minute NQ and ES continuous front-month futures. Copy each one unchanged into data/futures/raw/ and record its sha256:
- one back-adjusted file per market (parquet or CSV) with the columns ts_event (the UTC time of the bar's open), open, high, low, close, volume and contract (the contract id)
- optionally the same file unadjusted, row for row, for cross-checks only
- a rolls file per market with roll_time_utc, old_contract, new_contract and spread

### CLEAN AND CHECK
- Convert to America/New_York. A session runs 18:00 to 17:00 ET and is dated by the day it ends; Sunday evening belongs to Monday.
- Audit every file and save the results: duplicate timestamps, timestamps in order, gaps of more than 1, 5 and 60 minutes inside a session (counted, never filled), high and low consistent with open and close, zero-volume bars, and adjusted minus unadjusted prices stepping only at rolls. Drop any bar in the 17:00 halt hour.
- Use the back-adjusted bars for every signal and fill. No entries in the last 60 minutes before a roll; an open trade closes at the last bar before it.
- Keep a news calendar of CPI, jobs report and FOMC days from the official sources.

### WINDOWS
- Warm-up, 2021-10-01 to 2021-12-31: builds detector state only, never traded.
- Discovery, 2022-01-03 to 2024-07-31: every rule is found and judged here. Era 1 is 2022, era 2 is 2023 to July 2024.
- Final window, 2024-08-01 to the end of the data (2026-10-16 here): write it once to data/sealed/ with a sha256 manifest. Add a guard to every file reader in the lab that raises an error on any read of data/sealed/, or of any session dated in the final window, unless an explicit unlock is on: a named step plus an environment variable, with every unlock written to an access log. Test the guard now.
- With other dates, keep the shape: a short warm-up, about two and a half years of discovery, then the last two years sealed.

### CHARTS
- Build 5m, 15m, 1h, 4h and daily bars from the 1-minute bars, counted from the 18:00 open. The 4h bars run 18-22, 22-02, 02-06, 06-10, 10-14 and 14-17 ET; the daily bar is the whole session.
- A higher-chart bar is usable only from its close. ATR is Wilder's 14-bar ATR using only bars before the current one.

### THE DIALS
Turn every price action rule into dials with fixed values, and write them to rulebook.md and rulebook_spec.json:
- Ten setups: level bounce, breakout, flip zone, trend pullback, trend line, break of structure continuation, change of character reversal, sweep and reclaim, supply and demand zone, fair value gap.
- Six setup charts (1m, 5m, 15m, 1h, 4h, daily), each with a trigger chart of the same chart or one or two steps lower (daily setups trigger on 1h).
- The detectors behind them: swings of strength 2, 3, 5 or 8; trend from the last two swing highs and lows; break of structure and change of character as the first close past the last confirmed swing, with or against the trend; levels as clusters of swings within 0.1, 0.25 or 0.5 ATR, needing 1 to 4 touches, drawn as a line or a box; trend lines; supply and demand zones from runs of 2 to 5 candles covering 1.5 to 3 ATR; three-candle fair value gaps.

---

## Prompt 2 of 5: Engine and level tests

### WHAT THIS BUILDS
The trade engine, and the behavior tests that ask whether price respects levels, flips, breaks, sweeps, gaps and zones more than chance.

### INPUTS
From Prompt 1, or your own equivalent:
- cleaned, back-adjusted 1-minute bars of NQ and ES in the discovery window, with session dates in ET
- the 5m, 15m, 1h, 4h and daily bars built from them, with ATR
- the roll times and the news days
- the detectors: swings, trend, break of structure, change of character, levels, trend lines, zones and fair value gaps

Use discovery data only. The final window stays sealed.

### RULES
- One master seed for every draw, every result saved, every decision logged, no threshold moved after a result.
- No look-ahead: a trigger bar sees only setup-chart bars that have closed. Prove it with tests: every detector and every setup's signals, run on data cut at a random bar, must equal the full run up to the cut.

### ENGINE CHECKS
- Unit tests on made-up price paths, for longs and mirrored shorts: a target needs a tick through, stop and gap slippage, the stop first in one bar, limit windows and cancels, close stops, time and roll exits, one position at a time, half size.
- 20,000 market entries at random discovery minutes with a 1 ATR stop and a 2R target: after costs they should lose a little (the lab got -$9.80 a trade).
- A deliberate cheat that peeks 60 minutes ahead must make a lot of money. Delete it after it runs.

### BEHAVIOR TESTS
Measure what price did after each kind of event, and compare it with the same measure on a random stand-in. Run on NQ discovery, with ES discovery as the replication, on the 5m, 15m, 1h, 4h and daily charts, each alone and pooled. Use swing strength 3 and a 0.25 ATR level tolerance.
- Scoring a touch: it holds if price moves 1 ATR away before any close more than 0.25 ATR through the level; it breaks if that close comes first; it is unresolved after 50 bars.
- Random lines: move each real level up or down by 1 to 3 ATR, at least 0.5 ATR from any real level, keeping its lifetime and touches. Do the same for trend lines, zones and gaps.
- Random moments: compare breaks, sweeps and candles with 20 random bars from the same chart, the same 30 minutes of the day and the same direction.
- The tests: real levels against random lines; hold rate by touch count; a level touched many times as a wall or a target; shrinking bounces; a line against a box; a wick through a level against a close through it; flip zones; follow-through after a break of structure and a change of character at 10, 20 and 50 bars; sweeps against closes through; trends that start with a sweep; trend lines; fair value gaps (hold and fill); supply and demand zones, first return against later ones; pullback depth; engulfing and rejection candles at levels.
- Intervals: 2,000 bootstrap resamples of whole sessions, 95% intervals.
- Verdict: true when NQ pooled points the claimed way with an interval that excludes zero, ES has the same sign and both eras agree; split when only part of that holds; false otherwise.

### DONE WHEN
Every engine test passes, the random entries and the cheat behave as described, and each behavior test has a saved table and a one-line verdict.

---

## Prompt 3 of 5: The funnel

### WHAT THIS BUILDS
Over a hundred thousand strategy versions run through the engine on discovery data, a funnel that filters them, and a judgment of each setup as a whole.

### INPUTS
- From Prompts 1 and 2, or your own equivalent: every dial with its values and the rules for which combinations are valid, and an engine that turns a version into trades and a daily P&L with real costs.
- NQ and ES discovery bars. The final window stays sealed.

### RULES
- One master seed for every draw. Save every version's configuration before running anything, and hash the file.
- Fix every stage and threshold below before the first run.

### THE VERSIONS
- The spread set: 40,000 versions, 4,000 per setup, spread across every dial so that each value appears about as often as random draws would give it, given the dials it depends on. Keep only versions with at least 40 signals on NQ discovery. Fill each setup greedily: draw 400 candidates at a time and add the one that brings the dial shares closest to target.
- Six named versions inside it, each a full system built exactly as it is usually taught (the Bare Minimum, the Daily Version, the Checklist, the Sweep System, the Zone System, the Entry Model), with lab defaults for anything the description leaves out.
- The common set: each setup's usual settings (swing strength 3, both directions, the trigger on the setup chart, a hard stop 0.1 ATR beyond its reference, hold, exit after 96 bars) on all six charts, crossed with its usual entries (a limit at the level, a close back, a rejection or engulfing candle; for breaks, at the close or a limit back), two stops (beyond the level, or beyond the last swing), four targets (1R, 2R, 3R, the next swing), no filter or exactly one (the trend agrees, New York only, skip news days), and each setup's main variants. The lab's grid had 14,016 versions.
- Add a sample of versions as people trade them: per setup, 40% with no filter, 40% with one and 20% with two, every other dial drawn at random, no duplicates. The lab drew 47,329. Grade only versions with at least 40 NQ discovery signals.

### THE FUNNEL
Run every version on NQ discovery and on ES discovery. Then, on NQ:
1. Enough trades: 30 or more.
2. Profitable after all costs.
3. A Sharpe of 0.5 or more.
4. Beat matched random trades. Give each real trade a twin: the same direction, a random entry minute in the same 30 minutes of the day, the same stop and target distances in ATR, the same holding time and the same costs. Build 10,000 sets of twins; the version passes when fewer than 5% of the sets make as much as it did.
5. Held on ES: the same version on ES discovery made money with a Sharpe of 0.3 or more.

A version is a final-test candidate when it passes stages 1 to 4, makes money in both eras, and holds on ES.

### JUDGE EACH SETUP AS A WHOLE
For each setup, chart and market, average the daily P&L of all its versions with equal weight, one contract each. That average is the setup's result. It works in discovery when it made money per year, had a Sharpe of 0.5 or more, and made money in 2022 and in 2023-24 taken separately. Save the list of working setups and the candidates, with their hashes, before the final test.

### ALSO REPORT
The share passing each stage by setup, chart, dial and filter; win rate against profit; the effect of each rule with everything else equal; and the named versions' results.

### DONE WHEN
Every version has saved metrics and a daily P&L file, the funnel and per-setup tables are written, and the candidates and working setups are saved and hashed.

---

## Prompt 4 of 5: The final test

### WHAT THIS BUILDS
The one test on data no rule has seen.

### INPUTS
- From Prompt 1: the sealed final window (NQ and ES 1-minute bars from 2024-08-01), its sha256 manifest, and the guard that blocks every read of it unless a logged unlock is on.
- From Prompt 3, saved and hashed before this prompt: every version's configuration, the final-test candidates, the setups that worked in discovery, and the named versions.
- The engine from Prompt 2, unchanged.

### RULES
- Nothing chosen in discovery changes now: no configuration, rule, threshold or selection.
- Check the hashes of the configurations and the selections before unlocking, and stop if any differ.
- Rehearse first: play the last month of discovery, shifted in price, through the exact final-test code, and fix anything that breaks while the seal is still closed.
- Unlock once, through the guard, with the named step and the environment variable. The guard logs every unlock and relock. The script refuses to run if final results already exist.

### THE RUN
- Join the discovery bars from 2024-05-01 (to warm up the detectors) to the sealed bars. Keep only complete sessions, those with a bar opening at 15:59 ET or later, and treat a dropped session like a roll: close open positions before it.
- Run every version of both sets on NQ and on ES over the final window, with the same costs and fills as discovery.
- Save every version's metrics and daily P&L and commit them, since they cannot be rebuilt without opening the seal again.

### WHAT HELD UP
- A version held up on a market when it made money after costs with a Sharpe of 0.5 or more on that market's final window.
- A setup held up when the equal-weight average of its versions made money per year with a Sharpe of 0.5 or more.
- Report, on NQ and on ES: the share that held up among all versions and among the candidates; the share of versions that made money on each chart; every setup on every chart, per year and as a total over the window, with its Sharpe; each working setup against matched random trades over the same window; the best usual-settings version of each setup, picked by its discovery result; the named versions; and the dials the versions that held up have in common.

### DONE WHEN
The access log shows the unlock, the final tables are written, and every later analysis reads the saved results, never the sealed data.

---

## Prompt 5 of 5: The dashboard

### WHAT THIS BUILDS
One dark presenter dashboard to record the video from: a page per finding, plus slides for the concept beats, with every number traced to a saved file.

### INPUTS
The saved result tables from Prompts 2 to 4: the behavior test results, the funnel tables, the per-setup and per-chart tables, and the final-window tables and daily P&L. Read only these files, never the raw or sealed data.

### RULES
- Every number on screen comes from a saved file. Record each one in a provenance file (source file, row, column, format, value and the text shown), re-read every value with a second, independent reader before writing the page, and refuse to build if anything differs or any digit on the page has no source.
- Titles are plain nouns and subtitles plain descriptions. No em dashes.
- The bottom-right corner, 520 by 368 px, stays empty for the facecam.

### THE PAGE
- One HTML file that works offline, with its data and images inline, on a 1920x1080 stage scaled to fit any window: navy background (#0b0f1a), cyan and amber accents, mono uppercase eyebrows.
- Each finding gets a page: its section as the eyebrow, a title, a one-line subtitle, one chart, at most two big stats, and a short conclusion with a verdict badge (survived, split or didn't survive) where one applies. Green for gains, red for losses.
- Slides for the concept beats: a big white title, a short grey tagline, amber numbered badges, rounded cards, big cyan numbers and very few words.
- Charts and slides build in when a page opens, and R replays it.

### PRESENTING
- Arrow keys, space and page up and down move between pages; Home and End jump; #N in the address opens page N; M lists the sections.
- F shows the facecam area. A pointer ring follows the mouse, and dragging draws a yellow highlighter mark that fades after a few seconds.

### CHECKS
- A layout check in headless Chrome at 1920x1080 and 1280x720: nothing off the stage, nothing in the facecam corner, one-line subtitles, no scrolling.
- A screenshot of every page, and a list of every page's title, subtitle, numbers and conclusion.

### DONE WHEN
The provenance check passes on every number, the layout check is clean at both sizes, and the screenshots match the page list.
