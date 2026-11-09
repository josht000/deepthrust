"""Write a small synthetic es_1m.csv into data/futures/raw/ so the
ingest->audit->seal pipeline can be smoke-tested without Databento.
Sessions are structurally valid (18:00-17:00 ET) but prices are a
random walk — NOT for studies.
"""
import argparse
import csv
import datetime as dt
import random
from pathlib import Path


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="data/futures/raw/es_1m.csv")
    ap.add_argument("--days", type=int, default=8)
    ap.add_argument("--seed", type=int, default=7)
    args = ap.parse_args()

    rng = random.Random(args.seed)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)

    # sessions dated Mon-Fri; start Monday 2022-01-03.
    start = dt.date(2022, 1, 3)
    px = 4500.0
    with out.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["ts_event", "open", "high", "low", "close", "volume", "contract"])
        for i in range(args.days):
            sess = start + dt.timedelta(days=i)
            # session opens 18:00 ET the evening before
            t = dt.datetime.combine(sess - dt.timedelta(days=1), dt.time(18, 0))
            end = t + dt.timedelta(hours=23)
            while t < end:
                # convert ET->UTC (winter EST = +5)
                ts = t + dt.timedelta(hours=5)
                if t.hour == 17:  # halt hour
                    t += dt.timedelta(minutes=1)
                    continue
                o = px
                c = px + rng.uniform(-1.5, 1.5)
                h = max(o, c) + rng.uniform(0, 0.75)
                lo = min(o, c) - rng.uniform(0, 0.75)
                w.writerow([
                    ts.strftime("%Y-%m-%d %H:%M:%S"),
                    f"{o:.2f}", f"{h:.2f}", f"{lo:.2f}", f"{c:.2f}",
                    rng.randint(50, 500),
                    "ESH2",
                ])
                px = c
                t += dt.timedelta(minutes=1)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
