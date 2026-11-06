"""Download official CBOE volatility histories and CBOE-sourced early VXV history."""

import argparse
import csv
import hashlib
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CBOE_BASE = "https://cdn.cboe.com/api/global/us_indices/daily_prices/"
SOURCES = {
    "VIX": (CBOE_BASE + "VIX_History.csv", "DATE"),
    "VIX3M": (CBOE_BASE + "VIX3M_History.csv", "DATE"),
    "VIX9D": (CBOE_BASE + "VIX9D_History.csv", "DATE"),
    "VXVCLS": ("https://fred.stlouisfed.org/graph/fredgraph.csv?id=VXVCLS", "observation_date"),
}


def download(url, path):
    request = urllib.request.Request(
        url, headers={"User-Agent": "DeepThrust CBOE volatility research"}
    )
    temporary = path.with_suffix(path.suffix + ".part")
    with urllib.request.urlopen(request, timeout=90) as response, temporary.open("wb") as out:
        while True:
            block = response.read(1024 * 1024)
            if not block:
                break
            out.write(block)
    temporary.replace(path)


def fetch(output_dir):
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    rows = []
    for symbol, (url, date_column) in SOURCES.items():
        filename = "FRED_VXVCLS.csv" if symbol == "VXVCLS" else f"{symbol}_History.csv"
        path = output_dir / filename
        download(url, path)
        with path.open(encoding="utf-8-sig", newline="") as source:
            reader = csv.DictReader(source)
            if date_column not in (reader.fieldnames or []):
                raise ValueError(f"{path}: unexpected CSV headers {reader.fieldnames}")
            dates = [row[date_column] for row in reader if row.get(date_column)]
        if not dates:
            raise ValueError(f"{path}: no data rows")
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        rows.append(
            {
                "series": symbol,
                "url": url,
                "file": filename,
                "bytes": path.stat().st_size,
                "sha256": digest,
                "first_date": dates[0],
                "last_date": dates[-1],
                "downloaded_utc": datetime.now(timezone.utc).isoformat(),
            }
        )
        print(f"{symbol}: {len(dates)} rows, {dates[0]} .. {dates[-1]}")

    manifest_path = output_dir / "manifest.csv"
    with manifest_path.open("w", encoding="utf-8", newline="") as out:
        writer = csv.DictWriter(out, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    print(f"manifest: {manifest_path}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output-dir",
        default=str(ROOT / "data" / "cache" / "pit" / "cboe_volatility"),
    )
    args = parser.parse_args()
    fetch(args.output_dir)


if __name__ == "__main__":
    main()
