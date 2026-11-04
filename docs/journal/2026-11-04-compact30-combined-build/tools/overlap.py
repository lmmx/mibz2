#!/usr/bin/env python3
"""The isolated ledgers' class tables side by side (scanner U2 and descriptor arrays, each
against its own same-binary B2 profile), and every B2 class that both credit as removed.

    python3 overlap.py SCANNER_DIR DESCRIPTOR_DIR OUT_JSON

Each DIR holds that component's archived raw profiles (only-v2-run1 and only-{su,d}-run1)
and listings. A class both credit (both deltas negative) is a candidate overlap, bounded by
min(credits) × frequency; it is demonstrated only where the same B2 instruction is shown
absent from both listings (by inspection, recorded in the journal)."""
import collections, json, pathlib, sys
from comb_replay import profile, asm

def byf(p, n):
    r = collections.Counter()
    for v in p['addresses'][n].values():
        if v.get('Ir', 0): r[v['Ir']] += 1
    return r

def table(d, var, mod, cls):
    pb = profile(d / 'only-v2-run1.callgrind.gz'); pc = profile(d / f'only-{var}-run1.callgrind.gz')
    out = {}
    for g, b, c in (('build', '<c30walk::ctor2::Ctor2>::build::<false>', f'<c30walk::{mod}::{cls}>::build::<false>'),
                    ('refill', '<c30walk::ctor2::W<false>>::refill', f'<c30walk::{mod}::W<false>>::refill'),
                    ('finish', '<c30walk::ctor2::W<false>>::finish', f'<c30walk::{mod}::W<false>>::finish')):
        hb, hc = byf(pb, b), byf(pc, c)
        out[g] = {f: (hb[f], hc[f]) for f in hb.keys() | hc.keys() if hb[f] != hc[f]}
        out[g + '_b2'] = dict(hb)
    return out

s = table(pathlib.Path(sys.argv[1]), 'su', 'ctor2su', 'Ctor2su')
d = table(pathlib.Path(sys.argv[2]), 'd', 'ctor2d', 'Ctor2d')
rows = []
for g in ('build', 'refill', 'finish'):
    for f in sorted(set(s[g]) | set(d[g]), reverse=True):
        sb, sc = s[g].get(f, (s[g + '_b2'].get(f, 0),) * 2)
        db, dc = d[g].get(f, (d[g + '_b2'].get(f, 0),) * 2)
        rows.append(dict(group=g, frequency=f, b2=sb, b2_d=db, scanner=sc, descriptor=dc,
                         scanner_delta=(sc - sb) * f, descriptor_delta=(dc - db) * f,
                         both_credit=sc < sb and dc < db,
                         candidate_bound=min(sb - sc, db - dc) * f if (sc < sb and dc < db) else 0))
json.dump(rows, open(sys.argv[3], 'w'), indent=1)
print(f"{'group':6} {'freq':>11} {'B2':>4} {'scan':>4} {'desc':>4} {'Δscan':>13} {'Δdesc':>13}  both-credit bound")
for r in rows:
    print(f"{r['group']:6} {r['frequency']:>11,} {r['b2']:>4} {r['scanner']:>4} {r['descriptor']:>4} {r['scanner_delta']:>+13,} {r['descriptor_delta']:>+13,}  {r['candidate_bound'] if r['both_credit'] else ''}")
print('finish classes changed: scanner', s['finish'], 'descriptor', d['finish'])
print('candidate overlap bound (classes both credit):', sum(r['candidate_bound'] for r in rows))
