"""Fetch 1-minute ES/NQ futures from Databento and build continuous
front-month series for the Price Action Lab.

Pulls GLBX.MDP3 ohlcv-1m per quarterly contract, detects volume-flip
rolls, splices, back-adjusts, and writes into data/futures/raw/:
  es_1m.parquet / nq_1m.parquet            back-adjusted continuous
  es_1m_unadjusted.parquet / nq_...        row-for-row unadjusted twin
  rolls_es.csv / rolls_nq.csv              roll_time_utc,old,new,spread

Env: DATABENTO_API_KEY (or .env file via --env-file).
Usage:
  python tools/fetch_databento_futures.py --env-file C:/data/trading/trading_framework/.env
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import os
from pathlib import Path

import pandas as pd

DATASET = "GLBX.MDP3"
SCHEMA = "ohlcv-1m"
MARKETS = {"ES": "es", "NQ": "nq"}
# pull each contract from this many days before its expiry
TENURE_DAYS = 110


def contract_codes(root: str, start: dt.date, end: dt.date) -> list[tuple[str, dt.date]]:
    """Quarterly contracts (H/M/U/Z) with expiry between start and end,
    plus one lookback contract before start. Returns (databento symbol,
    expiry_date). CME equity index futures expire on the third Friday of
    the contract month."""
    def third_friday(y: int, m: int) -> dt.date:
        d = dt.date(y, m, 1)
        first_friday = d + dt.timedelta(days=(4 - d.weekday()) % 7)
        return first_friday + dt.timedelta(days=14)

    month_codes = {3: "H", 6: "M", 9: "U", 12: "Z"}
    out = []
    for y in range(start.year - 1, end.year + 1):
        for m, code in month_codes.items():
            exp = third_friday(y, m)
            if start - dt.timedelta(days=TENURE_DAYS) <= exp <= end:
                out.append((f"{root}{code}{y % 10}", exp))
    out.sort(key=lambda x: x[1])
    return out


def fetch_contract(client, symbol: str, start: dt.datetime, end: dt.datetime) -> pd.DataFrame:
    store = client.timeseries.get_range(
        dataset=DATASET,
        schema=SCHEMA,
        symbols=[symbol],
        stype_in="raw_symbol",
        start=start,
        end=end,
    )
    df = store.to_df().reset_index()
    df = df.rename(columns={"ts_event": "ts_event"})
    keep = ["ts_event", "open", "high", "low", "close", "volume"]
    df = df[keep].copy()
    df["contract"] = symbol
    return df


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--env-file", default=None)
    ap.add_argument("--start", default="2021-09-20")
    ap.add_argument("--end", default="2026-10-17")
    ap.add_argument("--out-dir", default="data/futures/raw")
    ap.add_argument("--contracts-dir", default="data/futures/contracts")
    args = ap.parse_args()

    if args.env_file:
        from dotenv import dotenv_values
        os.environ.update({k: v for k, v in dotenv_values(args.env_file).items() if v})
    key = os.environ.get("DATABENTO_API_KEY")
    if not key:
        raise SystemExit("DATABENTO_API_KEY not set")

    import databento as db
    client = db.Historical(key=key)

    start_d = dt.date.fromisoformat(args.start)
    end_d = dt.date.fromisoformat(args.end)
    out_dir = Path(args.out_dir)
    con_dir = Path(args.contracts_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    con_dir.mkdir(parents=True, exist_ok=True)

    for root, tag in MARKETS.items():
        print(f"== {root} ==", flush=True)
        frames = []
        contracts = contract_codes(root, start_d, end_d)
        print("contracts:", [c for c, _ in contracts], flush=True)
        for symbol, exp in contracts:
            pq = con_dir / f"{symbol}.parquet"
            if pq.exists():
                df = pd.read_parquet(pq)
            else:
                pull_start = exp - dt.timedelta(days=TENURE_DAYS)
                pull_end = exp + dt.timedelta(days=1)
                print(f"  {symbol} {pull_start}..{exp}", flush=True)
                df = fetch_contract(client, symbol, pull_start, pull_end)
                df.to_parquet(pq)
            df["ts_event"] = pd.to_datetime(df["ts_event"], utc=True)
            df = df[(df["ts_event"].dt.date >= start_d) & (df["ts_event"].dt.date <= end_d)]
            frames.append((symbol, exp, df))
            print(f"    {len(df)} bars", flush=True)

        rolls, adjusted, unadjusted = splice(frames)
        adj_path = out_dir / f"{tag}_1m.parquet"
        unadj_path = out_dir / f"{tag}_1m_unadjusted.parquet"
        rolls_path = Path("data/futures") / f"rolls_{tag}.csv"
        adjusted.to_parquet(adj_path)
        unadjusted.to_parquet(unadj_path)
        rolls.to_csv(rolls_path, index=False)
        for p in (adj_path, unadj_path, rolls_path):
            h = hashlib.sha256(p.read_bytes()).hexdigest()
            print(f"  wrote {p} sha256={h[:16]}..", flush=True)


def splice(frames: list[tuple[str, dt.date, pd.DataFrame]]):
    """Volume-flip splice: for each consecutive contract pair, the roll
    is the first hour where the new contract's hourly volume exceeds the
    old's. Earlier history is back-adjusted by the cumulative price
    spread at each roll boundary (spread = new close - old close at the
    splice bar)."""
    parts_adj, parts_unadj, roll_rows = [], [], []
    cumulative_adj = 0.0
    start_ts = None

    for i, (symbol, _exp, df) in enumerate(frames):
        df = df.sort_values("ts_event").reset_index(drop=True)
        if i == 0:
            start_ts = df["ts_event"].min()

        if i + 1 < len(frames):
            nxt_sym, _e2, ndf = frames[i + 1]
            ndf = ndf.sort_values("ts_event")
            end_ts = overlap_roll(df, ndf)
            if end_ts is None:
                end_ts = df["ts_event"].max() + pd.Timedelta(nanoseconds=1)
            spread = spread_at(df, ndf, end_ts)
            roll_rows.append({
                "roll_time_utc": end_ts,
                "old_contract": symbol,
                "new_contract": nxt_sym,
                "spread": spread,
            })
        else:
            end_ts = df["ts_event"].max() + pd.Timedelta(nanoseconds=1)

        sel = df[(df["ts_event"] >= start_ts) & (df["ts_event"] < end_ts)].copy()
        parts_unadj.append(sel)
        adj = sel.copy()
        for c in ("open", "high", "low", "close"):
            adj[c] = adj[c] + cumulative_adj
        parts_adj.append(adj)
        if roll_rows:
            cumulative_adj += roll_rows[-1]["spread"]
        start_ts = end_ts

    rolls = pd.DataFrame(roll_rows)
    return rolls, pd.concat(parts_adj), pd.concat(parts_unadj)


def overlap_roll(old: pd.DataFrame, new: pd.DataFrame):
    """First ts_event where new contract's per-hour volume exceeds
    old's (volume-flip roll). Fallback: None."""
    o = old.set_index("ts_event")["volume"].resample("1h").sum()
    n = new.set_index("ts_event")["volume"].resample("1h").sum()
    both = pd.DataFrame({"o": o, "n": n}).dropna()
    win = both[both["n"] > both["o"]]
    if win.empty:
        return None
    return win.index[0]


def spread_at(old: pd.DataFrame, new: pd.DataFrame, at_ts) -> float:
    oc = old[old["ts_event"] <= at_ts]["close"]
    nc = new[new["ts_event"] >= at_ts]["close"]
    if oc.empty or nc.empty:
        return 0.0
    return float(nc.iloc[0] - oc.iloc[-1])


if __name__ == "__main__":
    main()
