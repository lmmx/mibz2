"""Prepare separate untimed binaries; never edit the original live experiment."""

import difflib
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
LIVE = ROOT / "bench/cells-gap/live"
WORK = HERE / "target"
SOURCE = "d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2"
GENERATOR = "57f51bbc0ca62ac3ee1cc148a1010842eea6222a"


def sha(path):
    with Path(path).open("rb") as f:
        return hashlib.file_digest(f, "sha256").hexdigest()


def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])


def load_live():
    spec = importlib.util.spec_from_file_location("live_prepare", LIVE / "prepare.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def prepare():
    WORK.mkdir()  # no reuse of a source already used for evidence
    package = WORK / "reader"
    shutil.copytree(
        LIVE,
        package,
        ignore=shutil.ignore_patterns("target", ".source", ".generated", "__pycache__"),
    )
    live = load_live()
    live.HERE = package
    live.generate()
    source = package / ".source/src/cells.rs"
    before = source.read_text()
    after = live.insert(
        before,
        "    fn write_cells(&mut self, word: u32, len: u32, n: usize, out: &mut Vec<u8>) {",
        live.hook("self.screen_eligibility(word);"),
    )
    after += "\n" + (HERE / "eligibility.rs").read_text()
    source.write_text(after)
    patch = "".join(difflib.unified_diff(before.splitlines(True), after.splitlines(True)))
    assert not any(
        line.startswith("-") and not line.startswith("---") for line in patch.splitlines()
    )
    (package / ".generated/eligibility.patch").write_text(patch)
    main = package / "src/main.rs"
    text = main.read_text().replace("fn main() ->", "fn reader_main() ->", 1)
    text = text.replace(
        'if args.len() < 3 || !matches!(args[1].as_str(), "check" | "measure")',
        'if args.len() < 3 || args[1] != "check"',
    )
    text = text.replace(
        '#[cfg(not(feature = "cells-reader"))]',
        '#[cfg(not(any(feature = "cells-reader", feature = "oracle")))]',
    )
    text = live.insert(
        text,
        '    #[cfg(feature = "live-trace")]\n    drop(whole);',
        '    #[cfg(feature = "oracle")]\n    let result = drain(bzip2::read::MultiBzDecoder::new(input), &mut h, output);\n',
        False,
    )
    text += "\n" + (HERE / "memory.rs").read_text()
    main.write_text(text)
    cargo = package / "Cargo.toml"
    text = cargo.read_text().replace("[dev-dependencies]\n", "")
    text = text.replace("[features]\n", "[features]\noracle = []\n")
    cargo.write_text(text)
    # Exact generator in an isolated complete tracked tree: generated files remain
    # outside src/tests/examples so the text corpus cannot consume its own harness.
    tree = WORK / "generator"
    tree.mkdir()
    for path in git("ls-tree", "-r", "--name-only", GENERATOR).decode().splitlines():
        dest = tree / path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(git("show", f"{GENERATOR}:{path}"))
    paths = [
        p
        for folder in ("src", "tests", "examples")
        for p in sorted((tree / folder).iterdir())
        if p.is_file()
    ] + [tree / "README.md"]
    manifest = [
        {"path": str(p.relative_to(tree)), "bytes": p.stat().st_size, "sha256": sha(p)}
        for p in paths
    ]
    (WORK / "text-manifest.json").write_text(
        json.dumps({"tree": GENERATOR, "scale": 4, "files": manifest}, indent=2) + "\n"
    )
    # Keep full correspondence, including helper/source additions.
    files = {str(p.relative_to(package)): sha(p) for p in sorted(package.rglob("*")) if p.is_file()}
    (WORK / "source.json").write_text(
        json.dumps({"decoder": SOURCE, "files": files}, indent=2) + "\n"
    )


def build():
    rustc = subprocess.check_output(["rustc", "+1.99.1", "-Vv"], text=True)
    assert rustc.startswith("rustc 1.99.1 (58cf8f9c3 2026-10-13)")
    assert "host: x86_64-unknown-linux-gnu" in rustc
    assert not any(
        value
        for key, value in os.environ.items()
        if key.endswith("RUSTFLAGS")
        or key.startswith("CARGO_PROFILE_RELEASE_")
        or key in ("RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_TARGET")
    )
    binaries = WORK / "bin"
    binaries.mkdir()
    package = WORK / "reader"
    manifest = json.loads((WORK / "source.json").read_text())
    for path, expected in manifest["files"].items():
        assert sha(package / path) == expected, path
    for arm, features in (
        ("baseline", "live-trace"),
        ("cells", "live-trace,cells-reader"),
        ("oracle", "oracle"),
    ):
        with (WORK / f"{arm}.build.log").open("x") as log:
            subprocess.run(
                [
                    "cargo",
                    "+1.99.1",
                    "build",
                    "--release",
                    "--locked",
                    "--manifest-path",
                    str(package / "Cargo.toml"),
                    "--features",
                    features,
                ],
                stdout=log,
                stderr=subprocess.STDOUT,
                check=True,
                timeout=1800,
            )
        shutil.copy2(package / "target/release/cells-live", binaries / arm)
    subprocess.run(
        [
            "cc",
            "-std=c11",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            str(HERE / "launcher.c"),
            "-o",
            str(binaries / "launcher"),
        ],
        check=True,
    )
    (WORK / "build.json").write_text(
        json.dumps(
            {
                "rustc": rustc,
                "head": git("rev-parse", "HEAD").decode().strip(),
                "dirty": bool(git("status", "--porcelain")),
                "binaries": {p.name: sha(p) for p in binaries.iterdir()},
            },
            indent=2,
        )
        + "\n"
    )


if __name__ == "__main__":
    import sys

    {"prepare": prepare, "build": build}[sys.argv[1]]()
