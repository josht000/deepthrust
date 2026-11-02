"""Compare modeled S041 rebalance legs with two observed intraday price points."""

import argparse
import hashlib
import json
import math
from pathlib import Path

import pandas as pd


def load_snapshots(directories, day):
    bars = {}
    for directory in directories:
        manifest_path = directory / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        if manifest["interval"] != "2m" or manifest["adjustment"] != "raw":
            raise ValueError(f"{manifest_path}: expected unadjusted two-minute bars")
        for item in manifest["files"]:
            symbol = item["symbol"]
            if symbol in bars:
                raise ValueError(f"duplicate snapshot for {symbol}")
            path = directory / item["file"]
            if hashlib.sha256(path.read_bytes()).hexdigest() != item["sha256"]:
                raise ValueError(f"snapshot hash mismatch: {path}")
            frame = pd.read_parquet(path)
            if not isinstance(frame.index, pd.DatetimeIndex) or frame.index.tz is None:
                raise ValueError(f"{path}: timestamps must be timezone-aware")
            idx = frame.index.tz_convert("America/New_York")
            if idx.has_duplicates:
                raise ValueError(f"{path}: duplicate timestamps")
            wanted = [
                pd.Timestamp(f"{day} 09:30", tz="America/New_York"),
                pd.Timestamp(f"{day} 09:32", tz="America/New_York"),
            ]
            selected = frame.set_axis(idx).reindex(wanted)
            if selected["Open"].isna().any():
                raise ValueError(f"{path}: missing 09:30 or 09:32 bar")
            first, delayed = selected["Open"].astype(float)
            if not (math.isfinite(first) and math.isfinite(delayed) and first > 0 and delayed > 0):
                raise ValueError(f"{path}: invalid opening prices")
            bars[symbol] = (first, delayed)
    return bars


def diagnose(trades, bars, day):
    required = {"symbol", "entry_date", "exit_date", "exit_reason"}
    if not required.issubset(trades.columns):
        raise ValueError(f"trade file missing columns: {sorted(required - set(trades.columns))}")
    records = []
    for _, trade in trades.iterrows():
        sides = []
        if trade["entry_date"] == day:
            sides.append("buy")
        if trade["exit_date"] == day and trade["exit_reason"] not in ("censored", "delisted"):
            sides.append("sell")
        for side in sides:
            symbol = trade["symbol"]
            if symbol not in bars:
                raise ValueError(f"missing intraday snapshot for modeled {side} leg {symbol}")
            opening, delayed = bars[symbol]
            direction = 1 if side == "buy" else -1
            records.append(
                {
                    "symbol": symbol,
                    "modeled_leg": side,
                    "open_0930": opening,
                    "open_0932": delayed,
                    "adverse_move_bps": direction * (delayed / opening - 1) * 10000,
                }
            )
    if not records:
        raise ValueError(f"no modeled rebalance legs for {day}")
    result = pd.DataFrame(records)
    buy_symbols = set(result.loc[result.modeled_leg == "buy", "symbol"])
    sell_symbols = set(result.loc[result.modeled_leg == "sell", "symbol"])
    result["symbol_on_both_sides"] = result.symbol.isin(buy_symbols & sell_symbols)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--trades", type=Path, required=True)
    parser.add_argument("--date", required=True, help="rebalance date, YYYY-MM-DD")
    parser.add_argument("--snapshots", type=Path, nargs="+", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    pd.Timestamp(args.date)
    if args.out.exists():
        parser.error("--out already exists; refusing to overwrite")
    trades = pd.read_csv(args.trades, dtype={"symbol": str, "entry_date": str, "exit_date": str})
    bars = load_snapshots(args.snapshots, args.date)
    result = diagnose(trades, bars, args.date)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    result.to_csv(args.out, index=False)
    for side in ("buy", "sell"):
        subset = result[result.modeled_leg == side]
        if not subset.empty:
            print(f"{len(subset)} modeled {side} legs; median adverse move {subset.adverse_move_bps.median():+.1f} bps")
    print("Price points are not guaranteed fills; unchanged holdings are not netted or sized.")


if __name__ == "__main__":
    main()
