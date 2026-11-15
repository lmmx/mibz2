"""bench/cells_phase.py's ledgers, statistics, amended rule and validity on synthetic readings:
each verdict where the registration (LOG 6884716103, amended 6884895257 and 6884975709) puts
it, the rule's boundaries, and each defect the validity guards against leaves no verdict; the
baseline arm is reported and never gates.

    python3 bench/test_cells_phase.py
"""

import copy
import json
import math
import random
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import cells_phase as cp  # noqa: E402

OUT = 209_700_196
LOOPS = 2
CORES = [[c, c + 10] for c in range(10)]

# (B_ind, B_wr, B_rest) at 1 and 4 workers, demand-DRAM bytes per output byte
SCENES = {
    "WRITER": ((1.0, 1.0, 0.5), (1.1, 4.0, 0.55)),
    "SHARED WORKING SET": ((1.0, 1.0, 1.0), (4.0, 4.0, 4.0)),
    "INDUCTION": ((1.0, 0.3, 0.3), (2.0, 0.33, 0.36)),
    "MIXED": ((1.0, 1.0, 1.0), (1.1, 1.3, 1.2)),
}


def win(values, windows, mux=0):
    return {"values": values, "enabled": 1000, "running": 1000 - mux, "windows": windows,
            "mux": mux}  # fmt: skip


def decode(p, bs, case=None, k=0):
    """one decode's record at pass p, with phase counts for B = bs (bytes a line per output
    byte), each event the same split"""
    n = len(cp.EVENT_NAMES[p])
    ind, wr, rest = (round(b * OUT / 64) for b in bs)
    ph = {
        "pass": p,
        "events": cp.EVENT_NAMES[p],
        "modes": "user+kernel",
        "threads": 4,
        "blocks": 431,
        "whole": win([ind + wr + rest] * n, 431),
        "whole_in_induction": win([ind] * n, 200),
        "whole_in_writer": win([wr] * n, cp.BLOCKS),
        "induction": win([ind] * n, 200),
        "writer": win([wr] * n, cp.BLOCKS),
        "empty": win([3] * n, 200 + cp.BLOCKS),
        "empty_ns": 1000,
        "errors": [],
    }
    d = {"err": "", "out": OUT, "hash": "0123456789abcdef", "wall": 0.5, "dose_mb": 0,
         "dose_kind": "stream", "dose_blocks": cp.BLOCKS, "phase": ph}  # fmt: skip
    if k == 1:
        if case == "failed_decode":
            d["err"] = "crc"
        if case == "hash":
            d["hash"] = "fedcba9876543210"
        if case == "dose_blocks":
            d["dose_blocks"] = 195
        if case == "dose_mb":
            d["dose_mb"] = 8
        if case == "writer_scopes":
            ph["writer"]["windows"] = 195
        if case == "empty_scopes":
            ph["empty"]["windows"] -= 1
        if case == "mux":
            ph["induction"]["mux"] = 1
        if case == "running":
            ph["whole"]["running"] = 999
        if case == "pass":
            ph["pass"] = "B2"
        if case == "modes":
            ph["modes"] = "user"
        if case == "counter_errors":
            ph["errors"] = ["ioctl 0x2400: errno 22"]
        if case == "phase_sum":
            ph["induction"]["values"][0] = int(ph["induction"]["values"][0] * 1.02) + int(
                0.02 * (ind + wr + rest)
            )
        if case == "no_counters":
            del d["phase"]
    return d


def reading(p, t, rep, slot, bs, explained, case, rng, noise):
    """one reading: its IMC ledger and its decodes"""
    f = math.exp(rng.gauss(0, noise))
    bs = tuple(b * f for b in bs)
    arm = cp.BASE_ARM if p == cp.BASE else cp.ARM
    cpus = [1] if t == 1 else [c[0] for c in CORES[:t]]
    rec = {"pass": p, "arm": arm, "rep": rep, "threads": t, "slot": slot, "loops": LOOPS,
           "cpus": cpus, "cores": [c % 10 for c in cpus],
           "siblings": sorted(c + 10 for c in cpus)}  # fmt: skip
    span = 2.5
    idle = {"span": 3.0, "reads": 0.03e9, "writes": 0.02e9}
    core = sum(bs)
    reads = core / explained * OUT * LOOPS
    writes = 1.0 * OUT * LOOPS
    rec.update(error=None, wall=span + 0.05, span=span, idle=idle,
               reads=reads + idle["reads"] * span, writes=writes + idle["writes"] * span)  # fmt: skip
    if p == cp.BASE:
        rec["decodes"] = [
            {
                "err": "",
                "out": OUT,
                "hash": "0123456789abcdef",
                "wall": 0.5,
                "dose_mb": 0,
                "dose_blocks": 0,
                "phase": None,
            }  # fmt: skip
            for _ in range(LOOPS)
        ]
    else:
        rec["decodes"] = [decode(p, bs, case, k) for k in range(LOOPS)]
    if case == "no_idle":
        rec["idle"] = None
    if case == "idle_high":
        rate = (rec["reads"] + rec["writes"]) / span
        idle.update(reads=0.06 * rate, writes=0.04 * rate)
    if case == "net_negative":
        rec["writes"] = idle["writes"] * span * 0.9
    if case == "failed_reading":
        rec.update(error="bench-imc: perf stat failed", decodes=[])
    if case == "short_chain":
        rec["decodes"] = rec["decodes"][:-1]
    if case == "placement":
        rec["siblings"].append(rec["cpus"][0])
    if case == "same_core" and t > 1:
        rec["cores"][1] = rec["cores"][0]
    if case == "base_failed" and p == cp.BASE:
        rec.update(error="perf stat failed", decodes=[])
    return rec


def readings(scene="WRITER", explained=0.6, case=None, seed=11, noise=0.01, threads=None,
             reps=None):  # fmt: skip
    threads = threads or cp.REGISTERED_THREADS
    reps = cp.REGISTERED_REPS if reps is None else reps
    b1, b4 = SCENES[scene]
    rng = random.Random(seed)
    rows = [{"pass": "meta", "threads": threads, "reps": reps, "loops": LOOPS, "pair_seconds": 3,
             "placement": {str(t): [1] if t == 1 else list(range(t)) for t in threads}}]  # fmt: skip
    hi = max(threads)
    for rep in range(reps):
        for t in cp.order(threads, rep):
            frac = (t - min(threads)) / (hi - min(threads))
            bs = tuple(
                math.exp(math.log(a) + frac * (math.log(b) - math.log(a))) for a, b in zip(b1, b4)
            )
            for slot, p in enumerate(cp.order(cp.ITEMS, rep)):
                c = case if (p, t, rep) in TARGETS.get(case, {(p, t, rep)}) else None
                if case == "missing" and (p, t, rep) == ("A1", 4, 3):
                    continue
                rows.append(reading(p, t, rep, slot, bs, explained, c, rng, noise))
                if case == "duplicate" and (p, t, rep) == ("B1", 1, 0):
                    rows.append(copy.deepcopy(rows[-1]))
    if case == "extra":
        rows.append(dict(rows[1], threads=2))
    return rows


# (where each defect is put: one reading of the Cells arm, or the baseline's)
TARGETS = {
    k: {("A1", 4, 2)}
    for k in [
        "failed_decode",
        "hash",
        "dose_blocks",
        "dose_mb",
        "writer_scopes",
        "empty_scopes",
        "mux",
        "running",
        "pass",
        "modes",
        "counter_errors",
        "phase_sum",
        "no_counters",
        "no_idle",
        "idle_high",
        "net_negative",
        "failed_reading",
        "short_chain",
        "placement",
        "same_core",
    ]  # fmt: skip
}
TARGETS["base_failed"] = {(cp.BASE, 4, 5)}


def run(rows, *extra):
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / "readings.jsonl"
        path.write_text("".join(json.dumps(r) + "\n" for r in rows))
        stem = str(Path(d) / "phase")
        code = cp.main(["analyse", str(path), "--out", stem, *extra])
        res = json.loads(Path(stem + ".json").read_text())
        res["_md"] = Path(stem + ".md").read_text()
        return code, res


def test_rule_boundaries():
    s = {p: (0.2, 0.15, 0.25) for p in cp.PHASES}
    u = {p: (1.2, 1.1, 1.3) for p in cp.PHASES}
    flat = (1.0, 0.9, 1.1)
    assert cp.verdict(0.2999, s, u, flat) == "UNATTRIBUTED-DOMINATED, HELD"
    assert cp.verdict(0.30, s, u, flat) == "MIXED"
    # writer: the share and the uplift ratio, each interval above its line
    sw = dict(s, writer=(0.6, 0.41, 0.8))
    uw = dict(u, writer=(3.0, 2.5, 3.5), induction=(1.0, 0.9, 1.1))
    assert cp.verdict(0.5, sw, uw, (3.0, 1.51, 4.0)) == "WRITER"
    assert cp.verdict(0.5, sw, uw, (3.0, 1.50, 4.0)) != "WRITER"
    assert cp.verdict(0.5, dict(sw, writer=(0.6, 0.40, 0.8)), uw, (3.0, 2.0, 4.0)) != "WRITER"
    assert cp.verdict(0.5, dict(sw, writer=(0.39, 0.38, 0.8)), uw, (3.0, 2.0, 4.0)) != "WRITER"
    assert cp.verdict(0.5, sw, dict(uw, writer=(1.49, 1.4, 1.6)), (3.0, 2.0, 4.0)) != "WRITER"
    # shared: every U_p at least 3, every lower bound above 2
    us = {p: (3.0, 2.0001, 4.0) for p in cp.PHASES}
    assert cp.verdict(0.5, s, us, flat) == "SHARED WORKING SET"
    assert cp.verdict(0.5, s, dict(us, rest=(2.999, 2.5, 3.5)), flat) != "SHARED WORKING SET"
    assert cp.verdict(0.5, s, dict(us, rest=(3.5, 2.0, 4.0)), flat) != "SHARED WORKING SET"
    # induction: the share at least 0.50, U_ind at least U_wr
    si = dict(s, induction=(0.50, 0.45, 0.55))
    ui = dict(u, induction=(1.5, 1.4, 1.6), writer=(1.5, 1.4, 1.6))
    assert cp.verdict(0.5, si, ui, flat) == "INDUCTION"
    assert cp.verdict(0.5, dict(si, induction=(0.4999, 0.4, 0.6)), ui, flat) == "MIXED"
    assert cp.verdict(0.5, si, dict(ui, writer=(1.51, 1.4, 1.6)), flat) == "MIXED"


def test_verdicts():
    code, r = run(readings(explained=0.2))
    assert code == 0 and r["verdict"] == "UNATTRIBUTED-DOMINATED, HELD", r["verdict_line"]
    for scene in SCENES:
        code, r = run(readings(scene))
        assert code == 0 and r["verdict"] == scene, (scene, r["verdict_line"])
    assert r["registered"]


def test_ledgers():
    code, r = run(readings("WRITER", explained=0.6, noise=0.0))
    assert code == 0
    b1, b4 = SCENES["WRITER"]
    for p, x1, x4 in zip(cp.PHASES, b1, b4):
        assert abs(r["B"][p]["1"][0] - x1) < 1e-3 and abs(r["B"][p]["4"][0] - x4) < 1e-3
        assert abs(r["U"][p][0] - x4 / x1) < 1e-3
    assert abs(r["s"]["writer"][0] - 4.0 / sum(b4)) < 1e-6
    assert abs(r["explained"]["4"][0] - 0.6) < 1e-6
    # (the residual, IMC reads less the demand-DRAM reads: unattributed)
    m = r["imc"]["4"]
    assert abs(m["unattributed"][0] - sum(b4) * (1 / 0.6 - 1)) < 1e-3
    assert abs(m["writes"][0] - 1.0) < 1e-6
    assert r["overhead"]["1"][0] > 0 and r["base"]["4"][0] > 0
    md = r["_md"]
    for s in ("unattributed", "observed co-running uplift", "CML058", "baseline arm",
              "approximate", "scanner", "slots()", "b(1)"):  # fmt: skip
        assert s in md, s


def test_validity():
    for case in sorted(TARGETS) + ["missing", "duplicate", "extra"]:
        if case == "base_failed":
            continue
        code, r = run(readings(case=case))
        assert code == 1 and r["verdict"] == "INVALID", (case, r["verdict_line"])
        assert r["problems"], case


def test_phase_sum_on_a_diagnostic_pass():
    rows = readings()
    for rec in rows:
        if (rec.get("pass"), rec.get("threads"), rec.get("rep")) == ("B2", 1, 7):
            ph = rec["decodes"][0]["phase"]
            ph["writer"]["values"][0] += max(1, ph["whole"]["values"][0] // 20)
    code, r = run(rows)
    assert code == 1 and any("phase sum" in p for p in r["problems"]), r["problems"]


def test_baseline_never_gates():
    code, r = run(readings(case="base_failed"))
    assert code == 0 and r["verdict"] == "WRITER", r["verdict_line"]
    assert r["base_problems"]


def test_other_grid_is_marked():
    code, r = run(readings(threads=[1, 2], reps=3), "--threads", "1,2", "--reps", "3")
    assert code == 0 and not r["registered"] and (r["lo"], r["hi"]) == (1, 2)
    assert "not the registered grid" in r["verdict_line"]


def test_placement_and_topology():
    cores = [[0, 10], [1, 11], [2, 12], [3, 13], [4, 14]]
    assert cp.placement(cores, 4) == [0, 1, 2, 3]
    assert cp.placement(cores, 1) == [1]
    assert cp.placement([[0, 1]], 1) == [0]
    assert cp.core_of(cores, 12) == 2


def test_chain_sets_the_pass_and_pins():
    c = cp.chain("ab", "w.bz2", 4, 3, cp.ARM, "A2", [0, 1, 2, 3])
    assert c[:4] == ["env", "MIBZ2_PHASE_PASS=A2", "taskset", "-c"] and c[4] == "0,1,2,3"
    assert c[-4:-1] == ["child", cp.ARM, "4"]
    b = cp.chain("ab", "w.bz2", 1, 3, cp.BASE_ARM, cp.BASE, [1])
    assert b[1] == "MIBZ2_PHASE_PASS=" and cp.BASE_ARM in b


if __name__ == "__main__":
    for name, f in list(globals().items()):
        if name.startswith("test_") and callable(f):
            f()
            print(f"ok {name}")
