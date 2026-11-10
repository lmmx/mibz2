"""bench/service_tg.py on synthetic attempts and CI artifacts: the target-gated registration's
judge (docs/journal/2026-11-10-service-guarded-target-gated-registration.md).

- The workstation A/B: a clean attempt meets conditions 3 and 4; each condition fails on its
  own (a gain too small at 10, a CPU cost at 10, NOT SHOWN at 9 or 11, a CPU or RSS
  regression); each admission defect is refused with its kind (identity, the toolchain
  included; workload; preflight, a schema 1 record included; structural: termination, plans,
  rounds, a third arm).
- CI's condition 2: the 42-cell grid admitted only as registered (head, base, bench, rounds,
  threads, runner, corpora, completeness, termination, digests), each cell's one-sided bound,
  the all-pass rule, wall flags and the harm-confirmation rule, peak RSS.
- Student's t quantiles against fixed values (and scipy's, where it is installed).

    python3 bench/test_service_tg.py
"""

import copy
import math
import random
import sys
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
       "frozen": {"candidate": CAND, "base": BASESHA, "base_fetched": FETCHED, "rustc": RUSTC, "ci_rounds": 10}}
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
        "preflight.json": __import__("json").dumps(rec),
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
                files(**{"preflight.json": __import__("json").dumps(edit(rec, lambda r: r.__setitem__(
                    "schema", "mibz2.svc_target/1")))}),
                files(**{"preflight.json": __import__("json").dumps(record(False)[0]), "preflight.exit": "3\n"}),
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
# CI


def ci_rows(effect={}, sd=0.005, seed=5, rounds=10):
    """A Benchmark run's records: the head's figure times `effect[(corpus, threads, metric)]`"""
    rng = random.Random(seed)
    out = []
    for rnd in range(rounds):
        for corpus in tg.CI_CORPORA:
            for t in tg.CI_THREADS:
                for arm in tg.CI_ARMS:
                    w, c, r = 0.05 * rng.gauss(1, sd), 0.08 * rng.gauss(1, sd), 30000 * rng.gauss(1, 0.001)
                    if arm == "mibz2":
                        w *= effect.get((corpus, t, "wall"), 1.0)
                        c *= effect.get((corpus, t, "cpu"), 1.0)
                        r *= effect.get((corpus, t, "rss"), 1.0)
                    out.append({"round": rnd, "corpus": corpus, "threads": t, "arm": arm, "wall": w, "cpu": c,
                                "rss_kb": r, "out": 1000, "hash": f"{abs(hash(corpus)) % 10**16:016x}",
                                "termination": "success", "err": ""})
    return out


def result(**over):
    r = {"mode": "pr", "head": HEAD, "base": BASESHA, "bench": "3.5.0", "rounds": 10, "threads": [1, 2, 4],
         "machine": {"cpu": "AMD EPYC 7763 64-Core Processor", "cores": 4}}
    r.update(over)
    return r


class CI(unittest.TestCase):
    def ci(self, res=None, r=None, reg=REG):
        return tg.ci_judge(result() if res is None else res, ci_rows() if r is None else r, head=HEAD, reg=reg,
                           check_git=False)

    def test_a_null_run_passes_every_cell(self):
        v = self.ci()
        self.assertTrue(v["valid"], v["problems"])
        self.assertEqual(len(v["cells"]), 21)
        self.assertEqual(sum(len(tg.CI_METRICS) for _ in v["cells"]), 42)
        self.assertTrue(v["passes"], v["failing"])
        self.assertEqual(v["flags"], [])
        self.assertIn("met** at every cell", tg.ci_markdown(v))

    def test_the_bound_is_one_sided_95(self):
        r = ci_rows()
        v = self.ci(r=r)
        by = {(x["corpus"], x["threads"], x["arm"], x["round"]): x for x in r}
        logs = [math.log(by[("runs", 2, "mibz2", i)]["wall"] / by[("runs", 2, "mibz2@base/ab", i)]["wall"])
                for i in range(10)]
        m = sum(logs) / 10
        sd = math.sqrt(sum((x - m) ** 2 for x in logs) / 9)
        self.assertAlmostEqual(v["cells"]["runs/2"]["wall"]["upper"], math.exp(m + 1.833113 * sd / math.sqrt(10)),
                               places=6)

    def test_one_cell_over_the_margin_fails_the_condition(self):
        for metric in ("wall", "cpu"):
            with self.subTest(metric):
                v = self.ci(r=ci_rows(effect={("onebyte", 4, metric): 1.025}))
                self.assertTrue(v["valid"])
                self.assertFalse(v["passes"])
                self.assertEqual(len(v["failing"]), 1, v["failing"])
                self.assertIn(f"onebyte 4t {metric}", v["failing"][0])
        # (a wall slowdown inside the margin, shown: a flag, condition 2 still met)
        v = self.ci(r=ci_rows(effect={("runs", 2, "wall"): 1.008}, sd=0.001))
        self.assertTrue(v["passes"], v["failing"])
        self.assertEqual(v["flags"], ["runs/2"])
        # (noise too wide to show the bound: fails, not shown is not passed)
        v = self.ci(r=ci_rows(sd=0.08))
        self.assertFalse(v["passes"])

    def test_peak_rss(self):
        v = self.ci(r=ci_rows(effect={("text", 4, "rss"): 1.15}))
        self.assertEqual(v["rss_regression"], ["text/4"])
        self.assertFalse(v["passes"])
        v = self.ci(r=ci_rows(effect={("text", 4, "rss"): 1.05}))
        self.assertTrue(v["passes"])

    def test_admission(self):
        def drop(f):
            r = ci_rows()
            f(r)
            return r
        noname = result()
        noname["machine"] = {"cpu": tg.TARGET_NAME, "cores": 4}
        cases = {
            "master mode": (result(mode="master"), None),
            "another head": (result(head="0" * 40), None),
            "another base": (result(base="0" * 40), None),
            "another bench": (result(bench="3.4.0"), None),
            "20 rounds": (result(rounds=20), None),
            "a 2-CPU runner": (result(machine={"cpu": "x", "cores": 2}), None),
            "the target's CPU": (noname, None),
            "threads 1, 2": (result(threads=[1, 2]), None),
            "a missing round": (None, drop(lambda r: r.pop(0))),
            "a failed child": (None, drop(lambda r: r[3].__setitem__("termination", "signal 9"))),
            "an error": (None, drop(lambda r: r[3].__setitem__("err", "x"))),
            "two digests": (None, drop(lambda r: r[3].__setitem__("hash", "ffffffffffffffff"))),
            "a corpus outside the grid": (None, drop(lambda r: r[3].__setitem__("corpus", "other"))),
            "a missing corpus": (None, [x for x in ci_rows() if x["corpus"] != "pattern"]),
            "a repeated round": (None, drop(lambda r: r.append(dict(r[0])))),
        }
        for what, (res, r) in cases.items():
            with self.subTest(what):
                v = self.ci(res=res, r=r)
                self.assertFalse(v["valid"], what)
                self.assertTrue(v["problems"])
        reg = copy.deepcopy(REG)
        del reg["frozen"]["ci_rounds"]
        self.assertFalse(self.ci(reg=reg)["valid"])

    def test_harm_confirmation(self):
        flagged = self.ci(r=ci_rows(effect={("runs", 2, "wall"): 1.008, ("text", 1, "wall"): 1.008}, sd=0.001))
        again = self.ci(r=ci_rows(effect={("runs", 2, "wall"): 1.008}, sd=0.001, seed=9))
        v = tg.confirm(flagged, again)
        self.assertEqual(v, {"valid": True, "confirmed": ["runs/2"], "unresolved": ["text/1"]})
        # (an invalid confirmation run confirms nothing and settles nothing)
        bad = self.ci(res=result(head="0" * 40))
        self.assertFalse(tg.confirm(flagged, bad)["valid"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
