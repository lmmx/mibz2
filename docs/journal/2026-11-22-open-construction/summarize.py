"""Summarize one comparison per process on paired log time ratios."""

import csv, gzip, json, math, statistics as S, sys
from collections import defaultdict
from pathlib import Path

stages = [
    "S1 first pass",
    "S2a layout and nodes",
    "S2b elimination",
    "S3a coordinates bands and words",
    "S4 wire",
    "S3b separate inline table",
    "S5 emit",
    "S3a bottom only",
]


def summary(xs):
    return round(S.median(xs), 3)


def ci(pairs):
    logs = [math.log(b / a) for a, b in pairs]
    avg = S.mean(logs)
    if len(logs) != 12:
        raise ValueError(len(logs))
    radius = 2.200985 * S.stdev(logs) / math.sqrt(len(logs))
    return [round(math.exp(v), 5) for v in (avg, avg - radius, avg + radius)]


def make(raw, arms, out):
    groups = defaultdict(list)
    for ln in raw:
        r, conf, w, a, rows = json.loads(ln)
        if "parity" in conf:
            continue
        mode = conf[1:]
        if mode in ("t1", "t4"):
            blocks = [v for k, v in rows.items() if k != "pass"]
            rows["sum of block times"] = [
                sum(pair[i] for pair in blocks) for i in range(2)
            ]
        for key, (ref, cand) in rows.items():
            stage = "wall"
            unit = "us"
            block = key
            if mode.startswith("stage"):
                block, idx = key.rsplit(".", 1)
                stage = stages[int(idx)]
            if mode.startswith(("cs", "corpus")):
                block = "stream"
                idx = int(key.rsplit(".", 1)[1])
                unit = "ms"
                stage = (
                    stages[idx - 1]
                    if idx
                    else ("wall (timers enabled)" if mode.startswith("cs") else "wall")
                )
                ref /= 1000
                cand /= 1000
            groups[conf[0], mode, w, a, block, stage, unit].append((ref, cand))
    writer = csv.writer(out, lineterminator="\n")
    writer.writerow(
        [
            "dataset",
            "workers",
            "measurement",
            "block",
            "stage",
            "unit",
            "arm",
            "reference",
            "rounds",
            "reference_median",
            "candidate_median",
            "ratio",
            "lower95",
            "upper95",
        ]
    )
    for (ds, mode, w, a, block, stage, unit), pairs in sorted(groups.items()):
        if len(pairs) != 12:
            raise ValueError((a, mode, len(pairs)))
        reference, candidate = zip(*pairs)
        ratios = ci(pairs) if all(x > 0 and y > 0 for x, y in pairs) else ["", "", ""]
        arm, base = arms[a]
        writer.writerow(
            [
                {"s": "admitted" if block != "stream" else "wikidata", "d": "dense"}[
                    ds
                ],
                w,
                mode,
                block,
                stage,
                unit,
                arm,
                base,
                len(pairs),
                summary(reference),
                summary(candidate),
                *ratios,
            ]
        )


if __name__ == "__main__":
    with Path(sys.argv[3]).open("w") as out:
        make(
            Path(sys.argv[1]).read_text().splitlines(),
            json.loads(Path(sys.argv[2]).read_text()),
            out,
        )
