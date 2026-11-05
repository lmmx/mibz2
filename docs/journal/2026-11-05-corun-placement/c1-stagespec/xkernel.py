import os
# Cross-kernel prior: F_walk(kernel) vs the sibling's walk time share p in that kernel's passes.
# If pair factors were kernel-independent, F = f_WN + p (f_WW - f_WN): slope = f_WW - f_WN.
import sys, re, glob, statistics as st
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "../phases"))
base = os.path.join(os.path.dirname(os.path.abspath(__file__)), "../phases") + "/"
rows = ["scanner (headers, Huffman)","MTF + RLE2","T-vector build","walk, room (used)","RLE1","CRC",
        "walk, flagged (before #37)","walk, bitmap (before #4)","walk, plain"]
def parse(f):
    d = {}
    for l in open(f):
        for r in rows:
            if l.strip().startswith(r):
                d[r] = float(re.search(r"([0-9.]+) c/B", l[l.index(r)+len(r):]).group(1))
    return d
for run, name in [("sweep-4bfa", "7763"), ("sweep-42020095581", "9V74"), ("sweep-b773", "8370C")]:
    a = parse(base + run + "/sweep-1.txt"); cs = [parse(f) for f in sorted(glob.glob(base + run + "/sweep-4-cpu*.txt"))]
    m = {r: st.mean(c[r] for c in cs) for r in rows}
    n4 = sum(m[r] for r in rows[:3] + rows[4:6]); n1 = sum(a[r] for r in rows[:3] + rows[4:6])
    pts = []
    for k in ["walk, room (used)", "walk, flagged (before #37)", "walk, bitmap (before #4)"]:
        p = m[k] / (m[k] + n4); F = m[k] / a[k]; pts.append((p, F))
        print(f"{name:6} {k:28} p {p:.3f} F {F:.3f}")
    (p0, F0), (p2, F2) = pts[0], pts[2]
    slope = (F2 - F0) / (p2 - p0); fwn = F0 - p0 * slope; fww = fwn + slope
    sW = m["walk, room (used)"] / (m["walk, room (used)"] + n4)
    walk_term = -sW * (F0 - fwn) / F0
    print(f"{name:6} room->bitmap slope {slope:.2f}: fWN {fwn:.2f} fWW {fww:.2f}; walk-term gain {walk_term*100:+.1f}% (sW {sW:.3f}); non-walk mixed {n4/n1:.2f}")
