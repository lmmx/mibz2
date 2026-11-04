"""Opcode-level listing diff of v2's build::<false> against a prototype's, from one
binary, with the v2 (ctor2.rs) and prototype source lines of each changed instruction.
usage: ldiff.py BINARY V2_SYMBOL PROTO_SYMBOL V2_FILE PROTO_FILE"""
import sys, re, difflib
sys.path.insert(0, 'bench')
import c30_build_listing as L
B, va, pb, fa, fb = sys.argv[1:6]
syms = L.symbols(B)
def get(name, fname):
    st, sz, _ = [s for s in syms if s[2] == name][0]
    t = L.disassemble(B, st, sz, lines=True)
    out = []; cur = None
    for l in t.splitlines():
        m = re.match(r'^(/\S+|\S+\.rs):(\d+)', l)
        if m:
            cur = int(m.group(2)) if m.group(1).endswith(fname) else None
            continue
        m = L.INSN.match(l)
        if m:
            out.append((int(m.group(1), 16), re.sub(r'\s*#.*$', '', m.group(2)).rstrip(), cur))
    n = L.normalise([(a, t) for a, t, _ in out], st, sz)
    n = [re.sub(r'T[+-]\d+', 'T', re.sub(r'[Cc]tor2[ds]?', 'CtorX', x)) for x in n]
    return st, out, n
a = get(va, fa); b = get(pb, fb)
print('v2', len(a[1]), 'proto', len(b[1]))
sm = difflib.SequenceMatcher(None, a[2], b[2], autojunk=False)
for op, i1, i2, j1, j2 in sm.get_opcodes():
    if op == 'equal': continue
    la = sorted({x[2] for x in a[1][i1:i2] if x[2]}); lb = sorted({x[2] for x in b[1][j1:j2] if x[2]})
    print(f"{op} v2[{i1}:{i2}] +{(a[1][i1][0]-a[0]) if i1 < len(a[1]) else 0:#x} lines {la} | proto[{j1}:{j2}] +{(b[1][j1][0]-b[0]) if j1 < len(b[1]) else 0:#x} lines {lb}")
    for x in a[1][i1:i2]: print(f"   - {x[0]-a[0]:#x} {x[1]} {x[2]}")
    for x in b[1][j1:j2]: print(f"   + {x[0]-b[0]:#x} {x[1]} {x[2]}")
