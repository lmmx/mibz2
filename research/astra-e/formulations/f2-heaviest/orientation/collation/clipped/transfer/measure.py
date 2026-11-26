"""Test the two frozen interval choices against each block's matched-mass named interval."""
import argparse
import csv
from pathlib import Path
import sys

here = Path(__file__).resolve().parent
sys.path.insert(0, str(here.parent))
from induce import Column, Oracle, check, heavy_set

sys.path.insert(0, str(here.parents[7] / "docs/journal/2026-11-18-astra-c-output-first"))
from inputs import paths, read_input


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tapes", type=Path)
    args = parser.parse_args()
    writer = None
    for tape in paths(args.tapes):
        ordinal = int(tape.name.split(".")[0])
        runs, mark = read_input(tape)
        col = Column(runs)
        dense = Oracle(b"".join(bytes([c]) * width for c, width in runs))
        named = heavy_set(col, col.interval(b'"'), 1)[0][0]
        mass = named[1] - named[0]
        colon = sum(width for c, width in runs if c < ord(":"))
        quantile = (333831 * col.rows + 899931 // 2) // 899931
        for label, start in (("named", named[0]), ("colon", colon), ("quantile", quantile)):
            start = min(start, col.rows - mass)
            row = dict(ordinal=ordinal, policy=label, rows=col.rows, primary=mark, start=start, mass=mass,
                       **check(col, dense, (start, start + mass)))
            if writer is None:
                writer = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)
            sys.stdout.flush()


if __name__ == "__main__":
    main()
