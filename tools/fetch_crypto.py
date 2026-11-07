"""Fetch BTC/ETH spot daily bars from yfinance into the shared cache.

Writes <SYMBOL>.parquet files matching the etf_tr schema:
timestamp index (UTC), columns symbol/open/high/low/close/volume/trade_count/vwap.
"""

import argparse
from pathlib import Path

import pandas as pd
import yfinance as yf

MAP = {"BTCUSD": "BTC-USD", "ETHUSD": "ETH-USD"}


def fetch(symbol: str, yf_symbol: str, out_dir: Path) -> Path:
    df = yf.download(yf_symbol, start="2014-09-01", auto_adjust=False, progress=False)
    if isinstance(df.columns, pd.MultiIndex):
        df.columns = df.columns.get_level_values(0)
    df = df.dropna(subset=["Close"])
    out = pd.DataFrame(
        {
            "symbol": symbol,
            "open": df["Open"].astype(float),
            "high": df["High"].astype(float),
            "low": df["Low"].astype(float),
            "close": df["Close"].astype(float),
            "volume": df["Volume"].astype(float),
            "trade_count": 0.0,
            "vwap": (df["High"] + df["Low"] + df["Close"]).astype(float) / 3.0,
        },
        index=pd.to_datetime(df.index).tz_localize("UTC"),
    )
    out.index.name = "timestamp"
    out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / f"{symbol}.parquet"
    out.to_parquet(path)
    return path


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--out-dir",
        default=r"C:\data\trading\trading_framework\data\cache\crypto",
    )
    args = ap.parse_args()
    for symbol, yf_symbol in MAP.items():
        path = fetch(symbol, yf_symbol, Path(args.out_dir))
        print(f"{symbol}: {path} ({len(pd.read_parquet(path))} rows)")


if __name__ == "__main__":
    main()
