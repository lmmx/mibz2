#!/usr/bin/env python3
"""Targeted unsafe-kernel mutations, in an isolated source copy (never the checkout).

Usage: python3 .../mutations.py WORK_DIR
Requires cargo on PATH. Each mutant must compile and then fail its named test.
A timeout, build failure or surviving mutant fails the campaign.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(sys.argv[1]).resolve()
OUT.mkdir(parents=True, exist_ok=False)
TREE = OUT / "tree"
TREE.mkdir()
for name in ["Cargo.toml", "Cargo.lock"]:
    shutil.copy2(ROOT / name, TREE / name)
for name in ["src", "tests", "examples", "benches"]:
    shutil.copytree(ROOT / name, TREE / name)
source = (TREE / "src/cells.rs").read_text()
mutants = [
    ("uncounted-cell", "cells += 1;", "cells += 0;", "miri_cells_admission_and_invalid_records"),
    ("lost-mark", "wr32(base, tz + 8, 1 | MARK);", "wr32(base, tz + 8, 1);", "miri_cells_slots_mark_and_links"),
    ("wrong-backlink", "wr32(base, fb, rd32(base, bs));", "wr32(base, fb, fb as u32);", "cells_exhaustive_arbitrary_columns"),
    ("wrong-rotation", "const SHIFT: usize = 1;", "const SHIFT: usize = 0;", "cells_exhaustive_arbitrary_columns"),
    ("region-overrun", "let cap = (recs.len() + 2 + budget) * SLOT;", "let cap = (recs.len() + 1) * SLOT;", "miri_cells_exact_fit_and_reuse"),
    # Leave reach counters intact: output parity, not instrumentation, must kill it.
    ("missing-spine-recopy", "copy_long(dst.add(d), dst.add(f), lx);", "/* mutation: omit final marked-spine recopy */", "miri_cells_long_writer"),
]
env = dict(os.environ, CARGO_TARGET_DIR=str(OUT / "target"), CARGO_BUILD_JOBS="4")
results = {"source_sha256": hashlib.sha256(source.encode()).hexdigest(), "mutants": []}
for name, old, new, test in [("baseline", "", "", "cells::tests")] + mutants:
    if old:
        assert source.count(old) == 1, (name, "mutation anchor changed")
    (TREE / "src/cells.rs").write_text(source.replace(old, new, 1) if old else source)
    args = ["cargo", "test", "--lib", test]
    with (OUT / f"{name}.log").open("w") as log:
        build = subprocess.run(args + ["--no-run"], cwd=TREE, env=env, stdout=log, stderr=log, timeout=300)
        assert build.returncode == 0, (name, "build failure is not a kill")
        run = subprocess.run(args, cwd=TREE, env=env, stdout=log, stderr=log, timeout=60)
    killed = run.returncode != 0
    results["mutants"].append({"name": name, "test": test, "exit": run.returncode, "killed": killed})
    (OUT / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    assert killed == (name != "baseline"), (name, "baseline failed or mutant survived")
    print(name, "killed" if killed else "passed", flush=True)
