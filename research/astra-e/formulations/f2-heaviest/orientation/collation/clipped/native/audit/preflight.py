"""Predict the mixed suffix-trie branches by advancing at most two pairs of ranks."""
from bisect import bisect_right
import csv
from pathlib import Path
import sys


def detect(n, lo, hi, edge, forward):
    if not 0 < edge < n:
        return "outer_edge", 0, None
    x, left = forward(edge-1)
    y, right = forward(edge)
    probes = 2
    if left != right:
        return "F_boundary", probes, None
    initial = x
    word = []
    while True:
        # Under equal forward symbols, rank distance cannot decrease.
        if y-x >= hi-lo:
            return "gap", probes, None
        in_x, in_y = lo <= x < hi, lo <= y < hi
        if in_x or in_y:
            if in_x and in_y:
                return "mixed", probes, (tuple(reversed(word)), left)
            return "unequal_return", probes, None
        x, a = forward(x)
        y, b = forward(y)
        probes += 2
        if a != b:
            return "unequal_byte", probes, None
        if x == initial:
            return "cycle_miss", probes, None
        word.append(a)


def real(tapes):
    here = Path(__file__).resolve().parent
    root = here.parents[8]
    sys.path.insert(0, str(root/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths, pieces, read_input
    with (here.parents[1]/"wikidata.csv").open() as f:
        sections = list(csv.DictReader(f))
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(("ordinal", "section", "edge", "reason", "select_probes", "parent_depth", "extension_byte"))
    for tape in paths(tapes):
        ordinal = int(tape.name.split(".")[0])
        runs, _ = read_input(tape)
        indexed = pieces(runs)
        starts = [r[0] for r in indexed]
        n = indexed[-1][1]
        def step(x):
            _, _, displacement, byte = indexed[bisect_right(starts,x)-1]
            return x+displacement, byte
        for row in sections:
            if int(row["ordinal"]) != ordinal:
                continue
            a,b = int(row["lo"]),int(row["hi"])
            mixed = 0
            for label,e in (("left",a),("right",b)):
                reason, probes, branch = detect(n,a,b,e,step)
                mixed += branch is not None
                depth, byte = ("","") if branch is None else (len(branch[0]),branch[1])
                writer.writerow((ordinal,row["section"],label,reason,probes,depth,byte))
            assert mixed == int(row["partial"])


if __name__ == "__main__":
    real(Path(sys.argv[1]))
