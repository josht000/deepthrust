---
name: implement_strat
description: Implement one trading strategy from a source document (usually an S-numbered entry in docs/strategies/Trading Strategy Catalog) into signals.rs as a registered study, run it and all registered variants, mark the source doc's table of contents IMPLEMENTED, and record a reproduce command. Use when asked to implement, backtest, or study a numbered/named strategy from the catalog or another spec doc.
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
- Every variant is a trial: `log_trials()` per (strategy, variant, parameter set)
  into `results/trial_registry.jsonl`; `results/<name>/variants_comparison.csv`
  tracks them. Select on DEV only; report TEST once.
- All outputs under `results/<name>/` (never `reports/`)
- Python: `C:\data\envs\trading_framework\Scripts\python.exe` (plain `python` may
  be 2.7 on this machine).
- Attempt to download missing data. Report back to me results and any detlas from the catalog. Indicate IMPLEMENTED and/or DELTA in the catalog if needed. Ensure that all required data files are present and correctly formatted.

## Variants budget
- Baseline + AT MOST 100 variants total and no fewer than 30. You may add other indicators
  as additional signals to clean up false positives, noise, or detect for regime shifts.
  Prefer the entry's own `parameter_variants` first (they are registered alternatives), then one-lever
  modifications motivated by the baseline's failure mode, then a few combinations.

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
4. Update the results/all_trade_metrics.csv with the following header: study,variant,win_pct,mean_ret_pct,pf,sharpe,max_dd_pct,fill,period,days,trades,total_log,trend,trend_beta,beta,beta_t,avg_fraction_invested,avg_positions,turnover_annual,bias,cagr_pct,avg_hold,random_pick_universe,random_p_universe,skill_vs_universe,random_pick_pool,random_p_pool,skill_vs_pool,skill,skill_pct,skill_pct_random,skill_lo,skill_hi,skill_p

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
