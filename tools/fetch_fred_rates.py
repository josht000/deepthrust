"""Fetch public FRED daily interest-rate series (no API key required)."""

import argparse
import csv
import hashlib
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

FRED_BASE = "https://fred.stlouisfed.org/graph/fredgraph.csv?id="
SERIES = ["DGS10", "DGS2", "T10Y2Y", "DTB3"]


def _download(url, path):
    request = urllib.request.Request(url, headers={"User-Agent": "deepthrust public data research"})
    temporary = path.with_suffix(path.suffix + ".part")
    with urllib.request.urlopen(request, timeout=60) as response, temporary.open("wb") as target:
        while True:
            block = response.read(1024 * 1024)
            if not block:
                break
            target.write(block)
    temporary.replace(path)


def fetch(output_dir):
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    rows = []
    for series in SERIES:
        url = FRED_BASE + series
        filename = f"{series}.csv"
        path = output_dir / filename
        _download(url, path)
        with path.open(encoding="utf-8-sig", newline="") as source:
            reader = csv.DictReader(source)
            if "observation_date" not in (reader.fieldnames or []) or series not in reader.fieldnames:
                raise ValueError(f"{path}: unexpected CSV header {reader.fieldnames}")
            dates = [
                row["observation_date"]
                for row in reader
                if row.get("observation_date") and row.get(series) not in (None, "", ".")
            ]
        if not dates:
            raise ValueError(f"{path}: no data rows")
        rows.append(
            {
                "symbol": series,
                "url": url,
                "file": filename,
                "bytes": path.stat().st_size,
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "first_date": dates[0],
                "last_date": dates[-1],
                "downloaded_utc": datetime.now(timezone.utc).isoformat(),
            }
        )
        print(f"{series}: {len(dates)} rows, {dates[0]} .. {dates[-1]}")

    manifest = output_dir / "manifest.csv"
    with manifest.open("w", encoding="utf-8", newline="") as target:
        writer = csv.DictWriter(target, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    print(f"manifest: {manifest}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output-dir",
        default=r"C:\data\trading\trading_framework\data\pit\market\fred",
    )
    args = parser.parse_args()
    fetch(args.output_dir)


if __name__ == "__main__":
    main()
