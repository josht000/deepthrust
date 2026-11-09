# pal_dash

Build or extend DeepThrust presenter dashboards — the single offline
HTML page sets with a full provenance trail, generated only from saved
result tables, never raw or sealed data. The dashboard is its OWN
crate (`dash/`) so deepthrust builds never compile it.

- **PAL deck** — the Price Action Lab findings (`results/pal/`),
  page builders in `dash/src/pal.rs`.
- **Study deck** — ANY standard `results/<study>/` directory
  (partition_dev/test.csv, trade_summary, baselines, daily_returns,
  variants_comparison, cost_ladder, symbols_summary, per_etf). Page
  builders in `dash/src/lib.rs::study_pages`.

## Crate layout

```
dash/
  Cargo.toml        deepthrust-dash, deps: serde_json only
  src/lib.rs        engine: Book/Prov, verify_provenance, Page, SVG
                    helpers, render_html, emit, study_pages,
                    run_study_dash  (~1200 lines)
  src/pal.rs        PAL page builders build_pages + run_dash
  src/main.rs       `dash` CLI: bare | --study <name> | --all
```

Standalone: `cd dash && cargo build --release` produces
`dash/target/release/dash.exe` with its own Cargo.lock and target/
(gitignored). Deepthrust does NOT depend on it.

## Commands

```bat
target\release\pal.exe dash                      :: forwards to dash.exe (PAL deck)
target\release\pal.exe dash --study g070_supertrend
target\release\pal.exe dash --all
dash\target\release\dash.exe [--study <name>|--all]   :: direct, no pal needed
C:\data\envs\trading_framework\Scripts\python.exe tools\check_dash.py [--html <dir>/dashboard.html]
```

Run from the repo root. `pal dash` shells out to `dash.exe` (located
via CARGO_MANIFEST_DIR at pal's compile time); if the exe is missing it
prints the build command. Each build writes `<out>/dashboard.html` +
`provenance.csv`, `provenance_check.json`, `pages.json` and REFUSES
the HTML if the independent provenance re-read finds any mismatch.
`check_dash.py` verifies layout at 1920x1080 and 1280x720 in headless
Chrome/Edge (needs `CHROME_PATH` if no browser is found) and writes
`layout_check.json` + `shots/*.png`; it resolves pages.json/shots next
to whatever `--html` you pass, so the same command works for every deck.

## Provenance rules (spec prompt 5 — applies to BOTH decks)

- Every on-screen number goes through `Book::num`/`numk` (or the
  `pv()` wrapper in study_pages, which also returns the chart value).
  Row keys are the LEADING CONTIGUOUS columns: single key = first
  column (`fill`, `baseline` name); multi key = `k1|k2|..` joined by
  `|` and matched positionally — e.g. `variants_comparison` uses
  `variant|fill`, `cost_ladder` needs `fill|cost_mult|cost_bps_side|
  period` because `cost_bps_side` sits between `cost_mult` and
  `period`. Check the header before writing keys.
- Special columns: `__count__` (row count), `__sum__:<col>`,
  `__uniq__:<col>` (distinct values — used for symbol/variant counts).
  Both `num()` and `verify_provenance()` implement them — keep the two
  implementations in sync.
- Formats (`fmt_value`, shared by display AND verify): `money`, `pct`
  (0-1 fraction -> "52%"), `pct1` (already-percent -> "51.9%" —
  partition tables store win_pct 0-100), `f2`-`f4`, `int`.
- `Book`'s CSV reader is a naive `split(',')` — no quoted fields.
- Missing values render `NA`; prefer gating whole pages on file
  existence so decks degrade gracefully.

## Study deck pages (study_pages)

Slide (symbols/variants/dev+test trade cards) -> Dev Versus Test ->
Trade Profile -> Skill Decomposition -> Buy Hold And Timing (baselines
incl. random_timing p) -> Equity Curve (per-fill `*_r_day` + `b_day`
from daily_returns.csv) -> Dev Leaderboard (top 16 variants by
`dev_total_log`, primary fill only) -> Cost Ladder -> Where It Earned
(per-symbol, baseline variant, skipped for single-symbol studies) ->
closing slide. Each page is gated on its source CSV existing.

## Page contract

- Stage: fixed 1920x1080 `#stage`, CSS-scaled. Palette navy `#0b0f1a`,
  cyan `#22d3ee`, amber `#f59e0b`, green `#34d399`, red `#f87171`.
- Finding page: eyebrow, plain-noun title, one-line subtitle (no em
  dashes), ONE chart, <=2 big stats, short conclusion, optional verdict
  badge (`supported`/`survived`->SURVIVED, `split`->SPLIT, else
  DIDN'T SURVIVE). Slides: big white title, grey tagline, amber
  numbered badges, cards.
- Facecam corner bottom-right 520x368 (`FACECAM`) must stay
  empty — layout check fails on intrusions.
- Charts animate on page open; `R` replays. Keys: arrows/space/
  pageup/down, Home/End, `#N` deep link, `M` section list, `F`
  facecam, mouse ring + drag highlighter. `?check=1` exposes
  `window.__layoutReport()` — check_dash.py depends on it; do not
  remove.

## Extending

1. Study deck: add a gated block inside `study_pages` in
   `dash/src/lib.rs` reading the new CSV via `b.read_csv` + `col_idx`,
   route every number through `pv()`.
2. PAL deck: add the page inside `build_pages` in `dash/src/pal.rs`.
3. Rebuild: `cd dash && cargo build --release` — provenance must pass —
   then `check_dash.py --html <that deck>/dashboard.html`; fix layout
   violations (off-stage, facecam, subtitle overflow, page scroll)
   before calling it done.
4. New result tables: write CSV (first columns = row key) or JSONL
   (objects with `"name"`) so `Book` can read them.

## Pitfalls

- `svg_bars` scale: bars are `value * min(avail/max)` — do NOT re-add a
  `.max(1.0)` clamp; it forces 1px-per-unit and huge values (63k PAL
  trades) render off-stage. Fixed once; keep it that way.
- `win_pct`-style columns in study tables are 0-100 — use `pct1`, not
  `pct` (which expects 0-1 fractions like PAL's own tables).
- `prov.text` is what the page shows; verify re-derives it from
  `value`+`format`, so exotic formats need matching arms in BOTH
  places (fmt_value is shared — one arm covers both).
- `pages.json` is the checker's page list — regenerate, don't
  hand-edit. `results/` is gitignored; decks are local artifacts.
- Screenshots land in `<deck>/shots/`; `layout_check.json` `ok:true` +
  empty `bad` lists at both sizes = DONE WHEN met.
- After moving machines: `cd dash && cargo build --release` once, then
  `pal dash` works; deepthrust builds alone never produce dash.exe.
