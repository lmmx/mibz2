"""The run-fed stage diagnostic's admission verifier (docs/journal/2026-11-09-run-fed-stage.md).

examples/run_fed_stage.rs writes a record of its run (RUN_FED_RECORD): its identity, its
corpora's, and every round's raw times, arm by arm. This judges that record on its own:
VALID only if every registered figure is there, whole and finite, at the registered head,
rounds, kernels and corpus; then the readings, recomputed here from the raw times, not
taken from the binary's printout.

    python3 bench/run_fed_stage.py verify RECORD --head SHA [--wikidata FILE] [--unjudged]

Exit 0 for VALID, 1 for INVALID. `--unjudged` labels the readings so: for a run that is
not the registered one (any run before the freeze).
"""

import argparse
import hashlib
import json
import math
import re
import sys

ROUNDS = 15
WIKIDATA_SHA256 = "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
SYNTHETIC = [
    "synthetic text",
    "synthetic random",
    "synthetic one byte",
    "synthetic pattern",
    "synthetic runs",
    "synthetic records",
]
ARMS = ["A column", "B records", "C column+ckpt", "D floor"]
PHASES = ["front", "sort", "build", "fallback"]
ENTRY_PATHS = ["records", "fallback"]
NARMS = 4
HEX40 = re.compile(r"[0-9a-f]{40}")
HEX64 = re.compile(r"[0-9a-f]{64}")
T95 = [
    12.706,
    4.303,
    3.182,
    2.776,
    2.571,
    2.447,
    2.365,
    2.306,
    2.262,
    2.228,
    2.201,
    2.179,
    2.160,
    2.145,
    2.131,
    2.120,
    2.110,
    2.101,
    2.093,
    2.086,
    2.080,
    2.074,
    2.069,
    2.064,
    2.060,
    2.056,
    2.052,
    2.048,
    2.045,
    2.042,
]
# phases[arm][phase] that must be timed (positive) on a corpus with records' blocks;
# A's and C's sort is never timed (exactly 0)
REQUIRED_PHASES = [(0, 0), (0, 2), (1, 0), (1, 1), (1, 2), (2, 0), (2, 2), (3, 0), (3, 1), (3, 2)]
NEVER_TIMED = [(0, 1), (2, 1)]


def _reject_constant(name):
    raise ValueError(f"not a finite number: {name}")


def load(path):
    """The record, refusing NaN and infinities (which JSON has no word for)"""
    with open(path) as f:
        return json.load(f, parse_constant=_reject_constant)


def _is_int(x):
    return isinstance(x, int) and not isinstance(x, bool)


def _is_time(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) and x >= 0


def interval(logs):
    """The geometric mean and its 95% t-interval, from the paired rounds' log ratios"""
    n = len(logs)
    m = sum(logs) / n
    var = sum((x - m) ** 2 for x in logs) / (n - 1)
    h = T95[n - 2] * math.sqrt(var / n)
    return (math.exp(m), math.exp(m - h), math.exp(m + h))


def ratio(rounds, num, den):
    return interval([math.log(num(r) / den(r)) for r in rounds])


def check(rec, head, wikidata_sha256=None, primary=True):
    """The record's problems, each a reason it is INVALID (none: VALID)"""
    p = []
    if not isinstance(rec, dict):
        return ["the record is not an object"]
    if rec.get("schema") != "run_fed_stage/1":
        p.append(f"schema {rec.get('schema')!r}, not run_fed_stage/1")
    if not (isinstance(head, str) and HEX40.fullmatch(head)):
        p.append(f"the registered head {head!r} is not a full 40-character commit")
    rh = rec.get("head")
    if not (isinstance(rh, str) and HEX40.fullmatch(rh)):
        p.append(f"the record's head {rh!r} is not a full 40-character commit")
    elif rh != head:
        p.append(f"the record's head {rh} is not the registered {head}")
    if not (_is_int(rec.get("rounds")) and rec["rounds"] == ROUNDS):
        p.append(f"rounds {rec.get('rounds')!r}, not {ROUNDS}")
    k = rec.get("kernels")
    if not (isinstance(k, dict) and k.get("avx2") is True):
        p.append(f"kernels {k!r}: AVX2 not selected")
    if rec.get("correctness") != "passed":
        p.append(f"correctness {rec.get('correctness')!r}, not passed")
    host = rec.get("host")
    if not (isinstance(host, str) and host and host != "unknown"):
        p.append(f"host {host!r} not recorded")
    aff = rec.get("affinity")
    if not isinstance(aff, str) or (primary and not re.fullmatch(r"\d+", aff)):
        p.append(f"affinity {aff!r}: the primary condition is one copy pinned to one CPU")
    for key, want in (("arms", ARMS), ("phases", PHASES), ("entry_paths", ENTRY_PATHS)):
        if rec.get(key) != want:
            p.append(f"{key} {rec.get(key)!r}, not {want}")
    files = rec.get("files")
    if not isinstance(files, list) or len(files) != 1 + len(SYNTHETIC):
        n = len(files) if isinstance(files, list) else None
        p.append(f"{n} files, not the registered {1 + len(SYNTHETIC)}")
        return p
    rounds = rec.get("rounds") if _is_int(rec.get("rounds")) else ROUNDS
    for i, f in enumerate(files):
        p += check_file(i, f, rounds, wikidata_sha256)
    return p


def check_file(i, f, rounds, wikidata_sha256):
    p = []
    if not isinstance(f, dict):
        return [f"file {i} is not an object"]
    name = f.get("name")
    where = f"file {i} ({name!r})"
    if i == 0:
        if f.get("sha256") != WIKIDATA_SHA256:
            p.append(f"{where}: SHA-256 {f.get('sha256')!r}, not the registered {WIKIDATA_SHA256}")
        if wikidata_sha256 is not None and f.get("sha256") != wikidata_sha256:
            p.append(f"{where}: SHA-256 {f.get('sha256')!r}, not the file's {wikidata_sha256}")
    else:
        if name != SYNTHETIC[i - 1]:
            p.append(f"{where}: not the registered {SYNTHETIC[i - 1]!r}")
        if not (isinstance(f.get("sha256"), str) and HEX64.fullmatch(f["sha256"])):
            p.append(f"{where}: SHA-256 {f.get('sha256')!r} not recorded")
    counts = [
        "bytes",
        "blocks",
        "rows",
        "blocks_runs",
        "rows_runs",
        "records",
        "blocks_fallback",
        "cells",
        "slots",
    ]
    if not all(_is_int(f.get(c)) and f[c] >= 0 for c in counts):
        p.append(f"{where}: path counts missing or not whole: {[f.get(c) for c in counts]}")
        return p
    if f["bytes"] == 0 or f["blocks"] == 0 or f["rows"] == 0:
        p.append(f"{where}: an empty corpus")
    if f["blocks_runs"] + f["blocks_fallback"] != f["blocks"]:
        p.append(f"{where}: {f['blocks_runs']} + {f['blocks_fallback']} blocks, not {f['blocks']}")
    if f["blocks_runs"] > 0 and not (
        0 < f["rows_runs"] <= f["rows"]
        and f["records"] > 0
        and f["cells"] > 0
        and f["slots"] >= f["rows_runs"]
    ):
        p.append(f"{where}: the records' path counts are inconsistent")
    order = f.get("order")
    want = [[(j + r) % NARMS for j in range(NARMS)] for r in range(rounds)]
    if order != want:
        p.append(f"{where}: the arms' order is not the registered rotation")
    has = {0: f["blocks_runs"] > 0, 1: f["blocks_fallback"] > 0}
    entry = f.get("entry")
    if not (isinstance(entry, list) and len(entry) == rounds):
        p.append(
            f"{where}: entry times for {len(entry) if isinstance(entry, list) else None} "
            f"rounds, not {rounds}"
        )
    else:
        for r, row in enumerate(entry):
            if not (
                isinstance(row, list)
                and len(row) == NARMS
                and all(isinstance(a, list) and len(a) == 2 and all(map(_is_time, a)) for a in row)
            ):
                p.append(f"{where}: round {r}'s entry times are not 4 arms x 2 finite times")
                continue
            for a in range(NARMS):
                for path in (0, 1):
                    t = row[a][path]
                    if has[path] and t <= 0:
                        p.append(
                            f"{where}: round {r}, {ARMS[a]}, {ENTRY_PATHS[path]}: no "
                            f"time for a path with blocks"
                        )
                    if not has[path] and t != 0:
                        p.append(
                            f"{where}: round {r}, {ARMS[a]}, {ENTRY_PATHS[path]}: a "
                            f"time for a path with no blocks"
                        )
    phases = f.get("phases")
    if not (isinstance(phases, list) and len(phases) == rounds):
        p.append(
            f"{where}: phase times for "
            f"{len(phases) if isinstance(phases, list) else None} rounds, not {rounds}"
        )
    else:
        for r, row in enumerate(phases):
            if not (
                isinstance(row, list)
                and len(row) == NARMS
                and all(isinstance(a, list) and len(a) == 4 and all(map(_is_time, a)) for a in row)
            ):
                p.append(f"{where}: round {r}'s phase times are not 4 arms x 4 finite times")
                continue
            for a, ph in REQUIRED_PHASES:
                if has[0] and row[a][ph] <= 0:
                    p.append(f"{where}: round {r}, {ARMS[a]} {PHASES[ph]}: not timed")
            for a, ph in NEVER_TIMED + ([] if has[0] else REQUIRED_PHASES):
                if row[a][ph] != 0:
                    p.append(
                        f"{where}: round {r}, {ARMS[a]} {PHASES[ph]}: a time where none is taken"
                    )
            for a in range(NARMS):
                if has[1] != (row[a][3] > 0):
                    p.append(
                        f"{where}: round {r}, {ARMS[a]} fallback: its time and its "
                        f"{f['blocks_fallback']} blocks disagree"
                    )
    return p


def readings(rec):
    """The registered readings, from a VALID record's raw times"""
    files = rec["files"]
    out = {"files": []}
    for f in files:
        e, ph = f["entry"], f["phases"]
        row = {"name": f["name"], "blocks_fallback": f["blocks_fallback"]}
        row["R_stage"] = ratio(e, lambda r: r[1][0] + r[1][1], lambda r: r[0][0] + r[0][1])
        row["R_acc"] = ratio(e, lambda r: r[1][0], lambda r: r[0][0]) if f["blocks_runs"] else None
        row["R_fb"] = (
            ratio(e, lambda r: r[1][1], lambda r: r[0][1]) if f["blocks_fallback"] else None
        )
        if f["blocks_runs"]:
            row["R_phases"] = ratio(ph, lambda r: sum(r[1]), lambda r: sum(r[0]))
            row["R_lead"] = ratio(ph, lambda r: r[1][0] + r[1][1], lambda r: r[2][0])
            row["R_emit"] = ratio(ph, lambda r: r[1][0], lambda r: r[2][0])
            row["R_floor"] = ratio(ph, lambda r: r[3][1], lambda r: r[0][2])
            row["R_warm"] = ratio(ph, lambda r: r[3][2], lambda r: r[0][2])
            fill = sorted(r[1][2] for r in ph)[len(ph) // 2]
            row["M3_nominal_cycles_per_cell"] = fill * rec["ghz"] * 1e9 / f["cells"]
            row["slots_per_row"] = f["slots"] / f["rows_runs"]
        out["files"].append(row)
    w = out["files"][0]
    g, lo, hi = w["R_stage"]
    syn = max(out["files"][1:], key=lambda r: r["R_stage"][0])
    fbs = [r for r in out["files"] if r["R_fb"] is not None]
    fb = max(fbs, key=lambda r: r["R_fb"][0]) if fbs else None
    s_syn = syn["R_stage"][0] <= 1.02
    s_fb = fb is None or fb["R_fb"][0] <= 1.02
    out["screen_synthetic"] = {"worst": syn["name"], "geomean": syn["R_stage"][0], "pass": s_syn}
    out["screen_fallback"] = (
        {"worst": fb["name"], "geomean": fb["R_fb"][0], "pass": s_fb}
        if fb
        else {"worst": None, "geomean": None, "pass": True, "vacuous": True}
    )
    gate = g <= 0.90 and hi < 1.0
    if gate and s_syn and s_fb:
        q1 = "FEASIBLE"
    elif lo > 0.95:
        q1 = "STOP: no saving of 5% or more"
    elif gate:
        q1 = "STOP: a screen failed"
    else:
        q1 = "STOP, INCONCLUSIVE"
    out["Q1"] = q1
    if "R_lead" in w:
        lg, llo, lhi = w["R_lead"]
        out["Q2"] = (
            "LEADER LEAD"
            if lg <= 0.90 and lhi < 1.0
            else "NO LEADER LEAD"
            if llo > 0.95
            else "NO LEADER LEAD, INCONCLUSIVE"
        )
    else:
        out["Q2"] = "n/a"
    return out


NOTES = [
    "Q1's screens are point-estimate screens, not a noninferiority claim: a passing screen "
    "does not show that no corpus or fallback regresses by more than 2%.",
    "STOP and INCONCLUSIVE are operational pauses for this implementation on this runner, "
    "not proof of no effect.",
    "Q2 and M1-M3 are exploratory: estimates and intervals, not mechanism proofs.",
    "M1 is a pre-touch contrast and does not isolate cold write misses; M2's reference is "
    "store-only, not a floor every build obeys (B overstores at cell ends); M3's nominal "
    "cycles are wall time x the sampled clock, not counted cycles.",
]


def fmt(x):
    return "n/a" if x is None else f"{x[0]:.4f} ({x[1]:.4f}, {x[2]:.4f})"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    v = sub.add_parser("verify")
    v.add_argument("record")
    v.add_argument("--head", required=True, help="the registered (frozen) 40-character head")
    v.add_argument("--wikidata", help="the Wikidata file, to hash and compare")
    v.add_argument("--unjudged", action="store_true", help="not the registered run")
    v.add_argument(
        "--contended",
        action="store_true",
        help="a contended copy: its affinity need not be one CPU (never judged)",
    )
    a = ap.parse_args(argv)
    try:
        rec = load(a.record)
    except (OSError, ValueError) as e:
        print(f"INVALID: the record does not load: {e}")
        return 1
    sha = None
    if a.wikidata:
        with open(a.wikidata, "rb") as f:
            sha = hashlib.sha256(f.read()).hexdigest()
    problems = check(rec, a.head, sha, primary=not a.contended)
    if problems:
        print(f"INVALID ({len(problems)} problems):")
        for x in problems:
            print(f"  - {x}")
        return 1
    label = "UNJUDGED" if a.unjudged or a.contended else "JUDGED"
    r = readings(rec)
    print(f"VALID; readings {label}")
    for f in r["files"]:
        print(
            f"  {f['name']}: R_stage {fmt(f['R_stage'])}, R_acc {fmt(f['R_acc'])}, "
            f"R_fb {fmt(f['R_fb'])}"
        )
    w = r["files"][0]
    for key in ("R_phases", "R_lead", "R_emit", "R_floor", "R_warm"):
        print(f"  {key} {fmt(w.get(key))}")
    print(
        f"  M3 {w.get('M3_nominal_cycles_per_cell', float('nan')):.1f} nominal cycles a cell, "
        f"{w.get('slots_per_row', float('nan')):.4f} fill slots a row"
    )
    print(f"  screen, synthetic whole stage: {r['screen_synthetic']}")
    print(f"  screen, fallback (entry point, selector charged): {r['screen_fallback']}")
    print(f"  Q1 ({label}): {r['Q1']}")
    print(f"  Q2 (exploratory): {r['Q2']}")
    for n in NOTES:
        print(f"  note: {n}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
