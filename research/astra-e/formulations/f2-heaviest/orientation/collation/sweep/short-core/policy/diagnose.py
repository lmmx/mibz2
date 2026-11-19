"""Partition lost cap-eight savings by the frozen policy's first omitted literal. No policy search."""
import csv
import json
import lzma
import sys
from collections import Counter
from pathlib import Path

from model import get_input, obligations


def reason(required, enabled, seen):
    for i in range(len(required)+1):
        key = required[:i]
        if key not in enabled:
            return "disabled" if key in seen else "unseen"
    return None


def explain(ordinal, tape, seen, enabled, q8, qfit):
    _, _, _, words, _ = get_input(tape)
    lost = Counter()
    for required, weight in obligations(words, 8).items():
        reasons = {reason(prefix,enabled,seen) for prefix in required} - {None}
        if reasons:
            lost["+".join(sorted(reasons))] += weight
    total_lost = sum(lost.values())
    return dict(ordinal=ordinal,cap8_savings_lost=total_lost,
                disabled_only=lost["disabled"],unseen_only=lost["unseen"],
                disabled_and_unseen=lost["disabled+unseen"],
                additional_literal_savings=total_lost+q8-qfit)


if __name__ == "__main__":
    directory, corpus, output = map(Path,sys.argv[1:])
    enabled = {bytes.fromhex(key) for key in json.loads((directory/"policy.json").read_text())}
    with lzma.open(directory/"frozen-costs.csv.xz","rt") as f:
        seen = {bytes.fromhex(row["prefix_hex"]) for row in csv.DictReader(f)}
    frozen = json.loads((directory/"frozen.json").read_text())
    fit = json.loads((directory/"fit.json").read_text())
    with (directory/"heldout.csv").open() as f:
        values = {int(row["ordinal"]):(int(row["cap8_Q"]),int(row["literal_Q"])) for row in csv.DictReader(f)}
    values[0] = frozen["cap8"]["Q"],fit["fitted"]["Q"]
    with output.open("w") as f:
        writer = None
        for ordinal,(q8,qfit) in sorted(values.items()):
            row = explain(ordinal, corpus/f"{ordinal:04d}.irec.xz",seen,enabled,q8,qfit)
            if writer is None:
                writer = csv.DictWriter(f,fieldnames=row,lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)
