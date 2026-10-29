"""The machine's memory ceiling, measured with the memory controller's own counters.

Runs examples/dram.rs (a walk-shaped random-line probe, and a sequential copy) at 1-10
threads, one per physical core, for --secs each, under
`sudo perf stat -a -D -1 --control fifo:CTL,ACK -e uncore_imc/data_reads/,uncore_imc/data_writes/`:
perf starts with counting off, and the probe turns it on after its setup (the walk's 512 MB
permutation is built by a random shuffle, itself heavy traffic) and off when its measured
seconds end, so the counts are the probe's alone. Writes dram-<UTC time>.md and its .jsonl.

  python3 bench/dram.py [--threads 1,2,4,6,8,10] [--modes walk,copy]
"""

import argparse
import datetime
import json
import os
import subprocess
import sys
import tempfile

EVENTS = ["uncore_imc/data_reads/", "uncore_imc/data_writes/"]
BIN = "target/coop/release/examples/dram"


def cpus_one_a_core():
    seen, out = set(), []
    for c in sorted(__import__("os").sched_getaffinity(0)):
        s = open(f"/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list").read()
        first = int(s.strip().split(",")[0].split("-")[0])
        if first not in seen:
            seen.add(first)
            out.append(first)
    return out


def run(mode, threads, secs, cpus, events=EVENTS, system=True, perf=("sudo", "perf")):
    d = tempfile.mkdtemp(dir=".")
    ctl, ack = os.path.join(d, "ctl"), os.path.join(d, "ack")
    os.mkfifo(ctl)
    os.mkfifo(ack)
    cmd = [*perf, "stat"] + (["-a"] if system else []) + ["-x", ",", "-D", "-1"]
    cmd += ["--control", f"fifo:{ctl},{ack}", "-e", ",".join(events), "--"]
    cmd += ["env", f"DRAM_PERF_CTL={ctl}", f"DRAM_PERF_ACK={ack}"]
    cmd += ["taskset", "-c", ",".join(map(str, cpus[:threads])), BIN, mode, str(threads), str(secs)]
    try:
        try:
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=secs + 180)
        except subprocess.TimeoutExpired:
            sys.exit(
                f"dram: no answer from perf's control fifo in {secs + 180:.0f} s: {' '.join(cmd)}"
            )
    finally:
        for f in (ctl, ack):
            os.unlink(f)
        os.rmdir(d)
    if r.returncode:
        sys.exit(f"dram: {' '.join(cmd)} failed: {r.stderr[-500:]}")
    counts = {}
    for line in r.stderr.splitlines():
        f = line.split(",")
        if len(f) > 2 and f[2] in events:
            v = float(f[0])
            unit = f[1]
            counts[f[2]] = v * (1 << 20) if unit == "MiB" else v * 64 if unit == "" else v
    words = r.stdout.split()
    probe = dict(zip(words[0::2], words[1::2]))
    return counts, probe


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--threads", default="1,2,4,6,8,10")
    ap.add_argument("--modes", default="walk,copy")
    ap.add_argument("--secs", type=float, default=3.0)
    a = ap.parse_args()
    cpus = cpus_one_a_core()
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    rows, raw = [], open(f"dram-{stamp}.jsonl", "w")
    for mode in a.modes.split(","):
        for t in map(int, a.threads.split(",")):
            if t > len(cpus):
                continue
            counts, probe = run(mode, t, a.secs, cpus)
            secs = float(probe["secs"])
            rd = counts[EVENTS[0]] / secs / 1e9
            wr = counts[EVENTS[1]] / secs / 1e9
            rec = {
                "mode": mode,
                "threads": t,
                "secs": secs,
                "read_GBs": rd,
                "write_GBs": wr,
                "counts": counts,
                "probe": probe,
            }
            raw.write(json.dumps(rec) + "\n")
            rows.append(rec)
            print(
                f"dram: {mode} {t} threads: read {rd:.1f} + write {wr:.1f} = {rd + wr:.1f} GB/s",
                flush=True,
            )
    model = next(
        (l.split(":", 1)[1].strip() for l in open("/proc/cpuinfo") if l.startswith("model name")),
        "?",
    )
    with open(f"dram-{stamp}.md", "w") as md:
        md.write(f"# The memory ceiling ({stamp})\n\n- **Machine:** {model}; CPUs {cpus}\n")
        md.write(
            f"- **Method:** examples/dram.rs for {a.secs} s a case under `perf stat -a`, counting "
            "only between the probe's own enable and disable (after its setup): the memory "
            "controller's reads and writes\n\n"
        )
        md.write(
            "| mode | threads | DRAM read GB/s | DRAM write GB/s | total GB/s |\n|---|--:|--:|--:|--:|\n"
        )
        for r in rows:
            md.write(
                f"| {r['mode']} | {r['threads']} | {r['read_GBs']:.1f} | {r['write_GBs']:.1f} | "
                f"{r['read_GBs'] + r['write_GBs']:.1f} |\n"
            )
    print(f"dram: wrote dram-{stamp}.md and dram-{stamp}.jsonl")


if __name__ == "__main__":
    main()
