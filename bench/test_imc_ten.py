"""bench/imc_ten.py driven for real with stand-ins (a stub `perf` and a stub `ab`), and its
arithmetic and verdict on records built by hand: the frozen rule on both sides of each
threshold, the idle traffic taken off per event, Little's law, and every VOID case (the memory
controller unreadable, a failed run, an affinity short of every CPU, outputs that differ).

    python3 bench/test_imc_ten.py
"""

import io
import json
import os
import stat
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import imc_ten  # noqa: E402

# perf: runs the command after `--`, passes its stdout and stderr on, and writes each event's
# count. A decode moves STUB_FRAC_<workers> of 20 GB/s over its wall (half read, half written),
# its L3-miss latency STUB_LAT_<workers> cycles; a stream repetition 1 GB in 0.05 s (20 GB/s)
# plus 0.5 GB of first touch; `sleep` (idle) nothing
PERF = r"""#!/usr/bin/env python3
import json, os, subprocess, sys
a = sys.argv[1:]
out = a[a.index("-o") + 1]
events = a[a.index("-e") + 1].split(",")
cmd = a[a.index("--") + 1:]
p = subprocess.run(cmd, capture_output=True, text=True)
sys.stdout.write(p.stdout)
sys.stderr.write(p.stderr)
rec = {}
for line in p.stdout.strip().splitlines()[-1:]:
    try:
        rec = json.loads(line)
    except ValueError:
        pass
w = cmd[cmd.index("child") + 1].split("-")[2] if "child" in cmd else "0"
if "stream" in rec:
    b = rec["reps"] * 1e9 + 5e8
elif "wall" in rec:
    b = float(os.environ.get("STUB_FRAC_" + w, "0.9")) * 20e9 * rec["wall"]
else:
    b = 0.0
lat = float(os.environ.get("STUB_LAT_" + w, "500"))
with open(out, "w") as f:
    f.write("# started on a stub\n\n")
    for e in events:
        if e.startswith("uncore_imc/"):
            f.write(f"{b / 2 / 2**20:.6f},MiB,{e},1000,100.00,,\n")
        elif e == "offcore_requests.l3_miss_demand_data_rd":
            f.write(f"1000000,,{e},1000,100.00,,\n")
        elif e == "offcore_requests_outstanding.l3_miss_demand_data_rd":
            f.write(f"{int(1000000 * lat)},,{e},1000,100.00,,\n")
        else:
            f.write(f"12345,,{e},1000,100.00,,\n")
sys.exit(p.returncode)
"""

# ab: `child ARM T FILE`, the wall by the arm's workers (7/28 13% faster), and `stream`
AB = r"""#!/usr/bin/env python3
import json, os, sys
a = sys.argv[1:]
if a[0] == "stream":
    reps = int(a[a.index("--reps") + 1])
    t = int(a[a.index("--threads") + 1])
    k = a[a.index("--kernel") + 1]
    print(json.dumps({"stream": k, "threads": t, "reps": reps, "kernel_s": 0.05 * reps,
                      "best_gbs_wa": 20.0, "init_s": 0.1}))
    sys.exit(0)
arm, t, path = a[1], int(a[2]), a[3]
out = 400_000_000 if "x4" in path else 100_000_000
wall = (0.873 if arm == "mibz2-occ-7-28" else 1.0) * out * 1e-9
h = os.environ.get("STUB_HASH_" + arm.split("-")[2], "same") + ("4" if "x4" in path else "1")
if os.environ.get("STUB_FAIL") == arm:
    print(json.dumps({"wall": wall, "out": 0, "hash": "", "err": "boom"}))
    sys.exit(0)
print(json.dumps({"wall": wall, "cpu": 6 * wall, "rss_kb": 300000, "out": out, "hash": h,
                  "thread_cpu": "", "err": ""}))
"""

IMC = "uncore_imc/data_reads/,uncore_imc/data_writes/"
CORE = (
    "cycles,offcore_requests.l3_miss_demand_data_rd,"
    "offcore_requests_outstanding.l3_miss_demand_data_rd"
)


def script(d, name, body):
    p = Path(d) / name
    p.write_text(body)
    p.chmod(p.stat().st_mode | stat.S_IEXEC)
    return str(p)


def report(args):
    buf = io.StringIO()
    with redirect_stdout(buf):
        rc = imc_ten.main(["report", *args])
    return rc, buf.getvalue()


class Driven(unittest.TestCase):
    """the whole path, run, stream, idle and report, through the stubs"""

    def setUp(self):
        self.d = tempfile.mkdtemp()
        self.ab = script(self.d, "ab", AB)
        perf = script(self.d, "perf", PERF)
        self.env = dict(os.environ)
        os.environ["KNEE_PERF"] = perf
        for k in [k for k in os.environ if k.startswith("STUB_")]:
            del os.environ[k]
        self.files = []
        for name in ("wikidata", "wikidata-x4"):
            p = Path(self.d) / f"{name}.bz2"
            p.write_bytes(b"x")
            self.files += ["--file", f"{name}={p}"]

    def tearDown(self):
        os.environ.clear()
        os.environ.update(self.env)

    def drive(self, imc=IMC, core=CORE, rounds=2, cpus=None):
        d = self.d
        ev = ["--core-events", core, "--imc-events", imc]
        with redirect_stdout(io.StringIO()):
            imc_ten.main(["run", "--ab", self.ab, "--out", f"{d}/runs.jsonl", *self.files,
                          "--rounds", str(rounds), *ev])
            imc_ten.knee.main(["stream", "--ab", self.ab, "--out", f"{d}/stream.jsonl",
                               "--threads", "1,2", "--repeat", "1", "--mib", "1", *ev])
            imc_ten.knee.main(["idle", "--out", f"{d}/idle.jsonl", "--seconds", "0.01",
                               "--count", "2", "--imc-events", imc])
        n = len(imc_ten.online()) if cpus is None else cpus
        return report(["--runs", f"{d}/runs.jsonl", "--stream", f"{d}/stream.jsonl",
                       "--idle", f"{d}/idle.jsonl", "--cpus", str(n), "--json", f"{d}/r.json"])

    def test_supported(self):
        os.environ.update(STUB_FRAC_10="0.93", STUB_FRAC_7="0.75", STUB_LAT_10="671",
                          STUB_LAT_7="550")
        rc, out = self.drive()
        self.assertEqual(rc, 0, out)
        self.assertIn("record 52's prediction, on wikidata-x4: SUPPORTED", out)
        rep = json.load(open(f"{self.d}/r.json"))
        rows = rep["files"]["wikidata-x4"]["rows"]
        self.assertAlmostEqual(rep["ceiling_gbs"], 20.0, places=6)
        self.assertAlmostEqual(rows["mibz2-occ-10-40"]["of_ceiling"], 0.93, places=6)
        self.assertAlmostEqual(rows["mibz2-occ-7-28"]["of_ceiling"], 0.75, places=6)
        self.assertAlmostEqual(rows["mibz2-occ-10-40"]["latency"], 671.0, places=3)
        self.assertAlmostEqual(rep["files"]["wikidata-x4"]["latency_rise"], 0.22, places=6)
        # DRAM bytes a byte out: 0.93 x 20 GB/s over 1 GB/s of output = 18.6
        self.assertAlmostEqual(rows["mibz2-occ-10-40"]["dram_per_out"], 18.6, places=6)
        self.assertAlmostEqual(rows["mibz2-occ-10-40"]["reads_per_out"], 9.3, places=6)
        self.assertAlmostEqual(rep["files"]["wikidata-x4"]["wall_paired"]["median"], 0.873, places=6)
        # every arm's wall, GB/s, share, B/out and latency printed
        for s in ("DRAM_GB/s", "of_ceiling", "DRAM_B/out", "L3-miss_latency_cycles", "18.60",
                  "93.0%", "671.0", "+22.0%"):
            self.assertIn(s, out)
        # the runs were unpinned on every online CPU, and say so
        runs = imc_ten.knee.load(f"{self.d}/runs.jsonl")
        self.assertEqual(len(runs), 2 * 2 * 2)
        self.assertTrue(all(r["affinity"] == imc_ten.online() for r in runs))

    def test_refuted_by_dram(self):
        os.environ.update(STUB_FRAC_10="0.79", STUB_LAT_10="700", STUB_LAT_7="500")
        rc, out = self.drive()
        self.assertEqual(rc, 0, out)
        self.assertIn("on wikidata-x4: REFUTED", out)

    def test_imc_unreadable_is_void(self):
        rc, out = self.drive(imc="")
        self.assertEqual(rc, 1)
        self.assertIn("VOID", out)
        self.assertIn("the memory controller is unreadable", out)

    def test_affinity_short_of_the_cpus_is_void(self):
        # the runs are on every online CPU here, but this machine has fewer than 20
        rc, out = self.drive(cpus=len(imc_ten.online()) + 1)
        self.assertEqual(rc, 1)
        self.assertIn("not on all", out)

    def test_failed_run_is_void(self):
        os.environ["STUB_FAIL"] = "mibz2-occ-7-28"
        rc, out = self.drive()
        self.assertEqual(rc, 1)
        self.assertIn("failed", out)

    def test_outputs_differ_is_void(self):
        os.environ["STUB_HASH_7"] = "other"
        rc, out = self.drive()
        self.assertEqual(rc, 1)
        self.assertIn("outputs differ", out)

    def test_latency_unmeasured(self):
        os.environ.update(STUB_FRAC_10="0.93")
        rc, out = self.drive(core="cycles")
        self.assertEqual(rc, 0, out)
        self.assertIn("UNMEASURED", out)
        self.assertIn("on wikidata-x4: INCONCLUSIVE", out)


class Rule(unittest.TestCase):
    """the frozen rule either side of each threshold"""

    def test_thresholds(self):
        v = lambda f, r: imc_ten.verdict(f, r)[0]  # noqa: E731
        self.assertEqual(v(0.88, 0.20), "SUPPORTED")
        self.assertEqual(v(0.95, 0.35), "SUPPORTED")
        self.assertEqual(v(0.8799, 0.25), "INCONCLUSIVE")
        self.assertEqual(v(0.90, 0.1999), "INCONCLUSIVE")
        self.assertEqual(v(0.80, 0.10), "INCONCLUSIVE")
        self.assertEqual(v(0.7999, 0.30), "REFUTED")
        self.assertEqual(v(0.95, 0.0999), "REFUTED")
        self.assertEqual(v(0.95, -0.05), "REFUTED")
        self.assertEqual(v(0.79, None), "REFUTED")
        self.assertEqual(v(0.90, None), "INCONCLUSIVE")
        self.assertEqual(v(None, 0.3), "VOID")


def rec(arm, rnd, wall, reads, writes, req, occ, elapsed=None, file="wikidata-x4", cpus=20):
    perf = {
        "imc": {imc_ten.READS: {"value": reads, "pct": 100.0},
                imc_ten.WRITES: {"value": writes, "pct": 100.0}},
        "core": {imc_ten.LAT_REQ: {"value": req, "pct": 100.0},
                 imc_ten.LAT_OUT: {"value": occ, "pct": 100.0}},
    }
    return {"round": rnd, "file": file, "arm": arm, "threads": 10, "online": list(range(cpus)),
            "affinity": list(range(cpus)), "elapsed_s": elapsed or wall + 0.05, "perf": perf,
            "rc": 0, "err": "", "ab": {"wall": wall, "out": 4e8, "hash": "h", "err": ""}}


class Arithmetic(unittest.TestCase):
    """the table's figures from records built by hand"""

    def test_row(self):
        idle = {imc_ten.READS: 1e8, imc_ten.WRITES: 5e7}  # 0.15 GB/s
        rs = [
            rec("a", 0, 0.40, 6e9, 4e9, 1e6, 6e8, elapsed=0.5),
            rec("a", 1, 0.42, 6.2e9, 4.2e9, 1e6, 7e8, elapsed=0.5),
            rec("a", 2, 0.44, 6.4e9, 4.4e9, 2e6, 1.0e9, elapsed=0.5),
        ]
        r = imc_ten.row(rs, idle)
        self.assertEqual(r["n"], 3)
        self.assertAlmostEqual(r["wall"], 0.42)
        # each run's bytes less 0.15 GB/s x 0.5 s, the median
        self.assertAlmostEqual(r["dram"], 10.4e9 - 0.075e9)
        self.assertAlmostEqual(r["reads"], 6.2e9 - 0.05e9)
        self.assertAlmostEqual(r["writes"], 4.2e9 - 0.025e9)
        self.assertAlmostEqual(r["dram_bps"], (10.4e9 - 0.075e9) / 0.42)
        self.assertAlmostEqual(r["dram_per_out"], (10.4e9 - 0.075e9) / 4e8)
        # Little's law per run (600, 700, 500 cycles), the median
        self.assertAlmostEqual(r["latency"], 600.0)

    def test_idle_rates_per_event(self):
        idle = [{"rc": 0, "elapsed_s": 2.0, "perf": {"imc": {
            imc_ten.READS: {"value": 2e8}, imc_ten.WRITES: {"value": 1e8}}}}] * 3
        got = imc_ten.idle_rates(idle)
        self.assertAlmostEqual(got[imc_ten.READS], 1e8)
        self.assertAlmostEqual(got[imc_ten.WRITES], 5e7)

    def test_paired(self):
        runs = [rec("mibz2-occ-10-40", i, 0.40, 1, 1, 1, 1) for i in range(3)] + [
            rec("mibz2-occ-7-28", i, w, 1, 1, 1, 1) for i, w in enumerate((0.35, 0.36, 0.34))]
        p = imc_ten.paired(runs, "wikidata-x4", ["mibz2-occ-7-28", "mibz2-occ-10-40"],
                           lambda r: r["ab"]["wall"])
        self.assertEqual(p["n"], 3)
        self.assertAlmostEqual(p["median"], 0.875)
        self.assertAlmostEqual(p["min"], 0.85)

    def test_affinity_and_plan(self):
        self.assertEqual(imc_ten.cpu_list("0-3,8,10-11"), [0, 1, 2, 3, 8, 10, 11])
        self.assertEqual(imc_ten.affinity_of("noise\nCpus_allowed_list:\t0-19\n"), list(range(20)))
        self.assertIsNone(imc_ten.affinity_of("nothing"))
        p = imc_ten.plan(["x4", "x1"], ["A", "B"], 2)
        self.assertEqual([(f, a) for _, f, a in p],
                         [("x4", "A"), ("x4", "B"), ("x1", "B"), ("x1", "A"),
                          ("x4", "B"), ("x4", "A"), ("x1", "A"), ("x1", "B")])

    def test_void_on_a_short_affinity(self):
        runs = [rec(a, 0, 0.4, 1e9, 1e9, 1, 1, file=f) for a in imc_ten.ARMS
                for f in ("wikidata-x4", "wikidata")]
        runs[1]["affinity"] = list(range(10))
        v = imc_ten.voids(runs, [{}], list(imc_ten.ARMS), ["wikidata-x4", "wikidata"], 20)
        self.assertEqual(len(v), 1, v)
        self.assertIn("not on all 20 CPUs", v[0])
        runs[1]["affinity"] = list(range(20))
        self.assertEqual(imc_ten.voids(runs, [{}], list(imc_ten.ARMS),
                                       ["wikidata-x4", "wikidata"], 20), [])

    def test_estimate_is_within_budget(self):
        buf = io.StringIO()
        with redirect_stdout(buf):
            imc_ten.main(["estimate"])
        self.assertLessEqual(json.loads(buf.getvalue())["total_min"], 8.0)


if __name__ == "__main__":
    unittest.main()
