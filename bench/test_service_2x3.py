"""bench/service_2x3.py's validity and readings on synthetic records, no decoder run: a clean
fixture is VALID, and each counterexample of the Researcher's HOLD 6729358962 (its probe.py)
is INVALID on the check that guards it; the existing rejection controls still reject; the
illustrative walls (default 1, plain2 1.10, pipe2 0.90, svc2 0.95) never read as FOR SVC2 or as
the service threads causing it.

    python3 bench/test_service_2x3.py        (or python3 -m pytest bench/test_service_2x3.py)
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
import service_2x3 as s  # noqa: E402

warnings.simplefilter("ignore", ResourceWarning)

HEAD = "3e4477d035e96310eb4ac3d4014e01e13ffaa33c"
# the Researcher's illustrative walls (not timing evidence), with the other cells at 1
ILLUSTRATIVE = {
    "mibz2": 1.0,
    "mibz2-team2x3": 1.10,
    "mibz2-pipe2x3": 0.90,
    "mibz2-svc2": 0.95,
    "mibz2-team3": 1.0,
    "mibz2-team3-twin": 1.0,
}


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


def fixture(walls=None):
    """A clean attempt directory's records: every registered cell's 20 rounds, the registered
    digest, a complete as-registered trace"""
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
            rows.append(dict(
                arm=arm, threads=int(budget), round=r, wall=wall, cpu=6.0 * jitter, rss_kb=64000,
                hash="0123456789abcdef", out=s.REGISTERED_OUT, err="", corpus="wikidata",
                thread_cpu=" ".join(names),
            ))
    trace = {
        "meta": {"commit": HEAD},
        "settings": {"threads": [10], "teams": s.TRACE_PLAN},
        "parity": True,
        "output_bytes": s.REGISTERED_OUT,
        "results": [{"threads": 10, "timeline": timeline()}],
    }
    files = {
        "commit.txt": HEAD + "\n",
        "git-status.txt": "",
        "lscpu.txt": "Socket(s): 1\nCore(s) per socket: 10\nL3 cache: 20 MiB (1 instance)\n",
        "corpus.sha256": s.REGISTERED_CORPUS_SHA256 + "  /x/target/service-2x3/wikidata.bz2\n",
    }
    return rows, trace, files


def run(mutate=None, walls=None):
    rows, trace, files = fixture(walls)
    if mutate:
        mutate(rows, trace, files)
    w = Path(tempfile.mkdtemp(prefix="service2x3-test-"))
    try:
        for name, body in files.items():
            if body is not None:
                (w / name).write_text(body)
        (w / "runs.jsonl").write_text("\n".join(map(json.dumps, rows)) + "\n")
        (w / (s.TRACE + ".json")).write_text(json.dumps(trace))
        a = types.SimpleNamespace(runs=str(w / "runs.jsonl"), occupancy=[str(w / (s.TRACE + ".json"))], out=str(w / "result"))
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
    return lambda rows, trace, files: [r.update(kw) for r in rows]


def drop(key):
    return lambda rows, trace, files: [r.pop(key) for r in rows]


def set_trace(**kw):
    return lambda rows, trace, files: trace.update(kw)


class Validity(unittest.TestCase):
    def assertInvalidOn(self, mutate, needle):
        d = run(mutate)
        self.assertEqual(d["verdict"], "INVALID")
        self.assertEqual(d["exit"], 1)
        self.assertEqual(d["readings"]["overall"], "NONE (the run is invalid)")
        bad = failed(d)
        self.assertTrue(any(needle in t for t in bad), (needle, bad))

    def test_control_valid(self):
        d = run()
        self.assertEqual(d["verdict"], "VALID", failed(d))
        self.assertNotEqual(d["saturation_flag"], "UNRESOLVED")

    # the HOLD's counterexamples (probe.py), each now INVALID on its own check
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
        # a CORPUS= override: its digest is not the registered one
        self.assertInvalidOn(lambda r, t, f: f.update({"corpus.sha256": "b" * 64 + "  other.bz2\n"}), "`corpus.sha256`")

    def test_trace_complete_rounds_only(self):
        d = run(set_trace(results=[{"threads": 10, "timeline": {"complete_rounds": 1}}]))
        self.assertEqual(d["verdict"], "INVALID")
        self.assertTrue(any("saturation resolved" in t for t in failed(d)), failed(d))
        self.assertTrue(any("the trace" in t for t in failed(d)), failed(d))

    def test_trace_wrong_configuration(self):
        self.assertInvalidOn(
            lambda r, t, f: t.update(results=[{"threads": 1, "svc": 1, "timeline": timeline(**{"svc.n.services": 1})}]),
            "the trace",
        )

    def test_trace_wrong_plan(self):
        self.assertInvalidOn(set_trace(settings={"threads": [10], "teams": "3 of 3, forced"}), "the trace")

    def test_trace_other_commit(self):
        self.assertInvalidOn(set_trace(meta={"commit": "0" * 40}), "the trace")

    def test_trace_no_parity(self):
        self.assertInvalidOn(set_trace(parity=False), "the trace")

    def test_trace_metric_out_of_range(self):
        self.assertInvalidOn(set_trace(results=[{"threads": 10, "timeline": timeline(**{"svc.util.max": 1.5})}]), "the trace")

    def test_hashes_differ(self):
        def m(rows, trace, files):
            rows[5]["hash"] = "fedcba9876543210"
        self.assertInvalidOn(m, "same bytes")

    # deleted fields: INVALID with a reason, no longer an exception
    def test_deleted_hash(self):
        self.assertInvalidOn(drop("hash"), "record usable")

    def test_deleted_size(self):
        self.assertInvalidOn(drop("out"), "record usable")

    def test_deleted_wall(self):
        self.assertInvalidOn(drop("wall"), "record usable")

    # the existing rejection controls
    def test_one_round(self):
        self.assertInvalidOn(lambda r, t, f: r.__setitem__(slice(None), [x for x in r if x["round"] == 0]), "registered 20 rounds")

    def test_duplicate_round(self):
        self.assertInvalidOn(lambda r, t, f: r.append(dict(r[0])), "once each")

    def test_wrong_threads(self):
        self.assertInvalidOn(lambda r, t, f: r[0].update(thread_cpu="mibz2-0=.1"), "once each")

    def test_unregistered_arm(self):
        self.assertInvalidOn(lambda r, t, f: r[0].update(arm="mibz2@/other/base"), "beyond the registered")

    def test_dry(self):
        self.assertInvalidOn(lambda r, t, f: f.update({"DRY-RUN.txt": "dry"}), "`DRY-RUN.txt`")

    def test_not_frozen(self):
        self.assertInvalidOn(lambda r, t, f: f.update({"NOT-FROZEN.txt": "changed"}), "`NOT-FROZEN.txt`")


def o(speed="GAIN", cpu="NULL", g=None):
    d = {"speed_outcome": speed, "cpu_outcome": cpu}
    if g is not None:
        d["speed"] = list(g)
    return d


class Readings(unittest.TestCase):
    BANNED = ("FOR SVC2", "cause it", "CAUSE IT", "EARN THEIR KEEP", "NOT THE SERVICE THREADS")

    def assertNoBanned(self, text):
        for b in self.BANNED:
            self.assertNotIn(b, text)

    def test_illustrative_walls(self):
        d = run(walls=ILLUSTRATIVE)
        self.assertEqual(d["verdict"], "VALID", failed(d))
        j = {c["name"]: c for c in d["judged"]}
        self.assertEqual(j["P1"]["speed_outcome"], "GAIN")
        self.assertEqual(j["C1"]["speed_outcome"], "GAIN")
        self.assertEqual(j["P2"]["speed_outcome"], "GAIN")
        rd = d["readings"]
        self.assertIn("PIPE2X3@10 FASTER THAN SVC2@10", rd["versus_pipe2"])
        self.assertTrue(rd["overall"].startswith("REGISTERED STEP 6 MET, BUT SVC2@10 NOT SHOWN BETTER"), rd["overall"])
        self.assertNoBanned(json.dumps(rd) + d["md"])
        # the decomposition, on the common paired rounds' geometric means
        for key in ("speed", "cpu"):
            g = d["decomposition"][key]
            self.assertEqual(g["rounds"], 20)
            self.assertAlmostEqual(g["P1"], g["S1xP2"], places=12)
            self.assertAlmostEqual(g["P1"], g["C1xS2"], places=12)
        g = d["decomposition"]["speed"]
        self.assertAlmostEqual(g["P1"], 1 / 0.95, places=2)
        self.assertAlmostEqual(g["C1"], 1.10 / 0.95, places=2)
        self.assertAlmostEqual(g["P2"], 1 / 0.90, places=2)
        self.assertAlmostEqual(g["S1"], 0.90 / 0.95, places=2)
        self.assertIn("P1 = S1 × P2 = C1 × S2", d["md"])

    def test_without_s1_never_for_svc2(self):
        # the probe's call: P1, C1 and P2 all gains, S1 not given
        rd = s.readings({"P1": o(), "C1": o(), "P2": o()})
        self.assertNoBanned(json.dumps(rd))
        rd = s.readings({"P1": o(cpu="INCONCLUSIVE (the interval spans gain and harm)"), "C1": o(), "P2": o("REGRESSION")})
        self.assertNoBanned(json.dumps(rd))
        self.assertIn("no CPU cost detected", rd["product"])
        self.assertIn("not shown to cost nothing", rd["product"])

    def test_for_svc2_only_when_s1_shows_it(self):
        rd = s.readings({"P1": o(), "C1": o(), "P2": o("REGRESSION"), "S1": o(g=(1.08, 1.05, 1.11))})
        self.assertTrue(rd["overall"].startswith("FOR SVC2@10"), rd["overall"])
        self.assertNotIn("cause it", rd["overall"])
        rd = s.readings({"P1": o(), "C1": o(), "P2": o(), "S1": o(g=(1.01, 0.99, 1.03))})
        self.assertNoBanned(json.dumps(rd))

    def test_null_is_bounded(self):
        rd = s.readings({"P1": o(), "C1": o("NULL"), "P2": o(), "S1": o(g=(0.95, 0.94, 0.96))})
        self.assertIn("a small effect is not excluded", rd["cause"])
        self.assertIn("a small effect is not excluded", rd["overall"])
        self.assertNoBanned(json.dumps(rd))
        rd = s.readings({"P1": o("NULL"), "C1": o(), "P2": o()})
        self.assertIn("a small effect is not excluded", rd["product"])

    def test_saturation_is_a_hypothesis(self):
        sat = {"saturated": "SATURATED", "prep": "PREP STARVATION", "finish": "no finish backlog"}
        for c1 in ("NULL", "REGRESSION"):
            t = s.counterexample(sat, o(c1))
            self.assertTrue(t.startswith("SATURATION BINDS"))
            self.assertIn("hypothesis", t)
            self.assertIn("load imbalance", t)
            self.assertNotIn("not the idea", t)
        t = s.counterexample({"saturated": "not saturated", "prep": "PREP STARVATION", "finish": "x"}, o())
        self.assertIn("not shown to be it", t)

    def test_saturated_trace_flag_kept(self):
        tl = timeline(**{"svc.util.max": 0.95, "svc.util.0": 0.95})
        d = run(set_trace(results=[{"threads": 10, "timeline": tl}]), walls=ILLUSTRATIVE)
        self.assertEqual(d["verdict"], "VALID", failed(d))
        self.assertEqual(d["saturation_flag"], "SATURATED")
        self.assertTrue(d["saturation"].startswith("SATURATED, A GAIN NONETHELESS"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
