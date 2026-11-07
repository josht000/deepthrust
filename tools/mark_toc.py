import re

path = "docs/strategies/GitHub Strategy Catalog.md"
text = open(path, encoding="utf-8").read()
done = []
for g in ["G010", "G063", "G077", "G078", "G079", "G080", "G081", "G085",
          "G095", "G106", "G109", "G114", "G139"]:
    pat = re.compile(r"^(\s*- \[G" + g[1:] + r" \|[^\n]*\]\([^)]*\))(?!.*IMPLEMENTED)([^\n]*)$", re.M)
    text, n = pat.subn(lambda m: m.group(1) + " — IMPLEMENTED" + m.group(2), text)
    done.append(f"{g}:{n}")
open(path, "w", encoding="utf-8").write(text)
print(done)
