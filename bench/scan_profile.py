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
  decode phase's time on the block's symbols and switches (ns a symbol, ns a switch), from the
  decodes that keep every block's record (the first two rounds).

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

**Memory and scheduling,** every decode's (the harness's: peak and time-averaged resident memory,
user and system CPU, minor and major page faults, voluntary and involuntary context switches),
reported as the untimed decodes' medians at each count; every decode's row is in the bundle's
runs.tsv.

**VOID** (exit 1): the file missing or malformed; the machine not an i9-10900K; any (round,
workers, timed) decode missing, duplicated or unexpected; any decode's output not OUT_BYTES, or
its memory and scheduling figures missing; a
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
# every decode's memory and scheduling figures (the harness's)
SCHED = ["base_kb", "peak_kb", "mean_kb", "user_s", "sys_s", "minflt", "majflt", "nvcsw", "nivcsw"]


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
                bad = [
                    f
                    for f in SCHED
                    if not isinstance(x.get(f), (int, float))
                    or isinstance(x.get(f), bool)
                    or not math.isfinite(x[f])
                    or x[f] < 0
                ]
                if bad:
                    errors.append(f"{where}: no memory or scheduling figures for {bad}")
                if not timed:
                    continue
                n, sums = x.get("n"), x.get("sums")
                if (
                    type(n) is not int
                    or n <= 0
                    or not isinstance(sums, list)
                    or len(sums) != len(fields)
                ):
                    errors.append(f"{where}: no block sums, or malformed ones")
                    continue
                if x.get("indices_ok") is not True:
                    errors.append(f"{where}: the blocks are not one each of 0..{n - 1}")
                if n_blocks is None:
                    n_blocks = n
                elif n != n_blocks:
                    errors.append(f"{where}: {n} blocks, not {n_blocks}")
                if len(x.get("workers_rec") or []) != w:
                    errors.append(
                        f"{where}: {len(x.get('workers_rec') or [])} worker records, not {w}"
                    )
                x["sum"] = dict(zip(fields, sums))
                blocks = x.get("blocks")
                if blocks is not None:
                    if len(blocks) != n or any(len(b) != len(fields) for b in blocks):
                        errors.append(f"{where}: its block records not {n} of {len(fields)} fields")
                        continue
                    if sorted(b[0] for b in blocks) != list(range(n)):
                        errors.append(f"{where}: its block records not one each of 0..{n - 1}")
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
    b = x.get("b")
    s = lambda k: x["sum"][k]  # noqa: E731
    wall = x["wall_s"] * 1e9
    read, inw, slot, send = s("read_ns"), s("input_wait_ns"), s("slot_wait_ns"), s("send_ns")
    work = read - inw + send
    phases = {p: s(f"{p}_ns") for p in PHASES}
    phases["remainder"] = read - inw - sum(phases.values())
    phases["hand-off"] = send
    wr = x["workers_rec"]
    idle = sum(r[1] for r in wr)
    busy_per_worker = sum(r[2] - r[1] for r in wr) / w
    fit = ols([(r["decode_ns"], r["syms"], r["switches"]) for r in b]) if b else None
    return {
        "wall": wall,
        "work": work,
        "slot": slot,
        "input": inw,
        "phases": phases,
        "bits_per_s": s("bits") / (work / 1e9),
        "syms_per_s": s("syms") / (work / 1e9),
        "idle_workers": idle / wall,
        "idle_per_worker": idle / w,
        "busy_per_worker": busy_per_worker,
        "n_blocks": x["n"],
        "switches_per_block": s("switches") / x["n"],
        "selectors_per_block": s("selectors") / x["n"],
        "decode_ns_per_sym": phases["decode"] / s("syms"),
        "fit": fit,
        "read_ns": sorted(r["read_ns"] - r["input_wait_ns"] for r in b) if b else None,
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


def half(xs):
    """The 95% t-interval's half-width of the mean of `xs`, relative to the mean"""
    n = len(xs)
    if n < 2:
        return math.nan
    t95 = ta.T95[n - 2] if n - 1 <= len(ta.T95) else 1.96
    return t95 * statistics.stdev(xs) / math.sqrt(n) / statistics.mean(xs)


def fit_pipeline(points):
    """The pipeline model wall(k) = c + max(b, a / k), a serial stage (b, the whole decode's
    serial work) feeding k workers (a, their whole work; a / k each), fitted by least squares
    to (k, wall) points: for each shift point k* on a grid of hundredths from the smallest k to
    the largest, b = a / k*, the model linear in (c, a), solved in closed form; the k* of least
    squared error. (k*, a, b, c, residuals), or None"""
    ks = [k for k, _ in points]
    best = None
    lo, hi = min(ks), max(ks)
    steps = int(round((hi - lo) * 100))
    for i in range(steps + 1):
        kstar = lo + i / 100
        g = [1 / k if k <= kstar else 1 / kstar for k, _ in points]
        y = [v for _, v in points]
        n = len(points)
        mg, my = sum(g) / n, sum(y) / n
        sgg = sum((x - mg) ** 2 for x in g)
        if sgg <= 0:
            continue
        a = sum((x - mg) * (v - my) for x, v in zip(g, y)) / sgg
        c = my - a * mg
        sse = sum((c + a * x - v) ** 2 for x, v in zip(g, y))
        if best is None or sse < best[0]:
            best = (sse, kstar, a, c)
    if best is None:
        return None
    _, kstar, a, c = best
    res = [(k, v - (c + a * (1 / k if k <= kstar else 1 / kstar))) for k, v in points]
    return kstar, a, a / kstar, c, res


def crossover(rows):
    """The measured crossover: the count where rho (the scanner's rate over the workers'
    combined rate) first falls from at least 1 to below it, interpolated linearly in k between
    the two counts; None if it never does in the grid"""
    for (k0, r0), (k1, r1) in zip(rows, rows[1:]):
        if r0 >= 1 > r1:
            return k0 + (r0 - 1) / (r0 - r1) * (k1 - k0)
    return None


def report(got, workers, rounds, judged):
    print("# bench-scan-profile (diagnostic: decides nothing)\n")
    print(
        "Unpinned throughout: production's lazy workers at exactly k (`BzReader::parallel_workers`), "
        "the scanner on its own thread beside them.\n"
    )
    verdict, grid = None, []
    for w in workers:
        ms = [summarise(got[(r, w, True)], w) for r in range(rounds)]
        untimed = [got[(r, w, False)]["wall_s"] for r in range(rounds)]
        over = ta.step(
            [
                math.log(got[(r, w, True)]["wall_s"] / got[(r, w, False)]["wall_s"])
                for r in range(rounds)
            ]
        )
        m = {k: med(ms, k) for k in ("wall", "work", "slot", "input", "bits_per_s", "syms_per_s",
                                       "idle_workers", "idle_per_worker", "busy_per_worker",
                                       "switches_per_block", "selectors_per_block",
                                       "decode_ns_per_sym", "n_blocks")}  # fmt: skip
        m["phases"] = {p: statistics.median(x["phases"][p] for x in ms) for p in ms[0]["phases"]}
        # rho: the scanner's rate (blocks a second of its work) over the workers' combined rate
        # (k blocks a worker's busy second): the workers' busy time a worker over the scanner's work
        rho = m["busy_per_worker"] / m["work"]
        c, comp, dom = classify(m)
        label = f"{c}; the work's largest component: {comp}" + ("; DECODE-DOMINATED" if dom else "")
        grid.append({"k": w, "m": m, "rho": rho, "label": c, "untimed": statistics.median(untimed),
                     "over": over, "hw_work": half([x["work"] for x in ms]),
                     "hw_busy": half([x["busy_per_worker"] for x in ms])})  # fmt: skip
        tag = " (judged)" if w == judged else ""
        print(f"## {w} workers ({rounds} timed decodes, medians){tag}")
        print(
            f"wall {m['wall'] / 1e9:.4f} s timed, {statistics.median(untimed):.4f} s untimed (timed "
            f"over untimed: {over[0]:.4f} [{over[1]:.4f}, {over[2]:.4f}], the timers' overhead)"
        )
        print(
            f"the scanner: busy (work) {m['work'] / 1e9:.4f} s ({m['work'] / m['wall']:.3f} of the "
            f"wall, the mean's 95% half-width {grid[-1]['hw_work'] * 100:.1f}%); idle: slot waits "
            f"(the window full) {m['slot'] / 1e9:.4f} s ({m['slot'] / m['wall']:.3f}), input waits "
            f"{m['input'] / 1e9:.4f} s ({m['input'] / m['wall']:.3f})"
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
        fits = [x["fit"] for x in ms if x["fit"] is not None]
        print(
            f"table switching: {m['switches_per_block']:.0f} switches in {m['selectors_per_block']:.0f} "
            "selectors a block"
            + (
                ""
                if not fits
                else f"; the decode phase fit across blocks ({len(fits)} decodes with every block's record): "
                f"{statistics.median(f[1] for f in fits):.2f} ns a symbol, "
                f"{statistics.median(f[2] for f in fits):.1f} ns a switch, "
                f"{statistics.median(f[0] for f in fits) / 1e3:.1f} us a block"
            )
        )
        rds = [x["read_ns"] for x in ms if x["read_ns"]]
        if rds:
            rd = rds[0]
            q = lambda p: rd[min(len(rd) - 1, int(p * len(rd)))] / 1e3  # noqa: E731
            print(
                f"a block's read, less its input waits (the first decode with every block's record): "
                f"p10 {q(0.1):.0f} us, median {q(0.5):.0f} us, p90 {q(0.9):.0f} us, max {rd[-1] / 1e3:.0f} us"
            )
        print(
            f"the workers: busy {m['busy_per_worker'] / 1e9:.4f} s a worker (the mean's 95% "
            f"half-width {grid[-1]['hw_busy'] * 100:.1f}%), idle {m['idle_per_worker'] / 1e9:.4f} s a "
            f"worker; mean idle workers {m['idle_workers']:.2f} of {w}"
        )
        print(
            f"rho, the scanner's rate over the workers' combined rate: {rho:.3f} "
            f"({'the workers bind' if rho >= 1 else 'the scanner binds'})"
        )
        if w == judged:
            verdict = (label, over[1] > PERTURB, m)
            print(f"by the classification: {label}")
        else:
            print(f"by the classification: {label} (decides nothing at this count)")
        print()
    print("## The grid")
    print(
        "k  wall(untimed) s  scanner busy/idle (of wall)  worker busy/idle s a worker  rho    class"
    )
    for g in grid:
        m = g["m"]
        print(
            f"{g['k']:<3}{g['untimed']:>10.4f}       {m['work'] / m['wall']:.3f} / {(m['slot'] + m['input']) / m['wall']:.3f}"
            f"                {m['busy_per_worker'] / 1e9:.4f} / {m['idle_per_worker'] / 1e9:.4f}            "
            f"{g['rho']:.3f}  {g['label']}"
        )
    print(
        "\n## Memory and scheduling (the untimed decodes' medians; every decode's own figures are "
        "in the bundle's runs.tsv)\n"
        "The resident memory is the decode's own: the process's, less its base at the decode's "
        "start (the input and the runtime; each decode runs in a fresh process)."
    )
    print(
        "k  rss mean MB  rss peak MB  user s  sys s  cpu/wall  minor faults  major  vol. ctxsw  invol. ctxsw"
    )
    for w in workers:
        xs = [got[(r, w, False)] for r in range(rounds)]
        md = lambda f: statistics.median(x[f] for x in xs)  # noqa: E731
        own = lambda f: statistics.median(x[f] - x["base_kb"] for x in xs) / 1024  # noqa: E731
        cpu = statistics.median((x["user_s"] + x["sys_s"]) / x["wall_s"] for x in xs)
        print(
            f"{w:<3}{own('mean_kb'):>10.1f}{own('peak_kb'):>13.1f}{md('user_s'):>8.3f}"
            f"{md('sys_s'):>7.3f}{cpu:>10.2f}{md('minflt'):>14.0f}{md('majflt'):>7.0f}"
            f"{md('nvcsw'):>12.0f}{md('nivcsw'):>14.0f}"
        )
    bases = [x["base_kb"] / 1024 for x in got.values()]
    print(
        f"the base, every decode: {min(bases):.1f} to {max(bases):.1f} MB "
        f"(median {statistics.median(bases):.1f})"
    )
    cross = crossover([(g["k"], g["rho"]) for g in grid])
    print(
        "\nthe measured crossover (rho falls through 1, interpolated in k): "
        + (f"k = {cross:.2f}" if cross is not None else "none in the grid")
    )
    fit = fit_pipeline([(g["k"], g["untimed"]) for g in grid])
    lows = [g for g in grid if g["k"] <= 4]
    if lows:
        s_blk = statistics.median(g["m"]["work"] / g["m"]["n_blocks"] for g in lows)
        w_blk = statistics.median(
            g["m"]["busy_per_worker"] * g["k"] / g["m"]["n_blocks"] for g in lows
        )
        print(
            f"the model's service times, from the counts up to 4 (uncontended): the serial stage "
            f"{s_blk / 1e3:.1f} us a block, a worker {w_blk / 1e3:.1f} us a block; its predicted "
            f"shift point k = w / s = {w_blk / s_blk:.2f}"
        )
    if fit is not None:
        kstar, a, b, c, res = fit
        print(
            f"the pipeline model fitted to the untimed walls, wall(k) = c + max(b, a / k): "
            f"k* = {kstar:.2f}, a = {a:.4f} s, b = {b:.4f} s, c = {c:.4f} s; residuals (s): "
            + ", ".join(f"{k} {r:+.4f}" for k, r in res)
            + f"; RMS {math.sqrt(sum(r * r for _, r in res) / len(res)):.4f} s"
        )
    label, perturbed, m = verdict
    print(
        f"\nCLASSIFICATION at {judged} workers: {label}"
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
    ap.add_argument("--workers", default="1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20")
    ap.add_argument("--judged", type=int, default=10)
    ap.add_argument("--rounds", type=int, default=20)
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
