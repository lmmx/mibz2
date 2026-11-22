#!/usr/bin/env python3
"""bench-emit-cost's report (`just bench-emit-cost`): the open kernel's emit cost per retained row
in production's own workers, read from examples/emit_cost.rs's records once they are checked
whole. Diagnostic only: it decides no production default. The rule is the registration's, frozen
before the data: docs/journal/2026-11-22-emit-cost-registration.md. Standard library only; the
t quantiles are bench/two_arm.py's.

    emit_cost.py RUNS.tsv --lscpu LSCPU.txt [--workers 1,6,10] [--judged 10] [--rounds 20]
                 [--out-bytes N]

**Per decode** (one round at one worker count), over the blocks the open kernel took (not those
its first pass refused): b = sum(emit_ns) / sum(m), ns per retained row (m at LIMIT 16, the
walk's steps); and construct ns per block and per row (sum(construct_ns) / sum(n)).

**Per worker count:** b's mean over the rounds and its 95% interval (Student's t); the same for
construct; and the per-block regression emit_ns = a + slope * m by least squares, pooled over all
the count's blocks and rounds (a and slope), with the per-round slopes' mean and 95% interval.

**The class** (at the judged count, 10 workers; b's mean decides, the interval is reported):
- b > 15 ns: PORT BOTH (LazyTunnel and BSAC);
- b < 8 ns: SHELVE BOTH;
- otherwise: PORT THE CHEAPER.
A class whose interval crosses 8 or 15 is marked STRADDLES. The pooled slope's band is reported
beside it; where it differs, b still decides.

**VOID** (exit 1, no class): the file missing or malformed; the machine not an i9-10900K; any
(round, workers) decode missing, duplicated or unexpected; any decode's output not OUT_BYTES; a
decode with no open block; the blocks' (n, m, refused) not the same multiset in every decode (the
construction is deterministic, whatever the worker count).
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
HIGH, LOW = 15.0, 8.0
COLS = "kind round workers order seq n m construct_ns emit_ns bidi refused wall_s bytes".split()


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


def ols(pts):
    """(intercept, slope) of y on x by least squares"""
    n = len(pts)
    mx = sum(x for x, _ in pts) / n
    my = sum(y for _, y in pts) / n
    sxx = sum((x - mx) ** 2 for x, _ in pts)
    if sxx == 0:
        return my, math.nan
    slope = sum((x - mx) * (y - my) for x, y in pts) / sxx
    return my - slope * mx, slope


def band(b):
    return "PORT BOTH" if b > HIGH else "SHELVE BOTH" if b < LOW else "PORT THE CHEAPER"


def load(path, workers, rounds, out_bytes, errors):
    """{(round, workers): (run line, [block records])} once checked; errors appended"""
    want = {(r, w) for r in range(rounds) for w in workers}
    runs, blks = {}, {}
    try:
        with open(path) as f:
            lines = f.read().splitlines()
    except OSError as e:
        errors.append(f"{os.path.basename(path)}: unreadable ({e})")
        return {}
    if not lines or lines[0].split("\t") != COLS:
        errors.append(f"{os.path.basename(path)}: not emit_cost's header")
        return {}
    for i, line in enumerate(lines[1:], 2):
        where = f"{os.path.basename(path)}:{i}"
        f = line.split("\t")
        if len(f) != len(COLS):
            errors.append(f"{where}: {len(f)} fields, not {len(COLS)}")
            continue
        try:
            key = (int(f[1]), int(f[2]))
            if f[0] == "blk":
                rec = dict(zip(COLS[5:11], map(int, f[5:11])))
            elif f[0] == "run":
                rec = {"wall": float(f[11]), "bytes": int(f[12])}
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
    shapes = set()
    for key, run in sorted(runs.items()):
        if run["bytes"] != out_bytes:
            errors.append(f"decode {key} wrote {run['bytes']} bytes, not {out_bytes}")
        if not math.isfinite(run["wall"]) or run["wall"] <= 0:
            errors.append(f"decode {key} wall {run['wall']} not finite and positive")
        bs = blks.get(key, [])
        if not any(not b["refused"] for b in bs):
            errors.append(f"decode {key}: no block the open kernel took")
        shapes.add(frozenset(Counter((b["n"], b["m"], b["refused"]) for b in bs).items()))
    if len(shapes) > 1:
        errors.append(f"the blocks' (n, m, refused) differ between decodes ({len(shapes)} shapes)")
    return {key: (runs[key], blks.get(key, [])) for key in runs}


def report(got, workers, rounds, judged):
    lines, cls = [], None
    for w in workers:
        per = [got[(r, w)] for r in range(rounds)]
        bs, cb, cr, walls, slopes, pts = [], [], [], [], [], []
        for run, recs in per:
            open_ = [x for x in recs if not x["refused"]]
            bs.append(sum(x["emit_ns"] for x in open_) / sum(x["m"] for x in open_))
            cb.append(statistics.mean(x["construct_ns"] for x in open_) / 1e3)
            cr.append(sum(x["construct_ns"] for x in open_) / sum(x["n"] for x in open_))
            walls.append(run["wall"])
            p = [(x["m"], x["emit_ns"]) for x in open_]
            slopes.append(ols(p)[1])
            pts += p
        open0 = [x for x in per[0][1] if not x["refused"]]
        refused0 = [x for x in per[0][1] if x["refused"]]
        bidi = sum(x["bidi"] for x in open0)
        a, slope = ols(pts)
        b = interval(bs)
        s = interval(slopes)
        lines.append(f"## {w} workers ({rounds} rounds)")
        lines.append(
            f"open blocks a decode {len(open0)} ({bidi} two-ended), refused by the first pass "
            f"{len(refused0)}; retained rows a decode {sum(x['m'] for x in open0)}, mean m a block "
            f"{statistics.mean(x['m'] for x in open0):.0f}"
        )
        lines.append(f"decode wall, median s: {statistics.median(walls):.3f}")
        lines.append(f"b = sum(emit)/sum(m), ns a row: {b[0]:.3f} [{b[1]:.3f}, {b[2]:.3f}]")
        lines.append(
            f"emit = a + slope*m, pooled: a {a / 1e3:.1f} us, slope {slope:.3f} ns a row; "
            f"per-round slope {s[0]:.3f} [{s[1]:.3f}, {s[2]:.3f}]"
        )
        c = interval(cb)
        r = interval(cr)
        lines.append(
            f"construct, us a block: {c[0]:.1f} [{c[1]:.1f}, {c[2]:.1f}]; ns a block row: "
            f"{r[0]:.3f} [{r[1]:.3f}, {r[2]:.3f}]"
        )
        if w == judged:
            straddles = any(lo <= t <= hi for t in (LOW, HIGH) for lo, hi in [(b[1], b[2])])
            cls = band(b[0]) + (" (STRADDLES)" if straddles else "")
            lines.append(f"class at {w} workers: {cls}; the pooled slope's band: {band(slope)}")
        lines.append("")
    return lines, cls


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--workers", default="1,6,10")
    ap.add_argument("--judged", type=int, default=10)
    ap.add_argument("--rounds", type=int, default=20)
    ap.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_OUT)
    a = ap.parse_args(argv)
    workers = counts(a.workers)
    errors = []
    machine(a.lscpu, errors)
    got = load(a.runs, workers, a.rounds, a.out_bytes, errors)
    if errors:
        print("VOID: no class")
        for e in errors[:40]:
            print(f"- {e}")
        return 1
    lines, cls = report(got, workers, a.rounds, a.judged)
    print("# bench-emit-cost (diagnostic: decides no production default)")
    print()
    print("\n".join(lines))
    print(f"CLASS: {cls}" if cls else "CLASS: none (the judged count was not run)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
