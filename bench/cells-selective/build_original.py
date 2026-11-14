"""Build the matched original Cells arm: one exact file restored, everything else shared."""

import hashlib
import json
import os
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASE = "d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2"


def main():
    """Use an isolated source tree and preserve its sole source difference."""
    work = Path(tempfile.mkdtemp(prefix="mibz2-selective-original-"))
    archive = work / "source.tar"
    with archive.open("xb") as out:
        subprocess.run(["git", "archive", "HEAD"], cwd=ROOT, stdout=out, check=True)
    tree = work / "source"
    tree.mkdir()
    with tarfile.open(archive) as source:
        source.extractall(tree, filter="data")
    original = subprocess.check_output(["git", "show", BASE + ":src/cells.rs"], cwd=ROOT)
    assert (
        hashlib.sha256(original).hexdigest()
        == "42e377d8a980c94d95a4b162b0f7ba6466ff6b3844cbc8b10fce933fd25692b2"
    )
    (tree / "src/cells.rs").write_bytes(original)
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target/selective-original"))
    subprocess.run(
        [
            "cargo",
            "rustc",
            "--locked",
            "--release",
            "--example",
            "ab",
            "--",
            "--cfg",
            "mibz2_cells_bench",
        ],
        cwd=tree,
        env=env,
        check=True,
        timeout=1800,
    )
    binary = ROOT / "bin/original-cells-ab"
    binary.parent.mkdir(exist_ok=True)
    shutil.copy2(ROOT / "target/selective-original/release/examples/ab", binary)
    source = {
        "tree": str(tree),
        "common_head": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip(),
        "restored_ref": BASE,
        "sole_restored_path": "src/cells.rs",
        "original_cells_sha256": hashlib.sha256(original).hexdigest(),
        "candidate_cells_sha256": hashlib.sha256((ROOT / "src/cells.rs").read_bytes()).hexdigest(),
        "original_binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
    }
    (ROOT / "original-source.json").write_text(json.dumps(source, indent=2) + "\n")
    # Both builds run the same real-reader/oracle test, outside measurements.
    env["CELLS_PINNED_INPUTS"] = str(ROOT / "cells-corpora")
    env["RUSTFLAGS"] = "--cfg mibz2_cells_bench"
    with (ROOT / "original-parity.log").open("w") as log:
        subprocess.run(
            [
                "cargo",
                "test",
                "--release",
                "--example",
                "ab",
                "cells_pinned_readers_byte_parity",
                "--",
                "--ignored",
                "--nocapture",
            ],
            cwd=tree,
            env=env,
            stdout=log,
            stderr=log,
            check=True,
            timeout=300,
        )


if __name__ == "__main__":
    main()
