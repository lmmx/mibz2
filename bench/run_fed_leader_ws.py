"""The run-fed leader's workstation admission (docs/journal/2026-11-09-run-fed-leader.md
§8, §R4, revised by §R10 for the second attempt): fail-closed, on one attempt's directory
from `just bench-run-fed-leader`.
Separate from the CI reporter (bench/run_fed_leader.py), whose judged run is 15 rounds at
9 and 3 threads: this one admits exactly 20 paired rounds at a requested 10 threads (140
timed records), and the two occupancy captures apart from the timing.

    python3 bench/run_fed_leader_ws.py preflight DIR [--out STEM]
    python3 bench/run_fed_leader_ws.py admit DIR [--dry] [--out STEM]
    python3 bench/run_fed_leader_ws.py admit-occupancy DIR SUB [--dry] [--out STEM]
    python3 bench/run_fed_leader_ws.py diagnose DIR [--out STEM]

`preflight` (§R10) reads what the recipe collected read-only before any timing
(DIR/preflight/: `ab env`, lscpu, three `ab sample` readings) and refuses unless the host is
the registered one with its declared settings and its sysfs diagnostics readable. It holds
no clock or temperature to any value and waits for nothing: one pass, PASS or REFUSED.

`admit` reads DIR/identity.json (the recipe's, `schema` IDENTITY_SCHEMA) and DIR/runs.jsonl.
VALID only if every predicate holds: the bundle's identity (the frozen commit, the source,
recipe and admission digests at it, a clean tree, the build, one binary throughout, the
registered corpus by digest before and after, the arms, threads and rounds), then every
record's typed raw evidence (the CI reporter's own checks, `record_problems`), each run's
plan three leaders and six pipelined helpers in the decoders' own record of the threads
they ran (`ran_threads`, complete by construction; the census's polled `observed_threads`
is reported, not judged: it can catch a helper only under its leader's inherited name,
D059), every arm once a round in ab's rotation, and one output throughout, the registered
workload's (WORKLOAD: its length and hash, §R10). Then the reading, prospective and
unchanged from §6: the primary (`dir`) over the incumbent (`default33`) at 10 threads,
LEAD / STOP / INCONCLUSIVE, with its attribution, and the secondaries (`ls`, `hs`) and
the control contrasts reported, never judged. Intervals are 95% t-intervals on the paired
log ratios, df = 19.

`admit-occupancy` reads DIR/occupancy-SUB.json (SUB ctl or dir): the capture's own
admission, never part of the timing's: the instrument's format, the run-fed sub-arm at 10
threads in three teams of three, the registered rounds, every round complete in both
traced modes, every run's output the timing's incumbent's (bytes and hash), each team's
buffers recorded, and the commit and corpus the identity's. Its perturbation verdicts are
reported, not judged: a capture whose traced modes perturb is admitted as a capture whose
timeline is not to be interpreted, and says so.

`diagnose` (§R10) writes the diagnostics, never read by `admit` and never a verdict: every
record in run order with its position, its predecessor and the gap since it, the sysfs
snapshots before and after it, and each pair's log ratios; the arms by position and by
predecessor, as counts; and each pair's endpoint clock labels (HH, HL, LH, LL or unknown)
as counts only. Every pair enters `admit`'s reading whatever its label.

`--dry` admits a DRY bundle (`just bench-run-fed-leader 1`): DRY_ROUNDS rounds, the
occupancy at DRY_OCC_ROUNDS, a corpus other than the registered one allowed, and no
frozen commit required. Its label is DRY, never JUDGED, and it is never evidence.

Exit status: 0 VALID, 2 INVALID (its reasons printed), and any failure of this script
itself is INVALID with its reason, never a crash and never a VALID.
"""

import argparse
import hashlib
import json
import math
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_fed_leader as rfl  # noqa: E402

# The workstation registration (§8 / §R4), prospective
ROUNDS = 20
DRY_ROUNDS = 2
THREADS = 10  # requested; every arm runs three teams of three (rfl.PLAN)
ARMS = rfl.REGISTERED_ARMS  # the CI run's seven, in ab's --arms order
REF = rfl.REF
PRIMARY = rfl.PRIMARY
SECONDARY = rfl.SECONDARY
CORPUS_SHA256 = rfl.CORPUS_SHA256
IDENTITY_SCHEMA = "run_fed_leader/workstation-identity/2"
# The registered workload (§R10): the incumbent's decoded length and hash on the corpus
# (CORPUS_SHA256), as every one of the first attempt's 140 records read
# (run-fed-leader/20261109T080633Z) and the i9's release-scaling runs of 2026-11-06 before it.
# Every record of a registered run must decode exactly this
WORKLOAD = {"out": 209700196, "hash": "a8830caee5c511a6"}
# the binaries the bundle keeps (bin/), each rehashed here against the identity's digests
BUNDLED_BINARIES = ["ab", "occupancy"]
# The preflight (§R10): the registered host, and its declared settings as
# telemetry-check/20261109T125938Z read them before this revision was written. Read only;
# nothing here is a clock or temperature threshold
PREFLIGHT_SCHEMA = "run_fed_leader/preflight/1"
HOST_MODEL = "i9-10900K"
HOST_CPUS = 20
HOST_CORES = 10
DECLARED = {
    "cpufreq_driver": "intel_pstate",
    "governor": "powersave",
    "epp": "balance_performance",
    "turbo": True,
}
PREFLIGHT_SAMPLES = 3
# The diagnostics' endpoint label (§R10): a run's post-exit sysfs snapshot is "H" when at least
# STATE_CPUS of its CPUs read scaling_cur_freq at or above BASE_MHZ (the i9-10900K's nominal
# base clock), "L" when it read every CPU and fewer did, else unknown. A label of a moment
# after the run, not the clock during it
BASE_MHZ = 3700
STATE_CPUS = THREADS
FEATURES = ["run-fed"]
OCC_FEATURES = ["trace", "run-fed"]
PROFILE = "release"
# the identity's digests of the frozen sources, each checked against the file at the frozen
# commit by the recipe and recorded here
SOURCES = ["recipe", "admission", "reporter", "registration"]
# the occupancy captures: the instrument's rounds (each of its three modes twice), and its
# label for each sub-arm
OCC_SUBS = ["ctl", "dir"]
OCC_ROUNDS = 6
DRY_OCC_ROUNDS = 3
OCC_TEAMS = "pipelined teams of 3, forced, levers: rf{sub}"
OCC_PLAN = "pipe3x3-rf{sub}"
OCC_MODES = ("off", "timeline", "full")
# the instrument's verdicts (examples/occupancy.rs), the two a finite interval can give
OCC_NO_PERTURBATION = "no perturbation shown (interval includes no change)"
OCC_PERTURBS = "perturbs: its trace is not interpreted at this thread count"
# the reported interval against the grid's recomputation (the instrument writes 6 decimals)
OCC_TOLERANCE = 1e-5
TEAM_WORKERS = 3  # each team's leader records its buffers at its end
HEX40, HEX64 = rfl.HEX40, rfl.HEX64
# What a record's wall and CPU include of the thread record (src/threads.rs, D061): stated
# with every admission, never subtracted
CPU_SCOPE = (
    "wall and CPU both include the decoders' thread record's pushes (one a thread, under one "
    "lock, each allocating its name); the record's read, sort and join come after both "
    "readings, in every arm's child alike, so neither includes them"
)


FULL_DATA = (
    "every planned pair enters the reading: none excluded, replaced, stratified or "
    "clock-normalised (§R10); the sysfs snapshots are read only by `diagnose`"
)


def load_json(path, what):
    try:
        with open(path) as f:
            return json.load(f), None
    except (OSError, ValueError) as e:
        return None, f"the {what} does not load: {e}"


def identity_problems(ident, dry):
    """The bundle's identity against the workstation registration"""
    bad = []
    if not isinstance(ident, dict):
        return ["the identity is not an object"]
    if ident.get("schema") != IDENTITY_SCHEMA:
        bad.append(f"identity schema {ident.get('schema')!r}, not {IDENTITY_SCHEMA}")
    if ident.get("dry") is not dry:
        bad.append(
            f"the identity's dry {ident.get('dry')!r}: "
            + ("a DRY admission of a registered run" if dry else "a DRY run is not evidence")
        )
    head, frozen = ident.get("head"), ident.get("frozen")
    if not (isinstance(head, str) and HEX40.fullmatch(head)):
        bad.append(f"the run's head {head!r} is not a full 40-character commit")
    if not dry and not (isinstance(frozen, str) and HEX40.fullmatch(frozen)):
        bad.append(f"the frozen commit {frozen!r} is not a full 40-character commit")
    if ident.get("inputs_match_frozen") is not True and not dry:
        bad.append("the head's measured inputs do not match the frozen commit's")
    if ident.get("tree_clean") is not True:
        bad.append("the checkout's tree was not clean")
    src = ident.get("sources")
    if not isinstance(src, dict):
        bad.append("the frozen sources' digests are not recorded")
    else:
        for k in SOURCES:
            d = src.get(k)
            if not (isinstance(d, dict) and isinstance(d.get("path"), str)):
                bad.append(f"source {k}: no path recorded")
                continue
            if not (isinstance(d.get("sha256"), str) and HEX64.fullmatch(d["sha256"])):
                bad.append(f"source {k} ({d.get('path')}): sha256 {d.get('sha256')!r}")
            if not dry and d.get("at_frozen") != d.get("sha256"):
                bad.append(
                    f"source {k} ({d['path']}): {d.get('sha256')!r} at the head, "
                    f"{d.get('at_frozen')!r} at the frozen commit"
                )
    if ident.get("features") != FEATURES:
        bad.append(f"features {ident.get('features')!r}, not {FEATURES}")
    if ident.get("occupancy_features") != OCC_FEATURES:
        bad.append(
            f"the occupancy build's features {ident.get('occupancy_features')!r}, not {OCC_FEATURES}"
        )
    if ident.get("profile") != PROFILE:
        bad.append(f"profile {ident.get('profile')!r}, not {PROFILE}")
    if not (isinstance(ident.get("rustc"), str) and ident["rustc"].startswith("rustc ")):
        bad.append(f"rustc {ident.get('rustc')!r} not recorded")
    for b in ("ab", "occupancy"):
        b0, b1 = ident.get(f"{b}_sha256_before"), ident.get(f"{b}_sha256_after")
        if not (isinstance(b0, str) and HEX64.fullmatch(b0) and b0 == b1):
            bad.append(f"the {b} binary's SHA-256 {b0!r} before, {b1!r} after: not one build")
    c0, c1 = ident.get("corpus_sha256_before"), ident.get("corpus_sha256_after")
    if not (isinstance(c0, str) and HEX64.fullmatch(c0) and c0 == c1):
        bad.append(f"the corpus's SHA-256 {c0!r} before, {c1!r} after: not one corpus")
    elif not dry and c0 != CORPUS_SHA256:
        bad.append(f"the corpus's SHA-256 {c0}, not the registered {CORPUS_SHA256}")
    if ident.get("arms") != ARMS:
        bad.append(f"arms {ident.get('arms')!r}, not the registered {ARMS}")
    if ident.get("threads") != [THREADS]:
        bad.append(f"threads {ident.get('threads')!r}, not [{THREADS}]")
    if ident.get("sysfs_state") is not True:
        bad.append(
            f"sysfs_state {ident.get('sysfs_state')!r}: the run did not record its snapshots"
        )
    if not dry:
        if ident.get("preflight") != "PASS":
            bad.append(f"the preflight {ident.get('preflight')!r}, not PASS")
        if ident.get("workload") != WORKLOAD:
            bad.append(f"the identity's workload {ident.get('workload')!r}, not {WORKLOAD}")
    if not (rfl.is_int(ident.get("ab_exit")) and ident["ab_exit"] == 0):
        bad.append(f"ab run exited {ident.get('ab_exit')!r}: the timing did not finish")
    rounds = DRY_ROUNDS if dry else ROUNDS
    if ident.get("rounds") != rounds:
        bad.append(f"the identity's rounds {ident.get('rounds')!r}, not {rounds}")
    return bad


def sha256_of(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def binary_problems(d, ident):
    """The bundle's own copies of the binaries that ran, rehashed against the identity"""
    bad = []
    if not isinstance(ident, dict):
        return bad
    for b in BUNDLED_BINARIES:
        p = os.path.join(d, "bin", b)
        try:
            got = sha256_of(p)
        except OSError as e:
            bad.append(f"bin/{b}: not in the bundle ({e.strerror})")
            continue
        if got != ident.get(f"{b}_sha256_before"):
            bad.append(
                f"bin/{b}: SHA-256 {got}, not the identity's {ident.get(f'{b}_sha256_before')!r}"
            )
    return bad


def admit_records(recs, rounds, census_misreads=None, workload=None):
    """The records' problems (none: admitted), and the runs by (arm, round); the census's
    plans other than the registered one into `census_misreads` (reported, not judged)"""
    census_misreads = [] if census_misreads is None else census_misreads
    bad = []
    runs = {arm: {} for arm in ARMS}
    order = {}
    for rec in recs:
        t, arm, rnd = rec.get("threads"), rec.get("arm"), rec.get("round")
        where = f"line {rec['_line']} ({arm} t{t} r{rnd})"
        if not (rfl.is_int(t) and t == THREADS):
            bad.append(f"{where}: threads {t!r}, not {THREADS}")
            continue
        if arm not in ARMS:
            bad.append(f"{where}: an unregistered arm")
            continue
        if not (rfl.is_int(rnd) and 0 <= rnd < rounds):
            bad.append(f"{where}: round {rnd!r}, not 0..{rounds - 1}")
            continue
        if rnd in runs[arm]:
            bad.append(f"{where}: twice")
            continue
        runs[arm][rnd] = rec
        order.setdefault(rnd, []).append(arm)
        # (the CI reporter's typed checks; the plan from the decoders' own record of the
        # threads they ran, complete by construction, not from the census's polling, which
        # can catch a helper only under its leader's inherited name: D059)
        bad += rfl.record_problems(rec, where, None)
        ran = rec.get("ran_threads")
        if not isinstance(ran, str):
            bad.append(f"{where}: no ran_threads (the decoders' record of the threads they ran)")
        elif rfl.plan_of(ran) != rfl.PLAN:
            bad.append(
                f"{where}: ran {rfl.plan_of(ran)} (leaders, plain, pipelined helpers), not {rfl.PLAN}"
            )
        obs = rec.get("observed_threads")
        if isinstance(obs, str) and rfl.plan_of(obs) != rfl.PLAN:
            census_misreads.append(f"{where}: the census read {rfl.plan_of(obs)}")
    for arm in ARMS:
        missing = sorted(set(range(rounds)) - set(runs[arm]))
        if missing:
            bad.append(f"{arm}: rounds missing {missing}")
    for rnd in range(rounds):
        want = [ARMS[i] for i in rfl.arm_order(rnd, len(ARMS))]
        if order.get(rnd) and order[rnd] != want:
            bad.append(f"r{rnd}: the arms ran {order[rnd]}, not ab's rotation {want}")
    n = sum(len(v) for v in runs.values())
    if n != rounds * len(ARMS):
        bad.append(f"{n} timed records admitted, not {rounds * len(ARMS)}")
    outs = {(r.get("hash"), r.get("out")) for r in runs[REF].values()}
    if len(outs) > 1:
        bad.append(f"{REF}: its outputs differ across rounds: {sorted(map(str, outs))}")
    for arm in ARMS:
        for rnd, r in runs[arm].items():
            ref = runs[REF].get(rnd)
            if ref is not None and (r.get("hash"), r.get("out")) != (
                ref.get("hash"),
                ref.get("out"),
            ):
                bad.append(f"{arm} r{rnd}: output differs from the incumbent's")
    if workload is not None:
        wrong = [
            f"{r.get('arm')} r{r.get('round')}"
            for r in recs
            if (r.get("out"), r.get("hash")) != (workload["out"], workload["hash"])
        ]
        if wrong:
            bad.append(
                f"{len(wrong)} records did not decode the registered workload "
                f"({workload['out']} bytes, hash {workload['hash']}): {wrong[:6]}"
            )
    return bad, runs


def paired(runs, num, den, key, rounds):
    return [runs[num][i][key] / runs[den][i][key] for i in range(rounds)]


def reading(runs, rounds):
    """The registered reading (§6's tables, at 10 threads, df = rounds - 1)"""
    cells = {}
    for arm in ARMS:
        if arm != REF:
            cells[f"{arm}/{REF}"] = {
                k: rfl.geo_ci(paired(runs, arm, REF, k, rounds)) for k in ("wall", "cpu")
            }
    contrasts = {}
    for num, den in [(a, rfl.CTL) for a in rfl.RECORD_ARMS] + [
        (rfl.REG, rfl.CTL),
        (rfl.CTL, rfl.BOTH),
    ]:
        contrasts[f"{num}/{den}"] = {
            k: rfl.geo_ci(paired(runs, num, den, k, rounds)) for k in ("wall", "cpu")
        }
    missing = [k for k, v in cells.items() if v["wall"] is None or v["cpu"] is None]
    missing += [k for k, v in contrasts.items() if v["wall"] is None]
    if missing:
        return None, [f"{k}: an absent or non-finite interval" for k in missing]
    w, c = cells[f"{PRIMARY}/{REF}"]["wall"], cells[f"{PRIMARY}/{REF}"]["cpu"]
    read = rfl.reading_of(w, c)
    ctl = cells[f"{rfl.CTL}/{REF}"]["wall"]
    fidelity = rfl.FIDELITY[0] <= ctl[1] and ctl[2] <= rfl.FIDELITY[1]
    p_ctl = contrasts[f"{PRIMARY}/{rfl.CTL}"]["wall"]
    beats_ctl = p_ctl[2] < 1
    if read == "LEAD":
        if fidelity and beats_ctl:
            attribution = "attributed: ctl matches the incumbent, and the primary beats ctl"
        else:
            why = []
            if not fidelity:
                why.append(f"ctl/{REF} wall {rfl.fmt(ctl)} not inside {list(rfl.FIDELITY)}")
            if not beats_ctl:
                why.append(f"{PRIMARY}/ctl wall {rfl.fmt(p_ctl)} does not exclude 1 from above")
            attribution = "ATTRIBUTION FAILED (" + "; ".join(why) + ")"
    else:
        attribution = (
            "control fidelity held"
            if fidelity
            else f"control fidelity FAILED: ctl/{REF} wall {rfl.fmt(ctl)}"
        )
    secondaries = {
        a: rfl.reading_of(cells[f"{a}/{REF}"]["wall"], cells[f"{a}/{REF}"]["cpu"])
        for a in SECONDARY
    }
    return {
        "cells": cells,
        "contrasts": contrasts,
        "reading": read,
        "attribution": attribution,
        "fidelity": fidelity,
        "secondary_readings": secondaries,
        "df": rounds - 1,
    }, []


def write(out, stem):
    if stem:
        with open(stem + ".json", "w") as f:
            json.dump(out, f, indent=1)


def admit(args):
    d, dry = args.dir, args.dry
    rounds = DRY_ROUNDS if dry else ROUNDS
    label = "DRY (not evidence)" if dry else "JUDGED (workstation)"
    bad = []
    ident, why = load_json(os.path.join(d, "identity.json"), "identity")
    if why:
        bad.append(why)
    else:
        bad += identity_problems(ident, dry)
        bad += binary_problems(d, ident)
    recs, problems = rfl.load(os.path.join(d, "runs.jsonl"))
    bad += problems
    census = []
    b, runs = admit_records(recs, rounds, census, None if dry else WORKLOAD)
    bad += b
    out = {
        "cpu_scope": CPU_SCOPE,
        "census_misreads": census,
        "label": label,
        "threads": THREADS,
        "rounds": rounds,
        "workload": None if dry else WORKLOAD,
        "full_data": FULL_DATA,
        "identity": ident,
        "invalid": bad,
        "records": len(recs),
        "raw": [{k: v for k, v in r.items() if k != "_line"} for r in recs],
    }
    if not bad:
        r, b = reading(runs, rounds)
        bad += b
        if r:
            out.update(r)
    out["invalid"] = bad
    write(out, args.out)
    if bad:
        print("INVALID")
        for x in bad:
            print(f"  {x}")
        return 2
    print(
        f"VALID ({label}): {rounds} rounds at {THREADS} threads, {len(recs)} timed records, against {REF} (fixed)"
    )
    for arm in ARMS[1:]:
        x = out["cells"][f"{arm}/{REF}"]
        tag = " (PRIMARY)" if arm == PRIMARY else " (secondary)" if arm in SECONDARY else ""
        print(f"  {arm + tag:<36} wall {rfl.fmt(x['wall'])}  CPU {rfl.fmt(x['cpu'])}")
    if out["census_misreads"]:
        print(
            f"  (the census misread {len(out['census_misreads'])} plans; the plan is the decoders' record: reported, not judged)"
        )
    print(f"  (scope: {CPU_SCOPE})")
    print(f"  (full data: {FULL_DATA})")
    print("Control contrasts (reported, never judged):")
    for k, x in out["contrasts"].items():
        print(f"  {k:<44} wall {rfl.fmt(x['wall'])}  CPU {rfl.fmt(x['cpu'])}")
    print(f"Primary ({PRIMARY} / {REF}, {label}): {out['reading']}; {out['attribution']}")
    for a, v in out["secondary_readings"].items():
        print(f"  secondary ({a}, never judged): {v}")
    return 0


def occupancy_problems(occ, sub, ident, ref, dry):
    """One capture against its registration, and against the timing's incumbent output"""
    bad = []
    if not isinstance(occ, dict):
        return ["the capture is not an object"]
    rounds = DRY_OCC_ROUNDS if dry else OCC_ROUNDS
    if occ.get("format") != 1:
        bad.append(f"format {occ.get('format')!r}, not 1")
    s = occ.get("settings") if isinstance(occ.get("settings"), dict) else {}
    if s.get("threads") != [THREADS]:
        bad.append(f"threads {s.get('threads')!r}, not [{THREADS}]")
    if s.get("rounds") != rounds:
        bad.append(f"rounds {s.get('rounds')!r}, not {rounds}")
    if s.get("teams") != OCC_TEAMS.format(sub=sub):
        bad.append(f"teams {s.get('teams')!r}, not {OCC_TEAMS.format(sub=sub)!r}")
    if occ.get("parity") is not True:
        bad.append("the runs' outputs were not one (parity)")
    meta = occ.get("meta") if isinstance(occ.get("meta"), dict) else {}
    if isinstance(ident, dict):
        if meta.get("commit") != ident.get("head"):
            bad.append(f"the capture's commit {meta.get('commit')!r}, not the run's head")
        if meta.get("corpus_sha256") != ident.get("corpus_sha256_before"):
            bad.append("the capture's corpus is not the timing's")
    res = occ.get("results")
    if not (isinstance(res, list) and len(res) == 1 and isinstance(res[0], dict)):
        return bad + ["not one result (one thread count, one placement)"]
    r = res[0]
    for mode in ("timeline", "full"):
        m = r.get(mode) if isinstance(r.get(mode), dict) else {}
        if m.get("complete_rounds") != rounds:
            bad.append(f"{mode}: {m.get('complete_rounds')!r} complete rounds, not {rounds}")
        if not (rfl.positive(m.get("blocks")) and rfl.positive(m.get("pipe.n.run"))):
            bad.append(f"{mode}: no blocks or team runs traced")
        if m.get("f.w.recorded") != TEAM_WORKERS:
            bad.append(
                f"{mode}: {m.get('f.w.recorded')!r} team workers' buffers recorded, not {TEAM_WORKERS}"
            )
    # the raw grid: every (round, mode) exactly once, its clock finite and positive, its
    # output the timing's incumbent's (the summaries above are the instrument's own, and
    # never stand in for the grid: Researcher 6733680620)
    runs = occ.get("runs")
    if not isinstance(runs, list):
        return bad + [f"runs {runs!r}, not a list"]
    grid = {}
    for i, run in enumerate(runs):
        if not isinstance(run, dict):
            bad.append(f"run {i}: not an object")
            continue
        rnd, mode, line = run.get("round"), run.get("mode"), run.get("line")
        where = f"run {i} (round {rnd!r}, {mode!r})"
        if not (rfl.is_int(rnd) and 0 <= rnd < rounds) or mode not in OCC_MODES:
            bad.append(f"{where}: not a round 0..{rounds - 1} in {list(OCC_MODES)}")
            continue
        if run.get("threads") != THREADS:
            bad.append(f"{where}: threads {run.get('threads')!r}, not {THREADS}")
        if (rnd, mode) in grid:
            bad.append(f"{where}: twice")
            continue
        kv = dict(re.findall(r"(\S+?)=(\S+)", line)) if isinstance(line, str) else {}
        if kv.get("plan") != OCC_PLAN.format(sub=sub):
            bad.append(f"{where}: plan {kv.get('plan')!r}, not {OCC_PLAN.format(sub=sub)}")
        if kv.get("err") != "-":
            bad.append(f"{where}: err {kv.get('err')!r}")
        if ref is not None and (kv.get("hash"), kv.get("out")) != (ref[0], str(ref[1])):
            bad.append(
                f"{where}: output {kv.get('hash')!r}/{kv.get('out')!r}, not the incumbent's {ref[0]}/{ref[1]}"
            )
        try:
            wall, out_b = float(kv.get("wall", "")), int(kv.get("out", ""))
        except ValueError:
            wall, out_b = math.nan, 0
        if not (math.isfinite(wall) and wall > 0 and out_b > 0):
            bad.append(
                f"{where}: wall {kv.get('wall')!r}, out {kv.get('out')!r}: not a finite positive clock"
            )
            continue
        grid[(rnd, mode)] = out_b / wall / 1e6
    missing = [(r, m) for r in range(rounds) for m in OCC_MODES if (r, m) not in grid]
    if missing:
        bad.append(
            f"the grid lacks {len(missing)} of {rounds * len(OCC_MODES)} observations: {missing[:6]}"
        )
    # each traced mode's perturbation, recomputed from the grid (the instrument's own
    # `ratio_ci`: the geometric mean of mode / off, round by round, and its 95% t-interval),
    # and the reported interval and verdict held to it: finite, and classified as it says
    pert = r.get("perturbation") if isinstance(r.get("perturbation"), dict) else {}
    for mode in ("timeline", "full"):
        v = pert.get(mode)
        if not isinstance(v, dict):
            bad.append(f"perturbation {mode}: absent")
            continue
        tr = v.get("throughput_ratio") if isinstance(v.get("throughput_ratio"), dict) else {}
        got = tuple(tr.get(k) for k in ("ratio", "lo", "hi"))
        if not all(
            isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x)
            for x in got
        ):
            bad.append(f"perturbation {mode}: interval {got!r}, not finite")
            continue
        verdict = v.get("verdict")
        want_verdict = OCC_NO_PERTURBATION if got[1] <= 1 <= got[2] else OCC_PERTURBS
        if verdict != want_verdict:
            bad.append(f"perturbation {mode}: verdict {verdict!r}, not {want_verdict!r} for {got}")
        if missing:
            continue
        mine = rfl.geo_ci([grid[(rr, mode)] / grid[(rr, "off")] for rr in range(rounds)])
        if mine is None or any(
            abs(a - b) > OCC_TOLERANCE * max(abs(b), 1) for a, b in zip(got, mine)
        ):
            bad.append(f"perturbation {mode}: reported {got}, not the grid's {mine}")
    return bad


def admit_occupancy(args):
    d, sub, dry = args.dir, args.sub, args.dry
    bad = []
    if sub not in OCC_SUBS:
        bad.append(f"sub-arm {sub!r}, not one of {OCC_SUBS}")
    ident, why = load_json(os.path.join(d, "identity.json"), "identity")
    if why:
        bad.append(why)
    recs, problems = rfl.load(os.path.join(d, "runs.jsonl"))
    refs = {(r.get("hash"), r.get("out")) for r in recs if r.get("arm") == REF}
    ref = None
    if len(refs) != 1:
        bad.append(f"the timing's incumbent output is not one: {sorted(map(str, refs))}")
    else:
        ref = next(iter(refs))
    occ, why = load_json(os.path.join(d, f"occupancy-{sub}.json"), f"{sub} capture")
    if why:
        bad.append(why)
    else:
        bad += occupancy_problems(occ, sub, ident, ref, dry)
    out = {"sub": sub, "dry": dry, "invalid": bad}
    if not bad:
        pert = occ["results"][0]["perturbation"]
        out["perturbation"] = {m: pert[m] for m in ("timeline", "full")}
        # (interpretable only where both traced modes were checked above, each finite, its
        # verdict "no perturbation shown" by its own interval and the grid's: a missing,
        # unknown or insufficient comparison is INVALID above, never interpretable)
        perturbs = [m for m in ("timeline", "full") if pert[m]["verdict"] != OCC_NO_PERTURBATION]
        out["perturbing_modes"] = perturbs
        out["interpretable"] = not perturbs
    write(out, args.out)
    if bad:
        print(f"INVALID ({sub} capture)")
        for x in bad:
            print(f"  {x}")
        return 2
    note = (
        "its traced modes perturb: admitted, its timeline not to be interpreted"
        if not out["interpretable"]
        else "no perturbation shown"
    )
    print(f"VALID ({sub} capture{', DRY' if dry else ''}): {note}")
    return 0


# ---------------------------------------------------------------------------------------
# the preflight (§R10): read-only, bounded by the recipe, before anything is timed


def preflight_problems(env, lscpu, samples):
    """The registered host, its declared settings and its readable diagnostics; any fact
    missing or unreadable is a problem (fail closed). No clock or temperature value is held
    to anything"""
    bad = []
    model = None
    for line in (lscpu or "").splitlines():
        k, _, v = line.partition(":")
        if k.strip() == "Model name":
            model = v.strip()
    if not (isinstance(model, str) and HOST_MODEL in model):
        bad.append(f"lscpu's model {model!r}: not the registered {HOST_MODEL}")
    if not isinstance(env, dict):
        return bad + ["ab env: not an object"]
    if env.get("cpus") != HOST_CPUS:
        bad.append(f"ab env: {env.get('cpus')!r} CPUs allowed, not {HOST_CPUS}")
    if env.get("cores") != HOST_CORES:
        bad.append(f"ab env: {env.get('cores')!r} physical cores, not {HOST_CORES}")
    if env.get("cpu_quota") is not None:
        bad.append(f"ab env: a cgroup CPU quota {env.get('cpu_quota')!r}")
    m = env.get("machine") if isinstance(env.get("machine"), dict) else {}
    for k, want in DECLARED.items():
        if m.get(k) != want:
            bad.append(f"ab env: {k} {m.get(k)!r}, not the declared {want!r}")
    if not (isinstance(samples, list) and len(samples) == PREFLIGHT_SAMPLES):
        n = len(samples) if isinstance(samples, list) else samples
        return bad + [f"{n!r} sysfs samples, not {PREFLIGHT_SAMPLES}"]
    for i, smp in enumerate(samples):
        st = smp.get("state") if isinstance(smp, dict) else None
        if not isinstance(st, dict):
            bad.append(f"sample {i}: no state")
            continue
        bad += [f"sample {i}: {x}" for x in snapshot_problems(st)]
        for k in ("governor", "epp", "turbo"):
            if st.get(k) != DECLARED[k]:
                bad.append(f"sample {i}: {k} {st.get(k)!r}, not the declared {DECLARED[k]!r}")
    return bad


def snapshot_problems(st):
    """What a sysfs snapshot lacks of the diagnostics §R10 records"""
    bad = []
    khz = st.get("cpu_khz")
    if not isinstance(khz, dict) or len(khz) != HOST_CPUS:
        bad.append(
            f"scaling_cur_freq for {len(khz) if isinstance(khz, dict) else 0} CPUs, not {HOST_CPUS}"
        )
    elif not all(rfl.is_int(v) and v > 0 for v in khz.values()):
        bad.append("an unreadable scaling_cur_freq: " + json.dumps(st.get("cpu_khz_why")))
    t = st.get("pkg_temp_c")
    if not (isinstance(t, (int, float)) and not isinstance(t, bool) and math.isfinite(t)):
        bad.append(f"package temperature {t!r} ({st.get('pkg_temp_c_why')!r})")
    if not rfl.is_int(st.get("t_unix_ns")):
        bad.append("no timestamp")
    return bad


def preflight(args):
    d = os.path.join(args.dir, "preflight")
    bad, facts = [], {}
    try:
        env, why = load_json(os.path.join(d, "env.json"), "ab env")
        if why:
            bad.append(why)
        try:
            with open(os.path.join(d, "lscpu.txt")) as f:
                lscpu = f.read()
        except OSError as e:
            lscpu = None
            bad.append(f"lscpu: {e.strerror}")
        samples = []
        try:
            with open(os.path.join(d, "sample.jsonl")) as f:
                samples = [json.loads(line) for line in f if line.strip()]
        except (OSError, ValueError) as e:
            bad.append(f"the sysfs samples do not load: {e}")
        bad += preflight_problems(env, lscpu, samples)
        facts = {"env": env, "samples": samples}
    except Exception as e:  # (the preflight's own failure refuses; it never passes)
        bad.append(f"the preflight failed: {type(e).__name__}: {e}")
    verdict = "REFUSED" if bad else "PASS"
    out = {
        "schema": PREFLIGHT_SCHEMA,
        "verdict": verdict,
        "problems": bad,
        "host": HOST_MODEL,
        "declared": DECLARED,
        "facts": facts,
    }
    write(out, args.out)
    print(verdict + ("" if not bad else ": " + "; ".join(bad)))
    return 0 if not bad else 1


# ---------------------------------------------------------------------------------------
# the diagnostics (§R10): descriptive, never read by `admit`, never a verdict


def endpoint_label(st):
    """H, L or None (unknown) for one post-exit snapshot (BASE_MHZ, STATE_CPUS)"""
    khz = st.get("cpu_khz") if isinstance(st, dict) else None
    if not isinstance(khz, dict) or len(khz) != HOST_CPUS:
        return None
    if not all(rfl.is_int(v) and v > 0 for v in khz.values()):
        return None
    high = sum(1 for v in khz.values() if v >= BASE_MHZ * 1000)
    return "H" if high >= STATE_CPUS else "L"


def num(x):
    return (
        x if isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) else None
    )


def diagnose_records(recs):
    """Every record in run order, with its position, predecessor and snapshots"""
    rows, position = [], {}
    prev = None
    for i, r in enumerate(recs):
        rnd = r.get("round")
        pos = position.get(rnd, 0)
        position[rnd] = pos + 1
        pre = r.get("machine_pre") if isinstance(r.get("machine_pre"), dict) else None
        post = r.get("machine_post") if isinstance(r.get("machine_post"), dict) else None
        gap = None
        if (
            prev is not None
            and rfl.is_int(r.get("t_spawn_unix_ns"))
            and rfl.is_int(prev.get("t_exit_unix_ns"))
        ):
            gap = (r["t_spawn_unix_ns"] - prev["t_exit_unix_ns"]) / 1e9
        rows.append(
            {
                "seq": i,
                "round": rnd,
                "position": pos,
                "arm": r.get("arm"),
                "predecessor": None if prev is None else prev.get("arm"),
                "gap_s": gap,
                "wall": num(r.get("wall")),
                "cpu": num(r.get("cpu")),
                "pre_mhz": num((pre or {}).get("freq_mhz")),
                "post_mhz": num((post or {}).get("freq_mhz")),
                "pre_temp_c": num((pre or {}).get("pkg_temp_c")),
                "post_temp_c": num((post or {}).get("pkg_temp_c")),
                "post_governor": (post or {}).get("governor"),
                "post_epp": (post or {}).get("epp"),
                "post_turbo": (post or {}).get("turbo"),
                "post_label": endpoint_label(post),
                "snapshots": pre is not None and post is not None,
            }
        )
        prev = r
    return rows


def diagnose(args):
    recs, problems = rfl.load(os.path.join(args.dir, "runs.jsonl"))
    rows = diagnose_records(recs)
    by = {(x["arm"], x["round"]): x for x in rows}
    pairs, counts = [], {}
    for arm in ARMS[1:]:
        c = {"HH": 0, "HL": 0, "LH": 0, "LL": 0, "unknown": 0}
        for rnd in sorted({x["round"] for x in rows if x["arm"] == arm and rfl.is_int(x["round"])}):
            a, ref = by.get((arm, rnd)), by.get((REF, rnd))
            if ref is None or a is None:
                continue
            la, lr = a["post_label"], ref["post_label"]
            state = lr + la if la and lr else "unknown"
            c[state] += 1
            lw = math.log(a["wall"] / ref["wall"]) if a["wall"] and ref["wall"] else None
            lc = math.log(a["cpu"] / ref["cpu"]) if a["cpu"] and ref["cpu"] else None
            pairs.append(
                {
                    "arm": arm,
                    "round": rnd,
                    "ref_seq": ref["seq"],
                    "seq": a["seq"],
                    "state": state,
                    "log_wall_ratio": lw,
                    "log_cpu_ratio": lc,
                }
            )
        counts[arm] = c
    total = {k: sum(c[k] for c in counts.values()) for k in ("HH", "HL", "LH", "LL", "unknown")}
    by_position, by_predecessor = {}, {}
    for x in rows:
        by_position.setdefault(x["arm"], {}).setdefault(str(x["position"]), 0)
        by_position[x["arm"]][str(x["position"])] += 1
        by_predecessor.setdefault(x["arm"], {}).setdefault(str(x["predecessor"]), 0)
        by_predecessor[x["arm"]][str(x["predecessor"])] += 1
    lacking = [x["seq"] for x in rows if not x["snapshots"]]
    unknown_post = [x["seq"] for x in rows if x["post_label"] is None]
    status = (
        "complete"
        if rows and not lacking and not unknown_post and not problems
        else f"incomplete: {len(lacking)} of {len(rows)} records lack a snapshot, "
        f"{len(unknown_post)} have no endpoint label"
        + (f", {len(problems)} unreadable lines" if problems else "")
    )
    out = {
        "never_judged": "descriptive only: no pair is excluded, replaced or reweighted by "
        "anything here, and `admit`'s reading reads none of it (§R10)",
        "label_rule": f"post-exit snapshot: H if at least {STATE_CPUS} of {HOST_CPUS} CPUs "
        f"read scaling_cur_freq >= {BASE_MHZ} MHz, L if every CPU was read and fewer did, "
        "else unknown; a pair's state is the incumbent's label then the arm's. An endpoint "
        "label, not the clock during the decode",
        "status": status,
        "pair_states": counts,
        "pair_states_total": total,
        "arm_by_position": by_position,
        "arm_by_predecessor": by_predecessor,
        "records": rows,
        "pairs": pairs,
        "unreadable": problems,
    }
    write(out, args.out)
    if args.out:
        with open(args.out + ".md", "w") as f:
            f.write(diagnostics_md(out))
    print(
        f"diagnostics {status}; pair states (counts only, never judged): "
        + ", ".join(f"{k} {v}" for k, v in total.items())
    )
    return 0


def diagnostics_md(out):
    def cell(x, f="{:.3f}"):
        return "-" if x is None else f.format(x) if isinstance(x, float) else str(x)

    lines = [
        "# The run-fed leader's workstation diagnostics (descriptive, never judged)",
        "",
        out["never_judged"] + ".",
        "",
        f"Status: {out['status']}.",
        "",
        f"Endpoint label: {out['label_rule']}.",
        "",
        "## Pair states, counts only",
        "",
        "| arm | HH | HL | LH | LL | unknown |",
        "|---|---|---|---|---|---|",
    ]
    for arm, c in out["pair_states"].items():
        lines.append(
            f"| `{arm}` | "
            + " | ".join(str(c[k]) for k in ("HH", "HL", "LH", "LL", "unknown"))
            + " |"
        )
    t = out["pair_states_total"]
    lines.append(
        "| all | " + " | ".join(str(t[k]) for k in ("HH", "HL", "LH", "LL", "unknown")) + " |"
    )
    lines += [
        "",
        "## Every record, in run order",
        "",
        "| seq | round | pos | arm | predecessor | gap s | wall | cpu | MHz pre | MHz post | °C pre | °C post | label |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|",
    ]
    for x in out["records"]:
        lines.append(
            f"| {x['seq']} | {x['round']} | {x['position']} | `{x['arm']}` | {cell(x['predecessor'])} | "
            f"{cell(x['gap_s'])} | {cell(x['wall'], '{:.4f}')} | {cell(x['cpu'], '{:.4f}')} | "
            f"{cell(x['pre_mhz'], '{:.0f}')} | {cell(x['post_mhz'], '{:.0f}')} | "
            f"{cell(x['pre_temp_c'], '{:.0f}')} | {cell(x['post_temp_c'], '{:.0f}')} | {cell(x['post_label'])} |"
        )
    lines += [
        "",
        "## Each pair's log ratios over the incumbent, in run order (descriptive)",
        "",
        "| arm | round | state | log wall ratio | log CPU ratio |",
        "|---|---|---|---|---|",
    ]
    for p in sorted(out["pairs"], key=lambda p: p["seq"]):
        lines.append(
            f"| `{p['arm']}` | {p['round']} | {p['state']} | {cell(p['log_wall_ratio'], '{:+.4f}')} | {cell(p['log_cpu_ratio'], '{:+.4f}')} |"
        )
    return "\n".join(lines) + "\n"


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    pf = sub.add_parser("preflight")
    pf.add_argument("dir")
    pf.add_argument("--out")
    dg = sub.add_parser("diagnose")
    dg.add_argument("dir")
    dg.add_argument("--out")
    a = sub.add_parser("admit")
    a.add_argument("dir")
    a.add_argument("--dry", action="store_true")
    a.add_argument("--out")
    o = sub.add_parser("admit-occupancy")
    o.add_argument("dir")
    o.add_argument("sub")
    o.add_argument("--dry", action="store_true")
    o.add_argument("--out")
    args = ap.parse_args()
    if args.cmd == "preflight":
        sys.exit(preflight(args))
    if args.cmd == "diagnose":
        try:
            code = diagnose(args)
        except Exception as e:  # (the diagnostics' failure is stated; it judges nothing)
            print(f"diagnostics FAILED: {type(e).__name__}: {e}")
            code = 3
        sys.exit(code)
    try:
        code = admit(args) if args.cmd == "admit" else admit_occupancy(args)
    except Exception as e:  # (an admission's bug is never a VALID)
        print("INVALID")
        print(f"  the admission failed: {type(e).__name__}: {e}")
        code = 2
    sys.exit(code)


if __name__ == "__main__":
    main()
