---
name: implement_strat
description: Implement one trading strategy from a source document (usually an S-numbered entry in docs/strategies/Trading Strategy Catalog) into src/strategies/<name>/ as a registered study, run it plus up to 16 registered variants, mark the source doc's table of contents IMPLEMENTED, and record a reproduce command. Use when asked to implement, backtest, or study a numbered/named strategy from the catalog or another spec doc.
---

# implement_strat

Implement ONE strategy faithfully, then iterate within a fixed variant budget. Work
alongside other agents: minimal diffs, no refactors, no long comments.

## Inputs
- A strategy spec: usually a `### SNNN | ...` entry in
  `docs/strategies/Trading Strategy Catalog (105 strategies, markdown).md`, but any
  doc/rules/guidelines the user names.
- Conventions to follow: `AGENTS.md` (layout, partition-test requirement, variant
  rules), `README.md` ("Running a study", variant `variants:` block, evaluation
  protocol), and the catalog's "Shared backtest and execution specification" +
  "How to use this catalog" sections.

## Before you write code (required)
1. Read the whole strategy entry plus every field: `status`, `priority`,
   `data_needs`, `parameter_variants`, `merged_from`, `notes`, `review`.
   - `status: dropped` or `demoted`: do NOT implement as a normal study. Tell the
     user; run only the documented purpose (e.g. falsification, control).
   - A `data_needs` code whose table row says "to build" means BLOCKED: tell the
     user which prerequisite is missing instead of faking the data.
2. Open the cited sources BEFORE implementing: links, papers, white papers,
   referenced repos/source code, and `docs/strategies/review/` notes. Confirm the
   doc's rules match the source and harvest buried guidance — editor notes,
   corrections, alternate parameters, "the cited paper actually says..." remarks.
   Record discrepancies; the doc (as corrected by `review:`) wins over the raw
   source.
3. Baseline = the FIRST-listed value of every choice (the catalog's determinism
   convention). Everything after "or"/"optional"/"variant" is a variant, not the
   baseline. Implement the baseline exactly — no improvising.

## Implementation
- Name: `sNNN_<short_name>` for catalog entries (mirror `s075_residual_reversal`).
- `configs/<name>.yaml`: all baseline parameters at top level; optional levers with
  baseline defaults; a `variants:` block of named overlays (dicts of key overrides).
- `src/strategies/<name>/`: thin adapter, `@register(NAME)`, implement the
  `Strategy` operations (see `src/strategies/s075_residual_reversal/strategy.py`).
  Plug into `src/framework` (daily_partition/partition, report, dashboard, timing
  instrumentation) — reuse, don't rebuild.
- Every variant is a trial: `log_trials()` per (strategy, variant, parameter set)
  into `results/trial_registry.jsonl`; `results/<name>/variants_comparison.csv`
  tracks them. Select on DEV only; report TEST once.
- All outputs under `results/<name>/` (never `reports/`); golden copies under
  `results/<name>/golden/`.
- Python: `C:\data\envs\trading_framework\Scripts\python.exe` (plain `python` may
  be 2.7 on this machine).

## Variants budget
- Baseline + AT MOST 100 variants total. Prefer the entry's own
  `parameter_variants` first (they are registered alternatives), then one-lever
  modifications motivated by the baseline's failure mode, then a few combinations.
- Run: `python run.py --strategy <name> --variant a,b,c` (baseline runs with no
  `--variant`; outputs are `<variant>__<file>`).

## Run & report

- Every report must include, for BOTH dev and test: the partition test (trend,
  random-pick baseline, skill, bias, skill %), Sharpe ratio, profit factor,
  # trades, win rate %.
- Catalog validation rules still apply: minimum evidence (>=100 dev / >=30 test
  trades else say `insufficient evidence`), cost ladder, class gates, DECAYED flag
  when test skill sign differs from dev.

## After it runs: update the source doc
1. In the doc's Table of Contents append ` — IMPLEMENTED` to the entry line
   (format exactly like S075).
2. In the strategy's entry add an `implementation_results:` bullet: where the code
   lives, baseline-vs-winner results table (dev/test total, skill, Sharpe,
   max DD, trades dev, trades test), and ALWAYS a `Reproduce:` line with the exact command(s).
   Keep it as terse as the S075 entry.
3. Golden outputs: rebuild into `results/<name>/golden/` per the study's golden
   tooling convention.

## Hard rules
- Minimal changes only. No refactors, no drive-by cleanups, no long notes or
  verbose comments in code — other agents work in this tree concurrently.
- If you notice refactoring that SHOULD have been done, do not do it: leave the
  user a short note describing it.
- NEVER use the Schwab API for real trades — data only.
- If final dev OR test Sharpe > 2 or profit factor > 2, call it out to the user
  explicitly — this is important and unusual; also sanity-check for leakage
  before celebrating.
- If a strategy or variant beats the benchmark by 10% margin, explicitly call it out to the user in the report.
- End with a short report: what was implemented, source discrepancies found,
  baseline result, best variant (dev-selected), one-shot test result, if outperformance vs benchmark by 10% margin, and the reproduce command.
