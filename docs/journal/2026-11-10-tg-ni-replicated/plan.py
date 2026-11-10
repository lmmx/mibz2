"""Planning the replicated-dispatch design (registration §2.11, DRAFT): the between-dispatch SD
of a cell's dispatch-level mean paired log ratio, from neutral evidence only, and the power of
the dispatch-level NI bound at a true ratio of 1.005.

Inputs, neither a candidate's:
- `links.jsonl` (bench-results at ebb793af): merged PRs' last paired CI runs. Kept: bench 3.x
  (the arms' order fixed), balanced, `decoder: false` (nothing the decoder builds from changed),
  10 rounds, a 4-CPU runner. Strict set: also no change to examples/ab.rs between the merge's
  parent and the merge (the PR numbers are listed in HARNESS below, from `git diff`), so both
  arms run the same code: their true change is zero.
- `menu.json` (pilot run 42748681243, mean-free; e4f1d8be): each cell's largest LRV over the
  five registered lengths, the within-dispatch long-run variance per round.

Per cell, DerSimonian-Laird on the links: y_j their mean log ratio, v_j = sd_j^2 / 10 their iid
within-run variance. tau^2 is the extra spread between runs; the iid v_j understates
within-run dependence, so tau^2 absorbs it: conservative for planning. A dispatch-level mean of R
rounds then has variance sigma_D^2 = tau^2 + LRV / R. With k dispatches, the bound is
exp(mean_D + t(0.95, k - 1) * sd_D / sqrt(k)) <= 1.02; its power at 1.005 is the noncentral-t
tail with noncentrality log(1.02/1.005) * sqrt(k) / sigma_D. All-cell lower bound
max(0, 1 - sum of misses). A sensitivity doubles every cell's tau.

    python -I plan.py links.jsonl menu.json out.json
"""
import json, math, sys
import numpy as np
from scipy import stats

HARNESS = {69, 74, 219, 290, 298, 326}  # neutral links whose merge changed examples/ab.rs
CORPORA = ("onebyte", "pattern", "random", "records", "runs", "text", "wikidata")
DELTA = math.log(1.02 / 1.005)
PER_ROUND_MIN = 963.04 / 2880  # nine shards' measurement minutes a round, at the pilot's pace (§2.9)
SETUP = 15 * 9


def dl(y, v):
    y, v = np.asarray(y), np.asarray(v)
    w = 1 / v
    mu = (w * y).sum() / w.sum()
    q = (w * (y - mu) ** 2).sum()
    c = w.sum() - (w ** 2).sum() / w.sum()
    return max(0.0, (q - (len(y) - 1)) / c), float(np.sqrt(np.var(y, ddof=1)))


def main():
    links = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
    menu = json.load(open(sys.argv[2]))["cells"]
    neutral = [x for x in links if str(x["bench"]).startswith("3.") and x["balanced"] and x["decoder"] is False
               and x["rounds"] == 10 and x["cores"] == 4]
    sets = {"strict": [x for x in neutral if x["pr"] not in HARNESS], "all": neutral}
    out = {"links": {k: [x["pr"] for x in v] for k, v in sets.items()}, "cells": {}, "plan": {}}
    lrv = {c: max(b["lrv"] for b in v["batches"]) for c, v in menu.items()}
    for name, S in sets.items():
        for c in CORPORA:
            for t in ("1", "2", "4"):
                for m in ("wall", "cpu"):
                    ys = [x["change"][c][t][m][0] for x in S]
                    vs = [x["change"][c][t][m][1] ** 2 / 10 for x in S]
                    tau2, sdy = dl(ys, vs)
                    out["cells"].setdefault(f"{c}/{t}/{m}", {})[name] = {
                        "tau": math.sqrt(tau2), "sd_link_means": sdy, "median_within_se": float(np.median(np.sqrt(vs))),
                        "n": len(ys)}
    for name in list(sets) + ["strict_tau_x2"]:
        base = "strict" if name == "strict_tau_x2" else name
        mult = 2.0 if name == "strict_tau_x2" else 1.0
        for R in (240, 480, 960, 1440, 2880):
            for k in (4, 6, 8, 12):
                tq = stats.t.ppf(0.95, k - 1)
                miss, worst = 0.0, (2.0, "")
                for cell, e in out["cells"].items():
                    sd = math.sqrt((mult * e[base]["tau"]) ** 2 + lrv[cell] / R)
                    p = float(stats.nct.sf(tq, k - 1, DELTA * math.sqrt(k) / sd))
                    miss += 1 - p
                    worst = min(worst, (p, cell))
                minutes = k * (SETUP + PER_ROUND_MIN * R)
                out["plan"][f"{name} R{R} k{k}"] = {"lower": max(0.0, 1 - miss), "min": worst[0], "limiting": worst[1],
                                                    "runner_minutes": minutes}
    json.dump(out, open(sys.argv[3], "w"), indent=1)
    for name in list(sets) + ["strict_tau_x2"]:
        if name in sets:
            taus = sorted((e[name]["tau"], c) for c, e in out["cells"].items())
            print(f"{name}: {len(sets[name])} links; tau median {taus[21][0]:.4f}, max {taus[-1][0]:.4f} ({taus[-1][1]}); "
                  f"top: {[(c, round(t, 4)) for t, c in taus[-5:]]}")
        else:
            print(name + ": the strict set's tau doubled (a sensitivity, not a bound)")
        print("  R \\ k | " + " | ".join(f"k{k}" for k in (4, 6, 8, 12)))
        for R in (240, 480, 960, 1440, 2880):
            print(f"  R{R} | " + " | ".join(f"{out['plan'][f'{name} R{R} k{k}']['lower']:.3f} ({out['plan'][f'{name} R{R} k{k}']['runner_minutes']:.0f} min)" for k in (4, 6, 8, 12)))


if __name__ == "__main__":
    main()
