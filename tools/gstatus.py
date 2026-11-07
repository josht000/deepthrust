import io, json
p = 'docs/strategies/GitHub Strategy Catalog (JSON for coding agent).json'
d = json.load(io.open(p, encoding='utf-8'))
entries = d if isinstance(d, list) else d.get('strategies') or d.get('entries')
old = ['G033','G067','G068','G070','G072','G076','G091','G096','G097','G098','G100','G101']
new = ['G003','G021','G023','G037','G099','G104','G105','G119','G120','G121','G123','G127','G128','G129','G130','G146','G202']
dup = ['G039','G044','G113']
n = 0
for e in entries:
    if e.get('id') in old + new:
        e['status'] = 'implemented'
        n += 1
    elif e.get('id') in dup:
        e['status'] = 'duplicate'
io.open(p, 'w', encoding='utf-8').write(json.dumps(d, indent=2, ensure_ascii=False) + '\n')
print('marked', n, 'implemented +', len(dup), 'duplicate')
