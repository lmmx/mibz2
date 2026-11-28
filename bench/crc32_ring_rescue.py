"""Registered i9-10900K run: the zero-table AVX2 ring CRC-32 against its master merge base.

The candidate is `src/crc.rs` from PR #1179 (no-prefetch arm), rebased unchanged apart from
`# Safety` sections. It is built from this checkout (whose `src` tree must be the pinned one)
and compared with the master commit it is based on, built in a private worktree. Timing is
`ab run` paired rounds (bench 3.9.0: held pin, pre-spin, 2 s settle); the per-station 1T
`station_balance` pass is a diagnostic only. The decision rule is frozen in
docs/journal/2026-11-28-crc32-ring-rescue/registration.md before any data.
"""

import bz2
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import tempfile

from mtf_fold import paired_ratio_ci, topology
import wikidata


ROOT = Path(__file__).resolve().parents[1]
REGISTRATION = "docs/journal/2026-11-28-crc32-ring-rescue/registration.md"
# The master commit the candidate is based on, and the two `src` trees.
BASE = "0179ca9ddd74d49c83d3fe064c2f076a9d2f504a"
PINNED_TREES = {"head": "970ee2020bfe99902aa7ee83111f6c21fcbe3627", "base": "8b8180589d333afb68a44df365a1c9a5a880e879"}
X4_SHA256 = "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"
WIKIDATA_COUNTS = (1, 4, 8, 10)
GUARD_CORPORA = ("text", "records", "random", "runs", "pattern", "onebyte")
GUARD_COUNTS = (1, 8)
PRIMARY = ("wikidata", "text")
ROUNDS = 16
STATION_ROUNDS = 5
STATION_PASSES = 5
PRESPIN_MS = 100
SETTLE_S = "2"  # not 0.2: that lands in HWP's ramp hysteresis (i9 clock-modes verdict)
MAX_BUSY = "1.5"

CELLS = tuple(("wikidata", k) for k in WIKIDATA_COUNTS) + tuple(
    (name, k) for name in GUARD_CORPORA for k in GUARD_COUNTS
)
STATION_CORPORA = ("wikidata", "text", "records", "random")

# Frozen decision rule (registration section 3).
GAIN_HI = 0.985  # wikidata/1 wall: upper 95% bound at or below this (>= 1.5% paid)
SECONDARY_WALL = 1.020  # secondary: STOP iff point above this and lower bound above 1
RSS_K1_GM = 1.005  # every k = 1 cell: paired GM of peak RSS ...
RSS_K1_KIB = 256  # ... or median paired delta at most this
RSS_PRIMARY_GM = 1.030  # wikidata/text at k >= 4
RSS_SECONDARY_GM = 1.050  # secondary guards at k = 8
RING_SYMBOL = "avx2_ring8_update"


class InvalidRun(ValueError):
    """A registered prerequisite failed or the measurement cannot support a verdict."""


def output(*args, cwd=ROOT, env=None):
    return subprocess.check_output(args, cwd=cwd, text=True, env=env).strip()


def save(path, obj):
    path.write_text(json.dumps(obj, indent=2, allow_nan=False) + "\n")


def sha256_file(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def reference(path):
    with bz2.open(path, "rb") as source:
        digest = hashlib.sha256()
        size = 0
        while buf := source.read(1 << 20):
            size += len(buf)
            digest.update(buf)
    return {"out": size, "sha256": digest.hexdigest()}


def machine(out):
    description = output("lscpu")
    (out / "lscpu.txt").write_text(description + "\n")
    if "i9-10900K" not in description:
        raise InvalidRun("workstation model differs from the registration")
    grid = out / "lscpu-e.txt"
    grid.write_text(output("lscpu", "-e") + "\n")
    cores = topology(grid)
    cpus = {cpu for core in cores for cpu in core}
    if len(cores) != 10 or len(cpus) != 20 or cpus != os.sched_getaffinity(0):
        raise InvalidRun("expected unrestricted 10-core / 20-CPU i9 topology")
    return cores[0][0]


def sources(tmp, root=ROOT):
    """The candidate is this checkout; the control is BASE in a private worktree."""
    if output("git", "rev-parse", "HEAD:src", cwd=root) != PINNED_TREES["head"]:
        raise InvalidRun("this checkout's src tree is not the registered candidate")
    if subprocess.run(["git", "cat-file", "-e", f"{BASE}^{{commit}}"], cwd=root).returncode:
        subprocess.run(["git", "fetch", "-q", "--no-tags", "origin", BASE], cwd=root, check=True)
    if output("git", "rev-parse", f"{BASE}:src", cwd=root) != PINNED_TREES["base"]:
        raise InvalidRun("the control's src tree is not the registered base")
    if subprocess.run(["git", "merge-base", "--is-ancestor", BASE, "HEAD"], cwd=root).returncode:
        raise InvalidRun("the control is not an ancestor of the candidate")
    for path in ("Cargo.toml", "Cargo.lock", "examples/ab.rs", "examples/station_balance.rs"):
        if output("git", "rev-parse", f"HEAD:{path}", cwd=root) != output(
            "git", "rev-parse", f"{BASE}:{path}", cwd=root
        ):
            raise InvalidRun(f"{path} differs between candidate and control")
    base = tmp / "base-src"
    subprocess.run(["git", "worktree", "add", "-q", "--detach", base, BASE], cwd=root, check=True)
    return {"head": root, "base": base}


def ring_symbols(path):
    return [line for line in output("nm", "-C", str(path)).splitlines() if RING_SYMBOL in line]


def build(out, tmp, root=ROOT):
    trees = sources(tmp, root)
    bins = {}
    env = {
        k: v
        for k, v in os.environ.items()
        if k not in ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS")
    }
    # The base worktree must outlive the build: `ab gen` reads its text corpus from the
    # compiled-in CARGO_MANIFEST_DIR, so it is removed only at the end of `run`.
    for arm in ("base", "head"):
        names = ["ab", "station_balance"] + (["windows"] if arm == "base" else [])
        cmd = ["cargo", "build", "--release", "--locked"]
        for name in names:
            cmd.extend(["--example", name])
        target = tmp / f"target-{arm}"
        subprocess.run(
            cmd, cwd=trees[arm], check=True, env={**env, "CARGO_TARGET_DIR": str(target)}
        )
        for name in names:
            dest = tmp / f"{arm}-{name}"
            shutil.copy2(target / "release/examples" / name, dest)
            bins[arm, name] = dest
    audit = {arm: ring_symbols(bins[arm, "ab"]) for arm in ("base", "head")}
    if audit["base"] or not audit["head"]:
        raise InvalidRun("the ring kernel is not present in exactly the candidate build")
    save(out / "codegen_audit.json", audit)
    save(out / "binaries.json", {p.name: sha256_file(p) for p in bins.values()})
    shutil.copyfile(root / REGISTRATION, out / "registration.md")
    return bins


def inputs(out, tmp, bins, root=ROOT):
    spec = json.loads((root / "bench/wikidata.json").read_text())
    wiki = root / "corpora/wikidata.bz2"
    if not wiki.exists() or sha256_file(wiki) != spec["corpus_sha256"]:
        win = tmp / "windows"
        win.mkdir()
        bench_git = tmp / "bench-data-git"
        bench_git.mkdir()
        subprocess.run(["git", "init", "-q"], cwd=bench_git, check=True)
        origin = output("git", "remote", "get-url", "origin", cwd=root)
        subprocess.run(
            ["git", "fetch", "--depth", "1", "--no-tags", origin, "bench-data"],
            cwd=bench_git,
            check=True,
        )
        for w in spec["windows"]:
            name = w["name"] + ".bin"
            (win / name).write_bytes(
                subprocess.check_output(
                    ["git", "show", f"FETCH_HEAD:{spec['windows_dir']}/{name}"], cwd=bench_git
                )
            )
        wiki = tmp / "wikidata-single.bz2"
        wikidata.build(spec, win, str(bins["base", "windows"]), wiki)
    x4_dir = tmp / "x4"
    x4_dir.mkdir()
    x4 = x4_dir / "wikidata.bz2"
    with x4.open("wb") as dest:
        for _ in range(4):
            with wiki.open("rb") as source:
                shutil.copyfileobj(source, dest)
    if sha256_file(x4) != X4_SHA256:
        raise InvalidRun("registered Wikidata x4 input hash mismatch")
    gen = tmp / "gen"
    subprocess.run([bins["base", "ab"], "gen", str(gen), "4"], check=True)
    files = {"wikidata": x4, **{n: gen / f"{n}.bz2" for n in GUARD_CORPORA}}
    files["wikidata-single"] = wiki
    refs = {n: reference(p) for n, p in files.items()}
    save(
        out / "inputs.json",
        {n: {**refs[n], "compressed_sha256": sha256_file(p)} for n, p in files.items()},
    )
    return files, refs


def canonical_arm(label, head_exe=None):
    """Map an `ab run` arm label (`mibz2` or `mibz2@/path`) to `base` or `head`."""
    if label in ("base", "mibz2"):
        return "base"
    if label == "head":
        return "head"
    if label.startswith("mibz2@"):
        path = label.split("@", 1)[1]
        if head_exe is not None and Path(path).resolve() != Path(head_exe).resolve():
            raise InvalidRun(f"unexpected candidate executable in arm label: {label}")
        return "head"
    raise InvalidRun(f"unexpected arm label: {label}")


def verify_parity(out, bins, files, refs):
    """Untimed `AB_SHA256=1 ab child` on every cell, both arms, against libbzip2."""
    env = {**os.environ, "AB_SHA256": "1"}
    with (out / "parity.jsonl").open("x", buffering=1) as log:
        for corpus, k in CELLS:
            ref = refs[corpus]
            for arm in ("base", "head"):
                done = subprocess.run(
                    [str(bins[arm, "ab"]), "child", "mibz2", str(k), str(files[corpus])],
                    capture_output=True,
                    text=True,
                    timeout=300,
                    env=env,
                )
                if done.returncode != 0:
                    raise InvalidRun(f"parity child failed on {arm}/{corpus}/{k}: {done.stderr}")
                row = json.loads(done.stdout.strip())
                if row.get("err", "") not in (None, ""):
                    raise InvalidRun(f"parity decoder error on {arm}/{corpus}/{k}: {row['err']}")
                if row.get("out") != ref["out"] or row.get("sha256") != ref["sha256"]:
                    raise InvalidRun(f"parity byte/SHA-256 mismatch on {arm}/{corpus}/{k}")
                h = row.get("hash")
                if not h:
                    raise InvalidRun(f"missing hash on parity row {arm}/{corpus}/{k}")
                if ref.setdefault("hash", h) != h:
                    raise InvalidRun(f"hash mismatch between arms on {corpus}/{k}")
                log.write(json.dumps({"arm": arm, "corpus": corpus, "k": k, **row}) + "\n")


def run_ab_timed(out, bins, files, pin_cpu):
    head_exe = str(bins["head", "ab"].resolve())
    rows = []
    groups = (
        ("wikidata.jsonl", WIKIDATA_COUNTS, [str(files["wikidata"])]),
        ("guards.jsonl", GUARD_COUNTS, [str(files[n]) for n in GUARD_CORPORA]),
    )
    for filename, counts, paths in groups:
        dest = out / filename
        cmd = [
            str(bins["base", "ab"]),
            "run",
            str(dest),
            "--arms",
            f"mibz2,mibz2@{head_exe}",
            "--threads",
            ",".join(map(str, counts)),
            "--rounds",
            str(ROUNDS),
            "--pin",
            str(pin_cpu),
            "--prespin-ms",
            str(PRESPIN_MS),
            "--settle",
            SETTLE_S,
            "--max-busy",
            MAX_BUSY,
            *paths,
        ]
        subprocess.run(cmd, check=True)
        for line in dest.read_text().splitlines():
            if line.strip():
                rows.append(json.loads(line))
    return rows


def parse_station_line(text):
    parts = text.strip().split("\t")
    if len(parts) != 11:
        raise InvalidRun(f"unexpected station_balance line: {text!r}")
    keys = ("blocks", "full_blocks", "symbols", "out", "in_bytes")
    rec = {"file": parts[0], **{k: int(v) for k, v in zip(keys, parts[1:6])}}
    for k, v in zip(("s_ms", "w_ms", "k_star", "s_ns_sym", "w_ns_byte"), parts[6:]):
        rec[k] = float(v)
    return rec


def run_station_balance(out, bins, files, refs, pin_cpu):
    """Diagnostic only: 1T per-station times pinned to `pin_cpu`, alternating arm order."""
    rows = []
    with (out / "station_balance.jsonl").open("x", buffering=1) as log:
        for r in range(STATION_ROUNDS):
            order = ("base", "head") if r % 2 == 0 else ("head", "base")
            for corpus in STATION_CORPORA:
                key = "wikidata-single" if corpus == "wikidata" else corpus
                for arm in order:
                    done = subprocess.run(
                        [
                            "taskset",
                            "-c",
                            str(pin_cpu),
                            str(bins[arm, "station_balance"]),
                            str(files[key]),
                            str(STATION_PASSES),
                        ],
                        capture_output=True,
                        text=True,
                        timeout=300,
                    )
                    if done.returncode != 0:
                        raise InvalidRun(f"station_balance failed on {arm}/{corpus}: {done.stderr}")
                    rec = parse_station_line(done.stdout)
                    if rec["out"] != refs[key]["out"] or rec["blocks"] <= 0:
                        raise InvalidRun(f"station_balance byte/block mismatch on {arm}/{corpus}")
                    row = {"round": r, "corpus": corpus, "arm": arm, **rec}
                    log.write(json.dumps(row, allow_nan=False) + "\n")
                    rows.append(row)
    return rows


def summarize_stations(rows):
    by_key = {}
    for row in rows:
        key = (row["corpus"], row["round"], row["arm"])
        if key in by_key:
            raise InvalidRun(f"duplicate station_balance row: {key}")
        by_key[key] = row
    if len(by_key) != len(STATION_CORPORA) * STATION_ROUNDS * 2:
        raise InvalidRun("incomplete station_balance grid")
    summary = {}
    for corpus in STATION_CORPORA:
        arm = {a: [by_key[corpus, r, a] for r in range(STATION_ROUNDS)] for a in ("base", "head")}
        entry = {}
        for field in ("s_ms", "w_ms"):
            ratio, lo, hi = paired_ratio_ci(
                [x[field] for x in arm["head"]], [x[field] for x in arm["base"]]
            )
            entry[field] = {
                "base_med": statistics.median(x[field] for x in arm["base"]),
                "head_med": statistics.median(x[field] for x in arm["head"]),
                "ratio": ratio,
                "ci95": [lo, hi],
            }
        summary[corpus] = entry
    return summary


def _finite(*xs):
    return all(isinstance(x, (int, float)) and math.isfinite(x) and x > 0 for x in xs)


def score(rows, refs, head_exe=None, rounds=ROUNDS):
    """Score timed `ab run` rows against the frozen decision rule (registration section 3)."""
    pairs = {}
    for row in rows:
        if row.get("termination") not in (None, "success"):
            raise InvalidRun(f"nonzero child termination: {row.get('termination')}")
        if row.get("err", "") not in (None, ""):
            raise InvalidRun(f"decoder error recorded in timed sample: {row['err']}")
        arm = canonical_arm(row["arm"], head_exe=head_exe)
        cell = (row["corpus"], int(row["threads"]))
        rnd = int(row["round"])
        if cell not in CELLS or rnd not in range(rounds):
            raise InvalidRun(f"unexpected cell or round: {cell} r={rnd}")
        idx = (cell, rnd, arm)
        if idx in pairs:
            raise InvalidRun(f"duplicate sample: {idx}")
        ref = refs[cell[0]]
        if row.get("out") != ref["out"]:
            raise InvalidRun(f"output byte count mismatch on {cell}/{arm}/r{rnd}")
        if "hash" in ref and row.get("hash") != ref["hash"]:
            raise InvalidRun(f"output hash mismatch on {cell}/{arm}/r{rnd}")
        if not _finite(row.get("wall"), row.get("rss_kb")):
            raise InvalidRun(f"invalid wall or peak RSS on {cell}/{arm}/r{rnd}")
        pairs[idx] = row
    if len(pairs) != len(CELLS) * rounds * 2:
        raise InvalidRun(f"incomplete paired grid: {len(pairs)} != {len(CELLS) * rounds * 2}")

    findings, cells = [], {}
    for cell in CELLS:
        corpus, k = cell
        base = [pairs[cell, r, "base"] for r in range(rounds)]
        head = [pairs[cell, r, "head"] for r in range(rounds)]
        ratio, lo, hi = paired_ratio_ci([x["wall"] for x in head], [x["wall"] for x in base])
        rss, rss_lo, rss_hi = paired_ratio_ci(
            [x["rss_kb"] for x in head], [x["rss_kb"] for x in base]
        )
        if not _finite(ratio, lo, hi, rss, rss_lo, rss_hi):
            raise InvalidRun(f"undefined 95% interval at {cell}")
        rss_delta = statistics.median(h["rss_kb"] - b["rss_kb"] for h, b in zip(head, base))
        name = f"{corpus}/{k}"
        cells[name] = {
            "wall_ratio": ratio,
            "wall_ci95": [lo, hi],
            "base_wall_med": statistics.median(x["wall"] for x in base),
            "head_wall_med": statistics.median(x["wall"] for x in head),
            "rss_gm": rss,
            "rss_ci95": [rss_lo, rss_hi],
            "rss_median_delta_kib": rss_delta,
        }
        if cell == ("wikidata", 1) and hi > GAIN_HI:
            findings.append(f"{name}: no paid gain (upper 95% bound {hi:.4f} > {GAIN_HI})")
        if corpus in PRIMARY and lo > 1.0:
            findings.append(f"{name}: primary wall slower beyond noise (lower bound {lo:.4f})")
        if corpus not in PRIMARY and ratio > SECONDARY_WALL and lo > 1.0:
            findings.append(f"{name}: secondary wall {ratio:.4f} > {SECONDARY_WALL} beyond noise")
        if k == 1:
            if rss > RSS_K1_GM and rss_delta > RSS_K1_KIB:
                findings.append(f"{name}: k=1 peak RSS {rss:.4f}, +{rss_delta:.0f} KiB")
        elif corpus in PRIMARY:
            if rss > RSS_PRIMARY_GM:
                findings.append(f"{name}: peak RSS {rss:.4f} > {RSS_PRIMARY_GM}")
        elif rss > RSS_SECONDARY_GM:
            findings.append(f"{name}: peak RSS {rss:.4f} > {RSS_SECONDARY_GM}")
    return {"verdict": "STOP" if findings else "PASS", "findings": findings, "cells": cells}


def run(out):
    pin_cpu = machine(out)
    if output("git", "status", "--porcelain", "--untracked-files=no"):
        raise InvalidRun("recipe checkout has tracked changes")
    toolchain = output("rustc", "-Vv")
    if not toolchain.startswith("rustc 1.99.1 "):
        raise InvalidRun("registered toolchain requires Rust 1.99.1")
    save(
        out / "provenance.json",
        {
            "recipe": output("git", "rev-parse", "HEAD"),
            "base": BASE,
            "trees": PINNED_TREES,
            "pin_cpu": pin_cpu,
            "rustc": toolchain,
            "rounds": ROUNDS,
            "settle_s": SETTLE_S,
        },
    )
    with tempfile.TemporaryDirectory(dir=out.parent) as tmpdir:
        tmp = Path(tmpdir)
        try:
            measure_all(out, tmp, pin_cpu)
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", tmp / "base-src"], cwd=ROOT)
            subprocess.run(["git", "worktree", "prune"], cwd=ROOT)


def measure_all(out, tmp, pin_cpu):
    bins = build(out, tmp)
    if json.loads(output(bins["base", "ab"], "env")).get("cpu_quota", "absent") is not None:
        raise InvalidRun("workstation CPU quota must be unlimited")
    files, refs = inputs(out, tmp, bins)
    verify_parity(out, bins, files, refs)
    rows = run_ab_timed(out, bins, files, pin_cpu)
    stations = run_station_balance(out, bins, files, refs, pin_cpu)
    result = score(rows, refs, head_exe=bins["head", "ab"])
    result["station_balance"] = summarize_stations(stations)
    save(out / "summary.json", result)
    (out / "outcome.txt").write_text(result["verdict"] + "\n")


if __name__ == "__main__":
    dest = Path(sys.argv[1]).resolve()
    try:
        run(dest)
    except InvalidRun as err:
        (dest / "outcome.txt").write_text(f"VOID: {err}\n")
        raise SystemExit(1) from err
