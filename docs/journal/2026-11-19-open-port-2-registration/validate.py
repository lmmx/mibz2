"""Validate the locked successor schedule on fresh draws (the Researcher's HOLD 7013288271).

    python3 validate.py RUN_DIR SCHEDULE.json OUT.json [DRAWS] [SEED]

For each of the 40 cells of the schedule, with its locked rounds n and its serialised cutoff c
(the judge's own): G2's power at a true +1%, +2% and +3% slowdown under the cell's own residuals;
the first registration's rule's power G under the same residuals (12 rounds, S's 8); and the
size under each planning model and the stress model. Draws are fresh: another seed than the
planner's. A cell passes if its +2% power is at least max(0.80, G - 0.005).
"""

import collections
import json
import math
import sys

import numpy as np
from scipy import stats

RUN, SCHED, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
R = int(sys.argv[4]) if len(sys.argv) > 4 else 400_000
rng = np.random.default_rng(int(sys.argv[5]) if len(sys.argv) > 5 else 7013288271)
L1, L2, L3 = math.log(1.01), math.log(1.02), math.log(1.03)
TOL = 0.005


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
S = ("wikidata-x16", 10)
own[S] = sus - sus.mean()
pooled[S[0]] = own[S]


def means(res, n, spike=0.0):
    """R means of n rounds resampled from res.

    With `spike`, each round is instead, with probability `spike`, a symmetric spike of 0.2 to
    0.4.
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


def g_rule(res, n, d):
    """The first registration's G (or S).

    It fails on the mean above ln 1.02, or the 95% t-interval wholly above 0.
    """
    x = d + rng.choice(res, size=(R, n))
    m = x.mean(1)
    se = x.std(1, ddof=1) / math.sqrt(n)
    return float(((m > L2) | (m - stats.t.ppf(0.975, n - 1) * se > 0)).mean())


rows, short = [], []
for e in json.load(open(SCHED))["cells"]:
    k, n, c = (e["corpus"], e["threads"]), e["rounds"], e["c"]
    g = g_rule(own[k], 8 if k == S else 12, L2)
    mo = means(own[k], n)
    pw = [float((mo + d > c).mean()) for d in (L1, L2, L3)]
    target = max(0.80, g - TOL)
    size = {
        "own": float((mo > c).mean()),
        "pooled": float((means(pooled[k[0]], n) > c).mean()),
        "all": float((means(allres, n) > c).mean()),
        "spiky": float((means(own[k], n, spike=0.02) > c).mean()),
    }
    ok = pw[1] >= target
    rows.append(
        {
            "corpus": k[0],
            "threads": k[1],
            "rounds": n,
            "c": c,
            "G": g,
            "target": target,
            "g2": pw,
            "size": size,
            "ok": ok,
        }
    )
    if not ok:
        short.append(k)
    print(
        f"{k[0]:17s} {k[1]:2d}t n {n:3d} c {c:.4f} | +2% {pw[1]:.4f} target {target:.4f} {'ok' if ok else 'SHORT'} (G {g:.4f}) | +1% {pw[0]:.3f} +3% {pw[2]:.3f}",
        flush=True,
    )
fam = {
    m: 1 - float(np.prod([1 - r["size"][m] for r in rows]))
    for m in ("own", "pooled", "all", "spiky")
}
print(
    f"the lowest +2% power: {min(r['g2'][1] for r in rows):.4f}; cells short of target: {short or 'none'}"
)
print(
    f"P(any of 40 fails), no effect: own {fam['own']:.4f}, pooled {fam['pooled']:.4f}, all-guard {fam['all']:.4f}, stress {fam['spiky']:.4f}"
)
json.dump(
    {"draws": R, "cells": rows, "family": fam, "short": [list(k) for k in short]},
    open(OUT, "w"),
    indent=1,
)
