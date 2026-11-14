"""Pin only the existing experimental CI arm's inputs; preserve default-arm data."""

import hashlib
import json
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PIN = "57f51bbc0ca62ac3ee1cc148a1010842eea6222a"


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def main():
    # rust-cache restores target/ from earlier heads. Never reuse that source
    # tree or overwrite its evidence; extract a fresh pinned tree outside it.
    work = Path(tempfile.mkdtemp(prefix="mibz2-cells-generator-"))
    archive = work / "source.tar"
    with archive.open("xb") as out:
        subprocess.run(["git", "archive", PIN], cwd=ROOT, stdout=out, check=True)
    tree = work / "source"
    tree.mkdir()
    with tarfile.open(archive) as src:
        src.extractall(tree, filter="data")
    inputs = ROOT / "cells-corpora"
    assert not inputs.exists(), "refuse stale experimental corpus directory"
    subprocess.run(
        [
            "cargo",
            "run",
            "--release",
            "--locked",
            "--manifest-path",
            str(tree / "Cargo.toml"),
            "--example",
            "ab",
            "--",
            "gen",
            str(inputs),
            "4",
        ],
        check=True,
        timeout=1800,
    )
    shutil.copyfile(ROOT / "corpora/wikidata.bz2", inputs / "wikidata.bz2")
    pins = json.loads((ROOT / "bench/cells-small-spine/inputs.json").read_text())
    for name, (size, _, expected) in pins.items():
        path = inputs / (name + ".bz2")
        assert (path.stat().st_size, digest(path)) == (size, expected), name
    paths = [
        p
        for directory in ("src", "tests", "examples")
        for p in sorted((tree / directory).iterdir())
        if p.is_file()
    ] + [tree / "README.md"]
    manifest = {
        "generator": PIN,
        "scale": 4,
        "inputs": pins,
        "text_files": [{"path": str(p.relative_to(tree)), "sha256": digest(p)} for p in paths],
    }
    (ROOT / "cells-inputs.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
