import io, re, json, os, glob, time

PY_MAIN = r'C:\data\trading\deepthrust'
PY_TIER = r'C:\data\trading\deepthrust-gstrats'
PAL = r'C:\data\trading\deepthrust-pal'

def rd(p):
    return io.open(p, encoding='utf-8', errors='replace').read()

# --- 1. S catalog (main only) ---
s_doc = rd(os.path.join(PY_MAIN, r'docs\strategies\Trading Strategy Catalog (105 strategies, markdown).md'))
s_ids = set(re.findall(r'S\d{3}', s_doc))
toc_impl = set(re.findall(r'^\s*- \[(S\d{3})[^\]]*\][^\n]*IMPLEMENTED', s_doc, re.M))
s_impl_body = set()
for m in re.finditer(r'## (S\d{3})\b.*?(?=\n## S\d{3}|\Z)', s_doc, re.S):
    if 'rust_status: IMPLEMENTED' in m.group(0):
        s_impl_body.add(m.group(1))
s_impl = toc_impl | s_impl_body
cfgs = [os.path.basename(x) for x in glob.glob(os.path.join(PY_MAIN, 'configs', 's*.yaml'))]
print('S: identified=%d toc_impl=%d body_impl=%d union=%d configs=%d' % (
    len(s_ids), len(toc_impl), len(s_impl_body), len(s_impl), len(cfgs)))

# --- 2. G catalog union ---
g_impl = set()
g_ids = set()
dups = 0
for root in (PY_MAIN, PY_TIER):
    g_doc = rd(os.path.join(root, r'docs\strategies\GitHub Strategy Catalog.md'))
    g_ids |= set(re.findall(r'G\d{3}', g_doc))
    g_impl |= set(re.findall(r'(G\d{3})[^\n]*IMPLEMENTED', g_doc))
    dups = max(dups, len(re.findall(r'DUPLICATE \(S', g_doc)))
    both = re.findall(r'(?:DUPLICATE[^\n]*IMPLEMENTED|IMPLEMENTED[^\n]*DUPLICATE)', g_doc)
    if both:
        print('WARN implemented+dup lines in', root, both)
print('G: identified=%d dup_dropped=%d implemented_union=%d unique=%d' % (
    len(g_ids), dups, len(g_impl), len(g_ids) - dups))

# --- 3. Markov ---
mk_src = glob.glob(os.path.join(PY_MAIN, 'src', 'markov*')) + glob.glob(os.path.join(PY_TIER, 'src', 'markov*'))
mk_cfg = glob.glob(os.path.join(PY_MAIN, 'configs', '*markov*')) + glob.glob(os.path.join(PY_TIER, 'configs', '*markov*'))
mk_res = glob.glob(os.path.join(PY_MAIN, 'results', '*markov*')) + glob.glob(os.path.join(PY_TIER, 'results', '*markov*'))
print('Markov: src=%d cfg=%s res=%s' % (
    len(mk_src), [os.path.basename(x) for x in mk_cfg], [os.path.basename(x) for x in mk_res]))
mk_impl = 1 if (mk_cfg and mk_res) else 0

# --- 4. PAL ---
v = os.path.join(PAL, 'results', 'pal', 'funnel', 'versions.jsonl')
nvers = sum(1 for _ in io.open(v, encoding='utf-8')) if os.path.exists(v) else 0
bv = os.path.join(PAL, 'results', 'pal', 'behavior_verdicts.csv')
nbeh = (sum(1 for _ in io.open(bv, encoding='utf-8')) - 1) if os.path.exists(bv) else 0
mtime = time.strftime('%Y-%m-%d %H:%M', time.localtime(os.path.getmtime(v))) if os.path.exists(v) else 'n/a'
for rel in (r'results\pal\funnel\candidates.json', r'results\pal\funnel\working_setups.json',
            r'results\pal\candidates.json', r'results\pal\working_setups.json'):
    p = os.path.join(PAL, rel)
    if os.path.exists(p):
        try:
            n = len(json.load(io.open(p, encoding='utf-8')))
        except Exception:
            n = '?'
        print('PAL %s: %s (mtime %s)' % (rel, n, time.strftime('%m-%d %H:%M', time.localtime(os.path.getmtime(p)))))
print('PAL: versions_run=%d behavior=%d versions_mtime=%s' % (nvers, nbeh, mtime))

s_i, s_m = len(s_ids), len(s_impl)
g_i_u, g_m = len(g_ids) - dups, len(g_impl)
mk_i = 4
print('TOTAL strategy-level: %d/%d (S %d + G %d + MK %d); dropped-as-dup=%d' % (
    s_m + g_m + mk_impl, s_i + g_i_u + mk_i, s_i, g_i_u, mk_i, dups))
print('GRAND incl PAL: %d/%d' % (s_m + g_m + mk_impl + nvers, s_i + g_i_u + mk_i + 101345))
