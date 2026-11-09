"""bench/run_fed_leader_ws.py, the run-fed leader's workstation admission, on synthetic
bundles: a clean registered bundle (20 rounds at 10 threads, 140 records, the frozen
identities) is VALID and read against the incumbent; a tiny DRY bundle is admitted only as
DRY; and every failure it must refuse is refused, INVALID with its reason and never a crash:
wrong rounds, wrong threads, a missing arm, partial records, a mismatched digest (binary,
corpus, frozen source), an unfrozen or dirty run, a DRY bundle offered as evidence. The
occupancy captures are admitted apart, each refusal with its reason.

    python3 bench/test_run_fed_leader_ws.py
"""

import json
import math
import os
import re
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import run_fed_leader as rfl  # noqa: E402
import run_fed_leader_ws as ws  # noqa: E402
import test_run_fed_leader as ci  # noqa: E402

HEAD = "0123456789abcdef0123456789abcdef01234567"
FROZEN = "89abcdef0123456789abcdef0123456789abcdef"
# the decoders' own record of the threads they ran, at three teams of three
RAN = "mibz2-0 mibz2-1 mibz2-2 " + " ".join(f"mibz2-coop-p{1 + i % 2}" for i in range(6))
# D059's census misread: a helper caught only under its leader's inherited name
CENSUS_MISREAD = (
    "ab mibz2-0 mibz2-1 mibz2-1 mibz2-2 mibz2-coop-p1 mibz2-coop-p1 mibz2-coop-p1 "
    "mibz2-coop-p2 mibz2-coop-p2 mibz2-scan sampler"
)


def records(rounds=ws.ROUNDS, factors=None, threads=ws.THREADS):
    """Every arm once a round at 10 threads, in ab's rotation; each arm's wall
    `factors[arm]` times the incumbent's, with a little noise a round"""
    factors = factors or {}
    recs = []
    for rnd in range(rounds):
        base = 1.0 + 0.01 * ((rnd * 7) % 5)
        for i in rfl.arm_order(rnd, len(ws.ARMS)):
            arm = ws.ARMS[i]
            f = factors.get(arm, 1.0 + 0.002 * i)
            noise = 1 + 0.003 * (((rnd + i) * 3) % 5 - 2)
            rec = {
                "round": rnd,
                "corpus": "wikidata",
                "threads": threads,
                "arm": arm,
                "exit": 0,
                "err": "",
                "wall": base * f * noise,
                "cpu": 3 * base * f * noise,
                "hash": "6c462eda07e33ba8",
                "out": 3030698,
                "observed_threads": ci.PLAN9,
                "ran_threads": RAN,
            }
            if arm in rfl.RUNFED:
                rec["phases"] = ci.phases(arm)
            recs.append(rec)
    return recs


def identity(dry=False):
    src = {
        k: {"path": p, "sha256": "c" * 64, "at_frozen": None if dry else "c" * 64}
        for k, p in [
            ("recipe", ".just/bench.just"),
            ("admission", "bench/run_fed_leader_ws.py"),
            ("reporter", "bench/run_fed_leader.py"),
            ("registration", "docs/journal/2026-11-09-run-fed-leader.md"),
        ]
    }
    return {
        "schema": ws.IDENTITY_SCHEMA,
        "dry": dry,
        "head": HEAD,
        "frozen": None if dry else FROZEN,
        "inputs_match_frozen": not dry,
        "tree_clean": True,
        "sources": src,
        "features": ["run-fed"],
        "occupancy_features": ["trace", "run-fed"],
        "profile": "release",
        "rustc": "rustc 1.97.0 (2d8144b78 2026-07-07)",
        "ab_sha256_before": "a" * 64,
        "ab_sha256_after": "a" * 64,
        "occupancy_sha256_before": "b" * 64,
        "occupancy_sha256_after": "b" * 64,
        "corpus_sha256_before": "d" * 64 if dry else rfl.CORPUS_SHA256,
        "corpus_sha256_after": "d" * 64 if dry else rfl.CORPUS_SHA256,
        "arms": ws.ARMS,
        "threads": [10],
        "rounds": ws.DRY_ROUNDS if dry else ws.ROUNDS,
        "ab_exit": 0,
    }


T975 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228]


def interval(ratios):
    """The geometric mean and 95% t-interval of `ratios`, computed here, independently of
    both the admission and the instrument"""
    logs = [math.log(x) for x in ratios]
    n = len(logs)
    m = sum(logs) / n
    sd = math.sqrt(sum((x - m) ** 2 for x in logs) / (n - 1))
    h = T975[n - 2] * sd / math.sqrt(n)
    return {
        "ratio": round(math.exp(m), 6),
        "lo": round(math.exp(m - h), 6),
        "hi": round(math.exp(m + h), 6),
    }


def capture(sub, dry=False, ident=None, slow=None):
    """An occupancy capture in examples/occupancy.rs's schema: its raw grid (each round's
    off, timeline and full run, each its own wall), and each traced mode's perturbation
    computed from it; `slow` a mode's walls scaled up (a perturbing mode)"""
    rounds = ws.DRY_OCC_ROUNDS if dry else ws.OCC_ROUNDS
    ident = ident or identity(dry)
    mode_sum = {"complete_rounds": rounds, "blocks": 4, "pipe.n.run": 4, "f.w.recorded": 3}
    out = 3030698
    walls = {}
    for r in range(rounds):
        base = 0.027 * (1 + 0.04 * ((r * 5) % 3))
        for k, m in enumerate(("off", "timeline", "full")):
            jitter = 1 + 0.01 * (((r + 2 * k) * 7) % 5 - 2)
            # (to the line's six decimals: the instrument reads its clocks from its lines)
            walls[(r, m)] = round(base * jitter * (1.3 if m == slow else 1.0), 6)
    runs = [
        {
            "round": r,
            "threads": 10,
            "slots": 12,
            "placement": "",
            "mode": m,
            "line": f"wall={walls[(r, m)]:.6f} cpu=0.059 rss_kb=33628 out={out} hash=6c462eda07e33ba8 err=- plan=pipe3x3-rf{sub}",
        }
        for r in range(rounds)
        for m in ("off", "timeline", "full")
    ]
    pert = {}
    for m in ("timeline", "full"):
        # (throughput over off's, round by round: wall off over wall mode)
        iv = interval([walls[(r, "off")] / walls[(r, m)] for r in range(rounds)])
        verdict = ws.OCC_NO_PERTURBATION if iv["lo"] <= 1 <= iv["hi"] else ws.OCC_PERTURBS
        pert[m] = {"throughput_ratio": iv, "verdict": verdict}
    return {
        "tool": "examples/occupancy.rs (#39)",
        "format": 1,
        "meta": {"commit": ident["head"], "corpus_sha256": ident["corpus_sha256_before"]},
        "settings": {
            "threads": [10],
            "rounds": rounds,
            "teams": f"pipelined teams of 3, forced, levers: rf{sub}",
        },
        "parity": True,
        "results": [
            {
                "threads": 10,
                "timeline": dict(mode_sum),
                "full": dict(mode_sum),
                "perturbation": pert,
            }
        ],
        "runs": runs,
    }


def bundle(d, recs, ident, raw_lines=None, caps=None):
    with open(os.path.join(d, "runs.jsonl"), "w") as f:
        for r in recs:
            f.write(json.dumps(r) + "\n")
        for line in raw_lines or []:
            f.write(line + "\n")
    if ident is not None:
        with open(os.path.join(d, "identity.json"), "w") as f:
            json.dump(ident, f)
    for sub, cap in (caps or {}).items():
        with open(os.path.join(d, f"occupancy-{sub}.json"), "w") as f:
            json.dump(cap, f)


def run(recs, ident, dry=False, raw_lines=None, cmd="admit", sub=None, caps=None):
    with tempfile.TemporaryDirectory() as d:
        bundle(d, recs, ident, raw_lines, caps)
        out = os.path.join(d, "out")
        argv = [sys.executable, "-I", os.path.join(HERE, "run_fed_leader_ws.py"), cmd, d]
        argv += [sub] if sub else []
        argv += (["--dry"] if dry else []) + ["--out", out]
        p = subprocess.run(argv, capture_output=True, text=True)
        js = None
        if os.path.exists(out + ".json"):
            with open(out + ".json") as f:
                js = json.load(f)
        return p.returncode, p.stdout + p.stderr, js


class Admission(unittest.TestCase):
    def valid(self, *a, **k):
        code, text, js = run(*a, **k)
        self.assertEqual(code, 0, text)
        self.assertTrue(text.startswith("VALID"), text)
        return text, js

    def refused(self, recs, ident, why, dry=False, raw_lines=None):
        code, text, js = run(recs, ident, dry=dry, raw_lines=raw_lines)
        self.assertEqual(code, 2, text)
        self.assertTrue(text.startswith("INVALID"), text)
        self.assertIn(why, text)
        self.assertNotIn("Traceback", text)
        self.assertNotIn("the admission failed", text)
        return text

    def test_a_clean_registered_bundle_is_valid_and_judged(self):
        recs = records(factors={ws.PRIMARY: 0.95})
        self.assertEqual(len(recs), 140)
        text, js = self.valid(recs, identity())
        self.assertIn(
            "VALID (JUDGED (workstation)): 20 rounds at 10 threads, 140 timed records", text
        )
        self.assertEqual(js["df"], 19)
        self.assertEqual(js["reading"], "LEAD")
        self.assertTrue(js["attribution"].startswith("attributed"), js["attribution"])
        self.assertEqual(sorted(js["secondary_readings"]), sorted(rfl.SECONDARY))

    def test_the_reading_is_against_the_incumbent_with_attribution(self):
        # ctl far from the incumbent: a LEAD whose attribution fails, as §6 says
        f = {ws.PRIMARY: 0.95, rfl.CTL: 0.90}
        _, js = self.valid(records(factors=f), identity())
        self.assertEqual(js["reading"], "LEAD")
        self.assertTrue(js["attribution"].startswith("ATTRIBUTION FAILED"), js["attribution"])
        _, js = self.valid(records(factors={ws.PRIMARY: 1.02}), identity())
        self.assertEqual(js["reading"], "STOP")

    def test_a_tiny_dry_bundle_is_admitted_only_as_dry(self):
        recs = records(rounds=ws.DRY_ROUNDS)
        text, js = self.valid(recs, identity(dry=True), dry=True)
        self.assertIn("DRY (not evidence)", text)
        self.assertNotIn("JUDGED", text)
        # never as evidence
        self.refused(recs, identity(dry=True), "a DRY run is not evidence")
        # nor a registered bundle as a DRY one
        self.refused(records(), identity(), "a DRY admission of a registered run", dry=True)

    def test_wrong_rounds_are_refused(self):
        self.refused(records(rounds=19), identity(), "rounds missing [19]")
        ident = identity()
        ident["rounds"] = 15
        self.refused(records(), ident, "the identity's rounds 15, not 20")
        self.refused(records(rounds=21), identity(), "round 20, not 0..19")

    def test_wrong_threads_are_refused(self):
        self.refused(records(threads=9), identity(), "threads 9, not 10")
        ident = identity()
        ident["threads"] = [9, 3]
        self.refused(records(), ident, "threads [9, 3], not [10]")
        recs = records()
        recs[5]["ran_threads"] = "mibz2-0 mibz2-coop-p1 mibz2-coop-p2"
        self.refused(recs, identity(), "ran (1, 0, 2)")

    def test_a_missing_arm_is_refused(self):
        recs = [r for r in records() if r["arm"] != rfl.REG]
        self.refused(recs, identity(), f"{rfl.REG}: rounds missing")
        ident = identity()
        ident["arms"] = ws.ARMS[:-1]
        self.refused(records(), ident, "not the registered")

    def test_partial_records_are_refused(self):
        recs = records()
        del recs[37]
        self.refused(recs, identity(), "139 timed records admitted, not 140")
        self.refused(records(), identity(), "line 141: not JSON", raw_lines=['{"round": 3, "arm'])
        recs = records()
        recs[3] = {"broken": True, **{k: recs[3][k] for k in ("round", "threads", "arm")}}
        self.refused(recs, identity(), "a broken line")
        ident = identity()
        ident["ab_exit"] = 101
        self.refused(records(), ident, "ab run exited 101")
        with tempfile.TemporaryDirectory() as d:
            bundle(d, records(), None)
            p = subprocess.run(
                [sys.executable, "-I", os.path.join(HERE, "run_fed_leader_ws.py"), "admit", d],
                capture_output=True,
                text=True,
            )
            self.assertEqual(p.returncode, 2)
            self.assertIn("the identity does not load", p.stdout)

    def test_mismatched_digests_are_refused(self):
        for key, why in [
            ("ab_sha256_after", "the ab binary's SHA-256"),
            ("occupancy_sha256_after", "the occupancy binary's SHA-256"),
            ("corpus_sha256_after", "not one corpus"),
        ]:
            ident = identity()
            ident[key] = "e" * 64
            self.refused(records(), ident, why)
        ident = identity()
        ident["corpus_sha256_before"] = ident["corpus_sha256_after"] = "e" * 64
        self.refused(records(), ident, "not the registered 82f65ace")
        for k in ws.SOURCES:
            ident = identity()
            ident["sources"][k]["at_frozen"] = "f" * 64
            self.refused(records(), ident, f"source {k}")
        recs = records()
        recs[20]["hash"] = "different"
        self.refused(recs, identity(), "output differs from the incumbent's")

    def test_an_unfrozen_or_dirty_run_is_refused(self):
        ident = identity()
        ident["frozen"] = None
        self.refused(records(), ident, "the frozen commit None")
        ident = identity()
        ident["inputs_match_frozen"] = False
        self.refused(records(), ident, "do not match the frozen commit's")
        ident = identity()
        ident["tree_clean"] = False
        self.refused(records(), ident, "not clean")
        ident = identity()
        ident["features"] = []
        self.refused(records(), ident, "features []")

    def test_bad_evidence_inside_a_record_is_refused(self):
        # (the CI reporter's typed checks, reused at 10 threads)
        recs = records()
        recs[9]["wall"] = -1
        self.refused(recs, identity(), "wall -1")
        recs = records()
        i = next(i for i, r in enumerate(recs) if r["arm"] == ws.PRIMARY)
        recs[i]["phases"] = recs[i]["phases"].replace("fallbacks=0", "fallbacks=1")
        self.refused(recs, identity(), "fallbacks")
        recs = records()
        recs[2]["corpus"] = "other"
        self.refused(recs, identity(), "corpus 'other'")

    def test_a_census_misread_is_no_false_invalid(self):
        # (D059: the census caught a helper only under its leader's inherited name, (4, 0, 5);
        # the decoders' record, complete by construction, ran (3, 0, 6): VALID, the
        # misread reported, never judged)
        recs = records()
        for i in (4, 77, 139):
            recs[i]["observed_threads"] = CENSUS_MISREAD
        self.assertEqual(rfl.plan_of(CENSUS_MISREAD), (4, 0, 5))
        text, js = self.valid(recs, identity())
        self.assertEqual(len(js["census_misreads"]), 3)
        self.assertIn("the census misread 3 plans", text)

    def test_a_plan_not_run_is_refused_whatever_the_census(self):
        # a helper recorded under a leader's name (a delayed or inherited name: the record
        # takes a thread's own name, so this is a decoder that ran four leaders)
        recs = records()
        recs[10]["ran_threads"] = RAN.replace("mibz2-coop-p2", "mibz2-1", 1)
        self.refused(recs, identity(), "ran (4, 0, 5)")
        # an early exit: a team's helpers gone before their first task, never recorded
        recs = records()
        recs[11]["ran_threads"] = "mibz2-0 mibz2-1 mibz2-2 " + " ".join(["mibz2-coop-p1"] * 4)
        self.refused(recs, identity(), "ran (3, 0, 4)")
        # an absent helper
        recs = records()
        recs[12]["ran_threads"] = RAN.rsplit(" ", 1)[0]
        self.refused(recs, identity(), "ran (3, 0, 5)")
        # no record at all (an older harness): refused, never taken from the census
        recs = records()
        del recs[13]["ran_threads"]
        self.refused(recs, identity(), "no ran_threads")
        # and the census itself still required present
        recs = records()
        del recs[14]["observed_threads"]
        self.refused(recs, identity(), "no observed_threads")

    def test_the_rotation_is_enforced(self):
        recs = records()
        recs[0], recs[1] = recs[1], recs[0]
        self.refused(recs, identity(), "not ab's rotation")


class Occupancy(unittest.TestCase):
    def occ(self, sub="dir", cap=None, dry=False, ident=None):
        ident = ident or identity(dry)
        recs = records(rounds=ws.DRY_ROUNDS if dry else ws.ROUNDS)
        cap = cap if cap is not None else capture(sub, dry, ident)
        return run(recs, ident, dry=dry, cmd="admit-occupancy", sub=sub, caps={sub: cap})

    def test_both_captures_are_admitted_apart(self):
        for sub in ws.OCC_SUBS:
            code, text, js = self.occ(sub)
            self.assertEqual(code, 0, text)
            self.assertIn(f"VALID ({sub} capture)", text)
            self.assertTrue(js["interpretable"])
        code, text, _ = self.occ("ctl", dry=True)
        self.assertEqual(code, 0, text)
        self.assertIn("DRY", text)

    def test_a_perturbing_capture_is_admitted_and_marked(self):
        cap = capture("dir", slow="full")
        self.assertEqual(cap["results"][0]["perturbation"]["full"]["verdict"], ws.OCC_PERTURBS)
        code, text, js = self.occ("dir", cap)
        self.assertEqual(code, 0, text)
        self.assertIn("not to be interpreted", text)
        self.assertEqual(js["perturbing_modes"], ["full"])
        self.assertFalse(js["interpretable"])

    def test_a_real_schema_capture_with_numeric_intervals_is_interpretable(self):
        cap = capture("ctl")
        for m in ("timeline", "full"):
            iv = cap["results"][0]["perturbation"][m]["throughput_ratio"]
            self.assertTrue(all(isinstance(iv[k], float) for k in ("ratio", "lo", "hi")))
        code, text, js = self.occ("ctl", cap)
        self.assertEqual(code, 0, text)
        self.assertTrue(js["interpretable"])
        self.assertEqual(js["perturbing_modes"], [])

    def test_missing_unknown_or_insufficient_evidence_is_never_interpretable(self):
        # (the Researcher's five, 6733680620: each was VALID and interpretable)
        def no_perturbation(c):
            del c["results"][0]["perturbation"]

        def unknown_verdict(c):
            c["results"][0]["perturbation"]["full"]["verdict"] = "looks fine"

        def too_few_rounds(c):
            c["results"][0]["perturbation"]["timeline"] = {
                "throughput_ratio": {"ratio": 1.0, "lo": None, "hi": None},
                "verdict": "too few rounds",
            }

        def one_run_eighteen_times(c):
            c["runs"] = [dict(c["runs"][0]) for _ in range(18)]

        def every_clock_nan(c):
            for run in c["runs"]:
                run["line"] = re.sub(r"wall=\S+", "wall=NaN", run["line"])

        for mutate, why in [
            (no_perturbation, "perturbation timeline: absent"),
            (unknown_verdict, "verdict 'looks fine'"),
            (too_few_rounds, "not finite"),
            (one_run_eighteen_times, "twice"),
            (every_clock_nan, "not a finite positive clock"),
        ]:
            cap = capture("dir")
            mutate(cap)
            code, text, js = self.occ("dir", cap)
            self.assertEqual(code, 2, f"{mutate.__name__}: {text}")
            self.assertIn(why, text, mutate.__name__)
            self.assertNotIn("interpretable", js or {}, mutate.__name__)
        cap = capture("dir")
        cap["runs"] = [dict(cap["runs"][0]) for _ in range(18)]
        _, text, _ = self.occ("dir", cap)
        self.assertIn("the grid lacks 17 of 18", text)

    def test_a_misreported_or_misclassified_interval_is_refused(self):
        cap = capture("dir")
        cap["results"][0]["perturbation"]["full"]["throughput_ratio"]["ratio"] = 0.5
        code, text, _ = self.occ("dir", cap)
        self.assertEqual(code, 2, text)
        self.assertIn("not the grid's", text)
        cap = capture("dir", slow="full")
        cap["results"][0]["perturbation"]["full"]["verdict"] = ws.OCC_NO_PERTURBATION
        code, text, _ = self.occ("dir", cap)
        self.assertEqual(code, 2, text)
        self.assertIn("verdict", text)

    def refused(self, why, mutate, sub="dir"):
        cap = capture(sub)
        mutate(cap)
        code, text, _ = self.occ(sub, cap)
        self.assertEqual(code, 2, text)
        self.assertIn(why, text)
        self.assertNotIn("the admission failed", text)

    def test_capture_refusals(self):
        self.refused(
            "teams",
            lambda c: c["settings"].update(teams="pipelined teams of 3, forced, levers: rfctl"),
        )
        self.refused("threads [9]", lambda c: c["settings"].update(threads=[9]))
        self.refused("rounds 3, not 6", lambda c: c["settings"].update(rounds=3))
        self.refused(
            "complete rounds", lambda c: c["results"][0]["timeline"].update(complete_rounds=5)
        )
        self.refused(
            "buffers recorded", lambda c: c["results"][0]["full"].update({"f.w.recorded": 0})
        )
        self.refused("parity", lambda c: c.update(parity=False))
        self.refused(
            "not the incumbent's",
            lambda c: c["runs"][4].update(
                line=c["runs"][4]["line"].replace("hash=6c46", "hash=0000")
            ),
        )
        self.refused("the grid lacks 1 of 18", lambda c: c["runs"].pop())
        self.refused(
            "err",
            lambda c: c["runs"][0].update(
                line=c["runs"][0]["line"].replace("err=-", "err=Truncated")
            ),
        )
        self.refused("commit", lambda c: c["meta"].update(commit="x"))
        self.refused("corpus is not the timing's", lambda c: c["meta"].update(corpus_sha256="x"))
        self.refused("not one result", lambda c: c.update(results=[]))
        code, text, _ = run(
            records(), identity(), cmd="admit-occupancy", sub="hs", caps={"hs": capture("hs")}
        )
        self.assertEqual(code, 2, text)
        self.assertIn("sub-arm 'hs'", text)
        code, text, _ = run(records(), identity(), cmd="admit-occupancy", sub="dir")
        self.assertEqual(code, 2, text)
        self.assertIn("dir capture does not load", text)


if __name__ == "__main__":
    unittest.main()
