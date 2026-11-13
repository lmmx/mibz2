"""Noninteractive queue preparation; fixed inputs and one allowed CPU, no timing."""

import importlib.util
import json
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

from driver import DIGESTS, PIN, digest
from inputs import generate

ROOT = Path(__file__).resolve().parents[3]
WINDOW_PIN = "a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a"


def machine():
    allowed = sorted(os.sched_getaffinity(0))
    if not allowed:
        raise ValueError("no allowed logical CPU")
    cpu = allowed[0]  # Frozen selection rule, not a performance-based selection.
    topology = Path(f"/sys/devices/system/cpu/cpu{cpu}/topology")
    return dict(
        host=platform.node(),
        platform=platform.platform(),
        selection_rule="lowest allowed logical CPU at preparation start",
        selected_cpu=cpu,
        allowed_cpus=allowed,
        topology={
            field: (topology / field).read_text().strip()
            for field in ("physical_package_id", "core_id", "thread_siblings_list")
        },
        lscpu=subprocess.check_output(["lscpu", "--json"], text=True),
    )


def ensure_object(root, pin):
    command = ["git", "-C", str(root)]
    if subprocess.run(
        command + ["cat-file", "-e", pin + "^{commit}"], capture_output=True
    ).returncode:
        subprocess.run(command + ["fetch", "origin", pin], check=True)


def ensure_toolchain():
    probe = ["rustup", "run", "1.99.1", "rustc", "-Vv"]
    if subprocess.run(probe, capture_output=True).returncode:
        subprocess.run(
            ["rustup", "toolchain", "install", "1.99.1", "--profile", "minimal"], check=True
        )
    # No default-toolchain change; build.py checks the full compiler/host identity.
    subprocess.run(probe, check=True)


def stage_inputs(destination, root=ROOT):
    generate(destination)  # Existing generator checks both frozen synthetic hashes.
    canonical = root / "corpora/wikidata.bz2"
    if canonical.is_file() and digest(canonical) == DIGESTS["wikidata"]:
        shutil.copy2(canonical, destination / "wikidata.bz2")
        origin = "verified canonical corpora/wikidata.bz2"
    else:
        spec = json.loads((root / "bench/wikidata.json").read_text())
        if spec["corpus_sha256"] != DIGESTS["wikidata"]:
            raise ValueError("Wikidata manifest differs from frozen driver")
        ensure_object(root, WINDOW_PIN)
        windows = destination.parent / "windows"
        windows.mkdir()
        for window in spec["windows"]:
            path = f"{spec['windows_dir']}/{window['name']}.bin"
            data = subprocess.check_output(["git", "-C", str(root), "show", f"{WINDOW_PIN}:{path}"])
            (windows / (window["name"] + ".bin")).write_bytes(data)
        # Existing extractor verifies every pinned window, reconstructed stream and
        # final compressed hash; its libbzip2 validation is input construction only.
        module = importlib.util.spec_from_file_location(
            "cells_corpus", root / "bench/cells-gap/corpus.py"
        )
        corpus = importlib.util.module_from_spec(module)
        module.loader.exec_module(corpus)
        encoded = corpus.rebuild(spec, windows)
        (destination / "wikidata.bz2").write_bytes(encoded)
        canonical.parent.mkdir(exist_ok=True)
        shutil.copy2(destination / "wikidata.bz2", canonical)
        origin = "reconstructed from pinned benchmark windows"
    identities = {name: digest(destination / f"{name}.bz2") for name in DIGESTS}
    if identities != DIGESTS:
        raise ValueError("staged corpus differs from frozen driver")
    return dict(
        directory=str(destination.resolve()),
        sha256=identities,
        wikidata_origin=origin,
        window_pin=WINDOW_PIN,
    )


def prepare(attempt):
    record = machine()
    record["preparation_sha256"] = {
        path: digest(ROOT / path)
        for path in (
            ".just/bench.just",
            "bench/cells-gap/live/workstation.py",
            "bench/cells-gap/live/inputs.py",
            "bench/cells-gap/corpus.py",
            "bench/wikidata.json",
        )
    }
    path = attempt / "preflight.json"
    path.write_text(json.dumps(record, indent=2) + "\n")
    record["inputs"] = stage_inputs(attempt / "inputs")
    ensure_object(ROOT, PIN)
    ensure_toolchain()
    record["source_pin"] = PIN
    record["rustup_toolchain"] = os.environ.get("RUSTUP_TOOLCHAIN")
    path.write_text(json.dumps(record, indent=2) + "\n")
    (attempt / "cpu").write_text(str(record["selected_cpu"]) + "\n")


if __name__ == "__main__":
    prepare(Path(sys.argv[1]))
