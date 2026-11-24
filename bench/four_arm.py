#!/usr/bin/env python3
"""bench-four-arm's verdict (`just bench-four-arm`): the four-arm whole-decoder oracle, scanner x
emission, read from examples/four_arm.rs's records once they are checked whole. Diagnostic only
(RULES C1a): it decides no production default. The registration, rule and all, is
docs/journal/2026-11-24-four-arm-registration.md, frozen before the data; the brief is
docs/journal/2026-11-24-algorithm-level/FOUR-ARM-BRIEF.md (at 4934a495). Standard library only;
the paired-ratio interval is bench/two_arm.py's.

    four_arm.py RUNS.tsv --lscpu LSCPU.txt [--workers 1,6,10] [--judged 10] [--rounds 20]
                [--out-bytes N]

**The arms** (`examples/four_arm.rs`): A the decoder as shipped; B Open's codes supplied in output
order (bench-supplied-codes' arm B); C the scanner's output (each block's symbols) recorded
outside the clock and delivered through the production job queue; D both.

**Each count:** each round's paired ratios of whole-decoder wall, B/A, C/A and D/A, at the same
worker count; their geometric means and 95% intervals (Student's t on the logs,
`two_arm.step`). Then X, the better single arm (the lower of B's and C's geometric means), and
each round's D/X, with its interval. Reported, deciding nothing: the super-additivity contrast,
each round's (D x A) / (B x C), with its interval (below 1: D gains more than B's and C's gains
multiplied); each arm's peak resident memory (the median of its decodes', the process's whole
high-water mark, both stores included), over A's.

**The rule** (the registration's, the brief's thresholds on the intervals' bounds), at the judged
count (10 workers), on D/A's interval [lo, hi]:
- **INTERACTION** iff hi <= 0.90, D/A's geometric mean is below X/A's by more than 0.03, and D/X's
  interval lies wholly below 1;
- **ADDITIVE** iff hi <= 0.90 otherwise;
- **NO POTENTIAL** iff lo > 0.95;
- **MARGINAL** otherwise.
1 and 6 workers are reported and decide nothing.

**The harness check:** B/A at 10 workers must reproduce bench-supplied-codes' 0.988 [0.957, 1.020]
(journal §30): its interval overlapping that one. If not, the verdict stands, labelled UNVALIDATED.

**VOID** (exit 1, no verdict): the file missing or malformed; the machine not an i9-10900K; the
`meta` line missing or duplicated, or its stores empty, conflicting, colliding or missing a block;
any (round, workers, arm) decode missing, duplicated or unexpected; any decode's output not
OUT_BYTES or the reference's size, or its wall not a finite positive number; a scanning arm (A,
B) with any block replayed, or a replaying arm (C, D) with other than the store's blocks; any
round's check missing, duplicated, with a mismatch, another output (size or CRC-32) than the
reference's, or other than the store's blocks replayed.
"""

import argparse
import math
import os
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import two_arm as ta  # noqa: E402

MODEL = "i9-10900K"
# bench/wikidata.json's decoded bytes (the corpus's 209,700,196), four copies
WIKIDATA_X4_OUT = 4 * 209_700_196
ARMS = ("A", "B", "C", "D")
# the frozen rule's lines (the registration)
ADVANCE_AT = 0.90
CLOSED_ABOVE = 0.95
JOINT_MARGIN = 0.03
# bench-supplied-codes' B/A at 10 workers (journal §30, LOG 7132792060)
S30 = (0.988, 0.957, 1.020)
COLS = (
    "kind round workers arm order wall_s bytes crc peak_kb replayed open_blocks deliver_ns "
    "consume_ns mismatches detail"
).split()


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


def num(v, kind, errors, where):
    try:
        x = kind(v)
    except (TypeError, ValueError):
        errors.append(f"{where}: {v!r} is not a number")
        return None
    if isinstance(x, float) and not math.isfinite(x):
        errors.append(f"{where}: {v!r} is not finite")
        return None
    return x


def load(path, workers, rounds, out_bytes, errors):
    """(meta, runs {(round, workers, arm): row}, checks {round: row}), the records checked"""
    try:
        with open(path) as f:
            lines = f.read().splitlines()
    except OSError as e:
        errors.append(f"{os.path.basename(path)}: unreadable ({e})")
        return None, {}, {}
    if not lines or lines[0].split("\t") != COLS:
        errors.append(f"{os.path.basename(path)}: not the harness's columns")
        return None, {}, {}
    metas, runs, checks = [], {}, {}
    for ln, line in enumerate(lines[1:], 2):
        f = line.split("\t")
        if len(f) != len(COLS):
            errors.append(f"line {ln}: {len(f)} fields, not {len(COLS)}")
            continue
        r = dict(zip(COLS, f))
        where = f"line {ln}"
        if r["kind"] == "meta":
            metas.append(r)
        elif r["kind"] == "run":
            rd = num(r["round"], int, errors, where)
            w = num(r["workers"], int, errors, where)
            key = (rd, w, r["arm"])
            if rd not in range(rounds) or w not in workers or r["arm"] not in ARMS:
                errors.append(f"{where}: the decode {key} is outside the run")
                continue
            if key in runs:
                errors.append(f"{where}: the decode {key} twice")
                continue
            runs[key] = r
        elif r["kind"] == "check":
            rd = num(r["round"], int, errors, where)
            if rd not in range(rounds) or rd in checks:
                errors.append(
                    f"{where}: the check of round {r['round']!r} outside the run or twice"
                )
                continue
            checks[rd] = r
        else:
            errors.append(f"{where}: kind {r['kind']!r}")
    if len(metas) != 1:
        errors.append(f"{len(metas)} meta lines, not 1")
        return None, runs, checks
    meta = metas[0]
    detail = dict(kv.split("=", 1) for kv in meta["detail"].split() if "=" in kv)
    try:
        st = {k: int(v) for k, v in detail.items()}
    except ValueError:
        st = {}
        errors.append(f"meta: the stores' detail {meta['detail']!r} is not numbers")
    for k in ("blocks", "distinct", "symbol_bytes", "code_keys", "codes", "code_bytes"):
        if st.get(k, 0) <= 0:
            errors.append(f"meta: {k} {st.get(k)!r}: the store is empty")
    for k in ("conflicts", "missing"):
        if st.get(k, 1) != 0:
            errors.append(f"meta: {k} {st.get(k)!r}")
    if meta["mismatches"] != "0":
        errors.append(f"meta: {meta['mismatches']!r} collisions, conflicts or missing blocks")
    ref_bytes = num(meta["bytes"], int, errors, "meta")
    if ref_bytes != out_bytes:
        errors.append(f"meta: the reference's output {ref_bytes}, not {out_bytes}")
    meta = {**meta, **st, "ref_bytes": ref_bytes, "ref_crc": meta["crc"]}
    blocks = st.get("blocks")
    for rd in range(rounds):
        for w in workers:
            for arm in ARMS:
                r = runs.get((rd, w, arm))
                where = f"round {rd} workers {w} arm {arm}"
                if r is None:
                    errors.append(f"{where}: missing")
                    continue
                wall = num(r["wall_s"], float, errors, where)
                if wall is not None and not wall > 0:
                    errors.append(f"{where}: wall {wall}")
                if num(r["bytes"], int, errors, where) != out_bytes:
                    errors.append(f"{where}: {r['bytes']} bytes, not {out_bytes}")
                rep = num(r["replayed"], int, errors, where)
                want = blocks if arm in ("C", "D") else 0
                if rep != want:
                    errors.append(f"{where}: {rep} blocks replayed, not {want}")
                if num(r["peak_kb"], int, errors, where) is None:
                    pass
        c = checks.get(rd)
        if c is None:
            errors.append(f"round {rd}: no check")
            continue
        if c["mismatches"] != "0":
            errors.append(f"round {rd}'s check: {c['mismatches']} mismatches")
        if num(c["bytes"], int, errors, f"check {rd}") != out_bytes or c["crc"] != meta["ref_crc"]:
            errors.append(
                f"round {rd}'s check: output {c['bytes']} crc {c['crc']}, not the reference's"
            )
        if num(c["replayed"], int, errors, f"check {rd}") != blocks:
            errors.append(f"round {rd}'s check: {c['replayed']} blocks replayed, not {blocks}")
        if num(c["open_blocks"], int, errors, f"check {rd}") in (None, 0):
            errors.append(f"round {rd}'s check: no supplied block checked")
    return meta, runs, checks


def fmt(t):
    return f"{t[0]:.3f} [{t[1]:.3f}, {t[2]:.3f}]"


def classify(d, x, dx):
    """The rule: d, x and dx are (geometric mean, low, high) of D/A, X/A and D/X"""
    if d[2] <= ADVANCE_AT:
        if d[0] < x[0] - JOINT_MARGIN and dx[2] < 1:
            return "INTERACTION"
        return "ADDITIVE"
    if d[1] > CLOSED_ABOVE:
        return "NO POTENTIAL"
    return "MARGINAL"


def report(meta, runs, workers, rounds, judged):
    print("# bench-four-arm (diagnostic: decides no production default)\n")
    print(
        f"the stores, outside every clock: {meta['blocks']} blocks indexed ({meta['distinct']} "
        f"distinct), {meta['symbol_bytes']} bytes of symbols; {meta['code_keys']} code keys, "
        f"{meta['codes']} codes, {meta['code_bytes']} bytes; the reference output "
        f"{meta['ref_bytes']} bytes, CRC-32 {meta['ref_crc']}\n"
    )
    verdict = None
    for w in workers:
        wall = {a: [float(runs[(r, w, a)]["wall_s"]) for r in range(rounds)] for a in ARMS}
        res = {a: ratio_interval([x / y for x, y in zip(wall[a], wall["A"])]) for a in ARMS[1:]}
        best = min(("B", "C"), key=lambda a: res[a][0])
        dx = ratio_interval([d / x for d, x in zip(wall["D"], wall[best])])
        sup = ratio_interval(
            [d * a / (b * c) for a, b, c, d in zip(wall["A"], wall["B"], wall["C"], wall["D"])]
        )
        peaks = {a: [int(runs[(r, w, a)]["peak_kb"]) for r in range(rounds)] for a in ARMS}
        tag = " (judged)" if w == judged else ""
        print(f"## {w} workers ({rounds} rounds){tag}")
        print(f"A: wall median {statistics.median(wall['A']):.4f} s")
        for a in ARMS[1:]:
            print(
                f"{a}/A: {fmt(res[a])} {ta.label(res[a][1], res[a][2])}; "
                f"wall median {statistics.median(wall[a]):.4f} s"
            )
        print(f"the better single arm X = {best}; D/X: {fmt(dx)} {ta.label(dx[1], dx[2])}")
        print(f"super-additivity (D x A) / (B x C): {fmt(sup)} (decides nothing)")
        if all(p > 0 for v in peaks.values() for p in v):
            pa = statistics.median(peaks["A"])
            print(
                "peak resident memory, the median of each arm's decodes (kB; the process's, both "
                "stores included): "
                + ", ".join(
                    f"{a} {statistics.median(peaks[a]):.0f} ({statistics.median(peaks[a]) / pa:.3f} of A)"
                    for a in ARMS
                )
            )
        else:
            print("peak resident memory: not measured (the kernel refused the reset)")
        v = classify(res["D"], res[best], dx)
        if w == judged:
            verdict = (v, res, best, dx)
            print(f"by the rule: {v}")
        else:
            print(f"by the rule: {v} (decides nothing at this count)")
        print()
    v, res, best, dx = verdict
    b = res["B"]
    reproduced = b[1] <= S30[2] and b[2] >= S30[1]
    print(
        f"the harness check: B/A at {judged} workers {fmt(b)} against bench-supplied-codes' "
        f"{fmt(S30)} (journal §30): "
        + ("reproduced (the intervals overlap)" if reproduced else "NOT REPRODUCED")
    )
    print(
        f"\nDECISION: {v}{'' if reproduced else ' (UNVALIDATED: B did not reproduce §30)'} "
        f"(D/A at {judged} workers {fmt(res['D'])}; X = {best} {fmt(res[best])}; D/X {fmt(dx)}; "
        f"INTERACTION iff D/A's upper bound <= {ADVANCE_AT}, D/A below X/A by more than "
        f"{JOINT_MARGIN} and D/X's interval below 1; ADDITIVE iff the upper bound <= {ADVANCE_AT} "
        f"otherwise; NO POTENTIAL iff the lower bound > {CLOSED_ABOVE}; else MARGINAL)"
    )
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--workers", default="1,6,10")
    ap.add_argument("--judged", type=int, default=10)
    ap.add_argument("--rounds", type=int, default=20)
    ap.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_OUT)
    a = ap.parse_args(argv[1:])
    workers = counts(a.workers)
    errors = []
    machine(a.lscpu, errors)
    if a.judged not in workers:
        errors.append(f"the judged count {a.judged} is not among {workers}")
    meta, runs, _ = load(a.runs, workers, a.rounds, a.out_bytes, errors)
    if errors:
        print(
            f"VOID: the records do not make the registered run ({len(errors)} problems); no verdict"
        )
        for e in errors[:25]:
            print(f"  {e}")
        return 1
    return report(meta, runs, workers, a.rounds, a.judged)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
