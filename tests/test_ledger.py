"""Regression tests for ``frontier/tools/ledger.py`` and ``frontier/ledger.tsv``."""

from __future__ import annotations

import pathlib
import subprocess
import sys
import tempfile
import tomllib
import unittest

REPO = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO / "frontier" / "tools"))

import ledger  # noqa: E402


def _run_git(repo: pathlib.Path, *args: str, date: str = "2026-11-27T12:00:00+00:00") -> str:
    """Execute a git command in ``repo`` with deterministic test identity and dates."""
    env = {
        "GIT_AUTHOR_NAME": "Test User",
        "GIT_AUTHOR_EMAIL": "test@example.com",
        "GIT_COMMITTER_NAME": "Test User",
        "GIT_COMMITTER_EMAIL": "test@example.com",
        "GIT_AUTHOR_DATE": date,
        "GIT_COMMITTER_DATE": date,
        "PATH": "/usr/local/bin:/usr/bin:/bin",
    }
    proc = subprocess.run(
        ["git", "-C", str(repo), *args],
        capture_output=True,
        text=True,
        env=env,
        check=True,
    )
    return proc.stdout.strip()


def _init_fixture_repo(root: pathlib.Path) -> pathlib.Path:
    """Create a synthetic git repository with tag ``v0.5.14`` and mixed commits."""
    _run_git(root, "init", "-b", "master")
    (root / "src").mkdir()
    (root / "src" / "lib.rs").write_text("pub fn v() -> u32 { 0 }\n", encoding="utf-8")
    (root / "Cargo.toml").write_text('[package]\nname = "demo"\n', encoding="utf-8")
    (root / "docs").mkdir()
    (root / "docs" / "note.md").write_text("base\n", encoding="utf-8")
    (root / "bench" / "sub").mkdir(parents=True)
    (root / "bench" / "sub" / "Cargo.lock").write_text("v1\n", encoding="utf-8")
    _run_git(root, "add", ".")
    _run_git(root, "commit", "-m", "chore: release v0.5.14", date="2026-11-25T10:00:00+00:00")
    _run_git(root, "tag", "v0.5.14")

    # Docs-only PR (must be ignored by ledger).
    (root / "docs" / "note.md").write_text("update\n", encoding="utf-8")
    _run_git(root, "add", "docs/note.md")
    _run_git(root, "commit", "-m", "docs: update note (#1001)", date="2026-11-26T10:00:00+00:00")

    # Sub-bench Cargo.lock PR (must be ignored; not root software).
    (root / "bench" / "sub" / "Cargo.lock").write_text("v2\n", encoding="utf-8")
    _run_git(root, "add", "bench/sub/Cargo.lock")
    _run_git(
        root,
        "commit",
        "-m",
        "bench: bump lock (#1002)",
        date="2026-11-26T11:00:00+00:00",
    )

    # Software PR touching src/.
    (root / "src" / "lib.rs").write_text("pub fn v() -> u32 { 1 }\n", encoding="utf-8")
    _run_git(root, "add", "src/lib.rs")
    _run_git(root, "commit", "-m", "perf: faster decode (#1003)", date="2026-11-26T12:00:00+00:00")

    # Software PR touching python/.
    (root / "python").mkdir()
    (root / "python" / "mod.py").write_text("x = 1\n", encoding="utf-8")
    _run_git(root, "add", "python/mod.py")
    _run_git(
        root,
        "commit",
        "-m",
        "perf(python): avoid copy (#1004)",
        date="2026-11-27T09:00:00+00:00",
    )
    return root


class LedgerValidationTests(unittest.TestCase):
    """Unit tests for ``validate_ledger`` and schema enforcement."""

    def _write(self, tmp: str, content: str) -> pathlib.Path:
        """Write TSV content into a temporary directory and return its path."""
        path = pathlib.Path(tmp) / "ledger.tsv"
        path.write_text(content, encoding="utf-8")
        return path

    def test_valid_rows_pass_every_allowed_state(self) -> None:
        """All five allowed states and valid measurement strings pass validation."""
        with tempfile.TemporaryDirectory() as tmp:
            path = self._write(
                tmp,
                "\n".join(
                    [
                        "\t".join(ledger.COLUMNS),
                        "100\t2026-11-27\tBounds peak memory on zeros.\tdefault\t-50% RSS on zeros, local\tPR #100",
                        "101\t2026-11-27\tAdds feature flag.\texperimental\tnot measured\tPR #101",
                        "102\t2026-11-27\tAdapts seam window.\tawaiting-review\t-95% copies on wikidata, VM\tPR #102",
                        "103\t2026-11-27\tTests a new wire loop.\tprototype\t-20% wiring CPU on wikidata, VM\tPR #103",
                        "104\t2026-11-27\tSets throughput ceiling.\ttarget\tnot measured\tdocs/model.md",
                    ]
                )
                + "\n",
            )
            self.assertEqual(ledger.validate_ledger(path), [])

    def test_rejects_backticks_pr_refs_bad_state_and_missing_machine(self) -> None:
        """Validation rejects inline code, #N refs, unknown states, and unlabelled measurements."""
        with tempfile.TemporaryDirectory() as tmp:
            path = self._write(
                tmp,
                "\n".join(
                    [
                        "\t".join(ledger.COLUMNS),
                        "100\t2026-11-27\tUses `BufRead` like #99.\tshipped\t-10% faster on wikidata\tPR #100",
                    ]
                )
                + "\n",
            )
            errs = ledger.validate_ledger(path)
            joined = "\n".join(errs)
            self.assertIn("backticks", joined)
            self.assertIn("#N", joined)
            self.assertIn("invalid state", joined)
            self.assertIn("name a machine", joined)

    def test_rejects_duplicate_pr_and_bad_date(self) -> None:
        """Duplicate PR numbers and malformed ISO dates fail validation."""
        with tempfile.TemporaryDirectory() as tmp:
            path = self._write(
                tmp,
                "\n".join(
                    [
                        "\t".join(ledger.COLUMNS),
                        "100\t2026-11-27\tFirst row.\tdefault\tnot measured\tPR #100",
                        "100\t27-11-2026\tSecond row.\tdefault\tnot measured\tPR #100",
                    ]
                )
                + "\n",
            )
            errs = ledger.validate_ledger(path)
            joined = "\n".join(errs)
            self.assertIn("duplicate PR 100", joined)
            self.assertIn("YYYY-MM-DD", joined)


class LedgerGitDiscoveryTests(unittest.TestCase):
    """Unit tests for git discovery, ``--missing``, and list-since-newest modes."""

    def test_missing_and_since_newest_filter_to_software_prs(self) -> None:
        """Only root software PRs are required; docs and sub-bench commits are ignored."""
        with tempfile.TemporaryDirectory() as tmp:
            repo = _init_fixture_repo(pathlib.Path(tmp))
            ledger_path = repo / "ledger.tsv"
            ledger_path.write_text(
                "\n".join(
                    [
                        "\t".join(ledger.COLUMNS),
                        "1003\t2026-11-26\tSpeeds up decoding.\tdefault\t-5% wall on text, CI\tPR #1003",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            # PR 1004 is missing -> --missing must exit non-zero.
            rc = ledger.main(
                ["--missing", "--repo", str(repo), "--ref", "HEAD", "--ledger", str(ledger_path)]
            )
            self.assertEqual(rc, 1)

            pending = ledger.since_newest_prs(repo, ledger_path, "v0.5.14", "HEAD")
            self.assertEqual([p.pr for p in pending], [1004])

            # Add PR 1004 -> both --missing and --check must exit 0.
            with ledger_path.open("a", encoding="utf-8") as fh:
                fh.write(
                    "1004\t2026-11-27\tRemoves extra Python copy.\tdefault\t-7% on wikidata, VM\tPR #1004\n"
                )
            rc = ledger.main(
                [
                    "--check",
                    "--missing",
                    "--repo",
                    str(repo),
                    "--ref",
                    "HEAD",
                    "--ledger",
                    str(ledger_path),
                ]
            )
            self.assertEqual(rc, 0)
            self.assertEqual(ledger.since_newest_prs(repo, ledger_path, "v0.5.14", "HEAD"), [])


class RealRepoLedgerTests(unittest.TestCase):
    """Verify the repository's own ``frontier/ledger.tsv`` against the frozen stop test."""

    def test_repo_ledger_passes_check_and_missing_and_matches_master_features(self) -> None:
        """Check schema, completeness since v0.5.14, and default vs feature-gated states."""
        ledger_path = REPO / "frontier" / "ledger.tsv"
        self.assertEqual(ledger.validate_ledger(ledger_path), [])
        rows, errs = ledger.load_ledger(ledger_path)
        self.assertEqual(errs, [])
        by_pr = {row.pr: row for row in rows}

        # Verify required PRs from shipped-ledger stop_test exist with states matching master.
        expected_states = {
            1129: "default",
            1080: "default",
            1040: "default",
            1033: "default",
            1034: "default",
            1053: "experimental",
            1110: "experimental",
        }
        for pr_num, expected in expected_states.items():
            self.assertIn(pr_num, by_pr, f"missing required PR #{pr_num}")
            self.assertEqual(by_pr[pr_num].state, expected, f"state mismatch for PR #{pr_num}")

        # Cross-check Cargo.toml: experimental research features remain off by default.
        cargo = tomllib.loads((REPO / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(cargo["features"]["default"], [])
        for feat in ("mtf-fold", "a6b-endpoint-redesign", "forward-anchor-lanes", "scan-oracle"):
            self.assertIn(feat, cargo["features"])

        # Completeness check against git history (when v0.5.14 is reachable).
        ref = ledger.resolve_ref(REPO)
        missing = ledger.missing_prs(REPO, ledger_path, "v0.5.14", ref)
        self.assertEqual(missing, [], f"missing merged software PRs: {missing}")


if __name__ == "__main__":
    unittest.main()
