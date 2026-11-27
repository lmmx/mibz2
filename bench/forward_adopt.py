#!/usr/bin/env python3
"""Frozen C6 physical i9 full-guard adoption benchmark for the disjoint-slab four-lane
forward walk (`--features forward-anchor-lanes`, #1110 + #1217 -> default OpenCells walk).

Claim: frontier/claims/forward-emission-adopt.toml
Registration: docs/journal/2026-11-27-forward-adopt/registration.md

Usage:
    python3 bench/forward_adopt.py run OUT_DIR [--smoke] [--rounds N] [--settle S]
    python3 bench/forward_adopt.py score OUT_DIR
"""

from __future__ import annotations

import argparse
import bz2
import hashlib
import json
import math
import os
from pathlib import Path
import random
import shutil
import statistics
import subprocess
import sys
import tempfile
from typing import Any

BENCH_DIR = Path(__file__).resolve().parent
if str(BENCH_DIR) not in sys.path:
    sys.path.insert(0, str(BENCH_DIR))

from mtf_fold import paired_ratio_ci, t_crit_95, topology  # noqa: E402
import wikidata  # noqa: E402

ROOT = BENCH_DIR.parent
REGISTRATION = ROOT / "docs/journal/2026-11-27-forward-adopt/registration.md"

X4_SHA256 = "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"
WIKIDATA_COUNTS = (1, 6, 8, 10, 12)
GUARD_CORPORA = ("text", "records", "random", "runs", "pattern", "onebyte")
GUARD_COUNTS = (1, 8)
ROUNDS = 16
SETTLE_S = "2"
PRESPIN_MS = 100
MAX_BUSY = "1.5"
GEN_SCALE = 4
BOOT_SAMPLES = 10_000
BOOT_SEED = 1242

CELLS = tuple(("wikidata", k) for k in WIKIDATA_COUNTS) + tuple(
    (name, k) for name in GUARD_CORPORA for k in GUARD_COUNTS
)

# Frozen C6 adoption thresholds (verbatim from frontier/claims/forward-emission-adopt.toml)
WIKIDATA_WALL_HI_MAX = {1: 0.900, 8: 0.965, 10: 0.985}
GUARD_WALL_MAX = 1.020
WIKIDATA_RSS_MAX = {1: 1.020, 6: 1.080, 8: 1.080, 10: 1.080, 12: 1.080}
GUARD_RSS_MAX = 1.050

RUSTFLAGS_ENV_VARS = ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS")


class InvalidRun(ValueError):
    """A registered prerequisite failed or the measurement cannot support a verdict."""


def output(*args: str, cwd: Path = ROOT, env: dict[str, str] | None = None) -> str:
    return subprocess.check_output(args, cwd=cwd, text=True, env=env).strip()


def save(path: Path, obj: Any) -> None:
    path.write_text(json.dumps(obj, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def sha256_file(path: Path | str) -> str:
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def reference(path: Path | str) -> dict[str, Any]:
    """Independently decompress with C libbzip2 (`bz2.open`) and return exact size + SHA-256."""
    with bz2.open(path, "rb") as source:
        digest = hashlib.sha256()
        size = 0
        while buf := source.read(1 << 20):
            size += len(buf)
            digest.update(buf)
    return {"out": size, "sha256": digest.hexdigest()}


def arm_for_threads(threads: int) -> str:
    """Return the `ab` arm name: `mibz2-workers-lazy` at k=1 (OpenCells) and `mibz2` at k>=6."""
    return "mibz2-workers-lazy" if threads == 1 else "mibz2"


def check_baseline_env(env: dict[str, str] | None = None) -> None:
    """Enforce default baseline x86-64 codegen with no ambient RUSTFLAGS overrides."""
    cur = os.environ if env is None else env
    for var in RUSTFLAGS_ENV_VARS:
        val = cur.get(var, "").strip()
        if val:
            raise InvalidRun(f"non-empty {var}={val!r}; registered adoption run requires baseline")


def check_host_i9(out: Path, require_i9: bool = True) -> dict[str, Any]:
    """Record lscpu + affinity and verify unrestricted 10-core / 20-CPU i9-10900K when strict."""
    lscpu_txt = output("lscpu")
    (out / "lscpu.txt").write_text(lscpu_txt + "\n", encoding="utf-8")
    lscpu_e = output("lscpu", "-e")
    grid = out / "lscpu-e.txt"
    grid.write_text(lscpu_e + "\n", encoding="utf-8")
    cores = topology(str(grid))
    cpus = {cpu for core in cores for cpu in core}
    allowed = os.sched_getaffinity(0)
    pin_cpu = sorted(allowed)[0] if allowed else 0
    if require_i9:
        if "i9-10900K" not in lscpu_txt:
            raise InvalidRun("workstation model differs from registered i9-10900K")
        if len(cores) != 10 or len(cpus) != 20 or cpus != allowed:
            raise InvalidRun(
                f"expected unrestricted 10-core/20-CPU i9 topology, got "
                f"{len(cores)} cores / {len(cpus)} CPUs / allowed={sorted(allowed)}"
            )
        pin_cpu = cores[0][0]
    info = {
        "n_cores": len(cores),
        "n_cpus": len(cpus),
        "cpus_allowed": sorted(allowed),
        "pin_cpu": pin_cpu,
        "require_i9": require_i9,
    }
    save(out / "host.json", info)
    return info


def build_arms(
    out: Path,
    tmp: Path,
    *,
    root: Path = ROOT,
    base_features: str = "",
    cand_features: str = "forward-anchor-lanes",
    prebuilt_base: Path | None = None,
    prebuilt_cand: Path | None = None,
    prebuilt_windows: Path | None = None,
) -> tuple[dict[str, Path], dict[str, Any]]:
    """Build base (default) and cand (+forward-anchor-lanes) into isolated CARGO_TARGET_DIRs.

    Enforces the distinct-binary VOID guard (`base_ab.sha256 != cand_ab.sha256`).
    """
    clean_env = {k: v for k, v in os.environ.items() if k not in RUSTFLAGS_ENV_VARS}
    bins: dict[str, Path] = {}

    if prebuilt_base is not None:
        bins["base_ab"] = Path(prebuilt_base).resolve()
    else:
        t_base = tmp / "target-base"
        cmd = ["cargo", "build", "--release", "--locked", "--example", "ab"]
        if prebuilt_windows is None:
            cmd.extend(["--example", "windows"])
        if base_features:
            cmd.extend(["--features", base_features])
        subprocess.run(
            cmd,
            cwd=root,
            check=True,
            env={**clean_env, "CARGO_TARGET_DIR": str(t_base)},
        )
        dst_ab = tmp / "base-ab"
        shutil.copy2(t_base / "release/examples/ab", dst_ab)
        bins["base_ab"] = dst_ab
        if prebuilt_windows is None:
            dst_win = tmp / "base-windows"
            shutil.copy2(t_base / "release/examples/windows", dst_win)
            bins["windows"] = dst_win

    if prebuilt_windows is not None:
        bins["windows"] = Path(prebuilt_windows).resolve()

    if prebuilt_cand is not None:
        bins["cand_ab"] = Path(prebuilt_cand).resolve()
    else:
        t_cand = tmp / "target-cand"
        cmd = ["cargo", "build", "--release", "--locked", "--example", "ab"]
        if cand_features:
            cmd.extend(["--features", cand_features])
        subprocess.run(
            cmd,
            cwd=root,
            check=True,
            env={**clean_env, "CARGO_TARGET_DIR": str(t_cand)},
        )
        dst_cand = tmp / "cand-ab"
        shutil.copy2(t_cand / "release/examples/ab", dst_cand)
        bins["cand_ab"] = dst_cand

    base_sha = sha256_file(bins["base_ab"])
    cand_sha = sha256_file(bins["cand_ab"])
    distinct = base_sha != cand_sha
    meta = {
        "base": {
            "path": str(bins["base_ab"]),
            "features": base_features or "default",
            "sha256": base_sha,
        },
        "cand": {
            "path": str(bins["cand_ab"]),
            "features": cand_features or "default",
            "sha256": cand_sha,
        },
        "distinct_binaries": distinct,
    }
    if "windows" in bins:
        meta["windows"] = {"path": str(bins["windows"]), "sha256": sha256_file(bins["windows"])}
    save(out / "binaries.json", meta)
    if not distinct:
        raise InvalidRun(
            f"VOID: base and cand ab binaries have identical SHA-256 ({base_sha}); "
            "feature flag did not differentiate the candidate build"
        )
    return bins, meta


def materialize_wikidata_x4(
    tmp: Path,
    windows_exe: Path,
    *,
    root: Path = ROOT,
    verify_x4_sha: bool = True,
    repeat_factor: int = 4,
) -> Path:
    """Shallow-safely materialize `wikidata.bz2` (from local cache or --depth=1 origin bench-data)."""
    spec = json.loads((root / "bench/wikidata.json").read_text(encoding="utf-8"))
    wiki = root / "corpora/wikidata.bz2"
    if not wiki.exists() or sha256_file(wiki) != spec["corpus_sha256"]:
        win = tmp / "windows"
        win.mkdir(exist_ok=True)
        bench_git = tmp / "bench-data-git"
        bench_git.mkdir(exist_ok=True)
        subprocess.run(["git", "init", "-q"], cwd=bench_git, check=True)
        remote_url = output("git", "remote", "get-url", "origin", cwd=root)
        subprocess.run(
            ["git", "fetch", "--depth", "1", "--no-tags", remote_url, "bench-data"],
            cwd=bench_git,
            check=True,
        )
        for w in spec["windows"]:
            name = w["name"] + ".bin"
            (win / name).write_bytes(
                subprocess.check_output(
                    ["git", "show", f"FETCH_HEAD:{spec['windows_dir']}/{name}"],
                    cwd=bench_git,
                )
            )
        wiki = tmp / "wikidata-single.bz2"
        wikidata.build(spec, win, str(windows_exe), wiki)

    x4_dir = tmp / "x4"
    x4_dir.mkdir(exist_ok=True)
    x4 = x4_dir / "wikidata.bz2"
    with x4.open("wb") as dest:
        for _ in range(repeat_factor):
            with wiki.open("rb") as source:
                shutil.copyfileobj(source, dest)
    if verify_x4_sha and repeat_factor == 4 and sha256_file(x4) != X4_SHA256:
        raise InvalidRun("registered Wikidata x4 input SHA-256 mismatch")
    return x4


def prepare_inputs(
    out: Path,
    tmp: Path,
    bins: dict[str, Path],
    *,
    root: Path = ROOT,
    smoke: bool = False,
    guard_corpora: tuple[str, ...] = GUARD_CORPORA,
) -> tuple[dict[str, Path], dict[str, dict[str, Any]]]:
    """Prepare wikidata-x4 (or compact smoke wikidata) and all 6 guard corpora, hashed via libbzip2."""
    gen_dir = tmp / "gen"
    scale = 1 if smoke else GEN_SCALE
    subprocess.run([str(bins["base_ab"]), "gen", str(gen_dir), str(scale)], check=True)

    files: dict[str, Path] = {}
    if smoke:
        # In smoke mode, use `runs.bz2` / `text.bz2` or local wikidata if available without network cost.
        local_wiki = root / "corpora/wikidata.bz2"
        spec = json.loads((root / "bench/wikidata.json").read_text(encoding="utf-8"))
        if local_wiki.exists() and sha256_file(local_wiki) == spec["corpus_sha256"]:
            files["wikidata"] = local_wiki
        else:
            # Copy `runs.bz2` as a compact smoke stand-in that exercises OpenCells cleanly.
            smoke_wiki = tmp / "wikidata.bz2"
            shutil.copy2(gen_dir / "runs.bz2", smoke_wiki)
            files["wikidata"] = smoke_wiki
    else:
        files["wikidata"] = materialize_wikidata_x4(tmp, bins["windows"], root=root)

    for name in guard_corpora:
        path = gen_dir / f"{name}.bz2"
        if not path.exists():
            raise InvalidRun(f"missing generated guard corpus {path}")
        files[name] = path

    refs = {name: reference(path) for name, path in files.items()}
    save(
        out / "inputs.json",
        {
            name: {**refs[name], "compressed_sha256": sha256_file(path), "path": str(path)}
            for name, path in files.items()
        },
    )
    return files, refs


def canonical_arm(label: str, cand_exe: Path | str | None = None) -> str:
    """Map an `ab run` arm label (`mibz2`, `mibz2-workers-lazy`, or `<arm>@<path>`) to `base`/`cand`."""
    if label in ("base", "mibz2", "mibz2-workers-lazy"):
        return "base"
    if label == "cand":
        return "cand"
    if "@" in label:
        base_name, path = label.split("@", 1)
        if base_name not in ("mibz2", "mibz2-workers-lazy"):
            raise InvalidRun(f"unexpected arm prefix in {label!r}")
        if cand_exe is not None and Path(path).resolve() != Path(cand_exe).resolve():
            raise InvalidRun(f"unexpected cand executable path in arm label: {label}")
        return "cand"
    raise InvalidRun(f"unexpected arm label: {label!r}")


def verify_parity(
    out: Path,
    bins: dict[str, Path],
    files: dict[str, Path],
    refs: dict[str, dict[str, Any]],
    *,
    cells: tuple[tuple[str, int], ...] = CELLS,
) -> list[dict[str, Any]]:
    """Run untimed `AB_SHA256=1` `ab child` on every cell for both arms against C libbzip2."""
    env = {**os.environ, "AB_SHA256": "1"}
    records: list[dict[str, Any]] = []
    with (out / "parity.jsonl").open("w", encoding="utf-8", buffering=1) as log:
        for corpus, k in cells:
            ref = refs[corpus]
            arm_name = arm_for_threads(k)
            for arm_key, bin_key in (("base", "base_ab"), ("cand", "cand_ab")):
                done = subprocess.run(
                    [str(bins[bin_key]), "child", arm_name, str(k), str(files[corpus])],
                    capture_output=True,
                    text=True,
                    timeout=300,
                    env=env,
                )
                if done.returncode != 0:
                    raise InvalidRun(
                        f"parity child failed on {arm_key}/{corpus}/{k}: {done.stderr.strip()}"
                    )
                row = json.loads(done.stdout.strip())
                err = row.get("err", "")
                if err not in (None, ""):
                    raise InvalidRun(f"parity decoder error on {arm_key}/{corpus}/{k}: {err}")
                if row.get("out") != ref["out"] or row.get("sha256") != ref["sha256"]:
                    raise InvalidRun(
                        f"parity byte/SHA-256 mismatch on {arm_key}/{corpus}/{k}: "
                        f"got ({row.get('out')}, {row.get('sha256')}) != "
                        f"libbzip2 ({ref['out']}, {ref['sha256']})"
                    )
                h = row.get("hash")
                if not h:
                    raise InvalidRun(f"missing 64-bit hash on parity row {arm_key}/{corpus}/{k}")
                if "hash" in ref and ref["hash"] != h:
                    raise InvalidRun(f"64-bit hash mismatch between arms on {corpus}/{k}")
                ref["hash"] = h
                rec = {"arm": arm_key, "arm_name": arm_name, "corpus": corpus, "k": k, **row}
                log.write(json.dumps(rec, allow_nan=False) + "\n")
                records.append(rec)
    return records


def run_ab_group(
    base_ab: Path,
    cand_ab: Path,
    dest: Path,
    *,
    arm_name: str,
    counts: tuple[int, ...],
    corpus_paths: list[Path],
    rounds: int,
    pin_cpu: int,
    prespin_ms: int,
    settle_s: str,
    max_busy: str,
    machine_state: bool,
) -> list[dict[str, Any]]:
    """Run one paired `ab run` sweep and return parsed JSONL rows."""
    arms = f"{arm_name},{arm_name}@{cand_ab.resolve()}"
    cmd = [
        str(base_ab),
        "run",
        str(dest),
        "--arms",
        arms,
        "--threads",
        ",".join(map(str, counts)),
        "--rounds",
        str(rounds),
        "--pin",
        str(pin_cpu),
        "--settle",
        str(settle_s),
        "--max-busy",
        str(max_busy),
    ]
    if prespin_ms > 0:
        cmd.extend(["--prespin-ms", str(prespin_ms)])
    if machine_state:
        cmd.append("--machine-state")
    cmd.extend(str(p) for p in corpus_paths)
    subprocess.run(cmd, check=True)
    rows: list[dict[str, Any]] = []
    for line in dest.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def bootstrap_median_ci(
    ratios: list[float],
    *,
    n_boot: int = BOOT_SAMPLES,
    seed: int = BOOT_SEED,
) -> tuple[float, float, float]:
    """Deterministic paired median ratio and 95% percentile bootstrap CI."""
    if not ratios:
        return float("nan"), float("nan"), float("nan")
    med = statistics.median(ratios)
    if len(ratios) < 2:
        return med, med, med
    rng = random.Random(seed)
    n = len(ratios)
    boots = [statistics.median(rng.choices(ratios, k=n)) for _ in range(n_boot)]
    boots.sort()
    lo = boots[int(0.025 * n_boot)]
    hi = boots[int(0.975 * n_boot)]
    return med, lo, hi


def position_stratified_ci(
    paired_by_pos: dict[int, list[tuple[float, float]]],
) -> dict[str, Any]:
    """Compute position-stratified log-ratio estimate across pair positions (0 vs 1)."""
    strata: dict[str, Any] = {}
    weighted_mean = 0.0
    weighted_var = 0.0
    total_n = 0
    for pos in sorted(paired_by_pos):
        pairs = paired_by_pos[pos]
        if len(pairs) < 2:
            continue
        logs = [math.log(c / b) for b, c in pairs]
        n = len(logs)
        m = statistics.fmean(logs)
        v = statistics.variance(logs)
        se = math.sqrt(v / n)
        tc = t_crit_95(n - 1)
        strata[str(pos)] = {
            "n": n,
            "ratio": math.exp(m),
            "ci95": [math.exp(m - tc * se), math.exp(m + tc * se)],
        }
        weighted_mean += n * m
        weighted_var += (n**2) * (v / n)
        total_n += n
    if total_n < 2 or not strata:
        return {"strata": strata, "combined_ratio": None, "combined_ci95": None}
    comb_m = weighted_mean / total_n
    comb_se = math.sqrt(weighted_var) / total_n
    df = max(1, total_n - len(strata))
    tc = t_crit_95(df)
    return {
        "strata": strata,
        "combined_ratio": math.exp(comb_m),
        "combined_ci95": [math.exp(comb_m - tc * comb_se), math.exp(comb_m + tc * comb_se)],
    }


def _extract_cpr(row: dict[str, Any]) -> float | None:
    """Extract `cycles_per_ref` from an `ab --machine-state` row (top-level or nested `machine`)."""
    val = row.get("cycles_per_ref")
    if val is None:
        m = row.get("machine")
        if isinstance(m, dict):
            val = m.get("cycles_per_ref")
    if isinstance(val, (int, float)) and math.isfinite(val) and val > 0:
        return float(val)
    return None


def score(
    rows: list[dict[str, Any]],
    refs: dict[str, dict[str, Any]],
    *,
    cand_exe: Path | str | None = None,
    rounds: int = ROUNDS,
    wikidata_counts: tuple[int, ...] = WIKIDATA_COUNTS,
    guard_corpora: tuple[str, ...] = GUARD_CORPORA,
    guard_counts: tuple[int, ...] = GUARD_COUNTS,
    distinct_binaries: bool = True,
    host_ok: bool = True,
) -> dict[str, Any]:
    """Score timed `ab` rows against the frozen C6 adoption rule (ADOPT / DO NOT ADOPT / VOID)."""
    if not host_ok:
        return {"verdict": "VOID", "reason": "host/affinity/quota prerequisite failed"}
    if not distinct_binaries:
        return {
            "verdict": "VOID",
            "reason": "base and cand binaries have identical SHA-256",
        }

    expected_cells = tuple(("wikidata", k) for k in wikidata_counts) + tuple(
        (name, k) for name in guard_corpora for k in guard_counts
    )
    expected_set = set(expected_cells)

    by_key: dict[tuple[str, int, int, str], dict[str, Any]] = {}
    order_within_round: dict[tuple[str, int, int], list[str]] = {}
    exactness_ok = True

    for row in rows:
        raw_corpus = row.get("corpus")
        corpus = str(raw_corpus) if raw_corpus else Path(str(row.get("file", ""))).stem
        if corpus == "wikidata-single":
            corpus = "wikidata"
        k = int(row["threads"])
        r = int(row["round"])
        arm = canonical_arm(str(row["arm"]), cand_exe=cand_exe)
        if (corpus, k) not in expected_set:
            return {"verdict": "VOID", "reason": f"unexpected cell {(corpus, k)}"}
        if r < 0 or r >= rounds:
            return {"verdict": "VOID", "reason": f"out-of-range round {r}"}
        key = (corpus, k, r, arm)
        if key in by_key:
            return {"verdict": "VOID", "reason": f"duplicate row for {key}"}

        err = row.get("err", "")
        if err not in (None, ""):
            return {"verdict": "VOID", "reason": f"decoder error on {key}: {err}"}
        ref = refs.get(corpus)
        if ref is None:
            return {"verdict": "VOID", "reason": f"missing reference for corpus {corpus}"}
        if row.get("out") != ref["out"]:
            exactness_ok = False
            return {"verdict": "VOID", "reason": f"output length mismatch on {key}"}
        if "hash" in ref and row.get("hash") != ref["hash"]:
            exactness_ok = False
            return {"verdict": "VOID", "reason": f"output hash mismatch on {key}"}
        if "sha256" in row and row["sha256"] is not None and row["sha256"] != ref["sha256"]:
            exactness_ok = False
            return {"verdict": "VOID", "reason": f"output SHA-256 mismatch on {key}"}

        wall = row.get("wall")
        rss = row.get("rss_kb")
        if not (isinstance(wall, (int, float)) and math.isfinite(wall) and wall > 0):
            return {"verdict": "VOID", "reason": f"invalid wall {wall!r} on {key}"}
        if not (isinstance(rss, (int, float)) and math.isfinite(rss) and rss > 0):
            return {"verdict": "VOID", "reason": f"invalid rss_kb {rss!r} on {key}"}

        by_key[key] = row
        order_within_round.setdefault((corpus, k, r), []).append(arm)

    expected_total = len(expected_cells) * rounds * 2
    if len(by_key) != expected_total:
        return {
            "verdict": "VOID",
            "reason": f"incomplete grid: got {len(by_key)} rows, expected {expected_total}",
        }

    cells_out: dict[str, Any] = {}
    for corpus, k in expected_cells:
        base_walls: list[float] = []
        cand_walls: list[float] = []
        base_rss: list[float] = []
        cand_rss: list[float] = []
        paired_by_pos: dict[int, list[tuple[float, float]]] = {0: [], 1: []}
        base_cprs: list[float] = []
        cand_cprs: list[float] = []

        for r in range(rounds):
            b = by_key[(corpus, k, r, "base")]
            c = by_key[(corpus, k, r, "cand")]
            bw, cw = float(b["wall"]), float(c["wall"])
            br, cr = float(b["rss_kb"]), float(c["rss_kb"])
            base_walls.append(bw)
            cand_walls.append(cw)
            base_rss.append(br)
            cand_rss.append(cr)
            seq = order_within_round.get((corpus, k, r), [])
            cand_pos = 1 if (len(seq) >= 2 and seq[1] == "cand") else 0
            paired_by_pos[cand_pos].append((bw, cw))
            if (bc := _extract_cpr(b)) is not None:
                base_cprs.append(bc)
            if (cc := _extract_cpr(c)) is not None:
                cand_cprs.append(cc)

        wall_gm, wall_lo, wall_hi = paired_ratio_ci(cand_walls, base_walls)
        wall_ratios = [c / b for b, c in zip(base_walls, cand_walls, strict=True)]
        wall_med, wall_boot_lo, wall_boot_hi = bootstrap_median_ci(wall_ratios)

        rss_gm, rss_lo, rss_hi = paired_ratio_ci(cand_rss, base_rss)
        rss_ratios = [c / b for b, c in zip(base_rss, cand_rss, strict=True)]
        rss_med, rss_boot_lo, rss_boot_hi = bootstrap_median_ci(rss_ratios)

        if not all(
            math.isfinite(x) and x > 0 for x in (wall_gm, wall_lo, wall_hi, rss_gm, rss_lo, rss_hi)
        ):
            return {"verdict": "VOID", "reason": f"undefined CI on {corpus}/{k}"}

        cpr_info: dict[str, Any] | None = None
        if base_cprs and cand_cprs:
            b_med = statistics.median(base_cprs)
            c_med = statistics.median(cand_cprs)
            cpr_info = {
                "base_median": b_med,
                "cand_median": c_med,
                "ratio": c_med / b_med,
                "n_base": len(base_cprs),
                "n_cand": len(cand_cprs),
            }

        cells_out[f"{corpus}/{k}"] = {
            "corpus": corpus,
            "threads": k,
            "arm_name": arm_for_threads(k),
            "n": rounds,
            "wall_s": {
                "base_median": statistics.median(base_walls),
                "base_range": [min(base_walls), max(base_walls)],
                "cand_median": statistics.median(cand_walls),
                "cand_range": [min(cand_walls), max(cand_walls)],
                "t_ratio": wall_gm,
                "t_ci95": [wall_lo, wall_hi],
                "paired_median": wall_med,
                "boot_ci95": [wall_boot_lo, wall_boot_hi],
                "position_stratified": position_stratified_ci(paired_by_pos),
            },
            "rss_kib": {
                "base_median": statistics.median(base_rss),
                "cand_median": statistics.median(cand_rss),
                "t_ratio": rss_gm,
                "t_ci95": [rss_lo, rss_hi],
                "paired_median": rss_med,
                "boot_ci95": [rss_boot_lo, rss_boot_hi],
            },
            "cycles_per_ref": cpr_info,
        }

    # Evaluate every frozen C6 adoption clause
    clauses: dict[str, bool] = {
        "exactness_ok": exactness_ok,
        "distinct_binaries": distinct_binaries,
    }
    failed_reasons: list[str] = []

    for k, hi_max in WIKIDATA_WALL_HI_MAX.items():
        if k in wikidata_counts:
            cell = cells_out[f"wikidata/{k}"]
            hi = cell["wall_s"]["t_ci95"][1]
            ok = hi <= hi_max
            clauses[f"wikidata_{k}_hi_le_{hi_max:.3f}"] = ok
            if not ok:
                failed_reasons.append(f"wikidata/{k} t_ci95_hi={hi:.4f} > {hi_max:.3f}")

    guards_ok = True
    guard_failures: list[str] = []
    for corpus in guard_corpora:
        for k in guard_counts:
            cell = cells_out[f"{corpus}/{k}"]
            lo = cell["wall_s"]["t_ci95"][0]
            pmed = cell["wall_s"]["paired_median"]
            if lo > GUARD_WALL_MAX or pmed > GUARD_WALL_MAX:
                guards_ok = False
                guard_failures.append(
                    f"{corpus}/{k} (med={pmed:.4f}, t_ci95_lo={lo:.4f}, limit={GUARD_WALL_MAX:.3f})"
                )
    clauses["guards_no_regression"] = guards_ok
    if not guards_ok:
        failed_reasons.append("guard regression: " + "; ".join(guard_failures))

    wiki_rss_ok = True
    wiki_rss_fails: list[str] = []
    for k in wikidata_counts:
        limit = WIKIDATA_RSS_MAX.get(k, 1.080)
        pmed = cells_out[f"wikidata/{k}"]["rss_kib"]["paired_median"]
        if pmed > limit:
            wiki_rss_ok = False
            wiki_rss_fails.append(f"wikidata/{k} rss={pmed:.4f} > {limit:.3f}")
    clauses["wikidata_rss_ok"] = wiki_rss_ok
    if not wiki_rss_ok:
        failed_reasons.append("wikidata RSS exceeded: " + "; ".join(wiki_rss_fails))

    guard_rss_ok = True
    guard_rss_fails: list[str] = []
    for corpus in guard_corpora:
        for k in guard_counts:
            pmed = cells_out[f"{corpus}/{k}"]["rss_kib"]["paired_median"]
            if pmed > GUARD_RSS_MAX:
                guard_rss_ok = False
                guard_rss_fails.append(f"{corpus}/{k} rss={pmed:.4f} > {GUARD_RSS_MAX:.3f}")
    clauses["guards_rss_ok"] = guard_rss_ok
    if not guard_rss_ok:
        failed_reasons.append("guard RSS exceeded: " + "; ".join(guard_rss_fails))

    all_passed = all(clauses.values())
    verdict = "ADOPT" if all_passed else "DO NOT ADOPT"
    reason = (
        "all frozen C6 adoption clauses passed"
        if all_passed
        else "unmet C6 clause(s): " + "; ".join(failed_reasons)
    )
    return {
        "verdict": verdict,
        "reason": reason,
        "exactness_ok": exactness_ok,
        "distinct_binaries": distinct_binaries,
        "clauses": clauses,
        "cells": cells_out,
    }


def format_summary(summary: dict[str, Any]) -> str:
    """Format a compact human-readable report of all cells and clauses."""
    lines = [
        f"{summary['verdict']}: {summary['reason']}",
        "",
    ]
    if "clauses" in summary:
        lines.append("Clauses:")
        for k, v in summary["clauses"].items():
            lines.append(f"  {k}: {'PASS' if v else 'FAIL'}")
        lines.append("")
    cells = summary.get("cells", {})
    if cells:
        lines.append(
            f"{'cell':<14s} {'arm':<18s} {'base_s':>8s} {'cand_s':>8s} "
            f"{'t_ratio [ci95]':>24s} {'pmed':>7s} {'rss_r':>7s}"
        )
        for name, c in cells.items():
            w = c["wall_s"]
            r = c["rss_kib"]
            lo, hi = w["t_ci95"]
            lines.append(
                f"{name:<14s} {c['arm_name']:<18s} {w['base_median']:8.4f} {w['cand_median']:8.4f} "
                f"{w['t_ratio']:6.4f} [{lo:.4f}, {hi:.4f}] {w['paired_median']:7.4f} {r['paired_median']:7.4f}"
            )
    return "\n".join(lines) + "\n"


def run_adoption(
    out: Path,
    *,
    smoke: bool = False,
    allow_any_host: bool = False,
    rounds: int | None = None,
    settle_s: str | None = None,
    prespin_ms: int | None = None,
    wikidata_counts: tuple[int, ...] | None = None,
    guard_corpora: tuple[str, ...] | None = None,
    guard_counts: tuple[int, ...] | None = None,
    prebuilt_base: Path | None = None,
    prebuilt_cand: Path | None = None,
    prebuilt_windows: Path | None = None,
) -> dict[str, Any]:
    """Execute the full frozen C6 forward-emission adoption study (or `--smoke` run)."""
    out = Path(out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    strict_i9 = not (smoke or allow_any_host)

    eff_rounds = rounds if rounds is not None else (2 if smoke else ROUNDS)
    eff_settle = settle_s if settle_s is not None else ("0" if smoke else SETTLE_S)
    eff_prespin = prespin_ms if prespin_ms is not None else (0 if smoke else PRESPIN_MS)
    eff_w_counts = (
        wikidata_counts if wikidata_counts is not None else ((1, 2) if smoke else WIKIDATA_COUNTS)
    )
    eff_guards = (
        guard_corpora
        if guard_corpora is not None
        else (("text", "runs") if smoke else GUARD_CORPORA)
    )
    eff_g_counts = guard_counts if guard_counts is not None else ((1, 2) if smoke else GUARD_COUNTS)
    eff_cells = tuple(("wikidata", k) for k in eff_w_counts) + tuple(
        (name, k) for name in eff_guards for k in eff_g_counts
    )

    if strict_i9:
        check_baseline_env()
        if output("git", "status", "--porcelain", "--untracked-files=no"):
            raise InvalidRun("recipe checkout has tracked changes")
        toolchain = output("rustc", "-Vv")
        if not toolchain.startswith("rustc 1.99.1 "):
            raise InvalidRun("registered toolchain requires Rust 1.99.1")
        if not REGISTRATION.exists():
            raise InvalidRun(f"missing frozen registration {REGISTRATION}")
        shutil.copyfile(REGISTRATION, out / "registration.md")
    else:
        toolchain = output("rustc", "-Vv")
        if REGISTRATION.exists():
            shutil.copyfile(REGISTRATION, out / "registration.md")

    (out / "toolchain.txt").write_text(toolchain + "\n", encoding="utf-8")
    host = check_host_i9(out, require_i9=strict_i9)
    pin_cpu = int(host["pin_cpu"])
    save(
        out / "provenance.json",
        {
            "head": output("git", "rev-parse", "HEAD"),
            "smoke": smoke,
            "strict_i9": strict_i9,
            "rounds": eff_rounds,
            "settle_s": eff_settle,
            "prespin_ms": eff_prespin,
            "wikidata_counts": list(eff_w_counts),
            "guard_corpora": list(eff_guards),
            "guard_counts": list(eff_g_counts),
        },
    )

    with tempfile.TemporaryDirectory(prefix="forward-adopt-") as td:
        tmp = Path(td)
        bins, bin_meta = build_arms(
            out,
            tmp,
            root=ROOT,
            prebuilt_base=prebuilt_base,
            prebuilt_cand=prebuilt_cand,
            prebuilt_windows=prebuilt_windows,
        )
        files, refs = prepare_inputs(
            out, tmp, bins, root=ROOT, smoke=smoke, guard_corpora=eff_guards
        )
        verify_parity(out, bins, files, refs, cells=eff_cells)

        # Record passive env facts via base ab binary.
        env_facts = output(str(bins["base_ab"]), "env", "--facts")
        (out / "env-facts.json").write_text(env_facts + "\n", encoding="utf-8")

        all_rows: list[dict[str, Any]] = []
        w_1t = tuple(k for k in eff_w_counts if k == 1)
        w_mt = tuple(k for k in eff_w_counts if k > 1)
        g_1t = tuple(k for k in eff_g_counts if k == 1)
        g_mt = tuple(k for k in eff_g_counts if k > 1)
        use_ms = strict_i9

        if w_1t:
            all_rows.extend(
                run_ab_group(
                    bins["base_ab"],
                    bins["cand_ab"],
                    out / "wikidata_1t.jsonl",
                    arm_name="mibz2-workers-lazy",
                    counts=w_1t,
                    corpus_paths=[files["wikidata"]],
                    rounds=eff_rounds,
                    pin_cpu=pin_cpu,
                    prespin_ms=eff_prespin,
                    settle_s=eff_settle,
                    max_busy=MAX_BUSY,
                    machine_state=use_ms,
                )
            )
        if w_mt:
            all_rows.extend(
                run_ab_group(
                    bins["base_ab"],
                    bins["cand_ab"],
                    out / "wikidata_mt.jsonl",
                    arm_name="mibz2",
                    counts=w_mt,
                    corpus_paths=[files["wikidata"]],
                    rounds=eff_rounds,
                    pin_cpu=pin_cpu,
                    prespin_ms=eff_prespin,
                    settle_s=eff_settle,
                    max_busy=MAX_BUSY,
                    machine_state=use_ms,
                )
            )
        if g_1t:
            all_rows.extend(
                run_ab_group(
                    bins["base_ab"],
                    bins["cand_ab"],
                    out / "guards_1t.jsonl",
                    arm_name="mibz2-workers-lazy",
                    counts=g_1t,
                    corpus_paths=[files[n] for n in eff_guards],
                    rounds=eff_rounds,
                    pin_cpu=pin_cpu,
                    prespin_ms=eff_prespin,
                    settle_s=eff_settle,
                    max_busy=MAX_BUSY,
                    machine_state=use_ms,
                )
            )
        if g_mt:
            all_rows.extend(
                run_ab_group(
                    bins["base_ab"],
                    bins["cand_ab"],
                    out / "guards_mt.jsonl",
                    arm_name="mibz2",
                    counts=g_mt,
                    corpus_paths=[files[n] for n in eff_guards],
                    rounds=eff_rounds,
                    pin_cpu=pin_cpu,
                    prespin_ms=eff_prespin,
                    settle_s=eff_settle,
                    max_busy=MAX_BUSY,
                    machine_state=use_ms,
                )
            )

        with (out / "runs.jsonl").open("w", encoding="utf-8") as fh:
            for row in all_rows:
                fh.write(json.dumps(row, allow_nan=False) + "\n")

        summary = score(
            all_rows,
            refs,
            cand_exe=bins["cand_ab"],
            rounds=eff_rounds,
            wikidata_counts=eff_w_counts,
            guard_corpora=eff_guards,
            guard_counts=eff_g_counts,
            distinct_binaries=bin_meta["distinct_binaries"],
            host_ok=True,
        )

    save(out / "summary.json", summary)
    txt = format_summary(summary)
    (out / "summary.txt").write_text(txt, encoding="utf-8")
    (out / "outcome.txt").write_text(
        f"{summary['verdict']}: {summary['reason']}\n", encoding="utf-8"
    )
    print(txt, end="")
    return summary


def score_dir(out: Path) -> dict[str, Any]:
    """Re-score an existing run directory cleanly from its archived JSONL/JSON artifacts."""
    out = Path(out).resolve()
    prov = json.loads((out / "provenance.json").read_text(encoding="utf-8"))
    bins = json.loads((out / "binaries.json").read_text(encoding="utf-8"))
    refs = json.loads((out / "inputs.json").read_text(encoding="utf-8"))
    # Merge 64-bit hashes from parity.jsonl into refs so timed hash checks run.
    parity_path = out / "parity.jsonl"
    if parity_path.exists():
        for line in parity_path.read_text(encoding="utf-8").splitlines():
            if line.strip():
                rec = json.loads(line)
                c = rec.get("corpus")
                if c in refs and rec.get("hash"):
                    refs[c]["hash"] = rec["hash"]
    rows = [
        json.loads(line)
        for line in (out / "runs.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    summary = score(
        rows,
        refs,
        cand_exe=bins["cand"]["path"],
        rounds=int(prov["rounds"]),
        wikidata_counts=tuple(prov["wikidata_counts"]),
        guard_corpora=tuple(prov["guard_corpora"]),
        guard_counts=tuple(prov["guard_counts"]),
        distinct_binaries=bool(bins.get("distinct_binaries", False)),
        host_ok=True,
    )
    save(out / "summary.json", summary)
    txt = format_summary(summary)
    (out / "summary.txt").write_text(txt, encoding="utf-8")
    (out / "outcome.txt").write_text(
        f"{summary['verdict']}: {summary['reason']}\n", encoding="utf-8"
    )
    print(txt, end="")
    return summary


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_run = sub.add_parser("run", help="Execute the full-guard C6 adoption run (or --smoke)")
    p_run.add_argument("out", type=Path, help="Output directory for artifacts")
    p_run.add_argument("--smoke", action="store_true", help="Compact local VM smoke test")
    p_run.add_argument("--allow-any-host", action="store_true", help="Allow non-i9 host")
    p_run.add_argument("--rounds", type=int, default=None, help="Override paired rounds")
    p_run.add_argument("--settle", type=str, default=None, help="Override ab --settle seconds")
    p_run.add_argument("--prespin-ms", type=int, default=None, help="Override prespin ms")
    p_run.add_argument("--base-ab", type=Path, default=None, help="Pre-built base ab binary")
    p_run.add_argument("--cand-ab", type=Path, default=None, help="Pre-built cand ab binary")
    p_run.add_argument("--windows-exe", type=Path, default=None, help="Pre-built windows binary")

    p_score = sub.add_parser("score", help="Re-score an archived run directory")
    p_score.add_argument("out", type=Path, help="Run directory containing runs.jsonl")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        if args.cmd == "run":
            res = run_adoption(
                args.out,
                smoke=args.smoke,
                allow_any_host=args.allow_any_host,
                rounds=args.rounds,
                settle_s=args.settle,
                prespin_ms=args.prespin_ms,
                prebuilt_base=args.base_ab,
                prebuilt_cand=args.cand_ab,
                prebuilt_windows=args.windows_exe,
            )
            return 1 if res["verdict"] == "VOID" else 0
        if args.cmd == "score":
            res = score_dir(args.out)
            return 1 if res["verdict"] == "VOID" else 0
    except InvalidRun as exc:
        print(f"VOID: {exc}", file=sys.stderr)
        if getattr(args, "out", None):
            out = Path(args.out)
            out.mkdir(parents=True, exist_ok=True)
            (out / "outcome.txt").write_text(f"VOID: {exc}\n", encoding="utf-8")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
