import pathlib
import subprocess
import tempfile
import unittest

from checkpoint import CLIENT, observe


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
        (self.seed / CLIENT).parent.mkdir(parents=True)
        (self.seed / CLIENT).write_text("client v1\n")
        git(self.seed, "add", ".")
        git(self.seed, "commit", "-m", "initial")
        git(self.seed, "push", "origin", "master")
        git(root, "clone", str(self.remote), str(self.agent))

    def test_changed_client_detected_without_touching_work(self):
        first = observe(self.agent)
        (self.agent / "work.txt").write_text("unfinished research")
        (self.seed / CLIENT).write_text("client v2\n")
        git(self.seed, "commit", "-am", "client update")
        git(self.seed, "push", "origin", "master")
        result = observe(self.agent, first["observed_master"])
        self.assertFalse(result["client_current"])
        self.assertIn(CLIENT, result["control_changes"])
        self.assertEqual((self.agent / "work.txt").read_text(), "unfinished research")
        self.assertEqual((self.agent / CLIENT).read_text(), "client v1\n")
        self.assertFalse(result["authority_granted"])

    def test_repeat_observation_does_not_need_ack(self):
        first = observe(self.agent)
        second = observe(self.agent, first["observed_master"])
        self.assertTrue(second["client_current"])
        self.assertEqual(second["control_changes"], [])
        self.assertFalse(second["authority_granted"])

    def test_unreachable_origin_fails_closed(self):
        git(self.agent, "remote", "set-url", "origin", str(self.remote) + "-missing")
        with self.assertRaises(RuntimeError):
            observe(self.agent)


if __name__ == "__main__":
    unittest.main()
