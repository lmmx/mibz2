"""Synthetic blocker-removal cases for the shadow view."""

import copy
import unittest

from availability import candidates, newly_available


def state():
    return {
        "kind": "committed",
        "items": {
            "run": {"status": "open", "zone": "striated", "owned_paths": ["src/runs.rs"],
                    "mechanisms": ["mtf"], "premises": ["docs/build.md"], "needs": ["rust"]},
        },
        "live_holds": {"build": {"owned_paths": ["src/runs.rs"], "mechanisms": ["mtf"]}},
        "pending": [], "flagged": [], "paths": [], "cap": 3,
    }


class AvailabilityTest(unittest.TestCase):
    def test_merge_then_release(self):
        before = state()
        merged = copy.deepcopy(before)
        merged["paths"] = ["docs/build.md"]
        self.assertEqual(newly_available(before, merged), [])
        released = copy.deepcopy(merged)
        released["live_holds"] = {}
        self.assertEqual(newly_available(merged, released), ["run"])
        self.assertEqual(candidates(released, {"rust"})["available"], ["run"])
        self.assertEqual(candidates(released, {"web"})["available"], [])
        self.assertEqual(newly_available(released, released), [])
        self.assertEqual(candidates(released)["available"], ["run"])

    def test_release_without_committed_evidence_is_not_enough(self):
        before = state()
        after = copy.deepcopy(before)
        after["live_holds"] = {}
        self.assertEqual(newly_available(before, after), [])
        self.assertIn("missing-premise:docs/build.md", candidates(after)["blocked"]["run"])

    def test_other_blockers_remain(self):
        before = state()
        after = copy.deepcopy(before)
        after["paths"] = ["docs/build.md"]
        after["live_holds"] = {}
        after["pending"] = ["run"]
        after["flagged"] = ["run"]
        self.assertEqual(newly_available(before, after), [])
        self.assertEqual(candidates(after)["blocked"]["run"], ["flag", "pending-integration"])
        after["flagged"] = []
        self.assertEqual(candidates(after)["available"], [])
        after["pending"] = []
        self.assertEqual(candidates(after)["available"], ["run"])

    def test_proposal_and_unknown_are_not_eligible(self):
        before = state()
        for change in ({"kind": "proposed"}, {"live_holds": None}, {"pending": None}):
            after = {**before, **change}
            self.assertTrue(candidates(after)["unavailable"])
            self.assertIsNone(newly_available(before, after))

    def test_cap_and_non_open_items(self):
        before = state()
        before["paths"] = ["docs/build.md"]
        before["live_holds"] = {"other": {}}
        before["cap"] = 1
        before["items"]["idea"] = {"status": "idea"}
        before["items"]["blocked"] = {"status": "blocked"}
        after = copy.deepcopy(before)
        after["live_holds"] = {}
        self.assertEqual(newly_available(before, after), ["run"])
        self.assertEqual(candidates(after)["available"], ["run"])


if __name__ == "__main__":
    unittest.main()
