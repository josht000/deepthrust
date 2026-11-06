"""Normalize SEC quarterly Form 4 TSV archives and identify disclosed 10b5-1 plans."""

import argparse
import csv
import hashlib
import io
import json
import re
import sqlite3
import tempfile
import zipfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_INPUT = ROOT.parent / "trading_framework" / "data" / "pit" / "edgar" / "form345_raw"
DEFAULT_OUTPUT = ROOT / "data" / "cache" / "pit" / "form4"
PLAN_RE = re.compile(
    r"10\s*(?:b|\(\s*b\s*\))\s*(?:5|\(\s*5\s*\))\s*[- ]?\s*1",
    re.IGNORECASE,
)
NEGATED_PLAN_RE = re.compile(
    r"\b(?:not|no)\b.{0,45}\b(?:pursuant|under|subject|effected|made)\b.{0,60}"
    r"10\s*(?:b|\(\s*b\s*\))\s*(?:5|\(\s*5\s*\))\s*[- ]?\s*1",
    re.IGNORECASE,
)
def mentions_10b5_plan(text):
    text = text or ""
    match = PLAN_RE.search(text)
    if not match:
        return False
    return not NEGATED_PLAN_RE.search(text)


def parse_sec_date(value):
    value = (value or "").strip()
    if not value:
        return None
    for fmt in ("%d-%b-%Y", "%Y-%m-%d", "%m/%d/%Y"):
        try:
            return datetime.strptime(value.upper(), fmt).date().isoformat()
        except ValueError:
            pass
    return None


def number(value):
    try:
        return float((value or "").strip().replace(",", ""))
    except ValueError:
        return None


def cik(value):
    value = (value or "").strip()
    return value.lstrip("0") or "0"


def ticker(value):
    value = (value or "").strip().upper().replace(".", "-")
    value = value.strip('"').strip()
    if value.startswith("(") and value.endswith(")"):
        value = value[1:-1].strip().strip('"')
    return value


def table_rows(archive, name):
    with archive.open(name) as raw:
        stream = io.TextIOWrapper(raw, encoding="utf-8-sig", newline="")
        yield from csv.DictReader(stream, delimiter="\t")


def direct_plan_flag(submission):
    value = submission.get("AFF10B5ONE")
    if value is None or not value.strip():
        return None
    return value.strip().lower() in {"1", "true", "yes"}


def sha256_file(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def build(input_dir, output_dir, limit_archives=None):
    input_dir, output_dir = Path(input_dir), Path(output_dir)
    archives = sorted(input_dir.glob("*_form345.zip"))
    if limit_archives:
        archives = archives[:limit_archives]
    if not archives:
        raise FileNotFoundError(f"no *_form345.zip archives in {input_dir}")
    output_dir.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix=".form4_build_", dir=output_dir) as temp_dir:
        db = sqlite3.connect(Path(temp_dir) / "form4.sqlite")
        db.execute(
            """CREATE TABLE raw_tx (
                issuer TEXT, ticker TEXT, owner TEXT, owner_count INTEGER,
                filing_date TEXT, trans_date TEXT, code TEXT, shares REAL, price REAL,
                tenb5 INTEGER, flag_source TEXT, accession TEXT, month INTEGER, year INTEGER
            )"""
        )
        db.execute(
            """CREATE TABLE routine_years (
                issuer TEXT, owner TEXT, month INTEGER, year INTEGER, filing_date TEXT,
                PRIMARY KEY (issuer, owner, month, year, filing_date)
            )"""
        )
        for archive_path in archives:
            with zipfile.ZipFile(archive_path) as archive:
                submissions = {}
                for row in table_rows(archive, "SUBMISSION.tsv"):
                    accession = row.get("ACCESSION_NUMBER", "")
                    doc_type = row.get("DOCUMENT_TYPE", "").strip().upper()
                    filed = parse_sec_date(row.get("FILING_DATE"))
                    issuer = cik(row.get("ISSUERCIK"))
                    sym = ticker(row.get("ISSUERTRADINGSYMBOL"))
                    if accession and doc_type.startswith("4") and filed and sym:
                        submissions[accession] = {
                            "filing_date": filed,
                            "issuer": issuer,
                            "ticker": sym,
                            "doc_type": doc_type,
                            "checkbox": direct_plan_flag(row),
                            "remarks": row.get("REMARKS", ""),
                        }
                owners = {}
                for row in table_rows(archive, "REPORTINGOWNER.tsv"):
                    accession = row.get("ACCESSION_NUMBER", "")
                    owner = cik(row.get("RPTOWNERCIK"))
                    if accession and owner != "0":
                        owners.setdefault(accession, set()).add(owner)
                linked_plan_notes = {}
                filing_plan_notes = {}
                for row in table_rows(archive, "FOOTNOTES.tsv"):
                    text = row.get("FOOTNOTE_TXT", "")
                    if mentions_10b5_plan(text):
                        key = (row.get("ACCESSION_NUMBER", ""), row.get("FOOTNOTE_ID", ""))
                        linked_plan_notes[key] = True
                        filing_plan_notes.setdefault(key[0], True)

                batch = []
                for row in table_rows(archive, "NONDERIV_TRANS.tsv"):
                    accession = row.get("ACCESSION_NUMBER", "")
                    submission = submissions.get(accession)
                    code = row.get("TRANS_CODE", "").strip().upper()
                    if submission is None or code not in {"P", "S"}:
                        continue
                    trans_date = parse_sec_date(row.get("TRANS_DATE"))
                    if trans_date is None:
                        continue
                    date = datetime.strptime(trans_date, "%Y-%m-%d").date()
                    checkbox = submission["checkbox"]
                    linked = any(
                        linked_plan_notes.get((accession, value.strip()), False)
                        for key, value in row.items()
                        if key.endswith("_FN") and value and value.strip()
                    )
                    remarked = mentions_10b5_plan(submission["remarks"])
                    if checkbox is not None:
                        plan, source = checkbox, "checkbox"
                    elif linked:
                        plan, source = True, "footnote"
                    elif remarked or filing_plan_notes.get(accession, False):
                        plan, source = True, "remarks_or_filing_footnote"
                    else:
                        plan, source = False, "not_disclosed"

                    filing_owners = sorted(owners.get(accession, ()))
                    for owner in filing_owners:
                        batch.append(
                            (
                                submission["issuer"],
                                submission["ticker"],
                                owner,
                                len(filing_owners),
                                submission["filing_date"],
                                trans_date,
                                code,
                                number(row.get("TRANS_SHARES")),
                                number(row.get("TRANS_PRICEPERSHARE")),
                                int(plan),
                                source,
                                accession,
                                date.month,
                                date.year,
                            )
                        )
                        if code == "P":
                            db.execute(
                                "INSERT OR IGNORE INTO routine_years VALUES (?, ?, ?, ?, ?)",
                                (
                                    submission["issuer"],
                                    owner,
                                    date.month,
                                    date.year,
                                    submission["filing_date"],
                                ),
                            )
                        if len(batch) >= 5000:
                            db.executemany(
                                "INSERT INTO raw_tx VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                batch,
                            )
                            batch.clear()
                if batch:
                    db.executemany(
                        "INSERT INTO raw_tx VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        batch,
                    )
                db.commit()
            print(f"read {archive_path.name}", flush=True)

        db.execute(
            """CREATE TABLE owner_events (
                ticker TEXT, issuer TEXT, owner TEXT, filing_date TEXT,
                buy_eligible INTEGER, buy_prior_years INTEGER, has_sell INTEGER,
                tenb5_seen INTEGER, tenb5_sources TEXT, joint_owner_filing INTEGER,
                accession TEXT,
                PRIMARY KEY (ticker, issuer, owner, filing_date)
            )"""
        )
        histories = {}
        for issuer, owner, month, year, filed in db.execute(
            "SELECT issuer, owner, month, year, filing_date FROM routine_years"
        ):
            histories.setdefault((issuer, owner, month), []).append((year, filed))

        cursor = db.execute(
            """SELECT issuer, ticker, owner, owner_count, filing_date, trans_date, code, tenb5,
                      flag_source, accession, month, year
               FROM raw_tx ORDER BY filing_date, ticker, owner"""
        )
        for issuer, sym, owner, owner_count, filed, trans, code, plan, source, accession, month, year in cursor:
            years = histories.get((issuer, owner, month), ())
            prior = 0
            for offset in range(1, 5):
                prior_year = year - offset
                if any(
                    prior_year == history_year and history_filed <= filed
                    for history_year, history_filed in years
                ):
                    prior += 1
                else:
                    break
            buy = int(code == "P" and not plan and owner_count == 1)
            buy_prior = prior if buy else None
            has_sell = int(code == "S")
            joint = int(owner_count > 1)
            db.execute(
                """INSERT INTO owner_events VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                   ON CONFLICT(ticker, issuer, owner, filing_date) DO UPDATE SET
                     buy_prior_years = CASE
                       WHEN excluded.buy_eligible = 1 AND owner_events.buy_eligible = 1
                         THEN MIN(owner_events.buy_prior_years, excluded.buy_prior_years)
                       WHEN excluded.buy_eligible = 1 THEN excluded.buy_prior_years
                       ELSE owner_events.buy_prior_years END,
                     buy_eligible = MAX(owner_events.buy_eligible, excluded.buy_eligible),
                     has_sell = MAX(owner_events.has_sell, excluded.has_sell),
                     tenb5_seen = MAX(owner_events.tenb5_seen, excluded.tenb5_seen),
                     joint_owner_filing = MAX(owner_events.joint_owner_filing, excluded.joint_owner_filing),
                     tenb5_sources = CASE
                       WHEN excluded.tenb5_seen = 1 THEN excluded.tenb5_sources
                       WHEN owner_events.tenb5_sources = '' THEN excluded.tenb5_sources
                       ELSE owner_events.tenb5_sources END""",
                (
                    sym, issuer, owner, filed, buy, buy_prior, has_sell, int(plan),
                    source, joint, accession,
                ),
            )
        db.commit()

        output_file = output_dir / "form4_owner_events.csv"
        symbols_file = output_dir / "form4_symbols.csv"
        with output_file.open("w", encoding="utf-8", newline="") as out:
            writer = csv.writer(out)
            writer.writerow(
                [
                    "ticker", "issuer_cik", "owner_cik", "filing_date", "buy_eligible",
                    "buy_prior_same_month_years", "has_sell", "tenb5_seen", "tenb5_sources",
                    "joint_owner_filing", "accession",
                ]
            )
            rows = db.execute(
                """SELECT ticker, issuer, owner, filing_date, buy_eligible, buy_prior_years,
                          has_sell, tenb5_seen, tenb5_sources, joint_owner_filing, accession
                   FROM owner_events ORDER BY ticker, filing_date, owner"""
            )
            count = 0
            for row in rows:
                writer.writerow(row)
                count += 1
        with symbols_file.open("w", encoding="utf-8", newline="") as out:
            writer = csv.writer(out)
            writer.writerow(["ticker"])
            for (sym,) in db.execute("SELECT DISTINCT ticker FROM owner_events ORDER BY ticker"):
                writer.writerow([sym])

        digest = sha256_file(output_file)
        manifest = {
            "source": "SEC Insider Transactions Data Sets, as-filed quarterly Form 4 TSVs",
            "archives": len(archives),
            "first_archive": archives[0].name,
            "last_archive": archives[-1].name,
            "owner_events": count,
            "event_file": str(output_file),
            "symbols_file": str(symbols_file),
            "event_sha256": digest,
            "10b5_1": "AFF10B5ONE when present; otherwise disclosed footnote/remarks regex only (pre-2023 disclosure is voluntary)",
            "filing_time": "FILING_DATE; no acceptance time in bulk TSV",
            "joint_filings": "purchases in filings with multiple reporting owners are excluded because transaction rows do not identify the reporting owner",
        }
        (output_dir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
        db.close()
    print(f"wrote {count} owner-day rows to {output_file}")
    print(f"wrote {symbols_file}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input-dir", default=str(DEFAULT_INPUT))
    parser.add_argument("--output-dir", default=str(DEFAULT_OUTPUT))
    parser.add_argument("--limit-archives", type=int, help="parse only the first N archives for smoke tests")
    args = parser.parse_args()
    build(args.input_dir, args.output_dir, args.limit_archives)


if __name__ == "__main__":
    main()
