import csv
from pathlib import Path


HEADER = [
    "study",
    "variant",
    "win_pct",
    "mean_ret_pct",
    "pf",
    "sharpe",
    "max_dd_pct",
    "fill",
    "period",
    "days",
    "trades",
    "total_log",
    "trend",
    "trend_beta",
    "beta",
    "beta_t",
    "avg_fraction_invested",
    "avg_positions",
    "turnover_annual",
    "bias",
    "cagr_pct",
    "avg_hold",
    "random_pick_universe",
    "random_p_universe",
    "skill_vs_universe",
    "random_pick_pool",
    "random_p_pool",
    "skill_vs_pool",
    "skill",
    "skill_pct",
    "skill_pct_random",
    "skill_lo",
    "skill_hi",
    "skill_p",
]


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    results = root / "results"
    output = results / "all_trade_metrics.csv"
    rows = []

    for source in sorted(results.glob("*/*partition_*.csv")):
        name = source.name
        if name.startswith("partition_"):
            variant = "baseline"
            period = name.removeprefix("partition_").removesuffix(".csv")
        elif "__partition_" in name:
            variant, suffix = name.split("__partition_", 1)
            period = suffix.removesuffix(".csv")
        else:
            continue
        if period not in {"dev", "test"}:
            continue

        with source.open(newline="", encoding="utf-8-sig") as file:
            reader = csv.DictReader(file)
            missing = set(HEADER[2:]) - set(reader.fieldnames or ())
            if missing:
                raise ValueError(f"{source} is missing columns: {', '.join(sorted(missing))}")
            for record in reader:
                if record["period"] != period:
                    raise ValueError(f"{source} contains unexpected period {record['period']!r}")
                rows.append(
                    {
                        "study": source.parent.name,
                        "variant": variant,
                        **{column: record[column] for column in HEADER[2:]},
                    }
                )

    if not rows:
        raise RuntimeError(f"No partition metrics found under {results}")

    rows.sort(key=lambda row: (row["study"], row["variant"], row["fill"], row["period"]))
    temporary = output.with_suffix(".csv.tmp")
    try:
        with temporary.open("w", newline="", encoding="utf-8") as file:
            writer = csv.DictWriter(file, fieldnames=HEADER, extrasaction="raise")
            writer.writeheader()
            writer.writerows(rows)
        temporary.replace(output)
    finally:
        temporary.unlink(missing_ok=True)

    print(f"Updated {output.relative_to(root)} with {len(rows)} rows.")


if __name__ == "__main__":
    main()
