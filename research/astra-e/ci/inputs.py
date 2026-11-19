"""Identify completed measurements whose compiled repository inputs are all unchanged."""

import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess
import sys

# Cargo records compiler-tracked Rust modules and include! inputs in its binary
# depfile. These additional inputs control the build, corpus or measurement.
CONTROLS = [
    ".cargo", ".github/workflows/astra-e.yml", ".just", "Justfile", "Cargo.lock",
    "Cargo.toml", "rust-toolchain.toml", "bench/moonshot", "bench/wikidata.json",
    "bench/wikidata.py", "examples/windows.rs", "tests/common",
    "research/astra-e/ci/candidate", "research/astra-e/ci/bench.sh",
    "research/astra-e/ci/inputs.py", "research/astra-e/ci/paired.py",
    "research/astra-e/prepare.sh", ":(glob)research/astra-e/**/prepare.py",
]


# Each predecessor has its own frozen checkout, including generated inputs,
# worker code and Cargo dependencies. The harness receives the same scorer
# projection as the PR head and the master-based references.
COMBINED = {
    "commit": "5c7704b3d149b05b8e9dc2ace06da0179459a8b3",
    "path": "research/astra-e/narrow-short/ahead.rs",
}
MEASURED_COMBINED = {
    "commit": "80d8409359214b4e5abfd4430213921e82e9cd30",
    "path": "research/astra-e/narrow-short/ahead.rs",
}
MEASURED_EDGE = {
    "commit": "badb2aec3624bd65606c429b2e6f79e9d9d0050f",
    "path": "research/astra-e/edge-budget/candidate.rs",
}
STATIC_FRONTIER = {
    "commit": "d48e80ff01eeee6d23277945e2f58b61004470aa",
    "path": "research/astra-e/formulations/f2-frontier/candidate.rs",
}
COUNTED = {
    "commit": "252646aec0dd1a80a6969b2e1598c73486006efd",
    "path": "research/astra-e/formulations/f2-front-counts/candidate.rs",
}
GUARDED_POPCNT = {
    "commit": "3651470cb0ef5458e75cf97d3de363a1c73f2f76",
    "path": "research/astra-e/formulations/f2-pages/instructions/candidate.rs",
}
CAP_COUNTER = "research/astra-e/front-fusion/cap-counter/both.rs"
SCALAR = {
    "commit": "d0a42980e896211906234c7679b38760ae44a46e",
    "path": "research/astra-e/front-fusion/cap-batch/worker/candidate.rs",
}
COUNTER_EDGE = {
    "commit": "50b8ecb8946a498601e3fd894ed84a2687df6cc8",
    "path": "research/astra-e/front-fusion/cap-counter/counter_edge.rs",
}
PREDECESSORS = {
    CAP_COUNTER: SCALAR,
    "research/astra-e/front-fusion/cap-batch/worker/candidate.rs": MEASURED_EDGE,
    "research/astra-e/worker-dose/candidate.rs": MEASURED_EDGE,
    "research/astra-e/front-fusion/precount/worker/candidate.rs": MEASURED_COMBINED,
    "research/astra-e/formulations/f2-pages/payload/slots/preload.rs": GUARDED_POPCNT,
    "research/astra-e/formulations/f2-pages/instructions/candidate.rs": COUNTED,
    "research/astra-e/edge-budget/candidate.rs": COMBINED,
    "research/astra-e/final-slot/candidate.rs": COMBINED,
    "research/astra-e/formulations/f2-rle/combined/candidate.rs": COMBINED,
    "research/astra-e/formulations/f2-scheduler/candidate.rs": STATIC_FRONTIER,
}


def predecessor():
    candidate = Path("research/astra-e/ci/candidate").read_text().strip()
    return PREDECESSORS.get(candidate)


def extra_arms():
    candidate = Path("research/astra-e/ci/candidate").read_text().strip()
    return {"edge": MEASURED_EDGE, "counter": COUNTER_EDGE} if candidate == CAP_COUNTER else {}


def git(*args):
    return subprocess.check_output(["git", *args], text=True).strip()


def scorer_revision():
    return git("log", "-1", "--format=%H", "origin/master", "--", "bench/moonshot")


def record(deps, base):
    root = Path.cwd()
    sources = set()
    # The release depfile also contains the build binary and generated OUT_DIR files.
    for entry in shlex.split(deps.read_text().split(":", 1)[1]):
        path = Path(entry).resolve()
        if path.is_file() and path.is_relative_to(root):
            sources.add(str(path.relative_to(root)))
    candidate = Path("research/astra-e/ci/candidate").read_text().strip()
    if candidate not in sources:
        raise ValueError(f"Cargo depfile omits selected candidate {candidate}")
    tracked = set(git("ls-files").splitlines())
    generated = {
        path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
        for path in sources - tracked
    }
    return {
        "head": git("rev-parse", "HEAD"),
        "base": base,
        "scorer": scorer_revision(),
        "predecessor": predecessor(),
        "extra_arms": extra_arms(),
        "inputs": sorted(sources.intersection(tracked) | set(CONTROLS)),
        "generated": generated,
    }


def reusable(receipt, base):
    if receipt.get("predecessor") != predecessor() or receipt.get("extra_arms", {}) != extra_arms():
        return False
    if receipt["base"] != base or receipt["scorer"] != scorer_revision():
        return False
    for path, digest in receipt["generated"].items():
        if not Path(path).is_file() or hashlib.sha256(Path(path).read_bytes()).hexdigest() != digest:
            return False
    if subprocess.run(["git", "merge-base", "--is-ancestor", receipt["head"], "HEAD"]).returncode:
        return False
    return subprocess.run([
        "git", "diff", "--quiet", receipt["head"], "HEAD", "--", *receipt["inputs"]
    ]).returncode == 0


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("mode", choices=["check", "record"])
    p.add_argument("receipt", type=Path)
    p.add_argument("base")
    p.add_argument("--depfile", type=Path)
    args = p.parse_args()
    if args.mode == "record":
        args.receipt.write_text(json.dumps(record(args.depfile, args.base), indent=2) + "\n")
    else:
        skip = False
        if args.receipt.is_file():
            try:
                skip = reusable(json.loads(args.receipt.read_text()), args.base)
            except (ValueError, KeyError, OSError, subprocess.CalledProcessError):
                pass  # A stale cache entry simply causes a new run.
        print(f"run={'false' if skip else 'true'}")


if __name__ == "__main__":
    if sys.argv[1:] == ["comparators"]:
        arms = {"predecessor": predecessor(), **extra_arms()}
        for name, entry in arms.items():
            if entry:
                print(name, entry["commit"], entry["path"])
    elif sys.argv[1:] == ["predecessor"]:
        matched = predecessor()
        if matched:
            print(matched["commit"], matched["path"])
    else:
        main()
