"""bench/submit.sh against a scratch repository and a stand-in for gh: it archives a bundle
once, posts once, refuses a different bundle under the same run, and keeps the bundle when a
push fails, so the same command retries it.

    python3 bench/test_submit.py
"""

import json
import os
import subprocess
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path

PUBLISH = Path(__file__).resolve().parent / "submit.sh"

# gh's stand-in: `auth status` passes; `api` lists or appends to comments.json
FAKE_GH = r"""#!/usr/bin/env python3
import json, os, re, sys
store = os.environ["FAKE_COMMENTS"]
args = sys.argv[1:]
if args[:2] == ["auth", "status"]:
    sys.exit(0)
assert args[0] == "api", args
comments = json.load(open(store)) if os.path.exists(store) else []
if "--paginate" in args:
    jq = args[args.index("--jq") + 1]
    marker = re.search(r'contains\("(.*)"\)', jq).group(1)
    for c in comments:
        if marker in c["body"]:
            print(c["html_url"])
    sys.exit(0)
body = open(args[args.index("-F") + 1].split("=@", 1)[1]).read()
url = f"https://github.com/lmmx/mibz2/pull/145#issuecomment-{len(comments) + 1}"
comments.append({"body": body, "html_url": url})
json.dump(comments, open(store, "w"))
print(url)
"""


def git(cwd, *args):
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True, capture_output=True, text=True
    ).stdout.strip()


class Submit(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.remote = self.tmp / "lmmx" / "mibz2.git"
        self.remote.parent.mkdir()
        git(self.tmp, "init", "-q", "--bare", str(self.remote))
        self.work = self.tmp / "work"
        self.work.mkdir()
        git(self.work, "init", "-q")
        git(self.work, "config", "user.name", "Test")
        git(self.work, "config", "user.email", "test@example.com")
        git(self.work, "remote", "add", "origin", str(self.remote))
        gh = self.tmp / "gh"
        gh.write_text(FAKE_GH)
        gh.chmod(0o755)
        self.env = {**os.environ, "GH": str(gh), "FAKE_COMMENTS": str(self.tmp / "comments.json")}

    def bundle(self, outcome, stamp="20261106T120000Z"):
        run = self.work / "target" / "three-team-policy" / stamp
        run.mkdir(parents=True, exist_ok=True)
        (run / "outcome.txt").write_text(outcome + "\n")
        (run / "commits.txt").write_text("head " + "a" * 40 + "\nbase " + "b" * 40 + "\n")
        (run / "inputs.txt").write_text("deadbeef  bin/ab-head\n")
        out = run.parent / f"{stamp}.tar.gz"
        with tarfile.open(out, "w:gz") as t:
            t.add(run, arcname=stamp)
        return out

    def submit(self, tarball):
        return subprocess.run(
            ["bash", str(PUBLISH), str(tarball)],
            cwd=self.work,
            env=self.env,
            capture_output=True,
            text=True,
        )

    def comments(self):
        return json.loads(Path(self.env["FAKE_COMMENTS"]).read_text())

    def test_archives_and_posts_once(self):
        tarball = self.bundle("VALID PASS")
        first = self.submit(tarball)
        self.assertEqual(first.returncode, 0, first.stderr)
        tree = git(self.remote, "ls-tree", "-r", "--name-only", "evidence")
        self.assertEqual(
            tree.split(),
            [
                "three-team-policy/20261106T120000Z.tar.gz",
                "three-team-policy/20261106T120000Z.tar.gz.sha256",
            ],
        )
        stored = subprocess.run(
            ["git", "show", "evidence:three-team-policy/20261106T120000Z.tar.gz"],
            cwd=self.remote,
            capture_output=True,
            check=True,
        ).stdout
        self.assertEqual(stored, tarball.read_bytes())
        [c] = self.comments()
        for line in (
            "FROM: louis via workstation",
            "TO: coordinator",
            "TYPE: LOG",
            "**Outcome:** VALID PASS",
            "head " + "a" * 40,
        ):
            self.assertIn(line, c["body"])
        # again: nothing new committed or posted
        second = self.submit(tarball)
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertIn("already archived", second.stdout)
        self.assertIn("already posted", second.stdout)
        self.assertEqual(len(git(self.remote, "rev-list", "evidence").split()), 1)
        self.assertEqual(len(self.comments()), 1)

    def test_a_second_run_is_added_beside_the_first(self):
        self.assertEqual(self.submit(self.bundle("VALID PASS")).returncode, 0)
        self.assertEqual(self.submit(self.bundle("INVALID: x", "20261106T130000Z")).returncode, 0)
        self.assertEqual(
            len(git(self.remote, "ls-tree", "-r", "--name-only", "evidence").split()), 4
        )
        self.assertEqual(len(git(self.remote, "rev-list", "evidence").split()), 2)
        self.assertEqual(len(self.comments()), 2)

    def test_a_different_bundle_under_the_same_run_is_refused(self):
        self.assertEqual(self.submit(self.bundle("VALID PASS")).returncode, 0)
        other = self.submit(self.bundle("VALID FAIL"))
        self.assertNotEqual(other.returncode, 0)
        self.assertIn("different contents", other.stderr)
        self.assertEqual(len(self.comments()), 1)

    def test_a_failed_push_keeps_the_bundle_and_a_retry_submits_it(self):
        tarball = self.bundle("VALID PASS")
        moved = self.remote.with_suffix(".away")
        self.remote.rename(moved)
        failed = self.submit(tarball)
        self.assertNotEqual(failed.returncode, 0)
        self.assertTrue(tarball.exists())
        self.assertFalse(Path(self.env["FAKE_COMMENTS"]).exists())
        moved.rename(self.remote)
        self.assertEqual(self.submit(tarball).returncode, 0)
        self.assertEqual(len(self.comments()), 1)

    def test_a_remote_for_another_repository_is_refused(self):
        git(self.work, "remote", "set-url", "origin", str(self.tmp / "other.git"))
        r = self.submit(self.bundle("VALID PASS"))
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("not lmmx/mibz2", r.stderr)


if __name__ == "__main__":
    sys.exit(unittest.main())
