#!/usr/bin/env python3
"""bench-supplied-codes' verdict (`just bench-supplied-codes`): the open kernel's emitter handed
the block's own word codes in marked-cycle output order (arm B, `supplied`) against the kernel
as shipped (arm A, `open`), in production's own workers, read from
examples/supplied_codes.rs's records once they are checked whole. Diagnostic only: it decides
no production default. The registration, rule and all, is
docs/journal/2026-11-24-supplied-codes-registration.md, frozen before the data. Standard library
only; the paired-ratio interval is bench/two_arm.py's.

    supplied_codes.py RUNS.tsv --lscpu LSCPU.txt [--workers 1,10] [--judged 10] [--rounds 20]
                      [--out-bytes N]

**The primary receipt** (the decodes with nothing conditioned, cond 0): each round's paired
ratio of whole-decoder wall times, B over A, at the same worker count; their geometric mean and
95% interval (Student's t on the logs, `two_arm.step`). At the judged count (10 workers):
- **ADVANCE** iff the interval's upper bound < 0.90;
- **STOP** iff its lower bound > 0.95;
- **INCONCLUSIVE** otherwise.
1 worker is reported and decides nothing.

**The diagnostic receipt** (the conditioned decodes, cond 1; decides nothing): each round's
ratio of the consumer's time per retained row, sum(consume_ns) / sum(m) over the open blocks,
B over A, with the same interval. Also reported: each arm's construct, deliver, condition and
consume per row; B's charged emit (deliver + consume) against A's emit in the primary decodes;
the distinct blocks (by key id; the corpus's repeated copies are repeated observations of them,
not independent ones), their count and the spread of their own consumer ratios; the marked
cycles' lengths against the retained rows; the peak scratch per arm (a worker's kernel) and the
store's bytes (the supplied codes, held for arm B).

**VOID** (exit 1, no verdict): the file missing or malformed; the machine not an i9-10900K; the
`meta` line missing or duplicated, its store empty or with a collision; any (round, workers,
arm, cond) decode missing, duplicated or unexpected; any round's check missing, duplicated,
with a mismatch or no block checked; any decode's or check's output not OUT_BYTES; a decode
with no open block; a block recorded under another arm or cond than its decode's; an arm-A
block supplied, or an arm-B open block not supplied; the blocks' (n, m, refused) not the same
multiset in every decode, or the two-ended count not the same.
"""

import argparse
import math
import os
import statistics
import sys
from collections import Counter, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import two_arm as ta  # noqa: E402

MODEL = "i9-10900K"
# bench/wikidata.json's decoded bytes (the corpus's 209,700,196), four copies
WIKIDATA_X4_OUT = 4 * 209_700_196
# the frozen rule's lines (the registration)
ADVANCE_BELOW = 0.90
STOP_ABOVE = 0.95
COLS: list[str] = [
    str(c)
    for c in (
        "kind round workers arm cond order seq n m cycle bidi refused supplied construct_ns "
        "deliver_ns cond_ns consume_ns scratch id wall_s bytes mismatches"
    ).split()
]
INTS = "n m cycle bidi refused supplied construct_ns deliver_ns cond_ns consume_ns scratch".split()
ARMS = ("open", "supplied")


def counts(s):
    return [int(x) for x in s.split(",") if x]


def machine(lscpu, errors):
    try:
        with open(lscpu) as f:
            text = f.read()
    except OSError as e:
        errors.append(f"{os.path.basename(lscpu)}: no lscpu record ({e})")
        return
    names = [
        line.split(":", 1)[1].strip()
        for line in text.splitlines()
        if line.split(":", 1)[0].strip() == "Model name"
    ]
    if not any(MODEL in n for n in names):
        errors.append(f"lscpu: model {names!r}, not the {MODEL}")


def ratio_interval(ratios):
    """(geometric mean, 95% low, 95% high) of paired ratios: two_arm.step on their logs"""
    g, lo, hi, _ = ta.step([math.log(r) for r in ratios])
    return g, lo, hi


def interval(xs):
    """(mean, 95% low, 95% high), Student's t"""
    n = len(xs)
    mu = statistics.mean(xs)
    if n < 2:
        return mu, math.nan, math.nan
    t95 = ta.T95[n - 2] if n - 1 <= len(ta.T95) else 1.96
    h = t95 * statistics.stdev(xs) / math.sqrt(n)
    return mu, mu - h, mu + h


def load(path, workers, rounds, out_bytes, errors):
    """(meta, {(round, workers, arm, cond): (run, [blocks])}, {round: check}) once checked"""
    want = {(r, w, a, c) for r in range(rounds) for w in workers for a in ARMS for c in (0, 1)}
    runs, blks, checks, metas, order = {}, defaultdict(list), {}, [], {}
    try:
        with open(path) as f:
            lines = f.read().splitlines()
    except OSError as e:
        errors.append(f"{os.path.basename(path)}: unreadable ({e})")
        return None, {}, {}
    if not lines or lines[0].split("\t") != COLS:
        errors.append(f"{os.path.basename(path)}: not supplied_codes' header")
        return None, {}, {}
    pending = []
    for i, line in enumerate(lines[1:], 2):
        where = f"{os.path.basename(path)}:{i}"
        f = line.split("\t")
        if len(f) != len(COLS):
            errors.append(f"{where}: {len(f)} fields, not {len(COLS)}")
            continue
        d = dict(zip(COLS, f))
        try:
            if d["kind"] == "meta":
                metas.append(
                    {
                        "keys": int(d["n"]),
                        "codes": int(d["m"]),
                        "bytes": int(d["scratch"]),
                        "collisions": int(d["mismatches"]),
                        "out": int(d["bytes"]),
                    }
                )
                continue
            if d["kind"] == "check":
                r = int(d["round"])
                if r in checks:
                    errors.append(f"{where}: round {r}'s check duplicated")
                checks[r] = {
                    "checked": int(d["supplied"]),
                    "bytes": int(d["bytes"]),
                    "mismatches": int(d["mismatches"]),
                }
                continue
            key = (int(d["round"]), int(d["workers"]), d["arm"], int(d["cond"]))
            if d["kind"] == "blk":
                rec = {k: int(d[k]) for k in INTS}
                rec["id"] = int(d["id"], 16)
                pending.append((where, key, int(d["order"]), rec))
            elif d["kind"] == "run":
                if key not in want:
                    errors.append(f"{where}: unexpected decode {key}")
                elif key in runs:
                    errors.append(f"{where}: decode {key} duplicated")
                else:
                    runs[key] = {"wall": float(d["wall_s"]), "bytes": int(d["bytes"])}
                    order[(key[0], key[1], int(d["order"]))] = key
            else:
                errors.append(f"{where}: kind {d['kind']!r}")
        except ValueError:
            errors.append(f"{where}: malformed")
    # each block belongs to the decode at its (round, workers, order), whose arm and cond it
    # must carry
    for where, key, o, rec in pending:
        dec = order.get((key[0], key[1], o))
        if dec is None:
            errors.append(f"{where}: a block of no decode {key} (order {o})")
        elif dec != key:
            errors.append(f"{where}: a block under {key[2:]} in the {dec[2:]} decode")
        else:
            blks[key].append(rec)
    if len(metas) != 1:
        errors.append(f"{len(metas)} meta lines, not 1")
        meta = None
    else:
        meta = metas[0]
        if meta["keys"] <= 0 or meta["codes"] <= 0:
            errors.append(f"the store is empty: {meta}")
        if meta["collisions"]:
            errors.append(f"the store has {meta['collisions']} collisions")
        if meta["out"] != out_bytes:
            errors.append(f"the oracle pass wrote {meta['out']} bytes, not {out_bytes}")
    for key in sorted(want - set(runs)):
        errors.append(f"missing decode {key}")
    for r in range(rounds):
        c = checks.get(r)
        if c is None:
            errors.append(f"round {r}'s check missing")
        elif c["mismatches"] or c["checked"] <= 0 or c["bytes"] != out_bytes:
            errors.append(f"round {r}'s check: {c}")
    for r in set(checks) - set(range(rounds)):
        errors.append(f"an unexpected check, round {r}")
    shapes, bidis = set(), set()
    for key, run in sorted(runs.items()):
        if run["bytes"] != out_bytes:
            errors.append(f"decode {key} wrote {run['bytes']} bytes, not {out_bytes}")
        if not math.isfinite(run["wall"]) or run["wall"] <= 0:
            errors.append(f"decode {key} wall {run['wall']} not finite and positive")
        bs = blks.get(key, [])
        open_ = [b for b in bs if not b["refused"]]
        if not open_:
            errors.append(f"decode {key}: no block the open kernel took")
        if key[2] == "open" and any(b["supplied"] or b["cycle"] for b in open_):
            errors.append(f"decode {key}: an arm-A block supplied")
        if key[2] == "supplied" and any(not b["supplied"] or not b["cycle"] for b in open_):
            errors.append(f"decode {key}: an arm-B block not supplied")
        shapes.add(frozenset(Counter((b["n"], b["m"], b["refused"]) for b in bs).items()))
        bidis.add(sum(b["bidi"] for b in open_))
    if len(shapes) > 1:
        errors.append(f"the blocks' (n, m, refused) differ between decodes ({len(shapes)} shapes)")
    if len(bidis) > 1:
        errors.append(f"the two-ended block counts differ between decodes: {sorted(bidis)}")
    return meta, {k: (runs[k], blks.get(k, [])) for k in runs}, checks


def per_decode(recs):
    open_ = [x for x in recs if not x["refused"]]
    m = sum(x["m"] for x in open_)

    def row(field):
        return sum(x[field] for x in open_) / m

    return {
        "m": m,
        "open": len(open_),
        "bidi": sum(x["bidi"] for x in open_),
        "construct": row("construct_ns"),
        "deliver": row("deliver_ns"),
        "cond": row("cond_ns"),
        "consume": row("consume_ns"),
        "scratch": max((x["scratch"] for x in open_), default=0),
    }


def fmt(x, d=3):
    return f"{x[0]:.{d}f} [{x[1]:.{d}f}, {x[2]:.{d}f}]"


def quantiles(xs):
    xs = sorted(xs)
    if not xs:
        return "none"
    q = lambda p: xs[min(len(xs) - 1, int(p * (len(xs) - 1) + 0.5))]  # noqa: E731
    return f"min {xs[0]:.3f}, p10 {q(0.1):.3f}, median {q(0.5):.3f}, p90 {q(0.9):.3f}, max {xs[-1]:.3f}"


def classify(g):
    """the frozen rule's decision from the primary ratio's (geometric mean, low, high)"""
    if g[2] < ADVANCE_BELOW:
        return "ADVANCE"
    if g[1] > STOP_ABOVE:
        return "STOP"
    return "INCONCLUSIVE"


def report(meta, got, workers, rounds, judged):
    lines, decision = [], None
    for w in workers:
        is_judged = w == judged
        lines.append(f"## {w} workers ({rounds} rounds){' (judged)' if is_judged else ''}")
        per = {
            (a, c): [per_decode(got[(r, w, a, c)][1]) for r in range(rounds)]
            for a in ARMS
            for c in (0, 1)
        }
        walls = {
            (a, c): [got[(r, w, a, c)][0]["wall"] for r in range(rounds)]
            for a in ARMS
            for c in (0, 1)
        }
        d0 = per[("open", 0)][0]
        lines.append(
            f"open blocks a decode {d0['open']} ({d0['bidi']} two-ended); retained rows a decode "
            f"{d0['m']}"
        )
        for c in (0, 1):
            for a in ARMS:
                ps = per[(a, c)]
                lines.append(
                    f"{a}, cond {c}: wall median {statistics.median(walls[(a, c)]):.4f} s; per "
                    f"retained row, ns: construct {fmt(interval([p['construct'] for p in ps]))}, "
                    f"deliver {fmt(interval([p['deliver'] for p in ps]))}, condition "
                    f"{fmt(interval([p['cond'] for p in ps]))}, consume "
                    f"{fmt(interval([p['consume'] for p in ps]))}"
                )
        prim = ratio_interval(
            [walls[("supplied", 0)][r] / walls[("open", 0)][r] for r in range(rounds)]
        )
        cls = classify(prim)
        lines.append(
            f"PRIMARY: whole-decoder wall, B/A (cond 0, delivery charged): {fmt(prim)}; by the "
            f"rule {cls}" + ("" if is_judged else " (decides nothing at this count)")
        )
        diag = ratio_interval(
            [
                per[("supplied", 1)][r]["consume"] / per[("open", 1)][r]["consume"]
                for r in range(rounds)
            ]
        )
        lines.append(
            f"DIAGNOSTIC: consumer per retained row, B/A (cond 1, the substitution outside the "
            f"clock): {fmt(diag)} (decides nothing)"
        )
        charged = ratio_interval(
            [
                (per[("supplied", 0)][r]["deliver"] + per[("supplied", 0)][r]["consume"])
                / per[("open", 0)][r]["consume"]
                for r in range(rounds)
            ]
        )
        lines.append(
            f"emit with the delivery charged, B (deliver + consume) / A (consume), cond 0: "
            f"{fmt(charged)} (decides nothing)"
        )
        wall_cond = ratio_interval(
            [walls[("supplied", 1)][r] / walls[("open", 1)][r] for r in range(rounds)]
        )
        lines.append(f"whole-decoder wall, B/A, cond 1: {fmt(wall_cond)} (decides nothing)")
        # the distinct blocks: each key id's consumer time pooled over its copies and rounds
        by = defaultdict(lambda: [0, 0, 0])
        for r in range(rounds):
            for a, j in (("open", 0), ("supplied", 1)):
                for b in got[(r, w, a, 1)][1]:
                    if not b["refused"]:
                        by[b["id"]][j] += b["consume_ns"]
                        if a == "supplied":
                            by[b["id"]][2] = b["bidi"]
        rs = [v[1] / v[0] for v in by.values() if v[0] > 0]
        rs2 = [v[1] / v[0] for v in by.values() if v[0] > 0 and v[2]]
        lines.append(
            f"distinct open blocks {len(by)} ({sum(1 for v in by.values() if v[2])} two-ended); "
            f"their consumer ratios B/A, cond 1, each pooled over its copies and rounds: "
            f"{quantiles(rs)}; two-ended only: {quantiles(rs2)}"
        )
        scratch = {a: max(p["scratch"] for c in (0, 1) for p in per[(a, c)]) for a in ARMS}
        lines.append(
            f"peak scratch, a worker's kernel: open {scratch['open']} bytes, supplied "
            f"{scratch['supplied']} bytes (arm B adds none: the codes go in edges); up to {w} "
            f"kernels at once"
        )
        if is_judged:
            decision = (cls, prim, diag)
        lines.append("")
    # the cycles, from one arm-B decode at the first count
    recs = [b for b in got[(0, workers[0], "supplied", 0)][1] if not b["refused"]]
    short = [b for b in recs if b["cycle"] < b["m"]]
    lines.append(
        f"marked cycles: {len(short)} of {len(recs)} open blocks a decode with the cycle shorter "
        f"than the retained rows (the repeat); codes supplied a decode "
        f"{sum(b['cycle'] for b in recs)} against {sum(b['m'] for b in recs)} retained rows"
    )
    lines.append(
        f"the store (arm B's supplied codes, held in memory for the run, outside every clock "
        f"but the copy): {meta['keys']} distinct blocks, {meta['codes']} codes, {meta['bytes']} "
        f"bytes"
    )
    return lines, decision


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--workers", default="1,10")
    ap.add_argument("--judged", type=int, default=10)
    ap.add_argument("--rounds", type=int, default=20)
    ap.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_OUT)
    a = ap.parse_args(argv)
    workers = counts(a.workers)
    errors = []
    if a.judged not in workers:
        errors.append(f"--judged {a.judged} is not among --workers {a.workers}")
    if a.rounds < 2:
        errors.append(f"--rounds {a.rounds}: at least 2")
    machine(a.lscpu, errors)
    meta, got, _ = load(a.runs, workers, a.rounds, a.out_bytes, errors)
    if errors:
        print("VOID: no verdict")
        for e in errors[:40]:
            print(f"- {e}")
        return 1
    assert meta is not None
    lines, decision = report(meta, got, workers, a.rounds, a.judged)
    print("# bench-supplied-codes (diagnostic: decides no production default)")
    print()
    print("\n".join(lines))
    cls, prim, diag = decision
    print()
    print(
        f"DECISION: {cls} (whole-decoder wall B/A at {a.judged} workers {fmt(prim)}; ADVANCE "
        f"iff the upper bound < {ADVANCE_BELOW}, STOP iff the lower bound > {STOP_ABOVE}; the "
        f"diagnostic consumer ratio {fmt(diag)} decides nothing)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
