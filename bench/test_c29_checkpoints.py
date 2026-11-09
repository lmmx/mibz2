"""bench/c29_checkpoints.py on synthetic runs: a clean run is VALID with the reading its
ratios give, each defect the admission guards against makes it INVALID, and a run whose
registered environment (AVX2, the CPU allocation) is absent or other is never JUDGED.

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
import c29_identity  # noqa: E402

HEAD = "a" * 40
# the registered environment's facts, as ab's child and the workflow record them
RECORD_FACTS = {"avx2": True, "cpus_allowed": 4, "available_parallelism": 4, "cpus": "all"}
# `ab env`'s line on a registered runner: AVX2, 4 CPUs, a quota read and unlimited
ENV = {
    "cpus": 4,
    "avx2": True,
    "cpus_allowed": 4,
    "available_parallelism": 4,
    "cores": 2,
    "core_cpus": ["0,1", "2,3"],
    "cpu_quota": None,
    "cpu_quota_state": "unlimited",
}
# the identity's fields schema 1 lacked (the environment's, the toolchain's, the run status)
IDENTITY_FACTS = (
    "avx2",
    "nproc",
    "cpus_allowed",
    "available_parallelism",
    "cpu_quota",
    "cpu_model",
    "ab_run_status",
    "toolchain",
    "env",
)


def names(t):
    teams = t // 3
    return " ".join(
        [f"mibz2-{i}" for i in range(teams)] + ["mibz2-coop-p1"] * (2 * teams) + ["mibz2-scan"]
    )


def runs(ratio=0.97, cpu=1.0, rounds=c.ROUNDS, noise=0.004, facts=None):
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
                        **(RECORD_FACTS if facts is None else facts),
                    }
                )
    return out


def identity(env_text=None, nproc_text="4\n", run_status=0, **kw):
    """The identity as the workflow builds it (bench/c29_identity.py's `build`), from `ab
    env`'s line (default ENV's) and `nproc`'s output; then any field overridden"""
    d = c29_identity.build(
        json.dumps(ENV) if env_text is None else env_text,
        nproc_text,
        head=HEAD,
        tree_clean=True,
        rustc="rustc 1.97.0",
        toolchain={"rustc_vV": "rustc 1.97.0", "cargo": "cargo 1.97.0", "rustflags": None},
        binary=("b" * 64, "b" * 64),
        corpus=(c.CORPUS_SHA256, c.CORPUS_SHA256),
        arms=c.REGISTERED_ARMS,
        rounds=c.ROUNDS,
        model="AMD EPYC 7763 64-Core Processor",
        run_status=run_status,
    )
    d.update(kw)
    return d


def env_line(**kw):
    """ENV with fields replaced; a None value drops the field"""
    e = {**ENV, **kw}
    return json.dumps({k: v for k, v in e.items() if v is not None or k == "cpu_quota"})


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


class Environment(unittest.TestCase):
    """The judged reading bound to the executed kernel and the runner's capacity"""

    def not_judged(self, recs, ident):
        code, out = report(recs, ident)
        self.assertEqual(code, 3, out)
        self.assertTrue(out.startswith("NOT JUDGED"), out)
        self.assertNotIn("VALID (JUDGED)", out)
        self.assertNotIn("(judged)", out)
        self.assertIn("VALID (UNJUDGED (FALLBACK))", out)
        self.assertIn("Reading (UNJUDGED (FALLBACK), 9 threads)", out)
        return out

    def test_researchers_counterexample(self):
        # the Researcher's probe (REVIEW 6734359855): a ratio-one run with neither fact,
        # identity schema 1's fields only: was VALID (JUDGED) STOP at 1.0000 [0.9986, 1.0014]
        ident = {k: v for k, v in identity().items() if k not in IDENTITY_FACTS}
        out = self.not_judged(runs(ratio=1.0, facts={}), ident)
        self.assertIn("identity: avx2 None", out)
        self.assertIn("identity: nproc None", out)
        self.assertIn("avx2 None, not True", out)
        self.assertIn("cpus_allowed None", out)
        # and its illustrative fields, now schema keys: still not judged
        ident.update(avx2=False, cpu_count=2)
        self.not_judged(runs(ratio=1.0, facts={}), ident)

    def test_false_avx2(self):
        out = self.not_judged(runs(ratio=1.0), identity(avx2=False))
        self.assertIn("identity: avx2 False", out)
        # the records' own detection, the identity's true: still not judged
        out = self.not_judged(runs(ratio=1.0, facts={**RECORD_FACTS, "avx2": False}), identity())
        self.assertIn("avx2 False, not True", out)
        # one record alone
        r = runs(ratio=1.0)
        r[7]["avx2"] = False
        self.assertIn("line 8", self.not_judged(r, identity()))
        # a truthy non-bool is not a detection
        self.not_judged(runs(ratio=1.0), identity(avx2=1))

    def test_two_cpus(self):
        two = {"nproc": 2, "cpus_allowed": 2, "available_parallelism": 2}
        rec2 = {**RECORD_FACTS, "cpus_allowed": 2, "available_parallelism": 2}
        out = self.not_judged(runs(ratio=0.97, facts=rec2), identity(**two))
        self.assertIn("nproc 2, not 4", out)
        # each fact alone refuses it
        for k in two:
            self.not_judged(runs(ratio=0.97), identity(**{k: 2}))
        self.not_judged(
            runs(ratio=0.97, facts={**RECORD_FACTS, "available_parallelism": 2}), identity()
        )
        # a cgroup quota of 2 CPUs, the mask's 4: not the registered capacity
        two_quota = env_line(cpu_quota=2.0, cpu_quota_state="finite")
        self.assertIn("cpu_quota 2.0 CPUs", self.not_judged(runs(), identity(two_quota)))
        # a pinned run is not the registered, unpinned setting
        self.not_judged(runs(facts={**RECORD_FACTS, "cpus": "0,1"}), identity())

    def test_correct_positive(self):
        code, out = report(runs(ratio=1.0), identity())
        self.assertEqual(code, 0, out)
        self.assertTrue(out.startswith("VALID (JUDGED)"), out)
        self.assertIn("1.0000 [0.9986, 1.0014]", out)
        self.assertIn("Reading (JUDGED, 9 threads): STOP", out)
        code, out = report(runs(ratio=0.97), identity())
        self.assertEqual(code, 0, out)
        self.assertIn("Reading (JUDGED, 9 threads): LEAD", out)

    def test_quota_states(self):
        # the producer's three states, through the workflow's mapping (c29_identity.build)
        self.assertEqual(identity()["cpu_quota"], {"state": "unlimited"})
        for q in (4.0, 8.0):
            ident = identity(env_line(cpu_quota=q, cpu_quota_state="finite"))
            self.assertEqual(ident["cpu_quota"], {"state": "finite", "cpus": q})
            code, out = report(runs(ratio=0.97), ident)
            self.assertEqual(code, 0, out)
            self.assertIn("Reading (JUDGED, 9 threads): LEAD", out)
        # negatives: the Researcher's (REVIEW 6734701311) and their kin, each not JUDGED
        negatives = {
            # ab env's quota not read (neither cgroup file, or unparsable): null, "unknown"
            "failed detection": env_line(cpu_quota_state="unknown"),
            # the quota keys absent from ab env's line (the review's
            # workflow_absent_quota_mapped_to_null probe)
            "absent key": json.dumps(
                {k: v for k, v in ENV.items() if k not in ("cpu_quota", "cpu_quota_state")}
            ),
            # a null quota with no state (an older producer): never read as unlimited
            "null": json.dumps({k: v for k, v in ENV.items() if k != "cpu_quota_state"}),
            # a state that disagrees with its number
            "unlimited with a number": env_line(cpu_quota=2.0, cpu_quota_state="unlimited"),
            "finite without a number": env_line(cpu_quota_state="finite"),
            # ab env's line absent or broken: every fact unknown
            "no line": "",
            "broken line": "{not json",
        }
        for what, text in negatives.items():
            with self.subTest(what):
                ident = identity(text)
                if what not in ("no line", "broken line"):
                    self.assertEqual(ident["cpu_quota"], {"state": "unknown"}, what)
                self.assertIn("cpu_quota", self.not_judged(runs(ratio=1.0), ident))
        # and the identity's own quota field absent, null or a bare number
        for q in (None, 4.0, "unlimited"):
            self.not_judged(runs(ratio=1.0), identity(cpu_quota=q))
        ident = identity()
        del ident["cpu_quota"]
        self.not_judged(runs(ratio=1.0), ident)

    def test_failed_ab_run(self):
        # complete records, but `ab run` itself exited 101: the report refuses JUDGED too
        out = self.not_judged(runs(ratio=1.0), identity(run_status=101))
        self.assertIn("ab_run_status 101, not 0", out)
        ident = identity()
        del ident["ab_run_status"]
        self.not_judged(runs(ratio=1.0), ident)

    def test_invalid_comes_first(self):
        # an INVALID run is INVALID whatever its environment
        r = runs(facts={})
        r[2]["exit"] = 1
        code, out = report(r, identity(avx2=False))
        self.assertEqual(code, 2, out)
        self.assertTrue(out.startswith("INVALID"), out)

    def test_unjudged_notes_the_environment(self):
        code, out = report(runs(facts={}), judged=False)
        self.assertEqual(code, 0, out)
        self.assertIn("VALID (UNJUDGED)", out)
        self.assertIn("not the registered one", out)


if __name__ == "__main__":
    unittest.main()
