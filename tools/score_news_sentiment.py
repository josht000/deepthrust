"""Score the point-in-time news archive with a frozen FinBERT model and a
Loughran-McDonald-style lexicon; write sentiment.parquet for the S056/S058 studies.

Output columns: symbol, date (ET yyyy-mm-dd), hour (ET float), score (finbert
signed), lm_score, topic.

Usage:
    python tools/score_news_sentiment.py [--news <news.parquet>] [--out <sentiment.parquet>]
                                         [--no-finbert] [--batch 64]
"""

import argparse
import re
import sys
from pathlib import Path

import pandas as pd

NEWS = r"C:\data\trading\trading_framework\data\pit\news\news.parquet"
OUT = r"C:\data\trading\trading_framework\data\pit\news\sentiment.parquet"

TOPIC_RULES = [
    ("ratings", r"\b(upgrade|downgrade|price target|pt |rating|initiat|overweight|underweight|outperform|underperform|buy rating|sell rating|hold rating|neutral|reiterate)\b"),
    ("earnings", r"\b(earnings|eps|q[1-4] |quarter|guidance|outlook|forecast|profit|revenue beat|revenue miss|margin)\b"),
    ("mna", r"\b(merger|acquisition|acquire|takeover|buyout|m&a|bid for|stake|spin-?off|divest)\b"),
    ("product", r"\b(fda|approval|launch|product|patent|contract|order|deal|partnership|trial|phase [123])\b"),
    ("corp", r"\b(dividend|buyback|repurchase|split|offering|secondary|debt|bond|notes|refinanc|insider)\b"),
    ("litigation", r"\b(lawsuit|settlement|sec |investigation|probe|fine|fraud|charges|indict)\b"),
    ("macro", r"\b(fed |fomc|cpi|inflation|jobs report|payroll|treasury|yield|oil prices|recession|gdp|tariff)\b"),
]
TOPIC_RES = [(name, re.compile(pat, re.IGNORECASE)) for name, pat in TOPIC_RULES]

LM_POS = set("""beat beats beating exceed exceeds exceeded gain gains gained growth improve improves
improved improvement increase increases increased outperform outperformed positive profit profits
profitable record rise rises rose strong stronger strongest surge surged top tops topped upgrade
upgraded upside win wins won higher breakthrough""".split())
LM_NEG = set("""decline declines declined decrease decreases decreased downgrade downgraded fall falls
fell loss losses lose losing lower miss misses missed negative plunged plunge risk risks weak weaker
weakest warning warn warns downgrade downgrades investigation recall cut cuts shortfall worse
worst slump dropped lawsuit fines bankruptcy""" .split())


def classify_topic(text: str) -> str:
    for name, rx in TOPIC_RES:
        if rx.search(text):
            return name
    return "other"


def lm_score(text: str) -> float:
    words = re.findall(r"[a-z]+", text.lower())
    if not words:
        return 0.0
    pos = sum(1 for w in words if w in LM_POS)
    neg = sum(1 for w in words if w in LM_NEG)
    return (pos - neg) / len(words)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--news", default=NEWS)
    ap.add_argument("--out", default=OUT)
    ap.add_argument("--no-finbert", action="store_true")
    ap.add_argument("--batch", type=int, default=64)
    args = ap.parse_args()

    news = pd.read_parquet(args.news)
    news = news.dropna(subset=["symbol", "created_at"])
    text = (news["headline"].fillna("") + " " + news["summary"].fillna("")).str.strip()
    news = news.assign(_text=text).drop_duplicates(subset=["symbol", "created_at", "_text"])
    print(f"{len(news)} unique items", flush=True)

    ts = pd.to_datetime(news["created_at"], utc=True, errors="coerce")
    et = ts.dt.tz_convert("America/New_York")
    news["date"] = et.dt.strftime("%Y-%m-%d")
    news["hour"] = et.dt.hour + et.dt.minute / 60.0
    news["topic"] = news["_text"].map(classify_topic)
    news["lm_score"] = news["_text"].map(lm_score)

    if args.no_finbert:
        news["score"] = news["lm_score"]
    else:
        from transformers import pipeline

        scorer = pipeline(
            "text-classification",
            model="ProsusAI/finbert",
            tokenizer="ProsusAI/finbert",
            top_k=None,
            truncation=True,
            max_length=256,
        )
        texts = [t[:512] for t in news["_text"].tolist()]
        scores = []
        b = max(1, args.batch)
        for i in range(0, len(texts), b):
            chunk = texts[i : i + b]
            for result in scorer(chunk):
                label_score = {r["label"].lower(): r["score"] for r in result}
                scores.append(label_score.get("positive", 0.0) - label_score.get("negative", 0.0))
            if (i // b) % 50 == 0:
                print(f"  scored {i + len(chunk)}/{len(texts)}", flush=True)
        news["score"] = scores

    out = news[["symbol", "date", "hour", "score", "lm_score", "topic"]].copy()
    out.to_parquet(args.out, index=False)
    print(f"wrote {len(out)} rows -> {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
