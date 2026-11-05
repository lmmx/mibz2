"""Per-segment cost by phase between two callgrind runs of master's room walk.
usage: python3 -I anat2.py A.out NS_A B.out NS_B"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from cgparse import parse
from anat import PHASES
from collections import defaultdict
def phases(path):
    ev, tot, by_fn, by_line = parse(path)
    import cgparse
    out = defaultdict(lambda: [0] * len(ev))
    for (fn0, f, ln), v in cgparse.by_fnline.items():
        if not fn0 or not any(x in fn0 for x in ('room_segments', 'assemble', 'measured', 'walk_segments_room_with_prefetch')):
            continue
        if f and f.endswith('src/bwt.rs'):
            ph = 'bwt_other'
            for nm, a, b in PHASES:
                if a <= ln <= b: ph = nm; break
        elif f and 'slice/sort' in f: ph = 'sort(inlined)'
        elif f and ('slice/mod.rs' in f or 'cmp.rs' in f or 'slice/index.rs' in f): ph = 'core_slice(bsearch,...)'
        elif f and ('vec/' in f or 'raw_vec' in f or '/alloc/' in f): ph = 'alloc_inlined'
        elif f and ('ptr/' in f or 'intrinsics' in f): ph = 'core_ptr'
        else: ph = None
        if ph:
            for i, x in enumerate(v): out[ph][i] += x
    # out-of-line callees by function
    for fn, v in by_fn.items():
        if fn and 'room_segments' not in fn and 'assemble' not in fn and 'measured' not in fn and 'walk_segments_room_with_prefetch' not in fn:
            k = 'fn:' + ('memcpy' if 'memcpy' in fn else 'memset' if 'memset' in fn else 'sort' if 'sort' in fn else 'malloc/free/realloc' if any(x in fn for x in ('malloc','free','realloc','grow','reserve','alloc')) else fn[:40])
            for i, x in enumerate(v): out[k][i] += x
    return ev, tot, out
a, na, b, nb = sys.argv[1], int(sys.argv[2]), sys.argv[3], int(sys.argv[4])
ev, ta, pa = phases(a); _, tb, pb = phases(b)
show = ['Ir', 'Bcm', 'D1mr', 'D1mw', 'DLmr', 'DLmw']
idx = [ev.index(s) for s in show]
dn = nb - na
print(f'{os.path.basename(a)} -> {os.path.basename(b)}: {dn} added segments; per added segment:')
print(f'  {"phase":24s} ' + ' '.join(f'{s:>7s}' for s in show))
rows = []
for k in set(pa) | set(pb):
    va = pa.get(k, [0] * len(ev)); vb = pb.get(k, [0] * len(ev))
    rows.append((k, [(vb[i] - va[i]) / dn for i in idx]))
rows.sort(key=lambda r: -abs(r[1][0]))
acc = [0] * len(idx)
for k, v in rows:
    if max(abs(x) for x in v) < 0.05: continue
    acc = [x + y for x, y in zip(acc, v)]
    print(f'  {k:24s} ' + ' '.join(f'{x:7.2f}' for x in v))
print(f'  {"sum of rows":24s} ' + ' '.join(f'{x:7.2f}' for x in acc))
print(f'  {"TOTAL":24s} ' + ' '.join(f'{(tb[i] - ta[i]) / dn:7.2f}' for i in idx))
