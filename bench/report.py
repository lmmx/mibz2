"""The benchmark's figures from `ab run`'s records (one JSON line a run), the full-dump
model, the pull request comment, and the record kept for a master commit.

    report.py summary RUNS.jsonl --corpora DIR --mode pr|master|dispatch --head SHA
                      [--base SHA] [--date ISO] [--run URL] [--stages STAGES.jsonl] [--env ENV.json]
                      [--kernels KERNELS.jsonl]
                      --out RESULT.json --md OUT.md
    report.py record STORE RESULT.json RUNS.jsonl

`summary` reads the runs, the corpora's files (their sizes and SHA-256) and
bench/wikidata.json, and writes the result (every figure below, with the machine and
the toolchain) and its Markdown:

- **Measured,** per corpus and thread count, for each arm: the median and best wall
  time, the median CPU time (user + sys) and peak RSS, MB/s of output at the median
  wall, and CPU cycles a byte of output at the runner's reported clock. Against the
  reference arm (the base on a pull request; mibz2 on master), the paired ratios of
  wall, CPU and peak RSS: each round's ratio, their geometric mean and its 95%
  t-interval over the rounds, as `ab summary` gives them.
- **Where mibz2's CPU goes** on the Wikidata blocks at N threads: the scanner thread's
  CPU (each block's header and Huffman symbols, the one sequential part), the workers'
  (the rest of each block, in parallel across blocks) and the reading thread's.
- **Where a decode's time goes, stage by stage** (`--stages`, bench/stages.py's runs of
  examples/phases.rs): on the Wikidata blocks at 1 thread, each stage's median cycles a
  byte of output at the runner's reported clock (and ns a byte), for each build; on a
  pull request, head over base per stage, paired by round as above.
- **The inverse BWT alone, against bz2ngrs's** (`--kernels`, bench/alternatives'
  kernels binary, master and the Actions tab only): both kernels on the Wikidata blocks
  at 1 thread, build, walk and assembly, the median ns a byte and bz2ngrs's over mibz2's
  per phase, paired by pass.
- **Modelled, not measured:** a whole dump's decode (`model`), from the Wikidata blocks.
- **Resources** (`resources`, `assessment`), per corpus for each arm: CPU per byte at each
  thread count; its amplification, CPU per byte at N threads over the same arm's at the
  fewest, paired by round; parallel efficiency (speedup over N); each arm's best thread
  count; the threads' mean run-queue wait (time runnable, not running, over the wall) and
  the CPU the per-thread split leaves unattributed. On a pull request, head's
  amplification over base's, best against best, and a short **assessment**: what moved
  beyond noise among four pre-declared comparisons, whether the measurement could
  answer, and when a change needs a machine with more cores. The conditions (`--env`,
  `ab env`): the runner's cores, CPU quota, throttling and the run-queue counter.

`record` adds a master commit's result to the results store (the `bench-results`
branch): runs/<commit>.json, raw/<commit>.jsonl and a line of records.jsonl.

BENCH is the benchmark's version (semver), stamped on every result: what each kind of
change is, and every version's entry, are in bench/CHANGELOG.md, whose newest entry it
matches (bench/test_report.py). MEANING records, for each family of figures, the major
versions that redefined it, so the site's history joins a family's points only within the
major that last set its meaning.
"""

import argparse
import hashlib
import json
import math
import os
import platform
import re
import statistics
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

BENCH = "3.1.0"
# Each family of figures, and the major versions that set or redefined its meaning
# (bench/CHANGELOG.md): the decoder's own figures (throughput, CPU, peak RSS, by corpus
# and thread count) since 1.0.0, redefined by 3.0.0 (the harness's thread sampler every
# 10 ms instead of 50, off a run's pinned CPUs, its own CPU left out of the run's); the stage
# table's since 2.0.0 (1.2.0's measured warm repeats). A new major that redefines a family
# adds itself to that family's list.
MEANING = {"decoder": [1, 3], "stages": [1, 2]}


def meaning_major(family: str, bench: str) -> int:
    """The major version that last set the meaning of `family`'s figures, as measured by
    bench version `bench`: its points compare only with points of the same."""
    major = int(bench.split(".")[0])
    return max(m for m in MEANING[family] if m <= major)


# The other decoders, in the order they are shown: the block-parallel, then the
# single-threaded (examples/ab.rs: `parallel`)
ALTERNATIVES = [
    "pbz2",
    "lbzip2",
    "indexed_bzip2",
    "fbz",
    "lbzip2-rs",
    "libbzip2",
    "libbz2-rs",
    "bz2ngrs",
    "bz2ng",
]
HERE = Path(__file__).resolve().parent
WIKIDATA = json.loads((HERE / "wikidata.json").read_text())
MARK = "<!-- mibz2-bench -->"
STORE_README = """# Benchmark results

Data only, never merged: master's benchmark results, one per commit the Benchmark workflow
ran on (`.github/workflows/bench.yml`, `bench/report.py record`). Pull requests' runs are
not recorded.

- `records.jsonl`: a line a commit: its SHA, date, bench version, runner CPU and cores, rounds,
  corpora and the run's URL.
- `runs/<commit>.json`: its result (`bench/report.py summary`): every figure, the machine,
  the toolchain and the corpora's sizes and SHA-256.
- `raw/<commit>.jsonl`: the harness's records (`ab run`), one line a run.

A figure compares only within the major bench version that last set its meaning (MEANING in
bench/report.py, bench/CHANGELOG.md), one runner CPU model and one set of corpora.
"""


# ---------------------------------------------------------------------------------------
# Statistics


def t95(df: int) -> float:
    """The two-sided 95% t quantile for `df` degrees of freedom (as examples/ab.rs)."""
    t = [
        12.706,
        4.303,
        3.182,
        2.776,
        2.571,
        2.447,
        2.365,
        2.306,
        2.262,
        2.228,
        2.201,
        2.179,
        2.160,
        2.145,
        2.131,
        2.120,
        2.110,
        2.101,
        2.093,
        2.086,
        2.080,
        2.074,
        2.069,
        2.064,
        2.060,
        2.056,
        2.052,
        2.048,
        2.045,
        2.042,
    ]
    return math.nan if df <= 0 else t[df - 1] if df <= len(t) else 1.96


def mean_ci(v: list[float]) -> tuple[float, float]:
    """The mean and the half-width of its 95% interval (0 for one value)."""
    m = statistics.fmean(v)
    return m, (t95(len(v) - 1) * statistics.stdev(v) / math.sqrt(len(v)) if len(v) > 1 else 0.0)


def ratio_ci(r: list[float]) -> list[float] | None:
    """[geometric mean, low, high] of paired ratios: a t-interval on their logs."""
    r = [x for x in r if x and x > 0 and math.isfinite(x)]
    if len(r) < 2:
        return None
    m, h = mean_ci([math.log(x) for x in r])
    return [round(math.exp(m), 4), round(math.exp(m - h), 4), round(math.exp(m + h), 4)]


# ---------------------------------------------------------------------------------------
# The runner and the toolchain


def run_out(*cmd: str) -> str:
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        return (p.stdout or p.stderr).strip()
    except (OSError, subprocess.TimeoutExpired):
        return ""


def machine() -> dict:
    info = Path("/proc/cpuinfo").read_text() if Path("/proc/cpuinfo").exists() else ""
    field = lambda k: next(
        (l.split(":", 1)[1].strip() for l in info.splitlines() if l.startswith(k)), ""
    )
    mhz = field("cpu MHz")
    return {
        "cpu": field("model name") or platform.processor(),
        "cores": os.cpu_count(),
        "mhz": round(float(mhz), 1) if mhz else None,
        "flags": sorted(
            set(field("flags").split())
            & {"avx2", "avx512f", "pclmulqdq", "bmi1", "bmi2", "sse4_1", "ssse3"}
        ),
        "kernel": platform.release(),
        "image": " ".join(filter(None, [os.environ.get("ImageOS"), os.environ.get("ImageVersion")]))
        or None,
    }


def locked(name: str, lockfile: str = "Cargo.lock") -> str | None:
    """A package's version in a Cargo.lock (the first, if it is there twice)."""
    lock = Path(lockfile).read_text() if Path(lockfile).exists() else ""
    m = re.search(rf'name = "{re.escape(name)}"\nversion = "([^"]+)"', lock)
    return m.group(1) if m else None


def bz2ng_version() -> str | None:
    """bz2-ng's version, from its `bzip2 --version` (AB_BZ2NG): on stderr, after which it
    compresses stdin to stdout, so both are given nothing and dropped."""
    exe = os.environ.get("AB_BZ2NG")
    if not exe:
        return None
    try:
        p = subprocess.run(
            [exe, "--version"],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    line = next((l for l in p.stderr.decode(errors="replace").splitlines() if "bz2-ng" in l), "")
    m = re.search(r"bz2-ng ([\w.]+?)\.?$", line.strip())
    return m and m.group(1)


def git_locked(name: str, lockfile: str) -> str | None:
    """A git dependency's version and commit, as `lockfile` pins them."""
    lock = Path(lockfile).read_text() if Path(lockfile).exists() else ""
    m = re.search(
        rf'name = "{re.escape(name)}"\nversion = "([^"]+)"\nsource = "git\+[^"#]*#([0-9a-f]+)"',
        lock,
    )
    return m and f"{m.group(1)} at {m.group(2)[:7]} (git)"


def toolchain(arms: list[str]) -> dict:
    """The compiler and each arm's version (the alternatives' only where they ran)."""
    names = {label(a) for a in arms}
    lbzip2 = run_out("lbzip2", "--version").splitlines()
    alts = "bench/alternatives/Cargo.lock"
    py = os.environ.get("AB_PYTHON", "python3")
    ib = run_out(py, "-c", "import indexed_bzip2; print(indexed_bzip2.__version__)")
    bz2ng = bz2ng_version()
    out = {
        "rustc": run_out("rustc", "-V"),
        "pbz2": locked("pbz2"),
        "lbzip2": next((l for l in lbzip2 if "lbzip2" in l.lower()), lbzip2[0] if lbzip2 else None),
    }
    more = {
        "indexed_bzip2": ib if re.fullmatch(r"[\w.]+", ib) else None,
        "fbz": locked("fbz", alts),
        "lbzip2-rs": locked("lbzip2", alts) and f"{locked('lbzip2', alts)} (the lbzip2 crate)",
        "libbzip2": "1.0.8 (the bzip2 crate's bundled source)",
        "libbz2-rs": locked("libbz2-rs-sys", alts)
        and f"{locked('libbz2-rs-sys', alts)} (libbz2-rs-sys, through the bzip2 crate)",
        "bz2ngrs": git_locked("bz2ngrs", alts),
        "bz2ng": bz2ng and f"bz2-ng {bz2ng} (C, its `bzip2 -dc`)",
    }
    out |= {k: v for k, v in more.items() if k in names}
    out["libbzip2 (the corpora's encoder)"] = "1.0.8 (the bzip2 crate's bundled source)"
    return out


def corpora(directory: Path, runs: list[dict]) -> dict:
    out_bytes = {r["corpus"]: r["out"] for r in runs if not r.get("err")}
    found = {}
    for path in sorted(directory.glob("*.bz2")):
        name = path.stem
        found[name] = {
            "file": path.name,
            "bytes": path.stat().st_size,
            "out": out_bytes.get(name),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "source": (
                f"{WIKIDATA['dump']}: {len(WIKIDATA['windows'])} windows' whole blocks (bench/wikidata.json)"
                if name == "wikidata"
                else "ab gen (tests/common/mod.rs corpora), libbzip2 -9"
            ),
        }
    return found


# ---------------------------------------------------------------------------------------
# The figures


def threads_of(run: dict) -> dict[str, float]:
    """A run's busy threads' CPU seconds, by name (ab's thread_cpu: "name=1.23 ...")."""
    out = defaultdict(float)
    for part in run.get("thread_cpu", "").split():
        name, _, v = part.rpartition("=")
        try:
            out[name] += float(v)
        except ValueError:
            pass
    return out


def split_cpu(run: dict) -> dict | None:
    """mibz2's scanner, workers and the rest (the reading thread), or None for another
    decoder or a 1-thread run (one thread does it all)."""
    t = threads_of(run)
    if "mibz2-scan" not in t:
        return None
    workers = sum(v for k, v in t.items() if re.fullmatch(r"mibz2-\d+", k))
    return {
        "scan": t["mibz2-scan"],
        "workers": workers,
        "other": sum(t.values()) - t["mibz2-scan"] - workers,
        # the process's CPU (the kernel's total) that no sampled thread holds: threads
        # that ended between samples, below each thread's last tick, the sampler's own
        "unattributed": run["cpu"] - sum(t.values()),
    }


def label(arm: str, pr: bool = False) -> str:
    """An arm as shown: `mibz2@base/ab` (another build of mibz2) is "base"; another
    decoder in another build of the harness (`fbz@alts/ab`) is that decoder; and on a
    pull request this build's `mibz2` is "head"."""
    if "@" in arm:
        name, exe = arm.split("@", 1)
        return exe.split("/")[0] if name == "mibz2" else name
    return "head" if pr and arm == "mibz2" else arm


def parallel(runs: list[dict], pr: bool = False) -> dict[str, bool]:
    """Each arm, as shown, and whether it decodes blocks in parallel (an older record's
    runs say nothing: every arm then was), in ALTERNATIVES' order after the reference."""
    out = {label(r["arm"], pr): r.get("parallel", True) for r in runs}
    rank = lambda a: (a in ALTERNATIVES, ALTERNATIVES.index(a) if a in ALTERNATIVES else 0)
    return {a: out[a] for a in sorted(out, key=rank)}


def figures(runs: list[dict], ref: str, mhz: float | None, pr: bool) -> dict:
    """Per corpus and thread count, each arm's figures. A single-threaded arm runs at the
    fewest threads only: those runs stand at every thread count, marked (`parallel`
    false, `threads_run`), paired round by round with the reference's at that count."""
    by = defaultdict(lambda: defaultdict(dict))  # (corpus, threads) -> arm -> round -> run
    counts = sorted({r["threads"] for r in runs})
    for r in runs:
        at = counts if r.get("parallel") is False else [r["threads"]]
        for t in at:
            by[(r["corpus"], t)][r["arm"]][r["round"]] = r
    out = {}
    for (corpus, t), arms in sorted(by.items()):
        cell = {}
        base = arms.get(ref, {})
        for arm, rounds in arms.items():
            ok = [r for r in rounds.values() if not r.get("err")]
            if not ok:
                cell[label(arm, pr)] = {
                    "n": 0,
                    "errors": sorted({r["err"] for r in rounds.values()}),
                }
                continue
            col = lambda k: [r[k] for r in ok]
            wall, cpu = statistics.median(col("wall")), statistics.median(col("cpu"))
            ref_hashes = {r["hash"] for r in base.values() if not r.get("err")}
            f = {
                "arm": arm,
                "parallel": ok[0].get("parallel", True),
                "threads_run": ok[0]["threads"],
                "same_bytes": {r["hash"] for r in ok} == ref_hashes and len(ok) == len(rounds),
                "n": len(ok),
                "wall": round(wall, 4),
                "wall_best": round(min(col("wall")), 4),
                "cpu": round(cpu, 4),
                "rss_mb": round(statistics.median(col("rss_kb")) / 1024, 1),
                "out": ok[0]["out"],
                "mb_s": round(ok[0]["out"] / 1e6 / wall, 1) if wall else None,
                "cycles_per_byte": round(cpu * mhz * 1e6 / ok[0]["out"], 2)
                if mhz and ok[0]["out"]
                else None,
                "errors": sorted({r["err"] for r in rounds.values() if r.get("err")}),
            }
            splits = [s for r in ok if (s := split_cpu(r))]
            if splits:
                f["cpu_split"] = {
                    k: round(statistics.median([s[k] for s in splits]), 3) for k in splits[0]
                }
            if arm != ref and base:
                pairs = [
                    (r, base[k])
                    for k, r in rounds.items()
                    if k in base and not r.get("err") and not base[k].get("err")
                ]
                for k, key in (("wall", "wall"), ("cpu", "cpu"), ("rss", "rss_kb")):
                    f[f"{k}_ratio"] = ratio_ci([a[key] / b[key] for a, b in pairs if b[key]])
            cell[label(arm, pr)] = f
        hashes = {
            r["hash"] for rounds in arms.values() for r in rounds.values() if not r.get("err")
        }
        errors = any(r.get("err") for rounds in arms.values() for r in rounds.values())
        out.setdefault(corpus, {})[str(t)] = {
            "arms": cell,
            "same_bytes": len(hashes) == 1 and not errors,
        }
    return out


# ---------------------------------------------------------------------------------------
# Resources: CPU per byte, scaling, and the conditions


def accounting(runs: list[dict]) -> list[str]:
    """The CPU accounting's own check, on this build's runs (an arm without `@`, whose
    child is this harness): the per-thread split never holds more than the process's
    total (getrusage over every thread), beyond a tick a thread. More would be a thread
    counted twice, or a total that misses threads: broken instrumentation, not a figure."""
    bad = []
    for r in runs:
        if "@" in r["arm"] or r.get("err") or "runq_s" not in r:
            continue
        split = sum(threads_of(r).values())
        if split > r["cpu"] + 0.01 * (r.get("threads_busy", 1) + 1):
            bad.append(
                f"{r['arm']} {r['corpus']} {r['threads']}t round {r['round']}: "
                f"threads {split:.3f} s > total {r['cpu']:.3f} s"
            )
    return bad


def resources(runs: list[dict], pr: bool) -> dict:
    """Per corpus and arm (as shown), per thread count: CPU per byte, its amplification
    over the arm's own fewest-thread runs, speedup and parallel efficiency, the CPUs the
    runs had, the threads' mean run-queue wait and the split's unattributed CPU; and the
    arm's best thread count by median wall. Each ratio is paired by round: the rounds'
    ratios, their geometric mean and its 95% interval (`ratio_ci`)."""
    by = defaultdict(lambda: defaultdict(lambda: defaultdict(dict)))  # corpus, arm, t, round
    for r in runs:
        if r.get("parallel") is False or r.get("err"):
            continue
        by[r["corpus"]][label(r["arm"], pr)][r["threads"]][r["round"]] = r
    out = {}
    for corpus, arms in sorted(by.items()):
        for arm, ts in arms.items():
            fewest = min(ts)
            one = ts[fewest]
            per = {}
            for t, rounds in sorted(ts.items()):
                ok = list(rounds.values())
                med = lambda f: statistics.median([f(r) for r in ok])
                cell = {
                    "n": len(ok),
                    "cpus": statistics.mode([r.get("cpus", "all" if t > 1 else "1") for r in ok]),
                    "mb_s": round(ok[0]["out"] / 1e6 / med(lambda r: r["wall"]), 1),
                    "cpu_ns_per_byte": round(med(lambda r: 1e9 * r["cpu"] / r["out"]), 3),
                    "cpu_s_per_gib": round(med(lambda r: r["cpu"] / r["out"] * 2**30), 3),
                }
                if all("runq_s" in r for r in ok):
                    cell["runq_threads"] = round(med(lambda r: r["runq_s"] / r["wall"]), 3)
                if all(r.get("thread_cpu") for r in ok):
                    cell["unattributed"] = round(
                        med(lambda r: (r["cpu"] - sum(threads_of(r).values())) / r["cpu"]), 3
                    )
                thr = [r.get("throttled_n") for r in ok]
                cell["throttled_runs"] = (
                    sum(1 for x in thr if x) if all(x is not None for x in thr) else None
                )
                if t != fewest:
                    ks = [k for k in rounds if k in one]
                    cell["amplification"] = ratio_ci([rounds[k]["cpu"] / one[k]["cpu"] for k in ks])
                    sp = ratio_ci([one[k]["wall"] / rounds[k]["wall"] for k in ks])
                    cell["speedup"] = sp
                    cell["efficiency"] = [round(x * fewest / t, 4) for x in sp] if sp else None
                per[str(t)] = cell
            walls = {t: statistics.median([r["wall"] for r in rs.values()]) for t, rs in ts.items()}
            best = min(walls, key=walls.get)
            out.setdefault(corpus, {})[arm] = {"fewest": fewest, "best": best, "threads": per}
    return out


def paired_resources(runs: list[dict], head: str, base: str) -> dict:
    """On a pull request, per corpus: head's amplification over base's at each thread
    count (each round's (head N / head fewest) / (base N / base fewest) CPU), and head's
    best thread count against base's, best against best (each round's wall at head's
    best over base's at base's best, the best chosen by median wall on the same runs)."""
    by = defaultdict(lambda: defaultdict(lambda: defaultdict(dict)))
    for r in runs:
        if r.get("err") or r["arm"] not in (head, base):
            continue
        by[r["corpus"]][r["arm"]][r["threads"]][r["round"]] = r
    out = {}
    for corpus, arms in by.items():
        if not {head, base} <= set(arms):
            continue
        h, b = arms[head], arms[base]
        ts = sorted(set(h) & set(b))
        if len(ts) < 2:
            continue
        f = ts[0]
        amp = {}
        for t in ts[1:]:
            ks = [k for k in h[t] if k in b[t] and k in h[f] and k in b[f]]
            amp[str(t)] = ratio_ci(
                [(h[t][k]["cpu"] / h[f][k]["cpu"]) / (b[t][k]["cpu"] / b[f][k]["cpu"]) for k in ks]
            )
        med = lambda arm, t: statistics.median([r["wall"] for r in arm[t].values()])
        bh, bb = min(ts, key=lambda t: med(h, t)), min(ts, key=lambda t: med(b, t))
        ks = [k for k in h[bh] if k in b[bb]]
        out[corpus] = {
            "amplification_change": amp,
            "best": {"head": bh, "base": bb},
            "best_wall_ratio": ratio_ci([h[bh][k]["wall"] / b[bb][k]["wall"] for k in ks]),
        }
    return out


def direction(r: list[float] | None) -> int:
    """+1 where a ratio's interval lies above 1, −1 below, 0 where it holds 1 (or none)."""
    return 0 if not r else 1 if r[1] > 1 else -1 if r[2] < 1 else 0


def assessment(res: dict, corpus: str = "wikidata") -> dict | None:
    """A pull request's resource verdict on `corpus`, from four comparisons declared here
    in advance, each moved only where its 95% interval excludes no change:

    - **speed:** head's best thread count against base's, best against best (wall);
    - **CPU cost:** CPU per byte at the most threads (head over base);
    - **contention sensitivity:** CPU amplification over the fewest threads, head's over
      base's, at every other thread count (CI's 2-thread runs, one CPU a core, are the
      measure free of SMT; its 4 share each core's two siblings);
    - **memory:** peak RSS at the most threads.

    Then whether the run could answer (parity, broken accounting, throttling), and
    whether the change needs a machine with more cores: when the sensitivity rose, the
    memory grew, or the change in wall at one thread and at the most point opposite ways.
    """
    pairs = (res.get("paired_resources") or {}).get(corpus)
    figs = res["figures"].get(corpus)
    if not pairs or not figs:
        return None
    n = str(max(res["threads"]))
    one = str(min(res["threads"]))
    h_n = figs.get(n, {}).get("arms", {}).get("head", {})
    h_1 = figs.get(one, {}).get("arms", {}).get("head", {})
    amp = pairs["amplification_change"]
    signs = {
        "speed": -direction(pairs["best_wall_ratio"]),  # +1: faster
        "CPU cost": direction(h_n.get("cpu_ratio")),  # +1: more CPU a byte
        "contention sensitivity": max((direction(v) for v in amp.values()), default=0)
        or min((direction(v) for v in amp.values()), default=0),
        "memory": direction(h_n.get("rss_ratio")),
    }
    better = [k for k in signs if (signs[k] > 0) == (k == "speed") and signs[k] != 0]
    worse = [k for k in signs if signs[k] != 0 and k not in better]
    why = []
    if not res["parity"]:
        why.append("the outputs differ (parity failed)")
    if res.get("accounting"):
        why.append(f"the CPU accounting failed its own check ({len(res['accounting'])} runs)")
    throttled = [
        f"{a} {t}t"
        for a, ar in (res.get("resources", {}).get(corpus) or {}).items()
        for t, c in ar["threads"].items()
        if c.get("throttled_runs")
    ]
    if throttled:
        why.append(f"the cgroup throttled runs ({', '.join(throttled)})")
    w1, wn = direction(h_1.get("wall_ratio")), direction(h_n.get("wall_ratio"))
    broader = []
    if signs["contention sensitivity"] > 0:
        broader.append("CPU amplification rose")
    if signs["memory"] > 0:
        broader.append("peak RSS grew")
    if w1 and wn and w1 != wn:
        broader.append(f"the wall moved one way at {one} thread and the other at {n}")
    if why:
        verdict = "Inconclusive: " + "; ".join(why) + "."
    elif better and not worse:
        verdict = f"Better ({', '.join(better)}), and no measure worse."
    elif worse and not better:
        verdict = f"Worse ({', '.join(worse)}), and no measure better."
    elif better and worse:
        verdict = (
            f"A trade-off: better {', '.join(better)}; worse {', '.join(worse)}. "
            "Neither build dominates: weigh it for the deployment (wikidata-pq's release)."
        )
    else:
        verdict = (
            "No change detected on the four measures at this run's precision (every interval "
            "holds no change; not a proof that nothing changed)."
        )
    return {
        "corpus": corpus,
        "signs": signs,
        "better": better,
        "worse": worse,
        "inconclusive": why,
        "broader_hardware": broader,
        "verdict": verdict,
    }


# ---------------------------------------------------------------------------------------
# The stages


# The stages as shown, in examples/phases.rs's order (bench/stages.py's keys)
STAGE_ROWS = [
    ("scan", "scanner (headers, Huffman)"),
    ("mtf", "MTF + RLE2"),
    ("build", "T-vector build"),
    ("walk", "walk (the one each build uses)"),
    ("assemble", "of which: segment assembly"),
    ("rle", "RLE1"),
    ("crc", "CRC"),
    ("total", "total"),
]


# The inverse BWT's phases, mibz2's kernel against bz2ngrs's (bench/alternatives'
# kernels binary: master and the Actions tab only), as kernels.jsonl names them
KERNEL_ROWS = [
    ("build_s", "T-vector build"),
    ("walk_s", "walk"),
    ("assemble_s", "segment assembly (mibz2) / gather (bz2ngrs)"),
    ("total_s", "inverse BWT, total"),
]


def kernel_figures(lines: list[dict]) -> dict | None:
    """Each kernel's median ns a byte per phase over the passes, and bz2ngrs's over
    mibz2's per phase, paired by pass."""
    by = defaultdict(dict)  # kernel -> pass -> line
    for r in lines:
        by[r["kernel"]][r["pass"]] = r
    if not {"mibz2", "bz2ngrs"} <= set(by):
        return None
    bytes_ = lines[0]["bytes"]
    ns = lambda s: s * 1e9 / bytes_
    passes = sorted(set(by["mibz2"]) & set(by["bz2ngrs"]))
    out = {"tool": "bench/alternatives/src/kernels.rs", "passes": len(passes), "bytes": bytes_}
    for k in ("mibz2", "bz2ngrs"):
        out[k] = {
            key: round(ns(statistics.median(by[k][p][key] for p in passes)), 4)
            for key, _ in KERNEL_ROWS
        }
    out["ratio"] = {
        key: ratio_ci([by["bz2ngrs"][p][key] / by["mibz2"][p][key] for p in passes])
        for key, _ in KERNEL_ROWS
    }
    return out


def kernel_lines(k: dict) -> list[str]:
    """The kernels' table: each phase's median ns a byte and bz2ngrs over mibz2."""
    out = [
        "",
        "### The inverse BWT alone: mibz2's kernel against bz2ngrs's, phase by phase (median ns/B)",
        "",
        f"Both kernels on the same Wikidata blocks' last columns, on one pinned CPU, {k['passes']} "
        "passes of the file in decode order, the kernels' order alternating (`kernels`, "
        "bench/alternatives: bz2ngrs's src/bwt.rs as it is, its steps timed apart). Every block's "
        "bytes are checked the same from both, and its CRC, before the timed passes. mibz2's "
        "build takes its byte counts from MTF; bz2ngrs's counts them itself, so its build row "
        "holds that count. A ratio above 1× is bz2ngrs taking longer. Diagnostic only: the "
        "decoders' own figures are the tables above.",
        "",
        "| phase | mibz2 | bz2ngrs | bz2ngrs / mibz2 |",
        "|---|--:|--:|--:|",
    ]
    for key, name in KERNEL_ROWS:
        out.append(
            f"| {name} | {k['mibz2'][key]:.3f} | {k['bz2ngrs'][key]:.3f} | {ratio_x(k['ratio'][key])} |"
        )
    return out


def stage_figures(lines: list[dict], mhz: float | None, pr: bool) -> dict:
    """Each build's stages (bench/stages.py's runs, picoseconds a byte): the median ns and
    cycles a byte of output (at the reported clock) and each round's ns; on a pull
    request, head over base per stage, the rounds' paired ratios as `figures`'."""
    by = defaultdict(dict)  # arm -> round -> run
    for r in lines:
        by[r["arm"]][r["round"]] = r
    keys = [k for k, _ in STAGE_ROWS]
    arms = {}
    for arm, rounds in by.items():
        ok = {k: r for k, r in sorted(rounds.items()) if not r.get("err")}
        f = {
            "n": len(ok),
            "errors": sorted({r["err"] for r in rounds.values() if r.get("err")}),
            "stages": {},
            "rounds_ns_per_byte": {},
            # the walk the build uses, as its phases names it (the same in every run)
            "walk": next((r["walk"] for r in ok.values() if r.get("walk")), None),
        }
        for key in keys:
            # a stage a build's phases doesn't have (from an earlier bench version) is left out
            v = [r["ps_per_byte"][key] / 1000 for r in ok.values() if key in r["ps_per_byte"]]
            if not v:
                continue
            med = statistics.median(v)
            f["stages"][key] = {
                "ns_per_byte": round(med, 4),
                "cycles_per_byte": round(med * mhz / 1000, 3) if mhz else None,
            }
            f["rounds_ns_per_byte"][key] = v
        arms[arm] = f
    out = {
        "corpus": "wikidata",
        "tool": "examples/phases.rs (bench/stages.py)",
        "threads": 1,
        "repeats": max((r.get("repeats", 1) for r in lines), default=None),
        "rounds": len({r["round"] for r in lines}),
        "runs": len(lines),
        "seconds": round(sum(r.get("wall", 0) for r in lines), 1),
        "mhz": mhz,
        "arms": arms,
    }
    if pr and {"base", "head"} <= set(by):
        b, h = by["base"], by["head"]
        pairs = [
            (h[k]["ps_per_byte"], b[k]["ps_per_byte"])
            for k in h
            if k in b and not h[k].get("err") and not b[k].get("err")
        ]
        out["change"] = {
            key: ratio_ci([x[key] / y[key] for x, y in pairs if x.get(key) and y.get(key)])
            for key in keys
        }
    return out


# ---------------------------------------------------------------------------------------
# The full-dump model


def window_ratio() -> tuple[float, float]:
    """The Wikidata corpus's compression ratio, and the relative half-width of the 95%
    interval of its windows' mean ratio (how far another sample of the dump could fall)."""
    ws = WIKIDATA["windows"]
    total = WIKIDATA["decoded_bytes"] / WIKIDATA["corpus_bytes"]
    m, h = mean_ci([w["decoded_bytes"] / w["stream_bytes"] for w in ws])
    return total, h / m


def model(runs: list[dict], arm: str, n: int) -> dict | None:
    """A whole dump's decode by `arm` (a mibz2 build) at 1 thread and at `n`, modelled
    from its runs on the Wikidata blocks, round by round:

    - the scanner's CPU per byte of bz2 (measured at n threads: its own thread), scaled
      by the dump's bz2 bytes;
    - the workers' CPU per byte of output (measured at n threads), scaled by the dump's
      output, modelled as its bz2 bytes times the sample's compression ratio;
    - at n threads, the wall is the larger of the scanner's time and the workers' over
      n, times the sample's own wall over that bound (its overlap and fill, taken to grow
      with the work, which errs long); a fixed-cost reading of the same gap is given
      beside it (`wall_n_fixed`);
    - at 1 thread, the scanner's part (per byte of bz2) and the rest of the sample's wall
      (per byte of output).
    """
    rounds = defaultdict(dict)
    for r in runs:
        if r["corpus"] == "wikidata" and r["arm"] == arm and not r.get("err"):
            rounds[r["round"]][r["threads"]] = r
    c_s, c = WIKIDATA["corpus_bytes"], WIKIDATA["dump_bytes"]
    ratio, u_ratio = window_ratio()
    o = c * ratio
    per = defaultdict(list)
    for k, by_t in sorted(rounds.items()):
        one, many = by_t.get(1), by_t.get(n)
        split = split_cpu(many) if many else None
        if not (one and split):
            continue
        o_s = many["out"]
        scan_b, work_b = split["scan"] / c_s, split["workers"] / o_s  # s a byte: of bz2, of output
        bound_s = max(split["scan"], split["workers"] / n)
        bound = max(scan_b * c, work_b * o / n)
        per["scan_ns_per_bz2_byte"].append(1e9 * scan_b)
        per["workers_ns_per_out_byte"].append(1e9 * work_b)
        per["rest_1t_ns_per_out_byte"].append(1e9 * (one["wall"] - scan_b * c_s) / o_s)
        per["overlap"].append(many["wall"] / bound_s)
        per["scan_s"].append(scan_b * c)
        per["workers_s"].append(work_b * o / n)
        per["wall_n"].append(many["wall"] / bound_s * bound)
        per["wall_n_fixed"].append(bound + many["wall"] - bound_s)
        per["wall_1"].append(scan_b * c + (one["wall"] - scan_b * c_s) / o_s * o)
        per["scanner_binds_from"].append(work_b * o / (scan_b * c))
    if len(per["wall_n"]) < 2:
        return None
    out = {"rounds": len(per["wall_n"]), "threads": n}
    scan_share = statistics.fmean(per["scan_s"]) / statistics.fmean(per["wall_1"])
    for key, v in per.items():
        m, h = mean_ci(v)
        # the output's share of a time is scaled by the dump's output, modelled from the
        # sample's ratio: its uncertainty (the windows' spread) is added in quadrature
        share = {
            "wall_n": 0.0
            if statistics.fmean(per["scan_s"]) > statistics.fmean(per["workers_s"])
            else 1.0,
            "wall_n_fixed": 0.0
            if statistics.fmean(per["scan_s"]) > statistics.fmean(per["workers_s"])
            else 1.0,
            "wall_1": 1 - scan_share,
            "workers_s": 1.0,
            "scanner_binds_from": 1.0,
        }.get(key, 0.0)
        h = m * math.hypot(h / m if m else 0.0, share * u_ratio)
        out[key] = [round(m, 3), round(h, 3)]
    out["bound"] = "scanner" if out["scan_s"][0] > out["workers_s"][0] else "workers"
    out["paired"] = {k: per[k] for k in ("wall_1", "wall_n")}
    return out


def dump_facts() -> dict:
    ratio, u = window_ratio()
    return {
        "dump": WIKIDATA["dump"],
        "dump_bytes": WIKIDATA["dump_bytes"],
        "sample_ratio": round(ratio, 3),
        "sample_ratio_rel_ci": round(u, 4),
        "dump_out_bytes_modelled": round(WIKIDATA["dump_bytes"] * ratio),
        "windows": len(WIKIDATA["windows"]),
        "blocks": sum(w["blocks"] for w in WIKIDATA["windows"]),
    }


# ---------------------------------------------------------------------------------------
# Markdown


def duration(s: float) -> str:
    s = abs(s)
    if s < 60:
        return f"{s:.1f}s" if s >= 10 else f"{s:.2f}s"
    t = round(s)
    h, m = divmod(t // 60, 60)
    return f"{h}h{m:02d}m" if h else f"{m}m{t % 60:02d}s"


def size(b: float) -> str:
    for unit, k in (("TB", 1e12), ("GB", 1e9), ("MB", 1e6), ("kB", 1e3)):
        if b >= k:
            return f"{b / k:.3g} {unit}"
    return f"{b:.0f} B"


def change(r: list[float] | None, inverse: bool = False) -> str:
    """A paired ratio as a change: "−3.1% (−4.0, −2.2) **faster**", where the interval
    excludes none. `inverse` shows it the other way round: alternative over mibz2 as a
    multiple ("1.84× (1.80–1.88)")."""
    if not r:
        return "–"
    g, lo, hi = r
    if inverse:
        return f"{g:.2f}× ({lo:.2f}–{hi:.2f})"
    pct = lambda x: f"{100 * (x - 1):+.1f}%".replace("-", "−")
    mark = " **faster**" if hi < 1 else " **slower**" if lo > 1 else ""
    return f"{pct(g)} ({pct(lo)}, {pct(hi)}){mark}"


def rss_change(r: list[float] | None) -> str:
    if not r:
        return "–"
    g, lo, hi = r
    mark = " **less**" if hi < 1 else " **more**" if lo > 1 else ""
    return f"{100 * (g - 1):+.1f}%{mark}".replace("-", "−")


def corpus_cell(name: str, c: dict) -> str:
    if not c.get("out"):
        return f"`{name}`"
    return f"`{name}` ({size(c['bytes'])} → {size(c['out'])}, {c['out'] / c['bytes']:.1f}×)"


def held(m: list[float]) -> str:
    return f"{duration(m[0])} ± {duration(m[1])}"


def model_lines(models: dict[str, dict], n: int, pr: bool) -> list[str]:
    d = dump_facts()
    out = [
        "",
        "### Modelled, not measured: a whole dump",
        "",
        f"The decode of `{d['dump']}` ({size(d['dump_bytes'])} of bz2; {size(d['dump_out_bytes_modelled'])} of JSON at "
        f"the sample's {d['sample_ratio']:.2f}×), scaled from the Wikidata blocks' runs above, round by round "
        "(mean ± its 95% interval, the sample ratio's own uncertainty added in):",
        "",
    ]
    names = list(models)
    out.append(
        "| | "
        + " | ".join(names)
        + (" | head over base, paired |" if pr and len(names) == 2 else " |")
    )
    out.append("|---|" + "--:|" * len(names) + ("--:|" if pr and len(names) == 2 else ""))
    rows = [
        ("1 thread", "wall_1"),
        (f"{n} threads", "wall_n"),
        (f"{n} threads, the sample's overlap gap as a fixed cost", "wall_n_fixed"),
        ("the scanner alone (its serial part)", "scan_s"),
        (f"the workers alone, over {n}", "workers_s"),
    ]
    for title, key in rows:
        cells = [held(models[a][key]) for a in names]
        extra = ""
        if pr and len(names) == 2 and key in models[names[0]]["paired"]:
            a, b = (models[x]["paired"][key] for x in names)
            k = min(len(a), len(b))
            extra = f" | {change(ratio_ci([y / x for x, y in zip(a[:k], b[:k])]))}"
        elif pr and len(names) == 2:
            extra = " | "
        out.append(f"| {title} | " + " | ".join(cells) + extra + " |")
    m = models[names[-1]]
    out += [
        "",
        f"At {n} threads the {m['bound']} bound it. Per byte: the scanner "
        f"{m['scan_ns_per_bz2_byte'][0]:.1f} ns a byte of bz2, the workers {m['workers_ns_per_out_byte'][0]:.2f} ns a "
        f"byte of output at {n} threads (CPU, summed over them), {m['rest_1t_ns_per_out_byte'][0]:.2f} ns at 1 "
        f"thread (wall, less the scanner's part); the scanner would bind from about "
        f"{m['scanner_binds_from'][0]:.0f} worker threads ({names[-1]}).",
        "",
        "<details><summary>What the model assumes</summary>",
        "",
        "- **Linear in bytes:** the scanner's time in the dump's bz2 bytes, the workers' in its output bytes.",
        f"- **The sample stands for the dump:** {d['windows']} windows of 1 MiB at (2i + 1)/24 of the dump, "
        f"{d['blocks']} of its own blocks; their mix of entities and their ratio, {d['sample_ratio']:.2f}× "
        f"(the windows' 95% interval ±{100 * d['sample_ratio_rel_ci']:.1f}%), stand for the whole dump's. Another "
        "sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) measured 18.68×; wikidata-pq's "
        "CI slice, lines recompressed, is 13.69×.",
        "- **The wall at N threads** is the larger of the scanner's time and the workers' time over N (the scanner "
        "is one thread; the workers run blocks in parallel), times the sample's own wall over that bound: its "
        "overlap and fill taken to grow with the work, which errs long. Read as a fixed cost instead, the same gap "
        "gives the third row.",
        "- **The machine is this runner,** reading the dump from page cache: no download, no disk, no other work. "
        "The output is hashed by the reading thread and discarded.",
        "- The uncertainty is the rounds' spread, with the sample ratio's added in quadrature to the part that "
        "scales with the output; not the runner's: another CPU model gives other figures.",
        "",
        "</details>",
    ]
    return out


def ratio_x(r: list[float] | None) -> str:
    """A ratio as a multiple with its interval: "1.62× (1.60–1.64)"."""
    return f"{r[0]:.2f}× ({r[1]:.2f}–{r[2]:.2f})" if r else "–"


def placement(t: int, cpus: str, env: dict | None) -> str:
    """A thread count's placement as a reader needs it: the 1-thread runs' pinned CPU; a
    pinned set with one CPU on each of distinct cores; or the runs on every CPU, sharing
    cores' SMT siblings where there are more CPUs than cores."""
    if t == 1:
        return f"pinned to CPU {cpus}"
    cores = [set(map(int, c.split(","))) for c in (env or {}).get("core_cpus", [])]
    if cpus not in ("", "all"):
        mine = [int(c) for c in cpus.split(",")]
        hit = {i for c in mine for i, core in enumerate(cores) if c in core}
        return f"CPUs {cpus}, " + ("one a core" if cores and len(hit) == len(mine) else "pinned")
    if env and env.get("cores", 0) < env.get("cpus", 0) and t > env["cores"]:
        return f"all {env['cpus']} CPUs, sharing SMT siblings"
    return "all CPUs"


def conditions(env: dict | None, res: dict) -> str:
    """What the runner gave the runs, in a sentence: cores, quota, placement, throttling."""
    if not env:
        return "The runner's topology was not recorded (`ab env`)."
    groups = " | ".join(env.get("core_cpus", []))
    quota = env.get("cpu_quota")
    two = next(
        (
            c["cpus"]
            for a in (res.get("resources", {}).get("wikidata") or {}).values()
            for t, c in a["threads"].items()
            if t == "2"
        ),
        None,
    )
    parts = [
        f"{env['cpus']} CPUs on {env['cores']} physical cores ({groups})",
        f"a CPU quota of {quota:g} CPUs" if quota else "no CPU quota",
        "the run-queue counter checked and counting"
        if env.get("runq", {}).get("counts")
        else "**the run-queue counter not counting** (its waits read as unavailable)",
    ]
    if two:
        parts.append(
            f"2-thread runs on CPUs {two}"
            + (" (one CPU a core)" if two.count(",") == 1 and env["cores"] >= 2 else "")
        )
    if env["cores"] < env["cpus"]:
        parts.append(f"{max(res['threads'])}-thread runs share cores' SMT siblings")
    return "Conditions: " + "; ".join(parts) + "."


def resources_lines(res: dict, pr: bool, corpus: str = "wikidata") -> list[str]:
    """The resource table and, on a pull request, the assessment: one screen."""
    rs = (res.get("resources") or {}).get(corpus)
    if not rs:
        return []
    shown = [a for a in (["base", "head"] if pr else ["mibz2", "lbzip2", "pbz2"]) if a in rs]
    if not shown:
        return []
    ts = sorted({int(t) for a in shown for t in rs[a]["threads"]})
    out = ["", f"### Resources: CPU per byte and scaling, the `{corpus}` corpus", ""]
    a = res.get("assessment")
    if pr and a:
        out.append(f"**Assessment:** {a['verdict']}")
        if a["broader_hardware"]:
            out.append(
                f"**Needs a machine with more cores before a release:** {'; '.join(a['broader_hardware'])}. "
                f"This runner has {res.get('env', {}).get('cores', '?')} physical cores; a change in CPU "
                "amplification or in memory can grow with the cores (the 10-core workstation's knee: #39, #65)."
            )
        out.append("")
    out += [
        "**CPU per byte** is the process's CPU (every thread, user + sys) over the output. "
        "**Amplification** is that at N threads over the same build's at "
        f"{min(ts)}, paired by round: 1× is no extra CPU for the same bytes, and above it the work "
        "costs more as threads are added (stalls, sharing, synchronisation; CPU counts a thread "
        "stalled on memory as running). **Efficiency** is the speedup over N. **Run-queue** is the "
        "threads' mean count runnable but not running.",
        "",
    ]
    # (on master, the other decoders' amplification only: the control beside mibz2's, on
    # the same runner and rounds)
    full = lambda x: pr or x == "mibz2"
    head = "| threads: placement | " + " | ".join(
        f"{x} MB/s | {x} CPU ns/B | {x} amplification | {x} efficiency"
        if full(x)
        else f"{x} amplification"
        for x in shown
    )
    if pr and len(shown) == 2:
        head += " | amplification, head over base"
    out.append(head + " |")
    out.append(
        "|---|"
        + "".join("--:|--:|--:|--:|" if full(x) else "--:|" for x in shown)
        + ("--:|" if pr and len(shown) == 2 else "")
    )
    pairs = (res.get("paired_resources") or {}).get(corpus, {})
    for t in ts:
        cpus = next(
            (rs[x]["threads"][str(t)]["cpus"] for x in shown if str(t) in rs[x]["threads"]), ""
        )
        row = [f"{t}: {placement(t, cpus, res.get('env'))}"]
        for x in shown:
            c = rs[x]["threads"].get(str(t))
            if not c:
                row += ["–"] * (4 if full(x) else 1)
                continue
            eff = c.get("efficiency")
            if not full(x):
                row.append(ratio_x(c.get("amplification")) if "amplification" in c else "1")
                continue
            row += [
                f"{c['mb_s']:,.0f}",
                f"{c['cpu_ns_per_byte']:.2f}",
                ratio_x(c.get("amplification")) if "amplification" in c else "1",
                f"{100 * eff[0]:.0f}%" if eff else "–",
            ]
        if pr and len(shown) == 2:
            row.append(
                change(pairs.get("amplification_change", {}).get(str(t)))
                .replace("**faster**", "**less**")
                .replace("**slower**", "**more**")
                if str(t) in pairs.get("amplification_change", {})
                else ""
            )
        out.append("| " + " | ".join(row) + " |")
    best = [
        f"{x} at {rs[x]['best']} threads ({rs[x]['threads'][str(rs[x]['best'])]['mb_s']:,.0f} MB/s)"
        for x in shown
    ]
    line = "**Best:** " + "; ".join(best)
    if pr and pairs.get("best_wall_ratio"):
        line += f"; head's best against base's, wall: {change(pairs['best_wall_ratio'])}"
    out += ["", line + "."]
    rq = [
        f"{x} {t}t {c['runq_threads']:.2f}"
        for x in shown
        for t, c in rs[x]["threads"].items()
        if c.get("runq_threads") is not None
    ]
    un = [
        f"{x} {t}t {100 * c['unattributed']:.0f}%"
        for x in shown
        for t, c in rs[x]["threads"].items()
        if c.get("unattributed") is not None and int(t) > 1
    ]
    out += [
        "",
        conditions(res.get("env"), res),
        "",
        "<details><summary>How to read it, and what it cannot show</summary>",
        "",
        "- **The four comparisons the assessment reads,** declared in bench/report.py (`assessment`): "
        "best against best (wall), CPU per byte at the most threads, amplification at each thread "
        "count, and peak RSS at the most threads. Each moves only where its 95% interval excludes no "
        "change. With no real change, about one run in five flags one by chance: a flag that matters "
        "reproduces on a re-run.",
        "- **A trade-off is not a failure.** Faster for more CPU or memory, or the reverse, is for the "
        "reviewer to weigh against the deployment; the report names it, and nothing here fails CI but "
        "the outputs and the accounting.",
        "- **What it can't show:** which shared resource a rise in amplification comes from (cache, "
        "memory bandwidth, SMT, the scanner's handoffs): no hardware counters are read. CI's runners "
        "have 2 physical cores, so it cannot show a knee past 2 (the workstation's is at 4-5 "
        "workers, #39). The cores' L3 is shared with other tenants and its size unknown.",
        "- **Run-queue wait** (threads' mean count runnable, not running; schedstat): "
        + (", ".join(rq) if rq else "not recorded")
        + ". A pinned 2-thread run has four threads (two workers, the scanner, the reader) on two "
        "CPUs, so some wait is its shape, not interference.",
        "- **The per-thread split's unattributed CPU** (the process's total that no sampled thread "
        "holds, sampled every 10 ms): " + (", ".join(un) if un else "not recorded") + ".",
        "",
        "</details>",
    ]
    return out


def stage_lines(st: dict, pr: bool) -> list[str]:
    """The stage table: each stage's median cycles a byte (ns a byte without a clock),
    base → head and head over base on a pull request."""
    arms = st["arms"]
    shown = ["base", "head"] if pr and {"base", "head"} <= set(arms) else list(arms)
    cycles = bool(st.get("mhz"))
    unit = "c/B" if cycles else "ns/B"
    value = lambda a, k: (
        a["stages"][k]["cycles_per_byte"] if cycles else a["stages"][k]["ns_per_byte"]
    )
    last = arms[shown[-1]]
    out = [
        "",
        f"### Where a decode's time goes, stage by stage: the Wikidata blocks on one thread (median {unit})",
        "",
        f"Each of {st['rounds']} rounds runs {'each build' if len(shown) > 1 else 'the build'}'s "
        "`examples/phases.rs` once (bench/stages.py), pinned to one CPU"
        + (", the builds' order alternating" if len(shown) > 1 else "")
        + ": the file decoded in decode order, block by block, each stage timed as it runs"
        " with the library's own functions"
        + (
            f" (each stage's fastest of {st['repeats']} passes)"
            if (st.get("repeats") or 1) > 1
            else ""
        )
        + ", summed over the blocks, "
        + (
            f"in cycles a byte of output at the reported clock ({st['mhz']:.0f} MHz)"
            if cycles
            else "in ns a byte of output"
        )
        + (
            ". **Change** is head over base per stage, paired by round as above; with seven "
            "intervals a run, a mark on one stage alone can be chance (about one run in three). "
            if len(shown) > 1
            else ". "
        )
        + f"Diagnostic only: the decoder's own figures are the tables above. {st['runs']} runs, "
        f"{duration(st['seconds'])} of the run.",
        "",
    ]
    if len(shown) > 1:
        out += [
            f"| stage | base → head {unit} (median) | head share | change |",
            "|---|--:|--:|--:|",
        ]
    else:
        out += [f"| stage | {shown[0]} {unit} (median) | share |", "|---|--:|--:|"]
    for key, name in STAGE_ROWS:
        cells = []
        for arm in shown:
            a = arms[arm]
            cells.append(f"{value(a, key):.2f}" if key in a["stages"] else "–")
        tot = last["stages"].get("total")
        share = (
            f"{100 * last['stages'][key]['ns_per_byte'] / tot['ns_per_byte']:.1f}%"
            if key in last["stages"] and tot and key != "total"
            else ""
        )
        title = f"**{name}**" if key == "total" else name
        if len(shown) > 1:
            out.append(
                f"| {title} | {' → '.join(cells)} | {share} | {change((st.get('change') or {}).get(key))} |"
            )
        else:
            out.append(f"| {title} | {cells[0]} | {share} |")
    named = [(arm, arms[arm].get("walk")) for arm in shown]
    if any(w for _, w in named):
        out += [
            "",
            "The walk row is the stage, the same in every build; its implementation is "
            "each build's own, as its `phases` names it: "
            + "; ".join(f"{arm}, “{w}”" if w else f"{arm}, not named" for arm, w in named)
            + ".",
        ]
    errors = {arm: a["errors"] for arm, a in arms.items() if a["errors"]}
    if errors:
        out += [
            "",
            "**Stage runs failed:** "
            + "; ".join(f"{k}: {', '.join(v)}" for k, v in errors.items()),
        ]
    return out


def alternatives_lines(res: dict, order: list[str]) -> list[str]:
    """The master run's tables: every other decoder's wall, CPU and peak RSS against
    mibz2's, a column each; a single-threaded decoder's figures are its 1-thread runs,
    marked ¹ at every thread count."""
    classes = res.get("parallel") or {"mibz2": True, "pbz2": True, "lbzip2": True}
    others = [a for a in classes if a != "mibz2"]
    single = [a for a in others if not classes[a]]
    head = lambda a: f"{a}{' ¹' if a in single else ''}"
    out = [
        "Each round decodes every corpus once with each decoder at each thread count (`ab run`), their "
        "order rotated round to round. **Against mibz2** is the decoder's wall (or CPU, or peak RSS) over "
        "mibz2's: the geometric mean of the rounds' paired ratios and its 95% interval; above 1, mibz2 is "
        "faster (or leaner). 1-thread runs are pinned to one CPU.",
        "",
        f"Block-parallel: {', '.join(a for a in others if a not in single)}."
        + (
            f" Single-threaded (¹): {', '.join(single)}, run at 1 thread only; at {max(res['threads'])} "
            f"threads their figures are their 1-thread runs, against mibz2's at {max(res['threads'])} threads."
            if single
            else ""
        ),
    ]
    tables = [
        ("Wall time against mibz2", "wall_ratio", True),
        ("CPU time against mibz2", "cpu_ratio", False),
        ("Peak RSS against mibz2", "rss_ratio", False),
    ]
    for title, key, first in tables:
        out += [
            "",
            f"### {title}",
            "",
            "| corpus | threads | "
            + ("mibz2 wall (median) | mibz2 MB/s | mibz2 c/B | " if first else "")
            + ("mibz2 peak RSS | " if key == "rss_ratio" else "")
            + " | ".join(head(a) for a in others)
            + (" | same bytes |" if first else " |"),
            "|---|--:|"
            + ("--:|--:|--:|" if first else "")
            + ("--:|" if key == "rss_ratio" else "")
            + "--:|" * len(others)
            + ("---|" if first else ""),
        ]
        for corpus in order:
            for t, cell in sorted(res["figures"][corpus].items(), key=lambda kv: int(kv[0])):
                a = cell["arms"]
                mb = a.get("mibz2", {})
                name = corpus_cell(corpus, res["corpora"].get(corpus, {}))
                if not mb.get("n"):
                    out.append(f"| {name} | {t} | errors: {mb.get('errors')} |")
                    continue

                def ratio(x: dict) -> str:
                    if not x.get("n"):
                        return f"errors: {', '.join(x.get('errors', [])) or '–'}" if x else "–"
                    r = change(x.get(key), True)
                    return r if x.get("same_bytes", True) else f"{r} **bytes differ**"

                cells = [ratio(a.get(o, {})) for o in others]
                lead = ""
                if first:
                    cb = f"{mb['cycles_per_byte']:.1f}" if mb.get("cycles_per_byte") else "–"
                    lead = f"{mb['wall']:.3f} s | {mb['mb_s']:,.0f} | {cb} | "
                if key == "rss_ratio":
                    lead = f"{mb['rss_mb']:.0f} MB | "
                same = (" | yes" if cell["same_bytes"] else " | **NO**") if first else ""
                out.append(f"| {name} | {t} | {lead}{' | '.join(cells)}{same} |")
    return out


def markdown(res: dict) -> str:
    pr = res["mode"] == "pr"
    n = max(res["threads"])
    m, tc = res["machine"], res["toolchain"]
    run = f" · [run]({res['run']})" if res.get("run") else ""
    lines = [MARK, "## Benchmark: mibz2 decoder", ""]
    if pr:
        lines.append(
            f"**head `{res['head'][:7]}` against base `{res['base'][:7]}`** (the merge base with master): "
            f"{res['rounds']} paired rounds on one runner{run}"
        )
    else:
        others = [a for a in res.get("parallel", {}) if a != "mibz2"] or ["pbz2", "lbzip2"]
        lines.append(
            f"**`{res['head'][:7]}`, mibz2 against {', '.join(others[:-1])} and {others[-1]}**: "
            f"{res['rounds']} rounds on one runner{run}"
        )
    lines += ["", f"{m['cpu']}, {m['cores']} cores · {tc['rustc']} · bench {res['bench']}", ""]
    if pr:
        lines += [
            "Each round decodes every corpus once with each build at each thread count (`ab run`: the base's "
            "harness binary as the arm `mibz2@base/ab`, so both builds run in the same process shape), the "
            "builds' order alternating. **Change** is head over base: the geometric mean of the rounds' paired "
            "ratios and its 95% interval; − is faster, and **faster** or **slower** where the interval excludes "
            "no change. 1-thread runs are pinned to one CPU.",
            "",
            "| corpus | threads | wall, base → head (median) | wall change | CPU change | peak RSS, base → head | "
            "head MB/s | same bytes |",
            "|---|--:|--:|--:|--:|--:|--:|---|",
        ]
    order = sorted(res["figures"], key=lambda c: (c != "wikidata", c))
    if not pr:
        lines += alternatives_lines(res, order)
    for corpus in order if pr else []:
        for t, cell in sorted(res["figures"][corpus].items(), key=lambda kv: int(kv[0])):
            a = cell["arms"]
            same = "yes" if cell["same_bytes"] else "**NO**"
            name = corpus_cell(corpus, res["corpora"].get(corpus, {}))
            b, h = a.get("base", {}), a.get("head", {})
            if not b.get("n") or not h.get("n"):
                lines.append(
                    f"| {name} | {t} | errors: {b.get('errors')} {h.get('errors')} | | | | | {same} |"
                )
                continue
            lines.append(
                f"| {name} | {t} | {b['wall']:.3f} → {h['wall']:.3f} s | {change(h.get('wall_ratio'))} | "
                f"{change(h.get('cpu_ratio'))} | {b['rss_mb']:.0f} → {h['rss_mb']:.0f} MB "
                f"({rss_change(h.get('rss_ratio'))}) | {h['mb_s']:,.0f} | {same} |"
            )
    lines += [
        "",
        "**Same bytes:** every run of every arm gave the same output hash on that corpus, with no error."
        if res["parity"]
        else "**Same bytes: NO.** An arm's output differed or it failed on a corpus: see the table.",
    ]
    if res.get("accounting"):
        lines += [
            "",
            "**The CPU accounting failed its own check** (a thread's CPU counted twice, or a total "
            "missing threads): " + "; ".join(res["accounting"][:5]),
        ]
    lines += resources_lines(res, pr)
    w = res["figures"].get("wikidata", {}).get(str(n), {}).get("arms", {})
    splits = {k: v["cpu_split"] for k, v in w.items() if v.get("cpu_split")}
    if splits:
        lines += [
            "",
            f"### Where mibz2's CPU goes: the Wikidata blocks at {n} threads (median CPU seconds)",
            "",
            "| build | scanner (serial) | workers (summed) | the reading thread and the rest | "
            "unattributed | wall |",
            "|---|--:|--:|--:|--:|--:|",
        ]
        for k, s in splits.items():
            un = f"{s['unattributed']:.2f}" if "unattributed" in s else "–"
            lines.append(
                f"| {k} | {s['scan']:.2f} | {s['workers']:.2f} | {s['other']:.2f} | {un} | "
                f"{w[k]['wall']:.3f} |"
            )
        lines += [
            "",
            "Unattributed: the process's CPU that no sampled thread holds (threads ending between "
            "samples). Before bench 3.0.0 the sampler ran every 50 ms and left 0.1-0.35 s a run "
            "here; a base built before 3.0.0 still does, so its split and the model below undercount "
            "its workers against head's.",
        ]
    if res.get("stages"):
        lines += stage_lines(res["stages"], pr)
    if res.get("kernels"):
        lines += kernel_lines(res["kernels"])
    if res.get("models"):
        lines += model_lines(res["models"], n, pr)
    lines += [
        "",
        "<details><summary>The corpora, the machine and the toolchain</summary>",
        "",
        "| corpus | bz2 bytes | output bytes | SHA-256 | source |",
        "|---|--:|--:|---|---|",
    ]
    for name in order:
        c = res["corpora"].get(name, {})
        lines.append(
            f"| `{name}` | {c.get('bytes', 0):,} | {c.get('out') or 0:,} | `{c.get('sha256', '')[:16]}…` | "
            f"{c.get('source', '')} |"
        )
    lines += [
        "",
        f"- CPU: {m['cpu']}, {m['cores']} cores, reported clock {m['mhz']} MHz (cycles a byte use it), "
        f"flags {', '.join(m['flags'])}; kernel {m['kernel']}; image {m['image']}",
        f"- {tc['rustc']}; "
        + "; ".join(
            f"{k} {v}" if not str(v).startswith(k) else str(v)
            for k, v in tc.items()
            if k != "rustc" and v
        ),
        f"- bench {res['bench']} (bench/report.py), {res['rounds']} rounds, threads "
        f"{', '.join(map(str, res['threads']))}",
        "",
        "</details>",
        "",
    ]
    return "\n".join(lines)


# ---------------------------------------------------------------------------------------


def summary(a: argparse.Namespace) -> None:
    runs = [json.loads(l) for l in Path(a.runs).read_text().splitlines() if l.strip()]
    mach = machine()
    threads = sorted({r["threads"] for r in runs})
    ref = next((r["arm"] for r in runs if "@" in r["arm"]), "mibz2") if a.mode == "pr" else "mibz2"
    figs = figures(runs, ref, mach["mhz"], a.mode == "pr")
    n = max(threads)
    models = {}
    for arm in [ref, "mibz2"] if a.mode == "pr" else ["mibz2"]:
        if (mo := model(runs, arm, n)) is not None:
            models[label(arm, a.mode == "pr")] = mo
    res = {
        "bench": BENCH,
        "mode": a.mode,
        "head": a.head,
        "base": a.base,
        "date": a.date,
        "run": a.run,
        "rounds": len({r["round"] for r in runs}),
        "threads": threads,
        "machine": mach,
        "toolchain": toolchain(sorted({r["arm"] for r in runs})),
        "arms": sorted({r["arm"] for r in runs}),
        "parallel": parallel(runs, a.mode == "pr"),
        "reference": ref,
        "corpora": corpora(Path(a.corpora), runs),
        "dump": dump_facts(),
        "parity": all(c["same_bytes"] for t in figs.values() for c in t.values()),
        "figures": figs,
        "models": models,
    }
    res["accounting"] = accounting(runs)
    res["resources"] = resources(runs, a.mode == "pr")
    if a.env and Path(a.env).exists() and Path(a.env).read_text().strip():
        res["env"] = json.loads(Path(a.env).read_text())
    if a.mode == "pr":
        res["paired_resources"] = paired_resources(runs, "mibz2", ref)
        res["assessment"] = assessment(res)
    if a.stages and Path(a.stages).exists():
        lines = [json.loads(l) for l in Path(a.stages).read_text().splitlines() if l.strip()]
        if lines:
            res["stages"] = stage_figures(lines, mach["mhz"], a.mode == "pr")
    if a.kernels and Path(a.kernels).exists():
        lines = [json.loads(l) for l in Path(a.kernels).read_text().splitlines() if l.strip()]
        if lines and (k := kernel_figures(lines)):
            res["kernels"] = k
    Path(a.out).write_text(json.dumps(res, indent=1, allow_nan=False) + "\n")
    Path(a.md).write_text(markdown(res))
    print(
        f"{a.out}, {a.md}: {len(runs)} runs, parity {'ok' if res['parity'] else 'FAILED'}",
        file=sys.stderr,
    )
    if not res["parity"]:
        sys.exit(1)
    if res["accounting"]:
        print(
            "CPU accounting failed its own check:\n" + "\n".join(res["accounting"]), file=sys.stderr
        )
        sys.exit(1)


def record(a: argparse.Namespace) -> None:
    store, res = Path(a.store), json.loads(Path(a.result).read_text())
    sha = res["head"]
    for sub in ("runs", "raw"):
        (store / sub).mkdir(parents=True, exist_ok=True)
    if not (store / "README.md").exists():
        (store / "README.md").write_text(STORE_README)
    (store / "runs" / f"{sha}.json").write_text(Path(a.result).read_text())
    (store / "raw" / f"{sha}.jsonl").write_text(Path(a.runs).read_text())
    line = {
        "commit": sha,
        "date": res["date"],
        "bench": res["bench"],
        "cpu": res["machine"]["cpu"],
        "cores": res["machine"]["cores"],
        "rounds": res["rounds"],
        "run": res["run"],
        "corpora": sorted(res["corpora"]),
        "parity": res["parity"],
    }
    with open(store / "records.jsonl", "a") as f:
        f.write(json.dumps(line) + "\n")
    print(json.dumps(line))


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("summary")
    s.add_argument("runs")
    s.add_argument("--corpora", required=True)
    s.add_argument("--mode", choices=["pr", "master", "dispatch"], required=True)
    s.add_argument("--head", required=True)
    s.add_argument("--base")
    s.add_argument("--date")
    s.add_argument("--run")
    s.add_argument("--stages")
    s.add_argument("--kernels", help="the kernels binary's passes (bench/alternatives)")
    s.add_argument("--env", help="`ab env`'s line: the runner's cores, quota and counters")
    s.add_argument("--out", required=True)
    s.add_argument("--md", required=True)
    r = sub.add_parser("record")
    r.add_argument("store")
    r.add_argument("result")
    r.add_argument("runs")
    a = ap.parse_args()
    summary(a) if a.cmd == "summary" else record(a)


if __name__ == "__main__":
    main()
