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

`phases` is read by its text, by each line's label, so that a base built before any
change to phases.rs reads the same: GHZ is given as 1000, at which its "c/B" figures are
picoseconds a byte (cycles a byte at 1000 GHz), two decimals of them; the stage that
counts towards its total is the walk it uses, the others marked "(not in the total)".

The rows not in the total (the walks not used, and research rows such as design E's) are
kept too, from bench 3.5.0, by label (`others`): each one's picoseconds a byte and the stage
it stands against, where its line names one (`against: walk`, `against: build+walk`), for
bench/report.py's research table. They are never part of the stage table.
"""

import argparse
import json
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


AGAINST = re.compile(r"against: ([a-z+]+)")


def others(text: str) -> dict[str, dict]:
    """phases' rows not in the total: each one's picoseconds a byte and, where its line
    names one, the stage it stands against (stage keys joined by "+"), by label."""
    out = {}
    for line in text.splitlines():
        if (m := LINE.match(line)) and "not in the total" in m.group(3):
            a = AGAINST.search(m.group(3))
            against = a.group(1) if a and set(a.group(1).split("+")) <= set(STAGES) else None
            out[m.group(1)] = {"ps": float(m.group(2)), "against": against}
    return out


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
                t = time.monotonic()
                p = subprocess.run(cmd, capture_output=True, text=True)
                wall = time.monotonic() - t
                ps = parse(p.stdout)
                rec = {"round": rnd, "arm": arm, "repeats": a.repeats, "wall": round(wall, 3)}
                rec["walk"] = walk_name(p.stdout)
                if p.returncode or not set(STAGES) - OPTIONAL <= set(ps) <= set(STAGES):
                    rec["err"] = (p.stderr.strip().splitlines() or ["no stages read"])[-1][:200]
                rec["ps_per_byte"] = ps
                rec["others"] = others(p.stdout)
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
    r.add_argument("file")
    r.add_argument("arms", nargs="+", metavar="ARM=EXE")
    run(ap.parse_args())


if __name__ == "__main__":
    main()
