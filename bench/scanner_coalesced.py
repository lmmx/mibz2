"""Registered i9 keep/revert study for the scanner squash and its first parent."""

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

from mtf_fold import paired_ratio_ci, topology
import wikidata


ROOT = Path(__file__).resolve().parents[1]
HEAD = "5ed20c1d51afc56c088ae244ba0bf40f37cb93b1"
BASE = "5c7fe5d2443279318bbf84f3e2d3ebc4a5850561"
PHASES_BLOB = "403cfbbdd160559826793d173b673dad5bcac682"
X4 = "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"
NAMES = ("wikidata", "text", "runs", "records", "random", "pattern", "onebyte")
COUNTS = (1, 2, 6, 8, 10)
ROUNDS = 10
SEED = 1034
CELLS = tuple(("real", name, k) for name in NAMES for k in COUNTS) + (
    ("phases", "wikidata", 1),
    ("C", "wikidata", 8),
    ("C", "wikidata", 10),
)


class InvalidRun(ValueError):
    """A registered prerequisite failed or the measurement cannot support a verdict."""


def output(*args, cwd=ROOT):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def save(path, obj):
    path.write_text(json.dumps(obj, indent=2, allow_nan=False) + "\n")


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
        raise InvalidRun("expected unrestricted i9 topology")
    # The scanner occupies one physical core; other threads cannot run on its sibling.
    scanner = cores[0][0]
    mask = ",".join(map(str, sorted(cpus.difference(cores[0]))))
    return scanner, mask


def materialize_sources(tmp, checkout, head, base):
    # The workstation checks out the recipe shallowly. Fetch its two historical
    # inputs into a private repository, without changing the runner's checkout.
    origin = output("git", "remote", "get-url", "origin", cwd=checkout)
    source = tmp / "head"
    subprocess.run(["git", "init", "-q", source], check=True)
    subprocess.run(
        ["git", "fetch", "-q", "--depth=2", "--no-tags", origin, head], cwd=source, check=True
    )
    if output("git", "rev-parse", "FETCH_HEAD", cwd=source) != head:
        raise InvalidRun("the fetched squash differs from the registered head")
    if output("git", "rev-parse", "FETCH_HEAD^", cwd=source) != base:
        raise InvalidRun("the control is not the scanner squash's first parent")
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
        # Run one registered phases::pass in decode order and retain full precision.
        # The normal phases driver also times three unrelated historical walk kernels.
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
        for kind, features in (("real", "scan-oracle"), ("C", "scan-oracle ideal-construction")):
            names = ["scanner_coalesced_run"]
            if kind == "real":
                names.append("phases")
                if arm == "base":
                    names.extend(["ab", "windows"])
            cmd = ["cargo", "build", "--release", "--locked", "--features", features]
            for name in names:
                cmd.extend(["--example", name])
            subprocess.run(
                cmd,
                cwd=source,
                check=True,
                env={**os.environ, "CARGO_TARGET_DIR": str(source / "target")},
            )
            for name in names:
                key = kind if name == "scanner_coalesced_run" else name
                dest = tmp / f"{arm}-{key}"
                shutil.copy2(source / "target/release/examples" / name, dest)
                bins[arm, key] = dest
    save(
        out / "binaries.json",
        {str(path.name): hashlib.sha256(path.read_bytes()).hexdigest() for path in bins.values()},
    )
    # The reviewed block-level dispatch is unconditional on BMI2 hosts. Keep its actual
    # machine code alongside the measured CPU feature, affinity, and scanned-block counts.
    symbols = [
        line.split()[-1]
        for line in output("nm", bins["head", "real"]).splitlines()
        if "decode_runs_bmi2" in line
    ]
    if len(symbols) != 1:
        raise InvalidRun("cannot identify the registered BMI2 block decoder")
    assembly = output("objdump", "-d", "--disassemble=" + symbols[0], bins["head", "real"])
    (out / "bmi2.asm").write_text(assembly + "\n")
    if "shlx" not in assembly or "shrx" not in assembly or "vzeroupper" in assembly:
        raise InvalidRun("registered BMI2 code generation not observed")
    shutil.copyfile(
        tmp / "head/docs/journal/2026-11-26-scanner-coalesced-u8/registration.md",
        out / "registration.md",
    )
    return bins


def inputs(out, tmp, bins):
    spec = json.loads((ROOT / "bench/wikidata.json").read_text())
    wiki = ROOT / "corpora/wikidata.bz2"
    if not wiki.exists() or hashlib.sha256(wiki.read_bytes()).hexdigest() != spec["corpus_sha256"]:
        win = tmp / "windows"
        win.mkdir()
        source = tmp / "base"
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
        wiki = tmp / "wikidata.bz2"
        wikidata.build(spec, win, str(bins["base", "windows"]), wiki)
    x4 = tmp / "wikidata-x4.bz2"
    with x4.open("wb") as dest:
        for _ in range(4):
            with wiki.open("rb") as source:
                shutil.copyfileobj(source, dest)
    if hashlib.sha256(x4.read_bytes()).hexdigest() != X4:
        raise InvalidRun("registered Wikidata x4 input hash mismatch")
    gen = tmp / "gen"
    subprocess.run([bins["base", "ab"], "gen", gen, "4"], check=True)
    files = {n: x4 if n == "wikidata" else gen / f"{n}.bz2" for n in NAMES}
    files["wikidata-single"] = wiki
    refs = {n: reference(p) for n, p in files.items()}
    save(
        out / "inputs.json",
        {
            n: {**refs[n], "compressed_sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
            for n, p in files.items()
        },
    )
    return files, refs


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


def measure(exe, case, file, scanner, mask, ref, check=False):
    kind, _, k = case
    if kind == "phases":
        done = subprocess.run(
            ["taskset", "-c", str(scanner), exe, file],
            env={**os.environ, "MIBZ2_SCANNER_PHASE_SAMPLE": "1"},
            timeout=120,
            text=True,
            capture_output=True,
        )
        if done.returncode:
            raise InvalidRun(f"pinned phases decode or CRC check failed: {done.stderr}")
        row = json.loads(done.stdout)
    else:
        cmd = [
            "taskset",
            "-c",
            mask,
            exe,
            file,
            str(k),
            str(scanner),
            kind,
            "check" if check else "time",
        ]
        if check:
            # Keep the independent SHA pass outside the timing and stream the decoded bytes.
            done = subprocess.run(
                ["bash", "-o", "pipefail", "-c", '"$@" | sha256sum', "scanner-check", *cmd],
                capture_output=True,
                text=True,
                timeout=120,
            )
            msg = done.stderr
            if done.returncode:
                raise InvalidRun(f"decoder check failed: {msg}")
            if done.stdout.split()[:1] != [ref["sha256"]]:
                raise InvalidRun("decoded output differs from independent libbz2")
        else:
            done = subprocess.run(
                cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, timeout=120, text=True
            )
            msg = done.stderr
            if done.returncode:
                raise InvalidRun(f"decoder sample failed: {msg}")
        row = json.loads(msg)
        if row["scanner_cpu"] != scanner or row["bmi2"] is not True or row["peak_kib"] <= 0:
            raise InvalidRun("scanner placement, BMI2 activation or RSS missing")
        if (kind == "C") != (row["replayed_c"] > 0):
            raise InvalidRun("construction path differs from registered cell")
    if row["out"] != ref["out"] or row["scanned"] <= 0:
        raise InvalidRun("decoded byte or scanned block count mismatch")
    if "scanned" in ref and row["scanned"] != ref["scanned"]:
        raise InvalidRun("scanner did not process the same blocks as the parity check")
    elapsed = row["scan_s" if kind == "phases" else "wall_s"]
    if not math.isfinite(elapsed) or elapsed <= 0:
        raise InvalidRun("invalid sample duration")
    return row


def score(rows):
    findings, cells = [], {}
    pairs = {}
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
        if kind != "phases":
            rss = [[r["peak_kib"] for r in arm] for arm in arms]
            if not all(math.isfinite(x) and x > 0 for group in rss for x in group):
                raise InvalidRun(f"invalid RSS at {key}")
            detail["rss_ratio"] = statistics.median(rss[1]) / statistics.median(rss[0])
        if kind == "phases" and (hi > 0.920 or max(head) >= min(base)):
            findings.append("serial scanner misses the 0.920 upper bound or non-overlapping ranges")
        if kind == "real":
            if (name, k) == ("wikidata", 10) and (hi > 0.970 or max(head) >= min(base)):
                findings.append(
                    "10-worker Wikidata misses the 0.970 upper bound or non-overlapping ranges"
                )
            if name in ("wikidata", "text", "runs") and lo > 1.005:
                findings.append(f"{name}/{k} lower ratio exceeds 1.005")
            if detail["rss_ratio"] > 1.05:
                findings.append(f"{name}/{k} peak RSS exceeds 1.05")
        cells["/".join(map(str, key))] = detail
    for k in COUNTS:
        best = min(cells[f"real/{n}/{k}"]["ratio"] for n in ("wikidata", "text"))
        for n in NAMES[2:]:
            loss = cells[f"real/{n}/{k}"]["ratio"] - 1
            if loss > 0.10 or (loss > 0.05 and 1 - best < 2 * loss):
                findings.append(f"{n}/{k} exceeds secondary-loss trade under §2 rule 3")
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
            for name in (*NAMES, "wikidata-single"):
                for cell in (("real", name, 1), ("real", name, 10)):
                    for arm in ("base", "head"):
                        sample = measure(
                            bins[arm, cell[0]], cell, files[name], scanner, mask, refs[name], True
                        )
                        refs[name].setdefault("scanned", sample["scanned"])
                        checks.write(json.dumps([arm, cell, sample]) + "\n")
            for cell in (("C", "wikidata", 8), ("C", "wikidata", 10)):
                for arm in ("base", "head"):
                    sample = measure(
                        bins[arm, "C"],
                        cell,
                        files["wikidata"],
                        scanner,
                        mask,
                        refs["wikidata"],
                        True,
                    )
                    checks.write(json.dumps([arm, cell, sample]) + "\n")
        rows = []
        with (out / "runs.jsonl").open("x", buffering=1) as log:
            for r, cell, arm in plan():
                kind, name, k = cell
                sample = measure(bins[arm, kind], cell, files[name], scanner, mask, refs[name])
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
