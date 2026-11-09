"""bench/c29_checkpoints.py on synthetic runs: a clean run is VALID with the reading its
ratios give, and each defect the admission guards against makes it INVALID.

    python3 bench/test_c29_checkpoints.py
"""

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import c29_checkpoints as c  # noqa: E402

HEAD = "a" * 40


def names(t):
    teams = t // 3
    return " ".join(
        [f"mibz2-{i}" for i in range(teams)] + ["mibz2-coop-p1"] * (2 * teams) + ["mibz2-scan"]
    )


def runs(ratio=0.97, cpu=1.0, rounds=c.ROUNDS, noise=0.004):
    """ab's records for both arms at both thread counts, in ab's rotation"""
    out = []
    for t in (c.THREADS, c.REPORTED_THREADS):
        for rnd in range(rounds):
            jitter = 1 + noise * ((rnd * 7919) % 5 - 2) / 2
            for i in c.arm_order(rnd, 2):
                arm = c.REGISTERED_ARMS[i]
                w = 2.0 * (ratio * jitter if arm == c.C29 else 1.0)
                out.append(
                    {
                        "round": rnd,
                        "corpus": "wikidata",
                        "threads": t,
                        "arm": arm,
                        "observed_threads": names(t),
                        "exit": 0,
                        "err": "",
                        "hash": "h",
                        "out": 123,
                        "wall": w,
                        "cpu": 6.0 * (cpu if arm == c.C29 else 1.0),
                    }
                )
    return out


def identity(**kw):
    d = {
        "schema": c.IDENTITY_SCHEMA,
        "head": HEAD,
        "tree_clean": True,
        "features": [],
        "profile": "release",
        "rustc": "rustc 1.97.0",
        "binary_sha256_before": "b" * 64,
        "binary_sha256_after": "b" * 64,
        "corpus_sha256_before": c.CORPUS_SHA256,
        "corpus_sha256_after": c.CORPUS_SHA256,
        "arms": c.REGISTERED_ARMS,
        "threads": [c.THREADS, c.REPORTED_THREADS],
        "rounds": c.ROUNDS,
    }
    d.update(kw)
    return d


def report(recs, ident=None, judged=True):
    with tempfile.TemporaryDirectory() as d:
        p = Path(d) / "runs.jsonl"
        p.write_text("".join(json.dumps(r) + "\n" for r in recs))
        i = Path(d) / "id.json"
        i.write_text(json.dumps(identity() if ident is None else ident))
        cmd = [sys.executable, str(HERE / "c29_checkpoints.py"), "report", str(p)]
        cmd += ["--head", HEAD, "--identity", str(i)] if judged else ["--unjudged"]
        r = subprocess.run(cmd, capture_output=True, text=True)
        return r.returncode, r.stdout


class Reporter(unittest.TestCase):
    def test_lead(self):
        code, out = report(runs(0.97))
        self.assertEqual(code, 0, out)
        self.assertIn("VALID (JUDGED)", out)
        self.assertIn("Reading (JUDGED, 9 threads): LEAD", out)

    def test_stop_and_inconclusive(self):
        self.assertIn("): STOP", report(runs(1.0))[1])
        self.assertIn("): INCONCLUSIVE", report(runs(0.985))[1])
        # a wall gain at too much CPU is no LEAD
        self.assertNotIn("): LEAD", report(runs(0.97, cpu=1.03))[1])

    def test_unjudged(self):
        code, out = report(runs(0.97), judged=False)
        self.assertEqual(code, 0, out)
        self.assertIn("UNJUDGED", out)

    def invalid(self, recs, ident=None):
        code, out = report(recs, ident)
        self.assertEqual(code, 2, out)
        self.assertTrue(out.startswith("INVALID"), out)
        return out

    def test_defects(self):
        r = runs()
        r[3]["hash"] = "other"
        self.assertIn("output differs", self.invalid(r))
        r = runs()
        r[0]["observed_threads"] = "mibz2-0 mibz2-1 mibz2-2 mibz2-3"
        self.assertIn("ran (4, 0, 0)", self.invalid(r))
        r = runs()
        del r[5]
        self.assertIn("rounds missing", self.invalid(r))
        r = runs()
        r[0], r[1] = r[1], r[0]
        self.assertIn("not ab's rotation", self.invalid(r))
        r = runs()
        r[2]["wall"] = None
        self.assertIn("not finite and positive", self.invalid(r))
        r = runs()
        r[2]["exit"] = 1
        self.assertIn("exit 1", self.invalid(r))
        self.assertIn("not the registered", self.invalid(runs(), identity(head="c" * 40)))
        self.assertIn("not clean", self.invalid(runs(), identity(tree_clean=False)))
        self.assertIn("not one build", self.invalid(runs(), identity(binary_sha256_after="d" * 64)))
        self.assertIn("corpus's SHA-256", self.invalid(runs(), identity(corpus_sha256_after="e")))
        self.assertIn("rounds missing", self.invalid(runs(rounds=5), identity(rounds=5)))


if __name__ == "__main__":
    unittest.main()
