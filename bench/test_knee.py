"""bench/knee.py driven for real with stand-ins: a stub `perf` (perf stat's CSV, per process and
system-wide), a stub `ab` (`child` and `stream`) and a stub occupancy child. The parsers on
perf's and occupancy's lines, each hypothesis's rule on both sides of its thresholds, and the
VOID cases (outputs that differ, a cell with no clean run).

    python3 bench/test_knee.py
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
import knee  # noqa: E402

# perf: runs the command after `--`, passes its stdout on, and writes each event's count. The
# memory controller's (system-wide, -a) and the proxy's bytes follow the inner record: a
# decode moves STUB_FRAC of 20 GB/s over its wall, a stream repetition 1 GB in 0.05 s (20
# GB/s) plus 0.5 GB of first touch; `sleep` (idle) moves nothing
PERF = r"""#!/usr/bin/env python3
import json, os, subprocess, sys
a = sys.argv[1:]
out = a[a.index("-o") + 1]
events = a[a.index("-e") + 1].split(",")
cmd = a[a.index("--") + 1:]
p = subprocess.run(cmd, capture_output=True, text=True)
sys.stdout.write(p.stdout)
rec = {}
for line in p.stdout.strip().splitlines()[-1:]:
    try:
        rec = json.loads(line)
    except ValueError:
        pass
if "stream" in rec:
    b = rec["reps"] * 1e9 + 5e8
elif "wall" in rec:
    b = float(os.environ.get("STUB_FRAC", "0.9")) * 20e9 * rec["wall"]
else:
    b = 0.0
with open(out, "w") as f:
    f.write("# started on a stub\n\n")
    for e in events:
        if e.startswith("uncore_imc/"):
            # half read, half written, in MiB as perf scales them
            f.write(f"{b / 2 / 2**20:.6f},MiB,{e},1000,100.00,,\n")
        elif e == "longest_lat_cache.miss":
            f.write(f"{int(b / 64)},,{e},1000,100.00,,\n")
        elif e == "minor-faults":
            f.write(f"2000,,{e},1000,100.00,,\n")
        else:
            f.write(f"12345,,{e},1000,100.00,,\n")
sys.exit(p.returncode)
"""

# ab: `child ARM T FILE` (the wall a fixed part and a per-byte part, cheaper per byte at 12
# and 20; the file's size from its name) and `stream`
AB = r"""#!/usr/bin/env python3
import json, os, sys
a = sys.argv[1:]
if a[0] == "stream":
    reps = int(a[a.index("--reps") + 1])
    t = int(a[a.index("--threads") + 1])
    k = a[a.index("--kernel") + 1]
    print(json.dumps({"stream": k, "threads": t, "reps": reps, "kernel_s": 0.05 * reps,
                      "best_gbs_wa": 20.0 * (1 if k == "triad" else 0.9), "init_s": 0.1}))
    sys.exit(0)
t, path = int(a[2]), a[3]
out = 400_000_000 if "x4" in path else 100_000_000
fixed = float(os.environ.get("STUB_FIXED", "0.005"))
per = 1e-9 if t <= 8 else float(os.environ.get("STUB_PER", "0.95e-9"))
wall = fixed + out * per
h = os.environ.get("STUB_HASH", "same") + ("4" if "x4" in path else "1")
if os.environ.get("STUB_FAIL") == str(t):
    print(json.dumps({"wall": wall, "out": 0, "hash": "", "err": "boom"}))
    sys.exit(0)
tc = f"main={0.3 * wall:.4f} mibz2-scan={0.47 * wall:.4f} " + " ".join(
    f"mibz2-{i}={0.75 * wall:.4f}" for i in range(7))
print(json.dumps({"wall": wall, "cpu": 6 * wall, "rss_kb": 300000, "out": out, "hash": h,
                  "thread_cpu": tc, "err": ""}))
"""

# occupancy --child FILE T SLOTS LEVEL: the shares from the environment, level 1 2% slower
OCC = r"""#!/usr/bin/env python3
import os, sys
a = sys.argv[1:]
level = int(a[4])
wall = 0.1 * (1.0 + 0.02 * level) * float(os.environ.get("STUB_COST", "1.0") if level else 1.0)
kv = {"wall": wall, "cpu": 0.6, "rss_kb": 1, "out": 1, "hash": "h", "err": "-", "plan": "workers7"}
if level:
    kv.update({"t.complete": "yes", "t.w.idle": 0.25, "t.w.decoding": 0.75,
               "t.w.idle.reading": float(os.environ.get("STUB_FRONT", "0.02")),
               "t.w.idle.input": 0.0,
               "t.w.idle.slot_earliest_decoding": float(os.environ.get("STUB_WINDOW", "0.18")),
               "t.w.idle.slot_earliest_ready": float(os.environ.get("STUB_CONSUMER", "0.03")),
               "t.w.idle.outside": 0.02, "t.w.idle.jobs_queued": 0.0, "t.s.reading": 0.47,
               "t.s.input": 0.01, "t.r.waiting": 0.6,
               "t.r.consuming": float(os.environ.get("STUB_MAIN", "0.3")),
               "t.e.head": 0.01, "t.e.tail": 0.004, "t.e.end": 0.1})
print(" ".join(f"{k}={v}" for k, v in kv.items()))
"""

IMC = "uncore_imc/data_reads/,uncore_imc/data_writes/"
CORE = "cycles,longest_lat_cache.miss,minor-faults"


def script(d, name, body):
    p = Path(d) / name
    p.write_text(body)
    p.chmod(p.stat().st_mode | stat.S_IEXEC)
    return str(p)


class Driven(unittest.TestCase):
    """the four subcommands run against the stubs, then the report read"""

    def setUp(self):
        self.d = tempfile.TemporaryDirectory()
        d = self.d.name
        self.perf = script(d, "perf", PERF)
        self.ab = script(d, "ab", AB)
        self.occ = script(d, "occupancy", OCC)
        for n in ("w.bz2", "w-x4.bz2"):
            Path(d, n).write_bytes(b"BZh9")
        self.env = dict(os.environ)
        os.environ["KNEE_PERF"] = self.perf
        self.files = ["--file", f"wikidata={d}/w.bz2", "--file", f"wikidata-x4={d}/w-x4.bz2"]

    def tearDown(self):
        os.environ.clear()
        os.environ.update(self.env)
        self.d.cleanup()

    def p(self, n):
        return os.path.join(self.d.name, n)

    def drive(self, imc=True, core=True, threads="8,12,20", trace=True, stream=True, rounds=3):
        ev = ["--core-events", CORE if core else "", "--imc-events", IMC if imc else ""]
        knee.main(
            [
                "run",
                "--ab",
                self.ab,
                "--out",
                self.p("runs.jsonl"),
                *self.files,
                "--threads",
                threads,
                "--rounds",
                str(rounds),
                *ev,
            ]
        )
        if trace:
            knee.main(
                [
                    "trace",
                    "--occupancy",
                    self.occ,
                    "--out",
                    self.p("trace.jsonl"),
                    *self.files,
                    "--threads",
                    threads,
                    "--rounds",
                    str(rounds),
                ]
            )
        if stream:
            knee.main(
                [
                    "stream",
                    "--ab",
                    self.ab,
                    "--out",
                    self.p("stream.jsonl"),
                    "--threads",
                    "1,4",
                    "--repeat",
                    "2",
                    *ev,
                ]
            )
        if imc:
            knee.main(
                [
                    "idle",
                    "--out",
                    self.p("idle.jsonl"),
                    "--seconds",
                    "0",
                    "--count",
                    "2",
                    "--imc-events",
                    IMC,
                ]
            )

    def report(self):
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = knee.main(
                [
                    "report",
                    "--runs",
                    self.p("runs.jsonl"),
                    "--trace",
                    self.p("trace.jsonl"),
                    "--stream",
                    self.p("stream.jsonl"),
                    "--idle",
                    self.p("idle.jsonl"),
                    "--json",
                    self.p("report.json"),
                ]
            )
        text = buf.getvalue()
        got = None
        if rc == 0:
            with open(self.p("report.json")) as f:
                got = json.load(f)
        return rc, text, got

    def status(self, got):
        return {v["hypothesis"]: v["status"] for v in got["verdicts"]}

    def test_records(self):
        with redirect_stdout(io.StringIO()):
            self.drive()
        runs = knee.load(self.p("runs.jsonl"))
        # 2 files x 3 counts x 3 rounds, unpinned
        self.assertEqual(len(runs), 18)
        r = runs[0]
        self.assertEqual(r["err"], "")
        self.assertIn("uncore_imc/data_reads/", r["perf"]["imc"])
        self.assertEqual(r["perf"]["core"]["minor-faults"]["value"], 2000)
        # MiB scaled back to bytes: half the decode's 0.9 x 20 GB/s over its wall
        want = 0.9 * 20e9 * r["ab"]["wall"] / 2
        self.assertAlmostEqual(
            r["perf"]["imc"]["uncore_imc/data_reads/"]["value"] / want, 1, places=4
        )
        # each round runs every cell once, and the order moves
        firsts = [(x["file"], x["threads"]) for x in runs[::6]]
        self.assertEqual(len(set(firsts)), 3)
        tr = knee.load(self.p("trace.jsonl"))
        self.assertEqual(len(tr), 36)
        self.assertEqual({x["level"] for x in tr}, {0, 1})
        st = knee.load(self.p("stream.jsonl"))
        self.assertEqual(len(st), 2 * 2 * 2 * 2)

    def test_report_reads_each_hypothesis(self):
        with redirect_stdout(io.StringIO()):
            self.drive()
        rc, text, got = self.report()
        self.assertEqual(rc, 0, text)
        # the ceiling by the difference: 10 GB over 0.5 s, the first touch cancelled
        self.assertAlmostEqual(got["ceilings"]["imc"]["bps"] / 20e9, 1, places=6)
        self.assertAlmostEqual(got["ceilings"]["proxy"]["bps"] / 20e9, 1, places=4)
        self.assertEqual(got["ceilings"]["wa"]["kernel"], "triad")
        s = self.status(got)
        self.assertEqual(s["H1"], "SUPPORTED", text)
        self.assertEqual(s["H2"], "REFUTED", text)
        self.assertEqual(s["H2w"], "SUPPORTED", text)
        self.assertEqual(s["H4"], "REFUTED", text)
        self.assertEqual(s["H3"], "REFUTED", text)
        # H1 first, then the window
        self.assertEqual([p["hypothesis"] for p in got["priority"]], ["H1", "H2w"])
        self.assertIn("decides nothing in production", text)
        # the stub's x1 wall: 0.105 s at 8 threads, 0.100 at 20 (0.005 + 0.095): reproduced
        self.assertAlmostEqual(got["plateau"]["ratio"], 0.1 / 0.105, places=6)
        self.assertTrue(got["plateau"]["reproduced"])
        row = got["table"]["wikidata-x4|20|unpinned"]
        self.assertEqual(row["workers"], [7])
        self.assertAlmostEqual(row["busy"]["scan"], 0.47, places=3)
        self.assertEqual(row["minflt_source"], "perf")

    def test_h1_refuted_and_proxy(self):
        os.environ["STUB_FRAC"] = "0.3"
        with redirect_stdout(io.StringIO()):
            self.drive(trace=False)
        rc, text, got = self.report()
        self.assertEqual(rc, 0, text)
        s = self.status(got)
        self.assertEqual(s["H1"], "REFUTED", text)
        # no trace: the waits unmeasured, and nothing supported, so the contention model leads
        self.assertEqual(s["H2"], "UNMEASURED")
        self.assertEqual(s["H4"], "UNMEASURED")
        self.assertIsNone(got["priority"][0]["hypothesis"])

    def test_proxy_only_and_between(self):
        os.environ["STUB_FRAC"] = "0.65"
        with redirect_stdout(io.StringIO()):
            self.drive(imc=False)
        rc, text, got = self.report()
        self.assertEqual(rc, 0, text)
        self.assertNotIn("imc", got["ceilings"])
        self.assertEqual(self.status(got)["H1"], "INCONCLUSIVE (proxy)", text)

    def test_unmeasured_without_perf(self):
        with redirect_stdout(io.StringIO()):
            self.drive(imc=False, core=False)
        rc, text, got = self.report()
        self.assertEqual(rc, 0, text)
        self.assertEqual(self.status(got)["H1"], "UNMEASURED")
        self.assertEqual(
            got["table"]["wikidata|12|unpinned"]["minflt_source"], "rusage (perf's own included)"
        )
        # ab stream's own rate is measured without perf
        self.assertIn("wa", got["ceilings"])

    def test_front_end_consumer_and_fixed_supported(self):
        os.environ.update(
            {
                "STUB_FRONT": "0.15",
                "STUB_WINDOW": "0.0",
                "STUB_CONSUMER": "0.02",
                "STUB_MAIN": "0.95",
                "STUB_FIXED": "0.06",
                "STUB_FRAC": "0.3",
                "STUB_PER": "0.8e-9",
            }
        )
        with redirect_stdout(io.StringIO()):
            self.drive()
        rc, text, got = self.report()
        self.assertEqual(rc, 0, text)
        s = self.status(got)
        self.assertEqual(s["H2"], "SUPPORTED", text)
        self.assertEqual(s["H2w"], "REFUTED", text)
        # main consuming 95% of the span: the consumer binds whatever the idle split
        self.assertEqual(s["H4"], "SUPPORTED", text)
        # 0.06 s fixed of 0.14: 43%, and the larger file speeds up more past 8 threads (1.21
        # against 1.14)
        self.assertEqual(s["H3"], "SUPPORTED", text)
        # ranked by what each could recover at 20 threads: H3 0.43, H2 0.15, H4 0.02
        self.assertEqual([p["hypothesis"] for p in got["priority"]], ["H3", "H2", "H4"])
        # 0.14 against 0.16: 0.875, below the range, so not reproduced (and said)
        self.assertFalse(got["plateau"]["reproduced"])
        self.assertIn("NOT reproduced", text)

    def test_trace_too_costly_is_void(self):
        os.environ["STUB_COST"] = "1.2"
        with redirect_stdout(io.StringIO()):
            self.drive(stream=False, imc=False)
        rc, text, got = self.report()
        self.assertEqual(rc, 0, text)
        s = self.status(got)
        for h in ("H2", "H2w", "H4"):
            self.assertEqual(s[h], "VOID", text)

    def test_outputs_differ_void(self):
        with redirect_stdout(io.StringIO()):
            self.drive(trace=False, stream=False, imc=False, rounds=2)
            os.environ["STUB_HASH"] = "other"
            self.drive(trace=False, stream=False, imc=False, rounds=1)
        rc, text, _ = self.report()
        self.assertEqual(rc, 1)
        self.assertIn("outputs differ", text)

    def test_cell_without_a_clean_run_void(self):
        os.environ["STUB_FAIL"] = "12"
        with redirect_stdout(io.StringIO()):
            self.drive(trace=False, stream=False, imc=False, rounds=1)
        rc, text, _ = self.report()
        self.assertEqual(rc, 1)
        self.assertIn("no clean run", text)

    def test_pinned_past_the_cores_recorded(self):
        with redirect_stdout(io.StringIO()):
            knee.main(
                [
                    "run",
                    "--ab",
                    self.ab,
                    "--out",
                    self.p("runs.jsonl"),
                    *self.files,
                    "--threads",
                    "100000",
                    "--pinned",
                    "100000",
                    "--rounds",
                    "1",
                ]
            )
        recs = knee.load(self.p("runs.jsonl"))
        pinned = [r for r in recs if r["placement"] == "pinned"]
        self.assertEqual(len(pinned), 2)
        self.assertTrue(all("fewer than" in r["err"] for r in pinned))


class Parsers(unittest.TestCase):
    def test_perf_csv(self):
        text = (
            "# started\n\n"
            "1024.5,MiB,uncore_imc/data_reads/,2000,100.00,,\n"
            "512,MiB,uncore_imc/data_reads/,2000,100.00,,\n"
            "<not counted>,,uncore_imc/data_writes/,0,0.00,,\n"
        )
        got = knee.parse_perf(text, imc=True)
        self.assertEqual(got["uncore_imc/data_reads/"]["value"], (1024.5 + 512) * 2**20)
        self.assertIsNone(got["uncore_imc/data_writes/"]["value"])
        self.assertIsNone(knee.imc_bytes({"imc": got}))
        core = knee.parse_perf("100,,longest_lat_cache.miss,50,50.00,,\n7,,cycles,100,100.00,,\n")
        self.assertEqual(
            knee.value({"core": core}, "core", knee.PROXY), (100.0, "longest_lat_cache.miss")
        )
        # multiplexed: refused where a full count is asked for
        self.assertEqual(knee.value({"core": core}, "core", knee.PROXY, full=True), (None, None))
        bad = knee.parse_perf("5,furlongs,uncore_imc/data_reads/,1,100.00,,\n", imc=True)
        self.assertIsNone(bad["uncore_imc/data_reads/"]["value"])

    def test_kv_and_threads(self):
        kv = knee.parse_kv("wall=0.1 err=- plan=workers7 t.w.idle=0.250000 t.problem=- junk")
        self.assertEqual(kv["wall"], 0.1)
        self.assertEqual(kv["plan"], "workers7")
        self.assertEqual(kv["t.w.idle"], 0.25)
        n, s = knee.thread_shares(
            "main=0.02 mibz2-scan=0.05 mibz2-read=0.0 mibz2-0=0.08 mibz2-1=0.06 mibz2-coop-1=0.5",
            0.1,
        )
        self.assertEqual(n, 2)
        self.assertAlmostEqual(s["workers"], 0.7)
        self.assertAlmostEqual(s["scan"], 0.5)
        self.assertAlmostEqual(s["main"], 0.2)

    def test_order(self):
        xs = list("abc")
        self.assertEqual(knee.order(xs, 0), list("abc"))
        self.assertEqual(knee.order(xs, 1), list("bca"))
        self.assertEqual(knee.order(xs, 3), list("cba"))
        # over 2n rounds each item takes each position equally often
        for item in xs:
            pos = [knee.order(xs, r).index(item) for r in range(6)]
            self.assertEqual(sorted(pos), [0, 0, 1, 1, 2, 2])

    def test_wrap(self):
        cmd, files = knee.wrap(
            ["ab", "child"], "0,1", ["cycles"], ["uncore_imc/data_reads/"], "sudo -n", "/t"
        )
        self.assertEqual(cmd[:2], ["sudo", "-n"])
        self.assertIn("-a", cmd[:6])
        i = cmd.index("--")
        inner = cmd[i + 1 :]
        self.assertEqual(inner[:2], [os.environ.get("KNEE_PERF", "perf"), "stat"])
        self.assertNotIn("-a", inner)
        self.assertEqual(inner[-5:], ["taskset", "-c", "0,1", "ab", "child"])
        self.assertEqual(set(files), {"core", "imc"})
        bare, none = knee.wrap(["ab"], None, [], [], "", "/t")
        self.assertEqual((bare, none), (["ab"], {}))


if __name__ == "__main__":
    unittest.main(verbosity=2)
