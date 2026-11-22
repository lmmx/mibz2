"""bench/run-queue.sh (`just workstation-queue`) and bench/await-submit.sh, its per-entry
waiter, against a scratch origin and stand-ins for just and gh, with a scratch HOME and XDG
directories: an entry runs once per recipe and commit; its marker and the logs go to the
XDG state directory, never the home directory itself; HOLD is respected; and the Started
and Failed notices go to #145. Nothing is measured and nothing suspends.

    python3 bench/test_workstation_queue.py
"""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = "mibz2"
CHANNEL = 145

# just's stand-in: logs its arguments; exits FAKE_JUST_RC (default 0)
FAKE_JUST = r"""#!/usr/bin/env python3
import json, os, sys
with open(os.environ["FAKE_LOG"], "a") as f:
    f.write(json.dumps({"call": "just", "args": sys.argv[1:], "cwd": os.getcwd()}) + "\n")
sys.exit(int(os.environ.get("FAKE_JUST_RC", "0")))
"""

# gh's stand-in: `api <path> -f body=...` logs the post and prints a URL
FAKE_GH = r"""#!/usr/bin/env python3
import json, os, sys
args = sys.argv[1:]
if args[:2] == ["auth", "status"]:
    sys.exit(0)
assert args[0] == "api", args
body = args[args.index("-f") + 1].split("=", 1)[1]
with open(os.environ["FAKE_LOG"], "a") as f:
    f.write(json.dumps({"call": "gh", "path": args[1], "body": body}) + "\n")
print("https://github.com/example/comment")
"""


def git(cwd, *args, env=None):
    return subprocess.run(
        ["git", *args], cwd=cwd, env=env, check=True, capture_output=True, text=True
    ).stdout.strip()


class Queue(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.home = self.tmp / "home"
        self.home.mkdir()
        self.state = self.tmp / "xdg-state"
        self.fake = self.tmp / "bin"
        self.fake.mkdir()
        for name, src in (("just", FAKE_JUST), ("gh", FAKE_GH)):
            (self.fake / name).write_text(src)
            (self.fake / name).chmod(0o755)
        self.log = self.tmp / "calls.jsonl"
        self.env = {
            **os.environ,
            "HOME": str(self.home),
            "XDG_STATE_HOME": str(self.state),
            # (an empty config directory: no repository list, so one checkout is served)
            "XDG_CONFIG_HOME": str(self.tmp / "xdg-config"),
            "PATH": f"{self.fake}:{os.environ['PATH']}",
            "FAKE_LOG": str(self.log),
            "GIT_AUTHOR_NAME": "Test",
            "GIT_AUTHOR_EMAIL": "test@example.com",
            "GIT_COMMITTER_NAME": "Test",
            "GIT_COMMITTER_EMAIL": "test@example.com",
        }
        self.env.pop("XDG_CACHE_HOME", None)
        self.env.pop("GH", None)
        self.env.pop("AWAIT_SUBMIT_NOTICES", None)
        self.env.pop("EVIDENCE_REPO", None)
        self.env.pop("EVIDENCE_CHANNEL", None)
        self.env.pop("WORKSTATION_LOCK_FD", None)
        # origin, and the maintainer's (here: the coordinator's) clone that sets it up
        # (named as on GitHub: the state directory is named after the origin repository)
        self.origin = self.tmp / "lmmx" / f"{REPO}.git"
        self.origin.parent.mkdir()
        git(self.tmp, "init", "-q", "--bare", "-b", "master", str(self.origin))
        self.dev = self.tmp / "dev"
        git(self.tmp, "clone", "-q", str(self.origin), str(self.dev), env=self.env)
        (self.dev / "bench").mkdir()
        for f in ("run-queue.sh", "await-submit.sh"):
            shutil.copy(HERE / f, self.dev / "bench" / f)
        self.set_queue("bench-a", "bench-b")
        # the workstation's checkout
        self.ws = self.tmp / "workstation"
        git(self.tmp, "clone", "-q", str(self.origin), str(self.ws), env=self.env)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def commit(self, msg):
        git(self.dev, "add", "-A", env=self.env)
        git(self.dev, "commit", "-q", "--allow-empty", "-m", msg, env=self.env)
        git(self.dev, "push", "-q", "origin", "HEAD:master", env=self.env)
        return git(self.dev, "rev-parse", "HEAD", env=self.env)

    def set_queue(self, *recipes):
        lines = ["# the queue", f"# repo: lmmx/{REPO}", f"# channel: {CHANNEL}"]
        lines += ["# recipe  branch  what"]
        lines += [f"{r}  master  measures {r}" for r in recipes]
        (self.dev / "bench" / "workstation-queue.txt").write_text("\n".join(lines) + "\n")
        return self.commit("queue")

    def tag(self, kind, recipe, sha):
        git(self.dev, "tag", "-f", f"{kind}/{recipe}", sha, env=self.env)
        git(self.dev, "push", "-q", "-f", "origin", f"refs/tags/{kind}/{recipe}", env=self.env)

    def run_queue(self, **env):
        r = subprocess.run(
            ["bash", "bench/run-queue.sh", "--no-suspend"],
            cwd=self.ws,
            env={**self.env, **env},
            capture_output=True,
            text=True,
            timeout=120,
        )
        return r

    def calls(self, kind):
        if not self.log.exists():
            return []
        return [c for c in map(json.loads, self.log.read_text().splitlines()) if c["call"] == kind]

    def ran_dir(self):
        return self.state / REPO / "workstation" / "ran"

    def test_runs_once_per_commit_with_state_in_xdg(self):
        sha = git(self.dev, "rev-parse", "HEAD", env=self.env)
        self.tag("ready", "bench-a", sha)
        r = self.run_queue()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual([c["args"] for c in self.calls("just")], [["submit", "bench-a"]])
        self.assertEqual(Path(self.calls("just")[0]["cwd"]).resolve(), self.ws.resolve())
        # the marker and both logs in the XDG state directory; nothing in HOME at all
        self.assertTrue((self.ran_dir() / f"bench-a-{sha}").is_dir())
        logs = sorted(p.name for p in (self.state / REPO / "workstation" / "logs").iterdir())
        # (bench-b, not ready, was looked at once: its waiter logs that too)
        self.assertEqual(
            [n.rsplit("-", 1)[0] for n in logs],
            ["await-submit-bench-a", "await-submit-bench-b", "workstation-queue"],
        )
        self.assertEqual(list(self.home.iterdir()), [])
        # the Started notice, on the channel
        posts = self.calls("gh")
        self.assertEqual(
            [p["path"] for p in posts], [f"repos/lmmx/{REPO}/issues/{CHANNEL}/comments"]
        )
        self.assertIn("**Started:** `just submit bench-a`", posts[0]["body"])
        self.assertIn(sha, posts[0]["body"])
        # again: nothing new runs at the same commit
        r = self.run_queue()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(len(self.calls("just")), 1)
        self.assertIn("bench-a: refused: not run", r.stdout)
        # the tag moved to a new commit: that commit runs, once
        sha2 = self.commit("a rerun's registration")
        self.tag("ready", "bench-a", sha2)
        self.run_queue()
        self.assertEqual(len(self.calls("just")), 2)
        self.assertTrue((self.ran_dir() / f"bench-a-{sha2}").is_dir())
        self.assertEqual(list(self.home.iterdir()), [])

    def test_default_state_dir_is_local_state(self):
        sha = git(self.dev, "rev-parse", "HEAD", env=self.env)
        self.tag("ready", "bench-a", sha)
        env = {k: v for k, v in self.env.items() if k != "XDG_STATE_HOME"}
        r = subprocess.run(
            ["bash", "bench/run-queue.sh", "--no-suspend"],
            cwd=self.ws,
            env=env,
            capture_output=True,
            text=True,
            timeout=120,
        )
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        base = self.home / ".local" / "state" / REPO / "workstation"
        self.assertTrue((base / "ran" / f"bench-a-{sha}").is_dir())
        self.assertEqual([p.name for p in self.home.iterdir()], [".local"])
        # a relative XDG_STATE_HOME is ignored, as the spec says
        sha2 = self.commit("next")
        self.tag("ready", "bench-a", sha2)
        r = subprocess.run(
            ["bash", "bench/run-queue.sh", "--no-suspend"],
            cwd=self.ws,
            env={**env, "XDG_STATE_HOME": "relative"},
            capture_output=True,
            text=True,
            timeout=120,
        )
        self.assertTrue((base / "ran" / f"bench-a-{sha2}").is_dir())
        self.assertFalse((self.ws / "relative").exists())

    def test_failed_run_posts_failed(self):
        sha = git(self.dev, "rev-parse", "HEAD", env=self.env)
        self.tag("ready", "bench-a", sha)
        r = self.run_queue(FAKE_JUST_RC="3")
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        posts = self.calls("gh")
        self.assertEqual(len(posts), 2, posts)
        self.assertTrue(
            all(p["path"] == f"repos/lmmx/{REPO}/issues/{CHANNEL}/comments" for p in posts)
        )
        self.assertIn("**Started:**", posts[0]["body"])
        self.assertIn("**Failed:** `just submit bench-a`", posts[1]["body"])
        self.assertIn("exited 3", posts[1]["body"])
        # the failure names the waiter's log, in the state directory
        self.assertIn(
            str(self.state / REPO / "workstation" / "logs" / "await-submit-bench-a-"),
            posts[1]["body"],
        )
        # a failed run is still a run at that commit: not retried
        self.assertTrue((self.ran_dir() / f"bench-a-{sha}").is_dir())
        self.run_queue()
        self.assertEqual(len(self.calls("just")), 1)

    def test_hold_is_respected(self):
        sha = git(self.dev, "rev-parse", "HEAD", env=self.env)
        self.tag("ready", "bench-a", sha)
        self.tag("hold", "bench-a", sha)
        r = self.run_queue()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(self.calls("just"), [])
        self.assertEqual(self.calls("gh"), [])
        self.assertFalse((self.ran_dir() / f"bench-a-{sha}").exists())
        # the waiter on its own reports HOLD
        r = subprocess.run(
            ["bash", "bench/await-submit.sh", "bench-a", "master", "--no-suspend", "--hours", "0"],
            cwd=self.ws,
            env=self.env,
            capture_output=True,
            text=True,
            timeout=120,
        )
        self.assertEqual(r.returncode, 3, r.stdout + r.stderr)
        self.assertEqual(self.calls("just"), [])


if __name__ == "__main__":
    unittest.main()
