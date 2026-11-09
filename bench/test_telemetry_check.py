"""`just bench-telemetry-check` end to end, untimed, with stand-ins for cargo, the harness, lscpu
and gh (Guardian D065): a FAILed or INCOMPLETE check still makes and submits its bundle, and
still exits nonzero, through bench/bundle.sh and `just submit`; a clean check exits 0; the
executable run is the one Cargo reports, and its digest is in the bundle.

    python3 bench/test_telemetry_check.py
"""

import json
import os
import re
import shutil
import subprocess
import tarfile
import tempfile
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECIPE = "bench-telemetry-check"
JUST = shutil.which("just")

# the harness's stand-in: `env`, `sample` and `telemetry-check`, the last as FAKE_CHECK says
FAKE_AB = r"""#!/usr/bin/env python3
import json, os, sys
cmd = sys.argv[1]
if cmd == "env":
    print(json.dumps({"cpus": 4, "machine": {"perf": {"cycles": False, "why": "stand-in"}}}))
elif cmd == "sample":
    for i in range(int(sys.argv[sys.argv.index("--count") + 1])):
        print(json.dumps({"sample": i, "state": {}}))
elif cmd == "telemetry-check":
    mode = os.environ["FAKE_CHECK"]
    status = {"pass": "PASS", "fail": "FAIL", "incomplete": "INCOMPLETE", "unavailable": "UNAVAILABLE"}[mode]
    for c in ("control-failure", "missing-counter"):
        print(json.dumps({"case": c, "status": "PASS", "detail": "ok", "fields": {}}))
    print(json.dumps({"case": "inheritance-hardware-cycles", "status": status, "detail": mode, "fields": {}}))
    sys.exit({"pass": 0, "unavailable": 0, "fail": 1, "incomplete": 3}[mode])
else:
    sys.exit(f"unknown {cmd}")
"""

# cargo's stand-in: reports the stand-in harness as the example it built, in a target
# directory of its own (never target/release/examples/ab)
FAKE_CARGO = r"""#!/usr/bin/env python3
import json, os, sys
if sys.argv[1:2] == ["-V"]:
    print("cargo 0.0.0 (stand-in)"); sys.exit(0)
assert "--message-format=json-render-diagnostics" in sys.argv, sys.argv
exe = os.environ["FAKE_AB"]
print(json.dumps({"reason": "compiler-artifact", "target": {"name": "ab", "kind": ["example"]}, "executable": exe}))
print(json.dumps({"reason": "build-finished", "success": True}))
"""

FAKE_RUSTC = "#!/bin/sh\necho 'rustc 0.0.0 (stand-in)'\n"
FAKE_LSCPU = "#!/bin/sh\necho 'CPU(s): 4'\n"

FAKE_GH = r"""#!/usr/bin/env python3
import json, os, re, sys
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
    """The recipe's script, as just would run it (its 4-space indentation removed)"""
    text = (ROOT / ".just" / "bench.just").read_text()
    m = re.search(rf"^{RECIPE}:\n((?:    .*\n|\n)+)", text, re.M)
    assert m, f"no {RECIPE}"
    body = "\n".join(l[4:] if l.startswith("    ") else l for l in m.group(1).splitlines())
    assert "{{" not in body, "the recipe takes no just parameters"
    return body + "\n"


class EndToEnd(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.remote = self.tmp / "lmmx" / "mibz2.git"
        self.remote.parent.mkdir()
        git(self.tmp, "init", "-q", "--bare", str(self.remote))
        self.work = self.tmp / "work"
        for rel in ("Justfile", ".just", "bench/bundle.sh", "bench/submit.sh",
                    "docs/journal/2026-11-09-telemetry-check-registration.md"):
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
        (self.work / "bench" / "telemetry-check.frozen").write_text(f"frozen {frozen}\n")
        git(self.work, "add", "-A")
        git(self.work, "commit", "-q", "-m", "frozen")
        bin_ = self.tmp / "bin"
        bin_.mkdir()
        elsewhere = self.tmp / "other-target" / "release" / "examples"
        elsewhere.mkdir(parents=True)
        for name, body, where in (("ab", FAKE_AB, elsewhere), ("cargo", FAKE_CARGO, bin_),
                                  ("rustc", FAKE_RUSTC, bin_), ("lscpu", FAKE_LSCPU, bin_),
                                  ("gh", FAKE_GH, bin_)):
            p = where / name
            p.write_text(body)
            p.chmod(0o755)
        self.ab = elsewhere / "ab"
        self.env = {**os.environ, "PATH": f"{bin_}:{os.environ['PATH']}", "FAKE_AB": str(self.ab),
                    "FAKE_COMMENTS": str(self.tmp / "comments.json"), "GH": str(bin_ / "gh")}

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def bundles(self):
        return sorted((self.work / "target" / "telemetry-check").glob("*.tar.gz"))

    def outcome(self, tarball):
        with tarfile.open(tarball) as t:
            stamp = tarball.name[: -len(".tar.gz")]
            names = set(t.getnames())
            return {
                n: t.extractfile(f"{stamp}/{n}").read().decode()
                for n in ("outcome.txt", "inputs.txt", "build.txt")
                if f"{stamp}/{n}" in names
            }

    def direct(self, mode):
        """The recipe's script run by bash, as just runs it"""
        time.sleep(1.1)  # (an attempt's directory is named to the second)
        script = self.tmp / "recipe.sh"
        script.write_text(recipe_body())
        return subprocess.run(["bash", str(script)], cwd=self.work, env={**self.env, "FAKE_CHECK": mode},
                              capture_output=True, text=True)

    def test_exit_follows_the_check_through_the_bundle(self):
        for mode, code, word in (("pass", 0, "CHECKED"), ("unavailable", 0, "CHECKED"),
                                 ("fail", 1, "FAILED"), ("incomplete", 3, "INCOMPLETE")):
            with self.subTest(mode=mode):
                before = set(self.bundles())
                r = self.direct(mode)
                self.assertEqual(r.returncode, code, r.stdout + r.stderr)
                [new] = set(self.bundles()) - before
                got = self.outcome(new)
                self.assertTrue(got["outcome.txt"].startswith(word), got["outcome.txt"])
                # the executable Cargo reported, and its digest
                self.assertIn(f"executable {self.ab}", got["build.txt"])
                digest = subprocess.run(["sha256sum", str(self.ab)], capture_output=True, text=True).stdout.split()[0]
                self.assertEqual(got["inputs.txt"].split()[0], digest)

    def test_dirty_inputs_refuse_nonzero(self):
        (self.work / ".just" / "bench.just").write_text((self.work / ".just" / "bench.just").read_text() + "\n# edit\n")
        r = self.direct("pass")
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("REFUSED", self.outcome(self.bundles()[-1])["outcome.txt"])

    @unittest.skipUnless(JUST, "just is not installed")
    def test_submit_keeps_a_failure_nonzero_and_submits_it(self):
        for mode, code in (("fail", 1), ("incomplete", 3), ("pass", 0)):
            with self.subTest(mode=mode):
                time.sleep(1.1)
                r = subprocess.run([JUST, "submit", RECIPE], cwd=self.work, env={**self.env, "FAKE_CHECK": mode},
                                   capture_output=True, text=True)
                self.assertEqual(r.returncode, code, r.stdout + r.stderr)
                self.assertIn("archived", r.stdout + r.stderr)
        tree = git(self.remote, "ls-tree", "-r", "--name-only", "evidence").split()
        self.assertEqual(len([t for t in tree if t.endswith(".tar.gz")]), 3)
        bodies = [c["body"] for c in json.loads((self.tmp / "comments.json").read_text())]
        self.assertTrue(any("FAILED" in b for b in bodies))
        self.assertTrue(any("INCOMPLETE" in b for b in bodies))


if __name__ == "__main__":
    unittest.main(verbosity=2)
