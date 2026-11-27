#!/usr/bin/env python3
"""Frozen i9-10900K probe: separate clock modes and mode-robust ab analysis (i9-clock-modes).

Subcommands:
    python3 bench/clock_modes.py run OUT_DIR [--smoke] [--corpus PATH] [--rounds N]
                                 [--threads 1,6] [--prespin-ms 0] [--ab-bin PATH]
                                 [--allow-any-host]
    python3 bench/clock_modes.py score RUNS_JSONL [--out DIR] [--rounds N]
    python3 bench/clock_modes.py analyze RUNS_JSONL [--ref ARM] [--arm ARM]
                                 [--threads 1,12] [--json]
"""

from __future__ import annotations

import argparse
import bz2
import hashlib
import json
import math
import os
import random
import shutil
import statistics
import struct
import subprocess
import sys
import tempfile
import threading
import time
from collections import defaultdict
from pathlib import Path
from typing import Any

BENCH_DIR = Path(__file__).resolve().parent
if str(BENCH_DIR) not in sys.path:
    sys.path.insert(0, str(BENCH_DIR))

import wikidata  # noqa: E402
from mtf_fold import t_crit_95, topology  # noqa: E402

ROOT = BENCH_DIR.parent
REGISTRATION = ROOT / "docs/journal/2026-11-27-i9-clock-modes/registration.md"

WIKIDATA_X4_SHA256 = "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"

THREADS = (1, 6)
PINNINGS = ("pinned", "unpinned")
SETTLES = (0.0, 0.2)
PAIR_POSITIONS = (0, 1)
DEFAULT_PRESPIN_MS = (0,)
ROUNDS = 12
SEED = 1189
RUN_TIMEOUT_S = 180
SAMPLE_INTERVAL_S = 0.005

WALL_SLOW_RATIO = {1: 1.04, 6: 1.30}
CYCLES_PER_BYTE_TOL = 0.03
REAL_WORK_CYCLES_RATIO = 1.20
INSTRUCTIONS_PER_BYTE_TOL = 0.03
EXTRA_INSTRUCTIONS_RATIO = 1.15
EXPLAIN_THRESHOLD = 0.80
MIN_SLOW_RUNS = 3

MSR_MPERF = 0xE7
MSR_APERF = 0xE8


class InvalidRun(ValueError):
    """A registered prerequisite or validity invariant failed; the run is VOID."""


def output(*cmd: str, cwd: Path = ROOT) -> str:
    return subprocess.check_output(list(cmd), cwd=cwd, text=True).strip()


def save_json(path: Path, obj: Any) -> None:
    path.write_text(json.dumps(obj, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def sha256_file(path: Path) -> str:
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def wall_slow_threshold(threads: int) -> float:
    """Return the frozen slow-mode wall ratio threshold for a given thread count."""
    if threads in WALL_SLOW_RATIO:
        return WALL_SLOW_RATIO[threads]
    return 1.04 if threads <= 1 else 1.30


def reference_bz2(path: Path) -> dict[str, Any]:
    """Independently decompress a .bz2 file with Python's libbzip2 binding."""
    digest = hashlib.sha256()
    total = 0
    with bz2.open(path, "rb") as stream:
        while chunk := stream.read(1 << 20):
            total += len(chunk)
            digest.update(chunk)
    return {"out": total, "sha256": digest.hexdigest()}


def check_baseline_env(env: dict[str, str] | None = None) -> None:
    """Require default baseline x86-64 codegen with no RUSTFLAGS overrides."""
    check_env = os.environ if env is None else env
    for var in ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
        if check_env.get(var, "").strip():
            raise InvalidRun(f"{var} must be unset for baseline codegen")


def check_cpu_quota_unlimited(env_facts: dict[str, Any]) -> None:
    """Require unlimited cgroup CPU quota (supports both `ab env` and `ab env --facts` shapes)."""
    quota = env_facts.get("cpu_quota", "absent")
    if quota is None:
        return
    if isinstance(quota, dict) and quota.get("state") == "unlimited" and quota.get("cpus") is None:
        return
    raise InvalidRun(f"workstation CPU quota must be unlimited (got {quota!r})")


def discover_cores(out: Path | None = None, *, require_i9: bool = True) -> list[list[int]]:
    """Inspect host CPU topology via lscpu; enforce i9-10900K invariants when `require_i9`."""
    description = output("lscpu")
    grid_text = output("lscpu", "-e")
    if out is not None:
        (out / "lscpu.txt").write_text(description + "\n", encoding="utf-8")
        grid_path = out / "lscpu-e.txt"
        grid_path.write_text(grid_text + "\n", encoding="utf-8")
        cores = topology(str(grid_path))
    else:
        with tempfile.TemporaryDirectory() as td:
            gp = Path(td) / "lscpu-e.txt"
            gp.write_text(grid_text + "\n", encoding="utf-8")
            cores = topology(str(gp))

    if require_i9:
        if "i9-10900K" not in description:
            raise InvalidRun("registered machine is the i9-10900K")
        cpus = {cpu for core in cores for cpu in core}
        if len(cores) != 10 or any(len(core) != 2 for core in cores) or len(cpus) != 20:
            raise InvalidRun("expected ten physical cores with both hardware threads")
        if cpus != os.sched_getaffinity(0):
            raise InvalidRun("all 20 workstation CPUs must be available")
    return cores


def cpu_to_core_map(cores: list[list[int]]) -> dict[int, int]:
    """Map logical CPU ID -> 0-based physical core index."""
    mapping: dict[int, int] = {}
    for core_idx, siblings in enumerate(cores):
        for cpu in siblings:
            mapping[cpu] = core_idx
    return mapping


def pinned_cpu_list(cores: list[list[int]], threads: int) -> list[int]:
    """Return pinned CPU list matching `ab` conventions: core 0 for 1T, first sibling per core."""
    avail = sorted(os.sched_getaffinity(0))
    avail_set = set(avail)
    first_siblings = [core[0] for core in cores if core and core[0] in avail_set]
    if len(first_siblings) >= threads:
        return first_siblings[:threads]
    # Fallback on small VMs (e.g., smoke runs requesting more threads than physical cores).
    ordered = first_siblings + [c for c in avail if c not in set(first_siblings)]
    return ordered[: max(1, min(threads, len(ordered)))]


def materialize_wikidata(tmp: Path, checkout: Path, windows_bin: Path) -> dict[str, Any]:
    """Build or verify corpora/wikidata.bz2 and its 4x multistream wall input, shallow-safely."""
    checkout = Path(checkout)
    tmp = Path(tmp)
    spec = json.loads((checkout / "bench/wikidata.json").read_text(encoding="utf-8"))
    sample = None
    for candidate in (
        checkout / "corpora/wikidata.bz2",
        checkout / "corpora/wikidata-sample.bz2",
    ):
        if candidate.exists() and sha256_file(candidate) == spec["corpus_sha256"]:
            sample = candidate
            break
    if sample is None:
        sample = tmp / "wikidata-base.bz2"
        windows_dir = tmp / "windows"
        windows_dir.mkdir(parents=True, exist_ok=True)
        bd_repo = tmp / "bench-data-git"
        origin = output("git", "remote", "get-url", "origin", cwd=checkout)
        subprocess.run(["git", "init", "-q", str(bd_repo)], check=True)
        subprocess.run(
            ["git", "fetch", "-q", "--depth=1", "--no-tags", origin, "bench-data"],
            cwd=bd_repo,
            check=True,
        )
        for window in spec["windows"]:
            fname = window["name"] + ".bin"
            blob = subprocess.check_output(
                ["git", "show", f"FETCH_HEAD:{spec['windows_dir']}/{fname}"],
                cwd=bd_repo,
            )
            if hashlib.sha256(blob).hexdigest() != window["sha256"]:
                raise InvalidRun(f"bench-data window {fname} SHA-256 mismatch")
            (windows_dir / fname).write_bytes(blob)
        wikidata.build(spec, windows_dir, str(windows_bin), sample)

    base_hash = sha256_file(sample)
    if base_hash != spec["corpus_sha256"]:
        raise InvalidRun(f"wikidata base SHA-256 mismatch ({base_hash})")

    wall_path = tmp / "wikidata.bz2"
    with wall_path.open("wb") as dest:
        for _ in range(4):
            with sample.open("rb") as src:
                shutil.copyfileobj(src, dest)
    wall_hash = sha256_file(wall_path)
    if wall_hash != WIKIDATA_X4_SHA256:
        raise InvalidRun(f"wikidata x4 SHA-256 mismatch ({wall_hash})")
    return {
        "name": "wikidata",
        "base_path": sample,
        "wall_path": wall_path,
        "base_bz2_sha256": base_hash,
        "wall_bz2_sha256": wall_hash,
    }


def build_binaries(
    checkout: Path, tmp: Path, *, include_windows: bool = True, check_env: bool = True
) -> dict[str, Path]:
    """Build ab (and optionally windows) in an isolated CARGO_TARGET_DIR with baseline codegen."""
    if check_env:
        check_baseline_env()
    target_dir = Path(tmp) / "target"
    bin_dir = Path(tmp) / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    env = {**os.environ, "CARGO_TARGET_DIR": str(target_dir)}
    names = ["ab", "windows"] if include_windows else ["ab"]
    cmd = ["cargo", "build", "--release", "--locked"]
    for name in names:
        cmd.extend(["--example", name])
    subprocess.run(cmd, cwd=checkout, env=env, check=True)
    bins: dict[str, Path] = {}
    for name in names:
        src = target_dir / "release/examples" / name
        dst = bin_dir / name
        shutil.copy2(src, dst)
        bins[name] = dst
    return bins


def factorial_plan(
    rounds: int = ROUNDS,
    threads: tuple[int, ...] = THREADS,
    pinnings: tuple[str, ...] = PINNINGS,
    settles: tuple[float, ...] = SETTLES,
    prespin_ms_values: tuple[int, ...] = DEFAULT_PRESPIN_MS,
    seed: int = SEED,
) -> list[dict[str, Any]]:
    """Build deterministic balanced schedule of (threads, pinning, settle, prespin_ms) pairs.

    Each pair-cell runs back-to-back as pair_pos 0 then pair_pos 1 so pair position is a
    balanced 2-level factor within every round.
    """
    rng = random.Random(seed)
    pair_cells = [
        (t, pin, settle, prespin)
        for t in threads
        for pin in pinnings
        for settle in settles
        for prespin in prespin_ms_values
    ]
    schedule: list[dict[str, Any]] = []
    pair_id = 0
    for r in range(rounds):
        order = list(pair_cells)
        rng.shuffle(order)
        for order_in_round, (t, pin, settle, prespin) in enumerate(order):
            for pair_pos in PAIR_POSITIONS:
                schedule.append(
                    {
                        "round": r,
                        "pair_id": pair_id,
                        "order_in_round": order_in_round,
                        "threads": t,
                        "pinning": pin,
                        "settle_s": float(settle),
                        "prespin_ms": int(prespin),
                        "pair_pos": pair_pos,
                    }
                )
            pair_id += 1
    return schedule


def parse_proc_stat_comm_cpu(stat_text: str) -> tuple[str | None, int | None]:
    """Extract `(comm, processor)` from `/proc/<pid>/task/<tid>/stat`.

    Handles thread comm names containing spaces or parentheses by splitting after the final `)`.
    """
    lparen = stat_text.find("(")
    rparen = stat_text.rfind(")")
    if lparen < 0 or rparen <= lparen:
        return None, None
    comm = stat_text[lparen + 1 : rparen]
    tail = stat_text[rparen + 2 :].split()
    # After `)` field 3 (`state`) is index 0; field 39 (`processor`) is index 36.
    if len(tail) <= 36:
        return comm, None
    try:
        return comm, int(tail[36])
    except ValueError:
        return comm, None


def parse_proc_stat_cpu(stat_text: str) -> int | None:
    """Extract the `processor` field (field 39, 1-indexed) from `/proc/<pid>/task/<tid>/stat`."""
    return parse_proc_stat_comm_cpu(stat_text)[1]


class PlacementSampler:
    """Poll `/proc/<pid>/task/*/stat` in the parent while a child runs."""

    _WRAPPER_COMMS = frozenset({"taskset", "perf"})

    def __init__(
        self,
        pid: int,
        core_map: dict[int, int] | None = None,
        interval_s: float = SAMPLE_INTERVAL_S,
    ) -> None:
        self.pid = pid
        self.core_map = core_map or {}
        self.interval_s = interval_s
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, name="placement-sampler", daemon=True)
        self.sample_count = 0
        self.cpus_seen: set[int] = set()
        self.per_tid_cpus: dict[str, set[int]] = defaultdict(set)
        self._last_cpu_by_tid: dict[str, int] = {}
        self.migrations = 0
        self.smt_sibling_overlap_samples = 0

    def start(self) -> None:
        self._thread.start()

    def _sample_once(self) -> None:
        task_dir = Path(f"/proc/{self.pid}/task")
        try:
            entries = list(task_dir.iterdir())
        except OSError:
            return
        active_cpus: list[int] = []
        observed_any = False
        for entry in entries:
            tid = entry.name
            try:
                text = (entry / "stat").read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            comm, cpu = parse_proc_stat_comm_cpu(text)
            if cpu is None or comm in self._WRAPPER_COMMS:
                continue
            observed_any = True
            self.cpus_seen.add(cpu)
            self.per_tid_cpus[tid].add(cpu)
            prev = self._last_cpu_by_tid.get(tid)
            if prev is not None and prev != cpu:
                self.migrations += 1
            self._last_cpu_by_tid[tid] = cpu
            active_cpus.append(cpu)
        if not observed_any:
            return
        self.sample_count += 1
        if self.core_map and len(active_cpus) >= 2:
            cores_now: dict[int, set[int]] = defaultdict(set)
            for cpu in active_cpus:
                core = self.core_map.get(cpu)
                if core is not None:
                    cores_now[core].add(cpu)
            if any(len(siblings) >= 2 for siblings in cores_now.values()):
                self.smt_sibling_overlap_samples += 1

    def _run(self) -> None:
        while not self._stop.is_set():
            self._sample_once()
            self._stop.wait(self.interval_s)

    def stop(self) -> dict[str, Any]:
        self._stop.set()
        self._thread.join(timeout=2.0)
        sorted_cpus = sorted(self.cpus_seen)
        distinct_cores = (
            sorted({self.core_map[c] for c in sorted_cpus if c in self.core_map})
            if self.core_map
            else []
        )
        return {
            "sample_count": self.sample_count,
            "cpus_sampled": sorted_cpus,
            "n_distinct_cpus": len(sorted_cpus),
            "n_distinct_cores": len(distinct_cores) if self.core_map else None,
            "smt_sibling_overlap_samples": self.smt_sibling_overlap_samples,
            "migrations": self.migrations,
            "per_tid_cpus": {tid: sorted(cpus) for tid, cpus in sorted(self.per_tid_cpus.items())},
        }


def read_sysfs_snapshot(cpus: list[int]) -> dict[str, Any]:
    """Best-effort read of cpufreq / turbo sysfs state; never raises."""
    freqs_khz: dict[str, int] = {}
    governors: set[str] = set()
    epps: set[str] = set()
    for cpu in cpus:
        base = Path(f"/sys/devices/system/cpu/cpufreq/policy{cpu}")
        if not base.exists():
            continue
        for fname, target in (
            ("scaling_cur_freq", "freq"),
            ("scaling_governor", "gov"),
            ("energy_performance_preference", "epp"),
        ):
            try:
                val = (base / fname).read_text(encoding="utf-8").strip()
            except OSError:
                continue
            if target == "freq":
                try:
                    freqs_khz[str(cpu)] = int(val)
                except ValueError:
                    pass
            elif target == "gov" and val:
                governors.add(val)
            elif target == "epp" and val:
                epps.add(val)
    no_turbo = None
    try:
        raw = Path("/sys/devices/system/cpu/intel_pstate/no_turbo").read_text(encoding="utf-8")
        no_turbo = int(raw.strip())
    except (OSError, ValueError):
        pass
    return {
        "freqs_khz": freqs_khz,
        "median_ghz": round(statistics.median(freqs_khz.values()) / 1e6, 3) if freqs_khz else None,
        "governors": sorted(governors),
        "epps": sorted(epps),
        "no_turbo": no_turbo,
    }


def read_msr_aperf_mperf(cpus: list[int]) -> dict[int, tuple[int, int]] | None:
    """Read (APERF, MPERF) MSR pair for each CPU if `/dev/cpu/*/msr` is readable."""
    readings: dict[int, tuple[int, int]] = {}
    for cpu in cpus:
        path = Path(f"/dev/cpu/{cpu}/msr")
        try:
            with path.open("rb", buffering=0) as f:
                f.seek(MSR_APERF)
                aperf = struct.unpack("<Q", f.read(8))[0]
                f.seek(MSR_MPERF)
                mperf = struct.unpack("<Q", f.read(8))[0]
            readings[cpu] = (aperf, mperf)
        except OSError:
            return None
    return readings if readings else None


def msr_aperf_mperf_ratio(
    before: dict[int, tuple[int, int]] | None,
    after: dict[int, tuple[int, int]] | None,
) -> dict[str, Any] | None:
    """Compute aggregate delta APERF / MPERF across CPUs (64-bit wrap-safe)."""
    if not before or not after:
        return None
    mask = (1 << 64) - 1
    d_aperf = 0
    d_mperf = 0
    for cpu, (a0, m0) in before.items():
        if cpu not in after:
            continue
        a1, m1 = after[cpu]
        d_aperf += (a1 - a0) & mask
        d_mperf += (m1 - m0) & mask
    if d_mperf <= 0:
        return None
    return {
        "delta_aperf": d_aperf,
        "delta_mperf": d_mperf,
        "aperf_per_mperf": round(d_aperf / d_mperf, 4),
    }


def parse_perf_stat_csv(stderr_text: str) -> dict[str, int | None]:
    """Parse `perf stat -x,` stderr output into cycles, instructions, ref_cycles."""
    out: dict[str, int | None] = {"cycles": None, "instructions": None, "ref_cycles": None}
    for line in stderr_text.splitlines():
        parts = [p.strip() for p in line.split(",")]
        if len(parts) < 3:
            continue
        val_str, event = parts[0], parts[2].lower()
        if not val_str or val_str.startswith("<"):
            continue
        try:
            val = int(float(val_str))
        except ValueError:
            continue
        if "ref-cycles" in event or "ref_cycles" in event:
            out["ref_cycles"] = val
        elif "cycles" in event and out["cycles"] is None:
            out["cycles"] = val
        elif "instructions" in event and out["instructions"] is None:
            out["instructions"] = val
    return out


def probe_perf_backend(ab_bin: Path, sample_cpus: list[int]) -> dict[str, Any]:
    """Probe available hardware counter backends without failing on restricted VMs."""
    # 1. Try `ab env` passive/active perf probe via `ab env` (checks ab's own perf_event_open).
    ab_reason = None
    try:
        proc = subprocess.run(
            [str(ab_bin), "env"],
            text=True,
            capture_output=True,
            timeout=15,
            check=False,
        )
        if proc.returncode == 0 and proc.stdout.strip():
            data = json.loads(proc.stdout.splitlines()[-1])
            perf_info = data.get("perf", {})
            if perf_info.get("cycles") is True:
                return {"source": "ab_perf", "available": True, "reason": None, "detail": perf_info}
            ab_reason = perf_info.get("why") or "ab perf_event_open unavailable"
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        ab_reason = str(exc)

    # 2. Try `perf stat` CLI fallback.
    perf_exe = shutil.which("perf")
    perf_reason = "perf binary not found"
    if perf_exe:
        try:
            proc = subprocess.run(
                [perf_exe, "stat", "-x,", "-e", "cycles,instructions", "--", "true"],
                text=True,
                capture_output=True,
                timeout=10,
                check=False,
            )
            parsed = parse_perf_stat_csv(proc.stderr)
            if proc.returncode == 0 and parsed["cycles"] is not None:
                return {"source": "perf_stat", "available": True, "reason": None, "detail": parsed}
            perf_reason = (
                proc.stderr.strip().splitlines()[:1] or ["perf stat returned no cycles"]
            )[0]
        except (OSError, subprocess.SubprocessError) as exc:
            perf_reason = str(exc)

    # 3. Try `/dev/cpu/*/msr` APERF/MPERF fallback.
    if sample_cpus and read_msr_aperf_mperf(sample_cpus[:1]) is not None:
        return {"source": "msr_aperf_mperf", "available": True, "reason": None, "detail": {}}

    return {
        "source": "none",
        "available": False,
        "reason": f"ab_perf: {ab_reason}; perf_stat: {perf_reason}; msr: unreadable",
        "detail": {},
    }


def run_prespin(cpus: list[int], ms: int) -> float:
    """Spin worker threads across `cpus` for `ms` milliseconds to warm clocks before launch."""
    if ms <= 0:
        return 0.0
    t0 = time.monotonic()
    deadline = t0 + ms / 1000.0
    barrier = threading.Barrier(len(cpus))

    def _spin(cpu: int) -> None:
        try:
            os.sched_setaffinity(0, {cpu})
        except OSError:
            pass
        try:
            barrier.wait(timeout=1.0)
        except threading.BrokenBarrierError:
            pass
        acc = 0
        while time.monotonic() < deadline:
            acc = (acc + 1) & 0xFFFF
        _ = acc

    threads = [threading.Thread(target=_spin, args=(c,), daemon=True) for c in cpus]
    for th in threads:
        th.start()
    for th in threads:
        th.join(timeout=max(2.0, ms / 1000.0 + 1.0))
    return round(time.monotonic() - t0, 4)


def run_ab_child_simple(ab_bin: Path, arm: str, path: Path, threads: int) -> dict[str, Any]:
    """Run one untimed `ab child` (for parity verification)."""
    proc = subprocess.run(
        [str(ab_bin), "child", arm, str(threads), str(path)],
        text=True,
        capture_output=True,
        timeout=RUN_TIMEOUT_S,
        check=False,
    )
    if proc.returncode != 0:
        raise InvalidRun(
            f"ab child {arm} failed (rc={proc.returncode}) on {path}: {proc.stderr.strip()}"
        )
    lines = [ln for ln in proc.stdout.splitlines() if ln.strip()]
    if len(lines) != 1:
        raise InvalidRun(f"expected 1 JSON line from ab child {arm}, got {len(lines)}")
    return json.loads(lines[0])


def run_parity(out: Path, ab_bin: Path, corpus_info: dict[str, Any]) -> dict[str, Any]:
    """Verify untimed mibz2 parity against ab's libbzip2 arm and Python libbzip2."""
    wall_path = Path(corpus_info["wall_path"])
    base_path = Path(corpus_info.get("base_path", wall_path))
    if base_path == wall_path:
        wall_ref = reference_bz2(wall_path)
        expected_wall_out = wall_ref["out"]
        wall_raw_sha256: str | None = wall_ref["sha256"]
        base_out = wall_ref["out"]
        base_raw_sha256 = wall_ref["sha256"]
    else:
        base_ref = reference_bz2(base_path)
        expected_wall_out = base_ref["out"] * 4
        wall_raw_sha256 = None
        base_out = base_ref["out"]
        base_raw_sha256 = base_ref["sha256"]

    lib_row = run_ab_child_simple(ab_bin, "libbzip2", wall_path, 1)
    if lib_row.get("err") not in (None, ""):
        raise InvalidRun(f"libbzip2 parity decode errored: {lib_row}")
    if lib_row.get("out") != expected_wall_out or not lib_row.get("hash"):
        raise InvalidRun(
            f"libbzip2 output mismatch: got ({lib_row.get('out')}, {lib_row.get('hash')}), "
            f"expected out={expected_wall_out}"
        )

    mib_row = run_ab_child_simple(ab_bin, "mibz2", wall_path, 6)
    if mib_row.get("err") not in (None, ""):
        raise InvalidRun(f"mibz2 parity decode errored: {mib_row}")
    if mib_row.get("out") != lib_row["out"] or mib_row.get("hash") != lib_row["hash"]:
        raise InvalidRun(
            f"mibz2 parity mismatch against libbzip2: "
            f"got ({mib_row.get('out')}, {mib_row.get('hash')}), "
            f"expected ({lib_row['out']}, {lib_row['hash']})"
        )

    with (out / "parity.jsonl").open("w", encoding="utf-8") as log:
        log.write(json.dumps({"corpus": corpus_info["name"], "arm": "libbzip2", "row": lib_row}))
        log.write("\n")
        log.write(json.dumps({"corpus": corpus_info["name"], "arm": "mibz2", "row": mib_row}))
        log.write("\n")

    return {
        "corpus": corpus_info["name"],
        "base_out": base_out,
        "base_in": base_path.stat().st_size,
        "base_raw_sha256": base_raw_sha256,
        "base_bz2_sha256": corpus_info.get("base_bz2_sha256"),
        "wall_out": expected_wall_out,
        "wall_in": wall_path.stat().st_size,
        "wall_raw_sha256": wall_raw_sha256,
        "wall_bz2_sha256": corpus_info.get("wall_bz2_sha256", sha256_file(wall_path)),
        "ab_hash": lib_row["hash"],
    }


def run_one_child(
    ab_bin: Path,
    corpus_path: Path,
    item: dict[str, Any],
    cores: list[list[int]],
    perf_backend: dict[str, Any],
) -> dict[str, Any]:
    """Execute one timed factorial cell with PMU counters, sysfs, rusage, and CPU placement."""
    threads = int(item["threads"])
    pinning = str(item["pinning"])
    settle_s = float(item["settle_s"])
    prespin_ms = int(item["prespin_ms"])

    all_cpus = sorted(os.sched_getaffinity(0))
    if pinning == "pinned":
        target_cpus = pinned_cpu_list(cores, threads)
        cpus_label = ",".join(str(c) for c in target_cpus)
    else:
        target_cpus = all_cpus
        cpus_label = "all"

    if settle_s > 0.0:
        time.sleep(settle_s)

    prespin_wall_s = run_prespin(target_cpus, prespin_ms) if prespin_ms > 0 else 0.0

    cmd: list[str] = []
    if pinning == "pinned":
        cmd.extend(["taskset", "-c", cpus_label])

    perf_source = perf_backend["source"]
    if perf_source == "perf_stat":
        cmd.extend(["perf", "stat", "-x,", "-e", "cycles,instructions,ref-cycles", "--"])

    cmd.extend([str(ab_bin), "child", "mibz2", str(threads), str(corpus_path)])

    env = dict(os.environ)
    env["AB_MACHINE_STATE"] = "1"
    env.pop("AB_SHA256", None)

    sysfs_pre = read_sysfs_snapshot(target_cpus)
    msr_pre = read_msr_aperf_mperf(target_cpus) if perf_source == "msr_aperf_mperf" else None

    core_map = cpu_to_core_map(cores)
    t_spawn_ns = time.time_ns()
    proc = subprocess.Popen(
        cmd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    sampler = PlacementSampler(proc.pid, core_map=core_map)
    sampler.start()

    stdout_chunks: list[str] = []
    stderr_chunks: list[str] = []

    def _drain(stream: Any, bucket: list[str]) -> None:
        if stream is not None:
            bucket.append(stream.read())

    out_reader = threading.Thread(target=_drain, args=(proc.stdout, stdout_chunks), daemon=True)
    err_reader = threading.Thread(target=_drain, args=(proc.stderr, stderr_chunks), daemon=True)
    out_reader.start()
    err_reader.start()

    # Wait with os.wait4 to capture exact child user_s / sys_s / ru_maxrss.
    deadline = time.monotonic() + RUN_TIMEOUT_S
    status = None
    rusage = None
    while True:
        wpid, wstatus, wrusage = os.wait4(proc.pid, os.WNOHANG)
        if wpid == proc.pid:
            status = wstatus
            rusage = wrusage
            break
        if time.monotonic() > deadline:
            proc.kill()
            _, wstatus, wrusage = os.wait4(proc.pid, 0)
            sampler.stop()
            out_reader.join(timeout=2.0)
            err_reader.join(timeout=2.0)
            raise InvalidRun(f"ab child timed out after {RUN_TIMEOUT_S}s ({item})")
        time.sleep(0.002)

    # Inform Popen that the child has been reaped so its destructor does not wait again.
    rc = os.waitstatus_to_exitcode(status)
    proc.returncode = rc
    t_exit_ns = time.time_ns()
    placement = sampler.stop()
    out_reader.join(timeout=5.0)
    err_reader.join(timeout=5.0)
    msr_post = read_msr_aperf_mperf(target_cpus) if perf_source == "msr_aperf_mperf" else None
    sysfs_post = read_sysfs_snapshot(target_cpus)

    stdout_text = "".join(stdout_chunks)
    stderr_text = "".join(stderr_chunks)
    if rc != 0:
        raise InvalidRun(f"ab child failed (rc={rc}) for {item}: {stderr_text.strip()}")

    lines = [ln for ln in stdout_text.splitlines() if ln.strip()]
    if len(lines) != 1:
        raise InvalidRun(f"expected 1 JSON line from ab child, got {len(lines)} for {item}")
    child_row = json.loads(lines[0])

    # Merge hardware counter fields with graceful fallback.
    cycles = child_row.get("cycles")
    instructions = child_row.get("instructions")
    ref_cycles = child_row.get("ref_cycles")
    cycles_per_ref = child_row.get("cycles_per_ref")
    effective_perf_source = "ab_perf" if cycles is not None else "none"
    perf_available = cycles is not None
    perf_reason: str | None = None
    msr_info = None

    if not perf_available and perf_source == "perf_stat":
        parsed_perf = parse_perf_stat_csv(stderr_text)
        if parsed_perf["cycles"] is not None:
            cycles = parsed_perf["cycles"]
            instructions = parsed_perf["instructions"]
            ref_cycles = parsed_perf["ref_cycles"]
            if ref_cycles and ref_cycles > 0:
                cycles_per_ref = round(cycles / ref_cycles, 4)
            effective_perf_source = "perf_stat"
            perf_available = True

    if not perf_available and perf_source == "msr_aperf_mperf":
        msr_info = msr_aperf_mperf_ratio(msr_pre, msr_post)
        if msr_info is not None:
            cycles_per_ref = msr_info["aperf_per_mperf"]
            effective_perf_source = "msr_aperf_mperf"
            perf_available = True

    if not perf_available:
        perf_reason = (
            child_row.get("cycles_err") or perf_backend.get("reason") or "perf unavailable"
        )

    out_bytes = child_row.get("out")
    cycles_per_byte = (
        round(cycles / out_bytes, 6)
        if isinstance(cycles, (int, float))
        and isinstance(out_bytes, int)
        and out_bytes > 0
        and cycles > 0
        else None
    )
    instructions_per_byte = (
        round(instructions / out_bytes, 6)
        if isinstance(instructions, (int, float))
        and isinstance(out_bytes, int)
        and out_bytes > 0
        and instructions > 0
        else None
    )
    cpi = (
        round(cycles / instructions, 4)
        if isinstance(cycles, (int, float))
        and isinstance(instructions, (int, float))
        and cycles > 0
        and instructions > 0
        else None
    )

    return {
        **child_row,
        **item,
        "arm": "mibz2",
        "cpus": cpus_label,
        "prespin_wall_s": prespin_wall_s,
        "user_s": round(rusage.ru_utime, 4) if rusage is not None else None,
        "sys_s": round(rusage.ru_stime, 4) if rusage is not None else None,
        "parent_rss_kb": int(rusage.ru_maxrss) if rusage is not None else None,
        "t_spawn_unix_ns": t_spawn_ns,
        "t_exit_unix_ns": t_exit_ns,
        "cycles": cycles,
        "instructions": instructions,
        "ref_cycles": ref_cycles,
        "cycles_per_ref": cycles_per_ref,
        "cycles_per_byte": cycles_per_byte,
        "instructions_per_byte": instructions_per_byte,
        "cpi": cpi,
        "perf_available": perf_available,
        "perf_source": effective_perf_source,
        "perf_unavailable_reason": perf_reason,
        "msr_aperf_mperf": msr_info,
        "placement": placement,
        "sysfs_pre": sysfs_pre,
        "sysfs_post": sysfs_post,
    }


def validate_row(row: dict[str, Any], expected_ref: dict[str, Any] | None = None) -> None:
    """Validate one timed factorial row against frozen data-integrity criteria."""
    if row.get("arm") not in (None, "mibz2"):
        raise InvalidRun(f"unexpected arm {row.get('arm')!r}")
    if row.get("err") not in (None, ""):
        raise InvalidRun(f"decode error in row: {row.get('err')!r}")
    if row.get("termination") not in (None, "success"):
        raise InvalidRun(f"non-success termination: {row.get('termination')!r}")
    wall = row.get("wall")
    if not isinstance(wall, (int, float)) or not math.isfinite(wall) or wall <= 0.0:
        raise InvalidRun(f"invalid wall time: {wall!r}")
    rss = row.get("rss_kb")
    if not isinstance(rss, (int, float)) or not math.isfinite(rss) or rss <= 0:
        raise InvalidRun(f"invalid rss_kb: {rss!r}")
    if expected_ref is not None:
        if row.get("out") != expected_ref["wall_out"]:
            raise InvalidRun(
                f"output bytes mismatch: {row.get('out')} != {expected_ref['wall_out']}"
            )
        if row.get("hash") != expected_ref["ab_hash"]:
            raise InvalidRun(
                f"output hash mismatch: {row.get('hash')} != {expected_ref['ab_hash']}"
            )


def _fast_cluster_anchor(walls: list[float], threshold: float) -> float:
    """Estimate fast-mode median robustly even when >50% of runs are in the slow mode."""
    ordered = sorted(walls)
    # 5th percentile index guards against a single freak low outlier while staying inside the fast
    # cluster even if only ~10-12.5% of runs are fast.
    seed_idx = max(0, min(len(ordered) - 1, len(ordered) // 20))
    seed = ordered[seed_idx]
    fast_vals = [w for w in ordered if w < seed * threshold]
    if not fast_vals:
        fast_vals = [ordered[0]]
    return statistics.median(fast_vals)


def _row_cpb(row: dict[str, Any]) -> float | None:
    cpb = row.get("cycles_per_byte")
    if isinstance(cpb, (int, float)) and math.isfinite(cpb) and cpb > 0:
        return float(cpb)
    cycles = row.get("cycles")
    out_b = row.get("out")
    if (
        isinstance(cycles, (int, float))
        and isinstance(out_b, (int, float))
        and math.isfinite(cycles)
        and cycles > 0
        and out_b > 0
    ):
        return float(cycles) / float(out_b)
    return None


def _row_ipb(row: dict[str, Any]) -> float | None:
    ipb = row.get("instructions_per_byte")
    if isinstance(ipb, (int, float)) and math.isfinite(ipb) and ipb > 0:
        return float(ipb)
    ins = row.get("instructions")
    out_b = row.get("out")
    if (
        isinstance(ins, (int, float))
        and isinstance(out_b, (int, float))
        and math.isfinite(ins)
        and ins > 0
        and out_b > 0
    ):
        return float(ins) / float(out_b)
    return None


def _row_cpi(row: dict[str, Any]) -> float | None:
    cpi = row.get("cpi")
    if isinstance(cpi, (int, float)) and math.isfinite(cpi) and cpi > 0:
        return float(cpi)
    cycles = row.get("cycles")
    ins = row.get("instructions")
    if (
        isinstance(cycles, (int, float))
        and isinstance(ins, (int, float))
        and math.isfinite(cycles)
        and math.isfinite(ins)
        and cycles > 0
        and ins > 0
    ):
        return float(cycles) / float(ins)
    return None


def _row_cpr(row: dict[str, Any]) -> float | None:
    cpr = row.get("cycles_per_ref")
    if isinstance(cpr, (int, float)) and math.isfinite(cpr) and cpr > 0:
        return float(cpr)
    cycles = row.get("cycles")
    ref_c = row.get("ref_cycles")
    if (
        isinstance(cycles, (int, float))
        and isinstance(ref_c, (int, float))
        and math.isfinite(cycles)
        and math.isfinite(ref_c)
        and cycles > 0
        and ref_c > 0
    ):
        return float(cycles) / float(ref_c)
    return None


def classify_thread_rows(rows: list[dict[str, Any]], threads: int) -> dict[str, Any]:
    """Classify rows for a single thread count into fast / slow_clock / slow_work / slow_mixed.

    Also computes sub-diagnostics on `slow_work` runs (`slow_cpi` vs `slow_instructions`) via
    `instructions_per_byte`, `cpi = cycles / instructions`, and `cycles_per_ref` so uncore-clock
    or memory-stall slowdowns are separated from extra executed instructions without altering
    the frozen C2 verdict thresholds.
    """
    threshold = wall_slow_threshold(threads)
    walls = [float(r["wall"]) for r in rows]
    fast_wall_med = _fast_cluster_anchor(walls, threshold)

    wall_slow_flags = [float(r["wall"]) >= fast_wall_med * threshold for r in rows]
    fast_cpbs = [
        cpb
        for r, is_slow in zip(rows, wall_slow_flags, strict=True)
        if not is_slow and (cpb := _row_cpb(r)) is not None
    ]
    fast_ipbs = [
        ipb
        for r, is_slow in zip(rows, wall_slow_flags, strict=True)
        if not is_slow and (ipb := _row_ipb(r)) is not None
    ]
    fast_cpis = [
        cpi
        for r, is_slow in zip(rows, wall_slow_flags, strict=True)
        if not is_slow and (cpi := _row_cpi(r)) is not None
    ]
    fast_cprs = [
        cpr
        for r, is_slow in zip(rows, wall_slow_flags, strict=True)
        if not is_slow and (cpr := _row_cpr(r)) is not None
    ]
    fast_cpb_med = statistics.median(fast_cpbs) if fast_cpbs else None
    fast_ipb_med = statistics.median(fast_ipbs) if fast_ipbs else None
    fast_cpi_med = statistics.median(fast_cpis) if fast_cpis else None
    fast_cpr_med = statistics.median(fast_cprs) if fast_cprs else None
    perf_covered = sum(1 for r in rows if _row_cpb(r) is not None)

    classified: list[dict[str, Any]] = []
    counts = defaultdict(int)
    work_sub_counts = defaultdict(int)
    for r, is_wall_slow in zip(rows, wall_slow_flags, strict=True):
        wall_ratio = float(r["wall"]) / fast_wall_med
        cpb = _row_cpb(r)
        ipb = _row_ipb(r)
        cpi = _row_cpi(r)
        cpr = _row_cpr(r)
        cpb_ratio = (cpb / fast_cpb_med) if (cpb is not None and fast_cpb_med) else None
        ipb_ratio = (ipb / fast_ipb_med) if (ipb is not None and fast_ipb_med) else None
        cpi_ratio = (cpi / fast_cpi_med) if (cpi is not None and fast_cpi_med) else None
        cpr_ratio = (cpr / fast_cpr_med) if (cpr is not None and fast_cpr_med) else None

        work_subclass: str | None = None
        if not is_wall_slow:
            mode = "fast"
        elif cpb_ratio is None:
            mode = "slow_unclassified"
        elif abs(cpb_ratio - 1.0) <= CYCLES_PER_BYTE_TOL:
            mode = "slow_clock"
        elif cpb_ratio >= REAL_WORK_CYCLES_RATIO:
            mode = "slow_work"
            if ipb_ratio is None:
                work_subclass = "slow_work_no_instructions"
            elif ipb_ratio >= EXTRA_INSTRUCTIONS_RATIO:
                work_subclass = "slow_instructions"
            elif abs(ipb_ratio - 1.0) <= INSTRUCTIONS_PER_BYTE_TOL:
                work_subclass = "slow_cpi"
            else:
                work_subclass = "slow_work_other"
            work_sub_counts[work_subclass] += 1
        else:
            mode = "slow_mixed"
        counts[mode] += 1
        classified.append(
            {
                "row": r,
                "mode": mode,
                "work_subclass": work_subclass,
                "is_wall_slow": is_wall_slow,
                "wall_ratio": wall_ratio,
                "cpb": cpb,
                "cpb_ratio": cpb_ratio,
                "ipb": ipb,
                "ipb_ratio": ipb_ratio,
                "cpi": cpi,
                "cpi_ratio": cpi_ratio,
                "cpr": cpr,
                "cpr_ratio": cpr_ratio,
            }
        )

    return {
        "threads": threads,
        "wall_slow_threshold": threshold,
        "fast_wall_median_s": fast_wall_med,
        "fast_cpb_median": fast_cpb_med,
        "fast_ipb_median": fast_ipb_med,
        "fast_cpi_median": fast_cpi_med,
        "fast_cpr_median": fast_cpr_med,
        "perf_covered_runs": perf_covered,
        "total_runs": len(rows),
        "mode_counts": {
            "fast": counts["fast"],
            "slow_clock": counts["slow_clock"],
            "slow_work": counts["slow_work"],
            "slow_mixed": counts["slow_mixed"],
            "slow_unclassified": counts["slow_unclassified"],
        },
        "work_sub_counts": {
            "slow_cpi": work_sub_counts["slow_cpi"],
            "slow_instructions": work_sub_counts["slow_instructions"],
            "slow_work_other": work_sub_counts["slow_work_other"],
            "slow_work_no_instructions": work_sub_counts["slow_work_no_instructions"],
        },
        "classified": classified,
    }


def _eta_squared_log_wall(classified: list[dict[str, Any]], key_fn: Any) -> float:
    """Compute one-way ANOVA eta^2 on log(wall) grouped by `key_fn(row)`."""
    vals = [math.log(float(item["row"]["wall"])) for item in classified]
    if len(vals) < 2:
        return 0.0
    grand = statistics.fmean(vals)
    ss_tot = sum((v - grand) ** 2 for v in vals)
    if ss_tot <= 0.0:
        return 0.0
    groups: dict[Any, list[float]] = defaultdict(list)
    for item, v in zip(classified, vals, strict=True):
        groups[key_fn(item["row"])].append(v)
    ss_between = sum(len(g) * (statistics.fmean(g) - grand) ** 2 for g in groups.values())
    return ss_between / ss_tot


def evaluate_factors(
    classified: list[dict[str, Any]], target_mode: str = "slow_clock"
) -> dict[str, Any]:
    """Evaluate single factors and two-way interactions against the >=80% explanation criterion."""
    factors = ["pinning", "settle_s", "pair_pos"]
    prespin_levels = {item["row"].get("prespin_ms", 0) for item in classified}
    if len(prespin_levels) > 1:
        factors.append("prespin_ms")

    total_target = sum(1 for item in classified if item["mode"] == target_mode)
    total_wall_slow = sum(1 for item in classified if item["is_wall_slow"])

    def _eval_candidates(factor_tuples: list[tuple[str, ...]]) -> list[dict[str, Any]]:
        results = []
        for fnames in factor_tuples:
            by_level: dict[tuple[Any, ...], list[dict[str, Any]]] = defaultdict(list)
            for item in classified:
                key = tuple(item["row"].get(f) for f in fnames)
                by_level[key].append(item)
            eta2 = _eta_squared_log_wall(
                classified, lambda row, fs=fnames: tuple(row.get(f) for f in fs)
            )
            for level, members in sorted(by_level.items()):
                n_target = sum(1 for m in members if m["mode"] == target_mode)
                n_wall_slow = sum(1 for m in members if m["is_wall_slow"])
                share = (n_target / total_target) if total_target > 0 else 0.0
                wall_slow_share = (n_wall_slow / total_wall_slow) if total_wall_slow > 0 else 0.0
                slow_rate = n_target / len(members) if members else 0.0
                other_n = len(classified) - len(members)
                other_rate = (total_target - n_target) / other_n if other_n > 0 else 0.0
                label = " & ".join(f"{f}={v}" for f, v in zip(fnames, level, strict=True))
                results.append(
                    {
                        "factors": list(fnames),
                        "level": [str(v) for v in level],
                        "label": label,
                        "order": len(fnames),
                        "n_in_cell": len(members),
                        "n_target": n_target,
                        "explained_share": round(share, 4),
                        "wall_slow_share": round(wall_slow_share, 4),
                        "slow_rate": round(slow_rate, 4),
                        "complement_slow_rate": round(other_rate, 4),
                        "eta_squared_log_wall": round(eta2, 4),
                        "median_wall_s": round(
                            statistics.median(float(m["row"]["wall"]) for m in members), 4
                        ),
                    }
                )
        results.sort(
            key=lambda x: (
                x["explained_share"],
                x["slow_rate"],
                -x["order"],
                x["eta_squared_log_wall"],
            ),
            reverse=True,
        )
        return results

    single_tuples = [(f,) for f in factors]
    twoway_tuples = [
        (factors[i], factors[j]) for i in range(len(factors)) for j in range(i + 1, len(factors))
    ]
    singles = _eval_candidates(single_tuples)
    twoways = _eval_candidates(twoway_tuples)
    all_candidates = singles + twoways
    qualifying = [
        c
        for c in all_candidates
        if c["explained_share"] >= EXPLAIN_THRESHOLD and c["slow_rate"] > c["complement_slow_rate"]
    ]
    # Prefer highest slow_rate (specificity) among qualifying rules, then share & simplicity.
    qualifying.sort(
        key=lambda x: (x["slow_rate"], x["explained_share"], -x["order"]),
        reverse=True,
    )
    return {
        "total_target": total_target,
        "single_factors": singles,
        "two_way_interactions": twoways,
        "best_explainer": qualifying[0] if qualifying else None,
        "best_single": singles[0] if singles else None,
        "best_two_way": twoways[0] if twoways else None,
    }


def _cell_table(classified: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Build per-(pinning, settle_s, prespin_ms, pair_pos) summary table."""
    groups: dict[tuple[Any, ...], list[dict[str, Any]]] = defaultdict(list)
    for item in classified:
        r = item["row"]
        key = (r["pinning"], float(r["settle_s"]), int(r.get("prespin_ms", 0)), int(r["pair_pos"]))
        groups[key].append(item)

    table = []
    for (pinning, settle_s, prespin_ms, pair_pos), items in sorted(groups.items()):
        walls = [float(i["row"]["wall"]) for i in items]
        cpus = [float(i["row"]["cpu"]) for i in items if i["row"].get("cpu") is not None]
        cpbs = [i["cpb"] for i in items if i["cpb"] is not None]
        ipbs = [i["ipb"] for i in items if i["ipb"] is not None]
        cpis = [i["cpi"] for i in items if i["cpi"] is not None]
        cprs = [i["cpr"] for i in items if i["cpr"] is not None]
        distinct_cpus = [
            i["row"]["placement"]["n_distinct_cpus"]
            for i in items
            if isinstance(i["row"].get("placement"), dict)
            and i["row"]["placement"].get("n_distinct_cpus") is not None
        ]
        smt_overlaps = [
            i["row"]["placement"]["smt_sibling_overlap_samples"]
            for i in items
            if isinstance(i["row"].get("placement"), dict)
            and i["row"]["placement"].get("smt_sibling_overlap_samples") is not None
        ]
        table.append(
            {
                "pinning": pinning,
                "settle_s": settle_s,
                "prespin_ms": prespin_ms,
                "pair_pos": pair_pos,
                "n": len(items),
                "median_wall_s": round(statistics.median(walls), 4),
                "min_wall_s": round(min(walls), 4),
                "max_wall_s": round(max(walls), 4),
                "median_cpu_s": round(statistics.median(cpus), 4) if cpus else None,
                "median_cpb": round(statistics.median(cpbs), 4) if cpbs else None,
                "median_ipb": round(statistics.median(ipbs), 4) if ipbs else None,
                "median_cpi": round(statistics.median(cpis), 4) if cpis else None,
                "median_cycles_per_ref": round(statistics.median(cprs), 4) if cprs else None,
                "n_fast": sum(1 for i in items if i["mode"] == "fast"),
                "n_slow_clock": sum(1 for i in items if i["mode"] == "slow_clock"),
                "n_slow_work": sum(1 for i in items if i["mode"] == "slow_work"),
                "n_slow_cpi": sum(1 for i in items if i["work_subclass"] == "slow_cpi"),
                "n_slow_instructions": sum(
                    1 for i in items if i["work_subclass"] == "slow_instructions"
                ),
                "n_slow_mixed": sum(1 for i in items if i["mode"] == "slow_mixed"),
                "n_slow_unclassified": sum(1 for i in items if i["mode"] == "slow_unclassified"),
                "median_distinct_cpus": statistics.median(distinct_cpus) if distinct_cpus else None,
                "total_smt_overlap_samples": sum(smt_overlaps) if smt_overlaps else 0,
            }
        )
    return table


def score(
    rows: list[dict[str, Any]],
    expected_ref: dict[str, Any] | None = None,
    *,
    host_ok: bool = True,
    rounds: int = ROUNDS,
    threads: tuple[int, ...] = THREADS,
    pinnings: tuple[str, ...] = PINNINGS,
    settles: tuple[float, ...] = SETTLES,
    prespin_ms_values: tuple[int, ...] = DEFAULT_PRESPIN_MS,
    min_slow_runs: int = MIN_SLOW_RUNS,
) -> dict[str, Any]:
    """Score factorial runs under the frozen i9-clock-modes stop_test."""
    if not host_ok:
        return {"verdict": "VOID", "reason": "non-i9 or invalid workstation host"}

    seen: set[tuple[Any, ...]] = set()
    by_threads: dict[int, list[dict[str, Any]]] = {t: [] for t in threads}
    valid_settles = {round(float(s), 4) for s in settles}
    valid_prespins = {int(p) for p in prespin_ms_values}

    for row in rows:
        t = row.get("threads")
        pin = row.get("pinning")
        settle = row.get("settle_s")
        pos = row.get("pair_pos")
        r = row.get("round")
        prespin = int(row.get("prespin_ms", 0))
        if t not in by_threads:
            return {"verdict": "VOID", "reason": f"unexpected thread count {t!r}"}
        if pin not in pinnings:
            return {"verdict": "VOID", "reason": f"unexpected pinning {pin!r}"}
        if not isinstance(settle, (int, float)) or round(float(settle), 4) not in valid_settles:
            return {"verdict": "VOID", "reason": f"unexpected settle_s {settle!r}"}
        if prespin not in valid_prespins:
            return {"verdict": "VOID", "reason": f"unexpected prespin_ms {prespin!r}"}
        if pos not in PAIR_POSITIONS:
            return {"verdict": "VOID", "reason": f"unexpected pair_pos {pos!r}"}
        if not isinstance(r, int) or not (0 <= r < rounds):
            return {"verdict": "VOID", "reason": f"invalid round index {r!r}"}
        key = (r, t, pin, round(float(settle), 4), prespin, pos)
        if key in seen:
            return {"verdict": "VOID", "reason": f"duplicate factorial cell {key}"}
        try:
            validate_row(row, expected_ref)
        except InvalidRun as exc:
            return {"verdict": "VOID", "reason": str(exc)}
        seen.add(key)
        by_threads[t].append(row)

    expected_total = (
        rounds
        * len(threads)
        * len(pinnings)
        * len(settles)
        * len(prespin_ms_values)
        * len(PAIR_POSITIONS)
    )
    if len(seen) != expected_total:
        return {
            "verdict": "VOID",
            "reason": f"incomplete factorial sweep: {len(seen)} != {expected_total}",
        }

    per_thread: dict[str, Any] = {}
    mechanism_hits: list[str] = []
    work_hits: list[str] = []

    for t in threads:
        cls = classify_thread_rows(by_threads[t], t)
        mc = cls["mode_counts"]
        wsc = cls["work_sub_counts"]
        factor_eval = evaluate_factors(cls["classified"], target_mode="slow_clock")
        wall_slow_items = [i for i in cls["classified"] if i["is_wall_slow"]]
        wall_slow_cpb_ratios = [
            i["cpb_ratio"] for i in wall_slow_items if i["cpb_ratio"] is not None
        ]
        wall_slow_ipb_ratios = [
            i["ipb_ratio"] for i in wall_slow_items if i["ipb_ratio"] is not None
        ]
        wall_slow_cpi_ratios = [
            i["cpi_ratio"] for i in wall_slow_items if i["cpi_ratio"] is not None
        ]
        wall_slow_cpr_ratios = [
            i["cpr_ratio"] for i in wall_slow_items if i["cpr_ratio"] is not None
        ]
        median_slow_cpb_ratio = (
            round(statistics.median(wall_slow_cpb_ratios), 4) if wall_slow_cpb_ratios else None
        )
        median_slow_ipb_ratio = (
            round(statistics.median(wall_slow_ipb_ratios), 4) if wall_slow_ipb_ratios else None
        )
        median_slow_cpi_ratio = (
            round(statistics.median(wall_slow_cpi_ratios), 4) if wall_slow_cpi_ratios else None
        )
        median_slow_cpr_ratio = (
            round(statistics.median(wall_slow_cpr_ratios), 4) if wall_slow_cpr_ratios else None
        )

        # Sub-diagnosis for slow_work runs: separate uncore/memory-stall CPI inflation from
        # extra executed instructions (e.g. spin-loop/retry work).
        n_slow_work = mc["slow_work"]
        work_diagnosis: str | None = None
        if n_slow_work > 0:
            if wsc["slow_cpi"] >= wsc["slow_instructions"] and wsc["slow_cpi"] / n_slow_work >= 0.5:
                work_diagnosis = "uncore_or_memory_stall"
            elif wsc["slow_instructions"] / n_slow_work >= 0.5:
                work_diagnosis = "extra_instructions"
            else:
                work_diagnosis = "mixed_or_unavailable"

        # Evaluate per-thread verdict under the frozen stop test.
        best = factor_eval["best_explainer"]
        n_slow_clock = mc["slow_clock"]
        n_wall_slow = len(wall_slow_items)
        if n_slow_clock >= min_slow_runs and best is not None:
            t_verdict = "PASS (mechanism found)"
            rate_pct = best["slow_rate"] * 100
            comp_pct = best["complement_slow_rate"] * 100
            t_reason = (
                f"{t}T: {best['label']} explains {best['n_target']}/{n_slow_clock} "
                f"({best['explained_share'] * 100:.1f}%) of slow_clock runs "
                f"(rate {rate_pct:.1f}% vs {comp_pct:.1f}%, "
                f"med cpr_ratio={median_slow_cpr_ratio})"
            )
            mechanism_hits.append(t_reason)
        elif n_wall_slow >= min_slow_runs and (
            n_slow_work >= min_slow_runs
            and (
                (
                    median_slow_cpb_ratio is not None
                    and median_slow_cpb_ratio >= REAL_WORK_CYCLES_RATIO
                )
                or n_slow_work / n_wall_slow >= 0.5
            )
        ):
            t_verdict = "PASS (mode is real work)"
            t_reason = (
                f"{t}T: {n_slow_work}/{n_wall_slow} wall-slow runs have >=1.2x cycles "
                f"(median cpb_ratio={median_slow_cpb_ratio}, "
                f"ipb_ratio={median_slow_ipb_ratio}, cpi_ratio={median_slow_cpi_ratio}; "
                f"sub={work_diagnosis}: slow_cpi={wsc['slow_cpi']}, "
                f"slow_instructions={wsc['slow_instructions']})"
            )
            work_hits.append(t_reason)
        else:
            t_verdict = "STOP"
            if cls["perf_covered_runs"] == 0 and n_wall_slow > 0:
                t_reason = (
                    f"{t}T: {n_wall_slow} wall-slow runs observed but PMU cycles unavailable "
                    "(cannot separate frequency from work)"
                )
            elif n_wall_slow < min_slow_runs:
                t_reason = (
                    f"{t}T: unimodal / fewer than {min_slow_runs} slow runs "
                    f"(wall_slow={n_wall_slow}/{cls['total_runs']})"
                )
            else:
                top = factor_eval["best_two_way"] or factor_eval["best_single"]
                top_desc = (
                    f"top candidate {top['label']} explains {top['explained_share'] * 100:.1f}%"
                    if top
                    else "no candidate"
                )
                t_reason = (
                    f"{t}T: no factor/two-way interaction explains >=80% of {n_slow_clock} "
                    f"slow_clock runs ({top_desc}); slow_work={n_slow_work}/{n_wall_slow}"
                )

        per_thread[str(t)] = {
            "threads": t,
            "verdict": t_verdict,
            "reason": t_reason,
            "wall_slow_threshold": cls["wall_slow_threshold"],
            "fast_wall_median_s": round(cls["fast_wall_median_s"], 4),
            "fast_cpb_median": round(cls["fast_cpb_median"], 6)
            if cls["fast_cpb_median"] is not None
            else None,
            "fast_ipb_median": round(cls["fast_ipb_median"], 6)
            if cls["fast_ipb_median"] is not None
            else None,
            "fast_cpi_median": round(cls["fast_cpi_median"], 4)
            if cls["fast_cpi_median"] is not None
            else None,
            "fast_cpr_median": round(cls["fast_cpr_median"], 4)
            if cls["fast_cpr_median"] is not None
            else None,
            "median_slow_cpb_ratio": median_slow_cpb_ratio,
            "median_slow_ipb_ratio": median_slow_ipb_ratio,
            "median_slow_cpi_ratio": median_slow_cpi_ratio,
            "median_slow_cpr_ratio": median_slow_cpr_ratio,
            "perf_covered_runs": cls["perf_covered_runs"],
            "total_runs": cls["total_runs"],
            "mode_counts": mc,
            "work_sub_counts": wsc,
            "work_diagnosis": work_diagnosis,
            "best_explainer": factor_eval["best_explainer"],
            "best_single": factor_eval["best_single"],
            "best_two_way": factor_eval["best_two_way"],
            "single_factors": factor_eval["single_factors"],
            "two_way_interactions": factor_eval["two_way_interactions"],
            "cells": _cell_table(cls["classified"]),
        }

    if mechanism_hits:
        verdict = "PASS (mechanism found)"
        reason = "; ".join(mechanism_hits + work_hits)
    elif work_hits:
        verdict = "PASS (mode is real work)"
        reason = "; ".join(work_hits)
    else:
        verdict = "STOP"
        reason = "; ".join(per_thread[str(t)]["reason"] for t in threads)

    return {
        "verdict": verdict,
        "reason": reason,
        "rounds": rounds,
        "threads": list(threads),
        "pinnings": list(pinnings),
        "settles": [float(s) for s in settles],
        "prespin_ms_values": [int(p) for p in prespin_ms_values],
        "min_slow_runs": min_slow_runs,
        "thresholds": {
            "wall_slow_ratio": {str(t): wall_slow_threshold(t) for t in threads},
            "cycles_per_byte_tol": CYCLES_PER_BYTE_TOL,
            "real_work_cycles_ratio": REAL_WORK_CYCLES_RATIO,
            "instructions_per_byte_tol": INSTRUCTIONS_PER_BYTE_TOL,
            "extra_instructions_ratio": EXTRA_INSTRUCTIONS_RATIO,
            "explain_threshold": EXPLAIN_THRESHOLD,
        },
        "per_thread": per_thread,
    }


def format_summary(summary: dict[str, Any]) -> str:
    """Format a human-readable outcome and per-cell table."""
    if summary.get("verdict") == "VOID":
        return f"VOID: {summary.get('reason', 'unknown')}\n"
    lines = [
        f"{summary['verdict']}: {summary['reason']}",
        (
            f"Design: rounds={summary['rounds']} threads={summary['threads']} "
            f"pinnings={summary['pinnings']} settles={summary['settles']} "
            f"prespin_ms={summary['prespin_ms_values']}"
        ),
    ]
    for t in summary["threads"]:
        pt = summary["per_thread"][str(t)]
        mc = pt["mode_counts"]
        wsc = pt["work_sub_counts"]
        perf_cov = f"{pt['perf_covered_runs']}/{pt['total_runs']}"
        lines.append("")
        lines.append(
            f"=== {t}T ({pt['verdict']}) fast_wall={pt['fast_wall_median_s']:.4f}s "
            f"thr={pt['wall_slow_threshold']:.2f}x perf={perf_cov} "
            f"modes=fast:{mc['fast']},clock:{mc['slow_clock']},work:{mc['slow_work']}"
            f"(cpi:{wsc['slow_cpi']},ins:{wsc['slow_instructions']}),"
            f"mixed:{mc['slow_mixed']},uncls:{mc['slow_unclassified']} ==="
        )
        lines.append(f"  {pt['reason']}")
        lines.append(
            "  pinning   settle  prespin  pos   n   med_wall   min..max        "
            "med_cpr  med_cpi  med_ipb  fast/clk/cpi/ins/unc  cpus  smt_ovlp"
        )
        for c in pt["cells"]:
            cpr = f"{c['median_cycles_per_ref']:.3f}" if c["median_cycles_per_ref"] else "  -  "
            cpi_s = f"{c['median_cpi']:.3f}" if c["median_cpi"] else "  -  "
            ipb_s = f"{c['median_ipb']:.2f}" if c["median_ipb"] else "  -  "
            dcpus = f"{c['median_distinct_cpus']:.1f}" if c["median_distinct_cpus"] else " - "
            counts_s = (
                f"{c['n_fast']:>2d}/{c['n_slow_clock']:>2d}/{c['n_slow_cpi']:>2d}/"
                f"{c['n_slow_instructions']:>2d}/{c['n_slow_unclassified']:>2d}"
            )
            lines.append(
                f"  {c['pinning']:<8s}  {c['settle_s']:<6.1f}  {c['prespin_ms']:<7d}  "
                f"{c['pair_pos']:<3d}  {c['n']:<2d}  {c['median_wall_s']:<9.4f}  "
                f"{c['min_wall_s']:.4f}..{c['max_wall_s']:.4f}  {cpr:<7s}  "
                f"{cpi_s:<7s}  {ipb_s:<7s}  {counts_s:<20s}  "
                f"{dcpus:>4s}  {c['total_smt_overlap_samples']:>8d}"
            )
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# Mode-robust estimator for ab runs.jsonl (pooled, position-stratified, within-mode)
# ---------------------------------------------------------------------------


def _log_ratio_ci(logs: list[float]) -> dict[str, Any]:
    """Compute geometric mean ratio and two-sided 95% Student-t CI from log ratios."""
    n = len(logs)
    if n == 0:
        return {"n": 0, "ratio": None, "ci_lo": None, "ci_hi": None}
    mean = statistics.fmean(logs)
    ratio = math.exp(mean)
    if n < 2:
        return {"n": n, "ratio": round(ratio, 4), "ci_lo": None, "ci_hi": None}
    sd = statistics.stdev(logs)
    half = t_crit_95(n - 1) * sd / math.sqrt(n)
    return {
        "n": n,
        "ratio": round(ratio, 4),
        "ci_lo": round(math.exp(mean - half), 4),
        "ci_hi": round(math.exp(mean + half), 4),
    }


def _unpaired_log_ratio_ci(num: list[float], den: list[float]) -> dict[str, Any]:
    """Welch two-sample 95% t-interval on log(num) - log(den) for within-mode arms."""
    n1, n2 = len(num), len(den)
    if n1 == 0 or n2 == 0:
        return {"n_arm": n1, "n_ref": n2, "ratio": None, "ci_lo": None, "ci_hi": None}
    l1 = [math.log(x) for x in num]
    l2 = [math.log(x) for x in den]
    diff = statistics.fmean(l1) - statistics.fmean(l2)
    ratio = math.exp(diff)
    if n1 < 2 or n2 < 2:
        return {
            "n_arm": n1,
            "n_ref": n2,
            "ratio": round(ratio, 4),
            "median_ratio": round(statistics.median(num) / statistics.median(den), 4),
            "ci_lo": None,
            "ci_hi": None,
        }
    v1 = statistics.variance(l1)
    v2 = statistics.variance(l2)
    se2 = v1 / n1 + v2 / n2
    if se2 <= 0.0:
        lo = hi = ratio
    else:
        num_df = se2**2
        den_df = (v1 / n1) ** 2 / (n1 - 1) + (v2 / n2) ** 2 / (n2 - 1)
        df = max(1, int(round(num_df / den_df))) if den_df > 0 else min(n1, n2) - 1
        half = t_crit_95(df) * math.sqrt(se2)
        lo = math.exp(diff - half)
        hi = math.exp(diff + half)
    return {
        "n_arm": n1,
        "n_ref": n2,
        "ratio": round(ratio, 4),
        "median_ratio": round(statistics.median(num) / statistics.median(den), 4),
        "ci_lo": round(lo, 4),
        "ci_hi": round(hi, 4),
    }


def _classify_ab_modes(
    rows: list[dict[str, Any]], threads: int
) -> tuple[dict[int, str], dict[str, Any]]:
    """Classify rows in one (corpus, threads) cell into 'fast' vs 'slow' modes.

    Prefers `cycles_per_ref` / `cycles_per_byte` + wall when hardware counters are present;
    falls back to a deterministic 1D gap/threshold split on wall when counters are absent
    (as in historical bundles like bce3914d and 77236472).
    """
    walls = [float(r["wall"]) for r in rows]
    has_cpr = all(
        isinstance(r.get("cycles_per_ref"), (int, float)) and r["cycles_per_ref"] > 0 for r in rows
    )
    has_cycles = all(_row_cpb(r) is not None for r in rows)

    if has_cpr:
        cprs = [float(r["cycles_per_ref"]) for r in rows]
        # Higher cycles_per_ref = faster turbo clock. Anchor fast clock at upper cluster.
        ordered = sorted(cprs, reverse=True)
        seed = ordered[max(0, len(ordered) // 20)]
        # Fast clock is within 4% (1T) or 15% (multi-T) of peak cycles_per_ref.
        drop_tol = 0.04 if threads <= 1 else 0.15
        labels = {
            idx: ("fast" if float(r["cycles_per_ref"]) >= seed * (1.0 - drop_tol) else "slow")
            for idx, r in enumerate(rows)
        }
        return labels, {"basis": "cycles_per_ref", "fast_cpr_anchor": round(seed, 4)}

    # Determine wall split threshold: check both the registered threshold and largest relative gap
    # so 1T (~1.04-1.10x), 6T (~1.45x), and 12T (~1.08-1.25x) bimodal mixtures separate cleanly.
    reg_thr = wall_slow_threshold(threads)
    fast_med = _fast_cluster_anchor(walls, reg_thr)
    # Also detect clean bimodal gap (>= 4% jump in sorted walls) when wall clusters are tighter
    # than 1.30x (e.g. 12T release-scaling 0.060 vs 0.065 vs 0.076).
    ordered_w = sorted(walls)
    max_gap_ratio = 1.0
    gap_split = fast_med * reg_thr
    for i in range(len(ordered_w) - 1):
        ratio = ordered_w[i + 1] / ordered_w[i]
        if ratio > max_gap_ratio:
            max_gap_ratio = ratio
            gap_split = math.sqrt(ordered_w[i] * ordered_w[i + 1])

    use_split = fast_med * reg_thr
    if all(w < use_split for w in walls) and max_gap_ratio >= 1.04:
        use_split = gap_split

    labels: dict[int, str] = {}
    if has_cycles:
        fast_cpbs = [_row_cpb(r) for r in rows if float(r["wall"]) < use_split]
        fast_cpb_med = (
            statistics.median(c for c in fast_cpbs if c is not None) if fast_cpbs else None
        )
        for idx, r in enumerate(rows):
            if float(r["wall"]) < use_split:
                labels[idx] = "fast"
            else:
                cpb = _row_cpb(r)
                if (
                    cpb is not None
                    and fast_cpb_med
                    and abs(cpb / fast_cpb_med - 1.0) <= CYCLES_PER_BYTE_TOL
                ):
                    labels[idx] = "slow"
                else:
                    labels[idx] = "slow_work"
        basis = "cycles+wall"
    else:
        for idx, r in enumerate(rows):
            labels[idx] = "fast" if float(r["wall"]) < use_split else "slow"
        basis = "wall_only"

    return labels, {
        "basis": basis,
        "fast_wall_anchor_s": round(fast_med, 4),
        "split_wall_s": round(use_split, 4),
        "max_adjacent_gap_ratio": round(max_gap_ratio, 4),
    }


def analyze_ab_runs(
    rows: list[dict[str, Any]],
    *,
    ref_arm: str | None = None,
    target_arm: str | None = None,
    threads_filter: set[int] | None = None,
    metric: str = "wall",
) -> dict[str, Any]:
    """Run pooled, position-stratified, and within-mode estimators on any `ab` runs.jsonl.

    Ratio convention: `arm / ref_arm` (values < 1.0 mean `arm` is faster than `ref_arm`;
    pass `--ref <new> --arm <old>` for `old / new` release-scaling speedup convention).
    """
    # Filter to valid rows and infer pair_pos from appearance order within (round, corpus, threads).
    clean_rows = [
        r
        for r in rows
        if r.get("err") in (None, "")
        and isinstance(r.get(metric), (int, float))
        and math.isfinite(float(r[metric]))
        and float(r[metric]) > 0.0
    ]

    pos_counter: dict[tuple[Any, Any, Any], int] = defaultdict(int)
    annotated: list[dict[str, Any]] = []
    for r in clean_rows:
        gkey = (r.get("round"), r.get("corpus"), r.get("threads"))
        pos = r["pair_pos"] if "pair_pos" in r else pos_counter[gkey]
        pos_counter[gkey] += 1
        annotated.append({**r, "_inferred_pos": pos})

    arms_present = sorted({str(r["arm"]) for r in annotated if "arm" in r})
    if not arms_present:
        return {"error": "no arms found", "cells": []}

    # Choose default ref_arm and candidate arms cleanly.
    if ref_arm is None:
        ref_arm = "mibz2" if "mibz2" in arms_present else arms_present[0]
    if target_arm is not None:
        compare_arms = [target_arm]
    else:
        compare_arms = [a for a in arms_present if a != ref_arm]

    grouped: dict[tuple[str, int], list[dict[str, Any]]] = defaultdict(list)
    for r in annotated:
        t = int(r["threads"])
        if threads_filter and t not in threads_filter:
            continue
        grouped[(str(r.get("corpus", "unknown")), t)].append(r)

    cells: list[dict[str, Any]] = []
    for (corpus, t), cell_rows in sorted(grouped.items()):
        for arm in compare_arms:
            pair_rows = [r for r in cell_rows if r.get("arm") in (ref_arm, arm)]
            ref_rows = [r for r in pair_rows if r.get("arm") == ref_arm]
            arm_rows = [r for r in pair_rows if r.get("arm") == arm]
            if not ref_rows or not arm_rows:
                continue

            # 1. Pooled paired ratio over rounds.
            ref_by_round = {r["round"]: r for r in ref_rows}
            arm_by_round = {r["round"]: r for r in arm_rows}
            common_rounds = sorted(set(ref_by_round) & set(arm_by_round))
            pooled_logs = [
                math.log(float(arm_by_round[rd][metric]) / float(ref_by_round[rd][metric]))
                for rd in common_rounds
            ]
            pooled = _log_ratio_ci(pooled_logs)

            # 2. Position-stratified estimator: match arm & ref within the same pair position.
            positions = sorted({int(r["_inferred_pos"]) for r in pair_rows})
            by_pos: dict[str, Any] = {}
            strat_weights: list[tuple[float, float]] = []
            for p in positions:
                a_vals = [float(r[metric]) for r in arm_rows if int(r["_inferred_pos"]) == p]
                b_vals = [float(r[metric]) for r in ref_rows if int(r["_inferred_pos"]) == p]
                est = _unpaired_log_ratio_ci(a_vals, b_vals)
                by_pos[str(p)] = est
                if est["ratio"] is not None:
                    w = min(len(a_vals), len(b_vals))
                    strat_weights.append((math.log(est["ratio"]), float(w)))
            pos_combined = (
                round(
                    math.exp(
                        sum(v * w for v, w in strat_weights) / sum(w for _, w in strat_weights)
                    ),
                    4,
                )
                if strat_weights
                else None
            )

            # 3. Within-mode estimator (cycles-classified when present, wall-split fallback).
            mode_labels, mode_meta = _classify_ab_modes(pair_rows, t)
            arm_by_mode: dict[str, list[float]] = defaultdict(list)
            ref_by_mode: dict[str, list[float]] = defaultdict(list)
            arm_round_mode: dict[Any, str] = {}
            ref_round_mode: dict[Any, str] = {}
            for idx, r in enumerate(pair_rows):
                m = mode_labels[idx]
                if r["arm"] == arm:
                    arm_by_mode[m].append(float(r[metric]))
                    arm_round_mode[r["round"]] = m
                else:
                    ref_by_mode[m].append(float(r[metric]))
                    ref_round_mode[r["round"]] = m

            within_mode: dict[str, Any] = {}
            for m in ("fast", "slow", "slow_work"):
                a_m = arm_by_mode.get(m, [])
                r_m = ref_by_mode.get(m, [])
                if a_m or r_m:
                    within_mode[m] = _unpaired_log_ratio_ci(a_m, r_m)

            same_mode_logs = [
                math.log(float(arm_by_round[rd][metric]) / float(ref_by_round[rd][metric]))
                for rd in common_rounds
                if arm_round_mode.get(rd) == ref_round_mode.get(rd)
            ]
            same_mode_paired = _log_ratio_ci(same_mode_logs)

            cells.append(
                {
                    "corpus": corpus,
                    "threads": t,
                    "arm": arm,
                    "ref_arm": ref_arm,
                    "metric": metric,
                    "pooled": pooled,
                    "position_stratified": {
                        "combined_ratio": pos_combined,
                        "by_pos": by_pos,
                    },
                    "within_mode": {
                        **mode_meta,
                        "arm_mode_counts": {k: len(v) for k, v in sorted(arm_by_mode.items())},
                        "ref_mode_counts": {k: len(v) for k, v in sorted(ref_by_mode.items())},
                        "by_mode": within_mode,
                        "same_mode_paired": same_mode_paired,
                    },
                }
            )

    return {"ref_arm": ref_arm, "metric": metric, "cells": cells}


def format_analysis(result: dict[str, Any]) -> str:
    """Format mode-robust analysis output cleanly for humans."""
    lines = [
        f"Mode-robust ab analysis (metric={result.get('metric')}, ref={result.get('ref_arm')})"
    ]
    for c in result.get("cells", []):
        p = c["pooled"]
        ps = c["position_stratified"]
        wm = c["within_mode"]
        fast = wm["by_mode"].get("fast", {})
        slow = wm["by_mode"].get("slow", {})
        sm = wm["same_mode_paired"]
        lines.append(f"\n[{c['corpus']} t={c['threads']}] {c['arm']} / {c['ref_arm']}:")
        lines.append(
            f"  pooled paired:       {p['ratio']} [{p['ci_lo']}, {p['ci_hi']}] (n={p['n']})"
        )
        lines.append(
            f"  position-stratified: combined={ps['combined_ratio']} by_pos={ps['by_pos']}"
        )
        lines.append(
            f"  mode occupancy ({wm['basis']}): "
            f"arm={wm['arm_mode_counts']} ref={wm['ref_mode_counts']}"
        )
        f_ci = f"[{fast.get('ci_lo')}, {fast.get('ci_hi')}]"
        s_ci = f"[{slow.get('ci_lo')}, {slow.get('ci_hi')}]"
        lines.append(
            f"  within-fast:         {fast.get('ratio')} {f_ci} "
            f"(n={fast.get('n_arm', 0)}/{fast.get('n_ref', 0)})"
        )
        lines.append(
            f"  within-slow:         {slow.get('ratio')} {s_ci} "
            f"(n={slow.get('n_arm', 0)}/{slow.get('n_ref', 0)})"
        )
        lines.append(
            f"  same-mode paired:    {sm['ratio']} [{sm['ci_lo']}, {sm['ci_hi']}] (n={sm['n']})"
        )
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# End-to-end driver (`run`) and CLI entry point
# ---------------------------------------------------------------------------


def make_smoke_corpus(tmp: Path) -> dict[str, Any]:
    """Generate a compact deterministic multistream .bz2 corpus for local `--smoke` runs."""
    chunk = (b"mibz2 clock-modes smoke payload: " + bytes(range(32, 127)) + b"\n") * 8000
    compressed = bz2.compress(chunk, compresslevel=9)
    wall_path = tmp / "smoke.bz2"
    wall_path.write_bytes(compressed * 4)
    digest = sha256_file(wall_path)
    return {
        "name": "smoke",
        "base_path": wall_path,
        "wall_path": wall_path,
        "base_bz2_sha256": digest,
        "wall_bz2_sha256": digest,
    }


def run_probe(
    out: Path,
    *,
    smoke: bool = False,
    allow_any_host: bool = False,
    corpus_path: Path | None = None,
    ab_bin_path: Path | None = None,
    rounds: int | None = None,
    threads: tuple[int, ...] = THREADS,
    settles: tuple[float, ...] = SETTLES,
    prespin_ms_values: tuple[int, ...] = DEFAULT_PRESPIN_MS,
) -> dict[str, Any]:
    """Execute the full frozen i9-clock-modes probe (or local `--smoke` run)."""
    out = Path(out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    strict_i9 = not (smoke or allow_any_host)
    eff_rounds = rounds if rounds is not None else (2 if smoke else ROUNDS)
    eff_settles = (0.0, 0.05) if (smoke and settles == SETTLES) else settles
    eff_min_slow = 1 if smoke else MIN_SLOW_RUNS

    if strict_i9:
        check_baseline_env()
    cores = discover_cores(out, require_i9=strict_i9)

    if strict_i9:
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

    save_json(
        out / "provenance.json",
        {
            "head": output("git", "rev-parse", "HEAD"),
            "head_src_tree": output("git", "rev-parse", "HEAD:src"),
            "head_cargo_toml": output("git", "rev-parse", "HEAD:Cargo.toml"),
            "head_cargo_lock": output("git", "rev-parse", "HEAD:Cargo.lock"),
            "head_ab_rs": output("git", "rev-parse", "HEAD:examples/ab.rs"),
            "rustc": toolchain,
            "seed": SEED,
            "smoke": smoke,
            "rounds": eff_rounds,
            "threads": list(threads),
            "settles": list(eff_settles),
            "prespin_ms_values": list(prespin_ms_values),
            "cores": cores,
        },
    )

    with tempfile.TemporaryDirectory(dir=out.parent) as tmpdir:
        tmp = Path(tmpdir)
        if ab_bin_path is not None:
            ab_bin = Path(ab_bin_path).resolve()
            bins: dict[str, Path] = {"ab": ab_bin}
        else:
            need_windows = corpus_path is None and not smoke
            bins = build_binaries(ROOT, tmp, include_windows=need_windows, check_env=strict_i9)
            ab_bin = bins["ab"]

        save_json(out / "binaries.json", {k: sha256_file(v) for k, v in bins.items()})
        env_facts = json.loads(output(str(ab_bin), "env", "--facts"))
        perf_backend = probe_perf_backend(ab_bin, sorted(os.sched_getaffinity(0)))
        env_facts["perf_backend"] = perf_backend
        save_json(out / "env.json", env_facts)
        if strict_i9:
            check_cpu_quota_unlimited(env_facts)

        if corpus_path is not None:
            cp = Path(corpus_path).resolve()
            digest = sha256_file(cp)
            corpus_info: dict[str, Any] = {
                "name": cp.stem,
                "base_path": cp,
                "wall_path": cp,
                "base_bz2_sha256": digest,
                "wall_bz2_sha256": digest,
            }
        elif smoke:
            corpus_info = make_smoke_corpus(tmp)
        else:
            corpus_info = materialize_wikidata(tmp, ROOT, bins["windows"])

        ref = run_parity(out, ab_bin, corpus_info)
        save_json(out / "inputs.json", ref)

        schedule = factorial_plan(
            rounds=eff_rounds,
            threads=threads,
            pinnings=PINNINGS,
            settles=eff_settles,
            prespin_ms_values=prespin_ms_values,
            seed=SEED,
        )
        rows: list[dict[str, Any]] = []
        with (out / "runs.jsonl").open("w", encoding="utf-8", buffering=1) as log:
            for item in schedule:
                row = run_one_child(
                    ab_bin, Path(corpus_info["wall_path"]), item, cores, perf_backend
                )
                row["corpus"] = corpus_info["name"]
                validate_row(row, ref)
                log.write(json.dumps(row, allow_nan=False) + "\n")
                rows.append(row)

        summary = score(
            rows,
            ref,
            host_ok=True,
            rounds=eff_rounds,
            threads=threads,
            pinnings=PINNINGS,
            settles=eff_settles,
            prespin_ms_values=prespin_ms_values,
            min_slow_runs=eff_min_slow,
        )
        save_json(out / "summary.json", summary)
        text = format_summary(summary)
        (out / "summary.txt").write_text(text, encoding="utf-8")
        (out / "outcome.txt").write_text(
            f"{summary['verdict']}: {summary['reason']}\n", encoding="utf-8"
        )
        if summary["verdict"] == "VOID":
            raise SystemExit(1)
        return summary


def _parse_int_tuple(spec: str) -> tuple[int, ...]:
    vals = tuple(int(x.strip()) for x in spec.split(",") if x.strip())
    if not vals:
        raise argparse.ArgumentTypeError("expected comma-separated integers")
    return vals


def _parse_float_tuple(spec: str) -> tuple[float, ...]:
    vals = tuple(float(x.strip()) for x in spec.split(",") if x.strip())
    if not vals:
        raise argparse.ArgumentTypeError("expected comma-separated floats")
    return vals


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    p_run = sub.add_parser("run", help="Run the 2x2x2 clock-modes factorial probe")
    p_run.add_argument("out_dir", type=Path, help="Output directory for run artifacts")
    p_run.add_argument("--smoke", action="store_true", help="Fast local smoke run on any host")
    p_run.add_argument(
        "--allow-any-host", action="store_true", help="Bypass i9-10900K host & clean-tree checks"
    )
    p_run.add_argument("--corpus", type=Path, default=None, help="Override input .bz2 corpus")
    p_run.add_argument("--ab-bin", type=Path, default=None, help="Reuse prebuilt ab binary")
    p_run.add_argument("--rounds", type=int, default=None, help="Rounds (default: 12; smoke: 2)")
    p_run.add_argument(
        "--threads", type=_parse_int_tuple, default=THREADS, help="Thread counts (default: 1,6)"
    )
    p_run.add_argument(
        "--settles",
        type=_parse_float_tuple,
        default=SETTLES,
        help="Settle gaps in seconds (default: 0.0,0.2)",
    )
    p_run.add_argument(
        "--prespin-ms",
        type=_parse_int_tuple,
        default=DEFAULT_PRESPIN_MS,
        help="Comma-separated prespin durations in ms (default: 0; e.g. 0,100)",
    )

    p_score = sub.add_parser("score", help="Re-score a clock_modes runs.jsonl file")
    p_score.add_argument("runs_jsonl", type=Path, help="Path to runs.jsonl")
    p_score.add_argument("--out", type=Path, default=None, help="Optional output directory")
    p_score.add_argument(
        "--rounds", type=int, default=None, help="Expected rounds (auto if omitted)"
    )
    p_score.add_argument("--json", action="store_true", help="Print JSON summary")

    p_an = sub.add_parser("analyze", help="Mode-robust analysis of any ab runs.jsonl")
    p_an.add_argument("runs_jsonl", type=Path, help="Path to ab runs.jsonl")
    p_an.add_argument("--ref", default=None, help="Reference arm name (default: mibz2)")
    p_an.add_argument("--arm", default=None, help="Single target arm to compare against --ref")
    p_an.add_argument("--threads", type=_parse_int_tuple, default=None, help="Filter thread counts")
    p_an.add_argument("--metric", default="wall", choices=("wall", "cpu"), help="Metric to compare")
    p_an.add_argument("--json", action="store_true", help="Print JSON output")

    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.command == "run":
        try:
            summary = run_probe(
                args.out_dir,
                smoke=args.smoke,
                allow_any_host=args.allow_any_host,
                corpus_path=args.corpus,
                ab_bin_path=args.ab_bin,
                rounds=args.rounds,
                threads=args.threads,
                settles=args.settles,
                prespin_ms_values=args.prespin_ms,
            )
        except InvalidRun as err:
            args.out_dir.mkdir(parents=True, exist_ok=True)
            (args.out_dir / "outcome.txt").write_text(f"VOID: {err}\n", encoding="utf-8")
            print(f"VOID: {err}", file=sys.stderr)
            return 1
        print(format_summary(summary), end="")
        return 0

    if args.command == "score":
        rows = [
            json.loads(ln)
            for ln in args.runs_jsonl.read_text(encoding="utf-8").splitlines()
            if ln.strip()
        ]
        inferred_rounds = (
            args.rounds if args.rounds is not None else (max(r["round"] for r in rows) + 1)
        )
        inferred_threads = tuple(sorted({int(r["threads"]) for r in rows}))
        inferred_pinnings = tuple(sorted({str(r["pinning"]) for r in rows}))
        inferred_settles = tuple(sorted({float(r["settle_s"]) for r in rows}))
        inferred_prespins = tuple(sorted({int(r.get("prespin_ms", 0)) for r in rows}))
        summary = score(
            rows,
            rounds=inferred_rounds,
            threads=inferred_threads,
            pinnings=inferred_pinnings,
            settles=inferred_settles,
            prespin_ms_values=inferred_prespins,
        )
        if args.out is not None:
            args.out.mkdir(parents=True, exist_ok=True)
            save_json(args.out / "summary.json", summary)
            (args.out / "summary.txt").write_text(format_summary(summary), encoding="utf-8")
            (args.out / "outcome.txt").write_text(
                f"{summary['verdict']}: {summary['reason']}\n", encoding="utf-8"
            )
        if args.json:
            print(json.dumps(summary, indent=2))
        else:
            print(format_summary(summary), end="")
        return 1 if summary["verdict"] == "VOID" else 0

    if args.command == "analyze":
        rows = [
            json.loads(ln)
            for ln in args.runs_jsonl.read_text(encoding="utf-8").splitlines()
            if ln.strip()
        ]
        res = analyze_ab_runs(
            rows,
            ref_arm=args.ref,
            target_arm=args.arm,
            threads_filter=set(args.threads) if args.threads else None,
            metric=args.metric,
        )
        if args.json:
            print(json.dumps(res, indent=2))
        else:
            print(format_analysis(res), end="")
        return 0

    return 2


if __name__ == "__main__":
    raise SystemExit(main())
