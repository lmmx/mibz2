"""The benchmark's figures from `ab run`'s records (one JSON line a run), the full-dump
model, the pull request comment, and the record kept for a master commit.

    report.py summary RUNS.jsonl --corpora DIR --mode pr|master|dispatch --head SHA
                      [--base SHA] [--date ISO] [--run URL] [--stages STAGES.jsonl]
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
- **Modelled, not measured:** a whole dump's decode (`model`), from the Wikidata blocks.

`record` adds a master commit's result to the results store (the `bench-results`
branch): runs/<commit>.json, raw/<commit>.jsonl and a line of records.jsonl.

BENCH is the benchmark's version (semver), stamped on every result. A change to what is
measured or how (the corpora and their scale, the rounds, the thread counts, the
harness's timing, a figure's meaning) is a new major version, and the site's history
never joins a series across major versions; a change to how results are shown or stored,
or a new arm beside the others that leaves every existing figure as it was, is a minor or
patch version.

- 1.1.0: master's run has every alternative (ALTERNATIVES), each arm classed as
  block-parallel or single-threaded (`parallel`); a single-threaded arm runs at 1 thread
  only, and its 1-thread runs stand, marked, in every thread count's figures.
- 1.2.0: the stage table (`stages`, from bench/stages.py's runs of examples/phases.rs):
  each stage's cycles a byte on the Wikidata blocks at 1 thread, and on a pull request
  head over base per stage. Every existing figure is as it was.
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

BENCH = "1.2.0"
# The other decoders, in the order they are shown: the block-parallel, then the
# single-threaded (examples/ab.rs: `parallel`)
ALTERNATIVES = ["pbz2", "lbzip2", "indexed_bzip2", "fbz", "lbzip2-rs", "libbzip2", "libbz2-rs"]
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

Results compare only within one major bench version, one runner CPU model and one set of corpora.
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
    except OSError:
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


def toolchain(arms: list[str]) -> dict:
    """The compiler and each arm's version (the alternatives' only where they ran)."""
    names = {label(a) for a in arms}
    lbzip2 = run_out("lbzip2", "--version").splitlines()
    alts = "bench/alternatives/Cargo.lock"
    py = os.environ.get("AB_PYTHON", "python3")
    ib = run_out(py, "-c", "import indexed_bzip2; print(indexed_bzip2.__version__)")
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
# The stages


# The stages as shown, in examples/phases.rs's order (bench/stages.py's keys)
STAGE_ROWS = [
    ("scan", "scanner (headers, Huffman)"),
    ("mtf", "MTF + RLE2"),
    ("build", "T-vector build"),
    ("walk", "walk, flagged (used)"),
    ("rle", "RLE1"),
    ("crc", "CRC"),
    ("total", "total"),
]


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
        }
        for key in keys:
            v = [r["ps_per_byte"][key] / 1000 for r in ok.values()]
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
            key: ratio_ci([x[key] / y[key] for x, y in pairs if y.get(key)]) for key in keys
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
        + ": every block's stages timed one by one with the library's own functions"
        + (f" (each the best of {st['repeats']})" if (st.get("repeats") or 1) > 1 else "")
        + ", summed over the blocks, "
        + (
            f"in cycles a byte of output at the reported clock ({st['mhz']:.0f} MHz)"
            if cycles
            else "in ns a byte of output"
        )
        + (
            ". **Change** is head over base per stage, paired by round as above. "
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
    w = res["figures"].get("wikidata", {}).get(str(n), {}).get("arms", {})
    splits = {k: v["cpu_split"] for k, v in w.items() if v.get("cpu_split")}
    if splits:
        lines += [
            "",
            f"### Where mibz2's CPU goes: the Wikidata blocks at {n} threads (median CPU seconds)",
            "",
            "| build | scanner (serial) | workers (summed) | the reading thread and the rest | wall |",
            "|---|--:|--:|--:|--:|",
        ]
        for k, s in splits.items():
            lines.append(
                f"| {k} | {s['scan']:.2f} | {s['workers']:.2f} | {s['other']:.2f} | {w[k]['wall']:.3f} |"
            )
    if res.get("stages"):
        lines += stage_lines(res["stages"], pr)
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
    if a.stages and Path(a.stages).exists():
        lines = [json.loads(l) for l in Path(a.stages).read_text().splitlines() if l.strip()]
        if lines:
            res["stages"] = stage_figures(lines, mach["mhz"], a.mode == "pr")
    Path(a.out).write_text(json.dumps(res, indent=1, allow_nan=False) + "\n")
    Path(a.md).write_text(markdown(res))
    print(
        f"{a.out}, {a.md}: {len(runs)} runs, parity {'ok' if res['parity'] else 'FAILED'}",
        file=sys.stderr,
    )
    if not res["parity"]:
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
