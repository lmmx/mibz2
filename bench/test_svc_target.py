"""`just bench-svc-target` end to end, untimed, with stand-ins for Cargo, the preflight, lscpu
and gh (after bench/test_telemetry_check.py): every exit path makes its bundle with the
identity and the record in it, keeps its exit through bench/bundle.sh and `just submit`, and
a dirty or unfrozen checkout refuses; the executable run is the one Cargo reports, and its
digest is in the bundle.

    python3 bench/test_svc_target.py
"""

import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "bench"))
from svc_target_fixtures import edit, record  # noqa: E402

RECIPE = "bench-svc-target"
JUST = shutil.which("just")

# the preflight's stand-in: prints the record FAKE_RECORD names and exits FAKE_EXIT; "crash"
# exits 101 with no record
FAKE_PREFLIGHT = r"""#!/usr/bin/env python3
import os, sys
if os.environ["FAKE_MODE"] == "crash":
    print("thread 'main' panicked", file=sys.stderr); sys.exit(101)
sys.stdout.write(open(os.environ["FAKE_RECORD"]).read())
sys.exit(int(os.environ["FAKE_EXIT"]))
"""

FAKE_CARGO = r"""#!/usr/bin/env python3
import json, os, sys
if sys.argv[1:2] == ["-V"]:
    print("cargo 0.0.0 (stand-in)"); sys.exit(0)
assert "--message-format=json-render-diagnostics" in sys.argv and "svc_target" in sys.argv, sys.argv
exe = os.environ["FAKE_EXE"]
print(json.dumps({"reason": "compiler-artifact", "target": {"name": "svc_target", "kind": ["example"]}, "executable": exe}))
print(json.dumps({"reason": "build-finished", "success": True}))
"""

FAKE_RUSTC = "#!/bin/sh\necho 'rustc 0.0.0 (stand-in)'\n"

# timeout's stand-in: records its arguments, then either runs the real timeout or, with
# FAKE_TIMEOUT_EXIT set, exits as timeout does when it TERMs (124) or KILLs (137)
FAKE_TIMEOUT = r"""#!/bin/sh
echo "$@" > "$FAKE_TIMEOUT_ARGS"
if [ -n "$FAKE_TIMEOUT_EXIT" ]; then exit "$FAKE_TIMEOUT_EXIT"; fi
exec "$REAL_TIMEOUT" "$@"
"""
FAKE_LSCPU = "#!/bin/sh\necho 'CPU(s): 4'\n"

FAKE_GH = r"""#!/usr/bin/env python3
import json, os, sys
store = os.environ["FAKE_COMMENTS"]
args = sys.argv[1:]
if args[:2] == ["auth", "status"]:
    sys.exit(0)
comments = json.load(open(store)) if os.path.exists(store) else []
if "--paginate" in args:
    sys.exit(0)
body = open(args[args.index("-F") + 1].split("=@", 1)[1]).read()
comments.append({"body": body})
json.dump(comments, open(store, "w"))
print("https://github.com/lmmx/mibz2/pull/145#issuecomment-1")
"""


def git(cwd, *args):
    return subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


def recipe_body() -> str:
    text = (ROOT / ".just" / "bench.just").read_text()
    m = re.search(rf"^{RECIPE}:\n((?:    .*\n|\n)+)", text, re.M)
    assert m, f"no {RECIPE}"
    body = "\n".join(l[4:] if l.startswith("    ") else l for l in m.group(1).splitlines())
    assert "{{" not in body, "the recipe takes no just parameters"
    return body + "\n"


# (mode: record, exit, recipe's exit, outcome's first word)
def modes():
    t, tx = record(True)
    n, nx = record(False)
    failed = edit(t, lambda r: (r["at"][1]["built"].__setitem__("window", 12),
                                r.__setitem__("ok", False), r["why"].append("window")))
    lying = edit(t, lambda r: r["machine"].__setitem__("quota", {"kind": "unknown"}))
    return {
        "target": (t, tx, 0, "TARGET"),
        "not": (n, nx, 4, "NOT TARGET"),
        "failed": (failed, 3, 5, "FAILED"),
        "invalid": (lying, 0, 1, "INVALID RECORD"),
        "crash": (None, 101, 3, "INCOMPLETE"),
    }


class EndToEnd(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.remote = self.tmp / "lmmx" / "mibz2.git"
        self.remote.parent.mkdir()
        git(self.tmp, "init", "-q", "--bare", str(self.remote))
        self.work = self.tmp / "work"
        for rel in ("Justfile", ".just", "bench/bundle.sh", "bench/submit.sh", "bench/svc_target_check.py",
                    "docs/journal/2026-11-09-svc-target-registration.md"):
            src, dst = ROOT / rel, self.work / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            (shutil.copytree if src.is_dir() else shutil.copy)(src, dst)
        (self.work / ".gitignore").write_text("target/\n")
        git(self.work, "init", "-q")
        git(self.work, "config", "user.name", "Test")
        git(self.work, "config", "user.email", "test@example.com")
        git(self.work, "remote", "add", "origin", str(self.remote))
        git(self.work, "add", "-A")
        git(self.work, "commit", "-q", "-m", "inputs")
        frozen = git(self.work, "rev-parse", "HEAD")
        (self.work / "bench" / "svc-target.frozen").write_text(f"frozen {frozen}\n")
        git(self.work, "add", "-A")
        git(self.work, "commit", "-q", "-m", "frozen")
        bin_ = self.tmp / "bin"
        bin_.mkdir()
        elsewhere = self.tmp / "other-target" / "release" / "examples"
        elsewhere.mkdir(parents=True)
        for name, body, where in (("svc_target", FAKE_PREFLIGHT, elsewhere), ("cargo", FAKE_CARGO, bin_),
                                  ("rustc", FAKE_RUSTC, bin_), ("lscpu", FAKE_LSCPU, bin_),
                                  ("gh", FAKE_GH, bin_), ("timeout", FAKE_TIMEOUT, bin_)):
            p = where / name
            p.write_text(body)
            p.chmod(0o755)
        self.exe = elsewhere / "svc_target"
        self.env = {**os.environ, "PATH": f"{bin_}:{os.environ['PATH']}", "FAKE_EXE": str(self.exe),
                    "REAL_TIMEOUT": shutil.which("timeout"), "FAKE_TIMEOUT_ARGS": str(self.tmp / "timeout-args"),
                    "FAKE_COMMENTS": str(self.tmp / "comments.json"), "GH": str(bin_ / "gh")}

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def mode_env(self, mode):
        rec, ex, _, _ = modes()[mode]
        path = self.tmp / f"record-{mode}.json"
        path.write_text(json.dumps(rec) if rec is not None else "")
        return {**self.env, "FAKE_MODE": mode, "FAKE_RECORD": str(path), "FAKE_EXIT": str(ex)}

    def bundles(self):
        return sorted((self.work / "target" / "svc-target").glob("*.tar.gz"))

    def contents(self, tarball):
        with tarfile.open(tarball) as t:
            stamp = tarball.name[: -len(".tar.gz")]
            names = set(t.getnames())
            return {
                n: t.extractfile(f"{stamp}/{n}").read().decode()
                for n in ("outcome.txt", "inputs.txt", "build.txt", "commit.txt", "frozen.txt",
                          "preflight.json", "preflight.exit", "preflight-check.json", "raw/reads.txt",
                          "stages.txt")
                if f"{stamp}/{n}" in names
            }

    def direct(self, env):
        time.sleep(1.1)  # (an attempt's directory is named to the second)
        script = self.tmp / "recipe.sh"
        script.write_text(recipe_body())
        return subprocess.run(["bash", str(script)], cwd=self.work, env=env, capture_output=True, text=True)

    def test_every_outcome_is_bundled_with_its_identity_and_exit(self):
        for mode, (rec, _, code, word) in modes().items():
            with self.subTest(mode=mode):
                before = set(self.bundles())
                r = self.direct(self.mode_env(mode))
                self.assertEqual(r.returncode, code, r.stdout + r.stderr)
                [new] = set(self.bundles()) - before
                got = self.contents(new)
                self.assertTrue(got["outcome.txt"].startswith(word), got["outcome.txt"])
                self.assertIn(f"executable {self.exe}", got["build.txt"])
                digest = subprocess.run(["sha256sum", str(self.exe)], capture_output=True, text=True).stdout.split()[0]
                self.assertEqual(got["inputs.txt"].split()[0], digest)
                self.assertEqual(got["commit.txt"].strip(), git(self.work, "rev-parse", "HEAD"))
                self.assertIn("frozen", got["frozen.txt"])
                reads = got["raw/reads.txt"]
                self.assertIn("== /proc/self/cgroup", reads)
                self.assertTrue("cpu.max" in reads or "NOT RESOLVED" in reads, reads[-500:])
                self.assertIn("topology/core_id", reads)
                # every stage reached but the record's check, where the preflight crashed
                stages = got["stages.txt"].splitlines()
                self.assertEqual(len(stages), 5)
                want = 3 if mode == "crash" else 5
                self.assertEqual(sum(l.startswith("reached:") for l in stages), want, stages)
                self.assertIn("--kill-after=10 120", Path(self.env["FAKE_TIMEOUT_ARGS"]).read_text())
                if rec is not None:
                    self.assertEqual(json.loads(got["preflight.json"]), rec)
                    self.assertEqual(json.loads(got["preflight-check.json"])["verdict"], word)

    def test_dirty_inputs_refuse_nonzero(self):
        p = self.work / ".just" / "bench.just"
        p.write_text(p.read_text() + "\n# edit\n")
        r = self.direct(self.mode_env("target"))
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        got = self.contents(self.bundles()[-1])
        self.assertIn("REFUSED", got["outcome.txt"])
        # an early refusal reaches no stage, and its bundle has no later stage's files
        self.assertTrue(all(l.startswith("not reached:") for l in got["stages.txt"].splitlines()))
        for n in ("frozen.txt", "build.txt", "inputs.txt", "preflight.json", "preflight-check.json"):
            self.assertNotIn(n, got)

    def test_the_timeouts_are_incomplete_and_bundled(self):
        for code, words in ((124, "ended on TERM"), (137, "killed at 130 s")):
            with self.subTest(code=code):
                env = {**self.mode_env("target"), "FAKE_TIMEOUT_EXIT": str(code)}
                r = self.direct(env)
                self.assertEqual(r.returncode, 3, r.stdout + r.stderr)
                got = self.contents(self.bundles()[-1])
                self.assertIn(words, got["outcome.txt"])
                self.assertEqual(got["preflight.exit"].strip(), str(code))
                self.assertIn("not reached: preflight run", got["stages.txt"])

    def test_unfrozen_refuses_nonzero(self):
        (self.work / "bench" / "svc-target.frozen").write_text("# not frozen\n")
        git(self.work, "commit", "-q", "-am", "unfreeze")
        r = self.direct(self.mode_env("target"))
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("names no frozen commit", self.contents(self.bundles()[-1])["outcome.txt"])

    def test_changed_measured_inputs_refuse_nonzero(self):
        p = self.work / "bench" / "svc_target_check.py"
        p.write_text(p.read_text() + "\n# changed\n")
        git(self.work, "commit", "-q", "-am", "change")
        r = self.direct(self.mode_env("target"))
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("differ from the frozen", self.contents(self.bundles()[-1])["outcome.txt"])

    @unittest.skipUnless(JUST, "just is not installed")
    def test_submit_keeps_each_exit_and_submits_it(self):
        for mode in ("not", "invalid", "target"):
            with self.subTest(mode=mode):
                time.sleep(1.1)
                r = subprocess.run([JUST, "submit", RECIPE], cwd=self.work, env=self.mode_env(mode),
                                   capture_output=True, text=True)
                self.assertEqual(r.returncode, modes()[mode][2], r.stdout + r.stderr)
                self.assertIn("archived", r.stdout + r.stderr)
        tree = git(self.remote, "ls-tree", "-r", "--name-only", "evidence").split()
        self.assertEqual(len([t for t in tree if t.endswith(".tar.gz")]), 3)


if __name__ == "__main__":
    unittest.main(verbosity=2)
