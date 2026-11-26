import unittest
from replay import project


class ProjectionTest(unittest.TestCase):
    def test_lifecycle_and_pre_event_dependencies(self):
        before = {"a": {"status": "idea"}, "b": {"status": "open", "premises": ["docs/x.md"]}}
        after = {"a": {"status": "open"}, "b": {"status": "open", "premises": ["docs/y.md"]}}
        result = project(before, after, {"docs/x.md", "docs/y.md"})
        self.assertEqual(result[0]["reasons"], [{"kind": "state", "before": "idea", "after": "open"}])
        self.assertEqual(result[1]["reasons"], [{"kind": "premises", "paths": ["docs/x.md"]}])

    def test_related_mechanism_is_not_path_match(self):
        items = {"a": {"status": "done", "mechanisms": ["delivery"]}, "b": {"status": "idea", "mechanisms": ["delivery"]}}
        result = project(items, items, {"unrelated.md"}, "a")
        self.assertEqual(result, [{"claim": "b", "status": "idea", "reasons": [{"kind": "related-mechanism", "mechanisms": ["delivery"]}]}])

    def test_path_segments_and_terminal_claims(self):
        items = {"a": {"status": "open", "owned_paths": ["src/run"]}, "b": {"status": "done", "premises": ["src/runs"]}, "c": {"status": "open", "premises": ["src/runs/"]}}
        result = project(items, items, {"src/runs/fold.rs"})
        self.assertEqual([r["claim"] for r in result], ["c"])

    def test_deduplicates_paths_and_rejects_unknown_source(self):
        items = {"a": {"status": "open", "premises": ["docs/", "docs/a.md"]}}
        result = project(items, items, {"docs/a.md"})
        self.assertEqual(result[0]["reasons"], [{"kind": "premises", "paths": ["docs/a.md"]}])
        with self.assertRaises(ValueError):
            project(items, items, set(), "missing")


if __name__ == "__main__":
    unittest.main()
