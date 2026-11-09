# decisions.md — Price Action Lab

Spec: `docs/price_action_lab_five_prompts.md`. Every decision logged with its
reason, in order, numbered. No threshold is moved after a result exists; any
change is a numbered amendment written before the next look.

## Decisions

| # | Date | Decision | Reason |
|---|------|----------|--------|
| 1 | 2026-11-03 | Lab lives in git worktree `deepthrust-pal` (branch `pal`) | Spec: work in own worktree; isolates lab from main |
| 2 | 2026-11-03 | Rust engine (`src/pal/`, `src/bin/pal.rs`) + Python one-off tools in `tools/` | User directive; matches repo conventions |
| 3 | 2026-11-03 | Data source = Databento `GLBX.MDP3` `ohlcv-1m`, per-contract pulls, continuous front-month built in-house with volume-flip rolls | Spec's `ts_event` schema = Databento; per-contract pull gives contract column, unadjusted rows, and roll spreads for self-managed back-adjustment |
| 4 | 2026-11-03 | Master seed `PAL_MASTER_SEED` default recorded in `rulebook_spec.json`; xoshiro256** for all draws | Spec: one master seed, exact rebuilds |
| 5 | 2026-11-03 | Results under `results/pal/`; lab root files `decisions.md`, `rulebook.md`, `rulebook_spec.json` | AGENTS.md results convention + spec file names |
| 6 | 2026-11-03 | Seal guard enforced at `pal::store` reader level; unlock = `PAL_STEP=final-test` + `PAL_UNLOCK_SEALED=1`; log to `data/sealed/access_log.jsonl` | Spec: guard on every file reader in the lab |
| 7 | 2026-11-03 | Detection/execution split: cache detector event streams per dial-tuple, engine replays per version | ~100k funnel versions infeasible with per-version detection; also enables cheap no-lookahead proofs |
| 8 | 2026-11-03 | Costs (user-specified): commission $2.00/side ($3.50–$4.50 RT range, midpoint). Slippage 0 on limit fills (touch = 100% fill at limit); 1 tick on market/stop fills; gap-through fills at gap price. ES tick 0.25 = $12.50, NQ tick 0.25 = $5.00 | User directive; spec requires stop & gap slippage for non-limit fills |
| 9 | 2026-11-03 | Windows per spec: warm-up 2021-10-01→12-31; discovery 2022-01-03→2024-07-31 (era1=2022, era2=2023→2024-07); sealed 2024-08-01→2026-10-16 | Spec |
| 10 | 2026-11-03 | Dashboard provenance: `Book` records (file,row,col,format,value,shown) for every number; independent `Verifier` rereads all records before write; page refused on any diff | Spec prompt 5: every number from a saved file, double-read, refuse on mismatch |
| 11 | 2026-11-03 | Dashboard layout gate: `tools/check_dash.py` headless Chrome at 1920×1080 + 1280×720; fail on off-stage elements, facecam-corner (520×368 BR) overlap, or multi-line subtitles; screenshots every page | Spec prompt 5 layout checks |
| 12 | 2026-11-03 | databento Python client pinned to `0.90.0` (dbn 0.73.0) in `requirements-data.txt` — `0.65.x` wheels don't support Python 3.14 | Env reproducibility |
| 13 | 2026-11-09 | `sweep_reclaim` ported into the daily study framework as `pal_sweep_reclaim` (`src/family_pal.rs`, `configs/pal_sweep_reclaim.yaml`): PAL engine generates the trades, framework supplies panel/partition/variants/reports. Baseline = funnel candidate `h:sweep_reclaim:1169` (NQ); partition split 2023-01-01 mirrors PAL era1/era2; `fills: pal` = engine's own fill semantics | User request: PAL outputs lack the framework's partition/skill/variant statistics |
| 14 | 2026-11-09 | Bridge reads discovery window only (`DISCOVERY_START..DISCOVERY_END`, `unlock=None`) — no sealed access; trade-level `ret`/`ret_gross` carry exact PAL economics ($2/side, stop/gap slippage, limit fills); legs stay `weighted:false` so trend/exposure/random-pick accounting stays valid — the daily account series reports mean-trade-return/day (~28 sequential round-trips dilute total_log/CAGR/max_dd ~28x; Sharpe/PF/win-rate/random-pick p stay valid). Weighted mode rejected: leg weights sum into exposure (~28x) and break trend/skill. Minute-level daily P&L stays authoritative in `results/pal/funnel/daily_pnl.jsonl` | Seal rules + honest accounting |
| 15 | 2026-11-09 | `engine::Trade`/`partition::Leg` gained `dir_sign` (+1 long / −1 short) so the daily account series signs short legs correctly — previously every leg was treated as long, which sign-flipped short-side daily stats (found via pal_sweep_reclaim `short` variant: PF 1.65 but Sharpe −10.7). All existing strategies are long-only (`dir_sign: 1.0`) — behavior unchanged, 164 lib tests pass | PAL `direction: both/short` variants need honest daily stats |

## Amendments

| A1 | 2026-11-03 | `splice()` drops contracts with zero in-window bars before splicing | Pre-window lookback contracts (e.g. ESM1) fetch fine but produce empty frames whose NaT `min()` poisoned the running `start_ts` — caught before first real run |

(none — write before looking at results)
