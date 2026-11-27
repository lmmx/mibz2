"""Run #1053's frozen endpoint registration on an i9-10900K."""

import bz2
import hashlib
import itertools
import json
import math
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import tempfile

from mtf_fold import SCAN_RESIDUAL_MAX, fastest_core, paired_ratio_ci, run_child, topology
import wikidata

CANDIDATE = "faa9df70f98d72ece27befebb966a9d9683b6dd9"
FEATURE = "a6b-endpoint-redesign"
TRACE_FEATURES = "mtf-fold scan-oracle ideal-parts trace phase-counters"
ROOT = Path(__file__).resolve().parents[1]
WALL_CELLS = (("wikidata", 1), ("wikidata", 8), ("wikidata", 10), ("text", 1), ("text", 8))
REPEATS = 15


def output(*cmd, cwd=ROOT):
    return subprocess.check_output(cmd, cwd=cwd, text=True).strip()


def save(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def digest(path, opener=open):
    with opener(path, "rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def checkout_candidate(recipe, source, sha):
    """Fetch the registered source into the private build checkout, even on a shallow runner."""
    remote = output("git", "remote", "get-url", "origin", cwd=recipe)
    subprocess.run(["git", "init", "--quiet", source], check=True)
    subprocess.run(
        ["git", "fetch", "--quiet", "--no-tags", "--depth=1", remote, sha],
        cwd=source,
        check=True,
    )
    if output("git", "rev-parse", "FETCH_HEAD^{commit}", cwd=source) != sha:
        raise ValueError("fetched source differs from the registered A6b commit")
    subprocess.run(["git", "checkout", "--quiet", "--detach", sha], cwd=source, check=True)


def prepare(out, tmp):
    if "i9-10900K" not in output("lscpu"):
        raise ValueError("registered machine is the i9-10900K")
    if output("git", "status", "--porcelain", "--untracked-files=no"):
        raise ValueError("recipe checkout has tracked changes")
    if not output("rustc", "-V").startswith("rustc 1.99.1 "):
        raise ValueError("registered toolchain requires Rust 1.99.1")
    (out / "lscpu.txt").write_text(output("lscpu") + "\n")
    cores_file = out / "lscpu-e.txt"
    cores_file.write_text(output("lscpu", "-e") + "\n")
    cores = topology(cores_file)
    if len(cores) != 10 or any(len(c) != 2 for c in cores):
        raise ValueError("expected ten physical cores with both hardware threads")
    if set(itertools.chain.from_iterable(cores)) != os.sched_getaffinity(0):
        raise ValueError("all workstation CPUs must be available")
    (out / "commits.txt").write_text(
        f"head {output('git', 'rev-parse', 'HEAD')}\ncandidate {CANDIDATE} +{FEATURE}\n"
        f"control {CANDIDATE} default features\n"
    )
    source = tmp / "src"
    checkout_candidate(ROOT, source, CANDIDATE)
    shutil.copyfile(ROOT / "examples/a6b_endpoint_run.rs", source / "examples/a6b_endpoint_run.rs")
    (out / "registration.md").write_bytes(
        (source / "docs/journal/2026-11-27-a6b-endpoint-build/registration.md").read_bytes()
    )
    binaries = {}
    for arm, feat in (("base", ""), ("candidate", FEATURE)):
        bins = {}
        for examples, features in (
            (
                ["a6b_endpoint_run", "ab", "windows"] if arm == "base" else ["a6b_endpoint_run"],
                feat,
            ),
            (["mtf_fold"], (TRACE_FEATURES + " " + feat).strip()),
        ):
            args = ["cargo", "build", "--release", "--locked", "--target-dir", tmp / "build"]
            if features:
                args += ["--features", features]
            for name in examples:
                args += ["--example", name]
            subprocess.run(args, cwd=source, check=True)
            for name in examples:
                dest = tmp / f"{arm}-{name}"
                shutil.copy2(tmp / "build/release/examples" / name, dest)
                bins[name] = dest
        binaries[arm] = bins
    save(
        out / "binaries.json",
        {a: {k: digest(p) for k, p in b.items()} for a, b in binaries.items()},
    )
    sample = ROOT / "corpora/wikidata-sample.bz2"
    spec = json.loads((ROOT / "bench/wikidata.json").read_text())
    if not sample.exists() or digest(sample) != spec["corpus_sha256"]:
        sample = tmp / "wikidata.bz2"
        windows = tmp / "windows"
        windows.mkdir()
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
        for window in spec["windows"]:
            name = window["name"] + ".bin"
            (windows / name).write_bytes(
                subprocess.check_output(
                    ["git", "show", f"FETCH_HEAD:{spec['windows_dir']}/{name}"], cwd=source
                )
            )
        wikidata.build(spec, windows, str(binaries["base"]["windows"]), sample)
    if digest(sample) != spec["corpus_sha256"]:
        raise ValueError("unexpected Wikidata input hash")
    x4 = tmp / "wikidata-x4.bz2"
    with x4.open("wb") as dest:
        for _ in range(4):
            with sample.open("rb") as src:
                shutil.copyfileobj(src, dest)
    gen = tmp / "corpora"
    subprocess.run([binaries["base"]["ab"], "gen", gen, "16"], check=True)
    files = {"wikidata": x4, "text": gen / "text.bz2"}
    hashes = {name: digest(path, bz2.open) for name, path in files.items()}
    (out / "inputs.txt").write_text(
        "".join(
            f"{name} compressed {digest(path)} decompressed {hashes[name]}\n"
            for name, path in files.items()
        )
    )
    calibration = {
        str(core[0]): json.loads(
            output(
                "taskset",
                "-c",
                str(core[0]),
                binaries["base"]["mtf_fold"],
                "--calibrate",
                "200000000",
                "5",
            )
        )
        for core in cores
    }
    save(out / "cores.json", calibration)
    cpu, siblings, _ = fastest_core(cores, out / "cores.json")
    mask = sorted(set(itertools.chain.from_iterable(cores)).difference(siblings))
    return binaries, files, hashes, cpu, mask


def sample_trace(binary, path, corpus, k, oracle, block, cpu, mask):
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


class ByteMismatch(ValueError):
    """The measured decoder returned a wrong digest or reported a CRC mismatch."""


def verify_output(row, digest):
    if row.get("kind") == "failed":
        msg = row.get("stderr", "")
        if "block CRC mismatch" in msg or "stream CRC mismatch" in msg:
            raise ByteMismatch(msg)
        raise ValueError(f"sample failed: {row}")
    if row["measure"] == "wall":
        actual = row["sha256"]
    else:
        cp = row["check_pass"]
        if not cp["identical"] or cp["bytes"] != row["bytes"]:
            raise ByteMismatch("trace output or byte count differs from its reference")
        actual = cp["sha256"]
    if actual != digest:
        raise ByteMismatch(f"expected {digest}, measured {actual}")


def validate_trace(row, oracle):
    if row.get("kind") != "decode" or row.get("scanner_pin_ok") is not True:
        raise ValueError("missing trace decode or scanner affinity")
    trace = row["trace_summary"]
    o = row["oracles"]
    decoded = trace["blocks_decoded"]
    if not (
        trace["workers_seen"] == row["k"]
        and decoded > 0
        and 0 <= trace["blocks_open_pass1"] <= decoded
        and decoded == trace["blocks_issued"]
    ):
        raise ValueError("worker trace is incomplete")
    if trace["worker_busy_ns_per_w"] <= 0 or row["maxrss_kib"] <= 0:
        raise ValueError("missing worker busy time or RSS")
    if row["section"] == "wikidata" and trace["blocks_open_pass1"] == 0:
        raise ValueError("Wikidata did not exercise Open")
    if row["section"] == "text" and trace["blocks_open_pass1"] >= decoded:
        raise ValueError("text did not exercise fallback")
    if oracle == "S":
        if (
            o["scan_locks"]
            or o["scanned"]
            or o["replayed"] != decoded
            or o["recorded_blocks"] != decoded
        ):
            raise ValueError("scan oracle did not replay every block")
    elif o["replayed"] or o["scanned"] != decoded:
        raise ValueError("real scanner and worker block counts differ")


def wall_sample(binary, path, k):
    proc = subprocess.run([binary, path, str(k)], timeout=180, text=True, capture_output=True)
    if proc.returncode:
        return {"kind": "failed", "stderr": proc.stderr.strip(), "returncode": proc.returncode}
    return json.loads(proc.stdout)


def validate_pair(pair, corpus, k, block, measure, digest):
    for arm in ("base", "candidate"):
        row = pair[arm]
        context = f"{arm} {corpus} k={k} round={block} {measure}"
        try:
            verify_output(row, digest)
        except ByteMismatch as exc:
            if arm == "candidate":
                raise ByteMismatch(f"{context}: {exc}") from exc
            raise ValueError(f"control failed its reference check: {context}: {exc}") from exc
        if row["k"] != k or row["maxrss_kib"] <= 0:
            raise ValueError(f"unexpected topology or missing RSS: {context}")
        if measure != "wall":
            validate_trace(row, "none" if measure == "preflight" else measure)


def run(out, bins, files, hashes, cpu, mask):
    rows = []
    with (out / "runs.jsonl").open("x", buffering=1) as log:

        def sample_pair(corpus, k, block, measure, order):
            pair = {}
            for arm in order:
                if measure == "wall":
                    row = wall_sample(bins[arm]["a6b_endpoint_run"], files[corpus], k)
                else:
                    oracle = "none" if measure == "preflight" else measure
                    row = sample_trace(
                        bins[arm]["mtf_fold"], files[corpus], corpus, k, oracle, block, cpu, mask
                    )
                raw = {**row, "build": arm, "measure": measure, "section": corpus, "block": block}
                log.write(json.dumps(raw, allow_nan=False) + "\n")
                rows.append(raw)
                pair[arm] = raw
            validate_pair(pair, corpus, k, block, measure, hashes[corpus])

        for corpus in files:
            sample_pair(corpus, 1, -1, "preflight", ("base", "candidate"))
        for block in range(REPEATS):
            order = ("base", "candidate") if block % 2 == 0 else ("candidate", "base")
            for corpus, k in WALL_CELLS:
                sample_pair(corpus, k, block, "wall", order)
            for oracle in ("none", "S"):
                sample_pair("wikidata", 8, block, oracle, order)
    return rows


def paired(rows, metric):
    pairs = {}
    for r in rows:
        block = r["block"]
        key = (block, r["build"])
        if key in pairs or block not in range(REPEATS):
            raise ValueError("duplicate or unexpected round")
        x = metric(r)
        if not math.isfinite(x) or x <= 0:
            raise ValueError("measurement must be positive and finite")
        pairs[key] = x
    base, cand = ([pairs[i, arm] for i in range(REPEATS)] for arm in ("base", "candidate"))
    ratio, lo, hi = paired_ratio_ci(cand, base)

    # At n=15, the fourth and twelfth order statistics give a 96.48% median interval
    # (the narrowest symmetric exact order-statistic interval with coverage >= 95%).
    def median_ci(xs):
        vals = sorted(xs)
        return [vals[3], vals[-4]]

    return {
        "ratio": ratio,
        "ci95": [lo, hi],
        "paired_median_gain": statistics.median(a - b for a, b in zip(base, cand)),
        "base_median_ci": median_ci(base),
        "candidate_median_ci": median_ci(cand),
    }


def summarise(rows):
    checks, cells = {}, {}
    for corpus, k in WALL_CELLS:
        group = [r for r in rows if (r["measure"], r["section"], r["k"]) == ("wall", corpus, k)]
        name = f"{corpus}/{k}"
        wall = paired(group, lambda x: x["wall_s"])
        cells[name] = {"wall_s": wall, "rss_kib": paired(group, lambda x: x["maxrss_kib"])}
        if (corpus, k) == ("wikidata", 1):
            checks[name] = (
                wall["ratio"] <= 0.97
                and wall["paired_median_gain"] >= 0.039
                and wall["candidate_median_ci"][1] < wall["base_median_ci"][0]
            )
        else:
            checks[name] = wall["ratio"] <= 1.01
    for oracle in ("none", "S"):
        group = [
            r for r in rows if r["measure"] == oracle and r["k"] == 8 and r["section"] == "wikidata"
        ]
        busy = paired(group, lambda x: x["trace_summary"]["worker_busy_ns_per_w"])
        cells["wikidata/8/" + oracle] = {
            "busy_ns_per_w": busy,
            "cpu_ns_per_w": paired(group, lambda x: x["trace_summary"]["worker_cpu_ns_per_w"]),
            "wall_s": paired(group, lambda x: x["wall_s"]),
        }
        if oracle == "none":
            checks["worker_busy"] = (
                busy["ratio"] <= 0.965 and busy["paired_median_gain"] >= 6_500_000
            )
    residuals = {}
    for arm in ("base", "candidate"):
        real, ideal = (
            [
                r["trace_summary"]["scanner_busy_ns"]
                for r in rows
                if r["measure"] == mode and r["build"] == arm
            ]
            for mode in ("none", "S")
        )
        residuals[arm] = statistics.median(ideal) / statistics.median(real)
        if residuals[arm] > SCAN_RESIDUAL_MAX:
            raise ValueError(f"{arm}: scan oracle residual exceeds {SCAN_RESIDUAL_MAX}")
    return {
        "checks": checks,
        "cells": cells,
        "scan_busy_residual": residuals,
        "hardware_pass": all(checks.values()),
    }


def main():
    out = Path(sys.argv[1]).resolve()
    with tempfile.TemporaryDirectory(prefix="build-", dir=out.parent) as tmp:
        args = prepare(out, Path(tmp))
        try:
            rows = run(out, *args)
        except ByteMismatch as exc:
            (out / "outcome.txt").write_text(
                f"STOP/KILL: frozen exactness criterion failed: {exc}\n"
            )
            return
        result = summarise(rows)
        save(out / "summary.json", result)
        verdict = "HARDWARE PASS" if result["hardware_pass"] else "STOP/KILL"
        (out / "outcome.txt").write_text(
            verdict + " under #1053; see summary.json for each hardware criterion.\n"
        )


if __name__ == "__main__":
    main()
