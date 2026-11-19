#!/usr/bin/env python3
"""open-port-3, the proportionate validation for 0.5.7 (`just bench-open-port-3`; the registration
docs/journal/2026-11-19-open-port-3-registration.md; #145 QUEUED 7013818037, under Louis's
authority, LOG 7013839409).

One new prospective attempt, not a retry. The first registration's attempt
(open-port/run-20261119T125538Z) was NOT ADOPTED, and that verdict stands permanently. #635 is
shelved. `bench/open_port.py`, the first registration's scorer, is unchanged and imported here for
its loader, its identity checks, its manifest and its statistics.

    open_port3.py judge BUNDLE --port PORT_COMMIT [--part-a SUMMARY]
    open_port3.py verify BUNDLE --port PORT_COMMIT   (before any timing: the builds and inputs)
    open_port3.py guard TARGET --this RUN_DIR [--remote origin]   (this registration's one attempt)
    open_port3.py plan   (the recipe's runs: FILE THREADS ROUNDS SETTLE CORPUS...)

**The parts, each its own records file** (`ab run`, the port's build against v0.5.6's, paired
rounds in the harness's ABBA order, unpinned, at the registered settle of 2 s):
- **B1, zero-route guards** (`b1.jsonl`): the 9 corpora of `B1` at 8, 10 and 20 threads, 16 rounds.
- **B2, admitted guards** (`b2.jsonl`): the 4 corpora of `B2` at 8, 10 and 20 threads, 16 rounds.
- **B3, benefit** (`b3.jsonl`): wikidata-x4 and ho-wikidata-l9 at 8, 10 and 20 threads, 12 rounds.
- **B4, sustained** (`b4.jsonl`): wikidata-x16 at 10 threads, back to back (settle 0), 8 rounds.

**Each cell** (corpus, count): x_i, the rounds' paired log ratios of wall, the port over v0.5.6;
its mean m, standard error s/sqrt(n), and peak RSS as the median of the port's peaks over the median
of v0.5.6's.

**The rule (the Coordinator's criteria, QUEUED 7013818037):**
- **B1 and B2, pooled.** The pooled estimand is the mean of the cells' means (each cell weighted
  equally); its exponential is the pooled geometric mean ratio. Its 95% interval is a stratified
  bootstrap: rounds resampled with replacement within each cell, `BOOT` resamples, the 2.5th and
  97.5th percentiles, from a fixed generator (`SEED`, splitmix64, so the verdict is reproducible
  on any Python). Pass: the upper bound at most `POOLED_MAX` (1.01).
- **B1 and B2, credible regression.** A cell is flagged when its one-sided lower confidence bound,
  m - t(1 - 0.05/k, n - 1) * s/sqrt(n) with k the part's cell count (27 or 12), lies above
  log `FLAG_MIN` (1.02). Any flag fails.
- **Memory, every guard cell (B1 and B2):** peak RSS at most `RSS_GATE` (1.10).
- **B3, benefit:** every one of the 6 cells is a gain under Holm at family alpha 0.05 (one-sided
  t, as P1 was), and its peak RSS is at most 1.00.
- **B4, sustained:** not SLOWER: the two-sided 95% t-interval's lower bound is not above 1.
- **Part A** (`--part-a`, the cachegrind diagnostic's summary, committed before timing): no VOID,
  and every trigger carries a written attribution. Without it the verdict is withheld.

**VOID, and nothing is read,** where `open_port.py`'s conditions hold of any file (a record
missing, duplicated, failed or outside the roster; a field of the wrong type; a wall, RSS or output
not a finite positive number; the builds or the 18 inputs not the registered ones; the machine not
the i9; the Wikidata corpora's observed workers not the shipped 8/7/7; a dry run), or across files:
a corpus's output (size, digest) differs between files, or a build's arm differs between files.

**ADOPT** only where Part A, B1, B2, B3, B4 and the memory caps all hold. Otherwise **NOT ADOPTED**,
every failing condition named. One attempt (`guard`), archived as `open-port-3/run-*.tar.gz`.
Standard library only.
"""

import argparse
import glob
import json
import math
import os
import re
import statistics
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import open_port as op  # noqa: E402

PREFIX = "open-port-3"
COUNTS = op.COUNTS
# the zero-route guards (no block opens: Part A, and the first attempt's census) and the admitted
# guards, as the Coordinator listed them
B1 = (
    "text",
    "random",
    "records",
    "runs",
    "pattern",
    "onebyte",
    "ho-code-l1",
    "ho-code-l9",
    "ho-prose-l9",
)
B2 = ("ho-wikidata-l1", "ho-structured-l1", "ho-structured-l9", "ho-prose-l1")
B3 = (op.MAIN, op.HELDOUT)
B4 = (op.SUSTAINED, 10)
GUARD_ROUNDS, BENEFIT_ROUNDS, SUSTAINED_ROUNDS, SETTLE = 16, 12, 8, 2
POOLED_MAX, FLAG_MIN, RSS_GATE, BENEFIT_RSS, ALPHA = 1.01, 1.02, op.RSS_GATE, 1.00, 0.05
BOOT, SEED = 20000, 7013818037
assert sorted(B1 + B2 + B3) == sorted(op.CORPORA)

# (file, corpora, counts, rounds, settle): the recipe's runs, in its order, and the judge's roster
PARTS = (
    ("b1.jsonl", B1, COUNTS, GUARD_ROUNDS, SETTLE),
    ("b2.jsonl", B2, COUNTS, GUARD_ROUNDS, SETTLE),
    ("b3.jsonl", B3, COUNTS, BENEFIT_ROUNDS, SETTLE),
    ("b4.jsonl", (B4[0],), (B4[1],), SUSTAINED_ROUNDS, 0),
)


def logs(rows, corpus, t, rounds):
    return [
        math.log(rows[(corpus, t, "cand", i)]["wall"] / rows[(corpus, t, "ref", i)]["wall"])
        for i in range(rounds)
    ]


def memory(rows, corpus, t, rounds):
    return statistics.median(rows[(corpus, t, "cand", i)]["rss_kb"] for i in range(rounds)) / (
        statistics.median(rows[(corpus, t, "ref", i)]["rss_kb"] for i in range(rounds))
    )


def t_cdf(x, df):
    """Student's t distribution function (op.betainc)"""
    tail = 0.5 * op.betainc(df / 2, 0.5, df / (df + x * x))
    return 1 - tail if x > 0 else tail


def t_quantile(q, df):
    """The q quantile of Student's t (bisection on t_cdf; q above one half)"""
    lo, hi = 0.0, 1000.0
    for _ in range(200):
        mid = (lo + hi) / 2
        if t_cdf(mid, df) < q:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


class SplitMix64:
    """A fixed, version-independent generator for the bootstrap"""

    def __init__(self, seed):
        self.s = seed & 0xFFFFFFFFFFFFFFFF

    def next(self):
        self.s = (self.s + 0x9E3779B97F4A7C15) & 0xFFFFFFFFFFFFFFFF
        z = self.s
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & 0xFFFFFFFFFFFFFFFF
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & 0xFFFFFFFFFFFFFFFF
        return z ^ (z >> 31)

    def below(self, n):
        # Lemire's multiply-shift (bias at most n / 2^64, negligible for n <= 16)
        return (self.next() * n) >> 64


def pooled(cells, boot=BOOT, seed=SEED):
    """(pooled mean of the cells' mean log ratios, its stratified-bootstrap 2.5th and 97.5th
    percentiles): `cells` is a list of each cell's log ratios"""
    est = statistics.mean(statistics.mean(xs) for xs in cells)
    rng = SplitMix64(seed)
    draws = []
    for _ in range(boot):
        tot = 0.0
        for xs in cells:
            n = len(xs)
            tot += sum(xs[rng.below(n)] for _ in range(n)) / n
        draws.append(tot / len(cells))
    draws.sort()
    return est, draws[int(0.025 * boot)], draws[int(math.ceil(0.975 * boot)) - 1]


def flag_bound(xs, k):
    """The cell's one-sided lower confidence bound on its mean log ratio, at 1 - ALPHA/k"""
    n = len(xs)
    se = statistics.stdev(xs) / math.sqrt(n)
    return statistics.mean(xs) - t_quantile(1 - ALPHA / k, n - 1) * se


def joins(files, errors):
    """Across every file: one output (size, digest) for each corpus, and one arm for each build"""
    outs, arms = {}, {}
    for name, rows in files:
        for (c, _t, side, _i), r in rows.items():
            o = (r.get("out"), r.get("sha256") or r.get("hash"))
            outs.setdefault(c, {}).setdefault(o, name)
            arms.setdefault(side, {}).setdefault(r["arm"], name)
    for c, o in sorted(outs.items()):
        if len(o) > 1:
            errors.append(f"{c}: the outputs differ across files: {sorted(o.items(), key=str)[:3]}")
    for side, a in sorted(arms.items()):
        if len(a) > 1:
            errors.append(
                f"the {'port' if side == 'cand' else 'reference'} build's arm differs across files: "
                f"{sorted(a.items())[:3]}"
            )


def part_a(path, errors, fails):
    """Part A's committed summary: VOID on any of its VOIDs; NOT ADOPTED on an unattributed trigger"""
    if not path:
        errors.append("Part A: no summary given (--part-a); the verdict is withheld")
        return
    try:
        s = json.load(open(path))
        voids, triggers = s["voids"], s["triggers"]
        assert isinstance(voids, list) and isinstance(triggers, list)
    except (OSError, ValueError, KeyError, AssertionError, TypeError) as e:
        errors.append(f"Part A: {path} unreadable ({e})")
        return
    for v in voids:
        errors.append(f"Part A VOID: {v}")
    bare = [
        t
        for t in triggers
        if not (
            isinstance(t, dict)
            and isinstance(t.get("attribution"), str)
            and t["attribution"].strip()
        )
    ]
    print(
        f"Part A: {len(voids)} VOIDs; {len(triggers)} triggers, "
        f"{len(triggers) - len(bare)} with a written attribution"
    )
    for t in bare:
        fails.append(
            f"Part A: an unattributed trigger: {t.get('corpus')} at {t.get('threads')}, {t.get('event')} "
            f"x{t.get('ratio')}"
        )


def judge(a):
    errors, fails = [], []
    if os.path.exists(os.path.join(a.bundle, "DRY-RUN.txt")):
        errors.append("a dry run (DRY-RUN.txt in the bundle): not evidence")
    model = op.machine(
        os.path.join(a.bundle, "lscpu.txt"), os.path.join(a.bundle, "env.json"), errors
    )
    op.identities(a.bundle, a.port, errors)
    files, rows = [], {}
    for name, corpora, counts, rounds, _ in PARTS:
        r = op.load(os.path.join(a.bundle, name), list(corpora), list(counts), rounds, errors)
        files.append((name, r))
        rows[name] = r
    joins(files, errors)
    part_a(a.part_a, errors, fails)
    print(
        "records "
        + ", ".join(
            f"{n} {len(rows[n])} of {2 * len(cs) * len(ts) * k}" for n, cs, ts, k, _ in PARTS
        )
        + (f"; the machine: {model}" if model else "")
    )
    if errors:
        print(
            f"VOID: the records do not make the registered run ({len(errors)} problems); no verdict. "
            "A VOID allows one repair of untimed setup only."
        )
        for e in errors[:25]:
            print(f"  {e}")
        return 1
    for name, label, corpora in (
        ("b1.jsonl", "B1, zero-route guards", B1),
        ("b2.jsonl", "B2, admitted guards", B2),
    ):
        r = rows[name]
        cells = [(c, t) for t in COUNTS for c in corpora]
        k = len(cells)
        xs = {ct: logs(r, *ct, GUARD_ROUNDS) for ct in cells}
        est, lo, hi = pooled([xs[ct] for ct in cells])
        print(
            f"\n{label}: {k} cells, {GUARD_ROUNDS} rounds; the port over v0.5.6, wall as each cell's "
            f"geometric mean [95% t-interval]; the flag bound (one-sided, 1 - {ALPHA}/{k}); peak RSS"
        )
        for c, t in cells:
            m = statistics.mean(xs[(c, t)])
            h = op.t975(GUARD_ROUNDS - 1) * statistics.stdev(xs[(c, t)]) / math.sqrt(GUARD_ROUNDS)
            fb = flag_bound(xs[(c, t)], k)
            mem = memory(r, c, t, GUARD_ROUNDS)
            flagged = fb > math.log(FLAG_MIN)
            print(
                f"  {c} {t}t: {math.exp(m):.4f} [{math.exp(m - h):.4f}, {math.exp(m + h):.4f}]; "
                f"flag bound {math.exp(fb):.4f}{' FLAGGED' if flagged else ''}; memory {mem:.3f}"
                + (" ABOVE THE CAP" if mem > RSS_GATE else "")
            )
            if flagged:
                fails.append(
                    f"{label[:2]}: a credible regression, {c} at {t} threads, lower bound "
                    f"{math.exp(fb):.4f} above {FLAG_MIN}"
                )
            if mem > RSS_GATE:
                fails.append(f"memory: {c} at {t} threads peak RSS {mem:.3f}, above {RSS_GATE}")
        print(
            f"  pooled: {math.exp(est):.4f} [{math.exp(lo):.4f}, {math.exp(hi):.4f}] "
            f"(stratified bootstrap, {BOOT} resamples, seed {SEED}); pass at most {POOLED_MAX}"
        )
        if not math.exp(hi) <= POOLED_MAX:
            fails.append(
                f"{label[:2]}: the pooled upper bound {math.exp(hi):.4f} is above {POOLED_MAX}"
            )
    r = rows["b3.jsonl"]
    ps, res = {}, {}
    print(
        f"\nB3, benefit: {BENEFIT_ROUNDS} rounds; geometric mean [95% t-interval]; one-sided p (gain); peak RSS"
    )
    for t in COUNTS:
        for c in B3:
            g, lo, hi, p, mem = res[(c, t)] = op.cell(r, c, t, BENEFIT_ROUNDS)
            ps[(c, t)] = p
            print(
                f"  {c} {t}t: {g:.4f} [{lo:.4f}, {hi:.4f}] {op.label(lo, hi)}; p {p:.2e}; memory {mem:.3f}"
            )
    rejected = op.holm(ps, ALPHA)
    print(
        f"  Holm at alpha {ALPHA} over {len(ps)} cells: {len(rejected)} of {len(ps)} reject no gain"
    )
    for (c, t), (g, lo, hi, p, mem) in sorted(res.items()):
        if (c, t) not in rejected:
            fails.append(f"B3: no gain on {c} at {t} threads under Holm (p {p:.2e}, {g:.4f})")
        if mem > BENEFIT_RSS:
            fails.append(f"B3: peak RSS on {c} at {t} threads {mem:.3f}, above {BENEFIT_RSS}")
    g, lo, hi, p, mem = op.cell(rows["b4.jsonl"], B4[0], B4[1], SUSTAINED_ROUNDS)
    print(
        f"\nB4, sustained: {B4[1]} threads back to back on {B4[0]}, {SUSTAINED_ROUNDS} rounds: "
        f"{g:.4f} [{lo:.4f}, {hi:.4f}] {op.label(lo, hi)}; memory {mem:.3f} (reported)"
    )
    if lo > 1:
        fails.append(f"B4: sustained SLOWER, {g:.4f} [{lo:.4f}, {hi:.4f}]")
    if fails:
        print(f"\nVERDICT: NOT ADOPTED. {len(fails)} conditions fail:")
        for f in fails:
            print(f"  {f}")
    else:
        print(
            "\nVERDICT: ADOPT. Part A attributed; the zero-route and admitted guards' pooled upper "
            f"bounds within {POOLED_MAX} with no credible regression above {FLAG_MIN}; a gain on every "
            "benefit cell (Holm) at no more memory; sustained not SLOWER; every guard's memory within "
            f"{RSS_GATE}."
        )
    return 0


def verify(a):
    """Before any timing: the bundle's builds and inputs are the registered ones"""
    return op.verify(a)


def plan(a):
    for name, corpora, counts, rounds, settle in PARTS:
        for t in counts:
            print(name, t, rounds, settle, *corpora)
    return 0


def guard(a):
    """This registration's one attempt: as `open_port.py guard`, for its own evidence path. Exit 0
    only if no timed attempt has started in `target` (a run directory with `timing-started.txt`,
    other than `--this`) and the evidence branch, fetched afresh, is readable and archives no
    `open-port-3/run-*.tar.gz`. Fails closed."""
    for started in sorted(glob.glob(os.path.join(a.target, "run-*", "timing-started.txt"))):
        d = os.path.dirname(started)
        if a.this and os.path.realpath(d) == os.path.realpath(a.this):
            continue
        if not os.path.exists(os.path.join(d, "DRY-RUN.txt")):
            print(
                f"bench-open-port-3: a timed attempt already started here ({os.path.basename(d)}): "
                "this registration allows one"
            )
            return 1
    ref = f"refs/remotes/{a.remote}/evidence"

    def git(*args):
        return subprocess.run(["git", *args], capture_output=True, text=True)

    if git("fetch", "-q", a.remote, f"+refs/heads/evidence:{ref}").returncode != 0:
        print(
            f"bench-open-port-3: the evidence branch can't be fetched from {a.remote}, so a timed "
            "attempt archived there can't be ruled out: refusing (this registration allows one)"
        )
        return 1
    listing = git("ls-tree", "-r", "--name-only", ref, "--", PREFIX)
    if listing.returncode != 0:
        print(
            f"bench-open-port-3: the evidence branch ({ref}) can't be read: refusing "
            "(this registration allows one)"
        )
        return 1
    runs = [n for n in listing.stdout.split() if re.fullmatch(rf"{PREFIX}/run-[^/]*\.tar\.gz", n)]
    if runs:
        print(
            f"bench-open-port-3: a timed attempt is already archived on the evidence branch "
            f"({runs[0]}): this registration allows one"
        )
        return 1
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="mode", required=True)
    j = sub.add_parser("judge")
    j.add_argument("bundle")
    j.add_argument("--port", required=True)
    j.add_argument("--part-a", help="Part A's summary.json (committed beside the registration)")
    v = sub.add_parser("verify")
    v.add_argument("bundle")
    v.add_argument("--port", required=True)
    sub.add_parser("plan")
    g = sub.add_parser("guard")
    g.add_argument("target")
    g.add_argument("--this")
    g.add_argument("--remote", default="origin")
    a = ap.parse_args(argv[1:])
    return {"judge": judge, "verify": verify, "plan": plan, "guard": guard}[a.mode](a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
