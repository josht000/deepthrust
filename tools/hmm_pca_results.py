"""Summarize the HMM PCA matrix; select on DEV before reading TEST."""

import csv
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RESULTS = ROOT / "results" / "hmm_regime"
DOC = ROOT / "docs" / "strategies" / "markov" / "hmm_ablation.md"
NAMES = (
    ["baseline", "pca_a_full"]
    + [f"pca_b{n}_{cov}" for cov in ("diag", "full") for n in range(2, 7)]
    + [f"pca_c_{cov}" for cov in ("diag", "full")]
    + [f"pca_d{n}_{cov}" for cov in ("diag", "full") for n in range(2, 6)]
    + [f"pca_e_{cov}" for cov in ("diag", "full")]
    + [f"pca_f{n}_{cov}" for cov in ("diag", "full") for n in range(2, 7)]
)


def read_one(path: Path, fill: str | None = None) -> dict[str, str]:
    with path.open(newline="", encoding="utf-8-sig") as file:
        rows = list(csv.DictReader(file))
    if fill is not None:
        rows = [row for row in rows if row["fill"] == fill]
    if len(rows) != 1:
        raise ValueError(f"{path}: expected one matching row, found {len(rows)}")
    return rows[0]


def prefix(name: str) -> str:
    return "" if name == "baseline" else f"{name}__"


def diagnostics_for(name: str) -> dict[str, str]:
    path = RESULTS / f"{prefix(name)}hmm_metrics.csv"
    row = read_one(path)
    if name == "baseline" and "pca_components" not in row:
        return read_one(RESULTS / "pca_baseline__hmm_metrics.csv")
    return row


def expected_config(name: str) -> tuple[str, str, str]:
    if name == "baseline":
        group, components, covariance = "a", "0", "diagonal"
    else:
        _, token, cov = name.split("_")
        group, components = token[0], token[1:] or "0"
        covariance = {"diag": "diagonal", "full": "full"}[cov]
    features = {
        "a": "ret;rv;atr;volchg;skew;ma",
        "b": "ret;rv;atr;volchg;skew;ma",
        "c": "ret;rv;atr;volchg;ma",
        "d": "ret;rv;atr;volchg;ma",
        "e": "ret;rv;atr;volchg;ma;spy_ret",
        "f": "ret;rv;atr;volchg;ma;spy_ret",
    }[group]
    return features, components, covariance


def number(row: dict[str, str], key: str) -> float:
    value = float(row[key])
    if math.isnan(value):
        raise ValueError(f"NaN {key} in {row}")
    return value


def optional_number(row: dict[str, str], key: str) -> float | None:
    value = row[key].strip()
    if not value:
        return None
    result = float(value)
    return None if math.isnan(result) else result


def fmt(row: dict[str, str], key: str, digits: int = 3) -> str:
    value = row[key].strip()
    if not value:
        return "n/a"
    x = float(value)
    if not math.isfinite(x):
        return "n/a" if math.isnan(x) else ("inf" if x > 0 else "-inf")
    return f"{x:.{digits}f}"


def skill_percent(row: dict[str, str]) -> str:
    value = row["skill_pct"].strip()
    if not value or not math.isfinite(float(value)):
        return "n/a"
    return f"{100 * float(value):.1f}"


def benchmark_totals(name: str) -> dict[str, float]:
    path = RESULTS / f"{prefix(name)}baselines.csv"
    with path.open(newline="", encoding="utf-8-sig") as file:
        rows = [r for r in csv.DictReader(file) if r["baseline"] == "ew_buy_hold"]
    totals = {row["period"]: number(row, "total_log") for row in rows}
    if len(rows) != 2 or set(totals) != {"dev", "test"}:
        raise ValueError(f"{path}: expected DEV and TEST buy-and-hold baselines")
    return totals


def main() -> None:
    dev = {
        name: read_one(RESULTS / f"{prefix(name)}partition_dev.csv", "next_open")
        for name in NAMES
    }
    eligible = [
        name for name, row in dev.items()
        if number(row, "trades") >= 100
        and (optional_number(row, "sharpe") is not None and number(row, "sharpe") > 1)
        and (optional_number(row, "pf") is not None and number(row, "pf") > 1)
    ]
    ordered = sorted(
        NAMES,
        key=lambda name: optional_number(dev[name], "sharpe")
        if optional_number(dev[name], "sharpe") is not None else -math.inf,
        reverse=True,
    )
    if optional_number(dev[ordered[0]], "sharpe") is None:
        raise ValueError("No experiment has a defined DEV Sharpe")
    selected = max(eligible, key=lambda name: number(dev[name], "sharpe")) if eligible else ordered[0]
    diagnostics = {name: diagnostics_for(name) for name in NAMES}
    for name, row in diagnostics.items():
        actual = (row["features"], row["pca_components"], row["covariance"])
        expected = expected_config(name)
        if actual != expected:
            raise ValueError(f"{name}: HMM configuration {actual} does not match {expected}")
    test = {
        name: read_one(RESULTS / f"{prefix(name)}partition_test.csv", "next_open")
        for name in dict.fromkeys(("baseline", selected))
    }
    comparison = RESULTS / "pca_dev_comparison.csv"
    fields = ["variant", "rank", "eligible"] + list(next(iter(dev.values())))
    fields += [f"hmm_{key}" for key in next(iter(diagnostics.values()))]
    with comparison.open("w", newline="", encoding="utf-8") as file:
        writer = csv.DictWriter(file, fieldnames=fields)
        writer.writeheader()
        for rank, name in enumerate(ordered, 1):
            writer.writerow({
                "variant": name, "rank": rank, "eligible": name in eligible,
                **dev[name], **{f"hmm_{key}": val for key, val in diagnostics[name].items()},
            })

    choice = "DEV-selected winner" if eligible else "Best DEV Sharpe, exploratory (no eligible winner)"
    lines = [
        "## PCA Exp Results",
        "",
        "QQQ, next-open, 1 bp per side. SPY is used for market features and charts;",
        "the partition trend/skill benchmark is QQQ buy-and-hold, not SPY, because",
        "SPY is not in the tradable panel. All 34 configurations use the fixed",
        "three-state walk-forward HMM and four restarts. PCA-A diagonal is `baseline`.",
        "The table is ranked by DEV Sharpe; eligibility requires DEV Sharpe > 1,",
        "profit factor > 1, and at least 100 trades. TEST is shown only for the",
        "baseline and the DEV-selected candidate. TEST has been inspected in earlier",
        "experiments and is not a fresh holdout.",
        "",
        f"**{choice}: `{selected}`.**",
        "",
        "| DEV rank | Experiment | PF | Sharpe | Trades | Win % | Trend | Random pick | Bias | Skill | Skill % | OOS LL/bar | Persistence | Flip rate | Map churn / confidence |",
        "|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|",
    ]
    for rank, name in enumerate(ordered, 1):
        r, d = dev[name], diagnostics[name]
        churn = number(d, "map_churn") / max(number(d, "refits"), 1)
        lines.append(
            f"| {rank} | {name} | {fmt(r, 'pf', 2)} | {fmt(r, 'sharpe', 2)} | "
            f"{r['trades']} | {fmt(r, 'win_pct', 1)} | {fmt(r, 'trend')} | "
            f"{fmt(r, 'random_pick_universe')} | {fmt(r, 'bias')} | {fmt(r, 'skill')} | "
            f"{skill_percent(r)} | {fmt(d, 'oos_ll_per_bar', 2)} | "
            f"{fmt(d, 'persist')} | {fmt(d, 'flip_rate')} | "
            f"{churn:.0%} / {fmt(d, 'mean_conf', 2)} |"
        )
    lines += [
        "",
        "OOS likelihood and regime diagnostics summarize frozen-model segments across",
        "the full timeline; they were not used for DEV selection. Map churn compares",
        "arbitrary state IDs across refits and is not label-invariant. `n/a` denotes",
        "an undefined statistic; skill % requires positive total and nonnegative skill.",
        "",
        "| Candidate | Period | Total log | Trend | Random pick | Skill | Bias | Skill % | Sharpe | PF | Trades | Win % | Max DD % |",
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for name in dict.fromkeys(("baseline", selected)):
        for period, row in (("dev", dev[name]), ("test", test[name])):
            lines.append(
                f"| {name} | {period} | {fmt(row, 'total_log')} | {fmt(row, 'trend')} | "
                f"{fmt(row, 'random_pick_universe')} | {fmt(row, 'skill')} | "
                f"{fmt(row, 'bias')} | {skill_percent(row)} | {fmt(row, 'sharpe', 2)} | "
                f"{fmt(row, 'pf', 2)} | {row['trades']} | {fmt(row, 'win_pct', 1)} | "
                f"{fmt(row, 'max_dd_pct', 1)} |"
            )
    for name in dict.fromkeys(("baseline", selected)):
        d, t = dev[name], test[name]
        benchmarks = benchmark_totals(name)
        flags = []
        if number(d, "trades") < 100 or number(t, "trades") < 30:
            flags.append("insufficient evidence")
        if number(d, "skill") * number(t, "skill") < 0:
            flags.append("DECAYED")
        if any(
            value is not None and value > 2
            for row in (d, t) for key in ("sharpe", "pf")
            for value in (optional_number(row, key),)
        ):
            flags.append("Sharpe or PF > 2: review for leakage")
        if flags:
            lines += [f"- `{name}`: {', '.join(flags)}."]
        for period, row in (("dev", d), ("test", t)):
            excess = math.expm1(number(row, "total_log")) - math.expm1(benchmarks[period])
            if excess >= 0.10:
                lines += [f"- `{name}` {period}: outperformed QQQ buy-and-hold by {100 * excess:.1f} percentage points."]
    lines += [
        "",
        "Evidence is insufficient for any candidate with fewer than 100 DEV or 30 TEST",
        "trades. See the candidate reports for cost ladders and class gates; a change",
        "in the sign of skill between DEV and TEST is DECAYED.",
        "",
        "Reproduce: `scripts\\run_hmm_pca.bat`",
        "",
    ]
    report = "\n".join(lines)
    (RESULTS / "pca_report.md").write_text(report + "\n", encoding="utf-8")
    doc = DOC.read_text(encoding="utf-8")
    start = doc.index("## PCA Exp Results")
    end = doc.index("## XGBoost state-classification experiments", start)
    DOC.write_text(doc[:start] + report + "\n" + doc[end:], encoding="utf-8")
    print(f"{choice}: {selected}; wrote {comparison} and {DOC}")


if __name__ == "__main__":
    main()
