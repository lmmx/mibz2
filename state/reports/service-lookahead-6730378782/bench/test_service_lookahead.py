"""bench/service_lookahead.py's validity and readings on synthetic records, no decoder run: a
clean fixture is VALID; each of bench/test_service_2x3.py's counterexamples (the Researcher's
HOLD 6729358962) is INVALID on the check that guards it here too; each of the five traces is
checked at its own admission threshold and window, with its prepared count P; and the
registered order's readings come out as registered, speaking of an admission threshold, never
of a queue length.

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
        "svc.backlog.prep_max": 3, "svc.backlog.prep_median": 1, "svc.backlog.finish_max": 2,
        "svc.team.ms.run_wait": 1.0,
    }
    tl.update(kw)
    return tl


def trace_for(name, **kw):
    slots = s.TRACE_SLOTS[name]
    return {
        "meta": {"commit": HEAD},
        "settings": {"threads": [10], "teams": s.TRACE_PLAN, "slots": slots, "admission": s.TRACE_ADMISSION[name]},
        "parity": True,
        "output_bytes": s.REGISTERED_OUT,
        "results": [{"threads": 10, "slots": slots, "timeline": timeline(**kw)}],
    }


def fixture(walls=None, cpus=None):
    """A clean attempt directory's records: every registered cell's 20 rounds, the registered
    digest, the five complete as-registered traces"""
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


A2W8, A3W8, A4W8, A2W12, A4W12 = s.TRACES


def set_trace(name, **kw):
    return lambda rows, traces, files: traces[name].update(kw)


class Validity(unittest.TestCase):
    def assertInvalidOn(self, mutate, needle):
        d = run(mutate)
        self.assertEqual(d["verdict"], "INVALID")
        self.assertEqual(d["exit"], 1)
        self.assertEqual(d["readings"]["overall"], "NONE (the run is invalid)")
        self.assertEqual(d["best_arm"], "NONE (the run is invalid)")
        bad = failed(d)
        self.assertTrue(any(needle in t for t in bad), (needle, bad))

    def test_control_valid(self):
        d = run()
        self.assertEqual(d["verdict"], "VALID", failed(d))
        self.assertEqual(d["exit"], 0)
        self.assertEqual(sorted(d["traces"]), sorted(s.TRACES))
        self.assertEqual(
            sorted((t["admission"], t["window"]) for t in d["traces"].values()), [(2, 8), (2, 12), (3, 8), (4, 8), (4, 12)]
        )
        self.assertEqual({t["p_ceiling"] for t in d["traces"].values()}, {3, 4, 5})

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
        self.assertInvalidOn(lambda r, t, f: r[0].update(arm="mibz2-svc2-a8"), "beyond the registered")

    def test_missing_window_arm(self):
        self.assertInvalidOn(lambda r, t, f: r.__setitem__(slice(None), [x for x in r if x["arm"] != "mibz2-svc2-a4"]), "`mibz2-svc2-a4@10`")

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

    # the five traces, each at its own threshold and window
    def test_trace_missing(self):
        for name in s.TRACES:
            self.assertInvalidOn(lambda r, t, f, n=name: t.update({n: None}), f"`{name}`")

    def test_trace_wrong_window(self):
        # the a2-w12 pass run at the default's slots (a window of 8): not the registered window
        self.assertInvalidOn(
            set_trace(A2W12, results=[{"threads": 10, "slots": 40, "timeline": timeline()}]), f"`{A2W12}`"
        )

    def test_trace_slots_as_default_text(self):
        # no --slots given: occupancy records the default's text, not the registered number
        st = {"threads": [10], "teams": s.TRACE_PLAN, "slots": "SLOTS_PER_THREAD (4) x threads", "admission": s.DEFAULT_ADMISSION}
        self.assertInvalidOn(set_trace(A2W8, settings=st), f"`{A2W8}`")

    def test_trace_wrong_admission(self):
        # the a4 pass run without --admission (the default's threshold), and the a2 pass with one
        st = {"threads": [10], "teams": s.TRACE_PLAN, "slots": 40, "admission": s.DEFAULT_ADMISSION}
        self.assertInvalidOn(set_trace(A4W8, settings=st), f"`{A4W8}`")
        st = {"threads": [10], "teams": s.TRACE_PLAN, "slots": 40, "admission": 2}
        self.assertInvalidOn(set_trace(A2W8, settings=st), f"`{A2W8}`")

    def test_trace_no_prepared_count(self):
        tl = timeline()
        del tl["svc.backlog.prep_median"]
        self.assertInvalidOn(set_trace(A3W8, results=[{"threads": 10, "slots": 40, "timeline": tl}]), f"`{A3W8}`")

    def test_trace_complete_rounds_only(self):
        d = run(set_trace(A4W12, results=[{"threads": 10, "slots": 60, "timeline": {"complete_rounds": 1}}]))
        self.assertEqual(d["verdict"], "INVALID")
        self.assertTrue(any(f"saturation resolved from `{A4W12}`" in t for t in failed(d)), failed(d))

    def test_trace_wrong_plan(self):
        st = {"threads": [10], "teams": "3 of 3, forced", "slots": 40, "admission": s.DEFAULT_ADMISSION}
        self.assertInvalidOn(set_trace(A2W8, settings=st), f"`{A2W8}`")

    def test_trace_wrong_services(self):
        self.assertInvalidOn(
            set_trace(A3W8, results=[{"threads": 10, "slots": 40, "timeline": timeline(**{"svc.n.services": 1})}]), f"`{A3W8}`"
        )

    def test_trace_other_commit(self):
        self.assertInvalidOn(set_trace(A4W8, meta={"commit": "0" * 40}), f"`{A4W8}`")

    def test_trace_no_parity(self):
        self.assertInvalidOn(set_trace(A2W12, parity=False), f"`{A2W12}`")

    def test_trace_metric_out_of_range(self):
        self.assertInvalidOn(
            set_trace(A3W8, results=[{"threads": 10, "slots": 40, "timeline": timeline(**{"svc.util.max": 1.5})}]), f"`{A3W8}`"
        )


def o(name, speed="GAIN", cpu="NULL", g=(1.05, 1.03, 1.07), arm=None):
    arm = arm or {"LA3": "mibz2-svc2-a3@10", "LA4": "mibz2-svc2-a4@10"}[name]
    return {"name": name, "arm": arm, "speed_outcome": speed, "cpu_outcome": cpu, "speed": list(g)}


NULL = (1.0, 0.98, 1.02)
SLOW = (0.95, 0.93, 0.97)
STARVED = {"saturated": "not saturated", "prep": "PREP STARVATION", "finish": "no finish backlog"}
FED = dict(STARVED, prep="no prep starvation")


class Readings(unittest.TestCase):
    # the threshold is never read as a queue length, nor the cause shown
    BANNED = ("QUEUE LENGTH", "queue length", "THE LOOKAHEAD BINDS", "cause")

    def assertNoBanned(self, text):
        for b in self.BANNED:
            self.assertNotIn(b, text)

    def test_threshold_binds(self):
        rd = s.readings({"LA3": o("LA3"), "LA4": o("LA4", "NULL", g=NULL)})
        self.assertTrue(rd["overall"].startswith("THE ADMISSION THRESHOLD BINDS SVC2@10 AT 2"), rd["overall"])
        self.assertIn("a gain at 3", rd["overall"])
        self.assertIn("doesn't cap the queue", rd["overall"])
        self.assertIn("threshold + 1", rd["overall"])
        self.assertNoBanned(rd["overall"])

    def test_higher_threshold_trade(self):
        rd = s.readings({"LA3": o("LA3", cpu="CPU COST"), "LA4": o("LA4", "GAIN")})
        self.assertTrue(rd["overall"].startswith("A HIGHER ADMISSION THRESHOLD IS FASTER FOR A CPU COST"), rd["overall"])

    def test_mixed(self):
        rd = s.readings({"LA3": o("LA3"), "LA4": o("LA4", "REGRESSION", g=SLOW)})
        self.assertTrue(rd["overall"].startswith("INCONCLUSIVE (registered step 2: MIXED"), rd["overall"])

    def test_inconclusive(self):
        rd = s.readings({"LA3": o("LA3", "NULL", g=NULL), "LA4": o("LA4", "INCONCLUSIVE (the interval spans gain and harm)", g=(1.0, 0.96, 1.04))})
        self.assertTrue(rd["overall"].startswith("INCONCLUSIVE (registered step 3"), rd["overall"])

    def test_slower(self):
        rd = s.readings({"LA3": o("LA3", "REGRESSION", g=SLOW), "LA4": o("LA4", "REGRESSION", g=SLOW)})
        self.assertTrue(rd["overall"].startswith("A HIGHER ADMISSION THRESHOLD IS SLOWER"), rd["overall"])

    def test_null(self):
        rd = s.readings({"LA3": o("LA3", "NULL", g=NULL), "LA4": o("LA4", "NULL", g=NULL)})
        self.assertTrue(rd["overall"].startswith("NO GAIN DETECTED FROM A HIGHER ADMISSION THRESHOLD"), rd["overall"])
        self.assertIn("a small effect is not excluded", rd["overall"])
        self.assertNoBanned(rd["overall"])

    def test_mechanism(self):
        cells = list(s.ADMISSION)
        t = s.mechanism(
            {c: (STARVED if c == "mibz2-svc2@10" else FED) for c in cells}, "THE ADMISSION THRESHOLD BINDS SVC2@10 AT 2 (...)"
        )
        self.assertTrue(t.startswith("PREP STARVATION AT THRESHOLD 2, RELIEVED"), t)
        self.assertIn("not shown to be it", t)
        t = s.mechanism({c: STARVED for c in cells}, "NO GAIN DETECTED FROM A HIGHER ADMISSION THRESHOLD (...)")
        self.assertTrue(t.startswith("PREP STARVATION PERSISTS AT EVERY THRESHOLD"), t)
        self.assertIn("untested here", t)
        modes = {c: STARVED for c in cells}
        modes["mibz2-svc2-a4-w12@10"] = dict(STARVED, saturated="SATURATED")
        t = s.mechanism(modes, "NO GAIN DETECTED FROM A HIGHER ADMISSION THRESHOLD (...)")
        self.assertIn("SATURATED at a4 w12", t)

    def test_full_run_threshold_binds(self):
        # svc2 at threshold 3 5% and at 4 4% faster than at 2, at the same CPU
        walls = {"mibz2-svc2-a3": 0.95, "mibz2-svc2-a4": 0.96}
        fed = {"svc.team.idle_prep_share": 0.04, "svc.backlog.prep_max": 5, "svc.backlog.prep_median": 3}

        def m(rows, traces, files):
            for n in (A3W8, A4W8):
                traces[n]["results"][0]["timeline"].update(fed)
        d = run(m, walls=walls)
        self.assertEqual(d["verdict"], "VALID", failed(d))
        j = {c["name"]: c for c in d["judged"]}
        self.assertEqual(j["LA3"]["speed_outcome"], "GAIN")
        self.assertEqual(j["LA4"]["speed_outcome"], "GAIN")
        self.assertEqual(j["LA3"]["admission"], [3, 2])
        self.assertEqual(j["LA3"]["windows"], [8, 8])
        self.assertTrue(d["readings"]["overall"].startswith("THE ADMISSION THRESHOLD BINDS"), d["readings"]["overall"])
        self.assertTrue(d["mechanism"].startswith("PREP STARVATION AT THRESHOLD 2, RELIEVED"), d["mechanism"])
        self.assertIn("a3 w8", d["best_arm"])
        self.assertIn("Reported, never judged", d["best_arm"])
        self.assertIn("optimistic", d["best_arm"])
        self.assertEqual(d["traces"][A4W8]["p_max"], 5)
        self.assertEqual(d["traces"][A4W8]["p_ceiling"], 5)
        self.assertIn("5 / 3.0 (5)", d["md"])
        self.assertNoBanned(d["readings"]["overall"] + d["mechanism"])
        sec = {c["name"]: c for c in d["secondary"]}
        self.assertEqual(set(sec), {"W12", "J", "J4", "D-a2w8", "D-a3w8", "D-a4w8", "D-a2w12", "D-a4w12", "R1"})

    def test_full_run_null(self):
        d = run()
        self.assertEqual(d["verdict"], "VALID", failed(d))
        self.assertTrue(d["readings"]["overall"].startswith("NO GAIN DETECTED"), d["readings"]["overall"])
        self.assertTrue(d["mechanism"].startswith("PREP STARVATION PERSISTS"), d["mechanism"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
