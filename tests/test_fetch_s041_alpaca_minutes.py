import importlib.util
import unittest
from datetime import datetime, timezone
from pathlib import Path
from unittest.mock import Mock


PATH = Path(__file__).resolve().parents[1] / "tools" / "fetch_s041_alpaca_minutes.py"
SPEC = importlib.util.spec_from_file_location("fetch_s041_alpaca_minutes", PATH)
assert SPEC is not None and SPEC.loader is not None
fetch = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fetch)


class AlpacaMinuteTests(unittest.TestCase):
    def setUp(self):
        self.start = datetime(2024, 1, 2, tzinfo=timezone.utc)
        self.end = datetime(2024, 1, 3, tzinfo=timezone.utc)
        self.bar = {"t": "2024-01-02T14:30:00Z", "o": 10, "h": 11, "l": 9, "c": 10, "v": 20}

    def test_pagination_and_raw_iex_request(self):
        first, second = Mock(), Mock()
        first.json.return_value = {"bars": {"A": [self.bar]}, "next_page_token": "next"}
        second.json.return_value = {"bars": {"A": [self.bar]}, "next_page_token": None}
        session = Mock()
        session.get.side_effect = [first, second]
        result = fetch.fetch_bars(session, ["A"], self.start, self.end)
        self.assertEqual(len(result["A"]), 2)
        self.assertEqual(session.get.call_args.kwargs["params"]["feed"], "iex")
        self.assertEqual(session.get.call_args.kwargs["params"]["adjustment"], "raw")

    def test_invalid_or_duplicate_bar_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "duplicate"):
            fetch.normalize("A", [self.bar, self.bar], self.start, self.end)
        bad = {**self.bar, "h": 8}
        with self.assertRaisesRegex(ValueError, "inconsistent"):
            fetch.normalize("A", [bad], self.start, self.end)

    def test_repeated_pagination_token_is_rejected(self):
        response = Mock()
        response.json.return_value = {"bars": {}, "next_page_token": "repeated"}
        session = Mock()
        session.get.return_value = response
        with self.assertRaisesRegex(ValueError, "repeated"):
            fetch.fetch_bars(session, ["A"], self.start, self.end)


if __name__ == "__main__":
    unittest.main()
