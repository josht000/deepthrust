import importlib.util
import unittest
from pathlib import Path

import pandas as pd


PATH = Path(__file__).resolve().parents[1] / "tools" / "s041_frequency.py"
SPEC = importlib.util.spec_from_file_location("s041_frequency", PATH)
assert SPEC is not None and SPEC.loader is not None
frequency = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(frequency)


class FrequencyTests(unittest.TestCase):
    def test_counts_entries_exits_and_rebalances_in_each_partition(self):
        sessions = pd.to_datetime(["2023-12-29", "2024-01-02", "2024-01-03"])
        trades = pd.DataFrame(
            [
                {"entry_date": "2023-12-29", "exit_date": "2024-01-02", "exit_reason": "rebalance"},
                {"entry_date": "2024-01-02", "exit_date": "2024-01-03", "exit_reason": "censored"},
            ]
        )
        dev, test = frequency.count_frequency(sessions, trades, "2024-01-01")
        self.assertEqual((dev["entry_legs"], dev["completed_legs"], dev["rebalance_dates"]), (1, 0, 1))
        self.assertEqual((test["entry_legs"], test["completed_legs"], test["rebalance_dates"]), (1, 1, 1))
        self.assertEqual(test["entry_legs_per_session"], 0.5)

    def test_rejects_reversed_trade(self):
        trades = pd.DataFrame([{"entry_date": "2024-01-02", "exit_date": "2023-12-29", "exit_reason": "rebalance"}])
        with self.assertRaisesRegex(ValueError, "invalid entry or exit"):
            frequency.count_frequency(["2023-12-29", "2024-01-02"], trades, "2024-01-01")


if __name__ == "__main__":
    unittest.main()
