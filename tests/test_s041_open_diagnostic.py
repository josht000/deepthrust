import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

import pandas as pd


PATH = Path(__file__).resolve().parents[1] / "tools" / "s041_open_diagnostic.py"
SPEC = importlib.util.spec_from_file_location("s041_open_diagnostic", PATH)
assert SPEC is not None and SPEC.loader is not None
diagnostic = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(diagnostic)


class OpeningDiagnosticTests(unittest.TestCase):
    def test_snapshot_requires_valid_hash_and_both_opening_bars(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = root / "A.parquet"
            pd.DataFrame(
                {"Open": [100.0, 101.0]},
                index=pd.DatetimeIndex(
                    ["2026-10-01 09:30", "2026-10-01 09:32"],
                    tz="America/New_York",
                ),
            ).to_parquet(path)
            manifest = {
                "interval": "2m",
                "adjustment": "raw",
                "files": [
                    {
                        "symbol": "A",
                        "file": path.name,
                        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                    }
                ],
            }
            manifest_path = root / "manifest.json"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            self.assertEqual(diagnostic.load_snapshots([root], "2026-10-01"), {"A": (100.0, 101.0)})
            manifest["files"][0]["sha256"] = "wrong"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "hash mismatch"):
                diagnostic.load_snapshots([root], "2026-10-01")
            manifest["files"][0]["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "missing 09:30 or 09:32"):
                diagnostic.load_snapshots([root], "2026-10-02")

    def test_buy_and_sell_moves_have_opposite_signs(self):
        trades = pd.DataFrame(
            [
                {"symbol": "A", "entry_date": "2026-10-01", "exit_date": "2026-11-02", "exit_reason": "rebalance"},
                {"symbol": "B", "entry_date": "2026-09-01", "exit_date": "2026-10-01", "exit_reason": "rebalance"},
            ]
        )
        result = diagnostic.diagnose(trades, {"A": (100.0, 101.0), "B": (100.0, 101.0)}, "2026-10-01")
        self.assertAlmostEqual(result.loc[0, "adverse_move_bps"], 100)
        self.assertAlmostEqual(result.loc[1, "adverse_move_bps"], -100)

    def test_missing_symbol_fails_instead_of_skipping_a_leg(self):
        trades = pd.DataFrame(
            [{"symbol": "A", "entry_date": "2026-10-01", "exit_date": "2026-11-02", "exit_reason": "rebalance"}]
        )
        with self.assertRaisesRegex(ValueError, "missing intraday snapshot"):
            diagnostic.diagnose(trades, {}, "2026-10-01")

    def test_delisted_and_censored_are_not_opening_sell_orders(self):
        trades = pd.DataFrame(
            [{"symbol": "A", "entry_date": "2026-09-01", "exit_date": "2026-10-01", "exit_reason": "delisted"}]
        )
        with self.assertRaisesRegex(ValueError, "no modeled rebalance legs"):
            diagnostic.diagnose(trades, {}, "2026-10-01")


if __name__ == "__main__":
    unittest.main()
