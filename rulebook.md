# rulebook.md — Price Action Lab dials

Master seed: `11400714819323198485`. Every version is a point in this space.

## Setups

- `level_bounce` — entries: limit_at_level, close_back, rejection, engulfing
- `breakout` — entries: at_close, limit_back
- `flip_zone` — entries: limit_at_level, close_back, rejection, engulfing
- `trend_pullback` — entries: limit_at_level, close_back, rejection, engulfing
- `trend_line` — entries: limit_at_level, close_back, rejection, engulfing
- `bos_continuation` — entries: at_close, limit_back
- `choch_reversal` — entries: at_close, limit_back
- `sweep_reclaim` — entries: limit_at_level, close_back, rejection, engulfing
- `supply_demand_zone` — entries: limit_at_level, close_back, rejection, engulfing
- `fair_value_gap` — entries: limit_at_level, close_back, rejection, engulfing

## Charts

Setup charts: 1m, 5m, 15m, 1h, 4h, daily. Trigger chart: same or 1-2 steps lower (daily -> 1h).

## Detector dials

- swing strength: [2, 3, 5, 8]
- level tolerance (ATR): [0.1, 0.25, 0.5]; touches: [1, 2, 3, 4]; shape: line | box
- trend: last two confirmed swing highs + lows
- BOS/CHoCH: first close past last confirmed swing, with/against trend
- zones: runs of 2-5 candles covering 1.5-3 ATR; FVG: 3-candle gap
- trend lines: last [2, 3] anchors

## Execution dials

- stop: beyond level | beyond last swing, +0.1 ATR buffer
- target: 1R | 2R | 3R | next swing
- time exit: 96 trigger bars
- filters: none | trend_agree | ny_only | skip_news (<=2)
- direction: both | long | short; size 1.0 | 0.5

## Costs

Commission $2.00/side; 0 slippage on touched limit fills; 1 tick on market/stop fills; gap-through fills at the gap price.

