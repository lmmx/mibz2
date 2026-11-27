#!/usr/bin/env python3
"""Executable i9-10900K kill-test recipe for #1110 (`forward-anchor-lanes`).

Implements the frozen registration in
`docs/journal/2026-11-27-forward-emission/registration.md` (§2, §3, §5) with no
threshold changes (RULES C2):

    python3 bench/forward_lanes.py <OUT_DIR>
    python3 bench/forward_lanes.py plan   <OUT_DIR>
    python3 bench/forward_lanes.py run    <OUT_DIR>
    python3 bench/forward_lanes.py report <RUNS_JSONL> [--out <DIR>]

Arms:
  - `base`: merge base of #1110 (7beb5e9b), default features
  - `cand`: #1110 squash (7a7a207a), `--features forward-anchor-lanes`
  - For 8W `S ideal` (plus matched `none` trace for the <= 5% scan-residual gate),
    both commits are additionally built with `scan-oracle` + trace features.

Cells (15 interleaved AB/BA paired repeats each, output hashed on every repeat):
  - `wikidata x4` at k = 1 (forced Open route, `mibz2-workers-lazy`, §5)
  - `wikidata x4` at k = 8 (shipped scan, default Open route)
  - `wikidata x4` at k = 8 (`S ideal`, warm paired-oracle protocol)
  - `text` at k = 1 (forced Open route, non-wikidata guard)
"""

from __future__ import annotations

import argparse
import bz2
import hashlib
import itertools
import json
import math
import os
from pathlib import Path
import random
import re
import shutil
import statistics
import subprocess
import sys
import tempfile

from mtf_fold import SCAN_RESIDUAL_MAX, fastest_core, paired_ratio_ci, run_child, topology
import wikidata

BASE_SHA = "7beb5e9ba4730feb1f38d12b40100dace34b2d4e"
CANDIDATE_SHA = "7a7a207adfc2fee830b1b46f7fa6707d61271f0a"
FEATURE = "forward-anchor-lanes"
TRACE_FEATURES = "mtf-fold scan-oracle ideal-parts trace phase-counters"
ROOT = Path(__file__).resolve().parents[1]
WALL_CELLS = (("wikidata", 1), ("wikidata", 8), ("text", 1))
REPEATS = 15
BOOTSTRAP_SAMPLES = 10_000
BOOTSTRAP_SEED = 20261127
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")

# Frozen §3 thresholds (docs/journal/2026-11-27-forward-emission/registration.md).
PASS_R1_HI_MAX = 0.950
PASS_R8S_HI_MAX = 0.975
PASS_R8_HI_MAX = 1.010
PASS_TEXT_HI_MAX = 1.010
KILL_R1_LO_MIN = 0.980
KILL_R8S_LO_MIN = 1.000
KILL_R8_LO_MIN = 1.020


class ByteMismatch(ValueError):
    """The measured decoder returned a wrong digest, byte count, or CRC mismatch."""


class VoidRun(ValueError):
    """A harness, control, trace, or scan-oracle precondition failed (VOID, not KILL)."""


def output(*cmd, cwd=ROOT):
    return subprocess.check_output(cmd, cwd=cwd, text=True).strip()


def save(path: Path, value) -> None:
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def digest(path: Path, opener=open) -> str:
    with opener(path, "rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def checkout_commit(recipe: Path, dest: Path, sha: str) -> None:
    """Fetch one exact pinned commit into a private checkout, safe from shallow runners."""
    remote = output("git", "remote", "get-url", "origin", cwd=recipe)
    subprocess.run(["git", "init", "--quiet", dest], check=True)
    subprocess.run(
        ["git", "fetch", "--quiet", "--no-tags", "--depth=1", remote, sha],
        cwd=dest,
        check=True,
    )
    if output("git", "rev-parse", "FETCH_HEAD^{commit}", cwd=dest) != sha:
        raise VoidRun(f"fetched commit differs from pinned {sha}")
    subprocess.run(["git", "checkout", "--quiet", "--detach", sha], cwd=dest, check=True)


def build_plan() -> dict:
    """Construct the frozen §2/§3/§5 execution plan."""
    schedule = [
        {
            "block": b,
            "order": ["base", "cand"] if b % 2 == 0 else ["cand", "base"],
            "wall_cells": [[c, k] for c, k in WALL_CELLS],
            "oracle_cells": [["wikidata", 8, "none"], ["wikidata", 8, "S"]],
        }
        for b in range(REPEATS)
    ]
    return {
        "claim": "forward-emission-run",
        "registration": "docs/journal/2026-11-27-forward-emission/registration.md",
        "base": {"commit": BASE_SHA, "features": ""},
        "cand": {"commit": CANDIDATE_SHA, "features": FEATURE},
        "trace_features": TRACE_FEATURES,
        "repeats": REPEATS,
        "bootstrap_samples": BOOTSTRAP_SAMPLES,
        "bootstrap_seed": BOOTSTRAP_SEED,
        "scan_residual_max": SCAN_RESIDUAL_MAX,
        "thresholds": {
            "pass": {
                "r1_hi_max": PASS_R1_HI_MAX,
                "r8s_hi_max": PASS_R8S_HI_MAX,
                "r8_hi_max": PASS_R8_HI_MAX,
                "text_hi_max": PASS_TEXT_HI_MAX,
            },
            "kill": {
                "r1_lo_min": KILL_R1_LO_MIN,
                "r8s_lo_min": KILL_R8S_LO_MIN,
                "r8_lo_min": KILL_R8_LO_MIN,
            },
        },
        "schedule": schedule,
    }


def check_cpu_quota(quota: object, *, source: str) -> None:
    """Accept unlimited cgroup quota in both `ab env` (None) and `ab env --facts` (dict) schemas."""
    if quota is None:
        return
    if isinstance(quota, dict):
        state = quota.get("state")
        cpus = quota.get("cpus")
        if state == "unlimited" and cpus is None:
            return
        raise VoidRun(f"{source} reports CPU quota restriction: {quota!r}")
    raise VoidRun(f"{source} reports CPU quota restriction: {quota!r}")


def assert_distinct_binaries(digests: dict[str, dict[str, str]]) -> None:
    """Reject identical base/cand binaries (guards against stale Cargo artifacts)."""
    base = digests.get("base", {})
    cand = digests.get("cand", {})
    for name in ("a6b_endpoint_run", "mtf_fold"):
        if not base.get(name) or not cand.get(name):
            raise VoidRun(f"missing compiled binary digest for {name}")
        if base[name] == cand[name]:
            raise VoidRun(f"base and cand {name} produced identical binaries ({base[name]})")


def build_binaries(
    base_src: Path, cand_src: Path, build_root: Path, *, include_helpers: bool = True
) -> dict[str, dict[str, Path]]:
    """Build pinned base and cand wall + scan-oracle trace binaries into isolated target dirs."""
    binaries: dict[str, dict[str, Path]] = {}
    for arm, src, feat in (
        ("base", base_src, ""),
        ("cand", cand_src, FEATURE),
    ):
        target_dir = build_root / f"cargo-target-{arm}"
        bins: dict[str, Path] = {}
        wall_examples = ["a6b_endpoint_run"]
        if arm == "base" and include_helpers:
            wall_examples += ["ab", "windows"]
        builds = (
            (wall_examples, feat),
            (["mtf_fold"], (TRACE_FEATURES + " " + feat).strip()),
        )
        for examples, features in builds:
            args = ["cargo", "build", "--release", "--locked", "--target-dir", str(target_dir)]
            if features:
                args += ["--features", features]
            for name in examples:
                args += ["--example", name]
            subprocess.run(args, cwd=src, check=True)
            for name in examples:
                dest = build_root / f"{arm}-{name}"
                shutil.copy2(target_dir / "release" / "examples" / name, dest)
                bins[name] = dest
        binaries[arm] = bins
    return binaries


def prepare(out: Path, tmp: Path):
    """Validate workstation invariants, fetch pinned commits, build binaries, prepare corpora."""
    if "i9-10900K" not in output("lscpu"):
        raise VoidRun("registered machine is the i9-10900K")
    if output("git", "status", "--porcelain", "--untracked-files=no"):
        raise VoidRun("recipe checkout has tracked changes")
    if not output("rustc", "-V").startswith("rustc 1.99.1 "):
        raise VoidRun("registered toolchain requires Rust 1.99.1")
    (out / "lscpu.txt").write_text(output("lscpu") + "\n", encoding="utf-8")
    cores_file = out / "lscpu-e.txt"
    cores_file.write_text(output("lscpu", "-e") + "\n", encoding="utf-8")
    cores = topology(str(cores_file))
    if len(cores) != 10 or any(len(c) != 2 for c in cores):
        raise VoidRun("expected ten physical cores with both hardware threads")
    if set(itertools.chain.from_iterable(cores)) != os.sched_getaffinity(0):
        raise VoidRun("all workstation CPUs must be available")
    (out / "commits.txt").write_text(
        f"head {output('git', 'rev-parse', 'HEAD')}\n"
        f"base {BASE_SHA} default features\n"
        f"cand {CANDIDATE_SHA} +{FEATURE}\n",
        encoding="utf-8",
    )
    base_src = tmp / "src-base"
    cand_src = tmp / "src-cand"
    checkout_commit(ROOT, base_src, BASE_SHA)
    checkout_commit(ROOT, cand_src, CANDIDATE_SHA)
    (out / "registration.md").write_bytes(
        (cand_src / "docs/journal/2026-11-27-forward-emission/registration.md").read_bytes()
    )
    binaries = build_binaries(base_src, cand_src, tmp)
    binary_digests = {a: {k: digest(p) for k, p in b.items()} for a, b in binaries.items()}
    assert_distinct_binaries(binary_digests)
    save(out / "binaries.json", binary_digests)

    env_obj = json.loads(output(str(binaries["base"]["ab"]), "env"))
    facts_obj = json.loads(output(str(binaries["base"]["ab"]), "env", "--facts"))
    save(out / "env.json", env_obj)
    save(out / "env-facts.json", facts_obj)
    check_cpu_quota(env_obj.get("cpu_quota"), source="ab env")
    check_cpu_quota(facts_obj.get("cpu_quota"), source="ab env --facts")

    spec = json.loads((ROOT / "bench/wikidata.json").read_text(encoding="utf-8"))
    sample = next(
        (
            p
            for p in (ROOT / "corpora/wikidata-sample.bz2", ROOT / "corpora/wikidata.bz2")
            if p.exists() and digest(p) == spec["corpus_sha256"]
        ),
        None,
    )
    if sample is None:
        sample = tmp / "wikidata.bz2"
        windows = tmp / "windows"
        windows.mkdir()
        remote = output("git", "remote", "get-url", "origin", cwd=ROOT)
        subprocess.run(
            ["git", "fetch", "--quiet", "--no-tags", "--depth=1", remote, "bench-data"],
            cwd=base_src,
            check=True,
        )
        for window in spec["windows"]:
            name = window["name"] + ".bin"
            (windows / name).write_bytes(
                subprocess.check_output(
                    ["git", "show", f"FETCH_HEAD:{spec['windows_dir']}/{name}"], cwd=base_src
                )
            )
        wikidata.build(spec, windows, str(binaries["base"]["windows"]), sample)
    if digest(sample) != spec["corpus_sha256"]:
        raise VoidRun("unexpected Wikidata input hash")
    x4 = tmp / "wikidata-x4.bz2"
    with x4.open("wb") as dest:
        for _ in range(4):
            with sample.open("rb") as src:
                shutil.copyfileobj(src, dest)
    gen = tmp / "corpora"
    subprocess.run([str(binaries["base"]["ab"]), "gen", str(gen), "16"], check=True)
    files = {"wikidata": x4, "text": gen / "text.bz2"}
    hashes = {name: digest(path, bz2.open) for name, path in files.items()}
    (out / "inputs.txt").write_text(
        "".join(
            f"{name} compressed {digest(path)} decompressed {hashes[name]}\n"
            for name, path in files.items()
        ),
        encoding="utf-8",
    )
    calibration = {
        str(core[0]): json.loads(
            output(
                "taskset",
                "-c",
                str(core[0]),
                str(binaries["base"]["mtf_fold"]),
                "--calibrate",
                "200000000",
                "5",
            )
        )
        for core in cores
    }
    save(out / "cores.json", calibration)
    cpu, siblings, _ = fastest_core(cores, str(out / "cores.json"))
    mask = sorted(set(itertools.chain.from_iterable(cores)).difference(siblings))
    return binaries, files, hashes, cpu, mask


def wall_sample(binary: Path, path: Path, k: int) -> dict:
    """Run one single-sample wall+RSS+SHA-256 measurement via a6b_endpoint_run."""
    proc = subprocess.run(
        [str(binary), str(path), str(k)],
        timeout=180,
        text=True,
        capture_output=True,
    )
    if proc.returncode != 0:
        return {
            "kind": "failed",
            "stderr": proc.stderr.strip(),
            "returncode": proc.returncode,
        }
    try:
        parsed = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        return {
            "kind": "failed",
            "stderr": f"malformed JSON from wall binary: {exc}",
            "returncode": 0,
        }
    if not isinstance(parsed, dict):
        return {"kind": "failed", "stderr": "non-object JSON from wall binary", "returncode": 0}
    return parsed


def sample_trace(
    binary: Path,
    path: Path,
    corpus: str,
    k: int,
    oracle: str,
    block: int,
    cpu: int,
    mask: list[int],
) -> dict:
    """Run one warm paired-oracle trace measurement via mtf_fold."""
    cell = {
        "section": corpus,
        "arm": oracle,
        "k": k,
        "prespin_ms": 100,
        "pinned": True,
        "scanner_cpu": cpu,
        "mask": mask,
    }
    return run_child(str(binary), str(path), cell, 0, block, True, 180)


def verify_output(row: dict, expected_digest: str) -> None:
    """Check exact output SHA-256 and byte count; classify CRC/digest mismatch as ByteMismatch."""
    if row.get("kind") == "failed":
        msg = str(row.get("stderr", ""))
        if "block CRC mismatch" in msg or "stream CRC mismatch" in msg:
            raise ByteMismatch(msg)
        raise VoidRun(f"sample failed: {row}")
    if row.get("measure") == "wall":
        actual = row.get("sha256")
        if not isinstance(actual, str) or not SHA256_RE.fullmatch(actual):
            raise VoidRun(f"missing or malformed wall sha256: {actual!r}")
        if not isinstance(row.get("bytes"), int) or row["bytes"] <= 0:
            raise VoidRun(f"missing or non-positive decoded bytes: {row.get('bytes')!r}")
    else:
        cp = row.get("check_pass")
        if not isinstance(cp, dict):
            raise VoidRun("missing trace check_pass record")
        if cp.get("identical") is not True or cp.get("bytes") != row.get("bytes"):
            raise ByteMismatch("trace output or byte count differs from its reference")
        actual = cp.get("sha256")
        if not isinstance(actual, str) or not SHA256_RE.fullmatch(actual):
            raise VoidRun(f"missing or malformed trace sha256: {actual!r}")
    if actual != expected_digest:
        raise ByteMismatch(f"expected {expected_digest}, measured {actual}")


def validate_trace(row: dict, oracle: str) -> None:
    """Validate warm paired-oracle trace integrity and Open-kernel engagement."""
    if row.get("kind") != "decode" or row.get("scanner_pin_ok") is not True:
        raise VoidRun("missing trace decode or scanner affinity")
    trace = row.get("trace_summary")
    oracles = row.get("oracles")
    if not isinstance(trace, dict) or not isinstance(oracles, dict):
        raise VoidRun("missing trace_summary or oracles")
    decoded = trace.get("blocks_decoded", 0)
    if not (
        trace.get("workers_seen") == row.get("k")
        and decoded > 0
        and 0 <= trace.get("blocks_open_pass1", -1) <= decoded
        and decoded == trace.get("blocks_issued")
    ):
        raise VoidRun("worker trace is incomplete")
    if trace.get("worker_busy_ns_per_w", 0) <= 0 or row.get("maxrss_kib", 0) <= 0:
        raise VoidRun("missing worker busy time or RSS")
    if row.get("section") == "wikidata" and trace.get("blocks_open_pass1", 0) == 0:
        raise VoidRun("wikidata did not exercise Open")
    if oracle == "S":
        if (
            oracles.get("scan_locks")
            or oracles.get("scanned")
            or oracles.get("replayed") != decoded
            or oracles.get("recorded_blocks") != decoded
        ):
            raise VoidRun("scan oracle did not replay every block")
    elif oracles.get("replayed") or oracles.get("scanned") != decoded:
        raise VoidRun("real scanner and worker block counts differ")


def validate_pair(
    pair: dict[str, dict], corpus: str, k: int, block: int, measure: str, expected_digest: str
) -> None:
    """Validate both arms of a paired repeat; cand exactness errors are ByteMismatch (KILL)."""
    for arm in ("base", "cand"):
        row = pair[arm]
        context = f"{arm} {corpus} k={k} round={block} {measure}"
        try:
            verify_output(row, expected_digest)
        except ByteMismatch as exc:
            if arm == "cand":
                raise ByteMismatch(f"{context}: {exc}") from exc
            raise VoidRun(f"control failed exactness check: {context}: {exc}") from exc
        wall_s = row.get("wall_s")
        if (
            row.get("k") != k
            or not isinstance(wall_s, (int, float))
            or not math.isfinite(wall_s)
            or wall_s <= 0
            or not isinstance(row.get("maxrss_kib"), (int, float))
            or row["maxrss_kib"] <= 0
        ):
            raise VoidRun(f"unexpected topology, wall_s, or RSS: {context}")
        if measure != "wall":
            validate_trace(row, "none" if measure == "preflight" else measure)


def run_experiment(
    out: Path,
    bins: dict[str, dict[str, Path]],
    files: dict[str, Path],
    hashes: dict[str, str],
    cpu: int,
    mask: list[int],
) -> list[dict]:
    """Execute preflight and all 15 interleaved AB/BA paired repeats."""
    rows: list[dict] = []
    with (out / "runs.jsonl").open("x", encoding="utf-8", buffering=1) as log:

        def sample_pair(corpus: str, k: int, block: int, measure: str, order: tuple[str, str]):
            pair: dict[str, dict] = {}
            for arm in order:
                if measure == "wall":
                    row = wall_sample(bins[arm]["a6b_endpoint_run"], files[corpus], k)
                else:
                    oracle = "none" if measure == "preflight" else measure
                    row = sample_trace(
                        bins[arm]["mtf_fold"], files[corpus], corpus, k, oracle, block, cpu, mask
                    )
                raw = {
                    **row,
                    "build": arm,
                    "measure": measure,
                    "section": corpus,
                    "block": block,
                    "expected_sha256": hashes[corpus],
                }
                log.write(json.dumps(raw, allow_nan=False) + "\n")
                rows.append(raw)
                pair[arm] = raw
            validate_pair(pair, corpus, k, block, measure, hashes[corpus])

        for corpus in ("wikidata", "text"):
            sample_pair(corpus, 1, -1, "preflight", ("base", "cand"))
        for block in range(REPEATS):
            order = ("base", "cand") if block % 2 == 0 else ("cand", "base")
            for corpus, k in WALL_CELLS:
                sample_pair(corpus, k, block, "wall", order)
            for oracle in ("none", "S"):
                sample_pair("wikidata", 8, block, oracle, order)
    return rows


def _quantile(sorted_vals: list[float], q: float) -> float:
    """Linear-interpolated quantile on a non-empty sorted list."""
    pos = q * (len(sorted_vals) - 1)
    lo = math.floor(pos)
    hi = math.ceil(pos)
    if lo == hi:
        return sorted_vals[lo]
    return sorted_vals[lo] + (pos - lo) * (sorted_vals[hi] - sorted_vals[lo])


def paired_bootstrap_stats(
    base: list[float],
    cand: list[float],
    *,
    samples: int = BOOTSTRAP_SAMPLES,
    seed: int = BOOTSTRAP_SEED,
) -> dict:
    """Compute medians with [min, max], median of paired ratios, and 95% paired bootstrap CI."""
    if len(base) != len(cand) or len(base) < 2:
        raise VoidRun("paired bootstrap requires at least 2 paired observations")
    ratios = []
    for b, c in zip(base, cand):
        if not (math.isfinite(b) and math.isfinite(c) and b > 0 and c > 0):
            raise VoidRun("measurements must be positive and finite")
        ratios.append(c / b)
    n = len(ratios)
    rng = random.Random(seed)
    boot = sorted(statistics.median(rng.choices(ratios, k=n)) for _ in range(samples))
    lo = _quantile(boot, 0.025)
    hi = _quantile(boot, 0.975)
    gmean, t_lo, t_hi = paired_ratio_ci(cand, base)
    return {
        "n": n,
        "ratio": statistics.median(ratios),
        "ci95": [lo, hi],
        "t_ratio": gmean,
        "t_ci95": [t_lo, t_hi],
        "base_median": statistics.median(base),
        "base_range": [min(base), max(base)],
        "cand_median": statistics.median(cand),
        "cand_range": [min(cand), max(cand)],
        "paired_median_gain": statistics.median(b - c for b, c in zip(base, cand)),
    }


def _collect_pairs(
    rows: list[dict], metric, *, repeats: int = REPEATS
) -> tuple[list[float], list[float]]:
    pairs: dict[tuple[int, str], float] = {}
    for r in rows:
        block = r.get("block")
        arm = r.get("build")
        if arm not in ("base", "cand") or not isinstance(block, int) or block not in range(repeats):
            raise VoidRun(f"duplicate or unexpected round: build={arm!r} block={block!r}")
        key = (block, arm)
        if key in pairs:
            raise VoidRun(f"duplicate observation for round {block} arm {arm}")
        val = metric(r)
        if not isinstance(val, (int, float)) or not math.isfinite(val) or val <= 0:
            raise VoidRun(f"measurement must be positive and finite (round {block} arm {arm})")
        pairs[key] = float(val)
    missing = [
        (i, arm) for i in range(repeats) for arm in ("base", "cand") if (i, arm) not in pairs
    ]
    if missing:
        raise VoidRun(f"missing paired rounds: {missing[:4]}")
    base = [pairs[i, "base"] for i in range(repeats)]
    cand = [pairs[i, "cand"] for i in range(repeats)]
    return base, cand


def _check_all_rows_exact(rows: list[dict]) -> None:
    """Re-verify exactness and trace invariants on all rows when analyzing from disk."""
    by_corpus_ref: dict[str, str] = {}
    for r in rows:
        corpus = r.get("section")
        exp = r.get("expected_sha256")
        if isinstance(corpus, str) and isinstance(exp, str) and SHA256_RE.fullmatch(exp):
            prev = by_corpus_ref.get(corpus)
            if prev is not None and prev != exp:
                raise VoidRun(f"inconsistent expected_sha256 for {corpus}")
            by_corpus_ref[corpus] = exp
    for r in rows:
        arm = r.get("build")
        corpus = r.get("section")
        measure = r.get("measure")
        exp = by_corpus_ref.get(corpus)
        if not exp:
            raise VoidRun(f"missing expected_sha256 for {corpus!r}")
        try:
            verify_output(r, exp)
        except ByteMismatch as exc:
            if arm == "cand":
                raise ByteMismatch(
                    f"{arm} {corpus} k={r.get('k')} round={r.get('block')} {measure}: {exc}"
                ) from exc
            raise VoidRun(f"control exactness failure: {exc}") from exc
        if measure not in ("wall", "preflight"):
            validate_trace(r, measure)


def summarise(rows: list[dict], *, repeats: int = REPEATS) -> dict:
    """Apply frozen registration §2-§3 verbatim to produce PASS, KILL, RESCUE, or VOID."""
    try:
        _check_all_rows_exact(rows)
    except ByteMismatch as exc:
        return {
            "verdict": "KILL",
            "reason": f"exactness gate failed on cand: {exc}",
            "exactness_ok": False,
        }
    except VoidRun as exc:
        return {
            "verdict": "VOID",
            "reason": str(exc),
            "exactness_ok": False,
        }

    try:
        cells: dict[str, dict] = {}
        for corpus, k in WALL_CELLS:
            group = [
                r
                for r in rows
                if (r.get("measure"), r.get("section"), r.get("k")) == ("wall", corpus, k)
            ]
            name = f"{corpus}/{k}"
            b_wall, c_wall = _collect_pairs(group, lambda x: x["wall_s"], repeats=repeats)
            b_rss, c_rss = _collect_pairs(group, lambda x: x["maxrss_kib"], repeats=repeats)
            cells[name] = {
                "wall_s": paired_bootstrap_stats(b_wall, c_wall),
                "rss_kib": paired_bootstrap_stats(b_rss, c_rss),
            }

        for oracle in ("none", "S"):
            group = [
                r
                for r in rows
                if (r.get("measure"), r.get("section"), r.get("k")) == (oracle, "wikidata", 8)
            ]
            b_wall, c_wall = _collect_pairs(group, lambda x: x["wall_s"], repeats=repeats)
            b_busy, c_busy = _collect_pairs(
                group, lambda x: x["trace_summary"]["worker_busy_ns_per_w"], repeats=repeats
            )
            b_rss, c_rss = _collect_pairs(group, lambda x: x["maxrss_kib"], repeats=repeats)
            cells[f"wikidata/8/{oracle}"] = {
                "wall_s": paired_bootstrap_stats(b_wall, c_wall),
                "wkr_busy_ns_per_w": paired_bootstrap_stats(b_busy, c_busy),
                "rss_kib": paired_bootstrap_stats(b_rss, c_rss),
            }

        residuals: dict[str, float] = {}
        for arm in ("base", "cand"):
            real = [
                r["trace_summary"]["scanner_busy_ns"]
                for r in rows
                if (r.get("measure"), r.get("build"), r.get("section"), r.get("k"))
                == ("none", arm, "wikidata", 8)
            ]
            ideal = [
                r["trace_summary"]["scanner_busy_ns"]
                for r in rows
                if (r.get("measure"), r.get("build"), r.get("section"), r.get("k"))
                == ("S", arm, "wikidata", 8)
            ]
            if len(real) != repeats or len(ideal) != repeats:
                raise VoidRun(f"{arm}: incomplete scan-oracle trace rounds")
            med_real = statistics.median(real)
            med_ideal = statistics.median(ideal)
            if med_real <= 0 or med_ideal < 0:
                raise VoidRun(f"{arm}: invalid scanner_busy_ns")
            residuals[arm] = med_ideal / med_real
            if residuals[arm] > SCAN_RESIDUAL_MAX:
                raise VoidRun(
                    f"{arm}: scan oracle residual {residuals[arm]:.4f} exceeds {SCAN_RESIDUAL_MAX}"
                )
    except VoidRun as exc:
        return {
            "verdict": "VOID",
            "reason": str(exc),
            "exactness_ok": True,
        }

    r1_lo, r1_hi = cells["wikidata/1"]["wall_s"]["ci95"]
    r8_lo, r8_hi = cells["wikidata/8"]["wall_s"]["ci95"]
    r8s_lo, r8s_hi = cells["wikidata/8/S"]["wall_s"]["ci95"]
    text_lo, text_hi = cells["text/1"]["wall_s"]["ci95"]

    kill_clauses = {
        "r1_lo_gt_0.980": r1_lo > KILL_R1_LO_MIN,
        "r8s_lo_gt_1.000": r8s_lo > KILL_R8S_LO_MIN,
        "r8_lo_gt_1.020": r8_lo > KILL_R8_LO_MIN,
    }
    pass_clauses = {
        "r1_hi_le_0.950": r1_hi <= PASS_R1_HI_MAX,
        "r8s_hi_le_0.975": r8s_hi <= PASS_R8S_HI_MAX,
        "r8_hi_le_1.010": r8_hi <= PASS_R8_HI_MAX,
        "text_hi_le_1.010": text_hi <= PASS_TEXT_HI_MAX,
    }

    if any(kill_clauses.values()):
        verdict = "KILL"
        fired = [k for k, v in kill_clauses.items() if v]
        reason = f"§3 KILL clause(s) triggered: {', '.join(fired)}"
    elif all(pass_clauses.values()):
        verdict = "PASS"
        reason = "all §3 PASS clauses satisfied (r1<=0.950, r8S<=0.975, r8<=1.010, text<=1.010)"
    else:
        verdict = "RESCUE"
        unmet = [k for k, v in pass_clauses.items() if not v]
        reason = f"between §3 PASS and KILL (unmet PASS clause(s): {', '.join(unmet)})"

    return {
        "verdict": verdict,
        "reason": reason,
        "exactness_ok": True,
        "pass_clauses": pass_clauses,
        "kill_clauses": kill_clauses,
        "scan_busy_residual": residuals,
        "cells": cells,
        "ratios": {
            "r1": {"ratio": cells["wikidata/1"]["wall_s"]["ratio"], "ci95": [r1_lo, r1_hi]},
            "r8": {"ratio": cells["wikidata/8"]["wall_s"]["ratio"], "ci95": [r8_lo, r8_hi]},
            "r8S": {"ratio": cells["wikidata/8/S"]["wall_s"]["ratio"], "ci95": [r8s_lo, r8s_hi]},
            "text_1w": {"ratio": cells["text/1"]["wall_s"]["ratio"], "ci95": [text_lo, text_hi]},
        },
    }


def write_verdict(out: Path, summary: dict) -> int:
    """Persist summary.json and outcome.txt; return 1 on VOID, 0 otherwise."""
    save(out / "summary.json", summary)
    verdict = summary["verdict"]
    reason = summary["reason"]
    (out / "outcome.txt").write_text(
        f"{verdict}: {reason}\n",
        encoding="utf-8",
    )
    return 1 if verdict == "VOID" else 0


def cmd_plan(out: Path) -> int:
    out.mkdir(parents=True, exist_ok=True)
    save(out / "plan.json", build_plan())
    print(f"wrote {out / 'plan.json'}")
    return 0


def cmd_report(runs_path: Path, out: Path | None) -> int:
    rows = [
        json.loads(line)
        for line in runs_path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    summary = summarise(rows)
    if out is not None:
        out.mkdir(parents=True, exist_ok=True)
        rc = write_verdict(out, summary)
    else:
        rc = 1 if summary["verdict"] == "VOID" else 0
    print(json.dumps(summary, indent=2, allow_nan=False))
    return rc


def cmd_run(out: Path) -> int:
    out.mkdir(parents=True, exist_ok=True)
    save(out / "plan.json", build_plan())
    try:
        with tempfile.TemporaryDirectory(prefix="build-", dir=out.parent) as tmp:
            args = prepare(out, Path(tmp))
            try:
                rows = run_experiment(out, *args)
            except ByteMismatch as exc:
                summary = {
                    "verdict": "KILL",
                    "reason": f"exactness gate failed on cand: {exc}",
                    "exactness_ok": False,
                }
                return write_verdict(out, summary)
    except VoidRun as exc:
        summary = {"verdict": "VOID", "reason": str(exc), "exactness_ok": False}
        return write_verdict(out, summary)
    summary = summarise(rows)
    return write_verdict(out, summary)


def main(argv: list[str] | None = None) -> int:
    args_list = sys.argv[1:] if argv is None else argv
    if len(args_list) == 1 and args_list[0] not in ("plan", "run", "report", "-h", "--help"):
        return cmd_run(Path(args_list[0]).resolve())
    parser = argparse.ArgumentParser(description="forward-anchor-lanes i9 kill-test recipe")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p_plan = sub.add_parser("plan", help="write frozen plan.json")
    p_plan.add_argument("out", type=Path)
    p_run = sub.add_parser("run", help="run the full i9 kill test into OUT")
    p_run.add_argument("out", type=Path)
    p_rep = sub.add_parser("report", help="score a runs.jsonl file under frozen §3")
    p_rep.add_argument("runs", type=Path)
    p_rep.add_argument("--out", type=Path, default=None)
    ns = parser.parse_args(args_list)
    if ns.cmd == "plan":
        return cmd_plan(ns.out.resolve())
    if ns.cmd == "run":
        return cmd_run(ns.out.resolve())
    return cmd_report(ns.runs.resolve(), ns.out.resolve() if ns.out else None)


if __name__ == "__main__":
    sys.exit(main())
