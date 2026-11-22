"""bench/run-queue.sh serving several repositories from one process: two scratch repositories,
each with a bare origin (a local file remote), its own queue file, ready tags, state directory
and channel, and stand-ins for just and gh. Covered: the repositories take turns, one run each
(round-robin); a ready commit runs once per repository; the machine-wide lock refuses a second
runner (and a lone waiter) and is held while a recipe runs; each repository's entries go through
its own waiter, and an args= column through --args; one repository's trouble (a failed
recipe, a dirty checkout, an unreadable origin) never stops the other; and with no repository
list the current checkout alone is served, as before. Nothing is measured and nothing suspends.

    python3 bench/test_run_queue_multi.py
"""

import fcntl
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPTS = ("run-queue.sh", "await-submit.sh", "submit.sh", "bundle.sh")

# just's stand-in: logs its arguments, its checkout and whether the machine lock was free to a
# newcomer (it must not be: the runner holds it); exits 3 in a checkout under FAKE_JUST_FAIL
FAKE_JUST = r"""#!/usr/bin/env python3
import fcntl, json, os, sys
lock = os.path.join(os.environ["XDG_STATE_HOME"], "workstation", "queue.lock")
with open(lock, "a") as f:
    try:
        fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
        free = True
    except OSError:
        free = False
with open(os.environ["FAKE_LOG"], "a") as f:
    f.write(json.dumps({"call": "just", "args": sys.argv[1:], "cwd": os.getcwd(),
                        "lock_free": free}) + "\n")
fail = os.environ.get("FAKE_JUST_FAIL")
sys.exit(3 if fail and os.getcwd().startswith(fail) else 0)
"""

# gh's stand-in: `api <path> -f body=...` logs the post and prints a URL
FAKE_GH = r"""#!/usr/bin/env python3
import json, os, sys
args = sys.argv[1:]
if args[:2] == ["auth", "status"]:
    sys.exit(0)
body = args[args.index("-f") + 1].split("=", 1)[1]
with open(os.environ["FAKE_LOG"], "a") as f:
    f.write(json.dumps({"call": "gh", "path": args[1], "body": body}) + "\n")
print("https://github.com/example/comment")
"""


def git(cwd, *args, env=None):
    return subprocess.run(
        ["git", *args], cwd=cwd, env=env, check=True, capture_output=True, text=True
    ).stdout.strip()


class Repo:
    """A scratch repository: its bare origin, the coordinator's clone, the workstation's."""

    def __init__(self, tmp, name, channel, env):
        self.name, self.channel, self.env = name, channel, env
        self.origin = tmp / "lmmx" / f"{name}.git"
        self.origin.parent.mkdir(exist_ok=True)
        git(tmp, "init", "-q", "--bare", "-b", "master", str(self.origin))
        self.dev = tmp / f"{name}-dev"
        git(tmp, "clone", "-q", str(self.origin), str(self.dev), env=env)
        (self.dev / "bench").mkdir()
        for f in SCRIPTS:
            shutil.copy(HERE / f, self.dev / "bench" / f)
        self.head = self.set_queue()
        self.ws = tmp / "workstation" / name
        git(tmp, "clone", "-q", str(self.origin), str(self.ws), env=env)

    def commit(self, msg):
        git(self.dev, "add", "-A", env=self.env)
        git(self.dev, "commit", "-q", "--allow-empty", "-m", msg, env=self.env)
        git(self.dev, "push", "-q", "origin", "HEAD:master", env=self.env)
        return git(self.dev, "rev-parse", "HEAD", env=self.env)

    def set_queue(self, *recipes):
        lines = ["# the queue", f"# repo: lmmx/{self.name}", f"# channel: {self.channel}"]
        lines += [f"{r}  master  measures {r}" for r in recipes]
        (self.dev / "bench" / "workstation-queue.txt").write_text("\n".join(lines) + "\n")
        self.head = self.commit("queue")
        return self.head

    def ready(self, *recipes):
        for r in recipes:
            git(self.dev, "tag", "-f", f"ready/{r}", self.head, env=self.env)
            git(self.dev, "push", "-q", "-f", "origin", f"refs/tags/ready/{r}", env=self.env)


class MultiRepo(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.home = self.tmp / "home"
        self.home.mkdir()
        self.state = self.tmp / "xdg-state"
        self.config = self.tmp / "xdg-config"
        fake = self.tmp / "bin"
        fake.mkdir()
        for name, src in (("just", FAKE_JUST), ("gh", FAKE_GH)):
            (fake / name).write_text(src)
            (fake / name).chmod(0o755)
        self.log = self.tmp / "calls.jsonl"
        self.env = {
            **os.environ,
            "HOME": str(self.home),
            "XDG_STATE_HOME": str(self.state),
            "XDG_CONFIG_HOME": str(self.config),
            "PATH": f"{fake}:{os.environ['PATH']}",
            "FAKE_LOG": str(self.log),
            "GIT_AUTHOR_NAME": "Test",
            "GIT_AUTHOR_EMAIL": "test@example.com",
            "GIT_COMMITTER_NAME": "Test",
            "GIT_COMMITTER_EMAIL": "test@example.com",
        }
        for k in (
            "GH",
            "AWAIT_SUBMIT_NOTICES",
            "EVIDENCE_REPO",
            "EVIDENCE_CHANNEL",
            "WORKSTATION_LOCK_FD",
            "FAKE_JUST_FAIL",
            "XDG_CACHE_HOME",
        ):
            self.env.pop(k, None)
        # (the lock's directory exists for the stand-in just even before a runner makes it)
        (self.state / "workstation").mkdir(parents=True)
        self.a = Repo(self.tmp, "alpha", 11, self.env)
        self.b = Repo(self.tmp, "beta", 22, self.env)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def write_config(self, *paths):
        (self.config / "workstation").mkdir(parents=True, exist_ok=True)
        lines = ["# the workstation's repositories", *map(str, paths), ""]
        (self.config / "workstation" / "repos").write_text("\n".join(lines))

    def run_queue(self, *args, cwd=None, **env):
        return subprocess.run(
            ["bash", str(HERE / "run-queue.sh"), "--no-suspend", *args],
            cwd=cwd or self.tmp,
            env={**self.env, **env},
            capture_output=True,
            text=True,
            timeout=180,
        )

    def calls(self, kind):
        if not self.log.exists():
            return []
        return [c for c in map(json.loads, self.log.read_text().splitlines()) if c["call"] == kind]

    def runs(self):  # (repository, recipe) of each `just submit`, in order
        return [(Path(c["cwd"]).name, c["args"][1]) for c in self.calls("just")]

    def ran(self, repo, recipe):
        return (self.state / repo.name / "workstation" / "ran" / f"{recipe}-{repo.head}").is_dir()

    def test_repositories_take_turns_one_run_each(self):
        self.a.set_queue("a1", "a2", "a3")
        self.a.ready("a1", "a2", "a3")
        self.b.set_queue("b1", "b2")
        self.b.ready("b1", "b2")
        self.write_config(self.a.ws, self.b.ws)
        r = self.run_queue()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(
            self.runs(),
            [("alpha", "a1"), ("beta", "b1"), ("alpha", "a2"), ("beta", "b2"), ("alpha", "a3")],
        )
        # each ran in its own checkout, with the lock held for the run
        self.assertTrue(all(not c["lock_free"] for c in self.calls("just")))
        # each repository's notices went to its own channel
        paths = {p["path"] for p in self.calls("gh")}
        self.assertEqual(
            paths, {"repos/lmmx/alpha/issues/11/comments", "repos/lmmx/beta/issues/22/comments"}
        )
        # the run's log is machine-wide; each entry's log is its repository's
        self.assertEqual(
            [p.name.rsplit("-", 1)[0] for p in (self.state / "workstation" / "logs").iterdir()],
            ["workstation-queue"],
        )
        self.assertEqual(len(list((self.state / "beta" / "workstation" / "logs").iterdir())), 2)
        self.assertEqual(list(self.home.iterdir()), [])

    def test_a_skipped_entry_does_not_spend_a_turn(self):
        self.a.set_queue("a-unready", "a-held", "a1", "a2")
        self.a.ready("a-held", "a1", "a2")
        git(self.a.dev, "tag", "hold/a-held", self.a.head, env=self.env)
        git(self.a.dev, "push", "-q", "origin", "refs/tags/hold/a-held", env=self.env)
        self.b.set_queue("b1", "b2")
        self.b.ready("b1", "b2")
        r = self.run_queue("--repo", str(self.a.ws), "--repo", str(self.b.ws))
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(
            self.runs(), [("alpha", "a1"), ("beta", "b1"), ("alpha", "a2"), ("beta", "b2")]
        )

    def test_once_per_ready_commit_per_repository(self):
        # the same recipe name in both: each repository's markers are its own
        self.a.set_queue("bench-x")
        self.a.ready("bench-x")
        self.b.set_queue("bench-x")
        self.b.ready("bench-x")
        self.write_config(self.a.ws, self.b.ws)
        r = self.run_queue()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(self.runs(), [("alpha", "bench-x"), ("beta", "bench-x")])
        self.assertTrue(self.ran(self.a, "bench-x"))
        self.assertTrue(self.ran(self.b, "bench-x"))
        # again: nothing runs at the same commits
        r = self.run_queue()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(len(self.calls("just")), 2)
        self.assertIn("[alpha] bench-x: refused: not run", r.stdout)
        self.assertIn("[beta] bench-x: refused: not run", r.stdout)
        # beta's tag moves: only beta runs again
        self.b.set_queue("bench-x")
        self.b.ready("bench-x")
        r = self.run_queue()
        self.assertEqual(self.runs()[2:], [("beta", "bench-x")])

    def test_the_lock_refuses_a_second_runner_and_a_lone_waiter(self):
        self.a.set_queue("a1")
        self.a.ready("a1")
        self.write_config(self.a.ws)
        with open(self.state / "workstation" / "queue.lock", "a") as held:
            fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
            r = self.run_queue()
            self.assertEqual(r.returncode, 5, r.stdout + r.stderr)
            self.assertIn("REFUSED: another workstation run holds the machine lock", r.stdout)
            # a waiter run on its own (`just submit-when-ready`) refuses too, leaving no marker
            w = subprocess.run(
                ["bash", "bench/await-submit.sh", "a1", "master", "--no-suspend", "--hours", "0"],
                cwd=self.a.ws,
                env=self.env,
                capture_output=True,
                text=True,
                timeout=120,
            )
            self.assertEqual(w.returncode, 5, w.stdout + w.stderr)
            self.assertIn("holds the machine lock", w.stdout)
            self.assertEqual(self.calls("just"), [])
            self.assertFalse(self.ran(self.a, "a1"))
        # released: it runs
        r = self.run_queue()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(self.runs(), [("alpha", "a1")])

    def test_a_failed_recipe_does_not_stop_the_other_repository(self):
        self.a.set_queue("a1", "a2")
        self.a.ready("a1", "a2")
        self.b.set_queue("b1")
        self.b.ready("b1")
        self.write_config(self.a.ws, self.b.ws)
        r = self.run_queue(FAKE_JUST_FAIL=str(self.a.ws))
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertEqual(self.runs(), [("alpha", "a1"), ("beta", "b1"), ("alpha", "a2")])
        self.assertIn("[alpha] a1: ran: failed", r.stdout)
        self.assertIn("[beta] b1: ran: passed", r.stdout)

    def test_a_dirty_checkout_or_lost_origin_does_not_stop_the_other(self):
        self.a.set_queue("a1")
        self.a.ready("a1")
        self.b.set_queue("b1")
        self.b.ready("b1")
        (self.a.ws / "stray.txt").write_text("uncommitted\n")
        self.write_config(self.a.ws, self.tmp / "no-such-checkout", self.b.ws)
        r = self.run_queue()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(self.runs(), [("beta", "b1")])
        self.assertIn("alpha: the checkout", r.stdout)
        self.assertIn("not a git checkout; left out", r.stdout)
        # alpha clean again, beta's origin gone: alpha runs
        (self.a.ws / "stray.txt").unlink()
        self.b.origin.rename(self.b.origin.with_suffix(".away"))
        r = self.run_queue()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(self.runs(), [("beta", "b1"), ("alpha", "a1")])
        self.assertIn("beta: its queue could not be read", r.stdout)

    def test_each_repository_runs_its_own_waiter_and_an_args_column(self):
        # beta's own waiter, on its origin's master, stands in for a sibling's port
        own = (
            "#!/usr/bin/env bash\n"
            'echo "{\\"call\\": \\"waiter\\", \\"args\\": \\"$*\\"}" >> "$FAKE_LOG"\n'
        )
        (self.b.dev / "bench" / "await-submit.sh").write_text(own)
        self.b.set_queue("b1")
        self.b.ready("b1")
        # alpha's entry carries wikidata-pq's args= column
        (self.a.dev / "bench" / "workstation-queue.txt").write_text(
            "# repo: lmmx/alpha\n# channel: 11\na1  master  args=227,x  what it measures\n"
        )
        self.a.head = self.a.commit("queue")
        self.a.ready("a1")
        self.write_config(self.a.ws, self.b.ws)
        r = self.run_queue()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual([c["args"] for c in self.calls("just")], [["submit", "a1", "227", "x"]])
        self.assertEqual(
            [c["args"] for c in self.calls("waiter")], ["b1 master --no-suspend --hours 0"]
        )

    def test_single_repository_mode_is_unchanged(self):
        # no config and no --repo: the current checkout alone, its log in its own state
        self.a.set_queue("a1")
        self.a.ready("a1")
        self.b.set_queue("b1")
        self.b.ready("b1")
        r = self.run_queue(cwd=self.a.ws)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(self.runs(), [("alpha", "a1")])
        self.assertNotIn("[alpha]", r.stdout)
        self.assertIn("a1: ran: passed", r.stdout)
        logs = sorted(
            p.name.rsplit("-", 1)[0]
            for p in (self.state / "alpha" / "workstation" / "logs").iterdir()
        )
        self.assertEqual(logs, ["await-submit-a1", "workstation-queue"])
        self.assertFalse((self.state / "workstation" / "logs").exists())
        self.assertFalse((self.state / "beta").exists())
        # --repo wins over the config file
        self.write_config(self.a.ws, self.b.ws)
        r = self.run_queue("--repo", str(self.b.ws))
        self.assertEqual(self.runs(), [("alpha", "a1"), ("beta", "b1")])


if __name__ == "__main__":
    unittest.main()
