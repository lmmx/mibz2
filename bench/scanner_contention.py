#!/usr/bin/env python3
"""Frozen diagnostic PMU benchmark for `scanner-contention-residual`:
per-thread hardware PMU counters on the pinned `mibz2-scan` thread alone (`base` vs
`forward-anchor-lanes` at the same commit, k = 1, 4, 8 workers on Wikidata x4).

Claim: frontier/claims/scanner-contention-residual.toml
Registration: docs/journal/2026-11-28-scanner-contention-residual/registration.md

Usage:
    python3 bench/scanner_contention.py run OUT_DIR [--smoke] [--rounds N] [--settle S]
    python3 bench/scanner_contention.py score OUT_DIR
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
import time
from typing import Any

BENCH_DIR = Path(__file__).resolve().parent
if str(BENCH_DIR) not in sys.path:
    sys.path.insert(0, str(BENCH_DIR))

from mtf_fold import paired_ratio_ci, topology  # noqa: E402
import wikidata  # noqa: E402

ROOT = BENCH_DIR.parent
REGISTRATION = ROOT / "docs/journal/2026-11-28-scanner-contention-residual/registration.md"

X4_SHA256 = "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"
ARMS = ("base", "lanes")
COUNTS = (1, 4, 8)
PARITY_COUNTS = (1, 8)
ROUNDS = 12
SETTLE_S = "2"
SEED = 1301
SCANNER_THREAD_PREFIX = "mibz2-scan"

PERF_EVENTS = (
    "cycles:u",
    "ref-cycles:u",
    "instructions:u",
    "cycle_activity.stalls_l3_miss:u",
    "LLC-load-misses:u",
)
PERF_EVENTS_FALLBACK = (
    "cycles",
    "ref-cycles",
    "instructions",
    "cycle_activity.stalls_l3_miss",
    "LLC-load-misses",
)
EVENT_KEY_MAP = {
    "cycles": "cycles",
    "ref-cycles": "ref_cycles",
    "instructions": "instructions",
    "cycle_activity.stalls_l3_miss": "stalls_l3_miss",
    "llc-load-misses": "llc_load_misses",
}
REQUIRED_PMU_KEYS = tuple(EVENT_KEY_MAP.values())
L3_EXCESS_FRACTION_MIN = 0.50

RUSTFLAGS_ENV_VARS = ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS")
PROFILE_ENV_VARS = (
    "CARGO_PROFILE_RELEASE_LTO",
    "CARGO_PROFILE_RELEASE_CODEGEN_UNITS",
    "CARGO_PROFILE_RELEASE_OPT_LEVEL",
)


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


def check_baseline_env(env: dict[str, str] | None = None) -> None:
    """Enforce default baseline x86-64 codegen with no ambient RUSTFLAGS or profile overrides."""
    cur = os.environ if env is None else env
    for var in (*RUSTFLAGS_ENV_VARS, *PROFILE_ENV_VARS):
        val = cur.get(var, "").strip()
        if val:
            raise InvalidRun(f"non-empty {var}={val!r}; registered run requires baseline codegen")


def check_no_quota(facts: dict[str, Any]) -> None:
    """VOID under a CPU quota; supports both scalar and dict `ab env --facts/1` shapes."""
    if "cpu_quota" not in facts:
        raise InvalidRun("registered run requires cpu_quota in env-facts.json")
    quota = facts["cpu_quota"]
    if quota is None:
        return
    if isinstance(quota, dict) and quota.get("state") == "unlimited" and quota.get("cpus") is None:
        return
    raise InvalidRun(f"registered run requires no CPU quota; cgroup quota is {quota!r}")


def check_host_i9(out: Path, require_i9: bool = True) -> dict[str, Any]:
    """Record lscpu + affinity and return `(scanner_cpu, worker_mask)` for isolated placement."""
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
        scanner_cpu = cores[0][0]
        worker_cpus = sorted(cpus.difference(cores[0]))
    else:
        allowed_sorted = sorted(allowed) if allowed else [0]
        scanner_cpu = allowed_sorted[0]
        other = [c for c in allowed_sorted if c != scanner_cpu]
        worker_cpus = other if other else allowed_sorted
    worker_mask = ",".join(map(str, worker_cpus))
    info = {
        "n_cores": len(cores),
        "n_cpus": len(cpus),
        "cpus_allowed": sorted(allowed),
        "scanner_cpu": scanner_cpu,
        "worker_mask": worker_mask,
        "require_i9": require_i9,
    }
    save(out / "host.json", info)
    return info


def build_arms(
    out: Path,
    tmp: Path,
    *,
    root: Path = ROOT,
    prebuilt_base: Path | None = None,
    prebuilt_lanes: Path | None = None,
    prebuilt_ab: Path | None = None,
    prebuilt_windows: Path | None = None,
) -> tuple[dict[str, Path], dict[str, Any]]:
    """Build `base` (`scan-oracle`) and `lanes` (`scan-oracle forward-anchor-lanes`)
    into isolated `CARGO_TARGET_DIR`s and enforce distinct binary SHA-256s.
    """
    clean_env = {k: v for k, v in os.environ.items() if k not in RUSTFLAGS_ENV_VARS}
    bins: dict[str, Path] = {}

    if prebuilt_base is None or prebuilt_ab is None or prebuilt_windows is None:
        t_base = tmp / "target-base"
        cmd = ["cargo", "build", "--release", "--locked", "--features", "scan-oracle"]
        if prebuilt_base is None:
            cmd.extend(["--example", "scanner_coalesced_run"])
        if prebuilt_ab is None:
            cmd.extend(["--example", "ab"])
        if prebuilt_windows is None:
            cmd.extend(["--example", "windows"])
        subprocess.run(
            cmd,
            cwd=root,
            check=True,
            env={**clean_env, "CARGO_TARGET_DIR": str(t_base)},
        )
        if prebuilt_base is None:
            dst = tmp / "base-run"
            shutil.copy2(t_base / "release/examples/scanner_coalesced_run", dst)
            bins["base"] = dst
        if prebuilt_ab is None:
            dst = tmp / "base-ab"
            shutil.copy2(t_base / "release/examples/ab", dst)
            bins["ab"] = dst
        if prebuilt_windows is None:
            dst = tmp / "base-windows"
            shutil.copy2(t_base / "release/examples/windows", dst)
            bins["windows"] = dst

    if prebuilt_base is not None:
        bins["base"] = Path(prebuilt_base).resolve()
    if prebuilt_ab is not None:
        bins["ab"] = Path(prebuilt_ab).resolve()
    if prebuilt_windows is not None:
        bins["windows"] = Path(prebuilt_windows).resolve()

    if prebuilt_lanes is None:
        t_lanes = tmp / "target-lanes"
        cmd = [
            "cargo",
            "build",
            "--release",
            "--locked",
            "--features",
            "scan-oracle forward-anchor-lanes",
            "--example",
            "scanner_coalesced_run",
        ]
        subprocess.run(
            cmd,
            cwd=root,
            check=True,
            env={**clean_env, "CARGO_TARGET_DIR": str(t_lanes)},
        )
        dst = tmp / "lanes-run"
        shutil.copy2(t_lanes / "release/examples/scanner_coalesced_run", dst)
        bins["lanes"] = dst
    else:
        bins["lanes"] = Path(prebuilt_lanes).resolve()

    base_sha = sha256_file(bins["base"])
    lanes_sha = sha256_file(bins["lanes"])
    distinct = base_sha != lanes_sha
    meta: dict[str, Any] = {
        "base": {
            "path": str(bins["base"]),
            "features": "scan-oracle",
            "sha256": base_sha,
        },
        "lanes": {
            "path": str(bins["lanes"]),
            "features": "scan-oracle forward-anchor-lanes",
            "sha256": lanes_sha,
        },
        "distinct_binaries": distinct,
    }
    if "ab" in bins:
        meta["ab"] = {"path": str(bins["ab"]), "sha256": sha256_file(bins["ab"])}
    if "windows" in bins:
        meta["windows"] = {"path": str(bins["windows"]), "sha256": sha256_file(bins["windows"])}
    save(out / "binaries.json", meta)
    if not distinct:
        raise InvalidRun(
            f"VOID: base and lanes binaries have identical SHA-256 ({base_sha}); "
            "forward-anchor-lanes did not differentiate the build"
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
    """Shallow-safely materialize `wikidata-x4` (from local cache or `--depth=1` bench-data)."""
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


def prepare_input(
    out: Path,
    tmp: Path,
    bins: dict[str, Path],
    *,
    root: Path = ROOT,
    smoke: bool = False,
) -> tuple[Path, dict[str, Any]]:
    """Prepare Wikidata x4 (or compact smoke input) and independently hash via C `libbzip2`."""
    if smoke:
        local_wiki = root / "corpora/wikidata.bz2"
        spec = json.loads((root / "bench/wikidata.json").read_text(encoding="utf-8"))
        if local_wiki.exists() and sha256_file(local_wiki) == spec["corpus_sha256"]:
            path = local_wiki
        else:
            gen_dir = tmp / "gen"
            subprocess.run([str(bins["ab"]), "gen", str(gen_dir), "1"], check=True)
            path = tmp / "wikidata.bz2"
            shutil.copy2(gen_dir / "text.bz2", path)
    else:
        path = materialize_wikidata_x4(tmp, bins["windows"], root=root)

    ref = reference(path)
    save(
        out / "inputs.json",
        {"wikidata": {**ref, "compressed_sha256": sha256_file(path), "path": str(path)}},
    )
    return path, ref


def normalize_event_name(raw_event: str) -> str | None:
    """Normalize a `perf stat` event name (handling `:u`, `cpu/<ev>/u`, etc.) to canonical key."""
    text = raw_event.strip().split(":", 1)[0].lower()
    candidates = [p for p in text.split("/") if p] if "/" in text else [text]
    for cand in candidates:
        for pat, key in EVENT_KEY_MAP.items():
            if cand == pat.lower():
                return key
    return None


def parse_perf_scanner_csv(
    text: str,
    *,
    scanner_cpu: int | None = None,
    thread_prefix: str = SCANNER_THREAD_PREFIX,
) -> dict[str, Any]:
    """Parse `perf stat -C <scanner_cpu> -x,` (or prefixed `CPU<n>` / `mibz2-scan-*`) CSV output."""
    scopes: set[str] = set()
    counts: dict[str, float] = {}
    pct_running: dict[str, float] = {}
    expected_cpu = f"CPU{scanner_cpu}" if scanner_cpu is not None else None

    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split(",")
        if len(parts) < 3:
            continue

        key2 = normalize_event_name(parts[2])
        key3 = normalize_event_name(parts[3]) if len(parts) >= 4 else None
        if key2 is not None:
            scope = expected_cpu or "cpu"
            val_str = parts[0].strip()
            event_str = parts[2].strip()
            pct_str = parts[4].strip() if len(parts) >= 5 else ""
            key = key2
        elif key3 is not None:
            scope = parts[0].strip()
            if not (
                scope.startswith(f"{thread_prefix}-")
                or (expected_cpu is not None and scope == expected_cpu)
                or (expected_cpu is None and scope.startswith("CPU"))
            ):
                continue
            val_str = parts[1].strip()
            event_str = parts[3].strip()
            pct_str = parts[5].strip() if len(parts) >= 6 else ""
            key = key3
        else:
            continue

        scopes.add(scope)
        if val_str.startswith("<") or not val_str:
            raise InvalidRun(f"PMU event {event_str!r} on {scope} not counted: {val_str!r}")
        try:
            val = float(val_str)
        except ValueError as exc:
            raise InvalidRun(f"invalid PMU value {val_str!r} for {event_str!r}") from exc
        if not math.isfinite(val) or val < 0:
            raise InvalidRun(f"non-finite or negative PMU value {val!r} for {event_str!r}")
        counts[key] = val
        if pct_str:
            try:
                pct_running[key] = float(pct_str)
            except ValueError:
                pass

    if len(scopes) != 1:
        raise InvalidRun(
            f"expected exactly 1 scanner PMU scope in perf CSV, found {sorted(scopes)}"
        )
    missing = [k for k in REQUIRED_PMU_KEYS if k not in counts]
    if missing:
        raise InvalidRun(f"missing required PMU event(s) on scanner CPU: {', '.join(missing)}")
    if counts["cycles"] <= 0 or counts["ref_cycles"] <= 0:
        raise InvalidRun(f"non-positive cycles/ref_cycles in perf CSV: {counts!r}")

    return {
        "scanner_scope": next(iter(scopes)),
        **counts,
        "pct_running": pct_running,
        "clock_ratio": counts["cycles"] / counts["ref_cycles"],
        "ipc": counts["instructions"] / counts["cycles"],
        "l3_stall_share": counts["stalls_l3_miss"] / counts["cycles"],
    }


parse_perf_per_thread_csv = parse_perf_scanner_csv


def build_perf_stat_cmd(
    perf_cmd: str,
    scanner_cpu: int,
    perf_csv: Path,
    events: tuple[str, ...],
    inner: list[str],
) -> list[str]:
    """Construct `perf stat -C <scanner_cpu> -x, -o <csv> -e <events> -- <inner>`."""
    return [
        perf_cmd,
        "stat",
        "-C",
        str(scanner_cpu),
        "-x,",
        "-o",
        str(perf_csv),
        "-e",
        ",".join(events),
        "--",
        *inner,
    ]


def preflight_perf(
    out: Path,
    scanner_cpu: int = 0,
    perf_cmd: str = "perf",
) -> tuple[str, ...]:
    """Verify `perf stat -C <scanner_cpu>` counts all 5 required PMU events on a pinned workload.

    Tries user-space `:u` events first (excluding kernel IRQs on `scanner_cpu`), falling back
    to plain event names if a PMU alias rejects `:u` under `perf_event_paranoid == 0`.
    """
    inner = [
        "taskset",
        "-c",
        str(scanner_cpu),
        sys.executable,
        "-c",
        "sum(range(200000))",
    ]
    errors: list[str] = []
    for idx, events in enumerate((PERF_EVENTS, PERF_EVENTS_FALLBACK)):
        csv_path = out / f"perf-preflight-{idx}.csv"
        cmd = build_perf_stat_cmd(perf_cmd, scanner_cpu, csv_path, events, inner)
        done = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        csv_text = csv_path.read_text(encoding="utf-8") if csv_path.exists() else ""
        (out / "perf-preflight.txt").write_text(
            f"$ {' '.join(cmd)}\n" + done.stderr + done.stdout + "\n" + csv_text,
            encoding="utf-8",
        )
        if done.returncode != 0:
            errors.append(f"events[{idx}] rc={done.returncode}: {done.stderr.strip()}")
            csv_path.unlink(missing_ok=True)
            continue
        try:
            parse_perf_scanner_csv(csv_text or done.stderr, scanner_cpu=scanner_cpu)
        except InvalidRun as exc:
            errors.append(f"events[{idx}]: {exc}")
            csv_path.unlink(missing_ok=True)
            continue
        csv_path.unlink(missing_ok=True)
        return events
    raise InvalidRun("perf preflight failed: " + " | ".join(errors))


def verify_parity(
    out: Path,
    bins: dict[str, Path],
    corpus_path: Path,
    ref: dict[str, Any],
    *,
    scanner_cpu: int,
    worker_mask: str,
    arms: tuple[str, ...] = ARMS,
    parity_counts: tuple[int, ...] = PARITY_COUNTS,
) -> list[dict[str, Any]]:
    """Run untimed `check` parity piped to `sha256sum` against C `libbzip2` for both arms."""
    records: list[dict[str, Any]] = []
    with (out / "parity.jsonl").open("w", encoding="utf-8", buffering=1) as log:
        for arm in arms:
            for k in parity_counts:
                cmd = [
                    "taskset",
                    "-c",
                    worker_mask,
                    str(bins[arm]),
                    str(corpus_path),
                    str(k),
                    str(scanner_cpu),
                    "real",
                    "check",
                ]
                done = subprocess.run(
                    ["bash", "-o", "pipefail", "-c", '"$@" | sha256sum', "parity-check", *cmd],
                    capture_output=True,
                    text=True,
                    timeout=300,
                )
                if done.returncode != 0:
                    raise InvalidRun(f"parity check failed on {arm}/k={k}: {done.stderr.strip()}")
                got_sha = (done.stdout.split()[:1] or [""])[0]
                if got_sha != ref["sha256"]:
                    raise InvalidRun(
                        f"parity SHA-256 mismatch on {arm}/k={k}: {got_sha} != {ref['sha256']}"
                    )
                row = json.loads(done.stderr.strip())
                if row.get("out") != ref["out"] or int(row.get("scanned", 0)) <= 0:
                    raise InvalidRun(f"parity byte/block mismatch on {arm}/k={k}: {row!r}")
                if row.get("scanner_cpu") != scanner_cpu or row.get("bmi2") is not True:
                    raise InvalidRun(f"scanner pin or BMI2 inactive on {arm}/k={k}: {row!r}")
                scanned = int(row["scanned"])
                if "scanned" in ref and ref["scanned"] != scanned:
                    raise InvalidRun(
                        f"scanned block count mismatch on {arm}/k={k}: {ref['scanned']} != {scanned}"
                    )
                ref["scanned"] = scanned
                rec = {"arm": arm, "k": k, "sha256": got_sha, **row}
                log.write(json.dumps(rec, allow_nan=False) + "\n")
                records.append(rec)
    return records


def build_plan(
    arms: tuple[str, ...] = ARMS,
    counts: tuple[int, ...] = COUNTS,
    rounds: int = ROUNDS,
    *,
    seed: int = SEED,
) -> list[tuple[int, str, int]]:
    """Build an interleaved schedule shuffling all `(arm, k)` cells within every round."""
    rng = random.Random(seed)
    cells = [(arm, k) for arm in arms for k in counts]
    schedule: list[tuple[int, str, int]] = []
    for r in range(rounds):
        order = list(cells)
        rng.shuffle(order)
        for arm, k in order:
            schedule.append((r, arm, k))
    return schedule


def _stub_pmu_sample(arm: str, k: int, wall_s: float) -> dict[str, Any]:
    """Deterministic synthetic PMU values used ONLY under `--smoke --stub-pmu` on VMs lacking PMU."""
    ref_cycles = max(1.0, wall_s * 3.7e9)
    clock_ratio = 1.427 if k == 1 else 1.327
    cycles = ref_cycles * clock_ratio * (1.03 if (arm == "lanes" and k == 8) else 1.0)
    stalls_l3 = cycles * (0.14 if (arm == "lanes" and k == 8) else 0.11)
    return {
        "scanner_scope": "CPU0",
        "cycles": cycles,
        "ref_cycles": ref_cycles,
        "instructions": cycles * 1.65,
        "stalls_l3_miss": stalls_l3,
        "llc_load_misses": stalls_l3 / 120.0,
        "pct_running": {ev: 100.0 for ev in REQUIRED_PMU_KEYS},
        "clock_ratio": cycles / ref_cycles,
        "ipc": 1.65,
        "l3_stall_share": stalls_l3 / cycles,
        "stub_pmu": True,
    }


def measure_sample(
    exe: Path,
    corpus_path: Path,
    ref: dict[str, Any],
    *,
    arm: str,
    k: int,
    scanner_cpu: int,
    worker_mask: str,
    perf_cmd: str = "perf",
    events: tuple[str, ...] = PERF_EVENTS,
    stub_pmu: bool = False,
    tmp: Path,
) -> dict[str, Any]:
    """Run one timed decode under `perf stat -C <scanner_cpu>` and extract scanner CPU counters."""
    inner = [
        "taskset",
        "-c",
        worker_mask,
        str(exe),
        str(corpus_path),
        str(k),
        str(scanner_cpu),
        "real",
        "time",
    ]
    if stub_pmu:
        done = subprocess.run(
            inner,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
            timeout=300,
        )
        if done.returncode != 0:
            raise InvalidRun(f"sample failed on {arm}/k={k}: {done.stderr.strip()}")
        sample = json.loads(done.stderr.strip())
        pmu = _stub_pmu_sample(arm, k, float(sample["wall_s"]))
    else:
        perf_csv = tmp / f"perf-{arm}-k{k}-{time.time_ns()}.csv"
        cmd = build_perf_stat_cmd(perf_cmd, scanner_cpu, perf_csv, events, inner)
        done = subprocess.run(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
            timeout=300,
        )
        if done.returncode != 0:
            raise InvalidRun(f"perf sample failed on {arm}/k={k}: {done.stderr.strip()}")
        sample = json.loads(done.stderr.strip())
        pmu_text = perf_csv.read_text(encoding="utf-8")
        perf_csv.unlink(missing_ok=True)
        pmu = parse_perf_scanner_csv(pmu_text, scanner_cpu=scanner_cpu)

    if sample.get("out") != ref["out"]:
        raise InvalidRun(f"output length mismatch on {arm}/k={k}")
    if "scanned" in ref and sample.get("scanned") != ref["scanned"]:
        raise InvalidRun(f"scanned block mismatch on {arm}/k={k}")
    if sample.get("scanner_cpu") != scanner_cpu or sample.get("bmi2") is not True:
        raise InvalidRun(f"scanner pin or BMI2 inactive on {arm}/k={k}")
    wall = sample.get("wall_s")
    rss = sample.get("peak_kib")
    if not (isinstance(wall, (int, float)) and math.isfinite(wall) and wall > 0):
        raise InvalidRun(f"invalid wall_s {wall!r} on {arm}/k={k}")
    if not (isinstance(rss, (int, float)) and math.isfinite(rss) and rss > 0):
        raise InvalidRun(f"invalid peak_kib {rss!r} on {arm}/k={k}")

    return {
        **sample,
        "pmu": pmu,
    }


def score(
    rows: list[dict[str, Any]],
    ref: dict[str, Any],
    *,
    rounds: int = ROUNDS,
    arms: tuple[str, ...] = ARMS,
    counts: tuple[int, ...] = COUNTS,
    distinct_binaries: bool = True,
    host_ok: bool = True,
) -> dict[str, Any]:
    """Score per-thread `mibz2-scan` PMU rows against the frozen `scanner-contention-residual` rule."""
    if not host_ok:
        return {"verdict": "VOID", "reason": "host/affinity/quota prerequisite failed"}
    if not distinct_binaries:
        return {"verdict": "VOID", "reason": "base and lanes binaries have identical SHA-256"}

    expected_cells = tuple((arm, k) for arm in arms for k in counts)
    expected_set = set(expected_cells)
    by_key: dict[tuple[str, int, int], dict[str, Any]] = {}

    for row in rows:
        arm = str(row["arm"])
        k = int(row["k"])
        r = int(row["round"])
        if (arm, k) not in expected_set:
            return {"verdict": "VOID", "reason": f"unexpected cell {(arm, k)}"}
        if r < 0 or r >= rounds:
            return {"verdict": "VOID", "reason": f"out-of-range round {r}"}
        key = (arm, k, r)
        if key in by_key:
            return {"verdict": "VOID", "reason": f"duplicate row for {key}"}
        if row.get("out") != ref["out"]:
            return {"verdict": "VOID", "reason": f"output length mismatch on {key}"}
        if "scanned" in ref and row.get("scanned") != ref["scanned"]:
            return {"verdict": "VOID", "reason": f"scanned count mismatch on {key}"}
        pmu = row.get("pmu")
        if not isinstance(pmu, dict):
            return {"verdict": "VOID", "reason": f"missing pmu dict on {key}"}
        for pk in REQUIRED_PMU_KEYS:
            val = pmu.get(pk)
            if not (isinstance(val, (int, float)) and math.isfinite(val) and val >= 0):
                return {"verdict": "VOID", "reason": f"invalid PMU {pk}={val!r} on {key}"}
        if pmu["cycles"] <= 0 or pmu["ref_cycles"] <= 0:
            return {"verdict": "VOID", "reason": f"non-positive cycles/ref_cycles on {key}"}
        wall = row.get("wall_s")
        if not (isinstance(wall, (int, float)) and math.isfinite(wall) and wall > 0):
            return {"verdict": "VOID", "reason": f"invalid wall_s {wall!r} on {key}"}
        by_key[key] = row

    expected_total = len(expected_cells) * rounds
    if len(by_key) != expected_total:
        return {
            "verdict": "VOID",
            "reason": f"incomplete grid: got {len(by_key)} rows, expected {expected_total}",
        }

    cells_out: dict[str, Any] = {}
    for arm, k in expected_cells:
        grp = [by_key[(arm, k, r)] for r in range(rounds)]
        cycles = [float(x["pmu"]["cycles"]) for x in grp]
        ref_cycles = [float(x["pmu"]["ref_cycles"]) for x in grp]
        insns = [float(x["pmu"]["instructions"]) for x in grp]
        stalls = [float(x["pmu"]["stalls_l3_miss"]) for x in grp]
        llc = [float(x["pmu"]["llc_load_misses"]) for x in grp]
        walls = [float(x["wall_s"]) for x in grp]
        rss = [float(x["peak_kib"]) for x in grp]
        clocks = [c / rc for c, rc in zip(cycles, ref_cycles, strict=True)]
        ipcs = [i / c for i, c in zip(insns, cycles, strict=True)]
        l3_shares = [s / c for s, c in zip(stalls, cycles, strict=True)]
        cells_out[f"{arm}/{k}"] = {
            "arm": arm,
            "k": k,
            "n": rounds,
            "cycles_median": statistics.median(cycles),
            "ref_cycles_median": statistics.median(ref_cycles),
            "instructions_median": statistics.median(insns),
            "stalls_l3_miss_median": statistics.median(stalls),
            "llc_load_misses_median": statistics.median(llc),
            "clock_ratio_median": statistics.median(clocks),
            "ipc_median": statistics.median(ipcs),
            "l3_stall_share_median": statistics.median(l3_shares),
            "wall_s_median": statistics.median(walls),
            "peak_kib_median": statistics.median(rss),
        }

    comparisons: dict[str, Any] = {}
    for arm in arms:
        arm_comp: dict[str, Any] = {}
        for hi_k, lo_k in ((8, 4), (8, 1), (4, 1)):
            if hi_k in counts and lo_k in counts:
                hi_rows = [by_key[(arm, hi_k, r)] for r in range(rounds)]
                lo_rows = [by_key[(arm, lo_k, r)] for r in range(rounds)]
                hi_cyc = [float(x["pmu"]["cycles"]) for x in hi_rows]
                lo_cyc = [float(x["pmu"]["cycles"]) for x in lo_rows]
                hi_l3 = [float(x["pmu"]["stalls_l3_miss"]) for x in hi_rows]
                lo_l3 = [float(x["pmu"]["stalls_l3_miss"]) for x in lo_rows]
                hi_llc = [float(x["pmu"]["llc_load_misses"]) for x in hi_rows]
                lo_llc = [float(x["pmu"]["llc_load_misses"]) for x in lo_rows]
                hi_ins = [float(x["pmu"]["instructions"]) for x in hi_rows]
                lo_ins = [float(x["pmu"]["instructions"]) for x in lo_rows]

                cyc_gm, cyc_lo, cyc_hi = paired_ratio_ci(hi_cyc, lo_cyc)
                ins_gm, ins_lo, ins_hi = paired_ratio_ci(hi_ins, lo_ins)
                if not all(math.isfinite(x) and x > 0 for x in (cyc_gm, cyc_lo, cyc_hi)):
                    return {"verdict": "VOID", "reason": f"undefined CI on {arm} {hi_k}w/{lo_k}w"}
                exc_cyc = [h - lo for h, lo in zip(hi_cyc, lo_cyc, strict=True)]
                exc_l3 = [h - lo for h, lo in zip(hi_l3, lo_l3, strict=True)]
                mean_exc_cyc = statistics.fmean(exc_cyc)
                mean_exc_l3 = statistics.fmean(exc_l3)
                l3_frac = (mean_exc_l3 / mean_exc_cyc) if mean_exc_cyc > 0 else None
                llc_ratio = (
                    statistics.median(hi_llc) / statistics.median(lo_llc)
                    if statistics.median(lo_llc) > 0
                    else None
                )
                arm_comp[f"{hi_k}w_over_{lo_k}w"] = {
                    "cycles_ratio": cyc_gm,
                    "cycles_ci95": [cyc_lo, cyc_hi],
                    "instructions_ratio": ins_gm,
                    "instructions_ci95": [ins_lo, ins_hi],
                    "mean_excess_cycles": mean_exc_cyc,
                    "mean_excess_stalls_l3_miss": mean_exc_l3,
                    "l3_excess_fraction": l3_frac,
                    "llc_misses_ratio": llc_ratio,
                }
        comparisons[arm] = arm_comp

    # Evaluate frozen stop_test on 8W-over-4W scanner cycles.
    if "base" not in comparisons or "lanes" not in comparisons:
        return {"verdict": "VOID", "reason": "missing base or lanes arm"}
    if "8w_over_4w" not in comparisons["base"] or "8w_over_4w" not in comparisons["lanes"]:
        return {"verdict": "VOID", "reason": "missing 8w_over_4w comparison (requires k=4 and k=8)"}

    base_84 = comparisons["base"]["8w_over_4w"]
    lanes_84 = comparisons["lanes"]["8w_over_4w"]
    b_lo, b_hi = base_84["cycles_ci95"]
    l_lo, l_hi = lanes_84["cycles_ci95"]
    l_frac = lanes_84["l3_excess_fraction"]

    lanes_excludes_1_above = l_lo > 1.0
    lanes_l3_majority = l_frac is not None and l_frac >= L3_EXCESS_FRACTION_MIN
    base_includes_1 = b_lo <= 1.0 <= b_hi
    lanes_includes_1 = l_lo <= 1.0 <= l_hi

    clauses = {
        "lanes_8w_over_4w_ci_above_1": lanes_excludes_1_above,
        "lanes_l3_stall_share_ge_0.50": lanes_l3_majority,
        "base_8w_over_4w_includes_1": base_includes_1,
        "lanes_8w_over_4w_includes_1": lanes_includes_1,
    }

    if lanes_excludes_1_above and lanes_l3_majority:
        verdict = "CONFIRMED"
        reason = (
            f"lanes 8W/4W scanner cycles={lanes_84['cycles_ratio']:.4f} "
            f"[{l_lo:.4f}, {l_hi:.4f}] > 1 and L3-miss stalls explain "
            f"{l_frac * 100:.1f}% (>= 50%) of the cycle excess"
        )
    elif base_includes_1 and lanes_includes_1:
        verdict = "REFUTED"
        reason = (
            f"8W/4W scanner cycle 95% CI includes 1 for both base "
            f"[{b_lo:.4f}, {b_hi:.4f}] and lanes [{l_lo:.4f}, {l_hi:.4f}]"
        )
    else:
        verdict = "INCONCLUSIVE"
        frac_str = f"{l_frac * 100:.1f}%" if l_frac is not None else "n/a"
        reason = (
            f"mixed/non-L3-majority split: base 8W/4W=[{b_lo:.4f}, {b_hi:.4f}], "
            f"lanes 8W/4W=[{l_lo:.4f}, {l_hi:.4f}], lanes L3-miss excess share={frac_str}"
        )

    return {
        "verdict": verdict,
        "reason": reason,
        "clauses": clauses,
        "cells": cells_out,
        "comparisons": comparisons,
    }


def format_summary(summary: dict[str, Any]) -> str:
    """Format a compact human-readable report of per-k scanner PMU cells and comparisons."""
    lines = [
        f"{summary['verdict']}: {summary['reason']}",
        "",
    ]
    cells = summary.get("cells", {})
    if cells:
        lines.append(
            f"{'cell':<10s} {'cycles(M)':>10s} {'clk_r':>7s} {'ipc':>6s} "
            f"{'l3_st(M)':>10s} {'l3%':>6s} {'llc_m(M)':>9s} {'wall_s':>8s}"
        )
        for name, c in cells.items():
            lines.append(
                f"{name:<10s} {c['cycles_median'] / 1e6:10.2f} {c['clock_ratio_median']:7.4f} "
                f"{c['ipc_median']:6.3f} {c['stalls_l3_miss_median'] / 1e6:10.2f} "
                f"{c['l3_stall_share_median'] * 100:5.1f}% "
                f"{c['llc_load_misses_median'] / 1e6:9.3f} {c['wall_s_median']:8.4f}"
            )
        lines.append("")
    comps = summary.get("comparisons", {})
    if comps:
        lines.append("Paired scanner-cycle ratios:")
        for arm, arm_c in comps.items():
            for label, d in arm_c.items():
                lo, hi = d["cycles_ci95"]
                frac = (
                    f"{d['l3_excess_fraction'] * 100:.1f}%"
                    if d["l3_excess_fraction"] is not None
                    else "n/a"
                )
                lines.append(
                    f"  {arm:<6s} {label:<12s}: {d['cycles_ratio']:.4f} "
                    f"[{lo:.4f}, {hi:.4f}]  d_cyc={d['mean_excess_cycles'] / 1e6:+.2f}M  "
                    f"d_l3={d['mean_excess_stalls_l3_miss'] / 1e6:+.2f}M  l3_share={frac}"
                )
    return "\n".join(lines) + "\n"


def run_probe(
    out: Path,
    *,
    smoke: bool = False,
    allow_any_host: bool = False,
    stub_pmu: bool = False,
    rounds: int | None = None,
    settle_s: str | None = None,
    perf_cmd: str | None = None,
    prebuilt_base: Path | None = None,
    prebuilt_lanes: Path | None = None,
    prebuilt_ab: Path | None = None,
    prebuilt_windows: Path | None = None,
) -> dict[str, Any]:
    """Execute the frozen diagnostic PMU probe (or compact `--smoke` run)."""
    out = Path(out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    strict_i9 = not (smoke or allow_any_host)

    eff_rounds = rounds if rounds is not None else (2 if smoke else ROUNDS)
    eff_settle = settle_s if settle_s is not None else ("0" if smoke else SETTLE_S)
    eff_perf = perf_cmd if perf_cmd is not None else "perf"
    eff_stub_pmu = stub_pmu

    if strict_i9:
        overrides = {
            "--rounds": rounds,
            "--settle": settle_s,
            "--perf-cmd": perf_cmd,
            "--stub-pmu": True if stub_pmu else None,
            "--base-bin": prebuilt_base,
            "--lanes-bin": prebuilt_lanes,
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
    scanner_cpu = int(host["scanner_cpu"])
    worker_mask = str(host["worker_mask"])

    eff_events: tuple[str, ...] = PERF_EVENTS
    if not eff_stub_pmu:
        if smoke and perf_cmd is None:
            try:
                eff_events = preflight_perf(out, scanner_cpu=scanner_cpu, perf_cmd=eff_perf)
            except (InvalidRun, FileNotFoundError):
                eff_stub_pmu = True
        else:
            eff_events = preflight_perf(out, scanner_cpu=scanner_cpu, perf_cmd=eff_perf)

    save(
        out / "provenance.json",
        {
            "head": output("git", "rev-parse", "HEAD"),
            "smoke": smoke,
            "strict_i9": strict_i9,
            "stub_pmu": eff_stub_pmu,
            "rounds": eff_rounds,
            "settle_s": eff_settle,
            "arms": list(ARMS),
            "counts": list(COUNTS),
            "events": list(eff_events),
            "scanner_cpu": scanner_cpu,
            "worker_mask": worker_mask,
            "seed": SEED,
        },
    )

    with tempfile.TemporaryDirectory(prefix="scanner-contention-") as td:
        tmp = Path(td)
        bins, bin_meta = build_arms(
            out,
            tmp,
            root=ROOT,
            prebuilt_base=prebuilt_base,
            prebuilt_lanes=prebuilt_lanes,
            prebuilt_ab=prebuilt_ab,
            prebuilt_windows=prebuilt_windows,
        )
        corpus_path, ref = prepare_input(out, tmp, bins, root=ROOT, smoke=smoke)

        # Record passive env facts via ab binary and check quota in strict mode.
        env_facts = output(str(bins["ab"]), "env", "--facts")
        (out / "env-facts.json").write_text(env_facts + "\n", encoding="utf-8")
        if strict_i9:
            check_no_quota(json.loads(env_facts))

        verify_parity(
            out,
            bins,
            corpus_path,
            ref,
            scanner_cpu=scanner_cpu,
            worker_mask=worker_mask,
            arms=ARMS,
            parity_counts=(1, 4) if smoke else PARITY_COUNTS,
        )

        pause = float(eff_settle)
        rows: list[dict[str, Any]] = []
        with (out / "runs.jsonl").open("w", encoding="utf-8", buffering=1) as log:
            for r, arm, k in build_plan(ARMS, COUNTS, eff_rounds, seed=SEED):
                if pause > 0:
                    time.sleep(pause)
                sample = measure_sample(
                    bins[arm],
                    corpus_path,
                    ref,
                    arm=arm,
                    k=k,
                    scanner_cpu=scanner_cpu,
                    worker_mask=worker_mask,
                    perf_cmd=eff_perf,
                    events=eff_events,
                    stub_pmu=eff_stub_pmu,
                    tmp=tmp,
                )
                row = {"round": r, "arm": arm, "k": k, **sample}
                log.write(json.dumps(row, allow_nan=False) + "\n")
                rows.append(row)

        summary = score(
            rows,
            ref,
            rounds=eff_rounds,
            arms=ARMS,
            counts=COUNTS,
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
    """Re-score an archived run directory cleanly from its JSON/JSONL artifacts."""
    out = Path(out).resolve()
    prov = json.loads((out / "provenance.json").read_text(encoding="utf-8"))
    bins = json.loads((out / "binaries.json").read_text(encoding="utf-8"))
    inputs = json.loads((out / "inputs.json").read_text(encoding="utf-8"))
    ref = dict(inputs["wikidata"])

    parity_path = out / "parity.jsonl"
    if parity_path.exists():
        for line in parity_path.read_text(encoding="utf-8").splitlines():
            if line.strip():
                rec = json.loads(line)
                if rec.get("scanned"):
                    ref["scanned"] = int(rec["scanned"])

    rows = [
        json.loads(line)
        for line in (out / "runs.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    registered = (
        prov.get("strict_i9") is True
        and prov.get("stub_pmu") is False
        and int(prov["rounds"]) == ROUNDS
        and str(prov["settle_s"]) == SETTLE_S
        and tuple(prov["arms"]) == ARMS
        and tuple(prov["counts"]) == COUNTS
    )
    summary = score(
        rows,
        ref,
        rounds=int(prov["rounds"]),
        arms=tuple(prov["arms"]),
        counts=tuple(prov["counts"]),
        distinct_binaries=bool(bins.get("distinct_binaries", False)),
        host_ok=True,
    )
    if not registered and summary["verdict"] != "VOID":
        summary = {
            **summary,
            "verdict": "VOID",
            "reason": "not the registered run (smoke, non-i9 host, stub PMU "
            "or overridden parameters); diagnostic only: " + summary["reason"],
        }
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

    p_run = sub.add_parser("run", help="Execute the scanner contention PMU probe (or --smoke)")
    p_run.add_argument("out", type=Path, help="Output directory for artifacts")
    p_run.add_argument("--smoke", action="store_true", help="Compact local VM smoke test")
    p_run.add_argument("--allow-any-host", action="store_true", help="Allow non-i9 host")
    p_run.add_argument("--stub-pmu", action="store_true", help="Use synthetic PMU (smoke only)")
    p_run.add_argument("--rounds", type=int, default=None, help="Override interleaved rounds")
    p_run.add_argument("--settle", type=str, default=None, help="Override settle seconds")
    p_run.add_argument("--perf-cmd", type=str, default=None, help="Override perf executable")
    p_run.add_argument("--base-bin", type=Path, default=None, help="Pre-built base run binary")
    p_run.add_argument("--lanes-bin", type=Path, default=None, help="Pre-built lanes run binary")
    p_run.add_argument("--ab-bin", type=Path, default=None, help="Pre-built ab binary")
    p_run.add_argument("--windows-exe", type=Path, default=None, help="Pre-built windows binary")

    p_score = sub.add_parser("score", help="Re-score an archived run directory")
    p_score.add_argument("out", type=Path, help="Run directory containing runs.jsonl")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        if args.cmd == "run":
            res = run_probe(
                args.out,
                smoke=args.smoke,
                allow_any_host=args.allow_any_host,
                stub_pmu=args.stub_pmu,
                rounds=args.rounds,
                settle_s=args.settle,
                perf_cmd=args.perf_cmd,
                prebuilt_base=args.base_bin,
                prebuilt_lanes=args.lanes_bin,
                prebuilt_ab=args.ab_bin,
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
