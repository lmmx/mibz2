#!/usr/bin/env python3
"""Frozen C1a diagnostic calibration probe for `block-size-route-i9-probe`
(Step 1 of `open-admission-block-size`): per-block Open vs Segmented across
bzip2 levels -1, -2, -3, -5, -9 (`wikidata`) and -1, -9 (`text`) using
`examples/route_census` with `ROUTE_FULL=90000` and both `ROUTE_SHADOW=1`
(Open first) and `ROUTE_SHADOW=first` (Segmented first).

Claim: frontier/claims/block-size-route-i9-probe.toml
Registration: docs/journal/2026-11-28-block-size-route-i9/registration.md

Usage:
    python3 bench/block_size_route.py run OUT_DIR [--smoke] [--rounds N] [--reps R] [--settle S]
    python3 bench/block_size_route.py score OUT_DIR
"""

from __future__ import annotations

import argparse
import bz2
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
from typing import Any

BENCH_DIR = Path(__file__).resolve().parent
if str(BENCH_DIR) not in sys.path:
    sys.path.insert(0, str(BENCH_DIR))

from mtf_fold import topology  # noqa: E402
import wikidata  # noqa: E402

ROOT = BENCH_DIR.parent
REGISTRATION = ROOT / "docs/journal/2026-11-28-block-size-route-i9/registration.md"

WIKIDATA_LEVELS = (1, 2, 3, 5, 9)
TEXT_LEVELS = (1, 9)
SHADOW_ORDERS = ("1", "first")  # "1" = Open first, Seg second; "first" = Seg first, Open second
ROUTE_FULL = 90_000
ROUNDS = 3
REPS = 2
SETTLE_S = "2"
CONFIRM_THRESHOLD = 1.05
MIN_BLOCKS_PER_SWEEP = 10

ROW_BINS: tuple[tuple[str, int, int | None, bool], ...] = (
    # (label, lo_inclusive, hi_exclusive, is_below_400k)
    ("<150k", 90_000, 150_000, True),
    ("150k..250k", 150_000, 250_000, True),
    ("250k..350k", 250_000, 350_000, True),
    ("350k..400k", 350_000, 400_000, True),
    ("400k..600k", 400_000, 600_000, False),
    (">=600k", 600_000, None, False),
)
SUB_400K_BINS = tuple(label for label, _, _, sub in ROW_BINS if sub)

RN_BINS: tuple[tuple[str, float, float | None], ...] = (
    ("<0.06", 0.0, 0.06),
    ("0.06-0.07", 0.06, 0.07),
    ("0.07-0.08", 0.07, 0.08),
    ("0.08-0.09", 0.08, 0.09),
    ("0.09-0.10", 0.09, 0.10),
    ("0.10-0.12", 0.10, 0.12),
    ("0.12-0.15", 0.12, 0.15),
    ("0.15-0.21", 0.15, 0.21),
    (">=0.21", 0.21, None),
)

CANDIDATE_N0 = (150_000, 250_000, 350_000, 400_000)
CANDIDATE_T0 = (0.06, 0.07, 0.075, 0.08, 0.09, 0.10, 0.12, 0.15)

RUSTFLAGS_ENV_VARS = ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS")
PROFILE_ENV_VARS = (
    "CARGO_PROFILE_RELEASE_LTO",
    "CARGO_PROFILE_RELEASE_CODEGEN_UNITS",
    "CARGO_PROFILE_RELEASE_OPT_LEVEL",
)


class InvalidRun(ValueError):
    """A registered prerequisite failed or the measurement cannot support a verdict."""


def _sanitize_json(obj: Any) -> Any:
    """Recursively replace non-finite floats with None before strict JSON serialization."""
    if isinstance(obj, float):
        return obj if math.isfinite(obj) else None
    if isinstance(obj, dict):
        return {str(k): _sanitize_json(v) for k, v in obj.items()}
    if isinstance(obj, (list, tuple)):
        return [_sanitize_json(v) for v in obj]
    return obj


def save(path: Path, obj: Any) -> None:
    path.write_text(
        json.dumps(_sanitize_json(obj), indent=2, allow_nan=False) + "\n",
        encoding="utf-8",
    )


def output(*args: str, cwd: Path = ROOT, env: dict[str, str] | None = None) -> str:
    return subprocess.check_output(args, cwd=cwd, text=True, env=env).strip()


def sha256_file(path: Path | str) -> str:
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def reference(path: Path | str) -> dict[str, Any]:
    """Independently decompress with C libbzip2 (`bz2.open`) and return exact byte length + SHA-256."""
    with bz2.open(path, "rb") as source:
        digest = hashlib.sha256()
        size = 0
        while buf := source.read(1 << 20):
            size += len(buf)
            digest.update(buf)
    return {"out": size, "sha256": digest.hexdigest()}


def check_baseline_env(env: dict[str, str] | None = None) -> None:
    """Enforce default baseline x86-64 codegen with no ambient RUSTFLAGS or profile overrides."""
    cur = os.environ if env is None else env
    for var in (*RUSTFLAGS_ENV_VARS, *PROFILE_ENV_VARS):
        val = cur.get(var, "").strip()
        if val:
            raise InvalidRun(f"non-empty {var}={val!r}; registered run requires baseline codegen")


def check_no_quota(facts: dict[str, Any]) -> None:
    """VOID under a CPU quota; supports both scalar and dict `ab env --facts/1` shapes."""
    if not isinstance(facts, dict) or "cpu_quota" not in facts:
        raise InvalidRun("registered run requires cpu_quota in env-facts.json")
    quota = facts["cpu_quota"]
    if quota is None:
        return
    if isinstance(quota, dict) and quota.get("state") == "unlimited" and quota.get("cpus") is None:
        return
    raise InvalidRun(f"registered run requires no CPU quota; cgroup quota is {quota!r}")


def check_host_i9(out: Path, require_i9: bool = True) -> dict[str, Any]:
    """Record lscpu + affinity and return core-0 pin CPU for single-thread calibration."""
    lscpu_txt = output("lscpu")
    (out / "lscpu.txt").write_text(lscpu_txt + "\n", encoding="utf-8")
    lscpu_e = output("lscpu", "-e")
    grid = out / "lscpu-e.txt"
    grid.write_text(lscpu_e + "\n", encoding="utf-8")
    cores = topology(str(grid))
    cpus = {cpu for core in cores for cpu in core}
    allowed = os.sched_getaffinity(0)
    if require_i9:
        if "i9-10900K" not in lscpu_txt:
            raise InvalidRun("workstation model differs from registered i9-10900K")
        if len(cores) != 10 or len(cpus) != 20 or cpus != allowed:
            raise InvalidRun(
                f"expected unrestricted 10-core/20-CPU i9 topology, got "
                f"{len(cores)} cores / {len(cpus)} CPUs / allowed={sorted(allowed)}"
            )
        pin_cpu = cores[0][0]
    else:
        allowed_sorted = sorted(allowed) if allowed else [0]
        pin_cpu = allowed_sorted[0]
    info = {
        "n_cores": len(cores),
        "n_cpus": len(cpus),
        "cpus_allowed": sorted(allowed),
        "pin_cpu": pin_cpu,
        "require_i9": require_i9,
    }
    save(out / "host.json", info)
    return info


def row_bin_for(n: int) -> str | None:
    for label, lo, hi, _ in ROW_BINS:
        if n >= lo and (hi is None or n < hi):
            return label
    return None


def rn_bin_for(rn: float) -> str | None:
    if not math.isfinite(rn) or rn < 0.0:
        return None
    for label, lo, hi in RN_BINS:
        if rn >= lo and (hi is None or rn < hi):
            return label
    return None


def build_binaries(
    out: Path,
    tmp: Path,
    *,
    root: Path = ROOT,
    prebuilt_route_census: Path | None = None,
    prebuilt_ab: Path | None = None,
    prebuilt_windows: Path | None = None,
) -> dict[str, Path]:
    """Build `route_census` (--features moonshot, no census), `ab`, and `windows` cleanly."""
    clean_env = {
        k: v for k, v in os.environ.items() if k not in (*RUSTFLAGS_ENV_VARS, *PROFILE_ENV_VARS)
    }
    bins: dict[str, Path] = {}
    need: list[str] = []
    if prebuilt_route_census is None:
        need.append("route_census")
    if prebuilt_ab is None:
        need.append("ab")
    if prebuilt_windows is None:
        need.append("windows")

    if need:
        tdir = tmp / "target-probe"
        cmd = ["cargo", "build", "--release", "--locked", "--features", "moonshot"]
        for ex in need:
            cmd.extend(["--example", ex])
        subprocess.run(
            cmd,
            cwd=root,
            check=True,
            env={**clean_env, "CARGO_TARGET_DIR": str(tdir)},
        )
        for ex in need:
            dst = tmp / f"bin-{ex}"
            shutil.copy2(tdir / "release/examples" / ex, dst)
            bins[ex] = dst

    if prebuilt_route_census is not None:
        bins["route_census"] = Path(prebuilt_route_census).resolve()
    if prebuilt_ab is not None:
        bins["ab"] = Path(prebuilt_ab).resolve()
    if prebuilt_windows is not None:
        bins["windows"] = Path(prebuilt_windows).resolve()

    save(
        out / "binaries.json",
        {
            name: {"path": str(path), "sha256": sha256_file(path)}
            for name, path in sorted(bins.items())
        },
    )
    return bins


def materialize_wikidata_x1(tmp: Path, windows_exe: Path, *, root: Path = ROOT) -> Path:
    """Materialize canonical wikidata x1 from `corpora/wikidata.bz2` or `origin/bench-data`."""
    spec = json.loads((root / "bench/wikidata.json").read_text(encoding="utf-8"))
    local = root / "corpora/wikidata.bz2"
    if local.exists() and sha256_file(local) == spec["corpus_sha256"]:
        return local

    win = tmp / "wikidata-windows"
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
    return wiki


def reencode_bz2(raw_path: Path, dst_path: Path, level: int) -> None:
    """Compress `raw_path` using C libbzip2 (`bz2.BZ2Compressor`) at `level` (1..9)."""
    comp = bz2.BZ2Compressor(level)
    with raw_path.open("rb") as src, dst_path.open("wb") as dst:
        while chunk := src.read(1 << 20):
            out = comp.compress(chunk)
            if out:
                dst.write(out)
        tail = comp.flush()
        if tail:
            dst.write(tail)


def prepare_inputs(
    out: Path,
    tmp: Path,
    bins: dict[str, Path],
    *,
    root: Path = ROOT,
    smoke: bool = False,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Prepare re-encoded `wikidata` (-1,-2,-3,-5,-9) and `text` (-1,-9) and verify SHA-256 parity."""
    gen_dir = tmp / "gen"
    gen_scale = "1" if smoke else "4"
    subprocess.run([str(bins["ab"]), "gen", str(gen_dir), gen_scale], check=True)
    text_src = gen_dir / "text.bz2"

    if smoke:
        wiki_src = text_src
    else:
        wiki_src = materialize_wikidata_x1(tmp, bins["windows"], root=root)

    raw_dir = tmp / "raw"
    raw_dir.mkdir(exist_ok=True)
    reenc_dir = tmp / "reencoded"
    reenc_dir.mkdir(exist_ok=True)

    sources = {"wikidata": wiki_src, "text": text_src}
    canonical: dict[str, dict[str, Any]] = {}
    raw_paths: dict[str, Path] = {}
    for corpus, src_bz2 in sources.items():
        raw_p = raw_dir / f"{corpus}.bin"
        digest = hashlib.sha256()
        size = 0
        with bz2.open(src_bz2, "rb") as src, raw_p.open("wb") as dst:
            while buf := src.read(1 << 20):
                dst.write(buf)
                digest.update(buf)
                size += len(buf)
        canonical[corpus] = {
            "out": size,
            "sha256": digest.hexdigest(),
            "source_bz2": str(src_bz2),
            "source_compressed_sha256": sha256_file(src_bz2),
        }
        raw_paths[corpus] = raw_p

    jobs: list[tuple[str, int, Path]] = []
    for lvl in WIKIDATA_LEVELS:
        jobs.append(("wikidata", lvl, reenc_dir / f"wikidata-L{lvl}.bz2"))
    for lvl in TEXT_LEVELS:
        jobs.append(("text", lvl, reenc_dir / f"text-L{lvl}.bz2"))

    with ThreadPoolExecutor(max_workers=min(len(jobs), os.cpu_count() or 4)) as pool:
        futs = [
            (corpus, lvl, dst, pool.submit(reencode_bz2, raw_paths[corpus], dst, lvl))
            for corpus, lvl, dst in jobs
        ]
        for _, _, _, fut in futs:
            fut.result()

    # Independently verify decoded SHA-256 parity of every re-encoded file via C libbzip2.
    items: list[dict[str, Any]] = []
    parity_rows: list[dict[str, Any]] = []
    for corpus, lvl, dst in jobs:
        ref = reference(dst)
        exp = canonical[corpus]
        ok = ref["out"] == exp["out"] and ref["sha256"] == exp["sha256"]
        parity_rows.append(
            {
                "corpus": corpus,
                "level": lvl,
                "file": dst.name,
                "compressed_bytes": dst.stat().st_size,
                "compressed_sha256": sha256_file(dst),
                "decoded_bytes": ref["out"],
                "decoded_sha256": ref["sha256"],
                "expected_bytes": exp["out"],
                "expected_sha256": exp["sha256"],
                "ok": ok,
            }
        )
        if not ok:
            raise InvalidRun(f"re-encoded parity mismatch on {dst.name}: got {ref}, expected {exp}")
        items.append(
            {
                "corpus": corpus,
                "level": lvl,
                "path": dst,
                "file": dst.name,
                "decoded_bytes": ref["out"],
                "decoded_sha256": ref["sha256"],
            }
        )

    parity = {"all_ok": all(r["ok"] for r in parity_rows), "checks": parity_rows}
    save(out / "inputs.json", {"canonical": canonical, "smoke": smoke})
    save(out / "parity.json", parity)
    return items, parity


def parse_route_census_output(stdout_txt: str, stderr_txt: str) -> dict[str, Any]:
    """Parse `examples/route_census` stdout table row and stderr per-block shadow lines."""
    summary_row: dict[str, Any] | None = None
    for raw in stdout_txt.splitlines():
        line = raw.strip()
        if not line or line.startswith("file\t"):
            continue
        parts = line.split("\t")
        if len(parts) < 20:
            continue
        probe_ms = float(parts[16])
        seg_ms = float(parts[17])
        lazy_ms = float(parts[18])
        ratio = float(parts[19])
        if not all(math.isfinite(x) and x >= 0.0 for x in (probe_ms, seg_ms, lazy_ms, ratio)):
            raise InvalidRun(f"non-finite route_census stdout row: {line!r}")
        summary_row = {
            "file": parts[0],
            "blocks": int(parts[1]),
            "full": int(parts[2]),
            "column": int(parts[3]),
            "open": int(parts[4]),
            "runs_not_open": int(parts[5]),
            "bytes": int(parts[12]),
            "inv_ms_open": float(parts[13]),
            "inv_ms_runs_not_open": float(parts[14]),
            "inv_ms_column": float(parts[15]),
            "probe_ms": probe_ms,
            "segmented_ms": seg_ms,
            "lazy_ms": lazy_ms,
            "probe_over_segmented": ratio,
        }
    if summary_row is None:
        raise InvalidRun(f"missing route_census stdout summary row: {stdout_txt!r}")

    blocks: list[dict[str, Any]] = []
    for raw in stderr_txt.splitlines():
        line = raw.strip()
        if not line:
            continue
        parts = line.split("\t")
        if len(parts) < 9:
            continue
        route = parts[1]
        if route not in ("Open", "RunsNotOpen", "Column"):
            continue
        n = int(parts[2])
        rn = float(parts[3])
        open_ms = float(parts[4])
        seg_ms = float(parts[5])
        m_est = int(parts[6])
        thick = int(parts[7])
        cap_m = int(parts[8])
        if n <= 0 or not all(math.isfinite(x) and x >= 0.0 for x in (rn, open_ms, seg_ms)):
            raise InvalidRun(f"invalid route_census stderr block line: {line!r}")
        blocks.append(
            {
                "route": route,
                "n": n,
                "rn": rn,
                "open_ms": open_ms,
                "seg_ms": seg_ms,
                "m_est": m_est,
                "thick": thick,
                "cap_m": cap_m,
            }
        )

    return {"summary": summary_row, "blocks": blocks}


def aggregate_sweep_blocks(blocks: list[dict[str, Any]]) -> dict[str, Any]:
    """Aggregate Open-routed full blocks into row-count bins and 2D `(row_bin, rn_bin)` cells."""
    row_bins: dict[str, dict[str, Any]] = {}
    grid: dict[str, dict[str, dict[str, Any]]] = {}
    open_blocks = 0
    open_wins = 0
    open_ms_total = 0.0
    seg_ms_total = 0.0
    # Also pre-aggregate per (N0, t0) candidate rule savings for fast exact rule evaluation.
    rule_eval: dict[str, dict[str, float]] = {}
    oracle_ms_total = 0.0

    for n0 in CANDIDATE_N0:
        for t0 in CANDIDATE_T0:
            rule_eval[f"{n0}:{t0:.3f}"] = {"rule_ms": 0.0, "switched_blocks": 0.0}

    for b in blocks:
        if b["route"] != "Open" or b["n"] < ROUTE_FULL:
            continue
        rb = row_bin_for(b["n"])
        rnb = rn_bin_for(b["rn"])
        if rb is None or rnb is None:
            continue
        o_ms = float(b["open_ms"])
        s_ms = float(b["seg_ms"])
        win = 1 if o_ms < s_ms else 0

        open_blocks += 1
        open_wins += win
        open_ms_total += o_ms
        seg_ms_total += s_ms
        oracle_ms_total += min(o_ms, s_ms)

        acc = row_bins.setdefault(rb, {"blocks": 0, "open_wins": 0, "open_ms": 0.0, "seg_ms": 0.0})
        acc["blocks"] += 1
        acc["open_wins"] += win
        acc["open_ms"] += o_ms
        acc["seg_ms"] += s_ms

        g_row = grid.setdefault(rb, {})
        g_cell = g_row.setdefault(rnb, {"blocks": 0, "open_wins": 0, "open_ms": 0.0, "seg_ms": 0.0})
        g_cell["blocks"] += 1
        g_cell["open_wins"] += win
        g_cell["open_ms"] += o_ms
        g_cell["seg_ms"] += s_ms

        for n0 in CANDIDATE_N0:
            for t0 in CANDIDATE_T0:
                key = f"{n0}:{t0:.3f}"
                if b["n"] < n0 and b["rn"] >= t0:
                    rule_eval[key]["rule_ms"] += s_ms
                    rule_eval[key]["switched_blocks"] += 1.0
                else:
                    rule_eval[key]["rule_ms"] += o_ms

    return {
        "open_blocks": open_blocks,
        "open_wins": open_wins,
        "open_ms": open_ms_total,
        "seg_ms": seg_ms_total,
        "oracle_ms": oracle_ms_total,
        "row_bins": row_bins,
        "grid": grid,
        "rule_eval": rule_eval,
    }


def run_one_sweep(
    route_census_bin: Path,
    item: dict[str, Any],
    *,
    shadow_order: str,
    reps: int,
    pin_cpu: int,
    require_taskset: bool,
) -> dict[str, Any]:
    """Execute one pinned `examples/route_census` sweep and return parsed aggregates."""
    cmd: list[str] = []
    if shutil.which("taskset"):
        cmd.extend(["taskset", "-c", str(pin_cpu)])
    elif require_taskset:
        raise InvalidRun("registered i9 run requires `taskset` for core-0 pinning")
    cmd.extend([str(route_census_bin), str(item["path"]), str(reps)])

    env = {
        **os.environ,
        "ROUTE_FULL": str(ROUTE_FULL),
        "ROUTE_SHADOW": shadow_order,
    }
    proc = subprocess.run(cmd, capture_output=True, text=True, check=True, env=env)
    parsed = parse_route_census_output(proc.stdout, proc.stderr)
    if parsed["summary"]["bytes"] != item["decoded_bytes"]:
        raise InvalidRun(
            f"route_census byte count mismatch on {item['file']}: "
            f"{parsed['summary']['bytes']} != {item['decoded_bytes']}"
        )
    agg = aggregate_sweep_blocks(parsed["blocks"])
    return {
        "corpus": item["corpus"],
        "level": item["level"],
        "file": item["file"],
        "shadow_order": shadow_order,
        "reps": reps,
        "stdout": parsed["summary"],
        **agg,
    }


def build_sweep_plan(
    items: list[dict[str, Any]],
    rounds: int,
) -> list[tuple[int, str, dict[str, Any]]]:
    """Alternate shadow order across rounds (`1, first` on even; `first, 1` on odd)."""
    plan: list[tuple[int, str, dict[str, Any]]] = []
    for r in range(rounds):
        orders = SHADOW_ORDERS if r % 2 == 0 else tuple(reversed(SHADOW_ORDERS))
        for order in orders:
            ordered_items = items if r % 2 == 0 else list(reversed(items))
            for item in ordered_items:
                plan.append((r, order, item))
    return plan


def score(
    sweeps: list[dict[str, Any]],
    *,
    expected_rounds: int = ROUNDS,
    valid: bool = True,
    void_reasons: list[str] | None = None,
) -> dict[str, Any]:
    """Score pooled Open/Seg across row-count bins and evaluate the frozen C1a decision rule."""
    reasons = list(void_reasons or [])
    if not valid and not reasons:
        reasons.append("run marked invalid")

    # Verify expected sweep completeness.
    expected_pairs = [("wikidata", lv) for lv in WIKIDATA_LEVELS] + [
        ("text", lv) for lv in TEXT_LEVELS
    ]
    counts_by_key: dict[tuple[str, int, str], int] = {}
    for sw in sweeps:
        key = (str(sw["corpus"]), int(sw["level"]), str(sw["shadow_order"]))
        counts_by_key[key] = counts_by_key.get(key, 0) + 1

    for corpus, lv in expected_pairs:
        for order in SHADOW_ORDERS:
            got = counts_by_key.get((corpus, lv, order), 0)
            if got != expected_rounds:
                reasons.append(
                    f"incomplete sweeps for {corpus} -L{lv} shadow={order}: "
                    f"got {got}, expected {expected_rounds}"
                )

    # Aggregate per-(corpus, level), per-(corpus, row_bin), and 2D (corpus, row_bin, rn_bin).
    by_level: dict[str, dict[str, Any]] = {}
    by_row_bin: dict[str, dict[str, dict[str, Any]]] = {"wikidata": {}, "text": {}}
    grid_out: dict[str, dict[str, dict[str, dict[str, Any]]]] = {"wikidata": {}, "text": {}}
    rule_agg: dict[str, dict[str, dict[str, float]]] = {"wikidata": {}, "text": {}}

    for sw in sweeps:
        corpus = str(sw["corpus"])
        lv = int(sw["level"])
        order = str(sw["shadow_order"])
        lv_key = f"{corpus}-L{lv}"
        lv_acc = by_level.setdefault(
            lv_key,
            {
                "corpus": corpus,
                "level": lv,
                "sweeps": 0,
                "open_blocks": 0,
                "open_wins": 0,
                "open_ms": 0.0,
                "seg_ms": 0.0,
                "oracle_ms": 0.0,
                "by_order": {
                    "1": {"open_ms": 0.0, "seg_ms": 0.0, "sweeps": 0},
                    "first": {"open_ms": 0.0, "seg_ms": 0.0, "sweeps": 0},
                },
                "probe_over_seg_whole": [],
            },
        )
        lv_acc["sweeps"] += 1
        lv_acc["open_blocks"] += int(sw["open_blocks"])
        lv_acc["open_wins"] += int(sw["open_wins"])
        lv_acc["open_ms"] += float(sw["open_ms"])
        lv_acc["seg_ms"] += float(sw["seg_ms"])
        lv_acc["oracle_ms"] += float(sw["oracle_ms"])
        lv_acc["by_order"][order]["open_ms"] += float(sw["open_ms"])
        lv_acc["by_order"][order]["seg_ms"] += float(sw["seg_ms"])
        lv_acc["by_order"][order]["sweeps"] += 1
        lv_acc["probe_over_seg_whole"].append(float(sw["stdout"]["probe_over_segmented"]))

        for rb, vals in (sw.get("row_bins") or {}).items():
            rb_acc = by_row_bin.setdefault(corpus, {}).setdefault(
                rb,
                {
                    "blocks": 0,
                    "open_wins": 0,
                    "open_ms": 0.0,
                    "seg_ms": 0.0,
                    "sweeps": 0,
                    "by_order": {
                        "1": {"open_ms": 0.0, "seg_ms": 0.0, "blocks": 0},
                        "first": {"open_ms": 0.0, "seg_ms": 0.0, "blocks": 0},
                    },
                },
            )
            rb_acc["blocks"] += int(vals["blocks"])
            rb_acc["open_wins"] += int(vals["open_wins"])
            rb_acc["open_ms"] += float(vals["open_ms"])
            rb_acc["seg_ms"] += float(vals["seg_ms"])
            rb_acc["sweeps"] += 1
            rb_acc["by_order"][order]["open_ms"] += float(vals["open_ms"])
            rb_acc["by_order"][order]["seg_ms"] += float(vals["seg_ms"])
            rb_acc["by_order"][order]["blocks"] += int(vals["blocks"])

        for rb, row in (sw.get("grid") or {}).items():
            for rnb, cell in row.items():
                g_acc = (
                    grid_out.setdefault(corpus, {})
                    .setdefault(rb, {})
                    .setdefault(rnb, {"blocks": 0, "open_wins": 0, "open_ms": 0.0, "seg_ms": 0.0})
                )
                g_acc["blocks"] += int(cell["blocks"])
                g_acc["open_wins"] += int(cell["open_wins"])
                g_acc["open_ms"] += float(cell["open_ms"])
                g_acc["seg_ms"] += float(cell["seg_ms"])

        for rk, rv in (sw.get("rule_eval") or {}).items():
            ra = rule_agg.setdefault(corpus, {}).setdefault(
                rk, {"rule_ms": 0.0, "open_ms": 0.0, "oracle_ms": 0.0, "switched_blocks": 0.0}
            )
            ra["rule_ms"] += float(rv["rule_ms"])
            ra["open_ms"] += float(sw["open_ms"])
            ra["oracle_ms"] += float(sw["oracle_ms"])
            ra["switched_blocks"] += float(rv["switched_blocks"])

    def safe_ratio(num: float, den: float) -> float | None:
        if den <= 0.0 or not (math.isfinite(num) and math.isfinite(den)):
            return None
        return num / den

    # Finalize ratios
    for lv_acc in by_level.values():
        lv_acc["open_over_seg"] = safe_ratio(lv_acc["open_ms"], lv_acc["seg_ms"])
        lv_acc["open_over_seg_after"] = safe_ratio(
            lv_acc["by_order"]["1"]["open_ms"], lv_acc["by_order"]["1"]["seg_ms"]
        )
        lv_acc["open_over_seg_first"] = safe_ratio(
            lv_acc["by_order"]["first"]["open_ms"], lv_acc["by_order"]["first"]["seg_ms"]
        )
        lv_acc["open_win_frac"] = (
            lv_acc["open_wins"] / lv_acc["open_blocks"] if lv_acc["open_blocks"] > 0 else None
        )
        lv_acc["oracle_saving_pct"] = (
            (lv_acc["oracle_ms"] / lv_acc["open_ms"] - 1.0) * 100.0
            if lv_acc["open_ms"] > 0.0
            else None
        )

    confirmed_bins: list[dict[str, Any]] = []
    corpus_sweeps = max(1, expected_rounds * len(SHADOW_ORDERS))
    for corpus, bins_map in by_row_bin.items():
        for rb, rb_acc in bins_map.items():
            ratio = safe_ratio(rb_acc["open_ms"], rb_acc["seg_ms"])
            rb_acc["open_over_seg"] = ratio
            rb_acc["open_over_seg_after"] = safe_ratio(
                rb_acc["by_order"]["1"]["open_ms"], rb_acc["by_order"]["1"]["seg_ms"]
            )
            rb_acc["open_over_seg_first"] = safe_ratio(
                rb_acc["by_order"]["first"]["open_ms"], rb_acc["by_order"]["first"]["seg_ms"]
            )
            rb_acc["open_win_frac"] = (
                rb_acc["open_wins"] / rb_acc["blocks"] if rb_acc["blocks"] > 0 else None
            )
            blocks_per_sweep = rb_acc["blocks"] / corpus_sweeps
            rb_acc["blocks_per_sweep"] = blocks_per_sweep
            if (
                rb in SUB_400K_BINS
                and blocks_per_sweep >= MIN_BLOCKS_PER_SWEEP
                and ratio is not None
                and ratio >= CONFIRM_THRESHOLD
            ):
                confirmed_bins.append(
                    {
                        "corpus": corpus,
                        "row_bin": rb,
                        "open_over_seg": ratio,
                        "open_over_seg_after": rb_acc["open_over_seg_after"],
                        "open_over_seg_first": rb_acc["open_over_seg_first"],
                        "blocks_per_sweep": blocks_per_sweep,
                    }
                )

    for corpus_grid in grid_out.values():
        for row in corpus_grid.values():
            for cell in row.values():
                cell["open_over_seg"] = safe_ratio(cell["open_ms"], cell["seg_ms"])

    # Rank candidate (N0, t0) rules on wikidata (and report best rule).
    best_rules: list[dict[str, Any]] = []
    for rk, ra in (rule_agg.get("wikidata") or {}).items():
        if ra["open_ms"] <= 0.0:
            continue
        n0_s, t0_s = rk.split(":")
        pct = (ra["rule_ms"] / ra["open_ms"] - 1.0) * 100.0
        best_rules.append(
            {
                "n0": int(n0_s),
                "t0": float(t0_s),
                "wikidata_inv_pct": pct,
                "switched_blocks": int(ra["switched_blocks"]),
            }
        )
    best_rules.sort(key=lambda x: x["wikidata_inv_pct"])

    if reasons:
        verdict = "VOID"
        reason = "; ".join(reasons)
    elif confirmed_bins:
        verdict = "CONFIRMED"
        top = max(confirmed_bins, key=lambda x: x["open_over_seg"])
        reason = (
            f"{len(confirmed_bins)} sub-400k bin(s) >= {CONFIRM_THRESHOLD:.2f}x "
            f"(max {top['corpus']} {top['row_bin']} Open/Seg={top['open_over_seg']:.4f})"
        )
    else:
        verdict = "STOP"
        sub_ratios = [
            by_row_bin[c][b]["open_over_seg"]
            for c in ("wikidata", "text")
            for b in SUB_400K_BINS
            if b in by_row_bin.get(c, {})
            and by_row_bin[c][b].get("blocks_per_sweep", 0.0) >= MIN_BLOCKS_PER_SWEEP
            and by_row_bin[c][b].get("open_over_seg") is not None
        ]
        max_r = max(sub_ratios) if sub_ratios else 0.0
        reason = (
            f"no sub-400k row bin reached Open/Seg >= {CONFIRM_THRESHOLD:.2f} "
            f"(max={max_r:.4f}); VM-L2 artefact on physical i9"
        )

    return {
        "verdict": verdict,
        "reason": reason,
        "confirm_threshold": CONFIRM_THRESHOLD,
        "min_blocks_per_sweep": MIN_BLOCKS_PER_SWEEP,
        "confirmed_bins": confirmed_bins,
        "by_level": by_level,
        "by_row_bin": by_row_bin,
        "grid": grid_out,
        "candidate_rules_top5": best_rules[:5],
    }


def format_summary(summary: dict[str, Any]) -> str:
    lines = [
        f"Verdict: {summary['verdict']}",
        f"Reason:  {summary['reason']}",
        "",
        "=== Per-Level Pooled Open/Seg (ROUTE_SHADOW=1 + first) ===",
        f"{'level':14s} {'blocks/sw':>9s} {'Open/Seg':>9s} {'after':>8s} {'first':>8s} {'OpenWin%':>9s} {'Oracle%':>8s}",
    ]
    for lv_key in sorted(summary["by_level"]):
        v = summary["by_level"][lv_key]
        bps = v["open_blocks"] / v["sweeps"] if v["sweeps"] else 0.0
        r = f"{v['open_over_seg']:.4f}" if v["open_over_seg"] is not None else "n/a"
        ra = f"{v['open_over_seg_after']:.4f}" if v["open_over_seg_after"] is not None else "n/a"
        rf = f"{v['open_over_seg_first']:.4f}" if v["open_over_seg_first"] is not None else "n/a"
        wf = f"{v['open_win_frac'] * 100:.1f}%" if v["open_win_frac"] is not None else "n/a"
        oc = f"{v['oracle_saving_pct']:+.2f}%" if v["oracle_saving_pct"] is not None else "n/a"
        lines.append(f"{lv_key:14s} {bps:9.1f} {r:>9s} {ra:>8s} {rf:>8s} {wf:>9s} {oc:>8s}")

    lines.append("")
    lines.append("=== Per-Row-Bin Pooled Open/Seg (Sub-400k Gate >= 1.05) ===")
    lines.append(
        f"{'corpus':9s} {'row_bin':11s} {'blocks/sw':>9s} {'Open/Seg':>9s} {'after':>8s} {'first':>8s} {'gate':>8s}"
    )
    for corpus in ("wikidata", "text"):
        bmap = summary["by_row_bin"].get(corpus, {})
        for label, _, _, sub in ROW_BINS:
            if label not in bmap:
                continue
            v = bmap[label]
            r = f"{v['open_over_seg']:.4f}" if v["open_over_seg"] is not None else "n/a"
            ra = (
                f"{v['open_over_seg_after']:.4f}" if v["open_over_seg_after"] is not None else "n/a"
            )
            rf = (
                f"{v['open_over_seg_first']:.4f}" if v["open_over_seg_first"] is not None else "n/a"
            )
            gate = "sub400k" if sub else "ref"
            lines.append(
                f"{corpus:9s} {label:11s} {v['blocks_per_sweep']:9.1f} {r:>9s} {ra:>8s} {rf:>8s} {gate:>8s}"
            )

    if summary.get("candidate_rules_top5"):
        lines.append("")
        lines.append("=== Top Candidate (N0, t0) Rules on Wikidata Inverse ===")
        for rule in summary["candidate_rules_top5"]:
            lines.append(
                f"  N0={rule['n0']:7d}  t0={rule['t0']:.3f}  "
                f"wikidata_inv={rule['wikidata_inv_pct']:+.2f}%  "
                f"switched_blocks={rule['switched_blocks']}"
            )
    return "\n".join(lines) + "\n"


def run_benchmark(
    out: Path,
    *,
    smoke: bool = False,
    rounds: int | None = None,
    reps: int | None = None,
    settle_s: str | None = None,
    prebuilt_route_census: Path | None = None,
    prebuilt_ab: Path | None = None,
    prebuilt_windows: Path | None = None,
) -> dict[str, Any]:
    out = Path(out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    strict_i9 = not smoke

    eff_rounds = rounds if rounds is not None else (1 if smoke else ROUNDS)
    eff_reps = reps if reps is not None else (1 if smoke else REPS)
    eff_settle = settle_s if settle_s is not None else ("0" if smoke else SETTLE_S)

    if strict_i9:
        overrides = {
            "--rounds": rounds,
            "--reps": reps,
            "--settle": settle_s,
            "--route-census-bin": prebuilt_route_census,
            "--ab-bin": prebuilt_ab,
            "--windows-exe": prebuilt_windows,
        }
        used = sorted(flag for flag, val in overrides.items() if val is not None)
        if used:
            raise InvalidRun(f"registered run takes no overrides; got {', '.join(used)}")
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
            "reps": eff_reps,
            "settle_s": eff_settle,
            "route_full": ROUTE_FULL,
            "wikidata_levels": list(WIKIDATA_LEVELS),
            "text_levels": list(TEXT_LEVELS),
            "shadow_orders": list(SHADOW_ORDERS),
            "pin_cpu": pin_cpu,
        },
    )

    with tempfile.TemporaryDirectory(prefix="block-size-route-") as td:
        tmp = Path(td)
        bins = build_binaries(
            out,
            tmp,
            root=ROOT,
            prebuilt_route_census=prebuilt_route_census,
            prebuilt_ab=prebuilt_ab,
            prebuilt_windows=prebuilt_windows,
        )
        env_facts = output(str(bins["ab"]), "env", "--facts")
        (out / "env-facts.json").write_text(env_facts + "\n", encoding="utf-8")
        if strict_i9:
            check_no_quota(json.loads(env_facts))

        items, _ = prepare_inputs(out, tmp, bins, root=ROOT, smoke=smoke)
        pause = float(eff_settle)
        sweeps: list[dict[str, Any]] = []
        with (out / "sweeps.jsonl").open("w", encoding="utf-8", buffering=1) as log:
            for r, order, item in build_sweep_plan(items, eff_rounds):
                if pause > 0:
                    time.sleep(pause)
                sw = run_one_sweep(
                    bins["route_census"],
                    item,
                    shadow_order=order,
                    reps=eff_reps,
                    pin_cpu=pin_cpu,
                    require_taskset=strict_i9,
                )
                row = {"round": r, **sw}
                log.write(json.dumps(_sanitize_json(row), allow_nan=False) + "\n")
                sweeps.append(row)

    summary = score_dir(out)
    return summary


def score_dir(out: Path) -> dict[str, Any]:
    """Re-score an archived run directory fail-closed from its persisted artifacts."""
    out = Path(out).resolve()
    void_reasons: list[str] = []

    prov_path = out / "provenance.json"
    if not prov_path.exists():
        void_reasons.append("missing provenance.json")
        prov: dict[str, Any] = {}
    else:
        prov = json.loads(prov_path.read_text(encoding="utf-8"))
        if prov.get("smoke", True):
            void_reasons.append("smoke run (unregistered)")
        if not prov.get("strict_i9", False):
            void_reasons.append("non-strict_i9 run")
        if prov.get("rounds") != ROUNDS:
            void_reasons.append(f"unregistered rounds={prov.get('rounds')} (expected {ROUNDS})")
        if prov.get("reps") != REPS:
            void_reasons.append(f"unregistered reps={prov.get('reps')} (expected {REPS})")
        if str(prov.get("settle_s")) != SETTLE_S:
            void_reasons.append(
                f"unregistered settle_s={prov.get('settle_s')!r} (expected {SETTLE_S!r})"
            )

    host_path = out / "host.json"
    if not host_path.exists():
        void_reasons.append("missing host.json")
    else:
        host = json.loads(host_path.read_text(encoding="utf-8"))
        if not host.get("require_i9", False):
            void_reasons.append("host was not verified as i9-10900K")
        if host.get("n_cores") != 10 or host.get("n_cpus") != 20:
            void_reasons.append(
                f"unexpected host topology {host.get('n_cores')}c/{host.get('n_cpus')}t"
            )

    env_path = out / "env-facts.json"
    if not env_path.exists():
        void_reasons.append("missing env-facts.json")
    else:
        try:
            check_no_quota(json.loads(env_path.read_text(encoding="utf-8")))
        except InvalidRun as exc:
            void_reasons.append(str(exc))

    parity_path = out / "parity.json"
    if not parity_path.exists():
        void_reasons.append("missing parity.json")
    else:
        parity = json.loads(parity_path.read_text(encoding="utf-8"))
        if not parity.get("all_ok", False):
            void_reasons.append("SHA-256 parity check failed")

    sweeps_path = out / "sweeps.jsonl"
    sweeps: list[dict[str, Any]] = []
    if not sweeps_path.exists():
        void_reasons.append("missing sweeps.jsonl")
    else:
        for line in sweeps_path.read_text(encoding="utf-8").splitlines():
            if line.strip():
                sweeps.append(json.loads(line))

    expected_rounds = int(prov.get("rounds") or ROUNDS)
    summary = score(
        sweeps,
        expected_rounds=expected_rounds,
        valid=not void_reasons,
        void_reasons=void_reasons,
    )
    save(out / "summary.json", summary)
    txt = format_summary(summary)
    (out / "summary.txt").write_text(txt, encoding="utf-8")
    (out / "outcome.txt").write_text(
        f"{summary['verdict']}: {summary['reason']}\n", encoding="utf-8"
    )
    print(txt, end="")
    return summary


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)

    r = sub.add_parser("run")
    r.add_argument("out", type=Path)
    r.add_argument("--smoke", action="store_true")
    r.add_argument("--rounds", type=int, default=None)
    r.add_argument("--reps", type=int, default=None)
    r.add_argument("--settle", default=None)
    r.add_argument("--route-census-bin", type=Path, default=None)
    r.add_argument("--ab-bin", type=Path, default=None)
    r.add_argument("--windows-exe", type=Path, default=None)

    s = sub.add_parser("score")
    s.add_argument("out", type=Path)

    args = ap.parse_args(argv)
    try:
        if args.cmd == "run":
            run_benchmark(
                args.out,
                smoke=args.smoke,
                rounds=args.rounds,
                reps=args.reps,
                settle_s=args.settle,
                prebuilt_route_census=args.route_census_bin,
                prebuilt_ab=args.ab_bin,
                prebuilt_windows=args.windows_exe,
            )
            return 0
        if args.cmd == "score":
            score_dir(args.out)
            return 0
    except InvalidRun as exc:
        print(f"VOID: {exc}", file=sys.stderr)
        if args.cmd == "run":
            args.out.mkdir(parents=True, exist_ok=True)
            (args.out / "outcome.txt").write_text(f"VOID: {exc}\n", encoding="utf-8")
        return 2
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
