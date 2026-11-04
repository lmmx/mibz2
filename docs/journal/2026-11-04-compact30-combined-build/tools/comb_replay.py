#!/usr/bin/env python3
"""The combined build's gate-3 ledger (charter §6.3, §5.3), counted once and from scratch
against B2, with counted frequencies.

    python3 comb_replay.py PROFILE_DIR COUNTED_JSONL OUT_JSON [--listings DIR] [--classes-only]

PROFILE_DIR holds the four raw Callgrind profiles of one binary (`only-v2-run{1,2}` and
`only-c-run{1,2}.callgrind.gz`, each builder alone in its own process on the primary
corpus); COUNTED_JSONL the sequential counted record (`ctor2c-count --counted`). Reads
archives only: no compilation, no execution of Rust, no timing.

The parser (`profile`, `select`, `asm`) is the S62 review's (replay.py at e62fc15f), copied
unchanged through the components' f01_replay.py (blob 75987984).

Method (the components' f01_replay.py, generalised to the combined closure):
1. The constructor closure is every `c30walk` function reachable from `build::<false>`
   through the profile's call edges, in each profile. Functions are paired B2 → combined
   by name (`ctor2` → `ctor2c`); v3's two-function finish is paired as one group: B2's
   `finish` against the combined `finish` (the leaf) + `finish_slow`. Every other paired
   function (`canonicalise`, the allocator monomorphs) must be equal in own Ir, Dr and Dw.
2. Within each changed group, every own instruction is grouped by its exact execution
   count; a class whose instruction count changes is a changed path,
   Δ = (combined − B2) × frequency. The class sum is the group's exact own-Ir delta.
3. Each changed class is assigned the counted-build expression(s) of its path(s) (CLASSES
   below, read from the listings with the profile's per-instruction counts). It is
   counter-supported only if every one of its paths' counters equals the frequency. A class
   with no counter, or a disagreeing one, is taken at its least favourable end (§5.3c):
   an added cost charged, a removal withheld.
4. Library call sites (callees outside `c30walk`) of every closure function are compared
   site by site (caller group, callee, calls, Ir, Dr, Dw). Added sites are charged at the
   lower end; removed sites are credited at the upper end only (the review's convention,
   which avoids `rep movsb` per-byte counting).
5. Lower end = −(counter-supported Δ) − (uncounted added) − (added library);
   upper end = −(inclusive Δ of build::<false>). The identity
   inclusive Δ = Σ own Δ + added library − removed library is asserted. Both runs must
   agree exactly.
6. Reported, not gated: own Dr and Dw deltas per group; `c30walk::main`'s own delta (the
   harness's dispatch, S62-F04) and the interval it would give if charged.
"""
import collections, gzip, json, pathlib, re, sys


# --- the S62 review's parser (replay.py at e62fc15f), unchanged -------------------------
def profile(path):
    names = {}; own = collections.defaultdict(lambda: collections.Counter())
    sites = collections.defaultdict(lambda: collections.defaultdict(collections.Counter))
    addresses = collections.defaultdict(lambda: collections.defaultdict(collections.Counter))
    edges = collections.Counter(); current = callee = None; pending = None
    positions = []; previous = []; events = []; summary = None
    def symbol(raw):
        m = re.fullmatch(r'\((\d+)\)(?: (.*))?', raw)
        if not m: return raw
        key = int(m[1])
        if m[2] is not None: names[key] = m[2]
        return key
    for raw in gzip.open(path, 'rt'):
        line = raw.strip()
        if line.startswith('positions:'):
            positions=line.split()[1:]; previous=[0]*len(positions)
        elif line.startswith('events:'): events=line.split()[1:]
        elif line.startswith('summary:'): summary=dict(zip(events,map(int,line.split()[1:])))
        elif line.startswith('fn='): current=symbol(line[3:])
        elif line.startswith('cfn='): callee=symbol(line[4:])
        elif line.startswith('calls='): pending=int(line.split()[0][6:])
        elif line and (line[0].isdigit() or line[0] in '+-*'):
            parts=line.split(); assert len(parts)>=len(positions)
            for i,t in enumerate(parts[:len(positions)]):
                if t=='*': pass
                elif t[0] in '+-': previous[i]+=int(t,0)
                else: previous[i]=int(t,0)
            values=collections.Counter(dict(zip(events,map(int,parts[len(positions):]))))
            addr=previous[positions.index('instr')]
            assert current is not None
            if pending is not None:
                sites[current][(addr,callee)].update(values)
                edges[(current,addr,callee)]+=pending; pending=None
            else:
                own[current].update(values); addresses[current][addr].update(values)
    def resolve(k): return names[k] if isinstance(k,int) else k
    return {
      'own':{resolve(k):dict(v) for k,v in own.items()},
      'sites':{resolve(k):[{'address':a,'callee':resolve(c),'calls':edges[k,a,c],**v} for (a,c),v in s.items()] for k,s in sites.items()},
      'addresses':{resolve(k):{a:dict(v) for a,v in s.items()} for k,s in addresses.items()},
      'summary':summary,
    }

def select(p, pattern):
    keys=[k for k in p['own'] if re.search(pattern,k)]
    assert len(keys)==1,(pattern,keys)
    return keys[0]

def asm(path):
    result={}
    for line in path.read_text().splitlines():
        m=re.match(r'^\s*([0-9a-f]+):\s*(.*)',line)
        if not m: continue
        body=m[2]
        body=re.sub(r'^(?:[0-9a-f]{2}\s+)+','',body)
        if not body or re.fullmatch(r'[0-9a-f ]+',body): continue
        result[int(m[1],16)]=body
    return result
# ---------------------------------------------------------------------------------------

B2_BUILD = '<c30walk::ctor2::Ctor2>::build::<false>'
C_BUILD = '<c30walk::ctor2c::Ctor2c>::build::<false>'
# the changed groups: B2 functions -> combined functions
GROUPS = {
    'build': ([B2_BUILD], [C_BUILD]),
    'refill': (['<c30walk::ctor2::W<false>>::refill'], ['<c30walk::ctor2c::W<false>>::refill']),
    'finish': (['<c30walk::ctor2::W<false>>::finish'],
               ['<c30walk::ctor2c::W<false>>::finish', '<c30walk::ctor2c::W<false>>::finish_slow']),
}
LISTINGS = {
    'build': (['v2-build-false-same-binary.asm'], ['ctor2c-build-false.asm']),
    'refill': (['v2-refill-false-same-binary.asm'], ['ctor2c-refill-false.asm']),
    'finish': (['v2-finish-false-same-binary.asm'], ['ctor2c-finish-false.asm', 'ctor2c-finish-slow-false.asm']),
}

# Each changed class: (group, frequency) -> [(counter expression, path), ...], one entry per
# path at that frequency, read from the listings with the profile's per-instruction counts.
CLASSES = {}
try:
    from comb_classes import CLASSES  # noqa: F401  (the mapping, kept beside this tool)
except ImportError:
    pass


def evaluate(expr, rec):
    return eval(expr, {'__builtins__': {}}, dict(rec))


def closure(p, root):
    seen, todo = set(), [root]
    while todo:
        f = todo.pop()
        if f in seen:
            continue
        seen.add(f)
        for s in p['sites'].get(f, []):
            if 'c30walk' in s['callee'] and s['callee'] not in seen:
                todo.append(s['callee'])
    return seen


def pair(name):
    return name.replace('::ctor2::', '::ctor2c::').replace('::Ctor2>', '::Ctor2c>')


def byf(p, names):
    r = collections.Counter()
    for n in names:
        for v in p['addresses'].get(n, {}).values():
            if v.get('Ir', 0):
                r[v['Ir']] += 1
    return r


def analyse(pb, pc):
    cb, cc = closure(pb, B2_BUILD), closure(pc, C_BUILD)
    grouped_b = {f for g in GROUPS.values() for f in g[0]}
    grouped_c = {f for g in GROUPS.values() for f in g[1]}
    assert grouped_b <= cb and grouped_c <= cc, (grouped_b - cb, grouped_c - cc)
    others = sorted(cb - grouped_b)
    assert {pair(f) for f in others} == cc - grouped_c, ({pair(f) for f in others} ^ (cc - grouped_c))
    r = {'closure_b2': sorted(cb), 'closure_c': sorted(cc), 'classes': {}, 'own': {}, 'unchanged': {}}
    for g, (fb, fc) in GROUPS.items():
        hb, hc = byf(pb, fb), byf(pc, fc)
        r['classes'][g] = [{'frequency': f, 'baseline': hb[f], 'candidate': hc[f], 'delta': (hc[f] - hb[f]) * f}
                           for f in sorted(hb.keys() | hc.keys(), reverse=True) if hb[f] != hc[f]]
        r['own'][g] = {k: [sum(pb['own'][x].get(k, 0) for x in fb), sum(pc['own'][x].get(k, 0) for x in fc)]
                       for k in ('Ir', 'Dr', 'Dw')}
        assert sum(c['delta'] for c in r['classes'][g]) == r['own'][g]['Ir'][1] - r['own'][g]['Ir'][0]
    for f in others:
        a, b = pb['own'][f], pc['own'][pair(f)]
        r['unchanged'][f] = {k: [a.get(k, 0), b.get(k, 0)] for k in ('Ir', 'Dr', 'Dw')}
        assert all(a.get(k, 0) == b.get(k, 0) for k in ('Ir', 'Dr', 'Dw')), f'{f} changed'
        assert byf(pb, [f]) == byf(pc, [pair(f)]), f'{f} classes changed'
    def incl(p, name):
        t = collections.Counter(p['own'][name])
        for s in p['sites'].get(name, []):
            t.update({k: s.get(k, 0) for k in ('Ir', 'Dr', 'Dw')})
        return t
    r['inclusive'] = [incl(pb, B2_BUILD)['Ir'], incl(pc, C_BUILD)['Ir']]
    r['inclusive_delta'] = r['inclusive'][1] - r['inclusive'][0]
    key = lambda grp, s: (grp, s['callee'], s['calls'], s.get('Ir', 0), s.get('Dr', 0), s.get('Dw', 0))
    def lib(p, fns_by_group):
        c = collections.Counter()
        for grp, fns in fns_by_group.items():
            for f in fns:
                for s in p['sites'].get(f, []):
                    if 'c30walk' not in s['callee']:
                        c[key(grp, s)] += 1
        return c
    gb = {g: v[0] for g, v in GROUPS.items()}; gb.update({f: [f] for f in others})
    gc = {g: v[1] for g, v in GROUPS.items()}; gc.update({f: [pair(f)] for f in others})
    bs, cs = lib(pb, gb), lib(pc, gc)
    r['library'] = {'shared': sum((bs & cs).values()), 'removed': [list(k) for k in (bs - cs).elements()],
                    'added': [list(k) for k in (cs - bs).elements()]}
    r['caller_main'] = [pb['own']['c30walk::main']['Ir'], pc['own']['c30walk::main']['Ir']]
    r['totals'] = [pb['summary']['Ir'], pc['summary']['Ir']]
    return r


def ranges(listdir, pb, pc, classes):
    """Each changed class's instructions, as listing ranges with their text"""
    out = {}
    for g, (lb, lc) in LISTINGS.items():
        for side, p, names, files in (('B2', pb, GROUPS[g][0], lb), ('C', pc, GROUPS[g][1], lc)):
            for name, lf in zip(names, files):
                f = listdir / lf
                if not f.exists():
                    continue
                L = asm(f); base = min(L); counts = p['addresses'].get(name, {})
                tag = side + ('' if len(names) == 1 else ':' + name.rsplit('::', 1)[1])
                for c in classes[g]:
                    hits = [a for a in sorted(L) if counts.get(a, {}).get('Ir', 0) == c['frequency']]
                    if hits:
                        out.setdefault((g, c['frequency']), []).append(
                            (tag, [f'+{a - base:#06x} {L[a]}' for a in hits]))
    return out


def main():
    args = sys.argv[1:]
    listdir = None
    if '--listings' in args:
        i = args.index('--listings'); listdir = pathlib.Path(args[i + 1]); del args[i:i + 2]
    only_classes = '--classes-only' in args
    args = [a for a in args if a != '--classes-only']
    ev, counted, out = pathlib.Path(args[0]), args[1], args[2]
    recs = [json.loads(l) for l in open(counted) if l.strip()]
    rec = next(r for r in recs if r['input'] == 'wikidata-20261026.bz2')
    runs = {}
    for run in (1, 2):
        pb = profile(ev / f'only-v2-run{run}.callgrind.gz')
        pc = profile(ev / f'only-c-run{run}.callgrind.gz')
        runs[run] = analyse(pb, pc)
    assert json.dumps(runs[1], sort_keys=True) == json.dumps(runs[2], sort_keys=True), 'the two runs differ'
    r = runs[1]
    rg = ranges(listdir, pb, pc, r['classes']) if listdir else {}
    table, counted_delta, unc_pos, unc_neg = [], 0, 0, 0
    for g in GROUPS:
        for c in r['classes'][g]:
            m = CLASSES.get((g, c['frequency']))
            row = dict(group=g, **c, listing=rg.get((g, c['frequency'])))
            if m:
                vals = [evaluate(e, rec) for e, _ in m]
                row.update(counter=' & '.join(e for e, _ in m), counted=vals[0] if len(set(vals)) == 1 else vals,
                           counted_each=vals, path=' & '.join(p for _, p in m), agrees=all(v == c['frequency'] for v in vals))
            else:
                row.update(counter=None, counted=None, agrees=False, path='(no counter)')
            if row['agrees']:
                counted_delta += c['delta']
            elif c['delta'] > 0:
                unc_pos += c['delta']
            else:
                unc_neg += c['delta']
            table.append(row)
    own_delta = sum(c['delta'] for g in GROUPS for c in r['classes'][g])
    lib_added = sum(k[3] for k in r['library']['added'])
    lib_removed = sum(k[3] for k in r['library']['removed'])
    assert r['inclusive_delta'] == own_delta + lib_added - lib_removed, \
        ('inclusive accounting', r['inclusive_delta'], own_delta, lib_added, lib_removed)
    lower = -counted_delta - unc_pos - lib_added
    upper = -r['inclusive_delta']
    caller = r['caller_main'][1] - r['caller_main'][0]
    mem = {g: {k: r['own'][g][k][1] - r['own'][g][k][0] for k in ('Dr', 'Dw')} for g in GROUPS}
    memt = {k: sum(mem[g][k] for g in GROUPS) for k in ('Dr', 'Dw')}
    res = {'counted_record': rec, 'classes': table, 'counter_supported_delta': counted_delta,
           'uncounted_added': unc_pos, 'uncounted_removed': unc_neg, 'own_delta': own_delta,
           'own_by_group': r['own'], 'unchanged_functions': r['unchanged'],
           'library_added': lib_added, 'library_removed': lib_removed, 'library_shared_sites': r['library']['shared'],
           'library_added_sites': r['library']['added'], 'library_removed_sites': r['library']['removed'],
           'inclusive': r['inclusive'], 'inclusive_delta': r['inclusive_delta'], 'interval_removed': [lower, upper],
           'own_memory_delta': memt, 'own_memory_delta_by_group': mem, 'caller_main_own': r['caller_main'],
           'caller_main_delta': caller, 'interval_if_caller_charged': [lower - caller, upper - caller],
           'process_totals': r['totals'], 'closure_b2': r['closure_b2'], 'closure_c': r['closure_c'],
           'runs_identical': True}
    json.dump(res, open(out, 'w'), indent=1)
    print('combined build: changed classes by group (own code), primary corpus, B2 -> combined')
    print(f"{'group':7} {'profile freq':>12} {'counted':>12} {'B2→C':>9} {'Δ':>14}  path [counter]")
    for x in table:
        print(f"{x['group']:7} {x['frequency']:>12,} {str(x['counted']) if x['counted'] is not None else '-':>12} "
              f"{x['baseline']:>4}→{x['candidate']:<4} {x['delta']:>+14,}  {x['path']} [{x['counter']}]"
              f"{'' if x['agrees'] else '  ** NOT COUNTER-SUPPORTED **'}")
        if only_classes and x['listing'] and not x['agrees']:
            for tag, lines in x['listing']:
                print(f"      {tag}: " + ' | '.join(lines))
    print(f"own Δ {own_delta:+,} (counter-supported {counted_delta:+,}; uncounted added {unc_pos:+,}; uncounted removed {unc_neg:+,})")
    print(f"own by group: " + '; '.join(f"{g} {v['Ir'][0]:,} -> {v['Ir'][1]:,}" for g, v in r['own'].items()))
    print(f"unchanged closure functions: {len(r['unchanged'])} (own Ir, Dr, Dw equal)")
    print(f"library: {r['library']['shared']} shared sites; added {lib_added:,} Ir {r['library']['added']}; removed {lib_removed:,} Ir {r['library']['removed']}")
    print(f"inclusive build::<false> {r['inclusive'][0]:,} -> {r['inclusive'][1]:,} (Δ {r['inclusive_delta']:+,}); interval removed [{lower:,}, {upper:,}]")
    print(f"own reads {memt['Dr']:+,}, writes {memt['Dw']:+,}; by group {mem}")
    print(f"caller c30walk::main {r['caller_main'][0]:,} -> {r['caller_main'][1]:,} ({caller:+,}); interval if charged [{lower - caller:,}, {upper - caller:,}]")
    print(f"process totals {r['totals'][0]:,} -> {r['totals'][1]:,}")


if __name__ == '__main__':
    main()
