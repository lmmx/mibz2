"""bench/run_fed_leader_ws.py, the run-fed leader's workstation admission, on synthetic
bundles: a clean registered bundle (20 rounds at 10 threads, 140 records, the frozen
identities) is VALID and read against the incumbent; a tiny DRY bundle is admitted only as
DRY; and every failure it must refuse is refused, INVALID with its reason and never a crash:
wrong rounds, wrong threads, a missing arm, partial records, a mismatched digest (binary,
corpus, frozen source), an unfrozen or dirty run, a DRY bundle offered as evidence. The
occupancy captures are admitted apart, each refusal with its reason. §R10's additions: the
registered workload (length and hash) in every record, the bundle's own binaries rehashed,
the preflight (PASS, or REFUSED on each fact, never on a clock or temperature value), the
reading unchanged by anything the sysfs snapshots say, and the diagnostics (position,
predecessor, the endpoint labels' pair states as counts only).

    python3 bench/test_run_fed_leader_ws.py
"""

import copy
import hashlib
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
# the bundle's binaries (bin/), as the stand-in bundles write them
BIN = {"ab": b"stand-in ab\n", "occupancy": b"stand-in occupancy\n"}
BIN_SHA = {k: hashlib.sha256(v).hexdigest() for k, v in BIN.items()}
OUT, HASH = ws.WORKLOAD["out"], ws.WORKLOAD["hash"]


def snapshot(mhz, temp=45.0, t=0, cpus=ws.HOST_CPUS, unread=()):
    """One sysfs snapshot in ab's `machine_pre`/`machine_post` schema (examples/ab.rs's
    `State::json`): every CPU at `mhz`, those in `unread` unreadable"""
    khz = {str(c): (None if c in unread else int(mhz * 1000)) for c in range(cpus)}
    st = {
        "freq_mhz": float(mhz),
        "freq_cpus": cpus - len(unread),
        "governor": "powersave",
        "epp": "balance_performance",
        "turbo": True,
        "pkg_temp_c": temp,
        "turbo_src": "intel_pstate/no_turbo=0",
        "temp_src": "thermal_zone1 (x86_pkg_temp)",
        "t_unix_ns": t,
        "read_s": 0.0028,
        "cpu_khz": khz,
        "policies": [
            {
                "policy": f"policy{c}",
                "cpus": str(c),
                "driver": "intel_pstate",
                "governor": "powersave",
                "epp": "balance_performance",
            }
            for c in range(cpus)
        ],
        "platform_profile": None,
        "platform_profile_why": "no sys/firmware/acpi/platform_profile",
    }
    if unread:
        st["cpu_khz_why"] = {str(c): "unreadable" for c in unread}
    return st


FROZEN = "89abcdef0123456789abcdef0123456789abcdef"
# the decoders' own record of the threads they ran, at three teams of three
RAN = "mibz2-0 mibz2-1 mibz2-2 " + " ".join(f"mibz2-coop-p{1 + i % 2}" for i in range(6))
# D059's census misread: a helper caught only under its leader's inherited name
CENSUS_MISREAD = (
    "ab mibz2-0 mibz2-1 mibz2-1 mibz2-2 mibz2-coop-p1 mibz2-coop-p1 mibz2-coop-p1 "
    "mibz2-coop-p2 mibz2-coop-p2 mibz2-scan sampler"
)


def records(rounds=ws.ROUNDS, factors=None, threads=ws.THREADS, post=None):
    """Every arm once a round at 10 threads, in ab's rotation; each arm's wall
    `factors[arm]` times the incumbent's, with a little noise a round; each with its sysfs
    snapshots (`--sysfs-state`), the one after it at `post(seq)` MHz (4900 by default)"""
    factors = factors or {}
    post = post or (lambda seq: 4900)
    recs = []
    t = 1_794_229_198_000_000_000
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
                "termination": "success",
                "err": "",
                "wall": base * f * noise,
                "cpu": 3 * base * f * noise,
                "hash": HASH,
                "out": OUT,
                "observed_threads": ci.PLAN9,
                "ran_threads": RAN,
                "machine_pre": snapshot(800, t=t),
                "t_spawn_unix_ns": t + 3_000_000,
                "t_exit_unix_ns": t + 200_000_000,
                "machine_post": snapshot(post(len(recs)), t=t + 200_000_001),
            }
            t += 1_500_000_000
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
        "ab_sha256_before": BIN_SHA["ab"],
        "ab_sha256_after": BIN_SHA["ab"],
        "occupancy_sha256_before": BIN_SHA["occupancy"],
        "occupancy_sha256_after": BIN_SHA["occupancy"],
        "corpus_sha256_before": "d" * 64 if dry else rfl.CORPUS_SHA256,
        "corpus_sha256_after": "d" * 64 if dry else rfl.CORPUS_SHA256,
        "arms": ws.ARMS,
        "threads": [10],
        "rounds": ws.DRY_ROUNDS if dry else ws.ROUNDS,
        "ab_exit": 0,
        "sysfs_state": True,
        "preflight": "PASS",
        "workload": dict(ws.WORKLOAD),
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
    out = OUT
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
            "line": f"wall={walls[(r, m)]:.6f} cpu=0.059 rss_kb=33628 out={out} hash={HASH} err=- plan=pipe3x3-rf{sub}",
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


def bundle(d, recs, ident, raw_lines=None, caps=None, bins=True):
    if bins:
        os.makedirs(os.path.join(d, "bin"), exist_ok=True)
        for k, v in BIN.items():
            with open(os.path.join(d, "bin", k), "wb") as f:
                f.write(v)
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


def run(recs, ident, dry=False, raw_lines=None, cmd="admit", sub=None, caps=None, bins=True):
    with tempfile.TemporaryDirectory() as d:
        bundle(d, recs, ident, raw_lines, caps, bins)
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
                line=c["runs"][4]["line"].replace(f"hash={HASH}", "hash=0000")
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


# the preflight's inputs, as the recipe collects them (telemetry-check/20261109T125938Z's)
# `ab sample`'s CPU list on the i9: the allowed CPUs, grouped by core
I9_CPUS = ",".join(f"{c},{c + 10}" for c in range(10))
LSCPU_I9 = "Architecture: x86_64\nModel name:                           Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"


def host_env(**machine):
    """`ab env --facts`'s line on the registered i9 (examples/ab.rs `facts::line`): 20 CPUs
    allowed on 10 cores (siblings c and c + 10), the quota established unlimited through
    every level of the v2 hierarchy, and no counter probed"""
    m = {
        "cpufreq_driver": "intel_pstate",
        "governor": "powersave",
        "epp": "balance_performance",
        "turbo": True,
        "perf_event_paranoid": 4,
        "perf": None,
        "perf_why": "not probed: ab env --facts opens no counter",
    }
    m.update(machine)
    return {
        "facts": ws.FACTS_SCHEMA,
        "cpus": 20,
        "cpus_allowed": list(range(20)),
        "cores": 10,
        "core_cpus": [f"{c},{c + 10}" for c in range(10)],
        "cpu_quota": {
            "state": "unlimited",
            "cpus": None,
            "hierarchy": "v2",
            "levels": ["/sys/fs/cgroup", "/sys/fs/cgroup/user.slice"],
            "why": None,
        },
        "throttling": False,
        "psi_cpu": True,
        "spin_threads": 0,
        "perf_probed": False,
        "machine": m,
    }


OK_EXITS = "ab env --facts: exit 0\nlscpu: exit 0\nab sample: exit 0\n"


def preflight(env=None, lscpu=LSCPU_I9, samples=None, files=("env", "lscpu", "sample"), exits=OK_EXITS):
    env = host_env() if env is None else env
    samples = (
        samples
        if samples is not None
        else [{"sample": i, "cpus": I9_CPUS, "count": 3, "state": snapshot(800, temp=32.0, t=i)} for i in range(3)]
    )
    with tempfile.TemporaryDirectory() as d:
        os.makedirs(os.path.join(d, "preflight"))
        if "env" in files:
            with open(os.path.join(d, "preflight", "env.json"), "w") as f:
                f.write(env if isinstance(env, str) else json.dumps(env))
        if "lscpu" in files:
            with open(os.path.join(d, "preflight", "lscpu.txt"), "w") as f:
                f.write(lscpu)
        if "sample" in files:
            with open(os.path.join(d, "preflight", "sample.jsonl"), "w") as f:
                f.writelines(json.dumps(x) + "\n" for x in samples)
        if exits is not None:
            with open(os.path.join(d, "preflight", "exits.txt"), "w") as f:
                f.write(exits)
        out = os.path.join(d, "preflight")
        p = subprocess.run(
            [
                sys.executable,
                "-I",
                os.path.join(HERE, "run_fed_leader_ws.py"),
                "preflight",
                d,
                "--out",
                out,
            ],
            capture_output=True,
            text=True,
        )
        with open(out + ".json") as f:
            return p.returncode, p.stdout + p.stderr, json.load(f)


def diagnose(recs):
    with tempfile.TemporaryDirectory() as d:
        bundle(d, recs, identity())
        out = os.path.join(d, "diagnostics")
        p = subprocess.run(
            [
                sys.executable,
                "-I",
                os.path.join(HERE, "run_fed_leader_ws.py"),
                "diagnose",
                d,
                "--out",
                out,
            ],
            capture_output=True,
            text=True,
        )
        with open(out + ".json") as f:
            js = json.load(f)
        with open(out + ".md") as f:
            md = f.read()
        return p.returncode, p.stdout + p.stderr, js, md


class Revision10(unittest.TestCase):
    """§R10: the second attempt's additions, none of which touches the reading"""

    refused = Admission.refused
    valid = Admission.valid

    def test_every_record_must_decode_the_registered_workload(self):
        recs = records()
        for r in recs:  # (one output throughout, but not the registered one)
            r["out"], r["hash"] = 3030698, "6c462eda07e33ba8"
        self.refused(recs, identity(), "140 records did not decode the registered workload")
        ident = identity()
        ident["workload"] = {"out": 1, "hash": "x"}
        self.refused(records(), ident, "the identity's workload")
        # (a DRY check runs on any corpus: its workload is never held to the registered one)
        recs = records(rounds=ws.DRY_ROUNDS)
        for r in recs:
            r["out"], r["hash"] = 3030698, "6c462eda07e33ba8"
        self.valid(recs, identity(dry=True), dry=True)

    def test_the_bundled_binaries_are_rehashed(self):
        code, text, _ = run(records(), identity(), bins=False)
        self.assertEqual(code, 2, text)
        self.assertIn("bin/ab: not in the bundle", text)
        self.assertIn("bin/occupancy: not in the bundle", text)
        ident = identity()
        ident["occupancy_sha256_before"] = ident["occupancy_sha256_after"] = "e" * 64
        self.refused(records(), ident, "bin/occupancy: SHA-256")

    def test_the_preflight_and_the_snapshots_are_required_of_the_identity(self):
        for key, value, why in [
            ("preflight", "REFUSED", "the preflight 'REFUSED', not PASS"),
            ("sysfs_state", False, "did not record its snapshots"),
        ]:
            ident = identity()
            ident[key] = value
            self.refused(records(), ident, why)
        ident = identity()
        ident["schema"] = "run_fed_leader/workstation-identity/1"
        self.refused(records(), ident, "identity schema")

    def test_the_reading_is_the_full_data_whatever_the_snapshots_say(self):
        # 41 of 140 runs at twice the wall and CPU, each with a slow endpoint label, as the
        # first attempt's were: every pair still enters, unweighted, and the reading is the
        # same with the snapshots removed or contradicting the timing
        slow = {i for i in range(140) if (i * 7) % 24 < 7}
        self.assertEqual(len(slow), 41)
        recs = records(factors={ws.PRIMARY: 0.97}, post=lambda i: 2400 if i in slow else 4900)
        for i in slow:
            recs[i]["wall"] *= 2
            recs[i]["cpu"] *= 2
        _, js = self.valid(recs, identity())
        by = {(r["arm"], r["round"]): r for r in recs}
        for arm in ws.ARMS[1:]:
            # (every one of the 20 pairs, computed here: t(0.975, df 19) = 2.093)
            logs = [math.log(by[(arm, k)]["wall"] / by[(ws.REF, k)]["wall"]) for k in range(20)]
            m = sum(logs) / 20
            h = 2.093 * math.sqrt(sum((x - m) ** 2 for x in logs) / 19) / math.sqrt(20)
            got = js["cells"][f"{arm}/{ws.REF}"]["wall"]
            for a, b in zip(got, (math.exp(m), math.exp(m - h), math.exp(m + h))):
                self.assertAlmostEqual(a, b, places=4)
        self.assertIn("none excluded, replaced, stratified or clock-normalised", js["full_data"])
        for mutate in (
            lambda r: [r.pop(k) for k in ("machine_pre", "machine_post")],
            lambda r: r.update(machine_post=snapshot(800)),
            lambda r: r.update(machine_post=snapshot(5300, temp=99.0)),
        ):
            other = json.loads(json.dumps(recs))
            for r in other:
                mutate(r)
            _, js2 = self.valid(other, identity())
            for k in (
                "cells",
                "contrasts",
                "reading",
                "attribution",
                "fidelity",
                "secondary_readings",
            ):
                self.assertEqual(js2[k], js[k], k)

    def test_the_preflight_passes_the_registered_host_whatever_its_clock(self):
        code, text, js = preflight()
        self.assertEqual(code, 0, text)
        self.assertEqual(js["verdict"], "PASS")
        self.assertEqual(js["problems"], [])
        # (an idle 800 MHz, a hot package, a fast clock: no clock or temperature is held)
        hot = [
            {"sample": i, "cpus": I9_CPUS, "state": snapshot(5300 if i else 800, temp=97.0, t=i)} for i in range(3)
        ]
        code, text, _ = preflight(samples=hot)
        self.assertEqual(code, 0, text)

    def test_the_preflight_refuses_on_each_fact(self):
        bad_sample = [
            {"sample": i, "cpus": I9_CPUS, "state": snapshot(800, t=i, unread=(7,) if i == 1 else ())}
            for i in range(3)
        ]
        no_temp = [
            {
                "sample": i,
                "cpus": I9_CPUS,
                "state": {**snapshot(800, t=i), "pkg_temp_c": None, "pkg_temp_c_why": "no zone"},
            }
            for i in range(3)
        ]
        perf_gov = [
            {"sample": i, "cpus": I9_CPUS, "state": {**snapshot(800, t=i), "governor": "performance"}}
            for i in range(3)
        ]
        for kw, why in [
            ({"lscpu": "Model name: AMD EPYC 7763\n"}, "not the registered i9-10900K"),
            ({"env": {**host_env(), "cpus": 16}}, "16 CPUs allowed, not 20"),
            ({"env": {**host_env(), "cores": 8}}, "8 physical cores, not 10"),
            ({"env": {**host_env(), "cpu_quota": 4.0}}, "the cgroup CPU quota 4.0 is not established"),
            ({"env": host_env(governor="performance")}, "governor 'performance', not the declared"),
            ({"env": host_env(epp="performance")}, "epp 'performance'"),
            ({"env": host_env(turbo=False)}, "turbo False"),
            ({"env": host_env(cpufreq_driver="acpi-cpufreq")}, "cpufreq_driver 'acpi-cpufreq'"),
            ({"env": host_env(turbo=None)}, "turbo None"),
            ({"samples": bad_sample}, "sample 1: an unreadable scaling_cur_freq"),
            ({"samples": no_temp}, "package temperature None"),
            ({"samples": perf_gov}, "sample 0: governor 'performance'"),
            ({"samples": bad_sample[:2]}, "2 sysfs samples, not 3"),
            ({"files": ("lscpu", "sample")}, "the ab env does not load"),
            ({"files": ("env", "sample")}, "lscpu:"),
            ({"files": ("env", "lscpu")}, "the sysfs samples do not load"),
            ({"env": "not json"}, "the ab env does not load"),
            ({"env": []}, "ab env: not an object"),
        ]:
            with self.subTest(why=why):
                code, text, js = preflight(**kw)
                self.assertEqual(code, 1, text)
                self.assertEqual(js["verdict"], "REFUSED")
                self.assertIn(why, text)
                self.assertNotIn("Traceback", text)

    def test_the_preflight_admits_no_collector_that_did_not_complete(self):
        # (§R12, the Researcher's REVISE 6746559255: a complete positive payload followed by
        # a nonzero exit, a timeout or a kill is refused; so is a missing, malformed,
        # duplicated or unregistered status. The payloads here are the passing host's)
        for name in ("ab env --facts", "lscpu", "ab sample"):
            for code, how in ((17, ""), (124, " (timed out)"), (137, " (killed)"), (1, "")):
                with self.subTest(name=name, code=code):
                    exits = OK_EXITS.replace(f"{name}: exit 0", f"{name}: exit {code}")
                    rc, text, js = preflight(exits=exits)
                    self.assertEqual(rc, 1, text)
                    self.assertEqual(js["verdict"], "REFUSED")
                    self.assertIn(f"{name} exited {code}{how}: its output is not admitted", text)
        for exits, why in [
            (None, "the collectors' exit statuses (preflight/exits.txt) are missing"),
            ("", "ab env --facts: no exit status recorded"),
            (OK_EXITS.replace("lscpu: exit 0\n", ""), "lscpu: no exit status recorded"),
            (OK_EXITS + "lscpu: exit 0\n", "exits.txt: lscpu recorded twice"),
            (OK_EXITS.replace("ab sample: exit 0", "ab sample: exit -1"), "an unregistered or malformed line 'ab sample: exit -1'"),
            (OK_EXITS.replace("ab sample: exit 0", "ab sample: exit"), "an unregistered or malformed line"),
            (OK_EXITS + "ab env: exit 0\n", "an unregistered or malformed line 'ab env: exit 0'"),
        ]:
            with self.subTest(why=why):
                rc, text, js = preflight(exits=exits)
                self.assertEqual(rc, 1, text)
                self.assertEqual(js["verdict"], "REFUSED")
                self.assertIn(why, text)
                self.assertNotIn("Traceback", text)
        # (the successful control: every status 0, the same payloads, PASS)
        rc, text, js = preflight()
        self.assertEqual((rc, js["verdict"]), (0, "PASS"), text)

    def test_the_diagnostics_count_pair_states_and_carryover(self):
        recs = records(post=lambda i: 2400 if i % 3 == 0 else 4900)
        code, text, js, md = diagnose(recs)
        self.assertEqual(code, 0, text)
        self.assertEqual(js["status"], "complete")
        total = js["pair_states_total"]
        self.assertEqual(sum(total.values()), 120)
        self.assertEqual(total["unknown"], 0)
        # each pair's state is the incumbent's label, then the arm's, recomputed here
        lab = {(r["arm"], r["round"]): ("L" if i % 3 == 0 else "H") for i, r in enumerate(recs)}
        want = {"HH": 0, "HL": 0, "LH": 0, "LL": 0, "unknown": 0}
        for arm in ws.ARMS[1:]:
            for k in range(20):
                want[lab[(ws.REF, k)] + lab[(arm, k)]] += 1
        self.assertEqual(total, want)
        # position and predecessor, in run order
        rows = js["records"]
        self.assertIsNone(rows[0]["predecessor"])
        self.assertEqual(rows[8]["predecessor"], recs[7]["arm"])
        self.assertEqual(rows[8]["position"], 1)
        self.assertAlmostEqual(rows[8]["gap_s"], 1.303, places=3)
        for arm in ws.ARMS:
            self.assertEqual(sum(js["arm_by_position"][arm].values()), 20)
            self.assertEqual(sum(js["arm_by_predecessor"][arm].values()), 20)
        self.assertIn("counts only", text)
        self.assertIn("| all |", md)
        # never a verdict, and nothing it reads enters the admission
        self.assertIn("never judged", md)

    def test_unknown_states_are_counted_never_guessed(self):
        recs = records()
        del recs[0]["machine_post"]  # (r0's first record: the incumbent)
        recs[12]["machine_post"] = snapshot(4900, unread=(3,))  # (r1: two arms' pairs)
        recs[9]["machine_post"] = snapshot(4900, cpus=12)
        code, text, js, _ = diagnose(recs)
        self.assertEqual(code, 0, text)
        self.assertTrue(
            js["status"].startswith(
                "incomplete: 1 of 140 records lack a snapshot, 3 have no endpoint label"
            ),
            js["status"],
        )
        self.assertEqual(js["pair_states_total"]["unknown"], 6 + 2)
        self.assertEqual(sum(js["pair_states_total"].values()), 120)


class Revision11(unittest.TestCase):
    """§R11: the preflight's facts established, not presumed (the Researcher's REVIEW
    6745760175), and the diagnostics' descriptive additions (Q2, Q4, Q5)"""

    BASE = staticmethod(
        lambda: (
            host_env(),
            LSCPU_I9,
            [{"state": snapshot(4900, t=i)} for i in range(3)],
        )
    )

    def verdict(self, mutate):
        env, lscpu, samples = (copy.deepcopy(x) for x in self.BASE())
        mutate(env, samples)
        return ws.preflight_problems(env, lscpu, samples)

    def test_the_researchers_probe_now_refuses_each_mutation(self):
        # the probe (state/reports/run-fed-r10-6745615244/probe.py), each of its single-field
        # mutations of the positive fixture, called on the preflight itself
        self.assertEqual(self.verdict(lambda e, s: None), [])

        def turbo_1(e, s):
            e["machine"]["turbo"] = 1
            for x in s:
                x["state"]["turbo"] = 1

        def invented(e, s):
            for x in s:
                x["state"]["cpu_khz"] = {f"not-a-cpu-{i}": 4900000 for i in range(20)}

        for name, mutate, why in [
            ("missing_quota", lambda e, s: e.pop("cpu_quota"), "is not established"),
            ("float_cpu_counts", lambda e, s: e.update(cpus=20.0, cores=10.0), "20.0 CPUs"),
            ("integer_turbo", turbo_1, "turbo 1, not the declared True"),
            ("invented_cpu_ids", invented, "not keyed by the allowed CPUs"),
        ]:
            with self.subTest(name=name):
                bad = self.verdict(mutate)
                self.assertTrue(bad, name)
                self.assertTrue(any(why in b for b in bad), bad)

    def test_the_quota_must_be_established_unlimited(self):
        def quota(q):
            return lambda e, s: e.update(cpu_quota=q)

        unlimited = host_env()["cpu_quota"]
        # positive: established unlimited, v2 or v1
        self.assertEqual(self.verdict(quota(unlimited)), [])
        self.assertEqual(self.verdict(quota({**unlimited, "hierarchy": "v1"})), [])
        for name, q, why in [
            # unknown: not established, however it is written
            ("legacy null", None, "is not established"),
            ("legacy unlimited-or-unknown", "max", "is not established"),
            ("unknown", {**unlimited, "state": "unknown", "why": "cpu.max: EACCES"}, "EACCES"),
            ("no state", {k: v for k, v in unlimited.items() if k != "state"}, "not established"),
            ("unlimited, no levels", {**unlimited, "levels": []}, "without its hierarchy"),
            ("unlimited, no hierarchy", {**unlimited, "hierarchy": None}, "without its hierarchy"),
            ("unlimited, with a why", {**unlimited, "why": "partial"}, "without its hierarchy"),
            ("unlimited, a relative level", {**unlimited, "levels": ["cgroup"]}, "without"),
            # negative: established, and limited (the child under an unlimited root)
            ("limited", {**unlimited, "state": "limited", "cpus": 4.0}, "a cgroup CPU quota of 4.0"),
            ("limited at 20", {**unlimited, "state": "limited", "cpus": 20.0}, "quota of 20.0"),
        ]:
            with self.subTest(name=name):
                bad = self.verdict(quota(q))
                self.assertTrue(any(why in b for b in bad), bad)

    def test_types_are_exact_and_the_facts_are_the_passive_collectors(self):
        for name, mutate, why in [
            ("bool cores", lambda e, s: e.update(cores=True), "True physical cores"),
            ("string cpus", lambda e, s: e.update(cpus="20"), "'20' CPUs"),
            ("float khz", lambda e, s: s[0]["state"]["cpu_khz"].update({"3": 4900000.0}),
             "an unreadable scaling_cur_freq"),
            ("bool khz", lambda e, s: s[1]["state"]["cpu_khz"].update({"3": True}),
             "an unreadable scaling_cur_freq"),
            ("sample turbo 1", lambda e, s: s[2]["state"].update(turbo=1), "sample 2: turbo 1"),
            ("active env", lambda e, s: e.pop("facts"), "not the passive collector's"),
            ("perf probed", lambda e, s: e.update(perf_probed=True), "not the passive reading"),
            ("spun", lambda e, s: e.update(spin_threads=40), "not the passive reading"),
            ("no allowed list", lambda e, s: e.update(cpus_allowed=None), "no established allowed"),
            ("float ids", lambda e, s: e.update(cpus_allowed=[float(c) for c in range(20)]),
             "no established allowed"),
            ("duplicate ids", lambda e, s: e.update(cpus_allowed=[0] + list(range(19))),
             "not distinct"),
            ("19 ids", lambda e, s: e.update(cpus_allowed=list(range(19))), "19 allowed CPU ids"),
            # (another 20 CPUs than the samples read: the keys bind to these ids)
            ("other ids", lambda e, s: e.update(
                cpus_allowed=list(range(4, 24)),
                core_cpus=[f"{c},{c + 10}" for c in range(4, 14)]), "not keyed by the allowed"),
            ("a padded key", lambda e, s: s[0]["state"].update(
                cpu_khz={**{str(c): 4900000 for c in range(1, 20)}, "00": 4900000}),
             "not keyed by the allowed"),
            ("cores not a partition", lambda e, s: e.update(core_cpus=["0,1"] * 10),
             "do not partition"),
            # (Q3: the declared settings policy by policy, every allowed CPU covered once)
            ("a policy off", lambda e, s: s[1]["state"]["policies"][7].update(
                governor="performance"), "sample 1: policy 'policy7': governor 'performance'"),
            ("a policy unread", lambda e, s: s[0]["state"]["policies"][3].update(epp=None),
             "policy 'policy3': epp None"),
            ("a CPU in no policy", lambda e, s: s[2]["state"]["policies"].pop(),
             "CPUs [19] are not each in exactly one"),
            ("a CPU in two", lambda e, s: s[2]["state"]["policies"][0].update(cpus="0-1"),
             "CPUs [1] are not each in exactly one"),
            ("no policies", lambda e, s: s[0]["state"].update(policies=[]),
             "no cpufreq policies listed"),
        ]:
            with self.subTest(name=name):
                bad = self.verdict(mutate)
                self.assertTrue(any(why in b for b in bad), bad)

    def test_every_record_must_carry_a_successful_termination(self):
        # (the producer's termination, bench 3.5.0: output before a failure is refused, and
        # a record without its termination is refused)
        for val in ("exit 17", "signal 9", None):
            with self.subTest(termination=val):
                recs = records()
                if val is None:
                    del recs[5]["termination"]
                else:
                    recs[5]["termination"] = val
                Admission.refused(self, recs, identity(), f"termination {val!r}")

    def test_lag1_is_the_registered_formula_and_undefined_when_constant(self):
        xs = [0.01, -0.02, 0.03, 0.0, 0.015, -0.01]
        m = sum(xs) / len(xs)
        num_ = sum((xs[r] - m) * (xs[r - 1] - m) for r in range(1, len(xs)))
        self.assertAlmostEqual(ws.lag1(xs), num_ / sum((x - m) ** 2 for x in xs), places=12)
        self.assertAlmostEqual(ws.lag1([1.0, -1.0] * 10), -0.95, places=12)
        self.assertIsNone(ws.lag1([0.02] * 20))  # constant: undefined, never zero
        self.assertIsNone(ws.lag1([0.01, None, 0.02]))
        self.assertIsNone(ws.lag1([]))

    def test_the_diagnostics_keep_raw_readings_lag1_and_mark_the_objective(self):
        recs = records(post=lambda i: 2400 if i % 3 == 0 else 4900)
        code, text, js, md = diagnose(recs)
        self.assertEqual(code, 0, text)
        self.assertEqual(js["diagnostic_objective"], "met")
        for arm in ws.ARMS:
            self.assertEqual(sum(js["post_labels"][arm].values()), 20)
        row = js["records"][3]
        self.assertEqual(len(row["post_cpu_khz"]), ws.HOST_CPUS)
        self.assertIn("post_delay_s", row)
        self.assertIn("post_read_s", row)
        by = {(r["arm"], r["round"]): r for r in recs}
        for arm in ws.ARMS[1:]:
            xs = [math.log(by[(arm, k)]["wall"] / by[(ws.REF, k)]["wall"]) for k in range(20)]
            want = ws.lag1(xs)
            got = js["lag1"][arm]["wall"]
            self.assertTrue((got is None and want is None) or abs(got - want) < 1e-12, arm)
        self.assertIn("Lag-1 autocorrelation", md)
        self.assertIn("never zero", js["lag1_rule"])
        # Q4: an incomplete record marks the objective unmet, and the timing stands
        recs = records()
        for r in recs[:100]:
            del r["machine_post"]
        code, text, js, _ = diagnose(recs)
        self.assertEqual(code, 0, text)
        self.assertEqual(js["diagnostic_objective"], "unmet")
        self.assertIn("unmet", text)
        Admission.valid(self, recs, identity())


if __name__ == "__main__":
    unittest.main()
