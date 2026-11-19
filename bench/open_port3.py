#!/usr/bin/env python3
"""open-port-3, the proportionate validation for 0.5.7 (`just bench-open-port-3`; the registration
docs/journal/2026-11-19-open-port-3-registration.md; #145 QUEUED 7013818037, under Louis's
authority, LOG 7013839409).

One new prospective attempt, not a retry. The first registration's attempt
(open-port/run-20261119T125538Z) was NOT ADOPTED, and that verdict stands permanently. #635 is
shelved. `bench/open_port.py`, the first registration's scorer, is unchanged and imported here for
its loader, its identity checks, its manifest and its statistics.

    open_port3.py judge BUNDLE --port PORT_COMMIT --part-a SUMMARY
    open_port3.py verify BUNDLE --port PORT_COMMIT   (before any timing: the builds and inputs)
    open_port3.py part-a SUMMARY   (before any timing: Part A's summary is the committed one)
    open_port3.py guard TARGET --this RUN_DIR [--remote origin]   (this registration's one attempt)
    open_port3.py plan   (the recipe's runs: FILE THREADS ROUNDS SETTLE CORPUS...)

**The parts, each its own records file** (`ab run`, the port's build against v0.5.6's, paired
rounds in the harness's ABBA order, unpinned, at the registered settle of 2 s):
- **B1, zero-route guards** (`b1.jsonl`): the 9 corpora of `B1` at 8, 10 and 20 threads, 16 rounds.
- **B2, admitted guards** (`b2.jsonl`): the 4 corpora of `B2` at 8, 10 and 20 threads, 32 rounds.
- **B3, benefit** (`b3.jsonl`): wikidata-x4 and ho-wikidata-l9 at 8, 10 and 20 threads, 12 rounds.
- **B4, sustained** (`b4.jsonl`): wikidata-x16 at 10 threads, back to back (settle 0), 8 rounds.

**Each cell** (corpus, count): x_i, the rounds' paired log ratios of wall, the port over v0.5.6;
its mean m, standard error s/sqrt(n), and peak RSS as the median of the port's peaks over the median
of v0.5.6's.

**The rule (the Coordinator's criteria, QUEUED 7013818037, as corrected after the Researcher's
HOLD 7015030749):**
- **B1, pooled.** The pooled estimand is the mean of the 27 cells' means (each cell weighted
  equally); its exponential is the pooled geometric mean ratio. Its 95% interval is a stratified
  bootstrap: rounds resampled with replacement within each cell, `BOOT` resamples, the 2.5th and
  97.5th percentiles, from a fixed generator (`SEED`, splitmix64, so the verdict is reproducible
  on any Python). Pass: the upper bound at most `POOLED_MAX` (1.01).
- **B1, credible regression.** A cell is flagged when its one-sided lower confidence bound,
  m - t(1 - 0.05/27, n - 1) * s/sqrt(n), lies above log `FLAG_MIN` (1.02). Any flag fails.
- **B2 = G exactly, in each of the 12 cells** (Open executes there): not SLOWER (the two-sided
  95% t-interval wholly above 1), a geometric mean at most `GUARD_MAX` (1.02), and peak RSS at
  most 1.10. The pooled B2 figure is reported, not judged.
- **Memory, every guard cell (B1 and B2):** peak RSS at most `RSS_GATE` (1.10).
- **B3, benefit:** every one of the 6 cells is FASTER (its two-sided 95% t-interval wholly below
  1) and a gain under Holm at family alpha 0.05 (one-sided t, as P1 was); and its peak RSS is at
  most 1.00.
- **B4 = S exactly as registered:** not SLOWER, and a geometric mean at most 1.02; no memory
  condition.
- **Part A,** bound to its evidence: the summary's SHA-256 is the committed one's
  (`PART_A_SHA256`), its `voids` and `triggers` are present, its rows cover every B1 corpus at
  t8, t10, t20 and w7, and every one of those cells has zero-open receipts (the open kernel's
  `emit` and `induced` at zero instructions in every port run). Anything else, missing or empty,
  is VOID. Then: no VOID of its own, and every trigger carries a written attribution.

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
import hashlib
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
B1_ROUNDS, B2_ROUNDS, BENEFIT_ROUNDS, SUSTAINED_ROUNDS, SETTLE = 16, 32, 12, 8, 2
POOLED_MAX, FLAG_MIN, RSS_GATE, BENEFIT_RSS, ALPHA = 1.01, 1.02, op.RSS_GATE, 1.00, 0.05
GUARD_MAX = op.GUARD_MAX
# Part A's committed summary (docs/journal/2026-11-19-open-port-3-registration/part-a/summary.json)
PART_A_SHA256 = "b9ae3a702fefed51f7b4c2602f7d39e5b268d2d7db883f3bb576ecc3525b944b"
PART_A_LABELS = ("t8", "t10", "t20", "w7")
BOOT, SEED = 20000, 7013818037
assert sorted(B1 + B2 + B3) == sorted(op.CORPORA)

# (file, corpora, counts, rounds, settle): the recipe's runs, in its order, and the judge's roster
PARTS = (
    ("b1.jsonl", B1, COUNTS, B1_ROUNDS, SETTLE),
    ("b2.jsonl", B2, COUNTS, B2_ROUNDS, SETTLE),
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


def interval(xs):
    """(geometric mean, two-sided 95% t-interval) of a cell's log ratios, with the exact t
    quantile at any count (`op.t975`'s table stops at 25 degrees of freedom)"""
    n = len(xs)
    m = statistics.mean(xs)
    h = t_quantile(0.975, n - 1) * statistics.stdev(xs) / math.sqrt(n)
    return math.exp(m), math.exp(m - h), math.exp(m + h)


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


def part_a(path, errors, fails, expect=PART_A_SHA256):
    """Part A's summary, bound to its evidence (the Researcher's HOLD 7015030749): VOID unless it
    is the committed summary (its SHA-256), with `voids` and `triggers` present, rows for every B1
    corpus at every label of `PART_A_LABELS`, and zero-open receipts for each of those cells; VOID
    on any of its own VOIDs; NOT ADOPTED on an unattributed trigger. `expect` None skips the
    identity (for the checks of the rest)"""
    if not path:
        errors.append("Part A: no summary given (--part-a): VOID")
        return
    try:
        raw = open(path, "rb").read()
        s = json.loads(raw)
    except (OSError, ValueError) as e:
        errors.append(f"Part A: {path} unreadable ({e})")
        return
    digest = hashlib.sha256(raw).hexdigest()
    if expect is not None and digest != expect:
        errors.append(f"Part A: the summary's SHA-256 is {digest}, not the committed {expect}")
    if not isinstance(s, dict):
        errors.append("Part A: the summary is not an object")
        return
    voids, triggers = s.get("voids"), s.get("triggers")
    rows, emit = s.get("rows"), s.get("emit")
    if not isinstance(voids, list) or not isinstance(triggers, list):
        errors.append("Part A: `voids` or `triggers` missing")
        return
    if not isinstance(rows, list) or not isinstance(emit, dict):
        errors.append("Part A: no corpus rows or no zero-open receipts")
        return
    have = {(r.get("corpus"), r.get("threads")) for r in rows if isinstance(r, dict)}
    want = {(c, label) for c in B1 for label in PART_A_LABELS}
    if want - have:
        errors.append(f"Part A: no rows for {sorted(want - have)[:4]} ({len(want - have)} cells)")
    for c, label in sorted(want):
        got = [v for k, v in emit.items() if k.rsplit(" #", 1)[0] == f"{c} {label}"]
        if not got:
            errors.append(f"Part A: no zero-open receipt for {c} at {label}")
        elif any(v != [0, 0] for v in got):
            errors.append(f"Part A: {c} at {label}: the open kernel ran ({got})")
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
        f"Part A: sha256 {digest[:16]}…; {len(rows)} cells, {len(emit)} zero-open receipts; "
        f"{len(voids)} VOIDs; {len(triggers)} triggers, {len(triggers) - len(bare)} with a written "
        "attribution"
    )
    for t in bare:
        fails.append(
            f"Part A: an unattributed trigger: {t.get('corpus')} at {t.get('threads')}, {t.get('event')} "
            f"x{t.get('ratio')}"
        )


def check_part_a(a):
    """Before the timing marker: Part A's summary is the committed one, complete"""
    errors, fails = [], []
    part_a(a.summary, errors, fails)
    for e in errors + fails:
        print(f"  {e}")
    if errors or fails:
        print("bench-open-port-3: Part A's summary is not the registered one; nothing is timed")
        return 1
    return 0


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
    r = rows["b1.jsonl"]
    cells = [(c, t) for t in COUNTS for c in B1]
    k = len(cells)
    xs = {ct: logs(r, *ct, B1_ROUNDS) for ct in cells}
    est, lo, hi = pooled([xs[ct] for ct in cells])
    print(
        f"\nB1, zero-route guards: {k} cells, {B1_ROUNDS} rounds; the port over v0.5.6, wall as each "
        f"cell's geometric mean [95% t-interval]; the flag bound (one-sided, 1 - {ALPHA}/{k}); peak RSS"
    )
    for c, t in cells:
        g, glo, ghi = interval(xs[(c, t)])
        fb = flag_bound(xs[(c, t)], k)
        mem = memory(r, c, t, B1_ROUNDS)
        flagged = fb > math.log(FLAG_MIN)
        print(
            f"  {c} {t}t: {g:.4f} [{glo:.4f}, {ghi:.4f}]; "
            f"flag bound {math.exp(fb):.4f}{' FLAGGED' if flagged else ''}; memory {mem:.3f}"
            + (" ABOVE THE CAP" if mem > RSS_GATE else "")
        )
        if flagged:
            fails.append(
                f"B1: a credible regression, {c} at {t} threads, lower bound "
                f"{math.exp(fb):.4f} above {FLAG_MIN}"
            )
        if mem > RSS_GATE:
            fails.append(f"memory: {c} at {t} threads peak RSS {mem:.3f}, above {RSS_GATE}")
    print(
        f"  pooled: {math.exp(est):.4f} [{math.exp(lo):.4f}, {math.exp(hi):.4f}] "
        f"(stratified bootstrap, {BOOT} resamples, seed {SEED}); pass at most {POOLED_MAX}"
    )
    if not math.exp(hi) <= POOLED_MAX:
        fails.append(f"B1: the pooled upper bound {math.exp(hi):.4f} is above {POOLED_MAX}")
    r = rows["b2.jsonl"]
    cells = [(c, t) for t in COUNTS for c in B2]
    xs = {ct: logs(r, *ct, B2_ROUNDS) for ct in cells}
    print(
        f"\nB2, admitted guards (G, per cell): {len(cells)} cells, {B2_ROUNDS} rounds; geometric mean "
        f"[95% t-interval]; peak RSS. Fails: SLOWER, above {GUARD_MAX}, or memory above {RSS_GATE}"
    )
    for c, t in cells:
        g, glo, ghi = interval(xs[(c, t)])
        mem = memory(r, c, t, B2_ROUNDS)
        print(f"  {c} {t}t: {g:.4f} [{glo:.4f}, {ghi:.4f}] {op.label(glo, ghi)}; memory {mem:.3f}")
        if glo > 1:
            fails.append(f"B2: {c} at {t} threads SLOWER, {g:.4f} [{glo:.4f}, {ghi:.4f}]")
        elif g > GUARD_MAX:
            fails.append(f"B2: {c} at {t} threads {g:.4f}, above the bound {GUARD_MAX}")
        if mem > RSS_GATE:
            fails.append(f"memory: {c} at {t} threads peak RSS {mem:.3f}, above {RSS_GATE}")
    est, lo, hi = pooled([xs[ct] for ct in cells])
    print(
        f"  pooled (reported, not judged): {math.exp(est):.4f} [{math.exp(lo):.4f}, {math.exp(hi):.4f}]"
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
        if not hi < 1:
            fails.append(f"B3: {c} at {t} threads not FASTER, {g:.4f} [{lo:.4f}, {hi:.4f}]")
        if (c, t) not in rejected:
            fails.append(f"B3: no gain on {c} at {t} threads under Holm (p {p:.2e}, {g:.4f})")
        if mem > BENEFIT_RSS:
            fails.append(f"B3: peak RSS on {c} at {t} threads {mem:.3f}, above {BENEFIT_RSS}")
    g, lo, hi, p, mem = op.cell(rows["b4.jsonl"], B4[0], B4[1], SUSTAINED_ROUNDS)
    print(
        f"\nB4, sustained: {B4[1]} threads back to back on {B4[0]}, {SUSTAINED_ROUNDS} rounds: "
        f"{g:.4f} [{lo:.4f}, {hi:.4f}] {op.label(lo, hi)}; memory {mem:.3f} (reported, as S's)"
    )
    if lo > 1:
        fails.append(f"B4: sustained SLOWER, {g:.4f} [{lo:.4f}, {hi:.4f}]")
    elif g > GUARD_MAX:
        fails.append(f"B4: sustained {g:.4f}, above the bound {GUARD_MAX}")
    if fails:
        print(f"\nVERDICT: NOT ADOPTED. {len(fails)} conditions fail:")
        for f in fails:
            print(f"  {f}")
    else:
        print(
            "\nVERDICT: ADOPT. Part A bound and attributed; the zero-route guards' pooled upper bound "
            f"within {POOLED_MAX} with no credible regression above {FLAG_MIN}; every admitted guard "
            f"within G (not SLOWER, at most {GUARD_MAX}); every benefit cell FASTER and a gain under "
            f"Holm at no more memory; sustained within S; every guard's memory within {RSS_GATE}."
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
    pa = sub.add_parser("part-a")
    pa.add_argument("summary")
    g = sub.add_parser("guard")
    g.add_argument("target")
    g.add_argument("--this")
    g.add_argument("--remote", default="origin")
    a = ap.parse_args(argv[1:])
    return {"judge": judge, "verify": verify, "plan": plan, "guard": guard, "part-a": check_part_a}[
        a.mode
    ](a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
