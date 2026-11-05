"""The per-segment anatomy from callgrind outputs (cgparse.parse): master's room walk at
512 / 2048 / 16384 / 131072 segments, each event by phase, as totals and as the change per
added segment against 512.

usage: python3 -I anat.py OUTDIR TAG NS512 NS2048 NS16384 NS131072
  (TAG: w2-512K, w0-512K, ...; NS*: the block's segment counts after dedup)
"""
import sys
from collections import defaultdict

import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from cgparse import parse  # noqa: E402

PHASES = [
    # (name, first line, last line) in src/bwt.rs at d1fd4405
    ("assemble", 455, 472),
    ("random_rows", 1280, 1291),
    ("setup", 887, 902),
    ("rr_call", 903, 904),
    ("sort", 905, 905),
    ("dedup", 906, 906),
    ("flag_set", 907, 910),
    ("bufs_setup", 911, 916),
    ("next_init", 917, 924),
    ("take", 925, 937),
    ("finish", 938, 943),
    ("grow", 944, 952),
    ("lanes_init", 953, 971),
    ("flush_fn", 984, 996),
    ("pass", 972, 1016),
    ("step", 1017, 1062),
    ("handover", 1063, 1147),
    ("chunk_flush", 1148, 1154),
    ("tail", 1155, 1192),
    ("flag_clear", 1193, 1195),
    ("end", 1196, 1199),
    ("next_unset", 1270, 1279),
]


def phase_of(f, ln, fn):
    if f and f.endswith("src/bwt.rs"):
        for name, a, b in PHASES:
            if a <= ln <= b:
                return name
        return "bwt_other"
    fnl = (fn or "").lower()
    if "memcpy" in fnl or "memmove" in fnl:
        return "libc_memcpy"
    if "malloc" in fnl or "realloc" in fnl or "free" in fnl or "grow" in fnl or "reserve" in fnl:
        return "alloc"
    if "sort" in fnl:
        return "sort"
    f = f or ""
    if "slice/sort" in f:
        return "sort"
    if "slice/mod.rs" in f or "cmp.rs" in f:
        return "core_slice(bsearch)"
    if "/alloc/" in f or "raw_vec" in f or "vec/" in f:
        return "alloc_inl"
    return "lib:" + (f.split("/")[-1] if f else "?")


def by_phase(path):
    events, totals, by_fn, by_line = parse(path)
    out = defaultdict(lambda: [0] * len(events))
    # self cost by line carries (file, line); functions not inlined are split by fn name:
    # re-walk with function attribution: cgparse keys lines by (fi, ln) only, so use the
    # function table for out-of-line library functions and the line table for bwt.rs
    fn_of_line = None
    _ = fn_of_line
    for (f, ln), v in by_line.items():
        if f and f.endswith("src/bwt.rs"):
            ph = phase_of(f, ln, None)
            for i, x in enumerate(v):
                out[ph][i] += x
    bwt_total = [sum(v[i] for (f, _), v in by_line.items() if f and f.endswith("src/bwt.rs")) for i in range(len(events))]
    # the rest by function
    rest = [t - b for t, b in zip(totals, bwt_total)]
    fn_rest = defaultdict(lambda: [0] * len(events))
    for fn, v in by_fn.items():
        fn_rest[fn] = v
    return events, totals, out, rest, by_fn, by_line


def main():
    d, tag = sys.argv[1], sys.argv[2]
    ns = [int(x) for x in sys.argv[3:7]]
    segl = [512, 2048, 16384, 131072]
    res = {}
    for s in segl:
        res[s] = by_phase(f"{d}/cg-m{s}-{tag}.out")
    events = res[512][0]
    show = [e for e in ["Ir", "Bcm", "Bim", "D1mr", "D1mw", "DLmr", "DLmw"] if e in events]
    idx = [events.index(e) for e in show]
    print("totals (whole measured call: room_segments + assemble + their callees)")
    for s in segl:
        t = res[s][1]
        print(f"  segs {s:6d} ns {ns[segl.index(s)]:6d}: " + " ".join(f"{e} {t[i]:,}" for e, i in zip(show, idx)))
    for s in segl[1:]:
        dn = ns[segl.index(s)] - ns[0]
        t1, t0 = res[s][1], res[512][1]
        print(f"  per added segment, 512 -> {s}: " + " ".join(f"{e} {(t1[i] - t0[i]) / dn:.2f}" for e, i in zip(show, idx)))
    # phases (bwt.rs lines)
    names = sorted(set().union(*[set(res[s][2].keys()) for s in segl]))
    print("\nbwt.rs self cost by phase (per added segment vs 512, then the 512 total)")
    for nm in names:
        row = []
        for s in segl[1:]:
            dn = ns[segl.index(s)] - ns[0]
            a, b = res[s][2].get(nm, [0] * len(events)), res[512][2].get(nm, [0] * len(events))
            row.append(" ".join(f"{(a[i] - b[i]) / dn:7.2f}" for i in idx))
        tot = res[512][2].get(nm, [0] * len(events))
        print(f"  {nm:12s} | " + " | ".join(row) + " || 512: " + " ".join(f"{tot[i]:,}" for i in idx))
    # functions other than room_segments' own lines: by function name, top movers
    print("\nfunctions (self cost), per added segment 512 -> 2048 and -> 16384, and the 512 total")
    fns = set(res[512][4].keys()) | set(res[2048][4].keys()) | set(res[16384][4].keys())
    rows = []
    for fn in fns:
        a0 = res[512][4].get(fn) or [0] * len(events)
        a1 = res[2048][4].get(fn) or [0] * len(events)
        a2 = res[16384][4].get(fn) or [0] * len(events)
        rows.append((fn, a0, a1, a2))
    ir = events.index("Ir")
    rows.sort(key=lambda r: -abs(r[3][ir] - r[1][ir]))
    for fn, a0, a1, a2 in rows[:14]:
        d1 = ns[1] - ns[0]
        d2 = ns[2] - ns[0]
        print(f"  {fn[:70]:70s} | " + " ".join(f"{(a1[i] - a0[i]) / d1:7.2f}" for i in idx) + " | " + " ".join(f"{(a2[i] - a0[i]) / d2:7.2f}" for i in idx) + " || " + " ".join(f"{a0[i]:,}" for i in idx))
    print("columns:", " ".join(show))


if __name__ == "__main__":
    main()
