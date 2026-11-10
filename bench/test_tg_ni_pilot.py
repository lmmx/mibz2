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


def add_times(root, sid, rows, attempt=1, step=1.5):
    d = Path(root) / f"tg-ni-{sid}-a{attempt}" / sid
    (d / "row-times.txt").write_text("".join(f"{i} {1000 + step * (i // (len(tg.SHARDS[sid][1]) * 2)):.3f}\n"
                                             for i in range(len(rows))))


class Pilot(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp()
        self.saved = T.R
        T.R = N

    def tearDown(self):
        T.R = self.saved
        shutil.rmtree(self.root)

    def run_pilot(self, per, label="pilot"):
        man = T.build(self.root, label=label, **per)
        for sid in tg.SHARDS:
            for a in man["artifacts"]:
                if a["shard"] == sid and a["files"]:
                    rows = tg.read_jsonl(str(Path(self.root) / a["name"] / sid / "runs.jsonl"))[0]
                    add_times(self.root, sid, rows, a["attempt"])
        man = T.remanifest(self.root, label=label)
        for j in man["jobs"]:
            j["started_at"] = "2026-11-10T00:00:00Z"
        return pilot.analyse(self.root, man, T.HEAD, T.BASESHA, T.REG["frozen"]["ni_rustc"],
                             T.REG["frozen"]["corpus_text"], rounds=N)

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
        se30 = [b for b in dep["batches"] if b["m"] == 30]
        self.assertTrue(se30 and se30[0]["se_over_iid"] > 1.5, dep["batches"])
        flat = v["cells"]["onebyte/1/wall"]
        self.assertLess(abs(flat["acf"]["1"]), 0.3)
        od = v["cells"]["text/4/wall"]["order"]
        self.assertGreater(od["difference"] / od["se"], 5)
        self.assertGreater(abs(v["cells"]["random/1/wall"]["trend"]["t"]), 5)
        self.assertAlmostEqual(v["cells"]["runs/1/wall"]["batch_seconds"]["30"], 45.0)
        self.assertEqual(v["shards"]["runs"]["round_s"]["median"], 1.5)
        md = pilot.markdown(v)
        text = json.dumps(v)
        for leak in ("upper", "lo95", "ratio\"", "passes"):
            self.assertNotIn(leak, text, leak)
        self.assertIn("not evidence", md)

    def test_admission_is_the_judges_with_the_pilot_label(self):
        v = self.run_pilot({}, label="qualification")
        self.assertFalse(v["valid"])
        shutil.rmtree(self.root)
        self.root = tempfile.mkdtemp()
        v = self.run_pilot({"runs": [{"job": "cancelled", "step": "cancelled"}]})
        self.assertFalse(v["valid"])
        self.assertTrue(any(p.startswith("runs") for p in v["problems"]), v["problems"])

    def test_the_gate_takes_a_pilot_as_never_evidence(self):
        refused = tg.ni_judge(self.root, {"label": "pilot", "artifacts": []}, "f" * 40,
                              reg={"frozen": {}, "bench": None}, check_git=False)
        status, line = gate.gate("pilot", refused, 2)
        self.assertEqual(status, 0, line)
        self.assertIn("pilot: not evidence", line)
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
