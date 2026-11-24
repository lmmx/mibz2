#!/usr/bin/env python3
"""bench-scan-profile's report (`just bench-scan-profile`): the parallel decoder's serial scanner,
work against waiting, by component, at 1, 6 and 10 workers, read from examples/scan_profile.rs's
records once they are checked whole. Diagnostic only (RULES C1a): it decides nothing; its
classification is registered before the data in docs/journal/2026-11-24-scan-profile-registration.md.
Standard library only; the paired-ratio interval is bench/two_arm.py's.

    scan_profile.py RUNS.jsonl --lscpu LSCPU.txt [--workers 1,6,10] [--judged 10] [--rounds 12]
                    [--out-bytes N]

**Per timed decode** (the timers on), from its blocks' and workers' records:
- *the scanner's work:* each block's read less its input waits, plus its hand-off; *its waits:*
  the slot waits (the window full: the scanner waiting on the workers) and the input waits;
- *the work's components:* the read's phases (header, bitmap, selectors, tables, buffer
  preparation, Huffman decode), the read's remainder (stream headers and ends, the window), and
  the hand-off;
- *its rates:* compressed bits and symbols a second of work;
- *the workers:* their idle time (blocked on an empty queue) as mean idle workers over the wall,
  and their busy time a worker (life less idle, over the count): the decode time the workers
  alone would need;
- *table switching:* switches a block, and an ordinary least-squares fit across blocks of the
  decode phase's time on the block's symbols and switches (ns a symbol, ns a switch).

Each figure is the median over the timed decodes at a count. **The overhead:** each round's
paired wall ratio, timed over untimed, at each count (geometric mean, 95% interval).

**The classification at the judged count (10 workers), on the medians' shares of the decode's
wall** (registered before the data; it decides nothing):
- **SCANNER-BOUND** iff the scanner's work is at least 0.85 of the wall and its slot waits at most
  0.05;
- **QUEUE-BOUND** iff its slot waits exceed 0.10 (the scanner waits on the workers);
- **INPUT-BOUND** iff its input waits exceed 0.10;
- **MIXED** otherwise.
The work's largest component is named, and **DECODE-DOMINATED** added iff the Huffman decode is at
least 0.60 of the work. If the overhead's interval at the judged count lies wholly above 1.03, the
classification is labelled PERTURBED.

**VOID** (exit 1): the file missing or malformed; the machine not an i9-10900K; any (round,
workers, timed) decode missing, duplicated or unexpected; any decode's output not OUT_BYTES; a
timed decode without a block's or a worker's record, its blocks not one each of 0..N-1 (N the
same in every timed decode), or its worker records not one a worker.
"""

import argparse
import json
import math
import os
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import two_arm as ta  # noqa: E402

MODEL = "i9-10900K"
WIKIDATA_X4_OUT = 4 * 209_700_196
WORK_BOUND, SLOT_SMALL, WAIT_BIG, DECODE_DOMINANT, PERTURB = 0.85, 0.05, 0.10, 0.60, 1.03
PHASES = ["header", "bitmap", "selectors", "tables", "prep", "decode"]


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


def load(path, workers, rounds, out_bytes, errors):
    try:
        with open(path) as f:
            lines = [json.loads(x) for x in f if x.strip()]
    except (OSError, ValueError) as e:
        errors.append(f"{os.path.basename(path)}: unreadable ({e})")
        return {}, []
    metas = [x for x in lines if x.get("kind") == "meta"]
    if len(metas) != 1:
        errors.append(f"{len(metas)} meta lines, not 1")
        return {}, []
    fields = metas[0].get("block_fields")
    got = {}
    for x in lines:
        if x.get("kind") != "decode":
            continue
        key = (x.get("round"), x.get("workers"), x.get("timed"))
        if key[0] not in range(rounds) or key[1] not in workers or type(key[2]) is not bool:
            errors.append(f"the decode {key} is outside the run")
            continue
        if key in got:
            errors.append(f"the decode {key} twice")
            continue
        got[key] = x
    n_blocks = None
    for r in range(rounds):
        for w in workers:
            for timed in (True, False):
                x = got.get((r, w, timed))
                where = f"round {r} workers {w} {'timed' if timed else 'untimed'}"
                if x is None:
                    errors.append(f"{where}: missing")
                    continue
                wall = x.get("wall_s")
                if not isinstance(wall, (int, float)) or not (wall > 0 and math.isfinite(wall)):
                    errors.append(f"{where}: wall {wall!r}")
                if x.get("bytes") != out_bytes:
                    errors.append(f"{where}: {x.get('bytes')} bytes, not {out_bytes}")
                if not timed:
                    continue
                blocks = x.get("blocks") or []
                if not blocks or any(len(b) != len(fields) for b in blocks):
                    errors.append(f"{where}: no block records, or malformed ones")
                    continue
                idx = sorted(b[0] for b in blocks)
                if idx != list(range(len(idx))):
                    errors.append(f"{where}: the blocks are not one each of 0..{len(idx) - 1}")
                if n_blocks is None:
                    n_blocks = len(idx)
                elif len(idx) != n_blocks:
                    errors.append(f"{where}: {len(idx)} blocks, not {n_blocks}")
                if len(x.get("workers_rec") or []) != w:
                    errors.append(
                        f"{where}: {len(x.get('workers_rec') or [])} worker records, not {w}"
                    )
                x["b"] = [dict(zip(fields, b)) for b in blocks]
    return got, fields


def ols(rows):
    """Least squares of y on (1, a, b): (intercept, slope a, slope b), or None"""
    n = len(rows)
    if n < 4:
        return None
    sa = [[0.0] * 3 for _ in range(3)]
    sy = [0.0] * 3
    for y, a, b in rows:
        v = (1.0, a, b)
        for i in range(3):
            sy[i] += v[i] * y
            for j in range(3):
                sa[i][j] += v[i] * v[j]
    m = [row[:] + [sy[i]] for i, row in enumerate(sa)]
    for c in range(3):
        p = max(range(c, 3), key=lambda r: abs(m[r][c]))
        if abs(m[p][c]) < 1e-12:
            return None
        m[c], m[p] = m[p], m[c]
        for r in range(3):
            if r != c:
                f = m[r][c] / m[c][c]
                m[r] = [x - f * y for x, y in zip(m[r], m[c])]
    return tuple(m[i][3] / m[i][i] for i in range(3))


def summarise(x, w):
    b = x["b"]
    s = lambda k: sum(r[k] for r in b)  # noqa: E731
    wall = x["wall_s"] * 1e9
    read, inw, slot, send = s("read_ns"), s("input_wait_ns"), s("slot_wait_ns"), s("send_ns")
    work = read - inw + send
    phases = {p: s(f"{p}_ns") for p in PHASES}
    phases["remainder"] = read - inw - sum(phases.values())
    phases["hand-off"] = send
    wr = x["workers_rec"]
    idle = sum(r[1] for r in wr)
    busy_per_worker = sum(r[2] - r[1] for r in wr) / w
    fit = ols([(r["decode_ns"], r["syms"], r["switches"]) for r in b])
    return {
        "wall": wall,
        "work": work,
        "slot": slot,
        "input": inw,
        "phases": phases,
        "bits_per_s": s("bits") / (work / 1e9),
        "syms_per_s": s("syms") / (work / 1e9),
        "idle_workers": idle / wall,
        "busy_per_worker": busy_per_worker,
        "switches_per_block": s("switches") / len(b),
        "selectors_per_block": s("selectors") / len(b),
        "decode_ns_per_sym": phases["decode"] / s("syms"),
        "fit": fit,
        "read_ns": sorted(r["read_ns"] - r["input_wait_ns"] for r in b),
    }


def classify(m):
    work, slot, inw = m["work"] / m["wall"], m["slot"] / m["wall"], m["input"] / m["wall"]
    if work >= WORK_BOUND and slot <= SLOT_SMALL:
        c = "SCANNER-BOUND"
    elif slot > WAIT_BIG:
        c = "QUEUE-BOUND"
    elif inw > WAIT_BIG:
        c = "INPUT-BOUND"
    else:
        c = "MIXED"
    comp = max(m["phases"], key=lambda k: m["phases"][k])
    dom = m["phases"]["decode"] / m["work"] >= DECODE_DOMINANT
    return c, comp, dom


def med(ms, key):
    return statistics.median(m[key] for m in ms)


def report(got, workers, rounds, judged):
    print("# bench-scan-profile (diagnostic: decides nothing)\n")
    verdict = None
    for w in workers:
        ms = [summarise(got[(r, w, True)], w) for r in range(rounds)]
        over = ta.step(
            [
                math.log(got[(r, w, True)]["wall_s"] / got[(r, w, False)]["wall_s"])
                for r in range(rounds)
            ]
        )
        m = {
            k: med(ms, k)
            for k in (
                "wall",
                "work",
                "slot",
                "input",
                "bits_per_s",
                "syms_per_s",
                "idle_workers",
                "busy_per_worker",
                "switches_per_block",
                "selectors_per_block",
                "decode_ns_per_sym",
            )
        }
        m["phases"] = {p: statistics.median(x["phases"][p] for x in ms) for p in ms[0]["phases"]}
        tag = " (judged)" if w == judged else ""
        print(f"## {w} workers ({rounds} timed decodes, medians){tag}")
        print(
            f"wall {m['wall'] / 1e9:.4f} s (timed over untimed: {over[0]:.4f} [{over[1]:.4f}, "
            f"{over[2]:.4f}], the timers' overhead)"
        )
        print(
            f"the scanner: work {m['work'] / 1e9:.4f} s ({m['work'] / m['wall']:.3f} of the wall); "
            f"waits: slot (the window full) {m['slot'] / 1e9:.4f} s ({m['slot'] / m['wall']:.3f}), "
            f"input {m['input'] / 1e9:.4f} s ({m['input'] / m['wall']:.3f})"
        )
        tot = sum(m["phases"].values())
        print(
            "the work by component: "
            + ", ".join(f"{k} {v / 1e9:.4f} s ({v / tot:.3f})" for k, v in m["phases"].items())
        )
        print(
            f"rates: {m['bits_per_s'] / 1e6:.1f} Mbit and {m['syms_per_s'] / 1e6:.1f} M symbols a "
            f"second of work; decode {m['decode_ns_per_sym']:.2f} ns a symbol"
        )
        print(
            f"table switching: {m['switches_per_block']:.0f} switches in {m['selectors_per_block']:.0f} "
            "selectors a block"
            + (
                ""
                if any(x["fit"] is None for x in ms)
                else "; the decode phase fit across blocks: "
                f"{statistics.median(x['fit'][1] for x in ms):.2f} ns a symbol, "
                f"{statistics.median(x['fit'][2] for x in ms):.1f} ns a switch, "
                f"{statistics.median(x['fit'][0] for x in ms) / 1e3:.1f} us a block"
            )
        )
        rd = ms[len(ms) // 2]["read_ns"]
        q = lambda p: rd[min(len(rd) - 1, int(p * len(rd)))] / 1e3  # noqa: E731
        print(
            f"a block's read, less its input waits: p10 {q(0.1):.0f} us, median {q(0.5):.0f} us, p90 {q(0.9):.0f} us, max {rd[-1] / 1e3:.0f} us"
        )
        print(
            f"the workers: mean idle {m['idle_workers']:.2f} of {w}; busy {m['busy_per_worker'] / 1e9:.4f} s "
            f"a worker (the workers alone; the scanner's work is {m['work'] / max(m['busy_per_worker'], 1):.2f} of it)"
        )
        c, comp, dom = classify(m)
        label = f"{c}; the work's largest component: {comp}" + ("; DECODE-DOMINATED" if dom else "")
        if w == judged:
            perturbed = over[1] > PERTURB
            verdict = (label, perturbed, m)
            print(f"by the classification: {label}")
        else:
            print(f"by the classification: {label} (decides nothing at this count)")
        print()
    label, perturbed, m = verdict
    print(
        f"CLASSIFICATION at {judged} workers: {label}"
        + (" (PERTURBED: the timers' overhead interval lies above 1.03)" if perturbed else "")
        + f". What a cheaper or parallel scanner must beat: the serial scanner's work, "
        f"{m['work'] / 1e9:.4f} s a decode of the wall's {m['wall'] / 1e9:.4f} s, against the "
        f"workers' {m['busy_per_worker'] / 1e9:.4f} s each."
    )
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--workers", default="1,6,10")
    ap.add_argument("--judged", type=int, default=10)
    ap.add_argument("--rounds", type=int, default=12)
    ap.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_OUT)
    a = ap.parse_args(argv[1:])
    workers = counts(a.workers)
    errors = []
    machine(a.lscpu, errors)
    if a.judged not in workers:
        errors.append(f"the judged count {a.judged} is not among {workers}")
    got, _ = load(a.runs, workers, a.rounds, a.out_bytes, errors)
    if errors:
        print(
            f"VOID: the records do not make the registered run ({len(errors)} problems); no classification"
        )
        for e in errors[:25]:
            print(f"  {e}")
        return 1
    return report(got, workers, a.rounds, a.judged)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
