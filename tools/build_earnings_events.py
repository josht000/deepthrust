#!/usr/bin/env python3
"""Build a local symbol/date earnings calendar from SimFin and SEC submissions."""

from __future__ import annotations

import argparse
import csv
import gzip
import json
import os
import re
import sys
import threading
import time
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Any
from zoneinfo import ZoneInfo

SEC_SUBMISSIONS_URL = "https://data.sec.gov/submissions"
EASTERN = ZoneInfo("America/New_York")
OUTPUT_FIELDS = (
    "ticker",
    "date",
    "source",
    "release_timing",
    "acceptance_utc",
)


class RateLimiter:
    def __init__(self, requests_per_second: float) -> None:
        if requests_per_second <= 0:
            raise ValueError("requests_per_second must be positive")
        self.interval = 1.0 / requests_per_second
        self.lock = threading.Lock()
        self.next_request = 0.0

    def wait(self) -> None:
        with self.lock:
            now = time.monotonic()
            delay = max(0.0, self.next_request - now)
            self.next_request = max(now, self.next_request) + self.interval
        if delay:
            time.sleep(delay)


def parse_iso_date(value: str, label: str) -> date:
    try:
        return date.fromisoformat(value[:10])
    except ValueError as error:
        raise ValueError(f"invalid {label} date {value!r}") from error


def parse_acceptance(value: str) -> datetime | None:
    if not value:
        return None
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None
    if parsed.tzinfo is None:
        return parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def release_timing(timestamp: datetime | None) -> str:
    if timestamp is None:
        return "unknown"
    local_time = timestamp.astimezone(EASTERN).time()
    if local_time < datetime.strptime("09:30", "%H:%M").time():
        return "before_market"
    if local_time >= datetime.strptime("16:00", "%H:%M").time():
        return "after_market"
    return "intraday"


def _column(row: dict[str, str], name: str) -> str:
    return next(
        (value.strip() for key, value in row.items() if key.lower() == name.lower()),
        "",
    )


def read_company_map(path: Path) -> dict[str, set[str]]:
    by_cik: dict[str, set[str]] = {}
    if path.suffix.lower() == ".parquet":
        try:
            import pyarrow.parquet as parquet
        except ImportError as error:
            raise RuntimeError(
                "reading reference.parquet requires pyarrow; install requirements-data.txt"
            ) from error
        table = parquet.read_table(path, columns=["ticker", "cik"])
        rows = zip(table.column("ticker").to_pylist(), table.column("cik").to_pylist())
        for ticker_value, cik_value in rows:
            ticker = str(ticker_value or "").strip().upper()
            raw_cik = str(cik_value or "").strip()
            if ticker and raw_cik:
                digits = re.sub(r"\D", "", raw_cik)
                if digits:
                    by_cik.setdefault(digits.zfill(10), set()).add(ticker)
        if not by_cik:
            raise ValueError(f"{path}: no ticker/CIK mappings found")
        return by_cik

    with path.open(newline="", encoding="utf-8-sig") as source:
        reader = csv.DictReader(source, delimiter=";")
        if not reader.fieldnames or not {"ticker", "cik"}.issubset(
            {name.lower() for name in reader.fieldnames}
        ):
            raise ValueError(f"{path}: expected Ticker and CIK columns")
        for row_number, row in enumerate(reader, 2):
            ticker = _column(row, "Ticker").upper()
            raw_cik = _column(row, "CIK")
            if not ticker or not raw_cik:
                continue
            digits = re.sub(r"\D", "", raw_cik)
            if not digits:
                raise ValueError(f"{path} row {row_number}: invalid CIK {raw_cik!r}")
            by_cik.setdefault(digits.zfill(10), set()).add(ticker)
    if not by_cik:
        raise ValueError(f"{path}: no ticker/CIK mappings found")
    return by_cik


def filter_company_map(
    by_cik: dict[str, set[str]], symbols: set[str]
) -> dict[str, set[str]]:
    filtered = {
        cik: matching
        for cik, tickers in by_cik.items()
        if (matching := tickers & symbols)
    }
    if not filtered:
        raise ValueError("no reference ticker/CIK mappings match the requested symbols")
    return filtered


def read_rank_cache_symbols(
    path: Path, start_date: date, end_date: date, top_n: int
) -> set[str]:
    with path.open(encoding="utf-8") as source:
        payload = json.load(source)
    symbols = payload.get("symbols", [])
    dates = payload.get("dates", [])
    if not symbols:
        raise ValueError(f"{path}: no symbols listed in rank-cache index")
    if not dates:
        raise ValueError(f"{path}: no dates listed in rank-cache index")
    try:
        import numpy as np
    except ImportError as error:
        raise RuntimeError(
            "reading the PIT rank cache requires numpy; install requirements-data.txt"
        ) from error
    rank_path = path.with_name("ranks.npy")
    ranks = np.load(rank_path, mmap_mode="r")
    if ranks.shape != (len(dates), len(symbols)):
        raise ValueError(
            f"{rank_path}: rank shape {ranks.shape} does not match index "
            f"({len(dates)}, {len(symbols)})"
        )
    date_indices = [
        index
        for index, value in enumerate(dates)
        if start_date <= parse_iso_date(value, "rank cache") <= end_date
    ]
    if not date_indices:
        raise ValueError(f"{path}: no rank-cache dates in requested date range")
    selected = np.zeros(len(symbols), dtype=bool)
    if top_n == 0:
        selected[:] = True
    else:
        for start in range(min(date_indices), max(date_indices) + 1, 128):
            stop = min(start + 128, max(date_indices) + 1)
            chunk = ranks[start:stop]
            selected |= np.any(np.isfinite(chunk) & (chunk <= top_n), axis=0)
    return {
        str(symbol).strip().upper()
        for symbol, is_selected in zip(symbols, selected)
        if is_selected and str(symbol).strip()
    }


def read_simfin_events(
    path: Path | None, start_date: date, end_date: date
) -> list[dict[str, str]]:
    if path is None:
        return []
    events: list[dict[str, str]] = []
    with path.open(newline="", encoding="utf-8-sig") as source:
        reader = csv.DictReader(source, delimiter=";")
        if not reader.fieldnames or not {"ticker", "publish date"}.issubset(
            {name.lower() for name in reader.fieldnames}
        ):
            raise ValueError(f"{path}: expected Ticker and Publish Date columns")
        for row_number, row in enumerate(reader, 2):
            ticker = _column(row, "Ticker").upper()
            raw_date = _column(row, "Publish Date")
            if not ticker or not raw_date:
                continue
            event_date = parse_iso_date(raw_date, f"{path} row {row_number}")
            if start_date <= event_date <= end_date:
                events.append(
                    {
                        "ticker": ticker,
                        "date": event_date.isoformat(),
                        "source": "simfin_publish_date",
                        "release_timing": "unknown",
                        "acceptance_utc": "",
                    }
                )
    return events


def _filing_rows(payload: dict[str, Any]) -> list[dict[str, Any]]:
    filings = payload.get("filings", {})
    rows: list[dict[str, Any]] = []
    recent = filings.get("recent", {})
    keys = (
        "form",
        "filingDate",
        "acceptanceDateTime",
        "items",
        "accessionNumber",
    )
    for index, form in enumerate(recent.get("form", [])):
        rows.append(
            {
                key: recent.get(key, [""] * (index + 1))[index]
                if index < len(recent.get(key, []))
                else ""
                for key in keys
            }
        )
    return rows


def extract_sec_events(
    payloads: list[dict[str, Any]],
    ticker_by_cik: set[str],
    start_date: date,
    end_date: date,
) -> list[dict[str, str]]:
    events: dict[tuple[str, str], dict[str, str]] = {}
    seen_filings: set[str] = set()
    for payload in payloads:
        for filing in _filing_rows(payload):
            if filing["form"] not in {"8-K", "8-K/A"}:
                continue
            items = {
                item.strip()
                for item in re.split(r"[,;\s]+", str(filing["items"] or ""))
                if item.strip()
            }
            if "2.02" not in items:
                continue
            accession = str(filing["accessionNumber"] or "")
            if accession and accession in seen_filings:
                continue
            if accession:
                seen_filings.add(accession)
            raw_date = str(filing["filingDate"] or "")
            if not raw_date:
                continue
            event_date = parse_iso_date(raw_date, "SEC filing")
            if not start_date <= event_date <= end_date:
                continue
            timestamp = parse_acceptance(str(filing["acceptanceDateTime"] or ""))
            for ticker in ticker_by_cik:
                key = (ticker, event_date.isoformat())
                candidate = {
                    "ticker": ticker,
                    "date": event_date.isoformat(),
                    "source": "sec_8k_item_2.02",
                    "release_timing": release_timing(timestamp),
                    "acceptance_utc": timestamp.isoformat().replace("+00:00", "Z")
                    if timestamp
                    else "",
                }
                previous = events.get(key)
                if previous is None or (
                    candidate["acceptance_utc"]
                    and (
                        not previous["acceptance_utc"]
                        or candidate["acceptance_utc"] < previous["acceptance_utc"]
                    )
                ):
                    events[key] = candidate
    return list(events.values())


def _read_json(path: Path) -> dict[str, Any]:
    if path.suffix == ".gz":
        with gzip.open(path, "rt", encoding="utf-8") as source:
            return json.load(source)
    with path.open(encoding="utf-8") as source:
        return json.load(source)


def _write_json_cache(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    with gzip.open(temporary, "wt", encoding="utf-8") as target:
        json.dump(payload, target, separators=(",", ":"))
    temporary.replace(path)


def _fetch_json(url: str, user_agent: str, limiter: RateLimiter) -> dict[str, Any]:
    for attempt in range(6):
        limiter.wait()
        request = urllib.request.Request(
            url,
            headers={
                "User-Agent": user_agent,
                "Accept": "application/json",
                "Accept-Encoding": "gzip",
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                body = response.read()
                if response.headers.get("Content-Encoding", "").lower() == "gzip":
                    body = gzip.decompress(body)
                return json.loads(body)
        except urllib.error.HTTPError as error:
            if error.code == 429 or 500 <= error.code < 600:
                retry_after = error.headers.get("Retry-After", "")
                delay = float(retry_after) if retry_after.isdigit() else 2**attempt
                time.sleep(min(delay, 60.0))
                continue
            raise RuntimeError(f"SEC returned HTTP {error.code} for {url}") from error
        except (TimeoutError, urllib.error.URLError, json.JSONDecodeError) as error:
            if attempt == 5:
                raise RuntimeError(f"could not fetch valid SEC JSON from {url}: {error}") from error
            time.sleep(2**attempt)
    raise RuntimeError(f"SEC retries exhausted for {url}")


def fetch_company_submissions(
    cik: str,
    cache_dir: Path,
    start_date: date,
    end_date: date,
    user_agent: str,
    limiter: RateLimiter,
) -> list[dict[str, Any]]:
    cache_dir.mkdir(parents=True, exist_ok=True)
    main_path = cache_dir / f"CIK{cik}.json.gz"
    main = (
        _read_json(main_path)
        if main_path.exists()
        else _fetch_json(f"{SEC_SUBMISSIONS_URL}/CIK{cik}.json", user_agent, limiter)
    )
    if not main_path.exists():
        _write_json_cache(main_path, main)
    payloads = [main]
    for shard in main.get("filings", {}).get("files", []):
        shard_from = str(shard.get("filingFrom", ""))
        shard_to = str(shard.get("filingTo", ""))
        if not shard_from or not shard_to or shard_to < start_date.isoformat():
            continue
        if shard_from > end_date.isoformat():
            continue
        name = Path(str(shard.get("name", ""))).name
        if not name or not name.endswith(".json"):
            continue
        shard_path = cache_dir / f"{cik}_{name}.gz"
        payload = (
            _read_json(shard_path)
            if shard_path.exists()
            else _fetch_json(f"{SEC_SUBMISSIONS_URL}/{name}", user_agent, limiter)
        )
        if not shard_path.exists():
            _write_json_cache(shard_path, payload)
        payloads.append(payload)
    return payloads


def build_parser() -> argparse.ArgumentParser:
    root = Path(__file__).resolve().parents[1]
    trading_root = root.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--companies",
        type=Path,
        default=trading_root / "trading_framework" / "data" / "simfin" / "us-companies.csv",
    )
    parser.add_argument(
        "--reference",
        type=Path,
        default=trading_root / "trading_framework" / "data" / "pit" / "reference.parquet",
    )
    parser.add_argument(
        "--rank-cache-index",
        type=Path,
        default=trading_root
        / "trading_framework"
        / "data"
        / "cache"
        / "liquid_ranks"
        / "index.json",
    )
    parser.add_argument(
        "--simfin-quarterly",
        type=Path,
        default=trading_root
        / "trading_framework"
        / "data"
        / "simfin"
        / "us-income-quarterly.csv",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=root / "data" / "pit" / "earnings_events.csv",
    )
    parser.add_argument(
        "--cache-dir",
        type=Path,
        default=root / "data" / "pit" / "earnings_events_sec_cache",
    )
    parser.add_argument("--start-date", type=date.fromisoformat, default=date(2016, 1, 1))
    parser.add_argument("--end-date", type=date.fromisoformat, default=date.today())
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--requests-per-second", type=float, default=4.0)
    parser.add_argument(
        "--universe-top-n",
        type=int,
        default=500,
        help="include symbols ever ranked in the top N during the date range; 0 includes all",
    )
    parser.add_argument(
        "--user-agent",
        default=os.environ.get("SEC_USER_AGENT", ""),
        help="SEC-compliant application name and contact email (or SEC_USER_AGENT env var)",
    )
    return parser


def write_events(
    events: list[dict[str, str]],
    output: Path,
    start_date: date,
    end_date: date,
    cik_count: int,
    universe_top_n: int,
    requested_symbols: set[str],
    mapped_symbols: set[str],
) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    events.sort(
        key=lambda event: (
            event["ticker"],
            event["date"],
            event["source"],
            event["acceptance_utc"],
        )
    )
    temporary = output.with_suffix(output.suffix + ".tmp")
    with temporary.open("w", newline="", encoding="utf-8") as target:
        writer = csv.DictWriter(
            target, fieldnames=OUTPUT_FIELDS, delimiter=";", lineterminator="\n"
        )
        writer.writeheader()
        writer.writerows(events)
    temporary.replace(output)

    years: dict[str, int] = {}
    source_counts: dict[str, int] = {}
    for event in events:
        year = event["date"][:4]
        years[year] = years.get(year, 0) + 1
        source_counts[event["source"]] = source_counts.get(event["source"], 0) + 1
    manifest = {
        "start_date": start_date.isoformat(),
        "end_date": end_date.isoformat(),
        "universe_top_n": universe_top_n,
        "company_ciks": cik_count,
        "requested_symbols": len(requested_symbols),
        "mapped_symbols": len(mapped_symbols),
        "unmapped_symbols": sorted(requested_symbols - mapped_symbols),
        "event_rows": len(events),
        "source_counts": source_counts,
        "events_by_year": years,
        "output": str(output.resolve()),
    }
    manifest_path = output.with_suffix(output.suffix + ".manifest.json")
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if not args.user_agent or "@" not in args.user_agent:
        parser.error(
            "provide a real contact email using --user-agent or SEC_USER_AGENT; "
            "SEC blocks automated requests without one"
        )
    if args.workers < 1:
        parser.error("--workers must be at least 1")
    if args.universe_top_n < 0:
        parser.error("--universe-top-n must not be negative")
    if args.end_date < args.start_date:
        parser.error("--end-date must be on or after --start-date")
    for path in (args.companies, args.reference, args.rank_cache_index, args.simfin_quarterly):
        if not path.is_file():
            parser.error(f"required input file does not exist: {path}")

    symbols = read_rank_cache_symbols(
        args.rank_cache_index, args.start_date, args.end_date, args.universe_top_n
    )
    by_cik = read_company_map(args.reference)
    company_map = read_company_map(args.companies)
    for cik, tickers in company_map.items():
        by_cik.setdefault(cik, set()).update(tickers)
    by_cik = filter_company_map(by_cik, symbols)
    mapped_symbols = set().union(*by_cik.values())
    limiter = RateLimiter(args.requests_per_second)
    payloads_by_cik: dict[str, list[dict[str, Any]]] = {}
    failures: list[str] = []
    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {
            executor.submit(
                fetch_company_submissions,
                cik,
                args.cache_dir,
                args.start_date,
                args.end_date,
                args.user_agent,
                limiter,
            ): cik
            for cik in by_cik
        }
        for index, future in enumerate(as_completed(futures), 1):
            cik = futures[future]
            try:
                payloads_by_cik[cik] = future.result()
            except Exception as error:
                failures.append(f"CIK {cik}: {error}")
            if index % 100 == 0 or index == len(futures):
                print(f"SEC submissions checked: {index}/{len(futures)}", flush=True)

    if failures:
        for failure in failures[:20]:
            print(f"ERROR: {failure}", file=sys.stderr)
        if len(failures) > 20:
            print(f"and {len(failures) - 20} more failures", file=sys.stderr)
        print("No output calendar was written; rerun to resume from the local cache.", file=sys.stderr)
        return 1

    events = [
        event
        for event in read_simfin_events(args.simfin_quarterly, args.start_date, args.end_date)
        if event["ticker"] in symbols
    ]
    for cik, payloads in payloads_by_cik.items():
        events.extend(
            extract_sec_events(payloads, by_cik[cik], args.start_date, args.end_date)
        )
    write_events(
        events,
        args.output,
        args.start_date,
        args.end_date,
        len(by_cik),
        args.universe_top_n,
        symbols,
        mapped_symbols,
    )
    print(f"Wrote {len(events)} earnings-event rows to {args.output}")
    print(f"Manifest: {args.output}.manifest.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
