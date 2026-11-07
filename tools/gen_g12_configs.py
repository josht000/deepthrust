import io

FACTOR = """data_dir: C:\\data\\trading\\trading_framework\\data\\cache\\daily_10years_tr
data_suffix: .parquet
symbols: "*"
benchmark: SPY
benchmark_data_dir: C:\\data\\trading\\trading_framework\\data\\cache\\etf_tr
benchmark_data_suffix: .parquet
simfin_dir: ..\\trading_framework\\data\\simfin
rank_cache_dir: C:\\data\\trading\\trading_framework\\data\\cache\\liquid_ranks
start_date: "2016-01-01"
min_price: 5.0
min_history: 252
universe_top_n: 300
rebalance: daily
top_fraction: 0.10
factor_invert: false
beta_neutral: true
beta_window: 126
beta_min_obs: 60
borrow_bps_annual: 75.0
cost_bps_per_side: 10.0
fills: next_open,close
partition_split_date: "2024-01-01"
n_draws: 1000
n_boot: 5000
seed: 20261102
book_slots: 60
"""

FV = """  top05: {top_fraction: 0.05}
  top15: {top_fraction: 0.15}
  top20: {top_fraction: 0.20}
  n100: {universe_top_n: 100}
  n500: {universe_top_n: 500}
  n1000: {universe_top_n: 1000}
  invert: {factor_invert: true}
  no_ind: {industry_neutral: false}
  no_beta: {beta_neutral: false}
  weekly: {rebalance: weekly}
  monthly: {rebalance: monthly}
  minp10: {min_price: 10.0}
  minp20: {min_price: 20.0}
  cost5: {cost_bps_per_side: 5.0}
  cost25: {cost_bps_per_side: 25.0}
  bw60: {beta_window: 60}
  bw252: {beta_window: 252}
  lags2: {n_lags: 2}
  lags3: {n_lags: 3}
  lags5: {n_lags: 5}
  top15_n500: {top_fraction: 0.15, universe_top_n: 500}
  top20_n500: {top_fraction: 0.20, universe_top_n: 500}
  weekly_top15: {rebalance: weekly, top_fraction: 0.15}
  noind_top15: {industry_neutral: false, top_fraction: 0.15}
  nobeta_top15: {beta_neutral: false, top_fraction: 0.15}
  top05_n100: {top_fraction: 0.05, universe_top_n: 100}
  n1000_top20: {universe_top_n: 1000, top_fraction: 0.20}
  cost25_top05: {cost_bps_per_side: 25.0, top_fraction: 0.05}
  bw60_top15: {beta_window: 60, top_fraction: 0.15}
  bw252_top15: {beta_window: 252, top_fraction: 0.15}
  lags3_top15: {n_lags: 3, top_fraction: 0.15}
  slots30: {book_slots: 30}
"""

SWING = """data_dir: C:\\data\\trading\\trading_framework\\data\\cache\\daily_10years_tr
data_suffix: .parquet
symbols: "*"
benchmark: SPY
benchmark_data_dir: C:\\data\\trading\\trading_framework\\data\\cache\\etf_tr
benchmark_data_suffix: .parquet
rank_cache_dir: C:\\data\\trading\\trading_framework\\data\\cache\\liquid_ranks
start_date: "2016-01-01"
min_price: 5.0
min_history: 60
universe_top_n: 300
borrow_bps_annual: 75.0
cost_bps_per_side: 10.0
fills: next_open,close
partition_split_date: "2024-01-01"
n_draws: 1000
n_boot: 5000
seed: 20261102
book_slots: 10
max_units: 1
"""

SV = """  top500: {universe_top_n: 500}
  top1000: {universe_top_n: 1000}
  minp10: {min_price: 10.0}
  minp20: {min_price: 20.0}
  cost5: {cost_bps_per_side: 5.0}
  cost2: {cost_bps_per_side: 2.0}
  cost25: {cost_bps_per_side: 25.0}
  hist126: {min_history: 126}
  hist252: {min_history: 252}
"""

def w(name, header, body, variants):
    p = 'configs/%s.yaml' % name
    io.open(p, 'w', encoding='utf-8').write(header + body + "\nvariants:\n" + variants)
    print('wrote', p)

w('g021_voladj_rev',
"""# G021 volume-adjusted daily return reversal: score = -(relvol21 x
# beta-adjusted return), lag-averaged; daily decile L/S.
# DELTA: industry demean via simfin Industry (not Barra ind1).
""", FACTOR + "lookback: 21\nn_lags: 1\nindustry_neutral: true\n", FV)

w('g104_lagged_indmom',
"""# G104 232-session-lagged 20-session cumulative return, industry-demeaned;
# daily decile L/S. Source sign ambiguity -> momentum sign baseline.
""", FACTOR + "lookback: 20\nskip_days: 232\nindustry_neutral: true\n", FV +
"""  lag126: {skip_days: 126}
  lag189: {skip_days: 189}
  lag252: {skip_days: 252}
  win40: {lookback: 40}
  win60: {lookback: 60}
""")

w('g105_beta_div_rev',
"""# G105 beta-divided multi-period reversal: score = -(ret/beta) averaged
# over n_lags; daily decile L/S.
""", FACTOR + "lookback: 5\nn_lags: 1\nindustry_neutral: true\n", FV +
"""  lb1: {lookback: 1}
  lb10: {lookback: 10}
  lb21: {lookback: 21}
""")

w('g146_weekday_beta',
"""# G146 weekday-conditioned beta-adjusted return forecast: per-weekday mean
# of beta-adjusted returns over lookback -> decile L/S.
""", FACTOR + "lookback: 252\nindustry_neutral: true\n", FV +
"""  lb60: {lookback: 60}
  lb126: {lookback: 126}
  lb504: {lookback: 504}
""")

for name, extra, note in [
    ('g119_alpha009', '', 'G119 Alpha101-009 conditional reversal as decile score.'),
    ('g120_alpha012', '', 'G120 Alpha101-012 sign(dvol)*-dclose as decile score.'),
    ('g121_candle_loc', '', 'G121 Alpha101-101 (c-o)/(h-l) as decile score.'),
    ('g123_alpha191_020', 'skip_days: 6\n', 'G123 Alpha191-020 6-session pct change x100 as decile score.'),
]:
    w(name, '# %s\n# DELTA: CN A-share alpha applied to US top-300 liquid universe.\n' % note,
      FACTOR + "industry_neutral: false\n" + extra, FV)

w('g127_bb_w',
"""# G127 Bollinger(20,2) bottom-W: two lower-band tests + midpoint recovery +
# higher second low, enter long on upper-band cross; exit on bandwidth
# collapse (rel sd < weakness_threshold x window max).
""", SWING + "bb_len: 20\nbb_mult: 2.0\nlookback: 75\nbreakout_distance_atr: 0.0001\nweakness_threshold: 0.5\n", SV +
"""  bb15: {bb_len: 15}
  bb30: {bb_len: 30}
  bb40: {bb_len: 40}
  mult15: {bb_mult: 1.5}
  mult25: {bb_mult: 2.5}
  pat50: {lookback: 50}
  pat100: {lookback: 100}
  pat150: {lookback: 150}
  weak03: {weakness_threshold: 0.3}
  weak07: {weakness_threshold: 0.7}
  weak09: {weakness_threshold: 0.9}
  tol001: {breakout_distance_atr: 0.001}
  tol01: {breakout_distance_atr: 0.01}
  bb15_pat50: {bb_len: 15, lookback: 50}
  bb30_pat100: {bb_len: 30, lookback: 100}
  mult25_weak07: {bb_mult: 2.5, weakness_threshold: 0.7}
  pat100_top500: {lookback: 100, universe_top_n: 500}
  weak03_cost5: {weakness_threshold: 0.3, cost_bps_per_side: 5.0}
  bb30_weak07: {bb_len: 30, weakness_threshold: 0.7}
  mult15_pat150: {bb_mult: 1.5, lookback: 150}
  tol01_pat100: {breakout_distance_atr: 0.01, lookback: 100}
  maxu2: {max_units: 2}
""")

w('g128_ha_reversal',
"""# G128 Heikin-Ashi consecutive-candle reversal: long on bearish HA with
# open==high and body>prior body; exit on bullish HA open==low + prior bull.
""", SWING, SV +
"""  maxu2: {max_units: 2}
  maxu3: {max_units: 3}
  stop3: {disaster_stop_atr: 3.0}
  stop5: {disaster_stop_atr: 5.0}
""")

w('g129_rsi_hs',
"""# G129 RSI(14) head-and-shoulders short: three peaks, head>shoulders within
# tolerance; exit after RSI +4pts from entry or 5 sessions.
""", SWING + "rsi_len: 14\nrsi_threshold: 5.0\nadd_rsi: 4.0\nmax_hold: 5\n", SV +
"""  rsi10: {rsi_len: 10}
  rsi21: {rsi_len: 21}
  rsi7: {rsi_len: 7}
  tol3: {rsi_threshold: 3.0}
  tol8: {rsi_threshold: 8.0}
  tol10: {rsi_threshold: 10.0}
  exit2: {add_rsi: 2.0}
  exit6: {add_rsi: 6.0}
  exit8: {add_rsi: 8.0}
  hold3: {max_hold: 3}
  hold10: {max_hold: 10}
  hold15: {max_hold: 15}
  rsi10_tol8: {rsi_len: 10, rsi_threshold: 8.0}
  rsi21_exit6: {rsi_len: 21, add_rsi: 6.0}
  tol8_hold10: {rsi_threshold: 8.0, max_hold: 10}
  exit6_hold3: {add_rsi: 6.0, max_hold: 3}
  tol3_rsi10: {rsi_threshold: 3.0, rsi_len: 10}
  tol10_rsi21: {rsi_threshold: 10.0, rsi_len: 21}
  hold10_exit6: {max_hold: 10, add_rsi: 6.0}
  rsi7_hold3: {rsi_len: 7, max_hold: 3}
""")

w('g130_shooting_star',
"""# G130 shooting-star short: upwick>=2x body, lower wick<=20% range, body<=50%
# range; exit |move|>5% or 7 sessions. rsi_threshold/100 = body cap,
# weakness_threshold = lower-wick cap.
""", SWING + "rsi_threshold: 50.0\nweakness_threshold: 0.2\nstop_loss: 0.05\nmax_hold: 7\n", SV +
"""  body30: {rsi_threshold: 30.0}
  body70: {rsi_threshold: 70.0}
  wick10: {weakness_threshold: 0.10}
  wick30: {weakness_threshold: 0.30}
  stop3: {stop_loss: 0.03}
  stop8: {stop_loss: 0.08}
  stop10: {stop_loss: 0.10}
  hold5: {max_hold: 5}
  hold10: {max_hold: 10}
  hold14: {max_hold: 14}
  stop3_hold5: {stop_loss: 0.03, max_hold: 5}
  stop8_hold10: {stop_loss: 0.08, max_hold: 10}
  body30_wick10: {rsi_threshold: 30.0, weakness_threshold: 0.10}
  body70_wick30: {rsi_threshold: 70.0, weakness_threshold: 0.30}
  stop10_hold14: {stop_loss: 0.10, max_hold: 14}
  wick15_stop8: {weakness_threshold: 0.15, stop_loss: 0.08}
  body40: {rsi_threshold: 40.0}
  body60: {rsi_threshold: 60.0}
  wick25: {weakness_threshold: 0.25}
""")

w('g202_ternary',
"""# G202 ternary event engine: MACD(5,34) zero-cross + BB(20) breach + CMF(20)
# sign-change events; long on +event, exit on opposite (vote mode = summed).
""", SWING + "macd_fast: 5\nmacd_slow: 34\nbb_len: 20\nbb_mult: 2.0\nvolume_window: 20\nentry_mode: vote\n", SV +
"""  macd: {entry_mode: macd}
  bb: {entry_mode: bb}
  cmf: {entry_mode: cmf}
  all: {entry_mode: all}
  macd812: {macd_fast: 8, macd_slow: 12}
  macd1226: {macd_fast: 12, macd_slow: 26}
  bb15: {bb_len: 15}
  bb30: {bb_len: 30}
  bb10: {bb_len: 10}
  mult15: {bb_mult: 1.5}
  mult25: {bb_mult: 2.5}
  cmf10: {volume_window: 10}
  cmf30: {volume_window: 30}
  macd_macd812: {entry_mode: macd, macd_fast: 8, macd_slow: 12}
  bb_bb15: {entry_mode: bb, bb_len: 15}
  bb_mult25: {entry_mode: bb, bb_mult: 2.5}
  cmf_cmf10: {entry_mode: cmf, volume_window: 10}
  macd_cmf10: {entry_mode: macd, volume_window: 10}
  vote_bb15: {entry_mode: vote, bb_len: 15}
  all_mult25: {entry_mode: all, bb_mult: 2.5}
""")

w('g099_olps',
"""# G099 online portfolio selection zoo on 30 mega-caps; daily weight update.
# strategy_mode: olmar|pamr|corn|eg|up|crp. lookback=predictor window,
# resp_scale=step lambda, corr_min=CORN rho.
""",
"""data_dir: C:\\data\\trading\\trading_framework\\data\\cache\\daily_10years_tr
data_suffix: .parquet
symbols: AAPL,MSFT,GOOGL,AMZN,META,NVDA,TSLA,AVGO,AMD,ORCL,JPM,JNJ,PG,XOM,HD,KO,PEP,MCD,V,MA,WMT,DIS,GS,MRK,PFE,BA,CAT,IBM,GE,CVX
benchmark: SPY
benchmark_data_dir: C:\\data\\trading\\trading_framework\\data\\cache\\etf_tr
benchmark_data_suffix: .parquet
start_date: "2016-01-01"
strategy_mode: olmar
lookback: 5
resp_scale: 10.0
corr_min: 0.3
rebalance: daily
borrow_bps_annual: 75.0
cost_bps_per_side: 10.0
fills: next_open,close
partition_split_date: "2024-01-01"
n_draws: 1000
n_boot: 5000
seed: 20261102
book_slots: 30
""",
"""  pamr: {strategy_mode: pamr}
  corn: {strategy_mode: corn}
  eg: {strategy_mode: eg}
  up: {strategy_mode: up}
  crp: {strategy_mode: crp}
  lb3: {lookback: 3}
  lb10: {lookback: 10}
  lb21: {lookback: 21}
  lb60: {lookback: 60}
  resp1: {resp_scale: 1.0}
  resp5: {resp_scale: 5.0}
  resp20: {resp_scale: 20.0}
  resp50: {resp_scale: 50.0}
  rho01: {strategy_mode: corn, corr_min: 0.1}
  rho05: {strategy_mode: corn, corr_min: 0.5}
  rho07: {strategy_mode: corn, corr_min: 0.7}
  weekly: {rebalance: weekly}
  monthly: {rebalance: monthly}
  cost5: {cost_bps_per_side: 5.0}
  cost25: {cost_bps_per_side: 25.0}
  olmar_lb10: {lookback: 10}
  olmar_lb21: {lookback: 21}
  olmar_resp5: {resp_scale: 5.0}
  olmar_resp20: {resp_scale: 20.0}
  pamr_lb10: {strategy_mode: pamr, lookback: 10}
  corn_lb10: {strategy_mode: corn, lookback: 10}
  corn_lb21: {strategy_mode: corn, lookback: 21}
  eg_resp1: {strategy_mode: eg, resp_scale: 1.0}
  eg_resp20: {strategy_mode: eg, resp_scale: 20.0}
  weekly_olmar: {rebalance: weekly}
  monthly_pamr: {strategy_mode: pamr, rebalance: monthly}
  corn_rho05_lb10: {strategy_mode: corn, corr_min: 0.5, lookback: 10}
""")
