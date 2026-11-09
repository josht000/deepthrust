---
name: pal_dash
description: Build or extend the PAL presenter dashboard — the single offline HTML page set for the ES/NQ Price Action Lab findings (src/pal/dash.rs + tools/check_dash.py). Use when asked to regenerate, fix, or add pages/charts to the PAL dashboard, when provenance or layout checks fail, or when wiring new PAL result tables into the presenter.
---

# pal_dash

Rebuild or extend the offline presenter dashboard for the Price Action
Lab (ES/NQ futures, five-prompts spec prompt 5). One self-contained
`dashboard.html` + provenance trail, generated only from saved result
tables — never raw or sealed data.

## Commands

```bat
target\release\pal.exe dash                                  :: build dashboard.html
C:\data\envs\trading_framework\Scripts\python.exe tools\check_dash.py   :: layout check + screenshots
```

Run both from the repo root (`C:\data\trading\deepthrust`). `pal dash`
writes `results/pal/dash/{dashboard.html,provenance.csv,
provenance_check.json,pages.json}` and REFUSES to write the HTML if the
provenance re-read finds any mismatch. `check_dash.py` then verifies
layout at 1920x1080 and 1280x720 in headless Chrome/Edge and writes
`layout_check.json` + `shots/*.png` (exit 1 on violations; set
`CHROME_PATH` if no browser is found).

## Architecture (src/pal/dash.rs, ~970 lines)

- `Book` — caches parsed CSVs (`read_csv`, naive `split(',')` — no quoted
  fields allowed in sources) and JSONLs (`read_jsonl`, objects keyed by
  `"name"`). All paths are relative to `results/pal/`.
- `Prov` + `Book::num(page, element, file, row, col, fmt)` — THE way every
  on-screen number enters the page. It looks up the value, applies the
  format (`money`, `pct`, `f2`-`f4`, `int`), records provenance, and
  returns the display text. Special columns: `__count__` (row count of
  csv/jsonl/json-array), `__sum__:<col>` (column sum). Missing values
  render `NA` — prefer gating the whole page on file existence instead.
- `verify_provenance()` — deliberately a SECOND, independent code path:
  re-opens each source file fresh, re-parses with different code, and
  re-formats. Any value or format mismatch aborts the build. Keep it
  independent — do not "refactor" it to share `Book`'s readers.
- `build_pages(root)` — returns `(Vec<Page>, Vec<Prov>)`. Each page is
  gated on its source file existing, so the dashboard degrades
  gracefully as prompts 2-4 results land.
- `render_html(pages)` — one `<style>`+`<script>` block, no external
  assets. `run_dash` writes provenance.csv, runs the check, writes
  pages.json, then dashboard.html.

## Page contract (spec prompt 5)

- Stage: fixed 1920x1080 `#stage`, CSS-scaled to any window. Palette:
  navy `#0b0f1a`, cyan `#22d3ee`, amber `#f59e0b`, green `#34d399`,
  red `#f87171`. Mono uppercase `.eyebrow` = section name.
- Finding page: eyebrow, plain-noun title, one-line subtitle (no em
  dashes), ONE chart, at most two big stats, short conclusion, optional
  verdict badge (`supported`/`survived`→SURVIVED ok, `split`→SPLIT mid,
  else DIDN'T SURVIVE bad). Slides (`slide: true`) are concept beats:
  big white title, grey tagline, amber numbered badges, cards.
- Facecam corner: bottom-right 520x368 (`FACECAM` const) must stay
  empty — the layout check fails on any element intruding.
- Charts build in on page open (`buildline` dash animation); `R`
  replays. Keyboard: arrows/space/pageup/down, Home/End, `#N` deep link,
  `M` section list, `F` facecam placeholder, mouse ring + drag-to-draw
  yellow highlighter (fades). `?check=1` exposes
  `window.__layoutReport()` — do not remove it; check_dash.py depends on it.

## Chart helpers (fixed viewBox SVG strings)

- `svg_bars(&[(label, f64)], w, h)` — diverging horizontal bars, zero
  line at 42% width, green/red by sign, value labels.
- `svg_lines(&[(name, Vec<f64>, color)], dates, w, h)` — cumulative-sum
  polylines with end labels; used for daily-P&L equity curves (feed it
  per-day USD vectors sorted by date).
- `svg_ci(&[(label, diff, lo, hi)], w, h)` — CI-dot chart, dot colored
  by whether the interval excludes zero.

## Data sources consumed (all under results/pal/)

`funnel/{versions.jsonl,candidates.json,working_setups.json,
setup_results.csv,stage_shares.csv}`, `behavior_verdicts.csv`,
`final|final_rehearse/{held_up.csv,named_final.csv,daily_pnl.jsonl}`.
`final/` wins over `final_rehearse/` when both exist (the tag switches
to FINAL TEST vs FINAL REHEARSAL). The equity page takes the top-5 NQ
versions by total P&L from `daily_pnl.jsonl`.

## Extending

1. Add the page inside `build_pages` where its siblings live, gated on
   the new file (e.g. `results.join("funnel/new_table.csv").exists()`).
2. Route EVERY displayed number through `b.num(...)` — hardcoded digits
   violate the spec and are untraceable.
3. Keep titles plain nouns / subtitles plain descriptions, no em dashes.
4. Rebuild (`pal dash`) — provenance check must pass — then
   `check_dash.py`; fix layout violations (off-stage, facecam intrude,
   subtitle overflow, page scroll) before calling it done.
5. New result tables should be written by the producing stage as CSV
   (first column = row key) or JSONL (objects with a `"name"` field) so
   `Book` can read them.

## Pitfalls

- The CSV reader cannot handle quoted commas — keep source tables
  comma-simple or the row/col lookup silently misaligns.
- `prov.text` is what the page shows; `verify_provenance` re-derives it
  from `value`+`format`, so exotic formats need matching arms in BOTH
  places.
- `pages.json` is the checker's page list — regenerate via `pal dash`,
  don't hand-edit.
- Screenshots land in `results/pal/dash/shots/`; `layout_check.json`
  `ok:true` + empty `bad` lists at both sizes = DONE WHEN met.
