"""`just bench-run-fed-leader` end to end (docs/journal/2026-11-09-run-fed-leader.md §R10),
untimed, with stand-ins for Cargo, the harness (`ab`), the occupancy instrument, the corpus
builder, rustc, lscpu and gh, in a scratch repository with a freeze: the builds as Cargo
reports them, copied and hashed into the bundle; the preflight (its readings passive and
bounded, §R11) and its refusal; the run
with `--sysfs-state` and never `--machine-state`; the identity, the admission, the captures'
admissions and the diagnostics; and the bundle, on every exit path:
- VALID (0), every file the registration lists in the bundle;
- INVALID (1): a record off the registered workload, and the harness crashing mid-run
  (its partial records kept);
- REFUSED by the preflight (1), before anything is timed, its facts archived: not the i9,
  and a declared setting off;
- a DRY check (0, DRY, never JUDGED), going on past a preflight that did not pass;
- REFUSED (1): unfrozen, inputs off the frozen commit, a dirty checkout, a build override in
  the environment, a CORPUS without DRY, a failed build, a corpus off its digest;
- INCOMPLETE (3): a step failing outside the measured region (the corpus builder);
- and through `just submit`, the exit kept and the bundle archived.
Nothing here is a measurement: the stand-in harness writes the admission tests' synthetic
records.

    python3 bench/test_run_fed_leader_recipe.py
"""

import hashlib
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
RECIPE = "bench-run-fed-leader"
JUST = shutil.which("just")
CORPUS_BYTES = b"BZh9 stand-in corpus\n"
CORPUS_SHA = hashlib.sha256(CORPUS_BYTES).hexdigest()
REGISTRATION = "docs/journal/2026-11-09-run-fed-leader.md"

# The harness's stand-in: `env` and `sample` as the host says (FAKE_HOST: i9, other, or
# governor), `run` the admission tests' synthetic records (FAKE_RUN: valid, wrongout, crash)
FAKE_AB = r"""#!/usr/bin/env python3
import json, os, sys
sys.path.insert(0, os.environ["FAKE_BENCH"])
import test_run_fed_leader_ws as t
cmd, host = sys.argv[1], os.environ.get("FAKE_HOST", "i9")
if cmd == "env":
    assert sys.argv[2:] == ["--facts"], sys.argv  # (only the passive collector, §R11)
    env = t.host_env(governor="performance") if host == "governor" else t.host_env()
    if host == "other":
        env.update(cpus=4, cores=2)
    print(json.dumps(env))
elif cmd == "sample":
    n = int(sys.argv[sys.argv.index("--count") + 1])
    for i in range(n):
        print(json.dumps({"sample": i, "count": n, "state": t.snapshot(800, temp=33.0, t=i)}))
elif cmd == "run":
    a = sys.argv
    assert "--sysfs-state" in a and "--machine-state" not in a, a
    out, rounds = a[2], int(a[a.index("--rounds") + 1])
    assert a[a.index("--threads") + 1] == "10", a
    assert a[a.index("--arms") + 1].split(",") == t.ws.ARMS, a
    mode = os.environ.get("FAKE_RUN", "valid")
    recs = t.records(rounds=rounds, factors={t.ws.PRIMARY: 0.96})
    if mode == "wrongout":
        recs[30]["out"] += 1
    with open(out, "w") as f:
        for i, r in enumerate(recs):
            f.write(json.dumps(r) + "\n")
            if mode == "crash" and i == 6:
                f.flush()
                sys.exit("the stand-in harness crashing mid-run")
else:
    sys.exit(f"unknown {cmd}")
"""

# The occupancy instrument's stand-in: a capture of the admission tests' schema, its commit
# and corpus the ones the recipe passes
FAKE_OCC = r"""#!/usr/bin/env python3
import json, os, sys
sys.path.insert(0, os.environ["FAKE_BENCH"])
import test_run_fed_leader_ws as t
a = sys.argv
sub, rounds, out = a[a.index("--run-fed") + 1], int(a[a.index("--rounds") + 1]), a[a.index("--out") + 1]
meta = dict(a[i + 1].split("=", 1) for i, x in enumerate(a) if x == "--meta")
cap = t.capture(sub, dry=rounds == t.ws.DRY_OCC_ROUNDS,
                ident={"head": meta["commit"], "corpus_sha256_before": meta["corpus_sha256"]})
json.dump(cap, open(out, "w"))
print(f"occupancy stand-in: {sub}")
"""

FAKE_CARGO = r"""#!/usr/bin/env python3
import json, os, sys
if sys.argv[1:2] == ["-V"]:
    print("cargo 0.0.0 (stand-in)"); sys.exit(0)
if os.environ.get("FAKE_BUILD") == "fail":
    sys.exit(101)
assert "--message-format=json-render-diagnostics" in sys.argv, sys.argv
feats = sys.argv[sys.argv.index("--features") + 1]
for i, a in enumerate(sys.argv):
    if a == "--example":
        name = sys.argv[i + 1]
        assert feats == ("trace,run-fed" if name == "occupancy" else "run-fed"), (name, feats)
        print(json.dumps({"reason": "compiler-artifact", "target": {"name": name, "kind": ["example"]},
                          "executable": os.environ["FAKE_" + name.upper()]}))
print(json.dumps({"reason": "build-finished", "success": True}))
"""

FAKE_RUSTC = "#!/bin/sh\nif [ \"$1\" = -vV ]; then echo 'rustc 0.0.0 (stand-in)'; echo 'host: x86_64'; else echo 'rustc 0.0.0 (stand-in)'; fi\n"

FAKE_LSCPU = r"""#!/bin/sh
if [ "${FAKE_HOST:-i9}" = other ]; then
  echo 'Model name:                           AMD EPYC 7763 64-Core Processor'
else
  echo 'Model name:                           Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz'
fi
"""

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

STUB_WIKIDATA = (
    r"""import os, sys
cmd = sys.argv[1]
if os.environ.get("FAKE_CORPUS_MODE") == "fail":
    sys.exit("the stand-in corpus builder failing")
if cmd == "fetch":
    os.makedirs(sys.argv[3], exist_ok=True)
elif cmd == "build":
    data = %r
    if os.environ.get("FAKE_CORPUS_MODE") == "other":
        data = b"another corpus"
    open(sys.argv[5], "wb").write(data)
"""
    % CORPUS_BYTES
)


def git(cwd, *args):
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True, capture_output=True, text=True
    ).stdout.strip()


def recipe_body(dry="", corpus=""):
    """The recipe's script, as just would run it (its 4-space indentation removed)"""
    text = (ROOT / ".just" / "bench.just").read_text()
    m = re.search(rf"^{RECIPE} [^\n]*:\n((?:    .*\n|\n)+)", text, re.M)
    assert m, f"no {RECIPE}"
    body = "\n".join(
        line[4:] if line.startswith("    ") else line for line in m.group(1).splitlines()
    )
    return body.replace("{{DRY}}", dry).replace("{{CORPUS}}", corpus) + "\n"


class EndToEnd(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.remote = self.tmp / "lmmx" / "mibz2.git"
        self.remote.parent.mkdir()
        git(self.tmp, "init", "-q", "--bare", str(self.remote))
        w = self.work = self.tmp / "work"
        for rel in (
            ".just/bench.just",
            ".just/submit.just",
            "bench/bundle.sh",
            "bench/submit.sh",
            "bench/run_fed_leader_ws.py",
            "bench/run_fed_leader.py",
            "bench/team_size.py",
            REGISTRATION,
        ):
            (w / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(ROOT / rel, w / rel)
        (w / "Justfile").write_text('import ".just/bench.just"\nimport ".just/submit.just"\n')
        # (the scratch registration's corpus is the stand-in's: the one change to a copied
        # file, so that a registered run can be admitted end to end)
        rfl = w / "bench" / "run_fed_leader.py"
        text = rfl.read_text()
        self.assertEqual(
            text.count("82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"), 1
        )
        rfl.write_text(
            text.replace(
                "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915", CORPUS_SHA
            )
        )
        (w / "bench" / "wikidata.py").write_text(STUB_WIKIDATA)
        (w / "bench" / "wikidata.json").write_text(
            json.dumps({"windows_dir": "wd", "corpus_sha256": CORPUS_SHA})
        )
        (w / ".gitignore").write_text("target/\nwd/\n__pycache__/\n")
        (w / "src").mkdir()
        (w / "src" / "lib.rs").write_text("// the frozen source\n")
        (w / "bench" / "run-fed-leader.frozen").write_text("frozen none\n")
        git(w, "init", "-q")
        git(w, "config", "user.name", "Test")
        git(w, "config", "user.email", "test@example.com")
        git(w, "remote", "add", "origin", str(self.remote))
        git(w, "add", "-A")
        git(w, "commit", "-q", "-m", "the registration")
        self.frozen = git(w, "rev-parse", "HEAD")
        (w / "bench" / "run-fed-leader.frozen").write_text(f"frozen {self.frozen}\n")
        git(w, "commit", "-q", "-am", "freeze (test fixture)")
        bin_ = self.tmp / "bin"
        bin_.mkdir()
        ex = self.tmp / "cargo-target" / "release" / "examples"
        ex.mkdir(parents=True)
        for name, body, where in (
            ("ab", FAKE_AB, ex),
            ("windows", "#!/bin/sh\n", ex),
            ("occupancy", FAKE_OCC, ex),
            ("cargo", FAKE_CARGO, bin_),
            ("rustc", FAKE_RUSTC, bin_),
            ("lscpu", FAKE_LSCPU, bin_),
            ("gh", FAKE_GH, bin_),
        ):
            p = where / name
            p.write_text(body)
            p.chmod(0o755)
        self.ex = ex
        self.env = {
            **os.environ,
            "PATH": f"{bin_}:{os.environ['PATH']}",
            "FAKE_BENCH": str(ROOT / "bench"),
            "FAKE_AB": str(ex / "ab"),
            "FAKE_WINDOWS": str(ex / "windows"),
            "FAKE_OCCUPANCY": str(ex / "occupancy"),
            "FAKE_COMMENTS": str(self.tmp / "comments.json"),
            "GH": str(bin_ / "gh"),
        }
        for k in [
            k
            for k in self.env
            if k.startswith(("CARGO_", "RUST")) and k not in ("CARGO_HOME", "RUSTUP_HOME")
        ]:
            del self.env[k]

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def bundles(self):
        return sorted((self.work / "target" / "run-fed-leader").glob("*.tar.gz"))

    def read(self, tarball):
        stamp = tarball.name[: -len(".tar.gz")]
        with tarfile.open(tarball) as t:
            names = {n[len(stamp) + 1 :] for n in t.getnames() if n.startswith(stamp + "/")}

            def get(n):
                return t.extractfile(f"{stamp}/{n}").read().decode() if n in names else None

            files = {
                n: get(n)
                for n in (
                    "outcome.txt",
                    "identity.json",
                    "preflight.json",
                    "inputs.txt",
                    "commits.txt",
                    "diagnostics/diagnostics.json",
                )
            }
            bins = {
                n: hashlib.sha256(t.extractfile(f"{stamp}/bin/{n}").read()).hexdigest()
                for n in ("ab", "occupancy")
                if f"bin/{n}" in names
            }
        return names, files, bins

    def run_recipe(self, dry="", corpus="", **fake):
        time.sleep(1.1)  # (an attempt's directory is named to the second)
        before = set(self.bundles())
        script = self.tmp / "recipe.sh"
        script.write_text(recipe_body(dry, corpus))
        env = {**self.env, **{f"FAKE_{k.upper()}": v for k, v in fake.items()}}
        r = subprocess.run(
            ["bash", str(script)], cwd=self.work, env=env, capture_output=True, text=True
        )
        [new] = set(self.bundles()) - before
        names, files, bins = self.read(new)
        return r, names, files, bins

    def assertExit(self, r, code, files, start):
        self.assertEqual(r.returncode, code, r.stdout[-3000:] + r.stderr[-3000:])
        self.assertTrue(files["outcome.txt"].startswith(start), files["outcome.txt"])

    def test_valid(self):
        r, names, files, bins = self.run_recipe()
        self.assertExit(
            r, 0, files, "VALID (JUDGED (workstation)): 20 rounds at 10 threads, 140 timed records"
        )
        for n in (
            "console.log",
            "outcome.txt",
            "commits.txt",
            "inputs.txt",
            "identity.json",
            "env.json",
            "lscpu.txt",
            "runs.jsonl",
            "admission.json",
            "admission.txt",
            "preflight.json",
            "preflight.txt",
            "preflight/env.json",
            "preflight/lscpu.txt",
            "preflight/sample.jsonl",
            "preflight/exits.txt",
            "diagnostics/sample-after.jsonl",
            "diagnostics/diagnostics.json",
            "diagnostics/diagnostics.md",
            "occupancy-ctl.json",
            "occupancy-dir.json",
            "occupancy-ctl-admission.json",
            "occupancy-dir-admission.json",
            "bin/ab",
            "bin/occupancy",
            "bin/windows",
        ):
            self.assertIn(n, names)
        self.assertIn("ctl capture: VALID", files["outcome.txt"])
        self.assertIn("dir capture: VALID", files["outcome.txt"])
        self.assertIn("diagnostics complete", files["outcome.txt"])
        ident = json.loads(files["identity.json"])
        self.assertEqual(ident["schema"], "run_fed_leader/workstation-identity/2")
        self.assertEqual(ident["frozen"], self.frozen)
        self.assertEqual(ident["head"], git(self.work, "rev-parse", "HEAD"))
        self.assertEqual(ident["preflight"], "PASS")
        self.assertTrue(ident["sysfs_state"])
        self.assertEqual(ident["workload"], {"out": 209700196, "hash": "a8830caee5c511a6"})
        self.assertEqual(ident["corpus_sha256_before"], CORPUS_SHA)
        # the binaries that ran are the ones Cargo reported, copied, and hashed as bundled
        self.assertEqual(ident["ab_sha256_before"], bins["ab"])
        self.assertEqual(ident["occupancy_sha256_before"], bins["occupancy"])
        self.assertEqual(bins["ab"], hashlib.sha256((self.ex / "ab").read_bytes()).hexdigest())
        self.assertIn("--sysfs-state (no --machine-state", files["inputs.txt"])
        self.assertEqual(json.loads(files["preflight.json"])["verdict"], "PASS")
        diag = json.loads(files["diagnostics/diagnostics.json"])
        self.assertEqual(sum(diag["pair_states_total"].values()), 120)

    def test_every_env_reading_is_passive_and_bounded(self):
        # (§R11: both of the recipe's `ab env` calls are the facts-only collector, and every
        # reading outside the timed run has a TERM timeout with a forced kill after it)
        body = recipe_body()
        calls = re.findall(r"^.*\"\$AB\" env\b.*$", body, re.M)
        self.assertEqual(len(calls), 2, calls)
        for c in calls:
            self.assertRegex(c, r'^\s*(\{ )?timeout --kill-after=\d+ \d+ "\$AB" env --facts > ')
        for c in re.findall(r"^.*(?:\"\$AB\" sample|\blscpu >).*$", body, re.M):
            self.assertRegex(c, r"timeout --kill-after=\d+ \d+ ", c)
        self.assertNotIn("timeout 30", body)

    def test_invalid_off_the_registered_workload(self):
        r, _, files, _ = self.run_recipe(run="wrongout")
        self.assertExit(r, 1, files, "INVALID")
        self.assertIn("output differs from the incumbent's", r.stdout)
        self.assertIn("did not decode the registered workload", r.stdout)

    def test_a_crashing_harness_is_invalid_and_its_records_kept(self):
        r, names, files, _ = self.run_recipe(run="crash")
        self.assertExit(r, 1, files, "INVALID")
        self.assertIn("runs.jsonl", names)
        self.assertIn("ab run exited 1", r.stdout)

    def test_the_preflight_refuses_before_anything_is_timed(self):
        for host, why in (
            ("other", "not the registered i9-10900K"),
            ("governor", "governor 'performance', not the declared"),
        ):
            with self.subTest(host=host):
                r, names, files, _ = self.run_recipe(host=host)
                self.assertExit(r, 1, files, "REFUSED (preflight, before any timed run): REFUSED")
                self.assertIn(why, files["outcome.txt"])
                self.assertEqual(json.loads(files["preflight.json"])["verdict"], "REFUSED")
                for n in (
                    "preflight/env.json",
                    "preflight/lscpu.txt",
                    "preflight/sample.jsonl",
                    "bin/ab",
                ):
                    self.assertIn(n, names)
                self.assertNotIn("runs.jsonl", names)
                self.assertNotIn("identity.json", names)

    def test_a_dry_check_goes_on_past_the_preflight_and_is_never_judged(self):
        (self.work / "bench" / "run-fed-leader.frozen").write_text("frozen none\n")
        git(self.work, "commit", "-q", "-am", "unfreeze")
        corpus = self.tmp / "c.bz2"
        corpus.write_bytes(b"another corpus")
        r, names, files, _ = self.run_recipe(dry="1", corpus=str(corpus), host="other")
        self.assertExit(r, 0, files, "VALID (DRY (not evidence))")
        self.assertIn("DRY-RUN.txt", names)
        self.assertEqual(json.loads(files["identity.json"])["preflight"], "REFUSED")
        self.assertNotIn("JUDGED", files["outcome.txt"])

    def test_refusals(self):
        r, _, files, _ = self.run_recipe(corpus="/nowhere.bz2")
        self.assertExit(r, 1, files, "REFUSED: a CORPUS is for a DRY check only")
        r, _, files, _ = self.run_recipe(build="fail")
        self.assertExit(r, 1, files, "REFUSED: the build failed")
        env_override = {**self.env, "CARGO_TARGET_DIR": str(self.tmp / "elsewhere")}
        saved, self.env = self.env, env_override
        r, _, files, _ = self.run_recipe()
        self.env = saved
        self.assertExit(r, 1, files, "REFUSED: unset these")
        (self.work / "src" / "lib.rs").write_text("// edited\n")
        r, _, files, _ = self.run_recipe()
        self.assertExit(r, 1, files, "REFUSED: the checkout has uncommitted changes")
        git(self.work, "commit", "-q", "-am", "a source change after the freeze")
        r, _, files, _ = self.run_recipe()
        self.assertExit(
            r, 1, files, "REFUSED: HEAD's measured inputs differ from the frozen commit"
        )
        (self.work / "bench" / "run-fed-leader.frozen").write_text("frozen none\n")
        git(self.work, "commit", "-q", "-am", "unfreeze")
        r, _, files, _ = self.run_recipe()
        self.assertExit(r, 1, files, "REFUSED: bench/run-fed-leader.frozen names no frozen commit")

    def test_a_corpus_off_its_digest_is_refused(self):
        r, _, files, _ = self.run_recipe(corpus_mode="other")
        self.assertExit(r, 1, files, "REFUSED: the corpus built is not bench/wikidata.json's")

    def test_a_step_failing_outside_the_run_is_incomplete(self):
        r, _, files, _ = self.run_recipe(corpus_mode="fail")
        self.assertExit(r, 3, files, "INCOMPLETE")

    @unittest.skipUnless(JUST, "just is not installed")
    def test_submit_keeps_the_exit_and_archives_every_outcome(self):
        for fake, code in (({"host": "other"}, 1), ({}, 0)):
            with self.subTest(fake=fake):
                time.sleep(1.1)
                env = {**self.env, **{f"FAKE_{k.upper()}": v for k, v in fake.items()}}
                r = subprocess.run(
                    [JUST, "submit", RECIPE], cwd=self.work, env=env, capture_output=True, text=True
                )
                self.assertEqual(r.returncode, code, r.stdout[-3000:] + r.stderr[-3000:])
                self.assertIn("archived", r.stdout + r.stderr)
        tree = git(self.remote, "ls-tree", "-r", "--name-only", "evidence").split()
        self.assertEqual(len([t for t in tree if t.endswith(".tar.gz")]), 2)
        bodies = [c["body"] for c in json.loads((self.tmp / "comments.json").read_text())]
        self.assertTrue(any("REFUSED (preflight" in b for b in bodies))
        self.assertTrue(any("VALID (JUDGED" in b for b in bodies))


if __name__ == "__main__":
    unittest.main(verbosity=2)
