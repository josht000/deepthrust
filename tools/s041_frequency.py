"""Count S041 research trade legs and rebalance dates in DEV and TEST."""

import argparse
from pathlib import Path

import pandas as pd


def count_frequency(sessions, trades, split):
    dates = pd.DatetimeIndex(pd.to_datetime(sessions))
    if dates.empty or dates.has_duplicates or not dates.is_monotonic_increasing:
        raise ValueError("session dates must be nonempty, unique, and increasing")
    required = {"entry_date", "exit_date", "exit_reason"}
    if not required.issubset(trades.columns):
        raise ValueError(f"trade file missing columns: {sorted(required - set(trades.columns))}")
    entries = pd.to_datetime(trades["entry_date"], errors="raise")
    exits = pd.to_datetime(trades["exit_date"], errors="raise")
    if entries.isna().any() or exits.isna().any() or (exits < entries).any():
        raise ValueError("trades contain invalid entry or exit dates")
    if trades["exit_reason"].isna().any() or (trades["exit_reason"] == "").any():
        raise ValueError("trades contain missing exit reasons")
    unknown_dates = (set(entries) | set(exits)) - set(dates)
    if unknown_dates:
        raise ValueError(f"{len(unknown_dates)} trade dates are absent from the session calendar")
    split = pd.Timestamp(split)
    rows = []
    for period, mask in (("dev", dates < split), ("test", dates >= split)):
        period_dates = dates[mask]
        if period_dates.empty:
            raise ValueError(f"no {period} sessions")
        entry_mask = entries.isin(period_dates)
        exit_mask = exits.isin(period_dates) & (trades["exit_reason"] != "censored")
        rebalance_exit = exit_mask & (trades["exit_reason"] == "rebalance")
        rebalance_dates = set(entries[entry_mask]) | set(exits[rebalance_exit])
        rows.append(
            {
                "period": period,
                "sessions": len(period_dates),
                "entry_legs": int(entry_mask.sum()),
                "completed_legs": int(exit_mask.sum()),
                "rebalance_dates": len(rebalance_dates),
                "entry_legs_per_session": float(entry_mask.sum()) / len(period_dates),
                "completed_legs_per_session": float(exit_mask.sum()) / len(period_dates),
                "rebalance_dates_per_session": len(rebalance_dates) / len(period_dates),
            }
        )
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results", required=True, type=Path)
    parser.add_argument("--split", required=True)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    if args.out.exists():
        parser.error("--out already exists; refusing to overwrite")
    comparison = pd.read_csv(args.results / "variants_comparison.csv", usecols=["variant", "fill"])
    variants = comparison.loc[comparison["fill"] == "next_open", "variant"]
    if variants.empty or variants.duplicated().any():
        raise ValueError("expected unique next-open variants")
    rows = []
    for variant in variants:
        prefix = "" if variant == "baseline" else variant + "__"
        sessions = pd.read_csv(args.results / f"{prefix}daily_returns.csv", usecols=["date"])["date"]
        trades = pd.read_csv(args.results / f"{prefix}trades_next_open.csv")
        rows.extend({"variant": variant, **row} for row in count_frequency(sessions, trades, args.split))
    args.out.parent.mkdir(parents=True, exist_ok=True)
    pd.DataFrame(rows).to_csv(args.out, index=False)
    print(f"Recorded DEV and TEST frequencies for {len(variants)} research-scale variants.")
    print("Counts are gross modeled legs, not netted broker orders or whole-share executions.")


if __name__ == "__main__":
    main()
