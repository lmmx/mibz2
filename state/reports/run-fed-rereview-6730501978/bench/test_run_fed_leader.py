"""bench/run_fed_leader.py's admission and reading, on synthetic runs: a clean judged set
passes; every failure it must refuse is refused, as INVALID with a reason and never a
crash (the Researcher's probes of 6730026367 among them: absent hash and output fields,
negative CPU, negative phase counts with the timing phases absent, corpus labels varying by
round); the registration's identity is enforced; and the reading is always against the
incumbent, with failed attribution marked (the Researcher's ctl ≈ 0.816, hs ≈ 0.945
fixture).

    python3 bench/test_run_fed_leader.py
"""

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import run_fed_leader as rfl  # noqa: E402

HEAD = "0123456789abcdef0123456789abcdef01234567"
PLAN9 = "census mibz2-0 mibz2-1 mibz2-2 " + " ".join(f"mibz2-coop-p{i}" for i in range(6))
PLAN3 = "census mibz2-0 mibz2-coop-p1 mibz2-coop-p2"


def phases(arm):
    """A run-fed arm's whole, consistent phase string"""
    rec_arm = arm in rfl.RECORD_ARMS
    p = {k: 0 for k in rfl.PHASES}
    p.update(
        runs=10, team_records=5 if rec_arm else 0, team_column=5 if rec_arm else 10,
        lead_ns=7_000_000, lead_front_ns=4_000_000, lead_start_ns=200_000,
        lead_end_ns=7_500_000, build_end_ns=9_000_000, run_ns=12_000_000, join_wait_ns=1_000_000,
        leader_late=2, rule_blocks=4, rule_front_ns=800_000,
    )  # fmt: skip
    if rec_arm:
        p.update(
            records=70, rows=1000, stores=1500, cells=90, shares=10, share_records_max=40,
            share_rows_max=110, share_cells_max=50, share_stores_max=160,
            ready_last_ns=8_000_000, ready_skew_ns=500_000,
            lookups=30 if arm.endswith("-dir") else 0,
        )  # fmt: skip
    return " ".join(f"{k}={v}" for k, v in p.items())


def clean(rounds=rfl.ROUNDS, factors=None):
    """Every arm, both thread counts, in ab's rotation; each arm's wall `factors[arm]` times
    the incumbent's, with a little noise a round"""
    factors = factors or {}
    recs = []
    arms = rfl.REGISTERED_ARMS
    for rnd in range(rounds):
        for t in (rfl.THREADS, rfl.REPORTED_THREADS):
            base = 1.0 + 0.01 * ((rnd * 7) % 5)
            for i in rfl.arm_order(rnd, len(arms)):
                arm = arms[i]
                f = factors.get(arm, 1.0 + 0.002 * i)
                noise = 1 + 0.003 * (((rnd + i) * 3) % 5 - 2)
                rec = {
                    "round": rnd,
                    "corpus": "wikidata",
                    "threads": t,
                    "arm": arm,
                    "exit": 0,
                    "err": "",
                    "wall": base * f * noise,
                    "cpu": 3 * base * f * noise,
                    "hash": "abc",
                    "out": 100,
                    "observed_threads": PLAN9 if t == rfl.THREADS else PLAN3,
                }
                if arm in rfl.RUNFED:
                    rec["phases"] = phases(arm)
                recs.append(rec)
    return recs


def identity(rounds=rfl.ROUNDS):
    return {
        "schema": rfl.IDENTITY_SCHEMA,
        "head": HEAD,
        "tree_clean": True,
        "features": ["run-fed"],
        "profile": "release",
        "rustc": "rustc 1.99.0 (abc 2026-10-01)",
        "binary_sha256_before": "a" * 64,
        "binary_sha256_after": "a" * 64,
        "corpus_sha256_before": rfl.CORPUS_SHA256,
        "corpus_sha256_after": rfl.CORPUS_SHA256,
        "arms": rfl.REGISTERED_ARMS,
        "threads": [rfl.THREADS, rfl.REPORTED_THREADS],
        "rounds": rounds,
    }


def run(recs, ident=None, args=None, raw_lines=None):
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "runs.jsonl")
        with open(p, "w") as f:
            for r in recs:
                f.write(json.dumps(r) + "\n")
            for line in raw_lines or []:
                f.write(line + "\n")
        out = os.path.join(d, "rfl")
        cmd = [
            sys.executable,
            "-I",
            os.path.join(HERE, "run_fed_leader.py"),
            "report",
            p,
            "--out",
            out,
        ]
        if args is None:
            ip = os.path.join(d, "id.json")
            with open(ip, "w") as f:
                json.dump(identity() if ident is None else ident, f)
            args = ["--head", HEAD, "--identity", ip]
        r = subprocess.run(cmd + args, capture_output=True, text=True)
        raw = None
        if os.path.exists(out + ".json"):
            with open(out + ".json") as f:
                raw = json.load(f)
        return r.returncode, r.stdout + r.stderr, raw


class Admission(unittest.TestCase):
    def ok(self, recs, **kw):
        code, out, raw = run(recs, **kw)
        self.assertEqual(code, 0, out)
        self.assertIn("VALID", out)
        return out, raw

    def refused(self, recs, why, **kw):
        code, out, raw = run(recs, **kw)
        self.assertEqual(code, 2, out)
        self.assertIn("INVALID", out)
        self.assertNotIn("Traceback", out)
        self.assertNotIn("the reporter failed", out, "a crash, not a reason")
        self.assertIn(why, out)
        return out

    def test_clean_judged_passes_and_archives_every_record(self):
        out, raw = self.ok(clean())
        self.assertIn("VALID (JUDGED)", out)
        self.assertEqual(len(raw["raw"]), rfl.ROUNDS * 2 * len(rfl.REGISTERED_ARMS))
        self.assertEqual(raw["raw"][0]["arm"], rfl.REGISTERED_ARMS[0])

    def test_unjudged_any_rounds(self):
        out, _ = self.ok(clean(3), args=["--unjudged", "--rounds", "3"])
        self.assertIn("VALID (UNJUDGED)", out)
        self.assertIn("UNJUDGED): ", out)

    # The Researcher's probes (6730026367 §1), at the registered fifteen rounds
    def test_probe_absent_hash_and_out(self):
        recs = clean()
        for r in recs:
            del r["hash"], r["out"]
        self.refused(recs, "hash None")

    def test_probe_negative_cpu(self):
        recs = clean()
        for r in recs:
            r["cpu"] = -r["cpu"]
        self.refused(recs, "cpu -")

    def test_probe_negative_counts_timings_absent(self):
        recs = clean()
        for r in recs:
            if r["arm"] in rfl.RUNFED:
                tr = -1 if r["arm"] in rfl.RECORD_ARMS else 0
                r["phases"] = f"runs=-1 team_records={tr} fallbacks=0"
        out = self.refused(recs, "phases missing")
        self.assertIn("lead_end_ns", out)
        # and negative counts with every phase present
        recs = clean()
        for r in recs:
            if r["arm"] in rfl.RUNFED:
                r["phases"] = r["phases"].replace("runs=10", "runs=-1")
        self.refused(recs, "negative phases: runs")

    def test_probe_corpus_varies_by_round(self):
        recs = clean()
        for r in recs:
            r["corpus"] = f"wikidata-{r['round'] % 2}"
        self.refused(recs, "corpus 'wikidata-1'")

    # Missing measurements: INVALID with a reason, never a crash
    def test_missing_wall_or_cpu(self):
        for key in ("wall", "cpu"):
            recs = clean()
            del next(r for r in recs if r["arm"] == rfl.PRIMARY)[key]
            self.refused(recs, f"{key} None")

    def test_wrong_types(self):
        for key, val in (
            ("wall", "1.0"),
            ("out", True),
            ("exit", "0"),
            ("round", 1.0),
            ("cpu", float("nan")),
        ):
            recs = clean()
            recs[3][key] = val
            code, out, _ = run(recs)
            self.assertEqual(code, 2, (key, out))
            self.assertNotIn("Traceback", out)

    def test_not_json_and_not_objects(self):
        self.refused(clean(), "not JSON", raw_lines=["{oops"])
        self.refused(clean(), "not an object", raw_lines=["[1, 2]"])

    def test_missing_round(self):
        recs = [
            r
            for r in clean()
            if not (r["arm"] == rfl.PRIMARY and r["round"] == 1 and r["threads"] == 9)
        ]
        self.refused(recs, "rounds missing [1]")

    def test_failed_child(self):
        for key, val, why in (
            ("exit", 1, "exit 1"),
            ("exit", None, "exit None"),
            ("err", "boom", "err 'boom'"),
            ("broken", 1, "broken"),
        ):
            recs = clean()
            recs[3][key] = val
            self.refused(recs, why)

    def test_different_output(self):
        recs = clean()
        next(r for r in recs if r["arm"] == rfl.PRIMARY)["hash"] = "zzz"
        self.refused(recs, "output differs")

    def test_phases(self):
        for arm, old, new, why in (
            (rfl.PRIMARY, "fallbacks=0", "fallbacks=1", "1 fallbacks"),
            (rfl.CTL, "team_records=0", "team_records=2", "team_records 2 + team_column"),
            (rfl.PRIMARY, "lookups=30", "lookups=0", "no lookups"),
            (rfl.PRIMARY, "build_end_ns=9000000", "build_end_ns=0", "build_end_ns absent"),
            (rfl.PRIMARY, "stores=1500", "stores=999", "W >= n"),
            (rfl.PRIMARY, "leader_late=2", "leader_late=11", "leader_late 11 > runs"),
            (rfl.PRIMARY, "runs=10", "runs=10 bogus=1", "unregistered phases"),
            (rfl.PRIMARY, "runs=10", "runs=10 runs=10", "runs twice"),
            (rfl.PRIMARY, "runs=10", "runs=ten", "not name=integer"),
        ):
            recs = clean()
            r = next(r for r in recs if r["arm"] == arm)
            r["phases"] = r["phases"].replace(old, new, 1)
            self.refused(recs, why)
        recs = clean()
        del next(r for r in recs if r["arm"].endswith("-dir"))["phases"]
        self.refused(recs, "phases NoneType")

    def test_plan_and_order(self):
        recs = clean()
        next(r for r in recs if r["threads"] == 9)["observed_threads"] = PLAN3
        self.refused(recs, "not (3, 0, 6)")
        recs = clean()
        recs[0], recs[1] = recs[1], recs[0]
        self.refused(recs, "not ab's rotation")
        recs = clean()
        recs.append(dict(recs[0], arm="mibz2-pipe3-runfed-new"))
        self.refused(recs, "an unregistered arm")
        recs = clean()
        recs.append(dict(recs[0], threads=10))
        self.refused(recs, "threads 10")

    def test_zero_wall(self):
        recs = clean()
        for r in recs:
            if r["arm"] == rfl.PRIMARY:
                r["wall"] = 0.0
        self.refused(recs, "wall 0.0")

    # The registration's identity
    def test_identity(self):
        for key, val, why in (
            ("head", "f" * 40, "is not the registered"),
            ("tree_clean", False, "not clean"),
            ("binary_sha256_after", "b" * 64, "not one build"),
            ("corpus_sha256_before", "c" * 64, "not the registered"),
            ("features", [], "features"),
            ("profile", "debug", "profile"),
            ("rustc", None, "rustc None"),
            ("arms", rfl.REGISTERED_ARMS[::-1], "not the registered"),
            ("rounds", 3, "the identity's rounds 3"),
        ):
            ident = identity()
            ident[key] = val
            self.refused(clean(), why, ident=ident)

    def test_judged_needs_head_identity_and_fifteen_rounds(self):
        self.refused(clean(), "needs its identity", args=["--head", HEAD])
        self.refused(
            clean(),
            "not a full 40-character",
            args=["--head", HEAD[:12], "--identity", "/nonexistent"],
        )
        with tempfile.TemporaryDirectory() as d:
            ip = os.path.join(d, "id.json")
            with open(ip, "w") as f:
                json.dump(identity(3), f)
            self.refused(
                clean(3),
                "a judged run is 15 rounds",
                args=["--head", HEAD, "--identity", ip, "--rounds", "3"],
            )

    def test_no_retry_or_exemption_in_the_reporter(self):
        with open(os.path.join(HERE, "run_fed_leader.py")) as f:
            src = f.read()
        self.assertNotIn("re-dispatch", src)
        self.assertNotIn("diagnostic exemption", src)


class Reading(unittest.TestCase):
    def reading(self, factors):
        code, out, raw = run(clean(factors=factors))
        self.assertEqual(code, 0, out)
        return out, raw

    def test_researcher_fixture_is_no_clean_lead(self):
        # ctl ≈ 0.816 and hs ≈ 0.945 of the incumbent (6730026367 §2), dir too
        out, raw = self.reading(
            {rfl.CTL: 0.816, "mibz2-pipe3-runfed-hs": 0.945, rfl.PRIMARY: 0.945}
        )
        self.assertEqual(raw["reading"], "LEAD")
        self.assertIn("ATTRIBUTION FAILED", raw["attribution"])
        self.assertFalse(raw["fidelity"])
        # the contrasts reported apart: hs/ctl and dir/ctl ≈ 1.158, never the denominator
        for arm in ("mibz2-pipe3-runfed-hs", rfl.PRIMARY):
            g = raw["contrasts"][f"{arm}/{rfl.CTL}"]["wall"][0]
            self.assertAlmostEqual(g, 0.945 / 0.816, delta=0.01)
        w = raw["cells"][f"{rfl.PRIMARY}/{rfl.REF}@9"]["wall"][0]
        self.assertAlmostEqual(w, 0.945, delta=0.01)
        self.assertNotIn("attributed: ", out)

    def test_attributed_lead(self):
        out, raw = self.reading({rfl.CTL: 1.0, rfl.BOTH: 1.0, rfl.PRIMARY: 0.94})
        self.assertEqual(raw["reading"], "LEAD")
        self.assertTrue(raw["attribution"].startswith("attributed"))

    def test_stop_is_environment_specific(self):
        out, raw = self.reading({rfl.CTL: 1.0, rfl.PRIMARY: 1.03})
        self.assertEqual(raw["reading"], "STOP")
        self.assertIn("PROTOCOL §11", out)

    def test_primary_is_dir_secondary_hs(self):
        # hs fast, dir not: the primary reading is dir's, hs's only reported
        out, raw = self.reading({rfl.CTL: 1.0, "mibz2-pipe3-runfed-hs": 0.9, rfl.PRIMARY: 1.0})
        self.assertEqual(rfl.PRIMARY, "mibz2-pipe3-runfed-dir")
        self.assertEqual(raw["reading"], "STOP")
        self.assertIn("secondary (mibz2-pipe3-runfed-hs, never judged): LEAD", out)

    def test_geo_ci_refuses(self):
        self.assertIsNone(rfl.geo_ci([1.0]))
        self.assertIsNone(rfl.geo_ci([1.0, float("nan")]))
        self.assertIsNone(rfl.geo_ci([1.0, 0.0]))
        self.assertIsNone(rfl.geo_ci([1.0, -1.0]))
        self.assertIsNotNone(rfl.geo_ci([1.0, 1.1]))

    def test_arm_order_is_abs(self):
        self.assertEqual(rfl.arm_order(0, 3), [0, 1, 2])
        self.assertEqual(rfl.arm_order(1, 3), [1, 2, 0])
        self.assertEqual(rfl.arm_order(3, 3), [2, 1, 0])


if __name__ == "__main__":
    unittest.main()
