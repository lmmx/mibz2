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
  four-round order cycles, ab's own schedule; the full file-order chronology enforced before
  grouping, with its negative controls and a default-size control; the trend
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


R = 16  # (small, for speed: four whole cycles a dispatch; the registered 240 is the constant)
K = 12  # (the registered programme's dispatches)
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
WINDOW = {"start": 0.0, "length": 1800.0, "seed": 5}  # (each job's start and duration, seconds from T0, as GitHub records it)


def iso(t):
    import datetime
    return datetime.datetime.fromtimestamp(t, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


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
        "started_at": iso(T0 + WINDOW["start"]), "completed_at": iso(T0 + WINDOW["start"] + WINDOW["length"]),
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
        "runs.jsonl": "".join(json.dumps(r) + "\n" for r in (shard_rows(sid, seed=WINDOW["seed"]) if rows is None else rows)),
    }
    if measured:
        files["measurement-started.txt"] = "2026-11-10T00:00:00Z\n"
        files["measurement-ended.txt"] = "2026-11-10T00:30:00Z\n"
        n = files["runs.jsonl"].count("\n")
        step = min(1.0, 1700.0 / max(n, 1))  # (inside the 30-minute markers at any R)
        files["row-times.txt"] = f"# start {T0 + 0.5:.3f}\n" + "".join(f"{i} {T0 + 1 + i * step:.3f}\n" for i in range(n))
        files["stamp.exit"] = "0\n"
    else:
        del files["runs.jsonl"]
    for name, text in files.items():
        if name not in drop:
            (d / name).write_text(text)


def build(root, label="qualification", run=RUN, start=0.0, length=1800.0, seed=5, **per):
    """Every registered shard's attempts; `per[sid]` gives write_shard's arguments for each; the
    run's jobs start `start` seconds after T0 and last `length`"""
    HISTORY["artifacts"], HISTORY["jobs"] = [], []
    WINDOW["start"], WINDOW["length"], WINDOW["seed"] = start, length, seed
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
        return tg.ni_judge(self.root, man, HEAD, reg=reg, check_git=False, rounds=R)

    def test_a_dispatch_is_admitted_with_its_estimates(self):
        rows = shard_rows("runs")
        v = self.ni(build(self.root, runs=[{"rows": rows}]))
        self.assertTrue(v["valid"], v["problems"])
        self.assertEqual(v["schema"], "mibz2.tg_ni_dispatch/1")
        self.assertEqual(sorted(v["cells"]), sorted(f"{c}/{t}" for c in tg.CI_CORPORA for t in tg.CI_THREADS))
        self.assertEqual(v["cells"]["wikidata/2"]["shard"], "wikidata-2")
        self.assertNotIn("passes", v)  # (one dispatch is never a verdict)
        by = {(x["threads"], x["arm"], x["round"]): x for x in rows}
        logs = [math.log(by[(2, "mibz2", i)]["wall"] / by[(2, "mibz2@base/ab", i)]["wall"]) for i in range(R)]
        self.assertAlmostEqual(v["cells"]["runs/2"]["wall"]["d"], sum(logs) / R, places=12)
        self.assertEqual((v["started"], v["completed"]), (T0, T0 + 1800))
        self.assertEqual(v["shards"]["runs"]["model"], "AMD EPYC 7763 64-Core Processor")
        self.assertIn("not a verdict", tg.dispatch_markdown(v))

    def test_the_tails_are_kept(self):
        # (one round's long settle wait and slow child stay in: the dispatch's estimate is the
        # mean of all R paired logs)
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
        self.assertAlmostEqual(v["cells"]["runs/4"]["wall"]["d"], sum(logs) / R, places=12)

    def test_partial_cycles_are_refused(self):
        global R
        saved = R
        try:
            R = 18
            v = tg.ni_judge(self.root, build(self.root), HEAD, reg=REG, check_git=False, rounds=18)
        finally:
            R = saved
        self.assertFalse(v["valid"])
        self.assertIn("not whole 4-round order cycles", "; ".join(v["problems"]))

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
            "another rounds": dict(text=[{"over": {"rounds": R + 4}}]),
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


class Design(unittest.TestCase):
    def test_the_registered_programme(self):
        self.assertEqual((tg.NI_ROUNDS, tg.NI_DISPATCHES, tg.SPACING_MIN), (240, 12, 60))
        self.assertEqual(tg.rounds_problems(tg.NI_ROUNDS), [])
        # (the cycle is ab's own, and both a constant and a linear order coefficient cancel in it)
        self.assertEqual(tg.ORDER_CYCLE, tuple(ab_arm_order(r)[0] == 0 for r in range(4)))
        for r in range(tg.NI_ROUNDS):
            self.assertEqual(ab_arm_order(r)[0] == 0, tg.ORDER_CYCLE[r % 4])
        sign = [1 if h else -1 for h in tg.ORDER_CYCLE]
        self.assertEqual(sum(sign), 0)
        self.assertEqual(sum(i * x for i, x in enumerate(sign)), 0)
        # (a dispatch is 60 whole cycles: 120 rounds in each order)
        self.assertEqual(sum(tg.ORDER_CYCLE[r % 4] for r in range(tg.NI_ROUNDS)), 120)
        for r_ in (238, 242, 0, -4, 240.0):
            self.assertTrue(tg.rounds_problems(r_), r_)
        # (the gate restates the programme apart from the judge, and agrees)
        import tg_ni_gate as gate
        self.assertEqual(gate.K, tg.NI_DISPATCHES)
        self.assertEqual(gate.MARGIN, tg.MARGIN)
        self.assertAlmostEqual(gate.T95, tg.t_ppf(0.95, tg.NI_DISPATCHES - 1), places=8)
        try:
            from scipy import stats
            self.assertAlmostEqual(gate.T95, stats.t.ppf(0.95, 11), places=9)
        except ImportError:
            pass

    def test_the_dispatch_bound(self):
        rng = random.Random(4)
        ds = [rng.gauss(0.001, 0.004) for _ in range(12)]
        st = tg.dispatch_stats(ds)
        m = sum(ds) / 12
        sd = math.sqrt(sum((x - m) ** 2 for x in ds) / 11)
        self.assertAlmostEqual(st["upper"], math.exp(m + tg.t_ppf(0.95, 11) * sd / math.sqrt(12)), places=12)
        self.assertAlmostEqual(st["lo95"], math.exp(m - tg.t_ppf(0.975, 11) * sd / math.sqrt(12)), places=12)
        self.assertEqual(st["d"], ds)


def by_round(rows):
    """The shard's rows as complete round groups, in file order"""
    groups = {}
    for r in rows:
        groups.setdefault(r["round"], []).append(r)
    return [groups[k] for k in sorted(groups)]


def flat(groups):
    return [r for g in groups for r in g]


def shuffled_rounds(rows, seed=64):
    g = by_round(rows)
    random.Random(seed).shuffle(g)
    return flat(g)


def shuffled_cycles(rows, seed=64):
    # (whole four-round cycles moved: every round keeps ab's arm order for its own label)
    g = by_round(rows)
    cycles = [g[i:i + 4] for i in range(0, len(g), 4)]
    random.Random(seed).shuffle(cycles)
    return flat(flat(cycles))


def threads_421(rows):
    out = []
    for g in by_round(rows):
        pairs = {}
        for r in g:
            pairs.setdefault(r["threads"], []).append(r)
        out += [r for t in sorted(pairs, reverse=True) for r in pairs[t]]
    return out


def head_first_always(rows):
    out = []
    for i in range(0, len(rows), 2):
        out += sorted(rows[i:i + 2], key=lambda r: tg.CI_ARMS.index(r["arm"]))
    return out


def base_moved(rows):
    rows = list(rows)
    rows.append(rows.pop(1))
    return rows


# (the Researcher's REVIEW 6759371719 §1: complete, internally consistent artifacts whose rows
# ran in an order the registration doesn't name; each must be refused, never sorted back)
CONTROLS = {
    "whole-round permutation (seed 64)": shuffled_rounds,
    "whole-cycle permutation (seed 64)": shuffled_cycles,
    "reversed rounds": lambda rows: flat(by_round(rows)[::-1]),
    "thread order 4, 2, 1 in every round": threads_421,
    "head first in every round": head_first_always,
    "a base run moved to the end": base_moved,
}


class Chronology(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp()

    def tearDown(self):
        shutil.rmtree(self.root)

    def judged(self, rows=None, rounds=R):
        man = build(self.root, **({} if rows is None else {"runs": [{"rows": rows}]}))
        v = tg.ni_judge(self.root, man, HEAD, reg=REG, check_git=False, rounds=rounds)
        code = 0 if v["valid"] else 2
        import tg_ni_gate as gate
        return v, gate.gate("qualification", v, code)

    def test_the_traversal(self):
        want = list(tg.traversal("runs", (1, 2, 4), 4))
        self.assertEqual(want[:6], [(0, "runs", 1, "mibz2"), (0, "runs", 1, "mibz2@base/ab"), (0, "runs", 2, "mibz2"),
                                    (0, "runs", 2, "mibz2@base/ab"), (0, "runs", 4, "mibz2"),
                                    (0, "runs", 4, "mibz2@base/ab")])
        self.assertEqual([w[3] for w in want if w[2] == 1], ["mibz2", "mibz2@base/ab", "mibz2@base/ab", "mibz2",
                                                              "mibz2@base/ab", "mibz2", "mibz2", "mibz2@base/ab"])
        # (the fixtures emit it)
        rows = shard_rows("runs")
        self.assertEqual(tg.sequence_problems("runs", rows, "runs", (1, 2, 4), R), [])

    def test_every_negative_control_is_refused(self):
        clean, (status, line) = self.judged()
        self.assertTrue(clean["valid"], clean["problems"])
        self.assertEqual(status, 0, line)
        for name, change in CONTROLS.items():
            with self.subTest(name):
                shutil.rmtree(self.root)
                self.root = tempfile.mkdtemp()
                rows = change(shard_rows("runs"))
                self.assertEqual(len(rows), len(shard_rows("runs")))  # (complete: nothing missing)
                self.assertNotEqual(rows, shard_rows("runs"))
                v, (status, line) = self.judged(rows)
                self.assertFalse(v["valid"], name)
                self.assertTrue(any("not in the registered order" in p for p in v["problems"]), v["problems"])
                self.assertEqual(v["cells"], {})
                self.assertEqual(status, 1, line)
                self.assertIn("INVALID", line)

    def test_the_default_size(self):
        # (R 240, the registered constant, as the judge is called: a clean dispatch is admitted;
        # the Researcher's shuffled-round witness, with increasing observation times and a
        # regenerated manifest, is INVALID)
        global R
        saved = R
        import tg_ni_gate as gate
        try:
            R = tg.NI_ROUNDS
            v = tg.ni_judge(self.root, build(self.root), HEAD, reg=REG, check_git=False)
            self.assertTrue(v["valid"], v["problems"])
            self.assertEqual(v["rounds"], 240)
            self.assertEqual(gate.gate("qualification", v, 0)[0], 0)
            shutil.rmtree(self.root)
            self.root = tempfile.mkdtemp()
            build(self.root, runs=[{"rows": shuffled_rounds(shard_rows("runs"))}])
            for p in Path(self.root).glob("*/*/row-times.txt"):
                n = len(p.read_text().splitlines()) - 1
                p.write_text(f"# start {T0 + .5:.3f}\n" + "".join(f"{i} {T0 + 1 + i * .01:.3f}\n" for i in range(n)))
            v = tg.ni_judge(self.root, remanifest(self.root), HEAD, reg=REG, check_git=False)
            self.assertFalse(v["valid"])
            self.assertTrue(any("runs: rows not in the registered order" in p for p in v["problems"]), v["problems"])
            self.assertEqual(gate.gate("qualification", v, 2)[0], 1)
        finally:
            R = saved


class Replicated(unittest.TestCase):
    """The programme (§2.11): k dispatches, each its own run id, hourly; condition 2 per cell on
    the dispatch-level estimates"""

    def setUp(self):
        self.roots = []

    def tearDown(self):
        for r in self.roots:
            shutil.rmtree(r)

    def dispatch(self, j, label="qualification", start=None, length=1800.0, run=None, **per):
        root = tempfile.mkdtemp()
        self.roots.append(root)
        man = build(root, label=label, run=RUN + j if run is None else run,
                    start=3600.0 * j if start is None else start, length=length, seed=1000 + j, **per)
        return root, man

    def programme(self, k=K, per=lambda j: {}, **kw):
        out = []
        for j in range(k):
            root, man = self.dispatch(j, **dict(kw, **per(j)))
            out.append(tg.ni_judge(root, man, HEAD, reg=REG, check_git=False, rounds=R))
        return out

    def judge(self, ds):
        v = tg.replicated(ds, HEAD, rounds=R)
        import tg_ni_gate as gate
        code = 0 if v["valid"] and v["passes"] else 1 if v["valid"] else 2
        return v, code, gate.gate("replicated", v, code)

    def test_a_null_programme_passes_every_cell(self):
        ds = self.programme(per=lambda j: {"runs": [{"rows": shard_rows("runs", seed=j)}]})
        v, code, (status, line) = self.judge(ds)
        self.assertTrue(v["valid"], v["problems"])
        self.assertTrue(v["passes"], v["failing"])
        self.assertEqual(v["condition2"], "PASS")
        self.assertEqual(v["runs"], [RUN + j for j in range(K)])
        self.assertEqual(len(v["cells"]), 21)
        # (the bound is the dispatch-level one: the k estimates, t with k - 1 df)
        ds_ = [d["cells"]["runs/2"]["wall"]["d"] for d in ds]
        self.assertEqual(v["cells"]["runs/2"]["wall"]["d"], ds_)
        m = sum(ds_) / K
        sd = math.sqrt(sum((x - m) ** 2 for x in ds_) / (K - 1))
        self.assertAlmostEqual(v["cells"]["runs/2"]["wall"]["upper"], math.exp(m + tg.t_ppf(0.95, K - 1) * sd / math.sqrt(K)),
                               places=12)
        self.assertEqual((status, code), (0, 0), line)
        md = tg.ni_markdown(v)
        self.assertIn("met** at every cell", md)
        self.assertIn("nominal 5%, conditional on the dispatch model", md)
        self.assertIn("does not repair a cell's size", md)

    def test_one_cell_over_the_margin_is_not_shown(self):
        ds = self.programme(per=lambda j: {"onebyte": [{"rows": shard_rows("onebyte", {(4, "cpu"): 1.025}, seed=j)}]})
        v, code, (status, line) = self.judge(ds)
        self.assertTrue(v["valid"], v["problems"])
        self.assertFalse(v["passes"])
        self.assertEqual([f.split()[:2] for f in v["failing"]], [["onebyte/4", "cpu"]])
        self.assertEqual((code, status), (1, 1))
        self.assertIn("NOT SHOWN", line)

    def test_the_spread_between_dispatches_is_the_uncertainty(self):
        # (each dispatch tight, the dispatches apart: +-5% alternately at runs/2 wall. Within a
        # dispatch the estimate is precise; across them it is not, and the bound says so)
        ds = self.programme(per=lambda j: {"runs": [{"rows": shard_rows("runs", {(2, "wall"): 1.05 if j % 2 else 1 / 1.05},
                                                                         sd=0.001, seed=j)}]})
        v, code, _ = self.judge(ds)
        self.assertTrue(v["valid"], v["problems"])
        self.assertIn("runs/2", " ".join(v["failing"]))
        self.assertGreater(v["cells"]["runs/2"]["wall"]["upper"], 1.02)

    def test_flags_and_rss(self):
        ds = self.programme(per=lambda j: {"runs": [{"rows": shard_rows("runs", {(2, "wall"): 1.008}, sd=0.001, seed=j)}],
                                           "text": [{"rows": shard_rows("text", {(4, "rss"): 1.15}, seed=j)}]})
        v, code, _ = self.judge(ds)
        self.assertTrue(v["valid"], v["problems"])
        self.assertEqual(v["flags"], ["runs/2"])
        self.assertEqual(v["rss_regression"], ["text/4"])
        self.assertFalse(v["passes"])
        self.assertEqual(code, 1)

    def test_the_programme_is_admitted_whole_or_not_at_all(self):
        good = self.programme()
        bad = {
            "11 dispatches": good[:11],
            "13 dispatches": good + [good[0]],
            "a run twice": good[:11] + [good[10]],
            "a run id missing": good[:11] + [dict(good[11], run=None)],
            "one dispatch INVALID": good[:11] + [self.programme(k=1, label="smoke", start=3600.0 * 11, run=RUN + 11)[0]],
            "another head": good[:11] + [dict(good[11], head="0" * 40)],
            "another R": good[:11] + [dict(good[11], rounds=R + 4)],
            "too close: 30 minutes apart": good[:11] + [dict(good[11], started=good[10]["started"] + 1800)],
            "before the previous completed (an overrun not waited for)":
                good[:10] + [dict(good[10], completed=good[10]["started"] + 4 * 3600)] + [good[11]],
            "times unknown": good[:11] + [dict(good[11], started=None)],
            "a verdict unreadable": good[:11] + [None],
            "a cell's estimate missing": good[:11] + [dict(good[11], cells={k_: c for k_, c in good[11]["cells"].items()
                                                                          if k_ != "text/2"})],
        }
        v, code, (status, _) = self.judge(good)
        self.assertEqual((v["valid"], status), (True, 0), v["problems"])
        for what, ds in bad.items():
            with self.subTest(what):
                v, code, (status, line) = self.judge(ds)
                self.assertFalse(v["valid"], what)
                self.assertEqual(v["cells"], {})
                self.assertEqual((code, status), (2, 1), line)
                self.assertIn("INVALID", line)
        # (an overrun waited for is admitted: the next dispatch starts after the previous completes)
        late = good[:10] + [dict(good[10], completed=good[10]["started"] + 5000)] + [dict(good[11], started=good[10]["started"] + 5400)]
        self.assertTrue(tg.replicated(late, HEAD, rounds=R)["valid"])

    def test_the_gate_recomputes_the_bound(self):
        import copy
        import tg_ni_gate as gate
        v, code, _ = self.judge(self.programme())
        self.assertEqual(gate.gate("replicated", v, 0)[0], 0)
        # (a judge claiming a pass the dispatch estimates don't support)
        bent = copy.deepcopy(v)
        bent["cells"]["pattern/2"]["cpu"]["d"] = [0.05] * 11 + [0.0]
        status, line = gate.gate("replicated", bent, 0)
        self.assertEqual(status, 1)
        self.assertIn("disagrees", line)
        # (11 estimates, 11 run ids, a run id twice, a non-finite estimate: INVALID)
        for change in (lambda x: x["cells"]["text/1"]["wall"].__setitem__("d", x["cells"]["text/1"]["wall"]["d"][:11]),
                       lambda x: x.__setitem__("runs", x["runs"][:11]),
                       lambda x: x.__setitem__("runs", x["runs"][:11] + [x["runs"][0]]),
                       lambda x: x["cells"]["text/1"]["cpu"]["d"].__setitem__(3, float("nan"))):
            w = copy.deepcopy(v)
            change(w)
            status, line = gate.gate("replicated", w, 0)
            self.assertEqual(status, 1, line)
            self.assertIn("INVALID", line)
        # (an exit that disagrees)
        self.assertEqual(gate.gate("replicated", v, 1)[0], 1)
        self.assertEqual(gate.gate("replicated", v, 2)[0], 1)

    def test_the_command_line(self):
        pairs = []
        for j in range(K):
            root, man = self.dispatch(j)
            mpath = Path(root) / "manifest.json"
            mpath.write_text(json.dumps(man))
            pairs += ["--dispatch", root, str(mpath)]
        out = Path(self.roots[0]) / "verdict"
        real = tg.ni_judge
        try:
            tg.ni_judge = lambda root, m, head: real(root, m, head, reg=REG, check_git=False, rounds=R)
            real_rep = tg.replicated
            tg.replicated = lambda ds, head: real_rep(ds, head, rounds=R)
            code = tg.main(["replicated", *pairs, "--head", HEAD, "--out", str(out)])
            code11 = tg.main(["replicated", *pairs[:-3], "--head", HEAD, "--out", str(out) + "11"])
        finally:
            tg.ni_judge, tg.replicated = real, real_rep
        self.assertEqual(code, 0)
        self.assertEqual(code11, 2)
        v = json.loads(Path(str(out) + ".json").read_text())
        import tg_ni_gate as gate
        self.assertEqual(gate.gate("replicated", v, code)[0], 0)

    def test_harm_confirmation(self):
        flagged = tg.replicated(self.programme(per=lambda j: {
            "runs": [{"rows": shard_rows("runs", {(2, "wall"): 1.008}, sd=0.001, seed=j)}],
            "text": [{"rows": shard_rows("text", {(1, "wall"): 1.008}, sd=0.001, seed=j)}]}), HEAD, rounds=R)
        again = tg.replicated(self.programme(per=lambda j: {
            "run": RUN + 100 + j, "runs": [{"rows": shard_rows("runs", {(2, "wall"): 1.008}, sd=0.001, seed=50 + j)}]}),
            HEAD, rounds=R)
        self.assertTrue(again["valid"], again["problems"])
        self.assertEqual(flagged["flags"], ["runs/2", "text/1"])
        c = tg.confirm(flagged, again)
        self.assertTrue(c["valid"], c)
        self.assertEqual((c["confirmed"], c["unresolved"]), (["runs/2"], ["text/1"]))
        self.assertFalse(tg.confirm(flagged, {"valid": False})["valid"])
        # (the same programme twice, or one sharing a run, is not a confirmation)
        self.assertFalse(tg.confirm(flagged, flagged)["valid"])
        self.assertIn("distinct", tg.confirm(flagged, flagged)["problems"][0])
        self.assertFalse(tg.confirm(flagged, dict(again, runs=again["runs"][:11] + [RUN]))["valid"])
        self.assertFalse(tg.confirm(flagged, dict(again, head="0" * 40))["valid"])

if __name__ == "__main__":
    unittest.main(verbosity=2)
