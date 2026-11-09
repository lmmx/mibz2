"""bench/service_lookahead.py's validity and readings on synthetic records, no decoder run: a
clean fixture is VALID; each of bench/test_service_2x3.py's counterexamples (the Researcher's
HOLD 6729358962) is INVALID on the check that guards it here too; each of the three traces is
checked at its own window; and the registered order's readings come out as registered, never
naming the window as the prepared queue's lookahead.

    python3 bench/test_service_lookahead.py        (or python3 -m pytest bench/test_service_lookahead.py)
"""

import contextlib
import io
import json
import math
import shutil
import sys
import tempfile
import types
import unittest
import warnings
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import service_lookahead as s  # noqa: E402

warnings.simplefilter("ignore", ResourceWarning)

HEAD = "3e201246cf1266d4b2a984cb19eb9fdba5b9835d"


def timeline(**kw):
    tl = {
        "complete_rounds": 6, "blocks": 431, "svc.n.services": 2,
        "svc.util.max": 0.7, "svc.util.0": 0.7, "svc.util.1": 0.65,
        "svc.team.idle_prep_share": 0.15, "svc.team.idle_back_share": 0.01,
        "svc.queue.prep.median_ms": 0.2, "svc.queue.prep.p90_ms": 0.5,
        "svc.queue.finish.median_ms": 0.1, "svc.queue.finish.p90_ms": 0.2, "svc.queue.job.median_ms": 0.1,
        "svc.backlog.prep_mean": 0.5, "svc.backlog.finish_mean": 0.1,
        "svc.backlog.prep_max": 3, "svc.backlog.finish_max": 2, "svc.team.ms.run_wait": 1.0,
    }
    tl.update(kw)
    return tl


def trace_for(name, **kw):
    slots = s.TRACE_SLOTS[name]
    return {
        "meta": {"commit": HEAD},
        "settings": {"threads": [10], "teams": s.TRACE_PLAN, "slots": slots},
        "parity": True,
        "output_bytes": s.REGISTERED_OUT,
        "results": [{"threads": 10, "slots": slots, "timeline": timeline(**kw)}],
    }


def fixture(walls=None, cpus=None):
    """A clean attempt directory's records: every registered cell's 20 rounds, the registered
    digest, the three complete as-registered traces"""
    rows = []
    for r in range(s.REGISTERED_ROUNDS):
        for k, (c, (leaders, helpers, svc)) in enumerate(s.CELLS.items()):
            arm, budget = c.split("@")
            names = [f"mibz2-{i}=.1" for i in range(leaders)]
            names += [f"mibz2-coop-{i}=.1" for i in range(helpers)]
            names += [f"mibz2-svc-{i}=.1" for i in range(svc)]
            # a little noise, so the intervals are not points
            jitter = 1 + 0.002 * (((r * 3 + k) % 5) - 2)
            wall = (walls or {}).get(arm, 1.0) * jitter
            cpu = (cpus or {}).get(arm, 6.0) * jitter
            rows.append(dict(
                arm=arm, threads=int(budget), round=r, wall=wall, cpu=cpu, rss_kb=64000,
                hash="0123456789abcdef", out=s.REGISTERED_OUT, err="", corpus="wikidata",
                thread_cpu=" ".join(names),
            ))
    traces = {name: trace_for(name) for name in s.TRACES}
    files = {
        "commit.txt": HEAD + "\n",
        "git-status.txt": "",
        "lscpu.txt": "Socket(s): 1\nCore(s) per socket: 10\nL3 cache: 20 MiB (1 instance)\n",
        "corpus.sha256": s.REGISTERED_CORPUS_SHA256 + "  /x/target/service-lookahead/wikidata.bz2\n",
    }
    return rows, traces, files


def run(mutate=None, walls=None, cpus=None):
    rows, traces, files = fixture(walls, cpus)
    if mutate:
        mutate(rows, traces, files)
    w = Path(tempfile.mkdtemp(prefix="service-lookahead-test-"))
    try:
        for name, body in files.items():
            if body is not None:
                (w / name).write_text(body)
        (w / "runs.jsonl").write_text("\n".join(map(json.dumps, rows)) + "\n")
        occ = []
        for name, t in traces.items():
            if t is not None:
                (w / (name + ".json")).write_text(json.dumps(t))
                occ.append(str(w / (name + ".json")))
        a = types.SimpleNamespace(runs=str(w / "runs.jsonl"), occupancy=occ, out=str(w / "result"))
        with patch.object(s.subprocess, "run", return_value=types.SimpleNamespace(stdout=HEAD + "\n")), \
                contextlib.redirect_stdout(io.StringIO()):
            status = s.report(a)
        d = json.loads((w / "result.json").read_text())
        d["exit"] = status
        d["md"] = (w / "result.md").read_text()
        return d
    finally:
        shutil.rmtree(w)


def failed(d):
    return [t for t, ok in d["checks"] if not ok]


def setall(**kw):
    return lambda rows, traces, files: [r.update(kw) for r in rows]


def drop(key):
    return lambda rows, traces, files: [r.pop(key) for r in rows]


W8, W12, W16 = s.TRACES


def set_trace(name, **kw):
    return lambda rows, traces, files: traces[name].update(kw)


class Validity(unittest.TestCase):
    def assertInvalidOn(self, mutate, needle):
        d = run(mutate)
        self.assertEqual(d["verdict"], "INVALID")
        self.assertEqual(d["exit"], 1)
        self.assertEqual(d["readings"]["overall"], "NONE (the run is invalid)")
        self.assertEqual(d["best_window"], "NONE (the run is invalid)")
        bad = failed(d)
        self.assertTrue(any(needle in t for t in bad), (needle, bad))

    def test_control_valid(self):
        d = run()
        self.assertEqual(d["verdict"], "VALID", failed(d))
        self.assertEqual(d["exit"], 0)
        self.assertEqual(sorted(d["traces"]), sorted(s.TRACES))
        self.assertEqual({t["window"] for t in d["traces"].values()}, {8, 12, 16})

    # bench/test_service_2x3.py's counterexamples, each INVALID on its own check here too
    def test_null_output(self):
        self.assertInvalidOn(setall(hash=None, out=None), "record usable")

    def test_empty_hash_zero_output(self):
        self.assertInvalidOn(setall(hash="", out=0), "record usable")

    def test_negative_wall_and_cpu(self):
        self.assertInvalidOn(setall(wall=-1.0, cpu=-6.0), "record usable")

    def test_nonfinite_wall(self):
        self.assertInvalidOn(setall(wall=math.inf), "record usable")

    def test_wrong_corpus(self):
        self.assertInvalidOn(setall(corpus="different-workload"), "record usable")

    def test_absent_corpus_digest(self):
        self.assertInvalidOn(lambda r, t, f: f.update({"corpus.sha256": None}), "`corpus.sha256`")

    def test_other_corpus_digest(self):
        self.assertInvalidOn(lambda r, t, f: f.update({"corpus.sha256": "b" * 64 + "  other.bz2\n"}), "`corpus.sha256`")

    def test_hashes_differ(self):
        def m(rows, traces, files):
            rows[5]["hash"] = "fedcba9876543210"
        self.assertInvalidOn(m, "same bytes")

    def test_deleted_hash(self):
        self.assertInvalidOn(drop("hash"), "record usable")

    def test_deleted_size(self):
        self.assertInvalidOn(drop("out"), "record usable")

    def test_deleted_wall(self):
        self.assertInvalidOn(drop("wall"), "record usable")

    def test_one_round(self):
        self.assertInvalidOn(lambda r, t, f: r.__setitem__(slice(None), [x for x in r if x["round"] == 0]), "registered 20 rounds")

    def test_duplicate_round(self):
        self.assertInvalidOn(lambda r, t, f: r.append(dict(r[0])), "once each")

    def test_wrong_threads(self):
        self.assertInvalidOn(lambda r, t, f: r[0].update(thread_cpu="mibz2-0=.1"), "once each")

    def test_unregistered_arm(self):
        self.assertInvalidOn(lambda r, t, f: r[0].update(arm="mibz2-svc2-w24"), "beyond the registered")

    def test_missing_window_arm(self):
        self.assertInvalidOn(lambda r, t, f: r.__setitem__(slice(None), [x for x in r if x["arm"] != "mibz2-svc2-w16"]), "`mibz2-svc2-w16@10`")

    def test_dry(self):
        self.assertInvalidOn(lambda r, t, f: f.update({"DRY-RUN.txt": "dry"}), "`DRY-RUN.txt`")

    def test_not_frozen(self):
        self.assertInvalidOn(lambda r, t, f: f.update({"NOT-FROZEN.txt": "changed"}), "`NOT-FROZEN.txt`")

    def test_other_head(self):
        self.assertInvalidOn(lambda r, t, f: f.update({"commit.txt": "0" * 40}), "`commit.txt`")

    def test_other_topology(self):
        self.assertInvalidOn(
            lambda r, t, f: f.update({"lscpu.txt": "Socket(s): 1\nCore(s) per socket: 8\nL3 cache: 16 MiB\n"}),
            "ten-core topology",
        )

    def test_noise_control(self):
        # the twin 5% slower than its original: N1 outside 1 +/- 0.03
        def m(rows, traces, files):
            for x in rows:
                if x["arm"] == "mibz2-team3-twin":
                    x["wall"] *= 1.05
        d = run(m)
        self.assertEqual(d["verdict"], "INVALID")
        self.assertIn("noise control N1", d["md"])

    # the three traces, each at its own window
    def test_trace_missing(self):
        for name in s.TRACES:
            self.assertInvalidOn(lambda r, t, f, n=name: t.update({n: None}), f"`{name}`")

    def test_trace_wrong_window(self):
        # the w12 pass run at the default's slots (a window of 8): not the registered window
        self.assertInvalidOn(
            set_trace(W12, results=[{"threads": 10, "slots": 40, "timeline": timeline()}]), f"`{W12}`"
        )

    def test_trace_slots_as_default_text(self):
        # no --slots given: occupancy records the default's text, not the registered number
        self.assertInvalidOn(
            set_trace(W8, settings={"threads": [10], "teams": s.TRACE_PLAN, "slots": "SLOTS_PER_THREAD (4) x threads"}),
            f"`{W8}`",
        )

    def test_trace_complete_rounds_only(self):
        d = run(set_trace(W16, results=[{"threads": 10, "slots": 80, "timeline": {"complete_rounds": 1}}]))
        self.assertEqual(d["verdict"], "INVALID")
        self.assertTrue(any(f"saturation resolved from `{W16}`" in t for t in failed(d)), failed(d))

    def test_trace_wrong_plan(self):
        self.assertInvalidOn(set_trace(W8, settings={"threads": [10], "teams": "3 of 3, forced", "slots": 40}), f"`{W8}`")

    def test_trace_wrong_services(self):
        self.assertInvalidOn(
            set_trace(W12, results=[{"threads": 10, "slots": 60, "timeline": timeline(**{"svc.n.services": 1})}]), f"`{W12}`"
        )

    def test_trace_other_commit(self):
        self.assertInvalidOn(set_trace(W16, meta={"commit": "0" * 40}), f"`{W16}`")

    def test_trace_no_parity(self):
        self.assertInvalidOn(set_trace(W8, parity=False), f"`{W8}`")

    def test_trace_metric_out_of_range(self):
        self.assertInvalidOn(
            set_trace(W12, results=[{"threads": 10, "slots": 60, "timeline": timeline(**{"svc.util.max": 1.5})}]), f"`{W12}`"
        )


def o(name, speed="GAIN", cpu="NULL", g=(1.05, 1.03, 1.07), arm=None):
    arm = arm or {"L1": "mibz2-svc2-w12@10", "L2": "mibz2-svc2-w16@10"}[name]
    return {"name": name, "arm": arm, "speed_outcome": speed, "cpu_outcome": cpu, "speed": list(g)}


NULL = (1.0, 0.98, 1.02)
SLOW = (0.95, 0.93, 0.97)


class Readings(unittest.TestCase):
    # the window is never named as the lookahead, nor the cause shown
    BANNED = ("THE LOOKAHEAD BINDS", "LOOKAHEAD BINDS SVC2", "the window is the lookahead", "cause")

    def assertNoBanned(self, text):
        for b in self.BANNED:
            self.assertNotIn(b, text)

    def test_window_binds(self):
        rd = s.readings({"L1": o("L1"), "L2": o("L2", "NULL", g=NULL)})
        self.assertTrue(rd["overall"].startswith("THE WINDOW BINDS SVC2@10 AT 8"), rd["overall"])
        self.assertIn("a gain at 12", rd["overall"])
        self.assertIn("not the prepared queue's own lookahead", rd["overall"])
        self.assertNoBanned(rd["overall"])

    def test_wider_window_trade(self):
        rd = s.readings({"L1": o("L1", cpu="CPU COST"), "L2": o("L2", "GAIN")})
        self.assertTrue(rd["overall"].startswith("A WIDER WINDOW IS FASTER FOR A CPU COST"), rd["overall"])

    def test_mixed(self):
        rd = s.readings({"L1": o("L1"), "L2": o("L2", "REGRESSION", g=SLOW)})
        self.assertTrue(rd["overall"].startswith("INCONCLUSIVE (registered step 2: MIXED"), rd["overall"])

    def test_inconclusive(self):
        rd = s.readings({"L1": o("L1", "NULL", g=NULL), "L2": o("L2", "INCONCLUSIVE (the interval spans gain and harm)", g=(1.0, 0.96, 1.04))})
        self.assertTrue(rd["overall"].startswith("INCONCLUSIVE (registered step 3"), rd["overall"])

    def test_slower(self):
        rd = s.readings({"L1": o("L1", "REGRESSION", g=SLOW), "L2": o("L2", "REGRESSION", g=SLOW)})
        self.assertTrue(rd["overall"].startswith("A WIDER WINDOW IS SLOWER"), rd["overall"])

    def test_null_leaves_lookahead_untested(self):
        rd = s.readings({"L1": o("L1", "NULL", g=NULL), "L2": o("L2", "NULL", g=NULL)})
        self.assertTrue(rd["overall"].startswith("NO WINDOW GAIN DETECTED BEYOND 8"), rd["overall"])
        self.assertIn("a small effect is not excluded", rd["overall"])
        self.assertIn("untested by this run", rd["overall"])
        self.assertNoBanned(rd["overall"])

    def test_mechanism(self):
        starved = {"saturated": "not saturated", "prep": "PREP STARVATION", "finish": "no finish backlog"}
        fed = dict(starved, prep="no prep starvation")
        t = s.mechanism({8: starved, 12: fed, 16: fed}, "THE WINDOW BINDS SVC2@10 AT 8 (...)")
        self.assertTrue(t.startswith("PREP STARVATION AT 8, RELIEVED"), t)
        self.assertIn("not shown to be it", t)
        t = s.mechanism({8: starved, 12: starved, 16: starved}, "NO WINDOW GAIN DETECTED BEYOND 8 (...)")
        self.assertTrue(t.startswith("PREP STARVATION PERSISTS AT EVERY WINDOW"), t)
        self.assertIn("untested here", t)
        sat = dict(starved, saturated="SATURATED")
        t = s.mechanism({8: starved, 12: starved, 16: sat}, "NO WINDOW GAIN DETECTED BEYOND 8 (...)")
        self.assertIn("SATURATED at window 16", t)

    def test_full_run_window_binds(self):
        # svc2 at 12 5% and at 16 4% faster than at 8, at the same CPU
        walls = {"mibz2-svc2-w12": 0.95, "mibz2-svc2-w16": 0.96}
        fed = {"svc.team.idle_prep_share": 0.04}

        def m(rows, traces, files):
            for n in (W12, W16):
                traces[n]["results"][0]["timeline"].update(fed)
        d = run(m, walls=walls)
        self.assertEqual(d["verdict"], "VALID", failed(d))
        j = {c["name"]: c for c in d["judged"]}
        self.assertEqual(j["L1"]["speed_outcome"], "GAIN")
        self.assertEqual(j["L2"]["speed_outcome"], "GAIN")
        self.assertEqual(j["L1"]["windows"], [12, 8])
        self.assertTrue(d["readings"]["overall"].startswith("THE WINDOW BINDS"), d["readings"]["overall"])
        self.assertTrue(d["mechanism"].startswith("PREP STARVATION AT 8, RELIEVED"), d["mechanism"])
        self.assertIn("window 12", d["best_window"])
        self.assertIn("Reported, never judged", d["best_window"])
        self.assertIn("optimistic", d["best_window"])
        self.assertNoBanned(d["readings"]["overall"] + d["mechanism"])

    def test_full_run_null(self):
        d = run()
        self.assertEqual(d["verdict"], "VALID", failed(d))
        self.assertTrue(d["readings"]["overall"].startswith("NO WINDOW GAIN DETECTED"), d["readings"]["overall"])
        self.assertTrue(d["mechanism"].startswith("PREP STARVATION PERSISTS"), d["mechanism"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
