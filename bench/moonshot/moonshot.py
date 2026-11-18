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
    lbt = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * math.log(1 - x)
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
    ok = returncode == 0 and final is not None and final["verdict"] == "PASS" and cases and not failed
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
        "overlap_per_dep_link": p["dep_overlap_sum"] / p["dep_overlap_n"] if p["dep_overlap_n"] else None,
        "dram_loads_per_dep_link": p["dram_loads"] / dep if dep else None,
        # the miss mix and the work
        "past_l2_loads_per_byte": p["past_l2_loads"] / b,
        "dram_loads_per_byte": p["dram_loads"] / b,
        "instructions_per_byte": p["instructions"] / b,
        "cycles_per_byte": p["cycles"] / b,
    }


# the access-pattern statistics the out-of-calibration flag watches
RANGE_KEYS = [
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


def latencies(cal: dict, workers: int, kappa: float | None = None) -> dict:
    """the model's latencies at `workers` contending workers: DRAM latency on the i9's measured
    L3-miss latency (bench-imc-ten: 549 cycles at 7 workers, 649 at 10, linear between and
    beyond, never under the floor); L3-hit latency its floor plus `kappa` cycles a further
    worker (the one fitted parameter); the LLC share the i9's 20 MiB over the workers"""
    m = cal["model"]
    (w1, d1), (w2, d2) = m["dram_latency_points"]
    dram = d1 + (d2 - d1) * (workers - w1) / (w2 - w1)
    dram = max(m["dram_latency_floor"], dram)
    k = m["kappa"] if kappa is None else kappa
    return {
        "llc_kb": int(m["llc_kb"] / workers),
        "l3_lat": int(round(m["l3_latency_floor"] + k * (workers - 1))),
        "dram_lat": int(round(dram)),
    }


def predicted_ratio(cand_cpb: float, cand_w: int, ref_cpb: float, ref_w: int) -> float:
    """the model's wall ratio of a candidate of `cand_w` workers over a reference of `ref_w`, each
    worker at its cycles a byte: the workers saturated, the wall the bytes over their rate"""
    return (cand_cpb / cand_w) / (ref_cpb / ref_w)


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
            ["cargo", "build", "--release", "--locked", "--manifest-path", str(HERE / "Cargo.toml")],
            env=env,
            check=True,
        )

    def info(self) -> dict:
        return json.loads(subprocess.run([str(self.bin), "info"], capture_output=True, text=True, check=True).stdout)

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

    def timing(self, corpus: Path, out: Path, arms: list[str] | None = None, perf: bool = False) -> dict:
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

    def proxy(self, corpus: Path, out: Path) -> dict:
        vg = shutil.which("valgrind")
        if not vg:
            return {"skipped": "valgrind is not installed (apt-get install valgrind)"}
        vlib = self.target / "chaingrind"
        b = subprocess.run(["bash", str(HERE / "chaingrind" / "build.sh"), str(vlib)], capture_output=True, text=True)
        if b.returncode:
            return {"skipped": "chaingrind did not build: " + (b.stdout + b.stderr).strip()[-400:]}
        m = self.cal["model"]
        blocks = sample_blocks(m["corpus_blocks"], self.a.proxy_blocks)
        sym = out / "blocks.sym"
        subprocess.run(
            [str(self.bin), "extract", "--file", str(corpus), "--blocks", ",".join(map(str, blocks)), "--out", str(sym)],
            check=True,
            capture_output=True,
        )
        arms = ["lazy", "opend", "open", "cells", "candidate"]
        res: dict = {"blocks": blocks, "workers": m["workers"], "arms": {}}
        for w in m["workers"]:
            lat = latencies(self.cal, w)
            for arm in arms:
                log(f"proxy: {arm} at {w} workers ({lat})")
                p = self.chaingrind(vlib, arm, sym, lat)
                res["arms"].setdefault(arm, {})[str(w)] = {"raw": p, "metrics": proxy_metrics(p), "latencies": lat}
        (out / "proxy.json").write_text(json.dumps(res, indent=1) + "\n")
        return summarise_proxy(res, self.cal)

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
            "--symbols",
            str(sym),
        ]
        p = subprocess.run(cmd, capture_output=True, text=True, env=dict(os.environ, VALGRIND_LIB=str(vlib)))
        if p.returncode:
            raise RuntimeError(f"chaingrind failed on {arm}: {p.stderr[-800:]}")
        return parse_proxy(p.stderr)


# the stall counters a calibration run reads (Comet Lake's names), per process
PERF_EVENTS = [
    "cycles",
    "instructions",
    "cycle_activity.stalls_l3_miss",
    "cycle_activity.stalls_l2_miss",
    "l1d_pend_miss.pending",
    "l1d_pend_miss.pending_cycles",
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
                    e: median([by[a][r]["counters"].get(e, float("nan")) / by[a][r]["out_bytes"] for r in rounds])
                    for e in PERF_EVENTS
                }
                for a in arms
            }
        cell["lazy_wall_s"] = median([by["lazy"][r]["wall_s"] for r in rounds])
        cell["lazy_peak_rss_kb"] = median([by["lazy"][r]["peak_rss_kb"] for r in rounds])
        out[str(w)] = cell
    return out


def summarise_proxy(res: dict, cal: dict) -> dict:
    """the proxy's per-arm metrics and its predictions against lazy at each worker count, the
    calibration and held-out arms against the i9, and the candidate's range check"""
    arms = res["arms"]
    preds = {}
    for w in res["workers"]:
        ref = arms["lazy"][str(w)]["metrics"]["cycles_per_byte"]
        preds[str(w)] = {
            a: predicted_ratio(arms[a][str(w)]["metrics"]["cycles_per_byte"], w, ref, w) for a in arms if a != "lazy"
        }
    checks = []
    for c in cal["i9"]:
        cw, rw = c["workers"], c["reference_workers"]
        if str(cw) not in arms[c["arm"]] or str(rw) not in arms["lazy"]:
            continue
        p = predicted_ratio(
            arms[c["arm"]][str(cw)]["metrics"]["cycles_per_byte"],
            cw,
            arms["lazy"][str(rw)]["metrics"]["cycles_per_byte"],
            rw,
        )
        d = arms[c["arm"]][str(cw)]["metrics"]["dram_bytes_per_byte"] / arms["lazy"][str(rw)]["metrics"]["dram_bytes_per_byte"]
        checks.append(
            dict(c, predicted=p, predicted_dram_ratio=d, same_side=(p < 1) == (c["measured"] < 1), error=p - c["measured"])
        )
    held = [c for c in checks if c["role"] == "held-out"]
    fitted = [c for c in checks if c["role"] == "fitted"]
    # the held-out ranking: every pair of judged cells (the fitted opend included) in the
    # measured order, at the same thread count
    pairs, right = 0, 0
    for i, x in enumerate(checks):
        for y in checks[i + 1 :]:
            if x["threads"] != y["threads"] or not (x["role"] == "held-out" or y["role"] == "held-out"):
                continue
            pairs += 1
            right += (x["predicted"] < y["predicted"]) == (x["measured"] < y["measured"])
    opend_7 = next((c for c in checks if c["arm"] == "opend" and c["workers"] == 7 and c["reference_workers"] == 7), None)
    rank_opend = {
        str(w): preds[str(w)]["opend"] < 1 for w in res["workers"] if w >= 7 and "opend" in preds[str(w)]
    }
    # the out-of-calibration flag, at each worker count, against the reference arms
    flags = {}
    for w in res["workers"]:
        refs = [arms[a][str(w)]["metrics"] for a in ("lazy", "opend", "open", "cells")]
        flags[str(w)] = out_of_range(arms["candidate"][str(w)]["metrics"], refs)
    abs_dram = arms["lazy"]["7"]["metrics"]["dram_bytes_per_byte"] if "7" in arms["lazy"] else None
    return {
        "predicted_over_lazy": preds,
        "calibration": {
            "kappa": cal["model"]["kappa"],
            "fitted": fitted,
            "held_out": held,
            "held_out_pairs_ranked_right": [right, pairs],
            "held_out_all_same_side": all(c["same_side"] for c in held) if held else None,
            "opend_ranked_above_lazy": rank_opend,
            "opend_7_predicted": opend_7["predicted"] if opend_7 else None,
            "lazy_dram_bytes_per_byte_at_7": abs_dram,
            "i9_lazy_dram_bytes_per_byte_at_7": cal["i9_lazy_dram_bytes_per_byte"],
        },
        "out_of_calibration": flags,
        "metrics": {a: {w: v["metrics"] for w, v in arms[a].items()} for a in arms},
    }


# ---------------------------------------------------------------- the report


def fmt(x, nd=3) -> str:
    if x is None:
        return "-"
    if isinstance(x, float):
        return f"{x:.{nd}f}"
    return str(x)


def report(s: dict) -> str:
    L = []
    L.append("MOONSHOT SCORE (container evidence only: only CI's paired bench and an i9 workstation run")
    L.append("establish production performance; see bench/moonshot/README.md)")
    L.append(f"candidate: {s['info']['candidate']} ({s['info']['candidate_type']})")
    L.append(f"machine: {s['info']['cpus']} CPUs; target 1.25x 0.5.6's throughput (a wall ratio of 0.800 or less)")
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
        L.append("2. CONTAINER WALL TIME (paired rounds, candidate over 0.5.6's LazyCells; < 1 is faster)")
        L.append("   workers  rounds  ratio [95% interval]       verdict       speedup  peak RSS ratio")
        for w, c in t.items():
            for a in [k for k in c if k not in ("rounds", "lazy_wall_s", "lazy_peak_rss_kb")]:
                r = c[a]
                name = "" if a == "candidate" else f"  ({a})"
                L.append(
                    f"   {w:>7}  {c['rounds']:>6}  {fmt(r['geomean'])} [{fmt(r['lo'])}, {fmt(r['hi'])}]"
                    f"  {r['verdict']:<12}  {fmt(r['speedup'], 2)}x   {fmt(r['peak_rss_ratio'])}{name}"
                )
        L.append("   (a container's wall misleads at contention: opend was 9-21% slower on a 4-CPU VM and")
        L.append("   6-8% faster on the i9 at 8-10 workers)")
        L.append("")
    p = s.get("proxy")
    if p and "skipped" not in p:
        L.append("3. CONTENTION PROXY (chaingrind's model, one worker per row, the i9's 20 MiB LLC shared;")
        L.append("   three quantities kept apart, per output byte; the model's cycles drive the prediction)")
        cal = p["calibration"]
        for w in sorted(p["metrics"]["lazy"], key=int):
            L.append(f"   at {w} workers:")
            L.append(
                "     arm         pred/lazy | L2 B  L3 B  DRAM B | dep DRAM links  dep past-L2 links  max chain"
                " | MLP   overlap/link  misses/link | instr"
            )
            for a in ["lazy", "opend", "open", "cells", "candidate"]:
                m = p["metrics"][a][w]
                pr = 1.0 if a == "lazy" else p["predicted_over_lazy"][w][a]
                L.append(
                    f"     {a:<10}  {pr:8.3f}  | {m['l2_bytes_per_byte']:4.1f}  {m['l3_bytes_per_byte']:4.1f}"
                    f"  {m['dram_bytes_per_byte']:5.2f}  | {m['dep_dram_links_per_byte']:.5f}"
                    f"         {m['dep_past_l2_links_per_byte']:.4f}             {m['max_dram_chain']:>3}/{m['max_past_l2_chain']:<4}"
                    f" | {m['dram_mlp']:4.1f}  {fmt(m['overlap_per_dep_link'], 1):>12}  {fmt(m['dram_loads_per_dep_link'], 1):>11}"
                    f" | {m['instructions_per_byte']:5.1f}"
                )
            fl = p["out_of_calibration"][w]
            if fl:
                L.append("     OUT OF CALIBRATION: the candidate's access pattern is outside the reference arms'")
                L.append("     range, so its prediction here is unreliable: " + "; ".join(fl))
        L.append("")
        L.append("   Calibration against the i9 (evidence fa9707e9, open-return/run-20261118T210422Z):")
        L.append(f"   one fitted parameter, the L3-hit latency's rise a further worker: kappa {cal['kappa']}")
        L.append("     arm (workers / reference)  threads  role      measured  predicted  same side  DRAM ratio meas/pred")
        for c in cal["fitted"] + cal["held_out"]:
            L.append(
                f"     {c['label']:<26} {c['threads']:>7}  {c['role']:<8}  {c['measured']:.3f}     {c['predicted']:.3f}"
                f"      {'yes' if c['same_side'] else 'NO':<9}  {fmt(c.get('measured_dram_ratio'))}/{c['predicted_dram_ratio']:.3f}"
            )
        r, n = cal["held_out_pairs_ranked_right"]
        L.append(f"   held-out ranking: {r} of {n} pairs in the i9's order")
        ok = all(cal["opend_ranked_above_lazy"].values()) if cal["opend_ranked_above_lazy"] else False
        L.append(
            "   Does the proxy rank opend above LazyCells at 7-10 workers, as the i9 did? "
            + ("YES" if ok else "NO")
            + f" ({cal['opend_ranked_above_lazy']})"
        )
        L.append(
            f"   lazy's modelled DRAM at 7 workers: {fmt(cal['lazy_dram_bytes_per_byte_at_7'], 2)} B/B (one worker's"
            f" decode_block only) against the i9's whole-process {cal['i9_lazy_dram_bytes_per_byte_at_7']} B/B"
        )
    elif p:
        L.append(f"3. CONTENTION PROXY: skipped: {p['skipped']}")
    L.append("")
    L.append(f"4. PEAK MEMORY: see the peak RSS ratio column above (the candidate's process over 0.5.6's,")
    L.append("   the median of the rounds; the target is no extra peak memory, a ratio of 1.0 or less)")
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
        log(f"no corpus at {corpus}: run bench/corpus.sh's corpus step (README.md), or pass --corpus")
        return 2
    summary = {"info": s.info(), "corpus": str(corpus), "options": vars(a)}
    summary["gate"] = s.gate(corpus, out)
    if summary["gate"]["verdict"] == "PASS":
        if not a.no_timing:
            summary["timing"] = s.timing(corpus, out)
        if not a.no_proxy:
            summary["proxy"] = s.proxy(corpus, out)
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
    """refits kappa on the two fitted cells only (lazy and opend 7/28 at 7 workers): the value
    whose predicted ratio is nearest the measured 0.940, over a grid"""
    s = Scorer(a)
    cal = s.cal
    vlib = s.target / "chaingrind"
    subprocess.run(["bash", str(HERE / "chaingrind" / "build.sh"), str(vlib)], check=True, capture_output=True)
    m = cal["model"]
    blocks = sample_blocks(m["corpus_blocks"], a.proxy_blocks)
    out = s.target / "moonshot"
    out.mkdir(parents=True, exist_ok=True)
    sym = out / "calibrate.sym"
    corpus = Path(a.corpus) if a.corpus else ROOT / "corpora" / "wikidata.bz2"
    subprocess.run(
        [str(s.bin), "extract", "--file", str(corpus), "--blocks", ",".join(map(str, blocks)), "--out", str(sym)],
        check=True,
        capture_output=True,
    )
    target = next(c for c in cal["i9"] if c["role"] == "fitted")
    best = None
    for kappa in [float(k) for k in a.grid.split(",")]:
        lat = latencies(cal, 7, kappa)
        lazy = proxy_metrics(s.chaingrind(vlib, "lazy", sym, lat))["cycles_per_byte"]
        opend = proxy_metrics(s.chaingrind(vlib, "opend", sym, lat))["cycles_per_byte"]
        r = opend / lazy
        log(f"kappa {kappa}: opend/lazy at 7 workers {r:.4f} (measured {target['measured']})")
        if best is None or abs(r - target["measured"]) < abs(best[1] - target["measured"]):
            best = (kappa, r)
    print(json.dumps({"kappa": best[0], "predicted": best[1], "measured": target["measured"]}))
    return 0


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
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    sc = sub.add_parser("score", help="the whole score")
    sc.add_argument("candidate", help="the candidate's Rust source file (candidates/template.rs)")
    sc.add_argument("--corpus", help="the Wikidata bench corpus (default corpora/wikidata.bz2)")
    sc.add_argument("--rounds", type=int, default=5)
    sc.add_argument("--settle", type=float, default=2.0, help="seconds between runs: 0 or at least 2")
    sc.add_argument("--repeat", type=int, default=1, help="the corpus decoded this many times a run")
    sc.add_argument("--workers", type=lambda s: [int(x) for x in s.split(",")], help="default: 1 to the CPUs")
    sc.add_argument("--with-opend", action="store_true", help="time opend too, a third arm")
    sc.add_argument("--proxy-blocks", type=int, default=16)
    sc.add_argument("--quick", action="store_true", help="the gate's quick case set")
    sc.add_argument("--no-timing", action="store_true")
    sc.add_argument("--no-proxy", action="store_true")
    sc.add_argument("--no-build", action="store_true")
    sc.add_argument("--target-dir")
    sc.add_argument("--out")
    ca = sub.add_parser("calibrate", help="refit kappa on lazy and opend 7/28 (prints it; edit calibration.json)")
    ca.add_argument("--corpus")
    ca.add_argument("--proxy-blocks", type=int, default=16)
    ca.add_argument("--grid", default="0,4,8,12,16,20,24,32,40,48")
    ca.add_argument("--target-dir")
    sw = sub.add_parser("sweep", help="the i9 calibration run's timing (the reference arms; no candidate)")
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
