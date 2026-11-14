"""Compile three selected-materialisation mutants; require meaningful test failures."""

import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main(destination):
    """Mutate an isolated copy; compilation failure and timeout are not kills."""
    destination.mkdir()
    tree = destination / "tree"
    tree.mkdir()
    for name in ["Cargo.toml", "Cargo.lock"]:
        shutil.copy2(ROOT / name, tree / name)
    for name in ["src", "tests", "examples", "benches"]:
        shutil.copytree(ROOT / name, tree / name)
    source = (tree / "src/cells.rs").read_text()
    mutants = [
        ("baseline", "", ""),
        (
            "missing-short",
            "copy16(base.add(top), base.add((aw >> 32) as usize));",
            "/* omit retained bytes */",
        ),
        ("missing-rule", "wr64(base, rp, aw);", "/* omit retained left descriptor */"),
        (
            "masked-mark",
            "let small = len <= SMALL as u64;",
            "let small = (len & !(MARK as u64)) <= SMALL as u64;",
        ),
    ]
    results = {"source_sha256": hashlib.sha256(source.encode()).hexdigest(), "mutants": []}
    env = dict(os.environ, CARGO_TARGET_DIR=str(destination / "target"), CARGO_BUILD_JOBS="4")
    for name, old, new in mutants:
        if old:
            assert source.count(old) == 1
        (tree / "src/cells.rs").write_text(source.replace(old, new, 1) if old else source)
        command = ["cargo", "test", "--locked", "--lib", "miri_cells_selective_boundaries_reuse"]
        with (destination / f"{name}.log").open("x") as log:
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
        text = (destination / f"{name}.log").read_text()
        assert run.returncode == (101 if old else 0), (name, run.returncode)
        if old:
            assert "assertion `left == right` failed" in text
        if name == "masked-mark":
            assert "selected representation" in text
        results["mutants"].append(
            {
                "name": name,
                "compiled": True,
                "exit": run.returncode,
                "failure_kind": "descriptor classification"
                if name == "masked-mark"
                else "selected state/payload"
                if old
                else None,
            }
        )
        (destination / "results.json").write_text(json.dumps(results, indent=2) + "\n")
        print(name, run.returncode, flush=True)


if __name__ == "__main__":
    main(Path(sys.argv[1]).resolve())
