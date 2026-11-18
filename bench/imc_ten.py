#!/usr/bin/env python3
"""Ten workers against seven, unpinned, at the memory controller (`just bench-imc-ten`; the
registration, docs/journal/2026-11-18-imc-ten.md, frozen before the data as MONODROMY's record
52, REVIEW 6973478826 on #145). A diagnostic: it scores one prediction and decides nothing in
production. Standard library only; the perf wrappers, the ceiling and the idle traffic are
bench/knee.py's own.

    imc_ten.py run    --ab AB --out RUNS.jsonl --file NAME=PATH.. [--arms A,B] [--threads 10]
                      --rounds N [--core-events E,..] [--imc-events E,..]
    imc_ten.py report --runs RUNS.jsonl --stream STREAM.jsonl --idle IDLE.jsonl
                      [--main wikidata-x4] [--check wikidata] [--json OUT.json]
    imc_ten.py estimate [the recipe's grid]   the i9 time the recipe should take

**run**: `ab child ARM 10 FILE` for the lazy occupancy arms `mibz2-occ-10-40` (10 workers, 40
blocks in flight) and `mibz2-occ-7-28`, on each file, every round both arms on each file, the
arms' order alternating round by round (ABBA over the files), so each round is a pair. Every
run is unpinned on every online CPU: `taskset -c` with the whole online list (as `ab run
--unpinned` sets it), the affinity the child execs with read back from its own
/proc/PID/status (`Cpus_allowed_list`, by a shell that then execs `ab`) and recorded. Each run
is wrapped in `perf stat` of its own core events (the L3-miss latency's pair) and `perf stat -a`
of the memory controller's `uncore_imc/data_reads/` and `data_writes/` (knee.py's `wrap`).

**report**: per arm and file the medians of the wall, DRAM bytes (reads, writes, both; the
idle traffic taken off in proportion to each run's elapsed time, per event), DRAM GB/s (the
median bytes over the median wall, as knee.py's table), its share of the ceiling (`ab stream`,
knee.py's `ceiling` in the memory controller's bytes), DRAM bytes per output byte, and the
L3-miss demand-read latency in core cycles by Little's law
(`offcore_requests_outstanding.l3_miss_demand_data_rd` over `offcore_requests.l3_miss_demand_data_rd`,
bench-miss-cost's counted pair; per process, so `perf_event_paranoid` 1 or less, no root).
Then the verdict on the main file by the frozen rule (`verdict`), the check file's the same
way, reported and not judged.

VOID (exit 1): the memory controller not counted (or no ceiling), any run failed, any run's
affinity not every one of the 20 CPUs, the arms' outputs differ on a file, or a cell with no run.
"""

import argparse
import json
import os
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import knee  # noqa: E402

ARMS = ("mibz2-occ-10-40", "mibz2-occ-7-28")
# the i9-10900K's CPUs: every run's affinity must be all of them
CPUS = 20
# the frozen rule (record 52, REVIEW 6973478826): predicted DRAM 30.7 [29.0, 32.5] GB/s at
# 10/40, at least 88% of the ceiling, and the L3-miss latency +20% or more against 7/28;
# refuted if DRAM is under 80% of the ceiling or the latency rises under 10%
PREDICTED_GBS = (30.7, 29.0, 32.5)
SUPPORT_FRAC, SUPPORT_RISE = 0.88, 0.20
REFUTE_FRAC, REFUTE_RISE = 0.80, 0.10
LAT_REQ = "offcore_requests.l3_miss_demand_data_rd"
LAT_OUT = "offcore_requests_outstanding.l3_miss_demand_data_rd"
READS, WRITES = "uncore_imc/data_reads/", "uncore_imc/data_writes/"
# the shell that reports the affinity it execs `ab` with ($$ is the shell, which becomes ab)
SHIM = 'grep "^Cpus_allowed_list:" /proc/$$/status >&2; exec "$@"'


# ---------------------------------------------------------------------------------------
# running


def cpu_list(s):
    """'0-3,8,10-11' -> [0, 1, 2, 3, 8, 10, 11]"""
    out = []
    for part in (s or "").strip().split(","):
        part = part.strip()
        if not part:
            continue
        a, _, b = part.partition("-")
        out.extend(range(int(a), int(b or a) + 1))
    return sorted(set(out))


def online():
    """the online CPUs, as `ab run --unpinned` takes them"""
    with open("/sys/devices/system/cpu/online") as f:
        return cpu_list(f.read())


def affinity_of(stderr):
    """the CPUs the shim read back (the last `Cpus_allowed_list:` line), or None"""
    got = None
    for line in (stderr or "").splitlines():
        if line.startswith("Cpus_allowed_list:"):
            got = cpu_list(line.split(":", 1)[1])
    return got


def plan(files, arms, rounds):
    """(round, file, arm) in run order: every round each file with both arms, the arms' order
    alternating by round and by file (ABBA), so each round holds one pair a file"""
    out = []
    for r in range(rounds):
        for i, name in enumerate(files):
            order = list(arms) if (r + i) % 2 == 0 else list(reversed(arms))
            out.extend((r, name, arm) for arm in order)
    return out


def cmd_run(a):
    files = dict(knee.file_args(a.file))
    arms = [x for x in a.arms.split(",") if x]
    cpus = online()
    mask = ",".join(map(str, cpus))
    with open(a.out, "a") as out:
        for r, name, arm in plan(list(files), arms, a.rounds):
            cmd = ["sh", "-c", SHIM, "sh", a.ab, "child", arm, str(a.threads), files[name]]
            so, se, rc, el, faults, perf = knee.measured(cmd, mask, a)
            rec = {
                "round": r,
                "file": name,
                "arm": arm,
                "threads": a.threads,
                "online": cpus,
                "affinity": affinity_of(se),
                "elapsed_s": el,
                "faults": faults,
                "perf": perf,
                "rc": rc,
            }
            try:
                rec["ab"] = json.loads(so.strip().splitlines()[-1])
                rec["err"] = rec["ab"].get("err", "") or ("" if rc == 0 else f"exit {rc}")
            except (ValueError, IndexError):
                rec["err"] = f"exit {rc}, no record: {se.strip()[-300:]}"
            out.write(json.dumps(rec) + "\n")
            out.flush()
            print(
                f"imc-ten run: round {r} {name} {arm}: {rec.get('ab', {}).get('wall', 'NA')} s, "
                f"{len(rec['affinity'] or [])} CPUs {rec['err']}",
                flush=True,
            )


# ---------------------------------------------------------------------------------------
# reading


def med(xs):
    xs = [x for x in xs if x is not None]
    return statistics.median(xs) if xs else None


def idle_rates(idle):
    """the memory controller's idle bytes a second, per event (median over the readings)"""
    rates = {}
    for r in idle:
        got = (r.get("perf") or {}).get("imc") or {}
        if r.get("rc") != 0 or not r.get("elapsed_s"):
            continue
        for ev, e in got.items():
            if not ev.startswith("_") and e.get("value") is not None:
                rates.setdefault(ev, []).append(e["value"] / r["elapsed_s"])
    return {ev: med(v) for ev, v in rates.items()}


def imc_event(perf, ev):
    e = ((perf or {}).get("imc") or {}).get(ev)
    return None if not e else e.get("value")


def latency(perf):
    """the L3-miss demand-read latency in core cycles (Little's law), or None"""
    req, _ = knee.value(perf, "core", (LAT_REQ,))
    occ, _ = knee.value(perf, "core", (LAT_OUT,))
    if not req or occ is None:
        return None
    return occ / req


def row(rs, idle):
    """an arm's cell: its runs' medians"""
    walls = [r["ab"]["wall"] for r in rs]
    outb = rs[0]["ab"]["out"]
    out = {"n": len(rs), "wall": med(walls), "out": outb}
    for key, ev in (("reads", READS), ("writes", WRITES)):
        xs = []
        for r in rs:
            v = imc_event(r.get("perf"), ev)
            if v is not None:
                xs.append(v - (idle.get(ev) or 0.0) * r["elapsed_s"])
        out[key] = med(xs) if len(xs) == len(rs) else None
    both = []
    for r in rs:
        v = [imc_event(r.get("perf"), ev) for ev in (READS, WRITES)]
        if all(x is not None for x in v):
            both.append(sum(v) - sum(idle.get(ev) or 0.0 for ev in (READS, WRITES)) * r["elapsed_s"])
    out["dram"] = med(both) if len(both) == len(rs) else None
    out["dram_bps"] = out["dram"] / out["wall"] if out["dram"] is not None and out["wall"] else None
    for key in ("dram", "reads", "writes"):
        out[f"{key}_per_out"] = out[key] / outb if out[key] is not None and outb else None
    lats = [latency(r.get("perf")) for r in rs]
    out["latency"] = med(lats) if all(x is not None for x in lats) else None
    out["lat_pct"] = min(
        (
            e.get("pct", 0.0)
            for r in rs
            for ev, e in ((r.get("perf") or {}).get("core") or {}).items()
            if ev in (LAT_REQ, LAT_OUT)
        ),
        default=None,
    )
    return out


def paired(runs, file, arms, key):
    """per round, arms[0]'s figure over arms[1]'s: the median and range of the ratios"""
    by = {}
    for r in runs:
        if r["file"] == file and "ab" in r and not r.get("err"):
            by.setdefault(r["round"], {})[r["arm"]] = r
    ratios = []
    for pair in by.values():
        if all(x in pair for x in arms):
            a, b = (key(pair[x]) for x in arms)
            if a is not None and b:
                ratios.append(a / b)
    if not ratios:
        return None
    return {"n": len(ratios), "median": med(ratios), "min": min(ratios), "max": max(ratios)}


def verdict(frac, rise):
    """the frozen rule on the reference arm's share of the ceiling (`frac`) and the latency's
    rise against 7/28 (`rise`, None where the latency wasn't counted)"""
    if frac is None:
        return "VOID", "the memory controller's share of the ceiling is unknown"
    if frac < REFUTE_FRAC:
        return "REFUTED", f"DRAM at {frac:.1%} of the ceiling, under {REFUTE_FRAC:.0%}"
    if rise is None:
        return "INCONCLUSIVE", "the L3-miss latency UNMEASURED, and DRAM is not under 80%"
    if rise < REFUTE_RISE:
        return "REFUTED", f"the L3-miss latency rose {rise:+.1%}, under +{REFUTE_RISE:.0%}"
    if frac >= SUPPORT_FRAC and rise >= SUPPORT_RISE:
        return (
            "SUPPORTED",
            f"DRAM at {frac:.1%} of the ceiling (at least {SUPPORT_FRAC:.0%}) and the latency "
            f"{rise:+.1%} (at least +{SUPPORT_RISE:.0%})",
        )
    return (
        "INCONCLUSIVE",
        f"between the thresholds: DRAM at {frac:.1%} of the ceiling, the latency {rise:+.1%}",
    )


def voids(runs, stream, arms, files, cpus):
    """every reason the run is VOID (an empty list: none)"""
    out = []
    if not runs:
        return ["no runs"]
    imc_counted = all(
        imc_event(r.get("perf"), READS) is not None and imc_event(r.get("perf"), WRITES) is not None
        for r in runs
    )
    if not imc_counted:
        out.append(
            "the memory controller is unreadable: uncore_imc/data_reads/ and data_writes/ "
            "not counted in every run (kernel.perf_event_paranoid must be 0 or less)"
        )
    bad = [r for r in runs if r.get("err") or r.get("rc") or "ab" not in r]
    if bad:
        out.append(f"{len(bad)} run(s) failed, the first: {bad[0].get('err') or 'exit ' + str(bad[0].get('rc'))}")
    off = [
        r
        for r in runs
        if not r.get("affinity")
        or len(r["affinity"]) != cpus
        or r["affinity"] != sorted(r.get("online") or [])
    ]
    if off:
        a = off[0].get("affinity")
        out.append(
            f"{len(off)} run(s) not on all {cpus} CPUs, the first on "
            f"{len(a) if a else 'unknown'} ({knee_list(a)})"
        )
    hashes = {}
    for r in runs:
        if "ab" in r and not r.get("err"):
            hashes.setdefault(r["file"], set()).add(r["ab"].get("hash"))
    for f, hs in sorted(hashes.items()):
        if len(hs) != 1:
            out.append(f"{f}: the arms' outputs differ ({len(hs)} hashes)")
    have = {(r["file"], r["arm"]) for r in runs if "ab" in r and not r.get("err")}
    for f in files:
        for arm in arms:
            if (f, arm) not in have:
                out.append(f"{f} {arm}: no clean run")
    if imc_counted and not stream:
        out.append("no ceiling: ab stream's records are missing")
    return out


def knee_list(cpus):
    if not cpus:
        return "none read back"
    return ",".join(map(str, cpus)) if len(cpus) <= 8 else f"{cpus[0]}..{cpus[-1]}"


def fmt(x, p=3):
    return "NA" if x is None else f"{x:.{p}f}"


def cmd_report(a):
    runs, stream, idle = knee.load(a.runs), knee.load(a.stream), knee.load(a.idle)
    arms = [x for x in a.arms.split(",") if x]
    files = [a.main] + ([a.check] if a.check else [])
    print(
        "bench-imc-ten: a diagnostic, scores record 52's prediction (REVIEW 6973478826), decides "
        "nothing in production (docs/journal/2026-11-18-imc-ten.md)"
    )
    v = voids(runs, stream, arms, files, a.cpus)
    ir = idle_rates(idle)
    ceil = knee.ceiling(stream, sum(x for x in ir.values() if x is not None) if ir else None)
    if not v and "imc" not in ceil:
        v.append("no ceiling in the memory controller's bytes (ab stream's pairs not counted)")
    if v:
        print("VOID, no reading:\n- " + "\n- ".join(v))
        if a.json:
            with open(a.json, "w") as f:
                json.dump({"verdict": "VOID", "why": v}, f, indent=1)
        return 1
    peak = ceil["imc"]["bps"]
    idle_total = sum(x for x in ir.values() if x is not None)
    print(
        f"runs: {len(runs)}; idle traffic {idle_total / 1e9:.3f} GB/s taken off; the ceiling "
        f"{peak / 1e9:.2f} GB/s (ab stream, {ceil['imc']['threads']} threads, {ceil['imc']['kernel']}; "
        "the memory controller's bytes by the difference of two repetition counts)"
    )
    print("  the ceiling at each count: " + ", ".join(
        f"{k} {x / 1e9:.2f}" for k, x in ceil["imc"]["all"].items()))
    rep = {"ceiling_gbs": peak / 1e9, "ceiling_at": f"{ceil['imc']['threads']}/{ceil['imc']['kernel']}",
           "idle_gbs": idle_total / 1e9, "files": {}}
    for f in files:
        rows = {}
        for arm in arms:
            rs = [r for r in runs if r["file"] == f and r["arm"] == arm]
            rows[arm] = row(rs, ir)
            rows[arm]["of_ceiling"] = rows[arm]["dram_bps"] / peak if rows[arm]["dram_bps"] else None
        ref, alt = (rows[x] for x in arms)
        rise = ref["latency"] / alt["latency"] - 1 if ref["latency"] and alt["latency"] else None
        wall_pair = paired(runs, f, arms[::-1], lambda r: r["ab"]["wall"])
        lat_pair = paired(runs, f, arms, lambda r: latency(r.get("perf")))
        status, why = verdict(ref["of_ceiling"], rise)
        label = "judged" if f == a.main else "a check, reported, not judged"
        print(f"\n{f} ({label}), 10 threads asked, unpinned on all {a.cpus} CPUs")
        print("arm               n  wall_s   DRAM_GB/s  of_ceiling  reads_B/out  writes_B/out  "
              "DRAM_B/out  L3-miss_latency_cycles")
        for arm in arms:
            r = rows[arm]
            print(
                f"{arm:<16} {r['n']:>2}  {fmt(r['wall'], 4)}  {fmt(r['dram_bps'] and r['dram_bps'] / 1e9, 2):>9}  "
                f"{fmt(r['of_ceiling'] and 100 * r['of_ceiling'], 1):>9}%  {fmt(r['reads_per_out'], 2):>11}  "
                f"{fmt(r['writes_per_out'], 2):>12}  {fmt(r['dram_per_out'], 2):>10}  {fmt(r['latency'], 1)}"
                + ("" if r["lat_pct"] is None or r["lat_pct"] >= 99 else f" (counted {r['lat_pct']:.0f}% of the run)")
            )
        if wall_pair:
            print(f"  wall {arms[1]} / {arms[0]}, paired by round: {wall_pair['median']:.3f} "
                  f"[{wall_pair['min']:.3f}, {wall_pair['max']:.3f}], {wall_pair['n']} pairs")
        print(
            f"  L3-miss latency {arms[0]} against {arms[1]}: "
            + ("UNMEASURED (the offcore pair not counted)" if rise is None else
               f"{rise:+.1%} (medians)"
               + (f"; paired by round {lat_pair['median'] - 1:+.1%} "
                  f"[{lat_pair['min'] - 1:+.1%}, {lat_pair['max'] - 1:+.1%}]" if lat_pair else ""))
        )
        gbs = ref["dram_bps"] / 1e9
        lo, hi = PREDICTED_GBS[1], PREDICTED_GBS[2]
        print(f"  {arms[0]} DRAM {gbs:.2f} GB/s against the predicted {PREDICTED_GBS[0]} [{lo}, {hi}]: "
              + ("inside" if lo <= gbs <= hi else "outside") + " the interval")
        print(f"  verdict{'' if f == a.main else ' (the check, not judged)'}: {status}: {why}")
        rep["files"][f] = {"rows": rows, "latency_rise": rise, "wall_paired": wall_pair,
                           "latency_paired": lat_pair, "verdict": status, "why": why}
    main = rep["files"][a.main]
    rep["verdict"] = main["verdict"]
    print(f"\nrecord 52's prediction, on {a.main}: {main['verdict']}")
    if a.json:
        with open(a.json, "w") as f:
            json.dump(rep, f, indent=1, default=str)
    return 0


def cmd_estimate(a):
    """the recipe's i9 time: Wikidata x1 at about 0.11 s and xC at C times that (0.5.3's
    release-scaling at 10 threads), a process's start and both perf wrappers 0.15 s a run
    (assumed), ab stream as knee.py estimates it, the idle readings, the build and the corpus"""
    runs = a.rounds * len(ARMS) * ((0.11 + 0.15) + (0.11 * a.copies + 0.15))
    s_rep = 3 * a.mib * 2**20 * 4 / 3 / 30e9
    stream = a.repeat * len(knee.ints(a.stream_threads)) * 2 * (
        2 * (0.4 + 0.15) + (a.reps_lo + a.reps_hi) * s_rep)
    idle = 2 * a.idle_count * (a.idle_seconds + 0.1)
    build, corpus = 150, 30
    total = runs + stream + idle + build + corpus
    print(json.dumps({"runs": a.rounds * len(ARMS) * 2, "run_s": round(runs), "stream_s": round(stream),
                      "idle_s": round(idle), "build_s": build, "corpus_s": corpus,
                      "total_min": round(total / 60, 1)}))


def main(argv=None):
    p = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = p.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("run")
    s.add_argument("--ab", required=True)
    s.add_argument("--out", required=True)
    s.add_argument("--file", action="append", required=True)
    s.add_argument("--arms", default=",".join(ARMS))
    s.add_argument("--threads", type=int, default=10)
    s.add_argument("--rounds", type=int, default=10)
    s.add_argument("--core-events", type=knee.events, default=[])
    s.add_argument("--imc-events", type=knee.events, default=[])
    s.add_argument("--imc-prefix", default="")
    s = sub.add_parser("report")
    s.add_argument("--runs", required=True)
    s.add_argument("--stream")
    s.add_argument("--idle")
    s.add_argument("--arms", default=",".join(ARMS))
    s.add_argument("--main", default="wikidata-x4")
    s.add_argument("--check", default="wikidata")
    s.add_argument("--cpus", type=int, default=CPUS)
    s.add_argument("--json")
    s = sub.add_parser("estimate")
    s.add_argument("--rounds", type=int, default=10)
    s.add_argument("--copies", type=int, default=4)
    s.add_argument("--stream-threads", default="1,2,4,10,20")
    s.add_argument("--repeat", type=int, default=3)
    s.add_argument("--mib", type=int, default=256)
    s.add_argument("--reps-lo", type=int, default=2)
    s.add_argument("--reps-hi", type=int, default=12)
    s.add_argument("--idle-count", type=int, default=5)
    s.add_argument("--idle-seconds", type=float, default=2.0)
    a = p.parse_args(argv)
    return {"run": cmd_run, "report": cmd_report, "estimate": cmd_estimate}[a.cmd](a) or 0


if __name__ == "__main__":
    sys.exit(main())
