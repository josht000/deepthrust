import csv
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools import hmm_pca_results as report


def write_row(path, row):
    with path.open("w", newline="", encoding="utf-8") as file:
        writer = csv.DictWriter(file, fieldnames=row)
        writer.writeheader()
        writer.writerow(row)


class PcaReportTest(unittest.TestCase):
    def test_selects_eligible_dev_before_reading_only_its_test(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            results = root / "results"
            results.mkdir()
            doc = root / "ablation.md"
            doc.write_text("before\n## PCA Exp Results\nold\n## XGBoost state-classification experiments\nafter\n")
            dev = {
                "fill": "next_open", "period": "dev", "trades": "20",
                "sharpe": "0.5", "pf": "1.5", "win_pct": "50",
                "trend": "0.1", "random_pick_universe": "0.1", "bias": "0",
                "skill": "0.1", "skill_pct": "0.25", "total_log": "0.4",
                "max_dd_pct": "-5",
            }
            diagnostics = {
                "oos_ll_per_bar": "-5", "persist": "0.9", "flip_rate": "0.1",
                "map_churn": "2", "refits": "10", "mean_conf": "0.8",
            }
            selected = "pca_b2_diag"
            for name in report.NAMES:
                row = dev.copy()
                if name == selected:
                    row.update(trades="100", sharpe="1.5", pf="1.2")
                if name == "pca_a_full":
                    row.update(trades="99", sharpe="4.0", pf="4.0")
                write_row(results / f"{report.prefix(name)}partition_dev.csv", row)
                features, components, covariance = report.expected_config(name)
                write_row(results / f"{report.prefix(name)}hmm_metrics.csv", diagnostics | {
                    "features": features, "pca_components": components, "covariance": covariance,
                })
            for name in ("baseline", selected):
                row = dev | {"period": "test", "trades": "30", "skill": "-0.1"}
                write_row(results / f"{report.prefix(name)}partition_test.csv", row)
                path = results / f"{report.prefix(name)}baselines.csv"
                with path.open("w", newline="", encoding="utf-8") as file:
                    writer = csv.DictWriter(file, fieldnames=["baseline", "period", "total_log"])
                    writer.writeheader()
                    writer.writerows([
                        {"baseline": "ew_buy_hold", "period": "dev", "total_log": "0.1"},
                        {"baseline": "ew_buy_hold", "period": "test", "total_log": "0.1"},
                    ])
            with patch.object(report, "RESULTS", results), patch.object(report, "DOC", doc):
                report.main()
            text = doc.read_text()
            self.assertIn("DEV-selected winner: `pca_b2_diag`", text)
            self.assertIn("25.0", text)
            self.assertIn("DECAYED", text)
            self.assertIn("## XGBoost state-classification experiments\nafter", text)
            with (results / "pca_dev_comparison.csv").open(newline="") as file:
                rows = list(csv.DictReader(file))
            self.assertEqual(len(rows), 34)
            self.assertEqual(rows[0]["variant"], "pca_a_full")
            self.assertEqual(rows[0]["eligible"], "False")


if __name__ == "__main__":
    unittest.main()
