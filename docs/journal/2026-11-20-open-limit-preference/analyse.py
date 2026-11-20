"""The thin-run limit's per-block preference: the rule fixed in the brief (Step 2).

Usage: python3 analyse.py rounds.csv features.csv
Standard library only.
"""
import csv, math, random, statistics as st, sys
from collections import defaultdict

LS = [8, 12, 16, 24]
OTHER = [8, 12, 24]
T975 = {19: 2.093, 39: 2.023}  # two-sided 95% t quantiles

rounds_csv, feats_csv = sys.argv[1], sys.argv[2]
t = defaultdict(dict)  # (block) -> {(round, L): ns}
for r in csv.DictReader(open(rounds_csv)):
    t[int(r["block"])][(int(r["round"]), int(r["limit"]))] = int(r["ns"])
blocks = sorted(t)
R = 1 + max(k[0] for k in t[blocks[0]])
print(f"blocks {len(blocks)}, rounds {R}")


def d(b, L, rr):
    """log(t_L / t_16) in each round of rr"""
    return [math.log(t[b][(r, L)] / t[b][(r, 16)]) for r in rr]


def med(b, L, rr):
    return st.median(t[b][(r, L)] for r in rr)


def pearson(x, y):
    mx, my = st.fmean(x), st.fmean(y)
    sxy = sum((a - mx) * (b - my) for a, b in zip(x, y))
    return sxy / math.sqrt(sum((a - mx) ** 2 for a in x) * sum((b - my) ** 2 for b in y))


odd = [r for r in range(R) if r % 2 == 1]
even = [r for r in range(R) if r % 2 == 0]

print("\n1. split-half reliability of mean log(t_L/t_16), odd vs even rounds")
sh = {}
for L in OTHER:
    x = [st.fmean(d(b, L, odd)) for b in blocks]
    y = [st.fmean(d(b, L, even)) for b in blocks]
    sh[L] = pearson(x, y)
    sb = 2 * sh[L] / (1 + sh[L])
    sd = st.pstdev([st.fmean(d(b, L, range(R))) for b in blocks])
    print(f"  L={L:2d}: r = {sh[L]:+.3f} (Spearman-Brown, all rounds {sb:+.3f}); "
          f"sd across blocks of the mean log ratio {sd:.4f}")

print("\n2. per-block paired test, mean log ratio over all rounds, 95% t interval")
tq = T975.get(R - 1, 1.96)
faster = defaultdict(set)
slower = defaultdict(set)
for b in blocks:
    for L in OTHER:
        x = d(b, L, range(R))
        m, se = st.fmean(x), st.stdev(x) / math.sqrt(R)
        if m + tq * se < 0:
            faster[L].add(b)
        if m - tq * se > 0:
            slower[L].add(b)
anyf = set().union(*faster.values())
print(f"  some L != 16 faster: {len(anyf)} of {len(blocks)} ({len(anyf)/len(blocks):.1%})")
for L in OTHER:
    print(f"    L={L:2d}: faster on {len(faster[L])}, slower on {len(slower[L])}")
# Bonferroni over the 3 limits
tqb = 2.508 if R == 40 else 2.0  # t_{0.99583, 39}
anyb = set()
for b in blocks:
    for L in OTHER:
        x = d(b, L, range(R))
        if st.fmean(x) + tqb * st.stdev(x) / math.sqrt(R) < 0:
            anyb.add(b)
print(f"  with Bonferroni over the 3 limits: {len(anyb)} ({len(anyb)/len(blocks):.1%})")

print("\n3. honest oracle: L chosen per block on one half's medians, scored on the other's")
for name, sel, ev in (("choose odd, score even", odd, even), ("choose even, score odd", even, odd)):
    ratios, num, den, pick = [], 0.0, 0.0, defaultdict(int)
    for b in blocks:
        L = min(LS, key=lambda L: med(b, L, sel))
        pick[L] += 1
        a, z = med(b, L, ev), med(b, 16, ev)
        ratios.append(a / z)
        num += a
        den += z
    print(f"  {name}: mean ratio {st.fmean(ratios):.4f}, total-weighted {num/den:.4f}, "
          f"picks {dict(sorted(pick.items()))}")
# in-sample oracle, for contrast
num = sum(min(med(b, L, range(R)) for L in LS) for b in blocks)
den = sum(med(b, 16, range(R)) for b in blocks)
print(f"  in-sample oracle (not honest), all rounds: total-weighted {num/den:.4f}")

print("\n4. each fixed L against 16, total-time weighted (medians over all rounds)")
for L in LS:
    tot = sum(med(b, L, range(R)) for b in blocks) / sum(med(b, 16, range(R)) for b in blocks)
    # per-round totals, paired: interval across rounds
    pr = [math.log(sum(t[b][(r, L)] for b in blocks) / sum(t[b][(r, 16)] for b in blocks)) for r in range(R)]
    m, se = st.fmean(pr), st.stdev(pr) / math.sqrt(R) if L != 16 else 0
    print(f"  L={L:2d}: {tot:.4f}  (per-round totals: {math.exp(m):.4f}, "
          f"95% {math.exp(m - tq*se):.4f}..{math.exp(m + tq*se):.4f})")
tot16 = sum(med(b, 16, range(R)) for b in blocks)
print(f"  total at 16: {tot16/1e6:.1f} ms over {len(blocks)} blocks")

print("\n5. the earlier sign, re-read: blocks where median t_8 < median t_16")
w8 = [b for b in blocks if med(b, 8, range(R)) < med(b, 16, range(R))]
w4 = [b for b in w8 if med(b, 8, odd) < med(b, 16, odd) and med(b, 8, even) < med(b, 16, even)]
print(f"  i8 < i16 on {len(w8)} blocks; in both halves on {len(w4)}")
# prediction of the earlier 2-lap sign: use only rounds 0,1 per block
for L in OTHER:
    s2 = [st.fmean(d(b, L, [0, 1])) for b in blocks]
    full = [st.fmean(d(b, L, range(2, R))) for b in blocks]
    print(f"  L={L:2d}: corr(2-round sign, the other {R-2} rounds) = {pearson(s2, full):+.3f}")
