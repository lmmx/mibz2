#!/usr/bin/env python3
"""Planner, driver, and frozen C2 analyzer for the MTF-into-construction fold kill test
(`bench-mtf-fold`; registration: docs/journal/2026-11-24-algorithm-level/mtf-fold/registration.md).

    python3 bench/mtf_fold.py plan   LSCPU_E CORES.json OUT_PLAN.json [--blocks 12] [--trace-blocks 3]
    python3 bench/mtf_fold.py run    PLAN.json --bin BIN --corpus X4.bz2 --text-corpus TEXT.bz2 --out DIR
    python3 bench/mtf_fold.py report PLAN.json RUNS.jsonl [--text-runs TEXT.jsonl] --lscpu LSCPU.txt --out DIR

Decision rule (§3.3, frozen before data; C2 never relaxed):
  KILL if ANY of K1-K4 holds:
    K1: k = 1, upper 95% CI of paired F / none > 0.975
    K2: k = 8, upper 95% CI of paired FS / S   > 0.975
    K3: k = 6 or k = 8, lower 95% CI of paired F / none > 1.01
    K4: any k in {1, 6, 8, 9, 10}, median peak RSS of F / none > 1.05
  CONTINUE otherwise.
  VOID (exit 1) if any §3.5 condition holds.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import random
import re
import statistics
import subprocess
import sys
import time

COUNTS = (1, 6, 8, 9, 10)
ARMS_ALL_K = ("none", "F", "S", "FS")
ARMS_K8_EXTRA = ("SM", "SC")
TEXT_COUNTS = (1, 8)
TEXT_ARMS = ("none", "F")
DEFAULT_BLOCKS = 12
DEFAULT_TRACE_BLOCKS = 3
DEFAULT_TEXT_BLOCKS = 6
PRESPIN_MS = 100
WIKIDATA_X4_BYTES = 4 * 209_700_196
SCAN_RESIDUAL_MAX = 0.05

# Exact two-sided 95% Student-t critical values for df = 1..30 (alpha = 0.05, two-tailed).
T_CRIT_95 = {
    1: 12.7062047364,
    2: 4.3026527299,
    3: 3.1824463053,
    4: 2.7764451052,
    5: 2.5705818366,
    6: 2.4469118488,
    7: 2.3646242510,
    8: 2.3060041350,
    9: 2.2621571627,
    10: 2.2281388520,
    11: 2.2009851601,
    12: 2.1788128297,
    13: 2.1603686565,
    14: 2.1447866879,
    15: 2.1314495456,
    16: 2.1199052992,
    17: 2.1098155778,
    18: 2.1009220402,
    19: 2.0930240544,
    20: 2.0859634473,
    21: 2.0796138447,
    22: 2.0738730679,
    23: 2.0686576104,
    24: 2.0638985616,
    25: 2.0595385528,
    26: 2.0555294386,
    27: 2.0518305165,
    28: 2.0484071418,
    29: 2.0452296421,
    30: 2.0422724563,
}


def t_crit_95(df: int) -> float:
    if df < 1:
        return float("nan")
    if df in T_CRIT_95:
        return T_CRIT_95[df]
    # Cornish-Fisher expansion for large df (> 30)
    z = 1.95996398454
    return z + (z**3 + z) / (4.0 * df) + (5.0 * z**5 + 16.0 * z**3 + 3.0 * z) / (96.0 * df**2)


def med(xs) -> float:
    vals = [x for x in xs if x is not None and math.isfinite(x)]
    return statistics.median(vals) if vals else float("nan")


def paired_ratio_ci(num: list[float], den: list[float]) -> tuple[float, float, float]:
    """Paired geometric mean ratio and two-sided 95% t-interval of `num[i] / den[i]`."""
    if len(num) != len(den) or len(num) < 2:
        return float("nan"), float("nan"), float("nan")
    logs = []
    for a, b in zip(num, den):
        if (
            a is None
            or b is None
            or a <= 0
            or b <= 0
            or not (math.isfinite(a) and math.isfinite(b))
        ):
            return float("nan"), float("nan"), float("nan")
        logs.append(math.log(a / b))
    n = len(logs)
    m = statistics.fmean(logs)
    sd = statistics.stdev(logs)
    half = t_crit_95(n - 1) * sd / math.sqrt(n)
    return math.exp(m), math.exp(m - half), math.exp(m + half)


def topology(lscpu_e_path: str) -> list[list[int]]:
    """Parse `lscpu -e` into physical cores: `[[cpu, sibling], ...]` sorted by core id."""
    with open(lscpu_e_path, encoding="utf-8") as f:
        lines = f.read().splitlines()
    head = lines[0].split()
    core_col = head.index("CORE")
    cpu_col = head.index("CPU")
    cores: dict[int, list[int]] = {}
    for line in lines[1:]:
        parts = line.split()
        if (
            len(parts) > max(core_col, cpu_col)
            and parts[cpu_col].isdigit()
            and parts[core_col].isdigit()
        ):
            cores.setdefault(int(parts[core_col]), []).append(int(parts[cpu_col]))
    if not cores:
        raise ValueError(f"no cores parsed from {lscpu_e_path}")
    return [sorted(cs) for _, cs in sorted(cores.items())]


def fastest_core(
    by_core: list[list[int]], cores_json_path: str
) -> tuple[int, list[int], dict[str, float]]:
    """Choose the physical core whose first CPU ran the calibration pacing loop fastest."""
    with open(cores_json_path, encoding="utf-8") as f:
        cal = json.load(f)
    medians: dict[str, float] = {}
    for cs in by_core:
        cpu = str(cs[0])
        rec = cal.get(cpu)
        if rec and rec.get("repeats"):
            # Supports both [idx, ns] (mtf_fold) and [idx, cyc, ns] (character)
            medians[cpu] = med(float(row[-1]) for row in rec["repeats"])
    if not medians:
        return by_core[0][0], by_core[0], {}
    best_core = min(by_core, key=lambda cs: (medians.get(str(cs[0]), float("inf")), cs[0]))
    return best_core[0], best_core, medians


def cmd_plan(args) -> int:
    by_core = topology(args.lscpu_e)
    scpu, score_cpus, cal_ns = fastest_core(by_core, args.cores_json)
    others = [c for cs in by_core if cs != score_cpus for c in cs]
    if not others:
        # Single-core fallback (only for local dry tests; i9-10900K has 10 cores)
        others = [c for cs in by_core for c in cs]

    cells = []
    for k in COUNTS:
        arms = list(ARMS_ALL_K) + (list(ARMS_K8_EXTRA) if k == 8 else [])
        for arm in arms:
            cells.append(
                {
                    "section": "wikidata",
                    "k": k,
                    "arm": arm,
                    "pinned": True,
                    "scanner_cpu": scpu,
                    "mask": others,
                    "prespin_ms": args.prespin_ms,
                }
            )

    text_cells = []
    for k in TEXT_COUNTS:
        for arm in TEXT_ARMS:
            text_cells.append(
                {
                    "section": "text",
                    "k": k,
                    "arm": arm,
                    "pinned": True,
                    "scanner_cpu": scpu,
                    "mask": others,
                    "prespin_ms": args.prespin_ms,
                }
            )

    plan = {
        "design": "mtf-fold-kill-test",
        "registration": "docs/journal/2026-11-24-algorithm-level/mtf-fold/registration.md",
        "scanner_cpu": scpu,
        "scanner_core_cpus": score_cpus,
        "pinned_mask": others,
        "core_calibration_ns": cal_ns,
        "blocks": args.blocks,
        "trace_blocks": args.trace_blocks,
        "text_blocks": args.text_blocks,
        "prespin_ms": args.prespin_ms,
        "cells": cells,
        "text_cells": text_cells,
    }
    with open(args.out_plan, "w", encoding="utf-8") as f:
        json.dump(plan, f, indent=2)
        f.write("\n")
    print(
        f"mtf-fold plan: {len(cells)} wikidata cells ({args.blocks} blocks, "
        f"traces in first {args.trace_blocks}) + {len(text_cells)} text cells "
        f"({args.text_blocks} blocks); scanner CPU {scpu} (core {score_cpus}), "
        f"workers mask {len(others)} CPUs"
    )
    return 0


def run_child(
    bin_path: str,
    corpus: str,
    cell: dict,
    cell_idx: int,
    block: int,
    trace: bool,
    timeout: int,
) -> dict:
    cmd = [
        bin_path,
        "--one",
        corpus,
        "--arm",
        cell["arm"],
        "--workers",
        str(cell["k"]),
        "--prespin-ms",
        str(cell["prespin_ms"]),
    ]
    if cell.get("pinned") and cell.get("scanner_cpu") is not None:
        cmd += ["--scanner-cpu", str(cell["scanner_cpu"])]
    if trace:
        cmd.append("--trace")
    if cell.get("pinned") and cell.get("mask"):
        cmd = ["taskset", "-c", ",".join(map(str, cell["mask"]))] + cmd

    env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "LANG": "C", "LC_ALL": "C"}
    t0 = time.monotonic()
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout, env=env)
    except subprocess.TimeoutExpired:
        return {
            "kind": "failed",
            "timeout": True,
            "section": cell["section"],
            "cell": cell_idx,
            "arm": cell["arm"],
            "k": cell["k"],
            "block": block,
            "trace": 1 if trace else 0,
            "elapsed_s": round(time.monotonic() - t0, 3),
        }
    if p.returncode != 0:
        return {
            "kind": "failed",
            "returncode": p.returncode,
            "stderr": p.stderr.strip()[-400:],
            "section": cell["section"],
            "cell": cell_idx,
            "arm": cell["arm"],
            "k": cell["k"],
            "block": block,
            "trace": 1 if trace else 0,
        }
    lines = [ln for ln in p.stdout.splitlines() if ln.strip()]
    if not lines:
        return {
            "kind": "failed",
            "stderr": "empty stdout",
            "section": cell["section"],
            "cell": cell_idx,
            "arm": cell["arm"],
            "k": cell["k"],
            "block": block,
            "trace": 1 if trace else 0,
        }
    try:
        rec = json.loads(lines[-1])
    except json.JSONDecodeError as exc:
        return {
            "kind": "failed",
            "stderr": f"malformed JSON stdout: {exc}",
            "section": cell["section"],
            "cell": cell_idx,
            "arm": cell["arm"],
            "k": cell["k"],
            "block": block,
            "trace": 1 if trace else 0,
        }
    rec["section"] = cell["section"]
    rec["cell"] = cell_idx
    rec["block"] = block
    return rec


def cmd_run(args) -> int:
    with open(args.plan, encoding="utf-8") as f:
        plan = json.load(f)
    blocks = args.blocks if args.blocks is not None else plan["blocks"]
    trace_blocks = args.trace_blocks if args.trace_blocks is not None else plan["trace_blocks"]
    text_blocks = args.text_blocks if args.text_blocks is not None else plan["text_blocks"]

    runs_path = os.path.join(args.out, "runs.jsonl")
    cells = plan["cells"]
    total_wikidata = len(cells) * (blocks + min(blocks, trace_blocks))
    done = 0
    failed = 0

    with open(runs_path, "w", encoding="utf-8") as out_f:
        for b in range(blocks):
            order = list(range(len(cells)))
            random.Random(args.seed + b).shuffle(order)
            for ci in order:
                c = cells[ci]
                rec = run_child(args.bin, args.corpus, c, ci, b, False, args.timeout)
                out_f.write(json.dumps(rec) + "\n")
                out_f.flush()
                done += 1
                if rec.get("kind") != "decode":
                    failed += 1
                if b < trace_blocks:
                    trec = run_child(args.bin, args.corpus, c, ci, b, True, args.timeout)
                    out_f.write(json.dumps(trec) + "\n")
                    out_f.flush()
                    done += 1
                    if trec.get("kind") != "decode":
                        failed += 1
                if args.settle > 0:
                    time.sleep(args.settle)
            print(
                f"mtf-fold run: wikidata block {b + 1}/{blocks} done ({done}/{total_wikidata}, failed={failed})",
                flush=True,
            )

    if args.text_corpus and plan.get("text_cells") and text_blocks > 0:
        text_path = os.path.join(args.out, "text_runs.jsonl")
        tcells = plan["text_cells"]
        t_done = 0
        t_failed = 0
        with open(text_path, "w", encoding="utf-8") as out_f:
            for b in range(text_blocks):
                order = list(range(len(tcells)))
                random.Random(args.seed + 10_000 + b).shuffle(order)
                for ci in order:
                    c = tcells[ci]
                    rec = run_child(args.bin, args.text_corpus, c, ci, b, False, args.timeout)
                    out_f.write(json.dumps(rec) + "\n")
                    out_f.flush()
                    t_done += 1
                    if rec.get("kind") != "decode":
                        t_failed += 1
        print(
            f"mtf-fold run: text fallback sanity done ({t_done} decodes, failed={t_failed})",
            flush=True,
        )

    return 0


def analyse(
    plan: dict,
    recs: list[dict],
    text_recs: list[dict],
    lscpu_text: str,
    blocks: int,
    trace_blocks: int,
    out_bytes: int,
    allow_non_i9: bool = False,
    out=sys.stdout,
) -> tuple[str, dict]:
    pr = lambda *s: print(*s, file=out)  # noqa: E731
    void_reasons: list[str] = []

    models = re.findall(r"Model name:\s*(.+)", lscpu_text)
    is_i9 = any("i9-10900K" in m for m in models)
    if not is_i9 and not allow_non_i9:
        void_reasons.append(f"not the i9-10900K (lscpu: {models or 'unknown'})")

    cells = plan["cells"]
    got: dict[tuple[int, int, int], list[dict]] = {}
    for r in recs:
        if r.get("kind") != "decode":
            why = " (timeout)" if r.get("timeout") else ""
            void_reasons.append(
                f"decode failed{why}: {r.get('arm')} k={r.get('k')} block={r.get('block')} trace={r.get('trace')}"
            )
            continue
        where = f"{r.get('arm')} k={r.get('k')} b={r.get('block')} trace={r.get('trace')}"
        if out_bytes > 0 and r.get("bytes") != out_bytes:
            void_reasons.append(f"{where}: bytes={r.get('bytes')}, expected {out_bytes}")
        cp = r.get("check_pass") or {}
        if cp.get("identical") is not True:
            void_reasons.append(f"{where}: check_pass not byte-identical ({cp})")
        if r.get("majflt", 0) > 0:
            void_reasons.append(f"{where}: majflt={r.get('majflt')}")
        if r.get("scanner_cpu") is not None and r.get("scanner_pin_ok") is not True:
            void_reasons.append(
                f"{where}: scanner pin failed (scanner_pin_ok={r.get('scanner_pin_ok')})"
            )
        got.setdefault((r["cell"], r["block"], int(r.get("trace", 0))), []).append(r)

    for ci, c in enumerate(cells):
        for b in range(blocks):
            for tr in (0, 1) if b < trace_blocks else (0,):
                n = len(got.get((ci, b, tr), []))
                if n != 1:
                    void_reasons.append(
                        f"{c['arm']} k={c['k']} b={b} {'trace' if tr else 'timed'}: count={n}, expected 1"
                    )

    dec = {k: v[0] for k, v in got.items() if len(v) == 1}
    find = {(c["k"], c["arm"]): ci for ci, c in enumerate(cells)}

    def series(k: int, arm: str, tr: int = 0) -> list[dict]:
        ci = find.get((k, arm))
        if ci is None:
            return []
        limit = trace_blocks if tr else blocks
        return [dec[(ci, b, tr)] for b in range(limit) if (ci, b, tr) in dec]

    # Verify fold & oracle invariants per §3.5
    for k in COUNTS:
        f_folded_counts = set()
        for arm in ("F", "FS"):
            for r in series(k, arm, 0) + series(k, arm, 1):
                fc = r.get("fold") or {}
                if fc.get("folded", 0) <= 0:
                    void_reasons.append(
                        f"{arm} k={k} b={r['block']}: folded={fc.get('folded', 0)} (did not fold)"
                    )
                if fc.get("bailed", 0) > 0:
                    void_reasons.append(
                        f"{arm} k={k} b={r['block']}: bailed={fc.get('bailed')} > 0"
                    )
                f_folded_counts.add(fc.get("folded", 0))
        if len(f_folded_counts) > 1:
            void_reasons.append(
                f"k={k}: F and FS folded counts disagreed: {sorted(f_folded_counts)}"
            )

        for arm in ("none", "S", "SM", "SC"):
            if (k, arm) not in find:
                continue
            for r in series(k, arm, 0) + series(k, arm, 1):
                fc = r.get("fold") or {}
                if (
                    fc.get("folded", 0) != 0
                    or fc.get("bailed", 0) != 0
                    or fc.get("not_open", 0) != 0
                ):
                    void_reasons.append(
                        f"{arm} k={k} b={r['block']}: fold executed when disabled ({fc})"
                    )

        for arm in ("S", "FS", "SM", "SC"):
            if (k, arm) not in find:
                continue
            for r in series(k, arm, 0) + series(k, arm, 1):
                oc = r.get("oracles") or {}
                if (
                    oc.get("scanned", 0) != 0
                    or oc.get("replayed", 0) <= 0
                    or oc.get("scan_locks", 0) != 0
                ):
                    void_reasons.append(f"{arm} k={k} b={r['block']}: S oracle defect ({oc})")
                if arm == "SM":
                    if (
                        oc.get("mtf_executed", 0) != 0
                        or oc.get("mtf_replayed", 0) <= 0
                        or oc.get("mtf_locks", 0) != 0
                    ):
                        void_reasons.append(f"SM k={k} b={r['block']}: M oracle defect ({oc})")
                if arm == "SC":
                    if (
                        oc.get("constr_constructed", 0) != 0
                        or oc.get("constr_replayed", 0) <= 0
                        or oc.get("constr_locks", 0) != 0
                    ):
                        void_reasons.append(f"SC k={k} b={r['block']}: C oracle defect ({oc})")

    # Verify S oracle's removed scan busy residual <= 5% (§3.5)
    scan_residuals: dict[int, dict[str, float]] = {}
    for k in COUNTS:
        none_tr = [r.get("trace_summary") for r in series(k, "none", 1) if r.get("trace_summary")]
        if not none_tr:
            void_reasons.append(f"k={k}: missing trace_summary for none")
            continue
        none_scan = med(t["scanner_busy_ns"] for t in none_tr)
        scan_residuals[k] = {}
        for arm in ("S", "FS"):
            arm_tr = [r.get("trace_summary") for r in series(k, arm, 1) if r.get("trace_summary")]
            if not arm_tr:
                void_reasons.append(f"k={k}: missing trace_summary for {arm}")
                continue
            arm_scan = med(t["scanner_busy_ns"] for t in arm_tr)
            res = arm_scan / none_scan if none_scan > 0 else float("nan")
            scan_residuals[k][arm] = res
            if not math.isfinite(res) or res > SCAN_RESIDUAL_MAX:
                void_reasons.append(
                    f"k={k} {arm}: S oracle scan busy residual {res:.4f} exceeds {SCAN_RESIDUAL_MAX:.2f}"
                )

    # Compute paired wall ratios & peak RSS ratios
    wall_stats: dict[int, dict[str, tuple[float, float, float]]] = {}
    rss_ratios: dict[int, float] = {}
    medians_ms: dict[int, dict[str, float]] = {}

    for k in COUNTS:
        wall_stats[k] = {}
        medians_ms[k] = {}
        arms = list(ARMS_ALL_K) + (list(ARMS_K8_EXTRA) if k == 8 else [])
        for arm in arms:
            medians_ms[k][arm] = med(r["wall_s"] * 1e3 for r in series(k, arm, 0))
        none_w = [r["wall_s"] for r in series(k, "none", 0)]
        f_w = [r["wall_s"] for r in series(k, "F", 0)]
        s_w = [r["wall_s"] for r in series(k, "S", 0)]
        fs_w = [r["wall_s"] for r in series(k, "FS", 0)]
        wall_stats[k]["F/none"] = paired_ratio_ci(f_w, none_w)
        wall_stats[k]["FS/S"] = paired_ratio_ci(fs_w, s_w)

        none_rss = med(r["maxrss_kib"] for r in series(k, "none", 0))
        f_rss = med(r["maxrss_kib"] for r in series(k, "F", 0))
        rss_ratios[k] = (f_rss / none_rss) if (none_rss and none_rss > 0) else float("nan")

    # Capture ratios at k = 8 (§3.4)
    capture_k8 = {}
    if 8 in medians_ms:
        s8 = medians_ms[8].get("S", float("nan"))
        fs8 = medians_ms[8].get("FS", float("nan"))
        sm8 = medians_ms[8].get("SM", float("nan"))
        sc8 = medians_ms[8].get("SC", float("nan"))
        if all(math.isfinite(x) and x > 0 for x in (s8, fs8, sm8)) and s8 != sm8:
            capture_k8["vs_SM"] = math.log(s8 / fs8) / math.log(s8 / sm8)
        if all(math.isfinite(x) and x > 0 for x in (s8, fs8, sc8)) and s8 != sc8:
            capture_k8["vs_SC"] = math.log(s8 / fs8) / math.log(s8 / sc8)

    # Check judged 95% bounds defined (§3.5)
    k1_gm, k1_lo, k1_hi = wall_stats[1]["F/none"]
    k2_gm, k2_lo, k2_hi = wall_stats[8]["FS/S"]
    k3_6_gm, k3_6_lo, k3_6_hi = wall_stats[6]["F/none"]
    k3_8_gm, k3_8_lo, k3_8_hi = wall_stats[8]["F/none"]
    for label, val in [
        ("K1 F/none@1 upper", k1_hi),
        ("K2 FS/S@8 upper", k2_hi),
        ("K3 F/none@6 lower", k3_6_lo),
        ("K3 F/none@8 lower", k3_8_lo),
    ]:
        if not math.isfinite(val):
            void_reasons.append(f"judged 95% confidence bound undefined: {label}")
    for k in COUNTS:
        if not math.isfinite(rss_ratios[k]):
            void_reasons.append(f"K4 RSS ratio undefined at k={k}")

    # Evaluate frozen K1-K4 gates (§3.3)
    kills: list[str] = []
    if math.isfinite(k1_hi) and k1_hi > 0.975:
        kills.append(
            f"K1: k=1 F/none 95% upper {k1_hi:.4f} > 0.975 (GM {k1_gm:.4f}, 95% CI [{k1_lo:.4f}, {k1_hi:.4f}])"
        )
    if math.isfinite(k2_hi) and k2_hi > 0.975:
        kills.append(
            f"K2: k=8 FS/S 95% upper {k2_hi:.4f} > 0.975 (GM {k2_gm:.4f}, 95% CI [{k2_lo:.4f}, {k2_hi:.4f}])"
        )
    if math.isfinite(k3_6_lo) and k3_6_lo > 1.01:
        kills.append(
            f"K3: k=6 F/none 95% lower {k3_6_lo:.4f} > 1.01 (GM {k3_6_gm:.4f}, 95% CI [{k3_6_lo:.4f}, {k3_6_hi:.4f}])"
        )
    if math.isfinite(k3_8_lo) and k3_8_lo > 1.01:
        kills.append(
            f"K3: k=8 F/none 95% lower {k3_8_lo:.4f} > 1.01 (GM {k3_8_gm:.4f}, 95% CI [{k3_8_lo:.4f}, {k3_8_hi:.4f}])"
        )
    for k in COUNTS:
        if math.isfinite(rss_ratios[k]) and rss_ratios[k] > 1.05:
            kills.append(f"K4: k={k} median peak RSS F/none {rss_ratios[k]:.4f} > 1.05")

    if void_reasons:
        verdict = "VOID"
    elif kills:
        verdict = "KILL"
    else:
        verdict = "CONTINUE"

    # Print report
    pr("# mtf-fold kill test report")
    pr(f"Verdict: {verdict}")
    pr(
        f"Machine: {models[0] if models else 'unknown'} | scanner CPU: {plan.get('scanner_cpu')} | blocks: {blocks} (traces: {trace_blocks})"
    )
    if void_reasons:
        pr("\n## VOID reasons (§3.5)")
        for v in void_reasons[:30]:
            pr(f"- {v}")
        if len(void_reasons) > 30:
            pr(f"- ... ({len(void_reasons) - 30} more)")
    if kills:
        pr("\n## Kill criteria triggered (§3.3)")
        for km in kills:
            pr(f"- {km}")

    pr("\n## Judged gates (§3.3, frozen)")
    pr(
        f"- K1 (k=1 F/none upper <= 0.975):  GM={k1_gm:.4f}  95% CI=[{k1_lo:.4f}, {k1_hi:.4f}]  {'FAIL' if k1_hi > 0.975 else 'PASS'}"
    )
    pr(
        f"- K2 (k=8 FS/S   upper <= 0.975):  GM={k2_gm:.4f}  95% CI=[{k2_lo:.4f}, {k2_hi:.4f}]  {'FAIL' if k2_hi > 0.975 else 'PASS'}"
    )
    pr(
        f"- K3 (k=6 F/none lower <= 1.010):  GM={k3_6_gm:.4f}  95% CI=[{k3_6_lo:.4f}, {k3_6_hi:.4f}]  {'FAIL' if k3_6_lo > 1.01 else 'PASS'}"
    )
    pr(
        f"- K3 (k=8 F/none lower <= 1.010):  GM={k3_8_gm:.4f}  95% CI=[{k3_8_lo:.4f}, {k3_8_hi:.4f}]  {'FAIL' if k3_8_lo > 1.01 else 'PASS'}"
    )
    max_rss_ratio = max(
        (rss_ratios[k] for k in COUNTS if math.isfinite(rss_ratios[k])), default=float("nan")
    )
    pr(
        f"- K4 (all k RSS F/none <= 1.050):  max={max_rss_ratio:.4f}  {'FAIL' if max_rss_ratio > 1.05 else 'PASS'}"
    )

    pr("\n## Wall medians (ms) and paired ratios")
    pr(
        f"{'k':>3} {'none':>8} {'F':>8} {'F/none [95% CI]':>26} {'S':>8} {'FS':>8} {'FS/S [95% CI]':>26} {'RSS F/n':>8}"
    )
    for k in COUNTS:
        fn = wall_stats[k]["F/none"]
        fss = wall_stats[k]["FS/S"]
        pr(
            f"{k:>3} {medians_ms[k]['none']:>8.2f} {medians_ms[k]['F']:>8.2f} "
            f"{fn[0]:>6.4f} [{fn[1]:.4f}, {fn[2]:.4f}] "
            f"{medians_ms[k]['S']:>8.2f} {medians_ms[k]['FS']:>8.2f} "
            f"{fss[0]:>6.4f} [{fss[1]:.4f}, {fss[2]:.4f}] "
            f"{rss_ratios[k]:>8.4f}"
        )
    if 8 in medians_ms:
        pr(
            f"\nReference ceilings at k=8: SM={medians_ms[8].get('SM', float('nan')):.2f} ms, "
            f"SC={medians_ms[8].get('SC', float('nan')):.2f} ms | "
            f"capture vs SM={capture_k8.get('vs_SM', float('nan')):.3f}, "
            f"vs SC={capture_k8.get('vs_SC', float('nan')):.3f}"
        )
        s_store = med(
            (r.get("oracles") or {}).get("syms_store_bytes", 0) for r in series(8, "S", 0)
        )
        pr(
            f"S/FS cold syms store size: {s_store / (1024 * 1024):.2f} MiB (read by both S and FS; K2 unadjusted per C2)"
        )

    pr("\n## Traced stage breakdown (medians across first trace blocks, ms)")
    pr(
        f"{'k':>2} {'arm':<4} {'scan':>6} {'A5/w':>6} {'A6a/w':>6} {'F5+6a/w':>7} "
        f"{'A6bc7/w':>7} {'A9/w':>6} {'A9tot':>6} {'copy_out':>8} {'rdy_MiB':>7} {'wCPU':>7}"
    )
    trace_table = {}
    for k in COUNTS:
        arms = list(ARMS_ALL_K) + (list(ARMS_K8_EXTRA) if k == 8 else [])
        for arm in arms:
            ts = [r.get("trace_summary") for r in series(k, arm, 1) if r.get("trace_summary")]
            if not ts:
                continue
            row = {
                "scan_ms": med(t["scanner_busy_ns"] for t in ts) * 1e-6,
                "a5_per_w_ms": med(t["a5_ns_per_w"] for t in ts) * 1e-6,
                "a6a_per_w_ms": med(t["a6a_ns_per_w"] for t in ts) * 1e-6,
                "fused_per_w_ms": med(t["fused_a5_a6a_ns_per_w"] for t in ts) * 1e-6,
                "a6bc_a7_per_w_ms": med(t["a6bc_a7_ns_per_w"] for t in ts) * 1e-6,
                "a9_per_w_ms": med(t["a9_ns_per_w"] for t in ts) * 1e-6,
                "a9_total_ms": med(t["a9_ns_total"] for t in ts) * 1e-6,
                "copy_out_ms": med(t["copy_out_ns"] for t in ts) * 1e-6,
                "ready_mib": med(t["ready_bytes_hwm"] for t in ts) / (1024 * 1024),
                "worker_cpu_total_ms": med(t["worker_cpu_ns_total"] for t in ts) * 1e-6,
            }
            trace_table[f"{k}:{arm}"] = row
            pr(
                f"{k:>2} {arm:<4} {row['scan_ms']:>6.2f} {row['a5_per_w_ms']:>6.2f} "
                f"{row['a6a_per_w_ms']:>6.2f} {row['fused_per_w_ms']:>7.2f} "
                f"{row['a6bc_a7_per_w_ms']:>7.2f} {row['a9_per_w_ms']:>6.2f} "
                f"{row['a9_total_ms']:>6.1f} {row['copy_out_ms']:>8.2f} "
                f"{row['ready_mib']:>7.2f} {row['worker_cpu_total_ms']:>7.1f}"
            )

    # Fold counts summary on wikidata x4
    f_sample = series(8, "F", 0)
    fs_sample = series(8, "FS", 0)
    if f_sample and fs_sample:
        pr(
            f"\nFold counts (k=8 block 0): F={f_sample[0].get('fold')} | FS={fs_sample[0].get('fold')}"
        )

    # Text fallback sanity summary (§3.4)
    text_summary = {}
    if text_recs:
        pr("\n## Non-Open fallback sanity (`ab gen` text)")
        by_ka: dict[tuple[int, str], list[dict]] = {}
        for r in text_recs:
            if r.get("kind") == "decode":
                by_ka.setdefault((r["k"], r["arm"]), []).append(r)
        for k in TEXT_COUNTS:
            n_list = sorted(by_ka.get((k, "none"), []), key=lambda x: x["block"])
            f_list = sorted(by_ka.get((k, "F"), []), key=lambda x: x["block"])
            if n_list and f_list and len(n_list) == len(f_list):
                gm, lo, hi = paired_ratio_ci(
                    [r["wall_s"] for r in f_list],
                    [r["wall_s"] for r in n_list],
                )
                med_n = med(r["wall_s"] * 1e3 for r in n_list)
                med_f = med(r["wall_s"] * 1e3 for r in f_list)
                fc = f_list[0].get("fold")
                text_summary[str(k)] = {
                    "none_ms": med_n,
                    "F_ms": med_f,
                    "ratio_gm": gm,
                    "ci95": [lo, hi],
                    "fold": fc,
                }
                pr(
                    f"- text k={k}: none={med_n:.2f} ms, F={med_f:.2f} ms, "
                    f"F/none={gm:.4f} [{lo:.4f}, {hi:.4f}], fold={fc}"
                )

    summary = {
        "verdict": verdict,
        "void_reasons": void_reasons,
        "kills": kills,
        "gates": {
            "K1": {"gm": k1_gm, "lo": k1_lo, "hi": k1_hi, "limit": 0.975},
            "K2": {"gm": k2_gm, "lo": k2_lo, "hi": k2_hi, "limit": 0.975},
            "K3_k6": {"gm": k3_6_gm, "lo": k3_6_lo, "hi": k3_6_hi, "limit": 1.01},
            "K3_k8": {"gm": k3_8_gm, "lo": k3_8_lo, "hi": k3_8_hi, "limit": 1.01},
            "K4_rss_ratios": {str(k): rss_ratios[k] for k in COUNTS},
        },
        "medians_ms": {str(k): medians_ms[k] for k in COUNTS},
        "capture_k8": capture_k8,
        "scan_residuals": {str(k): scan_residuals.get(k, {}) for k in COUNTS},
        "trace_table": trace_table,
        "text_fallback": text_summary,
    }
    return verdict, summary


def cmd_report(args) -> int:
    with open(args.plan, encoding="utf-8") as f:
        plan = json.load(f)
    with open(args.runs, encoding="utf-8") as f:
        recs = [json.loads(ln) for ln in f if ln.strip()]
    text_recs = []
    if args.text_runs and os.path.exists(args.text_runs):
        with open(args.text_runs, encoding="utf-8") as f:
            text_recs = [json.loads(ln) for ln in f if ln.strip()]
    lscpu_text = ""
    if os.path.exists(args.lscpu):
        with open(args.lscpu, encoding="utf-8") as f:
            lscpu_text = f.read()
    blocks = args.blocks if args.blocks is not None else plan["blocks"]
    trace_blocks = args.trace_blocks if args.trace_blocks is not None else plan["trace_blocks"]
    verdict, summary = analyse(
        plan,
        recs,
        text_recs,
        lscpu_text,
        blocks=blocks,
        trace_blocks=trace_blocks,
        out_bytes=args.out_bytes,
        allow_non_i9=args.allow_non_i9,
        out=sys.stdout,
    )
    if args.out:
        with open(os.path.join(args.out, "summary.json"), "w", encoding="utf-8") as f:
            json.dump(summary, f, indent=2)
            f.write("\n")
    return 1 if verdict == "VOID" else 0


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description="mtf-fold kill test planner, runner, and analyzer")
    sub = p.add_subparsers(dest="cmd", required=True)

    sp = sub.add_parser("plan")
    sp.add_argument("lscpu_e")
    sp.add_argument("cores_json")
    sp.add_argument("out_plan")
    sp.add_argument("--blocks", type=int, default=DEFAULT_BLOCKS)
    sp.add_argument("--trace-blocks", type=int, default=DEFAULT_TRACE_BLOCKS)
    sp.add_argument("--text-blocks", type=int, default=DEFAULT_TEXT_BLOCKS)
    sp.add_argument("--prespin-ms", type=int, default=PRESPIN_MS)
    sp.set_defaults(fn=cmd_plan)

    sr = sub.add_parser("run")
    sr.add_argument("plan")
    sr.add_argument("--bin", required=True)
    sr.add_argument("--corpus", required=True)
    sr.add_argument("--text-corpus", default="")
    sr.add_argument("--out", required=True)
    sr.add_argument("--blocks", type=int, default=None)
    sr.add_argument("--trace-blocks", type=int, default=None)
    sr.add_argument("--text-blocks", type=int, default=None)
    sr.add_argument("--seed", type=int, default=20261127)
    sr.add_argument("--settle", type=float, default=0.0)
    sr.add_argument("--timeout", type=int, default=180)
    sr.set_defaults(fn=cmd_run)

    sa = sub.add_parser("report")
    sa.add_argument("plan")
    sa.add_argument("runs")
    sa.add_argument("--text-runs", default="")
    sa.add_argument("--lscpu", required=True)
    sa.add_argument("--out", default="")
    sa.add_argument("--blocks", type=int, default=None)
    sa.add_argument("--trace-blocks", type=int, default=None)
    sa.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_BYTES)
    sa.add_argument("--allow-non-i9", action="store_true")
    sa.set_defaults(fn=cmd_report)

    args = p.parse_args(argv)
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
