---
name: count_strats
description: Count identified vs implemented trading tests across all DeepThrust sources (105-catalog, GitHub catalog, Markov methods, PAL lab in its worktree), deduplicating catalog overlaps. Use when asked how many strategies/tests exist, how many are implemented, or for a progress inventory.
---

# count_strats

Report `Total implemented / Total identified` per source, plus a deduplicated
grand total and the number of entries dropped as duplicates.

## Sources and how to count each

Use the env python `C:\data\envs\trading_framework\Scripts\python.exe`
(plain `python` may be 2.7 on this machine). All paths below are relative to a
worktree root.

### 1. Trading Strategy Catalog (S###) — main worktree only
File: `docs/strategies/Trading Strategy Catalog (105 strategies, markdown).md`

- **Identified** = unique `S\d{3}` ids (105).
- **Implemented** = unique S-ids that are EITHER marked `— IMPLEMENTED` on a
  table-of-contents line (`^\s*- \[S\d{3}`) OR carry `rust_status: IMPLEMENTED`
  in their body section (e.g. S004 is implemented but TOC-deferred). Take the
  union of both signals. Cross-check: `ls configs/ | grep -cE '^s[0-9]'` should
  match the union count.

### 2. GitHub Strategy Catalog (G###) — union across worktrees
File: `docs/strategies/GitHub Strategy Catalog.md` in BOTH
`C:/data/trading/deepthrust` (main) and `C:/data/trading/deepthrust-gstrats`
(`g-catalog-tier12` branch). Implementation lands on the tier branch before
merge, so count the UNION of IMPLEMENTED G-ids across both copies.

- **Identified** = unique `G\d{3}` ids (209).
- **Duplicates (dropped)** = TOC entries tagged `— DUPLICATE (S###)`; these
  restate an S-catalog strategy and do NOT count as new identified tests.
  `grep -c "DUPLICATE (S"` on the newest copy of the file. Unique identified
  = identified − dropped.
- **Implemented** = unique `G\d{3}` on lines containing `IMPLEMENTED`, unioned
  across worktrees. Verify no implemented G is also dup-tagged
  (`grep -E "DUPLICATE.*IMPLEMENTED|IMPLEMENTED.*DUPLICATE"` should be empty).

### 3. Markov regime methods
File: `docs/strategies/markov/markov_regime_models.py` — reference
implementations of 4 models: GaussianHiddenMarkovModel (the "standard Roam"
model), MarkovSwitchingAR, MarkovSwitchingGARCH, TimeVaryingTransitionHMM.

- **Identified** = 4 (TODO.md: standard model + 3 advanced).
- **Implemented** = models wired into a runnable study. Check
  `grep -ri markov src/ configs/` and `ls results/ | grep -i markov`; as of
  2026-11-06 the reference file exists but no study consumes it, so 0.

### 4. Price Action Lab (ES + NQ) — `pal` worktree only
Root: `C:/data/trading/deepthrust-pal`. This is a different kind of source:
a funnel over a huge dial-defined version space, not a list of strategies.

- **Identified (versions)** = planned funnel sets from
  `docs/price_action_lab_five_prompts.md` prompt 3: spread set 40,000 (includes
  the 6 named versions — do not double count) + common set 14,016 + human set
  47,329 = 101,345. Confirm against `rulebook_spec.json` if the spec changed.
- **Identified (behavior tests)** = the distinct claims in prompt 2's
  "The tests" bullet (~15).
- **Implemented/run** = `wc -l results/pal/funnel/versions.jsonl` (each version
  runs on NQ AND ES — count versions, not version×market). Behavior tests run =
  `wc -l results/pal/behavior_verdicts.csv` minus header. Check file mtimes —
  the funnel may still be running; say so if results are partial. Also report
  `len(candidates.json)` and `working_setups.json` for context.
- PAL has no duplicate entries (versions are deduped at draw).

## Dedup rule

A "duplicate" = a G entry tagged `DUPLICATE (S###)` in the GitHub catalog TOC.
Drop it from identified, add 1 to the dropped count, and do not count it as a
new test even if it happens to be implemented (none are, but check). S-catalog
internal `merged_from`/`demoted` entries still count individually — they were
identified as distinct tests. Do NOT drop G entries merely marked `overlap:` —
only the explicit `— DUPLICATE (S###)` TOC tag counts as a drop.

## Output format

A table: source | identified | dropped-as-dup | implemented | notes.
Then two headline numbers:

- **Strategy-level total** (S + unique G + Markov):
  implemented / identified.
- **Grand total including PAL version grid**:
  implemented / identified (add PAL versions-run to numerator, PAL planned
  versions to denominator; behavior tests listed separately).

Note which worktrees contributed implemented counts (main vs g-catalog-tier12
vs pal) since unmerged branch work counts toward "implemented".
