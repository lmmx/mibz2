"""Signed additivity bridge for assembly 1, from gate2/paths.txt (same-binary class table).

For every execution-count class: d_s = scan - B2, d_d = desc - B2, d_c = comb - B2 (instructions
per execution), and nonadditivity n = d_c - (d_s + d_d), each weighted by the class frequency.
v3 has no column in paths.txt: its classes are the finish group, where the scanner and descriptor
change nothing, so the finish group's combined delta is v3's contribution.
Usage: python3 bridge.py paths.txt
"""
import sys
rows = []
for line in open(sys.argv[1]):
    t = line.split()
    if not t or line.startswith("#") or t[0] == "group" or len(t) < 7:
        continue
    try:
        f = int(t[1].replace(",", ""))
        b, s, d, c = (int(x.rstrip("*")) for x in t[2:6])
    except ValueError:
        continue
    rows.append((t[0], f, b, s, d, c))
G = {}
for g, f, b, s, d, c in rows:
    a = G.setdefault(g, [0, 0, 0, 0])
    a[0] += (s - b) * f; a[1] += (d - b) * f; a[2] += (c - b) * f
    a[3] += ((c - b) - ((s - b) + (d - b))) * f
print("group    scanner Δ       descriptor Δ    combined Δ      nonadditive")
for g, (s, d, c, n) in G.items():
    print(f"{g:7} {s:+15,} {d:+15,} {c:+15,} {n:+15,}")
own = sum(v[2] for v in G.values())
print(f"\nown Δ {own:+,}")
print(f"  = scanner {G['build'][0]:+,}"
      f" + descriptor {G['build'][1] + G['refill'][1]:+,} (build {G['build'][1]:+,}, refill {G['refill'][1]:+,})"
      f" + v3 (finish group) {G['finish'][2]:+,}"
      f" + build nonadditivity {G['build'][3]:+,}"
      f" + refill nonadditivity {G['refill'][3]:+,}")
print("\nbuild-group classes with nonzero nonadditivity (freq, B2, scan, desc, comb, n×freq):")
nz = [(f, b, s, d, c, ((c - b) - ((s - b) + (d - b))) * f) for g, f, b, s, d, c in rows if g == "build"]
nz = [r for r in nz if r[5]]
for f, b, s, d, c, n in sorted(nz, key=lambda r: r[5]):
    print(f"  {f:>11,}  {b:>3} {s:>3} {d:>3} {c:>3}  {n:+15,}")
neg = sum(r[5] for r in nz if r[5] < 0); pos = sum(r[5] for r in nz if r[5] > 0)
print(f"  removed beyond additive {neg:+,}; added beyond additive {pos:+,}; net {neg + pos:+,} over {len(nz)} classes")

# Attribution of the build-group nonadditivity, by mechanism (classes are disjoint):
#   unroll   - code that exists only in the combined build (B2 = scan = desc = 0 < comb), and the
#              row-loop classes the unroll empties (comb = 0 < max(scan, desc)); the 16 unrolled
#              row positions and their refill detours replace the looped rows.
#   head     - the outer loop head the descriptor lengthened and the combined build did not
#              (freq 3,003,834: scan 11, desc 17, comb 11).
#   s_class  - the per-epoch S class (freq 2,393,313, the O class): comb 44 vs scan 37 / desc 37.
#   rest     - every other class.
cat = {"unroll": 0, "head": 0, "s_class": 0, "rest": 0}
for f, b, s, d, c, n in nz:
    if (b == s == d == 0 and c > 0) or (c == 0 and max(s, d) > 0):
        cat["unroll"] += n
    elif f == 3_003_834:
        cat["head"] += n
    elif f == 2_393_313:
        cat["s_class"] += n
    else:
        cat["rest"] += n
print("\nattribution of build nonadditivity:")
for k, v in cat.items():
    print(f"  {k:8} {v:+15,}")
print(f"  {'total':8} {sum(cat.values()):+15,}")
