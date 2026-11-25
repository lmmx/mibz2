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
from contextlib import redirect_stdout
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


def records(arm="mibz2", step=None, mem=1.0, noise=0.01, workers=None):
    """A complete run: `step` maps a count to the head's wall over the base's (default level);
    `mem` the head's peak memory over the base's; workers as each build's registered plan unless
    `workers` maps (build, count) to another. Rounds alternate the noise above and below"""
    step = step or {}
    workers = workers or {}
    out = []
    for r in range(R):
        for t in COUNTS:
            for which, plan in (("base", kc.BASE_PLAN), ("head", kc.HEAD_PLAN)):
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


def plan(base=None, head=None):
    """The probes' records: each build's registered plan, unless overridden by count"""
    out = []
    for b, reg, over in (("base", kc.BASE_PLAN, base or {}), ("head", kc.HEAD_PLAN, head or {})):
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


class Verdict(unittest.TestCase):
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

    def assert_void(self, recs, *extra, **kw):
        rc, out = self.run_on(recs, *extra, **kw)
        self.assertEqual(rc, 1, out)
        self.assertTrue(out.startswith("VOID") or "DRY: VOID" in out, out)
        self.assertNotIn("verdict:", out)
        self.assertNotIn("verdict (not evidence):", out)
        return out

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


if __name__ == "__main__":
    unittest.main()
