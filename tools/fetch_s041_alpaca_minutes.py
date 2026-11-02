"""Fetch an Alpaca IEX raw-minute snapshot for historical S041 trade candidates."""

import argparse
import hashlib
import json
import os
from datetime import datetime, time, timezone
from pathlib import Path
from zoneinfo import ZoneInfo

import pandas as pd
import requests
from dotenv import load_dotenv

URL = "https://data.alpaca.markets/v2/stocks/bars"


def fetch_bars(session, symbols, start, end):
    params = {
        "symbols": ",".join(symbols),
        "timeframe": "1Min",
        "start": start.isoformat(),
        "end": end.isoformat(),
        "feed": "iex",
        "adjustment": "raw",
        "limit": 10000,
        "sort": "asc",
    }
    data = {symbol: [] for symbol in symbols}
    seen_tokens = set()
    while True:
        response = session.get(URL, params=params, timeout=90)
        response.raise_for_status()
        payload = response.json()
        for symbol, bars in (payload.get("bars") or {}).items():
            if symbol not in data:
                raise ValueError(f"unexpected symbol from provider: {symbol}")
            data[symbol].extend(bars)
        token = payload.get("next_page_token")
        if not token:
            return data
        if token in seen_tokens:
            raise ValueError("provider repeated a pagination token")
        seen_tokens.add(token)
        params["page_token"] = token


def normalize(symbol, bars, start, end):
    frame = pd.DataFrame(bars)
    required = {"t", "o", "h", "l", "c", "v"}
    if not required.issubset(frame.columns):
        raise ValueError(f"{symbol}: missing columns {sorted(required - set(frame.columns))}")
    frame = frame.rename(
        columns={"t": "timestamp", "o": "open", "h": "high", "l": "low", "c": "close", "v": "volume"}
    )
    frame["timestamp"] = pd.to_datetime(frame["timestamp"], utc=True, errors="raise")
    frame = frame.set_index("timestamp").sort_index()
    if frame.index.has_duplicates or ((frame.index < start) | (frame.index >= end)).any():
        raise ValueError(f"{symbol}: duplicate or out-of-range timestamps")
    cols = ["open", "high", "low", "close", "volume"]
    frame[cols] = frame[cols].apply(pd.to_numeric, errors="raise")
    if frame[cols].isna().any().any():
        raise ValueError(f"{symbol}: missing OHLCV data")
    if not frame[cols].map(lambda v: float("-inf") < v < float("inf")).all().all():
        raise ValueError(f"{symbol}: nonfinite OHLCV data")
    if (frame[["open", "high", "low", "close"]] <= 0).any().any() or (frame.volume < 0).any():
        raise ValueError(f"{symbol}: invalid price or volume")
    if (frame.high < frame[["open", "low", "close"]].max(axis=1)).any() or (
        frame.low > frame[["open", "high", "close"]].min(axis=1)
    ).any():
        raise ValueError(f"{symbol}: inconsistent OHLC data")
    return frame


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--trades", type=Path, required=True)
    parser.add_argument("--start", required=True, help="inclusive date")
    parser.add_argument("--end", required=True, help="exclusive date")
    parser.add_argument("--env-file", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    args = parser.parse_args()
    start_date = datetime.strptime(args.start, "%Y-%m-%d").date()
    end_date = datetime.strptime(args.end, "%Y-%m-%d").date()
    if start_date >= end_date:
        parser.error("--start must precede --end")
    if not args.env_file.is_file():
        parser.error("--env-file does not exist")
    if args.out_dir.exists() and any(args.out_dir.iterdir()):
        parser.error("--out-dir must be absent or empty")
    trades = pd.read_csv(args.trades, parse_dates=["entry_date", "exit_date"])
    selected = trades[(trades.entry_date < pd.Timestamp(end_date)) & (trades.exit_date >= pd.Timestamp(start_date))]
    symbols = sorted(set(selected.symbol) | {"SPY"})
    if len(symbols) == 1:
        raise ValueError("no overlapping stock trades")
    start = datetime.combine(start_date, time.min, tzinfo=ZoneInfo("America/New_York")).astimezone(timezone.utc)
    end = datetime.combine(end_date, time.min, tzinfo=ZoneInfo("America/New_York")).astimezone(timezone.utc)
    load_dotenv(args.env_file)
    key, secret = os.getenv("ALPACA_API_KEY"), os.getenv("ALPACA_API_SECRET")
    if not key or not secret:
        raise ValueError("Alpaca data credentials are not configured")
    session = requests.Session()
    session.headers.update({"APCA-API-KEY-ID": key, "APCA-API-SECRET-KEY": secret})
    started = datetime.now(timezone.utc).isoformat()
    raw = fetch_bars(session, symbols, start, end)
    data = {symbol: normalize(symbol, bars, start, end) for symbol, bars in raw.items() if bars}
    args.out_dir.mkdir(parents=True, exist_ok=True)
    files = []
    for symbol, frame in data.items():
        path = args.out_dir / f"{symbol}.parquet"
        frame.to_parquet(path)
        counts = pd.Series(frame.index.tz_convert("America/New_York").date).value_counts().sort_index()
        files.append(
            {
                "symbol": symbol,
                "file": path.name,
                "rows": len(frame),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "session_counts": {str(day): int(count) for day, count in counts.items()},
            }
        )
    manifest = {
        "provider": "Alpaca",
        "feed": "iex",
        "adjustment": "raw",
        "interval": "1m",
        "requested_start": args.start,
        "requested_end_exclusive": args.end,
        "request_started_at_utc": started,
        "fetched_at_utc": datetime.now(timezone.utc).isoformat(),
        "point_in_time_revisions_verified": False,
        "complete_session_coverage_verified": False,
        "selection_note": "Historical trade overlaps identify download candidates, not an execution signal.",
        "requested_symbols": symbols,
        "missing_symbols": sorted(set(symbols) - set(data)),
        "files": files,
    }
    (args.out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"Downloaded {len(data)}/{len(symbols)} symbols and {sum(len(f) for f in data.values())} bars.")
    if manifest["missing_symbols"]:
        print(f"Missing symbols: {manifest['missing_symbols']}")
    print("IEX is one exchange; publication-time revisions and complete session coverage are not verified.")


if __name__ == "__main__":
    main()
