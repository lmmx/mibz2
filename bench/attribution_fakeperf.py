#!/usr/bin/env python3
"""A stand-in for `perf`, for bench/attribution_run.sh's self-test only: it runs the command
it is given (so the decode, its pinning and its exit status are real), then writes fixed,
synthetic counts in perf's own formats (`stat -x,`, `evlist`, `report --stdio`). Every
figure it writes is SYNTHETIC, chosen to exercise each rule's arithmetic, never a
measurement. Supported: stat, record, evlist, report, --version.
"""
import os
import subprocess
import sys

argv = sys.argv[1:]
if not argv or argv[0] == "--version":
    print("perf version SYNTHETIC (bench/attribution_fakeperf.py)")
    sys.exit(0)
sub, rest = argv[0], argv[1:]


def opt(name):
    return rest[rest.index(name) + 1] if name in rest else None


def command():
    return rest[rest.index("--") + 1:] if "--" in rest else []


def workers(cmd):
    try:
        return int(cmd[-1])
    except (ValueError, IndexError):
        return 1


def size(cmd):
    try:
        return os.path.getsize(cmd[-2])
    except (OSError, IndexError):
        return 1


if sub == "stat":
    cmd = command()
    p = subprocess.run(cmd, stdout=subprocess.DEVNULL) if cmd else None
    w, b = workers(cmd), size(cmd) * 20  # about the decoded bytes
    f4 = 1.0 if w == 1 else 1.3  # the CPU-s/GB ratio the rules see
    g = 4.5 if w == 1 else 4.2  # effective GHz: 7% lower at four workers
    task_ms = 3.0e-9 * b * f4 * 1e3
    cyc = task_ms / 1e3 * g * 1e9
    vals = {"cycles": cyc, "ref-cycles": cyc / g * 3.7, "instructions": 7.0 * b, "task-clock": task_ms,
            "cycle_activity.stalls_l3_miss": cyc * (0.15 if w == 1 else 0.30),
            "cycle_activity.stalls_mem_any": cyc * 0.4, "cycle_activity.stalls_total": cyc * 0.6,
            "mem_load_retired.l3_miss": b * 0.01 * w, "mem_load_retired.l3_hit": b * 0.05,
            "l1d_pend_miss.pending": cyc * (3 if w == 1 else 5), "l1d_pend_miss.pending_cycles": cyc * 0.5,
            "l1d_pend_miss.fb_full": cyc * (0.01 if w == 1 else 0.05),
            "offcore_requests.all_data_rd": b * 0.05, "offcore_requests_outstanding.all_data_rd": b * 5,
            "idq_uops_not_delivered.core": cyc * 0.3, "uops_issued.any": cyc * 1.2, "uops_retired.retire_slots": cyc * 1.1,
            "int_misc.recovery_cycles": cyc * 0.01,
            "uncore_imc/data_reads/": 100.0 * w, "uncore_imc/data_writes/": 50.0 * w}
    spec = opt("-e").strip("{}").split(",")
    with open(opt("-o"), "w") as f:
        f.write("# SYNTHETIC\n")
        for e in spec:
            unit = "msec" if e == "task-clock" else ("MiB" if e.startswith("uncore") else "")
            f.write(f"{vals.get(e, 1.0):.2f},{unit},{e},1000,100.00,,\n")
    sys.exit(p.returncode if p else 0)
if sub == "record":
    cmd = command()
    p = subprocess.run(cmd, stdout=subprocess.DEVNULL)
    out = opt("-o")
    with open(out, "w") as f:
        f.write(f"SYNTHETIC\n{opt('-e')}\n{workers(cmd)}\n")
    sys.exit(p.returncode)
if sub == "evlist":
    print(open(opt("-i")).read().splitlines()[1])
    sys.exit(0)
if sub == "report":
    lines = open(opt("-i")).read().splitlines()
    ev, w = lines[1], int(lines[2])
    # build + walk: 55% of cycles at one worker, 70% at four; the stall samples more so
    bw = (0.55 if w == 1 else 0.70) if ev == "cycles" else (0.80 if w == 1 else 0.90)
    rows = [("mibz2-0", "_RNvNtCs0_5mibz23bwt5build", bw * 0.25), ("mibz2-0", "_RINvNtCs0_5mibz23bwt13room_segmentsKj20_Kh1_EB4_", bw * 0.75),
            ("mibz2-0", "_RNvNtCs0_5mibz23mtf8mtf_avx2", (1 - bw) * 0.5), ("mibz2-0", "__memmove_evex_unaligned_erms", (1 - bw) * 0.1),
            ("mibz2-scan", "_RNvNtCs0_5mibz25block9read_item", (1 - bw) * 0.3), ("attribution_dec", "__memmove_evex_unaligned_erms", (1 - bw) * 0.1)]
    for comm, sym, s in rows:
        print(f"    {100 * s:.2f}%  {comm:<16} [.] {sym}")
    sys.exit(0)
sys.exit(f"fakeperf: {sub} not supported")
