"""Tests for check_prose.py's reader gate: shorthand fails, plain prose passes.

    python3 docs/scripts/test_check_prose.py
"""
import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import check_prose as cp  # noqa: E402

PLAIN = "The decoder rebuilds each block from a few retained rows and the words between them. " * 40
SHORTHAND = (
    "On `wikidata ×4` the `scanner-seam-window` (`#1118`) ran `1.89%` faster (`0.9811`) at `10T`, "
    "while `READY_BYTES` gated `Shared::full` (#1094, #1103). "
) * 20


def post(title, body, companion=False):
    kicker = "[mibz2 · Research blog](index.md)" + (" · Companion to [X](x.md)" if companion else "")
    return f'---\ntitle: "{title}"\ndescription: "d"\n---\n\n{kicker}\n\n{body}\n\n---\n\n**Sources:**\n\n- #1118, #1119, `abc1234f`\n'


class ReaderGate(unittest.TestCase):
    def test_plain_prose_passes(self):
        problems, _ = cp.shorthand(post("A finding", PLAIN))
        self.assertEqual(problems, [])

    def test_shorthand_fails_on_code_numbers_and_references(self):
        problems, _ = cp.shorthand(post("A finding", SHORTHAND))
        text = " ".join(problems)
        self.assertIn("inline code spans", text)
        self.assertIn("numbers set as code", text)
        self.assertIn("PR or issue references", text)

    def test_sources_list_is_not_counted(self):
        problems, summary = cp.shorthand(post("A finding", PLAIN))
        self.assertEqual(problems, [])
        self.assertIn("PR or issue references 0.0/1k", summary)

    def test_list_title_fails_for_a_main_post_only(self):
        self.assertIsNotNone(cp.title_problem(post("An i9 rejection, a big input, and a bias", PLAIN)))
        self.assertIsNone(cp.title_problem(post("Beneath it: laws, analysis and form, in short", PLAIN, True)))
        self.assertIsNone(cp.title_problem(post("Correctness is uniform, cost is not", PLAIN)))


if __name__ == "__main__":
    unittest.main()
