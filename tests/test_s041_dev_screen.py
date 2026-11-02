import importlib.util
import unittest
from pathlib import Path

import pandas as pd


PATH = Path(__file__).resolve().parents[1] / "tools" / "s041_dev_screen.py"
SPEC = importlib.util.spec_from_file_location("s041_dev_screen", PATH)
assert SPEC is not None and SPEC.loader is not None
dev_screen = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(dev_screen)


class DevScreenTests(unittest.TestCase):
    def test_drawdown_includes_previous_close(self):
        idx = pd.DatetimeIndex(["2024-01-02", "2024-01-03", "2024-01-04"], tz="America/New_York")
        prices = pd.Series([100.0, 80.0, 90.0], index=idx)
        self.assertAlmostEqual(dev_screen.benchmark_drawdown(prices, pd.to_datetime(["2024-01-03", "2024-01-04"])), -20.0)

    def test_dev_only_gate_and_balanced_score(self):
        df = pd.DataFrame(
            [
                {"variant": "a", "dev_skill": 0.1, "dev_sharpe": 1.0, "dev_pf": 2.0, "dev_trades": 100, "dev_max_dd_pct": -10.0, "test_skill": -999},
                {"variant": "b", "dev_skill": 0.2, "dev_sharpe": 2.0, "dev_pf": 3.0, "dev_trades": 200, "dev_max_dd_pct": -20.0, "test_skill": 999},
                {"variant": "c", "dev_skill": 0.9, "dev_sharpe": 4.0, "dev_pf": 4.0, "dev_trades": 300, "dev_max_dd_pct": -26.0, "test_skill": 999},
            ]
        )
        result = dev_screen.screen(df, 25.0)
        self.assertEqual(list(result.variant), ["b", "a", "c"])
        self.assertEqual(list(result.eligible), [True, True, False])
        self.assertIn("drawdown", result.iloc[2].ineligibility)
        self.assertNotIn("test_skill", result.columns)
        changed = df.assign(test_skill=[1e6, -1e6, 0])
        pd.testing.assert_frame_equal(result, dev_screen.screen(changed, 25.0))

    def test_no_candidates_does_not_invent_a_winner(self):
        df = pd.DataFrame(
            [{"variant": "a", "dev_skill": 0.1, "dev_sharpe": 1.0, "dev_pf": 2.0, "dev_trades": 100, "dev_max_dd_pct": -26.0}]
        )
        result = dev_screen.screen(df, 25.0)
        self.assertFalse(result.eligible.any())
        self.assertTrue(result.balanced_dev_score.isna().all())

    def test_zero_drawdown_limit_is_valid(self):
        df = pd.DataFrame(
            [{"variant": "a", "dev_skill": 0.1, "dev_sharpe": 1.0, "dev_pf": 2.0, "dev_trades": 100, "dev_max_dd_pct": 0.0}]
        )
        self.assertTrue(dev_screen.screen(df, 0).eligible.iloc[0])


if __name__ == "__main__":
    unittest.main()
