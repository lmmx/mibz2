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
- The final design (registration §2.9): R 2880 in B 24 batches of 120, each 30 complete
  four-round order cycles, ab's own schedule; the schedule enforced on the rows; the trend
  veto's statistic, cutoff and effect (condition 2 INVALID, every estimate kept, condition 7's
  flags and confirmation unchanged); every row kept.

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


R, B = 96, 24  # (small, for speed: one four-round cycle a batch; the registered 2880 and 24 are constants)
RUN = 4242


def shard_rows(sid, effect={}, sd=0.003, seed=5):
    """One shard's records: the head's figure times `effect[(threads, metric)]`, placed as
    --spread places them on a runner whose cores are CPUs 0,1 and 2,3"""
    corpus, threads = tg.SHARDS[sid]
    rng = random.Random(f"{seed}{sid}")
    out = []
    for rnd in range(R):
        for t in threads:
            for arm in (tg.CI_ARMS if tg.ORDER_CYCLE[rnd % 4] else tg.CI_ARMS[::-1]):  # (ab's schedule)
                w, c, r = 0.05 * rng.gauss(1, sd), 0.08 * rng.gauss(1, sd), 30000 * rng.gauss(1, 0.001)
                if arm == "mibz2":
                    w *= effect.get((t, "wall"), 1.0)
                    c *= effect.get((t, "cpu"), 1.0)
                    r *= effect.get((t, "rss"), 1.0)
                out.append({"round": rnd, "corpus": corpus, "threads": t, "arm": arm, "wall": w, "cpu": c,
                            "rss_kb": r, "out": 1000, "hash": "0123456789abcdef", "termination": "success",
                            "err": "", "cpus": {1: "1", 2: "0,2", 4: "all"}[t]})
    return out


HISTORY = {"artifacts": [], "jobs": []}
T0 = 1794268800.0  # (2026-11-10T00:00:00Z, the fixtures' measurement start)


def write_shard(root, sid, attempt=1, rows=None, label="qualification", measured=True, over=None, drop=(),
                job=None, step=None, steps=True, listed=True, downloaded=True, artifact=True, run=RUN):
    """One shard attempt: its artifact's files (unless `artifact` is False or it wasn't
    `downloaded`), GitHub's record of the artifact (unless not `listed`) and of the job: its
    conclusion `job` and its measurement step's `step` (success and success when `measured`,
    failure and skipped otherwise), or no steps at all (`steps` False)"""
    corpus, threads = tg.SHARDS[sid]
    name = f"tg-ni-{sid}-a{attempt}"
    HISTORY["jobs"].append({
        "name": f"shard ({sid}, {corpus}, {','.join(map(str, threads))})", "run_attempt": attempt,
        "run_id": run, "head_sha": HEAD,
        "status": "completed", "conclusion": job or ("success" if measured else "failure"),
        **({"steps": [{"name": "Build the head and the base", "conclusion": "success"},
                      {"name": tg.MEASURE_STEP, "conclusion": step or ("success" if measured else "skipped"),
                       "started_at": "2026-11-10T00:00:00Z" if (step or measured) not in (False, "skipped") else None}]}
           if steps else {})})
    if artifact and listed:
        HISTORY["artifacts"].append({"name": name, "id": len(HISTORY["artifacts"]) + 1,
                                     "digest": f"sha256:{len(HISTORY['artifacts']):064x}"})
    if not (artifact and downloaded):
        return
    d = Path(root) / name / sid
    d.mkdir(parents=True)
    info = {"schema": "mibz2.tg_ni_shard/1", "label": label, "shard": sid, "corpus": corpus,
            "threads": list(threads), "rounds": R, "head": HEAD, "base": BASESHA, "github_sha": HEAD,
            "workflow_sha": HEAD, "repository": "lmmx/mibz2",
            "workflow_ref": "lmmx/mibz2/.github/workflows/target-gated-ni.yml@refs/heads/x",
            "run_id": run, "run_attempt": attempt}
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
        n = files["runs.jsonl"].count("\n")
        files["row-times.txt"] = f"# start {T0 + 0.5:.3f}\n" + "".join(f"{i} {T0 + 1 + i:.3f}\n" for i in range(n))
        files["stamp.exit"] = "0\n"
    else:
        del files["runs.jsonl"]
    for name, text in files.items():
        if name not in drop:
            (d / name).write_text(text)


def build(root, label="qualification", run=RUN, **per):
    """Every registered shard's attempts; `per[sid]` gives write_shard's arguments for each"""
    HISTORY["artifacts"], HISTORY["jobs"] = [], []
    for sid in tg.SHARDS:
        for kw in per.get(sid, [{}]):
            kw = dict(kw)
            write_shard(root, sid, label=kw.pop("label", label), run=run, **kw)
    return remanifest(root, label, run=run)


def remanifest(root, label="qualification", run=RUN):
    return tg.manifest(root, HISTORY["artifacts"], HISTORY["jobs"], run=run, attempt=1, label=label)


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
            build(self.root)
            path = Path(self.root) / "tg-ni-random-a1" / "random" / name
            path.write_text(f(path.read_text()))
            return remanifest(self.root)
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

    def fresh(self):
        shutil.rmtree(self.root)
        self.root = tempfile.mkdtemp()

    def refused(self, why, **per):
        self.fresh()
        v = self.ni(build(self.root, **per))
        self.assertFalse(v["valid"], why)
        return v

    def test_the_history_is_reconciled_fail_closed(self):
        # (the Researcher's four probes, review of inbox 6756614616 §5, each accepted by the
        # judge at eb46b5a8, and the other unknown histories, each refused)
        good = {"attempt": 2}
        cases = {
            "a cancelled job with a complete-looking artifact": [{"job": "cancelled", "step": "cancelled"}],
            "a cancelled job whose step says success": [{"job": "cancelled"}],
            "attempt 1 with rows but no start marker, then attempt 2":
                [{"attempt": 1, "drop": ("measurement-started.txt", "measurement-ended.txt")}, good],
            "an earlier measurement cancelled, with no artifact, then a good attempt 2":
                [{"attempt": 1, "job": "cancelled", "step": "cancelled", "artifact": False}, good],
            "an earlier measurement started per the API, artifact clean, then attempt 2":
                [{"attempt": 1, "measured": False, "step": "failure"}, good],
            "a wrong workflow sha": [{"over": {"workflow_sha": "0" * 40}}],
            "an inconsistent shard attempt": [{"over": {"run_attempt": 99}}],
            "another repository": [{"over": {"repository": "fork/mibz2"}}],
            "a workflow ref of another file": [{"over": {"workflow_ref": "lmmx/mibz2/.github/workflows/x/target-gated-ni.yml@r"}}],
            "the measured artifact listed but not downloaded": [{"downloaded": False}],
            "the measured artifact not listed by GitHub": [{"listed": False}],
            "the job's steps unknown": [{"steps": False}],
            "the measured job timed out": [{"job": "failure", "step": "cancelled"}],
            "the measured step failed in a job marked success": [{"step": "failure"}],
            "a setup failure whose step didn't record as skipped":
                [{"attempt": 1, "measured": False, "job": "cancelled", "step": "skipped"}, good],
            "a setup failure whose artifact wasn't downloaded":
                [{"attempt": 1, "measured": False, "downloaded": False}, good],
        }
        for what, attempts in cases.items():
            with self.subTest(what):
                v = self.refused(what, runs=attempts)
                self.assertTrue(any(x.startswith("runs") for x in v["problems"]), v["problems"])
        # (no job at all for a shard: an artifact without history)
        self.fresh()
        man = build(self.root)
        man["jobs"] = [j for j in man["jobs"] if not j["name"].startswith("shard (runs,")]
        self.assertFalse(self.ni(man)["valid"])
        # (a downloaded artifact GitHub didn't list)
        self.fresh()
        man = build(self.root)
        man["artifacts"] = [a for a in man["artifacts"] if a["shard"] != "text"]
        HISTORY["artifacts"] = [a for a in HISTORY["artifacts"] if not a["name"].startswith("tg-ni-text-")]
        v = self.ni(remanifest(self.root))
        self.assertFalse(v["valid"])
        self.assertTrue(any("not listed by GitHub" in x for x in v["problems"]), v["problems"])
        # (a job of another head, or another run, in the history)
        for key, value in (("head_sha", "0" * 40), ("run_id", 1)):
            self.fresh()
            man = build(self.root)
            for j in man["jobs"]:
                if j["name"].startswith("shard (pattern,"):
                    j[key] = value
            self.assertFalse(self.ni(man)["valid"], key)
        # (a manifest of the old schema, which kept no history)
        self.fresh()
        man = build(self.root)
        man["schema"] = "mibz2.tg_ni_manifest/1"
        self.assertFalse(self.ni(man)["valid"])

    def test_the_timing_must_be_complete_and_coherent(self):
        # (the Researcher's §2 probes on the pilot review, 6757361275: each was admitted before)
        cases = {
            "no row-times.txt": ("row-times.txt", None),
            "the stamper failed": ("stamp.exit", "1\n"),
            "no stamper exit": ("stamp.exit", None),
            "times backwards": ("row-times.txt", lambda t: "\n".join(
                [t.splitlines()[0]] + [f"{i} {T0 + 1000 - i:.3f}" for i in range(len(t.splitlines()) - 1)]) + "\n"),
            "a row missing its time": ("row-times.txt", lambda t: "\n".join(t.splitlines()[:-1]) + "\n"),
            "a row timed twice": ("row-times.txt", lambda t: t + t.splitlines()[-1] + "\n"),
            "no start time": ("row-times.txt", lambda t: "\n".join(t.splitlines()[1:]) + "\n"),
            "a time not finite": ("row-times.txt", lambda t: t.replace(f"0 {T0 + 1:.3f}", "0 nan")),
            "times outside the measurement": ("row-times.txt", lambda t: "\n".join(
                [f"# start {T0 + 9000:.3f}"] + [f"{i} {T0 + 9001 + i:.3f}" for i in range(len(t.splitlines()) - 1)]) + "\n"),
            "a malformed line": ("row-times.txt", lambda t: t + "garbage\n"),
        }
        for what, (name, change) in cases.items():
            with self.subTest(what):
                self.fresh()
                build(self.root)
                path = Path(self.root) / "tg-ni-records-a1" / "records" / name
                if change is None:
                    path.unlink()
                elif isinstance(change, str):
                    path.write_text(change)
                else:
                    path.write_text(change(path.read_text()))
                v = self.ni(remanifest(self.root))
                self.assertFalse(v["valid"], what)
                self.assertTrue(any(x.startswith("records") for x in v["problems"]), v["problems"])

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
            man = build(other, run=RUN + 1,
                        runs=[{"rows": shard_rows("runs", {(2, "wall"): 1.008}, sd=0.001, seed=9)}])
            again = tg.ni_judge(other, man, HEAD, reg=REG, check_git=False, rounds=R, batches=B)
        finally:
            shutil.rmtree(other)
        self.assertTrue(again["valid"], again["problems"])
        self.assertEqual(tg.confirm(flagged, again), {"valid": True, "registered_run": RUN, "confirmation_run": RUN + 1,
                                                      "confirmed": ["runs/2"], "unresolved": ["text/1"]})
        self.assertFalse(tg.confirm(flagged, {"valid": False})["valid"])
        # (the Researcher's finding: the same verdict twice is not a confirmation)
        same = tg.confirm(flagged, flagged)
        self.assertFalse(same["valid"])
        self.assertIn("distinct", same["problems"][0])
        # (nor a confirmation of another head)
        self.assertFalse(tg.confirm(flagged, dict(again, head="0" * 40))["valid"])

    def test_the_power_script(self):
        try:
            from scipy import integrate
        except ImportError:
            self.skipTest("scipy is not installed")
        import service_tg_power as pw
        # (size under the model at the boundary; power falls as the true ratio nears it)
        self.assertAlmostEqual(pw.power(600, 20, 0.05, 1.02), 0.05, places=6)
        self.assertGreater(pw.power(600, 20, 0.05, 1.0), pw.power(600, 20, 0.05, 1.005))
        # (the Researcher's NaN, at R900 / median / null with the library's noncentral-t CDF:
        # the integral is finite there, and a non-finite result is an error, never a printed 0)
        self.assertTrue(math.isfinite(pw.power(960, 20, 0.0139, 1.0)))
        real = integrate.quad
        try:
            integrate.quad = lambda *a, **k: (float("nan"), 0.0)
            with self.assertRaises(ArithmeticError):
                pw.power(600, 20, 0.05, 1.0)
        finally:
            integrate.quad = real
        # (batches of an odd length break the arms' two-round order balance: refused)
        with self.assertRaises(ValueError):
            pw.power(900, 20, 0.05, 1.0)
        # (the final design's limiting cell, pattern/2/wall at the pilot's largest LRV over the
        # five registered lengths, sigma 0.20139894: the Researcher's 0.98548, and 0.85610 at
        # twice the LRV, reproduced; conditional on iid normal batch means, not a guarantee)
        sigma = 0.20139894418068915
        self.assertAlmostEqual(pw.power(2880, 24, sigma, 1.005), 0.9854840380109718, places=6)
        self.assertAlmostEqual(pw.power(2880, 24, sigma * math.sqrt(2), 1.005), 0.8561073746929773, places=6)


def ab_arm_order(rnd, n=2):
    """examples/ab.rs `arm_order`, restated: rotated a step each round, reversed every other
    cycle of n rounds"""
    order = list(range(n))
    order = order[rnd % n:] + order[:rnd % n]
    return order[::-1] if (rnd // n) % 2 else order


def trended(sid, threads, metric, slope, **kw):
    """shard_rows, with the head's `metric` at `threads` drifting by exp(slope * round)"""
    rows = shard_rows(sid, **kw)
    for r in rows:
        if r["threads"] == threads and r["arm"] == tg.CI_ARMS[0]:
            r[metric] *= math.exp(slope * r["round"])
    return rows


class Design(unittest.TestCase):
    def test_the_registered_design(self):
        self.assertEqual((tg.NI_ROUNDS, tg.NI_BATCHES, tg.NI_BATCH_LEN, tg.NI_CYCLES_PER_BATCH), (2880, 24, 120, 30))
        self.assertEqual(tg.cycle_problems(tg.NI_ROUNDS, tg.NI_BATCHES), [])
        # (the cycle is ab's own, and both a constant and a linear order coefficient cancel in it)
        self.assertEqual(tg.ORDER_CYCLE, tuple(ab_arm_order(r)[0] == 0 for r in range(4)))
        for r in range(tg.NI_ROUNDS):
            self.assertEqual(ab_arm_order(r)[0] == 0, tg.ORDER_CYCLE[r % 4])
        sign = [1 if h else -1 for h in tg.ORDER_CYCLE]
        self.assertEqual(sum(sign), 0)
        self.assertEqual(sum(i * x for i, x in enumerate(sign)), 0)
        # (every batch starts on a cycle boundary and holds 30 whole cycles: 60 rounds each order)
        for b in range(tg.NI_BATCHES):
            rounds = range(b * tg.NI_BATCH_LEN, (b + 1) * tg.NI_BATCH_LEN)
            self.assertEqual(rounds[0] % 4, 0)
            self.assertEqual(sum(tg.ORDER_CYCLE[r % 4] for r in rounds), 60)
        # (R600/B20 (m 30), m 45, m 50, partial batches: not whole cycles, or not whole batches)
        for r_, b_ in ((600, 20), (900, 20), (1000, 20), (2880, 25), (2880, 1)):
            self.assertTrue(tg.cycle_problems(r_, b_), (r_, b_))
        with self.assertRaises(ValueError):
            tg.batch_stats([0.0] * 600, 20)

    def test_the_trend_cutoff(self):
        self.assertAlmostEqual(tg.t_ppf(1 - 0.01 / (2 * 42), 22), tg.TREND_T_MAX, places=8)
        import tg_ni_gate as gate
        self.assertEqual(gate.TREND_T_MAX, tg.TREND_T_MAX)
        self.assertEqual(gate.BATCHES, tg.NI_BATCHES)
        try:
            from scipy import stats
        except ImportError:
            return
        self.assertAlmostEqual(stats.t.ppf(1 - 0.01 / 84, 22), tg.TREND_T_MAX, places=9)

    def test_the_trend_statistic(self):
        rng = random.Random(7)
        ys = [rng.gauss(0, 1) + 0.05 * i for i in range(24)]
        import tg_ni_gate as gate
        self.assertAlmostEqual(tg.trend_t(ys), gate.slope_t(ys), places=12)
        try:
            from scipy import stats
            fit = stats.linregress(range(24), ys)
            self.assertAlmostEqual(tg.trend_t(ys), fit.slope / fit.stderr, places=9)
        except ImportError:
            pass
        # (invariant to a shift: the veto reads the trend, not the level)
        self.assertAlmostEqual(tg.trend_t([y + 3 for y in ys]), tg.trend_t(ys), places=9)
        # (zero residual variance: 0 for a zero slope, infinity otherwise; non-finite: None)
        self.assertEqual(tg.trend_t([0.25] * 24), 0.0)
        self.assertEqual(tg.trend_t([float(i) for i in range(24)]), math.inf)
        self.assertEqual(tg.trend_t([-float(i) for i in range(24)]), -math.inf)
        self.assertIsNone(tg.trend_t([0.0] * 23 + [math.nan]))
        self.assertIsNone(tg.trend_t([0.0, 1.0]))
        for f in (tg.trend_t, gate.slope_t):
            self.assertEqual(f([0.25] * 24), 0.0)
            self.assertIsNone(f([0.0] * 23 + [math.inf]))


class Veto(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp()

    def tearDown(self):
        shutil.rmtree(self.root)

    def ni(self, man):
        return tg.ni_judge(self.root, man, HEAD, reg=REG, check_git=False, rounds=R, batches=B)

    def test_a_null_run_is_not_vetoed(self):
        v = self.ni(build(self.root))
        self.assertTrue(v["valid"], v["problems"])
        self.assertEqual(v["vetoed"], [])
        self.assertEqual(v["condition2"], "PASS")
        for cell in v["cells"].values():
            for metric in tg.CI_METRICS:
                self.assertEqual(len(cell[metric]["batch_means"]), B)
                self.assertLessEqual(abs(cell[metric]["trend_t"]), tg.TREND_T_MAX)
        import tg_ni_gate as gate
        self.assertEqual(gate.veto(v), ([], []))

    def test_a_trend_invalidates_condition_2_and_keeps_every_estimate(self):
        for metric in tg.CI_METRICS:
            with self.subTest(metric):
                shutil.rmtree(self.root)
                self.root = tempfile.mkdtemp()
                v = self.ni(build(self.root, runs=[{"rows": trended("runs", 2, metric, 0.00015)}]))
                self.assertTrue(v["valid"], v["problems"])  # (admitted: the rows stand)
                self.assertEqual(len(v["vetoed"]), 1, v["vetoed"])
                self.assertTrue(v["vetoed"][0].startswith(f"runs/2 {metric} trend t +"), v["vetoed"])
                self.assertEqual(v["condition2"], "INVALID")
                self.assertFalse(v["passes"])
                # (the estimate is the plain mean of every round, not detrended: its upper bound
                # is still at most 1.02, the cell not failing, and still reported)
                self.assertEqual(v["failing"], [])
                self.assertEqual(len(v["cells"]), 21)
                self.assertIn("upper", v["cells"]["runs/2"][metric])
                md = tg.ni_markdown(v)
                self.assertIn("**INVALID** (the trend veto", md)
                self.assertIn("| runs/2 |", md)

    def test_the_veto_does_not_touch_condition_7(self):
        # (a wall flag and its confirmation stand whether or not a veto fired: condition 7 unchanged)
        rows = shard_rows("runs", {(2, "wall"): 1.008}, sd=0.001)
        rows_t = [dict(r) for r in rows]
        for r in rows_t:
            if r["threads"] == 1 and r["arm"] == tg.CI_ARMS[0]:
                r["cpu"] *= math.exp(0.0005 * r["round"])
        first = self.ni(build(self.root, runs=[{"rows": rows_t}]))
        self.assertTrue(first["vetoed"])
        self.assertEqual(first["flags"], ["runs/2"])
        other = tempfile.mkdtemp()
        try:
            again = tg.ni_judge(other, build(other, run=RUN + 1, runs=[{"rows": rows}]), HEAD, reg=REG,
                                check_git=False, rounds=R, batches=B)
        finally:
            shutil.rmtree(other)
        self.assertEqual(tg.confirm(first, again)["confirmed"], ["runs/2"])

    def test_the_judges_exit(self):
        man = build(self.root, runs=[{"rows": trended("runs", 2, "wall", 0.0005)}])
        mpath = Path(self.root) / "manifest.json"
        mpath.write_text(json.dumps(man))
        out = Path(self.root) / "ni"
        real = tg.ni_judge
        try:
            tg.ni_judge = lambda root, m, head: real(root, m, head, reg=REG, check_git=False, rounds=R, batches=B)
            code = tg.main(["ni", self.root, "--manifest", str(mpath), "--head", HEAD, "--out", str(out)])
        finally:
            tg.ni_judge = real
        self.assertEqual(code, 2, "a vetoed condition 2 is INVALID: exit 2")
        import tg_ni_gate as gate
        v = json.loads(Path(str(out) + ".json").read_text())
        status, line = gate.gate("qualification", v, code)
        self.assertEqual(status, 1)
        self.assertIn("INVALID (the trend veto", line)

    def test_the_schedule_is_enforced(self):
        # (head first in every round: not ab's four-round cycle)
        rows = shard_rows("runs")
        fixed = []
        for i in range(0, len(rows), 2):
            pair = sorted(rows[i:i + 2], key=lambda r: tg.CI_ARMS.index(r["arm"]))
            fixed += pair
        v = self.ni(build(self.root, runs=[{"rows": fixed}]))
        self.assertFalse(v["valid"])
        self.assertTrue(any("off ab's four-round schedule" in p and "head first" in p for p in v["problems"]),
                        v["problems"])
        # (the arms not back to back: a round's base run moved away from its head run)
        shutil.rmtree(self.root)
        self.root = tempfile.mkdtemp()
        rows = shard_rows("runs")
        rows.append(rows.pop(1))
        v = self.ni(build(self.root, runs=[{"rows": rows}]))
        self.assertFalse(v["valid"])
        self.assertTrue(any("not back to back" in p for p in v["problems"]), v["problems"])
        # (a design of partial cycles is refused before anything is estimated)
        shutil.rmtree(self.root)
        self.root = tempfile.mkdtemp()
        v = tg.ni_judge(self.root, build(self.root), HEAD, reg=REG, check_git=False, rounds=R, batches=48)
        self.assertFalse(v["valid"])
        self.assertIn("not whole 4-round order cycles", "; ".join(v["problems"]))

    def test_the_tails_are_kept(self):
        # (one round's long settle wait and slow child stay in: the estimate is the mean of all
        # R paired logs, the same as the mean of the batch means)
        rows = shard_rows("runs")
        for r in rows:
            if r["round"] == 5 and r["threads"] == 4:
                r["waited_s"] = 41
                if r["arm"] == tg.CI_ARMS[0]:
                    r["wall"] *= 3.0
        v = self.ni(build(self.root, runs=[{"rows": rows}]))
        self.assertTrue(v["valid"], v["problems"])
        by = {(x["threads"], x["arm"], x["round"]): x for x in rows}
        logs = [math.log(by[(4, tg.CI_ARMS[0], i)]["wall"] / by[(4, tg.CI_ARMS[1], i)]["wall"]) for i in range(R)]
        w = v["cells"]["runs/4"]["wall"]
        self.assertAlmostEqual(math.log(w["ratio"]), sum(logs) / R, places=12)
        self.assertAlmostEqual(math.log(w["ratio"]), sum(w["batch_means"]) / B, places=12)


if __name__ == "__main__":
    unittest.main(verbosity=2)
