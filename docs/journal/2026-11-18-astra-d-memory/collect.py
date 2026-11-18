"""Collect one measured lap per envelope with unmodified Valgrind Lackey."""

from fix_ranges import resolve
from symbols import static_ranges, append
import bz2
import os
import subprocess
import sys
from pathlib import Path

vg, probe, capture, blocks, out = (Path(s).resolve() for s in sys.argv[1:6])
names = sys.argv[6:] or [p.stem for p in sorted(blocks.glob("*.bz2"))]
symbols = subprocess.check_output(["nm", "-n", probe], text=True)
(mark,) = (line.split()[0] for line in symbols.splitlines() if line.endswith(" ASTRA_MARK"))
fixed = static_ranges(probe)
for name in names:
    run = out / name
    run.mkdir(parents=True, exist_ok=True)
    for dest, src in [("probe", probe), ("input.bz2", blocks / (name + ".bz2"))]:
        link = run / dest
        if not link.exists():
            link.symlink_to(src)
    env = {"LC_ALL": "C", "PATH": "/usr/bin:/bin"}
    if "VALGRIND_LIB" in os.environ:
        env["VALGRIND_LIB"] = os.environ["VALGRIND_LIB"]
    cmd = [
        vg,
        "--tool=lackey",
        "--basic-counts=no",
        "--trace-mem=yes",
        "--log-fd=2",
        "./probe",
        "input.bz2",
        "output",
        "inverse",
    ]
    with (run / "ranges").open("w") as metadata:
        with subprocess.Popen(
            cmd, cwd=run, env=env, stdout=metadata, stderr=subprocess.PIPE
        ) as valgrind:
            reduce_trace = subprocess.run(
                [capture, mark, "events", "instructions"], cwd=run, stdin=valgrind.stderr
            )
            valgrind.stderr.close()
            assert valgrind.wait() == 0 and reduce_trace.returncode == 0
    resolve(run)
    append(run, fixed)
    assert (run / "output").read_bytes() == bz2.decompress((run / "input.bz2").read_bytes())
    print(name, "matches libbz2", flush=True)
