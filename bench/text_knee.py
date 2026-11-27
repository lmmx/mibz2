"""Frozen i9-10900K probe: 8-worker knee on a pinned text panel vs wikidata (text-knee-i9)."""

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

from mtf_fold import topology
import wikidata

ROOT = Path(__file__).resolve().parents[1]
REGISTRATION = ROOT / "docs/journal/2026-11-27-text-knee-i9/registration.md"

PANEL_COMMIT = "2fed5f184bf66ddaa50c72779a126dafe7811eef"
WIKIDATA_X4_SHA256 = "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"

PANEL_CORPORA = ("docs_md", "repo_json", "src_rs")
ALL_CORPORA = (*PANEL_CORPORA, "wikidata")
COUNTS = (4, 5, 6, 7, 8, 9, 10)
ROUNDS = 9
STATION_ROUNDS = 3
STATION_PASSES = 5
KNEE_TOLERANCE = 0.02
SEED = 1135
PRESPIN_MS = 100
SETTLE_S = 0.2
MAX_BUSY = 1.5
RUN_TIMEOUT_S = 180

PANEL_SPECS = {
    "docs_md": {
        "glob_desc": "docs/**/*.md",
        "files": 656,
        "unit_bytes": 11_480_438,
        "raw_rep": 1,
        "base_raw_bytes": 11_480_438,
        "base_bz2_bytes": 2_952_993,
        "base_bz2_sha256": "ae6b9f59478b92f306e394eb1a8b36c45a20cd15dfbdf24fcd5ee0eebc5b2c8a",
        "stream_rep": 16,
        "wall_raw_bytes": 183_687_008,
        "wall_bz2_bytes": 47_247_888,
        "wall_bz2_sha256": "857095a4bd46e580c876ae20c95192176ba6a9ec7e32c3a5c68bbed5d8ee188d",
        "wall_raw_sha256": "ef330c7a80dc148c75abdb0ce8c8ce110f6c38d4ee02ee092c1c04483902f8ac",
    },
    "repo_json": {
        "glob_desc": "**/*.json",
        "files": 181,
        "unit_bytes": 1_897_118,
        "raw_rep": 5,
        "base_raw_bytes": 9_485_590,
        "base_bz2_bytes": 1_933_754,
        "base_bz2_sha256": "d87064ec66834ff4733fe0f62644111a5259c3e64f07b1a459957804e357f657",
        "stream_rep": 20,
        "wall_raw_bytes": 189_711_800,
        "wall_bz2_bytes": 38_675_080,
        "wall_bz2_sha256": "83fd2eb4c81fbb6f5cda058c51a8a5f954f3eeaf39d23e5e6b6855e4bfa7b523",
        "wall_raw_sha256": "2778e69b4d0b5f65b3f49cc4c668c41b5e60950d52b9ce7ed17e5ce19398fea6",
    },
    "src_rs": {
        "glob_desc": "src/**/*.rs",
        "files": 52,
        "unit_bytes": 1_525_862,
        "raw_rep": 6,
        "base_raw_bytes": 9_155_172,
        "base_bz2_bytes": 1_757_562,
        "base_bz2_sha256": "dd3329e28b2e2a1c4abfbac743a574ae039bfe9f13988312e13ecd761dcbc05b",
        "stream_rep": 20,
        "wall_raw_bytes": 183_103_440,
        "wall_bz2_bytes": 35_151_240,
        "wall_bz2_sha256": "6e7e6bb4ede8d1662976cbf364524db691d311ff1885c3634b520cc96d558ea1",
        "wall_raw_sha256": "a0486cee34ba6c74823c336024a41189ccf41ba4fb257db593c60e5e2c7285b8",
    },
}

PANEL_MATCHERS = {
    "docs_md": lambda p: p.startswith("docs/") and p.endswith(".md"),
    "repo_json": lambda p: p.endswith(".json"),
    "src_rs": lambda p: p.startswith("src/") and p.endswith(".rs"),
}


class InvalidRun(ValueError):
    """A registered prerequisite or validity invariant failed; the run is VOID."""


def output(*cmd, cwd=ROOT):
    return subprocess.check_output(cmd, cwd=cwd, text=True).strip()


def save(path, obj):
    path.write_text(json.dumps(obj, indent=2, allow_nan=False) + "\n")


def sha256_file(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def reference_bz2(path):
    """Independently decompress a .bz2 file with Python's libbzip2 binding."""
    digest = hashlib.sha256()
    total = 0
    with bz2.open(path, "rb") as stream:
        while chunk := stream.read(1 << 20):
            total += len(chunk)
            digest.update(chunk)
    return {"out": total, "sha256": digest.hexdigest()}


def check_baseline_env(env=None):
    """Require default baseline x86-64 codegen with no RUSTFLAGS overrides."""
    env = os.environ if env is None else env
    for var in ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
        if env.get(var, "").strip():
            raise InvalidRun(f"{var} must be unset for baseline codegen")


def check_cpu_quota_unlimited(env_facts):
    """Require unlimited cgroup CPU quota (supports both `ab env` and `ab env --facts` shapes)."""
    quota = env_facts.get("cpu_quota", "absent")
    if quota is None:
        return
    if isinstance(quota, dict) and quota.get("state") == "unlimited" and quota.get("cpus") is None:
        return
    raise InvalidRun(f"workstation CPU quota must be unlimited (got {quota!r})")


def check_machine(out):
    description = output("lscpu")
    (out / "lscpu.txt").write_text(description + "\n")
    if "i9-10900K" not in description:
        raise InvalidRun("registered machine is the i9-10900K")
    grid = out / "lscpu-e.txt"
    grid.write_text(output("lscpu", "-e") + "\n")
    cores = topology(grid)
    cpus = {cpu for core in cores for cpu in core}
    if len(cores) != 10 or any(len(core) != 2 for core in cores) or len(cpus) != 20:
        raise InvalidRun("expected ten physical cores with both hardware threads")
    if cpus != os.sched_getaffinity(0):
        raise InvalidRun("all 20 workstation CPUs must be available")
    return cores


def materialize_panel_commit(tmp, checkout, commit=PANEL_COMMIT):
    """Return a git directory holding `commit`, fetching shallowly when the runner lacks it."""
    checkout = Path(checkout)
    has_commit = (
        subprocess.run(
            ["git", "cat-file", "-e", f"{commit}^{{commit}}"],
            cwd=checkout,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        ).returncode
        == 0
    )
    if has_commit:
        return checkout
    origin = output("git", "remote", "get-url", "origin", cwd=checkout)
    repo = Path(tmp) / "panel-git"
    subprocess.run(["git", "init", "-q", repo], check=True)
    subprocess.run(
        ["git", "fetch", "-q", "--depth=1", "--no-tags", origin, commit],
        cwd=repo,
        check=True,
    )
    if output("git", "rev-parse", "FETCH_HEAD^{commit}", cwd=repo) != commit:
        raise InvalidRun(f"fetched panel commit differs from registered {commit}")
    return repo


def _read_blobs_batch(repo, commit, relpaths):
    """Stream blobs from `commit` in order using a single `git cat-file --batch` process."""
    query = "".join(f"{commit}:{p}\n" for p in relpaths).encode("utf-8")
    chunks = []
    with subprocess.Popen(
        ["git", "cat-file", "--batch"],
        cwd=repo,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
    ) as proc:
        assert proc.stdin is not None and proc.stdout is not None
        proc.stdin.write(query)
        proc.stdin.close()
        for rel in relpaths:
            header = proc.stdout.readline()
            if not header:
                raise InvalidRun(f"unexpected EOF reading {rel} from {commit}")
            parts = header.decode("utf-8").strip().split()
            if len(parts) != 3 or parts[1] != "blob":
                raise InvalidRun(f"missing or non-blob entry {rel} in {commit}: {header!r}")
            size = int(parts[2])
            data = proc.stdout.read(size)
            if len(data) != size:
                raise InvalidRun(f"truncated blob for {rel} in {commit}")
            lf = proc.stdout.read(1)
            if lf != b"\n":
                raise InvalidRun(f"unexpected delimiter after blob {rel}")
            chunks.append(data)
        rc = proc.wait()
    if rc != 0:
        raise InvalidRun(f"git cat-file --batch exited {rc}")
    return b"".join(chunks)


def build_text_panel(repo, out_dir, commit=PANEL_COMMIT):
    """Build and SHA-256-verify the three pinned text panel corpora."""
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    raw_tree = subprocess.check_output(
        ["git", "ls-tree", "-r", "--name-only", "-z", commit],
        cwd=repo,
    )
    all_paths = sorted(p.decode("utf-8") for p in raw_tree.split(b"\0") if p)

    built = {}
    for name in PANEL_CORPORA:
        spec = PANEL_SPECS[name]
        matched = [p for p in all_paths if PANEL_MATCHERS[name](p)]
        if len(matched) != spec["files"]:
            raise InvalidRun(
                f"{name}: expected {spec['files']} files at {commit}, found {len(matched)}"
            )
        unit = _read_blobs_batch(repo, commit, matched)
        if len(unit) != spec["unit_bytes"]:
            raise InvalidRun(f"{name}: expected {spec['unit_bytes']} unit bytes, got {len(unit)}")
        base_raw = unit * spec["raw_rep"]
        if len(base_raw) != spec["base_raw_bytes"]:
            raise InvalidRun(f"{name}: unexpected base raw length {len(base_raw)}")
        base_bz2 = bz2.compress(base_raw, compresslevel=9)
        if len(base_bz2) != spec["base_bz2_bytes"]:
            raise InvalidRun(f"{name}: unexpected base bz2 length {len(base_bz2)}")
        base_hash = hashlib.sha256(base_bz2).hexdigest()
        if base_hash != spec["base_bz2_sha256"]:
            raise InvalidRun(
                f"{name}: base bz2 SHA-256 mismatch ({base_hash} != {spec['base_bz2_sha256']})"
            )
        base_path = out_dir / f"{name}-base.bz2"
        base_path.write_bytes(base_bz2)

        wall_path = out_dir / f"{name}.bz2"
        with wall_path.open("wb") as dest:
            for _ in range(spec["stream_rep"]):
                dest.write(base_bz2)
        if wall_path.stat().st_size != spec["wall_bz2_bytes"]:
            raise InvalidRun(f"{name}: unexpected wall bz2 length")
        wall_hash = sha256_file(wall_path)
        if wall_hash != spec["wall_bz2_sha256"]:
            raise InvalidRun(
                f"{name}: wall bz2 SHA-256 mismatch ({wall_hash} != {spec['wall_bz2_sha256']})"
            )
        built[name] = {
            "base_path": base_path,
            "wall_path": wall_path,
            "base_bz2_sha256": base_hash,
            "wall_bz2_sha256": wall_hash,
            "base_raw_bytes": spec["base_raw_bytes"],
            "wall_raw_bytes": spec["wall_raw_bytes"],
            "expected_wall_raw_sha256": spec["wall_raw_sha256"],
        }
    return built


def materialize_wikidata(tmp, checkout, windows_bin):
    """Build or verify corpora/wikidata.bz2 and its 4x multistream wall input, shallow-safely."""
    checkout = Path(checkout)
    tmp = Path(tmp)
    spec = json.loads((checkout / "bench/wikidata.json").read_text())
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
        subprocess.run(["git", "init", "-q", bd_repo], check=True)
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
        "base_path": sample,
        "wall_path": wall_path,
        "base_bz2_sha256": base_hash,
        "wall_bz2_sha256": wall_hash,
    }


def build_binaries(checkout, tmp):
    """Build ab, station_balance, and windows from checkout HEAD with baseline codegen."""
    check_baseline_env()
    target_dir = Path(tmp) / "target"
    bin_dir = Path(tmp) / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    env = {**os.environ, "CARGO_TARGET_DIR": str(target_dir)}
    subprocess.run(
        [
            "cargo",
            "build",
            "--release",
            "--locked",
            "--example",
            "ab",
            "--example",
            "station_balance",
            "--example",
            "windows",
        ],
        cwd=checkout,
        env=env,
        check=True,
    )
    bins = {}
    for name in ("ab", "station_balance", "windows"):
        src = target_dir / "release/examples" / name
        dst = bin_dir / name
        shutil.copy2(src, dst)
        bins[name] = dst
    return bins


def parse_station_balance_line(line):
    """Parse one tab-separated output line from `examples/station_balance.rs`."""
    fields = line.strip().split("\t")
    if len(fields) != 11:
        raise InvalidRun(f"unexpected station_balance output ({len(fields)} fields): {line!r}")
    row = {
        "file": fields[0],
        "blocks": int(fields[1]),
        "full_blocks": int(fields[2]),
        "syms": int(fields[3]),
        "out_bytes": int(fields[4]),
        "in_bytes": int(fields[5]),
        "s_ms": float(fields[6]),
        "w_ms": float(fields[7]),
        "k_star": float(fields[8]),
        "s_ns_per_sym": float(fields[9]),
        "w_ns_per_out": float(fields[10]),
    }
    for key in ("s_ms", "w_ms", "k_star", "s_ns_per_sym", "w_ns_per_out"):
        if not math.isfinite(row[key]) or row[key] <= 0.0:
            raise InvalidRun(f"invalid station_balance {key}={row[key]}")
    if row["blocks"] <= 0 or row["syms"] <= 0 or row["out_bytes"] <= 0 or row["in_bytes"] <= 0:
        raise InvalidRun(f"non-positive counts in station_balance row: {row}")
    return row


def run_ab_child(ab_bin, arm, path, threads):
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


def run_parity(out, ab_bin, corpora):
    """Verify untimed mibz2 parity against ab's libbzip2 arm and Python libbzip2 SHA-256."""
    refs = {}
    with (out / "parity.jsonl").open("x", buffering=1) as log:
        for name in ALL_CORPORA:
            info = corpora[name]
            base_ref = reference_bz2(info["base_path"])
            if "expected_wall_raw_sha256" in info:
                wall_ref = reference_bz2(info["wall_path"])
                if wall_ref["sha256"] != info["expected_wall_raw_sha256"]:
                    raise InvalidRun(
                        f"{name}: decompressed wall SHA-256 mismatch ({wall_ref['sha256']})"
                    )
                if wall_ref["out"] != info["wall_raw_bytes"]:
                    raise InvalidRun(f"{name}: decompressed wall byte count mismatch")
                if base_ref["out"] != info["base_raw_bytes"]:
                    raise InvalidRun(f"{name}: decompressed base byte count mismatch")
                expected_wall_out = wall_ref["out"]
                wall_raw_sha256 = wall_ref["sha256"]
            else:
                expected_wall_out = base_ref["out"] * 4
                wall_raw_sha256 = None

            lib_row = run_ab_child(ab_bin, "libbzip2", info["wall_path"], 1)
            if lib_row.get("err") not in (None, "") or lib_row.get("termination") not in (
                None,
                "success",
            ):
                raise InvalidRun(f"{name}: libbzip2 parity decode errored: {lib_row}")
            if lib_row.get("out") != expected_wall_out or not lib_row.get("hash"):
                raise InvalidRun(
                    f"{name}: libbzip2 output mismatch: got ({lib_row.get('out')}, "
                    f"{lib_row.get('hash')}), expected out={expected_wall_out}"
                )

            mib_row = run_ab_child(ab_bin, "mibz2", info["wall_path"], 8)
            if mib_row.get("err") not in (None, "") or mib_row.get("termination") not in (
                None,
                "success",
            ):
                raise InvalidRun(f"{name}: mibz2 parity decode errored: {mib_row}")
            if mib_row.get("out") != lib_row["out"] or mib_row.get("hash") != lib_row["hash"]:
                raise InvalidRun(
                    f"{name}: mibz2 parity mismatch against libbzip2: "
                    f"got ({mib_row.get('out')}, {mib_row.get('hash')}), "
                    f"expected ({lib_row['out']}, {lib_row['hash']})"
                )

            log.write(json.dumps({"corpus": name, "arm": "libbzip2", "row": lib_row}) + "\n")
            log.write(json.dumps({"corpus": name, "arm": "mibz2", "row": mib_row}) + "\n")
            refs[name] = {
                "base_out": base_ref["out"],
                "base_in": info["base_path"].stat().st_size,
                "base_raw_sha256": base_ref["sha256"],
                "base_bz2_sha256": info["base_bz2_sha256"],
                "wall_out": expected_wall_out,
                "wall_in": info["wall_path"].stat().st_size,
                "wall_raw_sha256": wall_raw_sha256,
                "wall_bz2_sha256": info["wall_bz2_sha256"],
                "ab_hash": lib_row["hash"],
            }
    return refs


def wall_plan(rounds=ROUNDS, corpora=ALL_CORPORA, counts=COUNTS, seed=SEED):
    """Build deterministic per-round shuffled schedule of all (corpus, k) cells."""
    rng = random.Random(seed)
    cells = [(corpus, k) for corpus in corpora for k in counts]
    schedule = []
    for r in range(rounds):
        order = list(cells)
        rng.shuffle(order)
        for pos, (corpus, k) in enumerate(order):
            schedule.append((r, pos, corpus, k))
    return schedule


def station_plan(rounds=STATION_ROUNDS, corpora=ALL_CORPORA, seed=SEED + 1):
    """Build deterministic per-round shuffled schedule for station_balance."""
    rng = random.Random(seed)
    schedule = []
    for r in range(rounds):
        order = list(corpora)
        rng.shuffle(order)
        for pos, corpus in enumerate(order):
            schedule.append((r, pos, corpus))
    return schedule


def run_station_balance(out, station_bin, corpora, refs, cpu0):
    """Run interleaved single-thread station_balance rounds pinned to physical core 0."""
    rows = []
    with (out / "station_balance.jsonl").open("x", buffering=1) as log:
        for r, pos, name in station_plan():
            base_path = corpora[name]["base_path"]
            proc = subprocess.run(
                [
                    "taskset",
                    "-c",
                    str(cpu0),
                    str(station_bin),
                    str(base_path),
                    str(STATION_PASSES),
                ],
                text=True,
                capture_output=True,
                timeout=RUN_TIMEOUT_S,
                check=False,
            )
            if proc.returncode != 0:
                raise InvalidRun(
                    f"station_balance failed on {name} round {r}: {proc.stderr.strip()}"
                )
            lines = [ln for ln in proc.stdout.splitlines() if ln.strip()]
            if len(lines) != 1:
                raise InvalidRun(
                    f"expected 1 line from station_balance on {name}, got {len(lines)}"
                )
            parsed = parse_station_balance_line(lines[0])
            if parsed["out_bytes"] != refs[name]["base_out"]:
                raise InvalidRun(
                    f"station_balance {name} out_bytes {parsed['out_bytes']} != {refs[name]['base_out']}"
                )
            if parsed["in_bytes"] != refs[name]["base_in"]:
                raise InvalidRun(
                    f"station_balance {name} in_bytes {parsed['in_bytes']} != {refs[name]['base_in']}"
                )
            row = {**parsed, "corpus": name, "round": r, "order_in_round": pos, "cpu": cpu0}
            log.write(json.dumps(row, allow_nan=False) + "\n")
            rows.append(row)
    return rows


def run_wall_sweep(out, tmp, ab_bin, corpora, refs):
    """Run 9 interleaved rounds over all (corpus, k in 4..10) cells via `ab run`."""
    rows = []
    cell_file = Path(tmp) / "cell.jsonl"
    env = dict(os.environ)
    env.pop("AB_SHA256", None)
    with (out / "runs.jsonl").open("x", buffering=1) as log:
        for r, pos, name, k in wall_plan():
            if cell_file.exists():
                cell_file.unlink()
            cmd = [
                str(ab_bin),
                "run",
                str(cell_file),
                "--arms",
                "mibz2",
                "--threads",
                str(k),
                "--rounds",
                "1",
                "--prespin-ms",
                str(PRESPIN_MS),
                "--settle",
                str(SETTLE_S),
                "--max-busy",
                str(MAX_BUSY),
                "--sysfs-state",
                str(corpora[name]["wall_path"]),
            ]
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
                    f"ab run failed ({name} k={k} r={r}, rc={proc.returncode}): "
                    f"{proc.stderr.strip()}"
                )
            lines = [ln for ln in cell_file.read_text().splitlines() if ln.strip()]
            if len(lines) != 1:
                raise InvalidRun(
                    f"expected 1 JSON line from ab run ({name} k={k} r={r}), got {len(lines)}"
                )
            sample = json.loads(lines[0])
            row = {
                **sample,
                "corpus": name,
                "round": r,
                "order_in_round": pos,
            }
            validate_wall_row(row, refs[name])
            log.write(json.dumps(row, allow_nan=False) + "\n")
            rows.append(row)
    return rows


def validate_wall_row(row, expected):
    """Validate one timed wall row against frozen validity criteria."""
    if row.get("arm") not in (None, "mibz2"):
        raise InvalidRun(f"unexpected arm {row.get('arm')!r}")
    if row.get("err") not in (None, ""):
        raise InvalidRun(f"decode error in row: {row.get('err')!r}")
    if row.get("termination") not in (None, "success"):
        raise InvalidRun(f"non-success termination: {row.get('termination')!r}")
    if row.get("out") != expected["wall_out"]:
        raise InvalidRun(f"output bytes mismatch: {row.get('out')} != {expected['wall_out']}")
    if row.get("hash") != expected["ab_hash"]:
        raise InvalidRun(f"output hash mismatch: {row.get('hash')} != {expected['ab_hash']}")
    wall = row.get("wall")
    if not isinstance(wall, (int, float)) or not math.isfinite(wall) or wall <= 0.0:
        raise InvalidRun(f"invalid wall time: {wall!r}")
    rss = row.get("rss_kb")
    if not isinstance(rss, (int, float)) or not math.isfinite(rss) or rss <= 0:
        raise InvalidRun(f"invalid rss_kb: {rss!r}")


def compute_knee(median_by_k, counts=COUNTS, tolerance=KNEE_TOLERANCE):
    """Return (best_k, best_wall_s, threshold_s, knee_k) per the frozen stop_test."""
    best_k = min(counts, key=lambda k: (median_by_k[k], k))
    best_wall = median_by_k[best_k]
    threshold = best_wall * (1.0 + tolerance)
    knee_k = min(k for k in counts if median_by_k[k] <= threshold)
    return best_k, best_wall, threshold, knee_k


def score(
    wall_rows,
    station_rows,
    expected_refs,
    *,
    host_ok=True,
    rounds=ROUNDS,
    station_rounds=STATION_ROUNDS,
):
    """Score wall and station_balance rows under the frozen text-knee-i9 stop_test."""
    if not host_ok:
        return {"verdict": "VOID", "reason": "non-i9 or invalid workstation host"}

    # Validate and index wall rows: require exact coverage with zero duplicates.
    seen_wall = set()
    by_corpus_k = {(c, k): [] for c in ALL_CORPORA for k in COUNTS}
    rss_by_corpus_k = {(c, k): [] for c in ALL_CORPORA for k in COUNTS}
    for row in wall_rows:
        corpus = row.get("corpus")
        k = row.get("threads", row.get("k"))
        r = row.get("round")
        if corpus not in expected_refs or corpus not in ALL_CORPORA:
            return {"verdict": "VOID", "reason": f"unknown corpus {corpus!r}"}
        if k not in COUNTS:
            return {"verdict": "VOID", "reason": f"unexpected thread count {k!r}"}
        if not isinstance(r, int) or not (0 <= r < rounds):
            return {"verdict": "VOID", "reason": f"invalid round index {r!r}"}
        key = (corpus, k, r)
        if key in seen_wall:
            return {"verdict": "VOID", "reason": f"duplicate wall cell {key}"}
        try:
            validate_wall_row(row, expected_refs[corpus])
        except InvalidRun as exc:
            return {"verdict": "VOID", "reason": str(exc)}
        seen_wall.add(key)
        by_corpus_k[(corpus, k)].append(float(row["wall"]))
        rss_by_corpus_k[(corpus, k)].append(float(row["rss_kb"]))

    expected_wall_count = len(ALL_CORPORA) * len(COUNTS) * rounds
    if len(seen_wall) != expected_wall_count:
        return {
            "verdict": "VOID",
            "reason": f"incomplete wall sweep: {len(seen_wall)} != {expected_wall_count}",
        }

    # Validate and index station_balance rows.
    seen_station = set()
    station_by_corpus = {c: [] for c in ALL_CORPORA}
    for row in station_rows:
        corpus = row.get("corpus")
        r = row.get("round")
        if corpus not in expected_refs or corpus not in ALL_CORPORA:
            return {"verdict": "VOID", "reason": f"unknown station_balance corpus {corpus!r}"}
        if not isinstance(r, int) or not (0 <= r < station_rounds):
            return {"verdict": "VOID", "reason": f"invalid station_balance round {r!r}"}
        key = (corpus, r)
        if key in seen_station:
            return {"verdict": "VOID", "reason": f"duplicate station_balance cell {key}"}
        for metric in ("s_ms", "w_ms", "k_star", "s_ns_per_sym", "w_ns_per_out"):
            val = row.get(metric)
            if not isinstance(val, (int, float)) or not math.isfinite(val) or val <= 0.0:
                return {"verdict": "VOID", "reason": f"invalid station_balance {metric}={val!r}"}
        if row.get("out_bytes") != expected_refs[corpus]["base_out"]:
            return {"verdict": "VOID", "reason": f"station_balance out_bytes mismatch for {corpus}"}
        if row.get("in_bytes") != expected_refs[corpus]["base_in"]:
            return {"verdict": "VOID", "reason": f"station_balance in_bytes mismatch for {corpus}"}
        seen_station.add(key)
        station_by_corpus[corpus].append(row)

    expected_station_count = len(ALL_CORPORA) * station_rounds
    if len(seen_station) != expected_station_count:
        return {
            "verdict": "VOID",
            "reason": (
                f"incomplete station_balance: {len(seen_station)} != {expected_station_count}"
            ),
        }

    # Aggregate station_balance medians first so r = k* / k*_wikidata is exact.
    station_summary = {}
    for corpus in ALL_CORPORA:
        srows = station_by_corpus[corpus]
        s_ms = statistics.median(r["s_ms"] for r in srows)
        w_ms = statistics.median(r["w_ms"] for r in srows)
        k_star = statistics.median(r["k_star"] for r in srows)
        station_summary[corpus] = {
            "blocks": srows[0]["blocks"],
            "full_blocks": srows[0]["full_blocks"],
            "syms": srows[0]["syms"],
            "out_bytes": srows[0]["out_bytes"],
            "in_bytes": srows[0]["in_bytes"],
            "compression_ratio": srows[0]["out_bytes"] / srows[0]["in_bytes"],
            "s_ms_median": s_ms,
            "w_ms_median": w_ms,
            "k_star_median": k_star,
            "s_ns_per_sym_median": statistics.median(r["s_ns_per_sym"] for r in srows),
            "w_ns_per_out_median": statistics.median(r["w_ns_per_out"] for r in srows),
        }
    wiki_k_star = station_summary["wikidata"]["k_star_median"]
    for corpus in ALL_CORPORA:
        station_summary[corpus]["r_vs_wikidata"] = (
            station_summary[corpus]["k_star_median"] / wiki_k_star
        )

    # Aggregate wall curves and measured knees.
    corpora_summary = {}
    for corpus in ALL_CORPORA:
        medians = {k: statistics.median(by_corpus_k[(corpus, k)]) for k in COUNTS}
        rss_medians = {k: statistics.median(rss_by_corpus_k[(corpus, k)]) for k in COUNTS}
        best_k, best_wall, threshold, knee_k = compute_knee(medians)
        corpora_summary[corpus] = {
            "median_wall_s": {str(k): medians[k] for k in COUNTS},
            "median_rss_kb": {str(k): rss_medians[k] for k in COUNTS},
            "best_k": best_k,
            "best_wall_s": best_wall,
            "knee_threshold_s": threshold,
            "knee_k": knee_k,
            "station_balance": station_summary[corpus],
        }

    panel_knees = sorted(corpora_summary[c]["knee_k"] for c in PANEL_CORPORA)
    panel_knee = panel_knees[1]  # Exact integer median of 3 panel corpora.
    wikidata_knee = corpora_summary["wikidata"]["knee_k"]
    panel_median_k_star = statistics.median(
        station_summary[c]["k_star_median"] for c in PANEL_CORPORA
    )
    panel_median_r = statistics.median(station_summary[c]["r_vs_wikidata"] for c in PANEL_CORPORA)

    if panel_knee <= 7 and wikidata_knee >= 8:
        verdict = "CONFIRMED"
        reason = f"panel median knee k={panel_knee} <= 7 while wikidata knee k={wikidata_knee} >= 8"
    elif panel_knee >= 8:
        verdict = "REFUTED"
        reason = (
            f"panel median knee k={panel_knee} >= 8 (wikidata knee k={wikidata_knee}); "
            "VM station ratio does not transfer to i9 knee"
        )
    else:
        verdict = "INCONCLUSIVE"
        reason = (
            f"panel median knee k={panel_knee} <= 7 and wikidata knee k={wikidata_knee} <= 7; "
            f"i9 k*={panel_median_k_star:.2f} (panel median, r={panel_median_r:.3f}) vs "
            f"wikidata k*={wiki_k_star:.2f}"
        )

    return {
        "verdict": verdict,
        "reason": reason,
        "panel_commit": PANEL_COMMIT,
        "rounds": rounds,
        "station_rounds": station_rounds,
        "knee_tolerance": KNEE_TOLERANCE,
        "panel_knees": {c: corpora_summary[c]["knee_k"] for c in PANEL_CORPORA},
        "panel_median_knee": panel_knee,
        "wikidata_knee": wikidata_knee,
        "panel_median_k_star": panel_median_k_star,
        "panel_median_r_vs_wikidata": panel_median_r,
        "wikidata_k_star": wiki_k_star,
        "corpora": corpora_summary,
    }


def format_outcome(summary):
    lines = [
        f"{summary['verdict']}: {summary['reason']}",
        (
            f"panel_median_knee={summary['panel_median_knee']} "
            f"wikidata_knee={summary['wikidata_knee']} "
            f"panel_median_k_star={summary['panel_median_k_star']:.2f} "
            f"panel_median_r={summary['panel_median_r_vs_wikidata']:.3f} "
            f"wikidata_k_star={summary['wikidata_k_star']:.2f}"
        ),
    ]
    for name in ALL_CORPORA:
        c = summary["corpora"][name]
        sb = c["station_balance"]
        lines.append(
            f"{name}: knee={c['knee_k']} best_k={c['best_k']} "
            f"best_wall_ms={c['best_wall_s'] * 1000:.2f} "
            f"k_star={sb['k_star_median']:.2f} r={sb['r_vs_wikidata']:.3f} "
            f"S_ms={sb['s_ms_median']:.2f} W_ms={sb['w_ms_median']:.2f}"
        )
    return "\n".join(lines) + "\n"


def run(out):
    out = Path(out).resolve()
    check_baseline_env()
    cores = check_machine(out)
    if output("git", "status", "--porcelain", "--untracked-files=no"):
        raise InvalidRun("recipe checkout has tracked changes")
    toolchain = output("rustc", "-Vv")
    if not toolchain.startswith("rustc 1.99.1 "):
        raise InvalidRun("registered toolchain requires Rust 1.99.1")
    if not REGISTRATION.exists():
        raise InvalidRun(f"missing frozen registration {REGISTRATION}")
    shutil.copyfile(REGISTRATION, out / "registration.md")

    save(
        out / "provenance.json",
        {
            "head": output("git", "rev-parse", "HEAD"),
            "head_src_tree": output("git", "rev-parse", "HEAD:src"),
            "head_cargo_toml": output("git", "rev-parse", "HEAD:Cargo.toml"),
            "head_cargo_lock": output("git", "rev-parse", "HEAD:Cargo.lock"),
            "head_ab_rs": output("git", "rev-parse", "HEAD:examples/ab.rs"),
            "head_station_balance_rs": output(
                "git", "rev-parse", "HEAD:examples/station_balance.rs"
            ),
            "panel_commit": PANEL_COMMIT,
            "rustc": toolchain,
            "seed": SEED,
            "station_cpu": cores[0][0],
        },
    )

    with tempfile.TemporaryDirectory(dir=out.parent) as tmpdir:
        tmp = Path(tmpdir)
        bins = build_binaries(ROOT, tmp)
        save(
            out / "binaries.json",
            {name: sha256_file(path) for name, path in bins.items()},
        )
        env_facts = json.loads(output(str(bins["ab"]), "env", "--facts"))
        save(out / "env.json", env_facts)
        check_cpu_quota_unlimited(env_facts)

        panel_repo = materialize_panel_commit(tmp, ROOT, PANEL_COMMIT)
        corpora = build_text_panel(panel_repo, tmp / "corpora", PANEL_COMMIT)
        corpora["wikidata"] = materialize_wikidata(tmp, ROOT, bins["windows"])

        refs = run_parity(out, bins["ab"], corpora)
        save(out / "inputs.json", refs)

        station_rows = run_station_balance(out, bins["station_balance"], corpora, refs, cores[0][0])
        wall_rows = run_wall_sweep(out, tmp, bins["ab"], corpora, refs)

        summary = score(wall_rows, station_rows, refs, host_ok=True)
        save(out / "summary.json", summary)
        if summary["verdict"] == "VOID":
            (out / "outcome.txt").write_text(f"VOID: {summary['reason']}\n")
            raise SystemExit(1)
        (out / "outcome.txt").write_text(format_outcome(summary))


if __name__ == "__main__":
    dest = Path(sys.argv[1]).resolve()
    try:
        run(dest)
    except InvalidRun as err:
        (dest / "outcome.txt").write_text(f"VOID: {err}\n")
        raise SystemExit(1) from err
