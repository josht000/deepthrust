import csv, glob, io, os, re

DOC = 'docs/strategies/GitHub Strategy Catalog.md'
names = {
    'g003_hl_reversal': 'G003', 'g021_voladj_rev': 'G021', 'g023_screened_pairs': 'G023',
    'g037_pci_kalman': 'G037', 'g099_olps': 'G099', 'g104_lagged_indmom': 'G104',
    'g105_beta_div_rev': 'G105', 'g119_alpha009': 'G119', 'g120_alpha012': 'G120',
    'g121_candle_loc': 'G121', 'g123_alpha191_020': 'G123', 'g127_bb_w': 'G127',
    'g128_ha_reversal': 'G128', 'g129_rsi_hs': 'G129', 'g130_shooting_star': 'G130',
    'g146_weekday_beta': 'G146', 'g202_ternary': 'G202',
}
lines = {}
for f in glob.glob('results/g*/variants_comparison.csv'):
    name = os.path.basename(os.path.dirname(f))
    if name not in names:
        continue
    rows = [r for r in csv.DictReader(io.open(f, encoding='utf-8')) if r['fill'] == 'next_open']
    b = next((r for r in rows if r['variant'] == 'baseline'), None)
    if not b:
        continue
    cands = [r for r in rows if r['variant'] != 'baseline' and r['dev_trades'] and float(r['dev_trades']) >= 100]
    w = max(cands, key=lambda x: float(x['dev_skill'] or '-999')) if cands else None
    def m(r, k):
        try:
            return '%.3f' % float(r[k])
        except Exception:
            return 'nan'
    t = 'baseline: dev %s/%s trades, skill %s, sharpe %s; test skill %s' % (
        b['dev_trades'], b['test_trades'], m(b, 'dev_skill'), m(b, 'dev_sharpe'), m(b, 'test_skill'))
    if w:
        t += ' | winner %s: dev skill %s, sharpe %s; test skill %s, %s/%s trades' % (
            w['variant'], m(w, 'dev_skill'), m(w, 'dev_sharpe'), m(w, 'test_skill'),
            w['dev_trades'], w['test_trades'])
    t += '. Reproduce: `target\\release\\deepthrust.exe --strategy %s`' % name
    lines[names[name]] = t

s = io.open(DOC, encoding='utf-8').read()
for gid, txt in lines.items():
    pat = re.compile(r'(### ' + gid + r' \|[^\n]*\n)')
    if 'implementation_results' not in s.split('### ' + gid)[1].split('### G')[0]:
        s = pat.sub(lambda m: m.group(1) + '- implementation_results: code in `src/family12.rs`; ' + txt + '\n', s, count=1)
io.open(DOC, 'w', encoding='utf-8').write(s)
print('done', len(lines))
