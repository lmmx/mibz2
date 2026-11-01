"""The dispatch analysis refuses a definitive decision on incomplete evidence (amendment 1,
docs/journal/2026-11-01-dispatch-causality/amendment-1.md), and still reaches an ordinary
decision, negative or positive, on complete evidence:

- the kernel's loss records, unreadable lines and perf's own statistics are counted,
  never silently skipped, and scheduler loss makes the trace unsound;
- a scheduling stream that contradicts itself leaves those intervals unknown;
- every four-worker run in the decision, the expansion's included, must meet the
  scheduler checks (TIDs seen, state known for 99%) and the trace's completeness;
- a real capture from the preparation VM, whose kernel emits no switches out of idle on
  three of its CPUs, is refused.

    python3 bench/test_dispatch.py
"""

import argparse
import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dispatch  # noqa: E402

FIXTURE = Path(__file__).resolve().parent / "fixtures" / "dispatch"


def perf_line(t, cpu, tid, ev, rest, comm="x"):
    return f"{comm:>16} {tid:>6} [{cpu:03d}] {t // 10**9}.{t % 10**9:09d}: {ev}: {rest}\n"


def switch(t, cpu, prev, state, nxt):
    return perf_line(t, cpu, prev, "sched:sched_switch",
                     f"prev_comm=a prev_pid={prev} prev_prio=120 prev_state={state} ==> next_comm=b next_pid={nxt} next_prio=120")


def wake(t, cpu, pid, ev="sched:sched_waking"):
    return perf_line(t, cpu, 1, ev, f"comm=a pid={pid} prio=120 target_cpu={cpu:03d}")


STATS_CLEAN = """Aggregated stats:
           TOTAL events:       1000
            LOST events:          0  ( 0.0%)
sched:sched_switch stats:
          SAMPLE events:        600
sched:sched_waking stats:
          SAMPLE events:        400
"""

STATS_LOSSY = """Aggregated stats:
           TOTAL events:       1000
            LOST events:          2  ( 0.0%)
    LOST_SAMPLES events:          2  ( 0.0%)
sched:sched_switch stats:
          SAMPLE events:        600
    LOST_SAMPLES events:          7
syscalls:sys_enter_futex stats:
          SAMPLE events:        300
    LOST_SAMPLES events:          5
"""


def write(d, name, text):
    p = Path(d) / name
    p.write_text(text)
    return p


class Parse(unittest.TestCase):
    def test_counts_loss_unattributed_and_unreadable(self):
        with tempfile.TemporaryDirectory() as d:
            text = (switch(1_000, 1, 10, "R", 0)
                    + perf_line(2_000, 1, -1, "sched:sched_switch",
                                "prev_comm=a prev_pid=10 prev_prio=120 prev_state=X ==> next_comm=swapper/1 next_pid=0 next_prio=120", comm=":-1")
                    + "         python3     198 [002]   101.492847789: PERF_RECORD_LOST lost 162\n"
                    + "this is not a perf line\n\n")
            ev, rep = dispatch.load_perf(write(d, "p.txt", text))
        self.assertEqual(rep["events"], 2, "the unresolved-tid switch is kept, not dropped")
        self.assertEqual(rep["unattributed"], {"sched:sched_switch": 1})
        self.assertEqual((rep["lost_records"], rep["lost_events"]), (1, 162))
        self.assertEqual(rep["unparsed"], 1)

    def test_stats(self):
        with tempfile.TemporaryDirectory() as d:
            st = dispatch.load_stats(write(d, "s.txt", STATS_LOSSY))
            missing = dispatch.load_stats(Path(d) / "absent.txt")
        self.assertTrue(st["available"])
        self.assertEqual(st["lost_records"], 2)
        self.assertEqual(st["per_event"]["sched:sched_switch"], dict(samples=600, lost=7))
        self.assertFalse(missing["available"])


def stream_of(lines, ours, cpus=(1,)):
    with tempfile.TemporaryDirectory() as d:
        ev, _ = dispatch.load_perf(write(d, "p.txt", "".join(lines)))
    return dispatch.sched_reconstruct(ev, set(ours), list(cpus))


class Reconstruct(unittest.TestCase):
    def test_complete_stream(self):
        sched, _, st = stream_of([wake(100, 1, 10), switch(200, 1, 0, "R", 10), switch(300, 1, 10, "S", 0),
                                  wake(400, 1, 10), switch(500, 1, 0, "R", 10)], [10])
        self.assertEqual([s for _, s in sched[10]], ["Q", "R", "S", "Q", "R"])
        self.assertEqual((st["missed_switch_in"], st["missed_switch_out"], st["missing_wakeups"]), (0, 0, 0))
        self.assertTrue(st["idle_pairing_ok"])

    def test_missed_switch_in_is_unknown_not_runnable(self):
        # woken, then switched out with no switch in recorded (the idle exit missing)
        sched, _, st = stream_of([wake(100, 1, 10), switch(300, 1, 10, "S", 0), switch(400, 1, 0, "R", 11),
                                  switch(500, 1, 11, "S", 0)], [10])
        self.assertEqual(st["missed_switch_in"], 1)
        self.assertEqual(sched[10][0], (100, None), "the interval since the wake is unknown, never runnable")

    def test_missed_switch_out(self):
        sched, _, st = stream_of([wake(100, 1, 10), switch(200, 1, 0, "R", 10), switch(300, 1, 0, "R", 10)], [10])
        self.assertEqual(st["missed_switch_out"], 1)
        self.assertEqual(sched[10][1], (200, None))

    def test_missing_wakeup(self):
        sched, _, st = stream_of([switch(100, 1, 10, "S", 0), switch(200, 1, 0, "R", 10)], [10])
        self.assertEqual(st["missing_wakeups"], 1)
        self.assertEqual(sched[10][0], (100, None))

    def test_unpaired_idle(self):
        _, _, st = stream_of([switch(100, 1, 10, "S", 0), switch(200, 1, 11, "S", 0), switch(300, 1, 12, "S", 0)], [])
        self.assertFalse(st["idle_pairing_ok"])


class Trace(unittest.TestCase):
    def report(self, stats_text, perf_text=""):
        with tempfile.TemporaryDirectory() as d:
            ev, parse = dispatch.load_perf(write(d, "p.txt", perf_text))
            stats = dispatch.load_stats(write(d, "s.txt", stats_text)) if stats_text is not None else dispatch.load_stats(Path(d) / "none")
        _, _, stream = dispatch.sched_reconstruct(ev, set(), [1])
        return dispatch.trace_report(parse, stats, stream)

    def test_clean(self):
        t = self.report(STATS_CLEAN)
        self.assertTrue(t["ok"], t["problems"])
        self.assertTrue(t["futex_ok"])

    def test_scheduler_loss_is_unsound(self):
        lost = "".join(f"  x 1 [001] 1.00000000{i}: PERF_RECORD_LOST lost {n}\n" for i, n in enumerate((7, 5)))
        t = self.report(STATS_LOSSY, lost)
        self.assertFalse(t["ok"])
        self.assertFalse(t["futex_ok"])
        self.assertTrue(any("scheduler" in p for p in t["problems"]))

    def test_futex_loss_alone_spares_the_scheduler(self):
        stats = STATS_LOSSY.replace("    LOST_SAMPLES events:          7\n", "").replace("LOST events:          2", "LOST events:          1")
        t = self.report(stats, "  x 1 [001] 1.000000000: PERF_RECORD_LOST lost 5\n")
        self.assertTrue(t["ok"], t["problems"])
        self.assertFalse(t["futex_ok"])

    def test_loss_the_script_did_not_show(self):
        t = self.report(STATS_LOSSY)  # perf's statistics report loss the script never printed
        self.assertFalse(t["ok"])
        self.assertTrue(any("disagree" in p for p in t["problems"]))

    def test_missing_statistics(self):
        self.assertFalse(self.report(None)["ok"])

    def test_unreadable_lines(self):
        t = self.report(STATS_CLEAN, "garbled\n")
        self.assertFalse(t["ok"])


def run_json(R=0.01, coverage=0.95, sched_coverage=1.0, threads=True, trace_ok=True, mutex_wait=0.0):
    W = 2.0
    return dict(marks="-", workers=4, cpus=[1, 2, 3, 4], alignment=dict(ok=True), complete=True, threads_in_sched=threads,
                missing_wakeups=0, span_s=0.5, worker_time_s=W, queued_nondecoding_s=0.2,
                classes_s={"rx.blocked_by_owner.cpu_idle": R * W, "runnable.mutex_wait.cpu_idle": mutex_wait * W, "recv.running": 0.01},
                no_job_s={}, backlog_all_decoding_job_s=0.0, runnable_cpu_occupants_s={}, coverage=coverage,
                sched_coverage=sched_coverage, receiver_obstruction_share=R, receiver_obstruction_cpu_idle_share=R,
                reader={}, futex={}, trace=dict(ok=trace_ok, problems=[] if trace_ok else ["scheduler events lost"]))


PAIRS = "# w\trep\tmode\twall\tcpu\tout\trss_kb\ttier\n" + "".join(
    f"{w}\t{r}\t{m}\t1.0\t2.0\t1000\t100\tA\n" for w in (4, 1) for r in range(3) for m in ("off", "instr"))


def summarise(w4, w1=None):
    with tempfile.TemporaryDirectory() as d:
        for i, r in enumerate(w4):
            write(d, f"w4-r{i:02d}.json", json.dumps(r))
            write(d, f"w4-r{i:02d}.tsv.meta.json", "{}")  # a sidecar: never a run
        for i, r in enumerate(w1 if w1 is not None else [run_json(R=0.0)]):
            write(d, f"w1-r{i:02d}.json", json.dumps(r))
        pairs = write(d, "pairs.tsv", PAIRS)
        out = Path(d) / "result.json"
        with contextlib.redirect_stdout(io.StringIO()):
            dispatch.summary(argparse.Namespace(runs=d, pairs=str(pairs), out=str(out)))
        return json.loads(out.read_text())


class Summary(unittest.TestCase):
    def test_complete_evidence_negative(self):
        r = summarise([run_json(R=0.01) for _ in range(10)])
        self.assertTrue(r["interpreted"])
        self.assertEqual(r["outcome"], "do not proceed")
        self.assertFalse(r["decision"].startswith("not interpreted"))

    def test_complete_evidence_positive(self):
        r = summarise([run_json(R=0.06) for _ in range(10)])
        self.assertEqual(r["outcome"], "promote")

    def test_an_expansion_run_without_scheduler_coverage(self):
        # the first three pass the expansion; the eighth loses scheduler state
        runs = [run_json(R=0.06) for _ in range(10)]
        runs[7] = run_json(R=0.06, sched_coverage=0.95)
        r = summarise(runs)
        self.assertFalse(r["interpreted"])
        self.assertEqual(r["outcome"], "invalid evidence")
        self.assertFalse(r["validity"]["sched_coverage_all"])
        self.assertFalse(r["run_checks"]["w4-r07"]["sched_coverage"])
        self.assertTrue(r["decision"].startswith("not interpreted"))

    def test_a_thread_unseen_by_the_kernel(self):
        runs = [run_json(R=0.06) for _ in range(10)]
        runs[5] = run_json(R=0.06, threads=False)
        r = summarise(runs)
        self.assertEqual(r["outcome"], "invalid evidence")
        self.assertFalse(r["validity"]["threads_in_sched_all"])

    def test_trace_loss(self):
        runs = [run_json(R=0.01) for _ in range(10)]
        runs[9] = run_json(R=0.01, trace_ok=False)
        r = summarise(runs)
        self.assertEqual(r["outcome"], "invalid evidence", "not an ordinary negative result")
        self.assertFalse(r["validity"]["trace_complete_all"])
        self.assertEqual(r["trace_problems"]["w4-r09"], ["scheduler events lost"])

    def test_no_runs(self):
        r = summarise([])
        self.assertEqual(r["outcome"], "invalid evidence")
        self.assertFalse(r["validity"]["runs_present"])

    def test_the_associated_share_is_exploratory(self):
        r = summarise([run_json(R=0.01, mutex_wait=0.10) for _ in range(10)])
        self.assertEqual(r["outcome"], "do not proceed", "the associated share never promotes")
        self.assertAlmostEqual(r["receiver_associated_exploratory"]["mean_share"], 0.11)
        self.assertAlmostEqual(r["receiver_obstruction"]["mean_share"], 0.01)


class Quality(unittest.TestCase):
    def quality(self, runs):
        with tempfile.TemporaryDirectory() as d:
            for i, r in enumerate(runs):
                write(d, f"w4-r{i:02d}.json", json.dumps(r))
                write(d, f"w4-r{i:02d}.tsv.meta.json", "{}")
            pairs = write(d, "pairs.tsv", PAIRS)
            with contextlib.redirect_stdout(io.StringIO()) as out, self.assertRaises(SystemExit) as e:
                dispatch.quality_cmd(argparse.Namespace(runs=d, pairs=str(pairs)))
        return e.exception.code, json.loads(out.getvalue())

    def test_expands_on_complete_evidence(self):
        self.assertEqual(self.quality([run_json() for _ in range(3)])[0], 0)

    def test_trace_loss_stops_the_expansion(self):
        code, q = self.quality([run_json(), run_json(trace_ok=False), run_json()])
        self.assertEqual(code, 3)
        self.assertFalse(q["runs"][1]["trace_complete"])

    def test_no_runs_never_expand(self):
        self.assertEqual(self.quality([])[0], 3)


class RealCapture(unittest.TestCase):
    """the preparation VM's tier-B w = 4 dry-run capture (untimed): its kernel records
    switches into idle on CPUs 1-3 but none out of it, so threads it switched in from idle
    appear switched out while 'runnable'"""

    def run_fixture(self, cpus="0,1,2,3"):
        with tempfile.TemporaryDirectory() as d:
            stats = write(d, "s.txt", STATS_CLEAN)
            out = Path(d) / "run.json"
            with contextlib.redirect_stdout(io.StringIO()):
                dispatch.run(argparse.Namespace(marks=str(FIXTURE / "vm-w4.tsv.gz"), perf=str(FIXTURE / "vm-w4.perf.txt.gz"),
                                                stats=str(stats), cpus=cpus, workers=4, out=str(out)))
            return json.loads(out.read_text())

    def test_refused(self):
        r = self.run_fixture()
        self.assertTrue(r["complete"] and r["threads_in_sched"] and r["alignment"]["ok"])
        self.assertFalse(r["trace"]["ok"])
        self.assertEqual([c for c, v in r["trace"]["stream"]["idle_pairing"].items() if not v["ok"]], ["1", "2", "3"])
        self.assertGreater(r["trace"]["stream"]["missed_switch_in"], 0)
        self.assertLess(r["sched_coverage"], dispatch.SCHED_COVERAGE)
        s = summarise([r, r, r])
        self.assertEqual(s["outcome"], "invalid evidence")
        self.assertFalse(s["validity"]["trace_complete_all"])
        self.assertFalse(s["validity"]["sched_coverage_all"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
