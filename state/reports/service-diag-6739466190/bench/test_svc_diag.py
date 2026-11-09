"""bench/svc_diag.py's attribution on the Researcher's abstract witnesses (REVIEW 6736071231,
state/reports/service-starvation-6735725456/witnesses.py at d6f52bdd), replayed as event
logs, no decoder run: each is classified as the report says.

- the finish-priority witness: P = 1, J = 1, Q = 2, both service decisions are finishes
  (the source's priority, as the witness's `choose`), a team takes the last prepared block,
  and the other waits while both services finish: `finish_first`;
- the nine-unit wait-label witness: entry label prep for all nine units, but P = 0 for one
  and P > 0, B = 0 for eight;
- the equal-marginals witness: two service workers busy 70% each, overlapping the selected
  starvation unit fully in one log and not at all in the other, and attributed differently;
- the front-availability enumeration: 5,040 states, no eligible preparation without a front;
- the report's deterministic checks: a full window with later results ready and the earliest
  missing block finishing (`window_credit`), feeder lag, startup and drain apart.

The fixed-split bound (witnesses.py's last check) is arithmetic about capacity, not a
classification, and is checked as arithmetic here.

    python3 bench/test_svc_diag.py        (or python3 -m pytest bench/test_svc_diag.py)
"""

import io
import sys
import tempfile
import unittest
import unittest.mock
from fractions import Fraction
from itertools import product
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import svc_diag as d  # noqa: E402

# the source's priority (parallel.rs's service_worker; witnesses.py's `choose`)


def choose(p, j, front, q, l):
    if j and front and p == 0:
        return "prepare"
    if q:
        return "finish"
    if j and front and p < l:
        return "prepare"
    return "wait"


class Sim:
    """A hub, a window and their events, in the order and with the snapshots the decoder's
    log has; every hub snapshot is checked against the ownership identities as it is made"""

    def __init__(self, T=2, K=2, L=2, W=8):
        self.T, self.K, self.L, self.W = T, K, L, W
        self.F = T + L + K
        self.t = 0
        self.seq = 0
        self.h = dict(
            j=0, p=0, q=0, a=0, c=0, u=0, free_front=self.F, free_back=2 * T, live=0, closed=0
        )
        self.w = dict(
            issued=0,
            taken=0,
            ready=0,
            ready_min=None,
            ready_max=None,
            end=None,
            slots=W,
            cancelled=0,
        )
        self.ready = set()
        self.jobs, self.prepared, self.walked = [], [], []
        self.events = []
        self.emit("reader", "Config", T, K, hub=True, win=True)

    def tick(self, n=1):
        self.t += n

    def emit(self, thread, kind, place=None, arg=0, n=0, hub=False, win=False):
        if hub:
            h = self.h
            assert h["free_front"] + h["a"] + h["p"] + h["u"] == self.F, (kind, h)
            assert h["free_back"] + h["u"] + h["q"] + h["c"] == 2 * self.T, (kind, h)
            h["live"] = h["a"] + h["p"] + h["u"] + h["q"] + h["c"]
        if win:
            self.w["ready"] = len(self.ready)
            self.w["ready_min"] = min(self.ready) if self.ready else None
            self.w["ready_max"] = max(self.ready) if self.ready else None
        e = dict(seq=self.seq, ns=self.t, thread=thread, kind=kind, place=place, arg=arg, n=n)
        e["hub"] = dict(self.h) if hub else None
        e["win"] = dict(self.w) if win else None
        self.events.append(e)
        self.seq += 1

    # the scanner and the feeder
    def issue(self, place, enqueue=True, feed=True):
        self.w["issued"] += 1
        self.emit("mibz2-scan", "Issued", place, win=True)
        if enqueue:
            self.emit("mibz2-scan", "Enqueued", place)
        if feed:
            self.feed(place)

    def feed(self, place):
        self.emit("mibz2-svc-feed", "FeedRecv", place)
        self.jobs.append(place)
        self.h["j"] += 1
        self.emit("mibz2-svc-feed", "JobIn", place, hub=True)

    def slot_wait(self):
        self.emit("mibz2-scan", "SlotWait", win=True)

    def close(self):
        self.w["end"] = self.w["issued"]
        self.emit("mibz2-scan", "ScanEnd", win=True)
        self.h["closed"] = 1
        self.emit("mibz2-svc-feed", "Closed", hub=True)

    # a service worker's decision, by the source's priority
    def decide(self, s):
        h = self.h
        c = choose(h["p"], h["j"], h["free_front"], h["q"], self.L)
        th = f"mibz2-svc-{s}"
        if c == "prepare":
            rule = 0 if h["p"] == 0 else 1
            place = self.jobs.pop(0)
            h["j"] -= 1
            h["free_front"] -= 1
            h["a"] += 1
            self.emit(th, "SvcPrepare", place, rule, hub=True)
            return c, place
        if c == "finish":
            place = min(self.walked)
            self.walked.remove(place)
            h["q"] -= 1
            h["c"] += 1
            self.emit(th, "SvcFinish", place, hub=True)
            return c, place
        why = d.NO_JOB if not h["j"] else d.THRESHOLD
        self.emit(th, "SvcWait", None, why, hub=True)
        return c, None

    def prepared_(self, s, place):
        self.prepared.append(place)
        self.h["a"] -= 1
        self.h["p"] += 1
        self.emit(f"mibz2-svc-{s}", "Prepared", place, 1000, hub=True)

    def finished(self, s, place):
        self.ready.add(place)
        self.emit(f"mibz2-svc-{s}", "Publish", place, win=True)
        self.h["c"] -= 1
        self.h["free_back"] += 1
        self.emit(f"mibz2-svc-{s}", "FinishRetire", place, hub=True)

    def take(self):
        place = self.w["taken"]
        assert place in self.ready
        self.ready.remove(place)
        self.w["taken"] += 1
        self.emit("reader", "Taken", place, win=True)

    # a team
    def team_take(self, team):
        h = self.h
        assert h["p"] > 0 and h["free_back"] > 0
        place = self.prepared.pop(0)
        h["p"] -= 1
        h["free_back"] -= 1
        h["u"] += 1
        self.emit(f"mibz2-{team}", "TeamGot", place, hub=True)
        return place

    def team_wait(self, team):
        self.emit(f"mibz2-{team}", "TeamWait", None, int(self.h["p"] > 0), hub=True)

    def team_walked(self, team, place):
        h = self.h
        self.walked.append(place)
        h["u"] -= 1
        h["free_front"] += 1
        h["q"] += 1
        self.emit(f"mibz2-{team}", "Walked", place, hub=True)

    def end(self):
        self.tick(0)
        self.emit("reader", "ReadEnd", win=True)

    def analyse(self):
        return d.analyse(self.events)

    def tsv(self):
        """The log as the decoder writes it (src/diag.rs's `tsv`)"""
        cols = ["seq", "ns", "thread", "kind", "place", "arg", "n", *d.HUB, *d.WIN]
        rows = ["\t".join(cols)]
        for e in self.events:
            f = lambda v: "-" if v is None else str(v)  # noqa: E731
            row = [
                f(e["seq"]),
                f(e["ns"]),
                e["thread"],
                e["kind"],
                f(e["place"]),
                f(e["arg"]),
                f(e["n"]),
            ]
            row += [f(e["hub"][k]) if e["hub"] else "-" for k in d.HUB]
            row += [f(e["win"][k]) if e["win"] else "-" for k in d.WIN]
            rows.append("\t".join(row))
        return "\n".join(rows) + "\n"


def wait_of(result, team):
    (w,) = [w for w in result["waits"] if w["team"] == f"mibz2-{team}"][-1:]
    return w


def history(sim, team, place):
    """A team's first block, start to finish, so its later waits are not startup"""
    sim.issue(place)
    assert sim.decide(0) == ("prepare", place)
    sim.prepared_(0, place)
    assert sim.team_take(team) == place
    sim.team_walked(team, place)
    assert sim.decide(0) == ("finish", place)
    sim.finished(0, place)
    sim.take()


class FinishFirst(unittest.TestCase):
    """witnesses.py's finish_priority_witness, replayed"""

    def test_both_decisions_finish_and_the_other_team_waits_finish_first(self):
        s = Sim(T=2, K=2, L=2)
        history(s, 0, 0)
        history(s, 1, 1)
        for p in (2, 3, 4, 5, 6):
            s.issue(p)
        # both teams walking blocks 2 and 3, blocks 4 and 5 prepared
        assert s.decide(0) == ("prepare", 2) and s.decide(1) == ("prepare", 3)
        s.prepared_(0, 2)
        s.prepared_(1, 3)
        s.team_take(0), s.team_take(1)
        assert s.decide(0) == ("prepare", 4) and s.decide(1) == ("prepare", 5)
        s.prepared_(0, 4)
        s.prepared_(1, 5)
        assert s.decide(0)[0] == "wait" and s.decide(1)[0] == "wait"  # P = L
        s.team_walked(0, 2)
        assert s.team_take(0) == 4
        s.team_walked(1, 3)
        # the witness's state: P = 1, J = 1, Q = 2, both services free
        h = s.h
        self.assertEqual((h["p"], h["j"], h["q"], h["a"], h["c"]), (1, 1, 2, 0, 0))
        first, second = s.decide(0), s.decide(1)
        self.assertEqual((first[0], second[0]), ("finish", "finish"))
        # team 1 takes the last prepared block; team 0, its walk done, now waits
        s.team_take(1)
        s.tick()
        s.team_walked(0, 4)
        s.team_wait(0)
        self.assertEqual((s.h["p"], s.h["j"], s.h["a"], s.h["c"]), (0, 1, 0, 2))
        s.tick(5)  # both finishes run on
        s.finished(0, 2)
        assert s.decide(0) == ("prepare", 6)  # P = 0: rule 1
        s.tick(3)
        s.prepared_(0, 6)
        s.team_take(0)
        s.end()
        r = s.analyse()
        w = wait_of(r, 0)
        self.assertEqual(w["label"], "prep")
        self.assertEqual(w["classes"]["finish_first"], 5)
        self.assertEqual(w["classes"]["prep_in_progress"], 3)
        self.assertEqual(sum(w["classes"].values()), 8)
        self.assertEqual(r["finish_over_prep"], 2)
        self.assertEqual(r["defects"], {})
        summ = d.summarise([r])
        self.assertAlmostEqual(summ["classes"]["finish_first"]["share"], 5 / 8)
        self.assertEqual(summ["classes"]["finish_first"]["waits"], 1)


class NineUnits(unittest.TestCase):
    """witnesses.py's wait_label_witness: t0 P = B = 0; t1 P = 1, B = 0; t9 B = 1, taken"""

    def sim(self):
        s = Sim(T=1, K=2, L=1)
        history(s, 0, 0)
        for p in (1, 2, 3):
            s.issue(p)
        assert s.decide(0) == ("prepare", 1)
        s.prepared_(0, 1)
        s.team_take(0)
        s.team_walked(0, 1)  # Q = 1
        assert s.decide(1) == ("prepare", 2)  # P = 0: rule 1 before the finish
        s.prepared_(1, 2)
        s.team_take(0)
        assert s.decide(1) == ("prepare", 3)
        s.team_walked(0, 2)  # Q = 2: both backs walked, none free
        assert s.decide(0) == ("finish", 1)
        # t0: the team waits, P = 0, B = 0 (block 3 preparing)
        s.team_wait(0)
        self.assertEqual((s.h["p"], s.h["free_back"]), (0, 0))
        s.tick(1)
        # t1: block 3 prepared; P = 1, still B = 0
        s.prepared_(1, 3)
        assert s.decide(1) == ("finish", 2)
        s.tick(8)
        # t9: block 1's finish done, a back free: the team takes block 3
        s.finished(0, 1)
        s.team_take(0)
        s.end()
        return s

    def test_the_entry_label_says_nine_units_of_prep_the_states_say_one_and_eight(self):
        r = self.sim().analyse()
        w = wait_of(r, 0)
        self.assertEqual(w["label"], "prep")
        self.assertEqual(sum(w["classes"].values()), 9)
        # recorded_prep 9 = actual_P_zero 1 + P_positive_B_zero 8
        self.assertEqual(w["deficit"]["P0B0"], 1)
        self.assertEqual(w["deficit"]["P1B0"], 8)
        self.assertEqual(w["classes"]["prep_in_progress"], 1)
        self.assertEqual(w["other"]["no_back"], 8)
        summ = d.summarise([r])
        self.assertEqual(summ["entry_label"]["prep"], 1.0)
        self.assertAlmostEqual(summ["deficit"]["P0B0"], 1 / 9)
        self.assertAlmostEqual(summ["deficit"]["P1B0"], 8 / 9)

    def test_the_tsv_round_trip(self):
        s = self.sim()
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / "log.tsv"
            p.write_text(s.tsv())
            again = d.analyse(d.read_tsv(p))
        self.assertEqual(wait_of(again, 0)["deficit"], wait_of(s.analyse(), 0)["deficit"])
        buf = io.StringIO()
        with unittest.mock.patch("sys.stdout", buf):
            p2 = Path(tempfile.mkdtemp()) / "log.tsv"
            p2.write_text(s.tsv())
            self.assertEqual(d.main([str(p2)]), 0)
        self.assertIn("INDICATIVE ONLY", buf.getvalue())


class EqualMarginals(unittest.TestCase):
    """witnesses.py's equal_marginals: two services busy 70% each; the selected starvation
    unit overlapped fully (case a) or not at all (case b), and attributed differently"""

    def test_equal_marginals_different_attribution(self):
        logs = {}
        for case in ("a", "b"):
            s = Sim(T=2, K=2, L=2)
            history(s, 0, 0)
            history(s, 1, 1)
            if case == "a":
                # unit 0: the finish-priority witness's state, both services finishing
                for p in (2, 3, 4, 5, 6):
                    s.issue(p)
                s.decide(0), s.decide(1)
                s.prepared_(0, 2), s.prepared_(1, 3)
                s.team_take(0), s.team_take(1)
                s.decide(0), s.decide(1)
                s.prepared_(0, 4), s.prepared_(1, 5)
                s.decide(0), s.decide(1)  # wait: P = L
                s.team_walked(0, 2)
                s.team_take(0)
                s.team_walked(1, 3)
                assert s.decide(0)[0] == s.decide(1)[0] == "finish"
                s.team_take(1)
                s.team_walked(0, 4)
                s.team_wait(0)
                s.tick(1)
                # unit 1: block 2's finish done, block 6 preparing
                s.finished(0, 2)
                assert s.decide(0) == ("prepare", 6)
                nxt = 6
            else:
                # unit 0: no job, both services idle, the scanner reading
                s.decide(0), s.decide(1)
                s.team_wait(0)
                s.tick(1)
                # unit 1: block 2 issued and preparing
                s.issue(2)
                assert s.decide(0) == ("prepare", 2)
                nxt = 2
            s.tick(1)
            s.prepared_(0, nxt)
            s.team_take(0)
            s.end()
            r = s.analyse()
            logs[case] = (r, wait_of(r, 0))
        wa, wb = logs["a"][1], logs["b"][1]
        self.assertEqual((wa["label"], wb["label"]), ("prep", "prep"))
        self.assertEqual((wa["classes"]["finish_first"], wb["classes"]["finish_first"]), (1, 0))
        self.assertEqual(wb["other"]["scan_or_input"], 1)
        self.assertEqual(wa["classes"]["prep_in_progress"], wb["classes"]["prep_in_progress"])
        # the witness's marginals: equal busy sets, 7 units in 10, do not fix the overlap with
        # the selected starvation unit (unit 0), which is what tells the cases apart above
        busy_a, busy_b = set(range(7)), set(range(3, 10))
        starvation = {0}
        self.assertEqual(len(busy_a), len(busy_b))
        self.assertEqual((len(starvation & busy_a), len(starvation & busy_b)), (1, 0))


class FrontAvailability(unittest.TestCase):
    """witnesses.py's front_availability: 5,040 states, at least two fronts free at any
    eligible preparation, so the analyser never finds an eligible prep without a front"""

    def test_5040_states(self):
        states = 0
        for t, l, k in product(range(1, 5), range(1, 9), range(1, 5)):
            for p, a, u in product(range(l), range(k), range(t + 1)):
                f = t + l + k
                h = dict(
                    j=1, p=p, a=a, u=u, q=0, c=0, free_front=f - p - a - u, free_back=2 * t - u
                )
                self.assertGreaterEqual(h["free_front"], 2)
                self.assertFalse(d.no_front_defect(h, l))
                states += 1
        self.assertEqual(states, 5040)
        # and a lost front is found
        self.assertTrue(d.no_front_defect(dict(j=1, p=0, free_front=0), 2))


class WindowCredit(unittest.TestCase):
    """The report's deterministic checks: a full window with later results ready and the
    earliest missing block finishing; feeder lag; startup and drain apart"""

    def test_a_full_window_with_later_results_ready_is_window_credit(self):
        s = Sim(T=2, K=2, L=2, W=8)
        history(s, 0, 0)
        history(s, 1, 1)
        # blocks 2..9 issued: the window full (8 issued, none of them taken)
        for p in range(2, 10):
            s.issue(p)
        assert s.decide(0) == ("prepare", 2)
        s.prepared_(0, 2)
        s.team_take(0)
        s.team_walked(0, 2)
        assert s.decide(0) == ("prepare", 3)
        s.prepared_(0, 3)
        # service 1 takes block 2, the reader's next, to finish (a long one)
        assert s.decide(1) == ("finish", 2)
        # service 0 does blocks 3..9 meanwhile, each published, held behind block 2
        for p in range(3, 10):
            s.team_take(0)
            s.team_walked(0, p)
            if p < 9:
                assert s.decide(0) == ("prepare", p + 1)
                s.prepared_(0, p + 1)
            assert s.decide(0) == ("finish", p)
            s.finished(0, p)
        s.slot_wait()
        self.assertEqual(s.w["issued"] - s.w["taken"], 8)
        self.assertEqual((s.h["j"], s.h["p"], s.h["a"], s.h["c"]), (0, 0, 0, 1))
        s.team_wait(0)
        s.tick(4)
        # block 2 published: the reader takes 2..9, the scanner gets its credit
        s.finished(1, 2)
        for _ in range(8):
            s.take()
        s.emit("mibz2-scan", "SlotGot", None, 1, win=True)
        s.tick(1)  # the scanner reading block 10
        s.issue(10)
        assert s.decide(0) == ("prepare", 10)
        s.tick(2)
        s.prepared_(0, 10)
        s.team_take(0)
        s.end()
        r = s.analyse()
        w = wait_of(r, 0)
        self.assertEqual(w["classes"]["window_credit"], 4)
        self.assertEqual(dict(w["missing_stage"]), {"finishing": 4})
        self.assertEqual(w["other"]["scan_or_input"], 1)
        self.assertEqual(w["classes"]["prep_in_progress"], 2)
        summ = d.summarise([r])
        self.assertEqual(summ["window_credit_missing_stage"], {"finishing": 1.0})
        self.assertEqual(summ["slot_waits"], 1)

    def test_feeder_lag(self):
        s = Sim(T=1, K=1, L=1)
        history(s, 0, 0)
        s.decide(0)  # no job: wait
        s.team_wait(0)
        s.issue(1, enqueue=True, feed=False)
        s.tick(2)  # issued, not yet in the hub's queue
        s.feed(1)
        assert s.decide(0) == ("prepare", 1)
        s.tick(1)
        s.prepared_(0, 1)
        s.team_take(0)
        s.end()
        w = wait_of(s.analyse(), 0)
        self.assertEqual(w["other"]["feeder_lag"], 2)
        self.assertEqual(w["classes"]["prep_in_progress"], 1)

    def test_startup_and_drain_apart(self):
        s = Sim(T=1, K=1, L=1)
        s.team_wait(0)
        s.issue(0)
        s.decide(0)
        s.tick(3)  # startup: the team's first block preparing
        s.prepared_(0, 0)
        s.team_take(0)
        s.close()
        s.team_walked(0, 0)
        s.team_wait(0)
        assert s.decide(0) == ("finish", 0)
        s.tick(2)  # drain: no job will come
        s.finished(0, 0)
        s.emit("mibz2-0", "TeamExit", None, hub=True)
        s.end()
        r = s.analyse()
        first, last = r["waits"]
        self.assertEqual(first["classes"], {"startup": 3})
        self.assertEqual(last["classes"], {"drain": 2})
        summ = d.summarise([r])
        self.assertIsNone(summ["classes"]["startup"]["share_excl_startup_drain"])
        self.assertEqual(summ["classes"]["finish_first"]["share_excl_startup_drain"], 0.0)


class FixedSplit(unittest.TestCase):
    """witnesses.py's fixed_split_bound, as arithmetic (no classification)"""

    def test_400_pairs(self):
        n = 0
        for p, f in product(range(1, 21), repeat=2):
            self.assertLessEqual(min(Fraction(1, p), Fraction(1, f)), Fraction(2, p + f))
            n += 1
        self.assertEqual(n, 400)


class Defects(unittest.TestCase):
    def test_a_broken_identity_or_unowned_retire_is_a_defect(self):
        s = Sim(T=1, K=1, L=1)
        history(s, 0, 0)
        bad = dict(s.h, free_front=s.h["free_front"] - 1)
        s.events.append(
            dict(
                seq=999,
                ns=s.t,
                thread="mibz2-0",
                kind="RetireUnowned",
                place=None,
                arg=0,
                n=0,
                hub=bad,
                win=None,
            )
        )
        r = s.analyse()
        self.assertEqual(r["defects"]["front_identity"], 1)
        self.assertEqual(r["defects"]["retire_unowned"], 1)


if __name__ == "__main__":
    unittest.main()
