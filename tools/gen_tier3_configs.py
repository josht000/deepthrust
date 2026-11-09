import os

ETF = r"C:\data\trading\trading_framework\data\cache\etf_tr"
STK = r"C:\data\trading\trading_framework\data\cache\daily_10years_tr"
CRY = r"C:\data\trading\trading_framework\data\cache\crypto"
RANK = r"C:\data\trading\trading_framework\data\cache\liquid_ranks"
UNI = "SPY,EFA,EEM,TLT,IEF,UUP,GLD,SLV,CPER,USO,GSG,DBC"

def head(comment, symbols=UNI, data_dir=ETF, extra="", start="2005-01-01"):
    return f"""# {comment}
data_dir: {data_dir}
data_suffix: .parquet
symbols: {symbols}
benchmark: SPY
benchmark_data_dir: {ETF}
benchmark_data_suffix: .parquet
start_date: "{start}"
{extra}partition_split_date: "2024-01-01"
n_draws: 1000
n_boot: 5000
seed: 20261101
fills: next_open,close
"""

def variants(entries):
    lines = ["\nvariants:"]
    for name, ov in entries:
        body = ", ".join(f"{k}: {v}" for k, v in ov.items())
        lines.append(f"  {name}: {{{body}}}")
    return "\n".join(lines) + "\n"

cfgs = {}

# ---------- G077 CTA 1.0 ----------
v = []
for f, s in [(4,12),(8,24),(12,36),(16,48),(20,60),(24,72),(32,96),(32,128),(40,120),(48,144),(64,192),(96,288)]:
    v.append((f"f{f}s{s}", {"ewma_fast": f'"{f}"', "ewma_slow": f'"{s}"'}))
for cs in [0.01,0.025,0.075,0.10,0.15,0.20]:
    v.append((f"cs{int(cs*1000)}", {"cash_scale": cs}))
for mh in [50,150,200,300]:
    v.append((f"warm{mh}", {"min_history": mh}))
v += [("weekly", {"rebalance": "weekly"}), ("monthly", {"rebalance": "monthly"}),
      ("f16s48_cs10", {"ewma_fast": '"16"', "ewma_slow": '"48"', "cash_scale": 0.10}),
      ("f20s60_cs10", {"ewma_fast": '"20"', "ewma_slow": '"60"', "cash_scale": 0.10}),
      ("f64s192_cs10", {"ewma_fast": '"64"', "ewma_slow": '"192"', "cash_scale": 0.10}),
      ("f16s48_weekly", {"ewma_fast": '"16"', "ewma_slow": '"48"', "rebalance": "weekly"}),
      ("f64s192_weekly", {"ewma_fast": '"64"', "ewma_slow": '"192"', "rebalance": "weekly"}),
      ("f8s24_cs15", {"ewma_fast": '"8"', "ewma_slow": '"24"', "cash_scale": 0.15}),
      ("f48s144_cs025", {"ewma_fast": '"48"', "ewma_slow": '"144"', "cash_scale": 0.025}),
      ("f32s128_cs10", {"ewma_fast": '"32"', "ewma_slow": '"128"', "cash_scale": 0.10}),
      ("f40s120_warm200", {"ewma_fast": '"40"', "ewma_slow": '"120"', "min_history": 200}),
      ("f16s48_warm150", {"ewma_fast": '"16"', "ewma_slow": '"48"', "min_history": 150}),
      ("f24s72_cs075", {"ewma_fast": '"24"', "ewma_slow": '"72"', "cash_scale": 0.075})]
cfgs["g077_cta_binary"] = head(
    "G077 CTA 1.0: sign(EWM com32 - EWM com96, min 100) x 5e6 cash / 1e8 AUM daily.\n# Futures legs proxied by TR ETFs (DELTA; same sleeve as g067).",
    extra="""min_history: 100
ewma_fast: "32"
ewma_slow: "96"
cash_scale: 0.05
rebalance: daily
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 10.0
book_slots: 12
""") + variants(v)

# ---------- G078 CTA 2.0 ----------
v = []
for f, s in [(8,24),(16,48),(20,60),(24,72),(32,96),(32,128),(48,144),(64,192)]:
    v.append((f"f{f}s{s}", {"ewma_fast": f'"{f}"', "ewma_slow": f'"{s}"'}))
for vw in [16,32,48,64,96,128]:
    v.append((f"vol{vw}", {"vol_window": vw}))
for cs in [0.0005,0.002,0.003,0.005]:
    v.append((f"cs{int(cs*1e6)}", {"cash_scale": cs}))
v += [("weekly", {"rebalance": "weekly"}), ("monthly", {"rebalance": "monthly"}),
      ("warm200", {"min_history": 200}), ("warm400", {"min_history": 400}),
      ("f16s48_vol64", {"ewma_fast": '"16"', "ewma_slow": '"48"', "vol_window": 64}),
      ("f64s192_vol64", {"ewma_fast": '"64"', "ewma_slow": '"192"', "vol_window": 64}),
      ("f20s60_cs002", {"ewma_fast": '"20"', "ewma_slow": '"60"', "cash_scale": 0.002}),
      ("f48s144_cs002", {"ewma_fast": '"48"', "ewma_slow": '"144"', "cash_scale": 0.002}),
      ("f32s96_weekly", {"rebalance": "weekly"}),
      ("vol64_cs002", {"vol_window": 64, "cash_scale": 0.002}),
      ("vol16_cs002", {"vol_window": 16, "cash_scale": 0.002}),
      ("vol96_cs002", {"vol_window": 96, "cash_scale": 0.002}),
      ("f16s48_cs002", {"ewma_fast": '"16"', "ewma_slow": '"48"', "cash_scale": 0.002}),
      ("f64s192_cs002", {"ewma_fast": '"64"', "ewma_slow": '"192"', "cash_scale": 0.002}),
      ("f24s72_vol48", {"ewma_fast": '"24"', "ewma_slow": '"72"', "vol_window": 48}),
      ("f40s120_vol48", {"ewma_fast": '"40"', "ewma_slow": '"120"', "vol_window": 48})]
cfgs["g078_cta_volscaled"] = head(
    "G078 CTA 2.0: sign(EWM32-96)/EWM-std(pct_chg com32) x 1e5 cash / 1e8 AUM,\n# 300-obs warmup. Futures legs proxied by TR ETFs (DELTA).",
    extra="""min_history: 300
ewma_fast: "32"
ewma_slow: "96"
vol_window: 32
cash_scale: 0.001
rebalance: daily
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 10.0
book_slots: 12
""") + variants(v)

# ---------- G079 CTA 3.0 ----------
v = []
for clip in [2.0,3.0,4.2,5.0,6.0]:
    v.append((f"clip{str(clip).replace('.','')}", {"clip": clip}))
for vw in [16,32,64,96]:
    v.append((f"vol{vw}", {"vol_window": vw}))
for f, s in [(8,24),(16,48),(20,60),(32,96),(32,128),(48,144),(64,192)]:
    v.append((f"f{f}s{s}", {"ewma_fast": f'"{f}"', "ewma_slow": f'"{s}"'}))
for cs in [0.0005,0.002,0.003]:
    v.append((f"cs{int(cs*1e6)}", {"cash_scale": cs}))
v += [("weekly", {"rebalance": "weekly"}), ("monthly", {"rebalance": "monthly"}),
      ("clip42_f16s48", {"clip": 4.2, "ewma_fast": '"16"', "ewma_slow": '"48"'}),
      ("clip42_f64s192", {"clip": 4.2, "ewma_fast": '"64"', "ewma_slow": '"192"'}),
      ("clip42_vol64", {"clip": 4.2, "vol_window": 64}),
      ("clip42_cs002", {"clip": 4.2, "cash_scale": 0.002}),
      ("clip3_vol64", {"clip": 3.0, "vol_window": 64}),
      ("clip5_vol64", {"clip": 5.0, "vol_window": 64}),
      ("clip3_cs002", {"clip": 3.0, "cash_scale": 0.002}),
      ("clip42_weekly", {"clip": 4.2, "rebalance": "weekly"}),
      ("clip42_f20s60", {"clip": 4.2, "ewma_fast": '"20"', "ewma_slow": '"60"'}),
      ("clip42_f32s128", {"clip": 4.2, "ewma_fast": '"32"', "ewma_slow": '"128"'}),
      ("warm200", {"min_history": 200}), ("warm400", {"min_history": 400})]
cfgs["g079_cta_robust"] = head(
    "G079 CTA 3.0: tanh(osc(cumsum(vol_adj(vola32,clip3))))/EWM-std(com96)\n# x 1e5/1e8 daily. Catalog clip=3 (fn default; notebook slider is 4.2).\n# Futures legs proxied by TR ETFs (DELTA).",
    extra="""min_history: 300
ewma_fast: "32"
ewma_slow: "96"
vol_window: 32
clip: 3.0
cash_scale: 0.001
rebalance: daily
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 10.0
book_slots: 12
""") + variants(v)

# ---------- G080 CTA 4.0 ----------
v = []
for clip in [2.0,3.0,4.2,5.0]:
    v.append((f"clip{str(clip).replace('.','')}", {"clip": clip}))
for vw in [16,32,64,96]:
    v.append((f"vol{vw}", {"vol_window": vw}))
for f, s in [(8,24),(16,48),(20,60),(32,96),(32,128),(48,144),(64,192)]:
    v.append((f"f{f}s{s}", {"ewma_fast": f'"{f}"', "ewma_slow": f'"{s}"'}))
for cs in [0.0025,0.005,0.01,0.02]:
    v.append((f"cs{int(cs*1e4)}", {"cash_scale": cs}))
v += [("weekly", {"rebalance": "weekly"}), ("monthly", {"rebalance": "monthly"}),
      ("clip3_f16s48", {"clip": 3.0, "ewma_fast": '"16"', "ewma_slow": '"48"'}),
      ("clip3_f64s192", {"clip": 3.0, "ewma_fast": '"64"', "ewma_slow": '"192"'}),
      ("clip3_vol64", {"clip": 3.0, "vol_window": 64}),
      ("clip3_cs01", {"clip": 3.0, "cash_scale": 0.01}),
      ("clip5_vol64", {"clip": 5.0, "vol_window": 64}),
      ("clip42_cs01", {"clip": 4.2, "cash_scale": 0.01}),
      ("clip42_weekly", {"clip": 4.2, "rebalance": "weekly"}),
      ("clip42_f20s60", {"clip": 4.2, "ewma_fast": '"20"', "ewma_slow": '"60"'}),
      ("clip5_cs01", {"clip": 5.0, "cash_scale": 0.01}),
      ("clip3_weekly", {"clip": 3.0, "rebalance": "weekly"}),
      ("warm200", {"min_history": 200}), ("warm400", {"min_history": 400})]
cfgs["g080_cta_riskparity"] = head(
    "G080 CTA 4.0: tanh(osc(vol_adj-cumsum 32/4.2)) mu; risk_scaled = mu/||mu||;\n# w = 5e5*risk_scaled/ewmstd32 per asset on 1e8 AUM. ETF proxies (DELTA).",
    extra="""min_history: 300
ewma_fast: "32"
ewma_slow: "96"
vol_window: 32
clip: 4.2
cash_scale: 0.005
rebalance: daily
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 10.0
book_slots: 12
""") + variants(v)

# ---------- G081 CTA 5.0 ----------
v = []
for cw in [100,150,200,250,300,400]:
    v.append((f"corr{cw}", {"corr_window": cw}))
for sh in [0.0,0.25,0.5,0.75,1.0]:
    v.append((f"sh{int(sh*100)}", {"shrinkage": sh}))
for clip in [3.0,4.2,5.0]:
    v.append((f"clip{str(clip).replace('.','')}", {"clip": clip}))
for cs in [0.005,0.01,0.02]:
    v.append((f"cs{int(cs*1e4)}", {"cash_scale": cs}))
for f, s in [(16,48),(20,60),(32,96),(32,128),(48,144),(64,192)]:
    v.append((f"f{f}s{s}", {"ewma_fast": f'"{f}"', "ewma_slow": f'"{s}"'}))
v += [("weekly", {"rebalance": "weekly"}),
      ("corr100_sh25", {"corr_window": 100, "shrinkage": 0.25}),
      ("corr300_sh75", {"corr_window": 300, "shrinkage": 0.75}),
      ("corr150_sh50", {"corr_window": 150, "shrinkage": 0.5}),
      ("corr200_sh0", {"corr_window": 200, "shrinkage": 0.0}),
      ("corr200_sh100", {"corr_window": 200, "shrinkage": 1.0}),
      ("corr100_cs02", {"corr_window": 100, "cash_scale": 0.02}),
      ("corr200_f16s48", {"corr_window": 200, "ewma_fast": '"16"', "ewma_slow": '"48"'}),
      ("corr200_f64s192", {"corr_window": 200, "ewma_fast": '"64"', "ewma_slow": '"192"'}),
      ("corr150_clip3", {"corr_window": 150, "clip": 3.0}),
      ("corr250_sh25", {"corr_window": 250, "shrinkage": 0.25}),
      ("corr100_weekly", {"corr_window": 100, "rebalance": "weekly"}),
      ("warm200", {"min_history": 200}), ("warm400", {"min_history": 400})]
cfgs["g081_cta_dcc"] = head(
    "G081 CTA 5.0: CTA 3.0 mu + EWM(com200) corr of vol_adj returns shrunk 0.5\n# toward I; risk_pos = C^-1 mu / sqrt(mu'C^-1 mu); w = 1e6*risk_pos/vo.\n# ETF proxies (DELTA).",
    extra="""min_history: 300
ewma_fast: "32"
ewma_slow: "96"
vol_window: 32
clip: 4.2
corr_window: 200
shrinkage: 0.5
cash_scale: 0.01
rebalance: daily
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 10.0
book_slots: 12
""") + variants(v)

# ---------- G085 ----------
v = []
for lb in [20,40,60,63,80,100,126,189,252,315]:
    v.append((f"lb{lb}", {"lookback": lb}))
for tv in [0.15,0.25,0.40,0.60,0.80]:
    v.append((f"tv{int(tv*100)}", {"target_vol": tv}))
for cap in [0.5,0.75,0.95,1.5,2.0]:
    v.append((f"cap{int(cap*100)}", {"pos_cap": cap}))
for vw in [20,40,60,120]:
    v.append((f"vol{vw}", {"vol_window": vw}))
v += [("weekly", {"rebalance": "weekly"}), ("dom1", {"rebalance": "dom1"}), ("dom15", {"rebalance": "dom15"}),
      ("lb40_tv25", {"lookback": 40, "target_vol": 0.25}),
      ("lb126_tv25", {"lookback": 126, "target_vol": 0.25}),
      ("lb252_tv25", {"lookback": 252, "target_vol": 0.25}),
      ("lb126_tv60", {"lookback": 126, "target_vol": 0.60}),
      ("lb63_weekly", {"lookback": 63, "rebalance": "weekly"}),
      ("lb126_weekly", {"lookback": 126, "rebalance": "weekly"}),
      ("lb252_weekly", {"lookback": 252, "rebalance": "weekly"}),
      ("lb40_cap50", {"lookback": 40, "pos_cap": 0.5}),
      ("lb126_cap50", {"lookback": 126, "pos_cap": 0.5}),
      ("lb63_vol20", {"lookback": 63, "vol_window": 20}),
      ("lb126_vol120", {"lookback": 126, "vol_window": 120})]
cfgs["g085_local_extreme"] = head(
    "G085 local-extreme breakout x EWM(com60) vol scaling, target 40% cap 0.95,\n# month-end. Private vivace signal -> DELTA: Donchian high/low sign, lookback 63.",
    extra="""min_history: 100
lookback: 63
vol_window: 60
target_vol: 0.40
pos_cap: 0.95
rebalance: monthly
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 10.0
book_slots: 12
""") + variants(v)

# ---------- G095 SSA ----------
v = []
for l in [15,20,30,40,50]:
    v.append((f"eig{l}", {"latent_dim": l}))
for m in [60,90,120,180,252]:
    v.append((f"m{m}", {"lookback": m}))
for step in [10,21,42,63]:
    v.append((f"step{step}", {"rebalance_step": step}))
for tv in [0.05,0.10,0.15,0.20]:
    v.append((f"tv{int(tv*100)}", {"target_vol": tv}))
v += [("ew", {"weighting_mode": "EW"}),
      ("overq", {"strategy_mode": "overQuantity"}),
      ("indq", {"strategy_mode": "indQuantity"}),
      ("overq_thr30", {"strategy_mode": "overQuantity", "trend_threshold": 0.30}),
      ("indq_thr30", {"strategy_mode": "indQuantity", "trend_threshold": 0.30}),
      ("overq_thr70", {"strategy_mode": "overQuantity", "trend_threshold": 0.70}),
      ("indq_thr70", {"strategy_mode": "indQuantity", "trend_threshold": 0.70}),
      ("nolev", {"target_vol": 0.0}),
      ("m60_eig20", {"lookback": 60, "latent_dim": 20}),
      ("m120_eig40", {"lookback": 120, "latent_dim": 40}),
      ("m180_eig30", {"lookback": 180, "latent_dim": 30}),
      ("m120_step21", {"lookback": 120, "rebalance_step": 21}),
      ("ew_m120", {"weighting_mode": "EW", "lookback": 120}),
      ("ew_step42", {"weighting_mode": "EW", "rebalance_step": 42}),
      ("indq_ew", {"strategy_mode": "indQuantity", "weighting_mode": "EW"}),
      ("overq_ew", {"strategy_mode": "overQuantity", "weighting_mode": "EW"}),
      ("eig30_tv15", {"target_vol": 0.15}),
      ("m252_step21", {"lookback": 252, "rebalance_step": 21})]
cfgs["g095_ssa_trend"] = head(
    "G095 SSA first-PC trend: LatentDim 30, window M=90, 21-day allocation,\n# vol-parity weights x signal rescaled [-1,1], 10% vol-target leverage (QARMII).",
    extra="""min_history: 120
lookback: 90
latent_dim: 30
rebalance_step: 21
weighting_mode: volparity
strategy_mode: ""
trend_threshold: 0.5
target_vol: 0.10
rebalance: monthly
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 10.0
book_slots: 12
""") + variants(v)

# ---------- G106 ----------
v = []
for mw in [63,126,189,252,315,378]:
    v.append((f"mom{mw}", {"mom_window": mw}))
for nl in [1,2,3]:
    v.append((f"legs{nl}", {"n_legs": nl}))
v += [("weekly", {"rebalance": "weekly"}), ("dom1", {"rebalance": "dom1"}), ("dom15", {"rebalance": "dom15"}),
      ("mom126_legs1", {"mom_window": 126, "n_legs": 1}),
      ("mom126_legs3", {"mom_window": 126, "n_legs": 3}),
      ("mom252_legs1", {"mom_window": 252, "n_legs": 1}),
      ("mom252_legs3", {"mom_window": 252, "n_legs": 3}),
      ("mom189_legs1", {"mom_window": 189, "n_legs": 1}),
      ("mom189_legs3", {"mom_window": 189, "n_legs": 3}),
      ("mom63_legs1", {"mom_window": 63, "n_legs": 1}),
      ("mom63_legs3", {"mom_window": 63, "n_legs": 3}),
      ("mom315_legs1", {"mom_window": 315, "n_legs": 1}),
      ("mom315_legs3", {"mom_window": 315, "n_legs": 3}),
      ("mom126_weekly", {"mom_window": 126, "rebalance": "weekly"}),
      ("mom252_weekly", {"mom_window": 252, "rebalance": "weekly"}),
      ("mom189_weekly", {"mom_window": 189, "rebalance": "weekly"}),
      ("mom63_weekly", {"mom_window": 63, "rebalance": "weekly"}),
      ("mom378_legs1", {"mom_window": 378, "n_legs": 1}),
      ("mom378_legs3", {"mom_window": 378, "n_legs": 3}),
      ("legs1_weekly", {"n_legs": 1, "rebalance": "weekly"}),
      ("legs3_weekly", {"n_legs": 3, "rebalance": "weekly"}),
      ("mom126_dom1", {"mom_window": 126, "rebalance": "dom1"}),
      ("mom252_dom1", {"mom_window": 252, "rebalance": "dom1"}),
      ("mom189_dom15", {"mom_window": 189, "rebalance": "dom15"}),
      ("mom189_dom1", {"mom_window": 189, "rebalance": "dom1"}),
      ("mom126_dom15", {"mom_window": 126, "rebalance": "dom15"})]
cfgs["g106_xs_com_mom"] = head(
    "G106 XS 252-session commodity momentum, monthly top/bottom-2 L/S.\n# Hollstein futures universe proxied by commodity ETFs only (DELTA).",
    symbols="GLD,SLV,CPER,USO,GSG,DBC",
    extra="""min_history: 252
mom_window: 252
n_legs: 2
rebalance: monthly
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 10.0
book_slots: 6
""") + variants(v)

# ---------- G109 ----------
v = []
for mw in [63,126,189,252,315,378]:
    v.append((f"sk{mw}", {"mom_window": mw}))
for nl in [1,2,3]:
    v.append((f"legs{nl}", {"n_legs": nl}))
v += [("weekly", {"rebalance": "weekly"}), ("dom1", {"rebalance": "dom1"}), ("dom15", {"rebalance": "dom15"}),
      ("sk126_legs1", {"mom_window": 126, "n_legs": 1}),
      ("sk126_legs3", {"mom_window": 126, "n_legs": 3}),
      ("sk252_legs1", {"mom_window": 252, "n_legs": 1}),
      ("sk252_legs3", {"mom_window": 252, "n_legs": 3}),
      ("sk189_legs1", {"mom_window": 189, "n_legs": 1}),
      ("sk189_legs3", {"mom_window": 189, "n_legs": 3}),
      ("sk63_legs1", {"mom_window": 63, "n_legs": 1}),
      ("sk63_legs3", {"mom_window": 63, "n_legs": 3}),
      ("sk126_weekly", {"mom_window": 126, "rebalance": "weekly"}),
      ("sk252_weekly", {"mom_window": 252, "rebalance": "weekly"}),
      ("sk189_weekly", {"mom_window": 189, "rebalance": "weekly"}),
      ("sk63_weekly", {"mom_window": 63, "rebalance": "weekly"}),
      ("sk315_legs1", {"mom_window": 315, "n_legs": 1}),
      ("sk315_legs3", {"mom_window": 315, "n_legs": 3}),
      ("legs1_weekly", {"n_legs": 1, "rebalance": "weekly"}),
      ("legs3_weekly", {"n_legs": 3, "rebalance": "weekly"}),
      ("sk126_dom1", {"mom_window": 126, "rebalance": "dom1"}),
      ("sk252_dom1", {"mom_window": 252, "rebalance": "dom1"}),
      ("sk189_dom15", {"mom_window": 189, "rebalance": "dom15"}),
      ("sk189_dom1", {"mom_window": 189, "rebalance": "dom1"}),
      ("sk126_dom15", {"mom_window": 126, "rebalance": "dom15"})]
cfgs["g109_xs_skew"] = head(
    "G109 XS skewness risk premium: rank on NEGATED 252d return skewness,\n# monthly top/bottom-2 L/S (commodity notebook Negate orientation, DELTA sleeve).",
    symbols="GLD,SLV,CPER,USO,GSG,DBC",
    extra="""min_history: 252
mom_window: 252
n_legs: 2
rebalance: monthly
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 10.0
book_slots: 6
""") + variants(v)

# ---------- G063 ----------
v = []
for ze in [1.0,1.5,2.0,2.5,3.0]:
    v.append((f"z{str(ze).replace('.','')}", {"z_entry": ze}))
for ts in [2.0,3.0,4.0,6.0,8.0]:
    v.append((f"t{int(ts)}", {"min_t_stat": ts}))
for up in [4.0,6.0,8.0,12.0]:
    v.append((f"up{int(up)}", {"take_profit": up}))
for dn in [2.0,4.0,6.0,8.0]:
    v.append((f"dn{int(dn)}", {"stop_loss": dn}))
for zw in [40,60,120,180]:
    v.append((f"zw{zw}", {"z_window": zw}))
for sw in [0.5,1.0,1.5,2.0,2.5]:
    v.append((f"w{str(sw).replace('.','')}", {"spread_weight": sw}))
for mh in [21,42,63,126]:
    v.append((f"hold{mh}", {"max_hold": mh}))
v += [("units1", {"max_units": 1}), ("units8", {"max_units": 8}),
      ("z15_t3", {"z_entry": 1.5, "min_t_stat": 3.0}),
      ("z2_t3", {"z_entry": 2.0, "min_t_stat": 3.0}),
      ("z25_t6", {"z_entry": 2.5, "min_t_stat": 6.0}),
      ("z15_up6", {"z_entry": 1.5, "take_profit": 6.0}),
      ("z2_dn6", {"z_entry": 2.0, "stop_loss": 6.0}),
      ("z15_hold42", {"z_entry": 1.5, "max_hold": 42}),
      ("zw40_z15", {"z_window": 40, "z_entry": 1.5}),
      ("w15_z15", {"spread_weight": 1.5, "z_entry": 1.5}),
      ("w2_z15", {"spread_weight": 2.0, "z_entry": 1.5})]
cfgs["g063_tsy_spread"] = head(
    "G063 Treasury z/t-score RV entry: futures basket proxied by ln(TLT)-w*ln(IEF)\n# spread (DELTA); tick = mean|ds| over z_window; tranche exits on tick bands;\n# roll date proxied by max_hold.",
    symbols="TLT,IEF",
    extra="""pairs: "TLT,IEF"
min_history: 100
z_window: 60
z_entry: 2.0
min_t_stat: 4.0
take_profit: 8.0
stop_loss: 4.0
spread_weight: 1.0
max_units: 4
max_hold: 63
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 5.0
book_slots: 8
""") + variants(v)

# ---------- G010 ----------
v = []
for zw in [5,10,15,20,30,40,60,90]:
    v.append((f"w{zw}", {"z_window": zw}))
for zw in [10,15,20,30,40,60,90]:
    v.append((f"w{zw}_ls", {"z_window": zw, "strategy_mode": "ls"}))
v += [("ls", {"strategy_mode": "ls"}), ("weekly", {"rebalance": "weekly"}),
      ("w10_weekly", {"z_window": 10, "rebalance": "weekly"}),
      ("w20_weekly", {"z_window": 20, "rebalance": "weekly"}),
      ("w30_weekly", {"z_window": 30, "rebalance": "weekly"}),
      ("w60_weekly", {"z_window": 60, "rebalance": "weekly"}),
      ("w10_ls_weekly", {"z_window": 10, "strategy_mode": "ls", "rebalance": "weekly"}),
      ("w20_ls_weekly", {"z_window": 20, "strategy_mode": "ls", "rebalance": "weekly"}),
      ("w30_ls_weekly", {"z_window": 30, "strategy_mode": "ls", "rebalance": "weekly"}),
      ("w60_ls_weekly", {"z_window": 60, "strategy_mode": "ls", "rebalance": "weekly"}),
      ("w15_ls", {"z_window": 15, "strategy_mode": "ls"}),
      ("w5_ls", {"z_window": 5, "strategy_mode": "ls"}),
      ("w90_ls", {"z_window": 90, "strategy_mode": "ls"}),
      ("w5_weekly", {"z_window": 5, "rebalance": "weekly"}),
      ("w15_weekly", {"z_window": 15, "rebalance": "weekly"}),
      ("w40_weekly", {"z_window": 40, "rebalance": "weekly"}),
      ("w90_weekly", {"z_window": 90, "rebalance": "weekly"}),
      ("w40_ls", {"z_window": 40, "strategy_mode": "ls"})]
cfgs["g010_crypto_blend"] = head(
    "G010 crypto blend: sign(mean(-zscore(px,20), rollsum(ret,20))) long/flat\n# on BTC/ETH spot via yfinance (CoinGecko in source).",
    symbols="BTCUSD,ETHUSD",
    data_dir=CRY,
    extra="""min_history: 30
z_window: 20
strategy_mode: ""
rebalance: daily
periods_per_year: 365
borrow_bps_annual: 50.0
cost_bps_per_side: 15.0
book_slots: 4
""", start="2015-09-01") + variants(v)

# ---------- G114 ----------
v = []
for sp in [0.02,0.03,0.05,0.10,0.15]:
    v.append((f"near{int(sp*100)}", {"screen_pct": sp}))
for sm in [20,50,100,150,200]:
    v.append((f"sma{sm}", {"sma_window": sm}))
for lb in [126,189,252,315]:
    v.append((f"lb{lb}", {"lookback": lb}))
for rr in [1.0,1.5,2.0,3.0]:
    v.append((f"rr{str(rr).replace('.','')}", {"risk_reward": rr}))
for mh in [5,10,20,40,60]:
    v.append((f"hold{mh}", {"gap_max_hold": mh}))
for cp in [0.05,0.10,0.15,0.20]:
    v.append((f"con{int(cp*100)}", {"consol_pct": cp}))
for rt in [40.0,50.0,60.0]:
    v.append((f"rsi{int(rt)}", {"rsi_threshold": rt, "rsi_len": 14}))
v += [("con10_w20", {"consol_pct": 0.10, "consol_window": 20}),
      ("con15_w7", {"consol_pct": 0.15, "consol_window": 7}),
      ("near03_rr15", {"screen_pct": 0.03, "risk_reward": 1.5}),
      ("near10_sma20", {"screen_pct": 0.10, "sma_window": 20}),
      ("lb126_sma20", {"lookback": 126, "sma_window": 20}),
      ("con10_rr15", {"consol_pct": 0.10, "risk_reward": 1.5}),
      ("rsi50_rr15", {"rsi_threshold": 50.0, "rsi_len": 14, "risk_reward": 1.5}),
      ("u150", {"universe_top_n": 150}), ("u500", {"universe_top_n": 500}),
      ("etf_uni", {"data_dir": ETF, "symbols": UNI, "universe_top_n": 0, "min_price": 0.0})]
cfgs["g114_swing_scan"] = head(
    "G114 NSE scanner -> US DELTA: within 5% of 252d high AND above SMA50 ->\n# buy above signal-day high, SL below min(close,prev close), target 1:2.",
    symbols='"*"',
    data_dir=STK,
    extra=f"""rank_cache_dir: {RANK}
min_price: 5.0
universe_top_n: 300
min_history: 252
lookback: 252
sma_window: 50
screen_pct: 0.05
consol_pct: 0.0
rsi_threshold: 0.0
risk_reward: 2.0
gap_max_hold: 20
periods_per_year: 252
borrow_bps_annual: 0.0
cost_bps_per_side: 10.0
book_slots: 40
""", start="2016-01-01") + variants(v)

# ---------- G139 ----------
v = []
for thr in [0.95,1.0,1.05,1.10]:
    v.append((f"thr{int(thr*100)}", {"vix_ratio_threshold": thr}))
for ze in [0.5,1.0,1.5,2.0]:
    v.append((f"z{str(ze).replace('.','')}", {"z_entry": ze}))
for zw in [40,60,120,180]:
    v.append((f"zw{zw}", {"z_window": zw}))
for ez in [-0.5,0.0,0.5]:
    v.append((f"ez{str(ez).replace('.','').replace('-','m')}", {"exit_z": ez}))
for mh in [21,42,63,126]:
    v.append((f"hold{mh}", {"max_hold": mh}))
for sw in [0.5,1.0,1.5,2.0]:
    v.append((f"w{str(sw).replace('.','')}", {"spread_weight": sw}))
v += [("thr105_z05", {"vix_ratio_threshold": 1.05, "z_entry": 0.5}),
      ("thr105_z15", {"vix_ratio_threshold": 1.05, "z_entry": 1.5}),
      ("thr100_z05", {"vix_ratio_threshold": 1.0, "z_entry": 0.5}),
      ("thr100_z15", {"vix_ratio_threshold": 1.0, "z_entry": 1.5}),
      ("thr110_z10", {"vix_ratio_threshold": 1.10, "z_entry": 1.0}),
      ("zw40_z05", {"z_window": 40, "z_entry": 0.5}),
      ("zw120_z10", {"z_window": 120, "z_entry": 1.0}),
      ("hold21_z05", {"max_hold": 21, "z_entry": 0.5}),
      ("w05_z05", {"spread_weight": 0.5, "z_entry": 0.5}),
      ("w15_z05", {"spread_weight": 1.5, "z_entry": 0.5}),
      ("thr095_hold126", {"vix_ratio_threshold": 0.95, "max_hold": 126}),
      ("thr105_ezm05", {"vix_ratio_threshold": 1.05, "exit_z": -0.5}),
      ("thr100_zw40", {"vix_ratio_threshold": 1.0, "z_window": 40})]
cfgs["g139_vix_ts_pair"] = head(
    "G139 VIX term-structure study (partial DELTA): VIX/VIX3M spot ratio >=\n# threshold = backwardation proxy; long SPY/TLT log-spread on z <= -z_entry,\n# exit on z >= exit_z, contango, or max_hold.",
    symbols="SPY,TLT",
    extra="""pairs: "SPY,TLT"
vix_file: data/cache/pit/cboe_volatility/VIX_History.csv
vix3m_file: data/cache/pit/cboe_volatility/VIX3M_History.csv
min_history: 100
z_window: 60
z_entry: 1.0
exit_z: 0.0
vix_ratio_threshold: 1.0
spread_weight: 1.0
max_hold: 63
periods_per_year: 252
borrow_bps_annual: 25.0
cost_bps_per_side: 5.0
book_slots: 4
""", start="2008-01-01") + variants(v)

for name, text in cfgs.items():
    with open(f"configs/{name}.yaml", "w", newline="\n") as f:
        f.write(text)
print("wrote", len(cfgs), "configs")
