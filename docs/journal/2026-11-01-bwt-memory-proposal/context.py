"""Read-only, correlational: each 4-worker walk's CPU time (relative to the same block's
1-worker walk) against the time-averaged number of other workers in each phase during it."""
import bisect, collections, json, statistics, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from phases import load, blocks  # noqa: E402
d = Path(sys.argv[1])
base = collections.defaultdict(list)
for f in sorted(d.glob("r??-t1-full.tsv")):
    for b in blocks(load(f)):
        if b.get("rows") == 900000 and "WalkStart" in b["m"] and "InvertEnd" in b["m"]:
            base[b["place"]].append(b["m"]["InvertEnd"][1] - b["m"]["WalkStart"][1])
base = {p: statistics.median(v) for p, v in base.items()}
X, Y = [], []
for f in sorted(d.glob("r??-t4-full.tsv")):
    bl = [b for b in blocks(load(f)) if all(k in b["m"] for k in ("DecodeStart", "InvertStart", "WalkStart", "InvertEnd", "DecodeEnd"))]
    # each worker's phase intervals
    iv = collections.defaultdict(list)
    for b in bl:
        m = b["m"]
        for name, a, z in (("mtf", "DecodeStart", "InvertStart"), ("build", "InvertStart", "WalkStart"), ("walk", "WalkStart", "InvertEnd"), ("post", "InvertEnd", "DecodeEnd")):
            iv[b["worker"]].append((m[a][0], m[z][0], name))
    for b in bl:
        if b.get("rows") != 900000 or b["place"] not in base: continue
        t0, t1 = b["m"]["WalkStart"][0], b["m"]["InvertEnd"][0]
        occ = collections.Counter()
        for w, lst in iv.items():
            if w == b["worker"]: continue
            for a, z, name in lst:
                o = min(z, t1) - max(a, t0)
                if o > 0: occ[name] += o / (t1 - t0)
        X.append([occ["build"], occ["walk"], occ["mtf"], occ["post"]]); Y.append((b["m"]["InvertEnd"][1] - b["m"]["WalkStart"][1]) / base[b["place"]])
# least squares Y = c + b·X (normal equations, 5x5)
import itertools
n = len(Y); k = 5
A = [[1.0] + x for x in X]
AtA = [[sum(A[r][i] * A[r][j] for r in range(n)) for j in range(k)] for i in range(k)]
Aty = [sum(A[r][i] * Y[r] for r in range(n)) for i in range(k)]
M = [row[:] + [v] for row, v in zip(AtA, Aty)]
for i in range(k):
    piv = max(range(i, k), key=lambda r: abs(M[r][i])); M[i], M[piv] = M[piv], M[i]
    for r in range(k):
        if r != i:
            f_ = M[r][i] / M[i][i]; M[r] = [a - f_ * b for a, b in zip(M[r], M[i])]
coef = [M[i][k] / M[i][i] for i in range(k)]
pred = [sum(c * a for c, a in zip(coef, row)) for row in A]
ss_res = sum((y - p) ** 2 for y, p in zip(Y, pred)); my = sum(Y) / n; ss_tot = sum((y - my) ** 2 for y in Y)
# bins by others in build
bins = collections.defaultdict(list)
for x, y in zip(X, Y): bins[min(3, round(x[0]))].append(y)
binsw = collections.defaultdict(list)
for x, y in zip(X, Y): binsw[min(3, round(x[1]))].append(y)
tot = [sum(x[i] for x in X) / n for i in range(4)]
print(json.dumps(dict(n=n, walk_ratio_median=round(statistics.median(Y), 3), mean_others=dict(zip(["build", "walk", "mtf", "post"], [round(t, 2) for t in tot])),
    ols=dict(zip(["intercept", "per_other_building", "per_other_walking", "per_other_in_mtf", "per_other_in_post"], [round(c, 3) for c in coef]), r2=round(1 - ss_res / ss_tot, 3)),
    by_others_building={k: (len(v), round(statistics.median(v), 3)) for k, v in sorted(bins.items())},
    by_others_walking={k: (len(v), round(statistics.median(v), 3)) for k, v in sorted(binsw.items())}), indent=1))
