"""Screen existing S041 research-scale runs using DEV metrics only."""

import argparse
import math
from pathlib import Path

import pandas as pd


def benchmark_drawdown(prices, sessions):
    if not isinstance(prices.index, pd.DatetimeIndex):
        raise ValueError("benchmark must use a DatetimeIndex")
    if prices.index.tz is None:
        raise ValueError("benchmark timestamps must have a timezone")
    prices = prices.copy()
    prices.index = prices.index.tz_convert("America/New_York").normalize().tz_localize(None)
    if prices.index.has_duplicates or not prices.index.is_monotonic_increasing:
        raise ValueError("benchmark dates must be unique and increasing")
    dates = pd.DatetimeIndex(sessions)
    if dates.empty or dates.has_duplicates or not dates.is_monotonic_increasing:
        raise ValueError("DEV session dates must be nonempty, unique, and increasing")
    missing = dates.difference(prices.index)
    if not missing.empty:
        raise ValueError(f"benchmark missing {len(missing)} DEV sessions; first: {missing[0].date()}")
    first = prices.index.get_loc(dates[0])
    if first == 0:
        raise ValueError("benchmark lacks the session preceding DEV")
    values = prices.loc[dates].astype(float)
    if not all(math.isfinite(x) and x > 0 for x in values) or not (
        math.isfinite(float(prices.iloc[first - 1])) and prices.iloc[first - 1] > 0
    ):
        raise ValueError("benchmark has nonpositive or nonfinite prices")
    values = pd.concat([pd.Series([prices.iloc[first - 1]]), values.reset_index(drop=True)], ignore_index=True)
    return float((values / values.cummax() - 1).min() * 100)


def screen(frame, threshold):
    if not math.isfinite(threshold) or threshold < 0:
        raise ValueError("drawdown limit must be nonnegative and finite")
    required = ["variant", "dev_skill", "dev_sharpe", "dev_pf", "dev_trades", "dev_max_dd_pct"]
    missing = set(required) - set(frame.columns)
    if missing:
        raise ValueError(f"missing DEV columns: {sorted(missing)}")
    result = frame[required].copy()
    if result.variant.duplicated().any():
        raise ValueError("duplicate DEV variant")
    numeric = required[1:]
    for column in numeric:
        result[column] = pd.to_numeric(result[column], errors="raise")
    if not all(math.isfinite(v) for column in numeric for v in result[column]):
        raise ValueError("DEV metrics must be finite")
    if (result.dev_max_dd_pct > 0).any() or (result.dev_max_dd_pct < -100).any():
        raise ValueError("DEV max drawdowns must be between -100% and 0%")
    if (result.dev_trades < 0).any():
        raise ValueError("DEV trade counts must be nonnegative")
    reasons = []
    for row in result.itertuples(index=False):
        failures = []
        if row.dev_skill <= 0:
            failures.append("nonpositive DEV skill")
        if row.dev_sharpe <= 0:
            failures.append("nonpositive DEV Sharpe")
        if row.dev_pf <= 1:
            failures.append("DEV PF not above 1")
        if row.dev_trades < 100:
            failures.append("fewer than 100 DEV trades")
        if row.dev_max_dd_pct < -threshold:
            failures.append("DEV drawdown exceeds limit")
        reasons.append("; ".join(failures))
    result["ineligibility"] = reasons
    result["eligible"] = result.ineligibility == ""
    result["balanced_dev_score"] = float("nan")
    eligible = result.eligible
    if eligible.any():
        columns = ["dev_skill", "dev_sharpe", "dev_pf", "dev_max_dd_pct"]
        result.loc[eligible, "balanced_dev_score"] = (
            result.loc[eligible, columns].rank(method="average", pct=True).mean(axis=1)
        )
    return result.sort_values(
        ["eligible", "balanced_dev_score", "dev_skill", "variant"],
        ascending=[False, False, False, True],
        kind="stable",
        na_position="last",
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results", required=True, type=Path)
    parser.add_argument("--benchmark", required=True, type=Path)
    parser.add_argument("--split", required=True)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    if args.out.exists():
        parser.error("--out already exists; refusing to overwrite")
    split = pd.Timestamp(args.split)
    daily = pd.read_csv(args.results / "daily_returns.csv", usecols=["date"], parse_dates=["date"])
    sessions = daily.loc[daily.date < split, "date"]
    benchmark = pd.read_parquet(args.benchmark)["close"]
    spy_dd = benchmark_drawdown(benchmark, sessions)
    limit = min(25.0, -spy_dd)
    comparison = pd.read_csv(
        args.results / "variants_comparison.csv",
        usecols=["variant", "fill", "dev_skill", "dev_sharpe", "dev_pf", "dev_trades"],
    )
    comparison = comparison.loc[comparison["fill"] == "next_open"].drop(columns="fill")
    drawdowns = []
    for variant in comparison.variant:
        prefix = "" if variant == "baseline" else variant + "__"
        partition = pd.read_csv(args.results / f"{prefix}partition_dev.csv")
        row = partition.loc[(partition["fill"] == "next_open") & (partition["period"] == "dev")]
        if len(row) != 1:
            raise ValueError(f"{variant}: expected one DEV next-open partition row")
        drawdowns.append(row.iloc[0]["max_dd_pct"])
    comparison["dev_max_dd_pct"] = drawdowns
    result = screen(comparison, limit)
    result.insert(1, "spy_dev_max_dd_pct", spy_dd)
    result.insert(2, "dev_drawdown_limit_pct", limit)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    result.to_csv(args.out, index=False)
    winners = result[result.eligible]
    if winners.empty:
        print(f"No eligible variant: DEV drawdown limit {limit:.2f}%, SPY DEV drawdown {spy_dd:.2f}%.")
    else:
        print(f"Exploratory DEV winner: {winners.iloc[0].variant}; balanced score {winners.iloc[0].balanced_dev_score:.3f}.")
    print("Research-scale daily results only; not a whole-share account, and TEST was already inspected.")


if __name__ == "__main__":
    main()
