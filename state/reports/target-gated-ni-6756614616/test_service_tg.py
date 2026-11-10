"""bench/service_tg.py on synthetic attempts and CI artifacts: the target-gated registration's
judge (docs/journal/2026-11-10-service-guarded-target-gated-registration.md).

- The workstation A/B: a clean attempt meets conditions 3 and 4; each condition fails on its
  own (a gain too small at 10, a CPU cost at 10, NOT SHOWN at 9 or 11, a CPU or RSS
  regression); each admission defect is refused with its kind (identity, the toolchain
  included; workload; preflight, a schema 1 record included; structural: termination, plans,
  rounds, a third arm).
- CI's condition 2, from the sharded workflow's artifacts: the nine shards admitted only as
  registered (one measured attempt each, the manifest's digests, label, head, base, run,
  rounds, toolchain, runner topology and model, the predicate read back NOT TARGET, corpus
  digests, completeness, termination, placement), each cell's batch-means one-sided bound,
  the all-pass rule, wall flags and the harm-confirmation rule, peak RSS.
- Student's t quantiles against fixed values (and scipy's, where it is installed).

    python3 bench/test_service_tg.py
"""

import copy
import json
import math
import random
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import service_tg as tg  # noqa: E402
from svc_target_fixtures import edit, record  # noqa: E402

W = "/w/target/service-tg/20261110T000000Z"
BASE = f"mibz2@{W}/bin/ab-base"
CAND, BASESHA, HEAD = "c" * 40, "b" * 40, "f" * 40
RUSTC = "rustc 1.98.0 (88d9e12ae 2026-08-18)"
FETCHED = "2026-11-10T00:00:00Z"
REG = {"corpus_sha256": "a" * 64, "decoded_bytes": 209700196, "bench": "3.5.0",
       "frozen": {"candidate": CAND, "base": BASESHA, "base_fetched": FETCHED, "rustc": RUSTC,
                  "ni_rustc": "rustc 1.99.1 (58cf8f9c3 2026-10-13)", "corpus_text": "7" * 64}}
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


def rows(speed=1.068, cpu=0.85, wall={}, cpus={}, rss={}, seed=3, sd=0.01, case=None):
    """Synthetic rows: the head's wall at 10 divided by `speed`, its CPU at 10 times `cpu`;
    `wall`, `cpus` and `rss` multiply the head's figure at the budgets they name"""
    rng = random.Random(seed)
    out = []
    for rnd in range(tg.ROUNDS):
        for t in tg.THREADS:
            for b, arm in (("base", BASE), ("head", "mibz2")):
                if case == "missing round" and (b, t, rnd) == ("head", 10, 5):
                    continue
                plan = tg.EXPECT[t][b]
                if case == "svc2 at 11" and (b, t, rnd) == ("head", 11, 2):
                    plan = tg.SVC2
                if case == "not svc2 at 10" and (b, t, rnd) == ("head", 10, 2):
                    plan = tg.PIPED_3X3
                w = 0.14 * rng.gauss(1, sd)
                c = 1.0 * rng.gauss(1, sd)
                r = 70000 * rng.gauss(1, 0.002)
                if b == "head":
                    if t == 10:
                        w /= speed
                        c *= cpu
                    w *= wall.get(t, 1.0)
                    c *= cpus.get(t, 1.0)
                    r *= rss.get(t, 1.0)
                out.append({
                    "round": rnd, "corpus": "wikidata", "threads": t, "arm": arm,
                    "wall": w, "cpu": c, "rss_kb": r, "out": 209700196, "hash": HASH,
                    "thread_cpu": names(plan, True), "observed_threads": names(plan, False),
                    "termination": "success", "err": "",
                })
    return out


def files(**over):
    rec, ex = record(True)
    f = {
        "commits.txt": f"head {CAND}\ncandidate {CAND}\nbase {BASESHA}\nbase_fetched {FETCHED}\n",
        "arms.txt": f"mibz2\n{BASE}\n",
        "inputs.txt": f"{'d' * 64}  {W}/bin/ab-head\n{'e' * 64}  {W}/bin/ab-base\n{RUSTC}\nbinary: rustc\n",
        "corpus.sha256": "a" * 64 + "  /w/wikidata.bz2\n",
        "preflight.json": json.dumps(rec),
        "preflight.exit": f"{ex}\n",
    }
    f.update(over)
    return {k: v for k, v in f.items() if v is not None}


class TStats(unittest.TestCase):
    def test_quantiles(self):
        for q, df, want in ((0.9875, 127, 2.268287), (0.975, 127, 1.978820), (0.95, 9, 1.833113),
                            (0.975, 9, 2.262157), (0.975, 19, 2.093024), (0.9875, 19, 2.433440),
                            (0.95, 159, 1.654494), (0.95, 1, 6.313752), (0.975, 2, 4.302653)):
            self.assertAlmostEqual(tg.t_ppf(q, df), want, places=5, msg=(q, df))
        try:
            from scipy import stats
        except ImportError:
            return
        for df in (3, 9, 19, 63, 127, 399, 599):
            for q in (0.95, 0.975, 0.9875, 0.99167):
                self.assertAlmostEqual(tg.t_ppf(q, df), stats.t.ppf(q, df), places=6, msg=(q, df))


class Workstation(unittest.TestCase):
    def judge(self, f=None, r=None):
        return tg.judge(files() if f is None else f, rows() if r is None else r, reg=REG)

    def test_a_clean_attempt_meets_the_workstation_conditions(self):
        v = self.judge()
        self.assertTrue(v["valid"], v["problems"])
        self.assertTrue(v["qualifies"], v["reasons"])
        self.assertEqual(v["target"], {"wall": True, "cpu": True})
        self.assertEqual(v["fallback"], {9: True, 11: True})
        c = v["cells"][10]
        self.assertAlmostEqual(c["speed_k"]["conf"], 0.975)
        self.assertAlmostEqual(c["cpu_upper_k"]["conf"], 0.975)
        self.assertAlmostEqual(v["cells"][9]["wall_upper_m"]["conf"], 0.975)
        md = tg.markdown(v)
        self.assertIn("**met**", md)

    def test_the_bounds_are_the_registered_ones(self):
        # (the target's two-sided 97.5% interval and the fallback's one-sided 97.5% bound,
        # recomputed by hand from the rows)
        r = rows()
        v = tg.judge(files(), r, reg=REG)
        by = {(x["arm"], x["threads"], x["round"]): x for x in r}
        logs = [math.log(by[(BASE, 10, i)]["wall"] / by[("mibz2", 10, i)]["wall"]) for i in range(tg.ROUNDS)]
        m = sum(logs) / len(logs)
        sd = math.sqrt(sum((x - m) ** 2 for x in logs) / (len(logs) - 1))
        lo = math.exp(m - 2.268287 * sd / math.sqrt(tg.ROUNDS))
        self.assertAlmostEqual(v["cells"][10]["speed_k"]["lo"], lo, places=5)
        logs = [math.log(by[("mibz2", 11, i)]["wall"] / by[(BASE, 11, i)]["wall"]) for i in range(tg.ROUNDS)]
        m = sum(logs) / len(logs)
        sd = math.sqrt(sum((x - m) ** 2 for x in logs) / (len(logs) - 1))
        up = math.exp(m + 1.978820 * sd / math.sqrt(tg.ROUNDS))
        self.assertAlmostEqual(v["cells"][11]["wall_upper_m"]["upper"], up, places=5)

    def test_each_condition_fails_on_its_own(self):
        cases = {
            "a gain of 1.015 at 10": (rows(speed=1.015), "condition 3: wall speed"),
            "a CPU cost at 10": (rows(cpu=1.05), "condition 3: CPU"),
            "NOT SHOWN at 9": (rows(wall={9: 1.03}), "condition 4: NOT SHOWN at 9"),
            "NOT SHOWN at 11 (noise, no effect)": (rows(sd=0.12, speed=1.5, cpu=0.5), "condition 4: NOT SHOWN"),
            "a CPU regression at 11": (rows(cpus={11: 1.05}), "condition 7: CPU regression demonstrated at 11"),
            "an RSS regression at 10": (rows(rss={10: 1.2}), "condition 7: peak RSS regression demonstrated at 10"),
        }
        for what, (r, reason) in cases.items():
            with self.subTest(what):
                v = self.judge(r=r)
                self.assertTrue(v["valid"], v["problems"])
                self.assertFalse(v["qualifies"])
                self.assertTrue(any(reason in x for x in v["reasons"]), v["reasons"])
        # (a 1.5% CPU rise at 9 is reported, not a demonstrated regression past 1.02)
        v = self.judge(r=rows(cpus={9: 1.015}))
        self.assertTrue(v["qualifies"], v["reasons"])

    def test_admission(self):
        rec, _ = record(True)
        cases = {
            "identity": [
                files(**{"DRY-RUN.txt": "dry"}),
                files(**{"commits.txt": f"candidate {'0' * 40}\nbase {BASESHA}\nbase_fetched {FETCHED}\n"}),
                files(**{"commits.txt": f"candidate {CAND}\nbase {BASESHA}\n"}),
                files(**{"inputs.txt": f"{'e' * 64}  {W}/bin/ab-base\nrustc 1.99.0 (other)\n"}),
                files(**{"arms.txt": f"mibz2\n{BASE}\n{BASE}-twin\n"}),
                files(**{"inputs.txt": f"{RUSTC}\n"}),
            ],
            "workload": [files(**{"corpus.sha256": "9" * 64})],
            "preflight": [
                files(**{"preflight.exit": "17\n"}),
                files(**{"preflight.json": json.dumps(edit(rec, lambda r: r.__setitem__(
                    "schema", "mibz2.svc_target/1")))}),
                files(**{"preflight.json": json.dumps(record(False)[0]), "preflight.exit": "3\n"}),
            ],
        }
        for kind, fs in cases.items():
            for i, f in enumerate(fs):
                with self.subTest(kind=kind, i=i):
                    v = self.judge(f=f)
                    self.assertFalse(v["valid"])
                    self.assertEqual(v["failure"], kind, v["problems"])
        reg = copy.deepcopy(REG)
        del reg["frozen"]["rustc"]
        v = tg.judge(files(), rows(), reg=reg)
        self.assertEqual(v["failure"], "identity")
        self.assertTrue(any("names no rustc" in p for p in v["problems"]), v["problems"])

    def test_structural(self):
        def mutate(f):
            r = rows()
            f(r)
            return r
        cases = {
            "a child exited 17": mutate(lambda r: r[4].__setitem__("termination", "exit 17")),
            "no termination": mutate(lambda r: r[4].pop("termination")),
            "an error": mutate(lambda r: r[4].__setitem__("err", "boom")),
            "round as a boolean": mutate(lambda r: r[0].__setitem__("round", False)),
            "a round past 127": mutate(lambda r: r[0].__setitem__("round", 128)),
            "8 threads": mutate(lambda r: r[0].__setitem__("threads", 8)),
            "another output length": mutate(lambda r: r[0].__setitem__("out", 1)),
            "another digest": mutate(lambda r: r[0].__setitem__("hash", "fedcba9876543210")),
            "a twin arm": mutate(lambda r: r[0].__setitem__("arm", BASE + "-twin")),
            "a missing round": rows(case="missing round"),
            "svc2 at 11": rows(case="svc2 at 11"),
            "not svc2 at 10": rows(case="not svc2 at 10"),
            "a NaN wall": mutate(lambda r: r[0].__setitem__("wall", float("nan"))),
        }
        for what, r in cases.items():
            with self.subTest(what):
                v = self.judge(r=r)
                self.assertFalse(v["valid"])
                self.assertEqual(v["failure"], "structural", v["problems"])


# ---------------------------------------------------------------------------------------
# CI: the sharded workflow's artifacts


R, B = 40, 4  # (small, for speed: the judge takes them as parameters; the registered 600 and 20 are constants)
RUN = 4242


def shard_rows(sid, effect={}, sd=0.003, seed=5):
    """One shard's records: the head's figure times `effect[(threads, metric)]`, placed as
    --spread places them on a runner whose cores are CPUs 0,1 and 2,3"""
    corpus, threads = tg.SHARDS[sid]
    rng = random.Random(f"{seed}{sid}")
    out = []
    for rnd in range(R):
        for t in threads:
            for arm in tg.CI_ARMS:
                w, c, r = 0.05 * rng.gauss(1, sd), 0.08 * rng.gauss(1, sd), 30000 * rng.gauss(1, 0.001)
                if arm == "mibz2":
                    w *= effect.get((t, "wall"), 1.0)
                    c *= effect.get((t, "cpu"), 1.0)
                    r *= effect.get((t, "rss"), 1.0)
                out.append({"round": rnd, "corpus": corpus, "threads": t, "arm": arm, "wall": w, "cpu": c,
                            "rss_kb": r, "out": 1000, "hash": "0123456789abcdef", "termination": "success",
                            "err": "", "cpus": {1: "1", 2: "0,2", 4: "all"}[t]})
    return out


def write_shard(root, sid, attempt=1, rows=None, label="qualification", measured=True, over=None, drop=()):
    corpus, threads = tg.SHARDS[sid]
    d = Path(root) / f"tg-ni-{sid}-a{attempt}" / sid
    d.mkdir(parents=True)
    info = {"schema": "mibz2.tg_ni_shard/1", "label": label, "shard": sid, "corpus": corpus,
            "threads": list(threads), "rounds": R, "head": HEAD, "base": BASESHA, "github_sha": HEAD,
            "workflow_ref": "lmmx/mibz2/.github/workflows/target-gated-ni.yml@refs/heads/x",
            "run_id": RUN, "run_attempt": attempt}
    info.update(over or {})
    rec, ex = record(False)
    digest = REG["frozen"]["corpus_text"] if corpus == "text" else tg.CORPUS_SHA256[corpus]
    files = {
        "shard.json": json.dumps(info),
        "rustc.txt": REG["frozen"]["ni_rustc"] + "\n",
        "binaries.sha256": "x\n",
        "lscpu.txt": "Architecture: x86_64\nModel name:                           AMD EPYC 7763 64-Core Processor\n",
        "env.json": json.dumps({"facts": "ab env --facts/1", "cpus": 4, "cores": 2, "core_cpus": ["0,1", "2,3"]}),
        "preflight.json": json.dumps(rec),
        "preflight.exit": f"{ex}\n",
        "preflight-check.json": json.dumps({"verdict": "NOT TARGET"}),
        "corpus.sha256": f"{digest}  corpora/{corpus}.bz2\n",
        "runs.jsonl": "".join(json.dumps(r) + "\n" for r in (shard_rows(sid) if rows is None else rows)),
    }
    if measured:
        files["measurement-started.txt"] = "2026-11-10T00:00:00Z\n"
        files["measurement-ended.txt"] = "2026-11-10T00:30:00Z\n"
    else:
        del files["runs.jsonl"]
    for name, text in files.items():
        if name not in drop:
            (d / name).write_text(text)


def build(root, label="qualification", **per):
    """Every registered shard's artifact; `per[sid]` gives write_shard's arguments for one"""
    for sid in tg.SHARDS:
        for kw in per.get(sid, [{}]):
            write_shard(root, sid, label=kw.pop("label", label), **kw)
    return tg.manifest(root, run=RUN, attempt=1, label=label)


class CI(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp()

    def tearDown(self):
        shutil.rmtree(self.root)

    def ni(self, man, reg=REG):
        return tg.ni_judge(self.root, man, HEAD, reg=reg, check_git=False, rounds=R, batches=B)

    def test_a_null_run_passes_every_cell(self):
        v = self.ni(build(self.root))
        self.assertTrue(v["valid"], v["problems"])
        self.assertEqual(len(v["cells"]), 21)
        self.assertEqual(sorted(v["cells"]), sorted(f"{c}/{t}" for c in tg.CI_CORPORA for t in tg.CI_THREADS))
        self.assertTrue(v["passes"], v["failing"])
        self.assertEqual(v["cells"]["wikidata/2"]["shard"], "wikidata-2")
        self.assertIn("met** at every cell", tg.ni_markdown(v))

    def test_the_bound_is_batch_means_one_sided_95(self):
        rows = shard_rows("runs")
        v = self.ni(build(self.root, runs=[{"rows": rows}]))
        by = {(x["threads"], x["arm"], x["round"]): x for x in rows}
        logs = [math.log(by[(2, "mibz2", i)]["wall"] / by[(2, "mibz2@base/ab", i)]["wall"]) for i in range(R)]
        k = R // B
        means = [sum(logs[i * k:(i + 1) * k]) / k for i in range(B)]
        m = sum(logs) / R
        mm = sum(means) / B
        se = math.sqrt(sum((x - mm) ** 2 for x in means) / (B - 1)) / math.sqrt(B)
        self.assertAlmostEqual(v["cells"]["runs/2"]["wall"]["upper"], math.exp(m + tg.t_ppf(0.95, B - 1) * se), places=9)
        self.assertAlmostEqual(tg.t_ppf(0.95, 19), 1.729133, places=5)

    def test_one_cell_over_the_margin_fails_the_condition(self):
        for metric in ("wall", "cpu"):
            with self.subTest(metric):
                shutil.rmtree(self.root)
                self.root = tempfile.mkdtemp()
                v = self.ni(build(self.root, onebyte=[{"rows": shard_rows("onebyte", {(4, metric): 1.025})}]))
                self.assertTrue(v["valid"], v["problems"])
                self.assertFalse(v["passes"])
                self.assertEqual(len(v["failing"]), 1, v["failing"])
                self.assertIn(f"onebyte/4 {metric}", v["failing"][0])

    def test_flags_and_rss(self):
        v = self.ni(build(self.root, runs=[{"rows": shard_rows("runs", {(2, "wall"): 1.008}, sd=0.001)}],
                          text=[{"rows": shard_rows("text", {(4, "rss"): 1.15})}]))
        self.assertTrue(v["valid"], v["problems"])
        self.assertEqual(v["flags"], ["runs/2"])
        self.assertEqual(v["rss_regression"], ["text/4"])
        self.assertFalse(v["passes"])
        self.assertEqual(v["failing"], [])

    def test_admission(self):
        rows = shard_rows("records")
        cases = {
            "a smoke run": dict(label="smoke"),
            "a shard missing": dict(pattern=[]),
            "two measured attempts": dict(runs=[{"attempt": 1}, {"attempt": 2}]),
            "a measured attempt without its runs": dict(runs=[{"drop": ("runs.jsonl",)}]),
            "another head": dict(text=[{"over": {"head": "0" * 40}}]),
            "another base": dict(text=[{"over": {"base": "0" * 40}}]),
            "another run": dict(text=[{"over": {"run_id": 1}}]),
            "another rounds": dict(text=[{"over": {"rounds": R + 1}}]),
            "another workflow": dict(text=[{"over": {"workflow_ref": "x/bench.yml@y"}}]),
            "a failed child": dict(records=[{"rows": [dict(r, termination="signal 9") if i == 3 else r
                                                      for i, r in enumerate(rows)]}]),
            "a round missing": dict(records=[{"rows": rows[1:]}]),
            "two digests": dict(records=[{"rows": [dict(r, hash="fedcba9876543210") if i == 3 else r
                                                   for i, r in enumerate(rows)]}]),
            "2 threads on one core": dict(records=[{"rows": [dict(r, cpus="0,1") if r["threads"] == 2 else r
                                                             for r in rows]}]),
        }
        for what, per in cases.items():
            with self.subTest(what):
                shutil.rmtree(self.root)
                self.root = tempfile.mkdtemp()
                label = per.pop("label", "qualification")
                v = self.ni(build(self.root, label=label, **per))
                self.assertFalse(v["valid"], what)
                self.assertTrue(v["problems"])

    def test_the_runner_and_its_read_back(self):
        def mutate(name, f):
            shutil.rmtree(self.root)
            self.root = tempfile.mkdtemp()
            man = build(self.root)
            path = Path(self.root) / "tg-ni-random-a1" / "random" / name
            path.write_text(f(path.read_text()))
            return tg.manifest(self.root, run=RUN, attempt=1, label="qualification")
        rec, _ = record(True)
        cases = {
            "the target's lscpu model": ("lscpu.txt", lambda t: f"Model name: {tg.TARGET_NAME}\n"),
            "an 8-CPU runner": ("env.json", lambda t: json.dumps({"cpus": 8, "cores": 4, "core_cpus": ["0,1"] * 4})),
            "the predicate TARGET": ("preflight.json", lambda t: json.dumps(rec)),
            "another toolchain": ("rustc.txt", lambda t: "rustc 1.99.0 (x)\n"),
            "another corpus": ("corpus.sha256", lambda t: "9" * 64 + "  x\n"),
        }
        for what, (name, f) in cases.items():
            with self.subTest(what):
                v = self.ni(mutate(name, f))
                self.assertFalse(v["valid"], what)
        # (a file changed after the manifest was written)
        shutil.rmtree(self.root)
        self.root = tempfile.mkdtemp()
        man = build(self.root)
        (Path(self.root) / "tg-ni-runs-a1" / "runs" / "runs.jsonl").write_text("")
        v = self.ni(man)
        self.assertFalse(v["valid"])
        self.assertTrue(any("manifest's digest" in p for p in v["problems"]), v["problems"])

    def test_a_setup_failure_before_measuring_may_precede_the_measured_attempt(self):
        # (§2.5: an attempt that never started measuring holds no observations)
        man = build(self.root, runs=[{"attempt": 1, "measured": False}, {"attempt": 2}])
        v = self.ni(man)
        self.assertTrue(v["valid"], v["problems"])
        self.assertEqual(v["shards"]["runs"]["attempt"], 2)
        # (but never after it: a measured attempt is the attempt)
        shutil.rmtree(self.root)
        self.root = tempfile.mkdtemp()
        v = self.ni(build(self.root, runs=[{"attempt": 1}, {"attempt": 2, "measured": False}]))
        self.assertFalse(v["valid"])

    def test_the_frozen_file_must_name_the_ci_fields(self):
        for k in ("ni_rustc", "corpus_text"):
            reg = copy.deepcopy(REG)
            del reg["frozen"][k]
            self.assertFalse(self.ni(build(self.root), reg=reg)["valid"])
            shutil.rmtree(self.root)
            self.root = tempfile.mkdtemp()

    def test_harm_confirmation(self):
        flagged = self.ni(build(self.root, runs=[{"rows": shard_rows("runs", {(2, "wall"): 1.008}, sd=0.001)}],
                                text=[{"rows": shard_rows("text", {(1, "wall"): 1.008}, sd=0.001)}]))
        other = tempfile.mkdtemp()
        try:
            build(other, runs=[{"rows": shard_rows("runs", {(2, "wall"): 1.008}, sd=0.001, seed=9)}])
            again = tg.ni_judge(other, tg.manifest(other, run=RUN, attempt=1, label="qualification"), HEAD,
                                reg=REG, check_git=False, rounds=R, batches=B)
        finally:
            shutil.rmtree(other)
        self.assertEqual(tg.confirm(flagged, again), {"valid": True, "confirmed": ["runs/2"], "unresolved": ["text/1"]})
        self.assertFalse(tg.confirm(flagged, {"valid": False})["valid"])

    def test_the_power_script_runs(self):
        try:
            import scipy  # noqa: F401
        except ImportError:
            self.skipTest("scipy is not installed")
        import service_tg_power as pw
        self.assertAlmostEqual(pw.power(600, 20, 0.05, pw.DELTA), 0.05, places=6)
        self.assertGreater(pw.power(600, 20, 0.05, 0.0), pw.power(600, 20, 0.05, 0.005))


if __name__ == "__main__":
    unittest.main(verbosity=2)
