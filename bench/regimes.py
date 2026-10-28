"""Where the workstation's slow decodes come from (#39): the same decode, interleaved.

The occupancy instrument's runs at one thread count fall into fast and slow ones, a slow
one costing about 1.5x the CPU a block; bench-imc's process-scoped core readings (the same
decode, as root under `perf stat`, back to back) never do. This runs that decode in each
way, interleaved by round in a rotated order, so whatever differs is named by the arm that
differs:

- plain: the decode as the occupancy instrument runs it (this user, no perf);
- perf: the same under `perf stat -e cycles:u,ref-cycles:u,instructions:u` (this user;
  needs kernel.perf_event_paranoid <= 2, else the arm is left out and said so);
- sudo: as root without perf (`sudo` alone), which separates running as root from counting;
- sudo-perf: as bench-imc's core step runs it (root, `sudo perf stat`, user and kernel);

each with the occupancy instrument's 0.15 s gap before a run, and with none. Each run's
wall time comes from the decode's own line; where perf counts it, its clock against the
reference clock (cycles / ref-cycles) and IPC, so a slow run is a lower clock or more
cycles at the same clock.

`just bench-regimes [--threads 6] [--rounds 12] [--note TEXT] [--out PATH]` writes
regimes-<UTC time>.json and a summary.
"""

import argparse
import datetime
import json
import os
import statistics
import subprocess
import sys
import tempfile
import time

from imc import CORPUS, EXE, parse, run

EVENTS = {
    "perf": ["cycles:u", "ref-cycles:u", "instructions:u"],
    "sudo-perf": ["cycles", "ref-cycles", "instructions"],
}


def once(arm, t):
    """one decode at t threads in this arm: its line's fields, and its counters if any"""
    child = [os.path.abspath(EXE), "--child", os.path.abspath(CORPUS), str(t), str(2 * t), "0"]
    with tempfile.TemporaryDirectory() as d:
        out = os.path.join(d, "perf.csv")
        if arm == "plain":
            argv = child
        elif arm == "sudo":
            argv = ["sudo", *child]
        else:
            ev = EVENTS[arm]
            argv = ["perf", "stat", "-x,", "-o", out, "-e", ",".join(ev), "--", *child]
            argv = ["sudo", *argv] if arm == "sudo-perf" else argv
        r = subprocess.run(argv, capture_output=True, text=True)
        if r.returncode:
            sys.exit(f"bench-regimes: {arm} failed:\n{r.stderr}")
        line = next((ln for ln in r.stdout.splitlines() if ln.startswith("wall=")), "")
        kv = dict(w.split("=", 1) for w in line.split() if "=" in w)
        if kv.get("err", "-") != "-" or "wall" not in kv:
            sys.exit(f"bench-regimes: the decode failed: {line or r.stdout}")
        res = {"wall": float(kv["wall"]), "cpu": float(kv["cpu"]), "out": int(kv["out"])}
        if arm in EVENTS:
            with open(out) as f:
                got = parse(f.read(), EVENTS[arm])[None]
            c = {k.split(":")[0]: v for k, v in got.items()}
            res |= {
                "cycles": c["cycles"],
                "ref_cycles": c["ref-cycles"],
                "instructions": c["instructions"],
                "clock_ratio": c["cycles"] / c["ref-cycles"],
                "ipc": c["instructions"] / c["cycles"],
            }
    return res


def perf_allowed():
    """whether this user may count its own process's user-space cycles"""
    r = subprocess.run(
        ["perf", "stat", "-x,", "-e", "cycles:u", "--", "true"], capture_output=True, text=True
    )
    return r.returncode == 0 and "<not" not in r.stderr


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--threads", type=int, default=6)
    ap.add_argument("--rounds", type=int, default=12)
    ap.add_argument("--gap", type=float, default=0.15, help="the occupancy instrument's")
    ap.add_argument("--note", default="", help="what was closed or running, for the record")
    ap.add_argument("--out", default=None)
    a = ap.parse_args()
    root = run(["git", "rev-parse", "--show-toplevel"], check=True).stdout.strip()
    os.chdir(root)
    for p in [EXE, CORPUS]:
        if not os.path.exists(p):
            sys.exit(f"bench-regimes: no {p}; run bash bench/corpus.sh (just bench-regimes does)")
    arms = ["plain", "sudo", "sudo-perf"]
    paranoid = (run(["cat", "/proc/sys/kernel/perf_event_paranoid"]).stdout or "?").strip()
    if perf_allowed():
        arms.insert(1, "perf")
    else:
        print(
            f"bench-regimes: perf_event_paranoid is {paranoid}, so this user can't count its"
            " own process: the perf arm is left out"
        )
    print("bench-regimes: the sudo and sudo-perf arms need sudo")
    run(["sudo", "-v"])
    utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = a.out or f"regimes-{utc}.json"
    cells = [(arm, gap) for arm in arms for gap in [a.gap, 0.0]]
    runs = []
    for r in range(a.rounds):
        run(["sudo", "-v"])
        for arm, gap in cells[r % len(cells) :] + cells[: r % len(cells)]:
            time.sleep(gap)
            x = once(arm, a.threads)
            runs.append({"round": r, "arm": arm, "gap": gap, **x})
            extra = (
                f"  clock {x['clock_ratio']:.3f}  IPC {x['ipc']:.2f}" if "clock_ratio" in x else ""
            )
            print(f"r{r} {arm:<9} gap {gap:.2f}  wall {x['wall']:.3f} s{extra}", flush=True)
    res = {
        "meta": {
            "utc": utc,
            "note": a.note,
            "commit": run(["git", "rev-parse", "HEAD"]).stdout.strip(),
            "threads": a.threads,
            "rounds": a.rounds,
            "perf_event_paranoid": paranoid,
            "arms": arms,
        },
        "runs": runs,
    }
    with open(out, "w") as f:
        json.dump(res, f, indent=1)
    # a run is slow when it takes over 1.25x the fastest quarter's median wall, all arms
    walls = sorted(x["wall"] for x in runs)
    fast = statistics.median(walls[: max(1, len(walls) // 4)])
    print(f"\n== bench-regimes {utc}  {a.threads} threads  note: {a.note or '-'}")
    print(f"slow: over 1.25x the fastest quarter's median wall ({fast:.3f} s)")
    print("arm        gap    MB/s median  wall min-max      slow   clock fast/slow   IPC fast/slow")
    for arm, gap in cells:
        mine = [x for x in runs if x["arm"] == arm and x["gap"] == gap]
        slow = [x for x in mine if x["wall"] > 1.25 * fast]
        quick = [x for x in mine if x["wall"] <= 1.25 * fast]

        def m(xs, k):
            v = [x[k] for x in xs if k in x]
            return f"{statistics.fmean(v):.3f}" if v else "  -  "

        mbs = statistics.median(x["out"] / x["wall"] / 1e6 for x in mine)
        print(
            f"{arm:<10} {gap:.2f}  {mbs:11.0f}  {min(x['wall'] for x in mine):.3f}-"
            f"{max(x['wall'] for x in mine):.3f} s  {len(slow):2d}/{len(mine):<2d}"
            f"  {m(quick, 'clock_ratio')} / {m(slow, 'clock_ratio')}"
            f"   {m(quick, 'ipc')} / {m(slow, 'ipc')}"
        )
    print(f"\nbench-regimes: wrote {out}")


if __name__ == "__main__":
    main()
