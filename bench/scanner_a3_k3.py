"""Registered i9 adoption run for the three-symbol A3 scanner decode against its master parent.

Three-symbol root entries, byte staging and a refill under the last lookup (claim
scanner-a3-k3-pipe-u8) are compared with the master commit the change was made on. Its
frozen gate is the serial scan S' (A2 + A3); the registration is
docs/journal/2026-11-28-scanner-a3-k3/registration.md. The decode, hashing and placement
checks are #1034's (bench/scanner_coalesced.py), and the grid is #1117's
(bench/scanner_a2_fast.py), reused with the claim's own bars.
"""

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

from mtf_fold import paired_ratio_ci
from scanner_coalesced import (
    PHASES_BLOB,
    InvalidRun,
    inputs,
    machine,
    measure,
    output,
    save,
)


ROOT = Path(__file__).resolve().parents[1]
# The change's single commit, and the master commit it was made on (its only parent).
HEAD = "ad99d407e720d413f5795de263ec96656e08bcb2"
BASE = "07ab2e7e4cb0b037933ba5e1a1177b8fa353dd37"
REGISTRATION = "docs/journal/2026-11-28-scanner-a3-k3/registration.md"
GUARDS = ("text", "records", "random", "runs", "pattern", "onebyte")
CORPORA = ("wikidata", *GUARDS)
WIKI_COUNTS = (1, 8, 9, 10)
ROUNDS = 10
SEED = 1153
CELLS = (
    tuple(("phases", name, 1) for name in CORPORA)
    + tuple(("real", "wikidata", k) for k in WIKI_COUNTS)
    + tuple(("real", name, 8) for name in GUARDS)
)
S_BOUND = 0.960  # S' must fall by at least 4.0%: upper 95% bound of head/base.
GUARD_BOUND = 1.020  # No guard serial scan more than 2.0% slower, with its lower bound above 1.
RSS_BOUND = 1.02  # Median peak RSS ratio, every real cell.
WALL_BOUND = 1.020  # No real Wikidata wall loss beyond 2.0% with its lower bound above 1.


def materialize_sources(tmp, checkout, head, base):
    """Fetch the change and its master parent into a private repository."""
    origin = output("git", "remote", "get-url", "origin", cwd=checkout)
    source = tmp / "head"
    subprocess.run(["git", "init", "-q", source], check=True)
    subprocess.run(
        ["git", "fetch", "-q", "--depth=2", "--no-tags", origin, head], cwd=source, check=True
    )
    if output("git", "rev-parse", "FETCH_HEAD", cwd=source) != head:
        raise InvalidRun("the fetched head differs from the registered head")
    if output("git", "rev-list", "--parents", "-n", "1", "FETCH_HEAD", cwd=source).split()[1:] != [
        base
    ]:
        raise InvalidRun("the control is not the only parent of the registered change")
    subprocess.run(["git", "checkout", "-q", "--detach", head], cwd=source, check=True)
    subprocess.run(
        ["git", "worktree", "add", "-q", "--detach", tmp / "base", base], cwd=source, check=True
    )


def build(out, tmp):
    materialize_sources(tmp, ROOT, HEAD, BASE)
    bins = {}
    for arm in ("base", "head"):
        source = tmp / arm
        if output("git", "rev-parse", "HEAD:examples/phases.rs", cwd=source) != PHASES_BLOB:
            raise InvalidRun("the registered phases producer has changed")
        helper = source / "examples/scanner_coalesced_run.rs"
        shutil.copyfile(ROOT / "examples" / helper.name, helper)
        phases = source / "examples/phases.rs"
        entry = "    let k = Kernels::detect();"
        original = phases.read_text()
        if original.count(entry) != 1:
            raise InvalidRun("cannot locate the pinned phases sample entry")
        phases.write_text(
            original.replace(
                entry,
                entry
                + """
    if std::env::var_os("MIBZ2_SCANNER_PHASE_SAMPLE").is_some() {
        let (t, bytes, blocks, _) = pass(&data, k, None, false);
        println!("{{\\"scan_s\\":{},\\"out\\":{},\\"scanned\\":{}}}", t.scan, bytes, blocks);
        return;
    }""",
            )
        )
        names = ["scanner_coalesced_run", "phases"] + (["ab", "windows"] if arm == "base" else [])
        cmd = ["cargo", "build", "--release", "--locked", "--features", "scan-oracle"]
        for name in names:
            cmd.extend(["--example", name])
        subprocess.run(
            cmd,
            cwd=source,
            check=True,
            env={**os.environ, "CARGO_TARGET_DIR": str(source / "target")},
        )
        for name in names:
            key = "real" if name == "scanner_coalesced_run" else name
            dest = tmp / f"{arm}-{key}"
            shutil.copy2(source / "target/release/examples" / name, dest)
            bins[arm, key] = dest
    save(
        out / "binaries.json",
        {str(path.name): hashlib.sha256(path.read_bytes()).hexdigest() for path in bins.values()},
    )
    for arm in ("base", "head"):
        symbols = [
            line.split()[-1]
            for line in output("nm", bins[arm, "real"]).splitlines()
            if "decode_runs_bmi2" in line
        ]
        if len(symbols) != 1:
            raise InvalidRun(f"cannot identify the BMI2 block decoder in {arm}")
        assembly = output("objdump", "-d", "--disassemble=" + symbols[0], bins[arm, "real"])
        (out / f"bmi2-{arm}.asm").write_text(assembly + "\n")
        if "shlx" not in assembly or "shrx" not in assembly or "vzeroupper" in assembly:
            raise InvalidRun(f"BMI2 code generation not observed in {arm}")
    shutil.copyfile(ROOT / REGISTRATION, out / "registration.md")
    return bins


def plan():
    rng = random.Random(SEED)
    first = {}
    for cell in CELLS:
        first[cell] = ["base"] * (ROUNDS // 2) + ["head"] * (ROUNDS // 2)
        rng.shuffle(first[cell])
    for r in range(ROUNDS):
        cells = list(CELLS)
        rng.shuffle(cells)
        for cell in cells:
            a = first[cell][r]
            for arm in (a, "head" if a == "base" else "base"):
                yield r, cell, arm


def score(rows):
    findings, cells, pairs = [], {}, {}
    for row in rows:
        key = (row["kind"], row["corpus"], row["k"])
        idx = (key, row["round"], row["build"])
        if (
            key not in CELLS
            or row["round"] not in range(ROUNDS)
            or row["build"] not in ("base", "head")
            or idx in pairs
        ):
            raise InvalidRun("unexpected or duplicate sample")
        pairs[idx] = row
    if len(pairs) != len(CELLS) * ROUNDS * 2:
        raise InvalidRun("incomplete paired grid")
    for key in CELLS:
        kind, name, k = key
        field = "scan_s" if kind == "phases" else "wall_s"
        arms = [[pairs[key, i, arm] for i in range(ROUNDS)] for arm in ("base", "head")]
        base, head = [[r[field] for r in arm] for arm in arms]
        ratio, lo, hi = paired_ratio_ci(head, base)
        if not all(math.isfinite(x) and x > 0 for x in (ratio, lo, hi)):
            raise InvalidRun(f"undefined confidence interval at {key}")
        detail = {
            "ratio": ratio,
            "ci95": [lo, hi],
            "base_range": [min(base), max(base)],
            "head_range": [min(head), max(head)],
        }
        if kind == "real":
            rss = [[r["peak_kib"] for r in arm] for arm in arms]
            if not all(math.isfinite(x) and x > 0 for group in rss for x in group):
                raise InvalidRun(f"invalid RSS at {key}")
            detail["rss_ratio"] = statistics.median(rss[1]) / statistics.median(rss[0])
            if detail["rss_ratio"] > RSS_BOUND:
                findings.append(f"{name}/{k} peak RSS exceeds {RSS_BOUND}")
            if name == "wikidata" and ratio > WALL_BOUND and lo > 1:
                findings.append(f"wikidata/{k} wall more than 2.0% slower beyond noise")
        elif name == "wikidata":
            if hi > S_BOUND or max(head) >= min(base):
                findings.append("serial scan misses the 0.960 upper bound or disjoint ranges")
        elif ratio > GUARD_BOUND and lo > 1:
            findings.append(f"{name} serial scan more than 2.0% slower beyond noise")
        cells["/".join(map(str, key))] = detail
    return {"verdict": "STOP" if findings else "PASS", "findings": findings, "cells": cells}


def run(out):
    scanner, mask = machine(out)
    if output("git", "status", "--porcelain", "--untracked-files=no"):
        raise InvalidRun("recipe checkout has tracked changes")
    toolchain = output("rustc", "-Vv")
    if not toolchain.startswith("rustc 1.99.1 "):
        raise InvalidRun("wrong Rust toolchain")
    save(
        out / "provenance.json",
        {
            "recipe": output("git", "rev-parse", "HEAD"),
            "head": HEAD,
            "base": BASE,
            "scanner_cpu": scanner,
            "mask": mask,
            "rustc": toolchain,
            "seed": SEED,
        },
    )
    with tempfile.TemporaryDirectory(dir=out.parent) as tmpdir:
        tmp = Path(tmpdir)
        bins = build(out, tmp)
        if json.loads(output(bins["base", "ab"], "env")).get("cpu_quota", "absent") is not None:
            raise InvalidRun("workstation CPU quota must be unlimited")
        files, refs = inputs(out, tmp, bins)
        with (out / "parity.jsonl").open("x", buffering=1) as checks:
            for name in (*CORPORA, "wikidata-single"):
                for cell in (("real", name, 1), ("real", name, 10)):
                    for arm in ("base", "head"):
                        sample = measure(
                            bins[arm, "real"], cell, files[name], scanner, mask, refs[name], True
                        )
                        refs[name].setdefault("scanned", sample["scanned"])
                        checks.write(json.dumps([arm, cell, sample]) + "\n")
        rows = []
        with (out / "runs.jsonl").open("x", buffering=1) as log:
            for r, cell, arm in plan():
                kind, name, k = cell
                exe = bins[arm, "phases" if kind == "phases" else "real"]
                sample = measure(exe, cell, files[name], scanner, mask, refs[name])
                row = {**sample, "kind": kind, "corpus": name, "k": k, "round": r, "build": arm}
                log.write(json.dumps(row, allow_nan=False) + "\n")
                rows.append(row)
        result = score(rows)
        save(out / "summary.json", result)
        (out / "outcome.txt").write_text(result["verdict"] + "\n")


if __name__ == "__main__":
    dest = Path(sys.argv[1]).resolve()
    try:
        run(dest)
    except InvalidRun as err:
        (dest / "outcome.txt").write_text(f"VOID: {err}\n")
        raise SystemExit(1) from err
