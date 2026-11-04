"""Quote comma-separated symbol lists inside flow-style variant maps.

YAML flow mappings treat unquoted commas as entry separators, so
`{symbols: SPY,QQQ}` parses symbols="SPY" plus stray keys. Base-level
`symbols:` block scalars are unaffected and left alone.
"""
import re
from pathlib import Path

KEYS = r"(?:cyclical_symbols|defensive_symbols|symbols|long_symbols)"
PAT = re.compile(rf"({KEYS}: )([A-Z][A-Z,.0-9]*[A-Z0-9])(?=[,\s}}])")

files = [
    "configs/s010_turnaround_tuesday.yaml",
    "configs/s013_tlt_mean_reversion.yaml",
    "configs/s052_earnings_premium.yaml",
    "configs/s053_pre_earnings_runup.yaml",
    "configs/s054_pead_surprise.yaml",
    "configs/s056_news_sentiment.yaml",
    "configs/s058_topic_sentiment.yaml",
    "configs/s065_vol_compression.yaml",
    "configs/s067_sector_breadth_rotation.yaml",
    "configs/s069_tlt_crash_hedge.yaml",
    "configs/s070_qqq_regime_tqqq.yaml",
    "configs/s071_cointegrated_pairs.yaml",
    "configs/s078_statarb_reversal.yaml",
    "configs/s079_turn_of_month.yaml",
    "configs/s080_pre_holiday.yaml",
    "configs/s102_copper_gold_tilt.yaml",
    "configs/s104_crash_protection.yaml",
]

for name in files:
    path = Path(name)
    text = path.read_text(encoding="utf-8")
    out = []
    for line in text.splitlines():
        if "{" in line and not line.lstrip().startswith("#"):
            line = PAT.sub(
                lambda m: f'{m.group(1)}"{m.group(2)}"'
                if "," in m.group(2)
                else m.group(0),
                line,
            )
        out.append(line)
    new = "\n".join(out) + "\n"
    if new != text:
        path.write_text(new, encoding="utf-8")
        print("fixed", name)
