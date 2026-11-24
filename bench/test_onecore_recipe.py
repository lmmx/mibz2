#!/usr/bin/env python3
"""Untimed check of the real one-core wrapper and its unchanged driver.

Only git/just stand-ins run. This checks the frozen contrast, execution ordering,
failure propagation and driver identity; it does not build or time a decoder.
"""

import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
CANDIDATE = "d33787e138426cb441db6975f54b2eccca02e89a"
BASELINE = "b65b2aa54488f072cf005b8fe62461c32e6ef083"
DRIVER_SHA256 = "6167171b7889706226340adc0beba89bfbf5e933af944d7f68e49d97bc7bfa07"


def recipe(name):
    lines = (ROOT / ".just" / "bench.just").read_text().splitlines()
    start = next(i for i, line in enumerate(lines)
                 if line.startswith(name + ":") or line.startswith(name + " "))
    end = start + 1
    while end < len(lines) and (lines[end].startswith("    ") or not lines[end]):
        end += 1
    return "\n".join(lines[start:end]).rstrip() + "\n"


class OnecoreRecipe(unittest.TestCase):
    def test_real_just_dry_run(self):
        just = shutil.which("just")
        if just is None:
            if os.environ.get("ONECORE_REQUIRE_JUST") == "1":
                self.fail("just is required for the CI recipe dry run")
            self.skipTest("just unavailable locally; required in CI")
        result = subprocess.run(
            [just, "--justfile", str(ROOT / ".just" / "bench.just"),
             "--dry-run", "bench-onecore-inline-i9"],
            cwd=ROOT, capture_output=True, text=True, check=True,
        )
        commands = [shlex.split(line) for line in
                    (result.stdout + result.stderr).splitlines() if line.strip()]
        self.assertEqual(commands, [
            ["git", "fetch", "-q", "origin", "perf/onecore-walk-inline"],
            ["just", "bench-release-scaling", CANDIDATE, BASELINE, "10", "1"],
        ])

    def test_driver_is_frozen(self):
        driver = recipe("bench-release-scaling")
        self.assertEqual(hashlib.sha256(driver.encode()).hexdigest(), DRIVER_SHA256)
        self.assertIn('summary "$W/runs.jsonl" --ref mibz2', driver)

    def drive(self, fetch_exit=0):
        body = "\n".join(line[4:] for line in recipe("bench-onecore-inline-i9").splitlines()
                         if line.startswith("    ")) + "\n"
        with tempfile.TemporaryDirectory(prefix="onecore-wrapper-") as temp:
            tmp = Path(temp)
            script = tmp / "recipe.sh"
            script.write_text("set -eu\n" + body)
            stub = (
                "#!/usr/bin/env python3\n"
                "import json,os,sys\n"
                'with open(os.environ["CAPTURE"], "a") as f:\n'
                '    f.write(json.dumps([os.path.basename(sys.argv[0])] + sys.argv[1:]) + "\\n")\n'
                'sys.exit(int(os.environ["FETCH_EXIT"]) if os.path.basename(sys.argv[0]) == "git" else 0)\n'
            )
            for name in ("git", "just"):
                (tmp / name).write_text(stub)
                (tmp / name).chmod(0o755)
            env = dict(os.environ, PATH=str(tmp) + os.pathsep + os.environ["PATH"],
                       CAPTURE=str(tmp / "calls"), FETCH_EXIT=str(fetch_exit))
            subprocess.run(["bash", "-n", str(script)], check=True)
            result = subprocess.run(["bash", str(script)], env=env, check=False)
            calls = [json.loads(line) for line in (tmp / "calls").read_text().splitlines()]
            return result.returncode, calls

    def test_exact_contrast_and_grid(self):
        code, calls = self.drive()
        self.assertEqual(code, 0)
        self.assertEqual(calls, [
            ["git", "fetch", "-q", "origin", "perf/onecore-walk-inline"],
            ["just", "bench-release-scaling", CANDIDATE, BASELINE, "10", "1"],
        ])

    def test_failed_fetch_never_starts_measurement(self):
        code, calls = self.drive(fetch_exit=17)
        self.assertEqual(code, 17)
        self.assertEqual(calls, [["git", "fetch", "-q", "origin", "perf/onecore-walk-inline"]])


if __name__ == "__main__":
    unittest.main()

