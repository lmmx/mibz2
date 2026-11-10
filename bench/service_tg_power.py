"""The power calculation for the target-gated registration's CI condition 2
(docs/journal/2026-11-10-service-guarded-target-gated-registration.md, §2.4), published so the
Researcher can reproduce it. A design calculation, not evidence: it reads CI's history of
paired PR runs and models the registered test; it measures nothing.

    git show origin/bench-results:links.jsonl > links.jsonl
    python3 bench/service_tg_power.py links.jsonl [--rounds 600] [--batches 20]

**The variance inputs.** `links.jsonl` (bench-results) holds, for each merged PR's last paired
Benchmark run, per corpus and thread count, the mean and sd of the rounds' log ratios, head
over base, of wall and CPU. Selected: bench 3.x, 10 rounds, a 4-CPU runner (the standing grid),
every one of them (47 at 2026-11-10). Each sd is a 10-round within-run estimate (9 df) on one
runner, the noise a within-runner shard sees. Per cell, the median and the 90th percentile
over the runs are the planning sds; neither is a bound on a future runner.

**The test modelled** (service_tg.py `ni`): a cell's R paired logs in B consecutive batches of
R/B rounds; the estimate the mean of all R; its standard error from the B batch means; the
upper bound exp(mean + t(0.95, B - 1) * se). Under independent, identically distributed paired
logs with sd sigma, the batch means are normal with sd sigma / sqrt(R/B), and the bound is at
most log 1.02 with probability P(T >= t(0.95, B - 1)), T noncentral t with B - 1 df and
noncentrality (log 1.02 - mu) * sqrt(R) / sigma. Positive autocorrelation within a batch
inflates the batch means' variance: this model is then optimistic, and the batch-means
interval stays valid while the iid one would not (the reason it is the registered estimator).

**What it prints.**
- **Size:** at mu = log 1.02 (the null boundary), each cell passes with probability 0.05.
- **Power:** at mu = 0 (a true ratio of 1.00, inside the alternative), +0.5% and +1%, per cell,
  under the median and the 90th-percentile sd.
- **All 42:** two summaries of the all-pass probability, since cells share runners and wall and
  CPU are correlated: the dependence-free lower bound max(0, 1 - sum of the cells' misses)
  (D077), and the product, which assumes independence across cells and is not a bound.
"""

import argparse
import json
import math
import statistics

DELTA = math.log(1.02)
CORPORA = ("onebyte", "pattern", "random", "records", "runs", "text", "wikidata")
THREADS = ("1", "2", "4")
METRICS = ("wall", "cpu")


def history(path):
    runs = []
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        l = json.loads(line)
        if str(l.get("bench", "")).startswith("3.") and l.get("rounds") == 10 and l.get("cores") == 4 and l.get("change"):
            runs.append(l)
    sds = {}
    for l in runs:
        for c in CORPORA:
            for t in THREADS:
                for m in METRICS:
                    v = l["change"].get(c, {}).get(t)
                    if v:
                        sds.setdefault((c, t, m), []).append(v[m][1])
    return runs, sds


def quantile(xs, q):
    xs = sorted(xs)
    i = q * (len(xs) - 1)
    lo, hi = math.floor(i), math.ceil(i)
    return xs[lo] + (xs[hi] - xs[lo]) * (i - lo)


def power(R, B, sigma, mu):
    from scipy import stats

    df = B - 1
    return 1 - stats.nct.cdf(stats.t.ppf(0.95, df), df, (DELTA - mu) * math.sqrt(R) / sigma)


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("links")
    p.add_argument("--rounds", type=int, nargs="*", default=[300, 480, 600, 900])
    p.add_argument("--batches", type=int, default=20)
    a = p.parse_args(argv)
    runs, sds = history(a.links)
    print(f"history: {len(runs)} runs, {runs[0]['date']} to {runs[-1]['date']}, PRs {runs[0]['pr']}..{runs[-1]['pr']}")
    print("\nper-cell planning sds (log ratio, within run): median / p90 over the runs")
    for k in sorted(sds):
        print(f"  {k[0]:9s} {k[1]}t {k[2]:4s} n={len(sds[k]):2d}  {statistics.median(sds[k]):.4f} / {quantile(sds[k], 0.9):.4f}")
    print(f"\nsize at mu = log 1.02, B = {a.batches}: per cell "
          f"{power(600, a.batches, 0.05, DELTA):.4f} (any sigma, any R)")
    print(f"\n| R | B | sd | mu | min cell power | sum of misses | all-42 lower bound | all-42 if independent |")
    print("|--:|--:|---|--:|--:|--:|--:|--:|")
    for R in a.rounds:
        for which, f in (("median", statistics.median), ("p90", lambda x: quantile(x, 0.9))):
            for mu in (0.0, 0.005, 0.01):
                pw = [power(R, a.batches, f(sds[k]), mu) for k in sds]
                miss = sum(1 - x for x in pw)
                prod = math.prod(pw)
                print(f"| {R} | {a.batches} | {which} | {mu:+.3f} | {min(pw):.3f} | {miss:.3f} | {max(0.0, 1 - miss):.3f} | {prod:.3f} |")


if __name__ == "__main__":
    main()
