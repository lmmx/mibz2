"""The cooperative decoder's full-decoder A/B on a workstation, against its release gates.

Each run is examples/ab.rs's: one process decoding the benchmark's Wikidata corpus from
disk to hashed output, every block's and stream's CRC checked, wall, CPU-seconds and peak
RSS recorded. The arms, interleaved by round (rotated and reversed), back to back
(`--settle 0`, as a release runs), unpinned as users run them:

- `mibz2-workers`: the decoder before the policy, T workers of a block each
  (`Plan::Workers`; the `mibz2` arm until the policy became the default);
- `mibz2-coop2`, `mibz2-coop3`: the cooperative decoder, T cores as T/2 or T/3 teams;
- `mibz2-auto`: the cooperative policy (`Plan::for_threads`), the default since it passed
  here (what `mibz2` runs): workers below 6 physical cores, else teams of 2 capped by the
  L3.

Then a second, smaller pass in a process limited to one CPU on each of 4 physical cores
(a 4-core machine): `mibz2-workers` against `mibz2-auto` at 4 threads, where the policy
must choose workers and be no slower.

The release gates (registered before the run: the coop journal §17, the integration plan's
amendment; the user's, 2026-10-29), on the `mibz2-auto` arm at the machine's own thread
count (all its CPUs, as Python's default is) against `mibz2-workers` at 4 (the decoder
before the policy at its peak on the i9), paired by round:
- **throughput:** the paired ratio's geometric mean at least 1.15, its 95% lower bound
  above 1.05;
- **bytes:** every run of every arm the same output hash;
- **peak RSS:** at most 1.15 x the reference's median;
- **CPU-seconds:** at most 1.5 x the reference's median;
- **the smaller machine:** `mibz2-auto` at 4 threads on 4 cores no slower than
  `mibz2-workers` there (the paired ratio's 95% lower bound at least 0.97).
A smaller but reproducible gain (a lower bound above 1.00) is reported as such, for the
user to decide on; the gates are not moved.

`just bench-coop-decoder [--rounds 20] [--threads 4,6,8,10,ALL] [--no-small]` writes
coop-decoder-<UTC time>.md (the report), .json (its figures) and the raw .jsonl files.
`--arms mibz2-workers,mibz2 --gate mibz2` reads the same gates on the default decoder as
users call it (`BzReader::parallel`), the release's own path.
"""

import argparse
import datetime
import json
import math
import os
import statistics
import subprocess
import sys

from imc import CORPUS, run

AB = "target/release/examples/ab"
# Student's t, 97.5% quantile, df 1-30
T975 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064,
        2.060, 2.056, 2.052, 2.048, 2.045, 2.042]  # fmt: skip
REF = "mibz2-workers"
ARMS = f"{REF},mibz2-coop2,mibz2-coop3,mibz2-auto"


def geo_ci(ratios):
    """The geometric mean of `ratios` and its 95% t-interval, (mean, low, high)"""
    logs = [math.log(r) for r in ratios]
    m = statistics.fmean(logs)
    if len(logs) < 2:
        return math.exp(m), float("nan"), float("nan")
    h = T975[min(len(logs) - 1, len(T975)) - 1] * statistics.stdev(logs) / math.sqrt(len(logs))
    return math.exp(m), math.exp(m - h), math.exp(m + h)


def load(path):
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def cpus_one_a_core(n):
    """One CPU (the first sibling) of each of the first `n` physical cores, as a taskset list"""
    seen, cpus = set(), []
    for c in sorted(os.sched_getaffinity(0)):
        sib = open(f"/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list").read()
        first = sib.strip().split(",")[0].split("-")[0]
        if first not in seen:
            seen.add(first)
            cpus.append(int(first))
    return ",".join(map(str, cpus[:n])) if len(cpus) >= n else None


def table(recs, ref):
    """Each (arm, threads): throughput, paired ratio against `ref` (arm, threads), RSS and CPU"""
    by = {}
    for r in recs:
        if r.get("err"):
            continue
        by.setdefault((r["arm"], r["threads"]), {})[r["round"]] = r
    base = by.get(ref, {})
    rows = []
    for (arm, t), rs in sorted(by.items(), key=lambda kv: (kv[0][0], kv[0][1])):
        rounds = sorted(set(rs) & set(base))
        ratios = [base[i]["wall"] / rs[i]["wall"] for i in rounds]
        mean, lo, hi = geo_ci(ratios) if ratios else (float("nan"),) * 3
        med = lambda key, d: statistics.median(x[key] for x in d.values())  # noqa: E731
        rows.append(
            {
                "arm": arm,
                "threads": t,
                "mb_s": statistics.median(x["out"] / x["wall"] for x in rs.values()) / 1e6,
                "ratio": mean,
                "low": lo,
                "high": hi,
                "won": sum(r > 1 for r in ratios),
                "rounds": len(ratios),
                "rss": med("rss_kb", rs) / med("rss_kb", base) if base else float("nan"),
                "cpu": med("cpu", rs) / med("cpu", base) if base else float("nan"),
                "rss_mb": med("rss_kb", rs) / 1024,
                "cpu_s": med("cpu", rs),
            }
        )
    return rows


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--rounds", type=int, default=20)
    ap.add_argument("--threads", default="4,6,8,10,ALL")
    ap.add_argument("--arms", default=ARMS)
    ap.add_argument(
        "--gate",
        default="mibz2-auto",
        help="the arm the gates are read on (mibz2: the default decoder as users call it)",
    )
    ap.add_argument("--no-small", action="store_true", help="skip the 4-core pass")
    ap.add_argument("--file", default=CORPUS)
    ap.add_argument("--note", default="")
    a = ap.parse_args()
    root = run(["git", "rev-parse", "--show-toplevel"], check=True).stdout.strip()
    os.chdir(root)
    for p in [AB, a.file]:
        if not os.path.exists(p):
            sys.exit(f"coop-decoder: no {p}; `just bench-coop-decoder` builds it")
    ncpu = len(os.sched_getaffinity(0))
    threads = [ncpu if t == "ALL" else int(t) for t in a.threads.split(",")]
    threads = sorted(set(threads))
    if 4 not in threads:
        sys.exit("coop-decoder: --threads must include 4, the reference")
    utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    raw = f"coop-decoder-{utc}.jsonl"
    argv = [AB, "run", raw, "--arms", a.arms, "--threads", ",".join(map(str, threads))]
    argv += ["--rounds", str(a.rounds), "--settle", "0", a.file]
    print("coop-decoder: the full machine:", " ".join(argv), flush=True)
    if subprocess.run(argv).returncode:
        sys.exit("coop-decoder: ab run failed")
    small_raw, small_cpus = None, None
    if not a.no_small:
        small_cpus = cpus_one_a_core(4)
        if small_cpus:
            small_raw = f"coop-decoder-{utc}-4cores.jsonl"
            sargv = [
                "taskset",
                "-c",
                small_cpus,
                AB,
                "run",
                small_raw,
                "--arms",
                f"{REF},{a.gate}",
            ]
            sargv += [
                "--threads",
                "4",
                "--rounds",
                str(max(10, a.rounds // 2)),
                "--settle",
                "0",
                a.file,
            ]
            print("coop-decoder: a 4-core machine:", " ".join(sargv), flush=True)
            if subprocess.run(sargv).returncode:
                sys.exit("coop-decoder: the 4-core ab run failed")

    recs = load(raw)
    hashes = {r["hash"] for r in recs if not r.get("err")}
    errors = [r for r in recs if r.get("err")]
    small = load(small_raw) if small_raw else []
    hashes |= {r["hash"] for r in small if not r.get("err")}
    errors += [r for r in small if r.get("err")]
    same = len(hashes) == 1 and not errors
    rows = table(recs, (REF, 4))
    prod = [r for r in rows if r["arm"] == REF]
    pbest = max(prod, key=lambda r: r["mb_s"]) if prod else None
    rows_best = table(recs, (REF, pbest["threads"])) if pbest else []
    auto = next((r for r in rows if r["arm"] == a.gate and r["threads"] == ncpu), None)
    srows = table(small, (REF, 4)) if small else []
    sauto = next((r for r in srows if r["arm"] == a.gate), None)

    gates = []
    if auto:
        gates = [
            ("throughput: mean >= 1.15", auto["ratio"] >= 1.15, f"{auto['ratio']:.3f}"),
            ("throughput: 95% lower bound > 1.05", auto["low"] > 1.05, f"{auto['low']:.3f}"),
            ("bytes identical, no error", same, f"{len(hashes)} hash(es), {len(errors)} error(s)"),
            ("peak RSS <= 1.15x", auto["rss"] <= 1.15, f"{auto['rss']:.3f}"),
            ("CPU-seconds <= 1.5x", auto["cpu"] <= 1.5, f"{auto['cpu']:.3f}"),
        ]
    if sauto:
        gates.append(
            (
                "4 cores: no slower (lower bound >= 0.97)",
                sauto["low"] >= 0.97,
                f"{sauto['ratio']:.3f} ({sauto['low']:.3f}-{sauto['high']:.3f})",
            )
        )
    passed = bool(gates) and all(g[1] for g in gates)

    lines = [f"# The cooperative decoder's full-decoder A/B ({utc})", ""]
    head = run(["git", "rev-parse", "--short", "HEAD"]).stdout.strip()
    model = next(
        (l.split(":", 1)[1].strip() for l in open("/proc/cpuinfo") if l.startswith("model name")),
        "?",
    )
    lines += [
        f"- **Build:** {head}; **machine:** {model}, {ncpu} CPUs; **load:** {open('/proc/loadavg').read().split()[0]}"
    ]
    lines += [
        f"- **Rounds:** {a.rounds}, interleaved, unpinned, back to back; corpus {os.path.basename(a.file)}"
    ]
    if a.note:
        lines += [f"- **Note:** {a.note}"]
    lines += ["", f"## The gates ({a.gate} at all CPUs against {REF} at 4)", ""]
    lines += ["| gate | pass | measured |", "|---|---|--:|"]
    lines += [f"| {g} | {'yes' if ok else '**no**'} | {v} |" for g, ok, v in gates]
    if auto:
        verdict = (
            "**PASS**: the release gates are met"
            if passed
            else "**a reproducible gain below the headline, for the user's decision**"
            if auto["low"] > 1.0 and same
            else "**FAIL**"
        )
        lines += ["", f"Verdict: {verdict}."]
    for title, rs in [
        (f"against {REF} at 4 (p4)", rows),
        (
            f"against the workers' best here ({REF} at {pbest['threads']})" if pbest else "",
            rows_best,
        ),
    ]:
        if not rs:
            continue
        lines += ["", f"## Every arm, {title}", ""]
        lines += [
            "| arm | T | MB/s | paired ratio (95%) | won | peak RSS | RSS ratio | CPU-s | CPU ratio |",
            "|---|--:|--:|--:|--:|--:|--:|--:|--:|",
        ]
        for r in rs:
            lines.append(
                f"| {r['arm']} | {r['threads']} | {r['mb_s']:.0f} | {r['ratio']:.3f} ({r['low']:.3f}-{r['high']:.3f}) "
                f"| {r['won']}/{r['rounds']} | {r['rss_mb']:.0f} MB | {r['rss']:.2f} | {r['cpu_s']:.2f} | {r['cpu']:.2f} |"
            )
    if srows:
        lines += ["", f"## A 4-core machine (CPUs {small_cpus}), against {REF} at 4 there", ""]
        lines += [
            "| arm | MB/s | paired ratio (95%) | RSS ratio | CPU ratio |",
            "|---|--:|--:|--:|--:|",
        ]
        for r in srows:
            lines.append(
                f"| {r['arm']} | {r['mb_s']:.0f} | {r['ratio']:.3f} ({r['low']:.3f}-{r['high']:.3f}) | {r['rss']:.2f} | {r['cpu']:.2f} |"
            )
    lines += [
        "",
        f"Files: {raw}" + (f", {small_raw}" if small_raw else "") + f", coop-decoder-{utc}.json",
    ]
    md = "\n".join(lines) + "\n"
    with open(f"coop-decoder-{utc}.md", "w") as f:
        f.write(md)
    with open(f"coop-decoder-{utc}.json", "w") as f:
        json.dump(
            {
                "utc": utc,
                "argv": argv,
                "ncpu": ncpu,
                "same_bytes": same,
                "gates": gates,
                "passed": passed,
                "gate": a.gate,
                "rows_p4": rows,
                "rows_pbest": rows_best,
                "small": srows,
                "small_cpus": small_cpus,
            },
            f,
            indent=1,
        )
    print(md)
    print(f"coop-decoder: wrote coop-decoder-{utc}.md, .json and the .jsonl: upload them all")


if __name__ == "__main__":
    main()
