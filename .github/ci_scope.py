"""Skip heavy decoder CI only for a completely classified prose/frontier-data diff.

Use the complete Git diff, not the GitHub paths-filter 300-file list.
Unknown changes and unavailable comparisons always select the full suite.
"""
import json
import os
import re
import subprocess
import sys


def lightweight(path):
    if path in {
        "AGENTS.md",
        "CLAUDE.md",
        "GEMINI.md",
        "docs/STEWARDSHIP.md",
        "docs/RELEASING.md",
        "frontier/config.toml",
        "frontier/limits.toml",
        "frontier/surface.tsv",
        "frontier/dead-ends.tsv",
    }:
        return True
    if path.endswith(".md") and path.startswith(
        ("docs/journal/", "docs/team/", "docs/site/", "frontier/")
    ):
        return True
    return (
        path.startswith(("frontier/claims/", "frontier/blog/"))
        and path.endswith(".toml")
    )


def full_suite(paths):
    """All changed paths must be recognised as non-executable data."""
    return not paths or any(not lightweight(path) for path in paths)


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, check=True).stdout


def classify():
    event = os.environ.get("GITHUB_EVENT_NAME")
    if event not in ("pull_request", "push"):
        return True, "manual or other event"
    if os.environ.get("GITHUB_REF", "").startswith("refs/tags/"):
        return True, "release tag"
    with open(os.environ["GITHUB_EVENT_PATH"], encoding="utf-8") as file:
        payload = json.load(file)
    base = payload["pull_request"]["base"]["sha"] if event == "pull_request" else payload["before"]
    if not re.fullmatch(r"[0-9a-f]{40}", base) or base == "0" * 40:
        return True, "no usable base"
    try:
        git("cat-file", "-e", f"{base}^{{commit}}")
    except subprocess.CalledProcessError:
        git("fetch", "--no-tags", "--depth=1", "origin", base)
    # Compare the actual checked-out tree, including the PR merge result.
    # --no-renames counts both ends of a rename. NUL handles unusual filenames.
    raw = git("diff", "--name-only", "--no-renames", "--no-ext-diff", "-z", base, "HEAD")
    paths = [os.fsdecode(path) for path in raw.split(b"\0") if path]
    return full_suite(paths), f"{len(paths)} changed paths"


def self_test():
    assert not full_suite(["frontier/config.toml", "frontier/limits.toml", "frontier/flags/example.md"])
    assert not full_suite(["docs/journal/entry.md", "docs/site/blog/post.md"])
    assert not full_suite(["frontier/claims/new.toml", "frontier/surface.tsv"])
    for path in (
        "src/lib.rs", "tests/fixtures/induction-prose.txt", "Cargo.toml",
        "Cargo.lock", "pyproject.toml", "python/mibz2/__init__.py",
        "examples/probe.rs", "bench/recipe.sh", ".github/workflows/test.yml",
        ".github/ci_scope.py", "frontier/tools/frontier.py",
        "docs/scripts/build.py", "docs/journal/model.py", "unknown.file",
        "README.md", "CHANGELOG.md", "rust-toolchain.toml",
    ):
        assert full_suite(["docs/journal/entry.md", path]), path
    assert full_suite([f"docs/journal/{n}.md" for n in range(501)] + ["src/open.rs"])
    assert full_suite([])
    print("CI scope tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        self_test()
    else:
        try:
            full, reason = classify()
        except (OSError, KeyError, ValueError, TypeError, subprocess.CalledProcessError) as exc:
            full, reason = True, f"comparison unavailable ({type(exc).__name__})"
        print(f"full decoder suite: {full}; {reason}")
        with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as file:
            file.write(f"full={str(full).lower()}\n")
