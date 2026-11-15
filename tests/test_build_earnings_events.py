import csv
import tempfile
import unittest
from datetime import date
from pathlib import Path

from tools.build_earnings_events import (
    extract_sec_events,
    filter_company_map,
    read_company_map,
    read_rank_cache_symbols,
    read_simfin_events,
    release_timing,
    write_events,
)


class EarningsEventBuilderTests(unittest.TestCase):
    def test_company_map_normalizes_cik_and_keeps_multiple_share_classes(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "companies.csv"
            path.write_text(
                "Ticker;CIK\nABC;12345\nABC.A;0000012345\n;98765\n",
                encoding="utf-8",
            )
            self.assertEqual(read_company_map(path), {"0000012345": {"ABC", "ABC.A"}})

    def test_reference_parquet_maps_historical_symbols_to_cik(self):
        import pyarrow as pa
        import pyarrow.parquet as parquet

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "reference.parquet"
            parquet.write_table(
                pa.table(
                    {
                        "ticker": ["OLD", "NEW", "NO_CIK"],
                        "cik": ["0000000001", "0000000001", None],
                    }
                ),
                path,
            )
            mapping = read_company_map(path)
            self.assertEqual(mapping, {"0000000001": {"OLD", "NEW"}})
            self.assertEqual(
                filter_company_map(mapping, {"OLD"}),
                {"0000000001": {"OLD"}},
            )

    def test_rank_cache_symbols_are_filtered_by_historical_cutoff(self):
        import numpy as np

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "index.json"
            path.write_text(
                '{"dates":["2015-12-31","2016-01-04","2016-01-05"],'
                '"symbols":["A","B","C"]}',
                encoding="utf-8",
            )
            np.save(
                Path(directory) / "ranks.npy",
                np.array([[1, 1, 1], [501, 2, np.nan], [600, 510, 3]], dtype=np.float32),
            )
            symbols = read_rank_cache_symbols(
                path, date(2016, 1, 1), date(2016, 12, 31), 500
            )
            self.assertEqual(symbols, {"B", "C"})

    def test_simfin_events_use_publish_date_and_window(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "income.csv"
            with path.open("w", newline="", encoding="utf-8") as target:
                writer = csv.writer(target, delimiter=";")
                writer.writerow(["Ticker", "Publish Date"])
                writer.writerow(["abc", "2015-12-31"])
                writer.writerow(["abc", "2016-01-04"])
            events = read_simfin_events(path, date(2016, 1, 1), date(2016, 12, 31))
            self.assertEqual(len(events), 1)
            self.assertEqual(events[0]["ticker"], "ABC")
            self.assertEqual(events[0]["date"], "2016-01-04")
            self.assertEqual(events[0]["source"], "simfin_publish_date")

    def test_sec_events_require_item_202_and_preserve_release_timing(self):
        payload = {
            "filings": {
                "recent": {
                    "form": ["8-K", "8-K", "10-Q", "8-K/A"],
                    "filingDate": [
                        "2016-01-04",
                        "2016-01-05",
                        "2016-01-06",
                        "2016-01-07",
                    ],
                    "acceptanceDateTime": [
                        "2016-01-04T13:00:00.000Z",
                        "2016-01-05T21:30:00.000Z",
                        "2016-01-06T12:00:00.000Z",
                        "2016-01-07T18:00:00.000Z",
                    ],
                    "items": ["2.02,9.01", "7.01,9.01", "", "2.02"],
                    "accessionNumber": ["a", "b", "c", "d"],
                }
            }
        }
        events = extract_sec_events(
            [payload],
            {"ABC", "ABC.A"},
            date(2016, 1, 1),
            date(2016, 1, 31),
        )
        self.assertEqual(len(events), 4)
        self.assertEqual({event["ticker"] for event in events}, {"ABC", "ABC.A"})
        first = next(event for event in events if event["date"] == "2016-01-04")
        self.assertEqual(first["release_timing"], "before_market")
        amendment = next(event for event in events if event["date"] == "2016-01-07")
        self.assertEqual(amendment["release_timing"], "intraday")

    def test_release_timing_uses_eastern_dst_boundary(self):
        from datetime import datetime, timezone

        before_market = datetime(2024, 3, 11, 13, 0, tzinfo=timezone.utc)
        after_market = datetime(2024, 3, 11, 20, 0, tzinfo=timezone.utc)
        self.assertEqual(release_timing(before_market), "before_market")
        self.assertEqual(release_timing(after_market), "after_market")

    def test_writer_emits_semicolon_csv_and_coverage_manifest(self):
        import json

        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "earnings_events.csv"
            events = [
                {
                    "ticker": "ABC",
                    "date": "2016-01-04",
                    "source": "sec_8k_item_2.02",
                    "release_timing": "before_market",
                    "acceptance_utc": "2016-01-04T13:00:00Z",
                }
            ]
            write_events(
                events,
                output,
                date(2016, 1, 1),
                date(2016, 12, 31),
                1,
                500,
                {"ABC", "OLD"},
                {"ABC"},
            )
            with output.open(newline="", encoding="utf-8") as source:
                records = list(csv.DictReader(source, delimiter=";"))
            manifest = json.loads(
                output.with_suffix(output.suffix + ".manifest.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(records[0]["ticker"], "ABC")
            self.assertEqual(manifest["universe_top_n"], 500)
            self.assertEqual(manifest["unmapped_symbols"], ["OLD"])


if __name__ == "__main__":
    unittest.main()
