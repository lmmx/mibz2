"""Library calls from build::<false>, per call site (positions: instr line).
usage: sites.py PROFILE"""
import gzip
import re, sys, collections
names = {}; fn = cfn = None; pend = None; last = [0, 0]; ev = None
def nm(k, s):
    m = re.match(r"\((\d+)\)(?: (.*))?$", s)
    if not m: return s
    if m.group(2) is not None: names[(k, m.group(1))] = m.group(2)
    return names[(k, m.group(1))]
def sub(t, p):
    if t == '*': return p
    if t[0] in '+-': return p + int(t, 0)
    return int(t, 0)
sites = collections.defaultdict(lambda: [0, 0, 0, 0])
for line in (gzip.open(sys.argv[1], 'rt') if sys.argv[1].endswith('.gz') else open(sys.argv[1])):
    line = line.rstrip('\n')
    if line.startswith('positions:'): assert line.split(':')[1].split() == ['instr', 'line']; continue
    if line.startswith('events:'): ev = line.split(':')[1].split(); continue
    if not line: continue
    k, s, v = line.partition('=')
    if s and k in ('fn', 'cfn', 'fl', 'fi', 'fe', 'cfl', 'cfi', 'ob', 'cob'):
        x = nm('fn' if k in ('fn', 'cfn') else ('ob' if k in ('ob', 'cob') else 'fl'), v)
        if k == 'fn': fn = x
        if k == 'cfn': cfn = x
        continue
    if k == 'calls' and s: pend = int(v.split()[0]); continue
    p = line.split()
    if not (p[0][0].isdigit() or p[0][0] in '+-*'): continue
    last = [sub(p[0], last[0]), sub(p[1], last[1])]
    if pend is not None:
        if fn and 'build::<false>' in fn and not re.search(r'refill|finish|canonicalise', cfn):
            r = sites[(cfn, hex(last[0]))]; r[0] += pend
            for i, e in enumerate(('Ir', 'Dr', 'Dw')): r[i + 1] += int(p[2 + ev.index(e)])
        pend = None
for (c, a), (n, ir, dr, dw) in sorted(sites.items(), key=lambda kv: kv[0][1]):
    print(f"{a} {n:>6,} calls  Ir {ir:>10,} ({ir / n:9.1f}/call)  Dr {dr:>10,}  Dw {dw:>10,}  {c}")
