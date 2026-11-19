"""The successor's guard rule G2, planned from the first attempt's per-cell noise.

The noise is open-port/run-20261119T125538Z's; the Researcher's correction is REVIEW 7012181194,
and the Coordinator's decisions QUEUED 7012199074.

    python3 plan.py RUN_DIR OUT.json 100000   (numpy and scipy; planning only, not run by the recipe;
    bench/open-port-2-schedule.json is OUT.json's rounds and c)

G2, the scorer's own rule: a guard cell fails if the mean of its n rounds' paired log ratios (the
port's wall over v0.5.6's; its exponential is the geometric mean, G's estimand) exceeds a critical
value c fixed here, before the data. c is the largest of the (1 - 0.048/40) quantiles of that mean
under no effect in three noise models, each resampled n times with replacement: the cell's own 12
residuals (centred on its mean), its corpus's pooled 36, and all 468 guard residuals (which carry
the first attempt's spike rate). The rounds n are the smallest on the grid giving power 0.80 at a
true +2% slowdown under the cell's own residuals.

The output also gives each cell's power at +1%, +2% and +3%, the first registration's rule G at the
same effects (12 rounds), the sizes under each model and under a stress model (2% of rounds a
spike of 0.2 to 0.4 either way), and the restored S's false-fail rate and power.

The models assume the rounds are independent within a cell (the paired design takes out what the
two arms share in a round; serial dependence is not modelled). They are planning models: G2's
size is calibrated under them, not exact.
"""

import json, math, collections, sys
import numpy as np
from scipy import stats

RUN, OUT = sys.argv[1], sys.argv[2]
R = int(sys.argv[3]) if len(sys.argv) > 3 else 200_000
GRID = [12, 24, 36, 48, 60, 72, 96, 120, 144, 168, 192, 240, 288, 336, 384, 480, 600]
L1, L2, L3 = math.log(1.01), math.log(1.02), math.log(1.03)


def cells_of(path):
    """{(corpus, threads): the rounds' paired log ratios, the port over v0.5.6}."""
    d = collections.defaultdict(dict)
    for line in open(path):
        r = json.loads(line)
        d[(r["corpus"], r["threads"], r["round"])]["c" if "/build-port/" in r["arm"] else "r"] = r
    out = collections.defaultdict(list)
    for (c, t, i), v in sorted(d.items()):
        out[(c, t)].append(math.log(v["c"]["wall"] / v["r"]["wall"]))
    return {k: np.array(v) for k, v in out.items()}


cells = cells_of(f"{RUN}/runs.jsonl")
sus = cells_of(f"{RUN}/sustained.jsonl")[("wikidata-x16", 10)]
G = sorted(k for k in cells if k[0] not in ("wikidata-x4", "ho-wikidata-l9"))
own = {k: cells[k] - cells[k].mean() for k in G}
pooled = {c: np.concatenate([own[k] for k in G if k[0] == c]) for c in {k[0] for k in G}}
allres = np.concatenate([own[k] for k in G])
# S joins G2's family (the Coordinator's decision, 19 November): its own residuals; its "corpus" is
# itself; and the all-guard pool, as for every guard
S = ("wikidata-x16", 10)
own[S] = sus - sus.mean()
pooled[S[0]] = own[S]
CELLS = G + [S]
A = 0.048 / len(CELLS)
rng = np.random.default_rng(20261119)


def means(res, n, spike=0.0):
    """R means of n rounds resampled from res.

    With `spike`, each round is instead, with probability `spike`, a symmetric spike of 0.2 to 0.4
    in log ratio.
    """
    out = np.empty(R)
    step = max(1, 4_000_000 // n)
    for i in range(0, R, step):
        j = min(R, i + step)
        x = rng.choice(res, size=(j - i, n))
        if spike:
            hit = rng.random(x.shape) < spike
            x = np.where(hit, rng.uniform(0.2, 0.4, x.shape) * rng.choice((-1, 1), x.shape), x)
        out[i:j] = x.mean(1)
    return out


def g_rule(res, n, d, reps=100_000):
    """The registered G (or S): the mean above ln 1.02, or the 95% t-interval wholly above 0."""
    x = d + rng.choice(res, size=(reps, n))
    m = x.mean(1)
    se = x.std(1, ddof=1) / math.sqrt(n)
    return float(((m > L2) | (m - stats.t.ppf(0.975, n - 1) * se > 0)).mean())


plan, nulls = [], {"own": [], "pooled": [], "spiky": [], "all": []}
for k in CELLS:
    for n in GRID:
        mo, mp, ma = means(own[k], n), means(pooled[k[0]], n), means(allres, n)
        c = float(max(np.quantile(mo, 1 - A), np.quantile(mp, 1 - A), np.quantile(ma, 1 - A)))
        if (mo + L2 > c).mean() >= 0.80 or n == GRID[-1]:
            break
    ms = means(own[k], n, spike=0.02)
    row = {
        "corpus": k[0],
        "threads": k[1],
        "rounds": n,
        "c": round(c + 5e-5, 4),
        "sd": float(own[k].std(ddof=1)),
        "g2": [float((mo + d > c).mean()) for d in (L1, L2, L3)],
        "g2_pooled_2": float((mp + L2 > c).mean()),
        "g": [g_rule(own[k], 8 if k == S else 12, d) for d in (L1, L2, L3)],
        "size": {
            "own": float((mo > c).mean()),
            "pooled": float((mp > c).mean()),
            "spiky": float((ms > c).mean()),
            "all": float((ma > c).mean()),
        },
    }
    for m in nulls:
        nulls[m].append(row["size"][m])
    plan.append(row)
    print(
        f"{k[0]:17s} {k[1]:2d}t n {n:3d} c {row['c']:.4f} | G2 +1/2/3% {row['g2'][0]:.2f} {row['g2'][1]:.2f} {row['g2'][2]:.2f}"
        f" (pooled +2% {row['g2_pooled_2']:.2f}) | G {row['g'][0]:.2f} {row['g'][1]:.2f} {row['g'][2]:.2f}"
        f" | size own {row['size']['own']:.4f} pooled {row['size']['pooled']:.4f} spiky {row['size']['spiky']:.4f}",
        flush=True,
    )
sres = sus - sus.mean()
s = {"null": g_rule(sres, 8, 0.0, 400_000), "+2%": g_rule(sres, 8, L2, 400_000)}
fam = {m: 1 - float(np.prod([1 - v for v in nulls[m]])) for m in nulls}
print(
    f"G2 family ({len(CELLS)} cells) P(any fails), no effect: own {fam['own']:.4f}, pooled {fam['pooled']:.4f}, all-guard {fam['all']:.4f}, spiky (2% spikes) {fam['spiky']:.4f}"
)
print(f"S as registered (8 rounds): P(fails), no effect {s['null']:.3f}; at +2% {s['+2%']:.3f}")
ext = sum(r["rounds"] - 12 for r in plan if (r["corpus"], r["threads"]) != S)
sn = next(r["rounds"] for r in plan if (r["corpus"], r["threads"]) == S)
print(f"guard extension pairs {ext}: at 2.1 s a decode, {ext * 2 * 2.1 / 3600:.2f} h")
print(f"S: {sn} rounds back to back, about {sn * 2 * 1.2 / 60:.0f} min")
json.dump(
    {"alpha": A, "cells": plan, "family": fam, "S": s, "extension_pairs": ext},
    open(OUT, "w"),
    indent=1,
)
