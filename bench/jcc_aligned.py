"""Registered i9-10900K adoption study: baseline x86-64 + JCC-32B alignment vs default."""

import bz2
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import statistics
import subprocess
import sys
import tempfile

from mtf_fold import paired_ratio_ci, topology
import wikidata


ROOT = Path(__file__).resolve().parents[1]
SOURCE = "59cac2c52d7168e09a4b4d6edeb658c16ef9baf7"
ALIGN_FLAG = "-C llvm-args=-x86-branches-within-32B-boundaries"
X4_SHA256 = "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"
WIKIDATA_COUNTS = (1, 4, 6, 8, 10)
GUARD_CORPORA = ("text", "records", "random")
GUARD_COUNTS = (1, 8, 10)
ROUNDS = 12
STATION_ROUNDS = 5
STATION_PASSES = 5
PRESPIN_MS = 100
SETTLE_S = "0.2"
MAX_BUSY = "1.5"

CELLS = tuple(("wikidata", k) for k in WIKIDATA_COUNTS) + tuple(
    (name, k) for name in GUARD_CORPORA for k in GUARD_COUNTS
)
STATION_CORPORA = ("wikidata", *GUARD_CORPORA)

JCC_RE = re.compile(
    r"^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s)+)\s*"
    r"(j(?:a|ae|b|be|c|cxz|e|ecxz|g|ge|l|le|mp|mpq|na|nae|nb|nbe|nc|ne|ng|nge|nl|nle|no|np|ns|nz|o|p|pe|po|rcxz|s|z))\b"
    r"(?!\s*\*)"
)
FUNC_RE = re.compile(r"^[0-9a-f]+\s+<([^>]+)>:")
TEXT_SEC_RE = re.compile(r"^\s*\d+\s+\.text\s+([0-9a-f]+)\b")


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


def inspect_binary(path):
    """Return `.text` size in bytes and unfused 32B-boundary JCC/JMP crossings in `mibz2::*`."""
    headers = output("objdump", "-h", str(path))
    text_bytes = None
    for line in headers.splitlines():
        m = TEXT_SEC_RE.match(line)
        if m:
            text_bytes = int(m.group(1), 16)
            break
    if text_bytes is None or text_bytes <= 0:
        raise InvalidRun(f"could not read .text size of {path}")
    disasm = output("objdump", "-d", "-C", str(path))
    in_mibz2 = False
    jcc_total = 0
    unfused_crossings = 0
    for line in disasm.splitlines():
        fm = FUNC_RE.match(line)
        if fm:
            in_mibz2 = "mibz2::" in fm.group(1)
            continue
        if not in_mibz2:
            continue
        jm = JCC_RE.match(line)
        if not jm:
            continue
        jcc_total += 1
        addr = int(jm.group(1), 16)
        nbytes = len(jm.group(2).split())
        end = addr + nbytes
        if (addr >> 5) != ((end - 1) >> 5) or (end & 31) == 0:
            unfused_crossings += 1
    return {
        "sha256": sha256_file(path),
        "text_bytes": text_bytes,
        "mibz2_jcc_total": jcc_total,
        "mibz2_unfused_32b_crossings": unfused_crossings,
    }


def validate_codegen_audit(audit):
    for role in ("ab", "station_balance"):
        d = audit["default"][role]
        a = audit["aligned"][role]
        if a["sha256"] == d["sha256"]:
            raise InvalidRun(f"aligned and default {role} binaries are identical")
        if a["text_bytes"] <= d["text_bytes"]:
            raise InvalidRun(f"aligned {role} .text did not grow over default")
        if d["mibz2_unfused_32b_crossings"] <= 0:
            raise InvalidRun(f"default {role} reported zero mibz2 JCC 32B crossings")
        if a["mibz2_unfused_32b_crossings"] != 0:
            raise InvalidRun(
                f"aligned {role} retained {a['mibz2_unfused_32b_crossings']} unfused 32B crossings"
            )


def build(out, tmp):
    if subprocess.run(["git", "cat-file", "-e", SOURCE + "^{commit}"], cwd=ROOT).returncode:
        subprocess.run(["git", "fetch", "--no-tags", "origin", SOURCE], cwd=ROOT, check=True)
    bins = {}
    audit = {"default": {}, "aligned": {}}
    base_env = {k: v for k, v in os.environ.items() if k != "RUSTFLAGS"}
    for arm, rustflags in (("default", None), ("aligned", ALIGN_FLAG)):
        source = tmp / arm
        subprocess.run(["git", "clone", "-q", "--local", "--no-checkout", ROOT, source], check=True)
        subprocess.run(["git", "checkout", "-q", "--detach", SOURCE], cwd=source, check=True)
        names = ["ab", "station_balance"]
        if arm == "default":
            names.append("windows")
        cmd = ["cargo", "build", "--release", "--locked"]
        for name in names:
            cmd.extend(["--example", name])
        env = {**base_env, "CARGO_TARGET_DIR": str(source / "target")}
        if rustflags is not None:
            env["RUSTFLAGS"] = rustflags
        subprocess.run(cmd, cwd=source, check=True, env=env)
        for name in names:
            dest = tmp / f"{arm}-{name}"
            shutil.copy2(source / "target/release/examples" / name, dest)
            bins[arm, name] = dest
        for role in ("ab", "station_balance"):
            audit[arm][role] = inspect_binary(bins[arm, role])
    validate_codegen_audit(audit)
    save(out / "codegen_audit.json", audit)
    save(
        out / "binaries.json",
        {str(path.name): sha256_file(path) for path in bins.values()},
    )
    shutil.copyfile(
        ROOT / "docs/journal/2026-11-27-jcc-aligned-release/registration.md",
        out / "registration.md",
    )
    return bins, audit


def inputs(out, tmp, bins):
    spec = json.loads((ROOT / "bench/wikidata.json").read_text())
    wiki = ROOT / "corpora/wikidata.bz2"
    if not wiki.exists() or sha256_file(wiki) != spec["corpus_sha256"]:
        win = tmp / "windows"
        win.mkdir()
        source = tmp / "default"
        subprocess.run(
            [
                "git",
                "fetch",
                "--no-tags",
                output("git", "remote", "get-url", "origin"),
                "bench-data",
            ],
            cwd=source,
            check=True,
        )
        for w in spec["windows"]:
            name = w["name"] + ".bin"
            (win / name).write_bytes(
                subprocess.check_output(
                    ["git", "show", f"FETCH_HEAD:{spec['windows_dir']}/{name}"], cwd=source
                )
            )
        wiki = tmp / "wikidata-single.bz2"
        wikidata.build(spec, win, str(bins["default", "windows"]), wiki)
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
    subprocess.run([bins["default", "ab"], "gen", str(gen), "4"], check=True)
    files = {"wikidata": x4, **{n: gen / f"{n}.bz2" for n in GUARD_CORPORA}}
    files["wikidata-single"] = wiki
    refs = {n: reference(p) for n, p in files.items()}
    save(
        out / "inputs.json",
        {n: {**refs[n], "compressed_sha256": sha256_file(p)} for n, p in files.items()},
    )
    return files, refs


def canonical_arm(label, aligned_exe=None):
    """Map an `ab run` arm label (`mibz2` or `mibz2@/path/...`) to `default` or `aligned`."""
    if label in ("default", "mibz2"):
        return "default"
    if label == "aligned":
        return "aligned"
    if label.startswith("mibz2@"):
        path = label.split("@", 1)[1]
        if aligned_exe is not None and Path(path).resolve() != Path(aligned_exe).resolve():
            raise InvalidRun(f"unexpected aligned executable path in arm label: {label}")
        return "aligned"
    raise InvalidRun(f"unexpected arm label: {label}")


def verify_parity(out, bins, files, refs):
    """Run untimed `AB_SHA256=1` `ab child` on every cell for both arms against libbzip2."""
    env = {**os.environ, "AB_SHA256": "1"}
    with (out / "parity.jsonl").open("x", buffering=1) as log:
        for corpus, k in CELLS:
            ref = refs[corpus]
            for arm in ("default", "aligned"):
                done = subprocess.run(
                    [str(bins[arm, "ab"]), "child", "mibz2", str(k), str(files[corpus])],
                    capture_output=True,
                    text=True,
                    timeout=180,
                    env=env,
                )
                if done.returncode != 0:
                    raise InvalidRun(f"parity child failed on {arm}/{corpus}/{k}: {done.stderr}")
                row = json.loads(done.stdout.strip())
                err = row.get("err", "")
                if err not in (None, ""):
                    raise InvalidRun(f"parity decoder error on {arm}/{corpus}/{k}: {err}")
                if row.get("out") != ref["out"] or row.get("sha256") != ref["sha256"]:
                    raise InvalidRun(f"parity byte/SHA-256 mismatch on {arm}/{corpus}/{k}")
                h = row.get("hash")
                if not h:
                    raise InvalidRun(f"missing hash on parity row {arm}/{corpus}/{k}")
                if "hash" in ref and ref["hash"] != h:
                    raise InvalidRun(f"hash mismatch between arms on {corpus}/{k}")
                ref["hash"] = h
                log.write(json.dumps({"arm": arm, "corpus": corpus, "k": k, **row}) + "\n")


def run_ab_timed(out, bins, files, pin_cpu):
    """Run paired `ab run` for wikidata (k=1,4,6,8,10) and guard corpora (k=1,8,10)."""
    aligned_exe = str(bins["aligned", "ab"].resolve())
    arms = f"mibz2,mibz2@{aligned_exe}"
    rows = []
    groups = (
        ("wikidata.jsonl", WIKIDATA_COUNTS, [str(files["wikidata"])]),
        ("guards.jsonl", GUARD_COUNTS, [str(files[n]) for n in GUARD_CORPORA]),
    )
    for filename, counts, paths in groups:
        dest = out / filename
        cmd = [
            str(bins["default", "ab"]),
            "run",
            str(dest),
            "--arms",
            arms,
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
    return {
        "file": parts[0],
        "blocks": int(parts[1]),
        "full_blocks": int(parts[2]),
        "symbols": int(parts[3]),
        "out": int(parts[4]),
        "in_bytes": int(parts[5]),
        "s_ms": float(parts[6]),
        "w_ms": float(parts[7]),
        "k_star": float(parts[8]),
        "s_ns_sym": float(parts[9]),
        "w_ns_byte": float(parts[10]),
    }


def run_station_balance(out, bins, files, refs, pin_cpu):
    """Run 1T `examples/station_balance.rs` diagnostic paired rounds pinned to `pin_cpu`."""
    rows = []
    with (out / "station_balance.jsonl").open("x", buffering=1) as log:
        for r in range(STATION_ROUNDS):
            order = ("default", "aligned") if r % 2 == 0 else ("aligned", "default")
            for corpus in STATION_CORPORA:
                key = "wikidata-single" if corpus == "wikidata" else corpus
                path = files[key]
                ref_out = refs[key]["out"]
                for arm in order:
                    done = subprocess.run(
                        [
                            "taskset",
                            "-c",
                            str(pin_cpu),
                            str(bins[arm, "station_balance"]),
                            str(path),
                            str(STATION_PASSES),
                        ],
                        capture_output=True,
                        text=True,
                        timeout=180,
                    )
                    if done.returncode != 0:
                        raise InvalidRun(
                            f"station_balance failed on {arm}/{corpus}/r{r}: {done.stderr}"
                        )
                    rec = parse_station_line(done.stdout)
                    if rec["out"] != ref_out or rec["blocks"] <= 0:
                        raise InvalidRun(f"station_balance byte/block mismatch on {arm}/{corpus}")
                    if not all(
                        math.isfinite(rec[f]) and rec[f] > 0 for f in ("s_ms", "w_ms", "k_star")
                    ):
                        raise InvalidRun(f"invalid station_balance timings on {arm}/{corpus}")
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
    expected = len(STATION_CORPORA) * STATION_ROUNDS * 2
    if len(by_key) != expected:
        raise InvalidRun(f"incomplete station_balance grid: {len(by_key)} != {expected}")
    summary = {}
    for corpus in STATION_CORPORA:
        d_rows = [by_key[corpus, r, "default"] for r in range(STATION_ROUNDS)]
        a_rows = [by_key[corpus, r, "aligned"] for r in range(STATION_ROUNDS)]
        s_ratio, s_lo, s_hi = paired_ratio_ci(
            [x["s_ms"] for x in a_rows], [x["s_ms"] for x in d_rows]
        )
        w_ratio, w_lo, w_hi = paired_ratio_ci(
            [x["w_ms"] for x in a_rows], [x["w_ms"] for x in d_rows]
        )
        r_al, r_lo, r_hi = paired_ratio_ci(
            [x["k_star"] for x in a_rows], [x["k_star"] for x in d_rows]
        )
        if not all(
            math.isfinite(x) and x > 0
            for x in (s_ratio, s_lo, s_hi, w_ratio, w_lo, w_hi, r_al, r_lo, r_hi)
        ):
            raise InvalidRun(f"undefined station_balance CI on {corpus}")
        summary[corpus] = {
            "s_ms_default_med": statistics.median(x["s_ms"] for x in d_rows),
            "s_ms_aligned_med": statistics.median(x["s_ms"] for x in a_rows),
            "s_ratio": s_ratio,
            "s_ci95": [s_lo, s_hi],
            "w_ms_default_med": statistics.median(x["w_ms"] for x in d_rows),
            "w_ms_aligned_med": statistics.median(x["w_ms"] for x in a_rows),
            "w_ratio": w_ratio,
            "w_ci95": [w_lo, w_hi],
            "k_star_default_med": statistics.median(x["k_star"] for x in d_rows),
            "k_star_aligned_med": statistics.median(x["k_star"] for x in a_rows),
            "r_aligned": r_al,
            "r_aligned_ci95": [r_lo, r_hi],
        }
    return summary


def score(rows, refs, aligned_exe=None, rounds=ROUNDS):
    """Score timed `ab run` rows against the frozen C2 adoption rule."""
    pairs = {}
    for row in rows:
        if row.get("termination") not in (None, "success"):
            raise InvalidRun(f"nonzero child termination: {row.get('termination')}")
        err = row.get("err", "")
        if err not in (None, ""):
            raise InvalidRun(f"decoder error recorded in timed sample: {err}")
        arm = canonical_arm(row["arm"], aligned_exe=aligned_exe)
        corpus = row["corpus"]
        k = int(row["threads"])
        rnd = int(row["round"])
        cell = (corpus, k)
        if cell not in CELLS or rnd not in range(rounds):
            raise InvalidRun(f"unexpected cell or round: {cell} r={rnd}")
        idx = (cell, rnd, arm)
        if idx in pairs:
            raise InvalidRun(f"duplicate sample: {idx}")
        ref = refs[corpus]
        if row.get("out") != ref["out"]:
            raise InvalidRun(f"output byte count mismatch on {cell}/{arm}/r{rnd}")
        if "hash" in ref and row.get("hash") != ref["hash"]:
            raise InvalidRun(f"output hash mismatch on {cell}/{arm}/r{rnd}")
        wall = row.get("wall")
        rss = row.get("rss_kb")
        if not (isinstance(wall, (int, float)) and math.isfinite(wall) and wall > 0):
            raise InvalidRun(f"invalid wall duration on {cell}/{arm}/r{rnd}")
        if not (isinstance(rss, (int, float)) and math.isfinite(rss) and rss > 0):
            raise InvalidRun(f"invalid peak RSS on {cell}/{arm}/r{rnd}")
        pairs[idx] = row

    expected = len(CELLS) * rounds * 2
    if len(pairs) != expected:
        raise InvalidRun(f"incomplete paired grid: {len(pairs)} != {expected}")

    findings = []
    cells = {}
    for cell in CELLS:
        corpus, k = cell
        d_rows = [pairs[cell, r, "default"] for r in range(rounds)]
        a_rows = [pairs[cell, r, "aligned"] for r in range(rounds)]
        d_wall = [x["wall"] for x in d_rows]
        a_wall = [x["wall"] for x in a_rows]
        ratio, lo, hi = paired_ratio_ci(a_wall, d_wall)
        if not all(math.isfinite(x) and x > 0 for x in (ratio, lo, hi)):
            raise InvalidRun(f"undefined 95% CI at {cell}")
        d_rss_med = statistics.median(x["rss_kb"] for x in d_rows)
        a_rss_med = statistics.median(x["rss_kb"] for x in a_rows)
        rss_ratio = a_rss_med / d_rss_med
        detail = {
            "ratio": ratio,
            "ci95": [lo, hi],
            "default_wall_med": statistics.median(d_wall),
            "aligned_wall_med": statistics.median(a_wall),
            "default_range": [min(d_wall), max(d_wall)],
            "aligned_range": [min(a_wall), max(a_wall)],
            "default_rss_kib_med": d_rss_med,
            "aligned_rss_kib_med": a_rss_med,
            "rss_ratio": rss_ratio,
        }
        if cell == ("wikidata", 1) and hi > 0.970:
            findings.append(
                f"wikidata/1 95% CI upper bound {hi:.4f} exceeds 0.970 (requires >= 3.0% gain)"
            )
        if cell == ("wikidata", 8) and hi > 0.985:
            findings.append(
                f"wikidata/8 95% CI upper bound {hi:.4f} exceeds 0.985 (requires >= 1.5% gain)"
            )
        if (cell == ("wikidata", 10) or corpus in GUARD_CORPORA) and ratio > 1.010:
            findings.append(
                f"{corpus}/{k} regressed by {(ratio - 1.0) * 100:.2f}% (> 1.0% limit)"
            )
        if rss_ratio > 1.020:
            findings.append(f"{corpus}/{k} peak RSS ratio {rss_ratio:.4f} exceeds 1.020")
        cells[f"{corpus}/{k}"] = detail

    return {
        "verdict": "STOP" if findings else "PASS",
        "amd_ci_gate": "pending_release_pr_ci_bench",
        "findings": findings,
        "cells": cells,
    }


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
            "source": SOURCE,
            "align_flag": ALIGN_FLAG,
            "pin_cpu": pin_cpu,
            "rustc": toolchain,
            "rounds": ROUNDS,
            "station_rounds": STATION_ROUNDS,
            "station_passes": STATION_PASSES,
        },
    )
    with tempfile.TemporaryDirectory(dir=out.parent) as tmpdir:
        tmp = Path(tmpdir)
        bins, audit = build(out, tmp)
        if json.loads(output(bins["default", "ab"], "env")).get("cpu_quota", "absent") is not None:
            raise InvalidRun("workstation CPU quota must be unlimited")
        files, refs = inputs(out, tmp, bins)
        verify_parity(out, bins, files, refs)
        ab_rows = run_ab_timed(out, bins, files, pin_cpu)
        station_rows = run_station_balance(out, bins, files, refs, pin_cpu)
        result = score(ab_rows, refs, aligned_exe=bins["aligned", "ab"])
        result["station_balance"] = summarize_stations(station_rows)
        result["codegen_audit"] = audit
        save(out / "summary.json", result)
        (out / "outcome.txt").write_text(result["verdict"] + "\n")


if __name__ == "__main__":
    dest = Path(sys.argv[1]).resolve()
    try:
        run(dest)
    except InvalidRun as err:
        (dest / "outcome.txt").write_text(f"VOID: {err}\n")
        raise SystemExit(1) from err
