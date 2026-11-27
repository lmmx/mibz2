#!/usr/bin/env python3
"""Shallow-safe sources for bench-scanner-adaptive-seam (thirteenth-pass invariant).

The workstation runner works in a `--depth 1` checkout, so nothing here reads the local
repository's history or clones from it. A private repository is initialised under DEST and
fetches the exact commits from the origin URL; the merge base is computed there, from
fetched history only, and must equal the registered BASE.

    scanner_adaptive_seam_source.py ORIGIN_URL HEAD_SHA BASE_SHA DEST

Writes DEST/repo (the private repository), DEST/build-base and DEST/build-head (detached
worktrees at BASE and HEAD) and prints one JSON object: base, head, master, the changed
source paths, and the build inputs that differ (must be none). Exits 1 on any mismatch.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

DEPTH = 512
BUILD_INPUTS = ("Cargo.toml", "Cargo.lock", "build.rs", "rust-toolchain.toml", "examples/ab.rs")
SOURCE = ("src", "Cargo.toml", "Cargo.lock", "build.rs", "rust-toolchain.toml", "examples", "tests")


def git(*args, cwd):
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True, capture_output=True, text=True
    ).stdout.strip()


def fail(msg):
    raise SystemExit(f"scanner_adaptive_seam_source: {msg}")


def materialize(origin, head, base, dest):
    for name, sha in (("HEAD_SHA", head), ("BASE_SHA", base)):
        if not re.fullmatch("[0-9a-f]{40}", sha):
            fail(f"{name} {sha!r} is not a 40-character SHA")
    if head == base:
        fail("base and head are the same commit")
    dest = Path(dest)
    dest.mkdir(parents=True)
    repo = dest / "repo"
    subprocess.run(["git", "init", "-q", str(repo)], check=True)
    fetched = {}
    for name, ref in (("head", head), ("base", base), ("master", "refs/heads/master")):
        git("fetch", "-q", "--no-tags", f"--depth={DEPTH}", origin, ref, cwd=repo)
        fetched[name] = git("rev-parse", "FETCH_HEAD^{commit}", cwd=repo)
    if fetched["head"] != head:
        fail(f"fetched head {fetched['head']} is not {head}")
    if fetched["base"] != base:
        fail(f"fetched base {fetched['base']} is not {base}")
    try:
        merge_base = git("merge-base", head, fetched["master"], cwd=repo)
    except subprocess.CalledProcessError:
        fail(f"no merge base of {head} and master within {DEPTH} commits")
    if merge_base != base:
        fail(f"merge base of head and master is {merge_base}, not the registered {base}")
    for name, sha in (("base", base), ("head", head)):
        git("worktree", "add", "-q", "--detach", str(dest / f"build-{name}"), sha, cwd=repo)
        if git("rev-parse", "HEAD", cwd=dest / f"build-{name}") != sha:
            fail(f"build-{name} is not at {sha}")
    changed = git("diff", "--name-only", base, head, "--", *SOURCE, cwd=repo).split()
    if not changed:
        fail("the source does not differ")
    beyond = [p for p in changed if p in BUILD_INPUTS]
    if beyond:
        fail(f"build/harness inputs differ: {beyond}")
    return {
        "origin": origin,
        "base": base,
        "head": head,
        "master": fetched["master"],
        "changed": changed,
        "build_inputs_differ": beyond,
    }


if __name__ == "__main__":
    if len(sys.argv) != 5:
        fail("usage: ORIGIN_URL HEAD_SHA BASE_SHA DEST")
    print(json.dumps(materialize(*sys.argv[1:]), indent=2))
