"""The run-fed leader's A/B (docs/journal/2026-11-09-run-fed-leader.md, a DRAFT
registration, NOT FROZEN): its validity, its paired ratios, the leaders' phases, and the
reading the registration sets.

    python3 bench/run_fed_leader.py report RUNS.jsonl [--rounds N] [--threads T] [--out STEM]

RUNS.jsonl is one `ab run` of the arms below, interleaved by round, from a build of the
harness with the `run-fed` feature. The incumbent is the shipped decoder through its own
entry point (`mibz2-default33`: `BzReader::parallel` with the three-teams-of-three plan
forced); the arms are the run-fed teams at the same plan and window:

    mibz2-default33          the incumbent (the reference)
    mibz2-pipe3-both         the shipped worker through the research constructor (a check)
    mibz2-pipe3-runfed-ctl   this module's worker, no records, the shipped MTF kernel
    mibz2-pipe3-runfed-reg   no records, the register-list checkpointed MTF (C29's control)
    mibz2-pipe3-runfed-ls    the sort on the leader
    mibz2-pipe3-runfed-hs    the sort on the helpers (C30): the PRIMARY arm
    mibz2-pipe3-runfed-dir   no F-order array (C31)

The judged cells are at T threads (default 9, the plan's own: three teams of three). Each
ratio is paired by round: the arm's wall over the reference's (< 1: the arm is faster), and
its CPU over the reference's (< 1: it spends less), the geometric mean and its 95%
t-interval over the rounds.

VALID only if every predicate holds; any failure prints INVALID with its reasons and exits
2, and no reading is given:
- every arm has exactly ROUNDS runs at T threads (no more, none missing);
- every run's child exited 0 (`exit`), reported no error (`err`) and is not broken;
- every run's output hash and length equal the reference's in the same round, and every
  reference run's equal each other;
- every run-fed arm reports its phases, with team runs > 0 and no fallback (`fallbacks`,
  a refused check's rebuild: a bug), and every record arm took team blocks by records;
- every ratio's interval exists and is finite (at least 2 rounds, no zero or absent wall).
The raw paired timings (each round's wall and CPU, both arms) are written to STEM.json
beside the summary, so the reading can be recomputed.
"""

import argparse
import json
import math
import statistics
import sys
from collections import defaultdict

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from team_size import T975  # noqa: E402

REF = "mibz2-default33"
ARMS = [
    "mibz2-pipe3-both",
    "mibz2-pipe3-runfed-ctl",
    "mibz2-pipe3-runfed-reg",
    "mibz2-pipe3-runfed-ls",
    "mibz2-pipe3-runfed-hs",
    "mibz2-pipe3-runfed-dir",
]
RECORD_ARMS = {"mibz2-pipe3-runfed-ls", "mibz2-pipe3-runfed-hs", "mibz2-pipe3-runfed-dir"}
PRIMARY = "mibz2-pipe3-runfed-hs"
# The registration's thresholds (DRAFT, NOT FROZEN), on the primary arm's paired ratios
POSITIVE_WALL = 0.97  # geometric mean at most this, and its interval's high end below 1
CPU_CAP = 1.05  # and its CPU ratio's high end at most this
NEGATIVE_WALL = 0.99  # STOP: the wall ratio's low end above this (no 1% cut)


def geo_ci(ratios):
    """The geometric mean and its 95% t-interval; None where it can't be computed"""
    if len(ratios) < 2 or any(not (r > 0 and math.isfinite(r)) for r in ratios):
        return None
    logs = [math.log(r) for r in ratios]
    m = statistics.fmean(logs)
    h = T975[min(len(logs) - 1, 30) - 1] * statistics.stdev(logs) / math.sqrt(len(logs))
    out = (math.exp(m), math.exp(m - h), math.exp(m + h))
    return out if all(math.isfinite(x) for x in out) else None


def phases_of(rec):
    out = {}
    for part in rec.get("phases", "").split():
        k, _, v = part.partition("=")
        try:
            out[k] = int(v)
        except ValueError:
            pass
    return out


def load(path, threads):
    runs = defaultdict(dict)  # arm -> round -> record
    problems = []
    with open(path) as f:
        for i, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                problems.append(f"line {i}: not JSON")
                continue
            if rec.get("threads") != threads:
                continue
            arm, rnd = rec.get("arm"), rec.get("round")
            if rnd in runs[arm]:
                problems.append(f"{arm}: round {rnd} twice")
            runs[arm][rnd] = rec
    return runs, problems


def validate(runs, rounds):
    bad = []
    want = set(range(rounds))
    for arm in [REF] + ARMS:
        got = set(runs.get(arm, {}))
        if got != want:
            bad.append(f"{arm}: rounds {sorted(got)} (want 0..{rounds - 1})")
    for arm, by in runs.items():
        for rnd, r in by.items():
            if r.get("broken") or r.get("exit") != 0 or r.get("err"):
                bad.append(f"{arm} r{rnd}: exit {r.get('exit')}, err {r.get('err')!r}")
            if not (isinstance(r.get("wall"), (int, float)) and r["wall"] > 0):
                bad.append(f"{arm} r{rnd}: no wall")
    ref = runs.get(REF, {})
    hashes = {(r.get("hash"), r.get("out")) for r in ref.values()}
    if len(hashes) != 1:
        bad.append(f"{REF}: its outputs differ across rounds: {hashes}")
    for arm in ARMS:
        for rnd, r in runs.get(arm, {}).items():
            ref_r = ref.get(rnd)
            if ref_r and (r.get("hash"), r.get("out")) != (ref_r.get("hash"), ref_r.get("out")):
                bad.append(f"{arm} r{rnd}: output differs from the reference's")
            if "runfed" in arm:
                p = phases_of(r)
                if not p or p.get("runs", 0) == 0:
                    bad.append(f"{arm} r{rnd}: no phases or no team runs")
                if p.get("fallbacks", 1) != 0:
                    bad.append(f"{arm} r{rnd}: {p.get('fallbacks')} fallbacks")
                if arm in RECORD_ARMS and p.get("team_records", 0) == 0:
                    bad.append(f"{arm} r{rnd}: no team block by records")
                if arm not in RECORD_ARMS and p.get("team_records", 1) != 0:
                    bad.append(f"{arm} r{rnd}: a control took records")
    return bad


def ratios(runs, arm, key, rounds):
    return [runs[arm][i][key] / runs[REF][i][key] for i in range(rounds)]


def per_run(runs, arm, rounds):
    """The arm's phases, each a mean over rounds of its total over its team runs (ms a
    run), the counts as totals a decode"""
    tot = defaultdict(list)
    for i in range(rounds):
        p = phases_of(runs[arm][i])
        n = max(p.get("runs", 0), 1)
        for k, v in p.items():
            tot[k].append(v / n / 1e6 if k.endswith("_ns") else v)
    return {k: statistics.fmean(v) for k, v in tot.items()}


def report(args):
    runs, problems = load(args.runs, args.threads)
    bad = problems + validate(runs, args.rounds)
    cells = {}
    if not bad:
        for arm in ARMS:
            w = geo_ci(ratios(runs, arm, "wall", args.rounds))
            c = geo_ci(ratios(runs, arm, "cpu", args.rounds))
            if w is None or c is None:
                bad.append(f"{arm}: an absent or non-finite interval")
            cells[arm] = {"wall": w, "cpu": c}
    raw = {
        arm: [
            {
                "round": i,
                "ref_wall": runs[REF][i]["wall"],
                "ref_cpu": runs[REF][i]["cpu"],
                "wall": runs[arm][i]["wall"],
                "cpu": runs[arm][i]["cpu"],
            }
            for i in range(args.rounds)
            if i in runs.get(arm, {}) and i in runs.get(REF, {})
        ]
        for arm in ARMS
    }
    out = {"threads": args.threads, "rounds": args.rounds, "invalid": bad, "raw": raw}
    if bad:
        print("INVALID")
        for b in bad:
            print(f"  {b}")
        if args.out:
            json.dump(out, open(args.out + ".json", "w"), indent=1)
        return 2
    print(f"VALID: {args.rounds} rounds at {args.threads} threads, against {REF}")
    print(f"{'arm':<26} {'wall ratio [95%]':<26} {'CPU ratio [95%]':<26}")
    for arm in ARMS:
        w, c = cells[arm]["wall"], cells[arm]["cpu"]
        print(f"{arm:<26} {w[0]:.4f} [{w[1]:.4f}, {w[2]:.4f}]   {c[0]:.4f} [{c[1]:.4f}, {c[2]:.4f}]")
    ph = {arm: per_run(runs, arm, args.rounds) for arm in ARMS if "runfed" in arm}
    keys = [
        "lead_ns", "lead_front_ns", "lead_sort_ns", "lead_rle_ns", "lead_small_ns",
        "join_wait_ns", "build_ns", "run_ns", "helper_sort_ns", "helper_fill_ns",
        "helper_fused_ns",
    ]
    print("\nPhases, ms a team run (mean over rounds; the helpers' summed over helpers):")
    print(f"{'phase':<18}" + "".join(f"{a.rsplit('-', 1)[1]:>9}" for a in ph))
    for k in keys:
        print(f"{k:<18}" + "".join(f"{ph[a].get(k, 0):>9.3f}" for a in ph))
    for k in ["runs", "team_records", "team_column", "small_records", "small_column", "leader_late"]:
        print(f"{k:<18}" + "".join(f"{ph[a].get(k, 0):>9.0f}" for a in ph))
    # C30's gross leader release, a block the records' rule takes (the same blocks in
    # every arm, the controls' included): the control's MTF (M) less the arm's emitter (E)
    # and its new leader work (the sort), a block, against both controls; then the same
    # per team run (a block's release times the rule's blocks a run)
    def per_block(arm):
        t = defaultdict(list)
        for i in range(args.rounds):
            p = phases_of(runs[arm][i])
            nb = p.get("rule_blocks", 0)
            if nb:
                t["front"].append(p.get("rule_front_ns", 0) / nb / 1e6)
                t["sort"].append(p.get("rule_sort_ns", 0) / nb / 1e6)
                t["share"].append(nb / max(p.get("runs", 0), 1))
        return {k: statistics.fmean(v) for k, v in t.items()} if t else None

    blocks = {a: per_block(a) for a in ph}
    release = {}
    print("\nGross leader release (M - E - new leader work), ms a block the rule takes:")
    for arm in sorted(RECORD_ARMS):
        a = blocks.get(arm)
        if not a:
            continue
        new = a["front"] + a["sort"]
        release[arm] = {}
        for c in ("ctl", "reg"):
            m = blocks.get(f"mibz2-pipe3-runfed-{c}")
            if m:
                release[arm][c] = m["front"] - new
        r = release[arm]
        print(f"  {arm}: E {a['front']:.3f} + sort {a['sort']:.3f}; release against ctl "
              f"{r.get('ctl', math.nan):+.3f}, against reg {r.get('reg', math.nan):+.3f} "
              f"(the rule's blocks a team run: {a['share']:.2f})")
    for c in ("ctl", "reg"):
        m = blocks.get(f"mibz2-pipe3-runfed-{c}")
        if m:
            print(f"  M ({c}): {m['front']:.3f}")
    for arm in sorted(RECORD_ARMS):
        p = per_run(runs, arm, args.rounds)
        rows, stores = p.get("rows", 0), p.get("stores", 0)
        if rows:
            print(f"  {arm}: W/n {stores / rows:.4f}, records/rows {p.get('records', 0) / rows:.4f}"
                  + (f", lookups {p.get('lookups', 0):.0f}" if arm.endswith("dir") else ""))
    w, c = cells[PRIMARY]["wall"], cells[PRIMARY]["cpu"]
    if w[0] <= POSITIVE_WALL and w[2] < 1 and c[2] <= CPU_CAP:
        reading = "LEAD: the run-fed leader cuts decode wall at the plan"
    elif w[1] > NEGATIVE_WALL:
        reading = "STOP: no 1% wall cut on the primary arm"
    else:
        reading = "INCONCLUSIVE"
    print(f"\nPrimary ({PRIMARY}): {reading}")
    out.update({"cells": cells, "phases": ph, "release": release, "reading": reading})
    if args.out:
        json.dump(out, open(args.out + ".json", "w"), indent=1)
    return 0


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("report")
    r.add_argument("runs")
    r.add_argument("--rounds", type=int, default=15)
    r.add_argument("--threads", type=int, default=9)
    r.add_argument("--out")
    args = ap.parse_args()
    sys.exit(report(args))


if __name__ == "__main__":
    main()
