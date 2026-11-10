"""bench/tg_ni_pilot.py and bench/tg_ni_stamp.py: the registration's §2.8 design pilot.

The pilot is admitted as the judge admits a run, but labelled `pilot`. It shows no cell's ratio
or bound. Its menu sees what it is for:
- dependence (an AR(1) series' autocorrelation, and the batch-means standard error's growth
  over the iid one);
- an order effect;
- a trend.

The stamper times every row as it lands, and restarts on a truncation. The gate takes a pilot
as it takes a smoke run: never evidence.

    python3 bench/test_tg_ni_pilot.py
"""

import json
import math
import random
import shutil
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import service_tg as tg  # noqa: E402
import test_service_tg as T  # noqa: E402
import tg_ni_gate as gate  # noqa: E402
import tg_ni_pilot as pilot  # noqa: E402
import tg_ni_stamp as stamp  # noqa: E402

N = 120  # (rounds in these fixtures; the pilot's own are 600)
MENU = ((10, None), (20, None), (30, None), (48, 96), (60, None))  # (the registered shape, scaled to N)


def series_rows(sid, x_of, rounds=N, order_effect=0.0):
    """A shard's rows whose wall log ratio at round i is x_of(i) (plus `order_effect` when the
    head runs first), the two arms in ab's rotated order"""
    corpus, threads = tg.SHARDS[sid]
    rows = []
    for rnd in range(rounds):
        order = [0, 1] if (rnd % 2 == 0) == ((rnd // 2) % 2 == 0) else [1, 0]
        for t in threads:
            x = x_of(rnd) + (order_effect if order[0] == 0 else 0.0)
            for i in order:
                arm = tg.CI_ARMS[i]
                w = 0.05 * (math.exp(x) if i == 0 else 1.0)
                rows.append({"round": rnd, "corpus": corpus, "threads": t, "arm": arm, "wall": w, "cpu": w * 1.6,
                             "rss_kb": 30000, "out": 1000, "hash": "0123456789abcdef", "termination": "success",
                             "err": "", "cpus": {1: "1", 2: "0,2", 4: "all"}[t]})
    return rows


def add_times(root, sid, rows, attempt=1, step=1.5, round_seconds=None):
    """Each round's rows timed at its end: round r ends `round_seconds(r)` after the previous
    one (`step` if None), from the stamper's start at the fixtures' measurement start"""
    d = Path(root) / f"tg-ni-{sid}-a{attempt}" / sid
    per = len(tg.SHARDS[sid][1]) * 2
    t, ends = T.T0, []
    for r in range(len(rows) // per):
        t += step if round_seconds is None else round_seconds(r)
        ends.append(t)
    (d / "row-times.txt").write_text(f"# start {T.T0:.3f}\n" + "".join(
        f"{i} {ends[i // per]:.3f}\n" for i in range(len(rows))))


class Pilot(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp()
        self.saved = T.R
        T.R = N

    def tearDown(self):
        T.R = self.saved
        shutil.rmtree(self.root)

    def run_pilot(self, per, label="pilot", round_seconds=None, after=None, menu=MENU):
        """Build, time every shard, write the manifest, let `after(root, man)` tamper with it,
        and analyse"""
        man = T.build(self.root, label=label, **per)
        for sid in tg.SHARDS:
            for a in man["artifacts"]:
                if a["shard"] == sid and a["files"]:
                    rows = tg.read_jsonl(str(Path(self.root) / a["name"] / sid / "runs.jsonl"))[0]
                    add_times(self.root, sid, rows, a["attempt"], round_seconds=round_seconds)
        man = T.remanifest(self.root, label=label)
        for j in man["jobs"]:
            j["started_at"] = "2026-11-09T23:50:00Z"
        if after:
            after(self.root, man)
        return pilot.analyse(self.root, man, T.HEAD, T.BASESHA, T.REG["frozen"]["ni_rustc"],
                             T.REG["frozen"]["corpus_text"], rounds=N, menu=menu)

    def fresh(self):
        shutil.rmtree(self.root)
        self.root = tempfile.mkdtemp()

    def test_it_sees_dependence_order_and_trend_and_shows_no_ratio(self):
        rng = random.Random(7)
        ar, prev = [], 0.0
        for _ in range(N):
            prev = 0.9 * prev + rng.gauss(0, 0.01)
            ar.append(prev)
        iid = [rng.gauss(0, 0.01) for _ in range(N)]
        per = {
            "runs": [{"rows": series_rows("runs", lambda i: ar[i])}],
            "text": [{"rows": series_rows("text", lambda i: iid[i], order_effect=0.02)}],
            "random": [{"rows": series_rows("random", lambda i: iid[i] + 0.0005 * i)}],
        }
        for sid in tg.SHARDS:
            per.setdefault(sid, [{"rows": series_rows(sid, lambda i: iid[i])}])
        v = self.run_pilot(per)
        self.assertTrue(v["valid"], v["problems"])
        self.assertEqual(len(v["cells"]), 42)
        dep = v["cells"]["runs/2/wall"]
        self.assertGreater(dep["acf"]["1"], 0.6)
        b30 = [b for b in dep["batches"] if b["m"] == 30][0]
        self.assertGreater(b30["lrv_over_marginal"], 2.0, dep["batches"])
        flat = v["cells"]["onebyte/1/wall"]
        self.assertLess(abs(flat["acf"]["1"]), 0.3)
        od = v["cells"]["text/4/wall"]["order"]
        self.assertGreater(od["difference"] / od["se"], 5)
        self.assertGreater(abs(v["cells"]["random/1/wall"]["trend"]["t"]), 5)
        self.assertAlmostEqual(v["shards"]["runs"]["batch_s"]["30"]["median"], 45.0)
        self.assertEqual(v["shards"]["runs"]["round_s"]["median"], 1.5)
        self.assertEqual(v["shards"]["runs"]["setup_s"], 600.0)
        md = pilot.markdown(v)
        text = json.dumps(v)
        for leak in ("upper", "lo95", "ratio\"", "passes"):
            self.assertNotIn(leak, text, leak)
        self.assertIn("not evidence", md)

    def test_lrv_is_normalised_and_m48_uses_its_prefix(self):
        x = [random.Random(3).gauss(0, 1) for _ in range(600)]
        for m, n in pilot.BATCH_MENU:
            b = pilot.batches(x, m, n)
            used = 600 if n is None else n
            self.assertEqual(b["rounds"], used)
            self.assertEqual(b["batches"], used // m)
            means = [sum(x[i * m:(i + 1) * m]) / m for i in range(used // m)]
            mu = sum(means) / len(means)
            var = sum((y - mu) ** 2 for y in means) / (len(means) - 1)
            self.assertAlmostEqual(b["lrv"], m * var)
            self.assertAlmostEqual(b["lrv"], used * b["se"] ** 2)
        self.assertEqual([(m, n) for m, n in pilot.BATCH_MENU], [(10, None), (20, None), (30, None), (48, 576), (60, None)])
        with self.assertRaises(ValueError):
            pilot.batches(x, 48)  # (600 isn't 48s: never silently dropped)

    def test_batch_duration_is_the_actual_span(self):
        # (the Researcher's case: per batch of 30, 29 rounds of 1 s and one of 100 s. The proxy
        # m x median round said 30 s; the batches actually took 129 s)
        per = {sid: [{"rows": series_rows(sid, lambda i: 0.0)}] for sid in tg.SHARDS}
        v = self.run_pilot(per, round_seconds=lambda r: 100.0 if r % 30 == 29 else 1.0)
        self.assertTrue(v["valid"], v["problems"])
        b = v["shards"]["onebyte"]["batch_s"]["30"]
        self.assertAlmostEqual(b["median"], 129.0)
        self.assertAlmostEqual(b["max"], 129.0)
        self.assertEqual(v["shards"]["onebyte"]["round_s"]["max"], 100.0)

    def test_integrity_the_judges_own(self):
        # (the Researcher's two probes, 6757361275 §1: each was admitted by the pilot before)
        per = {sid: [{"rows": series_rows(sid, lambda i: 0.0)}] for sid in tg.SHARDS}

        def change_a_row(root, man):
            path = Path(root) / "tg-ni-runs-a1" / "runs" / "runs.jsonl"
            rows = [json.loads(l) for l in path.read_text().splitlines()]
            rows[5]["wall"] *= 1.5
            path.write_text("".join(json.dumps(r) + "\n" for r in rows))
        v = self.run_pilot(per, after=change_a_row)
        self.assertFalse(v["valid"])
        self.assertTrue(any("runs.jsonl does not match" in p for p in v["problems"]), v["problems"])
        self.fresh()
        v = self.run_pilot(per, after=lambda root, man: man["unlisted"].append("tg-ni-extra-a1"))
        self.assertFalse(v["valid"])
        self.assertTrue(any("not listed by GitHub" in p for p in v["problems"]), v["problems"])

    def test_timing_absent_or_impossible_is_refused(self):
        # (the Researcher's §2 probes: each was admitted, or printed as zero, before)
        per = {sid: [{"rows": series_rows(sid, lambda i: 0.0)}] for sid in tg.SHARDS}

        def drop_times(root, man):
            for sid in tg.SHARDS:
                (Path(root) / f"tg-ni-{sid}-a1" / sid / "row-times.txt").unlink()
                del [a for a in man["artifacts"] if a["shard"] == sid][0]["files"]["row-times.txt"]

        def backwards(root, man):
            d = Path(root) / "tg-ni-runs-a1" / "runs"
            lines = (d / "row-times.txt").read_text().splitlines()
            n = len(lines) - 1
            (d / "row-times.txt").write_text(lines[0] + "\n" + "".join(
                f"{i} {T.T0 + 100 - i:.3f}\n" for i in range(n)))
            [a for a in man["artifacts"] if a["shard"] == "runs"][0]["files"]["row-times.txt"] = \
                tg.sha256_file(str(d / "row-times.txt"))

        def no_job_start(root, man):
            for j in man["jobs"]:
                j.pop("started_at", None)

        def stamper_failed(root, man):
            d = Path(root) / "tg-ni-text-a1" / "text"
            (d / "stamp.exit").write_text("1\n")
            [a for a in man["artifacts"] if a["shard"] == "text"][0]["files"]["stamp.exit"] = \
                tg.sha256_file(str(d / "stamp.exit"))
        for what, after, needle in (("no timestamps", drop_times, "row-times.txt"),
                                    ("backwards timestamps", backwards, "go backwards"),
                                    ("no job start", no_job_start, "setup unknown"),
                                    ("the stamper failed", stamper_failed, "did not exit 0")):
            with self.subTest(what):
                self.fresh()
                v = self.run_pilot(per, after=after)
                self.assertFalse(v["valid"], what)
                self.assertTrue(any(needle in p for p in v["problems"]), v["problems"])
                self.assertNotIn("| 0 |", pilot.markdown(v))

    def test_admission_is_the_judges_with_the_pilot_label(self):
        v = self.run_pilot({}, label="qualification")
        self.assertFalse(v["valid"])
        shutil.rmtree(self.root)
        self.root = tempfile.mkdtemp()
        v = self.run_pilot({"runs": [{"job": "cancelled", "step": "cancelled"}]})
        self.assertFalse(v["valid"])
        self.assertTrue(any(p.startswith("runs") for p in v["problems"]), v["problems"])

    def test_the_gate_needs_the_structural_admission(self):
        # (the Researcher's §4: a refused label with nothing behind it said "plumbing OK")
        empty = {"schema": "mibz2.tg_ni_manifest/2", "label": "smoke", "run": T.RUN, "artifacts": [], "jobs": []}
        for label in ("smoke", "pilot"):
            with self.subTest(label):
                man = dict(empty, label=label)
                refused = tg.ni_judge(self.root, man, T.HEAD, reg={"frozen": {}, "bench": None}, check_git=False)
                structure = tg.structure(self.root, man, T.HEAD, T.BASESHA, 2, label, "1.99.1")
                self.assertFalse(structure["valid"])
                status, line = gate.gate(label, refused, 2, structure)
                self.assertEqual(status, 1, line)
                self.assertEqual(gate.gate(label, refused, 2, None)[0], 1)
        # (a complete R2 smoke: every shard, listed, downloaded, reconciled, whole: admitted)
        saved = T.R
        T.R = 2
        try:
            self.fresh()
            per = {sid: [{"rows": series_rows(sid, lambda i: 0.0, rounds=2)}] for sid in tg.SHARDS}
            man = T.build(self.root, label="smoke", **per)
            structure = tg.structure(self.root, man, T.HEAD, T.BASESHA, 2, "smoke", "1.99.1")
            self.assertTrue(structure["valid"], structure["problems"])
            self.assertEqual(len(structure["shards"]), 9)
            self.assertTrue(structure["unchecked"])
            refused = tg.ni_judge(self.root, man, T.HEAD, reg={"frozen": {}, "bench": None}, check_git=False, rounds=2)
            status, line = gate.gate("smoke", refused, 2, structure)
            self.assertEqual(status, 0, line)
            self.assertIn("smoke: not evidence", line)
            self.assertIn("all 9 shards", line)
            # (a structure of another label, or one missing a shard, fails)
            self.assertEqual(gate.gate("pilot", refused, 2, structure)[0], 1)
            man["artifacts"] = [a for a in man["artifacts"] if a["shard"] != "runs"]
            broken = tg.structure(self.root, man, T.HEAD, T.BASESHA, 2, "smoke", "1.99.1")
            self.assertFalse(broken["valid"])
            self.assertEqual(gate.gate("smoke", refused, 2, broken)[0], 1)
            # (another toolchain than the dispatched one)
            self.assertFalse(tg.structure(self.root, T.build(self.root + "x", label="smoke", **per), T.HEAD,
                                          T.BASESHA, 2, "smoke", "1.98.0")["valid"])
        finally:
            T.R = saved
            shutil.rmtree(self.root + "x", ignore_errors=True)
        self.assertEqual(gate.gate("pilot", {"valid": True, "passes": True}, 0)[0], 1)


class Stamp(unittest.TestCase):
    def test_each_row_is_timed_and_a_truncation_restarts(self):
        d = Path(tempfile.mkdtemp())
        try:
            runs, out, stop = d / "runs.jsonl", d / "times.txt", d / "stop"
            runs.write_text("")
            clock = iter(range(1000, 100000))
            t = threading.Thread(target=stamp.follow, args=(str(runs), str(out), str(stop)),
                                 kwargs={"interval": 0.01, "clock": lambda: next(clock)})
            t.start()
            for i in range(3):
                with open(runs, "a") as f:
                    f.write(json.dumps({"i": i}) + "\n")
                time.sleep(0.05)
            runs.write_text(json.dumps({"i": 0}) + "\n")  # (ab opening the file again)
            time.sleep(0.05)
            stop.write_text("")
            t.join(5)
            self.assertFalse(t.is_alive())
            lines = out.read_text().splitlines()
            self.assertEqual([l.split()[0] for l in lines if not l.startswith("#")][:3], ["0", "1", "2"])
            self.assertIn("# truncated", lines)
            self.assertEqual(lines[-1].split()[0], "0")
        finally:
            shutil.rmtree(d)


if __name__ == "__main__":
    unittest.main(verbosity=2)
