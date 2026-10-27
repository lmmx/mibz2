"""The workstation's clock, temperature and package power beside the decode (#39).

`just bench-thermal [ARGS]` runs it after bench/corpus.sh has built the occupancy example
and the corpus:

    python3 bench/thermal.py [--occupancy 6,8] [--threads 4,6,8,20] [--seconds 90]
                             [--cool 30] [--interval 0.2] [--note TEXT] [--out PATH] [STEP ...]

`turbostat` samples the whole package every --interval seconds throughout (busy clock,
package temperature, package power). Steps, both by default:
- occupancy: bench/occupancy.sh at the --occupancy thread counts, each run joined to the
  samples taken while it ran, so a slow run and a fast one can be compared by clock and
  temperature (the instrument's own result file is written as usual);
- sustained: at each --threads count in turn, after --cool seconds of idle,
  back-to-back decodes for --seconds, as a release decodes for an hour: throughput, clock,
  temperature and power over the first 10 s and over the last 30 s.

Everything goes to one JSON file (thermal-<UTC time>.json) and a summary on stdout. Needs
turbostat (linux-tools) and sudo.
"""

import argparse
import datetime
import glob
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

from imc import CORPUS, EXE, run

COLUMNS = ["Time_Of_Day_Seconds", "Busy%", "Bzy_MHz", "PkgTmp", "PkgWatt"]
RUN = re.compile(r"^r(\d+) t=(\d+)\s+(\S*)\s+(off|timeline|full)\s+wall\s+([\d.]+) s")


class Turbostat:
    """`sudo turbostat` in the background, its samples read back when stopped"""

    def __init__(self, interval):
        self.dir = tempfile.mkdtemp()
        self.out = os.path.join(self.dir, "turbostat.tsv")
        self.pid = os.path.join(self.dir, "pid")
        argv = "turbostat --quiet --Summary --interval {} --show {} --out {}".format(
            interval, ",".join(COLUMNS), self.out
        )
        self.proc = subprocess.Popen(
            ["sudo", "sh", "-c", f"echo $$ > {self.pid} && exec {argv}"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
        )
        deadline = time.monotonic() + 10
        while not self.samples() and time.monotonic() < deadline:
            if self.proc.poll() is not None:
                sys.exit(f"bench-thermal: turbostat failed:\n{self.proc.stderr.read()}")
            time.sleep(0.1)
        got = self.samples()
        if not got or any(c not in got[0] for c in COLUMNS):
            have = ", ".join(got[0]) if got else "none"
            self.stop()
            sys.exit(f"bench-thermal: turbostat gave no samples of {COLUMNS} in 10 s ({have})")

    def samples(self):
        try:
            with open(self.out) as f:
                text = f.read()
        except OSError:
            return []
        rows, head = [], None
        for line in text.splitlines():
            f = line.split()
            if not f:
                continue
            try:
                vals = [float(x) for x in f]
            except ValueError:
                head = f
                continue
            if head and len(vals) == len(head):
                rows.append(dict(zip(head, vals)))
        return rows

    def stop(self):
        try:
            with open(self.pid) as f:
                pid = f.read().strip()
            run(["sudo", "kill", "-INT", pid])
        except OSError:
            pass
        try:
            self.proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.proc.kill()
        rows = self.samples()
        shutil.rmtree(self.dir, ignore_errors=True)
        return rows


def during(samples, start, end):
    """the samples taken over [start, end] (each sample covers the interval before it),
    else the one nearest the middle; their clock and power means, the temperature's max"""
    inside = [s for s in samples if start < s["Time_Of_Day_Seconds"] <= end + 0.05]
    if not inside and samples:
        mid = (start + end) / 2
        inside = [min(samples, key=lambda s: abs(s["Time_Of_Day_Seconds"] - mid))]
    if not inside:
        return {"samples": 0}
    return {
        "samples": len(inside),
        "mhz": statistics.fmean(s["Bzy_MHz"] for s in inside),
        "busy": statistics.fmean(s["Busy%"] for s in inside),
        "watts": statistics.fmean(s["PkgWatt"] for s in inside),
        "temp_max": max(s["PkgTmp"] for s in inside),
    }


def occupancy(threads):
    """bench/occupancy.sh with its progress lines timestamped as they come"""
    proc = subprocess.Popen(
        ["bash", "bench/occupancy.sh", "--threads", threads],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    runs = []
    for line in proc.stdout:
        now = time.time()
        print(line, end="", flush=True)
        m = RUN.match(line)
        if m:
            wall = float(m[5])
            runs.append(
                {
                    "round": int(m[1]),
                    "threads": int(m[2]),
                    "placement": m[3],
                    "mode": m[4],
                    "wall": wall,
                    "start": now - wall,
                    "end": now,
                }
            )
    if proc.wait():
        sys.exit("bench-thermal: bench/occupancy.sh failed")
    return runs


def sustained(t, seconds):
    """back-to-back decodes at t threads for `seconds`, each one's line timestamped"""
    child = [os.path.abspath(EXE), "--child", os.path.abspath(CORPUS), str(t), str(2 * t), "0"]
    loop = 'end=$(($(date +%s) + $0)); while [ "$(date +%s)" -lt "$end" ]; do "$@" || exit 1; done'
    proc = subprocess.Popen(
        ["sh", "-c", loop, str(seconds), *child], stdout=subprocess.PIPE, text=True, bufsize=1
    )
    t0, decodes = time.time(), []
    for line in proc.stdout:
        now = time.time()
        kv = dict(w.split("=", 1) for w in line.split() if "=" in w)
        if kv.get("err", "-") != "-":
            sys.exit(f"bench-thermal: the decode failed: {kv['err']}")
        wall = float(kv["wall"])
        decodes.append(
            {"at": now - t0, "start": now - wall, "end": now, "wall": wall, "out": int(kv["out"])}
        )
    if proc.wait():
        sys.exit("bench-thermal: the decode failed")
    return decodes


def window(decodes, samples, lo, hi):
    """throughput (bytes over decode time) and the machine's state over decodes ending in
    [lo, hi) seconds into the run"""
    mine = [d for d in decodes if lo <= d["at"] < hi]
    if not mine:
        return None
    w = sum(d["wall"] for d in mine)
    state = during(samples, mine[0]["start"], mine[-1]["end"])
    return {"decodes": len(mine), "mb_s": sum(d["out"] for d in mine) / w / 1e6, **state}


def fmt(s):
    if not s or not s.get("samples"):
        return "     -        -      -"
    return f"{s['mhz']:6.0f}  {s['temp_max']:5.0f}  {s['watts']:5.0f}"


def report(res):
    print(f"\n== bench-thermal {res['meta']['utc']}  note: {res['meta'].get('note') or '-'}")
    if "occupancy" in res:
        print("occupancy runs        wall    MB/s   busy MHz  max C  watts")
        rows = res["occupancy"]
        for r in rows:
            print(
                f"r{r['round']} t={r['threads']:<3} {r['mode']:<8}  {r['wall']:.3f}"
                f"  {r['mb_s']:6.0f}   {fmt(r['state'])}"
            )
        print("by thread count: the slower half of the runs against the faster half")
        for t in sorted({r["threads"] for r in rows}):
            mine = [r for r in rows if r["threads"] == t and r["state"].get("samples")]
            if len(mine) < 4:
                continue
            ranked = sorted(mine, key=lambda r: r["wall"])
            fast, slow = ranked[: len(ranked) // 2], ranked[len(ranked) // 2 :]

            def mean(xs, k):
                return statistics.fmean(r["state"][k] for r in xs)

            print(
                f"  t={t:<3} fast {statistics.fmean(r['mb_s'] for r in fast):4.0f} MB/s at"
                f" {mean(fast, 'mhz'):4.0f} MHz, {mean(fast, 'temp_max'):3.0f} C;"
                f" slow {statistics.fmean(r['mb_s'] for r in slow):4.0f} MB/s at"
                f" {mean(slow, 'mhz'):4.0f} MHz, {mean(slow, 'temp_max'):3.0f} C"
            )
    if "sustained" in res:
        print(
            "sustained   threads   first 10 s: MB/s   MHz  max C  watts | last 30 s: MB/s"
            "   MHz  max C  watts"
        )
        for s in res["sustained"]:
            a, b = s["first"], s["last"]
            print(
                f"{'':12}{s['threads']:7}   {a['mb_s'] if a else 0:16.0f}  {fmt(a)} |"
                f" {b['mb_s'] if b else 0:15.0f}  {fmt(b)}"
            )


def main():
    steps = ["occupancy", "sustained"]
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("steps", nargs="*", choices=[[], *steps], default=[])
    ap.add_argument("--occupancy", default="6,8", help="the occupancy step's thread counts")
    ap.add_argument("--threads", default="4,6,8,20", help="the sustained step's")
    ap.add_argument("--seconds", type=int, default=90, help="each sustained decode's length")
    ap.add_argument("--cool", type=float, default=30, help="idle seconds before each")
    ap.add_argument("--interval", type=float, default=0.2, help="turbostat's, in seconds")
    ap.add_argument("--note", default="", help="what was closed or running, for the record")
    ap.add_argument("--out", default=None)
    a = ap.parse_args()
    todo = a.steps or steps
    root = run(["git", "rev-parse", "--show-toplevel"], check=True).stdout.strip()
    os.chdir(root)
    for p in [EXE, CORPUS]:
        if not os.path.exists(p):
            sys.exit(f"bench-thermal: no {p}; run bash bench/corpus.sh (just bench-thermal does)")
    utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = a.out or f"thermal-{utc}.json"
    print("bench-thermal: turbostat needs sudo")
    run(["sudo", "-v"])
    run(["sudo", "modprobe", "msr"])
    if not run(["sudo", "sh", "-c", "command -v turbostat"]).stdout.strip():
        sys.exit("bench-thermal: needs turbostat (linux-tools) on root's PATH")
    thp = {}
    for k in ["enabled", "defrag"]:
        try:
            with open(f"/sys/kernel/mm/transparent_hugepage/{k}") as f:
                thp[k] = f.read().strip()
        except OSError:
            thp[k] = None
    res = {
        "meta": {
            "utc": utc,
            "note": a.note,
            "commit": run(["git", "rev-parse", "HEAD"]).stdout.strip(),
            "turbostat_interval": a.interval,
            "transparent_hugepage": thp,
            "steps": todo,
        }
    }
    ts = Turbostat(a.interval)
    try:
        if "occupancy" in todo:
            print(f"bench-thermal: occupancy at {a.occupancy}", flush=True)
            before = set(glob.glob("occupancy-*.json"))
            res["occupancy"] = occupancy(a.occupancy)
            made = sorted(set(glob.glob("occupancy-*.json")) - before)
            res["occupancy_file"] = made[-1] if made else None
        if "sustained" in todo:
            threads = [int(t) for t in a.threads.split(",")]
            res["sustained_decodes"] = {}
            for t in threads:
                print(
                    f"bench-thermal: {a.cool:g} s idle, then {a.seconds} s at {t} threads",
                    flush=True,
                )
                run(["sudo", "-v"])
                time.sleep(a.cool)
                res["sustained_decodes"][t] = sustained(t, a.seconds)
    finally:
        samples = ts.stop()
    res["samples"] = samples
    out_bytes = None
    if res.get("occupancy_file"):
        with open(res["occupancy_file"]) as f:
            out_bytes = json.load(f).get("output_bytes")
    for r in res.get("occupancy", []):
        r["mb_s"] = out_bytes / r["wall"] / 1e6 if out_bytes and r["wall"] else 0
        r["state"] = during(samples, r["start"], r["end"])
    if "sustained_decodes" in res:
        res["sustained"] = [
            {
                "threads": t,
                "first": window(d, samples, 0, 10),
                "last": window(d, samples, max(0, a.seconds - 30), a.seconds + 5),
            }
            for t, d in res["sustained_decodes"].items()
        ]
    with open(out, "w") as f:
        json.dump(res, f, indent=1)
    report(res)
    print(f"\nbench-thermal: wrote {out} ({len(samples)} turbostat samples)")


if __name__ == "__main__":
    main()
