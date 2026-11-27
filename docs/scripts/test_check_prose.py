"""Tests for check_prose.py's reader gate: shorthand and claims-match-state rules.

    python3 docs/scripts/test_check_prose.py
"""

import io
import pathlib
import sys
import tempfile
import unittest
from contextlib import redirect_stdout

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import check_prose as cp  # noqa: E402

REPO = pathlib.Path(__file__).resolve().parents[2]
BLOG_DIR = REPO / "docs" / "site" / "blog"
NOTEBOOK_6GBS = REPO / "docs" / "notebook" / "2026-11-27-six-gbs-decoder-architecture.md"
SKILL_MD = REPO / ".claude" / "skills" / "write-blog-post" / "SKILL.md"
FRONTIER_README = REPO / "frontier" / "README.md"

PLAIN = "The decoder rebuilds each block from a few retained rows and the words between them. " * 40
SHORTHAND = (
    "On `wikidata ×4` the `scanner-seam-window` (`#1118`) ran `1.89%` faster (`0.9811`) at `10T`, "
    "while `READY_BYTES` gated `Shared::full` (#1094, #1103). "
) * 20

SAMPLE_LEDGER = {
    1040: {"pr": "1040", "state": "default", "effect": "Prechecks dense blocks."},
    1129: {"pr": "1129", "state": "default", "effect": "Bounds peak memory on compressible files."},
    1110: {"pr": "1110", "state": "experimental", "effect": "Four-lane forward walk behind a feature flag."},
    2001: {"pr": "2001", "state": "prototype", "effect": "Prototype composed worker pipeline."},
    2002: {"pr": "2002", "state": "target", "effect": "Six gigabytes per second eight-worker target."},
    2003: {"pr": "2003", "state": "awaiting-review", "effect": "Batched move-to-front fold awaiting review."},
}


def post(title, body, companion=False, dek="A plain subtitle describing the decoder result with no unsupported headline figures at all."):
    kicker = "[mibz2 · Research blog](index.md)" + (" · Companion to [X](x.md)" if companion else "")
    return (
        f'---\ntitle: "{title}"\ndescription: "{dek}"\n---\n\n'
        f"{kicker}\n{{: .post-kicker }}\n\n"
        f"# {title}\n\n"
        f"{dek}\n{{: .post-dek }}\n\n"
        f"{body}\n\n---\n\n**Sources:**\n\n- #1118, #1119, `abc1234f`\n"
    )


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


class ClaimsMatchState(unittest.TestCase):
    def test_headline_figure_detection_and_non_figures(self):
        self.assertEqual(cp.headline_figures("The architecture of a 6 GB/s bzip2 decoder"), ["6 GB/s"])
        self.assertEqual(cp.headline_figures("Peak memory drops 51.2% to 545 MiB in 140 ms"), ["51.2%", "545 MiB", "140 ms"])
        self.assertEqual(cp.headline_figures("Zero-copy drops plumbing to 5 milliseconds and 20 bytes"), ["5 milliseconds", "20 bytes"])
        self.assertEqual(cp.headline_figures("32 lanes, 8.6 chains in mibz2 0.5.8 from the 1990s"), [])
        self.assertEqual(cp.headline_figures("A 64-byte record across ten cores"), [])

    def test_title_or_dek_figure_fails_when_changes_include_non_default(self):
        title_post = post("Faster decoding at 6 GB/s on ten cores", PLAIN)
        dek_post = post(
            "Bounded memory on compressible streams",
            PLAIN,
            dek="Pausing workers cuts peak memory by 51% to 545 MiB on two gigabytes of zeros.",
        )

        # Non-default state in card changes -> FAIL for both title and dek figures.
        for bad_card in (
            {"changes": [1040, 1110]},
            {"changes": [2001]},
            {"changes": [2002]},
            {"changes": [2003]},
            {"changes": [9999]},
            {"changes": []},
            {},
        ):
            self.assertTrue(
                cp.headline_state_problems(title_post, card=bad_card, ledger=SAMPLE_LEDGER),
                f"expected title figure to fail for card={bad_card}",
            )
            self.assertTrue(
                cp.headline_state_problems(dek_post, card=bad_card, ledger=SAMPLE_LEDGER),
                f"expected dek figure to fail for card={bad_card}",
            )

        # Missing card on an undated/new post -> FAIL.
        self.assertTrue(cp.headline_state_problems(title_post, card=None, ledger=SAMPLE_LEDGER))
        self.assertTrue(cp.headline_state_problems(dek_post, card=None, ledger=SAMPLE_LEDGER))

        # All changes in state default -> PASS for both title and dek figures.
        good_card = {"changes": [1040, 1129]}
        self.assertEqual(cp.headline_state_problems(title_post, card=good_card, ledger=SAMPLE_LEDGER), [])
        self.assertEqual(cp.headline_state_problems(dek_post, card=good_card, ledger=SAMPLE_LEDGER), [])

        # Without a headline figure, non-default changes pass the headline gate.
        qualitative_post = post("Stages built to fit together", PLAIN)
        self.assertEqual(
            cp.headline_state_problems(qualitative_post, card={"changes": [1110, 2001]}, ledger=SAMPLE_LEDGER),
            [],
        )

    def test_published_six_gbs_title_fails_as_calibrated(self):
        # The verbatim retired 27 November notebook entry must fail.
        self.assertTrue(NOTEBOOK_6GBS.is_file(), f"missing {NOTEBOOK_6GBS}")
        nb_text = NOTEBOOK_6GBS.read_text(encoding="utf-8")
        self.assertTrue(
            cp.headline_state_problems(nb_text, path=NOTEBOOK_6GBS),
            "expected retired 6 GB/s post to fail headline state gate",
        )

        # A blog post carrying the published 6 GB/s title must fail against the real ledger and card.
        six_gbs_post = post(
            "The architecture of a 6 GB/s bzip2 decoder",
            PLAIN,
            dek="Five merged studies pair four-lane forward emission with slack-only scanning to reach 6 GB/s.",
        )
        real_ledger = cp.load_ledger()
        self.assertTrue(real_ledger, "expected frontier/ledger.tsv to load non-empty rows")
        self.assertTrue(
            cp.headline_state_problems(
                six_gbs_post,
                path="docs/site/blog/2026-11-27-six-gbs-decoder-architecture.md",
                ledger=real_ledger,
            )
        )
        self.assertTrue(
            cp.headline_state_problems(
                six_gbs_post,
                card={"changes": [1040, 1110]},
                ledger=real_ledger,
            )
        )

    def test_pre_26_november_posts_and_queueing_and_complementary_bottlenecks_pass(self):
        pre_posts = [
            p
            for p in sorted(BLOG_DIR.glob("*.md"))
            if p.name != "index.md" and cp._post_date(p) is not None and cp._post_date(p) < cp.LEDGER_BOARD_START
        ]
        self.assertGreaterEqual(len(pre_posts), 40)
        for p in pre_posts:
            text = p.read_text(encoding="utf-8")
            self.assertEqual(
                cp.claim_state_problems(text, path=p),
                [],
                f"pre-26 November post unexpectedly failed state gate: {p.name}",
            )

        for name in (
            "2026-11-25-the-decoder-as-a-queueing-network.md",
            "2026-11-26-complementary-bottlenecks.md",
            "2026-11-27-bounded-memory-on-highly-compressible-input.md",
            "2026-11-27-faster-decoding-for-ordinary-text.md",
            "2026-11-27-stages-built-to-fit-together.md",
        ):
            p = BLOG_DIR / name
            self.assertTrue(p.is_file(), f"missing {p}")
            self.assertEqual(
                cp.claim_state_problems(p.read_text(encoding="utf-8"), path=p),
                [],
                f"calibrated post unexpectedly failed state gate: {name}",
            )

    def test_body_first_mention_requires_state_word_for_non_default_changes(self):
        # 1. Explicit /pull/N link: passes when state word is in the first-mention paragraph.
        good_body = (
            "The [four-lane forward walk](https://github.com/lmmx/mibz2/pull/1110) sits behind an "
            "experimental feature flag and cuts single-worker time.\n\n"
            "A [composed worker](https://github.com/lmmx/mibz2/pull/2001) prototype pairs the stages, "
            "and [eight-worker scaling](https://github.com/lmmx/mibz2/pull/2002) aims for the target rate."
        )
        card = {"changes": [1040, 1110, 2001, 2002]}
        self.assertEqual(
            cp.body_state_problems(post("Stages built to fit together", good_body), card=card, ledger=SAMPLE_LEDGER),
            [],
        )

        # 2. First mention omits state word, even though a later paragraph includes it -> FAIL.
        late_body = (
            "The [four-lane forward walk](https://github.com/lmmx/mibz2/pull/1110) cuts single-worker time.\n\n"
            "Later note: it is still behind an experimental feature flag."
        )
        probs = cp.body_state_problems(
            post("Forward walk", late_body), card={"changes": [1110]}, ledger=SAMPLE_LEDGER
        )
        self.assertEqual(len(probs), 1)
        self.assertIn("#1110", probs[0])

        # 3. Card change_labels plain-prose convention without PR links in body.
        labeled_card = {"changes": [1110], "change_labels": {"1110": "four-lane forward walk"}}
        self.assertEqual(
            cp.body_state_problems(
                post("Forward walk", "The four-lane forward walk is behind an experimental feature."),
                card=labeled_card,
                ledger=SAMPLE_LEDGER,
            ),
            [],
        )
        self.assertTrue(
            cp.body_state_problems(
                post("Forward walk", "The four-lane forward walk cuts worker time.\n\nAn experimental flag exists."),
                card=labeled_card,
                ledger=SAMPLE_LEDGER,
            )
        )

        # 4. Unlinked non-default change fails if body never names its state word, passes if it does.
        self.assertTrue(
            cp.body_state_problems(
                post("Forward walk", "The new walk cuts single-worker time cleanly."),
                card={"changes": [1110]},
                ledger=SAMPLE_LEDGER,
            )
        )
        self.assertEqual(
            cp.body_state_problems(
                post("Forward walk", "The new walk runs behind an experimental feature flag."),
                card={"changes": [1110]},
                ledger=SAMPLE_LEDGER,
            ),
            [],
        )

    def test_changed_mode_gates_edited_headlines_and_ignores_unchanged(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = pathlib.Path(tmp) / "2026-11-27-draft-post.md"
            p.write_text(
                post(
                    "The architecture of a 6 GB/s bzip2 decoder",
                    PLAIN,
                    dek="A plain subtitle describing the decoder result with no unsupported headline figures at all.",
                ),
                encoding="utf-8",
            )
            orig_text_at = cp.text_at
            try:
                # New post (no version at base) -> FAIL.
                cp.text_at = lambda base, path: None
                buf = io.StringIO()
                with redirect_stdout(buf):
                    rc_new = cp.check(str(p), show_all=False, base="origin/master")
                self.assertEqual(rc_new, 1)
                self.assertIn("state gate (title): FAIL", buf.getvalue())

                # Unchanged title at base -> reported, not gated (PASS).
                cp.text_at = lambda base, path: p.read_text(encoding="utf-8")
                buf = io.StringIO()
                with redirect_stdout(buf):
                    rc_unchanged = cp.check(str(p), show_all=False, base="origin/master")
                self.assertEqual(rc_unchanged, 0)
                self.assertIn("unchanged since origin/master", buf.getvalue())
            finally:
                cp.text_at = orig_text_at

    def test_skill_and_readme_document_the_rule(self):
        skill = SKILL_MD.read_text(encoding="utf-8")
        readme = FRONTIER_README.read_text(encoding="utf-8")
        for doc_name, content in (("write-blog-post SKILL.md", skill), ("frontier/README.md", readme)):
            for needle in ("frontier/ledger.tsv", "default", "experimental", "prototype", "target"):
                self.assertIn(needle, content, f"{doc_name} missing {needle!r}")


if __name__ == "__main__":
    unittest.main()
