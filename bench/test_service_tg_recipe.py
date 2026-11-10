"""`just bench-service-tg` end to end, past its refusals, with stand-ins for Cargo, rustc, the
harness, the preflight, the corpus builder, lscpu and gh: the earlier attempt's recipe test
(bench/test_service_guarded_recipe.py) carried over to the target-gated registration
(docs/journal/2026-11-10-service-guarded-target-gated-registration.md). A scratch repository
with a base commit, a candidate and a freeze naming both, the base's fetch time and the
toolchain; both builds (the base's in its own worktree), the copied and hashed binaries (two
arms, no twin), the preflight and its check, the corpus and its digest, the arms, the run at 9,
10 and 11 threads over 128 rounds, the judge's report, and the bundle, on every exit path:
VALID with the workstation conditions met, VALID with them not met (NOT SHOWN at a fallback
budget), REFUSED by the preflight (including a complete TARGET record followed by a nonzero
exit), a DRY check (refused by the judge as a DRY run), INCOMPLETE (the harness crashing
mid-run), INVALID (structural) by a child whose termination is not success, REFUSED by an
unfrozen file, a freeze without the fetch time or the toolchain, another toolchain, and dirty
or changed inputs; and through `just submit`. The diagnostics stay passive, bounded and
archived, and a failing one changes nothing timed. Nothing here is a measurement: the stand-in
harness writes synthetic rows.

    python3 bench/test_service_tg_recipe.py
"""

import hashlib
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
from svc_target_fixtures import record  # noqa: E402

RECIPE = "bench-service-tg"
RUSTC = "rustc 0.0.0 (stand-in)"
JUST = shutil.which("just")
CORPUS_BYTES = b"BZh9 stand-in corpus\n"
DECODED = 209700196

FAKE_AB = r"""#!/usr/bin/env python3
import json, os, random, sys
cmd = sys.argv[1]
if cmd == "env":
    if sys.argv[2:] != ["--facts"]:
        sys.exit("the stand-in refuses `ab env` without --facts: only the passive collector")
    if os.environ.get("FAKE_DIAG") == "fail":
        sys.exit(3)
    print(json.dumps({"facts": "stand-in", "cpus": 20}))
elif cmd == "sample":
    if os.environ.get("FAKE_DIAG") == "fail":
        print("a sample failing", file=sys.stderr); sys.exit(3)
    for i in range(int(sys.argv[sys.argv.index("--count") + 1])):
        print(json.dumps({"sample": i}))
elif cmd == "run":
    a = sys.argv
    out, arms = a[2], a[a.index("--arms") + 1].split(",")
    threads = [int(t) for t in a[a.index("--threads") + 1].split(",")]
    rounds, corpus = int(a[a.index("--rounds") + 1]), os.path.basename(a[-1]).rsplit(".", 1)[0]
    mode = os.environ["FAKE_RUN"]
    plans = {t: {"base": (4, 4, 0, 0), "head": (4, 4, 0, 0)} if t < 10 else
                {"base": (3, 0, 6, 0), "head": (2, 4, 0, 2) if t == 10 else (3, 0, 6, 0)}
             for t in (9, 10, 11)}
    def names(p, busy):
        l, pl, pi, s = p
        n = [f"mibz2-{i}" for i in range(l)] + [f"mibz2-coop-{i+1}" for i in range(pl)]
        n += [f"mibz2-coop-p{i+1}" for i in range(pi)] + [f"mibz2-svc-{i}" for i in range(s)]
        return " ".join(x + "=0.1" for x in n) if busy else " ".join(sorted(n + ["ab", "mibz2-scan"]))
    rng = random.Random(1)
    n = 0
    with open(out, "w") as f:
        for rnd in range(rounds):
            for t in threads:
                for arm in arms:
                    b = "head" if arm == "mibz2" else "base"
                    wall = 0.14 * rng.gauss(1, 0.01) / (1.05 if b == "head" and t == 10 else 1)
                    if b == "head" and t == 11 and mode == "harm":
                        wall *= 1.03
                    p = plans[t][b]
                    f.write(json.dumps({"round": rnd, "corpus": corpus, "threads": t, "arm": arm,
                        "wall": wall, "cpu": rng.gauss(1, 0.01) / (1.1 if b == "head" and t == 10 else 1),
                        "rss_kb": 70000, "out": int(os.environ["FAKE_OUT"]), "hash": "0123456789abcdef",
                        "thread_cpu": names(p, True), "observed_threads": names(p, False), "err": "",
                        "termination": "exit 17" if mode == "exit17" and n == 4 else "success"}) + "\n")
                    n += 1
                    if mode == "crash" and n == 7:
                        sys.exit("the stand-in harness crashing mid-run")
else:
    sys.exit(f"unknown {cmd}")
"""

FAKE_PREFLIGHT = r"""#!/usr/bin/env python3
import os, sys
sys.stdout.write(open(os.environ["FAKE_RECORD"]).read())
sys.stdout.flush()
print("stand-in preflight: exiting " + os.environ["FAKE_EXIT"], file=sys.stderr)
sys.exit(int(os.environ["FAKE_EXIT"]))
"""

FAKE_CARGO = r"""#!/usr/bin/env python3
import json, os, sys
if sys.argv[1:2] == ["-V"]:
    print("cargo 0.0.0 (stand-in)"); sys.exit(0)
assert "--message-format=json-render-diagnostics" in sys.argv, sys.argv
base = "/build-" in os.getcwd()
for i, a in enumerate(sys.argv):
    if a == "--example":
        name = sys.argv[i + 1]
        exe = os.environ["FAKE_AB_BASE" if base and name == "ab" else "FAKE_" + name.upper()]
        print(json.dumps({"reason": "compiler-artifact", "target": {"name": name, "kind": ["example"]}, "executable": exe}))
print(json.dumps({"reason": "build-finished", "success": True}))
"""

FAKE_RUSTC = "#!/bin/sh\necho \"${FAKE_RUSTC_SAYS:-rustc 0.0.0 (stand-in)}\"\n"
FAKE_LSCPU = "#!/bin/sh\n[ \"$FAKE_DIAG\" = fail ] && exit 5\necho 'CPU(s): 20'\n"

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

STUB_WIKIDATA = r"""import os, sys
cmd = sys.argv[1]
if cmd == "fetch":
    os.makedirs(sys.argv[3], exist_ok=True)
elif cmd == "build":
    open(sys.argv[5], "wb").write(%r)
""" % CORPUS_BYTES


def git(cwd, *args):
    return subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


def recipe_body(dry="", corpus=""):
    text = (ROOT / ".just" / "service-tg.just").read_text()
    m = re.search(rf"^{RECIPE} [^\n]*:\n((?:    .*\n|\n)+)", text, re.M)
    assert m, f"no {RECIPE}"
    body = "\n".join(l[4:] if l.startswith("    ") else l for l in m.group(1).splitlines())
    return body.replace("{{DRY}}", dry).replace("{{CORPUS}}", corpus) + "\n"


class EndToEnd(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.remote = self.tmp / "lmmx" / "mibz2.git"
        self.remote.parent.mkdir()
        git(self.tmp, "init", "-q", "--bare", str(self.remote))
        w = self.work = self.tmp / "work"
        for rel in (".just/service-tg.just", ".just/submit.just", "bench/bundle.sh", "bench/submit.sh",
                    "bench/service_tg.py", "bench/service_guarded.py", "bench/svc_target_check.py",
                    "docs/journal/2026-11-10-service-guarded-target-gated-registration.md"):
            (w / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(ROOT / rel, w / rel)
        (w / "Justfile").write_text('import ".just/service-tg.just"\nimport ".just/submit.just"\n')
        (w / "bench" / "wikidata.py").write_text(STUB_WIKIDATA)
        (w / "bench" / "wikidata.json").write_text(json.dumps({
            "windows_dir": "wd", "corpus_sha256": hashlib.sha256(CORPUS_BYTES).hexdigest(),
            "decoded_bytes": DECODED}))
        (w / "bench" / "service-tg.frozen").write_text("# not frozen\n")
        (w / ".gitignore").write_text("target/\nwd/\n")
        (w / "src").mkdir()
        (w / "src" / "lib.rs").write_text("// the base\n")
        git(w, "init", "-q")
        git(w, "config", "user.name", "Test")
        git(w, "config", "user.email", "test@example.com")
        git(w, "remote", "add", "origin", str(self.remote))
        git(w, "add", "-A")
        git(w, "commit", "-q", "-m", "base")
        self.base = git(w, "rev-parse", "HEAD")
        (w / "src" / "lib.rs").write_text("// the candidate\n")
        git(w, "commit", "-q", "-am", "candidate")
        self.cand = git(w, "rev-parse", "HEAD")
        self.freeze = (f"candidate {self.cand}\nbase {self.base}\nbase_fetched 2026-11-10T00:00:00Z\n"
                       f"rustc {RUSTC}\nni_rustc rustc 1.99.1 (x)\ncorpus_text {'7' * 64}\n")
        (w / "bench" / "service-tg.frozen").write_text(self.freeze)
        git(w, "commit", "-q", "-am", "freeze (test fixture)")
        bin_ = self.tmp / "bin"
        bin_.mkdir()
        ex = self.tmp / "cargo-target" / "release" / "examples"
        bx = self.tmp / "base-target" / "release" / "examples"
        ex.mkdir(parents=True)
        bx.mkdir(parents=True)
        for name, body, where in (("ab", FAKE_AB, ex), ("ab", FAKE_AB, bx), ("windows", "#!/bin/sh\n", ex),
                                  ("svc_target", FAKE_PREFLIGHT, ex), ("cargo", FAKE_CARGO, bin_),
                                  ("rustc", FAKE_RUSTC, bin_), ("lscpu", FAKE_LSCPU, bin_), ("gh", FAKE_GH, bin_)):
            p = where / name
            p.write_text(body)
            p.chmod(0o755)
        self.env = {**os.environ, "PATH": f"{bin_}:{os.environ['PATH']}", "FAKE_AB": str(ex / "ab"),
                    "FAKE_AB_BASE": str(bx / "ab"), "FAKE_WINDOWS": str(ex / "windows"),
                    "FAKE_SVC_TARGET": str(ex / "svc_target"), "FAKE_OUT": str(DECODED),
                    "FAKE_COMMENTS": str(self.tmp / "comments.json"), "GH": str(bin_ / "gh")}
        for k in [k for k in self.env if k.startswith(("CARGO_", "RUST")) and k not in ("CARGO_HOME", "RUSTUP_HOME")]:
            del self.env[k]

    def tearDown(self):
        subprocess.run(["git", "worktree", "prune"], cwd=self.work, capture_output=True)
        shutil.rmtree(self.tmp)

    def env_for(self, run="valid", target=True):
        rec, ex = record(target)
        p = self.tmp / f"record-{target}.json"
        p.write_text(json.dumps(rec))
        return {**self.env, "FAKE_RUN": run, "FAKE_RECORD": str(p), "FAKE_EXIT": str(ex)}

    def bundles(self):
        return sorted((self.work / "target" / "service-tg").glob("*.tar.gz"))

    def member(self, tarball, name):
        with tarfile.open(tarball) as t:
            stamp = tarball.name[: -len(".tar.gz")]
            return t.extractfile(f"{stamp}/{name}").read().decode()

    def names(self, tarball):
        with tarfile.open(tarball) as t:
            stamp = tarball.name[: -len(".tar.gz")]
            got = {n[len(stamp) + 1:] for n in t.getnames() if n.startswith(stamp + "/")}
            outcome = t.extractfile(f"{stamp}/outcome.txt").read().decode()
            return got, outcome

    def run_recipe(self, env, dry="", corpus=""):
        time.sleep(1.1)
        before = set(self.bundles())
        script = self.tmp / "recipe.sh"
        script.write_text(recipe_body(dry, corpus))
        r = subprocess.run(["bash", str(script)], cwd=self.work, env=env, capture_output=True, text=True)
        [new] = set(self.bundles()) - before
        return r, *self.names(new)

    def test_valid(self):
        r, got, outcome = self.run_recipe(self.env_for("valid"))
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertTrue(outcome.startswith("VALID: WORKSTATION CONDITIONS MET"), outcome)
        self.assertNotIn("bin/ab-base-twin", got)
        for n in ("commits.txt", "arms.txt", "corpus.sha256", "inputs.txt", "preflight.json", "preflight.exit",
                  "preflight-check.json", "runs.jsonl", "service-tg.json", "service-tg.md",
                  "bin/ab-base", "bin/ab-head", "bin/svc_target", "bin/ab-base.sha256",
                  "diagnostics/env.json", "diagnostics/sample-before.jsonl", "diagnostics/sample-after.jsonl",
                  "diagnostics/lscpu.txt", "diagnostics/discovery.json", "diagnostics/exits.txt",
                  "console.log", "outcome.txt"):
            self.assertIn(n, got)
        [b] = self.bundles()
        # (every diagnostic ran and exited 0: `env` with --facts, which the stand-in insists on)
        self.assertEqual(self.member(b, "diagnostics/exits.txt").splitlines(), [
            "env.json: exit 0", "lscpu.txt: exit 0", "discovery.json: exit 0",
            "sample-before.jsonl: exit 0", "sample-after.jsonl: exit 0"])
        # (the preflight's successful control: exit 0, checked TARGET, the checker's exit 0)
        self.assertEqual(self.member(b, "preflight.exit").strip(), "0")
        self.assertEqual(self.member(b, "preflight-check.exit").strip(), "0")
        disc = json.loads(self.member(b, "diagnostics/discovery.json"))
        self.assertEqual(disc["schema"], "mibz2.service_guarded.discovery/1")
        self.assertIn("base_fetched 2026-11-10T00:00:00Z", self.member(b, "commits.txt"))
        self.assertEqual(len(self.member(b, "arms.txt").splitlines()), 2)
        self.assertIn(RUSTC, self.member(b, "inputs.txt").splitlines())
        runs = [json.loads(l) for l in self.member(b, "runs.jsonl").splitlines()]
        self.assertEqual(len(runs), 2 * 3 * 128)
        self.assertEqual(sorted({r["threads"] for r in runs}), [9, 10, 11])

    def test_a_failing_diagnostic_is_archived_and_changes_nothing_timed(self):
        env = dict(self.env_for("valid"), FAKE_DIAG="fail")
        r, got, outcome = self.run_recipe(env)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertTrue(outcome.startswith("VALID: WORKSTATION CONDITIONS MET"), outcome)
        [b] = self.bundles()
        exits = self.member(b, "diagnostics/exits.txt").splitlines()
        self.assertEqual(exits, ["env.json: exit 3", "lscpu.txt: exit 5", "discovery.json: exit 0",
                                 "sample-before.jsonl: exit 3", "sample-after.jsonl: exit 3"])
        self.assertIn("a sample failing", self.member(b, "diagnostics/sample-before.jsonl.stderr"))
        self.assertEqual(len(self.member(b, "runs.jsonl").splitlines()), 768)

    def test_every_diagnostic_is_passive_and_bounded(self):
        # (REVIEW 6746023579 §2: `ab env` only as the facts-only collector; every external
        # diagnostic outside the timed run through `diag`, whose one command is bounded by a
        # TERM timeout with a forced kill after it; the preflight keeps its own, longer bound)
        body = recipe_body()
        [d] = re.findall(r"^diag\(\) \{.*?^\}", body, re.M | re.S)
        self.assertRegex(d, r'\n\s*timeout --kill-after=5 30 "\$@" > ')
        rest = body.replace(d, "")
        envs = re.findall(r'^.*"\$AB" env\b.*$', rest, re.M)
        self.assertEqual(envs, ['diag env.json "$AB" env --facts'])
        for pat in (r'"\$AB" sample', r"\blscpu\b(?! and)", r"service_guarded\.py discovery"):
            calls = [c for c in re.findall(rf"^.*{pat}.*$", rest, re.M) if not c.lstrip().startswith("#")]
            self.assertTrue(calls, pat)
            for c in calls:
                self.assertRegex(c, r"^diag \S+ ", c)
        self.assertIn('timeout --kill-after=10 120 "$W/bin/svc_target"', rest)
        self.assertNotRegex(rest, r"timeout (?!--kill-after)")

    def test_a_child_that_did_not_succeed_is_invalid(self):
        r, _, outcome = self.run_recipe(self.env_for("exit17"))
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertTrue(outcome.startswith("INVALID (structural)"), outcome)
        self.assertIn("termination 'exit 17'", outcome)

    def test_a_complete_target_record_then_a_nonzero_exit_refuses(self):
        # (REVIEW 6746559255's requirement for every required collector: the preflight is
        # gated on its own successful completion. A stand-in prints a complete TARGET record
        # and then exits 17, or as `timeout` would, 124 (TERM) or 137 (KILL): REFUSED before
        # anything is timed, its raw output kept. The control: test_valid, the same record
        # with exit 0, runs)
        rec, ex = record(True)
        self.assertEqual(ex, 0)
        for status in (17, 124, 137):
            with self.subTest(status):
                env = dict(self.env_for("valid"), FAKE_EXIT=str(status))
                r, got, outcome = self.run_recipe(env)
                self.assertEqual(r.returncode, 4, r.stdout + r.stderr)
                self.assertTrue(outcome.startswith("REFUSED: the preflight is"), outcome)
                self.assertIn(f"(exit {status})", outcome)
                self.assertNotIn("runs.jsonl", got)
                b = max(self.bundles())
                self.assertEqual(json.loads(self.member(b, "preflight.json")), rec)
                self.assertEqual(self.member(b, "preflight.exit").strip(), str(status))
                self.assertIn(f"exiting {status}", self.member(b, "preflight.stderr"))
                self.assertNotEqual(json.loads(self.member(b, "preflight-check.json"))["verdict"], "TARGET")

    def test_not_shown_at_a_fallback_budget_is_valid_and_not_met(self):
        r, _, outcome = self.run_recipe(self.env_for("harm"))
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertTrue(outcome.startswith("VALID: WORKSTATION CONDITIONS NOT MET"), outcome)
        self.assertIn("condition 4: NOT SHOWN at 11 threads", outcome)

    def test_the_preflight_refuses_before_any_measurement(self):
        r, got, outcome = self.run_recipe(self.env_for("valid", target=False))
        self.assertEqual(r.returncode, 4, r.stdout + r.stderr)
        self.assertTrue(outcome.startswith("REFUSED: the preflight is NOT TARGET"), outcome)
        self.assertIn("preflight.json", got)
        self.assertNotIn("runs.jsonl", got)

    def test_a_dry_check_runs_through_and_is_never_valid(self):
        corpus = self.tmp / "c.bz2"
        corpus.write_bytes(b"another corpus")
        r, got, outcome = self.run_recipe(self.env_for("valid", target=False), dry="1", corpus=str(corpus))
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertTrue(outcome.startswith("INVALID (identity)"), outcome)
        self.assertIn("DRY-RUN.txt", got)
        self.assertIn("service-tg.json", got)

    def test_a_crashing_harness_is_incomplete(self):
        r, got, outcome = self.run_recipe(self.env_for("crash"))
        self.assertEqual(r.returncode, 3, r.stdout + r.stderr)
        self.assertTrue(outcome.startswith("INCOMPLETE"), outcome)

    def test_refusals(self):
        frozen = self.work / "bench" / "service-tg.frozen"
        frozen.write_text("# not frozen\n")
        git(self.work, "commit", "-q", "-am", "unfreeze")
        r, _, outcome = self.run_recipe(self.env_for())
        self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
        self.assertIn("names no frozen candidate", outcome)
        git(self.work, "reset", "-q", "--hard", "HEAD~1")
        # (a freeze naming the base without the time master was fetched, or without the toolchain)
        for drop in ("base_fetched", "rustc"):
            frozen.write_text("".join(l + "\n" for l in self.freeze.splitlines() if not l.startswith(drop)))
            git(self.work, "commit", "-q", "-am", f"a freeze without {drop}")
            r, _, outcome = self.run_recipe(self.env_for())
            self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
            self.assertIn("base fetch time and toolchain", outcome)
            git(self.work, "reset", "-q", "--hard", "HEAD~1")
        # (another toolchain than the frozen one: refused before anything is built)
        r, got, outcome = self.run_recipe(dict(self.env_for(), FAKE_RUSTC_SAYS="rustc 0.0.1 (another)"))
        self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
        self.assertIn("not the frozen rustc 0.0.0 (stand-in)", outcome)
        self.assertNotIn("runs.jsonl", got)
        p = self.work / "bench" / "service_tg.py"
        p.write_text(p.read_text() + "\n# dirty\n")
        r, _, outcome = self.run_recipe(self.env_for())
        self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
        self.assertIn("uncommitted", outcome)
        git(self.work, "commit", "-q", "-am", "changed after the freeze")
        r, got, outcome = self.run_recipe(self.env_for())
        self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
        self.assertIn("differ from the frozen candidate", outcome)
        self.assertIn("NOT-FROZEN.txt", got)

    @unittest.skipUnless(JUST, "just is not installed")
    def test_submit_keeps_each_exit(self):
        for env, code in ((self.env_for("valid"), 0), (self.env_for("valid", target=False), 4)):
            time.sleep(1.1)
            r = subprocess.run([JUST, "submit", RECIPE], cwd=self.work, env=env, capture_output=True, text=True)
            self.assertEqual(r.returncode, code, r.stdout + r.stderr)
            self.assertIn("archived", r.stdout + r.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
