#!/usr/bin/env python3
"""The open port's successor adoption verdict (`just bench-open-port-2`; the successor
registration docs/journal/2026-11-19-open-port-2-registration.md; #145 QUEUED 7011091171, and the
Researcher's correction REVIEW 7012181194 with the Coordinator's decisions, QUEUED 7012199074).

A new experiment, not a retry. The first registration's one attempt
(open-port/run-20261119T125538Z) was NOT ADOPTED, and that verdict stands. This scorer is the
successor's own; `bench/open_port.py`, the first registration's, is unchanged and still re-judges
the first attempt.

    open_port2.py judge BUNDLE --port PORT_COMMIT
    open_port2.py verify BUNDLE --port PORT_COMMIT   (before any timing: the builds and inputs)
    open_port2.py schedule   (the extension's batches, one line each: THREADS ROUNDS CORPUS...)
    open_port2.py guard TARGET --this RUN_DIR [--remote origin]   (the successor's one attempt)

**What is the same as the first registration** (`open_port.py`'s own code, imported):
- the builds, the machine, and the 18 inputs, checked against `bench/open-port-inputs.sha256`;
- the decision run, `runs.jsonl`: the 15 corpora at 8, 10 and 20 threads, 12 paired rounds each;
- P1, P2 and H, and every guard's peak RSS at most 1.10, all from those 12 rounds;
- S, exactly as registered: `sustained.jsonl`, 8 rounds, failing on SLOWER or a geometric mean
  above 1.02.

**What is new: the guards' wall-time rule, G2** (replacing G's point bound 1.02 and its per-cell
SLOWER test):
- *the cells:* the 13 guard corpora at the three counts, 39 cells;
- *the rounds:* each cell's own count, locked in `bench/open-port-2-schedule.json`. That is the
  decision run's 12, and the rest in the extension, one `ab run` a batch
  (`guards-t{threads}-r{rounds}.jsonl`);
- *the estimand:* G's. The mean of the cell's paired log ratios, the port's wall over v0.5.6's,
  over all its rounds; its exponential is the geometric mean;
- *the rule:* a cell fails if that mean exceeds its critical value c, locked in the schedule. c
  was set before any data by simulation from the first attempt's noise. It is the largest of the
  (1 - 0.05/39) quantiles of the mean, with no effect, under three models: the cell's own
  residuals, its corpus's, and all the guards'. The rounds give power 0.80 at a true +2% slowdown.
  The calibration is model-based, not exact; the registration states its assumptions.

**VOID, and nothing is read,** where `open_port.py`'s conditions hold of any file, or across them:
a corpus's output size and digest differ between any two files, or a build's arm differs between
any two files.

**ADOPTS** only where P1, P2, H, S, every guard's memory and G2 hold. Otherwise **NOT ADOPTED**,
every failing condition named. One attempt (`guard`), archived as `open-port-2/run-*.tar.gz`.
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

GUARDS = tuple(c for c in op.CORPORA if c not in (op.MAIN, op.HELDOUT))
SCHEDULE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "open-port-2-schedule.json")
PREFIX = "open-port-2"


def schedule(path=SCHEDULE):
    """{(corpus, threads): (rounds, c)}, the locked plan: every guard cell once"""
    cells = {}
    for e in json.load(open(path))["cells"]:
        k = (e["corpus"], e["threads"])
        assert (
            k not in cells and e["rounds"] >= op.ROUNDS and 0 < e["c"] < math.log(op.GUARD_MAX)
        ), e
        cells[k] = (e["rounds"], e["c"])
    assert sorted(cells) == sorted((c, t) for c in GUARDS for t in op.COUNTS), sorted(cells)
    return cells


PLAN = schedule()


def batches(plan=PLAN):
    """[(threads, extra rounds, [corpora])]: the extension, one `ab run` each, in a fixed order"""
    groups = {}
    for (c, t), (n, _) in plan.items():
        if n > op.ROUNDS:
            groups.setdefault((t, n - op.ROUNDS), []).append(c)
    return [(t, x, [c for c in GUARDS if c in cs]) for (t, x), cs in sorted(groups.items())]


def batch_file(t, extra):
    return f"guards-t{t}-r{extra}.jsonl"


def ratios(rows, corpus, t, rounds):
    return [
        math.log(rows[(corpus, t, "cand", i)]["wall"] / rows[(corpus, t, "ref", i)]["wall"])
        for i in range(rounds)
    ]


def summary(xs):
    m = statistics.mean(xs)
    h = op.t975(len(xs) - 1) * statistics.stdev(xs) / math.sqrt(len(xs))
    return m, math.exp(m), math.exp(m - h), math.exp(m + h)


def joins(files, errors):
    """Across every file the judge reads: one output (size, digest) for each corpus, and one arm
    for each build (the Researcher's correction: a different output or binary in an extension
    file is VOID)"""
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
    srows = op.load(
        os.path.join(a.bundle, "sustained.jsonl"), [op.SUSTAINED], [10], op.SUSTAINED_ROUNDS, errors
    )
    files = [("runs.jsonl", rows), ("sustained.jsonl", srows)]
    ext = {}
    for t, extra, cs in batches():
        name = batch_file(t, extra)
        b = op.load(os.path.join(a.bundle, name), cs, [t], extra, errors)
        files.append((name, b))
        for c in cs:
            ext[(c, t)] = (b, extra)
    stray = sorted(
        set(map(os.path.basename, glob.glob(os.path.join(a.bundle, "guards-*.jsonl"))))
        - {n for n, _ in files}
    )
    if stray:
        errors.append(f"files outside the schedule: {stray}")
    joins(files, errors)
    want = sum(2 * (n - op.ROUNDS) for n, _ in PLAN.values())
    got = sum(len(b) for n, b in files if n.startswith("guards-"))
    print(
        f"records {len(rows)} of {len(corpora) * len(counts) * 2 * op.ROUNDS} (decision), "
        f"{len(srows)} of {2 * op.SUSTAINED_ROUNDS} (sustained), {got} of {want} (the guards' "
        f"extension, {len(batches())} batches)" + (f"; the machine: {model}" if model else "")
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
        "\nG2, the guards' wall time: the mean of each cell's paired log ratios over all its rounds "
        "(the geometric mean, G's estimand) against its locked critical value c"
    )
    for t in counts:
        print(f"  {t} threads")
        for c in GUARDS:
            n, crit = PLAN[(c, t)]
            xs = ratios(rows, c, t, op.ROUNDS)
            if (c, t) in ext:
                b, extra = ext[(c, t)]
                xs += ratios(b, c, t, extra)
            m, g, lo, hi = summary(xs)
            bad = m > crit
            print(
                f"    {c}: {n} rounds, {g:.4f} [{lo:.4f}, {hi:.4f}]; mean log ratio {m:+.5f} against "
                f"c {crit:.4f} (x{math.exp(crit):.4f})" + (" FAILS" if bad else "")
            )
            if bad:
                fails.append(
                    f"G2: {c} at {t} threads, mean log ratio {m:+.5f} (x{g:.4f}) above c {crit:.4f}"
                )
    g, lo, hi, p, mem = op.cell(srows, op.SUSTAINED, 10, op.SUSTAINED_ROUNDS)
    print(
        f"\nS, sustained (as registered): 10 threads back to back on {op.SUSTAINED}, "
        f"{op.SUSTAINED_ROUNDS} rounds: {g:.4f} [{lo:.4f}, {hi:.4f}] {op.label(lo, hi)}; memory {mem:.3f}"
    )
    if lo > 1:
        fails.append(f"S: sustained SLOWER, {g:.4f} [{lo:.4f}, {hi:.4f}]")
    elif g > op.GUARD_MAX:
        fails.append(f"S: sustained {g:.4f}, above the bound {op.GUARD_MAX}")
    if fails:
        print(f"\nVERDICT: NOT ADOPTED. {len(fails)} conditions fail:")
        for f in fails:
            print(f"  {f}")
    else:
        print(
            "\nVERDICT: ADOPTS. A wall gain on Wikidata at every count (Holm), peak RSS not above "
            "v0.5.6's, the gain replicated on held-out Wikidata, every guard's memory within 1.10, "
            "every guard cell's mean within its critical value (G2), and the sustained cell within "
            "its bound."
        )
    return 0


def verify(a):
    """Before any timing: the bundle's builds and inputs are the registered ones"""
    return op.verify(a)


def print_schedule(a):
    for t, extra, cs in batches():
        print(t, extra, *cs)
    return 0


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
    sub.add_parser("schedule")
    g = sub.add_parser("guard")
    g.add_argument("target")
    g.add_argument("--this")
    g.add_argument("--remote", default="origin")
    a = ap.parse_args(argv[1:])
    return {"judge": judge, "verify": verify, "schedule": print_schedule, "guard": guard}[a.mode](a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
