#!/usr/bin/env python3
"""Pre-release physical i9 regression gate and workstation preflight runner (RELEASING §3.5).

Subcommands:
    python3 bench/release_preflight.py verify [--expected-head SHA] [--base-tag TAG] [--no-fetch] [--json]
    python3 bench/release_preflight.py judge BUNDLE_OR_DIR [--candidate SHA] [--base SHA] [--sha256 HEX] [--json]
    python3 bench/release_preflight.py status [--base-tag TAG] [--no-fetch] [--json]
    python3 bench/release_preflight.py run --out DIR [--candidate SHA] [--base SHA_OR_TAG]
                                           [--recovery-base SHA_OR_TAG] [--recovery-cells SPEC]
                                           [--rounds 10] [--threads GRID]

Fail-closed by design:
- Never trusts an arbitrary user-supplied PASS string, prose verdict, or environment switch.
- Always re-verifies archived evidence bundles from origin/evidence (SHA-256 sidecar + tarball digest,
  physical i9-10900K topology, exact full 40-hex candidate/base SHAs, release-equivalent build config,
  byte parity, complete paired rounds, full scaling thread grid including 8 and 10 threads, both
  primary corpora wikidata and text, PERFORMANCE-POLICY.md §2 secondary/memory bounds, §5c worker-cost
  recalibration ordering, and recovery against the last known good tag when correcting a regression).
- Allows only verified non-performance metadata-only exceptions or explicit owner-authorised waivers
  committed to docs/journal/decisions.md (RULES D / E2) bound to exact candidate and base SHAs.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import statistics
import subprocess
import sys
import tarfile
import tempfile
import tomllib

REPO_ROOT = Path(__file__).resolve().parents[1]

PRIMARY_CORPORA = ("wikidata", "text")
SECONDARY_CORPORA = ("records", "random", "pattern", "onebyte", "runs")
DEFAULT_THREADS = (1, 2, 4, 8, 10, 12, 16, 20)
MIN_ROUNDS = 10

REQUIRED_CPU_SUBSTR = "i9-10900K"
REQUIRED_CORES = 10
REQUIRED_CPUS = 20

SECONDARY_FREE_MAX = 1.05
SECONDARY_TRADEOFF_MAX = 1.10
RSS_FREE_MAX = 1.10
RSS_TRADEOFF_MIN_WALL_GAIN = 0.05
RSS_TRADEOFF_MAX_RATIO_PER_GAIN = 2.0
RSS_TRADEOFF_MAX_MB = 64.0

SHA40_RE = re.compile(r"^[0-9a-f]{40}$")
SHA64_RE = re.compile(r"^[0-9a-f]{64}$")
SEMVER_TAG_RE = re.compile(r"^v([0-9]+)\.([0-9]+)\.([0-9]+)$")

# Strict allowlist of demonstrably non-performance-affecting paths (RELEASING §3.5): paths that
# are not inputs to the published crate's library or the Python wheels' build. The harness
# (bench/, examples/, benches/; the gate's own code included, which runs from HEAD anyway),
# tests, research scratch and site config are measured with, never compiled into, the shipped
# decoder; without them here every queue entry (the
# preflight's own included) would stale the receipt it serves. Everything outside this
# allowlist — src/, python/, Cargo.toml, Cargo.lock, pyproject.toml, uv.lock, build.rs,
# rust-toolchain*, .cargo/, .github/ (CI.yml builds the wheels), .just/ and Justfile (Deploy
# Release runs `just release`, which edits Cargo.toml and Cargo.lock), and unknown paths — is
# treated as performance-relevant (fail closed).
NON_PERFORMANCE_PREFIXES = (
    "docs/",
    "frontier/",
    "bench/",
    "benches/",
    "examples/",
    "tests/",
    "research/",
    ".claude/",
)
NON_PERFORMANCE_EXACT = frozenset(
    {
        "README.md",
        "CHANGELOG.md",
        "LICENSE",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "CONTRIBUTING.md",
        "CLAUDE.md",
        "AGENTS.md",
        "GEMINI.md",
        "CHATGPT.md",
        "zensical.toml",
        "taplo.toml",
        ".gitignore",
        ".editorconfig",
        "SECURITY.md",
        "CITATION.cff",
    }
)

# Paths that can change what a worker costs (RELEASING §3.4 / §5c).
WORKER_COST_PREFIXES = (
    "src/open/",
    "src/runs/",
    "src/cells_",
)
WORKER_COST_EXACT = frozenset(
    {
        "src/open.rs",
        "src/cells.rs",
        "src/coop.rs",
        "src/parallel.rs",
        "src/bwt.rs",
        "src/mtf.rs",
        "src/rle.rs",
        "src/block.rs",
        "src/huff.rs",
        "src/runs.rs",
        "src/crc.rs",
        "src/read.rs",
        "src/run_fed.rs",
        "src/bits.rs",
    }
)
TRIM_ONLY_PATHS = frozenset({"src/parallel.rs"})

DISALLOWED_ENV_PREFIXES = ("CARGO_", "RUST")
ALLOWED_CARGO_RUST_ENV = re.compile(
    r"^(CARGO_HOME|RUSTUP_HOME|RUST_BACKTRACE|RUST_LOG|CARGO_(HTTP|NET|TERM)_[A-Z_]+)$"
)

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


class PreflightError(RuntimeError):
    """Fail-closed preflight refusal or verification error."""


def t_crit_95(df: int) -> float:
    if df < 1:
        return float("nan")
    if df in T_CRIT_95:
        return T_CRIT_95[df]
    z = 1.95996398454
    return z + (z**3 + z) / (4.0 * df) + (5.0 * z**5 + 16.0 * z**3 + 3.0 * z) / (96.0 * df**2)


def _positive(x: object) -> bool:
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) and x > 0


def paired_ratio_ci(num: list[float], den: list[float]) -> tuple[float, float, float]:
    """Paired geometric mean ratio and two-sided 95% Student-t CI of `num[i] / den[i]`."""
    if len(num) != len(den) or len(num) < 2:
        return float("nan"), float("nan"), float("nan")
    logs: list[float] = []
    for a, b in zip(num, den):
        if not (_positive(a) and _positive(b)):
            return float("nan"), float("nan"), float("nan")
        logs.append(math.log(a / b))
    n = len(logs)
    mean = statistics.fmean(logs)
    sd = statistics.stdev(logs)
    half = t_crit_95(n - 1) * sd / math.sqrt(n)
    return math.exp(mean), math.exp(mean - half), math.exp(mean + half)


def _git(repo: Path, *args: str, binary: bool = False) -> tuple[int, str | bytes]:
    proc = subprocess.run(
        ["git", "-C", str(repo), *args],
        capture_output=True,
        text=not binary,
    )
    if binary:
        return proc.returncode, proc.stdout if proc.returncode == 0 else proc.stderr
    out = proc.stdout.strip()
    if proc.returncode != 0 and proc.stderr.strip():
        out = (out + "\n" + proc.stderr.strip()).strip()
    return proc.returncode, out


def _normalize_path(path: str) -> str:
    return str(PurePosixPath(path.strip().lstrip("/")))


def is_non_performance_path(path: str) -> bool:
    norm = _normalize_path(path)
    if norm in NON_PERFORMANCE_EXACT:
        return True
    return any(norm.startswith(prefix) for prefix in NON_PERFORMANCE_PREFIXES)


def is_worker_cost_path(path: str) -> bool:
    norm = _normalize_path(path)
    if norm in WORKER_COST_EXACT:
        return True
    return any(norm.startswith(prefix) for prefix in WORKER_COST_PREFIXES)


def classify_paths(paths: list[str]) -> dict:
    """Classify changed repository paths into performance-relevant, worker-cost, and metadata."""
    perf: list[str] = []
    worker_cost: list[str] = []
    non_perf: list[str] = []
    for raw in paths:
        if not raw.strip():
            continue
        p = _normalize_path(raw)
        if is_non_performance_path(p):
            non_perf.append(p)
        else:
            perf.append(p)
            if is_worker_cost_path(p):
                worker_cost.append(p)
    return {
        "performance_relevant": sorted(set(perf)),
        "worker_cost": sorted(set(worker_cost)),
        "non_performance": sorted(set(non_perf)),
        "requires_i9_gate": bool(perf),
        "requires_recalibration": bool(worker_cost),
    }


def _parse_lscpu(text: str) -> dict:
    fields: dict[str, str] = {}
    for line in text.splitlines():
        if ":" in line:
            k, _, v = line.partition(":")
            fields[k.strip()] = v.strip()

    def _num(key: str) -> int | None:
        v = fields.get(key, "")
        return int(v) if v.isdigit() else None

    sockets = _num("Socket(s)")
    per_socket = _num("Core(s) per socket")
    return {
        "cpu": fields.get("Model name", ""),
        "cpus": _num("CPU(s)"),
        "cores": sockets * per_socket if sockets and per_socket else None,
        "threads_per_core": _num("Thread(s) per core"),
    }


def _parse_kv_lines(text: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if "=" in stripped:
            k, _, v = stripped.partition("=")
            out[k.strip()] = v.strip()
        else:
            parts = stripped.split(None, 1)
            if len(parts) == 2:
                out[parts[0].strip()] = parts[1].strip()
    return out


def _identify_arms(
    rows: list[dict],
    *,
    candidate_arm: str = "mibz2",
    base_arm_prefix: str = "mibz2@",
    recovery_arm: str | None = None,
) -> tuple[str | None, str | None, list[str]]:
    """Identify base and optional recovery arm names from `runs.jsonl`."""
    problems: list[str] = []
    other_mibz2 = sorted(
        {
            r.get("arm")
            for r in rows
            if isinstance(r, dict)
            and isinstance(r.get("arm"), str)
            and r["arm"].startswith(base_arm_prefix)
        }
    )
    if not other_mibz2:
        return None, None, ["no base mibz2@... arm found in runs.jsonl"]
    if len(other_mibz2) == 1:
        if recovery_arm and recovery_arm != other_mibz2[0]:
            return other_mibz2[0], None, []
        return other_mibz2[0], None, []
    if len(other_mibz2) == 2:
        if recovery_arm and recovery_arm in other_mibz2:
            base = [a for a in other_mibz2 if a != recovery_arm][0]
            return base, recovery_arm, []
        rec_candidates = [a for a in other_mibz2 if "recovery" in a.lower()]
        if len(rec_candidates) == 1:
            rec = rec_candidates[0]
            base = [a for a in other_mibz2 if a != rec][0]
            return base, rec, []
        problems.append(f"ambiguous mibz2@ arms in runs.jsonl: {', '.join(other_mibz2)}")
        return None, None, problems
    problems.append(
        f"unexpected number of mibz2@ arms ({len(other_mibz2)}): {', '.join(other_mibz2)}"
    )
    return None, None, problems


def judge_runs(
    rows: list[dict],
    *,
    candidate_arm: str = "mibz2",
    base_arm_prefix: str = "mibz2@",
    recovery_arm: str | None = None,
    recovery_cells: tuple[tuple[str, int], ...] | list[tuple[str, int]] = (),
    required_threads: tuple[int, ...] = DEFAULT_THREADS,
    required_primary: tuple[str, ...] = PRIMARY_CORPORA,
    min_rounds: int = MIN_ROUNDS,
    unreadable_lines: tuple[int, ...] | list[int] = (),
) -> dict:
    """Judge paired `ab run` rows against RELEASING §3.5 and PERFORMANCE-POLICY.md §2."""
    problems: list[str] = []
    regressions: list[str] = []
    recovery_failures: list[str] = []
    secondary_failures: list[str] = []
    memory_failures: list[str] = []
    parity_ok = True
    cells: dict[str, dict] = {}
    recovery_report: dict[str, dict] = {}

    if unreadable_lines:
        problems.append(f"unreadable non-JSON line(s) in runs.jsonl: {list(unreadable_lines)[:5]}")
    if not rows:
        problems.append("runs.jsonl contains no rows")
    if any(not isinstance(r, dict) for r in rows):
        problems.append("runs.jsonl contains non-object JSON rows")

    if problems:
        return {
            "valid": False,
            "pass": False,
            "parity_ok": False,
            "problems": problems,
            "regressions": regressions,
            "recovery_failures": recovery_failures,
            "secondary_failures": secondary_failures,
            "memory_failures": memory_failures,
            "cells": cells,
            "recovery": recovery_report,
        }

    # Validate row fields and failures
    for idx, r in enumerate(rows):
        if r.get("err"):
            problems.append(
                f"row {idx}: failed run ({r.get('corpus')}@{r.get('threads')}T {r.get('arm')}): {r['err']}"
            )
            continue
        term = r.get("termination")
        if term is not None and term != "success":
            problems.append(f"row {idx}: non-success termination {term!r}")
            continue
        if not (
            isinstance(r.get("corpus"), str)
            and r["corpus"]
            and isinstance(r.get("arm"), str)
            and r["arm"]
            and isinstance(r.get("threads"), int)
            and not isinstance(r.get("threads"), bool)
            and r["threads"] > 0
            and isinstance(r.get("round"), int)
            and not isinstance(r.get("round"), bool)
            and r["round"] >= 0
            and isinstance(r.get("hash"), str)
            and r["hash"]
            and isinstance(r.get("out"), int)
            and not isinstance(r.get("out"), bool)
            and r["out"] > 0
            and all(_positive(r.get(k)) for k in ("wall", "cpu", "rss_kb"))
        ):
            problems.append(f"row {idx}: malformed run record")

    base_arm, detected_recovery_arm, arm_problems = _identify_arms(
        rows,
        candidate_arm=candidate_arm,
        base_arm_prefix=base_arm_prefix,
        recovery_arm=recovery_arm,
    )
    problems.extend(arm_problems)

    # Byte parity check across EVERY run of EVERY corpus (all arms & rounds)
    outputs_by_corpus: dict[str, set[tuple[str, int, str | None]]] = {}
    for r in rows:
        if r.get("err"):
            continue
        corpus = r.get("corpus")
        if isinstance(corpus, str):
            outputs_by_corpus.setdefault(corpus, set()).add(
                (str(r.get("hash")), int(r.get("out", 0)), r.get("sha256"))
            )
    for corpus, sigs in sorted(outputs_by_corpus.items()):
        if len(sigs) != 1:
            parity_ok = False
            problems.append(
                f"byte parity failure on corpus {corpus!r}: {len(sigs)} distinct (hash, out) outputs"
            )

    if problems or base_arm is None:
        return {
            "valid": False,
            "pass": False,
            "parity_ok": parity_ok,
            "problems": problems,
            "regressions": regressions,
            "recovery_failures": recovery_failures,
            "secondary_failures": secondary_failures,
            "memory_failures": memory_failures,
            "cells": cells,
            "recovery": recovery_report,
        }

    # Index rows by (corpus, threads, arm, round)
    by_cell: dict[tuple[str, int, str], dict[int, dict]] = {}
    corpora_seen: set[str] = set()
    threads_by_corpus: dict[str, set[int]] = {}
    for r in rows:
        arm = r["arm"]
        if arm not in (candidate_arm, base_arm, detected_recovery_arm):
            continue
        corpus = r["corpus"]
        t = r["threads"]
        rd = r["round"]
        corpora_seen.add(corpus)
        threads_by_corpus.setdefault(corpus, set()).add(t)
        slot = by_cell.setdefault((corpus, t, arm), {})
        if rd in slot:
            problems.append(f"duplicate round {rd} for ({corpus}, {t}T, {arm})")
        slot[rd] = r

    # Check required primary corpora & required thread grid
    for corpus in required_primary:
        if corpus not in corpora_seen:
            problems.append(f"missing required primary corpus {corpus!r}")
            continue
        missing_t = sorted(set(required_threads) - threads_by_corpus.get(corpus, set()))
        if missing_t:
            problems.append(
                f"primary corpus {corpus!r} missing required thread count(s): {missing_t}"
            )

    # Compute paired cell metrics for candidate vs base
    primary_cells_by_t: dict[int, list[dict]] = {}
    corpora_with_rss_over_10: set[str] = set()

    for corpus in sorted(corpora_seen):
        for t in sorted(threads_by_corpus.get(corpus, set())):
            cand_map = by_cell.get((corpus, t, candidate_arm), {})
            base_map = by_cell.get((corpus, t, base_arm), {})
            if not cand_map and not base_map:
                continue
            cand_rounds = sorted(cand_map)
            base_rounds = sorted(base_map)
            expected_rounds = list(range(len(cand_rounds)))
            if (
                cand_rounds != base_rounds
                or cand_rounds != expected_rounds
                or len(cand_rounds) < min_rounds
            ):
                problems.append(
                    f"incomplete paired rounds for ({corpus}, {t}T): "
                    f"candidate={len(cand_rounds)}, base={len(base_rounds)}, min={min_rounds}"
                )
                continue
            cand_walls = [cand_map[i]["wall"] for i in expected_rounds]
            base_walls = [base_map[i]["wall"] for i in expected_rounds]
            wall_ratio, wall_lo95, wall_hi95 = paired_ratio_ci(cand_walls, base_walls)
            cand_rss_med = statistics.median(cand_map[i]["rss_kb"] for i in expected_rounds)
            base_rss_med = statistics.median(base_map[i]["rss_kb"] for i in expected_rounds)
            rss_ratio = cand_rss_med / base_rss_med
            rss_delta_mb = (cand_rss_med - base_rss_med) / 1024.0
            cand_cpu_med = statistics.median(cand_map[i]["cpu"] for i in expected_rounds)
            base_cpu_med = statistics.median(base_map[i]["cpu"] for i in expected_rounds)
            cpu_ratio = cand_cpu_med / base_cpu_med

            cell = {
                "corpus": corpus,
                "threads": t,
                "rounds": len(expected_rounds),
                "wall_ratio": wall_ratio,
                "wall_lo95": wall_lo95,
                "wall_hi95": wall_hi95,
                "cpu_ratio": cpu_ratio,
                "rss_ratio": rss_ratio,
                "rss_delta_mb": rss_delta_mb,
                "primary": corpus in PRIMARY_CORPORA,
            }
            cells[f"{corpus}:{t}"] = cell
            if corpus in PRIMARY_CORPORA:
                primary_cells_by_t.setdefault(t, []).append(cell)
                if wall_lo95 > 1.0:
                    regressions.append(
                        f"primary wall regression on {corpus} @ {t}T: "
                        f"{wall_ratio:.4f} [{wall_lo95:.4f}, {wall_hi95:.4f}] (lo95 > 1.0)"
                    )
            if rss_ratio > RSS_FREE_MAX:
                corpora_with_rss_over_10.add(corpus)

    # Secondary wall-time trade-off checks (PERFORMANCE-POLICY.md §2 rule 3)
    for cell in cells.values():
        corpus = cell["corpus"]
        if corpus in PRIMARY_CORPORA:
            continue
        t = cell["threads"]
        ratio = cell["wall_ratio"]
        if ratio > SECONDARY_TRADEOFF_MAX:
            secondary_failures.append(
                f"secondary wall loss > 10% on {corpus} @ {t}T: {ratio:.4f} (needs owner sign-off)"
            )
        elif ratio > SECONDARY_FREE_MAX:
            loss = ratio - 1.0
            primaries = primary_cells_by_t.get(t, [])
            justified = any(
                p["wall_hi95"] < 1.0 and (1.0 - p["wall_ratio"]) >= 2.0 * loss for p in primaries
            )
            if not justified:
                secondary_failures.append(
                    f"secondary wall loss {loss * 100:.2f}% on {corpus} @ {t}T without >= 2x "
                    f"primary wall gain beyond noise at {t}T"
                )

    # Peak RSS memory policy checks (PERFORMANCE-POLICY.md §2 rule 4)
    if len(corpora_with_rss_over_10) > 1:
        memory_failures.append(
            f"peak RSS grew > 10% on multiple corpora ({', '.join(sorted(corpora_with_rss_over_10))}); "
            "PERFORMANCE-POLICY §2 rule 4 requires no other corpus > 10%"
        )
    for cell in cells.values():
        if cell["rss_ratio"] <= RSS_FREE_MAX:
            continue
        corpus = cell["corpus"]
        t = cell["threads"]
        rss_growth = cell["rss_ratio"] - 1.0
        delta_mb = cell["rss_delta_mb"]
        if corpus not in PRIMARY_CORPORA:
            memory_failures.append(
                f"peak RSS grew {rss_growth * 100:.1f}% on secondary corpus {corpus} @ {t}T (> 10%)"
            )
            continue
        wall_gain = 1.0 - cell["wall_ratio"]
        wall_beyond_noise = cell["wall_hi95"] < 1.0
        if not (
            wall_beyond_noise
            and wall_gain >= RSS_TRADEOFF_MIN_WALL_GAIN
            and rss_growth <= RSS_TRADEOFF_MAX_RATIO_PER_GAIN * wall_gain
            and delta_mb <= RSS_TRADEOFF_MAX_MB
        ):
            memory_failures.append(
                f"peak RSS grew {rss_growth * 100:.1f}% (+{delta_mb:.1f} MB) on {corpus} @ {t}T "
                f"outside PERFORMANCE-POLICY §2 rule 4 (wall ratio {cell['wall_ratio']:.4f} "
                f"[{cell['wall_lo95']:.4f}, {cell['wall_hi95']:.4f}])"
            )

    # Corrective recovery check against last known good tag (RELEASING §3.5)
    norm_recovery_cells = [(c, int(t)) for c, t in recovery_cells]
    if norm_recovery_cells:
        if detected_recovery_arm is None:
            recovery_failures.append(
                f"corrective release missing recovery arm for affected cell(s): {norm_recovery_cells}"
            )
        else:
            for corpus, t in norm_recovery_cells:
                cand_map = by_cell.get((corpus, t, candidate_arm), {})
                rec_map = by_cell.get((corpus, t, detected_recovery_arm), {})
                cand_rounds = sorted(cand_map)
                rec_rounds = sorted(rec_map)
                expected_rounds = list(range(len(cand_rounds)))
                if (
                    cand_rounds != rec_rounds
                    or cand_rounds != expected_rounds
                    or len(cand_rounds) < min_rounds
                ):
                    recovery_failures.append(
                        f"incomplete recovery rounds for ({corpus}, {t}T): "
                        f"candidate={len(cand_rounds)}, recovery={len(rec_rounds)}"
                    )
                    continue
                cand_walls = [cand_map[i]["wall"] for i in expected_rounds]
                rec_walls = [rec_map[i]["wall"] for i in expected_rounds]
                rec_ratio, rec_lo95, rec_hi95 = paired_ratio_ci(cand_walls, rec_walls)
                recovery_report[f"{corpus}:{t}"] = {
                    "corpus": corpus,
                    "threads": t,
                    "rounds": len(expected_rounds),
                    "wall_ratio": rec_ratio,
                    "wall_lo95": rec_lo95,
                    "wall_hi95": rec_hi95,
                    "recovered": rec_lo95 <= 1.0,
                }
                if rec_lo95 > 1.0:
                    recovery_failures.append(
                        f"recovery failure on {corpus} @ {t}T vs last known good: "
                        f"{rec_ratio:.4f} [{rec_lo95:.4f}, {rec_hi95:.4f}] (lo95 > 1.0)"
                    )

    valid = not problems and parity_ok
    passed = (
        valid
        and not regressions
        and not recovery_failures
        and not secondary_failures
        and not memory_failures
    )
    return {
        "valid": valid,
        "pass": passed,
        "parity_ok": parity_ok,
        "problems": problems,
        "regressions": regressions,
        "recovery_failures": recovery_failures,
        "secondary_failures": secondary_failures,
        "memory_failures": memory_failures,
        "cells": cells,
        "recovery": recovery_report,
    }


def _validate_build_inputs(inputs_text: str) -> list[str]:
    problems: list[str] = []
    kv = _parse_kv_lines(inputs_text)
    if kv.get("profile") != "release":
        problems.append(f"build inputs profile must be 'release', got {kv.get('profile')!r}")
    if kv.get("features") not in ("default", ""):
        problems.append(f"build inputs features must be 'default', got {kv.get('features')!r}")
    if kv.get("rustflags"):
        problems.append(f"disallowed RUSTFLAGS in build inputs: {kv['rustflags']!r}")
    if kv.get("disallowed_env"):
        problems.append(f"disallowed build environment overrides: {kv['disallowed_env']!r}")
    cand_tc = kv.get("candidate_toolchain", "")
    base_tc = kv.get("base_toolchain", "")
    if not cand_tc or not base_tc:
        problems.append("build inputs missing candidate_toolchain or base_toolchain")
    elif cand_tc != base_tc:
        problems.append(f"toolchain mismatch between candidate ({cand_tc}) and base ({base_tc})")
    return problems


def _validate_machine(
    lscpu_text: str, env_obj: dict | None, *, require_i9: bool = True
) -> list[str]:
    if not require_i9:
        return []
    problems: list[str] = []
    parsed = _parse_lscpu(lscpu_text)
    cpu_model = parsed.get("cpu") or (env_obj.get("cpu", "") if isinstance(env_obj, dict) else "")
    cpus = parsed.get("cpus") or (env_obj.get("cpus") if isinstance(env_obj, dict) else None)
    cores = parsed.get("cores") or (env_obj.get("cores") if isinstance(env_obj, dict) else None)
    if REQUIRED_CPU_SUBSTR not in str(cpu_model):
        problems.append(
            f"wrong machine: expected physical {REQUIRED_CPU_SUBSTR} workstation, got {cpu_model!r}"
        )
    if cpus != REQUIRED_CPUS or cores != REQUIRED_CORES:
        problems.append(
            f"wrong machine topology: expected {REQUIRED_CORES} cores / {REQUIRED_CPUS} CPUs, "
            f"got {cores} cores / {cpus} CPUs"
        )
    return problems


def _read_bundle_members(bundle_path: Path) -> tuple[dict[str, bytes], str, list[str]]:
    problems: list[str] = []
    if not bundle_path.exists():
        return {}, "", [f"bundle does not exist: {bundle_path}"]
    if bundle_path.is_dir():
        members: dict[str, bytes] = {}
        for name in ("commits.txt", "lscpu.txt", "env.json", "inputs.txt", "runs.jsonl"):
            p = bundle_path / name
            if p.is_file():
                members[name] = p.read_bytes()
        return members, "", []

    raw = bundle_path.read_bytes()
    actual_sha = hashlib.sha256(raw).hexdigest()
    members = {}
    try:
        with tarfile.open(fileobj=io.BytesIO(raw), mode="r:gz") as tf:
            for ti in tf.getmembers():
                if not ti.isfile():
                    continue
                parts = PurePosixPath(ti.name).parts
                rel = "/".join(parts[1:]) if len(parts) >= 2 else parts[0]
                if rel in ("commits.txt", "lscpu.txt", "env.json", "inputs.txt", "runs.jsonl"):
                    fh = tf.extractfile(ti)
                    if fh is not None:
                        members[rel] = fh.read()
    except (tarfile.TarError, OSError, EOFError) as exc:
        problems.append(f"unreadable or corrupt bundle tarball: {exc}")
    return members, actual_sha, problems


def judge_bundle(
    bundle_path: Path,
    *,
    expected_candidate: str | None = None,
    expected_base: str | None = None,
    expected_sha256: str | None = None,
    sidecar_sha256: str | None = None,
    expected_recovery_base: str | None = None,
    recovery_cells: tuple[tuple[str, int], ...] | list[tuple[str, int]] = (),
    require_i9: bool = True,
    min_rounds: int = MIN_ROUNDS,
) -> dict:
    """Verify and judge an archived preflight bundle tarball or directory from scratch."""
    members, actual_sha, problems = _read_bundle_members(Path(bundle_path))

    if expected_sha256:
        if not SHA64_RE.fullmatch(expected_sha256):
            problems.append(f"invalid expected SHA-256 digest format: {expected_sha256!r}")
        elif actual_sha and actual_sha != expected_sha256:
            problems.append(
                f"bundle digest (SHA-256) mismatch: actual {actual_sha} != expected {expected_sha256}"
            )
    if sidecar_sha256 is not None:
        if not SHA64_RE.fullmatch(sidecar_sha256):
            problems.append(f"invalid .sha256 sidecar digest: {sidecar_sha256!r}")
        elif actual_sha and actual_sha != sidecar_sha256:
            problems.append(
                f"bundle digest (SHA-256) mismatch vs sidecar: actual {actual_sha} != sidecar {sidecar_sha256}"
            )

    for req in ("commits.txt", "lscpu.txt", "env.json", "inputs.txt", "runs.jsonl"):
        if req not in members:
            problems.append(f"bundle missing required member {req!r}")

    if problems:
        return {
            "valid": False,
            "pass": False,
            "sha256": actual_sha,
            "problems": problems,
            "regressions": [],
            "recovery_failures": [],
            "secondary_failures": [],
            "memory_failures": [],
            "cells": {},
            "recovery": {},
        }

    # Machine validation
    try:
        env_obj = json.loads(members["env.json"].decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        env_obj = None
        problems.append(f"invalid env.json: {exc}")
    problems.extend(
        _validate_machine(
            members["lscpu.txt"].decode("utf-8", errors="replace"), env_obj, require_i9=require_i9
        )
    )

    # Build inputs validation
    problems.extend(_validate_build_inputs(members["inputs.txt"].decode("utf-8", errors="replace")))

    # Commit identities
    commits = _parse_kv_lines(members["commits.txt"].decode("utf-8", errors="replace"))
    cand_sha = commits.get("candidate") or commits.get("head") or ""
    base_sha = commits.get("base") or ""
    rec_sha = commits.get("recovery_base") or ""
    if not SHA40_RE.fullmatch(cand_sha):
        problems.append(f"bundle commits.txt has invalid candidate SHA: {cand_sha!r}")
    if not SHA40_RE.fullmatch(base_sha):
        problems.append(f"bundle commits.txt has invalid base SHA: {base_sha!r}")
    if expected_candidate and cand_sha != expected_candidate:
        problems.append(f"bundle candidate SHA {cand_sha} != expected {expected_candidate}")
    if expected_base and base_sha != expected_base:
        problems.append(f"bundle base SHA {base_sha} != expected {expected_base}")
    if expected_recovery_base:
        if rec_sha != expected_recovery_base:
            problems.append(
                f"bundle recovery_base SHA {rec_sha!r} != expected last known good {expected_recovery_base}"
            )

    # Parse runs.jsonl
    rows: list[dict] = []
    unreadable: list[int] = []
    for lineno, line in enumerate(
        members["runs.jsonl"].decode("utf-8", errors="replace").splitlines(), 1
    ):
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            unreadable.append(lineno)

    run_verdict = judge_runs(
        rows,
        recovery_cells=recovery_cells,
        min_rounds=min_rounds,
        unreadable_lines=unreadable,
    )
    all_problems = problems + run_verdict["problems"]
    valid = not all_problems and run_verdict["valid"]
    passed = valid and run_verdict["pass"]
    return {
        "valid": valid,
        "pass": passed,
        "sha256": actual_sha,
        "candidate": cand_sha,
        "base": base_sha,
        "recovery_base": rec_sha or None,
        "problems": all_problems,
        "regressions": run_verdict["regressions"],
        "recovery_failures": run_verdict["recovery_failures"],
        "secondary_failures": run_verdict["secondary_failures"],
        "memory_failures": run_verdict["memory_failures"],
        "parity_ok": run_verdict["parity_ok"],
        "cells": run_verdict["cells"],
        "recovery": run_verdict["recovery"],
    }


def _sorted_release_tags(repo: Path) -> list[tuple[tuple[int, int, int], str, str]]:
    rc, out = _git(repo, "tag", "-l", "v*")
    if rc != 0 or not isinstance(out, str):
        return []
    found: list[tuple[tuple[int, int, int], str, str]] = []
    for tag in out.splitlines():
        m = SEMVER_TAG_RE.fullmatch(tag.strip())
        if not m:
            continue
        rc2, sha = _git(repo, "rev-parse", "--verify", f"{tag}^{{commit}}")
        if rc2 == 0 and isinstance(sha, str) and SHA40_RE.fullmatch(sha):
            found.append(((int(m.group(1)), int(m.group(2)), int(m.group(3))), tag, sha))
    found.sort(key=lambda x: x[0], reverse=True)
    return found


def resolve_previous_release_tag(
    repo: Path, head_sha: str, *, base_tag: str | None = None
) -> tuple[str | None, str | None, str | None]:
    """Resolve the previous published `vX.Y.Z` tag and its 40-hex commit SHA."""
    if base_tag:
        if not SEMVER_TAG_RE.fullmatch(base_tag):
            return None, None, f"base tag {base_tag!r} is not a published vX.Y.Z semver tag"
        rc, sha = _git(repo, "rev-parse", "--verify", f"{base_tag}^{{commit}}")
        if rc != 0 or not isinstance(sha, str) or not SHA40_RE.fullmatch(sha):
            return None, None, f"cannot resolve base tag {base_tag!r}"
        return base_tag, sha, None

    tags = _sorted_release_tags(repo)
    if not tags:
        return None, None, "no published vX.Y.Z release tag found"

    rc_sh, sh_out = _git(repo, "rev-parse", "--is-shallow-repository")
    is_shallow = rc_sh == 0 and sh_out == "true"

    if not is_shallow:
        for _, tag, sha in tags:
            if sha == head_sha:
                continue
            rc_anc, _ = _git(repo, "merge-base", "--is-ancestor", sha, head_sha)
            if rc_anc == 0:
                return tag, sha, None

    for _, tag, sha in tags:
        if sha != head_sha:
            return tag, sha, None
    return None, None, "no published vX.Y.Z release tag earlier than HEAD"


def detect_known_regression(
    repo: Path, base_tag: str, base_sha: str, *, head_ref: str = "HEAD"
) -> dict | None:
    """Detect if `base_tag` has a recorded primary regression in `bench/release-scaling/`."""
    target_ver = base_tag.lstrip("v")
    rc, listing = _git(
        repo, "ls-tree", "-r", "--name-only", head_ref, "--", "bench/release-scaling"
    )
    if rc != 0 or not isinstance(listing, str) or not listing:
        return None

    meta_paths = sorted(p for p in listing.splitlines() if p.endswith("/meta.json"))
    for meta_path in reversed(meta_paths):
        rc_m, meta_raw = _git(repo, "show", f"{head_ref}:{meta_path}")
        if rc_m != 0 or not isinstance(meta_raw, str):
            continue
        try:
            meta = json.loads(meta_raw)
        except json.JSONDecodeError:
            continue
        ref_arm = meta.get("ref", "mibz2")
        arms = meta.get("arms", {})
        if arms.get(ref_arm) != target_ver:
            continue
        prev_arm = None
        prev_ver = None
        for arm_name, label in arms.items():
            if arm_name != ref_arm and arm_name.startswith("mibz2@") and isinstance(label, str):
                if re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", label):
                    prev_arm = arm_name
                    prev_ver = label
                    break
        if not prev_arm or not prev_ver:
            continue
        runs_path = str(PurePosixPath(meta_path).with_name("runs.jsonl"))
        rc_r, runs_raw = _git(repo, "show", f"{head_ref}:{runs_path}")
        if rc_r != 0 or not isinstance(runs_raw, str):
            continue
        rows: list[dict] = []
        for line in runs_raw.splitlines():
            if line.strip():
                try:
                    rows.append(json.loads(line))
                except json.JSONDecodeError:
                    continue
        by_key: dict[tuple[str, int, str], dict[int, float]] = {}
        for r in rows:
            corpus = r.get("corpus")
            t = r.get("threads")
            arm = r.get("arm")
            rd = r.get("round")
            wall = r.get("wall")
            if (
                corpus in PRIMARY_CORPORA
                and isinstance(t, int)
                and arm in (ref_arm, prev_arm)
                and isinstance(rd, int)
                and _positive(wall)
            ):
                by_key.setdefault((corpus, t, arm), {})[rd] = float(wall)

        affected: list[tuple[str, int]] = []
        keys = sorted({(c, t) for c, t, _ in by_key})
        for corpus, t in keys:
            c_map = by_key.get((corpus, t, ref_arm), {})
            p_map = by_key.get((corpus, t, prev_arm), {})
            common = sorted(set(c_map) & set(p_map))
            if len(common) < 2:
                continue
            _, lo95, _ = paired_ratio_ci([c_map[i] for i in common], [p_map[i] for i in common])
            if math.isfinite(lo95) and lo95 > 1.0:
                affected.append((corpus, t))

        if affected:
            rec_tag = f"v{prev_ver}"
            rc_t, rec_sha = _git(repo, "rev-parse", "--verify", f"{rec_tag}^{{commit}}")
            return {
                "regressed_tag": base_tag,
                "regressed_sha": base_sha,
                "recovery_tag": rec_tag,
                "recovery_base": rec_sha
                if (rc_t == 0 and isinstance(rec_sha, str) and SHA40_RE.fullmatch(rec_sha))
                else None,
                "affected_cells": affected,
                "source": meta_path,
            }
    return None


def find_owner_exception(
    decisions_text: str,
    *,
    head_sha: str,
    base_sha: str,
    repo: Path | None = None,
) -> dict | None:
    """Find an explicit owner-authorised §3.5 waiver committed in `docs/journal/decisions.md`."""
    sections = re.split(r"(?m)^(?=## )", decisions_text)
    for sec in sections:
        if not sec.startswith("## "):
            continue
        lines = sec.splitlines()
        heading = lines[0].lstrip("#").strip()

        source_lines = [
            l for l in lines if re.match(r"^\s*-?\s*\*\*Source:\*\*|^\s*Source:", l, re.I)
        ]
        if not source_lines:
            continue
        source_joined = " ".join(source_lines)
        if not re.search(r"\b(louis|repository owner|maintainer)\b", source_joined, re.I):
            continue
        if re.search(r"\bagent\b", source_joined, re.I) and not re.search(
            r"\b(louis|repository owner)\b", source_joined, re.I
        ):
            continue

        if not any(re.match(r"^\s*-?\s*\*\*Observed:\*\*|^\s*Observed:", l, re.I) for l in lines):
            continue

        # Must carry verbatim owner words in a Markdown blockquote
        has_quote = any(re.match(r"^\s*>\s*\S", l) for l in lines)
        if not has_quote:
            continue

        # Must explicitly apply to RELEASING §3.5 / pre-release-i9-gate / release-preflight
        if not re.search(
            r"(RELEASING\s*§?\s*3\.5|pre-release-i9-gate|release-preflight)", sec, re.I
        ):
            continue

        # Must explicitly bind exact 40-hex candidate and base SHAs
        m = re.search(r"release-preflight-waiver:\s*([0-9a-f]{40})\s+([0-9a-f]{40})", sec)
        if not m:
            continue
        waiver_cand, waiver_base = m.group(1), m.group(2)
        if waiver_base != base_sha:
            continue
        if waiver_cand == head_sha:
            return {"heading": heading, "candidate": waiver_cand, "base": waiver_base}
        if repo is not None:
            rc_d, diff_out = _git(repo, "diff", "--name-only", waiver_cand, head_sha)
            if rc_d == 0 and isinstance(diff_out, str):
                changed = [p for p in diff_out.splitlines() if p.strip()]
                if changed and all(is_non_performance_path(p) for p in changed):
                    return {"heading": heading, "candidate": waiver_cand, "base": waiver_base}
    return None


def _parse_receipt_fields(text: str) -> dict[str, str]:
    fields: dict[str, str] = {}
    for line in text.splitlines():
        m = re.match(r"^\s*(?:[-*]\s*)?`?(preflight_[a-z0-9_]+)`?\s*:\s*(.+?)\s*$", line)
        if m:
            key = m.group(1)
            val = m.group(2).strip().strip("`").strip()
            fields[key] = val
    return fields


def find_preflight_receipts(repo: Path, head_ref: str = "HEAD") -> list[dict]:
    """Read committed preflight receipts from `frontier/deltas/*.md` at `head_ref`."""
    rc, listing = _git(repo, "ls-tree", "-r", "--name-only", head_ref, "--", "frontier/deltas")
    if rc != 0 or not isinstance(listing, str) or not listing:
        return []
    receipts: list[dict] = []
    for rel in sorted(listing.splitlines(), reverse=True):
        if not rel.endswith(".md") or rel.endswith("/README.md"):
            continue
        rc_s, body = _git(repo, "show", f"{head_ref}:{rel}")
        if rc_s != 0 or not isinstance(body, str):
            continue
        fields = _parse_receipt_fields(body)
        if fields:
            fields["_path"] = rel
            receipts.append(fields)
    return receipts


def _live_release_needs_louis(repo: Path, head_ref: str = "HEAD") -> list[str]:
    rc, listing = _git(repo, "ls-tree", "-r", "--name-only", head_ref, "--", "frontier/claims")
    if rc != 0 or not isinstance(listing, str) or not listing:
        return []
    blocked: list[str] = []
    for rel in sorted(listing.splitlines()):
        name = PurePosixPath(rel).name
        if not (name.startswith("release-") and name.endswith(".toml")):
            continue
        rc_s, body = _git(repo, "show", f"{head_ref}:{rel}")
        if rc_s != 0 or not isinstance(body, str):
            continue
        try:
            data = tomllib.loads(body)
        except tomllib.TOMLDecodeError:
            continue
        if (
            data.get("status") not in ("done", "dropped")
            and str(data.get("needs_louis", "")).strip()
        ):
            blocked.append(f"{data.get('id', name)}: needs_louis={data['needs_louis']!r}")
    return blocked


def _parse_recovery_cells(spec: str) -> list[tuple[str, int]]:
    out: list[tuple[str, int]] = []
    for item in spec.split(","):
        item = item.strip()
        if not item:
            continue
        corpus, sep, threads_s = item.partition(":")
        if sep and threads_s.isdigit():
            out.append((corpus.strip(), int(threads_s)))
    return out


def _ensure_commit_present(repo: Path, remote: str, sha: str, *, fetch: bool) -> bool:
    rc, _ = _git(repo, "cat-file", "-e", f"{sha}^{{commit}}")
    if rc == 0:
        return True
    if not fetch:
        return False
    _git(repo, "fetch", "-q", "--depth=1", "--no-tags", remote, sha)
    rc2, _ = _git(repo, "cat-file", "-e", f"{sha}^{{commit}}")
    if rc2 == 0:
        return True
    _git(repo, "fetch", "-q", "--no-tags", remote, sha)
    rc3, _ = _git(repo, "cat-file", "-e", f"{sha}^{{commit}}")
    return rc3 == 0


def _check_recalibration_5c(
    repo: Path,
    receipt: dict,
    *,
    worker_cost_paths: list[str],
    candidate_sha: str,
    remote: str,
    fetch: bool,
) -> list[str]:
    problems: list[str] = []
    status_raw = receipt.get("preflight_recalibration", "").strip()
    status_low = status_raw.lower()
    recal_commit = receipt.get("preflight_recalibration_commit", "").strip()
    recal_ev = receipt.get("preflight_recalibration_evidence", "").strip()

    if not worker_cost_paths:
        if status_low and status_low.startswith(("pending", "required")):
            problems.append(f"§5c recalibration is marked {status_raw!r}")
        return problems

    # Worker-cost paths changed: §5c recalibration is mandatory before final candidate comparison!
    if not status_low:
        problems.append(
            f"§5c worker-count recalibration required (changed worker-cost paths: {', '.join(worker_cost_paths[:5])}) "
            "but preflight_recalibration is unrecorded"
        )
        return problems
    if any(tok in status_low for tok in ("route census", "route-census", "crossover", "waive")):
        problems.append(
            f"§5c recalibration cannot be waived by route census or prior crossover ({status_raw!r})"
        )
        return problems
    if not status_low.startswith("completed"):
        problems.append(
            f"§5c worker-count recalibration must be 'completed' before final candidate comparison, got {status_raw!r}"
        )
        return problems
    if not recal_ev:
        problems.append(
            "§5c recalibration marked completed without preflight_recalibration_evidence"
        )
    if not SHA40_RE.fullmatch(recal_commit):
        problems.append(
            f"§5c recalibration missing valid 40-hex preflight_recalibration_commit (got {recal_commit!r})"
        )
        return problems

    if not _ensure_commit_present(repo, remote, recal_commit, fetch=fetch):
        problems.append(f"§5c recalibration commit {recal_commit} is not available in git history")
        return problems

    # Verify no non-trim worker-cost files changed AFTER recalibration
    rc_d, diff_out = _git(repo, "diff", "--name-only", recal_commit, candidate_sha)
    if rc_d != 0 or not isinstance(diff_out, str):
        problems.append(
            f"cannot verify §5c recalibration freshness between {recal_commit[:8]} and {candidate_sha[:8]}"
        )
        return problems
    post_recal_changed = [p for p in diff_out.splitlines() if p.strip()]
    stale_worker_changes = [
        p
        for p in post_recal_changed
        if not is_non_performance_path(p) and _normalize_path(p) not in TRIM_ONLY_PATHS
    ]
    if stale_worker_changes:
        problems.append(
            f"§5c recalibration at {recal_commit[:8]} is stale: performance/worker-cost files changed "
            f"after recalibration ({', '.join(stale_worker_changes[:5])})"
        )
    return problems


def _extract_evidence_tarball(
    repo: Path,
    *,
    remote: str,
    ev_commit: str,
    bundle_rel: str,
    tmp_dir: Path,
    fetch: bool,
) -> tuple[Path | None, str | None, list[str]]:
    problems: list[str] = []
    if not SHA40_RE.fullmatch(ev_commit):
        return None, None, [f"invalid preflight_evidence_commit SHA: {ev_commit!r}"]
    if not bundle_rel or ".." in PurePosixPath(bundle_rel).parts or bundle_rel.startswith("/"):
        return None, None, [f"invalid preflight_bundle path: {bundle_rel!r}"]

    if fetch:
        _git(
            repo,
            "fetch",
            "-q",
            "--no-tags",
            remote,
            f"+refs/heads/evidence:refs/remotes/{remote}/evidence",
        )
    if not _ensure_commit_present(repo, remote, ev_commit, fetch=fetch):
        return None, None, [f"evidence commit {ev_commit} not found on {remote}/evidence"]

    # Confirm evidence commit is actually reachable on origin/evidence (or matches its tip)
    rc_tip, tip = _git(repo, "rev-parse", "--verify", f"refs/remotes/{remote}/evidence^{{commit}}")
    if rc_tip != 0 or not isinstance(tip, str) or not SHA40_RE.fullmatch(tip):
        return None, None, [f"missing {remote}/evidence branch"]
    rc_anc, _ = _git(repo, "merge-base", "--is-ancestor", ev_commit, tip)
    if rc_anc != 0 and ev_commit != tip:
        return None, None, [f"evidence commit {ev_commit} is not on {remote}/evidence"]

    rc_b, tar_bytes = _git(repo, "show", f"{ev_commit}:{bundle_rel}", binary=True)
    if rc_b != 0 or not isinstance(tar_bytes, bytes) or not tar_bytes:
        return None, None, [f"missing evidence bundle {bundle_rel!r} at {ev_commit[:8]}"]

    rc_s, sidecar_raw = _git(repo, "show", f"{ev_commit}:{bundle_rel}.sha256")
    if rc_s != 0 or not isinstance(sidecar_raw, str) or not sidecar_raw.strip():
        return None, None, [f"missing .sha256 sidecar for {bundle_rel!r} at {ev_commit[:8]}"]
    sidecar_sha = sidecar_raw.strip().split()[0]

    out_tar = tmp_dir / Path(bundle_rel).name
    out_tar.write_bytes(tar_bytes)
    return out_tar, sidecar_sha, problems


def verify_release(
    repo: Path = REPO_ROOT,
    *,
    remote: str = "origin",
    head_ref: str = "HEAD",
    expected_head: str | None = None,
    base_tag: str | None = None,
    fetch: bool = True,
) -> dict:
    """Fail-closed pre-release gate verification (RELEASING §3.5)."""
    repo = Path(repo).resolve()
    problems: list[str] = []

    rc_h, head_sha = _git(repo, "rev-parse", "--verify", f"{head_ref}^{{commit}}")
    if rc_h != 0 or not isinstance(head_sha, str) or not SHA40_RE.fullmatch(head_sha):
        return {
            "allowed": False,
            "mode": "rejected",
            "reason": f"cannot resolve {head_ref}",
            "problems": [f"cannot resolve {head_ref}"],
            "head": "",
            "base_tag": None,
            "base_sha": None,
        }

    # 1. Working tree must be clean (never trust uncommitted decisions, deltas, or dirty builds)
    rc_st, status_out = _git(repo, "status", "--porcelain")
    if rc_st != 0 or (isinstance(status_out, str) and status_out.strip()):
        problems.append("working tree is not clean (uncommitted changes or untracked files)")

    # 2. Fetch origin/master & v* tags shallow-safely and guard against master advancing (race)
    if fetch:
        rc_fm, fm_out = _git(
            repo,
            "fetch",
            "-q",
            "--no-tags",
            remote,
            f"+refs/heads/master:refs/remotes/{remote}/master",
        )
        if rc_fm != 0:
            problems.append(f"cannot fetch {remote}/master: {fm_out}")
        rc_ft, ft_out = _git(repo, "fetch", "-q", remote, "+refs/tags/v*:refs/tags/v*")
        if rc_ft != 0:
            problems.append(f"cannot fetch release tags from {remote}: {ft_out}")

    rc_m, master_sha = _git(
        repo, "rev-parse", "--verify", f"refs/remotes/{remote}/master^{{commit}}"
    )
    if rc_m != 0 or not isinstance(master_sha, str) or not SHA40_RE.fullmatch(master_sha):
        problems.append(f"cannot resolve {remote}/master")
    else:
        if head_sha != master_sha:
            problems.append(
                f"master race detected: HEAD ({head_sha[:8]}) != {remote}/master ({master_sha[:8]}); "
                "master advanced before/during dispatch"
            )
        if expected_head:
            if not SHA40_RE.fullmatch(expected_head):
                problems.append(f"invalid --expected-head SHA: {expected_head!r}")
            elif expected_head != head_sha or expected_head != master_sha:
                problems.append(
                    f"dispatch race: expected head {expected_head[:8]} != HEAD {head_sha[:8]} / "
                    f"{remote}/master {master_sha[:8]}"
                )

    # 3. Resolve exact previous published vX.Y.Z tag
    resolved_tag, base_sha, tag_err = resolve_previous_release_tag(
        repo, head_sha, base_tag=base_tag
    )
    if tag_err or not resolved_tag or not base_sha:
        problems.append(tag_err or "cannot resolve previous release tag")

    # 4. Block if any live release-* claim carries unresolved needs_louis
    needs_louis_blockers = _live_release_needs_louis(repo, head_ref=head_ref)
    for b in needs_louis_blockers:
        problems.append(f"live release claim has unresolved needs_louis: {b}")

    if problems or not resolved_tag or not base_sha:
        return {
            "allowed": False,
            "mode": "rejected",
            "reason": "; ".join(problems),
            "problems": problems,
            "head": head_sha,
            "base_tag": resolved_tag,
            "base_sha": base_sha,
        }

    # 5. Compute changed files between base_sha and head_sha (shallow-safe via trees)
    if not _ensure_commit_present(repo, remote, base_sha, fetch=fetch):
        msg = f"previous release commit {base_sha} ({resolved_tag}) is unavailable"
        return {
            "allowed": False,
            "mode": "rejected",
            "reason": msg,
            "problems": [msg],
            "head": head_sha,
            "base_tag": resolved_tag,
            "base_sha": base_sha,
        }
    rc_diff, diff_out = _git(repo, "diff", "--name-only", base_sha, head_sha)
    if rc_diff != 0 or not isinstance(diff_out, str):
        msg = f"cannot compute diff {resolved_tag}..HEAD: {diff_out}"
        return {
            "allowed": False,
            "mode": "rejected",
            "reason": msg,
            "problems": [msg],
            "head": head_sha,
            "base_tag": resolved_tag,
            "base_sha": base_sha,
        }
    changed = [p for p in diff_out.splitlines() if p.strip()]
    if not changed:
        msg = f"no changes between {resolved_tag} ({base_sha[:8]}) and HEAD ({head_sha[:8]})"
        return {
            "allowed": False,
            "mode": "rejected",
            "reason": msg,
            "problems": [msg],
            "head": head_sha,
            "base_tag": resolved_tag,
            "base_sha": base_sha,
        }
    classified = classify_paths(changed)

    # 6. Demonstrably non-performance metadata-only release exception (§3.5)
    if not classified["requires_i9_gate"]:
        return {
            "allowed": True,
            "mode": "non-performance-metadata",
            "reason": f"non-performance metadata-only changes since {resolved_tag} ({len(changed)} file(s))",
            "problems": [],
            "head": head_sha,
            "base_tag": resolved_tag,
            "base_sha": base_sha,
            "candidate_sha": head_sha,
            "changed_paths": classified,
        }

    # 7. Explicit owner-authorised exception committed in docs/journal/decisions.md (RULES D / E2)
    rc_dec, dec_text = _git(repo, "show", f"{head_ref}:docs/journal/decisions.md")
    if rc_dec == 0 and isinstance(dec_text, str):
        waiver = find_owner_exception(dec_text, head_sha=head_sha, base_sha=base_sha, repo=repo)
        if waiver is not None:
            return {
                "allowed": True,
                "mode": "owner-exception",
                "reason": f"explicit owner-authorised exception in docs/journal/decisions.md ({waiver['heading']})",
                "problems": [],
                "head": head_sha,
                "base_tag": resolved_tag,
                "base_sha": base_sha,
                "candidate_sha": waiver["candidate"],
                "waiver": waiver,
            }

    # 8. Discover read-and-disposed preflight receipt in committed frontier/deltas/*.md
    all_receipts = find_preflight_receipts(repo, head_ref=head_ref)
    matching_receipts: list[dict] = []
    stale_reasons: list[str] = []
    wrong_base_reasons: list[str] = []

    for r in all_receipts:
        r_base = r.get("preflight_base", "")
        r_cand = r.get("preflight_candidate", "")
        if r_base != base_sha:
            wrong_base_reasons.append(
                f"{r['_path']}: receipt base {r_base[:8]} != previous release tag {resolved_tag} ({base_sha[:8]})"
            )
            continue
        if not SHA40_RE.fullmatch(r_cand):
            stale_reasons.append(f"{r['_path']}: invalid preflight_candidate SHA {r_cand!r}")
            continue
        if r_cand == head_sha:
            matching_receipts.append(r)
            continue
        if not _ensure_commit_present(repo, remote, r_cand, fetch=fetch):
            stale_reasons.append(f"{r['_path']}: candidate commit {r_cand[:8]} unavailable")
            continue
        rc_rd, rd_out = _git(repo, "diff", "--name-only", r_cand, head_sha)
        if rc_rd != 0 or not isinstance(rd_out, str):
            stale_reasons.append(f"{r['_path']}: cannot diff {r_cand[:8]}..HEAD")
            continue
        delta_files = [p for p in rd_out.splitlines() if p.strip()]
        perf_since = [p for p in delta_files if not is_non_performance_path(p)]
        if not perf_since:
            matching_receipts.append(r)
        else:
            stale_reasons.append(
                f"{r['_path']}: stale evidence for {r_cand[:8]}; performance-relevant input(s) changed "
                f"since preflight: {', '.join(perf_since[:5])}"
            )

    if not matching_receipts:
        reasons = (
            stale_reasons
            or wrong_base_reasons
            or [
                f"missing or unread i9 preflight receipt in frontier/deltas/ for HEAD {head_sha[:8]} vs "
                f"{resolved_tag} ({base_sha[:8]})"
            ]
        )
        return {
            "allowed": False,
            "mode": "rejected",
            "reason": "; ".join(reasons),
            "problems": reasons,
            "head": head_sha,
            "base_tag": resolved_tag,
            "base_sha": base_sha,
        }

    # Evaluate the newest matching receipt (deltas sorted newest filename first)
    receipt = matching_receipts[0]
    cand_sha = receipt["preflight_candidate"]

    # 9. Required §5c worker-count recalibration check
    recal_problems = _check_recalibration_5c(
        repo,
        receipt,
        worker_cost_paths=classified["worker_cost"],
        candidate_sha=cand_sha,
        remote=remote,
        fetch=fetch,
    )

    # 10. Auto-detect known regression on base_tag + explicit recovery fields
    known_reg = detect_known_regression(repo, resolved_tag, base_sha, head_ref=head_ref)
    expected_recovery_base: str | None = None
    recovery_cells: list[tuple[str, int]] = []
    if known_reg is not None:
        expected_recovery_base = known_reg["recovery_base"]
        recovery_cells = list(known_reg["affected_cells"])
    if receipt.get("preflight_recovery_base"):
        rec_from_receipt = receipt["preflight_recovery_base"]
        if expected_recovery_base and rec_from_receipt != expected_recovery_base:
            recal_problems.append(
                f"receipt recovery_base {rec_from_receipt[:8]} != last known good {expected_recovery_base[:8]}"
            )
        expected_recovery_base = expected_recovery_base or rec_from_receipt
    if receipt.get("preflight_recovery_cells"):
        for pair in _parse_recovery_cells(receipt["preflight_recovery_cells"]):
            if pair not in recovery_cells:
                recovery_cells.append(pair)

    # 11. Fetch archived tarball & sidecar from origin/evidence and re-judge from scratch
    ev_commit = receipt.get("preflight_evidence_commit", "")
    bundle_rel = receipt.get("preflight_bundle", "")
    expected_sha = receipt.get("preflight_sha256", "")

    with tempfile.TemporaryDirectory(prefix="mibz2-preflight-verify-") as td:
        tar_path, sidecar_sha, ev_problems = _extract_evidence_tarball(
            repo,
            remote=remote,
            ev_commit=ev_commit,
            bundle_rel=bundle_rel,
            tmp_dir=Path(td),
            fetch=fetch,
        )
        if ev_problems or tar_path is None:
            all_probs = recal_problems + ev_problems
            return {
                "allowed": False,
                "mode": "rejected",
                "reason": "; ".join(all_probs),
                "problems": all_probs,
                "head": head_sha,
                "base_tag": resolved_tag,
                "base_sha": base_sha,
                "candidate_sha": cand_sha,
                "receipt": receipt["_path"],
            }

        verdict = judge_bundle(
            tar_path,
            expected_candidate=cand_sha,
            expected_base=base_sha,
            expected_sha256=expected_sha,
            sidecar_sha256=sidecar_sha,
            expected_recovery_base=expected_recovery_base,
            recovery_cells=recovery_cells,
            require_i9=True,
        )

    all_probs = (
        recal_problems
        + verdict["problems"]
        + verdict["regressions"]
        + verdict["recovery_failures"]
        + verdict["secondary_failures"]
        + verdict["memory_failures"]
    )
    if all_probs or not verdict["pass"]:
        return {
            "allowed": False,
            "mode": "rejected",
            "reason": "; ".join(all_probs) if all_probs else "preflight bundle did not pass",
            "problems": all_probs,
            "head": head_sha,
            "base_tag": resolved_tag,
            "base_sha": base_sha,
            "candidate_sha": cand_sha,
            "receipt": receipt["_path"],
            "verdict": verdict,
        }

    return {
        "allowed": True,
        "mode": "verified-i9-pass",
        "reason": f"verified i9 preflight PASS ({receipt['_path']}, sha256 {verdict['sha256'][:12]})",
        "problems": [],
        "head": head_sha,
        "base_tag": resolved_tag,
        "base_sha": base_sha,
        "candidate_sha": cand_sha,
        "receipt": receipt["_path"],
        "verdict": verdict,
    }


def materialize_sources(
    tmp: Path,
    checkout: Path,
    commits: dict[str, str],
    *,
    remote: str = "origin",
) -> dict[str, Path]:
    """Shallow-safe materialisation of exact 40-hex commit SHAs into isolated directories."""
    tmp = Path(tmp).resolve()
    checkout = Path(checkout).resolve()
    rc_u, remote_url = _git(checkout, "remote", "get-url", remote)
    if rc_u != 0 or not isinstance(remote_url, str) or not remote_url:
        raise PreflightError(f"checkout {checkout} has no git remote {remote!r}")

    dirs: dict[str, Path] = {}
    for label, sha in commits.items():
        if not SHA40_RE.fullmatch(sha):
            raise PreflightError(f"commit {label!r} must be a 40-hex SHA, got {sha!r}")
        dest = tmp / f"src-{label}"
        dest.mkdir(parents=True, exist_ok=False)
        subprocess.run(["git", "init", "-q", str(dest)], check=True)
        # Try shallow fetch from origin; fall back to local checkout objects if already present locally
        fetched = subprocess.run(
            ["git", "-C", str(dest), "fetch", "-q", "--depth=1", "--no-tags", remote_url, sha],
            capture_output=True,
            text=True,
        )
        if fetched.returncode != 0:
            fetched2 = subprocess.run(
                ["git", "-C", str(dest), "fetch", "-q", "--no-tags", str(checkout), sha],
                capture_output=True,
                text=True,
            )
            if fetched2.returncode != 0:
                raise PreflightError(
                    f"cannot materialize {label} ({sha}) from {remote_url} or {checkout}: "
                    f"{fetched.stderr.strip()} / {fetched2.stderr.strip()}"
                )
        subprocess.run(
            ["git", "-C", str(dest), "checkout", "-q", "--detach", "FETCH_HEAD"],
            check=True,
        )
        rc_v, got = _git(dest, "rev-parse", "HEAD")
        if rc_v != 0 or got != sha:
            raise PreflightError(f"materialized {label} at {got}, expected {sha}")
        dirs[label] = dest
    return dirs


def _disallowed_build_env(env: dict[str, str] | None = None) -> list[str]:
    env = os.environ if env is None else env
    bad = []
    for k in sorted(env):
        if k.startswith(DISALLOWED_ENV_PREFIXES) and not ALLOWED_CARGO_RUST_ENV.fullmatch(k):
            bad.append(k)
    return bad


def _default_thread_grid() -> list[int]:
    n = os.cpu_count() or 1
    return sorted(t for t in {1, 2, 4, 8, 10, 12, 16, n // 2, n} if 1 <= t <= n)


def _format_markdown(verdict: dict, commits: dict[str, str]) -> str:
    lines = [
        "# Pre-release i9 regression gate report",
        "",
        f"- **Candidate:** `{commits.get('candidate', '')}`",
        f"- **Base:** `{commits.get('base', '')}`",
    ]
    if commits.get("recovery_base"):
        lines.append(f"- **Recovery base:** `{commits['recovery_base']}`")
    status = "VALID PASS" if verdict["pass"] else ("VALID FAIL" if verdict["valid"] else "INVALID")
    lines.append(f"- **Verdict:** **{status}**")
    lines.append("")
    lines.append(
        "| corpus | threads | wall ratio (95% CI) | CPU ratio | RSS ratio | RSS delta MB |"
    )
    lines.append("|---|--:|--:|--:|--:|--:|")
    for key in sorted(
        verdict.get("cells", {}), key=lambda s: (s.split(":")[0], int(s.split(":")[1]))
    ):
        c = verdict["cells"][key]
        lines.append(
            f"| {c['corpus']} | {c['threads']} | "
            f"{c['wall_ratio']:.4f} [{c['wall_lo95']:.4f}, {c['wall_hi95']:.4f}] | "
            f"{c['cpu_ratio']:.4f} | {c['rss_ratio']:.4f} | {c['rss_delta_mb']:+.1f} |"
        )
    if verdict.get("recovery"):
        lines.extend(["", "## Recovery vs last known good", ""])
        lines.append("| corpus | threads | cand / last-good (95% CI) | recovered |")
        lines.append("|---|--:|--:|---|")
        for key, rc in sorted(verdict["recovery"].items()):
            lines.append(
                f"| {rc['corpus']} | {rc['threads']} | "
                f"{rc['wall_ratio']:.4f} [{rc['wall_lo95']:.4f}, {rc['wall_hi95']:.4f}] | "
                f"{'yes' if rc['recovered'] else 'NO'} |"
            )
    issues = (
        verdict.get("problems", [])
        + verdict.get("regressions", [])
        + verdict.get("recovery_failures", [])
        + verdict.get("secondary_failures", [])
        + verdict.get("memory_failures", [])
    )
    if issues:
        lines.extend(["", "## Failures / problems", ""])
        lines.extend(f"- {item}" for item in issues)
    return "\n".join(lines) + "\n"


def _cmd_run(args: argparse.Namespace) -> int:
    """Execute the workstation pre-release paired measurement into `--out DIR`."""
    out = Path(args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    repo = Path(args.repo).resolve()

    bad_env = _disallowed_build_env()
    if bad_env:
        raise PreflightError(
            f"unset disallowed Cargo/Rust environment overrides: {', '.join(bad_env)}"
        )

    rc_st, st_out = _git(repo, "status", "--porcelain")
    if rc_st != 0 or (isinstance(st_out, str) and st_out.strip()):
        raise PreflightError("checkout has uncommitted changes or untracked files")

    _git(repo, "fetch", "-q", args.remote, "+refs/tags/v*:refs/tags/v*")
    cand_ref = args.candidate or "HEAD"
    rc_c, cand_sha = _git(repo, "rev-parse", "--verify", f"{cand_ref}^{{commit}}")
    if rc_c != 0 or not isinstance(cand_sha, str) or not SHA40_RE.fullmatch(cand_sha):
        raise PreflightError(f"cannot resolve candidate {cand_ref!r}")

    if args.base:
        rc_b, base_sha = _git(repo, "rev-parse", "--verify", f"{args.base}^{{commit}}")
        if rc_b != 0 or not isinstance(base_sha, str) or not SHA40_RE.fullmatch(base_sha):
            raise PreflightError(f"cannot resolve base {args.base!r}")
        base_tag = args.base if SEMVER_TAG_RE.fullmatch(args.base) else None
    else:
        base_tag, base_sha, err = resolve_previous_release_tag(repo, cand_sha)
        if err or not base_sha:
            raise PreflightError(err or "cannot resolve previous release tag")

    recovery_base_sha: str | None = None
    recovery_cells: list[tuple[str, int]] = _parse_recovery_cells(args.recovery_cells or "")
    if args.recovery_base:
        rc_rb, rb_sha = _git(repo, "rev-parse", "--verify", f"{args.recovery_base}^{{commit}}")
        if rc_rb != 0 or not isinstance(rb_sha, str) or not SHA40_RE.fullmatch(rb_sha):
            raise PreflightError(f"cannot resolve recovery_base {args.recovery_base!r}")
        recovery_base_sha = rb_sha
    elif base_tag:
        known = detect_known_regression(repo, base_tag, base_sha, head_ref=cand_sha)
        if known and known.get("recovery_base"):
            recovery_base_sha = known["recovery_base"]
            for cell in known["affected_cells"]:
                if cell not in recovery_cells:
                    recovery_cells.append(cell)

    commits = {"candidate": cand_sha, "base": base_sha}
    if recovery_base_sha:
        commits["recovery_base"] = recovery_base_sha
    (out / "commits.txt").write_text(
        "".join(f"{k} {v}\n" for k, v in commits.items()), encoding="utf-8"
    )

    with tempfile.TemporaryDirectory(prefix="mibz2-preflight-build-") as td:
        tmp = Path(td)
        dirs = materialize_sources(tmp, repo, commits, remote=args.remote)

        def toolchain(d: Path) -> str:
            r = subprocess.check_output(["rustc", "-vV"], cwd=d, text=True).splitlines()[0].strip()
            c = subprocess.check_output(["cargo", "-V"], cwd=d, text=True).strip()
            return f"{r} | {c}"

        cand_tc = toolchain(dirs["candidate"])
        base_tc = toolchain(dirs["base"])
        if cand_tc != base_tc:
            raise PreflightError(f"toolchain mismatch: candidate={cand_tc} vs base={base_tc}")

        bin_dir = out / "bin"
        bin_dir.mkdir(exist_ok=True)
        built: dict[str, Path] = {}
        for label, d in dirs.items():
            examples = ["ab", "windows"] if label == "candidate" else ["ab"]
            cmd = ["cargo", "build", "-q", "--release", "--locked"]
            for ex in examples:
                cmd.extend(["--example", ex])
            env = {**os.environ, "CARGO_TARGET_DIR": str(d / "target")}
            subprocess.run(cmd, cwd=d, env=env, check=True)
            ab_dst = bin_dir / f"ab-{label}"
            shutil.copy2(d / "target/release/examples/ab", ab_dst)
            built[label] = ab_dst
            if label == "candidate":
                win_dst = bin_dir / "windows"
                shutil.copy2(d / "target/release/examples/windows", win_dst)
                built["windows"] = win_dst

        # Generate corpora: wikidata (verified by wikidata.json) + ab gen (text & secondary corpora)
        corpora_dir = tmp / "corpora"
        corpora_dir.mkdir()
        subprocess.run([str(built["candidate"]), "gen", str(corpora_dir), "4"], check=True)

        wiki_spec = json.loads((repo / "bench/wikidata.json").read_text(encoding="utf-8"))
        win_dir = repo / wiki_spec["windows_dir"]
        if not list(win_dir.glob("*.bin")):
            _git(
                repo,
                "fetch",
                "-q",
                args.remote,
                f"bench-data:refs/remotes/{args.remote}/bench-data",
            )
            arch = subprocess.Popen(
                [
                    "git",
                    "-C",
                    str(repo),
                    "archive",
                    f"{args.remote}/bench-data",
                    wiki_spec["windows_dir"],
                ],
                stdout=subprocess.PIPE,
            )
            subprocess.run(["tar", "-x"], cwd=repo, stdin=arch.stdout, check=False)
            arch.wait()
        subprocess.run(
            [
                sys.executable,
                str(repo / "bench/wikidata.py"),
                "fetch",
                str(repo / "bench/wikidata.json"),
                str(win_dir),
            ],
            check=True,
            stdout=subprocess.DEVNULL,
        )
        wiki_bz2 = corpora_dir / "wikidata.bz2"
        subprocess.run(
            [
                sys.executable,
                str(repo / "bench/wikidata.py"),
                "build",
                str(repo / "bench/wikidata.json"),
                str(win_dir),
                str(built["windows"]),
                str(wiki_bz2),
            ],
            check=True,
        )

        # Record inputs.txt, env.json, lscpu.txt
        inputs_lines = [
            "profile=release",
            "features=default",
            "rustflags=",
            "disallowed_env=",
            f"candidate_toolchain={cand_tc}",
            f"base_toolchain={base_tc}",
        ]
        for label, path in sorted(built.items()):
            inputs_lines.append(
                f"bin_{label}_sha256={hashlib.sha256(path.read_bytes()).hexdigest()}"
            )
        for bz2_file in sorted(corpora_dir.glob("*.bz2")):
            inputs_lines.append(
                f"corpus_{bz2_file.stem}_sha256={hashlib.sha256(bz2_file.read_bytes()).hexdigest()}"
            )
        (out / "inputs.txt").write_text("\n".join(inputs_lines) + "\n", encoding="utf-8")

        env_json = subprocess.check_output([str(built["candidate"]), "env"], text=True)
        (out / "env.json").write_text(env_json, encoding="utf-8")
        lscpu_txt = subprocess.check_output(["lscpu"], text=True)
        (out / "lscpu.txt").write_text(lscpu_txt, encoding="utf-8")

        threads = args.threads or ",".join(str(t) for t in _default_thread_grid())
        arms = ["mibz2", f"mibz2@{built['base']}"]
        if recovery_base_sha:
            arms.append(f"mibz2@{built['recovery_base']}")

        hold_args: list[str] = []
        hold_cpu = os.environ.get("HOLD_CPU", "auto")
        if hold_cpu != "off" and b"--hold-max-threads" in built["candidate"].read_bytes():
            hold_args = ["--hold-cpu", hold_cpu, "--hold-max-threads", "10"]

        primary_files = [str(corpora_dir / "wikidata.bz2"), str(corpora_dir / "text.bz2")]
        runs_jsonl = out / "runs.jsonl"
        subprocess.run(
            [
                str(built["candidate"]),
                "run",
                str(runs_jsonl),
                "--arms",
                ",".join(arms),
                "--threads",
                threads,
                "--rounds",
                str(args.rounds),
                "--spread",
                *hold_args,
                *primary_files,
            ],
            check=True,
        )
        if not args.no_secondary:
            sec_files = [
                str(corpora_dir / f"{name}.bz2")
                for name in SECONDARY_CORPORA
                if (corpora_dir / f"{name}.bz2").is_file()
            ]
            if sec_files:
                sec_threads = ",".join(str(t) for t in (1, 8, 10) if t <= (os.cpu_count() or 1))
                subprocess.run(
                    [
                        str(built["candidate"]),
                        "run",
                        str(runs_jsonl),
                        "--arms",
                        ",".join(arms[:2]),
                        "--threads",
                        sec_threads or "1",
                        "--rounds",
                        str(args.rounds),
                        "--spread",
                        *hold_args,
                        *sec_files,
                    ],
                    check=True,
                )

        summary = subprocess.check_output(
            [str(built["candidate"]), "summary", str(runs_jsonl), "--ref", "mibz2"],
            text=True,
        )
        (out / "summary.txt").write_text(summary, encoding="utf-8")

    verdict = judge_bundle(
        out,
        expected_candidate=cand_sha,
        expected_base=base_sha,
        expected_recovery_base=recovery_base_sha,
        recovery_cells=recovery_cells,
        require_i9=True,
        min_rounds=args.rounds,
    )
    (out / "preflight.json").write_text(json.dumps(verdict, indent=2) + "\n", encoding="utf-8")
    (out / "preflight.md").write_text(_format_markdown(verdict, commits), encoding="utf-8")
    if verdict["pass"]:
        outcome = "VALID PASS"
    elif verdict["valid"]:
        issues = (
            verdict["regressions"]
            + verdict["recovery_failures"]
            + verdict["secondary_failures"]
            + verdict["memory_failures"]
        )
        outcome = "VALID FAIL: " + "; ".join(issues)
    else:
        outcome = "INVALID: " + "; ".join(verdict["problems"])
    (out / "outcome.txt").write_text(outcome + "\n", encoding="utf-8")
    print(f"release-preflight: {outcome}")
    return 0 if verdict["pass"] else 1


def _cmd_status(args: argparse.Namespace) -> int:
    res = verify_release(
        Path(args.repo),
        remote=args.remote,
        head_ref=args.head,
        expected_head=args.expected_head,
        base_tag=args.base_tag,
        fetch=not args.no_fetch,
    )
    if args.json:
        print(json.dumps(res, indent=2))
    else:
        state = "ALLOW" if res["allowed"] else "BLOCK"
        print(
            f"release-preflight status: {state} ({res['mode']}) "
            f"head={res['head'][:8]} base={res['base_tag'] or '?'}({(res['base_sha'] or '')[:8]})"
        )
        print(f"  reason: {res['reason']}")
        for p in res.get("problems", []):
            print(f"  - {p}")
    return 0


def _cmd_verify(args: argparse.Namespace) -> int:
    res = verify_release(
        Path(args.repo),
        remote=args.remote,
        head_ref=args.head,
        expected_head=args.expected_head,
        base_tag=args.base_tag,
        fetch=not args.no_fetch,
    )
    if args.json:
        print(json.dumps(res, indent=2))
    elif res["allowed"]:
        print(f"release-preflight PASS ({res['mode']}): {res['reason']}")
    else:
        print(f"release-preflight REJECTED: {res['reason']}", file=sys.stderr)
        for p in res.get("problems", []):
            print(f"  - {p}", file=sys.stderr)
    return 0 if res["allowed"] else 1


def _cmd_judge(args: argparse.Namespace) -> int:
    rec_cells = _parse_recovery_cells(args.recovery_cells or "")
    verdict = judge_bundle(
        Path(args.bundle),
        expected_candidate=args.candidate,
        expected_base=args.base,
        expected_sha256=args.sha256,
        sidecar_sha256=args.sidecar_sha256,
        expected_recovery_base=args.recovery_base,
        recovery_cells=rec_cells,
        require_i9=not args.no_machine_check,
        min_rounds=args.min_rounds,
    )
    if args.json:
        print(json.dumps(verdict, indent=2))
    elif verdict["pass"]:
        print(f"release-preflight judge: PASS (sha256 {verdict['sha256'][:12]})")
    else:
        issues = (
            verdict["problems"]
            + verdict["regressions"]
            + verdict["recovery_failures"]
            + verdict["secondary_failures"]
            + verdict["memory_failures"]
        )
        print("release-preflight judge: FAIL", file=sys.stderr)
        for item in issues:
            print(f"  - {item}", file=sys.stderr)
    return 0 if verdict["pass"] else 1


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)

    p_verify = sub.add_parser(
        "verify", help="Fail-closed pre-release gate check before bump/tag/publish"
    )
    p_verify.add_argument("--repo", default=str(REPO_ROOT))
    p_verify.add_argument("--remote", default="origin")
    p_verify.add_argument("--head", default="HEAD")
    p_verify.add_argument("--expected-head", default=None)
    p_verify.add_argument("--base-tag", default=None)
    p_verify.add_argument("--no-fetch", action="store_true")
    p_verify.add_argument("--json", action="store_true")

    p_status = sub.add_parser("status", help="Read-only pre-release gate status")
    p_status.add_argument("--repo", default=str(REPO_ROOT))
    p_status.add_argument("--remote", default="origin")
    p_status.add_argument("--head", default="HEAD")
    p_status.add_argument("--expected-head", default=None)
    p_status.add_argument("--base-tag", default=None)
    p_status.add_argument("--no-fetch", action="store_true")
    p_status.add_argument("--json", action="store_true")

    p_judge = sub.add_parser(
        "judge", help="Judge an archived preflight tarball or attempt directory"
    )
    p_judge.add_argument("bundle")
    p_judge.add_argument("--candidate", default=None)
    p_judge.add_argument("--base", default=None)
    p_judge.add_argument("--sha256", default=None)
    p_judge.add_argument("--sidecar-sha256", default=None)
    p_judge.add_argument("--recovery-base", default=None)
    p_judge.add_argument("--recovery-cells", default="")
    p_judge.add_argument("--min-rounds", type=int, default=MIN_ROUNDS)
    p_judge.add_argument("--no-machine-check", action="store_true")
    p_judge.add_argument("--json", action="store_true")

    p_run = sub.add_parser("run", help="Run the workstation pre-release paired comparison")
    p_run.add_argument("--out", required=True)
    p_run.add_argument("--repo", default=str(REPO_ROOT))
    p_run.add_argument("--remote", default="origin")
    p_run.add_argument("--candidate", default="")
    p_run.add_argument("--base", default="")
    p_run.add_argument("--recovery-base", default="")
    p_run.add_argument("--recovery-cells", default="")
    p_run.add_argument("--rounds", type=int, default=12)  # 2 x arms balance for 2 and 3 arms
    p_run.add_argument("--threads", default="")
    p_run.add_argument("--no-secondary", action="store_true")

    args = ap.parse_args(argv)
    if args.cmd == "verify":
        return _cmd_verify(args)
    if args.cmd == "status":
        return _cmd_status(args)
    if args.cmd == "judge":
        return _cmd_judge(args)
    if args.cmd == "run":
        try:
            return _cmd_run(args)
        except PreflightError as exc:
            out = Path(args.out).resolve()
            out.mkdir(parents=True, exist_ok=True)
            (out / "outcome.txt").write_text(f"REFUSED: {exc}\n", encoding="utf-8")
            print(f"release-preflight REFUSED: {exc}", file=sys.stderr)
            return 2
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
