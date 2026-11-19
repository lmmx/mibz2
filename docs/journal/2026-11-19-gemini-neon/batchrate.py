"""Measure how often Neon's greedy decode can batch >= 2 fronts on real text."""
import sys
from collections import Counter
import verify_thm3 as v

v.MultiBoundaryDIET.check_invariants = lambda self: None  # speed: invariants already verified small

def run(text, K):
    n = len(text)
    chars = sorted(set(text)); acc = 0; bounds = []
    for c in chars:
        if 0 < acc < n: bounds.append(acc)
        acc += text.count(c)
    # choose up to K-1 cuts at roughly equal mass among the C[c] boundaries
    cuts = sorted({min(bounds, key=lambda b: abs(b - n * i / K)) for i in range(1, K)}) if bounds else []
    diet, p0 = v.build_bwt_diet(text, tuple(cuts))
    T0, W0, loc0 = diet.to_row_map()
    marks = {p0: (loc0[p0][0], loc0[p0][1], 0)}
    sizes = Counter(); loops = 0; r0 = len(diet.live)
    while diet.live:
        did = False
        for j in range(diet.K):
            pr = diet.front_pair(j)
            if pr is not None and pr[0] == pr[1]:
                diet.apply_self_loop(j, marks); loops += 1; did = True; break
        if did: continue
        batch = []
        for j in range(diet.K):
            if diet.is_admissible_batch(tuple(batch + [j])): batch.append(j)
        for j in batch: diet.apply_front(j, marks)
        sizes[len(batch)] += 1
    rounds = sum(sizes.values()); multi = sum(c for s, c in sizes.items() if s >= 2)
    steps = sum(s * c for s, c in sizes.items())
    print(f"n={n} r={r0} K={len(cuts)+1}: rounds={rounds} fronts-steps={steps} "
          f"rounds-with->=2={multi} ({100*multi/max(rounds,1):.1f}%) mean-batch={steps/max(rounds,1):.3f} sizes={dict(sorted(sizes.items()))}")

src = open(sys.argv[1], encoding="utf-8", errors="replace").read()
for n in (2000, 6000, 20000):
    t = src[:n]
    for K in (1, 4, 16, 32):
        run(t, K)
