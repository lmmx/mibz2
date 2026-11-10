"""bench/walk_stats_check.py's negative controls: synthetic ledgers only, no decoder.

Each case is a record set the native diagnostic's §3 must call INVALID, or its §5 NOT
ENGAGED, and a correct set must be VALID and ENGAGED. Among them the Researcher's
counterexample (#145 6760210997): a lost zero-work walker record, with the terminal count
adjusted, which conserved bytes and takes in revision 2's design.

    python3 bench/test_walk_stats_check.py
"""

import copy
import json
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import walk_stats_check as wc  # noqa: E402

CLASSES = dict(enumerate("STTSTTTSST"))  # blocks 1, 2, 4, 5, 6, 9 are T


def ledger(walker="traverse_staged"):
    """Two teams (0 and 1) alternating T blocks; per block helper 1 and 2 walk, the
    leader joins on even blocks, and one helper's call on block 6 is zero-work"""
    recs = []
    t_blocks = [b for b, c in CLASSES.items() if c == "T"]
    for i, b in enumerate(t_blocks):
        team = i % 2
        joined = b % 2 == 0
        calls = {1: (5, 500), 2: (4, 400)}
        if joined:
            calls[0] = (1, 100)
        if b == 6:
            calls[2] = (0, 0)  # found the queue exhausted at once
        for h in (1, 2):
            recs.append(dict(kind="share", team=team, block=b, thread=h))
        for t, (takes, nbytes) in calls.items():
            recs.append(
                dict(
                    kind="call",
                    team=team,
                    block=b,
                    thread=t,
                    role="leader" if t == 0 else "helper",
                    walker=walker,
                    takes=takes,
                    exhausts=1,
                    bytes=nbytes,
                    tail=min(10, nbytes - takes),
                    fragments=3,
                    copied=max(nbytes - takes - 10, 0),
                )
            )
        recs.append(
            dict(
                kind="summary",
                team=team,
                block=b,
                rows=sum(v[1] for v in calls.values()),
                starts=sum(v[0] for v in calls.values()),
                seg_bytes=sum(v[1] for v in calls.values()),
                entered=sum(1 << t for t in calls),
                joined=joined,
                built=0b110,
            )
        )
    for b, c in CLASSES.items():
        if c == "S":
            recs.append(dict(kind="solo", team=b % 2, block=b, rows=20))
    return recs


def lines(recs, end=True):
    out = [json.dumps(r) for r in recs]
    if end:
        out.append(json.dumps(dict(kind="end", records=len(recs))))
    return out


def run(recs, walker="traverse_staged", end=True):
    by, bad = wc.validate(lines(recs, end), CLASSES, 2)
    if bad:
        return "INVALID", bad
    preds = wc.predicates(by, walker, 2)
    return ("ENGAGED" if all(h for h, _ in preds.values()) else "NOT ENGAGED"), preds


def find(recs, **kw):
    return next(i for i, r in enumerate(recs) if all(r.get(k) == v for k, v in kw.items()))


class Check(unittest.TestCase):
    def test_correct_is_valid_and_engaged(self):
        self.assertEqual(run(ledger())[0], "ENGAGED")

    def test_lost_zero_work_helper_record(self):
        r = ledger()
        i = find(r, kind="call", block=6, thread=2)
        self.assertEqual(r[i]["takes"], 0)
        del r[i]  # (the terminal count follows the file: `lines` recounts it)
        self.assertEqual(run(r)[0], "INVALID")

    def test_lost_zero_work_leader_record(self):
        r = ledger()
        i = find(r, kind="call", block=2, thread=0)
        r[i].update(takes=0, bytes=0, tail=0, copied=0)
        s = find(r, kind="summary", block=2)
        r[s]["starts"] -= 1
        r[s]["seg_bytes"] -= 100
        self.assertEqual(run(r)[0], "ENGAGED")  # zero-work, but present: fine
        del r[i]
        self.assertEqual(run(r)[0], "INVALID")

    def test_absent_helper_call_is_not_engaged(self):
        r = ledger()
        del r[find(r, kind="call", block=6, thread=2)]
        s = find(r, kind="summary", block=6)
        r[s]["entered"] &= ~(1 << 2)  # it never entered the walk: a fact, not a loss
        verdict, preds = run(r)
        self.assertEqual(verdict, "NOT ENGAGED")
        self.assertFalse(preds["a"][0])

    def test_helpers_absent_from_a_t_block(self):
        """The Researcher's counterexample (REVIEW 6763089509 item 3, probes.py at
        07b453d1): two T blocks, the first with the leader only (no share, no helper call,
        built 0), the second with both helpers. The team-wide roster held and (a)-(d) all
        held; the per-block roster must call it INVALID"""
        classes = {0: "T", 1: "T"}

        def call(b, h):
            return dict(
                kind="call",
                team=0,
                block=b,
                thread=h,
                role="leader" if h == 0 else "helper",
                walker="traverse",
                takes=1,
                exhausts=1,
                bytes=1,
                tail=0,
                fragments=0,
                copied=0,
            )

        rows = [
            call(0, 0),
            dict(
                kind="summary",
                team=0,
                block=0,
                rows=1,
                starts=1,
                seg_bytes=1,
                entered=1,
                joined=True,
                built=0,
            ),
        ]
        for h in (1, 2):
            rows += [call(1, h), dict(kind="share", team=0, block=1, thread=h)]
        rows.append(
            dict(
                kind="summary",
                team=0,
                block=1,
                rows=2,
                starts=2,
                seg_bytes=2,
                entered=6,
                joined=False,
                built=6,
            )
        )
        by, bad = wc.validate(lines(rows), classes, 2)
        self.assertTrue(any("block 0: built set 0" in x for x in bad), bad)
        self.assertTrue(any("block 0: share records from []" in x for x in bad), bad)
        # the second block alone is correct: the check names only the first
        self.assertFalse(any("block 1" in x for x in bad), bad)

    def test_one_helper_absent_from_a_t_block(self):
        r = ledger()
        del r[find(r, kind="share", block=4, thread=2)]
        del r[find(r, kind="call", block=4, thread=2)]
        s = find(r, kind="summary", block=4)
        r[s]["built"] = 0b010
        r[s]["entered"] &= ~(1 << 2)
        r[s]["starts"] -= 4
        r[s]["seg_bytes"] -= 400
        r[s]["rows"] -= 400
        verdict, bad = run(r)
        self.assertEqual(verdict, "INVALID")
        self.assertTrue(any("block 4: built set 10" in x for x in bad), bad)

    def test_zero_work_calls_reported(self):
        verdict, preds = run(ledger())
        self.assertEqual(verdict, "ENGAGED")
        self.assertIn("1 zero-work helper calls", preds["a"][1])

    def test_duplicate_record(self):
        r = ledger()
        r.append(copy.deepcopy(r[find(r, kind="call", block=1, thread=1)]))
        self.assertEqual(run(r)[0], "INVALID")

    def test_misattributed_team_or_block(self):
        r = ledger()
        r[find(r, kind="call", block=1, thread=1)]["team"] = 1
        self.assertEqual(run(r)[0], "INVALID")
        r = ledger()
        r[find(r, kind="call", block=1, thread=1)]["block"] = 3
        self.assertEqual(run(r)[0], "INVALID")

    def test_missing_summary(self):
        r = ledger()
        del r[find(r, kind="summary", block=4)]
        self.assertEqual(run(r)[0], "INVALID")

    def test_missing_solo(self):
        r = ledger()
        del r[find(r, kind="solo", block=0)]
        self.assertEqual(run(r)[0], "INVALID")

    def test_bytes_off_by_one(self):
        r = ledger()
        r[find(r, kind="call", block=5, thread=1)]["bytes"] += 1
        self.assertEqual(run(r)[0], "INVALID")

    def test_negative_or_overflowed_counter(self):
        for v in (-1, 2**64, "7", True):
            r = ledger()
            r[find(r, kind="call", block=5, thread=1)]["fragments"] = v
            self.assertEqual(run(r)[0], "INVALID", v)

    def test_terminal_record(self):
        self.assertEqual(run(ledger(), end=False)[0], "INVALID")
        r = ledger()
        bad = lines(r)
        bad[-1] = json.dumps(dict(kind="end", records=len(r) + 1))
        self.assertTrue(wc.validate(bad, CLASSES, 2)[1])

    def test_exhausts_not_one(self):
        r = ledger()
        r[find(r, kind="call", block=1, thread=2)]["exhausts"] = 2
        self.assertEqual(run(r)[0], "INVALID")

    def test_lost_share(self):
        r = ledger()
        del r[find(r, kind="share", block=9, thread=1)]
        self.assertEqual(run(r)[0], "INVALID")

    def test_unbalanced_team_is_not_engaged(self):
        r = ledger()
        # give team 0's helper 2 nearly everything: 3 * max > 2 * sum
        for i, x in enumerate(r):
            if x["kind"] == "call" and x["team"] == 0 and x["thread"] == 2:
                s = find(r, kind="summary", block=x["block"])
                r[s]["seg_bytes"] += 4000 - x["bytes"]
                x["bytes"] = 4000
        verdict, preds = run(r)
        self.assertEqual(verdict, "NOT ENGAGED")
        self.assertFalse(preds["b"][0])

    def test_exactly_two_thirds_holds(self):
        by = {"call": [], "share": [], "summary": [dict(team=0, block=1)]}
        for h, n in ((1, 200), (2, 100)):
            by["share"].append(dict(team=0, block=1, thread=h))
            by["call"].append(dict(team=0, block=1, thread=h, walker="traverse_staged", bytes=n))
        by["call"].append(dict(team=0, block=1, thread=0, walker="traverse_staged", bytes=0))
        self.assertTrue(wc.predicates(by, "traverse_staged", 2)["b"][0])
        by["call"][0]["bytes"] = 201
        self.assertFalse(wc.predicates(by, "traverse_staged", 2)["b"][0])

    def test_zero_denominator_is_not_engaged(self):
        by = {"call": [], "share": [], "summary": [dict(team=0, block=1)]}
        for h in (1, 2):
            by["share"].append(dict(team=0, block=1, thread=h))
            by["call"].append(dict(team=0, block=1, thread=h, walker="traverse_staged", bytes=0))
        self.assertFalse(wc.predicates(by, "traverse_staged", 2)["b"][0])

    def test_the_other_walker(self):
        verdict, preds = run(ledger("traverse"), walker="traverse_staged")
        self.assertEqual(verdict, "NOT ENGAGED")
        self.assertFalse(preds["d"][0])
        self.assertFalse(preds["c"][0])
        self.assertEqual(run(ledger("traverse"), walker="traverse")[0], "ENGAGED")


if __name__ == "__main__":
    unittest.main()

