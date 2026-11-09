"""bench/service_guarded.py's verdict on synthetic i9-shaped attempts: a clean attempt is valid
and read as registered, and each defect the rule guards against is refused with its kind of
failure. Among them, the Researcher's admission witnesses (REVIEW 6742157313, probe.py at
b365250e), each of which the draft at 3c9df670 accepted: an output length of 1, a non-hex
digest, a preflight of only {"ok": true}, ok=true beside an unknown quota and no
observations, a row with exit 17 and an empty err, round 0 as false, a base path outside the
attempt's registered arms. And the Researcher's termination witness (REVIEW 6746023579):
every row must carry ab's `termination` as "success"; the producer itself, `ab run`, is driven
with stand-in children that write a valid-looking record and then exit 0, exit 17, are killed,
or write nothing, and its records are judged (`Producer`, given a built `ab`). The raw
discovery diagnostics are read from fake trees.

    python3 bench/test_service_guarded.py

`Producer` needs ab's executable: SG_AB names it (CI builds it: `cargo build --example ab`),
else target/debug/examples/ab if built; it skips without one unless SG_REQUIRE_AB=1.
"""

import json
import os
import random
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import service_guarded as sg  # noqa: E402
from svc_target_fixtures import edit, record  # noqa: E402

W = "/w/target/service-guarded/20261110T000000Z"
BASE = f"mibz2@{W}/bin/ab-base"
TWIN = f"mibz2@{W}/bin/ab-base-twin"
CAND, BASESHA = "c" * 40, "b" * 40
FETCHED = "2026-11-10T00:00:00Z"
REG = {"corpus_sha256": "a" * 64, "decoded_bytes": 209700196,
       "frozen": {"candidate": CAND, "base": BASESHA, "base_fetched": FETCHED}}
HASH = "0123456789abcdef"


def names(plan, busy):
    leaders, plain, piped, svc = plan
    n = [f"mibz2-{i}" for i in range(leaders)]
    n += [f"mibz2-coop-{i + 1}" for i in range(plain)]
    n += [f"mibz2-coop-p{i + 1}" for i in range(piped)]
    n += [f"mibz2-svc-{i}" for i in range(svc)]
    if busy:
        return " ".join(x + "=0.1" for x in n + ["mibz2-scan"])
    extra = ["ab", "mibz2-read", "mibz2-scan", "sampler"] + (["mibz2-svc-feed"] if svc else [])
    return " ".join(sorted(n + extra))


def rows(case=None, speed=1.05, cpu=1.13, twin=1.0, seed=3, harm=1.0):
    rng = random.Random(seed)
    out = []
    for rnd in range(20):
        for t in sg.THREADS:
            for b, arm in (("base", BASE), ("head", "mibz2"), ("twin", TWIN)):
                if case == "missing round" and (b, t, rnd) == ("head", 10, 5):
                    continue
                plan = sg.EXPECT[t][b]
                if case == "head not svc2" and (b, t, rnd) == ("head", 10, 2):
                    plan = sg.PIPED_3X3
                if case == "svc2 at 11" and (b, t, rnd) == ("head", 11, 2):
                    plan = sg.SVC2
                wall = 0.14 * rng.gauss(1, 0.01)
                c = 1.0 * rng.gauss(1, 0.01)
                if b == "head" and t == 10:
                    wall /= speed
                    c /= cpu
                if b == "twin":
                    wall /= twin
                if b == "head" and t in sg.NO_HARM:
                    wall *= harm
                out.append({
                    "round": rnd, "corpus": "wikidata", "threads": t, "arm": arm,
                    "wall": wall, "cpu": c,
                    "rss_kb": 70000 * (0.88 if b == "head" and t == 10 else 1.0),
                    "out": 209700196,
                    "hash": "fedcba9876543210" if case == "wrong bytes" and (b, t, rnd) == ("head", 10, 1) else HASH,
                    "thread_cpu": names(plan, True), "observed_threads": names(plan, False),
                    "err": "boom" if case == "failed run" and (b, t, rnd) == ("twin", 9, 0) else "",
                    "termination": "success",
                })
    return out


def files(**over):
    rec, status = record(True)
    f = {
        "commits.txt": f"head {CAND}\ncandidate {CAND}\nbase {BASESHA}\nbase_fetched {FETCHED}\n",
        "arms.txt": f"mibz2\n{BASE}\n{TWIN}\n",
        "inputs.txt": f"{'1' * 64}  {W}/bin/ab-head\n{'2' * 64}  {W}/bin/ab-base\n{'2' * 64}  {W}/bin/ab-base-twin\n",
        "corpus.sha256": f"{'a' * 64}  /w/target/service-guarded/wikidata.bz2\n",
        "preflight.json": json.dumps(rec),
        "preflight.exit": f"{status}\n",
    }
    f.update(over)
    return {k: v for k, v in f.items() if v is not None}


def judge(f=None, r=None, unreadable=()):
    return sg.judge(files() if f is None else f, rows() if r is None else r, unreadable, REG)


class Judge(unittest.TestCase):
    def assertRefused(self, v, kind, what):
        self.assertFalse(v["valid"], what)
        self.assertIsNone(v["p1"], what)
        self.assertEqual(v["failure"], kind, (what, v["problems"]))

    def test_clean_attempt_reads_as_registered(self):
        v = judge()
        self.assertTrue(v["valid"], v["problems"])
        self.assertEqual(v["preflight"]["verdict"], "TARGET")
        self.assertEqual(v["no_harm"], {t: True for t in (11, 12, 16, 20)})
        self.assertEqual(sg.THREADS, (8, 9, 10, 11, 12, 16, 20))
        self.assertEqual((v["p1"]["speed"], v["p1"]["cpu"]), ("GAIN", "CPU SAVING"))
        self.assertEqual(v["p1"]["reading"], "THE SERVICE DEFAULT BEATS THE CURRENT DEFAULT")
        self.assertLess(v["cells"][10]["rss"]["ratio"], 1)
        md = sg.markdown(v)
        for words in ("not a simultaneous", "does not identify the service threads",
                      "no memory condition was registered", "not binary identity"):
            self.assertIn(words, md)

    def test_readings_and_their_qualifications(self):
        v = judge(r=rows(speed=1.0, cpu=1.0))
        self.assertEqual(v["p1"]["reading"], "NOT REPLICATED")
        self.assertIn("a small effect is not excluded", sg.markdown(v))
        self.assertEqual(judge(r=rows(speed=1.06, cpu=0.9))["p1"]["reading"], "A TRADE")
        self.assertEqual(judge(r=rows(speed=0.9, cpu=1.0))["p1"]["speed"], "REGRESSION")
        # a speed gain with CPU inconclusive keeps the reading, and says no cost was detected,
        # not that none exists
        v = judge(r=rows(speed=1.06, cpu=1.0, seed=11))
        if v["p1"]["cpu"] == "INCONCLUSIVE":
            self.assertIn("not shown to cost nothing", sg.markdown(v))

    def test_outcome_map_boundaries(self):
        o = sg.outcome
        self.assertEqual(o({"lo": 1.021, "hi": 1.1}), "GAIN")
        self.assertEqual(o({"lo": 1.02, "hi": 1.1}), "SMALL GAIN")
        self.assertEqual(o({"lo": 0.97, "hi": 1.03}), "NULL")
        self.assertEqual(o({"lo": 0.96, "hi": 1.03}), "INCONCLUSIVE")
        self.assertEqual(o({"lo": 0.9, "hi": 0.99}), "REGRESSION")
        self.assertEqual(o({"lo": 0.9, "hi": 0.99}, cpu=True), "CPU COST")

    def test_the_researchers_witnesses(self):
        rec, _ = record(True)
        cases = {
            "every output size 1": ("structural", None, [dict(r, out=1) for r in rows()]),
            "non-hex digests": ("structural", None, [dict(r, hash="not-a-digest") for r in rows()]),
            "preflight only ok": ("preflight", files(**{"preflight.json": json.dumps({"ok": True})}), None),
            "ok beside unknown quota, no observations": ("preflight", files(**{"preflight.json": json.dumps(
                edit(rec, lambda r: (r["machine"].__setitem__("quota", {"kind": "unknown"}),
                                     r["machine"].__setitem__("target", None), r.__setitem__("at", []))))}), None),
            "exit 17 with an empty err": ("structural", None, [dict(r, exit=17) if i == 0 else r for i, r in enumerate(rows())]),
            "round zero as false": ("structural", None, [dict(r, round=False) if r["round"] == 0 and i == 0 else r for i, r in enumerate(rows())]),
            "an unregistered base path": ("structural", None, [dict(r, arm="mibz2@/unregistered/bin/ab-base") if r["arm"] == BASE else r for r in rows()]),
        }
        for what, (kind, f, r) in cases.items():
            with self.subTest(what):
                self.assertRefused(judge(f, r), kind, what)
        # the positive control the review kept: one output digest changed
        self.assertRefused(judge(r=rows("wrong bytes")), "structural", "one digest changed")

    def test_the_producers_failed_child(self):
        # ab's own encodings: a child that printed an error, and one that printed nothing
        # (`"broken":1`, no figures)
        r = rows()
        failed = dict(r[0], err="decode failed: BlockCrc")
        crashed = {k: r[1][k] for k in ("round", "corpus", "threads", "arm", "observed_threads")}
        crashed["broken"] = 1
        for what, bad in (("err", failed), ("broken", crashed)):
            with self.subTest(what):
                self.assertRefused(judge(r=[bad] + r[1:]), "structural", what)
        self.assertRefused(judge(r=[{k: v for k, v in r[0].items() if k != "err"}] + r[1:]), "structural", "no err")

    def test_every_row_must_terminate_successfully(self):
        # (the Researcher's termination witness, REVIEW 6746023579: a valid-looking record
        # written before a nonzero exit or a signal; ab's records carry `termination`, no `exit`)
        for what, val in (("missing", None), ("exit 17", "exit 17"), ("signal 9", "signal 9"),
                          ("signal 15", "signal 15"), ("unknown", "unknown"), ("exit 0 spelled out", "exit 0"),
                          ("not a string", 0)):
            with self.subTest(what):
                r = rows()
                r[5] = {k: v for k, v in r[5].items() if k != "termination"}
                if val is not None:
                    r[5]["termination"] = val
                v = judge(r=r)
                self.assertRefused(v, "structural", what)
                self.assertTrue(any("termination" in p for p in v["problems"]), v["problems"])
        self.assertTrue(judge()["valid"])

    def test_identity(self):
        cases = {
            "not frozen": sg.judge(files(), rows(), (), dict(REG, frozen={})),
            "another candidate": judge(files(**{"commits.txt": f"candidate {'d' * 40}\nbase {BASESHA}\nbase_fetched {FETCHED}\n"})),
            "another base": judge(files(**{"commits.txt": f"candidate {CAND}\nbase {'d' * 40}\nbase_fetched {FETCHED}\n"})),
            # (base currency: the time master was fetched at the freeze, beside its SHA)
            "no fetch time frozen": sg.judge(files(), rows(), (), dict(REG, frozen={"candidate": CAND, "base": BASESHA})),
            "no fetch time recorded": judge(files(**{"commits.txt": f"candidate {CAND}\nbase {BASESHA}\n"})),
            "another fetch time": judge(files(**{"commits.txt": f"candidate {CAND}\nbase {BASESHA}\nbase_fetched 2026-11-11T00:00:00Z\n"})),
            "a dry run": judge(files(**{"DRY-RUN.txt": "DRY RUN"})),
            "arms missing": judge(files(**{"arms.txt": None})),
            "a fourth arm": judge(files(**{"arms.txt": f"mibz2\n{BASE}\n{TWIN}\nmibz2@/x\n"})),
            "twin not the base's bytes": judge(files(**{"inputs.txt": f"{'2' * 64}  {W}/bin/ab-base\n{'3' * 64}  {W}/bin/ab-base-twin\n"})),
            "no digests": judge(files(**{"inputs.txt": None})),
        }
        for what, v in cases.items():
            with self.subTest(what):
                self.assertRefused(v, "identity", what)

    def test_workload(self):
        for what, f in (("another corpus", files(**{"corpus.sha256": f"{'e' * 64}  x\n"})),
                        ("no corpus digest", files(**{"corpus.sha256": None}))):
            with self.subTest(what):
                self.assertRefused(judge(f), "workload", what)
        self.assertRefused(judge(r=[dict(r, corpus="other") for r in rows()]), "structural", "rows' corpus")

    def test_preflight(self):
        n, nx = record(False)
        t, _ = record(True)
        for what, f in (("not target", files(**{"preflight.json": json.dumps(n), "preflight.exit": str(nx)})),
                        ("exit disagrees", files(**{"preflight.exit": "3"})),
                        ("no exit", files(**{"preflight.exit": None})),
                        ("no record", files(**{"preflight.json": None})),
                        ("ok, unknown quota", files(**{"preflight.json": json.dumps(
                            edit(t, lambda r: r["machine"].__setitem__("quota", {"kind": "unknown"})))}))):
            with self.subTest(what):
                self.assertRefused(judge(f), "preflight", what)

    def test_structural(self):
        for case in ("missing round", "head not svc2", "svc2 at 11", "wrong bytes", "failed run"):
            with self.subTest(case):
                self.assertRefused(judge(r=rows(case)), "structural", case)
        self.assertRefused(judge(unreadable=(7,)), "structural", "an unreadable line")
        r = rows()
        self.assertRefused(judge(r=r + [dict(r[0])]), "structural", "a duplicate round")
        self.assertRefused(judge(r=r + [dict(r[0], threads=13)]), "structural", "13 threads")

    def test_no_harm_is_judged_beside_p1(self):
        v = judge(r=rows(harm=1.04))
        self.assertTrue(v["valid"], v["problems"])
        self.assertEqual(set(v["no_harm"].values()), {False})
        self.assertIn("not shown at 11, 12, 16, 20", sg.markdown(v))
        # the precedent's quantile: one-sided, Bonferroni over four, at 19 degrees of freedom
        self.assertEqual(sg.T_9875, 2.4334)

    def test_n1_is_its_own_failure(self):
        v = judge(r=rows(twin=1.05))
        self.assertRefused(v, "n1", "N1 out of band")
        self.assertIn("N1", v["problems"][0])

    def test_plan_names(self):
        self.assertEqual(sg.plan_of(names(sg.SVC2, False)), sg.SVC2)
        self.assertEqual(sg.plan_of(names(sg.PIPED_3X3, True)), sg.PIPED_3X3)
        self.assertEqual(sg.plan_of("mibz2-svc-feed mibz2-read"), (0, 0, 0, 0))

    def test_registered_values_are_the_manifests(self):
        reg = sg.registered()
        manifest = json.loads((HERE / "wikidata.json").read_text())
        self.assertEqual(reg["decoded_bytes"], manifest["decoded_bytes"])
        self.assertEqual(reg["corpus_sha256"], manifest["corpus_sha256"])


def ab_exe():
    exe = os.environ.get("SG_AB") or str(HERE.parent / "target" / "debug" / "examples" / "ab")
    return exe if os.access(exe, os.X_OK) else None


class Producer(unittest.TestCase):
    """`ab run` itself, the producer, with stand-in children (ARM@EXE) that each write the clean
    fixture's head row at 10 threads, round 0 (the child's own fields), and then exit 0, exit
    17, are killed by SIGKILL, or write nothing at all. Each record ab writes takes that row's
    place in the clean attempt, with only the arm and the observed thread names (a shell's,
    here) put back as the fixture's; everything else is ab's, `termination` included. Success
    is VALID; every other ending is INVALID (structural), named by its termination"""

    def setUp(self):
        self.ab = ab_exe()
        if not self.ab:
            if os.environ.get("SG_REQUIRE_AB") == "1":
                self.fail("SG_REQUIRE_AB=1 and no ab executable (SG_AB, or target/debug/examples/ab)")
            self.skipTest("no ab executable: build it with `cargo build --example ab`, or set SG_AB")
        self.tmp = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_the_producer_records_each_ending_and_the_judge_refuses_all_but_success(self):
        clean = rows()
        at = next(i for i, r in enumerate(clean) if (r["arm"], r["threads"], r["round"]) == ("mibz2", 10, 0))
        theirs = ("round", "corpus", "threads", "arm", "observed_threads")
        child = json.dumps({k: v for k, v in clean[at].items() if k not in theirs + ("termination",)})
        arms = []
        for name, tail in (("ok", "exit 0"), ("seventeen", "exit 17"), ("killed", "kill -KILL $$"), ("empty", None)):
            p = self.tmp / name
            body = "exit 0\n" if tail is None else f"printf '%s\\n' '{child}'\n{tail}\n"
            p.write_text("#!/bin/sh\n" + body)
            p.chmod(0o755)
            arms.append(f"{name}@{p}")
        corpus = self.tmp / "wikidata.bz2"
        corpus.write_bytes(b"BZh9")
        out = self.tmp / "runs.jsonl"
        r = subprocess.run([self.ab, "run", str(out), "--arms", ",".join(arms), "--threads", "10", "--rounds", "1",
                            "--max-busy", "64", "--settle", "0.2", str(corpus)],
                           capture_output=True, text=True, timeout=300)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        recs = {json.loads(l)["arm"].split("@")[0]: json.loads(l) for l in out.read_text().splitlines() if l.strip()}
        self.assertEqual(set(recs), {"ok", "seventeen", "killed", "empty"})
        want = {"ok": "success", "seventeen": "exit 17", "killed": "signal 9", "empty": "success"}
        for name, term in want.items():
            with self.subTest(name):
                rec = recs[name]
                self.assertEqual(rec["termination"], term)
                self.assertNotIn("exit", rec)
                self.assertEqual((rec["round"], rec["threads"], rec["corpus"]), (0, 10, "wikidata"))
                got = dict(rec, arm="mibz2", observed_threads=clean[at]["observed_threads"])
                v = judge(r=clean[:at] + [got] + clean[at + 1:])
                if name == "ok":
                    self.assertTrue(v["valid"], v["problems"])
                    continue
                self.assertFalse(v["valid"], name)
                self.assertEqual(v["failure"], "structural", v["problems"])
                if name == "empty":
                    self.assertTrue(any("broken" in p for p in v["problems"]), v["problems"])
                else:
                    self.assertTrue(any(f"termination '{term}'" in p for p in v["problems"]), v["problems"])


class Discovery(unittest.TestCase):
    """The raw discovery diagnostics on fake trees: each read's contents, absence or error
    archived as such, and no quota verdict"""

    def tree(self, membership, mountinfo, files):
        root = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, root, True)
        (root / "proc" / "self").mkdir(parents=True)
        (root / "proc" / "self" / "cgroup").write_text(membership)
        (root / "proc" / "self" / "mountinfo").write_text(mountinfo)
        for rel, text in files.items():
            p = root / rel.lstrip("/")
            p.parent.mkdir(parents=True, exist_ok=True)
            if text is None:
                p.mkdir()  # (unreadable as a file: a read error)
            else:
                p.write_text(text)
        return str(root)

    def test_v2_every_level_with_absence_and_errors(self):
        root = self.tree("0::/a/b\n", "30 1 0:26 / /sys/fs/cgroup rw - cgroup2 cgroup2 rw\n"
                         "31 1 0:27 / /proc rw - proc proc rw\n",
                         {"/sys/fs/cgroup/a/cpu.max": "800000 100000\n", "/sys/fs/cgroup/a/b/cpu.max": None})
        d = sg.discovery(root)
        keys = set()

        def walk(x):
            if isinstance(x, dict):
                keys.update(x)
                for v in x.values():
                    walk(v)
            elif isinstance(x, list):
                for v in x:
                    walk(v)
        walk(d)
        self.assertFalse(keys & {"verdict", "quota", "kind", "target", "cpus"}, keys)
        [h] = d["hierarchies"]
        self.assertEqual(h["membership"], "0::/a/b")
        self.assertEqual(h["top_cgroup_type"]["state"], "absent")
        self.assertEqual([lv["dir"] for lv in h["levels"]], ["/sys/fs/cgroup", "/sys/fs/cgroup/a", "/sys/fs/cgroup/a/b"])
        states = [lv["files"][0]["state"] for lv in h["levels"]]
        self.assertEqual(states, ["absent", "read", "error"])
        self.assertEqual(h["levels"][1]["files"][0]["value"], "800000 100000\n")

    def test_v1_cpu_and_a_path_outside_the_mount(self):
        root = self.tree("4:cpu,cpuacct:/x\n", "40 1 0:30 /y /sys/fs/cgroup/cpu rw - cgroup cgroup rw,cpu,cpuacct\n", {})
        [h] = sg.discovery(root)["hierarchies"]
        self.assertEqual(h["levels"], [])
        self.assertIn("not under the mount's root", h["notes"][0])

    def test_unreadable_membership_is_archived(self):
        root = self.tree("", "", {})
        os.remove(os.path.join(root, "proc", "self", "cgroup"))
        d = sg.discovery(root)
        self.assertEqual(d["membership"]["state"], "absent")
        self.assertEqual(d["hierarchies"], [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
