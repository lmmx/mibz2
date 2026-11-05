"""Offline tests for coord.py: run with `python3 -m unittest discover -s .github/coordination`.

fixtures.json holds the header lines of every comment on #145, #160, #161 and #162 up to
2026-11-05 08:49 UTC, the channel's whole history when this was written.
"""

import io
import json
import os
import unittest
from contextlib import redirect_stdout
from unittest import mock

import coord

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURES = json.load(open(os.path.join(HERE, "fixtures.json")))
MISROUTED = {6652503445, 6653242752}  # the two Optimizer checkpoints posted on #145


def msg(**h):
    return "\n".join(f"{k}: {v}" for k, v in h.items()) + "\n\nbody"


class History(unittest.TestCase):
    def test_flags_exactly_the_known_misroutes(self):
        flagged = {c["id"] for c in FIXTURES
                   if coord.misrouted(c["issue"], c["header"], c["id"])}
        self.assertEqual(flagged, MISROUTED)

    def test_before_the_cutover_checkpoints_on_the_channel_were_correct(self):
        old = [c for c in FIXTURES if c["issue"] == 145 and c["id"] < coord.CUTOVER
               and "CHECKPOINT" in c["header"] and "TO:" in c["header"]]
        self.assertGreater(len(old), 20)
        for c in old:
            self.assertIsNone(coord.misrouted(145, c["header"], c["id"]))

    def test_inbox_comments_before_the_queue_began_are_history(self):
        # the Guardian's REVIEW of 2026-11-04 23:42 in the Optimizer's inbox, before its offset
        c = next(c for c in FIXTURES if c["id"] == 6645337230)
        self.assertEqual(c["issue"], 160)
        self.assertLess(c["id"], coord.ROLES["optimizer"]["offset"])
        self.assertIsNone(coord.misrouted(160, c["header"], c["id"]))
        self.assertIsNotNone(coord.misrouted(160, c["header"]))  # the same, live

    def test_the_misroute_names_the_right_inbox(self):
        c = next(c for c in FIXTURES if c["id"] == 6652503445)
        reason = coord.misrouted(145, c["header"], c["id"])
        self.assertIn("#160", reason)
        self.assertIn("QUEUED", reason)


class Audit(unittest.TestCase):
    def test_resolved_misroutes_do_not_fail_the_audit(self):
        history = {n: [comment(c["issue"], c["id"], c["header"]) for c in FIXTURES
                       if c["issue"] == n] for n in (145, 160, 161, 162)}
        with mock.patch.object(coord, "comments", lambda n: history[n]), \
                redirect_stdout(io.StringIO()):
            self.assertEqual(coord.main(["audit"]), 0)
        late = comment(145, 7_000_000_000, msg(FROM="coordinator", TO="guardian",
                                               TYPE="CHECKPOINT"))
        history[145] = history[145] + [late]
        with mock.patch.object(coord, "comments", lambda n: history[n]), \
                redirect_stdout(io.StringIO()):
            self.assertEqual(coord.main(["audit"]), 1)


class Channel(unittest.TestCase):
    def setUp(self):
        self.new = coord.CUTOVER + 10_000_000

    def test_publications_pass(self):
        for kind in ("ACK", "REVIEW", "DECISION", "QUEUED"):
            body = msg(FROM="coordinator", TO="astra", TYPE=kind)
            self.assertIsNone(coord.misrouted(145, body, self.new), kind)

    def test_a_coordinator_checkpoint_with_no_role_is_a_record(self):
        for to in (None, "record", "coordinator"):
            h = {"FROM": "coordinator", "TYPE": "CHECKPOINT"}
            if to:
                h["TO"] = to
            self.assertIsNone(coord.misrouted(145, msg(**h), self.new))

    def test_assignments_to_any_role_are_flagged(self):
        for to, inbox in (("optimizer", "#160"), ("Astra", "#160"), ("guardian", "#161"),
                          ("researcher", "#162")):
            for kind in ("CHECKPOINT", "PROPOSAL"):
                reason = coord.misrouted(145, msg(FROM="coordinator", TO=to, TYPE=kind),
                                         self.new)
                self.assertIsNotNone(reason, (to, kind))
                self.assertIn(inbox, reason)

    def test_a_broadcast_names_every_inbox(self):
        reason = coord.misrouted(
            145, msg(FROM="coordinator", TO="guardian, researcher, optimizer",
                     TYPE="CHECKPOINT"), self.new)
        for inbox in ("#160", "#161", "#162"):
            self.assertIn(inbox, reason)

    def test_other_senders_on_the_channel_pass(self):
        body = msg(FROM="astra", TO="coordinator, guardian", TYPE="REVIEW")
        self.assertIsNone(coord.misrouted(145, body, self.new))

    def test_the_warning_itself_never_trips(self):
        text = coord.warning(145, 1, "a CHECKPOINT ...")
        self.assertIsNone(coord.misrouted(145, text, self.new))
        self.assertIn(coord.MARKER.format(id=1), text)


class Inbox(unittest.TestCase):
    def test_a_checkpoint_to_its_role_passes(self):
        for issue, to in ((160, "optimizer"), (160, "astra"), (161, "guardian"),
                          (162, "researcher")):
            self.assertIsNone(coord.misrouted(
                issue, msg(FROM="coordinator", TO=to, TYPE="CHECKPOINT")), (issue, to))

    def test_the_wrong_role_is_flagged(self):
        self.assertIn("guardian", coord.misrouted(
            160, msg(FROM="coordinator", TO="guardian", TYPE="CHECKPOINT")))

    def test_several_roles_in_one_inbox_are_flagged(self):
        self.assertIsNotNone(coord.misrouted(
            161, msg(FROM="coordinator", TO="guardian, researcher", TYPE="CHECKPOINT")))

    def test_only_the_coordinator_writes_in_an_inbox(self):
        self.assertIn("Only the coordinator", coord.misrouted(
            160, msg(FROM="guardian", TYPE="REVIEW")))
        self.assertIsNotNone(coord.misrouted(160, "no header at all"))

    def test_queued_belongs_on_the_channel(self):
        self.assertIn("#145", coord.misrouted(
            162, msg(FROM="coordinator", TO="researcher", TYPE="QUEUED")))

    def test_other_prs_are_not_checked(self):
        self.assertIsNone(coord.misrouted(183, msg(FROM="coordinator", TO="optimizer",
                                                   TYPE="CHECKPOINT")))


def comment(issue, cid, body, bot=False):
    return {"id": cid, "body": body, "created_at": "2026-11-05T09:00:00Z",
            "issue_url": f"https://api.github.com/repos/lmmx/mibz2/issues/{issue}",
            "user": {"login": "github-actions[bot]" if bot else "lmmx",
                     "type": "Bot" if bot else "User"}}


class Queue(unittest.TestCase):
    """state() and route() against a fake inbox, channel and cursor."""
    off = coord.ROLES["optimizer"]["offset"]

    def fake(self, inbox, channel, done):
        cur = {"inbox_briefs": [{"comment_id": i, "completed": True} for i in done]}
        return (mock.patch.object(coord, "comments",
                                  lambda n: {160: inbox, 145: channel}.get(n, [])),
                mock.patch.object(coord, "cursor", lambda role: cur))

    def run_route(self, inbox, channel, done):
        a, b = self.fake(inbox, channel, done)
        with a, b:
            return coord.route("optimizer", "CHECKPOINT")

    def test_free_when_every_item_is_complete_and_reviewed(self):
        item = comment(160, self.off + 1, msg(FROM="coordinator", TO="optimizer",
                                              TYPE="CHECKPOINT"))
        review = comment(145, self.off + 2, msg(FROM="astra", TO="coordinator",
                                                TYPE="REVIEW") + f" inbox {self.off + 1}")
        r = self.run_route([item], [review], [self.off + 1])
        self.assertEqual((r["issue"], r["type"]), (160, "CHECKPOINT"))

    def test_busy_while_an_item_is_incomplete(self):
        item = comment(160, self.off + 1, msg(FROM="coordinator", TO="optimizer",
                                              TYPE="CHECKPOINT"))
        r = self.run_route([item], [], [])
        self.assertEqual((r["issue"], r["type"]), (145, "QUEUED"))

    def test_busy_while_a_complete_item_awaits_its_review(self):
        item = comment(160, self.off + 1, msg(FROM="coordinator", TO="optimizer",
                                              TYPE="CHECKPOINT"))
        r = self.run_route([item], [], [self.off + 1])
        self.assertEqual(r["type"], "QUEUED")

    def test_an_older_queued_item_goes_first(self):
        queued = comment(145, self.off + 5, msg(FROM="coordinator", TO="optimizer",
                                                TYPE="QUEUED"))
        r = self.run_route([], [queued], [])
        self.assertEqual(r["issue"], 160)
        self.assertEqual(r["queued"], [self.off + 5])

    def test_a_queued_item_cited_by_the_inbox_is_dispatched(self):
        queued = comment(145, self.off + 5, msg(FROM="coordinator", TO="optimizer",
                                                TYPE="QUEUED"))
        item = comment(160, self.off + 6, msg(FROM="coordinator", TO="optimizer",
                                              TYPE="CHECKPOINT") + f" QUEUED {self.off + 5}")
        review = comment(145, self.off + 7, msg(FROM="astra", TYPE="REVIEW")
                         + f" {self.off + 6}")
        r = self.run_route([item], [queued, review], [self.off + 6])
        self.assertNotIn("queued", r)

    def test_publications_always_go_to_the_channel(self):
        self.assertEqual(coord.route("optimizer", "ACK")["issue"], 145)


class Verify(unittest.TestCase):
    def check(self, issue, body, role="optimizer", kind="CHECKPOINT", state=None):
        c = comment(issue, 7_000_000_000, body)
        st = state or {"incomplete": [7_000_000_000], "unreviewed": [7_000_000_000],
                       "queued": []}
        with mock.patch.object(coord, "get", lambda url: c), \
                mock.patch.object(coord, "state", lambda r: st):
            return coord.verify(7_000_000_000, role, kind)

    def test_a_delivered_checkpoint_passes(self):
        r = self.check(160, msg(FROM="coordinator", TO="optimizer", TYPE="CHECKPOINT"))
        self.assertTrue(r["ok"], r)

    def test_the_overnight_mistake_fails(self):
        r = self.check(145, msg(FROM="coordinator", TO="optimizer", TYPE="CHECKPOINT"))
        self.assertFalse(r["ok"])
        self.assertTrue(any("#160" in p for p in r["problems"]))

    def test_a_second_assignment_in_flight_fails(self):
        st = {"incomplete": [6_999_999_999, 7_000_000_000], "unreviewed": [], "queued": []}
        r = self.check(160, msg(FROM="coordinator", TO="optimizer", TYPE="CHECKPOINT"),
                       state=st)
        self.assertFalse(r["ok"])

    def test_queued_must_be_on_the_channel(self):
        ok = self.check(145, msg(FROM="coordinator", TO="optimizer", TYPE="QUEUED"),
                        kind="QUEUED")
        self.assertTrue(ok["ok"], ok)

    def test_wrong_type_or_role_fails(self):
        self.assertFalse(self.check(160, msg(FROM="coordinator", TO="optimizer",
                                             TYPE="ACK"))["ok"])
        self.assertFalse(self.check(161, msg(FROM="coordinator", TO="guardian",
                                             TYPE="CHECKPOINT"))["ok"])


class HeadSha(unittest.TestCase):
    head = "c32ba5db9df80a08147c35eeb1fb91259faab965"

    def run_guard(self, given, merged=False, is_merged=False):
        pull = {"head": {"sha": self.head}, "merged": is_merged}
        with mock.patch.object(coord, "get", lambda url: pull):
            return coord.head_sha(185, given, merged)["ok"]

    def test_only_the_exact_head_passes(self):
        self.assertTrue(self.run_guard(self.head))
        self.assertFalse(self.run_guard("c32ba5db"))  # a prefix
        self.assertFalse(self.run_guard("c32ba5db3b0f9eec2b3d0dc3cbe8e4cb24a71c51"))  # the guess
        self.assertFalse(self.run_guard(self.head.upper()))

    def test_after_the_merge_the_pr_is_merged_at_that_head(self):
        self.assertTrue(self.run_guard(self.head, merged=True, is_merged=True))
        self.assertFalse(self.run_guard(self.head, merged=True, is_merged=False))  # not merged
        self.assertFalse(self.run_guard("c32ba5db3b0f9eec2b3d0dc3cbe8e4cb24a71c51",
                                        merged=True, is_merged=True))  # merged, another head
        self.assertTrue(self.run_guard(self.head, merged=False, is_merged=False))  # before

    def test_cli_merged_flag_reaches_the_check(self):
        pull = {"head": {"sha": self.head}, "merged": False}
        with mock.patch.object(coord, "get", lambda url: pull), redirect_stdout(io.StringIO()):
            self.assertEqual(coord.main(["head-sha", "--pr", "185", "--sha", self.head]), 0)
            self.assertEqual(coord.main(["head-sha", "--pr", "185", "--sha", self.head,
                                         "--merged"]), 1)


class Cli(unittest.TestCase):
    def test_warn_prints_only_for_a_misroute(self):
        for body, want in ((msg(FROM="coordinator", TO="optimizer", TYPE="CHECKPOINT"), True),
                           (msg(FROM="coordinator", TO="astra", TYPE="ACK"), False)):
            out = io.StringIO()
            with mock.patch("sys.stdin", io.StringIO(body)), redirect_stdout(out):
                code = coord.main(["warn", "--issue", "145",
                                   "--comment-id", str(coord.CUTOVER + 1)])
            self.assertEqual(code, 0)
            self.assertEqual(bool(out.getvalue().strip()), want)


if __name__ == "__main__":
    unittest.main()
