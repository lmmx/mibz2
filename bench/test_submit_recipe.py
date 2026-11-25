"""`just submit <recipe>` end to end, with a stand-in for gh and a scratch origin: the bundle it
submits is the one bench/bundle.sh says the recipe made, so a tarball the recipe fetched or
cached under target/ is never counted as a bundle (bench-character-stage2's first attempt on
25 November made stage1.tar.gz beside its bundle, and `just submit` refused both). A recipe that
never runs bundle.sh still falls back to the new target/*/*.tar.gz; two bundles are refused; a
failing recipe's bundle is submitted and its exit kept.

    python3 bench/test_submit_recipe.py
"""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
JUST = shutil.which("just")

FAKE_GH = r"""#!/usr/bin/env python3
import json, os, sys
store = os.environ["FAKE_COMMENTS"]
args = sys.argv[1:]
if args[:2] == ["auth", "status"]:
    sys.exit(0)
comments = json.load(open(store)) if os.path.exists(store) else []
if "--paginate" in args:
    print(json.dumps([{"body": c["body"]} for c in comments]))
    sys.exit(0)
body = open(args[args.index("-F") + 1].split("=@", 1)[1]).read()
comments.append({"body": body})
json.dump(comments, open(store, "w"))
print(f"https://github.com/lmmx/mibz2/pull/145#issuecomment-{len(comments)}")
"""

# the recipes under test: each makes target/<name>/<stamp>/ and bundles it as a workstation
# recipe does, with what the case needs beside it
RECIPES = r"""
import ".just/submit.just"

# a tarball fetched under target/ beside the bundle (stage 2's stage1.tar.gz)
stray:
    #!/usr/bin/env bash
    set -euo pipefail
    d=target/stray/$(date -u +%Y%m%dT%H%M%SZ); mkdir -p "$d"
    echo fetched > target/stray/x.txt && tar -czf target/stray/stage1.tar.gz -C target/stray x.txt
    echo PASS > "$d/outcome.txt"
    bash bench/bundle.sh "$d"

# a recipe that fails after bundling: submitted, its exit kept
failing:
    #!/usr/bin/env bash
    d=target/failing/$(date -u +%Y%m%dT%H%M%SZ); mkdir -p "$d"
    echo FAILED > "$d/outcome.txt"
    bash bench/bundle.sh "$d"
    exit 3

# two bundles: refused
two:
    #!/usr/bin/env bash
    set -euo pipefail
    for n in a b; do d=target/two/$n$(date -u +%Y%m%dT%H%M%SZ); mkdir -p "$d"; bash bench/bundle.sh "$d"; done

# no bundle.sh: the new target/*/*.tar.gz, as before
bare:
    #!/usr/bin/env bash
    set -euo pipefail
    s=$(date -u +%Y%m%dT%H%M%SZ); mkdir -p target/bare/$s
    echo PASS > target/bare/$s/outcome.txt
    tar -czf target/bare/$s.tar.gz -C target/bare $s
"""


def git(cwd, *args):
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True, capture_output=True, text=True
    ).stdout.strip()


@unittest.skipUnless(JUST, "just is not installed (CI installs it for this test)")
class Submit(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.remote = self.tmp / "lmmx" / "mibz2.git"
        self.remote.parent.mkdir()
        git(self.tmp, "init", "-q", "--bare", str(self.remote))
        self.work = self.tmp / "work"
        for rel in (
            ".just/submit.just",
            "bench/bundle.sh",
            "bench/submit.sh",
            "bench/workstation-queue.txt",
        ):
            dst = self.work / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(ROOT / rel, dst)
        (self.work / "Justfile").write_text(RECIPES)
        (self.work / ".gitignore").write_text("target/\n")
        git(self.work, "init", "-q")
        git(self.work, "config", "user.name", "Test")
        git(self.work, "config", "user.email", "test@example.com")
        git(self.work, "remote", "add", "origin", str(self.remote))
        git(self.work, "add", "-A")
        git(self.work, "commit", "-q", "-m", "inputs")
        git(self.work, "push", "-q", "origin", "HEAD:master")
        bin_ = self.tmp / "bin"
        bin_.mkdir()
        (bin_ / "gh").write_text(FAKE_GH)
        (bin_ / "gh").chmod(0o755)
        self.env = {
            **os.environ,
            "PATH": f"{bin_}:{os.environ['PATH']}",
            "GH": str(bin_ / "gh"),
            "FAKE_COMMENTS": str(self.tmp / "comments.json"),
            "XDG_STATE_HOME": str(self.tmp / "state"),
        }
        self.env.pop("WORKSTATION_LOCK_FD", None)
        self.env.pop("SUBMIT_BUNDLES", None)

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def submit(self, recipe):
        return subprocess.run(
            [JUST, "submit", recipe], cwd=self.work, env=self.env, capture_output=True, text=True
        )

    def archived(self):
        try:
            tree = git(self.remote, "ls-tree", "-r", "--name-only", "evidence").split()
        except subprocess.CalledProcessError:
            return []
        return [t for t in tree if t.endswith(".tar.gz")]

    def test_a_tarball_beside_the_bundle_is_not_counted(self):
        r = self.submit("stray")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        [a] = self.archived()
        self.assertTrue(a.startswith("stray/2"), a)  # the run's bundle, never stage1.tar.gz

    def test_a_failing_recipe_is_submitted_and_its_exit_kept(self):
        r = self.submit("failing")
        self.assertEqual(r.returncode, 3, r.stdout + r.stderr)
        self.assertEqual(len(self.archived()), 1)

    def test_two_bundles_are_refused(self):
        r = self.submit("two")
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("made 2 bundles, not one", r.stdout + r.stderr)
        self.assertEqual(self.archived(), [])

    def test_a_recipe_without_bundle_sh_falls_back_to_the_new_tarball(self):
        r = self.submit("bare")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        [a] = self.archived()
        self.assertTrue(a.startswith("bare/"), a)


if __name__ == "__main__":
    unittest.main(verbosity=2)
