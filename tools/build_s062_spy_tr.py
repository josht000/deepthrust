"""Build a continuous SPY total-return Parquet series for S062."""

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import pandas as pd

ROOT = Path(__file__).resolve().parents[1]
FRAMEWORK_DATA = ROOT.parent / "trading_framework" / "data" / "cache"


def _dates(index):
    index = pd.DatetimeIndex(index)
    if index.tz is not None:
        index = index.tz_convert("America/New_York").tz_localize(None)
    return index.normalize()


def build(history_dir, recent_dir, output_dir):
    history_path = Path(history_dir) / "SPY.parquet"
    recent_path = Path(recent_dir) / "SPY.parquet"
    if not history_path.is_file() or not recent_path.is_file():
        raise FileNotFoundError(f"need SPY history and recent bars: {history_path}, {recent_path}")

    yahoo = pd.read_parquet(
        history_path, columns=["open", "high", "low", "close", "adj_close", "volume"]
    )
    yahoo.index = _dates(yahoo.index)
    factor = yahoo["adj_close"] / yahoo["close"]
    yahoo_tr = pd.DataFrame(
        {
            "open": yahoo["open"] * factor,
            "high": yahoo["high"] * factor,
            "low": yahoo["low"] * factor,
            "close": yahoo["adj_close"],
            "volume": yahoo["volume"],
        },
        index=yahoo.index,
    )
    recent = pd.read_parquet(
        recent_path, columns=["open", "high", "low", "close", "volume"]
    )
    recent.index = _dates(recent.index)

    overlap = yahoo_tr.index.intersection(recent.index)
    if overlap.empty:
        raise ValueError("Yahoo SPY and Alpaca TR bars have no common session")
    seam = overlap.min()
    scale = float(yahoo_tr.loc[seam, "close"] / recent.loc[seam, "close"])
    if not np.isfinite(scale) or scale <= 0:
        raise ValueError("invalid total-return splice scale")

    early = yahoo_tr.loc[yahoo_tr.index <= seam]
    late = recent.loc[recent.index > seam].copy()
    late.loc[:, ["open", "high", "low", "close"]] *= scale
    combined = pd.concat([early, late]).sort_index()
    combined = combined[~combined.index.duplicated(keep="last")]
    combined = combined.replace([np.inf, -np.inf], np.nan).dropna()
    if combined.empty or combined.index.has_duplicates:
        raise ValueError("SPY total-return series is empty or has duplicate dates")

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    output_path = output_dir / "SPY.parquet"
    frame = combined.rename_axis("date").reset_index()
    temporary = output_path.with_suffix(".parquet.tmp")
    frame.to_parquet(temporary, index=False)
    temporary.replace(output_path)
    manifest = {
        "symbol": "SPY",
        "series": "total_return",
        "history_source": str(history_path),
        "recent_source": str(recent_path),
        "splice_date": seam.date().isoformat(),
        "start_date": combined.index[0].date().isoformat(),
        "end_date": combined.index[-1].date().isoformat(),
        "rows": int(len(combined)),
        "output": str(output_path),
        "sha256": hashlib.sha256(output_path.read_bytes()).hexdigest(),
    }
    (output_dir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print(
        f"SPY TR: {len(combined)} sessions, {combined.index[0].date()} .. "
        f"{combined.index[-1].date()}, splice {seam.date()}"
    )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--history-dir", default=str(FRAMEWORK_DATA / "etf_yahoo_2008_2025"))
    parser.add_argument("--recent-dir", default=str(FRAMEWORK_DATA / "etf_tr"))
    parser.add_argument("--output-dir", default=str(ROOT / "data" / "cache" / "s062_spy_tr"))
    args = parser.parse_args()
    build(args.history_dir, args.recent_dir, args.output_dir)


if __name__ == "__main__":
    main()
