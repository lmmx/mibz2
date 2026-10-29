"""The hardware counters behind the cooperative grid (lmmx/mibz2#76; research only).

The method is the one registered before the confirmation, in
docs/journal/2026-10-28-cooperative-inversion.md §8.2:

- **Each arm alone, pinned as in the grid**, as `coop grid FILE 1 ARM` and `coop grid FILE
  21 ARM` under `perf stat`. The difference of the two is 20 timed rounds, without the setup
  (reading the corpus, MTF) or the untimed check round.
- **Each counter group in a pass of its own**, beside the fixed counters (cycles,
  instructions, ref-cycles), so a ratio's events share one interval and nothing is
  multiplexed. The groups are bench/imc.py's (run on the i9 before), plus the misses in
  flight per core, the fill buffers, cross-core snoops and the top-down slots.
  The memory controller's reads and writes are counted machine-wide (`perf stat -a`).
- **The guards:**
  - every run is repeated --reps times, the arms and groups rotated, the short and long
    runs' order alternating;
  - the raw counts of every run are kept (the .jsonl), not only the differences;
  - a reading whose events were counted less than all of the time (multiplexed) is dropped;
  - a difference is flagged when its spread over the repeats exceeds 10% of it;
  - each long run's own MB/s is checked against the confirmed runs', so a counted run is a
    typical one.
- **The phase attribution:** with --record (the default), one `perf record` each of p4, 4x1
  and s4x1 (libsais), the hottest symbols in user cycles: libsais's build, walk and copies
  against ours.

Needs sudo for perf (the memory controller's events count the whole machine). Writes
coop-counters-<UTC time>.md, and every run's raw counts in coop-counters-<UTC time>.jsonl.

  just bench-coop-counters
  just bench-coop-counters --arms p4,3x3 --reps 1 --groups llc --no-imc --no-record

The quick form checks that the events count on this machine, in about a minute.
"""

import argparse
import datetime
import json
import math
import os
import shlex
import statistics
import subprocess
import sys
import tempfile
import time

import imc

CORPUS = "corpora/wikidata.bz2"
# Skylake-family names (the i9-10900K's), two to a pass as bench/imc.py's are
EXTRA = {
    # L1D misses in flight: their sum over cycles, and the cycles with any
    "mlp": ["l1d_pend_miss.pending", "l1d_pend_miss.pending_cycles"],
    # cycles a load missed L1D for want of a fill buffer, and the loads that missed L1D
    "fill": ["l1d_pend_miss.fb_full", "mem_load_retired.l1_miss"],
    # loads served from another core's modified line, and the stores (#81's handoff)
    "snoop": ["mem_load_l3_hit_retired.xsnp_hitm", "mem_inst_retired.all_stores"],
    # top-down level 1 (Yasin 2014), in two passes: slots issued and retired, then slots
    # the front end left empty and the cycles lost recovering from bad speculation
    "slots": ["uops_issued.any", "uops_retired.retire_slots"],
    "frontend": ["idq_uops_not_delivered.core", "int_misc.recovery_cycles"],
    # the walks prefetch each lane's next entry (to L1, libsais a round ahead), so the demand
    # loads above see only what the prefetches miss: the software prefetches, and the loads
    # that found their line still arriving (a late prefetch)
    "prefetch": ["sw_prefetch_access.t0", "load_hit_pre.sw_pf"],
    # every data read past L2, prefetches included, by where the L3 found it (the offcore
    # response events; the PMU's name differs by perf version, so both are tried and the
    # one that doesn't count here is left out)
    "l3_all": [
        "offcore_response.all_data_rd.l3_hit.any_snoop",
        "offcore_response.all_data_rd.l3_miss.any_snoop",
    ],
    "l3_all_ocr": ["ocr.all_data_rd.l3_hit.any_snoop", "ocr.all_data_rd.l3_miss.any_snoop"],
    # the same two, raw (RAW below), for a perf whose tables carry neither name
    "l3_all_raw": ["ocr_all_data_rd_l3_hit", "ocr_all_data_rd_l3_miss"],
    # the architectural last-level-cache events (perf's generic names, on every Intel core):
    # core requests that missed L2, by whether L3 had them; data, code, RFOs and
    # prefetches together, so a second, independent prefetch-inclusive share
    "llc_all": ["cache-references", "cache-misses"],
    # checks on the raw encoding (journal §10.6): the scheme against perf's own name for the
    # same demand-read event; the raw pair without bit 7 (PF_L3_DATA_RD) beside
    # offcore_requests.all_data_rd, which the PMU describes as excluding L3 prefetches; and
    # bit 7 alone
    "ocr_named": ["offcore_response.demand_data_rd.l3_hit.any_snoop", "ocr_dmnd_l3_hit"],
    "ocr_no_l3pf": ["ocr_nol3pf_l3_hit", "ocr_nol3pf_l3_miss", "offcore_requests.all_data_rd"],
    "ocr_l3pf": ["ocr_l3pf_l3_hit", "ocr_l3pf_l3_miss"],
}
# events given by encoding, under the alias perf reports them by: OFFCORE_RESPONSE (event
# 0xB7, umask 0x01) with the response MSR's value from Intel's Skylake-client tables:
# ALL_DATA_RD (demand data reads, L2 and L3 data prefetches, L1D and software prefetches:
# 0x491) by supplier, L3_HIT (0x3C0000) or L3_MISS (0x3C000000), any snoop (0x3F80000000).
# The kernel moves the second onto the other response MSR (event 0xBB) itself
RAW = {
    "ocr_dmnd_l3_hit": "cpu/event=0xb7,umask=0x1,offcore_rsp=0x3F803C0001,name=ocr_dmnd_l3_hit/",
    "ocr_nol3pf_l3_hit": "cpu/event=0xb7,umask=0x1,offcore_rsp=0x3F803C0411,name=ocr_nol3pf_l3_hit/",
    "ocr_nol3pf_l3_miss": "cpu/event=0xb7,umask=0x1,offcore_rsp=0x3FBC000411,name=ocr_nol3pf_l3_miss/",
    "ocr_l3pf_l3_hit": "cpu/event=0xb7,umask=0x1,offcore_rsp=0x3F803C0080,name=ocr_l3pf_l3_hit/",
    "ocr_l3pf_l3_miss": "cpu/event=0xb7,umask=0x1,offcore_rsp=0x3FBC000080,name=ocr_l3pf_l3_miss/",
    "ocr_all_data_rd_l3_hit": "cpu/event=0xb7,umask=0x1,offcore_rsp=0x3F803C0491,name=ocr_all_data_rd_l3_hit/",
    "ocr_all_data_rd_l3_miss": "cpu/event=0xb7,umask=0x1,offcore_rsp=0x3FBC000491,name=ocr_all_data_rd_l3_miss/",
}
GROUPS = {**imc.GROUPS, **EXTRA}
# the medians of the confirmation and exploratory runs (coop-20261029T08*), MB/s
TYPICAL = {
    "p4": 1470,
    "p6": 1097,
    "p8": 701,
    "p10": 553,
    "4x1": 1268,
    "4x2": 1808,
    "3x3": 1945,
    "s4x1": 1570,
    "s6x1": 1248,
    "s8x1": 886,
    "s10x1": 742,
    "s3x3": 2355,
}
SPREAD = 0.10
COVERED = 99.5


def threads(arm):
    s = arm.lstrip("pswrqnmfxylg").split("@")[0]
    n, _, k = s.partition("x")
    return int(n) * int(k or 1)


def build():
    r = subprocess.run(["bash", "bench/coop.sh", "--build-only"], capture_output=True, text=True)
    if r.returncode:
        sys.exit(f"bench-coop-counters: the build failed:\n{r.stderr[-3000:]}")
    line = r.stdout.strip().splitlines()[-1]
    return dict(kv.split("=", 1) for kv in line.split())


def counted(perf, events, cmd, system):
    """`cmd` under `perf stat`: its wall, the parsed counts (or None) and perf's raw CSV"""
    with tempfile.TemporaryDirectory() as d:
        out = os.path.join(d, "perf.csv")
        argv = [*perf, "stat", "-x,", "-o", out, "-e", ",".join(RAW.get(e, e) for e in events)]
        argv += ["-a"] if system else []
        t0 = time.monotonic()
        r = subprocess.run([*argv, "--", *cmd], capture_output=True, text=True)
        wall = time.monotonic() - t0
        text = open(out).read() if os.path.exists(out) else ""
    if r.returncode:
        return wall, None, text, r.stderr[-2000:]
    try:
        rows = imc.parse(text, events)
    except SystemExit as e:
        return wall, None, text, str(e)
    return wall, rows[None], text, ""


def grid(env_cpus, binary, rounds, arm, jsonl):
    return [
        "env",
        f"COOP_CPUS={env_cpus}",
        f"COOP_JSONL={jsonl}",
        "taskset",
        "-c",
        env_cpus,
        binary,
        "grid",
        os.environ.get("COOP_FILE", CORPUS),
        str(rounds),
        arm,
    ]


def timed(jsonl):
    """the grid's own rounds (it writes only the timed ones): (round, wall_s, bytes)"""
    if not os.path.exists(jsonl):
        return []
    out = []
    for line in open(jsonl):
        if line.strip():
            r = json.loads(line)
            out.append((r["round"], r["wall_s"], r["mb_s"] * r["wall_s"] * 1e6))
    return out


def check_events(perf, groups, use_imc):
    """the groups whose events count here: a pass of each over `true`"""
    ok, left = {}, {}
    for g, evs in groups.items():
        _, rows, _, err = counted(perf, imc.CORE + evs, ["true"], system=False)
        (ok if rows is not None else left)[g] = evs if rows is not None else err.strip()[:300]
    imc_ok = False
    if use_imc:
        _, rows, _, err = counted(perf, imc.IMC, ["sleep", "0.1"], system=True)
        imc_ok = rows is not None
        if not imc_ok:
            left["imc"] = err.strip()[:300]
    return ok, imc_ok, left


def median(xs):
    return statistics.median(xs) if xs else math.nan


def differences(records):
    """per (arm, group, rep): the long run less the short, with the bytes and wall between"""
    by = {}
    for r in records:
        by.setdefault((r["arm"], r["group"], r["rep"]), {})[r["rounds"]] = r
    out = []
    for (arm, group, rep), runs in sorted(by.items()):
        if len(runs) != 2 or any(x["counts"] is None for x in runs.values()):
            continue
        short, long_ = (runs[k] for k in sorted(runs))
        cov = min(min(x["counted_pct"].values(), default=100.0) for x in (short, long_))
        d = {e: long_["counts"][e] - short["counts"][e] for e in long_["counts"]}
        out.append(
            {
                "arm": arm,
                "group": group,
                "rep": rep,
                "rounds": long_["rounds"] - short["rounds"],
                "bytes": sum(t[2] for t in long_["timed"]) - sum(t[2] for t in short["timed"]),
                "timed_wall": sum(t[1] for t in long_["timed"]) - sum(t[1] for t in short["timed"]),
                "covered": cov >= COVERED,
                "counts": d,
            }
        )
    return out


# a derived figure: (its label, its group, f(counts, bytes, threads) -> value)
FIGURES = [
    ("instructions / B", "llc", lambda c, b, t: c["instructions"] / b),
    ("IPC", "llc", lambda c, b, t: c["instructions"] / c["cycles"]),
    ("cycles / B (all threads)", "llc", lambda c, b, t: c["cycles"] / b),
    (
        "data reads past L2 / B (prefetches included)",
        "reads",
        lambda c, b, t: c["offcore_requests.all_data_rd"] / b,
    ),
    (
        "L3 hit share of data reads, prefetches included",
        "l3_all",
        lambda c, b, t: c["offcore_response.all_data_rd.l3_hit.any_snoop"]
        / (
            c["offcore_response.all_data_rd.l3_hit.any_snoop"]
            + c["offcore_response.all_data_rd.l3_miss.any_snoop"]
        ),
    ),
    (
        "L3 hit share of data reads, prefetches included",
        "l3_all_ocr",
        lambda c, b, t: c["ocr.all_data_rd.l3_hit.any_snoop"]
        / (c["ocr.all_data_rd.l3_hit.any_snoop"] + c["ocr.all_data_rd.l3_miss.any_snoop"]),
    ),
    (
        "L3 hit share of data reads, prefetches included",
        "l3_all_raw",
        lambda c, b, t: c["ocr_all_data_rd_l3_hit"]
        / (c["ocr_all_data_rd_l3_hit"] + c["ocr_all_data_rd_l3_miss"]),
    ),
    (
        "data reads past L2 that missed L3 / B (offcore response)",
        "l3_all_raw",
        lambda c, b, t: c["ocr_all_data_rd_l3_miss"] / b,
    ),
    (
        "raw demand-read L3 hits / perf's named event for them (same pass)",
        "ocr_named",
        lambda c, b, t: c["ocr_dmnd_l3_hit"]
        / c["offcore_response.demand_data_rd.l3_hit.any_snoop"],
    ),
    (
        "raw data reads past L2 without L3 prefetches / offcore_requests.all_data_rd (same pass)",
        "ocr_no_l3pf",
        lambda c, b, t: (c["ocr_nol3pf_l3_hit"] + c["ocr_nol3pf_l3_miss"])
        / c["offcore_requests.all_data_rd"],
    ),
    (
        "L3 prefetch reads (PF_L3_DATA_RD) / B",
        "ocr_l3pf",
        lambda c, b, t: (c["ocr_l3pf_l3_hit"] + c["ocr_l3pf_l3_miss"]) / b,
    ),
    (
        "L3 hit share of L3 prefetch reads",
        "ocr_l3pf",
        lambda c, b, t: c["ocr_l3pf_l3_hit"] / (c["ocr_l3pf_l3_hit"] + c["ocr_l3pf_l3_miss"]),
    ),
    (
        "L3 hit share of all requests past L2 (architectural LLC events)",
        "llc_all",
        lambda c, b, t: 1 - c["cache-misses"] / c["cache-references"],
    ),
    (
        "requests past L2 / B (architectural LLC events)",
        "llc_all",
        lambda c, b, t: c["cache-references"] / b,
    ),
    (
        "software prefetches executed / B (useful or not)",
        "prefetch",
        lambda c, b, t: c["sw_prefetch_access.t0"] / b,
    ),
    (
        "late prefetches (a load found its line still arriving) / prefetch",
        "prefetch",
        lambda c, b, t: c["load_hit_pre.sw_pf"] / c["sw_prefetch_access.t0"],
    ),
    (
        "L3 hit share of demand loads beyond L2 (what prefetch missed)",
        "llc",
        lambda c, b, t: c["mem_load_retired.l3_hit"]
        / (c["mem_load_retired.l3_hit"] + c["mem_load_retired.l3_miss"]),
    ),
    ("L3-missing loads / KB", "llc", lambda c, b, t: c["mem_load_retired.l3_miss"] / (b / 1024)),
    (
        "L3-missing demand reads / KB",
        "beyond_llc",
        lambda c, b, t: c["offcore_requests.l3_miss_demand_data_rd"] / (b / 1024),
    ),
    (
        "cycles an L3-missing read waits",
        "beyond_llc",
        lambda c, b, t: c["offcore_requests_outstanding.l3_miss_demand_data_rd"]
        / c["offcore_requests.l3_miss_demand_data_rd"],
    ),
    (
        "L3-missing reads in flight, machine-wide",
        "beyond_llc",
        lambda c, b, t: c["offcore_requests_outstanding.l3_miss_demand_data_rd"] / c["cycles"] * t,
    ),
    (
        "data reads in flight past L2, machine-wide",
        "reads",
        lambda c, b, t: c["offcore_requests_outstanding.all_data_rd"] / c["cycles"] * t,
    ),
    (
        "cycles a read past L2 waits",
        "reads",
        lambda c, b, t: c["offcore_requests_outstanding.all_data_rd"]
        / c["offcore_requests.all_data_rd"],
    ),
    (
        "share of cycles stalled on an L3 miss",
        "queue",
        lambda c, b, t: c["cycle_activity.stalls_l3_miss"] / c["cycles"],
    ),
    (
        "share of cycles with the super queue full",
        "queue",
        lambda c, b, t: c["offcore_requests_buffer.sq_full"] / c["cycles"],
    ),
    (
        "share of cycles walking page tables",
        "tlb",
        lambda c, b, t: c["dtlb_load_misses.walk_active"] / c["cycles"],
    ),
    (
        "L1D demand misses in flight, while any (not the prefetches)",
        "mlp",
        lambda c, b, t: c["l1d_pend_miss.pending"] / c["l1d_pend_miss.pending_cycles"],
    ),
    (
        "share of cycles with an L1D demand miss pending",
        "mlp",
        lambda c, b, t: c["l1d_pend_miss.pending_cycles"] / c["cycles"],
    ),
    # fb_full counts requests (loads, stores, software prefetches) that found no free fill
    # buffer, not cycles: a rate, not a share of time
    (
        "requests that found the fill buffers full / cycle",
        "fill",
        lambda c, b, t: c["l1d_pend_miss.fb_full"] / c["cycles"],
    ),
    (
        "requests that found the fill buffers full / B",
        "fill",
        lambda c, b, t: c["l1d_pend_miss.fb_full"] / b,
    ),
    ("L1D-missing loads / B", "fill", lambda c, b, t: c["mem_load_retired.l1_miss"] / b),
    (
        "loads from another core's modified line / KB",
        "snoop",
        lambda c, b, t: c["mem_load_l3_hit_retired.xsnp_hitm"] / (b / 1024),
    ),
    ("stores / B", "snoop", lambda c, b, t: c["mem_inst_retired.all_stores"] / b),
    (
        "top-down: retiring",
        "slots",
        lambda c, b, t: c["uops_retired.retire_slots"] / (4 * c["cycles"]),
    ),
    (
        "top-down: front end (its own pass)",
        "frontend",
        lambda c, b, t: c["idq_uops_not_delivered.core"] / (4 * c["cycles"]),
    ),
    ("DRAM bytes read / output B", "imc", lambda c, b, t: c["uncore_imc/data_reads/"] / b),
    ("DRAM bytes written / output B", "imc", lambda c, b, t: c["uncore_imc/data_writes/"] / b),
]


# each figure's unit: counts of requests, cycles and occupancy are not interchangeable
UNITS = {
    "instructions / B": "instructions per output byte, all threads",
    "IPC": "instructions per core cycle",
    "cycles / B (all threads)": "core cycles per output byte, summed over threads",
    "data reads past L2 / B (prefetches included)": "requests per output byte",
    "L3 hit share of data reads, prefetches included": "a fraction of requests (offcore responses)",
    "data reads past L2 that missed L3 / B (offcore response)": "requests per output byte",
    "L3 hit share of all requests past L2 (architectural LLC events)": "a fraction of requests (data, code, RFO, prefetch)",
    "requests past L2 / B (architectural LLC events)": "requests per output byte",
    "raw demand-read L3 hits / perf's named event for them (same pass)": "a ratio of two counts of the same event (1 if the encoding is right)",
    "raw data reads past L2 without L3 prefetches / offcore_requests.all_data_rd (same pass)": "a ratio of requests",
    "L3 prefetch reads (PF_L3_DATA_RD) / B": "requests per output byte",
    "L3 hit share of L3 prefetch reads": "a fraction of requests",
    "software prefetches executed / B (useful or not)": "instructions per output byte",
    "late prefetches (a load found its line still arriving) / prefetch": "loads per software prefetch",
    "L3 hit share of demand loads beyond L2 (what prefetch missed)": "a fraction of retired loads",
    "L3-missing loads / KB": "retired loads per KB of output",
    "L3-missing demand reads / KB": "requests per KB of output",
    "cycles an L3-missing read waits": "cycles (occupancy over requests, Little's law)",
    "L3-missing reads in flight, machine-wide": "requests (mean occupancy per thread x threads)",
    "data reads in flight past L2, machine-wide": "requests (mean occupancy per thread x threads)",
    "cycles a read past L2 waits": "cycles (occupancy over requests, Little's law)",
    "share of cycles stalled on an L3 miss": "a fraction of cycles",
    "share of cycles with the super queue full": "a fraction of cycles",
    "share of cycles walking page tables": "a fraction of cycles",
    "L1D demand misses in flight, while any (not the prefetches)": "requests (occupancy over the cycles with any)",
    "share of cycles with an L1D demand miss pending": "a fraction of cycles",
    "requests that found the fill buffers full / cycle": "requests per cycle (a rate, not a share of time)",
    "requests that found the fill buffers full / B": "requests per output byte",
    "L1D-missing loads / B": "retired loads per output byte",
    "loads from another core's modified line / KB": "retired loads per KB of output",
    "stores / B": "retired stores per output byte",
    "top-down: retiring": "a fraction of issue slots (4 a cycle)",
    "top-down: front end (its own pass)": "a fraction of issue slots (4 a cycle)",
    "DRAM bytes read / output B": "bytes per output byte, machine-wide",
    "DRAM bytes written / output B": "bytes per output byte, machine-wide",
}


def descriptions(events):
    """each event's description from this machine's own PMU tables (`perf list -j`)"""
    r = subprocess.run(["perf", "list", "-j", "--long-desc"], capture_output=True, text=True)
    try:
        listed = json.loads(r.stdout)
    except json.JSONDecodeError:
        return {}
    out = {}
    for e in listed:
        name = str(e.get("EventName", "")).lower()
        if name in events:
            out[name] = (e.get("PublicDescription") or e.get("BriefDescription") or "").strip()
    return out


def figures(diffs, arms):
    """per arm, each figure's median over the repeats, and its spread (max - min) / median"""
    out = {}
    for arm in arms:
        for label, group, f in FIGURES:
            vals = []
            for d in diffs:
                if d["arm"] != arm or d["group"] != group or not d["covered"] or d["bytes"] <= 0:
                    continue
                try:
                    vals.append(f(d["counts"], d["bytes"], threads(arm)))
                except (KeyError, ZeroDivisionError):
                    pass
            if vals:
                m = median(vals)
                spread = (max(vals) - min(vals)) / abs(m) if m else math.nan
                out[(arm, label)] = (m, spread, len(vals))
    return out


def fmt(x):
    if x is None or (isinstance(x, float) and math.isnan(x)):
        return ""
    a = abs(x)
    return f"{x:.3f}" if a < 10 else f"{x:.1f}" if a < 1000 else f"{x:.0f}"


def record_symbols(perf, binary, cpus, arms, top):
    """one `perf record` of each arm's grid (11 rounds), its hottest symbols in user cycles"""
    out = {}
    for arm in arms:
        with tempfile.TemporaryDirectory() as d:
            data = os.path.join(d, "perf.data")
            cmd = grid(cpus, binary, 11, arm, os.path.join(d, "r.jsonl"))
            r = subprocess.run(
                [*perf, "record", "-q", "-e", "cycles:u", "-o", data, "--", *cmd],
                capture_output=True,
                text=True,
            )
            if r.returncode:
                out[arm] = f"(perf record failed: {r.stderr.strip()[-300:]})"
                continue
            rep = subprocess.run(
                [*perf, "report", "-i", data, "--no-children", "--sort", "sym", "--stdio", "-q"],
                capture_output=True,
                text=True,
            )
            lines = [x for x in rep.stdout.splitlines() if x.strip()][:top]
            out[arm] = "\n".join(lines)
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--arms", default=None, help="default p4,p8,4x1,4x2,3x3 (+s4x1,s3x3)")
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--rounds", type=int, default=21, help="the long run's; the short is 1")
    ap.add_argument("--groups", default=None, help=f"any of {','.join(GROUPS)}")
    ap.add_argument("--no-imc", action="store_true")
    ap.add_argument("--no-record", action="store_true")
    ap.add_argument(
        "--record-arms",
        default="p4,4x1,s4x1",
        help="the arms `perf record` profiles by symbol (an l arm splits the library's build and walk)",
    )
    ap.add_argument("--perf", default="sudo perf")
    ap.add_argument("--note", default="")
    ap.add_argument("--file", default=CORPUS, help="the corpus (default the bench's)")
    a = ap.parse_args()
    os.environ["COOP_FILE"] = a.file
    perf = shlex.split(a.perf)

    b = build()
    binary, cpus, libsais = b["bin"], b["cpus"], b["libsais"] == "1"
    cores = len(cpus.split(","))
    arms = a.arms.split(",") if a.arms else ["p4", "p8", "4x1", "4x2", "3x3"]
    if not a.arms and libsais:
        arms += ["s4x1", "s3x3"]
    if not libsais and any(x.startswith("s") for x in arms):
        sys.exit("bench-coop-counters: s arms need libsais (gcc with OpenMP)")
    too_big = [x for x in arms if threads(x) > cores]
    if too_big:
        sys.exit(f"bench-coop-counters: {too_big} need more than this machine's {cores} cores")
    want = {g: GROUPS[g] for g in (a.groups.split(",") if a.groups else GROUPS)}

    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    base = f"coop-counters-{stamp}"
    print(
        f"bench-coop-counters: {cores} cores ({cpus}), libsais {'built' if libsais else 'not built'}"
    )
    print("bench-coop-counters: perf needs sudo for the memory controller's events")
    groups, use_imc, left = check_events(perf, want, not a.no_imc)
    for g, why in left.items():
        print(f"bench-coop-counters: group {g} left out: {why}")
    passes = list(groups) + (["imc"] if use_imc else [])
    n_runs = a.reps * len(arms) * len(passes) * 2
    print(
        f"bench-coop-counters: {len(arms)} arms x {len(passes)} passes x {a.reps} repeats x "
        f"(1 and {a.rounds} rounds) = {n_runs} runs",
        flush=True,
    )

    records = []
    raw = open(f"{base}.jsonl", "w")
    done = 0
    t_start = time.monotonic()
    for rep in range(a.reps):
        order = arms[rep % len(arms) :] + arms[: rep % len(arms)]
        for arm in order:
            ps = passes[rep % len(passes) :] + passes[: rep % len(passes)]
            for i, g in enumerate(ps):
                system = g == "imc"
                events = imc.IMC if system else imc.CORE + groups[g]
                rounds = [1, a.rounds] if (rep + i) % 2 == 0 else [a.rounds, 1]
                for n in rounds:
                    with tempfile.TemporaryDirectory() as d:
                        jl = os.path.join(d, "rounds.jsonl")
                        wall, counts, text, err = counted(
                            perf, events, grid(cpus, binary, n, arm, jl), system
                        )
                        rounds_done = timed(jl)
                    rec = {
                        "rep": rep,
                        "arm": arm,
                        "group": g,
                        "rounds": n,
                        "system_wide": system,
                        "events": events,
                        "wall_s": wall,
                        "counts": None if counts is None else {e: counts[e] for e in events},
                        "counted_pct": {} if counts is None else counts.get("_counted_pct", {}),
                        "run_ns": None if counts is None else counts.get("_run_ns"),
                        "timed": rounds_done,
                        "error": err,
                        "perf_csv": text,
                    }
                    records.append(rec)
                    raw.write(json.dumps(rec) + "\n")
                    raw.flush()
                    done += 1
                    if err:
                        print(f"bench-coop-counters: {arm} {g} {n} rounds: {err[:200]}")
                if done % 20 < 2:
                    el = time.monotonic() - t_start
                    print(
                        f"bench-coop-counters: {done}/{n_runs} runs, "
                        f"about {el / done * (n_runs - done) / 60:.0f} min left",
                        flush=True,
                    )
    raw.close()

    symbols = {}
    if not a.no_record:
        rec_arms = [x for x in a.record_arms.split(",") if libsais or not x.startswith("s")]
        symbols = record_symbols(
            perf, binary, cpus, [x for x in rec_arms if threads(x) <= cores], 25
        )

    diffs = differences(records)
    fig = figures(diffs, arms)
    with open(f"{base}.md", "w") as md:
        w = lambda s="": print(s, file=md)  # noqa: E731
        w(f"# The cooperative grid's counters ({stamp})")
        w()
        model = next(
            (
                x.split(":", 1)[1].strip()
                for x in open("/proc/cpuinfo")
                if x.startswith("model name")
            ),
            "?",
        )
        w(f"- **Machine:** {model}; {cores} physical cores used ({cpus})")
        w(f"- **Corpus:** {a.file} ({os.path.getsize(a.file)} bytes)")
        w(
            f"- **perf:** {subprocess.run([*perf, '--version'], capture_output=True, text=True).stdout.strip()}"
        )
        w(
            f"- **Commit:** {subprocess.run(['git', 'rev-parse', '--short', 'HEAD'], capture_output=True, text=True).stdout.strip()}; the pilot's kernels unchanged from c2620b8"
        )
        w(
            f"- **Method:** each arm alone, `grid FILE 1 ARM` and `grid FILE {a.rounds} ARM`; the difference, {a.rounds - 1} timed rounds; {a.reps} repeats; one counter pair a pass"
        )
        w(
            f"- **Left out:** {', '.join(f'{g} ({why[:80]})' for g, why in left.items()) or 'nothing'}"
        )
        if a.note:
            w(f"- **Note:** {a.note}")
        w()
        w(
            "## Each counted run against the confirmed runs (its own grid MB/s, median of its long runs)"
        )
        w()
        w("| arm | MB/s while counted | confirmed | ratio |")
        w("|---|--:|--:|--:|")
        for arm in arms:
            mbs = [
                t[2] / t[1] / 1e6
                for r in records
                if r["arm"] == arm and r["rounds"] == a.rounds
                for t in r["timed"]
            ]
            m = median(mbs)
            # the confirmed medians are the bench corpus's: another corpus has none here
            ref = TYPICAL.get(arm) if a.file == CORPUS else None
            w(f"| {arm} | {fmt(m)} | {ref or ''} | {fmt(m / ref) if ref else ''} |")
        w()
        w(
            f"## Per output byte, the difference of the two runs (median of the repeats; † spread over {SPREAD:.0%})"
        )
        w()
        w("| figure | " + " | ".join(arms) + " |")
        w("|---|" + "--:|" * len(arms))
        for label in dict.fromkeys(label for label, _, _ in FIGURES):
            cells = []
            for arm in arms:
                v = fig.get((arm, label))
                cells.append("" if v is None else fmt(v[0]) + (" †" if v[1] > SPREAD else ""))
            if any(cells):
                w(f"| {label} | " + " | ".join(cells) + " |")
        w()
        dropped = [d for d in diffs if not d["covered"]]
        w(
            f"Readings dropped as multiplexed (counted under {COVERED}% of the time): {len(dropped)}."
        )
        w()
        w("Each pass's least coverage over its runs (100: never multiplexed), and its failed runs:")
        w()
        w("| pass | events | least % counted | failed runs |")
        w("|---|---|--:|--:|")
        for p in passes:
            rs = [r for r in records if r["group"] == p]
            pcts = [min(r["counted_pct"].values()) for r in rs if r["counted_pct"]]
            evs = imc.IMC if p == "imc" else groups[p]
            least = fmt(min(pcts)) if pcts else ""
            w(f"| {p} | {', '.join(evs)} | {least} | {sum(r['counts'] is None for r in rs)} |")
        w()
        w("## §8.2's predictions, read mechanically (the journal interprets them)")
        w()

        def g(arm, label):
            v = fig.get((arm, label))
            return None if v is None else v[0]

        for arm in [x for x in ("4x2", "3x3") if x in arms]:
            if "p4" not in arms:
                break
            w(f"**{arm}:**")
            # the prefetch-inclusive shares first; the demand-only one only with a warning,
            # since the walks' prefetches hide most of their traffic from it
            share = next(
                (
                    x
                    for x in (
                        "L3 hit share of data reads, prefetches included",
                        "L3 hit share of all requests past L2 (architectural LLC events)",
                    )
                    if g("p4", x) is not None and g(arm, x) is not None
                ),
                "L3 hit share of demand loads beyond L2 (what prefetch missed)",
            )
            if "demand" in share:
                w(
                    "- **No prefetch-inclusive L3 share counted here,** so the residency line below "
                    "uses the demand-only share, which sees only what the prefetches missed. "
                    "It is not evidence of residency on its own."
                )
            w(f"- the share used: {share}")
            hs, hp4, hp8 = (g(x, share) for x in (arm, "p4", "p8"))
            mk, mp4 = (
                g(arm, "L3-missing demand reads / KB"),
                g("p4", "L3-missing demand reads / KB"),
            )
            fl, fp4 = (
                g(arm, "L3-missing reads in flight, machine-wide"),
                g("p4", "L3-missing reads in flight, machine-wide"),
            )
            oc, op4 = (
                g(arm, "cycles an L3-missing read waits"),
                g("p4", "cycles an L3-missing read waits"),
            )
            dw, dp4 = (
                g(arm, "DRAM bytes written / output B"),
                g("p4", "DRAM bytes written / output B"),
            )
            yn = lambda c: "" if c is None else ("yes" if c else "no")  # noqa: E731
            ok = lambda *xs: all(x is not None for x in xs)  # noqa: E731
            w(
                f"- residency: hit share {fmt(hs)} ≥ p4's {fmt(hp4)}: {yn(hs >= hp4 if ok(hs, hp4) else None)}; above p8's {fmt(hp8)}: {yn(hs > hp8 if ok(hs, hp8) else None)}"
            )
            w(
                f"- residency: L3-missing reads / KB {fmt(mk)} ≤ 1.05 × p4's {fmt(mp4)}: {yn(mk <= 1.05 * mp4 if ok(mk, mp4) else None)}"
            )
            w(
                f"- residency: DRAM writes / B {fmt(dw)} ≤ 1.1 × p4's {fmt(dp4)}: {yn(dw <= 1.1 * dp4 if ok(dw, dp4) else None)}"
            )
            w(
                f"- overlap: L3-missing reads in flight {fmt(fl)} ≥ 1.5 × p4's {fmt(fp4)}: {yn(fl >= 1.5 * fp4 if ok(fl, fp4) else None)}; each waits {fmt(oc)} cycles against p4's {fmt(op4)}"
            )
            w()
        w("## What each figure counts")
        w()
        w("| figure | unit |")
        w("|---|---|")
        for label in dict.fromkeys(label for label, _, _ in FIGURES):
            if any((arm, label) in fig for arm in arms):
                w(f"| {label} | {UNITS.get(label, '')} |")
        w()
        evs = sorted({e for p in passes for e in (imc.IMC if p == "imc" else groups[p])})
        desc = descriptions({e.lower() for e in evs})
        w("Each event as this machine's PMU tables describe it (`perf list -j --long-desc`):")
        w()
        w("| event | description |")
        w("|---|---|")
        for e in evs:
            d = (
                desc.get(e.lower(), "(not in perf list's JSON)")
                .replace("|", "/")
                .replace("\n", " ")
            )
            w(f"| `{e}` | {d} |")
        w()
        if symbols:
            w("## The hottest symbols (one `perf record` each, user cycles, setup included)")
            w()
            for arm, text in symbols.items():
                w(f"**{arm}:**")
                w()
                w("```")
                w(text)
                w("```")
                w()
    print(f"bench-coop-counters: wrote {base}.md and {base}.jsonl")


if __name__ == "__main__":
    main()
