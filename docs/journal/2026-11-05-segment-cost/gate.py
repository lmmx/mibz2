"""The gate: the lean path's cost per segment, and the dense-start walk's total against
master's per byte in the SMT-pair (four-copy) cell, under the census's figures.

Inputs (all files under segcost/out):
- callgrind totals of master (cg-m512-w2-512K.out) and of the lean prototype at K = 8, 16, 32
  (cg-lean{3,4,5}-w2-512K.out), block 3 of the bench corpus (n 900,000);
- the trace simulation's L2 misses a byte in the SMT pair (sim-smt-1M-b1b233.txt for Zen 4/5's
  1 MiB L2, sim-smt-512K-b1b233.txt for Zen 3's 512 KiB), held-out blocks 1 + 233;
- the census's four-copy fit on the sweeps 41989592674 (9V45) and 41989596336 (9V74):
  t = a4 + lat / min(L, M), a4 = 4.49 / 5.68 c/B, lat/M = 2.03 / 3.43 c/B
  (docs/journal/2026-11-05-binding-census/walk.md:111, :125, :129; refit by
  census/C/scripts/fit.py on docs/journal/2026-11-05-coresident-walk/).
"""
import re
import sys

import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from cgparse import parse  # noqa: E402

O = os.path.dirname(os.path.abspath(__file__)) + "/out"
N = 900000


def sim(path, label_re):
    vals = []
    for line in open(path):
        if re.match(label_re, line):
            d = {"all": float(re.search(r"L2/B all ([\d.]+)", line)[1])}
            for k in ["tt", "flag", "tile", "wmeta", "segw", "jmeta", "segr", "j2w", "out"]:
                m = re.search(rf" {k} ([\d.]+)", line)
                d[k] = float(m[1]) if m else 0.0
            d["segs"] = int(re.search(r"segs +(\d+)", line)[1])
            vals.append(d)
    assert vals, label_re
    return {k: sum(v[k] for v in vals) / len(vals) for k in vals[0]}


def ir(path, fns=None):
    events, totals, by_fn, _ = parse(path)
    i = events.index("Ir")
    b = events.index("Bcm") if "Bcm" in events else None
    if fns is None:
        return totals[i], (totals[b] if b is not None else 0)
    s = sum(v[i] for f, v in by_fn.items() if any(x in (f or "") for x in fns))
    return s, None


def main():
    fit = {"9V45": (4.49, 2.03), "9V74": (5.68, 3.43)}
    m_ir, m_bcm = ir(f"{O}/cg-m512-w2-512K.out")
    print(f"master, 512 segments, block 3: Ir {m_ir:,} = {m_ir / N:.2f} Ir/B, Bcm {m_bcm:,}")
    lean = {}
    for bits in (3, 4, 5):
        t, bcm = ir(f"{O}/cg-lean{bits}-w2-512K.out")
        w, _ = ir(f"{O}/cg-lean{bits}-w2-512K.out", ["lean_walk"])
        j, _ = ir(f"{O}/cg-lean{bits}-w2-512K.out", ["lean_join", "memcpy", "memmove"])
        lean[bits] = (t, w, j, bcm)
        print(f"lean K={1 << bits}: Ir {t:,} = {t / N:.2f} Ir/B (walk {w / N:.2f}, join + copies {(t - w) / N:.2f}), Bcm {bcm:,}")
    for l2, path in [("1 MiB (Zen 4/5)", "sim-smt-1M-b1b233.txt"), ("512 KiB (Zen 3)", "sim-smt-512K-b1b233.txt")]:
        ms = sim(f"{O}/{path}", r"mfull L32 S512 ")
        print(f"\nSMT pair, L2 {l2}: master (mfull S512) {ms['all']:.4f} L2 misses/B")
        for kind in ("lean", "lean2"):
            for bits in (2, 3, 4):
                ls = sim(f"{O}/{path}", rf"{kind} L32 K{1 << bits} K2_64 L2_16 ")
                dmiss = ls["all"] - ms["all"]
                segs_b = ls["segs"] / N
                row = f"  {kind:5s} K={1 << bits:2d}: {ls['all']:.4f} misses/B ({dmiss:+.4f}; per segment {dmiss / segs_b:+.2f})"
                for cpu, (a4, mem) in fit.items():
                    cm = mem / ms["all"]
                    dmc = dmiss * cm
                    if kind == "lean" and bits in lean:
                        dir_b = (lean[bits][0] - m_ir) / N
                        cpi_scaled = a4 / (m_ir / N)
                        d_scaled = dir_b * cpi_scaled
                        row += f" | {cpu}: c/miss {cm:.2f}, misses {dmc:+.3f} c/B, Ir {dir_b:+.2f}/B -> {d_scaled:+.3f} c/B (a4 scaled), total {dmc + d_scaled:+.3f} c/B ({100 * (dmc + d_scaled) / (a4 + mem):+.1f}%)"
                    else:
                        row += f" | {cpu}: misses {dmc:+.3f} c/B"
                print(row)


if __name__ == "__main__":
    main()
