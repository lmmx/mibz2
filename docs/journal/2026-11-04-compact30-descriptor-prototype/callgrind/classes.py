"""Decompose build::<false>'s own instructions by execution-count class.

usage: classes2.py BINARY V2_PROFILE D_PROFILE   (run from the worktree root)

Each profile is a callgrind profile with `positions: instr line` from a process
that ran one builder alone. The per-instruction counts come from the profile,
the listing from the binary; instructions are grouped by their exact
execution count and the per-class instruction-count difference is printed.
"""
import collections
import re
import sys

sys.path.insert(0, "bench")
import c30_build_listing as L  # noqa: E402


def per_instr(path, fnpat):
    names = {}
    fn = None
    pending = False
    last = [0, 0]
    out = collections.Counter()

    def nm(k, s):
        m = re.match(r"\((\d+)\)(?: (.*))?$", s)
        if not m:
            return s
        if m.group(2) is not None:
            names[(k, m.group(1))] = m.group(2)
        return names[(k, m.group(1))]

    def sub(t, p):
        if t == "*":
            return p
        if t[0] in "+-":
            return p + int(t, 0)
        return int(t, 0)

    positions = None
    for line in open(path):
        line = line.rstrip("\n")
        if line.startswith("positions:"):
            positions = line.split(":", 1)[1].split()
            assert positions == ["instr", "line"], positions
            continue
        if not line:
            continue
        k, s, v = line.partition("=")
        if s and k in ("fn", "cfn", "fl", "fi", "fe", "cfl", "cfi", "ob", "cob"):
            x = nm("fn" if k in ("fn", "cfn") else ("ob" if k in ("ob", "cob") else "fl"), v)
            if k == "fn":
                fn = x
            continue
        if k == "calls" and s:
            pending = True
            continue
        if not (line[0].isdigit() or line[0] in "+-*"):
            continue
        p = line.split()
        last = [sub(p[0], last[0]), sub(p[1], last[1])]
        if not pending and fn == fnpat and len(p) > 2:
            out[last[0]] += int(p[2])
        pending = False
    return out


B, PV, PD = sys.argv[1:4]
syms = L.symbols(B)


def rows(name, prof):
    st, sz, _ = [s for s in syms if s[2] == name][0]
    c = per_instr(prof, name)
    ins = L.instructions(L.disassemble(B, st, sz))
    assert sum(c.values()) == sum(c.get(a, 0) for a, _ in ins)
    return [(a - st, t, c.get(a, 0)) for a, t in ins]


for va, da in [
    ("<c30walk::ctor2::Ctor2>::build::<false>", "<c30walk::ctor2d::Ctor2d>::build::<false>"),
    ("<c30walk::ctor2::W<false>>::refill", "<c30walk::ctor2d::W<false>>::refill"),
]:
    a, b = rows(va, PV), rows(da, PD)
    ha = collections.Counter(r[2] for r in a)
    hb = collections.Counter(r[2] for r in b)
    ta, tb = sum(r[2] for r in a), sum(r[2] for r in b)
    print(f"== {da} vs {va}: {ta:,} -> {tb:,}  delta {tb - ta:,}")
    lines = []
    for c in set(ha) | set(hb):
        dn = hb.get(c, 0) - ha.get(c, 0)
        if dn and c > 0:
            lines.append((dn * c, c, ha.get(c, 0), hb.get(c, 0)))
    lines.sort()
    small = 0
    for x, c, na, nb in lines:
        if abs(x) >= int(__import__("os").environ.get("MINX", "100000")):
            print(f"  count {c:>12,}: {na:4} -> {nb:4} instructions  {x:>14,}")
        else:
            small += x
    print(f"  classes with |delta| < 100,000: {small:,}")
