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
- **Size under the model:** at a true ratio of 1.02 (the null boundary), each cell passes with
  probability 0.05 only if the batch means are iid normal. Dependent batch means can make the
  test anticonservative (the Researcher's AR(1) falsifier: 8.5% at rho 0.9, 28.7% at 0.99), so
  this is not a calibration (registration §2.3).
- **Power:** at true ratios 1.000, 1.005 and 1.010, per cell, under the median and the
  90th-percentile ten-round sd. Those sds are a scenario, not an envelope: each is a 9-df
  marginal estimate, and the design needs a long-run variance the pilot (§2.8) is to estimate.
- R must split into B batches of an even length (the arms' two-round order balance).
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


def power(R, B, sigma, ratio):
    """One cell's probability of passing (its bound at most 1.02) at a true head/base ratio
    `ratio`, under the model above: P(Z + lambda >= t * sqrt(V / df)) with Z standard normal,
    V chi-square on df = B - 1, lambda = (log 1.02 - log ratio) * sqrt(R) / sigma, computed
    as the integral over V (the noncentral-t CDF's library call returned NaN in the
    Researcher's probe). A non-finite or out-of-range result is an error, never a printed 0"""
    from scipy import integrate, stats

    if R % B or (R // B) % 2:
        raise ValueError(f"R {R} / B {B}: batches must be of an even length, to keep the arms' two-round order balance")
    df = B - 1
    tq = stats.t.ppf(0.95, df)
    lam = (DELTA - math.log(ratio)) * math.sqrt(R) / sigma
    f = lambda v: stats.norm.cdf(lam - tq * math.sqrt(v / df)) * stats.chi2.pdf(v, df)  # noqa: E731
    val, _ = integrate.quad(f, 0, math.inf, limit=200)
    if not (math.isfinite(val) and -1e-9 <= val <= 1 + 1e-9):
        raise ArithmeticError(f"power not finite or out of range at R {R}, B {B}, sigma {sigma}, ratio {ratio}: {val}")
    return min(1.0, max(0.0, val))


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("links")
    p.add_argument("--rounds", type=int, nargs="*", default=[600, 960])
    p.add_argument("--batches", type=int, default=20)
    a = p.parse_args(argv)
    runs, sds = history(a.links)
    print(f"history: {len(runs)} runs, {runs[0]['date']} to {runs[-1]['date']}, PRs {runs[0]['pr']}..{runs[-1]['pr']}")
    print("\nper-cell planning sds (log ratio, within run): median / p90 over the runs")
    for k in sorted(sds):
        print(f"  {k[0]:9s} {k[1]}t {k[2]:4s} n={len(sds[k]):2d}  {statistics.median(sds[k]):.4f} / {quantile(sds[k], 0.9):.4f}")
    print(f"\nsize at a true ratio of 1.02, B = {a.batches}: per cell "
          f"{power(600, a.batches, 0.05, 1.02):.4f} (under the model's iid batch means; not a calibration: §2.3)")
    print("\n| R | B | sd | true ratio | min cell power | sum of misses | all-42 lower bound | all-42 if independent |")
    print("|--:|--:|---|--:|--:|--:|--:|--:|")
    for R in a.rounds:
        for which, f in (("median", statistics.median), ("p90", lambda x: quantile(x, 0.9))):
            for ratio in (1.0, 1.005, 1.01):
                pw = [power(R, a.batches, f(sds[k]), ratio) for k in sds]
                miss = sum(1 - x for x in pw)
                prod = math.prod(pw)
                print(f"| {R} | {a.batches} | {which} | {ratio:.3f} | {min(pw):.4f} | {miss:.4f} | {max(0.0, 1 - miss):.4f} | {prod:.4f} |")


if __name__ == "__main__":
    main()

