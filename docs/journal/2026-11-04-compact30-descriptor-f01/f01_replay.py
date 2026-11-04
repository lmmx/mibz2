#!/usr/bin/env python3
"""S62-F01 remedy: the gate-3 ledger replayed with counted frequencies only.

    python3 f01_replay.py ITEM EVIDENCE_DIR COUNTED_JSONL OUT_JSON

ITEM is `scanner` or `descriptor`. EVIDENCE_DIR is the prototype's archived evidence folder
(its callgrind/ raw profiles and listings/), COUNTED_JSONL the counted build's records with
the S62-F01 counters (the primary corpus's natural record is used). No compilation, no
execution of Rust, no timing: it reads archives only.

The parser (`profile`, `select`, `asm`) is the S62 review's replay.py (e62fc15f), copied
unchanged: header-aware (`positions:`/`events:`), compressed names, call-edge costs kept
apart from own costs.

Method:
1. Every own instruction of the changed functions (`build::<false>` and `refill`) is grouped
   by its exact execution count in the raw profile, v2 (B2) against the candidate; a class
   whose instruction count changes is a changed path, Δ = (candidate − B2) × frequency.
2. Each changed class is assigned the counted-build expression for its path (CLASSES below,
   each with the listing ranges it was read from). The expression is evaluated on the
   counted record and must equal the profile's frequency; only then is the class
   counter-supported. A class with no counter, or whose counter disagrees, is treated at its
   least favourable end (§5.3c): an added cost charged, a removal withheld.
3. Library call sites of `build::<false>` are compared call site by call site (callee, calls,
   Ir, Dr, Dw). Added sites are charged at the lower end; removed sites are credited only
   at the upper end (the review's convention, avoiding `rep movsb` per-byte counting).
4. Lower end = −(counter-supported Δ) − (uncounted added) − (added library);
   upper end = −(inclusive Δ of build::<false>). Both runs must agree.
5. Reported, not gated: own Dr/Dw deltas over build, refill, finish and canonicalise; the
   harness caller `c30walk::main`'s own delta (S62-F04), with the interval it would give if
   charged.
"""
import collections, gzip, hashlib, json, pathlib, re, sys


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

# Each changed class: (function, frequency) -> [(counter expression, path), ...]: one entry
# per path at that frequency, read from the listings with the profile's per-instruction
# counts (each journal's S62-F01 addendum). The listing ranges are derived below from the
# profile itself. A class is counter-supported only if every one of its paths' counters
# equals the frequency.
CLASSES = {
  'scanner': {
    ('build', 10544477): [('lookahead_pairs + runs_no_full_tile', 'uniform tiles U = J + Q (B2 tile test)')],
    ('build', 10541706): [('lookahead_pairs', 'lookahead pairs J')],
    ('build', 9933956): [('lookahead_pairs - held_reuses', 'matched lookaheads J - H')],
    ('build', 3003834): [('tiles_nonuniform + runs', 'outer scan iterations S + R')],
    ('build', 2396084): [('fresh_pairs', 'fresh pairs A')],
    ('build', 2393313): [('tiles_nonuniform', 'nonuniform full tiles S')],
    ('build', 1116556): [('tiles_second_segment', 'second segments X')],
    ('build', 996634): [('tile_refills_first', "refill detour from tile16's first segment")],
    ('build', 801475): [('fills_vector', "fill's vector loop entry (m >= 8) and its alias check")],
    ('build', 610521): [('runs', 'runs R')],
    ('build', 610356): [('runs_lookahead', 'runs entering the lookahead loop (alignment nop executed)')],
    ('build', 607750): [('held_reuses', 'held reuses H')],
    ('build', 302049): [('tile_refills_second', "refill detour from tile16's second segment")],
    ('build', 296405): [('complete_calls', "run's whole-group branch (complete)")],
    ('build', 23559): [('final_finishes', 'finish from the final close loop')],
    ('build', 18684): [('preset_crossing', 'crossing groups preset RAW (create_raw)')],
    ('build', 3654): [('epochs', 'epochs')],
    ('build', 3575): [('epochs_tiled', 'epochs with a full tile (scan loop entry)')],
    ('build', 3424): [('epochs_untailed', "epochs with no tail (B2's epoch-loop back edge)")],
    ('build', 2606): [('runs_exhausted', 'runs whose lookahead exhausts the epoch')],
    ('build', 786): [('tail_refills_first', "refill detour from the tail's first segment")],
    ('build', 434): [('blocks', 'blocks')],
    ('build', 230): [('tails', 'tails')],
    ('build', 218): [('tails_len_ge2', 'tail advance loop (unrolled by two) entry, len >= 2')],
    ('build', 165): [('runs_no_lookahead', 'runs with no lookahead tile')],
    ('build', 121): [('tails_even', 'tail advance loop exit, even len')],
    ('build', 106): [('tail_rows_second', 'tail second-segment row loop')],
    ('build', 79): [('epochs_short', 'epochs with no full tile')],
    ('build', 22): [('tails_second_segment', 'tail second-segment entry and exit')],
    ('build', 12): [('tails_len1', 'tail of length 1')],
    ('build', 9): [('tail_refills_second', "refill detour from the tail's second segment")],
  },
  'descriptor': {
    ('build', 29357924): [('rows_scalar - tile_rows_second - tail_rows', "full tiles' first-segment row loop")],
    ('build', 8935084): [('tile_rows_second', "full tiles' second-segment row loop")],
    ('build', 3003834): [('outer_iterations', 'outer scan iterations')],
    ('build', 2393313): [('outer_iterations - runs', 'nonuniform full tiles S')],
    ('build', 1085833): [('fills', 'fill calls (k > 0)')],
    ('build', 801475): [('fills_vector', "fill's vector loop entry (m >= 8) and its alias check")],
    ('build', 610521): [('runs', 'runs')],
    ('build', 597255): [('runs_end_fill', 'runs ending after a fill (B2 return path)')],
    ('build', 496004): [('run_opens', "open from run")],
    ('build', 467712): [('128 * epochs', "B2's snapshot loop: 256 ranks, two per iteration (fixed trip)")],
    ('build', 302049): [('tile_refills_second', "refill detour from a full tile's second segment")],
    ('build', 296405): [('complete_calls', "run's whole-group branch (complete)")],
    ('build', 230202): [('63 * epochs', "the snapshot loop: ranks 4..256, four per iteration (fixed trip)")],
    ('build', 55986): [('sweep_checks // 2 + blocks', 'final sweep loop head (two checks per iteration, plus the exit)')],
    ('build', 55552): [('sweep_checks // 2', 'final sweep loop body (two checks per iteration)')],
    ('build', 27776): [('init_descs // 4', 'descriptor initialisation and the cumulative-rank loop (four per iteration)')],
    ('build', 18684): [('preset_crossing', 'crossing groups preset RAW (create_raw)')],
    ('build', 13266): [('runs_end_complete', 'runs ending after complete (B2 return path)')],
    ('build', 3654): [('epochs', 'epochs')],
    ('build', 3575): [('epochs_tiled', 'epochs with a full tile (scan loop entry)')],
    ('build', 1769): [('tail_rows - tail_rows_second', "the tail's first-segment row loop")],
    ('build', 786): [('tail_refills_first', "refill detour from the tail's first segment")],
    ('build', 434): [('blocks', 'blocks')],
    ('build', 432): [('bitmap_in_place', 'bitmap resize without a reserve')],
    ('build', 233): [('pad_blocks', "the last group's padding (memset)")],
    ('build', 230): [('tails', 'tails'), ('bitmap_memsets', "bitmap resize's fill (>= 2 words)")],
    ('build', 106): [('tail_rows_second', "the tail's second-segment row loop")],
    ('build', 79): [('epochs_short', 'epochs with no full tile (B2 path)')],
    ('build', 9): [('tail_refills_second', "refill detour from the tail's second segment")],
    ('build', 8): [('raw_grows', 'raw arena regrowth')],
    ('build', 2): [('rec_grows', 'rec arena regrowth'), ('bitmap_reserves', 'bitmap reserve')],
    ('build', 1): [('rec_grow_frees', "rec regrowth freeing the old allocation"), ('snaps_reserves', 'snapshot reserve'),
                   ('bitmap_reserves_single', 'bitmap reserve of one word')],
    ('refill', 1299478): [('refills', 'refill calls')],
    ('refill', 1262935): [('refill_finishes', 'refills that finish a portion')],
  },
}

ITEMS = {
  'scanner': dict(mod='ctor2su', cls='Ctor2su', var='su', minimum=60_000_000),
  'descriptor': dict(mod='ctor2d', cls='Ctor2d', var='d', minimum=40_000_000),
}


def evaluate(expr, rec):
    return eval(expr, {'__builtins__': {}}, dict(rec))


def classes(pb, pc, nb, nc):
    def byf(p, n):
        r = collections.Counter()
        for v in p['addresses'][n].values():
            if v.get('Ir', 0): r[v['Ir']] += 1
        return r
    b, d = byf(pb, nb), byf(pc, nc)
    return [{'frequency': f, 'baseline': b[f], 'candidate': d[f], 'delta': (d[f] - b[f]) * f}
            for f in sorted(b.keys() | d.keys(), reverse=True) if b[f] != d[f]]


def main():
    item, ev, counted, out = sys.argv[1:5]
    cfg = ITEMS[item]; ev = pathlib.Path(ev); mod, cls, var = cfg['mod'], cfg['cls'], cfg['var']
    recs = [json.loads(l) for l in open(counted) if l.strip()]
    rec = next(r for r in recs if r['input'] == 'wikidata-20261026.bz2' and r['permille'] == 0 and not r['extra_starts'])
    runs = {}
    for run in (1, 2):
        pb = profile(ev / f'callgrind/only-v2-run{run}.callgrind.gz')
        pc = profile(ev / f'callgrind/only-{var}-run{run}.callgrind.gz')
        n = {('B2', 'build'): select(pb, r'::Ctor2>::build::<false>$'), ('c', 'build'): select(pc, rf'::{cls}>::build::<false>$'),
             ('B2', 'refill'): select(pb, r'::ctor2::W<false>>::refill$'), ('c', 'refill'): select(pc, rf'::{mod}::W<false>>::refill$'),
             ('B2', 'finish'): select(pb, r'::ctor2::W<false>>::finish$'), ('c', 'finish'): select(pc, rf'::{mod}::W<false>>::finish$'),
             ('B2', 'canonicalise'): select(pb, r'::canonicalise$'), ('c', 'canonicalise'): select(pc, r'::canonicalise$')}
        r = {'classes': {f: classes(pb, pc, n['B2', f], n['c', f]) for f in ('build', 'refill', 'finish', 'canonicalise')}}
        def incl(p, name):
            t = collections.Counter(p['own'][name])
            for s in p['sites'].get(name, []): t.update({k: s.get(k, 0) for k in ('Ir', 'Dr', 'Dw')})
            return t
        r['inclusive_delta'] = incl(pc, n['c', 'build'])['Ir'] - incl(pb, n['B2', 'build'])['Ir']
        r['own'] = {f: {k: [pb['own'][n['B2', f]].get(k, 0), pc['own'][n['c', f]].get(k, 0)] for k in ('Ir', 'Dr', 'Dw')}
                    for f in ('build', 'refill', 'finish', 'canonicalise')}
        key = lambda s: (s['callee'], s['calls'], s.get('Ir', 0), s.get('Dr', 0), s.get('Dw', 0))
        lib = lambda p, nm: collections.Counter(key(s) for s in p['sites'][nm] if 'c30walk' not in s['callee'])
        bs, cs = lib(pb, n['B2', 'build']), lib(pc, n['c', 'build'])
        r['library'] = {'shared': sum((bs & cs).values()), 'removed': [list(k) for k in (bs - cs).elements()],
                        'added': [list(k) for k in (cs - bs).elements()]}
        r['caller_main'] = [pb['own']['c30walk::main']['Ir'], pc['own']['c30walk::main']['Ir']]
        runs[run] = r
    assert json.dumps(runs[1], sort_keys=True) == json.dumps(runs[2], sort_keys=True), 'the two runs differ'
    r = runs[1]
    # each changed class's instructions, as listing ranges (offsets from the function's start)
    ranges = {}
    for fn, lists in (('build', ('v2-build-false-same-binary.asm', f'{mod}-build-false.asm')),
                      ('refill', ('v2-refill-false-same-binary.asm', f'{mod}-refill-false.asm'))):
        for side, p, pat, lf in (('B2', pb, r'::Ctor2>::build::<false>$' if fn == 'build' else r'::ctor2::W<false>>::refill$', lists[0]),
                                 ('', pc, rf'::{cls}>::build::<false>$' if fn == 'build' else rf'::{mod}::W<false>>::refill$', lists[1])):
            f = ev / 'listings' / lf
            if not f.exists():
                continue
            L = asm(f); base = min(L); counts = p['addresses'][select(p, pat)]
            for c in r['classes'][fn]:
                addrs = sorted(L); segs = []; start = prev = None
                for a in addrs + [None]:
                    hit = a is not None and counts.get(a, {}).get('Ir', 0) == c['frequency']
                    if hit and start is None: start = a
                    if not hit and start is not None:
                        segs.append(f'+{start - base:#x}' + (f'..+{prev - base:#x}' if prev != start else '')); start = None
                    prev = a
                if segs:
                    ranges.setdefault((fn, c['frequency']), []).append((side + ' ' if side else '') + ', '.join(segs))
    for f in ('finish', 'canonicalise'):
        assert not r['classes'][f] and r['own'][f]['Ir'][0] == r['own'][f]['Ir'][1], f'{f} changed'
    table, counted_delta, unc_pos, unc_neg = [], 0, 0, 0
    for fn in ('build', 'refill'):
        for c in r['classes'][fn]:
            m = CLASSES[item].get((fn, c['frequency']))
            row = dict(function=fn, **c, listing=ranges.get((fn, c['frequency'])))
            if m:
                vals = [evaluate(e, rec) for e, _ in m]
                row.update(counter=' & '.join(e for e, _ in m), counted=vals[0] if len(set(vals)) == 1 else vals,
                           counted_each=vals, path=' & '.join(p for _, p in m), profile_frequency=c['frequency'],
                           agrees=all(v == c['frequency'] for v in vals))
            else:
                row.update(counter=None, counted=None, agrees=False, path='(no counter)')
            if row['agrees']:
                counted_delta += c['delta']
            elif c['delta'] > 0:
                unc_pos += c['delta']
            else:
                unc_neg += c['delta']
            table.append(row)
    own_delta = sum(c['delta'] for fn in ('build', 'refill') for c in r['classes'][fn])
    assert counted_delta + unc_pos + unc_neg == own_delta
    lib_added = sum(k[2] for k in r['library']['added'])
    lib_removed = sum(k[2] for k in r['library']['removed'])
    # the inclusive delta of build::<false>: own build + every callee (refill included)
    assert r['inclusive_delta'] == own_delta + lib_added - lib_removed, 'inclusive accounting'
    lower = -counted_delta - unc_pos - lib_added
    upper = -r['inclusive_delta']
    caller = r['caller_main'][1] - r['caller_main'][0]
    mem = {k: sum(r['own'][f][k][1] - r['own'][f][k][0] for f in ('build', 'refill', 'finish', 'canonicalise')) for k in ('Dr', 'Dw')}
    mem_by_fn = {f: {k: r['own'][f][k][1] - r['own'][f][k][0] for k in ('Dr', 'Dw')} for f in ('build', 'refill')}
    res = {'item': item, 'counted_record': {k: rec[k] for k in rec}, 'classes': table,
           'counter_supported_delta': counted_delta, 'uncounted_added': unc_pos, 'uncounted_removed': unc_neg,
           'own_delta_build_and_refill': own_delta, 'library_added': lib_added, 'library_removed': lib_removed,
           'library_shared_sites': r['library']['shared'], 'library_added_sites': r['library']['added'],
           'library_removed_sites': r['library']['removed'], 'inclusive_delta': r['inclusive_delta'],
           'interval_removed': [lower, upper], 'minimum': cfg['minimum'], 'lower_end_minus_minimum': lower - cfg['minimum'],
           'own_memory_delta': mem, 'own_memory_delta_by_function': mem_by_fn, 'caller_main_own': r['caller_main'], 'caller_main_delta': caller,
           'interval_if_caller_charged': [lower - caller, upper - caller], 'runs_identical': True}
    json.dump(res, open(out, 'w'), indent=1)
    w = max(len(x['path']) for x in table)
    print(f'{item}: changed classes (build::<false> and refill own code), primary corpus')
    print(f"{'function':8} {'profile freq':>12} {'counted':>12} {'B2→cand':>8} {'Δ':>13}  path [counter]")
    for x in table:
        print(f"{x['function']:8} {x['frequency']:>12,} {x['counted'] if x['counted'] is not None else '-':>12} "
              f"{x['baseline']:>3}→{x['candidate']:<4} {x['delta']:>+13,}  {x['path']} [{x['counter']}]{'' if x['agrees'] else '  ** NOT COUNTER-SUPPORTED **'}")
    print(f"counter-supported Δ {counted_delta:+,}; uncounted added {unc_pos:+,}; uncounted removed {unc_neg:+,}")
    print(f"library: {r['library']['shared']} shared sites; added {lib_added:,} Ir; removed {lib_removed:,} Ir")
    print(f"inclusive Δ {r['inclusive_delta']:+,}; interval removed [{lower:,}, {upper:,}]; minimum {cfg['minimum']:,}")
    print(f"own reads {mem['Dr']:+,}, writes {mem['Dw']:+,}; by function {mem_by_fn}")
    print(f"caller c30walk::main {r['caller_main'][0]:,} -> {r['caller_main'][1]:,} ({caller:+,}); interval if charged [{lower-caller:,}, {upper-caller:,}]")


if __name__ == '__main__':
    main()
