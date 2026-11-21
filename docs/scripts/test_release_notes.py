"""The release notes (release_notes.py): CHANGELOG.md's sections as each release's lede, "why"
sentences and changes, and the blog posts tagged with the releases they explain.

    python3 docs/scripts/test_release_notes.py
"""

import re
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import release_notes as rn  # noqa: E402

SYNTHETIC = """# Changelog

## [Unreleased]

## [1.2.0](https://github.com/lmmx/mibz2/compare/v1.1.0...v1.2.0) - 2026-12-02

### <!-- 3 -->Performance

- *(walk)* eight lanes a turn ([#900](https://github.com/lmmx/mibz2/pull/900))

- **The walk in lanes (#900): 4% faster at 8 threads, at the same
  peak memory.** Two things changed:
  - the lanes;
  - the turn.

**Release scaling** (§5; run-1): against 1.1.0 the release is faster at 8 threads.
- 8 threads: +4.0% (+3.1, +4.9)
- 10 threads: +2.0% (+1.0, +3.0)

### <!-- 4 -->Documentation

- *(blog)* the lanes post ([#901](https://github.com/lmmx/mibz2/pull/901))

## [1.1.0](https://github.com/lmmx/mibz2/compare/v1.0.0...v1.1.0) - 2026-12-01

A plan release. On ten cores the decoder runs one worker fewer (#890), e.g. at 10 threads
nine workers.

- **Caveats:**
  - one machine.
- **One worker fewer** (#890). It frees a core for the scanner.

| threads | wall |
|---|---|
| 10 | 0.95 |

<details>
<summary>Raw</summary>

- **Hidden:** never read.

</details>

### <!-- 2 -->Bug Fixes

- *(trace)* account for the slack
- **A breaking fix, kept** ([#892](https://github.com/lmmx/mibz2/pull/892))
  - *Mechanism:* the fix.

### Measured

- **Result:** faster at every count.
- **The table:** below.

### <!-- 9 -->Other

- one worker fewer on ten cores ([#890](https://github.com/lmmx/mibz2/pull/890))
- a queue entry ([#891](https://github.com/lmmx/mibz2/pull/891))

## [1.0.0](https://github.com/lmmx/mibz2/releases/tag/v1.0.0) - 2026-11-30

### <!-- 9 -->Other

- the same code, released again

The first release under the new plan: every machine, every thread count and every corpus is
decoded with the plan it had before, so the figures in this section are unchanged from the
last one, which a closing note would not say at such length.
"""


class Synthetic(unittest.TestCase):
    def setUp(self):
        self.notes = {r["version"]: r for r in rn.parse(SYNTHETIC)}

    def test_versions_and_links(self):
        self.assertEqual(list(self.notes), ["1.2.0", "1.1.0", "1.0.0"])
        r = self.notes["1.2.0"]
        self.assertEqual(r["date"], "2026-12-02")
        self.assertEqual(r["anchor"], "release-1-2-0")
        self.assertTrue(r["changelog"].endswith("CHANGELOG.md#120---2026-12-02"))
        self.assertTrue(r["compare"].endswith("v1.1.0...v1.2.0"))

    def test_bold_summaries_are_the_why(self):
        r = self.notes["1.2.0"]
        self.assertIsNone(r["lede"])
        self.assertEqual(r["why"], [
            "The walk in lanes (#900): 4% faster at 8 threads, at the same peak memory.",
            "Release scaling (§5; run-1): against 1.1.0 the release is faster at 8 threads.",
        ])
        # the list straight under the release-scaling summary is its own, not a change
        self.assertEqual([c["prs"][0]["n"] for c in r["changes"]], [900])
        c = r["changes"][0]
        self.assertEqual((c["kind"], c["scope"], c["title"]), ("perf", "walk", "eight lanes a turn"))
        self.assertEqual(c["prs"][0]["url"], "https://github.com/lmmx/mibz2/pull/900")

    def test_lede_labels_tables_and_details(self):
        r = self.notes["1.1.0"]
        # "e.g." does not end the sentence, and the lede is the first sentence plus the next
        # when it is under four words
        self.assertEqual(r["lede"], "A plan release. On ten cores the decoder runs one worker fewer (#890), e.g. at 10 threads nine workers.")
        # "Caveats:" over a list, the Measured section's labels and <details> are left out
        self.assertEqual(r["why"], ["One worker fewer (#890). It frees a core for the scanner."])

    def test_changes_fixes_and_cited_others(self):
        r = self.notes["1.1.0"]
        got = [(c["kind"], c["scope"], c["title"], [p["n"] for p in c["prs"]]) for c in r["changes"]]
        self.assertEqual(got, [
            ("fix", "trace", "account for the slack", []),
            ("fix", None, "A breaking fix, kept", [892]),
            # Other's bullets only where the notes cite their pull request: #890, not #891
            ("other", None, "one worker fewer on ten cores", [890]),
        ])
        # documentation bullets are never changes
        self.assertNotIn(901, [p["n"] for c in self.notes["1.2.0"]["changes"] for p in c["prs"]])

    def test_lede_under_a_later_section(self):
        r = self.notes["1.0.0"]
        self.assertTrue(r["lede"].startswith("The first release under the new plan"))
        self.assertEqual(r["changes"], [])

    def test_sentences(self):
        self.assertEqual(rn.sentences("It is 0.95 of the wall. Next: more."), ["It is 0.95 of the wall.", "Next: more."])
        self.assertEqual(rn.sentences("See e.g. the table. Then #12 merged."), ["See e.g. the table.", "Then #12 merged."])
        self.assertIsNone(rn.first_sentence("**Caveats:**"))
        long = "word " * 120 + "end."
        self.assertTrue(rn.first_sentence(long).endswith("…"))


class Posts(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.blog = Path(self.tmp.name)
        self.notes = rn.parse(SYNTHETIC)
        self.by = {r["version"]: r for r in self.notes}
        (self.blog / "2026-12-02-lanes.md").write_text(
            '---\ntitle: "Lanes"\ndescription: "d"\nreleases: ["1.2.0", "1.1.0"]\n---\n\n'
            "# Lanes\n\n1 December · [Research result](#x)\n{: .post-meta }\n\n</div>\n\nBody, see [#900](https://github.com/lmmx/mibz2/pull/900) and [#890](https://github.com/lmmx/mibz2/pull/890).\n"
        )
        (self.blog / "2026-12-01-other.md").write_text(
            '---\ntitle: "Other"\n---\n\n# Other\n\nx\n{: .post-meta }\n\nIt cites [#890](https://github.com/lmmx/mibz2/pull/890).\n'
        )
        (self.blog / "index.md").write_text(
            '<a class="post-card" href="2026-12-02-lanes.md"><span class="post-card-text"><span class="post-card-dek">d</span></span></a>\n'
            '<a class="post-card" href="2026-12-01-other.md"><span class="post-card-text"><span class="post-card-dek">e</span></span></a>\n'
        )

    def tearDown(self):
        self.tmp.cleanup()

    def test_tags_lines_chips_and_candidates(self):
        posts = rn.posts(self.blog)
        rn.attach(self.notes, posts)
        self.assertEqual([p["title"] for p in self.by["1.2.0"]["posts"]], ["Lanes"])
        self.assertEqual([p["path"] for p in self.by["1.1.0"]["posts"]], ["blog/2026-12-02-lanes/"])
        # the check fails before the lines are written, and passes after; writing is idempotent
        self.assertTrue(rn.sync_posts(posts, self.by, self.blog, write=False))
        self.assertEqual(rn.sync_posts(posts, self.by, self.blog, write=True), [])
        self.assertEqual(rn.sync_posts(posts, self.by, self.blog, write=False), [])
        text = (self.blog / "2026-12-02-lanes.md").read_text()
        self.assertIn("{: .post-meta }\n\nShipped in mibz2 [1.2.0](../benchmarks/history.md#release-1-2-0)", text)
        self.assertIn(" and [1.1.0](../benchmarks/history.md#release-1-1-0)", text)
        self.assertEqual(text.count(".post-shipped"), 1)
        index = (self.blog / "index.md").read_text()
        self.assertIn('<span class="post-card-rel">Shipped in 1.2.0 and 1.1.0</span></span></a>', index)
        self.assertEqual(index.count("post-card-rel"), 1)
        # an untagged post that links a shipped pull request is a candidate, never applied
        self.assertEqual(rn.candidates(self.notes, posts), [
            {"post": "2026-12-01-other.md", "release": "1.1.0", "pr": 890, "change": "one worker fewer on ten cores"},
        ])
        # dropping the tag takes the line and the chip away again
        p = self.blog / "2026-12-02-lanes.md"
        p.write_text(p.read_text().replace('releases: ["1.2.0", "1.1.0"]\n', ""))
        posts = rn.posts(self.blog)
        self.assertEqual(rn.sync_posts(posts, self.by, self.blog, write=True), [])
        self.assertNotIn("Shipped in", p.read_text())
        self.assertNotIn("post-card-rel", (self.blog / "index.md").read_text())

    def test_unknown_release_fails(self):
        (self.blog / "2026-12-03-bad.md").write_text('---\ntitle: "Bad"\nreleases: ["9.9.9"]\n---\n')
        with self.assertRaises(SystemExit):
            rn.attach(self.notes, rn.posts(self.blog))
        (self.blog / "2026-12-03-bad.md").write_text('---\ntitle: "Bad"\nreleases: 0.5.8\n---\n')
        with self.assertRaises(SystemExit):
            rn.posts(self.blog)


class RealChangelog(unittest.TestCase):
    """The repository's own CHANGELOG.md and posts."""

    @classmethod
    def setUpClass(cls):
        text = rn.CHANGELOG.read_text()
        cls.heads = re.findall(r"^## \[(\d+\.\d+\.\d+)\]", text, re.M)
        cls.notes = rn.parse(text)
        cls.by = {r["version"]: r for r in cls.notes}

    def test_every_section(self):
        self.assertEqual([r["version"] for r in self.notes], self.heads)
        for r in self.notes:
            self.assertRegex(r["date"], r"^\d{4}-\d{2}-\d{2}$")
            for w in r["why"]:
                self.assertFalse(w.endswith(":"), w)
                self.assertNotIn("**", w)
            for c in r["changes"]:
                self.assertTrue(c["title"], c)
                for pr in c["prs"]:
                    self.assertTrue(pr["url"].endswith(f"/pull/{pr['n']}"), pr)

    def test_known_releases(self):
        prs = lambda v: [p["n"] for c in self.by[v]["changes"] for p in c["prs"]]  # noqa: E731
        if "0.5.10" in self.by:
            self.assertIn(692, prs("0.5.10"))
            self.assertTrue(self.by["0.5.10"]["why"][0].startswith("The Open kernel's memory diet (#692)"))
        if "0.5.8" in self.by:
            self.assertEqual(prs("0.5.8"), [668])
        if "0.3.0" in self.by:
            # Cells, under Other, cited by the notes
            self.assertIn(383, prs("0.3.0"))
        if "0.2.6" in self.by:
            self.assertTrue(self.by["0.2.6"]["lede"].startswith("On a machine with 10 physical cores"))
        if "0.2.0" in self.by:
            self.assertEqual(prs("0.2.0"), [101, 103, 104, 105, 106])

    def test_posts(self):
        posts = rn.posts(rn.BLOG)
        rn.attach(self.notes, posts)
        self.assertEqual(rn.sync_posts(posts, self.by, rn.BLOG, write=False), [])


if __name__ == "__main__":
    unittest.main()
