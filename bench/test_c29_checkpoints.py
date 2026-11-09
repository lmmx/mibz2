"""bench/c29_checkpoints.py on synthetic runs: a clean run is VALID with the reading its
ratios give, each defect the admission guards against makes it INVALID, and a run whose
registered environment (AVX2, the CPU allocation, the cgroup quota's coverage) is absent or
other is never JUDGED. The quota cases are examples/ab.rs's own census over fake trees
(bench/c29_fixtures, written and checked by `cargo test --example ab`), through the identity
builder and the reporter.

    python3 bench/test_c29_checkpoints.py
"""

import json
import math
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
FIXTURES = HERE / "c29_fixtures"


def fixture(name):
    """`ab env`'s quota fields (cpu_quota, cpu_quota_state, cgroup) for a fake tree"""
    return json.loads((FIXTURES / f"{name}.json").read_text())


# the registered environment's facts, as ab's child and the workflow record them
RECORD_FACTS = {"avx2": True, "cpus_allowed": 4, "available_parallelism": 4, "cpus": "all"}
# `ab env`'s line on a registered runner: AVX2, 4 CPUs, every level's quota max to the real
# root (the census's fixture v2_all_max)
ENV = {
    "cpus": 4,
    "avx2": True,
    "cpus_allowed": 4,
    "available_parallelism": 4,
    "cores": 2,
    "core_cpus": ["0,1", "2,3"],
    "cpus_allowed_ids": [0, 1, 2, 3],
    **fixture("v2_all_max"),
}
# the identity's fields schema 1 lacked (the environment's, the toolchain's, the run status)
IDENTITY_FACTS = (
    "avx2",
    "nproc",
    "cpus_allowed",
    "available_parallelism",
    "cpu_quota",
    "cpu_quota_after",
    "cgroup_stable",
    "cpus_allowed_ids",
    "env_after",
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


def identity(env_text=None, nproc_text="4\n", run_status=0, after=None, **kw):
    """The identity as the workflow builds it (bench/c29_identity.py's `build`), from `ab
    env`'s line before (default ENV's) and after the run (default the same) and `nproc`'s
    output; then any field overridden"""
    text = json.dumps(ENV) if env_text is None else env_text
    d = c29_identity.build(
        text,
        nproc_text,
        env_after_text=text if after is None else after,
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


def census_line(name, **kw):
    """ENV with a fixture's quota fields in place of v2_all_max's"""
    return json.dumps({**ENV, **fixture(name), **kw})


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
        # a cgroup quota of 2 CPUs (at the parent, max below), the mask's 4: not the
        # registered capacity
        two_quota = census_line("v2_parent_two_child_max")
        self.assertIn("below 4 CPUs", self.not_judged(runs(), identity(two_quota)))
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
        # through the workflow's mapping (c29_identity.build), from the census
        self.assertEqual(identity()["cpu_quota"], {"state": "unlimited", "coverage": "complete"})
        for name, q in (("v2_exactly_four", 200000), ("v2_parent_eight_child_four", 400000)):
            ident = identity(census_line(name))
            self.assertEqual(ident["cpu_quota"]["state"], "finite")
            self.assertEqual(ident["cpu_quota"]["quota"], q)
            code, out = report(runs(ratio=0.97), ident)
            self.assertEqual(code, 0, out)
            self.assertIn("Reading (JUDGED, 9 threads): LEAD", out)
        # negatives: the earlier producer's shapes, and their kin, each not JUDGED
        no_census = {k: v for k, v in ENV.items() if k != "cgroup"}
        negatives = {
            # the first registration's producer: a state, no census (run 42582708226's shape)
            "state without a census": json.dumps({**no_census, "cpu_quota_state": "unlimited"}),
            "failed detection": env_line(cpu_quota_state="unknown"),
            "absent keys": json.dumps(
                {k: v for k, v in no_census.items() if k not in ("cpu_quota", "cpu_quota_state")}
            ),
            "null, no state": json.dumps({k: v for k, v in ENV.items() if k != "cpu_quota_state"}),
            # a state that disagrees with its number, or with the census
            "unlimited with a number": env_line(cpu_quota=2.0),
            "finite without a number": env_line(cpu_quota_state="finite"),
            "census says finite": json.dumps(
                {**ENV, "cgroup": {**ENV["cgroup"], "state": "finite"}}
            ),
            "no line": "",
            "broken line": "{not json",
        }
        for what, text in negatives.items():
            with self.subTest(what):
                ident = identity(text)
                self.assertEqual(ident["cpu_quota"]["state"], "unknown", what)
                self.assertIn("cpu_quota", self.not_judged(runs(ratio=1.0), ident))
        # and the identity's own quota field absent, null, a bare number, or a state alone
        for q in (None, 4.0, "unlimited", {"state": "unlimited"}, {"state": "finite", "cpus": 4.0}):
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


# Every census fixture's expected outcome: judged (True), or the reason a judged reading is
# refused (False). Written by hand from the Researcher's REVIEW 6735626248, not from the
# collector's output
EXPECTED = {
    # complete coverage, unlimited
    "v2_all_max": True,
    "v2_in_root": True,  # the real root without cpu.max: a justified absence
    "v2_no_controller_parent": True,  # no cpu controller below the root: no limit there
    "v2_escaped_mount_point": True,  # a non-default mount point, `\040` unescaped
    "v1_combined_unlimited": True,  # v1 cpu,cpuacct, -1 to the root
    # complete coverage, finite, Q >= 4P exactly
    "v2_exactly_four": True,  # 200000/50000: Q = 4P
    "v2_parent_eight_child_four": True,  # the tightest is the child's 4
    "hybrid_cpu_on_v1": True,  # cpu bound to v1 beside a v2 hierarchy, 4 CPUs
    # complete coverage, below four
    "v2_parent_two_child_max": False,  # finite at the parent, max below: 2
    "v2_just_under_four": False,  # 399999/100000
    "v2_rational_under_four": False,  # 1199999/300000: "4.00" rounded, under 4 exactly
    "v1_combined_two": False,
    # incomplete coverage: unknown, never a pass
    "witness1_max_hidden2": False,
    "witness2_four_hidden2": False,
    "witness3_eight_hidden2": False,
    "witness4_max_hidden8": False,
    "v2_malformed_parent": False,
    "v2_zero_period": False,
    "v2_three_fields": False,
    "v2_no_cpu_max_with_cpu": False,  # cpu enabled, cpu.max missing: unexplained
    "v2_namespace_root": False,  # a visible / in a cgroup namespace: ancestors hidden
    "v2_mount_root_offset": False,  # the mount's root /docker/abc: ancestors hidden
    "v2_path_outside_namespace": False,
    "v2_overmounted": False,
    "v1_malformed_quota": False,
    "no_membership": False,
    "malformed_membership": False,
}
BELOW = {
    "v2_parent_two_child_max",
    "v2_just_under_four",
    "v2_rational_under_four",
    "v1_combined_two",
}


class Coverage(unittest.TestCase):
    """The collector's census, through the identity, to the reporter's admission"""

    def outcome(self, name, **kw):
        ident = identity(census_line(name), **kw)
        return ident, report(runs(ratio=0.97), ident)

    def test_every_fixture_is_expected(self):
        on_disk = {p.stem for p in FIXTURES.glob("*.json")}
        self.assertEqual(on_disk, set(EXPECTED))

    def test_each_fixture(self):
        for name, judged in EXPECTED.items():
            with self.subTest(name):
                ident, (code, out) = self.outcome(name)
                q = ident["cpu_quota"]
                if judged:
                    self.assertEqual(code, 0, out)
                    self.assertIn("VALID (JUDGED)", out)
                    self.assertEqual(q["coverage"], "complete")
                else:
                    self.assertEqual(code, 3, out)
                    self.assertIn("UNJUDGED (FALLBACK)", out)
                    self.assertNotIn("VALID (JUDGED)", out)
                    if name in BELOW:
                        self.assertEqual(q["coverage"], "complete")
                        self.assertIn("below 4 CPUs", out)
                    else:
                        self.assertEqual(q["state"], "unknown")
                        self.assertIn("not known", out)

    def test_the_witnesses(self):
        # the Researcher's witnesses.py: (observed, hidden parent). "Unknown only when no level
        # parses" admits three of them falsely; coverage refuses all four
        worlds = {
            "witness1_max_hidden2": 2,
            "witness2_four_hidden2": 2,
            "witness3_eight_hidden2": 2,
            "witness4_max_hidden8": 8,
        }
        false_admissions = 0
        for name, hidden in worlds.items():
            cg = fixture(name)["cgroup"]
            parsed = [
                lv["quota"] / lv["period"] if lv["kind"] == "limit" else math.inf
                for lv in cg["levels"]
                if lv["kind"] in ("limit", "unlimited")
            ]
            proposed = min(parsed)
            actual = min(proposed, hidden)
            false_admissions += proposed >= 4 and actual < 4
            ident, (code, out) = self.outcome(name)
            self.assertEqual(code, 3, out)
            self.assertEqual(ident["cpu_quota"]["state"], "unknown")
        self.assertEqual(false_admissions, 3)
        # an observed limit under incomplete coverage is kept, as an upper bound only
        ident, _ = self.outcome("witness2_four_hidden2")
        self.assertEqual(ident["cpu_quota"]["observed_tightest"]["quota"], 400000)
        # witnesses 1 and 4: one observation, two worlds, one census
        self.assertEqual(fixture("witness1_max_hidden2"), fixture("witness4_max_hidden8"))

    def test_a_tampered_census_is_not_trusted(self):
        cg = fixture("witness1_max_hidden2")["cgroup"]
        # the producer's kinds and state rewritten to unlimited: the files still say EACCES
        lv = [
            {**x, "kind": "unlimited" if x["kind"] == "unknown" else x["kind"]}
            for x in cg["levels"]
        ]
        forged = {**cg, "levels": lv, "state": "unlimited", "coverage": "complete", "gaps": []}
        line = json.dumps({**ENV, "cgroup": forged, "cpu_quota_state": "unlimited"})
        self.assertIn("cpu_quota", Environment.not_judged(self, runs(), identity(line)))
        good = ENV["cgroup"]
        tampered = {
            "a non-initial namespace": {
                **good,
                "ns": {**good["ns"], "cgroup": {"text": "cgroup:[1]"}},
            },
            "the boundary not claimed": {
                **good,
                "boundary": {**good["boundary"], "real_root": False},
            },
            "a mount-root offset": {**good, "mount": {**good["mount"], "root": "/x"}},
            "a level dropped": {**good, "levels": good["levels"][1:]},
            "a gap in the chain": {**good, "levels": [good["levels"][0], good["levels"][2]]},
            "a bool quota": {
                **good,
                "levels": [{**good["levels"][0], "kind": "limit", "quota": True, "period": 1}]
                + good["levels"][1:],
            },
            "no schema": {k: v for k, v in good.items() if k != "schema"},
        }
        for what, cg in tampered.items():
            with self.subTest(what):
                ident = identity(json.dumps({**ENV, "cgroup": cg}))
                self.assertEqual(ident["cpu_quota"]["state"], "unknown", what)
                Environment.not_judged(self, runs(), ident)

    def test_before_and_after(self):
        # the configuration changed between the readings: not judged
        after = census_line("v2_exactly_four")
        out = Environment.not_judged(self, runs(), identity(after=after))
        self.assertIn("cgroup_stable", out)
        # no reading after: not judged
        ident = identity()
        ident["cgroup_stable"] = c29_identity.stability(ENV, {})
        Environment.not_judged(self, runs(), ident)
        self.not_judged_without("cgroup_stable")
        # the counters alone moved: still judged
        cg = json.loads(json.dumps(ENV["cgroup"]))
        cg["levels"][0]["files"]["cpu.stat"] = {"text": "usage_usec 99999\n"}
        code, out = report(runs(), identity(after=json.dumps({**ENV, "cgroup": cg})))
        self.assertEqual(code, 0, out)
        # the affinity's CPU IDs changed: not judged
        Environment.not_judged(
            self, runs(), identity(after=json.dumps({**ENV, "cpus_allowed_ids": [0, 1, 2, 4]}))
        )

    def not_judged_without(self, key):
        ident = identity()
        del ident[key]
        Environment.not_judged(self, runs(), ident)


class Preflight(unittest.TestCase):
    """`preflight`: the environment alone, no records, no run status"""

    def run_preflight(self, before, after=None, nproc="4\n"):
        with tempfile.TemporaryDirectory() as d:
            paths = []
            for name, text in (
                ("env", before),
                ("after", after if after is not None else before),
                ("nproc", nproc),
            ):
                p = Path(d) / name
                if text is not None:
                    p.write_text(text)
                paths.append(str(p))
            cmd = [sys.executable, str(HERE / "c29_checkpoints.py"), "preflight"]
            cmd += ["--env", paths[0], "--env-after", paths[1], "--nproc", paths[2]]
            cmd += ["--out", str(Path(d) / "pf")]
            r = subprocess.run(cmd, capture_output=True, text=True)
            saved = (
                json.loads((Path(d) / "pf.json").read_text()) if r.returncode in (0, 3) else None
            )
            return r.returncode, r.stdout, saved

    def test_pass_and_fail(self):
        code, out, saved = self.run_preflight(json.dumps(ENV))
        self.assertEqual(code, 0, out)
        self.assertTrue(out.startswith("PREFLIGHT PASS"), out)
        self.assertEqual(saved["preflight"], "PASS")
        for name in ("witness1_max_hidden2", "v2_namespace_root", "v2_just_under_four"):
            code, out, saved = self.run_preflight(census_line(name))
            self.assertEqual(code, 3, out)
            self.assertTrue(out.startswith("PREFLIGHT FAIL"), out)
        # two CPUs, or no line at all
        self.assertEqual(self.run_preflight(json.dumps(ENV), nproc="2\n")[0], 3)
        self.assertEqual(self.run_preflight("")[0], 3)
        # changed between its two readings
        self.assertEqual(self.run_preflight(json.dumps(ENV), census_line("v2_exactly_four"))[0], 3)


if __name__ == "__main__":
    unittest.main()
