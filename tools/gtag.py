import io, re
p = 'docs/strategies/GitHub Strategy Catalog.md'
s = io.open(p, encoding='utf-8').read()
ids = ['G003','G021','G023','G037','G099','G104','G105','G119','G120','G121','G123','G127','G128','G129','G130','G146','G202']
n = 0
for gid in ids:
    pat = re.compile(r'(- \[' + gid + r' \| [^\]]+\]\([^)]+\))(?!\s*—)')
    s, c = pat.subn(r'\1 — IMPLEMENTED', s, count=1)
    n += c
io.open(p, 'w', encoding='utf-8').write(s)
print('tagged', n)
