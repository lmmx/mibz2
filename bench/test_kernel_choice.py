"""bench/kernel_choice.py on synthetic runs: each verdict where the registration puts it, and every
defect the validity checks guard against leaves no verdict. bench/test_scanner_plumbing.py's
cases (#850) with the counts and the rule moved, and the plan check in place of its workers
check: each build's probe must be its registered plan, and its timed runs' workers its probe's.

    python3 bench/test_kernel_choice.py
"""

import io
import json
import shutil
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import kernel_choice as kc  # noqa: E402

R = 16
BASE_EXE = "/w/target/kernel-choice/build-base/target/release/examples/ab"
ENV = {
    "cpus": 20,
    "cores": 10,
    "core_cpus": [f"{c},{c + 10}" for c in range(10)],
    "cpu_quota": None,
}
LSCPU = "Architecture: x86_64\nModel name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
COUNTS = (3, 4, 5, 7, 8, 10, 12)
# the predicted pattern (journal §76): faster at 5, 7, 10 and 12, level at 3, 4 and 8
GOOD = {5: 0.83, 7: 0.62, 10: 0.95, 12: 0.95}


def records(arm="mibz2", step=None, mem=1.0, noise=0.01, workers=None, head_plan=None):
    """A complete run: `step` maps a count to the head's wall over the base's (default level);
    `mem` the head's peak memory over the base's; workers as each build's registered plan unless
    `workers` maps (build, count) to another. Rounds alternate the noise above and below"""
    step = step or {}
    workers = workers or {}
    head_plan = head_plan or kc.HEAD_PLAN
    out = []
    for r in range(R):
        for t in COUNTS:
            for which, plan in (("base", kc.BASE_PLAN), ("head", head_plan)):
                wall, rss = 1.0, 500_000
                if which == "head":
                    wall = step.get(t, 1.0) * (1 + (noise if r % 2 else -noise))
                    rss = int(rss * mem)
                n = workers.get((which, t), plan[t][0])
                names = ["ab"] + [f"mibz2-{i}" for i in range(n)] + ["mibz2-read", "mibz2-scan"]
                out.append(
                    {
                        "round": r,
                        "corpus": "wikidata-x4",
                        "threads": t,
                        "arm": f"{arm}@{BASE_EXE}" if which == "base" else arm,
                        "cpus": "all",
                        "termination": "success",
                        "wall": wall,
                        "cpu": wall * t,
                        "rss_kb": rss,
                        "out": kc.WIKIDATA_X4_OUT,
                        "hash": "0123456789abcdef",
                        "err": "",
                        "observed_threads": " ".join(names + ["sampler"]),
                    }
                )
    return out


def plan(base=None, head=None, head_plan=None):
    """The probes' records: each build's registered plan (the head's `head_plan`, by default the
    first head's), unless overridden by count"""
    out = []
    head_plan = head_plan or kc.HEAD_PLAN
    for b, reg, over in (("base", kc.BASE_PLAN, base or {}), ("head", head_plan, head or {})):
        for t in COUNTS:
            n, w, lazy = over.get(t, reg[t])
            out.append(
                {
                    "kind": "built",
                    "build": b,
                    "asked": t,
                    "threads": n,
                    "window": w,
                    "lazy": lazy,
                    "service": False,
                    "pipelined": False,
                }
            )
    return out


# off the i9 (the container's DRY): no cells, so each count builds the plan's own workers, eager
# below 8, lazy from 8, the window as asked
OFF_I9 = {t: (t, 4 * t, t >= 8) for t in COUNTS}


class Harness(unittest.TestCase):
    """The runs, the machine and the probes in a scratch directory, and the verdict read back"""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        (self.tmp / "env.json").write_text(json.dumps(ENV))
        (self.tmp / "lscpu.txt").write_text(LSCPU)

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def run_on(self, recs, *extra, env=None, lscpu=None, probes=None):
        p = self.tmp / "runs.jsonl"
        p.write_text("".join(json.dumps(r) + "\n" for r in recs))
        q = self.tmp / "plan.jsonl"
        q.write_text("".join(json.dumps(r) + "\n" for r in (plan() if probes is None else probes)))
        if env is not None:
            (self.tmp / "env.json").write_text(json.dumps(env))
        if lscpu is not None:
            (self.tmp / "lscpu.txt").write_text(lscpu)
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = kc.main(
                [
                    str(p),
                    "--env",
                    str(self.tmp / "env.json"),
                    "--lscpu",
                    str(self.tmp / "lscpu.txt"),
                    "--plan",
                    str(q),
                    *extra,
                ]
            )
        return rc, buf.getvalue()

    def verdict(self, out):
        return next(line for line in out.splitlines() if "verdict" in line.split(":")[0])

    def assert_void(self, recs, *extra, **kw):
        rc, out = self.run_on(recs, *extra, **kw)
        self.assertEqual(rc, 1, out)
        self.assertTrue(out.startswith("VOID") or "DRY: VOID" in out, out)
        self.assertNotIn("verdict:", out)
        self.assertNotIn("verdict (not evidence):", out)
        return out


class Verdict(Harness):
    """The first head, #860 (`--expect lazy-from-four`, the default)"""

    def test_registered_plans(self):
        # the plans the registration names: the head lazy from 4 and 8 workers from 10; the base
        # eager at 3, 5 and 7, 7 workers from 10; 3 eager and 8 lazy at both
        self.assertEqual({t for t, p in kc.HEAD_PLAN.items() if p[2]}, {4, 5, 7, 8, 10, 12})
        self.assertEqual({t for t, p in kc.BASE_PLAN.items() if p[2]}, {4, 8, 10, 12})
        self.assertEqual([kc.HEAD_PLAN[t][:2] for t in (10, 12)], [(8, 32), (8, 32)])
        self.assertEqual([kc.BASE_PLAN[t][:2] for t in (10, 12)], [(7, 28), (7, 28)])
        for t in (3, 4, 8):
            self.assertEqual(kc.BASE_PLAN[t], kc.HEAD_PLAN[t])

    def test_adopts(self):
        rc, out = self.run_on(records(step=GOOD))
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")

    def test_each_judged_count_must_be_faster(self):
        for t in (5, 7, 10):
            step = {**GOOD, t: 1.0}
            rc, out = self.run_on(records(step=step))
            self.assertEqual(rc, 0, out)
            self.assertEqual(self.verdict(out), f"verdict: NOT ADOPTED ({t} threads not FASTER)")

    def test_slower_at_a_supporting_count_not_adopted(self):
        for t in (3, 4, 8, 12):
            rc, out = self.run_on(records(step={**GOOD, t: 1.05}))
            self.assertIn(f"NOT ADOPTED (SLOWER at {t} threads)", self.verdict(out))

    def test_twelve_level_is_enough(self):
        rc, out = self.run_on(records(step={**GOOD, 12: 1.0}))
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")

    def test_both_reasons(self):
        rc, out = self.run_on(records(step={**GOOD, 7: 1.0, 3: 1.05}))
        self.assertEqual(
            self.verdict(out), "verdict: NOT ADOPTED (7 threads not FASTER; SLOWER at 3 threads)"
        )

    def test_plans_and_peak_rss_reported(self):
        rc, out = self.run_on(records(step=GOOD, mem=0.5))
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")
        row = next(ln for ln in out.splitlines() if ln.strip().startswith("7  judged"))
        self.assertIn("7/28 eager", row)
        self.assertIn("7/28 lazy", row)
        row = next(ln for ln in out.splitlines() if ln.strip().startswith("10  judged"))
        self.assertIn("7/28 lazy", row)
        self.assertIn("8/32 lazy", row)
        self.assertIn("10: 7, 8", out)
        line = next(ln for ln in out.splitlines() if ln.strip().startswith("5  base"))
        self.assertIn("base 488.3 MiB, head 244.1 MiB, head - base -244.1 MiB (0.5000)", line)

    def test_held_on_memory_trade_not_met(self):
        # 5% faster at 10 for 20% more memory: more than twice the gain
        rc, out = self.run_on(records(step=GOOD, mem=1.2))
        self.assertIn("verdict: HELD ON MEMORY (peak RSS above 1.1 at 3, 4, 5, 7, 8, 10, 12", out)
        self.assertIn(
            "rule 4's trade at 10 threads (reported; the Coordinator weighs it): not met", out
        )

    def test_held_on_memory_trade_met(self):
        recs = records(step={t: 0.88 for t in COUNTS}, mem=1.15)
        for r in recs:
            r["rss_kb"] = r["rss_kb"] // 10  # 48.8 MiB base, +7.3 MiB
        rc, out = self.run_on(recs)
        self.assertIn("verdict: HELD ON MEMORY", out)
        self.assertIn(
            "rule 4's trade at 7 threads (reported; the Coordinator weighs it): met:", out
        )

    def test_memory_at_the_gate_passes(self):
        rc, out = self.run_on(records(step=GOOD, mem=1.10))
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")

    def test_noisy_gain_not_adopted(self):
        rc, out = self.run_on(records(step={**GOOD, 10: 0.99}, noise=0.05))
        self.assertIn("NOT ADOPTED (10 threads not FASTER)", self.verdict(out))

    def test_void_head_not_the_intended_plan(self):
        # the head still eager at 7, or trimmed to 7 at 10: not the PR's plan
        out = self.assert_void(records(step=GOOD), probes=plan(head={7: (7, 28, False)}))
        self.assertIn("at 7 threads the head built 7/28 eager, not its registered 7/28 lazy", out)
        out = self.assert_void(
            records(step=GOOD, workers={("head", 10): 7}), probes=plan(head={10: (7, 28, True)})
        )
        self.assertIn("the head built 7/28 lazy, not its registered 8/32 lazy", out)

    def test_void_base_not_the_old_plan(self):
        out = self.assert_void(records(step=GOOD), probes=plan(base={5: (5, 20, True)}))
        self.assertIn("at 5 threads the base built 5/20 lazy", out)

    def test_void_probe_not_what_was_timed(self):
        out = self.assert_void(records(step=GOOD, workers={("head", 12): 10}))
        self.assertIn("at 12 threads the head's runs used 10 workers, its probe 8", out)

    def test_void_probe_records(self):
        probes = plan()
        self.assertIn(
            "missing probe ('head', 8)",
            self.assert_void(records(), probes=probes[:-3] + probes[-2:]),
        )
        self.assertIn("duplicated", self.assert_void(records(), probes=probes + probes[:1]))
        bad = plan()
        bad[2]["pipelined"] = True
        self.assertIn("pipelined", self.assert_void(records(), probes=bad))
        bad = plan()
        bad[2]["lazy"] = "yes"
        self.assertIn("malformed", self.assert_void(records(), probes=bad))
        bad = plan()
        bad[0]["build"] = "other"
        self.assertIn("unexpected probe", self.assert_void(records(), probes=bad))
        self.assertIn("no plan record", self.assert_void_no_plan())

    def assert_void_no_plan(self):
        p = self.tmp / "runs.jsonl"
        p.write_text("".join(json.dumps(r) + "\n" for r in records(step=GOOD)))
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = kc.main(
                [
                    str(p),
                    "--env",
                    str(self.tmp / "env.json"),
                    "--lscpu",
                    str(self.tmp / "lscpu.txt"),
                    "--plan",
                    str(self.tmp / "nowhere.jsonl"),
                ]
            )
        self.assertEqual(rc, 1)
        return buf.getvalue()

    def test_dry_labels_and_skips_the_machine_and_the_registered_plans(self):
        recs = records(
            step=GOOD,
            workers={(b, t): t for b in ("base", "head") for t in COUNTS},
        )
        rc, out = self.run_on(
            recs,
            "--dry",
            lscpu="Model name: Intel Xeon\n",
            env={**ENV, "cpus": 4},
            probes=plan(base=OFF_I9, head=OFF_I9),
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("DRY: not applied: lscpu", out)
        self.assertIn("DRY: not applied: at 5 threads the head built 5/20 eager", out)
        self.assertIn("DRY verdict (not evidence): ADOPTS", out)
        self.assertNotIn("\nverdict:", out)
        # the same records, not DRY: VOID on the machine and the plans
        rc, out = self.run_on(
            recs,
            lscpu="Model name: Intel Xeon\n",
            env={**ENV, "cpus": 4},
            probes=plan(OFF_I9, OFF_I9),
        )
        self.assertEqual(rc, 1, out)

    def test_dry_still_checks_probe_against_timed(self):
        self.assert_void(records(step=GOOD, workers={("base", 10): 10}), "--dry")

    def test_dry_still_voids_the_records(self):
        self.assert_void(records(step=GOOD)[:-1], "--dry")

    def test_void_missing_record(self):
        self.assertIn("missing", self.assert_void(records(step=GOOD)[:-1]))

    def test_void_duplicate(self):
        recs = records(step=GOOD)
        self.assertIn("duplicated", self.assert_void(recs + recs[:1]))

    def test_void_unexpected_arm(self):
        recs = records(step=GOOD)
        recs[0]["arm"] = "mibz2@/elsewhere/ab"
        self.assert_void(recs)

    def test_void_workers_unobserved(self):
        recs = records(step=GOOD)
        del recs[4]["observed_threads"]
        self.assertIn("observed_threads", self.assert_void(recs))

    def test_void_failed_run(self):
        for field, bad in (
            ("termination", "signal 9"),
            ("err", "boom"),
            ("wall", 0),
            ("rss_kb", float("nan")),
            ("cpu", None),
        ):
            recs = records(step=GOOD)
            recs[5][field] = bad
            self.assert_void(recs)

    def test_void_pinned_or_spinner(self):
        recs = records(step=GOOD)
        recs[3]["cpus"] = "0,1,2,3,4,5"
        self.assertIn("not unpinned", self.assert_void(recs))
        recs = records(step=GOOD)
        recs[3]["held_spinner"] = {"cpu": 19}
        self.assertIn("spinner", self.assert_void(recs))

    def test_void_outputs(self):
        recs = records(step=GOOD)
        recs[7]["hash"] = "ffff"
        self.assertIn("outputs differ", self.assert_void(recs))
        recs = records(step=GOOD)
        recs[7]["out"] = 1
        self.assertIn("wrote", self.assert_void(recs))
        recs = records(step=GOOD)
        recs[7]["corpus"] = "text"
        self.assert_void(recs)

    def test_void_machine(self):
        self.assert_void(records(step=GOOD), lscpu="Model name: AMD Ryzen 9 7950X\n")
        self.assert_void(records(step=GOOD), env={**ENV, "cpus": 16})
        self.assert_void(records(step=GOOD), env={**ENV, "cpu_quota": 4.0})

    def test_void_empty_or_not_records(self):
        self.assertIn("missing", self.assert_void([]))
        self.assertIn("unexpected key", self.assert_void([{}]))
        self.assertIn("not a record", self.assert_void([["a list"]]))


# The second head (journal §80, `--expect lazy-four-to-seven`): #860 without the trim to 8 from
# 10. Its predicted pattern: faster at 5 and 7 as #860's run measured (0.845, 0.634), level
# everywhere else, 10 and 12 included (the same 7/28 plan as the base there)
FOUR7 = kc.HEAD_PLAN_FOUR_TO_SEVEN
GOOD47 = {5: 0.845, 7: 0.634}
EXPECT47 = ("--expect", "lazy-four-to-seven")


class FourToSeven(Harness):
    """The second head, #860 without the trim to 8 from 10 (`--expect lazy-four-to-seven`)"""

    def run47(self, recs=None, *extra, probes=None, **kw):
        recs = records(step=GOOD47, head_plan=FOUR7) if recs is None else recs
        probes = plan(head_plan=FOUR7) if probes is None else probes
        return self.run_on(recs, *EXPECT47, *extra, probes=probes, **kw)

    def test_registered_plan(self):
        # master's plan at every count but 5 and 7, where the plan's own workers go lazy
        self.assertEqual(set(FOUR7), set(kc.BASE_PLAN))
        for t in FOUR7:
            if t in (5, 7):
                self.assertEqual(FOUR7[t], (t, 4 * t, True), t)
                self.assertEqual(kc.BASE_PLAN[t], (t, 4 * t, False), t)
            else:
                self.assertEqual(FOUR7[t], kc.BASE_PLAN[t], t)
        self.assertEqual([FOUR7[t] for t in (10, 12)], [(7, 28, True)] * 2)
        # and the first head's expectation stays what the archived run was judged by
        self.assertEqual(kc.DEFAULT_EXPECT, "lazy-from-four")
        self.assertEqual(
            kc.EXPECTATIONS["lazy-from-four"], (kc.HEAD_PLAN, (5, 7, 10), (3, 4, 8, 12))
        )
        self.assertEqual(kc.EXPECTATIONS["lazy-four-to-seven"][1:], ((5, 7), (3, 4, 8, 10, 12)))

    def test_adopts(self):
        rc, out = self.run47()
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")
        self.assertIn("expectation lazy-four-to-seven", out)
        row = next(ln for ln in out.splitlines() if ln.strip().startswith("10  supporting"))
        self.assertEqual(row.count("7/28 lazy"), 2, row)

    def test_adopts_with_the_recipes_counts(self):
        rc, out = self.run47(None, "--judged", "5,7", "--supporting", "3,4,8,10,12")
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")

    def test_each_judged_count_must_be_faster(self):
        for t in (5, 7):
            rc, out = self.run47(records(step={**GOOD47, t: 1.0}, head_plan=FOUR7))
            self.assertEqual(rc, 0, out)
            self.assertEqual(self.verdict(out), f"verdict: NOT ADOPTED ({t} threads not FASTER)")

    def test_ten_level_is_enough(self):
        # 10 is supporting here, not judged: level there adopts
        rc, out = self.run47(records(step={**GOOD47, 10: 1.0, 12: 1.0}, head_plan=FOUR7))
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")
        row = next(ln for ln in out.splitlines() if ln.strip().startswith("10  supporting"))
        self.assertIn("INCONCLUSIVE", row)

    def test_slower_at_a_supporting_count_not_adopted(self):
        for t in (3, 4, 8, 10, 12):
            rc, out = self.run47(records(step={**GOOD47, t: 1.05}, head_plan=FOUR7))
            self.assertEqual(self.verdict(out), f"verdict: NOT ADOPTED (SLOWER at {t} threads)")

    def test_held_on_memory(self):
        rc, out = self.run47(records(step=GOOD47, mem=1.2, head_plan=FOUR7))
        self.assertIn("verdict: HELD ON MEMORY (peak RSS above 1.1 at 3, 4, 5, 7, 8, 10, 12", out)
        # (+102 MB a process: over rule 4's 64 MB even where the gain is large)
        self.assertIn(
            "rule 4's trade at 7 threads (reported; the Coordinator weighs it): not met", out
        )
        self.assertIn(
            "rule 4's trade at 3 threads (reported; the Coordinator weighs it): not met", out
        )

    def test_void_head_is_the_first_heads_plan(self):
        # the head trimmed to 8 at 10 (#860 itself): not the second head
        out = self.assert_void(records(step=GOOD47), *EXPECT47, probes=plan(head_plan=kc.HEAD_PLAN))
        self.assertIn("at 10 threads the head built 8/32 lazy, not its registered 7/28 lazy", out)
        self.assertIn("at 12 threads the head built 8/32 lazy, not its registered 7/28 lazy", out)

    def test_void_head_still_eager(self):
        out = self.assert_void(
            records(step=GOOD47, head_plan=kc.BASE_PLAN),
            *EXPECT47,
            probes=plan(head_plan=kc.BASE_PLAN),
        )
        self.assertIn("at 5 threads the head built 5/20 eager, not its registered 5/20 lazy", out)

    def test_first_expectation_voids_on_the_second_head(self):
        # the default expectation run on the second head's build: VOID, never a verdict
        out = self.assert_void(records(step=GOOD47, head_plan=FOUR7), probes=plan(head_plan=FOUR7))
        self.assertIn("at 10 threads the head built 7/28 lazy, not its registered 8/32 lazy", out)

    def test_void_counts_not_the_registered_rule(self):
        out = self.assert_void(
            records(step=GOOD47, head_plan=FOUR7),
            *EXPECT47,
            "--judged",
            "5,7,10",
            "--supporting",
            "3,4,8,12",
            probes=plan(head_plan=FOUR7),
        )
        self.assertIn("not lazy-four-to-seven's registered judged [5, 7]", out)
        # and the first expectation with the second's counts
        out = self.assert_void(records(step=GOOD), "--judged", "5,7", "--supporting", "3,4,8,10,12")
        self.assertIn("not lazy-from-four's registered judged [5, 7, 10]", out)

    def test_unknown_expectation_refused(self):
        with self.assertRaises(SystemExit), redirect_stderr(io.StringIO()):
            self.run_on(records(), "--expect", "lazy-everywhere")

    def test_dry(self):
        recs = records(step=GOOD47, workers={(b, t): t for b in ("base", "head") for t in COUNTS})
        rc, out = self.run47(
            recs,
            "--dry",
            lscpu="Model name: Intel Xeon\n",
            env={**ENV, "cpus": 4},
            probes=plan(base=OFF_I9, head=OFF_I9),
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("DRY: not applied: at 5 threads the head built 5/20 eager", out)
        self.assertIn("DRY verdict (not evidence): ADOPTS", out)


if __name__ == "__main__":
    unittest.main()
