"""bench/phase_gates.py's figures, validity, exposure and readouts on synthetic readings: each
readout where the registration (#145, LOG 6893071988) puts it, the rules' boundaries (5%, 15%
and -10%; the 1% calibration and the 10% exposure), each defect the validity guards against
leaving no readout, and the collection driven end to end with stand-ins for ab and perf.

    python3 bench/test_phase_gates.py
"""

import copy
import json
import math
import os
import random
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import phase_gates as pg  # noqa: E402

OUT = 209_700_196
LOOPS = 2
BLOCKS = 431
INDUCTIONS = 200
CORES = [[c, c + 10] for c in range(10)]
# K's figures: active CPU ns per output byte by phase, IMC B per output byte, decode wall s
BASE = {"induction": 1.0, "writer": 0.5, "rest": 1.5, "imc": 12.0, "wall": 0.16}
EXPOSED = 0.30


def win(values, windows, mux=0):
    return {"values": values, "enabled": 1000, "running": 1000 - mux, "windows": windows,
            "mux": mux}  # fmt: skip


def decode(arm, f, case=None, k=0):
    """one decode's record for `arm` at figures `f` (fractions of K's), its capped gates
    delayed at `f['exposure']`"""
    ind, wr, rest = (round(BASE[p] * f.get(p, f["active"]) * OUT) for p in pg.PHASES)
    ph = {
        "pass": pg.PASS, "events": ["task-clock", "page-faults"], "modes": "user+kernel",
        "threads": 4, "blocks": BLOCKS,
        "whole": win([ind + wr + rest, 9], BLOCKS),
        "whole_in_induction": win([ind, 1], INDUCTIONS),
        "whole_in_writer": win([wr, 1], pg.BLOCKS),
        "induction": win([ind, 1], INDUCTIONS),
        "writer": win([wr, 1], pg.BLOCKS),
        "empty": win([3, 0], INDUCTIONS + pg.BLOCKS),
        "empty_ns": 1000, "errors": [],
    }  # fmt: skip
    d = {"err": "", "out": OUT, "hash": "0123456789abcdef", "wall": BASE["wall"] * f["wall"],
         "phase": ph}  # fmt: skip
    caps = pg.ARMS[arm]
    if caps is not None:
        entries = {"induction": INDUCTIONS, "writer": pg.BLOCKS, "blocks": BLOCKS}
        gates = {}
        for n, cap in zip(pg.GATES, caps):
            e = entries[n]
            exp = f["exposure"].get(n, EXPOSED) if n in pg.CAPPED.get(arm, []) else 0.0
            w = round(exp * e)
            gates[n] = {"capacity": cap, "entries": e, "waited": w, "wait_ns": 1000 * w,
                        "max_held": min(cap, 4)}  # fmt: skip
        d["gates"] = gates
    if k == 1:
        if case == "failed_decode":
            d["err"] = "crc"
        if case == "hash":
            d["hash"] = "fedcba9876543210"
        if case == "writer_scopes":
            ph["writer"]["windows"] = 195
        if case == "pass":
            ph["pass"] = "A1"
        if case == "counter_errors":
            ph["errors"] = ["perf_event_open task-clock: errno 13"]
        if case == "mux":
            ph["whole"]["running"] = 999
        if case == "no_counters":
            del d["phase"]
        if case == "gates_in_ungated":
            d["gates"] = {"induction": None, "writer": None, "blocks": None}
        if case == "gate_missing":
            d["gates"]["writer"] = None
        if case == "no_gates":
            d.pop("gates", None)
        if case == "gate_capacity":
            d["gates"]["induction"]["capacity"] = 2
        if case == "gate_entries":
            d["gates"]["writer"]["entries"] -= 1
        if case == "gate_held":
            d["gates"]["induction"]["max_held"] = 2
        if case == "gate_waits":
            d["gates"]["blocks"]["waited"] = BLOCKS + 1
    return d


def reading(arm, rep, slot, f, case, rng, noise):
    g = math.exp(rng.gauss(0, noise))
    f = dict(f, active=f["active"] * g, imc=f["imc"] * g, wall=f["wall"] * g)
    for p in pg.PHASES:
        if p in f:
            f[p] *= g
    cpus = [c[0] for c in CORES[:4]]
    rec = {"pass": "reading", "arm": arm, "caps": pg.ARMS[arm], "rep": rep, "threads": 4,
           "slot": slot, "loops": LOOPS, "cpus": cpus, "cores": [c % 10 for c in cpus],
           "siblings": sorted(c + 10 for c in cpus)}  # fmt: skip
    span = 2.5
    idle = {"span": 3.0, "reads": 0.03e9, "writes": 0.02e9}
    traffic = BASE["imc"] * f["imc"] * OUT * LOOPS
    rec.update(error=None, wall=span + 0.05, span=span, idle=idle,
               reads=0.6 * traffic + idle["reads"] * span,
               writes=0.4 * traffic + idle["writes"] * span)  # fmt: skip
    rec["decodes"] = [decode(arm, f, case, k) for k in range(LOOPS)]
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
    if case == "same_core":
        rec["cores"][1] = rec["cores"][0]
    return rec


def figs(**over):
    """every arm's figures as fractions of K's (1: as K), with `over` {arm: {key: value}}"""
    out = {}
    for arm in pg.ITEMS:
        f = {"active": 1.0, "imc": 1.0, "wall": 1.0, "exposure": {}}
        f.update(over.get(arm, {}))
        out[arm] = f
    return out


def readings(f=None, case=None, seed=7, noise=0.003, reps=None, target=("B", 2)):
    f = f or figs()
    reps = pg.REGISTERED_REPS if reps is None else reps
    rng = random.Random(seed)
    rows = [{"pass": "meta", "arms": pg.ARMS, "threads": 4, "reps": reps, "loops": LOOPS,
             "pair_seconds": 3, "placement": [0, 1, 2, 3], "cpu_model": "stand-in"}]  # fmt: skip
    for rep in range(reps):
        for slot, arm in enumerate(pg.order(pg.ITEMS, rep)):
            if case == "missing" and (arm, rep) == ("D", 3):
                continue
            c = case if (arm, rep) == target else None
            rows.append(reading(arm, rep, slot, f[arm], c, rng, noise))
            if case == "duplicate" and (arm, rep) == ("C", 0):
                rows.append(copy.deepcopy(rows[-1]))
    if case == "extra":
        rows.append(dict(rows[1], arm="E4"))
    return rows


def run(rows, *extra):
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / "readings.jsonl"
        path.write_text("".join(json.dumps(r) + "\n" for r in rows))
        stem = str(Path(d) / "gates")
        code = pg.main(["analyse", str(path), "--out", stem, *extra])
        res = json.loads(Path(stem + ".json").read_text())
        res["_md"] = Path(stem + ".md").read_text()
        return code, res


def lines(r):
    return "\n".join(r["verdict_lines"])


# ----------------------------------------------------------------------------- the rules


def test_phase_rule_boundaries():
    fall = (-0.05, -0.08, -0.02)
    down = (-0.04, -0.06, -0.01)
    assert pg.meets_phase(fall, down)
    assert not pg.meets_phase((-0.0499, -0.08, -0.02), down)
    assert pg.meets_phase((-0.05, -0.08, -1e-9), down)
    assert not pg.meets_phase((-0.05, -0.10, 0.0), down)
    assert not pg.meets_phase(fall, (-0.04, -0.08, 0.0))
    assert not pg.meets_phase(fall, (0.01, -0.01, 0.03))


def test_e2_rule_boundaries():
    assert pg.meets_e2((-0.15, -0.20, -0.1001))
    assert not pg.meets_e2((-0.1499, -0.20, -0.12))
    assert not pg.meets_e2((-0.15, -0.20, -0.10))
    assert not pg.meets_e2((-0.30, -0.50, -0.05))


def test_calibration_and_exposure_boundaries():
    assert pg.calibrated((0.01, 0.0, 0.02)) and pg.calibrated((-0.01, -0.02, 0.0))
    assert not pg.calibrated((0.0101, 0.0, 0.02)) and not pg.calibrated((-0.0101, -0.02, 0.0))
    assert pg.exposed(0.10) and not pg.exposed(0.0999)


# ------------------------------------------------------------------------------ readouts


def test_null_screen():
    code, r = run(readings())
    assert code == 0 and r["registered"], r["problems"]
    assert r["verdict"] == {"induction": "NOT MET", "writer": "NOT MET", "e2": "NOT MET"}
    t = lines(r)
    assert t.count(pg.NULL_RULE) == 3 and pg.WALL_RULE in t and pg.SCOPE in t
    assert "not the registered grid" not in t


def test_induction_readout():
    code, r = run(readings(figs(B={"active": 0.90, "imc": 0.92})))
    assert code == 0 and r["verdict"]["induction"] == "MET", lines(r)
    assert r["readouts"]["induction"]["met"] == ["B"]
    assert f"READOUT induction (B or D): MET by B: {pg.LINES['induction']}" in lines(r)
    assert r["verdict"]["writer"] == "NOT MET" and r["verdict"]["e2"] == "NOT MET"
    d = r["delta"]["B"]
    assert abs(d["active"][0] + 0.10) < 0.005 and abs(d["imc"][0] + 0.08) < 0.005


def test_induction_needs_traffic_to_fall():
    code, r = run(readings(figs(B={"active": 0.90, "imc": 1.0}, D={"active": 0.90, "imc": 1.01})))
    assert code == 0 and r["verdict"]["induction"] == "NOT MET", lines(r)


def test_writer_readout_by_d():
    code, r = run(readings(figs(D={"active": 0.93, "imc": 0.95})))
    assert code == 0, r["problems"]
    assert r["verdict"]["writer"] == "MET" and r["readouts"]["writer"]["met"] == ["D"]
    assert r["verdict"]["induction"] == "MET" and r["readouts"]["induction"]["met"] == ["D"]
    assert pg.LINES["writer"] in lines(r)


def test_d_read_by_each_cap_exposure():
    # (D meets the rule, but its writer cap delayed only 5%: read for induction, not writer)
    f = figs(D={"active": 0.90, "imc": 0.90, "exposure": {"writer": 0.05}},
             C={"exposure": {"writer": 0.02}})  # fmt: skip
    code, r = run(readings(f))
    assert code == 0, r["problems"]
    assert r["verdict"]["induction"] == "MET" and r["verdict"]["writer"] == "NOT EXPOSED"
    assert "READOUT writer (C or D): NOT EXPOSED (C 2.0%, D 5.1%" in lines(r), lines(r)
    assert pg.NULL_RULE in lines(r).split("READOUT writer")[1]


def test_e2_readout():
    code, r = run(readings(figs(E2={"active": 0.80, "wall": 1.4})))
    assert code == 0 and r["verdict"]["e2"] == "MET", lines(r)
    assert f"READOUT e2 (E2): MET by E2: {pg.LINES['e2']}" in lines(r)
    assert "0.196 s for two teams against 0.162 s independent" in lines(r)
    assert "E2" in r["interference"]
    # (E3 alone has no readout: the curve is reported)
    code, r = run(readings(figs(E3={"active": 0.70})))
    assert code == 0 and r["verdict"]["e2"] == "NOT MET"
    assert "E3" in r["_md"] and "occupancy curve" in r["_md"]


def test_e2_short_of_its_bound():
    code, r = run(readings(figs(E2={"active": 0.88})))
    assert code == 0 and r["verdict"]["e2"] == "NOT MET", lines(r)


def test_exposure_boundary_end_to_end():
    # (INDUCTIONS 200 entries a decode: 20 waited is exactly 10%, 19 below it)
    for w, want in [(0.10, "MET"), (0.095, "NOT EXPOSED")]:
        f = figs(B={"active": 0.9, "imc": 0.9, "exposure": {"induction": w}},
                 D={"exposure": {"induction": 0.0}})  # fmt: skip
        code, r = run(readings(f))
        assert code == 0 and r["verdict"]["induction"] == want, (w, lines(r))


def test_calibration_end_to_end():
    code, r = run(readings(figs(U={"active": 1 / 1.0095}), noise=0.0))
    assert code == 0 and r["verdict"] != "INVALID", r["problems"]
    code, r = run(readings(figs(U={"active": 1 / 1.0105}), noise=0.0))
    assert code == 1 and r["verdict"] == "INVALID"
    assert any(p.startswith("calibration") for p in r["problems"]), r["problems"]
    assert "VERDICT: INVALID" in lines(r) and pg.WALL_RULE in lines(r)


def test_interference_is_flagged():
    code, r = run(readings(figs(C={"active": 0.9, "wall": 1.2})))
    assert code == 0 and r["interference"] == ["C"]
    assert "Interference (active CPU lower, wall higher): C" in r["_md"]


def test_figures_recovered():
    code, r = run(readings(noise=0.0))
    assert code == 0
    m = r["mean"]["K"]
    for p in pg.PHASES:
        assert abs(m[p] - BASE[p]) < 1e-6, (p, m[p])
    assert abs(m["active"] - sum(BASE[p] for p in pg.PHASES)) < 1e-6
    assert abs(m["imc"] - BASE["imc"]) < 1e-6 and abs(m["wall"] - BASE["wall"]) < 1e-9
    assert abs(r["exposure"]["B"]["induction"] - EXPOSED) < 1e-9
    assert r["exposure"]["E2"] == {"blocks": round(EXPOSED * BLOCKS) / BLOCKS}


# ------------------------------------------------------------------------------ validity

DEFECTS = {
    "failed_decode": ("B", 2), "hash": ("C", 4), "writer_scopes": ("K", 1), "pass": ("U", 5),
    "counter_errors": ("E2", 0), "mux": ("D", 7), "no_counters": ("E3", 9),
    "gates_in_ungated": ("U", 3), "gate_missing": ("K", 2), "no_gates": ("E2", 6),
    "gate_capacity": ("B", 8), "gate_entries": ("C", 1), "gate_held": ("D", 5),
    "gate_waits": ("E3", 4), "failed_reading": ("K", 6), "short_chain": ("B", 1),
    "no_idle": ("U", 0), "idle_high": ("C", 9), "net_negative": ("D", 2),
    "placement": ("E2", 3), "same_core": ("K", 8), "missing": None, "duplicate": None,
    "extra": None,
}  # fmt: skip


def test_validity():
    for case, target in DEFECTS.items():
        code, r = run(readings(case=case, target=target or ("B", 2)))
        assert code == 1 and r["verdict"] == "INVALID", (case, lines(r))
        assert r["problems"], case
        assert "VERDICT: INVALID" in lines(r), case


def test_other_grid_is_marked():
    code, r = run(readings(reps=3), "--reps", "3")
    assert code == 0 and not r["registered"]
    assert all("not the registered grid" in x for x in r["verdict_lines"][:3])
    assert "NOT the registered grid" in r["_md"]


def test_chain_sets_the_arm_and_pins():
    c = pg.chain("ab", "w.bz2", 3, "B", [0, 1, 2, 3])
    assert c[:5] == ["env", "MIBZ2_PHASE_PASS=SW", "MIBZ2_GATE_INDUCTION=1",
                     "MIBZ2_GATE_WRITER=4", "MIBZ2_GATE_BLOCKS=4"]  # fmt: skip
    assert c[5:8] == ["taskset", "-c", "0,1,2,3"] and c[-4:-1] == ["child", pg.ARM, "4"]
    u = pg.chain("ab0", "w.bz2", 3, "U", [0, 1, 2, 3])
    assert u[2:5] == ["MIBZ2_GATE_INDUCTION=", "MIBZ2_GATE_WRITER=", "MIBZ2_GATE_BLOCKS="]
    assert pg.ARMS["E2"] == (4, 4, 2) and pg.ARMS["E3"] == (4, 4, 3) and pg.ARMS["K"] == (4, 4, 4)


# ------------------------------------------------------------------- collect, stand-ins

STAND_IN = r"""#!/usr/bin/env python3
import json, os, sys
gated = os.path.basename(sys.argv[0]) == "ab"
assert sys.argv[1:4] == ["child", "mibz2-workers", "4"], sys.argv
caps = [os.environ.get(v, "") for v in
        ("MIBZ2_GATE_INDUCTION", "MIBZ2_GATE_WRITER", "MIBZ2_GATE_BLOCKS")]
def win(v, n):
    return {"values": [v, 0], "enabled": 10, "running": 10, "windows": n, "mux": 0}
ph = None
if os.environ.get("MIBZ2_PHASE_PASS") == "SW":
    ph = {"pass": "SW", "events": ["t", "p"], "modes": "user+kernel", "threads": 4,
          "blocks": 431, "whole": win(3000, 431), "whole_in_induction": win(1000, 198),
          "whole_in_writer": win(500, 196), "induction": win(1000, 198),
          "writer": win(500, 196), "empty": win(3, 394), "empty_ns": 9, "errors": []}
rec = {"wall": 0.01, "out": 1000, "hash": "ab", "err": "", "phase": ph}
if gated:
    entries = [198, 196, 431]
    if any(caps):
        rec["gates"] = {n: {"capacity": int(c), "entries": e, "waited": e // 2 if int(c) < 4
                            else 0, "wait_ns": 5, "max_held": min(int(c), 4)}
                        for n, c, e in zip(("induction", "writer", "blocks"), caps, entries)}
    else:
        rec["gates"] = None
print(json.dumps(rec))
"""


def test_collect_with_stand_ins():
    """collect end to end, then analyse: stand-ins for both builds' `ab` (a JSON record a
    decode, as `ab child` prints, its gates as the environment sets them) and for imc.perf
    (runs the command without its taskset; counts made up), so the roster, rotation, the
    arms' environments, idle pairing and records are what analyse reads"""
    with tempfile.TemporaryDirectory() as d:
        ab0, ab = Path(d) / "ab0", Path(d) / "ab"
        for p in (ab0, ab):
            p.write_text(STAND_IN)
            p.chmod(0o755)
        seen = []

        def perf(events, cmd, system, interval=None):
            assert system and events == pg.IMC
            if cmd[0] == "env":
                i = cmd.index("taskset")
                seen.append((cmd[1:i], cmd[i + 2]))
                cmd = cmd[:i] + cmd[i + 3 :]
                r = subprocess.run(cmd, capture_output=True, text=True)
                assert r.returncode == 0, r.stderr
                n = int(cmd[cmd.index("sh") + 3])
                row = {e: 1e9 * n for e in events}
            else:
                r = subprocess.CompletedProcess(cmd, 0, "", "")
                row = {e: 1e6 for e in events}
            row["_run_ns"] = 1e9
            return 1.0, r.stdout, {None: row}

        saved = pg.imc.perf, pg.imc.run, pg.cells_phase.topology
        pg.imc.perf = perf
        pg.imc.run = lambda argv, check=False: subprocess.CompletedProcess(argv, 0, "", "")
        pg.cells_phase.topology = lambda root=None: [[c, c + 4] for c in range(4)]
        try:
            out = str(Path(d) / "readings.jsonl")
            argv = ["collect", "--ungated", str(ab0), "--gated", str(ab), "--corpus", os.devnull]
            assert (
                pg.main([*argv, "--reps", "3", "--loops", "2", "--settle", "0", "--out", out]) == 0
            )
            argv[0] = "preflight"
            assert pg.main(argv) == 0
        finally:
            pg.imc.perf, pg.imc.run, pg.cells_phase.topology = saved
        recs = [json.loads(x) for x in Path(out).read_text().splitlines()]
        assert recs[0]["pass"] == "meta" and recs[0]["loops"] == 2
        assert recs[0]["placement"] == [0, 1, 2, 3]
        got = [r["arm"] for r in recs[1:]]
        assert got == pg.ITEMS + pg.ITEMS[1:] + pg.ITEMS[:1] + pg.ITEMS[2:] + pg.ITEMS[:2]
        assert all(r["idle"] and len(r["decodes"]) == 2 for r in recs[1:])
        assert {cpus for _, cpus in seen} == {"0,1,2,3"}
        envs = {tuple(e) for e, _ in seen}
        assert ("MIBZ2_PHASE_PASS=SW", "MIBZ2_GATE_INDUCTION=4", "MIBZ2_GATE_WRITER=4",
                "MIBZ2_GATE_BLOCKS=2") in envs and len(envs) == len(pg.ITEMS)  # fmt: skip
        u = next(r for r in recs[1:] if r["arm"] == "U")
        assert all("gates" not in x for x in u["decodes"])
        code, r = run(recs, "--reps", "3")
        assert code == 0, r["problems"]
        assert r["verdict"] == {"induction": "NOT MET", "writer": "NOT MET", "e2": "NOT MET"}
        assert abs(r["exposure"]["B"]["induction"] - 99 / 198) < 1e-9


if __name__ == "__main__":
    for name, f in list(globals().items()):
        if name.startswith("test_") and callable(f):
            f()
            print(f"ok {name}")
