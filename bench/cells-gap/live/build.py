"""Build the four frozen executables without running any decoder."""

import json
import os
import shutil
import sys
import _bz2
import subprocess
from pathlib import Path

from driver import HERE, PIN, digest

RUSTC = "rustc 1.99.1 (58cf8f9c3 2026-10-13)"


def build(destination):
    destination.mkdir(parents=True)
    env = os.environ.copy()
    if any(
        value
        for key, value in env.items()
        if key.endswith("RUSTFLAGS")
        or key in ("RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_TARGET")
        or key.startswith("CARGO_PROFILE_RELEASE_")
    ):
        raise ValueError("non-frozen Rust flags")
    rustc = subprocess.check_output(["rustc", "-Vv"], text=True)
    if rustc.splitlines()[0] != RUSTC or "host: x86_64-unknown-linux-gnu" not in rustc:
        raise ValueError("requires frozen Rust1.99.1 x86_64-unknown-linux-gnu")
    source = json.loads((HERE / ".generated/source.json").read_text())
    for path, identity in source["original"].items():
        expected = source["edits"].get(path, {}).get("diagnostic", identity)
        if digest(HERE / ".source" / path) != expected:
            raise ValueError("prepared source changed: " + path)
    if digest(HERE / ".source/src/live_diagnostic.rs") != source["trace_sha256"]:
        raise ValueError("trace source mismatch")
    if (
        digest(HERE / "prepare.py") != source["generator_sha256"]
        or digest(HERE / "src/trace.rs") != source["trace_sha256"]
    ):
        raise ValueError("generator/trace changed after preparation")
    if digest(HERE / ".generated/hash.rs") != source["hash_sha256"]:
        raise ValueError("original drain hash changed")
    binaries = {}
    for arm in ("baseline", "cells"):
        for trace in (False, True):
            key = arm + ("-on" if trace else "-off")
            features = (["cells-reader"] if arm == "cells" else []) + (
                ["live-trace"] if trace else []
            )
            command = [
                "cargo",
                "build",
                "--release",
                "--locked",
                "--manifest-path",
                str(HERE / "Cargo.toml"),
                "--no-default-features",
            ]
            if features:
                command += ["--features", ",".join(features)]
            with (destination / f"{key}.build.log").open("x") as log:
                subprocess.run(command, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
            shutil.copy2(HERE / "target/release/cells-live", destination / key)
            binaries[key] = digest(destination / key)
    record = dict(
        diagnostic_head=subprocess.check_output(
            ["git", "-C", str(HERE), "rev-parse", "HEAD"], text=True
        ).strip(),
        tracked_dirty=bool(
            subprocess.check_output(
                ["git", "-C", str(HERE), "status", "--porcelain", "--untracked-files=no"], text=True
            ).strip()
        ),
        pin=PIN,
        python=sys.version,
        python_executable_sha256=digest(sys.executable),
        bz2_module_origin=_bz2.__spec__.origin,
        bz2_extension_sha256=digest(_bz2.__file__) if getattr(_bz2, "__file__", None) else None,
        rustc=rustc,
        binaries=binaries,
        source=source,
        harness={
            str(p.relative_to(HERE)): digest(p)
            for p in [
                HERE / "Cargo.toml",
                HERE / "Cargo.lock",
                HERE / "prepare.py",
                HERE / "build.py",
                HERE / "driver.py",
                HERE / "src/main.rs",
                HERE / "src/trace.rs",
                HERE / "inputs.py",
                HERE / "run.sh",
                HERE / "check.py",
                HERE / "test_driver.py",
            ]
        },
    )
    (destination / "build.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps({"built": binaries, "measurement": False}))


if __name__ == "__main__":
    import sys

    build(Path(sys.argv[1]))
