"""The phase attribution of Cells' memory traffic (#145, LOG 6884716103, amended before any data
by LOGs 6884895257 and 6884975709): of the DRAM traffic per output byte at 1 and 4 workers,
how much is Cells' induction's, its writer's, and the rest of the block's decode's, and how
much of each is co-running uplift?

`just bench-cells-phase` runs it on the i9 (C5, sudo):

    python3 bench/cells_phase.py probe
    python3 bench/cells_phase.py instrument --ab AB --corpus WIKIDATA
    python3 bench/cells_phase.py collect --ab AB --corpus WIKIDATA [--threads 1,4] [--reps 10]
        [--loops auto|N] [--pair 3] --out READINGS.jsonl
    python3 bench/cells_phase.py analyse READINGS.jsonl [--threads 1,4] [--reps 10] [--out STEM]

**probe** fails clearly (exit 2), listing what it found, unless `sudo perf stat -a` counts the
memory controller's `uncore_imc/data_reads` and `data_writes` and `perf stat` counts every
registered core event (`EVENTS`, Skylake-client encodings, user and kernel mode).
**instrument** then checks the build's own counters (the `phase-counters` feature,
src/phase.rs): one decode with each pass, every group opened, no window multiplexed.

**collect.** The dose run's P0 build with `phase-counters`, forced Workers + Cells (`ab child
mibz2-workers <t>`), on the Wikidata corpus of the premise test. For each repetition, the
thread counts in a rotated order, and at each the five readings in a rotated order: the four
matched counter passes A1, A2, B1 and B2 (`MIBZ2_PHASE_PASS`), and the baseline arm
(`mibz2-workers-segmented`: the Segmented T-vector walk, never Cells; IMC only). Each is
bench/cells_imc.py's paired reading: an idle reading of --pair seconds, then --loops decodes
in one chain under `sudo perf stat -a` on the memory controller's events, so that every pass
has its own IMC ledger. The decode is pinned (`taskset`) as `ab --spread` places it: at t > 1
the first logical CPU of each of the first t physical cores, at t = 1 the second core's first CPU (ab's --pin 1 on the i9), so
that the workers sit on distinct physical cores and their SMT siblings are left idle (which is
arranged, and cannot be verified from inside the run).

**The two ledgers, apart.**
- IMC (physical): B_IMC = (reads + writes, idle-subtracted) / output bytes; writes are a total
  only, never split by phase.
- Core (an estimate of selected traffic): the verdict metric, A1's demand-data reads served by
  local DRAM, x 64 B, per phase: induction (`induce_cells`), writer (`write_cells`), rest (the
  worker's `decode_block` less the two, Cells' `slots()` included). The scanner thread's bit
  reading and Huffman stage lie outside every worker scope: their traffic is only in the IMC
  ledger.
- The residual, IMC reads less demand-DRAM reads, is **unattributed**: hardware prefetch, RFOs
  and the scanner among it, never assigned to a phase.

**Statistics.** Per repetition, B_p(t) = 64 x the phase's A1 demand-DRAM count / output bytes.
U_p = B_p(4) / B_p(1), "observed co-running uplift", with its paired log-t 95% interval; the
shares at 4 workers, s_p = B_p(4) / B_core(4), with t-intervals; explained = demand-DRAM reads /
IMC reads at 4 workers, from the A1 readings' own ledgers.

**Verdict** (the first rule that matches decides; LOG 6884716103 as amended):
1. UNATTRIBUTED-DOMINATED, HELD: demand-DRAM reads explain less than 30% of IMC reads at t = 4;
2. WRITER: s_wr >= 0.40 and U_wr >= 1.5 x U_ind, both intervals excluding equality (s_wr's
   lower bound above 0.40, and the paired U_wr / (1.5 U_ind) interval's above 1);
3. SHARED WORKING SET: every phase's U_p >= 3, its lower bound above 2;
4. INDUCTION: s_ind >= 0.50 and U_ind >= U_wr;
5. MIXED otherwise.

**Validity** (any failure: INVALID, no verdict):
- a complete roster of the Cells arm's passes (each t x repetition x pass exactly once), each
  with its idle pair;
- every decode successful, --loops decodes a reading, one output hash (and size) across all;
- `dose_mb` = 0 and `dose_blocks` = 196 in every Cells decode, and the writer's scopes exactly
  the blocks Cells inverted (196 a decode);
- the counters as registered: the reading's pass, user and kernel mode both (CML070), every
  group opened and read without error;
- no multiplexing: every window of every group ran for all the time it was enabled;
- the empty-scope overhead measured: one empty scope for each phase scope;
- placement: the CPUs on distinct physical cores, none another's SMT sibling;
- net IMC traffic positive (reads and writes), the idle rate below 10% of the decode's;
- the phase sum, for every event of every pass: B_ind + B_wr + B_rest = the whole count within
  1%, the phase group's counts against the whole group's over the same windows.

**Reported, never gating:** B_p(1) and B_p(4) in full; the IMC reads and writes; b(1) against
`bench-cells-imc`'s 1.25 B; the unattributed residual; A1's L3-miss demand requests, A2's RFO
local-DRAM responses (CML058: may overcount, no workaround), B1's and B2's request counts, by
phase; the empty-scope overhead; the baseline arm's IMC ledger beside the Cells arm's; the
approximate phase boundaries (requests in flight cross them).

A grid other than the registered one (threads 1,4, 10 repetitions) still gets its statistics,
on the most threads against the fewest, its verdict line marked as not the registered grid.
"""

import argparse
import json
import math
import os
import shutil
import statistics
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cells_imc  # noqa: E402
import imc  # noqa: E402
from dose import T975, mean_ci  # noqa: E402

IMC = imc.IMC
ARM = "mibz2-workers"
BASE_ARM = "mibz2-workers-segmented"
PASSES = ["A1", "A2", "B1", "B2"]
BASE = "base"
ITEMS = PASSES + [BASE]
# the registered core events, as perf names them (Skylake-client, the i9-10900K's 06_A5)
EVENTS = {
    "A1": [
        "cpu/event=0xb7,umask=0x01,offcore_rsp=0x3fc4000001/",
        "cpu/event=0xb0,umask=0x10/",
    ],
    "A2": ["cpu/event=0xbb,umask=0x01,offcore_rsp=0x3fc4000002/"],
    "B1": ["cpu/event=0x24,umask=0x38/", "cpu/event=0xb0,umask=0x08/"],
    "B2": ["cpu/event=0xb0,umask=0x01/"],
}
EVENT_NAMES = {
    "A1": ["demand-data local-DRAM responses", "L3-miss demand data requests"],
    "A2": ["RFO local-DRAM responses (CML058)"],
    "B1": ["L2 prefetch misses", "all data reads offcore"],
    "B2": ["demand data reads offcore"],
}
REGISTERED_THREADS = [1, 4]
REGISTERED_REPS = 10
BLOCKS = 196
LINE = 64
UNATTRIBUTED_AT = 0.30
WRITER_SHARE = 0.40
WRITER_UPLIFT = 1.5
SHARED_UPLIFT = 3.0
SHARED_LOWER = 2.0
INDUCTION_SHARE = 0.50
PHASE_SUM = 0.01
IMC_B1 = 1.25
PHASES = ["induction", "writer", "rest"]


def order(items, rep):
    return cells_imc.order(items, rep)


# ------------------------------------------------------------------------------- placement


def topology(root="/sys/devices/system/cpu"):
    """The physical cores as lists of their logical CPUs, ordered by first CPU"""
    cores = {}
    for d in sorted(os.listdir(root)):
        if not (d.startswith("cpu") and d[3:].isdigit()):
            continue
        base = os.path.join(root, d, "topology")
        try:
            pkg = open(os.path.join(base, "physical_package_id")).read().strip()
            core = open(os.path.join(base, "core_id")).read().strip()
        except OSError:
            continue
        cores.setdefault((pkg, core), []).append(int(d[3:]))
    return sorted((sorted(c) for c in cores.values()), key=lambda c: c[0])


def placement(cores, t):
    """`ab --spread`'s CPUs for t workers (t > 1): the first CPU of each of the first t
    cores; at t = 1 the second core's first CPU (ab's --pin 1 on the i9), else the first's"""
    if t > 1:
        if t > len(cores):
            sys.exit(f"bench-cells-phase: {t} workers need {t} physical cores, not {len(cores)}")
        return [c[0] for c in cores[:t]]
    return [cores[1][0]] if len(cores) > 1 else [cores[0][0]]


def core_of(cores, cpu):
    return next(i for i, c in enumerate(cores) if cpu in c)


# ----------------------------------------------------------------------------------- probe


def probe_events():
    """Each registered event under `sudo perf stat`, user and kernel mode: what counted"""
    found, missing = [], []
    for p, evs in EVENTS.items():
        for e in evs:
            r = subprocess.run(
                ["sudo", "perf", "stat", "-x,", "-e", e, "--", "sleep", "0.1"],
                capture_output=True,
                text=True,
            )
            line = next((ln for ln in r.stderr.splitlines() if e in ln), "")
            ok = r.returncode == 0 and line and not line.split(",")[0].startswith("<not")
            (found if ok else missing).append(f"{p} {e}: {line.strip() or r.stderr.strip()[:200]}")
    return found, missing


def probe():
    if not shutil.which("perf"):
        sys.exit("bench-cells-phase: no perf on PATH (linux-tools); this run needs it")
    if subprocess.run(["sudo", "-v"]).returncode:
        sys.exit("bench-cells-phase: sudo refused; perf stat -a needs it for the memory controller")
    got, err = cells_imc.counted(IMC, ["sleep", "0.2"], True)
    found, missing = probe_events()
    print("bench-cells-phase: the event probe found:")
    print(f"  IMC uncore_imc/data_reads, data_writes: {'counted' if not err else 'NOT counted'}")
    for f in found:
        print(f"  counted: {f}")
    for m in missing:
        print(f"  NOT counted: {m}")
    if err or missing:
        print(
            "bench-cells-phase: a registered event cannot be counted here; no readings taken",
            file=sys.stderr,
        )
        if err:
            print(err, file=sys.stderr)
        sys.exit(2)


def instrument(a):
    """One decode at 1 worker with each pass: the build's groups open, read, unmultiplexed"""
    bad = []
    for p in PASSES:
        cmd = ["sudo", "env", f"MIBZ2_PHASE_PASS={p}", os.path.abspath(a.ab), "child", ARM, "1"]
        r = subprocess.run([*cmd, os.path.abspath(a.corpus)], capture_output=True, text=True)
        recs = cells_imc.records(r.stdout)
        ph = recs[0].get("phase") if recs else None
        if r.returncode or not ph or ph.get("errors") or ph.get("pass") != p:
            bad.append(f"{p}: {ph and ph.get('errors')} {r.stderr.strip()[:200]}")
            continue
        mux = sum(ph[k]["mux"] for k in WINDOWS)
        print(
            f"bench-cells-phase: pass {p}: {ph['threads']} worker(s), {ph['blocks']} blocks, "
            f"writer scopes {ph['writer']['windows']}, multiplexed windows {mux}"
        )
        if mux:
            bad.append(f"{p}: {mux} windows multiplexed")
    if bad:
        print("bench-cells-phase: the instrument cannot count as registered:", file=sys.stderr)
        for b in bad:
            print(f"  {b}", file=sys.stderr)
        sys.exit(2)


WINDOWS = ["whole", "whole_in_induction", "whole_in_writer", "induction", "writer", "empty"]


# --------------------------------------------------------------------------------- collect


def chain(ab, corpus, t, loops, arm, item, cpus):
    """--loops decodes at t threads in one chain, the pass in the environment, pinned"""
    env = ["env", f"MIBZ2_PHASE_PASS={item if item in PASSES else ''}"]
    pin = ["taskset", "-c", ",".join(map(str, cpus))]
    child = [os.path.abspath(ab), "child", arm, str(t), os.path.abspath(corpus)]
    loop = 'for i in $(seq "$0"); do "$@" || exit 1; done'
    return [*env, *pin, "sh", "-c", loop, str(loops), *child]


def collect(a):
    threads = [int(t) for t in a.threads.split(",")]
    cores = topology()
    walls = cells_imc.calibrate(a.ab, a.corpus, threads)
    loops = (
        max(1, math.ceil(cells_imc.AUTO_SPAN / min(walls.values())))
        if a.loops == "auto"
        else int(a.loops)
    )
    place = {t: placement(cores, t) for t in threads}
    meta = {
        "pass": "meta",
        "threads": threads,
        "reps": a.reps,
        "loops": loops,
        "pair_seconds": a.pair,
        "calibration_wall": {str(t): w for t, w in walls.items()},
        "placement": {str(t): c for t, c in place.items()},
        "cores": cores,
        "ab": os.path.abspath(a.ab),
        "corpus": os.path.abspath(a.corpus),
        "perf": imc.run(["perf", "--version"]).stdout.strip(),
    }
    print(f"bench-cells-phase: {loops} decodes a reading; placement {place}", flush=True)
    with open(a.out, "w") as f:
        f.write(json.dumps(meta) + "\n")
        if a.settle:
            time.sleep(a.settle)
        for rep in range(a.reps):
            imc.run(["sudo", "-v"])
            for t in order(threads, rep):
                for slot, item in enumerate(order(ITEMS, rep)):
                    arm = BASE_ARM if item == BASE else ARM
                    cpus = place[t]
                    rec = {
                        "pass": item,
                        "arm": arm,
                        "rep": rep,
                        "threads": t,
                        "slot": slot,
                        "loops": loops,
                        "cpus": cpus,
                        "cores": [core_of(cores, c) for c in cpus],
                        "siblings": sorted(
                            x for c in cpus for x in cores[core_of(cores, c)] if x != c
                        ),
                    }
                    got, err = cells_imc.counted(IMC, ["sleep", str(a.pair)], True)
                    if err:
                        rec["idle"], rec["idle_error"] = None, err
                    else:
                        iw, _, irows = got
                        ir, iwr = imc.rates(iw, irows)
                        rec["idle"] = {"span": imc.span(iw, irows), "reads": ir, "writes": iwr}
                    cmd = chain(a.ab, a.corpus, t, loops, arm, item, cpus)
                    got, err = cells_imc.counted(IMC, cmd, True)
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
                            decodes=cells_imc.records(stdout),
                        )
                    f.write(json.dumps(rec) + "\n")
                    f.flush()
                    print(
                        f"bench-cells-phase: rep {rep} t={t} {item} {'ok' if not err else 'FAILED'}",
                        flush=True,
                    )
    return 0


# --------------------------------------------------------------------------------- analyse


def num(x):
    return cells_imc.num(x)


def phase_sums(rec):
    """The reading's counter sums over its decodes: per window kind, the values summed, with
    enabled, running, windows and mux"""
    out = {}
    for d in rec.get("decodes") or []:
        ph = d.get("phase") or {}
        for k in WINDOWS:
            w = ph.get(k)
            if not isinstance(w, dict):
                continue
            o = out.setdefault(k, {"values": [0] * len(w["values"]), "enabled": 0, "running": 0,
                                   "windows": 0, "mux": 0})  # fmt: skip
            for i, v in enumerate(w["values"]):
                o["values"][i] += v
            for f in ("enabled", "running", "windows", "mux"):
                o[f] += w[f]
    return out


def attribute(rec):
    """Per event: (induction, writer, rest, whole) counts, and the phase sum's error over whole"""
    s = phase_sums(rec)
    rows = []
    for i in range(len(s["whole"]["values"])):
        whole = s["whole"]["values"][i]
        inside = s["whole_in_induction"]["values"][i] + s["whole_in_writer"]["values"][i]
        ind, wr = s["induction"]["values"][i], s["writer"]["values"][i]
        rest = whole - inside
        err = abs(ind + wr + rest - whole) / whole if whole else float(ind + wr != inside)
        rows.append({"induction": ind, "writer": wr, "rest": rest, "whole": whole, "err": err})
    return rows


def check_reading(r, where, problems, outputs, cells_arm=True):
    """The decode and counter rules for one reading of the Cells arm (`cells_arm`), or the
    decode rules alone for the baseline's"""
    if r.get("error"):
        problems.append(f"not successful: {where}: {str(r['error'])[:200]}")
        return
    ds = r.get("decodes") or []
    if len(ds) != r.get("loops"):
        problems.append(f"{len(ds)} decodes, not {r.get('loops')}: {where}")
    for k, d in enumerate(ds):
        if d.get("err"):
            problems.append(f"decode {k} failed ({d.get('err')!r}): {where}")
        outputs.add((d.get("hash"), d.get("out")))
        if not cells_arm:
            continue
        if d.get("dose_mb") != 0 or d.get("dose_blocks") != BLOCKS:
            problems.append(
                f"decode {k}: dose_mb {d.get('dose_mb')!r}, dose_blocks {d.get('dose_blocks')!r}: {where}"
            )
        ph = d.get("phase")
        if not isinstance(ph, dict):
            problems.append(f"decode {k}: no counters: {where}")
            continue
        if ph.get("pass") != r.get("pass"):
            problems.append(f"decode {k}: counted pass {ph.get('pass')!r}: {where}")
        if ph.get("modes") != "user+kernel":
            problems.append(
                f"decode {k}: modes {ph.get('modes')!r}, not user+kernel (CML070): {where}"
            )
        if ph.get("errors"):
            problems.append(f"decode {k}: counter errors {ph['errors']}: {where}")
        if any(not isinstance(ph.get(w), dict) for w in WINDOWS):
            problems.append(f"decode {k}: a window missing: {where}")
            continue
        if ph["writer"]["windows"] != BLOCKS:
            problems.append(
                f"decode {k}: {ph['writer']['windows']} writer scopes, not {BLOCKS}: {where}"
            )
        if ph["empty"]["windows"] != ph["induction"]["windows"] + ph["writer"]["windows"]:
            problems.append(f"decode {k}: the empty scopes not one a phase scope: {where}")
        for w in WINDOWS:
            if ph[w]["mux"] or ph[w]["enabled"] != ph[w]["running"]:
                problems.append(f"decode {k}: {w} multiplexed: {where}")


def check_imc(r, where, problems):
    i = r.get("idle")
    if not isinstance(i, dict) or not all(num(i.get(k)) for k in ("span", "reads", "writes")):
        problems.append(f"no idle pair: {where}")
        return
    if not all(num(r.get(k)) for k in ("span", "reads", "writes")) or not r["span"] > 0:
        problems.append(f"no traffic counted: {where}")
        return
    if cells_imc.out_bytes(r) <= 0:
        problems.append(f"no output bytes: {where}")
        return
    nr, nw = cells_imc.net(r)
    if not (nr > 0 and nw > 0):
        problems.append(f"net IMC traffic not positive (reads {nr}, writes {nw}): {where}")
    rate = (r["reads"] + r["writes"]) / r["span"]
    if not i["reads"] + i["writes"] < cells_imc.IDLE_SHARE * rate:
        problems.append(
            f"idle rate {(i['reads'] + i['writes']) / rate:.1%} of the decode's: {where}"
        )


def check_placement(r, where, problems):
    cpus, cores, sib = r.get("cpus") or [], r.get("cores") or [], set(r.get("siblings") or [])
    t = r.get("threads")
    want = t if isinstance(t, int) and t > 1 else 1
    if len(cpus) != want or len(set(cores)) != len(cpus) or sib & set(cpus):
        problems.append(f"placement: CPUs {cpus} on cores {cores}, siblings {sorted(sib)}: {where}")


def validate(recs, bad, threads, reps):
    problems = [f"line {i} is not JSON" for i in bad]
    base_problems = []
    cells, base = {}, {}
    outputs = set()
    for r in recs:
        p = r.get("pass")
        if p == "meta":
            continue
        key = (p, r.get("threads"), r.get("rep"))
        where = f"{p} t={key[1]} rep {key[2]}"
        if p not in ITEMS or key[1] not in threads or key[2] not in range(reps):
            problems.append(f"an unexpected reading: {where}")
            continue
        if p == BASE:
            if key in base:
                base_problems.append(f"duplicated: {where}")
            base[key] = r
            check_reading(r, where, base_problems, set(), cells_arm=False)
            if not r.get("error"):
                check_imc(r, where, base_problems)
                for d in r.get("decodes") or []:
                    if not d.get("err"):
                        outputs.add((d.get("hash"), d.get("out")))
            continue
        if key in cells:
            problems.append(f"duplicated: {where}")
            continue
        cells[key] = r
        check_reading(r, where, problems, outputs)
        if r.get("error"):
            continue
        check_placement(r, where, problems)
        check_imc(r, where, problems)
        try:
            for i, a in enumerate(attribute(r)):
                if a["err"] > PHASE_SUM:
                    problems.append(
                        f"phase sum off by {a['err']:.2%} of whole (event {i}): {where}"
                    )
        except (KeyError, TypeError, IndexError):
            problems.append(f"counters unreadable: {where}")
    for p in PASSES:
        for t in threads:
            for rep in range(reps):
                if (p, t, rep) not in cells:
                    problems.append(f"missing: {p} t={t} rep {rep}")
    for t in threads:
        for rep in range(reps):
            if (BASE, t, rep) not in base:
                base_problems.append(f"missing: base t={t} rep {rep}")
    if len(outputs) > 1:
        problems.append(f"the output differs across readings: {len(outputs)} distinct (hash, size)")
    elif outputs and next(iter(outputs))[0] is None:
        problems.append("no output hash recorded")
    return problems, cells, base, base_problems


def arith_ci(xs):
    m = statistics.fmean(xs)
    if len(xs) < 2:
        return m, math.nan, math.nan
    h = T975[min(len(xs) - 1, 30) - 1] * statistics.stdev(xs) / math.sqrt(len(xs))
    return m, m - h, m + h


def verdict(explained, s, u, wr_vs_ind):
    """The amended rule, the first matching line deciding: `explained` the mean share of IMC
    reads the demand-DRAM reads explain at t = 4; `s` and `u` per phase (mean, lo, hi);
    `wr_vs_ind` the paired U_wr / U_ind (mean, lo, hi)"""
    if explained < UNATTRIBUTED_AT:
        return "UNATTRIBUTED-DOMINATED, HELD"
    sw, uw, ui = s["writer"], u["writer"], u["induction"]
    if (
        sw[0] >= WRITER_SHARE
        and uw[0] >= WRITER_UPLIFT * ui[0]
        and sw[1] > WRITER_SHARE
        and wr_vs_ind[1] > WRITER_UPLIFT
    ):
        return "WRITER"
    if all(u[p][0] >= SHARED_UPLIFT and u[p][1] > SHARED_LOWER for p in PHASES):
        return "SHARED WORKING SET"
    if s["induction"][0] >= INDUCTION_SHARE and ui[0] >= uw[0]:
        return "INDUCTION"
    return "MIXED"


def per_byte(rec, count):
    return LINE * count / cells_imc.out_bytes(rec)


def analyse(cells, base, threads, reps):
    hi = 4 if 4 in threads else max(threads)
    lo = 1 if 1 in threads and hi != 1 else min(threads)
    a1 = {(t, r): attribute(cells[("A1", t, r)])[0] for t in threads for r in range(reps)}
    B = {
        (p, t, r): per_byte(cells[("A1", t, r)], a1[(t, r)][p])
        for p in PHASES + ["whole"]
        for t in threads
        for r in range(reps)
    }
    res = {"hi": hi, "lo": lo}
    res["B"] = {
        p: {t: arith_ci([B[(p, t, r)] for r in range(reps)]) for t in threads}
        for p in PHASES + ["whole"]
    }

    def logs(p):
        return [math.log(B[(p, hi, r)]) - math.log(B[(p, lo, r)]) for r in range(reps)]

    ok = all(B[(p, t, r)] > 0 for p in PHASES for t in (lo, hi) for r in range(reps))
    nan3 = (math.nan, math.nan, math.nan)
    res["U"] = {p: mean_ci(logs(p)) if ok else nan3 for p in PHASES}
    res["wr_vs_ind"] = (
        mean_ci([w - i for w, i in zip(logs("writer"), logs("induction"))]) if ok else nan3
    )
    res["s"] = {
        p: arith_ci([B[(p, hi, r)] / B[("whole", hi, r)] for r in range(reps)]) for p in PHASES
    }
    imc_reads = {
        (t, r): cells_imc.net(cells[("A1", t, r)])[0] / cells_imc.out_bytes(cells[("A1", t, r)])
        for t in threads
        for r in range(reps)
    }
    imc_writes = {
        (t, r): cells_imc.net(cells[("A1", t, r)])[1] / cells_imc.out_bytes(cells[("A1", t, r)])
        for t in threads
        for r in range(reps)
    }
    res["explained"] = {
        t: arith_ci([B[("whole", t, r)] / imc_reads[(t, r)] for r in range(reps)]) for t in threads
    }
    res["imc"] = {
        t: {
            "reads": arith_ci([imc_reads[(t, r)] for r in range(reps)]),
            "writes": arith_ci([imc_writes[(t, r)] for r in range(reps)]),
            "unattributed": arith_ci([imc_reads[(t, r)] - B[("whole", t, r)] for r in range(reps)]),
        }
        for t in threads
    }
    res["verdict"] = verdict(res["explained"][hi][0], res["s"], res["U"], res["wr_vs_ind"])
    # reported: every pass's events by phase (bytes a line per output byte), the empty scopes
    diag = {}
    for p in PASSES:
        for t in threads:
            rows = [attribute(cells[(p, t, r)]) for r in range(reps)]
            for i in range(len(rows[0])):
                for ph in PHASES + ["whole"]:
                    diag.setdefault(p, {}).setdefault(i, {}).setdefault(ph, {})[t] = arith_ci(
                        [per_byte(cells[(p, t, r)], rows[r][i][ph]) for r in range(reps)]
                    )
    res["diag"] = diag
    over = {}
    for t in threads:
        xs = []
        for r in range(reps):
            s = phase_sums(cells[("A1", t, r)])
            e = s["empty"]
            per_scope = e["values"][0] / e["windows"] if e["windows"] else 0.0
            scopes = s["induction"]["windows"] + s["writer"]["windows"]
            xs.append(per_byte(cells[("A1", t, r)], per_scope * scopes))
        over[t] = arith_ci(xs)
    res["overhead"] = over
    res["base"] = {}
    for t in threads:
        rs = [base[(BASE, t, r)] for r in range(reps) if (BASE, t, r) in base]
        rs = [x for x in rs if not x.get("error") and cells_imc.out_bytes(x) > 0 and x.get("idle")]
        if rs:
            res["base"][t] = arith_ci([sum(cells_imc.net(x)) / cells_imc.out_bytes(x) for x in rs])
    res["cells_imc_total"] = {
        t: arith_ci([imc_reads[(t, r)] + imc_writes[(t, r)] for r in range(reps)]) for t in threads
    }
    return res


def fmt(x, d=4):
    m, lo, hi = x
    return f"{m:.{d}f} [{lo:.{d}f}, {hi:.{d}f}]"


def report(a, recs, problems, cells, base, base_problems, threads, registered):
    meta = next((r for r in recs if r.get("pass") == "meta"), {})
    lines = [
        "# The phase attribution of Cells' memory traffic (#145, LOG 6884716103, amended "
        "6884895257 and 6884975709)",
        "",
        f"- readings: {a.readings} ({len(recs)} records)",
        f"- grid: threads {','.join(map(str, threads))}, {a.reps} repetitions, "
        f"{meta.get('loops', '?')} decodes a reading, passes {', '.join(PASSES)} and the baseline"
        + ("" if registered else " (NOT the registered grid: threads 1,4, 10 repetitions)"),
        f"- placement: {meta.get('placement')} (distinct physical cores; sibling idleness "
        "arranged, not verifiable from inside the run)",
        "- scopes: whole = each worker's decode_block; induction = induce_cells; writer = "
        "write_cells; rest = whole less those (Cells' slots() included). The scanner thread's "
        "bit reading and Huffman stage are outside every worker scope: IMC ledger only. Phase "
        "boundaries are approximate: requests in flight cross them.",
    ]
    result = {
        "readings": a.readings,
        "threads": threads,
        "reps": a.reps,
        "registered": registered,
        "problems": problems,
        "base_problems": base_problems,
    }
    tag = "" if registered else " [not the registered grid: no verdict for the record]"
    if problems:
        lines += ["", f"## Validity: FAILED ({len(problems)})", ""]
        lines += [f"- {p}" for p in problems[:60]]
        if len(problems) > 60:
            lines.append(f"- ... and {len(problems) - 60} more")
        verdict_line = f"VERDICT: INVALID ({len(problems)} validity problems){tag}"
        result["verdict"] = "INVALID"
    else:
        r = analyse(cells, base, threads, a.reps)
        hi, lo = r["hi"], r["lo"]
        lines += [
            "",
            "## Validity: passed",
            "",
            "## The core ledger (an estimate): A1's demand-data local-DRAM reads, 64 B each, per "
            "output byte",
            "",
            "| phase | "
            + " | ".join(f"B_p({t})" for t in threads)
            + f" | U_p = B_p({hi})/B_p({lo}), observed co-running uplift | s_p at {hi} |",
            "|---|" + "---|" * len(threads) + "---|---|",
        ]
        for p in PHASES:
            lines.append(
                f"| {p} | "
                + " | ".join(fmt(r["B"][p][t]) for t in threads)
                + f" | {fmt(r['U'][p])} | {fmt(r['s'][p])} |"
            )
        lines.append(
            "| core (whole) | " + " | ".join(fmt(r["B"]["whole"][t]) for t in threads) + " | | |"
        )
        lines += [
            "",
            f"paired U_wr / U_ind = {fmt(r['wr_vs_ind'])}",
            "",
            "## The IMC ledger (physical), per output byte, from the A1 readings",
            "",
            "| t | reads | writes (total only) | unattributed (IMC reads - demand-DRAM reads) "
            "| demand-DRAM reads / IMC reads |",
            "|---|---|---|---|---|",
        ]
        for t in threads:
            m = r["imc"][t]
            lines.append(
                f"| {t} | {fmt(m['reads'])} | {fmt(m['writes'])} | {fmt(m['unattributed'])} "
                f"| {fmt(r['explained'][t])} |"
            )
        if 1 in threads:
            lines.append(
                f"\nb(1) = IMC reads + writes at 1 worker: {r['cells_imc_total'][1][0]:.3f} B "
                f"against bench-cells-imc's {IMC_B1} B"
            )
        lines += ["", "## Reported, not gating", ""]
        for p in PASSES:
            for i, by in r["diag"][p].items():
                name = EVENT_NAMES[p][i]
                lines.append(
                    f"- {p} {name}: "
                    + "; ".join(
                        f"{ph} " + ", ".join(f"t={t} {by[ph][t][0]:.4f}" for t in threads)
                        for ph in PHASES
                    )
                    + " (B a line per output byte)"
                )
        lines.append(
            "- the empty-scope overhead (A1 demand-DRAM, B per output byte): "
            + ", ".join(f"t={t} {fmt(r['overhead'][t])}" for t in threads)
        )
        lines.append(
            "- the baseline arm (Segmented T-vector walk), IMC reads + writes per output byte: "
            + ", ".join(
                f"t={t} {fmt(r['base'][t])} (Cells {r['cells_imc_total'][t][0]:.4f})"
                for t in threads
                if t in r["base"]
            )
        )
        lines += [f"- baseline (not gating): {p}" for p in base_problems[:20]]
        lines.append("- A2's RFO local-DRAM responses are flagged CML058: they may overcount.")
        verdict_line = (
            f"VERDICT: {r['verdict']}: explained at t={hi} {r['explained'][hi][0]:.3f}; "
            f"s_ind {r['s']['induction'][0]:.3f}, s_wr {r['s']['writer'][0]:.3f}, "
            f"s_rest {r['s']['rest'][0]:.3f}; U_ind {r['U']['induction'][0]:.3f}, "
            f"U_wr {r['U']['writer'][0]:.3f}, U_rest {r['U']['rest'][0]:.3f}{tag}"
        )
        result.update(
            verdict=r["verdict"],
            hi=hi,
            lo=lo,
            U=r["U"],
            s=r["s"],
            wr_vs_ind=r["wr_vs_ind"],
            explained={str(t): v for t, v in r["explained"].items()},
            B={p: {str(t): v for t, v in d.items()} for p, d in r["B"].items()},
            imc={str(t): v for t, v in r["imc"].items()},
            overhead={str(t): v for t, v in r["overhead"].items()},
            base={str(t): v for t, v in r["base"].items()},
        )
    lines += ["", "## Verdict", "", verdict_line]
    result["verdict_line"] = verdict_line
    return "\n".join(lines) + "\n", result


def analyse_main(a):
    threads = sorted({int(t) for t in a.threads.split(",")})
    if len(threads) < 2:
        sys.exit("cells_phase: --threads takes at least two thread counts")
    registered = threads == REGISTERED_THREADS and a.reps == REGISTERED_REPS
    recs, bad = cells_imc.load(a.readings)
    problems, cells, base, base_problems = validate(recs, bad, threads, a.reps)
    text, result = report(a, recs, problems, cells, base, base_problems, threads, registered)
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
    sub.add_parser("probe", help="fail (exit 2) unless every registered event counts here")
    i = sub.add_parser("instrument", help="the build's own counters, one decode a pass")
    i.add_argument("--ab", required=True)
    i.add_argument("--corpus", required=True)
    c = sub.add_parser("collect", help="take the readings (sudo perf)")
    c.add_argument("--ab", required=True, help="the P0 build with phase-counters")
    c.add_argument("--corpus", required=True)
    c.add_argument("--threads", default=",".join(map(str, REGISTERED_THREADS)))
    c.add_argument("--reps", type=int, default=REGISTERED_REPS)
    c.add_argument("--loops", default="auto")
    c.add_argument("--pair", type=float, default=3)
    c.add_argument("--settle", type=float, default=20)
    c.add_argument("--out", required=True)
    n = sub.add_parser("analyse", help="the ledgers, statistics, verdict and validity")
    n.add_argument("readings")
    n.add_argument("--threads", default=",".join(map(str, REGISTERED_THREADS)))
    n.add_argument("--reps", type=int, default=REGISTERED_REPS)
    n.add_argument("--out")
    a = ap.parse_args(argv)
    if a.cmd == "probe":
        probe()
        return 0
    if a.cmd == "instrument":
        instrument(a)
        return 0
    if a.cmd == "collect":
        return collect(a)
    return analyse_main(a)


if __name__ == "__main__":
    sys.exit(main())
