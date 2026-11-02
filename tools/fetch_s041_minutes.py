"""Download timestamped, unadjusted Yahoo intraday snapshots for S041 research."""

import argparse
import hashlib
import json
import re
from datetime import datetime, timedelta, timezone
from pathlib import Path

import numpy as np
import pandas as pd
import yfinance as yf


def windows(start, end):
    while start < end:
        next_start = min(start + timedelta(days=7), end)
        yield start, next_start
        start = next_start


def validate_bars(frame, start, end):
    columns = ["Open", "High", "Low", "Close", "Volume"]
    missing = set(columns) - set(frame.columns)
    if missing:
        raise ValueError(f"minute bars missing columns: {sorted(missing)}")
    if frame.empty:
        raise ValueError("provider returned no minute bars")
    if not isinstance(frame.index, pd.DatetimeIndex) or frame.index.tz is None:
        raise ValueError("minute bars must have timezone-aware timestamps")
    bars = frame[columns].copy()
    bars.index = bars.index.tz_convert("UTC")
    bars.index.name = "timestamp"
    if bars.index.has_duplicates or not bars.index.is_monotonic_increasing:
        raise ValueError("minute timestamps must be unique and increasing")
    if (bars.index < pd.Timestamp(start, tz="UTC")).any() or (
        bars.index >= pd.Timestamp(end, tz="UTC")
    ).any():
        raise ValueError("provider returned bars outside the requested interval")
    if bars.isna().any().any():
        raise ValueError("minute bars contain missing OHLCV values")
    if not np.isfinite(bars.to_numpy(dtype=float)).all():
        raise ValueError("minute bars contain non-finite OHLCV values")
    if not (bars[["Open", "High", "Low", "Close"]] > 0).all().all():
        raise ValueError("minute bars contain nonpositive prices")
    if (bars["Volume"] < 0).any():
        raise ValueError("minute bars contain negative volume")
    if (
        (bars["High"] < bars[["Open", "Close", "Low"]].max(axis=1)).any()
        or (bars["Low"] > bars[["Open", "Close", "High"]].min(axis=1)).any()
    ):
        raise ValueError("minute bars contain inconsistent OHLC values")
    return bars


def fetch_symbol(symbol, start, end, interval):
    frames = []
    coverage = []
    for window_start, window_end in windows(start, end):
        frame = yf.Ticker(symbol).history(
            start=window_start.isoformat(),
            end=window_end.isoformat(),
            interval=interval,
            auto_adjust=False,
            actions=False,
            prepost=False,
            raise_errors=True,
        )
        coverage.append(
            {"start": window_start.isoformat(), "end_exclusive": window_end.isoformat(), "rows": len(frame)}
        )
        if frame.empty:
            print(f"WARNING: {symbol}: provider returned no bars for {window_start} .. {window_end}")
        if not frame.empty:
            frames.append(frame)
    if not frames:
        raise ValueError(f"{symbol}: provider returned no bars")
    return validate_bars(pd.concat(frames), start, end), coverage


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--symbols", nargs="+", required=True)
    parser.add_argument("--start", required=True, help="inclusive date, YYYY-MM-DD")
    parser.add_argument("--end", required=True, help="exclusive date, YYYY-MM-DD")
    parser.add_argument("--interval", choices=("1m", "2m"), default="1m")
    parser.add_argument("--out-dir", required=True, type=Path)
    args = parser.parse_args()
    start = datetime.strptime(args.start, "%Y-%m-%d").date()
    end = datetime.strptime(args.end, "%Y-%m-%d").date()
    if start >= end:
        parser.error("--start must precede --end")
    symbols = list(dict.fromkeys(args.symbols))
    if any(not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9.-]*", symbol) for symbol in symbols):
        parser.error("symbols must contain only letters, digits, dot, or hyphen")
    if args.out_dir.exists() and any(args.out_dir.iterdir()):
        parser.error("--out-dir must be absent or empty; existing snapshots are not overwritten")

    request_started_at = datetime.now(timezone.utc).isoformat()
    frames = {symbol: fetch_symbol(symbol, start, end, args.interval) for symbol in symbols}
    fetched_at = datetime.now(timezone.utc).isoformat()
    args.out_dir.mkdir(parents=True, exist_ok=True)
    files = []
    for symbol, (bars, coverage) in frames.items():
        path = args.out_dir / f"{symbol}.parquet"
        bars.to_parquet(path, index=True)
        files.append(
            {
                "symbol": symbol,
                "file": path.name,
                "rows": len(bars),
                "first_utc": bars.index[0].isoformat(),
                "last_utc": bars.index[-1].isoformat(),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "windows": coverage,
                "session_counts": {
                    str(day): int(count)
                    for day, count in pd.Series(
                        bars.index.tz_convert("America/New_York").date
                    ).value_counts().sort_index().items()
                },
            }
        )
    manifest = {
        "provider": "Yahoo Finance via yfinance",
        "yfinance_version": yf.__version__,
        "requested_start": args.start,
        "requested_end_exclusive": args.end,
        "request_started_at_utc": request_started_at,
        "fetched_at_utc": fetched_at,
        "interval": args.interval,
        "adjustment": "raw",
        "extended_hours": False,
        "point_in_time_revisions_verified": False,
        "complete_session_coverage_verified": False,
        "files": files,
    }
    (args.out_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    print(f"Saved {sum(item['rows'] for item in files)} bars to {args.out_dir}")
    print("Historical publication times and revisions are not verified.")


if __name__ == "__main__":
    main()
