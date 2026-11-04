#!/usr/bin/env python3
"""Gate 2 (diagnostic): the combined build's compiled paths against B2 and each component's
isolated build, from archived listings and raw profiles only.

    python3 gate2.py EVIDENCE_DIR ARCHIVE_DIR OUT_DIR

EVIDENCE_DIR is this build's evidence (listings/, callgrind/); ARCHIVE_DIR holds the
components' archived listings and profiles (scanner/, descriptor/, v3/, b2/, each fetched
with `git show` from its pin; see the README). Writes:
- paths.txt: every execution-count class that changed in any of the three isolated or the
  combined ledger, with its instruction count in B2, the scanner (U2), the descriptor
  arrays and the combined build, per group (B2 from each component's own same-binary
  profile; a class absent from a component's diff has B2's count there);
- stack.txt: dynamic stack traffic of `build::<false>`, refill and finish: executions of
  instructions with a plain `disp(%rsp)` memory operand (spills, reloads and stack-held
  state; indexed `(%rsp,%reg,8)` accesses to the stack arrays `wr`/`en` and pushes and pops
  are listed apart), loads and stores, in each build against its B2 profile.
"""
import collections, pathlib, re, sys
from comb_replay import profile, asm

ev, ar, out = map(pathlib.Path, sys.argv[1:4])
out.mkdir(parents=True, exist_ok=True)

B2 = {'build': '<c30walk::ctor2::Ctor2>::build::<false>', 'refill': '<c30walk::ctor2::W<false>>::refill',
      'finish': '<c30walk::ctor2::W<false>>::finish'}
BUILDS = {
  'scanner': (ar / 'scanner', 'su', {'build': ['<c30walk::ctor2su::Ctor2su>::build::<false>'], 'refill': ['<c30walk::ctor2su::W<false>>::refill'],
                                     'finish': ['<c30walk::ctor2su::W<false>>::finish']}),
  'descriptor': (ar / 'descriptor', 'd', {'build': ['<c30walk::ctor2d::Ctor2d>::build::<false>'], 'refill': ['<c30walk::ctor2d::W<false>>::refill'],
                                          'finish': ['<c30walk::ctor2d::W<false>>::finish']}),
  'combined': (ev / 'callgrind', 'c', {'build': ['<c30walk::ctor2c::Ctor2c>::build::<false>'], 'refill': ['<c30walk::ctor2c::W<false>>::refill'],
                                       'finish': ['<c30walk::ctor2c::W<false>>::finish', '<c30walk::ctor2c::W<false>>::finish_slow']}),
}
LIST = {
  'scanner': (ar / 'scanner', {'build': 'v2-build-false-same-binary.asm'}, {'build': ['ctor2su-build-false.asm'], 'refill': ['ctor2su-refill-false.asm'], 'finish': ['ctor2su-finish-false.asm']}),
  'descriptor': (ar / 'descriptor', {'build': 'v2-build-false-same-binary.asm', 'refill': 'v2-refill-false-same-binary.asm'},
                 {'build': ['ctor2d-build-false.asm'], 'refill': ['ctor2d-refill-false.asm'], 'finish': ['ctor2d-finish-false.asm']}),
  'combined': (ev / 'listings', {'build': 'v2-build-false-same-binary.asm', 'refill': 'v2-refill-false-same-binary.asm', 'finish': 'v2-finish-false-same-binary.asm'},
               {'build': ['ctor2c-build-false.asm'], 'refill': ['ctor2c-refill-false.asm'], 'finish': ['ctor2c-finish-false.asm', 'ctor2c-finish-slow-false.asm']}),
}


def byf(p, names):
    r = collections.Counter()
    for n in names:
        for v in p['addresses'].get(n, {}).values():
            if v.get('Ir', 0):
                r[v['Ir']] += 1
    return r


prof, hist, b2hist = {}, {}, {}
for k, (d, var, fns) in BUILDS.items():
    pb = profile(d / 'only-v2-run1.callgrind.gz'); pc = profile(d / f'only-{var}-run1.callgrind.gz')
    prof[k] = (pb, pc)
    hist[k] = {g: byf(pc, fs) for g, fs in fns.items()}
    b2hist[k] = {g: byf(pb, [B2[g]]) for g in fns}
for g in ('build', 'refill', 'finish'):
    assert b2hist['scanner'][g] == b2hist['descriptor'][g] == b2hist['combined'][g], f'B2 {g} differs between profiles'

lines = ['# Execution-count classes, B2 -> scanner U2 -> descriptor arrays -> combined: instructions executed at',
         '# exactly that count in the group (own code); "*" marks a class whose count differs from B2 in that build.',
         '# B2 is identical in all three same-binary B2 profiles (asserted).',
         f"{'group':7} {'frequency':>12} {'B2':>5} {'scan':>5} {'desc':>5} {'comb':>5}   Δcomb vs B2"]
for g in ('build', 'refill', 'finish'):
    b = b2hist['combined'][g]
    fs = set()
    for k in BUILDS:
        h = hist[k][g]
        fs |= {f for f in b.keys() | h.keys() if b[f] != h[f]}
    for f in sorted(fs, reverse=True):
        row = [hist[k][g][f] for k in BUILDS]
        mark = ['*' if x != b[f] else ' ' for x in row]
        lines.append(f"{g:7} {f:>12,} {b[f]:>5} " + ' '.join(f"{x:>4}{m}" for x, m in zip(row, mark))
                     + f"   {(row[2] - b[f]) * f:>+14,}")
(out / 'paths.txt').write_text('\n'.join(lines) + '\n')

# dynamic stack traffic
MEM = re.compile(r'(-?0x[0-9a-f]+)?\(%rsp(,%r[0-9a-z]+,[1248])?\)')


def stack_traffic(L, counts):
    t = collections.Counter()
    for a, ins in L.items():
        n = counts.get(a, {}).get('Ir', 0)
        if not n:
            continue
        op, _, args = ins.partition(' ')
        if op.startswith(('push', 'pop')):
            t['push/pop'] += n
            continue
        m = MEM.search(args)
        if not m or op.startswith(('lea', 'nop')):
            continue
        kind = 'array' if m.group(2) else 'slot'
        operands = args.strip()
        dst = operands.rsplit(',', 1)[-1] if ',' in operands else operands
        is_store = bool(MEM.search(dst)) and ',' in operands and not op.startswith(('cmp', 'test'))
        rmw = is_store and op.startswith(('add', 'sub', 'inc', 'dec', 'or', 'and', 'xor', 'shl', 'shr'))
        if is_store:
            t[f'{kind} stores'] += n
            if rmw:
                t[f'{kind} loads'] += n
        else:
            t[f'{kind} loads'] += n
    return t


rows = ['# Dynamic stack traffic (primary corpus, own code): executions of instructions with a plain disp(%rsp)',
        '# operand ("slot": spills, reloads and stack-held scan state), with an indexed (%rsp,%reg,s) operand',
        '# ("array": the descriptor arrays wr/en/cold, which live in W on the stack), and push/pop.',
        '# Each build against its own same-binary B2 profile.']
for k, (d, b2files, cfiles) in LIST.items():
    pb, pc = prof[k]
    for g, files in cfiles.items():
        c_t = collections.Counter()
        for name, f in zip(BUILDS[k][2][g], files):
            c_t += stack_traffic(asm(d / f), pc['addresses'].get(name, {}))
        bf = b2files.get(g)
        if not bf:
            rows.append(f"{k:10} {g:7} (no same-binary B2 listing archived for this group; its own Ir is unchanged against B2)")
            continue
        bpath = d / bf
        b_t = stack_traffic(asm(bpath), pb['addresses'].get(B2[g], {}))
        keys = sorted(b_t.keys() | c_t.keys())
        rows.append(f"{k:10} {g:7} " + '; '.join(f"{x} {b_t[x]:,} -> {c_t[x]:,} ({c_t[x] - b_t[x]:+,})" for x in keys))
(out / 'stack.txt').write_text('\n'.join(rows) + '\n')
print((out / 'paths.txt').read_text())
print((out / 'stack.txt').read_text())
