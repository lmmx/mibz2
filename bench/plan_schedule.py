#!/usr/bin/env python3
"""Shallow-safe C6 physical i9-10900K verification runner for:

1. `plan-schedule` (`plan-schedule-and-lazy-threshold`):
   Paired base vs head verification across pinned `wikidata x4` (4..10T),
   unpinned `wikidata x4` (4, 8, 10, 12, 16, 20T), and pinned off-Wikidata
   guards (`text`, `records`, `random` at 4, 6, 8T).
2. `trim-kstar` (`trim-at-measured-kstar`):
   Paired 8 vs 7 workers at pinned 10T across `wikidata x4` + guards
   (`text`, `records`, `random`) with 12 order-balanced position-stratified rounds.

Subcommands:
    python3 bench/plan_schedule.py run OUT_DIR --mode {plan-schedule,trim-kstar} [...]
    python3 bench/plan_schedule.py score RUNS_JSONL --mode {plan-schedule,trim-kstar} [...]
"""

from __future__ import annotations

import argparse
import bz2
from collections import defaultdict
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

from mtf_fold import t_crit_95, topology  # noqa: E402
import wikidata  # noqa: E402

ROOT = BENCH_DIR.parent
REGISTRATION = ROOT / "docs/journal/2026-11-27-plan-schedule-i9/registration.md"

WIKIDATA_X4_SHA256 = "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"

MODES = ("plan-schedule", "trim-kstar")
ARMS = ("base", "head")

PLAN_SCHEDULE_WIKIDATA_PINNED = (4, 5, 6, 7, 8, 9, 10)
PLAN_SCHEDULE_WIKIDATA_UNPINNED = (4, 8, 10, 12, 16, 20)
GUARD_CORPORA = ("text", "records", "random")
PLAN_SCHEDULE_GUARD_COUNTS = (4, 6, 8)
TRIM_KSTAR_CORPORA = ("wikidata", *GUARD_CORPORA)
TRIM_KSTAR_THREADS = 10

DEFAULT_ROUNDS = 12
SMOKE_ROUNDS = 2
DEFAULT_CORPUS_SCALE = 8
SMOKE_CORPUS_SCALE = 1
SEED = 1210
PRESPIN_MS = 100
SETTLE_S = 2.0
MAX_BUSY = 1.5
RUN_TIMEOUT_S = 180

# Frozen stop-test thresholds for `plan-schedule-and-lazy-threshold`.
PS_5T_WALL_MAX = 0.85
PS_5T_RSS_MAX = 0.55
PS_7T_WALL_MAX = 0.75
PS_7T_RSS_MAX = 0.45
PS_9T_WALL_MAX = 1.02
PS_9T_RSS_MAX = 0.92
PS_NEUTRAL_WALL_MAX = 1.015
PS_NEUTRAL_RSS_MAX = 1.02
PS_GUARD_DISJOINT_WALL_MAX = 1.03
PS_GUARD_RSS_MAX = 1.05

# Frozen stop-test thresholds for `trim-at-measured-kstar`.
TK_WIKIDATA_CI_HI_MAX = 0.97
TK_GUARD_WALL_MAX = 1.01
TK_RSS_MAX = 1.05


class InvalidRun(ValueError):
    """A registered prerequisite or validity invariant failed; the run is VOID."""


def output(*cmd: str, cwd: Path = ROOT, env: dict[str, str] | None = None) -> str:
    return subprocess.check_output(list(cmd), cwd=cwd, text=True, env=env).strip()


def save_json(path: Path, obj: Any) -> None:
    path.write_text(json.dumps(obj, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def sha256_file(path: Path | str) -> str:
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def reference_bz2(path: Path | str) -> dict[str, Any]:
    """Independently decompress a .bz2 file via Python's libbzip2 binding."""
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
            raise InvalidRun(f"{var} must be unset for baseline x86-64 codegen")


def check_cpu_quota_unlimited(env_facts: dict[str, Any]) -> None:
    """Require unlimited cgroup CPU quota (supports both `ab env` and `ab env --facts`)."""
    quota = env_facts.get("cpu_quota", "absent")
    if quota is None:
        return
    if isinstance(quota, dict) and quota.get("state") == "unlimited" and quota.get("cpus") is None:
        return
    raise InvalidRun(f"workstation CPU quota must be unlimited (got {quota!r})")


def check_machine(out: Path, *, require_i9: bool = True) -> list[list[int]]:
    """Inspect CPU topology via lscpu; enforce i9-10900K invariants when `require_i9`."""
    description = output("lscpu")
    grid_text = output("lscpu", "-e")
    (out / "lscpu.txt").write_text(description + "\n", encoding="utf-8")
    grid_path = out / "lscpu-e.txt"
    grid_path.write_text(grid_text + "\n", encoding="utf-8")
    cores = topology(str(grid_path))
    if require_i9:
        if "i9-10900K" not in description:
            raise InvalidRun("registered machine is the i9-10900K")
        cpus = {cpu for core in cores for cpu in core}
        if len(cores) != 10 or any(len(core) != 2 for core in cores) or len(cpus) != 20:
            raise InvalidRun("expected ten physical cores with both hardware threads")
        if cpus != os.sched_getaffinity(0):
            raise InvalidRun("all 20 workstation CPUs must be available")
    return cores


def assert_distinct_binaries(base_bin: Path | str, head_bin: Path | str) -> tuple[str, str]:
    """Fail closed (VOID) if base and head binaries have identical SHA-256."""
    base_sha = sha256_file(base_bin)
    head_sha = sha256_file(head_bin)
    if base_sha == head_sha:
        raise InvalidRun(
            f"base and head binaries are identical (sha256={base_sha}); "
            "refusing to compare a binary against itself"
        )
    return base_sha, head_sha


def checkout_ref_shallow(recipe_repo: Path, dest: Path, ref_spec: str) -> str:
    """Shallow-safely (`--depth=1`) materialize `ref_spec` into `dest` and return its commit SHA.

    Works on shallow `--depth=1` checkouts where `ref_spec` is not in local history.
    """
    recipe_repo = Path(recipe_repo)
    dest = Path(dest)
    dest.mkdir(parents=True, exist_ok=True)
    subprocess.run(["git", "init", "-q"], cwd=dest, check=True)

    remote_url = None
    try:
        remote_url = output("git", "remote", "get-url", "origin", cwd=recipe_repo)
    except subprocess.CalledProcessError:
        remote_url = None

    # Prefer local repo fetch if ref exists locally (fast, offline-safe, works in unit tests),
    # otherwise fetch `--depth=1 --no-tags` from `origin` (stripping a local `origin/` prefix).
    fetched = False
    candidates: list[tuple[str, str]] = [(str(recipe_repo), ref_spec)]
    if remote_url:
        candidates.append((remote_url, ref_spec.removeprefix("origin/")))
    for source, fetch_ref in candidates:
        proc = subprocess.run(
            ["git", "fetch", "-q", "--depth=1", "--no-tags", source, fetch_ref],
            cwd=dest,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        if proc.returncode == 0:
            fetched = True
            break
    if not fetched:
        raise InvalidRun(f"failed shallow --depth=1 fetch of ref {ref_spec!r}")

    sha = output("git", "rev-parse", "FETCH_HEAD^{commit}", cwd=dest)
    subprocess.run(["git", "checkout", "-q", "--detach", sha], cwd=dest, check=True)
    return sha


def build_arm_binaries(
    src_dir: Path,
    target_dir: Path,
    bin_dir: Path,
    arm: str,
    *,
    include_windows: bool = False,
    env: dict[str, str] | None = None,
) -> dict[str, Path]:
    """Build `ab` (and optionally `windows`) in a strictly isolated `CARGO_TARGET_DIR`."""
    build_env = dict(os.environ if env is None else env)
    check_baseline_env(build_env)
    target_dir = Path(target_dir).resolve()
    bin_dir = Path(bin_dir).resolve()
    bin_dir.mkdir(parents=True, exist_ok=True)
    build_env["CARGO_TARGET_DIR"] = str(target_dir)

    names = ["ab", "windows"] if include_windows else ["ab"]
    cmd = ["cargo", "build", "--release", "--locked"]
    for name in names:
        cmd.extend(["--example", name])
    subprocess.run(cmd, cwd=src_dir, env=build_env, check=True)

    bins: dict[str, Path] = {}
    for name in names:
        src = target_dir / "release/examples" / name
        dst = bin_dir / f"{arm}-{name}"
        shutil.copy2(src, dst)
        bins[name] = dst
    return bins


def expected_cells(mode: str, *, smoke: bool = False) -> list[tuple[str, str, int]]:
    """Return the exact ordered list of `(corpus, pinning, threads)` cells for `mode`."""
    if mode == "plan-schedule":
        if smoke:
            return [
                ("wikidata", "pinned", 4),
                ("wikidata", "pinned", 5),
                ("wikidata", "pinned", 6),
                ("wikidata", "pinned", 7),
                ("wikidata", "unpinned", 4),
                ("text", "pinned", 4),
            ]
        cells: list[tuple[str, str, int]] = []
        for t in PLAN_SCHEDULE_WIKIDATA_PINNED:
            cells.append(("wikidata", "pinned", t))
        for t in PLAN_SCHEDULE_WIKIDATA_UNPINNED:
            cells.append(("wikidata", "unpinned", t))
        for corpus in GUARD_CORPORA:
            for t in PLAN_SCHEDULE_GUARD_COUNTS:
                cells.append((corpus, "pinned", t))
        return cells
    if mode == "trim-kstar":
        if smoke:
            return [
                ("wikidata", "pinned", 4),
                ("text", "pinned", 4),
            ]
        return [(corpus, "pinned", TRIM_KSTAR_THREADS) for corpus in TRIM_KSTAR_CORPORA]
    raise InvalidRun(f"unknown mode {mode!r}")


def build_schedule(
    cells: list[tuple[str, str, int]],
    *,
    rounds: int = DEFAULT_ROUNDS,
    seed: int = SEED,
) -> list[dict[str, Any]]:
    """Build deterministic order-balanced alternating schedule.

    Over every even number of rounds, each arm runs in `pair_pos = 0` and `pair_pos = 1`
    an equal number of times for every `(corpus, pinning, threads)` cell.
    """
    if rounds < 1:
        raise InvalidRun(f"rounds must be >= 1, got {rounds}")
    schedule: list[dict[str, Any]] = []
    pair_id = 0
    for r in range(rounds):
        rng = random.Random(seed + r * 9973)
        order = list(cells)
        rng.shuffle(order)
        for order_in_round, (corpus, pinning, threads) in enumerate(order):
            arm_order = ("base", "head") if (r % 2 == 0) else ("head", "base")
            for pair_pos, arm in enumerate(arm_order):
                schedule.append(
                    {
                        "round": r,
                        "pair_id": pair_id,
                        "order_in_round": order_in_round,
                        "corpus": corpus,
                        "pinning": pinning,
                        "threads": threads,
                        "arm": arm,
                        "pair_pos": pair_pos,
                    }
                )
            pair_id += 1
    return schedule


def materialize_wikidata(
    tmp: Path,
    checkout: Path,
    windows_bin: Path,
    *,
    x4: bool = True,
) -> dict[str, Any]:
    """Shallow-safely materialize `wikidata` sample (and x4 multistream when `x4=True`)."""
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

    if not x4:
        return {
            "path": sample,
            "compressed_sha256": base_hash,
            "reps": 1,
        }

    wall_path = tmp / "wikidata-x4.bz2"
    with wall_path.open("wb") as dest:
        for _ in range(4):
            with sample.open("rb") as src:
                shutil.copyfileobj(src, dest)
    wall_hash = sha256_file(wall_path)
    if wall_hash != WIKIDATA_X4_SHA256:
        raise InvalidRun(f"wikidata x4 SHA-256 mismatch ({wall_hash})")
    return {
        "path": wall_path,
        "compressed_sha256": wall_hash,
        "reps": 4,
    }


def materialize_inputs(
    out: Path,
    tmp: Path,
    checkout: Path,
    base_ab: Path,
    base_windows: Path,
    corpora_needed: set[str],
    *,
    corpus_scale: int = DEFAULT_CORPUS_SCALE,
    wikidata_x4: bool = True,
) -> tuple[dict[str, Path], dict[str, dict[str, Any]]]:
    """Materialize required corpora and compute independent libbzip2 SHA-256 references."""
    files: dict[str, Path] = {}
    if "wikidata" in corpora_needed:
        wk = materialize_wikidata(tmp, checkout, base_windows, x4=wikidata_x4)
        files["wikidata"] = wk["path"]

    gen_needed = sorted(c for c in corpora_needed if c in GUARD_CORPORA)
    if gen_needed:
        gen_dir = tmp / "gen"
        subprocess.run(
            [str(base_ab), "gen", str(gen_dir), str(corpus_scale)],
            check=True,
        )
        for name in gen_needed:
            p = gen_dir / f"{name}.bz2"
            if not p.exists():
                raise InvalidRun(f"ab gen did not produce expected corpus {p}")
            files[name] = p

    refs: dict[str, dict[str, Any]] = {}
    inputs_meta: dict[str, Any] = {}
    for name, path in sorted(files.items()):
        ref = reference_bz2(path)
        refs[name] = ref
        inputs_meta[name] = {
            "path": str(path),
            "compressed_bytes": path.stat().st_size,
            "compressed_sha256": sha256_file(path),
            "out": ref["out"],
            "sha256": ref["sha256"],
        }
    save_json(out / "inputs.json", inputs_meta)
    return files, refs


def run_ab_child(
    ab_bin: Path,
    arm: str,
    threads: int,
    corpus_path: Path,
    *,
    sha256: bool = False,
) -> dict[str, Any]:
    env = dict(os.environ)
    if sha256:
        env["AB_SHA256"] = "1"
    else:
        env.pop("AB_SHA256", None)
    proc = subprocess.run(
        [str(ab_bin), "child", arm, str(threads), str(corpus_path)],
        env=env,
        text=True,
        capture_output=True,
        timeout=RUN_TIMEOUT_S,
        check=False,
    )
    if proc.returncode != 0:
        raise InvalidRun(
            f"ab child {arm} t={threads} failed (rc={proc.returncode}): {proc.stderr.strip()}"
        )
    lines = [ln for ln in proc.stdout.splitlines() if ln.strip()]
    if len(lines) != 1:
        raise InvalidRun(f"expected 1 JSON line from ab child, got {len(lines)}")
    return json.loads(lines[0])


def run_parity(
    out: Path,
    base_ab: Path,
    head_ab: Path,
    files: dict[str, Path],
    refs: dict[str, dict[str, Any]],
    cells: list[tuple[str, str, int]],
) -> dict[str, dict[str, Any]]:
    """Verify untimed `AB_SHA256=1` parity for both arms against libbzip2 across all corpora."""
    # Check each unique (corpus, threads) at least once on both arms; record ab hash per corpus.
    pairs_to_check = sorted({(c, t) for c, _, t in cells})
    corpus_ab_hash: dict[str, str] = {}
    with (out / "parity.jsonl").open("x", buffering=1, encoding="utf-8") as log:
        for corpus, threads in pairs_to_check:
            ref = refs[corpus]
            for arm_name, exe in (("base", base_ab), ("head", head_ab)):
                row = run_ab_child(exe, "mibz2", threads, files[corpus], sha256=True)
                if row.get("err") not in (None, "") or row.get("termination") not in (
                    None,
                    "success",
                ):
                    raise InvalidRun(f"parity error on {arm_name}/{corpus}/t={threads}: {row}")
                if row.get("out") != ref["out"]:
                    raise InvalidRun(
                        f"parity byte count mismatch on {arm_name}/{corpus}/t={threads}: "
                        f"{row.get('out')} != {ref['out']}"
                    )
                if row.get("sha256") != ref["sha256"]:
                    raise InvalidRun(
                        f"parity SHA-256 mismatch on {arm_name}/{corpus}/t={threads}: "
                        f"{row.get('sha256')} != {ref['sha256']}"
                    )
                h = row.get("hash")
                if not h:
                    raise InvalidRun(f"missing ab hash on {arm_name}/{corpus}/t={threads}")
                prev = corpus_ab_hash.get(corpus)
                if prev is not None and prev != h:
                    raise InvalidRun(f"ab hash mismatch across arms on {corpus}: {h} != {prev}")
                corpus_ab_hash[corpus] = h
                log.write(
                    json.dumps(
                        {
                            "corpus": corpus,
                            "threads": threads,
                            "arm": arm_name,
                            "out": row["out"],
                            "hash": h,
                            "sha256": row["sha256"],
                        },
                        allow_nan=False,
                    )
                    + "\n"
                )
    return {
        c: {"out": refs[c]["out"], "sha256": refs[c]["sha256"], "ab_hash": corpus_ab_hash[c]}
        for c in files
    }


def extract_machine_telemetry(row: dict[str, Any]) -> dict[str, Any]:
    """Normalize hardware counter fields from `ab --machine-state` / `AB_MACHINE_STATE=1`."""
    machine = row.get("machine") if isinstance(row.get("machine"), dict) else {}
    counters = machine.get("counters") if isinstance(machine.get("counters"), dict) else {}

    def _pos(val: Any) -> float | None:
        if isinstance(val, (int, float)) and math.isfinite(val) and val > 0:
            return float(val)
        return None

    cycles = _pos(row.get("cycles")) or _pos(counters.get("cycles"))
    instructions = _pos(row.get("instructions")) or _pos(counters.get("instructions"))
    ref_cycles = _pos(row.get("ref_cycles")) or _pos(counters.get("ref_cycles"))
    cpr = _pos(row.get("cycles_per_ref")) or _pos(counters.get("cycles_per_ref"))
    if cpr is None and cycles is not None and ref_cycles is not None:
        cpr = round(cycles / ref_cycles, 4)

    runq = row.get("runq_s")
    runq_val = (
        float(runq)
        if isinstance(runq, (int, float)) and math.isfinite(runq) and runq >= 0
        else None
    )
    return {
        "cycles": cycles,
        "instructions": instructions,
        "ref_cycles": ref_cycles,
        "cycles_per_ref": cpr,
        "runq_s": runq_val,
    }


def validate_timed_row(row: dict[str, Any], expected: dict[str, Any] | None = None) -> None:
    """Validate one timed row against frozen validity criteria."""
    if row.get("arm") not in ARMS:
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
    out_bytes = row.get("out")
    if not isinstance(out_bytes, int) or out_bytes <= 0:
        raise InvalidRun(f"invalid out bytes: {out_bytes!r}")
    if expected is not None:
        if out_bytes != expected["out"]:
            raise InvalidRun(f"output bytes mismatch: {out_bytes} != {expected['out']}")
        if expected.get("ab_hash") and row.get("hash") != expected["ab_hash"]:
            raise InvalidRun(f"output hash mismatch: {row.get('hash')} != {expected['ab_hash']}")


def run_paired_sweep(
    out: Path,
    tmp: Path,
    base_ab: Path,
    head_ab: Path,
    files: dict[str, Path],
    expected_refs: dict[str, dict[str, Any]],
    schedule: list[dict[str, Any]],
    *,
    prespin_ms: int = PRESPIN_MS,
    settle_s: float = SETTLE_S,
    max_busy: float = MAX_BUSY,
    smoke: bool = False,
) -> list[dict[str, Any]]:
    """Execute the order-balanced alternating schedule via `ab run --machine-state`.

    Under `--smoke`, invokes `ab child` directly with `AB_MACHINE_STATE=1` so smoke runs
    on shared multi-agent VMs do not block on `ab run`'s `one > 0.5` idle-machine wait.
    """
    rows: list[dict[str, Any]] = []
    cell_file = tmp / "cell.jsonl"
    env = dict(os.environ)
    env.pop("AB_SHA256", None)
    env["AB_MACHINE_STATE"] = "1"

    with (out / "runs.jsonl").open("x", buffering=1, encoding="utf-8") as log:
        for step in schedule:
            corpus = step["corpus"]
            pinning = step["pinning"]
            threads = step["threads"]
            arm = step["arm"]

            if smoke:
                exe = head_ab if arm == "head" else base_ab
                proc = subprocess.run(
                    [str(exe), "child", "mibz2", str(threads), str(files[corpus])],
                    env=env,
                    text=True,
                    capture_output=True,
                    timeout=RUN_TIMEOUT_S,
                    check=False,
                )
                if proc.returncode != 0:
                    raise InvalidRun(
                        f"ab child failed ({corpus} {pinning} t={threads} {arm} r={step['round']}, "
                        f"rc={proc.returncode}): {proc.stderr.strip()}"
                    )
                lines = [ln for ln in proc.stdout.splitlines() if ln.strip()]
                if len(lines) != 1:
                    raise InvalidRun(f"expected 1 JSON line from ab child, got {len(lines)}")
                sample = json.loads(lines[0])
            else:
                if cell_file.exists():
                    cell_file.unlink()
                arm_arg = "mibz2" if arm == "head" else f"mibz2@{base_ab}"
                cmd = [
                    str(head_ab),
                    "run",
                    str(cell_file),
                    "--arms",
                    arm_arg,
                    "--threads",
                    str(threads),
                    "--rounds",
                    "1",
                    "--prespin-ms",
                    str(prespin_ms),
                    "--settle",
                    str(settle_s),
                    "--max-busy",
                    str(max_busy),
                    "--machine-state",
                ]
                if pinning == "pinned":
                    cmd.append("--spread")
                cmd.append(str(files[corpus]))

                proc = subprocess.run(
                    cmd,
                    env=env,
                    text=True,
                    capture_output=True,
                    timeout=RUN_TIMEOUT_S,
                    check=False,
                )
                if proc.returncode != 0:
                    raise InvalidRun(
                        f"ab run failed ({corpus} {pinning} t={threads} {arm} r={step['round']}, "
                        f"rc={proc.returncode}): {proc.stderr.strip()}"
                    )
                lines = [
                    ln for ln in cell_file.read_text(encoding="utf-8").splitlines() if ln.strip()
                ]
                if len(lines) != 1:
                    raise InvalidRun(f"expected 1 JSON line from ab run, got {len(lines)}")
                sample = json.loads(lines[0])

            telem = extract_machine_telemetry(sample)
            row = {
                **sample,
                **telem,
                "corpus": corpus,
                "pinning": pinning,
                "threads": threads,
                "arm": arm,
                "round": step["round"],
                "pair_id": step["pair_id"],
                "order_in_round": step["order_in_round"],
                "pair_pos": step["pair_pos"],
            }
            validate_timed_row(row, expected_refs[corpus])
            log.write(json.dumps(row, allow_nan=False) + "\n")
            rows.append(row)
    return rows


# ---------------------------------------------------------------------------
# Statistical estimators: pooled paired, position-stratified, within-mode
# ---------------------------------------------------------------------------


def _paired_log_ratio_ci(logs: list[float]) -> dict[str, Any]:
    n = len(logs)
    if n == 0:
        return {"n": 0, "ratio": None, "ci_lo": None, "ci_hi": None}
    mean = sum(logs) / n
    ratio = math.exp(mean)
    if n == 1:
        return {"n": 1, "ratio": round(ratio, 6), "ci_lo": None, "ci_hi": None}
    var = sum((x - mean) ** 2 for x in logs) / (n - 1)
    se = math.sqrt(var / n)
    half = t_crit_95(n - 1) * se
    return {
        "n": n,
        "ratio": round(ratio, 6),
        "ci_lo": round(math.exp(mean - half), 6),
        "ci_hi": round(math.exp(mean + half), 6),
    }


def _unpaired_log_ratio_ci(head_vals: list[float], base_vals: list[float]) -> dict[str, Any]:
    na, nb = len(head_vals), len(base_vals)
    if na == 0 or nb == 0:
        return {"n_head": na, "n_base": nb, "ratio": None, "ci_lo": None, "ci_hi": None}
    la = [math.log(v) for v in head_vals]
    lb = [math.log(v) for v in base_vals]
    ma = sum(la) / na
    mb = sum(lb) / nb
    diff = ma - mb
    ratio = math.exp(diff)
    if na < 2 or nb < 2:
        return {
            "n_head": na,
            "n_base": nb,
            "ratio": round(ratio, 6),
            "ci_lo": None,
            "ci_hi": None,
        }
    va = sum((x - ma) ** 2 for x in la) / (na - 1)
    vb = sum((x - mb) ** 2 for x in lb) / (nb - 1)
    se2 = va / na + vb / nb
    if se2 <= 0.0:
        r = round(ratio, 6)
        return {"n_head": na, "n_base": nb, "ratio": r, "ci_lo": r, "ci_hi": r}
    df_num = se2 * se2
    df_den = ((va / na) ** 2) / (na - 1) + ((vb / nb) ** 2) / (nb - 1)
    df = max(1, round(df_num / df_den)) if df_den > 0 else min(na, nb) - 1
    half = t_crit_95(df) * math.sqrt(se2)
    return {
        "n_head": na,
        "n_base": nb,
        "ratio": round(ratio, 6),
        "ci_lo": round(math.exp(diff - half), 6),
        "ci_hi": round(math.exp(diff + half), 6),
    }


def classify_clock_modes(
    rows: list[dict[str, Any]], threads: int
) -> tuple[dict[int, str], dict[str, Any]]:
    """Classify rows in one cell into 'fast' vs 'slow' clock modes.

    Prefers hardware `cycles_per_ref` (from `AB_MACHINE_STATE=1`); falls back to wall-gap split
    when PMU counters are unavailable.
    """
    has_cpr = all(
        isinstance(r.get("cycles_per_ref"), (int, float))
        and math.isfinite(float(r["cycles_per_ref"]))
        and float(r["cycles_per_ref"]) > 0.0
        for r in rows
    )
    if has_cpr:
        cprs = [float(r["cycles_per_ref"]) for r in rows]
        ordered = sorted(cprs, reverse=True)
        anchor = ordered[max(0, len(ordered) // 20)]
        drop_tol = 0.04 if threads <= 1 else 0.15
        labels = {
            idx: ("fast" if float(r["cycles_per_ref"]) >= anchor * (1.0 - drop_tol) else "slow")
            for idx, r in enumerate(rows)
        }
        return labels, {
            "basis": "cycles_per_ref",
            "fast_cpr_anchor": round(anchor, 4),
            "drop_tol": drop_tol,
        }

    walls = [float(r["wall"]) for r in rows]
    ordered_w = sorted(walls)
    fast_anchor = statistics.median(ordered_w[: max(1, (len(ordered_w) + 1) // 2)])
    thr = 1.04 if threads <= 1 else 1.30
    split = fast_anchor * thr
    max_gap = 1.0
    gap_split = split
    for i in range(len(ordered_w) - 1):
        ratio = ordered_w[i + 1] / ordered_w[i]
        if ratio > max_gap:
            max_gap = ratio
            gap_split = math.sqrt(ordered_w[i] * ordered_w[i + 1])
    if all(w < split for w in walls) and max_gap >= 1.08:
        split = gap_split
    labels = {idx: ("fast" if float(r["wall"]) < split else "slow") for idx, r in enumerate(rows)}
    return labels, {
        "basis": "wall_fallback",
        "fast_wall_anchor_s": round(fast_anchor, 6),
        "split_wall_s": round(split, 6),
    }


def analyze_cell(cell_rows: list[dict[str, Any]], threads: int) -> dict[str, Any]:
    """Compute pooled paired, position-stratified, within-mode, RSS, CPU, and telemetry stats."""
    base_rows = [r for r in cell_rows if r["arm"] == "base"]
    head_rows = [r for r in cell_rows if r["arm"] == "head"]
    base_by_round = {int(r["round"]): r for r in base_rows}
    head_by_round = {int(r["round"]): r for r in head_rows}
    common_rounds = sorted(set(base_by_round) & set(head_by_round))

    wall_logs = [
        math.log(float(head_by_round[rd]["wall"]) / float(base_by_round[rd]["wall"]))
        for rd in common_rounds
    ]
    pooled = _paired_log_ratio_ci(wall_logs)

    base_walls = [float(r["wall"]) for r in base_rows]
    head_walls = [float(r["wall"]) for r in head_rows]
    base_med_wall = statistics.median(base_walls)
    head_med_wall = statistics.median(head_walls)

    # Disjoint regression / improvement indicators.
    range_disjoint_slower = min(head_walls) > max(base_walls)
    ci_disjoint_slower = pooled["ci_lo"] is not None and pooled["ci_lo"] > 1.0
    disjoint_slower = range_disjoint_slower or ci_disjoint_slower

    range_disjoint_faster = max(head_walls) < min(base_walls)
    ci_disjoint_faster = pooled["ci_hi"] is not None and pooled["ci_hi"] < 1.0
    disjoint_faster = range_disjoint_faster or ci_disjoint_faster

    # Position-stratified estimator (pair_pos 0 vs 1).
    positions = sorted({int(r.get("pair_pos", 0)) for r in cell_rows})
    by_pos: dict[str, Any] = {}
    strat_terms: list[tuple[float, float]] = []
    for p in positions:
        h_p = [float(r["wall"]) for r in head_rows if int(r.get("pair_pos", 0)) == p]
        b_p = [float(r["wall"]) for r in base_rows if int(r.get("pair_pos", 0)) == p]
        est = _unpaired_log_ratio_ci(h_p, b_p)
        by_pos[str(p)] = est
        if est["ratio"] is not None:
            w = float(min(len(h_p), len(b_p)))
            strat_terms.append((math.log(est["ratio"]), w))
    pos_combined = (
        round(math.exp(sum(v * w for v, w in strat_terms) / sum(w for _, w in strat_terms)), 6)
        if strat_terms
        else None
    )

    # Within-mode (`cycles_per_ref` preferred) estimator.
    mode_labels, mode_meta = classify_clock_modes(cell_rows, threads)
    head_by_mode: dict[str, list[float]] = defaultdict(list)
    base_by_mode: dict[str, list[float]] = defaultdict(list)
    head_round_mode: dict[int, str] = {}
    base_round_mode: dict[int, str] = {}
    for idx, r in enumerate(cell_rows):
        m = mode_labels[idx]
        if r["arm"] == "head":
            head_by_mode[m].append(float(r["wall"]))
            head_round_mode[int(r["round"])] = m
        else:
            base_by_mode[m].append(float(r["wall"]))
            base_round_mode[int(r["round"])] = m

    within_mode_by_name: dict[str, Any] = {}
    for m in ("fast", "slow"):
        if head_by_mode.get(m) or base_by_mode.get(m):
            within_mode_by_name[m] = _unpaired_log_ratio_ci(
                head_by_mode.get(m, []), base_by_mode.get(m, [])
            )
    same_mode_logs = [
        math.log(float(head_by_round[rd]["wall"]) / float(base_by_round[rd]["wall"]))
        for rd in common_rounds
        if head_round_mode.get(rd) == base_round_mode.get(rd)
    ]
    same_mode_paired = _paired_log_ratio_ci(same_mode_logs)

    # cycles_per_ref ratio (clock telemetry).
    base_cprs = [
        float(r["cycles_per_ref"])
        for r in base_rows
        if isinstance(r.get("cycles_per_ref"), (int, float)) and r["cycles_per_ref"] > 0
    ]
    head_cprs = [
        float(r["cycles_per_ref"])
        for r in head_rows
        if isinstance(r.get("cycles_per_ref"), (int, float)) and r["cycles_per_ref"] > 0
    ]
    base_med_cpr = round(statistics.median(base_cprs), 4) if base_cprs else None
    head_med_cpr = round(statistics.median(head_cprs), 4) if head_cprs else None
    cpr_ratio = (
        round(head_med_cpr / base_med_cpr, 4)
        if base_med_cpr and head_med_cpr and base_med_cpr > 0
        else None
    )

    # RSS and CPU summaries.
    base_rss = statistics.median(float(r["rss_kb"]) for r in base_rows)
    head_rss = statistics.median(float(r["rss_kb"]) for r in head_rows)
    rss_ratio = head_rss / base_rss

    base_cpus = [
        float(r["cpu"])
        for r in base_rows
        if isinstance(r.get("cpu"), (int, float)) and r["cpu"] > 0
    ]
    head_cpus = [
        float(r["cpu"])
        for r in head_rows
        if isinstance(r.get("cpu"), (int, float)) and r["cpu"] > 0
    ]
    cpu_summary = None
    if base_cpus and head_cpus:
        bm_cpu = statistics.median(base_cpus)
        hm_cpu = statistics.median(head_cpus)
        cpu_summary = {
            "base_median_s": round(bm_cpu, 6),
            "head_median_s": round(hm_cpu, 6),
            "ratio": round(hm_cpu / bm_cpu, 6),
        }

    base_runqs = [
        float(r["runq_s"])
        for r in base_rows
        if isinstance(r.get("runq_s"), (int, float)) and r["runq_s"] >= 0
    ]
    head_runqs = [
        float(r["runq_s"])
        for r in head_rows
        if isinstance(r.get("runq_s"), (int, float)) and r["runq_s"] >= 0
    ]

    return {
        "base_median_wall_s": round(base_med_wall, 6),
        "head_median_wall_s": round(head_med_wall, 6),
        "pooled": pooled,
        "disjoint_slower": disjoint_slower,
        "disjoint_faster": disjoint_faster,
        "position_stratified": {
            "combined_ratio": pos_combined,
            "by_pos": by_pos,
        },
        "within_mode": {
            **mode_meta,
            "base_mode_counts": {k: len(v) for k, v in sorted(base_by_mode.items())},
            "head_mode_counts": {k: len(v) for k, v in sorted(head_by_mode.items())},
            "by_mode": within_mode_by_name,
            "same_mode_paired": same_mode_paired,
            "base_median_cycles_per_ref": base_med_cpr,
            "head_median_cycles_per_ref": head_med_cpr,
            "cycles_per_ref_ratio": cpr_ratio,
        },
        "rss": {
            "base_median_kb": round(base_rss, 1),
            "head_median_kb": round(head_rss, 1),
            "ratio": round(rss_ratio, 6),
        },
        "cpu": cpu_summary,
        "runq": {
            "base_median_s": round(statistics.median(base_runqs), 6) if base_runqs else None,
            "head_median_s": round(statistics.median(head_runqs), 6) if head_runqs else None,
        },
    }


def _cell_key(corpus: str, pinning: str, threads: int) -> str:
    return f"{corpus}:{pinning}:t{threads}"


def score(
    rows: list[dict[str, Any]],
    *,
    mode: str,
    rounds: int = DEFAULT_ROUNDS,
    cells: list[tuple[str, str, int]] | None = None,
    expected_refs: dict[str, dict[str, Any]] | None = None,
    host_ok: bool = True,
    distinct_binaries: bool = True,
    smoke: bool = False,
) -> dict[str, Any]:
    """Evaluate frozen stop-test verdict (`PASS` / `STOP` / `VOID`) for `mode`."""
    if mode not in MODES:
        return {"verdict": "VOID", "mode": mode, "reason": f"unknown mode {mode!r}"}
    if not host_ok:
        return {"verdict": "VOID", "mode": mode, "reason": "non-i9 or invalid workstation host"}
    if not distinct_binaries:
        return {
            "verdict": "VOID",
            "mode": mode,
            "reason": "base and head binaries are identical (same SHA-256)",
        }

    req_cells = expected_cells(mode, smoke=smoke) if cells is None else list(cells)
    req_cell_set = set(req_cells)
    seen: set[tuple[str, str, int, str, int]] = set()
    by_cell: dict[tuple[str, str, int], list[dict[str, Any]]] = {c: [] for c in req_cells}
    hash_by_corpus: dict[str, tuple[int, str | None]] = {}

    for row in rows:
        corpus = row.get("corpus")
        pinning = row.get("pinning")
        threads = row.get("threads")
        arm = row.get("arm")
        rd = row.get("round")
        cell = (corpus, pinning, threads)
        if cell not in req_cell_set:
            return {"verdict": "VOID", "mode": mode, "reason": f"unexpected cell {cell!r}"}
        if arm not in ARMS:
            return {"verdict": "VOID", "mode": mode, "reason": f"unexpected arm {arm!r}"}
        if not isinstance(rd, int) or isinstance(rd, bool) or not (0 <= rd < rounds):
            return {"verdict": "VOID", "mode": mode, "reason": f"invalid round index {rd!r}"}
        key = (corpus, pinning, threads, arm, rd)
        if key in seen:
            return {"verdict": "VOID", "mode": mode, "reason": f"duplicate observation {key!r}"}
        try:
            ref = expected_refs[corpus] if expected_refs and corpus in expected_refs else None
            validate_timed_row(row, ref)
        except InvalidRun as exc:
            return {"verdict": "VOID", "mode": mode, "reason": str(exc)}

        # Cross-arm byte & hash identity check even when scoring standalone runs.jsonl.
        obs_sig = (int(row["out"]), row.get("hash"))
        prev_sig = hash_by_corpus.get(corpus)
        if prev_sig is not None and prev_sig != obs_sig:
            return {
                "verdict": "VOID",
                "mode": mode,
                "reason": f"output mismatch on {corpus}: {obs_sig} != {prev_sig}",
            }
        hash_by_corpus[corpus] = obs_sig

        seen.add(key)
        by_cell[cell].append(row)

    expected_total = len(req_cells) * len(ARMS) * rounds
    if len(seen) != expected_total:
        return {
            "verdict": "VOID",
            "mode": mode,
            "reason": f"incomplete run coverage: {len(seen)} != {expected_total}",
        }

    # Check exact pair_pos balance when rounds is even and pair_pos is recorded.
    if rounds % 2 == 0:
        for cell in req_cells:
            crows = by_cell[cell]
            if all("pair_pos" in r for r in crows):
                for arm in ARMS:
                    pos0 = sum(1 for r in crows if r["arm"] == arm and int(r["pair_pos"]) == 0)
                    pos1 = sum(1 for r in crows if r["arm"] == arm and int(r["pair_pos"]) == 1)
                    if pos0 != rounds // 2 or pos1 != rounds // 2:
                        return {
                            "verdict": "VOID",
                            "mode": mode,
                            "reason": (
                                f"unbalanced pair_pos on {cell} arm={arm}: pos0={pos0}, pos1={pos1}"
                            ),
                        }

    cell_summaries: dict[str, Any] = {}
    for cell in req_cells:
        corpus, pinning, threads = cell
        cell_summaries[_cell_key(corpus, pinning, threads)] = {
            "corpus": corpus,
            "pinning": pinning,
            "threads": threads,
            **analyze_cell(by_cell[cell], threads),
        }

    checks: list[dict[str, Any]] = []
    if mode == "plan-schedule":
        checks = _evaluate_plan_schedule_gates(cell_summaries)
    elif mode == "trim-kstar":
        checks = _evaluate_trim_kstar_gates(cell_summaries)

    failures = [c for c in checks if not c["passed"]]
    if failures:
        verdict = "STOP"
        reason = "; ".join(f["detail"] for f in failures)
    else:
        verdict = "PASS"
        reason = f"all {len(checks)} frozen {mode} stop-test gates passed"

    return {
        "verdict": verdict,
        "mode": mode,
        "reason": reason,
        "rounds": rounds,
        "smoke": smoke,
        "checks": checks,
        "failed_checks": failures,
        "cells": cell_summaries,
    }


def _evaluate_plan_schedule_gates(cells: dict[str, Any]) -> list[dict[str, Any]]:
    checks: list[dict[str, Any]] = []

    def add(name: str, passed: bool, detail: str) -> None:
        checks.append({"name": name, "passed": bool(passed), "detail": detail})

    # 1. Odd pinned cliffs on wikidata: 5T, 7T (incl. monotonic vs 6T), 9T.
    k5 = _cell_key("wikidata", "pinned", 5)
    if k5 in cells:
        c = cells[k5]
        wr = c["pooled"]["ratio"]
        rr = c["rss"]["ratio"]
        add(
            "wikidata_pinned_5t_wall",
            wr <= PS_5T_WALL_MAX,
            f"5T pinned wall {wr:.4f}x <= {PS_5T_WALL_MAX:.2f}x",
        )
        add(
            "wikidata_pinned_5t_rss",
            rr <= PS_5T_RSS_MAX,
            f"5T pinned RSS {rr:.4f}x <= {PS_5T_RSS_MAX:.2f}x",
        )

    k6 = _cell_key("wikidata", "pinned", 6)
    k7 = _cell_key("wikidata", "pinned", 7)
    if k7 in cells:
        c7 = cells[k7]
        wr = c7["pooled"]["ratio"]
        rr = c7["rss"]["ratio"]
        add(
            "wikidata_pinned_7t_wall",
            wr <= PS_7T_WALL_MAX,
            f"7T pinned wall {wr:.4f}x <= {PS_7T_WALL_MAX:.2f}x",
        )
        add(
            "wikidata_pinned_7t_rss",
            rr <= PS_7T_RSS_MAX,
            f"7T pinned RSS {rr:.4f}x <= {PS_7T_RSS_MAX:.2f}x",
        )
        if k6 in cells:
            w6 = cells[k6]["head_median_wall_s"]
            w7 = c7["head_median_wall_s"]
            add(
                "wikidata_pinned_7t_faster_than_6t",
                w7 < w6,
                f"7T pinned head median {w7:.4f}s < 6T pinned head median {w6:.4f}s",
            )

    k9 = _cell_key("wikidata", "pinned", 9)
    if k9 in cells:
        c = cells[k9]
        wr = c["pooled"]["ratio"]
        rr = c["rss"]["ratio"]
        add(
            "wikidata_pinned_9t_wall",
            wr <= PS_9T_WALL_MAX,
            f"9T pinned wall {wr:.4f}x <= {PS_9T_WALL_MAX:.2f}x",
        )
        add(
            "wikidata_pinned_9t_rss",
            rr <= PS_9T_RSS_MAX,
            f"9T pinned RSS {rr:.4f}x <= {PS_9T_RSS_MAX:.2f}x",
        )

    # 2. Neutral-or-faster cells on wikidata: pinned {4,6,8,10} and unpinned {4,8,10,12,16,20}.
    neutral_specs = [("pinned", t) for t in (4, 6, 8, 10)] + [
        ("unpinned", t) for t in PLAN_SCHEDULE_WIKIDATA_UNPINNED
    ]
    for pinning, t in neutral_specs:
        key = _cell_key("wikidata", pinning, t)
        if key not in cells:
            continue
        c = cells[key]
        wr = c["pooled"]["ratio"]
        rr = c["rss"]["ratio"]
        add(
            f"wikidata_{pinning}_{t}t_wall",
            wr <= PS_NEUTRAL_WALL_MAX,
            f"wikidata {pinning} {t}T wall {wr:.4f}x <= {PS_NEUTRAL_WALL_MAX:.3f}x",
        )
        add(
            f"wikidata_{pinning}_{t}t_rss",
            rr <= PS_NEUTRAL_RSS_MAX,
            f"wikidata {pinning} {t}T RSS {rr:.4f}x <= {PS_NEUTRAL_RSS_MAX:.2f}x",
        )

    # 3. Off-Wikidata guards: text, records, random at pinned {4, 6, 8}.
    for corpus in GUARD_CORPORA:
        for t in PLAN_SCHEDULE_GUARD_COUNTS:
            key = _cell_key(corpus, "pinned", t)
            if key not in cells:
                continue
            c = cells[key]
            wr = c["pooled"]["ratio"]
            rr = c["rss"]["ratio"]
            disj_reg = c["disjoint_slower"] and wr > PS_GUARD_DISJOINT_WALL_MAX
            add(
                f"guard_{corpus}_{t}t_wall",
                not disj_reg,
                (
                    f"guard {corpus} {t}T wall {wr:.4f}x "
                    f"(disjoint_slower={c['disjoint_slower']}, limit <= {PS_GUARD_DISJOINT_WALL_MAX:.2f}x)"
                ),
            )
            add(
                f"guard_{corpus}_{t}t_rss",
                rr <= PS_GUARD_RSS_MAX,
                f"guard {corpus} {t}T RSS {rr:.4f}x <= {PS_GUARD_RSS_MAX:.2f}x",
            )
    return checks


def _evaluate_trim_kstar_gates(cells: dict[str, Any]) -> list[dict[str, Any]]:
    checks: list[dict[str, Any]] = []

    def add(name: str, passed: bool, detail: str) -> None:
        checks.append({"name": name, "passed": bool(passed), "detail": detail})

    for key, c in cells.items():
        corpus = c["corpus"]
        wr = c["pooled"]["ratio"]
        ci_hi = c["pooled"]["ci_hi"]
        rr = c["rss"]["ratio"]
        if corpus == "wikidata":
            ci_ok = ci_hi is not None and ci_hi <= TK_WIKIDATA_CI_HI_MAX
            add(
                f"{key}_wall_ci_hi",
                ci_ok,
                f"{key} wall 95% CI upper {ci_hi} <= {TK_WIKIDATA_CI_HI_MAX:.2f} (ratio={wr:.4f}x)",
            )
            add(
                f"{key}_rss",
                rr <= TK_RSS_MAX,
                f"{key} RSS {rr:.4f}x <= {TK_RSS_MAX:.2f}x",
            )
        else:
            add(
                f"{key}_guard_wall",
                wr <= TK_GUARD_WALL_MAX,
                f"{key} guard wall {wr:.4f}x <= {TK_GUARD_WALL_MAX:.2f}x",
            )
            add(
                f"{key}_guard_rss",
                rr <= TK_RSS_MAX,
                f"{key} guard RSS {rr:.4f}x <= {TK_RSS_MAX:.2f}x",
            )
    return checks


def format_summary(summary: dict[str, Any]) -> str:
    """Format human-readable summary covering pooled, position-stratified, and within-mode stats."""
    lines = [
        f"VERDICT: {summary['verdict']} (mode={summary['mode']}, rounds={summary.get('rounds')})",
        f"REASON: {summary['reason']}",
        "",
    ]
    if "cells" not in summary:
        return "\n".join(lines) + "\n"

    lines.append(
        "cell                      | base_s -> head_s | pooled ratio [95% CI]      | "
        "pos_strat | fast_mode | cpr_rat | rss_rat"
    )
    lines.append("-" * 114)
    for key, c in summary["cells"].items():
        p = c["pooled"]
        ps = c["position_stratified"]["combined_ratio"]
        wm = c["within_mode"]
        fast = wm["by_mode"].get("fast", {}).get("ratio")
        cpr = wm["cycles_per_ref_ratio"]
        rr = c["rss"]["ratio"]
        ci_str = (
            f"[{p['ci_lo']:.4f}, {p['ci_hi']:.4f}]"
            if p["ci_lo"] is not None and p["ci_hi"] is not None
            else "[n/a]"
        )
        ps_str = f"{ps:.4f}x" if ps is not None else "  n/a  "
        fast_str = f"{fast:.4f}x" if fast is not None else "  n/a  "
        cpr_str = f"{cpr:.4f}" if cpr is not None else "  n/a "
        lines.append(
            f"{key:<25s} | {c['base_median_wall_s']:6.4f} -> {c['head_median_wall_s']:6.4f} | "
            f"{p['ratio']:6.4f}x {ci_str:<18s} | {ps_str:^9s} | {fast_str:^9s} | "
            f"{cpr_str:^7s} | {rr:6.4f}x"
        )

    lines.append("")
    lines.append("Stop-test checks:")
    for chk in summary.get("checks", []):
        mark = "PASS" if chk["passed"] else "FAIL"
        lines.append(f"  [{mark}] {chk['name']}: {chk['detail']}")
    return "\n".join(lines) + "\n"


def format_outcome(summary: dict[str, Any]) -> str:
    return f"{summary['verdict']}: {summary['reason']}\n"


# ---------------------------------------------------------------------------
# CLI commands
# ---------------------------------------------------------------------------


def check_registered_params(
    *,
    rounds: int,
    corpus_scale: int,
    prespin_ms: int,
    settle_s: float,
    max_busy: float,
    base_dir: str | None = None,
    head_dir: str | None = None,
) -> None:
    """Refuse parameter overrides on a strict registered i9 run."""
    mismatches: list[str] = []
    if rounds != DEFAULT_ROUNDS:
        mismatches.append(f"rounds={rounds} != {DEFAULT_ROUNDS}")
    if corpus_scale != DEFAULT_CORPUS_SCALE:
        mismatches.append(f"corpus_scale={corpus_scale} != {DEFAULT_CORPUS_SCALE}")
    if prespin_ms != PRESPIN_MS:
        mismatches.append(f"prespin_ms={prespin_ms} != {PRESPIN_MS}")
    if settle_s != SETTLE_S:
        mismatches.append(f"settle_s={settle_s} != {SETTLE_S}")
    if max_busy != MAX_BUSY:
        mismatches.append(f"max_busy={max_busy} != {MAX_BUSY}")
    if base_dir or head_dir:
        mismatches.append("--base-dir/--head-dir are forbidden on registered i9 runs")
    if mismatches:
        raise InvalidRun("frozen registered parameters overridden: " + "; ".join(mismatches))


def cmd_run(args: argparse.Namespace) -> int:
    out = Path(args.out_dir).resolve()
    out.mkdir(parents=True, exist_ok=True)
    mode = args.mode
    smoke = bool(args.smoke)
    require_i9 = not (smoke or args.allow_any_host)
    rounds = args.rounds if args.rounds is not None else (SMOKE_ROUNDS if smoke else DEFAULT_ROUNDS)
    corpus_scale = (
        args.corpus_scale
        if args.corpus_scale is not None
        else (SMOKE_CORPUS_SCALE if smoke else DEFAULT_CORPUS_SCALE)
    )

    save_json(
        out / "provenance.json",
        {
            "mode": mode,
            "strict_i9": require_i9,
            "smoke": smoke,
            "rounds": rounds,
            "corpus_scale": corpus_scale,
            "prespin_ms": args.prespin_ms,
            "settle_s": args.settle,
            "max_busy": args.max_busy,
        },
    )

    try:
        if require_i9:
            check_registered_params(
                rounds=rounds,
                corpus_scale=corpus_scale,
                prespin_ms=args.prespin_ms,
                settle_s=args.settle,
                max_busy=args.max_busy,
                base_dir=args.base_dir,
                head_dir=args.head_dir,
            )
            if output("git", "status", "--porcelain", "--untracked-files=no", cwd=ROOT):
                raise InvalidRun("recipe checkout has uncommitted tracked changes")
        check_baseline_env()
        check_machine(out, require_i9=require_i9)

        with tempfile.TemporaryDirectory(prefix=f"mibz2-{mode}-") as tmp_str:
            tmp = Path(tmp_str)

            # Resolve base & head source checkouts shallow-safely.
            if args.base_dir:
                base_src = Path(args.base_dir).resolve()
                base_sha = output("git", "rev-parse", "HEAD", cwd=base_src)
            else:
                base_ref = args.base_ref or "origin/master"
                base_src = tmp / "src-base"
                base_sha = checkout_ref_shallow(ROOT, base_src, base_ref)

            if args.head_dir:
                head_src = Path(args.head_dir).resolve()
                head_sha = output("git", "rev-parse", "HEAD", cwd=head_src)
            elif args.head_ref:
                head_src = tmp / "src-head"
                head_sha = checkout_ref_shallow(ROOT, head_src, args.head_ref)
            else:
                head_src = ROOT
                head_sha = output("git", "rev-parse", "HEAD", cwd=head_src)

            (out / "commits.txt").write_text(
                f"mode {mode}\n"
                f"recipe {output('git', 'rev-parse', 'HEAD', cwd=ROOT)}\n"
                f"base {base_sha}\n"
                f"head {head_sha}\n",
                encoding="utf-8",
            )
            if REGISTRATION.exists():
                shutil.copyfile(REGISTRATION, out / "registration.md")

            # Strictly isolated per-arm CARGO_TARGET_DIR builds (or prebuilt --smoke binaries).
            if args.base_ab or args.head_ab or args.base_windows:
                if require_i9:
                    raise InvalidRun(
                        "--base-ab/--head-ab/--base-windows are forbidden on registered i9 runs"
                    )
                if not (args.base_ab and args.head_ab):
                    raise InvalidRun("both --base-ab and --head-ab must be provided together")
                base_bins = {"ab": Path(args.base_ab).resolve()}
                if args.base_windows:
                    base_bins["windows"] = Path(args.base_windows).resolve()
                head_bins = {"ab": Path(args.head_ab).resolve()}
            else:
                bin_dir = tmp / "bin"
                base_bins = build_arm_binaries(
                    base_src, tmp / "target-base", bin_dir, "base", include_windows=True
                )
                head_bins = build_arm_binaries(
                    head_src, tmp / "target-head", bin_dir, "head", include_windows=False
                )
            base_sha256, head_sha256 = assert_distinct_binaries(base_bins["ab"], head_bins["ab"])
            save_json(
                out / "binaries.json",
                {
                    "base_ab_sha256": base_sha256,
                    "head_ab_sha256": head_sha256,
                    "base_commit": base_sha,
                    "head_commit": head_sha,
                },
            )

            # Active/passive machine facts from ab env --facts.
            env_facts = json.loads(output(str(head_bins["ab"]), "env", "--facts", cwd=ROOT))
            save_json(out / "env.json", env_facts)
            if require_i9:
                check_cpu_quota_unlimited(env_facts)

            cells = expected_cells(mode, smoke=smoke)
            corpora_needed = {c for c, _, _ in cells}
            files, refs = materialize_inputs(
                out,
                tmp,
                ROOT,
                base_bins["ab"],
                base_bins.get("windows", base_bins["ab"]),
                corpora_needed,
                corpus_scale=corpus_scale,
                wikidata_x4=not smoke,
            )

            expected_refs = run_parity(out, base_bins["ab"], head_bins["ab"], files, refs, cells)
            schedule = build_schedule(cells, rounds=rounds, seed=SEED)
            rows = run_paired_sweep(
                out,
                tmp,
                base_bins["ab"],
                head_bins["ab"],
                files,
                expected_refs,
                schedule,
                prespin_ms=args.prespin_ms,
                settle_s=args.settle,
                max_busy=args.max_busy,
                smoke=smoke,
            )

            summary = score(
                rows,
                mode=mode,
                rounds=rounds,
                cells=cells,
                expected_refs=expected_refs,
                host_ok=True,
                distinct_binaries=True,
                smoke=smoke,
            )
    except InvalidRun as exc:
        summary = {"verdict": "VOID", "mode": mode, "reason": str(exc), "rounds": rounds}

    save_json(out / "summary.json", summary)
    summary_txt = format_summary(summary)
    (out / "summary.txt").write_text(summary_txt, encoding="utf-8")
    (out / "outcome.txt").write_text(format_outcome(summary), encoding="utf-8")
    sys.stdout.write(summary_txt)
    return 0 if summary["verdict"] != "VOID" else 2


def cmd_score(args: argparse.Namespace) -> int:
    jsonl_path = Path(args.runs_jsonl)
    smoke = bool(args.smoke)
    prov_path = jsonl_path.parent / "provenance.json"
    if not smoke and prov_path.exists():
        prov = json.loads(prov_path.read_text(encoding="utf-8"))
        if (
            not prov.get("strict_i9")
            or prov.get("smoke")
            or prov.get("mode") != args.mode
            or prov.get("rounds") != DEFAULT_ROUNDS
            or prov.get("corpus_scale") != DEFAULT_CORPUS_SCALE
            or prov.get("prespin_ms") != PRESPIN_MS
            or prov.get("settle_s") != SETTLE_S
            or prov.get("max_busy") != MAX_BUSY
        ):
            summary: dict[str, Any] = {
                "verdict": "VOID",
                "mode": args.mode,
                "reason": "not the registered i9 run (provenance mismatch)",
                "rounds": int(prov.get("rounds", 0)),
            }
            text = format_summary(summary)
            if args.out:
                out_dir = Path(args.out)
                out_dir.mkdir(parents=True, exist_ok=True)
                save_json(out_dir / "summary.json", summary)
                (out_dir / "summary.txt").write_text(text, encoding="utf-8")
                (out_dir / "outcome.txt").write_text(format_outcome(summary), encoding="utf-8")
            sys.stdout.write(text)
            return 2

    rows = [
        json.loads(line)
        for line in jsonl_path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    rounds = args.rounds
    if rounds is None:
        rounds = max((int(r["round"]) for r in rows), default=-1) + 1
    summary = score(rows, mode=args.mode, rounds=rounds, smoke=smoke)
    text = format_summary(summary)
    if args.out:
        out_dir = Path(args.out)
        out_dir.mkdir(parents=True, exist_ok=True)
        save_json(out_dir / "summary.json", summary)
        (out_dir / "summary.txt").write_text(text, encoding="utf-8")
        (out_dir / "outcome.txt").write_text(format_outcome(summary), encoding="utf-8")
    sys.stdout.write(text)
    return 0 if summary["verdict"] != "VOID" else 2


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="cmd", required=True)

    run_p = sub.add_parser("run", help="Run paired i9 verification and score frozen stop_test")
    run_p.add_argument("out_dir", help="Output directory for run artifacts")
    run_p.add_argument("--mode", choices=MODES, required=True)
    run_p.add_argument("--base-ref", default=None, help="Git ref/SHA for base arm")
    run_p.add_argument("--head-ref", default=None, help="Git ref/SHA for head arm")
    run_p.add_argument("--base-dir", default=None, help="Local checkout directory for base arm")
    run_p.add_argument("--head-dir", default=None, help="Local checkout directory for head arm")
    run_p.add_argument("--base-ab", default=None, help="Prebuilt base ab (--smoke only)")
    run_p.add_argument("--head-ab", default=None, help="Prebuilt head ab (--smoke only)")
    run_p.add_argument("--base-windows", default=None, help="Prebuilt windows (--smoke only)")
    run_p.add_argument("--rounds", type=int, default=None)
    run_p.add_argument("--corpus-scale", type=int, default=None)
    run_p.add_argument("--prespin-ms", type=int, default=PRESPIN_MS)
    run_p.add_argument("--settle", type=float, default=SETTLE_S)
    run_p.add_argument("--max-busy", type=float, default=MAX_BUSY)
    run_p.add_argument("--smoke", action="store_true", help="Fast small-cell smoke run")
    run_p.add_argument("--allow-any-host", action="store_true", help="Skip i9-10900K host check")
    run_p.set_defaults(func=cmd_run)

    score_p = sub.add_parser("score", help="Re-score an existing runs.jsonl")
    score_p.add_argument("runs_jsonl", help="Path to runs.jsonl")
    score_p.add_argument("--mode", choices=MODES, required=True)
    score_p.add_argument("--rounds", type=int, default=None)
    score_p.add_argument("--out", default=None, help="Optional directory to write summaries")
    score_p.add_argument("--smoke", action="store_true")
    score_p.set_defaults(func=cmd_score)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
