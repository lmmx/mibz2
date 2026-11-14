"""Byte-correctness kills for the new omission guard, in an isolated source copy."""

import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main(destination):
    destination.mkdir()
    tree = destination / "tree"
    tree.mkdir()
    for name in ("Cargo.toml", "Cargo.lock"):
        shutil.copy2(ROOT / name, tree / name)
    for name in ("src", "tests", "examples", "benches"):
        shutil.copytree(ROOT / name, tree / name)
    source = (tree / "src/cells.rs").read_text()
    mutants = [
        ("baseline", None, "cells::tests"),
        (
            "missing-small-copy",
            "copy_small(dst.add(d), base.add((plain >> 32) as usize), lx);",
            "miri_cells_small_spine_boundary_and_reuse",
        ),
        ("missing-long-recopy", "copy_long(dst.add(d), dst.add(f), lx);", "miri_cells_long_writer"),
    ]
    results = {"source_sha256": hashlib.sha256(source.encode()).hexdigest(), "mutants": []}
    env = dict(os.environ, CARGO_TARGET_DIR=str(destination / "target"), CARGO_BUILD_JOBS="4")
    for name, omitted, test in mutants:
        if omitted:
            assert source.count(omitted) == 1
        (tree / "src/cells.rs").write_text(
            source.replace(omitted, "/* omit copy; keep counters */") if omitted else source
        )
        with (destination / (name + ".log")).open("x") as log:
            command = ["cargo", "test", "--locked", "--lib", test]
            subprocess.run(
                command + ["--no-run"],
                cwd=tree,
                env=env,
                stdout=log,
                stderr=log,
                check=True,
                timeout=300,
            )
            run = subprocess.run(command, cwd=tree, env=env, stdout=log, stderr=log, timeout=60)
        text = (destination / (name + ".log")).read_text()
        if omitted:
            assert run.returncode == 101 and "assertion `left == right` failed" in text, (
                "must fail output equality"
            )
        else:
            assert run.returncode == 0
        results["mutants"].append(
            {"name": name, "exit": run.returncode, "byte_equality_kill": bool(omitted)}
        )
        (destination / "results.json").write_text(json.dumps(results, indent=2) + "\n")
        print(name, run.returncode, flush=True)


if __name__ == "__main__":
    main(Path(sys.argv[1]).resolve())
