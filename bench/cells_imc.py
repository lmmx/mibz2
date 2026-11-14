"""The working-set premise test (#145, LOG 6880984644, registered before any data): does
Cells' DRAM traffic per output byte grow with the number of Cells working sets live at once?

`just bench-cells-imc` runs it on the i9 (C5): the dose run's P0 build of `examples/ab.rs`
(`--features dose`, MIBZ2_DOSE_MB=0, stream), forced Workers + Cells (`ab child
mibz2-workers <t>`), on the `wikidata.bz2` that bench-cells-dose builds. Three steps:

    python3 bench/cells_imc.py probe
    python3 bench/cells_imc.py collect --ab AB --corpus WIKIDATA [--threads 1,2,3,4,6,8,10]
        [--reps 10] [--loops auto|N] [--pair 3] --out READINGS.jsonl
    python3 bench/cells_imc.py analyse READINGS.jsonl [--threads ...] [--reps 10] [--out STEM]

**probe** fails clearly (exit 2) unless `sudo perf stat -a` counts the memory controller's
`uncore_imc/data_reads` and `data_writes`, and `perf` counts `mem_load_retired.l3_miss` for a
process: the run needs both, and a VM or a laptop without them gets no readings.

**collect** reuses bench/imc.py's paired-idle curve (its `perf`, `rates` and `span`): for each
repetition, the thread counts in a rotated order, an idle reading of --pair seconds just
before each decode reading. A decode reading is --loops decodes of `ab child mibz2-workers
<t> WIKIDATA` in one process chain under `sudo perf stat -a` (all 196 admitted blocks every
decode, so the work is fixed); `--loops auto` sets one count for every thread count, from a
timed decode at each before the readings, so the fastest reading spans at least 2.5 s. Then,
reported only, a second pass of the same roster counts `mem_load_retired.l3_miss`,
process-scoped (the chain and every decode thread inherited). One JSON line a reading.

**analyse** reads them:

    b(t) = (data_reads + data_writes, bytes, less the paired idle rate x the reading's span)
           / (output bytes x loops)

**Registered statistic.** R = b(10)/b(3), its 95% interval from the per-repetition log ratios
log b_r(10) - log b_r(3), Student's t over the repetitions.

**Verdict:**
- PREMISE HOLDS: R >= 1.25 and the interval's lower bound above 1;
- PREMISE FAILS: R <= 1.10 and the interval's upper bound at most 1.15;
- INCONCLUSIVE otherwise.

**Validity** (any failure: INVALID, no verdict):
- a complete roster: each t x repetition exactly once, with its idle pair;
- every decode successful (each reading's perf and chain, each decode's record without an
  error, --loops decodes a reading), with one output hash (and size) across all readings;
- `dose_mb` = 0 and `dose_blocks` = 196 in every decode;
- every idle-subtracted traffic figure positive (net reads, net writes, and so their sum);
- the idle rate in each pair below 10% of its decode reading's rate (reads + writes per
  second); above it the reading is invalid, not dropped.

**Reported, never gating:** the whole b(t) curve (reads and writes apart too), b(10)/b(2)
and b(10)/b(4) paired the same way; b(1) against the census's 25 B per output byte of heap
traffic; b(10) against the dose LOG's estimate of 24 B; process-scoped
`mem_load_retired.l3_miss` per output byte; wall time (each decode's own, and each reading's
span, with any reading under 2 s listed).

A grid other than the registered one (threads 1,2,3,4,6,8,10, 10 repetitions) still gets its
statistic and rule, on b(10)/b(3) where both are in the grid (else the most threads over the
fewest), its verdict line marked as not the registered grid (a dry run).
"""

import argparse
import json
import math
import os
import shutil
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import imc  # noqa: E402
from dose import mean_ci  # noqa: E402

IMC = imc.IMC
L3 = "mem_load_retired.l3_miss"
ARM = "mibz2-workers"
REGISTERED_THREADS = [1, 2, 3, 4, 6, 8, 10]
REGISTERED_REPS = 10
HI, LO = 10, 3
REPORTED_PAIRS = [(10, 2), (10, 4)]
BLOCKS = 196
DOSE_MB = 0
HOLDS_AT = 1.25
FAILS_AT = 1.10
FAILS_UPPER = 1.15
IDLE_SHARE = 0.10
CENSUS_HEAP_B = 25.0
DOSE_LOG_B10 = 24.0
MIN_SPAN = 2.0
AUTO_SPAN = 2.5


def order(threads, rep):
    """The thread counts' order in repetition `rep`: rotated by one each repetition (imc.py's)"""
    k = rep % len(threads)
    return threads[k:] + threads[:k]


def chain(ab, corpus, t, loops):
    """--loops decodes at t threads in one chain of processes"""
    child = [os.path.abspath(ab), "child", ARM, str(t), os.path.abspath(corpus)]
    return ["sh", "-c", 'for i in $(seq "$0"); do "$@" || exit 1; done', str(loops), *child]


def records(stdout):
    """The chain's decodes: one JSON record a decode, as `ab child` prints them"""
    out = []
    for ln in stdout.splitlines():
        if ln.startswith("{"):
            try:
                out.append(json.loads(ln))
            except json.JSONDecodeError:
                out.append({"err": f"unparsed: {ln[:200]}"})
    return out


def counted(events, cmd, system):
    """imc.perf, a failure (perf's, the chain's, or an event not counted) kept as an error"""
    try:
        wall, stdout, rows = imc.perf(events, cmd, system=system)
    except SystemExit as e:
        return None, str(e)
    return (wall, stdout, rows), None


# ---------------------------------------------------------------------------------- probe


def probe():
    """Exit 2, saying why, unless the counters the run needs count here"""
    if not shutil.which("perf"):
        sys.exit("bench-cells-imc: no perf on PATH (linux-tools); this run needs it")
    if subprocess.run(["sudo", "-v"]).returncode:
        sys.exit("bench-cells-imc: sudo refused; perf stat -a needs it for the memory controller")
    for events, system, what in [
        (IMC, True, "the memory controller's uncore_imc/data_reads and data_writes (perf stat -a)"),
        ([L3], False, f"the core event {L3} (process-scoped)"),
    ]:
        got, err = counted(events, ["sleep", "0.2"], system)
        if err:
            print(f"bench-cells-imc: {what} cannot be counted here:\n{err}", file=sys.stderr)
            sys.exit(2)
    print("bench-cells-imc: the counters count (uncore_imc data_reads/data_writes, l3_miss)")


# -------------------------------------------------------------------------------- collect


def calibrate(ab, corpus, threads):
    """One timed decode at each thread count, outside perf: its wall"""
    walls = {}
    for t in threads:
        t0 = time.monotonic()
        r = subprocess.run(chain(ab, corpus, t, 1), capture_output=True, text=True)
        walls[t] = time.monotonic() - t0
        if r.returncode:
            sys.exit(f"bench-cells-imc: the decode at {t} threads failed:\n{r.stderr}")
    return walls


def collect(a):
    threads = [int(t) for t in a.threads.split(",")]
    walls = calibrate(a.ab, a.corpus, threads)
    loops = (
        max(1, math.ceil(AUTO_SPAN / min(walls.values()))) if a.loops == "auto" else int(a.loops)
    )
    meta = {
        "pass": "meta",
        "threads": threads,
        "reps": a.reps,
        "loops": loops,
        "loops_arg": a.loops,
        "pair_seconds": a.pair,
        "calibration_wall": {str(t): w for t, w in walls.items()},
        "ab": os.path.abspath(a.ab),
        "corpus": os.path.abspath(a.corpus),
        "perf": imc.run(["perf", "--version"]).stdout.strip(),
    }
    print(f"bench-cells-imc: {loops} decodes a reading; one decode's wall {walls}", flush=True)
    with open(a.out, "w") as f:

        def emit(rec):
            f.write(json.dumps(rec) + "\n")
            f.flush()

        emit(meta)
        if a.settle:
            time.sleep(a.settle)
        # (the registered readings: each decode reading paired with an idle reading before it)
        for rep in range(a.reps):
            imc.run(["sudo", "-v"])
            for slot, t in enumerate(order(threads, rep)):
                rec = {"pass": "imc", "rep": rep, "threads": t, "slot": slot, "loops": loops}
                got, err = counted(IMC, ["sleep", str(a.pair)], system=True)
                if err:
                    rec["idle"], rec["idle_error"] = None, err
                else:
                    iw, _, irows = got
                    ir, iwr = imc.rates(iw, irows)
                    rec["idle"] = {"span": imc.span(iw, irows), "reads": ir, "writes": iwr}
                rec["utc_start"] = time.time()
                got, err = counted(IMC, chain(a.ab, a.corpus, t, loops), system=True)
                if err:
                    rec.update(error=err, decodes=[])
                else:
                    wall, stdout, rows = got
                    rec.update(
                        error=None,
                        wall=wall,
                        span=imc.span(wall, rows),
                        reads=rows[None][IMC[0]],
                        writes=rows[None][IMC[1]],
                        decodes=records(stdout),
                    )
                emit(rec)
                print(
                    f"bench-cells-imc: rep {rep} t={t} {'ok' if not err else 'FAILED'}", flush=True
                )
        # (reported only: the same roster, process-scoped L3 misses)
        for rep in range(a.reps):
            imc.run(["sudo", "-v"])
            for slot, t in enumerate(order(threads, rep)):
                rec = {"pass": "l3", "rep": rep, "threads": t, "slot": slot, "loops": loops}
                got, err = counted([L3], chain(a.ab, a.corpus, t, loops), system=False)
                if err:
                    rec.update(error=err, decodes=[])
                else:
                    wall, stdout, rows = got
                    rec.update(
                        error=None,
                        wall=wall,
                        span=imc.span(wall, rows),
                        l3_miss=rows[None][L3],
                        decodes=records(stdout),
                    )
                emit(rec)
    return 0


# -------------------------------------------------------------------------------- analyse


def verdict(ratio, lo, hi):
    """The LOG's rule on R = b(10)/b(3) and its interval"""
    if any(math.isnan(x) for x in (ratio, lo, hi)):
        return "INCONCLUSIVE"
    if ratio >= HOLDS_AT and lo > 1.0:
        return "PREMISE HOLDS"
    if ratio <= FAILS_AT and hi <= FAILS_UPPER:
        return "PREMISE FAILS"
    return "INCONCLUSIVE"


def load(path):
    recs, bad = [], []
    with open(path) as f:
        for i, line in enumerate(f, 1):
            if not line.strip():
                continue
            try:
                recs.append(json.loads(line))
            except json.JSONDecodeError:
                bad.append(i)
    return recs, bad


def num(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x)


def out_bytes(rec):
    return sum(d.get("out", 0) for d in rec.get("decodes", []) if num(d.get("out")))


def net(rec):
    """(net reads, net writes): the reading's bytes less its idle pair's rate x its span"""
    i, s = rec["idle"], rec["span"]
    return rec["reads"] - i["reads"] * s, rec["writes"] - i["writes"] * s


def check_decodes(rec, where, problems, outputs):
    loops = rec.get("loops")
    ds = rec.get("decodes") or []
    if len(ds) != loops:
        problems.append(f"{len(ds)} decodes, not {loops}: {where}")
    for k, d in enumerate(ds):
        if d.get("err"):
            problems.append(f"decode {k} failed ({d.get('err')!r}): {where}")
        if d.get("dose_mb") != DOSE_MB:
            problems.append(f"decode {k}'s dose_mb is {d.get('dose_mb')!r}, not 0: {where}")
        if d.get("dose_blocks") != BLOCKS:
            problems.append(
                f"decode {k}'s dose_blocks is {d.get('dose_blocks')!r}, not {BLOCKS}: {where}"
            )
        outputs.add((d.get("hash"), d.get("out")))


def validate(recs, bad, threads, reps):
    """The problems that leave no verdict (empty: valid), the decode readings by (t, rep), and
    the reported-only L3 readings by (t, rep) with their own problems"""
    problems = [f"line {i} is not JSON" for i in bad]
    cells, l3cells, l3problems = {}, {}, []
    outputs = set()
    for r in recs:
        p = r.get("pass")
        if p == "meta":
            continue
        key = (r.get("threads"), r.get("rep"))
        where = f"{p} t={key[0]} rep {key[1]}"
        if p not in ("imc", "l3") or key[0] not in threads or key[1] not in range(reps):
            problems.append(f"an unexpected reading: {where}")
            continue
        if p == "l3":
            if key in l3cells:
                l3problems.append(f"duplicated: {where}")
            l3cells[key] = r
            if r.get("error"):
                l3problems.append(f"not counted: {where}: {r['error'][:200]}")
            else:
                # (its decodes are decodes too: their output joins the one-hash check)
                for d in r.get("decodes") or []:
                    if not d.get("err"):
                        outputs.add((d.get("hash"), d.get("out")))
            continue
        if key in cells:
            problems.append(f"duplicated: {where}")
            continue
        cells[key] = r
        if r.get("error"):
            problems.append(f"not successful: {where}: {r['error'][:200]}")
            continue
        check_decodes(r, where, problems, outputs)
        i = r.get("idle")
        if not isinstance(i, dict) or not all(num(i.get(k)) for k in ("span", "reads", "writes")):
            problems.append(f"no idle pair: {where} ({r.get('idle_error', '')[:200]})")
            continue
        if not all(num(r.get(k)) for k in ("span", "reads", "writes")) or not r["span"] > 0:
            problems.append(f"no traffic counted: {where}")
            continue
        if out_bytes(r) <= 0:
            problems.append(f"no output bytes: {where}")
            continue
        nr, nw = net(r)
        if not (nr > 0 and nw > 0):
            problems.append(
                f"idle-subtracted traffic not positive (reads {nr}, writes {nw}): {where}"
            )
        rate = (r["reads"] + r["writes"]) / r["span"]
        if not i["reads"] + i["writes"] < IDLE_SHARE * rate:
            problems.append(
                f"idle rate {(i['reads'] + i['writes']) / rate:.1%} of the decode's, not below "
                f"{IDLE_SHARE:.0%}: {where}"
            )
    for t in threads:
        for rep in range(reps):
            if (t, rep) not in cells:
                problems.append(f"missing: t={t} rep {rep}")
    if len(outputs) > 1:
        problems.append(f"the output differs across readings: {len(outputs)} distinct (hash, size)")
    elif outputs and next(iter(outputs))[0] is None:
        problems.append("no output hash recorded")
    return problems, cells, l3cells, l3problems


def analyse(cells, l3cells, threads, reps):
    """b(t) per repetition, R and the reported ratios, and the reported figures"""
    b = {
        (t, r): sum(net(cells[(t, r)])) / out_bytes(cells[(t, r)])
        for t in threads
        for r in range(reps)
    }
    hi = HI if HI in threads else max(threads)
    lo = LO if LO in threads and LO != hi else min(threads)

    def paired(x, y):
        return mean_ci([math.log(b[(x, r)]) - math.log(b[(y, r)]) for r in range(reps)])

    res = {"hi": hi, "lo": lo, "key": paired(hi, lo)}
    res["verdict"] = verdict(*res["key"])
    res["curve"] = {}
    for t in threads:
        rs = [cells[(t, r)] for r in range(reps)]
        nr = [net(c)[0] / out_bytes(c) for c in rs]
        nw = [net(c)[1] / out_bytes(c) for c in rs]
        dwall = [
            sum(d["wall"] for d in c["decodes"] if num(d.get("wall"))) / len(c["decodes"])
            for c in rs
        ]
        row = {
            "b": mean_ci([math.log(b[(t, r)]) for r in range(reps)]),
            "reads": sum(nr) / reps,
            "writes": sum(nw) / reps,
            "decode_wall": mean_ci([math.log(w) for w in dwall]),
            "span_min": min(c["span"] for c in rs),
            "idle_share_max": max(
                (c["idle"]["reads"] + c["idle"]["writes"])
                / ((c["reads"] + c["writes"]) / c["span"])
                for c in rs
            ),
        }
        l3 = [
            l3cells[(t, r)]["l3_miss"] / out_bytes(l3cells[(t, r)])
            for r in range(reps)
            if (t, r) in l3cells
            and num(l3cells[(t, r)].get("l3_miss"))
            and out_bytes(l3cells[(t, r)]) > 0
        ]
        row["l3_miss_per_byte"] = (sum(l3) / len(l3), len(l3)) if l3 else None
        res["curve"][t] = row
    res["reported"] = {
        f"b({x})/b({y})": paired(x, y)
        for x, y in REPORTED_PAIRS
        if x in threads and y in threads and x != y
    }
    if 1 in threads:
        res["b1_over_census"] = res["curve"][1]["b"][0] / CENSUS_HEAP_B
    if 10 in threads:
        res["b10_over_dose_log"] = res["curve"][10]["b"][0] / DOSE_LOG_B10
    res["short"] = sorted(
        (t, r, cells[(t, r)]["span"])
        for t in threads
        for r in range(reps)
        if cells[(t, r)]["span"] < MIN_SPAN
    )
    return res


def fmt(x):
    m, lo, hi = x
    return f"{m:.4f} [{lo:.4f}, {hi:.4f}]"


def report(a, recs, problems, cells, l3cells, l3problems, threads, registered):
    meta = next((r for r in recs if r.get("pass") == "meta"), {})
    lines = [
        "# The working-set premise test (#145, LOG 6880984644)",
        "",
        f"- readings: {a.readings} ({len(recs)} records)",
        f"- grid: threads {','.join(map(str, threads))}, {a.reps} repetitions, "
        f"{meta.get('loops', '?')} decodes a reading, idle pairs of {meta.get('pair_seconds', '?')} s"
        + (
            ""
            if registered
            else " (NOT the registered grid: threads 1,2,3,4,6,8,10, 10 repetitions)"
        ),
        f"- build: `{meta.get('ab', '?')}` (P0: dose feature, MIBZ2_DOSE_MB=0, stream), arm {ARM}",
    ]
    result = {
        "readings": a.readings,
        "threads": threads,
        "reps": a.reps,
        "registered": registered,
        "loops": meta.get("loops"),
        "problems": problems,
        "l3_problems": l3problems,
    }
    tag = "" if registered else " [not the registered grid: no verdict for the record]"
    if problems:
        lines += ["", f"## Validity: FAILED ({len(problems)})", ""]
        lines += [f"- {p}" for p in problems[:50]]
        if len(problems) > 50:
            lines.append(f"- ... and {len(problems) - 50} more")
        verdict_line = f"VERDICT: INVALID ({len(problems)} validity problems){tag}"
        result["verdict"] = "INVALID"
    else:
        r = analyse(cells, l3cells, threads, a.reps)
        hi, lo = r["hi"], r["lo"]
        lines += [
            "",
            "## Validity: passed",
            "",
            "a complete roster with its idle pairs, every decode successful, one output hash, "
            f"dose_mb 0 and dose_blocks {BLOCKS} in every decode, every net figure positive, "
            f"every idle rate below {IDLE_SHARE:.0%} of its decode's",
            "",
            "## b(t): DRAM bytes per output byte (idle-subtracted reads + writes)",
            "",
            "| t | b(t) [95% CI] | reads/B | writes/B | decode wall s [95% CI] "
            "| L3 misses/B (process) | shortest reading s | idle/decode rate (max) |",
            "|---|---|---|---|---|---|---|---|",
        ]
        for t in threads:
            c = r["curve"][t]
            l3 = (
                f"{c['l3_miss_per_byte'][0]:.4f} (n={c['l3_miss_per_byte'][1]})"
                if c["l3_miss_per_byte"]
                else "n/a"
            )
            lines.append(
                f"| {t} | {fmt(c['b'])} | {c['reads']:.3f} | {c['writes']:.3f} "
                f"| {fmt(c['decode_wall'])} | {l3} | {c['span_min']:.2f} | {c['idle_share_max']:.1%} |"
            )
        lines += ["", "## Reported, not gating", ""]
        for k, v in r["reported"].items():
            lines.append(f"- {k} = {fmt(v)}")
        if "b1_over_census" in r:
            lines.append(
                f"- b(1) = {r['curve'][1]['b'][0]:.2f} B against the census's {CENSUS_HEAP_B:g} B "
                f"of heap traffic per output byte: {r['b1_over_census']:.3f}x"
            )
        if "b10_over_dose_log" in r:
            lines.append(
                f"- b(10) = {r['curve'][10]['b'][0]:.2f} B against the dose LOG's estimate of "
                f"{DOSE_LOG_B10:g} B: {r['b10_over_dose_log']:.3f}x"
            )
        if r["short"]:
            lines.append(
                f"- readings under {MIN_SPAN:g} s: "
                + ", ".join(f"t={t} rep {rep} ({s:.2f} s)" for t, rep, s in r["short"])
            )
        if l3problems:
            lines += [f"- L3 pass (reported only): {p}" for p in l3problems[:20]]
        m, klo, khi = r["key"]
        verdict_line = (
            f"VERDICT: {r['verdict']}: R = b({hi})/b({lo}) = {m:.4f} [{klo:.4f}, {khi:.4f}]{tag}"
        )
        result.update(
            verdict=r["verdict"],
            key=r["key"],
            hi=hi,
            lo=lo,
            curve={str(t): v for t, v in r["curve"].items()},
            reported=r["reported"],
            b1_over_census=r.get("b1_over_census"),
            b10_over_dose_log=r.get("b10_over_dose_log"),
            short=r["short"],
        )
    lines += ["", "## Verdict", "", verdict_line]
    result["verdict_line"] = verdict_line
    return "\n".join(lines) + "\n", result


def analyse_main(a):
    threads = sorted({int(t) for t in a.threads.split(",")})
    if len(threads) < 2:
        sys.exit("cells_imc: --threads takes at least two thread counts")
    registered = threads == REGISTERED_THREADS and a.reps == REGISTERED_REPS
    recs, bad = load(a.readings)
    problems, cells, l3cells, l3problems = validate(recs, bad, threads, a.reps)
    text, result = report(a, recs, problems, cells, l3cells, l3problems, threads, registered)
    print(text, end="")
    if a.out:
        with open(a.out + ".md", "w") as f:
            f.write(text)
        with open(a.out + ".json", "w") as f:
            json.dump(result, f, indent=1)
            f.write("\n")
    return 0 if not problems else 1


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("probe", help="fail (exit 2) unless the counters count here")
    c = sub.add_parser("collect", help="take the readings (sudo perf)")
    c.add_argument("--ab", required=True, help="the P0 build's examples/ab")
    c.add_argument("--corpus", required=True)
    c.add_argument("--threads", default=",".join(map(str, REGISTERED_THREADS)))
    c.add_argument("--reps", type=int, default=REGISTERED_REPS)
    c.add_argument("--loops", default="auto", help="decodes a reading, or auto")
    c.add_argument("--pair", type=float, default=3, help="seconds of each paired idle")
    c.add_argument("--settle", type=float, default=20, help="seconds to wait first")
    c.add_argument("--out", required=True)
    n = sub.add_parser("analyse", help="the statistic, verdict and validity")
    n.add_argument("readings")
    n.add_argument("--threads", default=",".join(map(str, REGISTERED_THREADS)))
    n.add_argument("--reps", type=int, default=REGISTERED_REPS)
    n.add_argument("--out", help="write STEM.md and STEM.json")
    a = ap.parse_args(argv)
    if a.cmd == "probe":
        probe()
        return 0
    if a.cmd == "collect":
        return collect(a)
    return analyse_main(a)


if __name__ == "__main__":
    sys.exit(main())
