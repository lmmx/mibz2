#!/usr/bin/env python3
"""bench-emit-cost-sq's verdict (`just bench-emit-cost-sq`): MONODROMY record 93's T² two-ended
emit arms (`sq`, `sq-shadow`) against the kernel as shipped (`base`), in production's own
workers, read from examples/emit_cost_sq.rs's records once they are checked whole. Diagnostic
only (C1a): it decides no production default. The rule is frozen before the data: record 93 §5
(docs/journal/monodromy/2026-11-23-squared-trace-arm.md at c3dab1ef), its addendum (b36ec511)
and the Coordinator's LOG 7097052169 on #145; the registration is
docs/journal/2026-11-23-emit-cost-sq-registration.md. Standard library only; the t quantiles
are bench/two_arm.py's.

    emit_cost_sq.py RUNS.tsv --lscpu LSCPU.txt [--workers 1,6,10] [--arms base,sq,sq-shadow]
                    [--judged 10] [--rounds 20] [--out-bytes N]

**Per decode** (one round, one worker count, one arm), over the blocks the open kernel took
(not those its first pass refused), with M = sum(m) the decode's retained rows:
emit = sum(emit_ns) / M and net = sum(construct_ns + square_ns + emit_ns) / M, ns per retained
row (base's square_ns is its two adjacent clock reads' gap: all arms take four reads a block).

**Per arm, paired with base in the same round at the same worker count:**
- E = emit_arm / emit_base;
- K = net_arm - net_base, ns per retained row;
each the mean over the rounds with its 95% interval (Student's t).

**The rule** (at the judged count, 10 workers; 1 and 6 are reported and decide nothing):
- `sq`: KILL if E > 0.80; PORT-CANDIDATE if E <= 0.80, K <= -0.3 and K's 95% interval lies
  wholly below 0; SHADOW otherwise.
- `sq-shadow`: PORT-CANDIDATE by the same condition on its own E and K; END otherwise.
- If `sq` is KILL, the line ends: `sq-shadow` is reported and decides nothing.
- Otherwise the PORT-CANDIDATEs go forward; if both are, the one with the lower K (its mean);
  if neither is, the line ends.

**Reported, not judged:** base's emit at 10 workers against bench-emit-cost's 5.786 ns
(within 10%, else the machine's state has moved); the square pass and the construction's
change per row; the arms' peak scratch growth (`succ` and `x2`, a worker's kernel, against
record 93's 8m bytes).

**VOID** (exit 1, no verdict): the file missing or malformed; the machine not an i9-10900K;
any (round, workers, arm) decode missing, duplicated or unexpected; a block recorded under
another arm than its decode's; any decode's output not OUT_BYTES; a decode with no open block;
the blocks' (n, m, refused) not the same multiset in every decode (the construction is
deterministic, whatever the arm or the worker count); the two-ended block count not the same in
every decode; a base block with the T² walk or the arms' scratch.
"""

import argparse
import math
import os
import statistics
import sys
from collections import Counter

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import two_arm as ta  # noqa: E402

MODEL = "i9-10900K"
# bench/wikidata.json's decoded bytes (the corpus's 209,700,196), four copies
WIKIDATA_X4_OUT = 4 * 209_700_196
# the frozen rule's lines (record 93 §5, LOG 7097052169)
E_KILL = 0.80
K_PORT = -0.3
# bench-emit-cost's base emit at 10 workers (record 93 §1), the reported machine check
BASE_B10 = 5.786
COLS = (
    "kind round workers arm order seq n m construct_ns square_ns emit_ns bidi refused "
    "sq_rounds scratch wall_s bytes"
).split()
INTS = "n m construct_ns square_ns emit_ns bidi refused sq_rounds scratch".split()
ARMS = ("base", "sq", "sq-shadow")


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


def interval(xs):
    """(mean, 95% low, 95% high), Student's t; the mean alone (nan bounds) for one value"""
    n = len(xs)
    mu = statistics.mean(xs)
    if n < 2:
        return mu, math.nan, math.nan
    t95 = ta.T95[n - 2] if n - 1 <= len(ta.T95) else 1.96
    h = t95 * statistics.stdev(xs) / math.sqrt(n)
    return mu, mu - h, mu + h


def load(path, workers, arms, rounds, out_bytes, errors):
    """{(round, workers, arm): (run line, [block records])} once checked; errors appended"""
    want = {(r, w, a) for r in range(rounds) for w in workers for a in arms}
    runs, blks = {}, {}
    try:
        with open(path) as f:
            lines = f.read().splitlines()
    except OSError as e:
        errors.append(f"{os.path.basename(path)}: unreadable ({e})")
        return {}
    if not lines or lines[0].split("\t") != COLS:
        errors.append(f"{os.path.basename(path)}: not emit_cost_sq's header")
        return {}
    for i, line in enumerate(lines[1:], 2):
        where = f"{os.path.basename(path)}:{i}"
        f = line.split("\t")
        if len(f) != len(COLS):
            errors.append(f"{where}: {len(f)} fields, not {len(COLS)}")
            continue
        try:
            key = (int(f[1]), int(f[2]), f[3])
            if f[0] == "blk":
                rec = dict(zip(INTS, map(int, f[6:15])))
            elif f[0] == "run":
                rec = {"wall": float(f[15]), "bytes": int(f[16])}
            else:
                errors.append(f"{where}: kind {f[0]!r}")
                continue
        except ValueError:
            errors.append(f"{where}: malformed")
            continue
        if key not in want:
            errors.append(f"{where}: unexpected decode {key}")
            continue
        if f[0] == "blk":
            blks.setdefault(key, []).append(rec)
        elif key in runs:
            errors.append(f"{where}: decode {key} duplicated")
        else:
            runs[key] = rec
    for key in sorted(want - set(runs)):
        errors.append(f"missing decode {key}")
    shapes, bidis = set(), set()
    for key, run in sorted(runs.items()):
        if run["bytes"] != out_bytes:
            errors.append(f"decode {key} wrote {run['bytes']} bytes, not {out_bytes}")
        if not math.isfinite(run["wall"]) or run["wall"] <= 0:
            errors.append(f"decode {key} wall {run['wall']} not finite and positive")
        bs = blks.get(key, [])
        if not any(not b["refused"] for b in bs):
            errors.append(f"decode {key}: no block the open kernel took")
        if key[2] == "base" and any(b["sq_rounds"] or b["scratch"] for b in bs):
            errors.append(f"decode {key}: a base block with the T² walk or its scratch")
        shapes.add(frozenset(Counter((b["n"], b["m"], b["refused"]) for b in bs).items()))
        bidis.add(sum(b["bidi"] for b in bs if not b["refused"]))
    if len(shapes) > 1:
        errors.append(f"the blocks' (n, m, refused) differ between decodes ({len(shapes)} shapes)")
    if len(bidis) > 1:
        errors.append(f"the two-ended block counts differ between decodes: {sorted(bidis)}")
    return {key: (runs[key], blks.get(key, [])) for key in runs}


def load_arms(path, errors):
    """each blk line's own arm checked against its decode's (the run's `arm` column is the
    decode's; the harness writes the block's own arm in the same column, so a block recorded
    under another arm shows as a mismatch with its run line's order)"""
    try:
        with open(path) as f:
            lines = f.read().splitlines()[1:]
    except OSError:
        return
    order = {}
    for line in lines:
        f = line.split("\t")
        if len(f) == len(COLS) and f[0] == "run":
            order[(f[1], f[2], f[4])] = f[3]
    for i, line in enumerate(lines, 2):
        f = line.split("\t")
        if len(f) == len(COLS) and f[0] == "blk":
            run_arm = order.get((f[1], f[2], f[4]))
            if run_arm is not None and run_arm != f[3]:
                errors.append(
                    f"{os.path.basename(path)}:{i}: a block under {f[3]} in the {run_arm} decode"
                )


def per_decode(recs):
    open_ = [x for x in recs if not x["refused"]]
    m = sum(x["m"] for x in open_)
    return {
        "m": m,
        "emit": sum(x["emit_ns"] for x in open_) / m,
        "square": sum(x["square_ns"] for x in open_) / m,
        "construct": sum(x["construct_ns"] for x in open_) / m,
        "net": sum(x["construct_ns"] + x["square_ns"] + x["emit_ns"] for x in open_) / m,
        "open": len(open_),
        "bidi": sum(x["bidi"] for x in open_),
        "sq_rounds": sum(x["sq_rounds"] for x in open_),
        "scratch": max((x["scratch"] for x in open_), default=0),
        "max_m": max((x["m"] for x in open_ if x["bidi"]), default=0),
    }


def classify(arm, e, k):
    """the frozen rule's class for `arm` from E's and K's (mean, low, high)"""
    port = e[0] <= E_KILL and k[0] <= K_PORT and k[2] < 0
    if arm == "sq":
        return "KILL" if e[0] > E_KILL else "PORT-CANDIDATE" if port else "SHADOW"
    return "PORT-CANDIDATE" if port else "END"


def fmt(x, d=3):
    return f"{x[0]:.{d}f} [{x[1]:.{d}f}, {x[2]:.{d}f}]"


def report(got, workers, arms, rounds, judged):
    lines, judged_ek = [], {}
    for w in workers:
        per = {a: [per_decode(got[(r, w, a)][1]) for r in range(rounds)] for a in arms}
        walls = {
            a: statistics.median(got[(r, w, a)][0]["wall"] for r in range(rounds)) for a in arms
        }
        d0 = per["base"][0]
        lines.append(f"## {w} workers ({rounds} rounds){' (judged)' if w == judged else ''}")
        lines.append(
            f"open blocks a decode {d0['open']} ({d0['bidi']} two-ended); retained rows a decode "
            f"{d0['m']}"
        )
        for a in arms:
            ps = per[a]
            lines.append(
                f"{a}: emit {fmt(interval([p['emit'] for p in ps]))} ns a row; square "
                f"{fmt(interval([p['square'] for p in ps]))}; construct "
                f"{fmt(interval([p['construct'] for p in ps]))}; net "
                f"{fmt(interval([p['net'] for p in ps]))}; T² walk on "
                f"{ps[0]['sq_rounds']} blocks; decode wall, median s {walls[a]:.3f}"
            )
        for a in arms:
            if a == "base":
                continue
            es = [p["emit"] / b["emit"] for p, b in zip(per[a], per["base"])]
            ks = [p["net"] - b["net"] for p, b in zip(per[a], per["base"])]
            dc = [p["construct"] - b["construct"] for p, b in zip(per[a], per["base"])]
            e, k = interval(es), interval(ks)
            cls = classify(a, e, k)
            scratch = max(p["scratch"] for p in per[a])
            max_m = max(p["max_m"] for p in per[a])
            lines.append(
                f"{a} against base: E {fmt(e)}; K {fmt(k)} ns a row; construct change "
                f"{fmt(interval(dc))} ns a row; class by the rule {cls}"
                + ("" if w == judged else " (decides nothing at this count)")
            )
            lines.append(
                f"{a} peak scratch growth: {scratch} bytes in a worker's kernel (succ and x2), "
                f"against 8m = {8 * max_m} bytes at the largest two-ended m {max_m}; up to "
                f"{w} kernels at once"
            )
            if w == judged:
                judged_ek[a] = (e, k, cls)
        if w == judged:
            b = interval([p["emit"] for p in per["base"]])
            near = abs(b[0] - BASE_B10) <= 0.10 * BASE_B10
            lines.append(
                f"machine check (reported, not a VOID): base emit {b[0]:.3f} against "
                f"bench-emit-cost's {BASE_B10} ns: "
                + ("within 10%" if near else "NOT within 10%: the machine's state has moved")
            )
        lines.append("")
    return lines, judged_ek


def decide(judged_ek):
    """the frozen rule's decision from the judged count's classes"""
    if "sq" not in judged_ek or "sq-shadow" not in judged_ek:
        return None
    sq, sh = judged_ek["sq"], judged_ek["sq-shadow"]
    if sq[2] == "KILL":
        return "KILL: sq's E > 0.80, the T² line ends (sq-shadow is reported and decides nothing)"
    ports = [(v[1][0], a) for a, v in (("sq", sq), ("sq-shadow", sh)) if v[2] == "PORT-CANDIDATE"]
    if not ports:
        return "END: neither arm is PORT-CANDIDATE, the T² line ends"
    if len(ports) == 2:
        k, a = min(ports)
        return f"PORT-CANDIDATE: {a} (both are; the lower K, {k:.3f} ns a row), to ordinary adoption with the memory gate"
    return f"PORT-CANDIDATE: {ports[0][1]}, to ordinary adoption with the memory gate"


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--workers", default="1,6,10")
    ap.add_argument("--arms", default=",".join(ARMS))
    ap.add_argument("--judged", type=int, default=10)
    ap.add_argument("--rounds", type=int, default=20)
    ap.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_OUT)
    a = ap.parse_args(argv)
    workers = counts(a.workers)
    arms = [x for x in a.arms.split(",") if x]
    errors = []
    if "base" not in arms or any(x not in ARMS for x in arms):
        errors.append(f"--arms {a.arms!r}: base and only {', '.join(ARMS)}")
    machine(a.lscpu, errors)
    got = load(a.runs, workers, arms, a.rounds, a.out_bytes, errors)
    load_arms(a.runs, errors)
    if errors:
        print("VOID: no verdict")
        for e in errors[:40]:
            print(f"- {e}")
        return 1
    lines, judged_ek = report(got, workers, arms, a.rounds, a.judged)
    print("# bench-emit-cost-sq (diagnostic, C1a: decides no production default)")
    print()
    print("\n".join(lines))
    for arm, (e, k, cls) in judged_ek.items():
        tail = (
            " (decides nothing: sq is KILL)"
            if arm == "sq-shadow" and judged_ek.get("sq", (0, 0, ""))[2] == "KILL"
            else ""
        )
        print(f"{arm} at {a.judged} workers: {cls}{tail}")
    d = decide(judged_ek)
    print(f"DECISION: {d}" if d else "DECISION: none (the judged count or an arm was not run)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
