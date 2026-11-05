#!/usr/bin/env python3
"""One run's evidence, archived: a manifest and a single tarball under target/ (report 142).

    python3 bench/archive.py target/EXPERIMENT-STAMP --corpus FILE [--corpus FILE ...]
        [--binary PATH] --command "the command run" [--status N]

Writes DIR/MANIFEST.json and then target/<basename of DIR>.tar.gz, which holds that directory
alone (never the rest of target/, never earlier runs). The manifest records:
- the source commit, its branch, and whether tracked files differed from it;
- every corpus's path, size and SHA-256;
- the binary's SHA-256, when given;
- the command, its exit status and the time;
- every file in the run's directory: path, size and SHA-256.

The run's files are archived as they are: nothing in the directory is rewritten except the
manifest it adds.
"""

import argparse
import datetime
import hashlib
import json
import os
import subprocess
import sys
import tarfile


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def git(*args):
    p = subprocess.run(["git", *args], capture_output=True, text=True)
    return p.stdout.strip() if p.returncode == 0 else None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--corpus", action="append", default=[])
    ap.add_argument("--binary")
    ap.add_argument("--command", required=True)
    ap.add_argument("--status", type=int)
    a = ap.parse_args()
    d = os.path.normpath(a.dir)
    if not os.path.isdir(d):
        sys.exit(f"archive: {d} is not a directory")
    name = os.path.basename(d)
    files = []
    for root, _, names in os.walk(d):
        for n in sorted(names):
            p = os.path.join(root, n)
            rel = os.path.relpath(p, d)
            if rel == "MANIFEST.json":
                continue
            files.append({"path": rel, "bytes": os.path.getsize(p), "sha256": sha256(p)})
    dirty = git("status", "--porcelain", "--untracked-files=no")
    manifest = {
        "run": name,
        "archived": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "source": {"commit": git("rev-parse", "HEAD"),
                   "branch": git("rev-parse", "--abbrev-ref", "HEAD"),
                   "tracked_changes": dirty.splitlines() if dirty else []},
        "corpora": [{"path": c, "bytes": os.path.getsize(c), "sha256": sha256(c)}
                    for c in a.corpus],
        "binary": ({"path": a.binary, "sha256": sha256(a.binary)}
                   if a.binary and os.path.exists(a.binary) else None),
        "command": a.command,
        "exit_status": a.status,
        "files": sorted(files, key=lambda f: f["path"]),
    }
    with open(os.path.join(d, "MANIFEST.json"), "w") as f:
        json.dump(manifest, f, indent=1)
    tar = os.path.join(os.path.dirname(d) or ".", name + ".tar.gz")
    with tarfile.open(tar, "w:gz") as t:
        t.add(d, arcname=name)
    print(f"archive: {tar} ({os.path.getsize(tar)} bytes, sha256 {sha256(tar)}): "
          f"{len(files)} files and MANIFEST.json, commit {manifest['source']['commit']}"
          + (", WITH tracked changes" if manifest["source"]["tracked_changes"] else ""))


if __name__ == "__main__":
    main()
