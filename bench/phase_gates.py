"""The four-worker phase-concurrency screen (#145, registration LOG 6893071988, which adopts the
Optimiser's pricing, REVIEW 6893060396): at 4 workers on the shipped Workers + Cells path, does
capping how many workers are in Cells' induction, in its writer, or decoding a block at once
lower the active worker CPU per output byte and the memory traffic?

`just bench-phase-gates` runs it on the i9 (C5, sudo):

    python3 bench/phase_gates.py probe
    python3 bench/phase_gates.py preflight --ungated AB0 --gated AB --corpus WIKIDATA
    python3 bench/phase_gates.py collect --ungated AB0 --gated AB --corpus WIKIDATA [--reps 10]
        [--loops auto|N] [--pair 3] --out READINGS.jsonl
    python3 bench/phase_gates.py analyse READINGS.jsonl [--reps 10] [--out STEM]

**probe** fails clearly (exit 2) unless `sudo perf stat -a` counts the memory controller's
`uncore_imc/data_reads` and `data_writes`. The core figures are software (task clock), so no
core event is probed. **preflight** checks both builds on one decode each: Cells engaged on all
196 blocks, the software pass counting, the gates absent from the ungated build and at the
capacities set in the gated one.

**The arms,** all `ab child mibz2-workers 4` (forced Workers + Cells), the Wikidata corpus,
`MIBZ2_PHASE_PASS=SW` (the phase-counters software pass: each worker's task clock over each
block's decode_block, and inside induce_cells and write_cells):

| arm | build | MIBZ2_GATE_INDUCTION, _WRITER, _BLOCKS |
|---|---|---|
| U | ungated (`phase-counters` only) | - |
| K | `phase-gates,phase-counters` | 4, 4, 4: the calibration arm and the reference (E4) |
| B | the same | 1, 4, 4 |
| C | the same | 4, 1, 4 |
| D | the same | 1, 1, 4 |
| E2 | the same | 4, 4, 2: 2 blocks in flight inside the workers |
| E3 | the same | 4, 4, 3 |

**collect.** For each repetition, the seven arms in a rotated order (bench/cells_imc.py's
`order`), each one bench/cells_imc.py's paired reading: an idle reading of --pair seconds, then
--loops decodes in one chain under `sudo perf stat -a` on the memory controller. Placement as
bench-cells-phase's at 4 workers: the first logical CPU of each of the first 4 physical cores
(`taskset`), SMT siblings left idle (arranged, not verifiable from inside the run).

**The figures, per reading,** over its decodes:
- active CPU per output byte: the workers' task clock over their blocks' decode_block, which a
  sleeping wait at a gate does not run; the scanner thread, and the workers' job handling
  outside decode_block, are outside it;
- the same by phase: induction, writer, rest (decode_block less the two);
- IMC traffic per output byte: reads and writes, idle-subtracted;
- wall: the mean decode's (ab's own timed region);
- each gate's entries, waited entries and wait.

**Validity** (any failure: INVALID, no readout):
- the complete roster, each arm x repetition exactly once, each with its idle pair;
- every decode successful, --loops decodes a reading, one output hash (and size) across every
  arm: byte-identical output;
- the software pass counting in every decode (pass SW, no errors, no window multiplexed), and
  Cells' writer entered on all 196 blocks;
- the gates as set: none in U; in the gated arms every gate at its arm's capacity, every phase
  entry counted by its gate (induction and writer entries = their scopes, block leases = the
  workers' blocks), never more permits held than its capacity;
- placement on distinct physical cores, none another's sibling; net IMC traffic positive and
  the idle rate below 10% of the decode's;
- **calibration:** K's active CPU per byte within 1% of U's (the paired ratio's estimate).

**Exposure.** A capped gate is exposed when at least 10% of its entries, over all the arm's
readings, waited. An arm whose cap delayed fewer is NOT EXPOSED and its result is not read. D
has two caps: its induction cap's exposure decides whether D is read for the induction readout,
its writer cap's for the writer readout. E2's and E3's cap is the block lease.

**The readouts, against K.** For each arm X, per repetition the ratio X / K, paired, and its
log-t 95% interval; Δ = ratio - 1.
- B or D: Δ active CPU <= -5% with its upper bound below 0, and IMC traffic falls (its Δ's
  upper bound below 0): price the induction's representation and residency work.
- C or D: the same: price the writer's byte reduction.
- E2: Δ active CPU <= -15% with its upper bound below -10%: price Cooperative Cells, with the
  registration's conditions.
- E3 is reported only: E2, E3 and K (E4) are the occupancy curve.

Every readout's verdict line carries the two rules that hold for all of them, and wall is
reported for every arm, with interference flagged where active CPU falls and wall rises.

A grid other than the registered one (10 repetitions) still gets its figures, its verdict lines
marked as not the registered grid.
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
import cells_phase  # noqa: E402
import imc  # noqa: E402
from dose import mean_ci  # noqa: E402

IMC = imc.IMC
ARM = "mibz2-workers"
THREADS = 4
REGISTERED_REPS = 10
BLOCKS = 196
PASS = "SW"
GATES = ["induction", "writer", "blocks"]
VARS = ["MIBZ2_GATE_INDUCTION", "MIBZ2_GATE_WRITER", "MIBZ2_GATE_BLOCKS"]
# the arms' capacities, [induction, writer, blocks]; U is the ungated build
ARMS = {
    "U": None,
    "K": (4, 4, 4),
    "B": (1, 4, 4),
    "C": (4, 1, 4),
    "D": (1, 1, 4),
    "E2": (4, 4, 2),
    "E3": (4, 4, 3),
}
ITEMS = list(ARMS)
REF = "K"
CAPPED = {"B": ["induction"], "C": ["writer"], "D": ["induction", "writer"],
          "E2": ["blocks"], "E3": ["blocks"]}  # fmt: skip
TREATED = ["B", "C", "D", "E2", "E3"]
CALIBRATION = 0.01
EXPOSURE = 0.10
PHASE_FALL = -0.05
E2_FALL = -0.15
E2_UPPER = -0.10
PHASES = ["induction", "writer", "rest"]
WINDOWS = cells_phase.WINDOWS

LINES = {
    "induction": "price the induction's representation and residency work",
    "writer": "price the writer's byte reduction",
    "e2": "price Cooperative Cells, but only with a mechanism beyond the serial-share model. "
    "The Optimiser's floor applies: 0.196 s for two teams against 0.162 s independent.",
}
NULL_RULE = "A null result never eliminates a direction. The byte-saving routes stay open."
WALL_RULE = (
    "Wall time is reported for every arm, and no arm ships. Lower active CPU with a higher wall "
    "shows interference; it doesn't deliver a faster decoder."
)
SCOPE = (
    "Decision scope: the screen chooses which directions to price. No candidate qualifies for "
    "production from it alone."
)
READOUTS = [
    ("induction", "B or D", ["B", "D"], "induction"),
    ("writer", "C or D", ["C", "D"], "writer"),
    ("e2", "E2", ["E2"], "blocks"),
]


def order(items, rep):
    return cells_imc.order(items, rep)


# ----------------------------------------------------------------------------------- probe


def probe():
    if not shutil.which("perf"):
        sys.exit("bench-phase-gates: no perf on PATH (linux-tools); this run needs it")
    if subprocess.run(["sudo", "-v"]).returncode:
        sys.exit("bench-phase-gates: sudo refused; perf stat -a needs it for the memory controller")
    got, err = cells_imc.counted(IMC, ["sleep", "0.2"], True)
    if err:
        print(
            "bench-phase-gates: the memory controller's uncore_imc/data_reads and data_writes "
            f"cannot be counted here; no readings taken:\n{err}",
            file=sys.stderr,
        )
        sys.exit(2)
    print("bench-phase-gates: the counters count (uncore_imc data_reads/data_writes)")


def env_for(arm):
    """The arm's environment: the software pass, and its gates' capacities (empty: none)"""
    caps = ARMS[arm] or ("", "", "")
    return [f"MIBZ2_PHASE_PASS={PASS}", *(f"{v}={c}" for v, c in zip(VARS, caps))]


def binary(a, arm):
    return a.ungated if ARMS[arm] is None else a.gated


def preflight(a):
    """One decode of U and of K: Cells engaged, the pass counting, the gates as set"""
    bad, hashes = [], set()
    for arm in ("U", "K"):
        cmd = ["env", *env_for(arm), os.path.abspath(binary(a, arm)), "child", ARM, str(THREADS)]
        r = subprocess.run([*cmd, os.path.abspath(a.corpus)], capture_output=True, text=True)
        recs = cells_imc.records(r.stdout)
        if r.returncode or len(recs) != 1:
            bad.append(f"{arm}: exit {r.returncode}: {r.stderr.strip()[:200]}")
            continue
        d = recs[0]
        problems = []
        check_decode(d, arm, 0, arm, problems)
        bad += problems
        hashes.add((d.get("hash"), d.get("out")))
        print(f"bench-phase-gates: preflight {arm}: {json.dumps(d.get('gates'))}")
    if len(hashes) > 1:
        bad.append(f"the two builds' outputs differ: {sorted(hashes)}")
    if bad:
        print("bench-phase-gates: the builds are not as the screen needs:", file=sys.stderr)
        for b in bad:
            print(f"  {b}", file=sys.stderr)
        sys.exit(2)


# --------------------------------------------------------------------------------- collect


def chain(ab, corpus, loops, arm, cpus):
    """--loops decodes of `arm` at 4 workers in one chain, its environment set, pinned"""
    pin = ["taskset", "-c", ",".join(map(str, cpus))]
    child = [os.path.abspath(ab), "child", ARM, str(THREADS), os.path.abspath(corpus)]
    loop = 'for i in $(seq "$0"); do "$@" || exit 1; done'
    return ["env", *env_for(arm), *pin, "sh", "-c", loop, str(loops), *child]


def cpu_model():
    try:
        with open("/proc/cpuinfo") as f:
            return next((ln.split(":", 1)[1].strip() for ln in f if ln.startswith("model name")),
                        "?")  # fmt: skip
    except OSError:
        return "?"


def collect(a):
    cores = cells_phase.topology()
    walls = cells_imc.calibrate(a.ungated, a.corpus, [THREADS])
    loops = (
        max(1, math.ceil(cells_imc.AUTO_SPAN / walls[THREADS]))
        if a.loops == "auto"
        else int(a.loops)
    )
    cpus = cells_phase.placement(cores, THREADS)
    meta = {
        "pass": "meta",
        "arms": ARMS,
        "threads": THREADS,
        "reps": a.reps,
        "loops": loops,
        "pair_seconds": a.pair,
        "calibration_wall": walls[THREADS],
        "placement": cpus,
        "cores": cores,
        "cpu_model": cpu_model(),
        "ungated": os.path.abspath(a.ungated),
        "gated": os.path.abspath(a.gated),
        "corpus": os.path.abspath(a.corpus),
        "perf": imc.run(["perf", "--version"]).stdout.strip(),
    }
    print(f"bench-phase-gates: {loops} decodes a reading; placement {cpus}", flush=True)
    with open(a.out, "w") as f:
        f.write(json.dumps(meta) + "\n")
        if a.settle:
            time.sleep(a.settle)
        for rep in range(a.reps):
            imc.run(["sudo", "-v"])
            for slot, arm in enumerate(order(ITEMS, rep)):
                rec = {
                    "pass": "reading",
                    "arm": arm,
                    "caps": ARMS[arm],
                    "rep": rep,
                    "threads": THREADS,
                    "slot": slot,
                    "loops": loops,
                    "cpus": cpus,
                    "cores": [cells_phase.core_of(cores, c) for c in cpus],
                    "siblings": sorted(
                        x for c in cpus for x in cores[cells_phase.core_of(cores, c)] if x != c
                    ),
                }
                got, err = cells_imc.counted(IMC, ["sleep", str(a.pair)], True)
                if err:
                    rec["idle"], rec["idle_error"] = None, err
                else:
                    iw, _, irows = got
                    ir, iwr = imc.rates(iw, irows)
                    rec["idle"] = {"span": imc.span(iw, irows), "reads": ir, "writes": iwr}
                cmd = chain(binary(a, arm), a.corpus, loops, arm, cpus)
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
                    f"bench-phase-gates: rep {rep} {arm} {'ok' if not err else 'FAILED'}",
                    flush=True,
                )
    return 0


# --------------------------------------------------------------------------------- analyse


def check_decode(d, arm, k, where, problems):
    """One decode's rules: success, the software pass, Cells engaged, the gates as set"""
    if d.get("err"):
        problems.append(f"decode {k} failed ({d.get('err')!r}): {where}")
        return
    ph = d.get("phase")
    if not isinstance(ph, dict):
        problems.append(f"decode {k}: no counters: {where}")
        return
    if ph.get("pass") != PASS or ph.get("errors"):
        problems.append(
            f"decode {k}: pass {ph.get('pass')!r}, errors {ph.get('errors')!r}: {where}"
        )
        return
    if any(not isinstance(ph.get(w), dict) for w in WINDOWS):
        problems.append(f"decode {k}: a window missing: {where}")
        return
    for w in WINDOWS:
        if ph[w]["mux"] or ph[w]["enabled"] != ph[w]["running"]:
            problems.append(f"decode {k}: {w} multiplexed: {where}")
    if ph["writer"]["windows"] != BLOCKS:
        problems.append(
            f"decode {k}: {ph['writer']['windows']} writer scopes, not {BLOCKS}: {where}"
        )
    caps = ARMS[arm]
    g = d.get("gates")
    if caps is None:
        if g is not None:
            problems.append(f"decode {k}: gates in the ungated build ({g}): {where}")
        return
    if not isinstance(g, dict) or any(not isinstance(g.get(n), dict) for n in GATES):
        problems.append(f"decode {k}: the gates not all set ({g!r}): {where}")
        return
    want = {
        "induction": ph["induction"]["windows"],
        "writer": ph["writer"]["windows"],
        "blocks": ph["blocks"],
    }
    for n, cap in zip(GATES, caps):
        x = g[n]
        if x.get("capacity") != cap:
            problems.append(
                f"decode {k}: the {n} gate at {x.get('capacity')!r}, not {cap}: {where}"
            )
        if x.get("entries") != want[n]:
            problems.append(
                f"decode {k}: the {n} gate counted {x.get('entries')!r} entries, not "
                f"{want[n]}: {where}"
            )
        if not (0 <= x.get("waited", -1) <= x.get("entries", -1)) or x.get("wait_ns", -1) < 0:
            problems.append(f"decode {k}: the {n} gate's waits unreadable ({x}): {where}")
        if not 0 < x.get("max_held", 0) <= cap:
            problems.append(
                f"decode {k}: the {n} gate held {x.get('max_held')!r} of {cap}: {where}"
            )


def validate(recs, bad, reps):
    problems = [f"line {i} is not JSON" for i in bad]
    got, outputs = {}, set()
    for r in recs:
        if r.get("pass") == "meta":
            continue
        arm, rep = r.get("arm"), r.get("rep")
        where = f"{arm} rep {rep}"
        if r.get("pass") != "reading" or arm not in ARMS or rep not in range(reps):
            problems.append(f"an unexpected reading: {where}")
            continue
        if (arm, rep) in got:
            problems.append(f"duplicated: {where}")
            continue
        got[(arm, rep)] = r
        if r.get("error"):
            problems.append(f"not successful: {where}: {str(r['error'])[:200]}")
            continue
        ds = r.get("decodes") or []
        if len(ds) != r.get("loops"):
            problems.append(f"{len(ds)} decodes, not {r.get('loops')}: {where}")
        for k, d in enumerate(ds):
            check_decode(d, arm, k, where, problems)
            outputs.add((d.get("hash"), d.get("out")))
        cells_phase.check_placement(r, where, problems)
        cells_phase.check_imc(r, where, problems)
    for arm in ITEMS:
        for rep in range(reps):
            if (arm, rep) not in got:
                problems.append(f"missing: {arm} rep {rep}")
    if len(outputs) > 1:
        problems.append(f"the output differs across readings: {len(outputs)} distinct (hash, size)")
    elif outputs and next(iter(outputs))[0] is None:
        problems.append("no output hash recorded")
    return problems, got


def figures(r):
    """A reading's figures: active CPU (ns per output byte, whole and by phase), IMC traffic
    (B per output byte), mean decode wall (s), and each gate's entries, waits and wait (s)"""
    out = cells_imc.out_bytes(r)
    s = cells_phase.phase_sums(r)
    whole = s["whole"]["values"][0]
    ind, wr = s["induction"]["values"][0], s["writer"]["values"][0]
    rest = whole - s["whole_in_induction"]["values"][0] - s["whole_in_writer"]["values"][0]
    nr, nw = cells_imc.net(r)
    ds = r.get("decodes") or []
    f = {
        "active": whole / out,
        "induction": ind / out,
        "writer": wr / out,
        "rest": rest / out,
        "imc": (nr + nw) / out,
        "wall": statistics.fmean(d["wall"] for d in ds),
        "gates": {},
    }
    for n in GATES:
        gs = [(d.get("gates") or {}).get(n) for d in ds]
        if all(isinstance(g, dict) for g in gs):
            f["gates"][n] = {
                "entries": sum(g["entries"] for g in gs),
                "waited": sum(g["waited"] for g in gs),
                "wait_s": sum(g["wait_ns"] for g in gs) / 1e9 / len(ds),
            }
    return f


def paired(F, arm, ref, key, reps):
    """The paired ratio arm / ref of `key` over the repetitions, as Δ = ratio - 1 with its
    log-t 95% interval"""
    logs = [math.log(F[(arm, r)][key]) - math.log(F[(ref, r)][key]) for r in range(reps)]
    m, lo, hi = mean_ci(logs)
    return m - 1, lo - 1, hi - 1


def meets_phase(cpu, traffic):
    """The B/C/D readouts' rule: active CPU falls >= 5%, its upper bound below 0, and IMC
    traffic falls (its upper bound below 0)"""
    return cpu[0] <= PHASE_FALL and cpu[2] < 0 and traffic[2] < 0


def meets_e2(cpu):
    """E2's rule: active CPU at least 15% below K's, its upper bound below -10%"""
    return cpu[0] <= E2_FALL and cpu[2] < E2_UPPER


def calibrated(k_vs_u):
    """K's active CPU per byte within 1% of the ungated build's (the paired estimate)"""
    return abs(k_vs_u[0]) <= CALIBRATION


def exposed(share):
    """A cap that delayed at least 10% of its entries is exposed"""
    return share >= EXPOSURE


def exposure(F, arm, gate, reps):
    e = sum(F[(arm, r)]["gates"][gate]["entries"] for r in range(reps))
    w = sum(F[(arm, r)]["gates"][gate]["waited"] for r in range(reps))
    return w / e if e else 0.0


def analyse(got, reps):
    F = {key: figures(r) for key, r in got.items()}
    res = {"calibration": paired(F, "K", "U", "active", reps)}
    keys = ["active", *PHASES, "imc", "wall"]
    res["delta"] = {x: {k: paired(F, x, REF, k, reps) for k in keys} for x in TREATED}
    res["delta"]["U"] = {k: paired(F, "U", REF, k, reps) for k in keys}
    res["mean"] = {
        x: {k: statistics.fmean(F[(x, r)][k] for r in range(reps)) for k in keys} for x in ITEMS
    }
    res["exposure"] = {x: {g: exposure(F, x, g, reps) for g in CAPPED[x]} for x in TREATED}
    res["waits"] = {
        x: {
            g: statistics.fmean(F[(x, r)]["gates"][g]["wait_s"] for r in range(reps)) for g in GATES
        }
        for x in ["K", *TREATED]
    }
    res["k_waited"] = sum(F[("K", r)]["gates"][g]["waited"] for r in range(reps) for g in GATES)
    res["interference"] = [
        x for x in TREATED if res["delta"][x]["active"][0] < 0 and res["delta"][x]["wall"][0] > 0
    ]
    res["readouts"] = {}
    for name, label, arms, gate in READOUTS:
        read = [x for x in arms if exposed(res["exposure"][x][gate])]
        if name == "e2":
            met = [x for x in read if meets_e2(res["delta"][x]["active"])]
        else:
            met = [
                x for x in read if meets_phase(res["delta"][x]["active"], res["delta"][x]["imc"])
            ]
        status = "MET" if met else ("NOT MET" if read else "NOT EXPOSED")
        res["readouts"][name] = {"label": label, "status": status, "read": read, "met": met,
                                 "gate": gate}  # fmt: skip
    return res


def pct(x):
    m, lo, hi = x
    return f"{100 * m:+.2f}% [{100 * lo:+.2f}, {100 * hi:+.2f}]"


def readout_line(name, ro, res, tag):
    label, gate = ro["label"], ro["gate"]
    arms = {"induction": ["B", "D"], "writer": ["C", "D"], "e2": ["E2"]}[name]
    exp = ", ".join(f"{x} {res['exposure'][x][gate]:.1%}" for x in arms)
    head = f"READOUT {name} ({label}): {ro['status']}"
    if ro["status"] == "MET":
        return f"{head} by {', '.join(ro['met'])}: {LINES[name]}{tag}"
    if ro["status"] == "NOT EXPOSED":
        return (
            f"{head} ({exp} of {gate} entries delayed, against {EXPOSURE:.0%}): not read. "
            f"{NULL_RULE}{tag}"
        )
    d = "; ".join(
        f"{x}: active CPU {pct(res['delta'][x]['active'])}, IMC {pct(res['delta'][x]['imc'])}"
        for x in ro["read"]
    )
    return f"{head} ({d}). {NULL_RULE}{tag}"


def report(a, recs, problems, got, registered):
    meta = next((r for r in recs if r.get("pass") == "meta"), {})
    tag = "" if registered else " [not the registered grid: no verdict for the record]"
    lines = [
        "# The four-worker phase-concurrency screen (#145, LOG 6893071988)",
        "",
        f"- readings: {a.readings} ({len(recs)} records)",
        f"- grid: arms {', '.join(ITEMS)} at {THREADS} workers, {a.reps} repetitions, "
        f"{meta.get('loops', '?')} decodes a reading"
        + ("" if registered else f" (NOT the registered grid: {REGISTERED_REPS} repetitions)"),
        f"- machine: {meta.get('cpu_model', '?')}. These figures are this machine's alone: the "
        "i9's decide, CI's 4-vCPU runner's are reported separately and never pooled with them.",
        f"- placement: {meta.get('placement')} (distinct physical cores; sibling idleness "
        "arranged, not verifiable from inside the run)",
        "- active CPU = the workers' task clock over each block's decode_block per output byte "
        "(the software pass SW); a sleeping wait at a gate is not in it. The scanner thread and "
        "the workers' job handling outside decode_block are not in it either.",
        "- arms: "
        + "; ".join(
            f"{x} " + ("ungated" if c is None else "/".join(map(str, c))) for x, c in ARMS.items()
        )
        + " (induction / writer / blocks in flight)",
    ]
    result = {"readings": a.readings, "reps": a.reps, "registered": registered,
              "problems": problems}  # fmt: skip
    if not problems:
        res = analyse(got, a.reps)
        if not calibrated(res["calibration"]):
            problems.append(
                f"calibration: K's active CPU per byte {pct(res['calibration'])} against the "
                f"ungated build's, outside {CALIBRATION:.0%}"
            )
    if problems:
        lines += ["", f"## Validity: FAILED ({len(problems)})", ""]
        lines += [f"- {p}" for p in problems[:60]]
        if len(problems) > 60:
            lines.append(f"- ... and {len(problems) - 60} more")
        verdict = [f"VERDICT: INVALID ({len(problems)} validity problems){tag}"]
        result["verdict"] = "INVALID"
    else:
        lines += [
            "",
            "## Validity: passed",
            "",
            f"- calibration: K / U active CPU per byte {pct(res['calibration'])} (within "
            f"{CALIBRATION:.0%}); K's gates waited {res['k_waited']} times",
            "",
            "## Against K (Δ = arm / K - 1, paired over the repetitions, 95% interval)",
            "",
            "| arm | exposure | active CPU | induction | writer | rest | IMC traffic | wall |",
            "|---|---|---|---|---|---|---|---|",
        ]
        for x in TREATED + ["U"]:
            e = res["exposure"].get(x, {})
            es = ", ".join(
                f"{g} {v:.1%}{'' if exposed(v) else ' NOT EXPOSED'}" for g, v in e.items()
            )
            d = res["delta"][x]
            lines.append(
                f"| {x} | {es or '-'} | "
                + " | ".join(pct(d[k]) for k in ["active", *PHASES, "imc", "wall"])
                + " |"
            )
        lines += [
            "",
            "## Each arm's means",
            "",
            "| arm | active CPU ns/B | induction | writer | rest | IMC B/B | wall s | "
            "wait s a decode (induction, writer, blocks) |",
            "|---|---|---|---|---|---|---|---|",
        ]
        for x in ITEMS:
            m = res["mean"][x]
            w = res["waits"].get(x)
            ws = ", ".join(f"{w[g]:.4f}" for g in GATES) if w else "-"
            lines.append(
                f"| {x} | {m['active']:.4f} | {m['induction']:.4f} | {m['writer']:.4f} | "
                f"{m['rest']:.4f} | {m['imc']:.4f} | {m['wall']:.4f} | {ws} |"
            )
        lines += [
            "",
            "The occupancy curve (blocks in flight inside the workers): E2, E3 and K (E4), "
            "reported; only E2 has a readout.",
            "",
            "Interference (active CPU lower, wall higher): "
            + (", ".join(res["interference"]) or "none"),
        ]
        verdict = [readout_line(n, res["readouts"][n], res, tag) for n, *_ in READOUTS]
        result.update(
            verdict={n: ro["status"] for n, ro in res["readouts"].items()},
            readouts=res["readouts"],
            calibration=res["calibration"],
            delta=res["delta"],
            mean=res["mean"],
            exposure=res["exposure"],
            interference=res["interference"],
        )
    verdict += [WALL_RULE, SCOPE]
    lines += ["", "## Verdict", "", *verdict]
    result["verdict_lines"] = verdict
    result["problems"] = problems
    return "\n".join(lines) + "\n", result


def analyse_main(a):
    registered = a.reps == REGISTERED_REPS
    recs, bad = cells_imc.load(a.readings)
    problems, got = validate(recs, bad, a.reps)
    text, result = report(a, recs, problems, got, registered)
    print(text, end="")
    if a.out:
        with open(a.out + ".md", "w") as f:
            f.write(text)
        with open(a.out + ".json", "w") as f:
            json.dump(result, f, indent=1)
            f.write("\n")
    return 0 if not result["problems"] else 1


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("probe", help="fail (exit 2) unless the memory controller counts here")
    for name, text in [("preflight", "one decode of each build, checked"),
                       ("collect", "take the readings (sudo perf)")]:  # fmt: skip
        c = sub.add_parser(name, help=text)
        c.add_argument("--ungated", required=True, help="ab with phase-counters only")
        c.add_argument("--gated", required=True, help="ab with phase-gates,phase-counters")
        c.add_argument("--corpus", required=True)
        if name == "collect":
            c.add_argument("--reps", type=int, default=REGISTERED_REPS)
            c.add_argument("--loops", default="auto")
            c.add_argument("--pair", type=float, default=3)
            c.add_argument("--settle", type=float, default=20)
            c.add_argument("--out", required=True)
    n = sub.add_parser("analyse", help="the figures, validity, exposure and readouts")
    n.add_argument("readings")
    n.add_argument("--reps", type=int, default=REGISTERED_REPS)
    n.add_argument("--out")
    a = ap.parse_args(argv)
    if a.cmd == "probe":
        probe()
        return 0
    if a.cmd == "preflight":
        preflight(a)
        return 0
    if a.cmd == "collect":
        return collect(a)
    return analyse_main(a)


if __name__ == "__main__":
    sys.exit(main())
