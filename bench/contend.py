"""Which of the decode's stages slow down as more decodes share the machine (#39).

The parallel decoder's CPU cost a block rises 3-5x from 2 to 10 workers at a constant
clock. This says where: N copies of examples/phases.rs (the decode's stages timed one by
one on one thread: the scanner, MTF, the T-vector build, the walk with its segment
assembly, RLE1, the CRC) run at once, each pinned to a physical core of its own, so they
share the L3 and the memory system as N workers do (each copy's own scanner included,
which the parallel decoder runs once). Each stage's time a byte against one copy alone,
the counts in a rotated order, `--reps` times.

`just bench-contend [--copies 1,2,4,6,8,10] [--reps 2] [--passes 2]` writes
contend-<UTC time>.json and a summary.
"""

import argparse
import datetime
import json
import os
import re
import statistics
import subprocess
import sys

from imc import CORPUS, run

PHASES = "target/release/examples/phases"
GHZ = 1000  # phases' "c/B" at 1000 GHz: picoseconds a byte
STAGES = [
    "scanner (headers, Huffman)",
    "MTF + RLE2",
    "T-vector build",
    "walk, room (used)",
    "segment assembly",
    "RLE1",
    "CRC",
    "total",
]


def cores():
    """one CPU a physical core, in core order (the first of each sibling list)"""
    seen, out = set(), []
    base = "/sys/devices/system/cpu"
    cpus = sorted(int(d[3:]) for d in os.listdir(base) if re.fullmatch(r"cpu\d+", d))
    allowed = os.sched_getaffinity(0)
    for c in cpus:
        try:
            sib = open(f"{base}/cpu{c}/topology/thread_siblings_list").read().strip()
        except OSError:
            sib = str(c)
        if sib not in seen and c in allowed:
            seen.add(sib)
            out.append(c)
    return out


def parse(text):
    """phases' rows: stage -> picoseconds a byte of output"""
    got = {}
    for line in text.splitlines():
        for s in STAGES:
            m = re.match(rf"\s+{re.escape(s)}\s+([\d.]+) c/B", line)
            if m:
                got[s] = float(m[1])
    return got


def copies(n, cpus, passes, file):
    procs = [
        subprocess.Popen(
            ["taskset", "-c", str(cpus[i]), PHASES, file, str(passes), str(GHZ)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        for i in range(n)
    ]
    out = []
    for p in procs:
        o, e = p.communicate()
        if p.returncode:
            sys.exit(f"bench-contend: phases failed:\n{e}")
        out.append(parse(o))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--copies", default="1,2,4,6,8,10")
    ap.add_argument("--reps", type=int, default=2)
    ap.add_argument("--passes", type=int, default=2, help="phases' passes (fastest kept)")
    ap.add_argument("--file", default=CORPUS, help="the corpus (default the benchmark's)")
    ap.add_argument("--note", default="", help="what was closed or running, for the record")
    a = ap.parse_args()
    root = run(["git", "rev-parse", "--show-toplevel"], check=True).stdout.strip()
    os.chdir(root)
    for p in [PHASES, a.file]:
        if not os.path.exists(p):
            sys.exit(f"bench-contend: no {p}; `just bench-contend` builds it")
    cpus = cores()
    ns = [int(x) for x in a.copies.split(",")]
    if max(ns) > len(cpus):
        sys.exit(f"bench-contend: {max(ns)} copies but {len(cpus)} physical cores allowed")
    if 1 not in ns:
        ns = [1, *ns]
    utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    runs = []
    for rep in range(a.reps):
        for n in ns[rep % len(ns) :] + ns[: rep % len(ns)]:
            print(f"bench-contend: rep {rep}, {n} copies at once", flush=True)
            for i, got in enumerate(copies(n, cpus, a.passes, a.file)):
                runs.append({"rep": rep, "copies": n, "copy": i, "cpu": cpus[i], **got})
    med = {
        n: {s: statistics.median(r[s] for r in runs if r["copies"] == n and s in r) for s in STAGES}
        for n in ns
    }
    print(f"\n== bench-contend {utc}  note: {a.note or '-'}  (CPUs {cpus[: max(ns)]})")
    print("ps a byte of output, each stage (median over copies and reps), and x one copy")
    print(f"{'stage':<28}" + "".join(f"{n:>14}" for n in ns))
    for s in STAGES:
        base = med[1][s]
        print(f"{s:<28}" + "".join(f"{med[n][s]:7.0f} {med[n][s] / base:5.2f}x" for n in ns))
    print("added ps a byte against one copy, by stage (share of the total added)")
    for n in ns[1:]:
        add = {s: med[n][s] - med[1][s] for s in STAGES if s not in ("total", "segment assembly")}
        tot = med[n]["total"] - med[1]["total"]
        print(
            f"  {n:>2} copies: "
            + ", ".join(f"{s.split(' (')[0]} {add[s]:+.0f} ({add[s] / tot:.0%})" for s in add)
        )
    with open(f"contend-{utc}.json", "w") as f:
        json.dump(
            {
                "meta": {
                    "utc": utc,
                    "note": a.note,
                    "commit": run(["git", "rev-parse", "HEAD"]).stdout.strip(),
                    "cpus": cpus,
                    "passes": a.passes,
                    "reps": a.reps,
                    "unit": "picoseconds a byte of output (phases' c/B at 1000 GHz)",
                },
                "median": med,
                "runs": runs,
            },
            f,
            indent=1,
        )
    print(f"\nbench-contend: wrote contend-{utc}.json")


if __name__ == "__main__":
    main()
