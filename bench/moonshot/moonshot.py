#!/usr/bin/env python3
"""The moonshot scorer (bench/moonshot/README.md): one candidate kernel against mibz2 0.5.6's.

    python3 bench/moonshot/moonshot.py score CANDIDATE.rs [options]

Three parts, reported side by side and never combined into one number:

1. the correctness gate (`moonshot check`): byte-exact against libbzip2, CRCs rechecked; any
   failure fails the score outright and nothing is timed;
2. container timing: the candidate against 0.5.6's kernel (LazyCells), paired, alternating,
   at every worker count from 1 to this machine's CPUs, with a 2 s settle (or back to back),
   the geometric mean ratio with a 95% interval, and peak RSS;
3. the contention proxy (chaingrind, a Valgrind tool): per output byte, modelled transfers,
   DRAM bytes, dependent-miss chain links and the overlap available to them, at the i9's
   worker counts, beside the calibration and held-out arms (lazy, opend, open, cells).

Only CI's paired bench and an i9 workstation run establish production performance.
"""

from __future__ import annotations

import argparse
import datetime
import json
import math
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
CALIBRATION = HERE / "calibration.json"

# ---------------------------------------------------------------- statistics


def _betacf(a: float, b: float, x: float) -> float:
    """the continued fraction of the regularised incomplete beta (Numerical Recipes' betacf)"""
    tiny = 1e-300
    qab, qap, qam = a + b, a + 1.0, a - 1.0
    c, d = 1.0, 1.0 - qab * x / qap
    d = 1.0 / (d if abs(d) > tiny else tiny)
    h = d
    for m in range(1, 300):
        m2 = 2 * m
        aa = m * (b - m) * x / ((qam + m2) * (a + m2))
        d = 1.0 + aa * d
        d = 1.0 / (d if abs(d) > tiny else tiny)
        c = 1.0 + aa / c
        c = c if abs(c) > tiny else tiny
        h *= d * c
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))
        d = 1.0 + aa * d
        d = 1.0 / (d if abs(d) > tiny else tiny)
        c = 1.0 + aa / c
        c = c if abs(c) > tiny else tiny
        delta = d * c
        h *= delta
        if abs(delta - 1.0) < 1e-14:
            break
    return h


def betainc(a: float, b: float, x: float) -> float:
    """the regularised incomplete beta I_x(a, b)"""
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    lbt = (
        math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * math.log(1 - x)
    )
    if x < (a + 1) / (a + b + 2):
        return math.exp(lbt) * _betacf(a, b, x) / a
    return 1.0 - math.exp(lbt) * _betacf(b, a, 1 - x) / b


def t_cdf(t: float, df: int) -> float:
    """Student's t distribution's CDF"""
    x = df / (df + t * t)
    tail = 0.5 * betainc(df / 2, 0.5, x)
    return 1 - tail if t > 0 else tail


def t_quantile(p: float, df: int) -> float:
    """the t distribution's p quantile (p in (0.5, 1)), by bisection on the CDF"""
    lo, hi = 0.0, 1000.0
    for _ in range(200):
        mid = (lo + hi) / 2
        if t_cdf(mid, df) < p:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def paired_ratio(cand: list[float], base: list[float]) -> dict:
    """The rounds' paired wall ratios cand/base: their geometric mean and its 95% interval (a
    t interval on the logs). Fewer than two pairs: the mean alone, no interval."""
    if len(cand) != len(base) or not cand:
        raise ValueError("paired_ratio needs equal, non-empty lists")
    if any(x <= 0 for x in cand + base):
        raise ValueError("walls must be positive")
    logs = [math.log(c / b) for c, b in zip(cand, base)]
    n = len(logs)
    mean = sum(logs) / n
    out = {"n": n, "geomean": math.exp(mean), "lo": None, "hi": None}
    if n >= 2:
        sd = math.sqrt(sum((x - mean) ** 2 for x in logs) / (n - 1))
        half = t_quantile(0.975, n - 1) * sd / math.sqrt(n)
        out["lo"], out["hi"] = math.exp(mean - half), math.exp(mean + half)
    return out


def verdict(r: dict) -> str:
    """FASTER or SLOWER when the interval excludes 1, else INCONCLUSIVE (no interval: n < 2)"""
    if r["lo"] is None:
        return "NO INTERVAL"
    if r["hi"] < 1:
        return "FASTER"
    if r["lo"] > 1:
        return "SLOWER"
    return "INCONCLUSIVE"


def median(v: list[float]) -> float:
    s = sorted(v)
    n = len(s)
    return s[n // 2] if n % 2 else (s[n // 2 - 1] + s[n // 2]) / 2


def order(arms: list[str], rnd: int) -> list[str]:
    """round `rnd`'s order: the arms rotated by the round, reversed on odd rotations, so each
    arm runs in each position equally often and before and after each other arm"""
    k = len(arms)
    rot = arms[rnd % k :] + arms[: rnd % k]
    return rot[::-1] if (rnd // k) % 2 else rot


def check_settle(settle: float) -> float:
    """the settle between runs: 0 (back to back) or at least 2 s. A short idle gap is refused:
    MONODROMY record 68 §4 found a 0.25 s gap gives slow starts"""
    if settle != 0 and settle < 2:
        raise ValueError(f"settle {settle} s: use 0 (back to back) or at least 2 s")
    return settle


# ---------------------------------------------------------------- the gate


def gate_verdict(lines: list[str], returncode: int) -> dict:
    """`moonshot check`'s JSON lines read: PASS only with exit 0, a final PASS line, at least one
    case and no failing case"""
    cases, final = [], None
    for line in lines:
        line = line.strip()
        if not line.startswith("{"):
            continue
        d = json.loads(line)
        if "verdict" in d:
            final = d
        elif "case" in d:
            cases.append(d)
    failed = [c for c in cases if not c.get("pass")]
    ok = (
        returncode == 0
        and final is not None
        and final["verdict"] == "PASS"
        and cases
        and not failed
    )
    return {
        "verdict": "PASS" if ok else "FAIL",
        "cases": len(cases),
        "failed": [{"case": c["case"], "failure": c.get("failure")} for c in failed],
        "exit": returncode,
    }


# ---------------------------------------------------------------- the proxy

PROXY_RE = re.compile(r"MOONSHOT-PROXY (\{.*\})")


def parse_proxy(text: str) -> dict:
    """chaingrind's output line, read"""
    m = PROXY_RE.search(text)
    if not m:
        raise ValueError("no MOONSHOT-PROXY line in chaingrind's output")
    return json.loads(m.group(1))


def proxy_metrics(p: dict) -> dict:
    """per output byte: the three proxy quantities kept apart (transfers, chains, overlap),
    with the model's cycles and work"""
    b = p["out_bytes"]
    if b <= 0:
        raise ValueError("the proxy decoded no bytes")
    dram_lines = p["dram_read_lines"] + p["dram_write_lines"] + p["dram_drain_lines"]
    dep = p["dep_dram_loads"]
    return {
        # transfers
        "l2_bytes_per_byte": 64 * p["l2_transfers"] / b,
        "l3_bytes_per_byte": 64 * p["l3_transfers"] / b,
        "dram_bytes_per_byte": 64 * dram_lines / b,
        # chains
        "dep_dram_links_per_byte": dep / b,
        "dep_past_l2_links_per_byte": p["dep_past_l2_loads"] / b,
        "dep_loads_per_byte": p["dep_any_loads"] / b,
        "max_dram_chain": p["max_chain_dram"],
        "max_past_l2_chain": p["max_chain_past_l2"],
        # overlap
        "dram_mlp": p["dram_latency_sum"] / p["dram_busy_cycles"] if p["dram_busy_cycles"] else 0.0,
        "overlap_per_dep_link": p["dep_overlap_sum"] / p["dep_overlap_n"]
        if p["dep_overlap_n"]
        else None,
        "dram_loads_per_dep_link": p["dram_loads"] / dep if dep else None,
        # the miss mix and the work
        "past_l2_loads_per_byte": p["past_l2_loads"] / b,
        "dram_loads_per_byte": p["dram_loads"] / b,
        "instructions_per_byte": p["instructions"] / b,
        "cycles_per_byte": p["cycles"] / b,
    }


# the access-pattern statistics the out-of-calibration flag watches
RANGE_KEYS = [
    "dram_latency_sensitivity",
    "dep_dram_links_per_byte",
    "dep_past_l2_links_per_byte",
    "l2_bytes_per_byte",
    "l3_bytes_per_byte",
    "dram_bytes_per_byte",
    "past_l2_loads_per_byte",
    "dram_loads_per_byte",
    "instructions_per_byte",
]


def out_of_range(cand: dict, refs: list[dict], margin: float = 0.10) -> list[str]:
    """the statistics of `cand` outside the range the reference arms span (widened by `margin`
    of that range's top): where the proxy has no calibration and its prediction is unreliable"""
    out = []
    for k in RANGE_KEYS:
        vals = [r[k] for r in refs if r.get(k) is not None]
        if not vals or cand.get(k) is None:
            continue
        lo, hi = min(vals), max(vals)
        pad = margin * max(abs(hi), 1e-12)
        if cand[k] < lo - pad or cand[k] > hi + pad:
            out.append(f"{k} {cand[k]:.4g} outside [{lo:.4g}, {hi:.4g}]")
    return out


def pick(items: list, k: int) -> list:
    """`k` of `items` spread evenly, the middle of each stretch (all of them if k >= len)"""
    if k >= len(items):
        return list(items)
    return [items[(2 * i + 1) * len(items) // (2 * k)] for i in range(k)]


def proxy_sample(cal: dict, spec: str) -> dict:
    """the proxy's blocks: `spec` is FULL+TINY, how many of the bench corpus's full blocks and
    of its tiny ones (the dump's stream tails, a few hundred bytes each) to decode, spread evenly,
    each after the warm block. The full blocks' first `slope` are also run at a second DRAM
    latency"""
    b = cal["model"]["blocks"]
    nf, nt = (int(x) for x in spec.split("+"))
    full = pick([x for x in b["full"] if x != b["warm"]], nf)
    tiny = pick([x for x in b["tiny"] if x != b["warm"]], nt)
    return {
        "warm": b["warm"],
        "full": full,
        "tiny": tiny,
        "slope": pick(full, min(len(full), cal["model"]["slope_blocks"])),
        "n_full": len(b["full"]),
        "n_tiny": len(b["tiny"]),
    }


def corpus_total(runs: dict, full: list, tiny: list, n_full: int, n_tiny: int) -> dict:
    """one pass of the corpus, estimated from per-block runs ({block: chaingrind's counters}):
    each stratum's mean block times the corpus's count of such blocks, summed; the longest chains
    are the longest seen"""
    out: dict = {}
    for blocks, n in ((full, n_full), (tiny, n_tiny)):
        if not blocks:
            continue
        for k in runs[blocks[0]]:
            if k == "config":
                continue
            vals = [runs[b][k] for b in blocks]
            if k.startswith("max_"):
                out[k] = max(out.get(k, 0), max(vals))
            else:
                out[k] = out.get(k, 0) + sum(vals) / len(vals) * n
    return out


def dram_latency(gbs: float, m: dict) -> float:
    """the i9's L3-miss latency (cycles) at `gbs` GB/s at the memory controller: a queue,
    idle + k U/(1 - U) with U the share of the measured ceiling, fitted to bench-imc-ten's four
    (bandwidth, latency) readings (calibration.json's model.dram), never to an arm's timing"""
    d = m["dram"]
    u = min(max(gbs, 0.0) / d["ceiling_gbs"], d["u_max"])
    return d["idle"] + d["k"] * u / (1 - u)


def nominal_latencies(m: dict, workers: int, kappa: float | None = None) -> dict:
    """the latencies a proxy run is made at, for `workers` contending workers: the LLC's share
    (20 MiB over the workers); the L3-hit latency, its floor plus kappa cycles a further worker
    (the one fitted parameter); the DRAM latency at lazy's nominal bandwidth there (the i9's
    20.0 GB/s at 7 workers, in proportion to the workers). The fixed point (`solve`) moves each
    arm off this DRAM latency by its measured slope"""
    k = m["kappa"] if kappa is None else kappa
    a = m["lazy_anchor"]
    return {
        "llc_kb": int(m["llc_kb"] / workers),
        "l3_lat": int(round(m["l3_latency_floor"] + k * (workers - 1))),
        "dram_lat": int(round(dram_latency(a["gbs"] * workers / a["workers"], m))),
    }


def cell(
    total: dict, slope_lo: dict | None, slope_hi: dict | None, lat0: float, delta: float
) -> dict:
    """one arm at one worker count, for the fixed point: cycles a byte at the run's DRAM latency,
    that latency, the relative slope of cycles in DRAM latency (from the slope blocks' two runs)
    and modelled DRAM bytes a byte"""
    rel = 0.0
    if slope_lo and slope_hi and slope_lo["cycles"]:
        rel = (slope_hi["cycles"] / slope_lo["cycles"] - 1) / delta
    b = total["out_bytes"]
    dram = total["dram_read_lines"] + total["dram_write_lines"] + total["dram_drain_lines"]
    return {"cpb": total["cycles"] / b, "lat0": lat0, "rel_slope": rel, "dram_bpb": 64 * dram / b}


def solve(cells: dict, m: dict, iters: int = 60) -> dict:
    """the bandwidth-latency fixed point. cells: {W: {arm: cell()}}, lazy among the arms at the
    anchor's W. An arm's cycles a byte at DRAM latency L are cpb (1 + rel_slope (L - lat0)).
    - lazy: its bandwidth at W is the anchor's (20.0 GB/s at 7 workers, measured) scaled by its
      modelled bytes a second, W (DRAM B/B + c0) / cycles a byte, against the anchor's; its
      latency the queue's at that bandwidth;
    - an arm: lazy's bandwidth at W times its DRAM bytes a byte and its rate against lazy's, and
      its latency the queue's at that.
    c0 is the process's DRAM traffic outside decode_block (input, scan, output copy), the i9's
    whole-process 8.0 B/B for lazy at 7 workers less the model's, never below 0.
    Returns {W: {arm: {cpb, lat, gbs}}}"""
    a = m["lazy_anchor"]
    aw = str(a["workers"])
    if aw not in cells or "lazy" not in cells[aw]:
        raise ValueError(f"the fixed point needs lazy at the anchor's {aw} workers")
    c0 = max(0.0, m["i9_lazy_dram_bpb"] - cells[aw]["lazy"]["dram_bpb"])

    def cpb(c: dict, lat: float) -> float:
        return c["cpb"] * max(0.05, 1 + c["rel_slope"] * (lat - c["lat0"]))

    def fixed(gbs_at) -> float:
        """the latency L with L = dram_latency(gbs_at(L)): the bandwidth falls as the latency
        rises (the cycles do), so L - dram_latency(gbs_at(L)) rises and bisection finds it"""
        lo, hi = 0.0, dram_latency(1e12, m)
        for _ in range(iters):
            mid = (lo + hi) / 2
            if mid < dram_latency(gbs_at(mid), m):
                lo = mid
            else:
                hi = mid
        return (lo + hi) / 2

    def rate(w: str, lat: float) -> float:
        lz = cells[w]["lazy"]
        return int(w) * (lz["dram_bpb"] + c0) / cpb(lz, lat)

    lat7 = dram_latency(a["gbs"], m)
    r7 = rate(aw, lat7)
    lat = {w: fixed(lambda L, w=w: a["gbs"] * rate(w, L) / r7) for w in cells if "lazy" in cells[w]}
    lat[aw] = lat7
    gbs = {w: a["gbs"] * rate(w, lat[w]) / r7 for w in lat}
    out: dict = {}
    for w, arms in cells.items():
        if w not in lat:
            continue
        lz = arms["lazy"]
        c_l = cpb(lz, lat[w])
        out[w] = {"lazy": {"cpb": c_l, "lat": lat[w], "gbs": gbs[w], "c0": c0}}
        for arm, c in arms.items():
            if arm == "lazy":
                continue

            def g_at(L, c=c):
                return gbs[w] * (c["dram_bpb"] + c0) / (lz["dram_bpb"] + c0) * c_l / cpb(c, L)

            la = fixed(g_at)
            out[w][arm] = {"cpb": cpb(c, la), "lat": la, "gbs": g_at(la)}
    return out


def ratio(sol: dict, arm: str, w: int, ref_w: int) -> float:
    """the model's wall ratio of `arm` at `w` workers over lazy at `ref_w`, each worker at its
    cycles a byte (the workers saturated, the wall the bytes over their rate)"""
    return (sol[str(w)][arm]["cpb"] / w) / (sol[str(ref_w)]["lazy"]["cpb"] / ref_w)


def predicted_ratio(cand_cpb: float, cand_w: int, ref_cpb: float, ref_w: int) -> float:
    """the same, from cycles a byte directly"""
    return (cand_cpb / cand_w) / (ref_cpb / ref_w)


def bootstrap_cells(
    runs: dict, sample: dict, m: dict, reps: int = 400, seed: int = 1
) -> list[dict]:
    """the fixed point on `reps` resamples of the blocks, the same blocks for every arm (paired);
    the slopes held at the whole sample's. runs: {W: {arm: {"base": {block: counters}, "lo":
    {...}, "hi": {...}, "lat0": L}}}"""
    rng = __import__("random").Random(seed)
    full, tiny = sample["full"], sample["tiny"]
    delta = m["slope_delta"]
    base = {
        w: {
            a: cell(
                corpus_total(r["base"], full, tiny, sample["n_full"], sample["n_tiny"]),
                corpus_total(r["lo"], sample["slope"], [], 1, 0) if r.get("lo") else None,
                corpus_total(r["hi"], sample["slope"], [], 1, 0) if r.get("hi") else None,
                r["lat0"],
                delta,
            )
            for a, r in arms.items()
        }
        for w, arms in runs.items()
    }
    sols = []
    for _ in range(reps):
        f = [rng.choice(full) for _ in full]
        t = [rng.choice(tiny) for _ in tiny] if tiny else []
        cells = {}
        for w, arms in runs.items():
            cells[w] = {}
            for a, r in arms.items():
                tot = corpus_total(r["base"], f, t, sample["n_full"], sample["n_tiny"])
                c = dict(base[w][a])
                c["cpb"] = tot["cycles"] / tot["out_bytes"]
                c["dram_bpb"] = (
                    64
                    * (tot["dram_read_lines"] + tot["dram_write_lines"] + tot["dram_drain_lines"])
                    / tot["out_bytes"]
                )
                cells[w][a] = c
        sols.append(solve(cells, m))
    return sols


def quantiles(v: list[float], lo: float = 0.025, hi: float = 0.975) -> tuple[float, float]:
    v = sorted(v)
    return v[int(lo * (len(v) - 1))], v[int(round(hi * (len(v) - 1)))]


# ---------------------------------------------------------------- driving


def log(msg: str) -> None:
    print(f"moonshot: {msg}", file=sys.stderr, flush=True)


class Scorer:
    def __init__(self, a: argparse.Namespace):
        self.a = a
        self.target = Path(a.target_dir or HERE / "target").resolve()
        self.bin = self.target / "release" / "moonshot"
        self.cal = json.loads(CALIBRATION.read_text())

    def build(self, candidate: Path) -> None:
        env = dict(os.environ, MOONSHOT_CANDIDATE=str(candidate), CARGO_TARGET_DIR=str(self.target))
        log(f"building the harness with {candidate}")
        subprocess.run(
            [
                "cargo",
                "build",
                "--release",
                "--locked",
                "--manifest-path",
                str(HERE / "Cargo.toml"),
            ],
            env=env,
            check=True,
        )

    def info(self) -> dict:
        return json.loads(
            subprocess.run(
                [str(self.bin), "info"], capture_output=True, text=True, check=True
            ).stdout
        )

    def gate(self, corpus: Path | None, out: Path) -> dict:
        cmd = [str(self.bin), "check", "--arm", "candidate"]
        if corpus:
            cmd += ["--wikidata", str(corpus)]
        if self.a.quick:
            cmd.append("--quick")
        log("the correctness gate: " + " ".join(cmd[1:]))
        p = subprocess.run(cmd, capture_output=True, text=True)
        (out / "check.jsonl").write_text(p.stdout)
        (out / "check.stderr").write_text(p.stderr)
        return gate_verdict(p.stdout.splitlines(), p.returncode)

    def run_once(self, arm: str, workers: int, corpus: Path, perf: Path | None = None) -> dict:
        cmd = [str(self.bin), "run", "--arm", arm, "--workers", str(workers), "--file", str(corpus)]
        cmd += ["--repeat", str(self.a.repeat)]
        if perf:
            cmd = ["perf", "stat", "-x,", "-o", str(perf), "-e", ",".join(PERF_EVENTS), "--"] + cmd
        p = subprocess.run(cmd, capture_output=True, text=True, check=True)
        r = json.loads(p.stdout.strip().splitlines()[-1])
        if perf:
            r["counters"] = parse_perf(perf.read_text())
            perf.unlink()
        return r

    def timing(
        self, corpus: Path, out: Path, arms: list[str] | None = None, perf: bool = False
    ) -> dict:
        settle = check_settle(self.a.settle)
        cpus = os.cpu_count() or 1
        counts = list(range(1, cpus + 1)) if not self.a.workers else self.a.workers
        arms = arms or ["lazy", "candidate"] + (["opend"] if self.a.with_opend else [])
        runs = []
        with open(out / "runs.jsonl", "w") as f:
            for rnd in range(self.a.rounds):
                for w in counts:
                    for arm in order(arms, rnd + w):
                        r = self.run_once(arm, w, corpus, out / "perf.tmp" if perf else None)
                        r["round"] = rnd
                        runs.append(r)
                        f.write(json.dumps(r) + "\n")
                        f.flush()
                        if settle:
                            time.sleep(settle)
                log(f"round {rnd + 1} of {self.a.rounds} timed")
        return summarise_timing(runs, arms, counts)

    def tool(self) -> Path | str:
        """chaingrind built into the target dir: its path, or why it isn't there"""
        if not shutil.which("valgrind"):
            return "valgrind is not installed (apt-get install valgrind)"
        vlib = self.target / "chaingrind"
        b = subprocess.run(
            ["bash", str(HERE / "chaingrind" / "build.sh"), str(vlib)],
            capture_output=True,
            text=True,
        )
        if b.returncode:
            return "chaingrind did not build: " + (b.stdout + b.stderr).strip()[-400:]
        return vlib

    def proxy_runs(
        self,
        vlib: Path,
        arms: list[str],
        workers: list[int],
        sample: dict,
        corpus: Path,
        out: Path,
        kappa: float | None = None,
        slopes: bool = True,
    ) -> dict:
        """chaingrind on each sampled block, one Valgrind process a block (the block decoded once
        unseen, the warm block decoded to set the caches, then the block measured), for each
        arm and worker count, in parallel over this machine's CPUs; with `slopes`, the slope
        blocks again at the DRAM latency plus the model's slope_delta. Each run's counters go to
        proxy-runs.jsonl. Returns {W: {arm: {"base", "lo", "hi": {block: counters}, "lat0"}}}"""
        from concurrent.futures import ThreadPoolExecutor

        m = self.cal["model"]
        syms = out / "blocks"
        syms.mkdir(exist_ok=True)
        blocks = sample["full"] + sample["tiny"]
        for b in blocks:
            f = syms / f"{b}.sym"
            if not f.exists():
                subprocess.run(
                    [
                        str(self.bin),
                        "extract",
                        "--file",
                        str(corpus),
                        "--blocks",
                        f"{sample['warm']},{b}",
                        "--out",
                        str(f),
                    ],
                    check=True,
                    capture_output=True,
                )
        jobs = []
        for w in workers:
            lat = nominal_latencies(m, w, kappa)
            hi = dict(lat, dram_lat=lat["dram_lat"] + m["slope_delta"])
            for arm in arms:
                jobs += [(w, arm, "base", b, lat) for b in blocks]
                if slopes:
                    jobs += [(w, arm, "hi", b, hi) for b in sample["slope"]]
        res: dict = {}
        log(
            f"proxy: {len(jobs)} chaingrind runs ({len(arms)} arms, {len(workers)} worker counts,"
            f" {len(sample['full'])}+{len(sample['tiny'])} blocks)"
        )
        done = [0]

        def run(j):
            w, arm, kind, b, lat = j
            p = self.chaingrind(vlib, arm, syms / f"{b}.sym", lat)
            done[0] += 1
            if done[0] % 200 == 0:
                log(f"proxy: {done[0]} of {len(jobs)} runs")
            return j, p

        with (
            ThreadPoolExecutor(os.cpu_count() or 1) as ex,
            open(out / "proxy-runs.jsonl", "a") as f,
        ):
            for (w, arm, kind, b, lat), p in ex.map(run, jobs):
                f.write(
                    json.dumps(
                        {
                            "workers": w,
                            "arm": arm,
                            "kind": kind,
                            "block": b,
                            "latencies": lat,
                            "counters": p,
                        }
                    )
                    + "\n"
                )
                r = res.setdefault(str(w), {}).setdefault(
                    arm, {"base": {}, "lat0": nominal_latencies(m, w, kappa)["dram_lat"]}
                )
                r.setdefault(kind, {})[b] = p
        for arms_ in res.values():
            for r in arms_.values():
                if "hi" in r:
                    r["lo"] = {b: r["base"][b] for b in sample["slope"]}
        return res

    def proxy(self, corpus: Path, out: Path) -> dict:
        """the candidate and lazy on the proxy's sample at the i9's worker counts, read against
        the reference arms in calibration.json"""
        vlib = self.tool()
        if isinstance(vlib, str):
            return {"skipped": vlib}
        m = self.cal["model"]
        sample = proxy_sample(self.cal, self.a.proxy_blocks)
        runs = self.proxy_runs(vlib, ["lazy", "candidate"], m["workers"], sample, corpus, out)
        res = read_proxy(runs, sample, self.cal, reps=self.a.bootstrap)
        (out / "proxy.json").write_text(json.dumps(res, indent=1) + "\n")
        return res

    def chaingrind(self, vlib: Path, arm: str, sym: Path, lat: dict) -> dict:
        m = self.cal["model"]
        cmd = [
            "valgrind",
            "--tool=chaingrind",
            f"--llc-kb={lat['llc_kb']}",
            f"--l3-lat={lat['l3_lat']}",
            f"--dram-lat={lat['dram_lat']}",
            f"--l1-lat={m['l1_latency']}",
            f"--l2-lat={m['l2_latency']}",
            f"--rob={m['rob']}",
            f"--width={m['width']}",
            f"--mshr={m['mshr']}",
            str(self.bin),
            "proxy",
            "--arm",
            arm,
            "--warm",
            "--symbols",
            str(sym),
        ]
        p = subprocess.run(
            cmd, capture_output=True, text=True, env=dict(os.environ, VALGRIND_LIB=str(vlib))
        )
        if p.returncode:
            raise RuntimeError(f"chaingrind failed on {arm}: {p.stderr[-800:]}")
        return parse_proxy(p.stderr)


# the stall counters a calibration run reads (Comet Lake's names), per process
# (the last two give the mean L3-miss latency of demand reads, outstanding over requests, as
# bench-imc-ten measured it: the scorer's DRAM latency, per arm, without the memory controller's
# counters or sudo; five programmable events on four counters, so perf multiplexes and scales)
PERF_EVENTS = [
    "cycles",
    "instructions",
    "cycle_activity.stalls_l3_miss",
    "l1d_pend_miss.pending",
    "l1d_pend_miss.pending_cycles",
    "offcore_requests.l3_miss_demand_data_rd",
    "offcore_requests_outstanding.l3_miss_demand_data_rd",
]


def parse_perf(text: str) -> dict:
    """`perf stat -x,`'s lines: each counted event's value (an event not counted is left out)"""
    out = {}
    for line in text.splitlines():
        f = line.split(",")
        if len(f) < 3 or line.startswith("#"):
            continue
        try:
            out[f[2]] = float(f[0])
        except ValueError:
            continue
    return out


def sample_blocks(total: int, n: int) -> list[int]:
    """`n` block indices spread evenly over `total` blocks, the middle of each stretch"""
    n = max(1, min(n, total))
    return sorted({(2 * i + 1) * total // (2 * n) for i in range(n)})


def summarise_timing(runs: list[dict], arms: list[str], counts: list[int]) -> dict:
    """per worker count: each arm over lazy, paired by round"""
    out = {}
    for w in counts:
        by = {a: {r["round"]: r for r in runs if r["arm"] == a and r["workers"] == w} for a in arms}
        rounds = sorted(set.intersection(*(set(by[a]) for a in arms)))
        cell = {"rounds": len(rounds)}
        for a in arms:
            if a == "lazy":
                continue
            cand = [by[a][r]["wall_s"] for r in rounds]
            base = [by["lazy"][r]["wall_s"] for r in rounds]
            pr = paired_ratio(cand, base)
            pr["verdict"] = verdict(pr)
            pr["speedup"] = 1 / pr["geomean"]
            pr["peak_rss_ratio"] = median([by[a][r]["peak_rss_kb"] for r in rounds]) / median(
                [by["lazy"][r]["peak_rss_kb"] for r in rounds]
            )
            pr["wall_s"] = median(cand)
            cell[a] = pr
        if rounds and all("counters" in by[a][r] for a in arms for r in rounds):
            cell["counters_per_byte"] = {
                a: {
                    e: median(
                        [
                            by[a][r]["counters"].get(e, float("nan")) / by[a][r]["out_bytes"]
                            for r in rounds
                        ]
                    )
                    for e in PERF_EVENTS
                }
                for a in arms
            }
        cell["lazy_wall_s"] = median([by["lazy"][r]["wall_s"] for r in rounds])
        cell["lazy_peak_rss_kb"] = median([by["lazy"][r]["peak_rss_kb"] for r in rounds])
        out[str(w)] = cell
    return out


def load_runs(path: Path) -> tuple[dict, dict]:
    """proxy-runs.jsonl read back: the runs in proxy_runs' shape, and the latencies of each W's
    base runs"""
    res: dict = {}
    lats: dict = {}
    for line in path.read_text().splitlines():
        j = json.loads(line)
        w = str(j["workers"])
        r = res.setdefault(w, {}).setdefault(j["arm"], {"base": {}})
        r.setdefault(j["kind"], {})[j["block"]] = j["counters"]
        if j["kind"] == "base":
            r["lat0"] = j["latencies"]["dram_lat"]
            lats[w] = j["latencies"]
    for arms in res.values():
        for r in arms.values():
            if "hi" in r:
                r["lo"] = {b: r["base"][b] for b in r["hi"]}
    return res, lats


def read_proxy(runs: dict, sample: dict, cal: dict, reps: int = 400) -> dict:
    """the proxy's reading of per-block runs ({W: {arm: ...}}, `Scorer.proxy_runs`): each arm's
    metrics a byte (corpus-weighted), the fixed point, each arm's predicted ratio over lazy with a
    paired bootstrap 95% interval over the blocks and the metrics that drive it, the i9 cells
    where their arms were run, and the candidate's range check against calibration.json's
    reference arms"""
    m = cal["model"]
    nf, nt = sample["n_full"], sample["n_tiny"]
    metrics: dict = {}
    cells: dict = {}
    for w, arms in runs.items():
        for a, r in arms.items():
            tot = corpus_total(r["base"], sample["full"], sample["tiny"], nf, nt)
            lo = corpus_total(r["lo"], sample["slope"], [], 1, 0) if r.get("lo") else None
            hi = corpus_total(r["hi"], sample["slope"], [], 1, 0) if r.get("hi") else None
            c = cell(tot, lo, hi, r["lat0"], m["slope_delta"])
            cells.setdefault(w, {})[a] = c
            x = proxy_metrics(tot)
            # (cycles a byte rise, in per cent, for 100 cycles more DRAM latency)
            x["dram_latency_sensitivity"] = 100 * 100 * c["rel_slope"]
            metrics.setdefault(a, {})[w] = x
    sol = solve(cells, m)
    boots = bootstrap_cells(runs, sample, m, reps=reps) if reps else []
    preds: dict = {}
    for w in sol:
        for a in sol[w]:
            if a == "lazy":
                continue
            r = ratio(sol, a, int(w), int(w))
            iv = quantiles([ratio(b, a, int(w), int(w)) for b in boots]) if boots else (None, None)
            ml, ma = metrics["lazy"][w], metrics[a][w]
            common = cells[w][a]["cpb"] / cells[w]["lazy"]["cpb"]
            preds.setdefault(a, {})[w] = {
                "ratio": r,
                "lo": iv[0],
                "hi": iv[1],
                "drivers": {
                    "instructions_ratio": ma["instructions_per_byte"] / ml["instructions_per_byte"],
                    "at_nominal_latency": common,
                    "contention_feedback": r / common,
                    "dram_bytes_ratio": ma["dram_bytes_per_byte"] / ml["dram_bytes_per_byte"],
                    "dram_latency": [sol[w][a]["lat"], sol[w]["lazy"]["lat"]],
                    "dram_gbs": [sol[w][a]["gbs"], sol[w]["lazy"]["gbs"]],
                    "latency_sensitivity": [
                        ma["dram_latency_sensitivity"],
                        ml["dram_latency_sensitivity"],
                    ],
                    "dep_past_l2_links": [
                        ma["dep_past_l2_links_per_byte"],
                        ml["dep_past_l2_links_per_byte"],
                    ],
                    "dram_mlp": [ma["dram_mlp"], ml["dram_mlp"]],
                },
            }
    # the i9 cells whose arms were run
    checks = []
    for c in cal["i9"]:
        cw, rw = str(c["workers"]), str(c["reference_workers"])
        if c["arm"] not in sol.get(cw, {}) or rw not in sol:
            continue
        p = ratio(sol, c["arm"], int(cw), int(rw))
        iv = (
            quantiles([ratio(b, c["arm"], int(cw), int(rw)) for b in boots])
            if boots
            else (None, None)
        )
        d = (metrics[c["arm"]][cw]["dram_bytes_per_byte"] + sol[rw]["lazy"]["c0"]) / (
            metrics["lazy"][rw]["dram_bytes_per_byte"] + sol[rw]["lazy"]["c0"]
        )
        checks.append(
            dict(
                c,
                predicted=p,
                lo=iv[0],
                hi=iv[1],
                predicted_dram_ratio=d,
                same_side=(p < 1) == (c["measured"] < 1),
                covered=None if iv[0] is None else iv[0] <= c["measured"] <= iv[1],
                error=p - c["measured"],
            )
        )
    pairs, right = 0, 0
    for i, x in enumerate(checks):
        for y in checks[i + 1 :]:
            if x["threads"] != y["threads"] or "held-out" not in (x["role"], y["role"]):
                continue
            pairs += 1
            right += (x["predicted"] < y["predicted"]) == (x["measured"] < y["measured"])
    held = [c for c in checks if c["role"] == "held-out"]
    flags = {}
    ref = cal.get("reference", {})
    if "candidate" in metrics:
        for w, x in metrics["candidate"].items():
            refs = list(ref.get(w, {}).values())
            flags[w] = (
                out_of_range(x, refs)
                if refs
                else ["no reference arms at this worker count (moonshot.py calibrate)"]
            )
    return {
        "sample": sample,
        "kappa": m["kappa"],
        "metrics": metrics,
        "fixed_point": sol,
        "predicted_over_lazy": preds,
        "calibration": {
            "fitted": [c for c in checks if c["role"] == "fitted"],
            "held_out": held,
            "held_out_pairs_ranked_right": [right, pairs],
            "held_out_all_same_side": all(c["same_side"] for c in held) if held else None,
        },
        "out_of_calibration": flags,
    }


# ---------------------------------------------------------------- the report


def fmt(x, nd=3) -> str:
    if x is None:
        return "-"
    if isinstance(x, float):
        return f"{x:.{nd}f}"
    return str(x)


def proxy_report(p: dict, ref: dict) -> list[str]:
    """section 3 of a score: the candidate's predicted ratio over lazy at each worker count with
    its interval, the metrics that drive it, its three quantities beside lazy's, and the flag"""
    L = [
        "3. CONTENTION PROXY (chaingrind's model of one worker, the i9's LLC shared by W workers,",
        "   DRAM latency from each arm's own bandwidth; three quantities kept apart, per output byte)",
        f"   sample: {len(p['sample']['full'])} full + {len(p['sample']['tiny'])} tiny blocks,"
        f" corpus-weighted; kappa {p['kappa']:g}; intervals: paired bootstrap over the blocks",
        "",
        "   workers  predicted/lazy [95%]    drivers: instr  at-nominal  feedback | DRAM lat (cand/lazy)  sensitivity %/100cyc",
    ]
    for w, x in sorted(
        p["predicted_over_lazy"].get("candidate", {}).items(), key=lambda t: int(t[0])
    ):
        d = x["drivers"]
        L.append(
            f"   {w:>7}  {x['ratio']:.3f} [{fmt(x['lo'])}, {fmt(x['hi'])}]"
            f"          {d['instructions_ratio']:.3f}  {d['at_nominal_latency']:.3f}       {d['contention_feedback']:.3f}"
            f"    | {d['dram_latency'][0]:4.0f}/{d['dram_latency'][1]:4.0f}"
            f"             {d['latency_sensitivity'][0]:5.1f}/{d['latency_sensitivity'][1]:5.1f}"
        )
    L += [
        "   (instr: instructions a byte over lazy's; at-nominal: modelled cycles over lazy's at the same",
        "   latencies; feedback: what the arm's own DRAM bandwidth does to its latency, against lazy's)",
        "",
        "   the three quantities, candidate against lazy:",
        "     W   transfers: L2 B  L3 B  DRAM B   | chains: dep past-L2 links  dep DRAM links  max chain | overlap: MLP  misses/link",
    ]
    for w in sorted(p["metrics"].get("candidate", {}), key=int):
        for a in ("candidate", "lazy"):
            x = p["metrics"][a][w]
            L.append(
                f"   {w:>3} {a[:4]:<4}    {x['l2_bytes_per_byte']:5.1f} {x['l3_bytes_per_byte']:5.1f} {x['dram_bytes_per_byte']:5.2f}"
                f"   |        {x['dep_past_l2_links_per_byte']:.4f}         {x['dep_dram_links_per_byte']:.5f}"
                f"      {x['max_dram_chain']:>3}/{x['max_past_l2_chain']:<4} |       {x['dram_mlp']:4.2f}  {fmt(x['dram_loads_per_dep_link'], 1)}"
            )
        fl = p["out_of_calibration"].get(w)
        if fl:
            L.append("       OUT OF CALIBRATION, unreliable here: " + "; ".join(fl))
    L.append("")
    L.append("   The reference arms on the i9 (bench/moonshot/README.md, 'Calibration'):")
    for c in ref.get("fitted", []) + ref.get("held_out", []):
        L.append(
            f"     {c['label']:<27} {c['role']:<8} measured {c['measured']:.3f}  predicted {c['predicted']:.3f}"
            f" [{fmt(c['lo'])}, {fmt(c['hi'])}]"
        )
    if ref:
        rr, n = ref["held_out_pairs_ranked_right"]
        L.append(f"     held-out pairs in the i9's order: {rr} of {n}")
    return L


def report(s: dict) -> str:
    L = []
    L.append(
        "MOONSHOT SCORE (container evidence only: only CI's paired bench and an i9 workstation run"
    )
    L.append("establish production performance; see bench/moonshot/README.md)")
    L.append(f"candidate: {s['info']['candidate']} ({s['info']['candidate_type']})")
    L.append(
        f"machine: {s['info']['cpus']} CPUs; target 1.25x 0.5.6's throughput (a wall ratio of 0.800 or less)"
    )
    L.append("")
    g = s["gate"]
    L.append(f"1. CORRECTNESS: {g['verdict']} ({g['cases']} cases, {len(g['failed'])} failed)")
    for f in g["failed"][:10]:
        L.append(f"   FAILED {f['case']}: {f['failure']}")
    if g["verdict"] != "PASS":
        L.append("   The score fails outright: nothing was timed or modelled.")
        return "\n".join(L) + "\n"
    L.append("")
    t = s.get("timing")
    if t:
        L.append(
            "2. CONTAINER WALL TIME (paired rounds, candidate over 0.5.6's LazyCells; < 1 is faster)"
        )
        L.append(
            "   workers  rounds  ratio [95% interval]       verdict       speedup  peak RSS ratio"
        )
        for w, c in t.items():
            for a in [k for k in c if k not in ("rounds", "lazy_wall_s", "lazy_peak_rss_kb")]:
                r = c[a]
                name = "" if a == "candidate" else f"  ({a})"
                L.append(
                    f"   {w:>7}  {c['rounds']:>6}  {fmt(r['geomean'])} [{fmt(r['lo'])}, {fmt(r['hi'])}]"
                    f"  {r['verdict']:<12}  {fmt(r['speedup'], 2)}x   {fmt(r['peak_rss_ratio'])}{name}"
                )
        L.append(
            "   (a container's wall misleads at contention: opend was 9-21% slower on a 4-CPU VM and"
        )
        L.append("   6-8% faster on the i9 at 8-10 workers)")
        L.append("")
    p = s.get("proxy")
    if p and "skipped" not in p:
        L += proxy_report(p, s.get("reference", {}))
    elif p:
        L.append(f"3. CONTENTION PROXY: skipped: {p['skipped']}")
    L.append("")
    L.append(
        f"4. PEAK MEMORY: see the peak RSS ratio column above (the candidate's process over 0.5.6's,"
    )
    L.append(
        "   the median of the rounds; the target is no extra peak memory, a ratio of 1.0 or less)"
    )
    return "\n".join(L) + "\n"


def score(a: argparse.Namespace) -> int:
    cand = Path(a.candidate)
    if not cand.is_absolute():
        cand = (Path.cwd() / cand).resolve()
    s = Scorer(a)
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = Path(a.out or s.target / "moonshot" / f"score-{stamp}")
    out.mkdir(parents=True, exist_ok=False)
    if not a.no_build:
        s.build(cand)
    corpus = Path(a.corpus) if a.corpus else ROOT / "corpora" / "wikidata.bz2"
    if not corpus.exists():
        log(
            f"no corpus at {corpus}: run bench/corpus.sh's corpus step (README.md), or pass --corpus"
        )
        return 2
    summary = {"info": s.info(), "corpus": str(corpus), "options": vars(a)}
    summary["gate"] = s.gate(corpus, out)
    if summary["gate"]["verdict"] == "PASS":
        if not a.no_timing:
            summary["timing"] = s.timing(corpus, out)
        if not a.no_proxy:
            summary["proxy"] = s.proxy(corpus, out)
            summary["reference"] = s.cal.get("reference_predictions", {})
    (out / "summary.json").write_text(json.dumps(summary, indent=1, default=str) + "\n")
    text = report(summary)
    (out / "summary.txt").write_text(text)
    print(text)
    log(f"written to {out}")
    return 0 if summary["gate"]["verdict"] == "PASS" else 1


def rereport(a: argparse.Namespace) -> int:
    """a score's summary.txt again, from its summary.json"""
    d = Path(a.dir)
    text = report(json.loads((d / "summary.json").read_text()))
    (d / "summary.txt").write_text(text)
    print(text)
    return 0


def calibrate(a: argparse.Namespace) -> int:
    """refits kappa on the one fitted cell (opend 7/28 against lazy at 7 workers): over a grid,
    the value whose predicted ratio, after the fixed point, is nearest the measured 0.940. Then
    every reference arm (lazy, opend, open, cells) at every worker count at that kappa: their
    metrics become the range the candidate's are checked against, and every other i9 cell is
    held out. With --write, calibration.json takes the kappa and the reference arms"""
    s = Scorer(a)
    vlib = s.tool()
    if isinstance(vlib, str):
        raise SystemExit(vlib)
    cal = s.cal
    m = cal["model"]
    sample = proxy_sample(cal, a.proxy_blocks)
    corpus = Path(a.corpus) if a.corpus else ROOT / "corpora" / "wikidata.bz2"
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = Path(a.out or s.target / "moonshot" / f"calibrate-{stamp}")
    out.mkdir(parents=True, exist_ok=False)
    target = next(c for c in cal["i9"] if c["role"] == "fitted")
    fits = []
    for kappa in [float(k) for k in a.grid.split(",")]:
        d = out / f"kappa-{kappa:g}"
        if a.again:
            runs = load_runs(Path(a.again) / d.name / "proxy-runs.jsonl")[0]
        else:
            d.mkdir()
            runs = s.proxy_runs(vlib, ["lazy", target["arm"]], [7], sample, corpus, d, kappa=kappa)
        cal_k = json.loads(json.dumps(cal))
        cal_k["model"]["kappa"] = kappa
        r = read_proxy(runs, sample, cal_k, reps=0)
        pr = r["predicted_over_lazy"][target["arm"]]["7"]["ratio"]
        log(
            f"kappa {kappa:g}: {target['arm']}/lazy at 7 workers {pr:.4f} (measured {target['measured']})"
        )
        fits.append({"kappa": kappa, "predicted": pr})
    best = min(fits, key=lambda f: abs(f["predicted"] - target["measured"]))
    m["kappa"] = best["kappa"]
    log(f"kappa {best['kappa']:g} fitted; the reference arms at every worker count")
    if a.again:
        runs, lats = load_runs(Path(a.again) / "proxy-runs.jsonl")
        if any(lt["l3_lat"] != nominal_latencies(m, int(w))["l3_lat"] for w, lt in lats.items()):
            raise SystemExit(
                f"calibrate --again: {a.again}'s reference runs were not made at kappa {m['kappa']:g}"
            )
    else:
        runs = s.proxy_runs(
            vlib, ["lazy", "opend", "open", "cells"], m["workers"], sample, corpus, out
        )
    r = read_proxy(runs, sample, cal, reps=a.bootstrap)
    r["kappa_grid"] = fits
    (out / "calibration-reading.json").write_text(json.dumps(r, indent=1) + "\n")
    print(calibration_report(r))
    if a.write:
        cal["reference"] = {
            w: {arm: r["metrics"][arm][w] for arm in r["metrics"]} for w in r["metrics"]["lazy"]
        }
        cal["reference_predictions"] = {
            "sample": a.proxy_blocks,
            "kappa_grid": fits,
            "fitted": r["calibration"]["fitted"],
            "held_out": r["calibration"]["held_out"],
            "held_out_pairs_ranked_right": r["calibration"]["held_out_pairs_ranked_right"],
            "predicted_over_lazy": {
                arm: {w: {k: v for k, v in x.items() if k != "drivers"} for w, x in ws.items()}
                for arm, ws in r["predicted_over_lazy"].items()
            },
        }
        CALIBRATION.write_text(json.dumps(cal, indent=1) + "\n")
        log(
            f"calibration.json written: kappa {m['kappa']:g}, the reference arms at {sorted(cal['reference'], key=int)}"
        )
    log(f"written to {out}")
    return 0


def calibration_report(r: dict) -> str:
    """the held-out table: each i9 cell, measured against predicted with its interval"""
    L = [
        f"kappa {r['kappa']:g}; sample {len(r['sample']['full'])} full + {len(r['sample']['tiny'])} tiny blocks"
    ]
    L.append(
        "  cell                        threads  role      measured  predicted [95% interval]   side  covered  DRAM meas/pred"
    )
    for c in r["calibration"]["fitted"] + r["calibration"]["held_out"]:
        L.append(
            f"  {c['label']:<27} {c['threads']:>7}  {c['role']:<8}  {c['measured']:.3f}     {c['predicted']:.3f} [{fmt(c['lo'])}, {fmt(c['hi'])}]"
            f"  {'yes' if c['same_side'] else 'NO':<4}  {fmt(c['covered']):<7}  {fmt(c.get('measured_dram_ratio'))}/{c['predicted_dram_ratio']:.3f}"
        )
    rr, n = r["calibration"]["held_out_pairs_ranked_right"]
    L.append(f"  held-out pairs in the i9's order: {rr} of {n}")
    return "\n".join(L)


def sweep(a: argparse.Namespace) -> int:
    """the i9 calibration run's timing (`just bench-moonshot-calibrate`): the reference arms,
    paired and alternating, at each worker count, with each run's stall counters (--perf);
    then their ratios over lazy. Decides nothing; the scorer's calibration reads it"""
    s = Scorer(a)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    arms = a.arms.split(",")
    if arms[0] != "lazy":
        raise SystemExit("sweep: lazy (0.5.6's kernel) must be the first arm, the reference")
    summary = s.timing(Path(a.corpus), out, arms=arms, perf=a.perf)
    (out / "sweep.json").write_text(json.dumps(summary, indent=1) + "\n")
    for w, c in summary.items():
        for arm in arms[1:]:
            r = c[arm]
            print(
                f"W {w:>2}  {arm:<6} over lazy: {r['geomean']:.4f} [{fmt(r['lo'], 4)}, {fmt(r['hi'], 4)}]"
                f" {r['verdict']:<12} peak RSS {r['peak_rss_ratio']:.3f} ({c['rounds']} rounds)"
            )
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = ap.add_subparsers(dest="cmd", required=True)
    sc = sub.add_parser("score", help="the whole score")
    sc.add_argument("candidate", help="the candidate's Rust source file (candidates/template.rs)")
    sc.add_argument("--corpus", help="the Wikidata bench corpus (default corpora/wikidata.bz2)")
    sc.add_argument("--rounds", type=int, default=5)
    sc.add_argument(
        "--settle", type=float, default=2.0, help="seconds between runs: 0 or at least 2"
    )
    sc.add_argument(
        "--repeat", type=int, default=1, help="the corpus decoded this many times a run"
    )
    sc.add_argument(
        "--workers", type=lambda s: [int(x) for x in s.split(",")], help="default: 1 to the CPUs"
    )
    sc.add_argument("--with-opend", action="store_true", help="time opend too, a third arm")
    sc.add_argument(
        "--proxy-blocks", default="48+16", help="FULL+TINY blocks of the bench corpus for the proxy"
    )
    sc.add_argument("--bootstrap", type=int, default=400, help="the proxy's resamples of blocks")
    sc.add_argument("--quick", action="store_true", help="the gate's quick case set")
    sc.add_argument("--no-timing", action="store_true")
    sc.add_argument("--no-proxy", action="store_true")
    sc.add_argument("--no-build", action="store_true")
    sc.add_argument("--target-dir")
    sc.add_argument("--out")
    ca = sub.add_parser(
        "calibrate", help="refit kappa on opend 7/28, read the reference arms (--write to keep)"
    )
    ca.add_argument("--corpus")
    ca.add_argument("--proxy-blocks", default="48+16", help="FULL+TINY blocks of the bench corpus")
    ca.add_argument("--grid", default="0,4,8,12,16,24,32")
    ca.add_argument("--bootstrap", type=int, default=400)
    ca.add_argument("--write", action="store_true", help="write kappa and the reference arms")
    ca.add_argument(
        "--again", help="a calibrate run's directory: read its saved runs again, running nothing"
    )
    ca.add_argument("--target-dir")
    ca.add_argument("--out")
    sw = sub.add_parser(
        "sweep", help="the i9 calibration run's timing (the reference arms; no candidate)"
    )
    sw.add_argument("--corpus", required=True)
    sw.add_argument("--out", required=True)
    sw.add_argument("--arms", default="lazy,opend,open,cells")
    sw.add_argument("--workers", type=lambda s: [int(x) for x in s.split(",")], required=True)
    sw.add_argument("--rounds", type=int, default=10)
    sw.add_argument("--settle", type=float, default=2.0)
    sw.add_argument("--repeat", type=int, default=1)
    sw.add_argument("--perf", action="store_true", help="each run under perf stat (PERF_EVENTS)")
    sw.add_argument("--target-dir")
    rp = sub.add_parser("report", help="a score's summary.txt again, from its summary.json")
    rp.add_argument("dir")
    a = ap.parse_args(argv)
    return {"score": score, "calibrate": calibrate, "sweep": sweep, "report": rereport}[a.cmd](a)


if __name__ == "__main__":
    sys.exit(main())
