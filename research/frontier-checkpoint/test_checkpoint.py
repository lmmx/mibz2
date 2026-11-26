import pathlib
import subprocess
import tempfile
import unittest

from checkpoint import CLIENT, Unavailable, observe

CLAIM = "frontier/claims/probe-a.toml"
CARD = 'id = "probe-a"\nstatus = "open"\npremises = ["research/a/"]\nowned_paths = []\nstop_test = "{}"\n'
FLAG = "flags: {}\ninvariant: exact bytes\nby: test\nstatus: {}\n\n**Evidence:** fixture.\n"


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True, stderr=subprocess.DEVNULL).strip()


class CheckpointTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        root = pathlib.Path(self.tmp.name)
        self.remote = root / "remote.git"
        self.seed = root / "seed"
        self.agent = root / "agent"
        git(root, "init", "--bare", "-b", "master", str(self.remote))
        git(root, "clone", str(self.remote), str(self.seed))
        git(self.seed, "config", "user.name", "Test")
        git(self.seed, "config", "user.email", "test@example.invalid")
        self.write(CLIENT, "client v1\n")
        self.write(CLAIM, CARD.format("frozen"))
        self.write("research/a/notes.md", "premise v1\n")
        self.write("research/b/notes.md", "unrelated v1\n")
        self.push("initial")
        git(root, "clone", str(self.remote), str(self.agent))
        self.cursor = self.agent / ".git" / "checkpoint-cursor"

    def write(self, rel, text):
        path = self.seed / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def push(self, message):
        git(self.seed, "add", "-A")
        git(self.seed, "commit", "-m", message)
        git(self.seed, "push", "-q", "origin", "master")

    def snapshot(self):
        return git(self.agent, "rev-parse", "HEAD"), git(self.agent, "status", "--porcelain", "-uall")

    def test_stale_client_detected_and_dirty_work_preserved(self):
        observe(self.agent, cursor=self.cursor)
        (self.agent / "research/a/notes.md").write_text("unfinished edit")
        (self.agent / "scratch.txt").write_text("unfinished research")
        before = self.snapshot()
        self.write(CLIENT, "client v2\n")
        self.write("research/a/notes.md", "premise v2\n")
        self.push("client and premise update")
        result = observe(self.agent, cursor=self.cursor)
        self.assertFalse(result["client_current"])
        self.assertIn(CLIENT, result["control_changes"])
        self.assertEqual(result["dirty_also_changed_on_master"], ["research/a/notes.md"])
        self.assertEqual(self.snapshot(), before)
        self.assertEqual((self.agent / "scratch.txt").read_text(), "unfinished research")
        self.assertEqual((self.agent / CLIENT).read_text(), "client v1\n")
        self.assertFalse(result["authority_granted"])

    def test_locally_edited_client_is_not_current(self):
        (self.agent / CLIENT).write_text("client v1 with local patch\n")
        self.assertFalse(observe(self.agent)["client_current"])

    def test_relevant_scope_change_surfaced_unrelated_not(self):
        start = observe(self.agent, claim="probe-a")["observed_master"]
        self.write("research/b/notes.md", "unrelated v2\n")
        self.push("unrelated")
        unrelated = observe(self.agent, since=start, claim="probe-a")
        self.assertEqual(unrelated["scope"]["premise_changes"], [])
        self.assertEqual(unrelated["scope"]["open_flags"], [])
        self.assertEqual(unrelated["control_changes"], [])
        self.assertEqual(unrelated["next_action"], ["continue current work"])
        self.write("research/a/notes.md", "premise v2\n")
        self.write("frontier/flags/2026-11-26-a.md", FLAG.format("research/a/notes.md", "open"))
        self.write(CLAIM, CARD.format("re-frozen"))
        self.push("premise, flag and stop test")
        relevant = observe(self.agent, since=start, claim="probe-a")
        s = relevant["scope"]
        self.assertEqual(s["premise_changes"], ["research/a/notes.md"])
        self.assertEqual(s["new_flags"], ["2026-11-26-a.md"])
        self.assertTrue(s["card_changed"] and s["stop_test_changed"])
        self.assertEqual(relevant["commits_coalesced"], 2)
        self.assertTrue(relevant["client_current"])

    def test_open_restriction_stays_visible_after_cursor_moves(self):
        self.write("frontier/flags/2026-11-26-a.md", FLAG.format("probe-a", "open"))
        self.push("flag")
        observe(self.agent, claim="probe-a", cursor=self.cursor)
        again = observe(self.agent, claim="probe-a", cursor=self.cursor)
        self.assertTrue(again["unchanged"])
        self.assertEqual(again["scope"]["open_flags"], ["2026-11-26-a.md"])
        self.assertEqual(again["scope"]["new_flags"], [])
        self.write("frontier/flags/2026-11-26-a.md", FLAG.format("probe-a", "resolved"))
        self.push("resolve")
        self.assertEqual(observe(self.agent, claim="probe-a", cursor=self.cursor)["scope"]["open_flags"], [])

    def test_repeated_snapshot_idempotent_and_coalesced(self):
        first = observe(self.agent, cursor=self.cursor)
        for i in range(3):
            self.write("research/b/notes.md", f"unrelated {i}\n")
            self.push(f"unrelated {i}")
        one = observe(self.agent, since=first["observed_master"], claim="probe-a")
        two = observe(self.agent, since=first["observed_master"], claim="probe-a")
        strip = lambda r: {k: v for k, v in r.items() if k != "observation_seconds"}
        self.assertEqual(strip(one), strip(two))
        self.assertEqual(one["commits_coalesced"], 3)
        observe(self.agent, cursor=self.cursor)
        idle = observe(self.agent, cursor=self.cursor)
        self.assertTrue(idle["unchanged"])
        self.assertEqual(idle["commits_coalesced"], 0)

    def test_unavailable_origin_never_uses_cached_ref_or_moves_cursor(self):
        observe(self.agent, cursor=self.cursor)
        held = self.cursor.read_text()
        self.write(CLIENT, "client v2\n")
        self.push("client update the agent cannot see")
        git(self.agent, "remote", "set-url", "origin", str(self.remote) + "-missing")
        with self.assertRaises(Unavailable):
            observe(self.agent, cursor=self.cursor)
        self.assertEqual(self.cursor.read_text(), held)
        proc = subprocess.run(
            ["python3", str(pathlib.Path(__file__).with_name("checkpoint.py")), "--repo", str(self.agent)],
            capture_output=True, text=True,
        )
        self.assertEqual(proc.returncode, 2)
        self.assertIn('"authority_granted": false', proc.stdout)

    def test_mixed_old_client_limit_is_explicit(self):
        self.write(CLIENT, "client v2\n")
        self.push("protocol change")
        result = observe(self.agent)
        self.assertFalse(result["client_current"])
        # An old client that never observes can still write refs; nothing here fences it.
        git(self.agent, "push", "-q", "origin", "HEAD:refs/heads/hold/probe-a")
        self.assertTrue(git(self.remote, "rev-parse", "refs/heads/hold/probe-a"))
        self.assertTrue(any("not detected or fenced" in line for line in result["limits"]))


if __name__ == "__main__":
    unittest.main()
