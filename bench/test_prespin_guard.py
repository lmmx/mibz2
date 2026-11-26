#!/usr/bin/env python3
"""The PRESPIN guard of bench-kernel-choice's and bench-scanner-plumbing's resolve phase, as the
recipe writes it, run under the recipe's `set -euo pipefail` against real commits: it must pass
on a head whose examples/ab.rs has --prespin-ms and fail on one whose ab.rs lacks it.

The first form, `git show "$head:examples/ab.rs" | grep -q ...`, failed on a head that has the
option (§84's test 1, kernel-choice/run-20261125T055310Z). grep -q exits at the first match, git
show takes SIGPIPE writing the rest of the ~250 KB file, and pipefail fails the pipeline. This
test runs the guard on the real ab.rs with the option near its top, where that form fails.

    python3 bench/test_prespin_guard.py
"""

import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECIPES = ("bench-kernel-choice", "bench-scanner-plumbing")


def guard(recipe: str) -> str:
    """The guard's command, from the recipe in .just/bench.just: the lines from the PRESPIN test
    to its `exit 1; }`"""
    text = (ROOT / ".just" / "bench.just").read_text()
    body = text[text.index(f"\n{recipe} HEAD_REF=") :]
    body = body[: body.index("\n\n")] if "\n\n" in body else body
    m = re.search(r'\n( *if \[ -n "\$PRESPIN" \]; then\n.*?\n *fi)\n', body, re.S)
    assert m, f"{recipe}: no PRESPIN guard"
    return m.group(1)


def git(cwd, *args):
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True, capture_output=True, text=True
    ).stdout.strip()


def run(cmd: str, cwd: Path, head: str) -> int:
    script = f"set -euo pipefail\nPRESPIN=100\nhead={head}\n{cmd}\n"
    return subprocess.run(
        ["bash", "-c", script], cwd=cwd, capture_output=True, text=True
    ).returncode


def main() -> int:
    ab = (ROOT / "examples" / "ab.rs").read_text()
    ab = ab.replace('"--prespin-ms"', '"--no-such-option"')  # (this checkout's own, if any)
    ok = True
    with tempfile.TemporaryDirectory() as d:
        repo = Path(d)
        git(repo, "init", "-q")
        git(repo, "config", "user.name", "t")
        git(repo, "config", "user.email", "t@example.com")
        (repo / "examples").mkdir()
        # (without the option; then with it on the file's third line, so a pipe's reader quits
        # with most of the file unread)
        (repo / "examples" / "ab.rs").write_text(ab)
        git(repo, "add", "-A")
        git(repo, "commit", "-q", "-m", "without")
        without = git(repo, "rev-parse", "HEAD")
        lines = ab.splitlines(keepends=True)
        (repo / "examples" / "ab.rs").write_text(
            "".join(lines[:2] + ['//! "--prespin-ms"\n'] + lines[2:])
        )
        git(repo, "commit", "-q", "-am", "with")
        with_ = git(repo, "rev-parse", "HEAD")
        size = len((repo / "examples" / "ab.rs").read_bytes())
        print(f"examples/ab.rs: {size} bytes")
        text = (ROOT / ".just" / "bench.just").read_text()
        for recipe in RECIPES:
            if f"\n{recipe} HEAD_REF=" not in text:
                continue
            cmd = guard(recipe)
            a, b = run(cmd, repo, with_), run(cmd, repo, without)
            good = a == 0 and b == 1
            ok &= good
            print(
                f"{'ok  ' if good else 'FAIL'} {recipe}: the guard passes a head with --prespin-ms (exit {a}) and refuses one without (exit {b})"
            )
        # (the first form, for the record: under pipefail it fails on the head that has it)
        old = 'git show "$head:examples/ab.rs" | grep -q -- \'"--prespin-ms"\''
        print(
            f"the first form on the head with the option exits {run(old, repo, with_)} (141 or 1: SIGPIPE under pipefail)"
        )
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
