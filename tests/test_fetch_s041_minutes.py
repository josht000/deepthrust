import unittest
from datetime import date
from pathlib import Path
from unittest.mock import patch
import importlib.util

import pandas as pd


PATH = Path(__file__).resolve().parents[1] / "tools" / "fetch_s041_minutes.py"
SPEC = importlib.util.spec_from_file_location("fetch_s041_minutes", PATH)
assert SPEC is not None and SPEC.loader is not None
fetch = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fetch)


class MinuteDataTests(unittest.TestCase):
    def bars(self):
        return pd.DataFrame(
            {"Open": [10.0], "High": [11.0], "Low": [9.0], "Close": [10.5], "Volume": [100]},
            index=pd.DatetimeIndex(["2026-10-05 09:30"], tz="America/New_York"),
        )

    def test_windows_are_nonoverlapping_and_at_most_seven_days(self):
        self.assertEqual(
            list(fetch.windows(date(2026, 10, 5), date(2026, 10, 15))),
            [(date(2026, 10, 5), date(2026, 10, 12)), (date(2026, 10, 12), date(2026, 10, 15))],
        )

    def test_valid_bar_uses_utc(self):
        bars = fetch.validate_bars(self.bars(), date(2026, 10, 5), date(2026, 10, 6))
        self.assertEqual(bars.index[0].isoformat(), "2026-10-05T13:30:00+00:00")

    def test_bad_ohlc_and_duplicates_are_rejected(self):
        bars = self.bars()
        bars["High"] = 9.0
        with self.assertRaisesRegex(ValueError, "inconsistent OHLC"):
            fetch.validate_bars(bars, date(2026, 10, 5), date(2026, 10, 6))
        bars = pd.concat([self.bars(), self.bars()])
        with self.assertRaisesRegex(ValueError, "unique"):
            fetch.validate_bars(bars, date(2026, 10, 5), date(2026, 10, 6))

    def test_provider_failure_does_not_create_empty_data(self):
        with patch.object(fetch.yf, "Ticker") as ticker:
            ticker.return_value.history.return_value = pd.DataFrame()
            with self.assertRaisesRegex(ValueError, "no bars"):
                fetch.fetch_symbol("SPY", date(2026, 10, 5), date(2026, 10, 6), "1m")

    def test_fetch_records_requested_window_and_interval(self):
        with patch.object(fetch.yf, "Ticker") as ticker:
            ticker.return_value.history.return_value = self.bars()
            bars, coverage = fetch.fetch_symbol(
                "SPY", date(2026, 10, 5), date(2026, 10, 6), "2m"
            )
            self.assertEqual(len(bars), 1)
            self.assertEqual(coverage, [{"start": "2026-10-05", "end_exclusive": "2026-10-06", "rows": 1}])
            self.assertEqual(ticker.return_value.history.call_args.kwargs["interval"], "2m")


if __name__ == "__main__":
    unittest.main()
