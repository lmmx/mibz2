"""The decode's stages timed on their own (examples/phases.rs), each build's `phases` run
in turn, round by round, on the Wikidata blocks: the runs bench/report.py's stage table
is made from (`--stages`).

    stages.py run OUT.jsonl --rounds R [--repeats N] [--pin CPU] FILE.bz2 ARM=EXE ...

Each round runs every arm's `phases FILE N GHZ` once (N: phases' passes, each a decode of
the file in decode order, each stage's fastest kept; a build from before bench 2.0.0 reads
N as repeats of each stage on each block), pinned to one CPU (`taskset`,
as `ab run` pins its 1-thread runs), the arms' order reversed every other round; one JSON
line a run: the round, the arm, the run's wall seconds, and each stage's time in
picoseconds a byte of output.

P74 Gate 1 (docs/journal/2026-10-30-p74-gate1-preregistration.md): a build's "MTF arm, ..."
lines, each MTF kernel timed on its own, are kept beside the stages in each run's record
(`arms`: picoseconds a byte, by "KERNEL/ARM"), not as stages; `--mtf-arms` runs phases with
MIBZ2_PHASES_MTF_ONLY set, those lines only (bench/p74.py reads them).

`phases` is read by its text, by each line's label, so that a base built before any
change to phases.rs reads the same: GHZ is given as 1000, at which its "c/B" figures are
picoseconds a byte (cycles a byte at 1000 GHz), two decimals of them; the stage that
counts towards its total is the walk it uses, the others marked "(not in the total)".
"""

import argparse
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

GHZ = 1000  # phases' "c/B" at 1000 GHz: picoseconds a byte
LINE = re.compile(r"^  (\S.*?)\s+(-?\d+(?:\.\d+)?) c/B(.*)$")
# The stages in phases' order (its labels' prefixes); `walk` is the walk in the total
STAGES = ["scan", "mtf", "build", "walk", "assemble", "rle", "crc", "total"]
# Stages a build's phases may not have: the segments' assembly, timed on its own from
# bench 2.1.0, within the walk's figure (which counts it, as it always has)
OPTIONAL = {"assemble"}


def stage_key(name: str, rest: str) -> str | None:
    if "not in the total" in rest:
        return None
    for key, prefix in (
        ("scan", "scanner"),
        ("mtf", "MTF"),
        ("build", "T-vector build"),
        ("walk", "walk"),
        ("assemble", "segment assembly"),
        ("rle", "RLE1"),
        ("crc", "CRC"),
        ("total", "total"),
    ):
        if name.startswith(prefix):
            return key
    return None


def parse(text: str) -> dict[str, float]:
    """phases' output: each stage's picoseconds a byte, by key."""
    out = {}
    for line in text.splitlines():
        if (m := LINE.match(line)) and (key := stage_key(m.group(1), m.group(3))):
            out[key] = float(m.group(2))
    return out


def mtf_arms(text: str) -> dict[str, float]:
    """phases' "MTF arm, KERNEL, ARM" lines: each kernel's picoseconds a byte, by
    "KERNEL/ARM" (P74 Gate 1)."""
    out = {}
    for line in text.splitlines():
        if (m := LINE.match(line)) and m.group(1).startswith("MTF arm, "):
            kernel, arm = m.group(1).removeprefix("MTF arm, ").split(", ", 1)
            out[f"{kernel}/{arm}"] = float(m.group(2))
    return out


SELECTION = re.compile(
    r"^  MTF selector: P74 for (\d+) of (\d+) blocks, (\d+) of (\d+) bytes of L$"
)


def selection(text: str) -> dict | None:
    """phases' selector line: the blocks and L bytes the Gate 1 selector sent to P74"""
    for line in text.splitlines():
        if m := SELECTION.match(line):
            a, n, ab, nb = map(int, m.groups())
            return {"blocks": a, "of_blocks": n, "bytes": ab, "of_bytes": nb}
    return None


def walk_name(text: str) -> str | None:
    """The walk the build's total counts (its line's label, e.g. "walk, room (used)"): the
    stage is the same across builds, its implementation each build's own."""
    for line in text.splitlines():
        if (m := LINE.match(line)) and stage_key(m.group(1), m.group(3)) == "walk":
            return m.group(1)
    return None


def run(a: argparse.Namespace) -> None:
    arms = [x.split("=", 1) for x in a.arms]
    with open(a.out, "w") as f:
        for rnd in range(a.rounds):
            order = arms if rnd % 2 == 0 else arms[::-1]
            for arm, exe in order:
                cmd = ["taskset", "-c", a.pin, exe, a.file, str(a.repeats), str(GHZ)]
                env = dict(os.environ, MIBZ2_PHASES_MTF_ONLY="1") if a.mtf_arms else None
                t = time.monotonic()
                p = subprocess.run(cmd, capture_output=True, text=True, env=env)
                wall = time.monotonic() - t
                ps = parse(p.stdout)
                rec = {"round": rnd, "arm": arm, "repeats": a.repeats, "wall": round(wall, 3)}
                rec["walk"] = walk_name(p.stdout)
                if kernels := mtf_arms(p.stdout):
                    rec["arms"] = kernels
                if sel := selection(p.stdout):
                    rec["selection"] = sel
                if a.mtf_arms:
                    if p.returncode or len(kernels) != 8:
                        rec["err"] = (p.stderr.strip().splitlines() or ["no arms read"])[-1][:200]
                elif p.returncode or not set(STAGES) - OPTIONAL <= set(ps) <= set(STAGES):
                    rec["err"] = (p.stderr.strip().splitlines() or ["no stages read"])[-1][:200]
                rec["ps_per_byte"] = ps
                f.write(json.dumps(rec) + "\n")
                f.flush()
                shown = " ".join(f"{k} {ps[k] / 1000:.2f}" for k in STAGES if k in ps)
                print(
                    f"r{rnd} {arm:<6} {wall:6.1f}s  ns/B: {shown} {rec.get('err', '')}",
                    file=sys.stderr,
                )


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("out")
    r.add_argument("--rounds", type=int, required=True)
    r.add_argument("--repeats", type=int, default=1)
    r.add_argument("--pin", default="1")
    r.add_argument(
        "--mtf-arms", action="store_true", help="phases' MTF arm lines only (P74 Gate 1)"
    )
    r.add_argument("file")
    r.add_argument("arms", nargs="+", metavar="ARM=EXE")
    run(ap.parse_args())


if __name__ == "__main__":
    main()
