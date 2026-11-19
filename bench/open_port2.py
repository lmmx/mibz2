#!/usr/bin/env python3
"""The open port's successor adoption verdict (`just bench-open-port-2`; the successor
registration docs/journal/2026-11-19-open-port-2-registration.md; #145 QUEUED 7011091171).

A new experiment, not a retry. The first registration's one attempt
(open-port/run-20261119T125538Z) was NOT ADOPTED, and that verdict stands. This scorer is the
successor's own; `bench/open_port.py`, the first registration's, is unchanged and still re-judges
the first attempt.

    open_port2.py judge BUNDLE --port PORT_COMMIT
    open_port2.py verify BUNDLE --port PORT_COMMIT   (before any timing: the builds and inputs)
    open_port2.py guard TARGET --this RUN_DIR [--remote origin]   (the successor's one attempt)

**What is the same as the first registration** (`open_port.py`'s own code, imported):
- the builds, the machine, and the 18 inputs, checked against the manifest
  `bench/open-port-inputs.sha256`;
- the decision run, `runs.jsonl`: the 15 corpora at 8, 10 and 20 threads, 12 paired rounds each,
  every field checked before use;
- P1 (Holm over the three counts on wikidata-x4), P2 (its peak RSS at most 1.00), H
  (ho-wikidata-l9 FASTER at every count), and every guard's peak RSS at most 1.10, all from those
  12 rounds.

**What is new: the guards' wall-time rule, G2** (replacing G's point bound 1.02 and its
per-cell SLOWER test, whose false-fail rate for a port with no effect was about 0.96):
- *the cells:* the 13 guard corpora at the three counts (39 cells), and the sustained cell;
- *the rounds:*
  - tier A (`TIER_A`): 120 rounds a cell, the decision run's 12 and 108 more
    (`guards-a.jsonl`);
  - tier B: 60 rounds a cell, 12 and 48 more (`guards-b.jsonl`);
  - the sustained cell: 40 rounds (`sustained.jsonl`);
- *the test:* a cell fails if a one-sided exact Wilcoxon signed-rank test of "the port is slower",
  on the rounds' paired log ratios, gives p <= 0.048/40 = 0.0012. Zero differences are dropped;
  tied magnitudes take their mean rank, and the exact null distribution is computed for those
  ranks.
- *Its false-fail rate:* for a port with no true effect on any guard, the arms are exchangeable in
  every round, each test's size is at most 0.0012, and P(any of the 40 fails) <= 0.048. With the
  memory guard's (<= 0.0015, by resampling the first attempt), the whole guard family's is at most
  0.05.

**VOID, and nothing is read,** where `open_port.py`'s conditions hold of any of the four files, or
where a file is missing or empty, a record lies outside its roster, or a field is malformed.

**ADOPTS** only where P1, P2, H, every guard's memory and G2 hold. Otherwise **NOT ADOPTED**, every
failing condition named. One attempt (`guard`), archived as `open-port-2/run-*.tar.gz`.
Standard library only.
"""

import argparse
import glob
import math
import os
import re
import statistics
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import open_port as op  # noqa: E402

TIER_A = ("runs", "ho-prose-l1", "ho-code-l1", "ho-code-l9")
GUARDS = tuple(c for c in op.CORPORA if c not in (op.MAIN, op.HELDOUT))
TIER_B = tuple(c for c in GUARDS if c not in TIER_A)
ROUNDS_A, ROUNDS_B, SUSTAINED_ROUNDS = 120, 60, 40
EXTRA_A, EXTRA_B = ROUNDS_A - op.ROUNDS, ROUNDS_B - op.ROUNDS
FAMILY = len(GUARDS) * len(op.COUNTS) + 1
ALPHA2 = 0.048 / FAMILY
PREFIX = "open-port-2"


def signed_rank_p(xs):
    """One-sided exact p of the Wilcoxon signed-rank test that `xs` lie above 0: P(W+ >= the
    observed W+) when each nonzero value's sign is + or - with probability 1/2, its magnitude's rank
    fixed. Zeros are dropped; tied magnitudes take their mean rank (doubled to an integer here)."""
    xs = [x for x in xs if x != 0]
    n = len(xs)
    if n == 0:
        return 1.0
    order = sorted(range(n), key=lambda i: abs(xs[i]))
    rank2 = [0] * n
    i = 0
    while i < n:
        j = i
        while j + 1 < n and abs(xs[order[j + 1]]) == abs(xs[order[i]]):
            j += 1
        for k in range(i, j + 1):
            rank2[order[k]] = i + j + 2  # twice the mean of ranks i+1 .. j+1
        i = j + 1
    w2 = sum(r for r, x in zip(rank2, xs) if x > 0)
    total = sum(rank2)
    dist = [0.0] * (total + 1)
    dist[0] = 1.0
    top = 0
    for r in rank2:
        new = [0.0] * (total + 1)
        for s in range(top + 1):
            p = dist[s]
            if p:
                new[s] += 0.5 * p
                new[s + r] += 0.5 * p
        dist, top = new, top + r
    return min(1.0, sum(dist[w2:]))


def ratios(rows, corpus, t, rounds):
    return [
        math.log(rows[(corpus, t, "cand", i)]["wall"] / rows[(corpus, t, "ref", i)]["wall"])
        for i in range(rounds)
    ]


def summary(xs):
    m = statistics.mean(xs)
    h = op.t975(len(xs) - 1) * statistics.stdev(xs) / math.sqrt(len(xs))
    return math.exp(m), math.exp(m - h), math.exp(m + h)


def judge(a):
    counts, corpora = list(op.COUNTS), list(op.CORPORA)
    errors = []
    if os.path.exists(os.path.join(a.bundle, "DRY-RUN.txt")):
        errors.append("a dry run (DRY-RUN.txt in the bundle): not evidence")
    model = op.machine(
        os.path.join(a.bundle, "lscpu.txt"), os.path.join(a.bundle, "env.json"), errors
    )
    op.identities(a.bundle, a.port, errors)
    rows = op.load(os.path.join(a.bundle, "runs.jsonl"), corpora, counts, op.ROUNDS, errors)
    arows = op.load(os.path.join(a.bundle, "guards-a.jsonl"), TIER_A, counts, EXTRA_A, errors)
    brows = op.load(os.path.join(a.bundle, "guards-b.jsonl"), TIER_B, counts, EXTRA_B, errors)
    srows = op.load(
        os.path.join(a.bundle, "sustained.jsonl"), [op.SUSTAINED], [10], SUSTAINED_ROUNDS, errors
    )
    want = (
        len(corpora) * len(counts) * 2 * op.ROUNDS,
        len(TIER_A) * len(counts) * 2 * EXTRA_A,
        len(TIER_B) * len(counts) * 2 * EXTRA_B,
        2 * SUSTAINED_ROUNDS,
    )
    print(
        f"records {len(rows)} of {want[0]} (decision), {len(arows)} of {want[1]} (guards, tier A), "
        f"{len(brows)} of {want[2]} (guards, tier B), {len(srows)} of {want[3]} (sustained)"
        + (f"; the machine: {model}" if model else "")
    )
    if errors:
        print(
            f"VOID: the records do not make the registered run ({len(errors)} problems); no verdict. "
            "This registration's one confirmatory attempt gives no adoption evidence, and opens no retry."
        )
        for e in errors[:25]:
            print(f"  {e}")
        return 1
    fails, ps, res = [], {}, {}
    print(
        "\nthe decision run (12 rounds): the port over v0.5.6, wall as the rounds' paired ratios' "
        "geometric mean [95% interval]; one-sided p (gain); peak RSS, median over median"
    )
    for t in counts:
        print(f"  {t} threads")
        for c in corpora:
            g, lo, hi, p, mem = res[(c, t)] = op.cell(rows, c, t, op.ROUNDS)
            role = (
                "primary"
                if c == op.MAIN
                else "held-out replication"
                if c == op.HELDOUT
                else "guard"
            )
            print(
                f"    {c} ({role}): {g:.4f} [{lo:.4f}, {hi:.4f}] {op.label(lo, hi)}; p {p:.2e}; memory {mem:.3f}"
            )
            if c == op.MAIN:
                ps[t] = p
    rejected = op.holm(ps, op.ALPHA)
    print(
        f"\nP1, wall on {op.MAIN}: Holm at α {op.ALPHA} over {len(ps)} cells: "
        + ", ".join(
            f"{t} threads p {ps[t]:.2e} {'rejects' if t in rejected else 'does not'}"
            for t in counts
        )
    )
    for t in counts:
        if t not in rejected:
            fails.append(f"P1: no gain on {op.MAIN} at {t} threads")
        if res[(op.MAIN, t)][4] > 1.0:
            fails.append(
                f"P2: peak RSS on {op.MAIN} at {t} threads {res[(op.MAIN, t)][4]:.3f}, above 1.00"
            )
        g, lo, hi, p, mem = res[(op.HELDOUT, t)]
        if not hi < 1:
            fails.append(f"H: {op.HELDOUT} at {t} threads {g:.4f} [{lo:.4f}, {hi:.4f}], not FASTER")
        for c in GUARDS:
            if res[(c, t)][4] > op.RSS_GATE:
                fails.append(
                    f"G: {c} at {t} threads peak RSS {res[(c, t)][4]:.3f}, above {op.RSS_GATE}"
                )
    print(
        f"\nG2, the guards' wall time: one-sided exact Wilcoxon signed-rank (the port slower), "
        f"fails at p <= {ALPHA2:.4f} (0.048 over {FAMILY} cells)"
    )
    for t in counts:
        print(f"  {t} threads")
        for c in GUARDS:
            extra, n = (arows, EXTRA_A) if c in TIER_A else (brows, EXTRA_B)
            xs = ratios(rows, c, t, op.ROUNDS) + ratios(extra, c, t, n)
            p = signed_rank_p(xs)
            g, lo, hi = summary(xs)
            bad = p <= ALPHA2
            print(
                f"    {c}: {len(xs)} rounds, {g:.4f} [{lo:.4f}, {hi:.4f}]; p (slower) {p:.2e}"
                + (" FAILS" if bad else "")
            )
            if bad:
                fails.append(f"G2: {c} at {t} threads slower, p {p:.2e} <= {ALPHA2:.4f}")
    xs = ratios(srows, op.SUSTAINED, 10, SUSTAINED_ROUNDS)
    p = signed_rank_p(xs)
    g, lo, hi = summary(xs)
    print(
        f"\nS, sustained: 10 threads back to back on {op.SUSTAINED}, {SUSTAINED_ROUNDS} rounds: "
        f"{g:.4f} [{lo:.4f}, {hi:.4f}]; p (slower) {p:.2e}" + (" FAILS" if p <= ALPHA2 else "")
    )
    if p <= ALPHA2:
        fails.append(f"G2: sustained slower, p {p:.2e} <= {ALPHA2:.4f}")
    if fails:
        print(f"\nVERDICT: NOT ADOPTED. {len(fails)} conditions fail:")
        for f in fails:
            print(f"  {f}")
    else:
        print(
            "\nVERDICT: ADOPTS. A wall gain on Wikidata at every count (Holm), peak RSS not above "
            "v0.5.6's, the gain replicated on held-out Wikidata, every guard's memory within 1.10, "
            "and no guard nor the sustained cell slower (G2)."
        )
    return 0


def verify(a):
    """Before any timing: the bundle's builds and inputs are the registered ones"""
    return op.verify(a)


def guard(a):
    """The successor's one attempt: as `open_port.py guard`, for its own evidence path. Exit 0 only
    if no timed attempt has started in `target` (a run directory with `timing-started.txt`, other
    than `--this`) and the evidence branch, fetched afresh, is readable and archives no
    `open-port-2/run-*.tar.gz`. Fails closed. The first registration's `open-port/` bundle is not
    this registration's attempt."""
    for started in sorted(glob.glob(os.path.join(a.target, "run-*", "timing-started.txt"))):
        d = os.path.dirname(started)
        if a.this and os.path.realpath(d) == os.path.realpath(a.this):
            continue
        if not os.path.exists(os.path.join(d, "DRY-RUN.txt")):
            print(
                f"bench-open-port-2: a timed attempt already started here ({os.path.basename(d)}): "
                "this registration allows one"
            )
            return 1
    ref = f"refs/remotes/{a.remote}/evidence"

    def git(*args):
        return subprocess.run(["git", *args], capture_output=True, text=True)

    if git("fetch", "-q", a.remote, f"+refs/heads/evidence:{ref}").returncode != 0:
        print(
            f"bench-open-port-2: the evidence branch can't be fetched from {a.remote}, so a timed "
            "attempt archived there can't be ruled out: refusing (this registration allows one)"
        )
        return 1
    listing = git("ls-tree", "-r", "--name-only", ref, "--", PREFIX)
    if listing.returncode != 0:
        print(
            f"bench-open-port-2: the evidence branch ({ref}) can't be read: refusing "
            "(this registration allows one)"
        )
        return 1
    runs = [n for n in listing.stdout.split() if re.fullmatch(rf"{PREFIX}/run-[^/]*\.tar\.gz", n)]
    if runs:
        print(
            f"bench-open-port-2: a timed attempt is already archived on the evidence branch "
            f"({runs[0]}): this registration allows one"
        )
        return 1
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="mode", required=True)
    for name in ("judge", "verify"):
        j = sub.add_parser(name)
        j.add_argument("bundle")
        j.add_argument("--port", required=True)
    g = sub.add_parser("guard")
    g.add_argument("target")
    g.add_argument("--this")
    g.add_argument("--remote", default="origin")
    a = ap.parse_args(argv[1:])
    return {"judge": judge, "verify": verify, "guard": guard}[a.mode](a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
