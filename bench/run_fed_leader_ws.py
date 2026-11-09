"""The run-fed leader's workstation admission (docs/journal/2026-11-09-run-fed-leader.md
§8, §R4): fail-closed, on one attempt's directory from `just bench-run-fed-leader`.
Separate from the CI reporter (bench/run_fed_leader.py), whose judged run is 15 rounds at
9 and 3 threads: this one admits exactly 20 paired rounds at a requested 10 threads (140
timed records), and the two occupancy captures apart from the timing.

    python3 bench/run_fed_leader_ws.py admit DIR [--dry] [--out STEM]
    python3 bench/run_fed_leader_ws.py admit-occupancy DIR SUB [--dry] [--out STEM]

`admit` reads DIR/identity.json (the recipe's, `schema` IDENTITY_SCHEMA) and DIR/runs.jsonl.
VALID only if every predicate holds: the bundle's identity (the frozen commit, the source,
recipe and admission digests at it, a clean tree, the build, one binary throughout, the
registered corpus by digest before and after, the arms, threads and rounds), then every
record's typed raw evidence (the CI reporter's own checks, `record_problems`, at 10
threads: each run seen running three leaders and six pipelined helpers), every arm once a
round in ab's rotation, and one output throughout. Then the reading, prospective and
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

`--dry` admits a DRY bundle (`just bench-run-fed-leader 1`): DRY_ROUNDS rounds, the
occupancy at DRY_OCC_ROUNDS, a corpus other than the registered one allowed, and no
frozen commit required. Its label is DRY, never JUDGED, and it is never evidence.

Exit status: 0 VALID, 2 INVALID (its reasons printed), and any failure of this script
itself is INVALID with its reason, never a crash and never a VALID.
"""

import argparse
import json
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
IDENTITY_SCHEMA = "run_fed_leader/workstation-identity/1"
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
TEAM_WORKERS = 3  # each team's leader records its buffers at its end
HEX40, HEX64 = rfl.HEX40, rfl.HEX64


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
    if not (rfl.is_int(ident.get("ab_exit")) and ident["ab_exit"] == 0):
        bad.append(f"ab run exited {ident.get('ab_exit')!r}: the timing did not finish")
    rounds = DRY_ROUNDS if dry else ROUNDS
    if ident.get("rounds") != rounds:
        bad.append(f"the identity's rounds {ident.get('rounds')!r}, not {rounds}")
    return bad


def admit_records(recs, rounds):
    """The records' problems (none: admitted), and the runs by (arm, round)"""
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
        # (the CI reporter's typed checks, with the plan checked at this run's threads)
        bad += rfl.record_problems(rec, where, THREADS)
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
    recs, problems = rfl.load(os.path.join(d, "runs.jsonl"))
    bad += problems
    b, runs = admit_records(recs, rounds)
    bad += b
    out = {
        "label": label,
        "threads": THREADS,
        "rounds": rounds,
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
    runs = occ.get("runs")
    if not isinstance(runs, list) or len(runs) != 3 * rounds:
        bad.append(
            f"{len(runs) if isinstance(runs, list) else runs!r} runs, not {3 * rounds} (three modes a round)"
        )
        runs = runs if isinstance(runs, list) else []
    for i, run in enumerate(runs):
        line = run.get("line") if isinstance(run, dict) else None
        kv = dict(re.findall(r"(\S+?)=(\S+)", line)) if isinstance(line, str) else {}
        if kv.get("plan") != OCC_PLAN.format(sub=sub):
            bad.append(f"run {i}: plan {kv.get('plan')!r}, not {OCC_PLAN.format(sub=sub)}")
        if kv.get("err") != "-":
            bad.append(f"run {i}: err {kv.get('err')!r}")
        if ref is not None and (kv.get("hash"), kv.get("out")) != (ref[0], str(ref[1])):
            bad.append(
                f"run {i}: output {kv.get('hash')!r}/{kv.get('out')!r}, not the incumbent's {ref[0]}/{ref[1]}"
            )
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
        pert = occ["results"][0].get("perturbation", {})
        out["perturbation"] = pert
        # (the instrument's own verdict: "perturbs: ..." where a traced mode's paired
        # throughput interval excludes no change)
        perturbs = [
            m
            for m, v in pert.items()
            if isinstance(v, dict) and str(v.get("verdict", "")).startswith("perturbs")
        ]
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


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
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
    try:
        code = admit(args) if args.cmd == "admit" else admit_occupancy(args)
    except Exception as e:  # (an admission's bug is never a VALID)
        print("INVALID")
        print(f"  the admission failed: {type(e).__name__}: {e}")
        code = 2
    sys.exit(code)


if __name__ == "__main__":
    main()
