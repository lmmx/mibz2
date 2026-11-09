"""Tables for the certified-packet census (examples/packet_census.rs; journal
docs/journal/2026-11-09-packet-census.md): its JSON lines, one a block, summed by corpus.

    python3 bench/packet_census.py CENSUS.jsonl [...]

Counts only. Every figure is a ratio of counts the census made; the loads a byte are a
model (one dependent load a packet), printed apart from the measured coverage.
"""

import json
import sys
from collections import defaultdict

MODELS = [
    "single_k1",
    "single_k2r",
    "single_k4r",
    "single_k8r",
    "single_k16r",
    "pair_k1",
    "pair_k2r",
    "pair_k3r",
    "pair_k4r",
    "pair_k8r",
    "pair_k4path",
]
THRESH = [2, 4, 8, 16, 64]
QS = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0]


def quantiles(xs):
    xs = sorted(xs)
    out = []
    for q in QS:
        i = q * (len(xs) - 1)
        lo, hi = int(i), min(int(i) + 1, len(xs) - 1)
        out.append(xs[lo] + (xs[hi] - xs[lo]) * (i - lo))
    return out


def load(paths):
    """Each corpus's blocks; a corpus's short blocks (the plain walk's, below
    MIN_SEGMENTED) as a row of their own, `CORPUS (short)`"""
    rows = defaultdict(list)
    for p in paths:
        with open(p) as f:
            for line in f:
                d = json.loads(line)
                name = d["corpus"] if d["segmented"] else d["corpus"] + " (short)"
                rows[name].append(d)
    return rows


def main(paths):
    rows = load(paths)
    print("## Cells and bounds\n")
    print(
        "| corpus | blocks | segmented | n (rows) | r/n | Ψ cells/r | single cells/r | Ψ² cells/r | pair cells/r | bounds |"
    )
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|---|")
    for c, bs in rows.items():
        n = sum(b["n"] for b in bs)
        r = sum(b["r"] for b in bs)
        ok = all(
            b["psi_cells"] <= b["r"]
            and b["single_cells"] <= 2 * b["r"] - 1
            and b["psi2_cells"] <= 2 * b["r"] - 1
            and b["pair_cells"] <= 3 * b["r"] - 2
            for b in bs
        )
        print(
            f"| {c} | {len(bs)} | {sum(b['segmented'] for b in bs)} | {n:,} | {r / n:.4f} | "
            f"{sum(b['psi_cells'] for b in bs) / r:.3f} | {sum(b['single_cells'] for b in bs) / r:.3f} | "
            f"{sum(b['psi2_cells'] for b in bs) / r:.3f} | {sum(b['pair_cells'] for b in bs) / r:.3f} | "
            f"{'all hold' if ok else 'VIOLATED'} |"
        )
    print("\n## Coverage: the share of output bytes in packets of at least L bytes\n")
    print(
        "| corpus | model | ≥2 | ≥4 | ≥8 | ≥16 | ≥64 | mean packet (B) | packets/B | lookups/B | descriptors/r |"
    )
    print("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for c, bs in rows.items():
        n = sum(b["n"] for b in bs)
        r = sum(b["r"] for b in bs)
        for m in MODELS:
            ge = [sum(b[m]["bytes_ge"][i] for b in bs) / n for i in range(5)]
            pk = sum(b[m]["packets"] for b in bs)
            lk = sum(b[m]["lookups"] for b in bs)
            de = sum(b[m]["descriptors"] for b in bs)
            print(
                f"| {c} | {m} | "
                + " | ".join(f"{g:.4f}" for g in ge)
                + f" | {n / pk:.3f} | {pk / n:.4f} | {lk / n:.4f} | {de / r:.3f} |"
            )
    print(
        "\n## Ceiling: bytes inside a tandem repeat (two whole copies or more) within a segment\n"
    )
    print("Any repeated packet W^m (m >= 2) is one, of period |W|. Period at most:\n")
    print("| corpus | ≤4 | ≤8 | ≤16 | ≤32 | ≤64 |")
    print("|---|---:|---:|---:|---:|---:|")
    for c, bs in rows.items():
        n = sum(b["n"] for b in bs)
        print(
            f"| {c} | "
            + " | ".join(f"{sum(b['tandem_le'][i] for b in bs) / n:.4f}" for i in range(5))
            + " |"
        )
    print("\n## Per-block quantiles (min, p10, p25, median, p75, p90, max), unweighted\n")
    print(
        "| corpus | statistic | "
        + " | ".join(["min", "p10", "p25", "p50", "p75", "p90", "max"])
        + " |"
    )
    print("|---|---|" + "---:|" * len(QS))
    stats = [
        ("r/n", lambda b: b["r"] / b["n"]),
        ("pair cells/r", lambda b: b["pair_cells"] / b["r"]),
        ("single cells/r", lambda b: b["single_cells"] / b["r"]),
        ("single_k1 ≥2", lambda b: b["single_k1"]["bytes_ge"][0] / b["n"]),
        ("single_k8r ≥4", lambda b: b["single_k8r"]["bytes_ge"][1] / b["n"]),
        ("pair_k1 ≥4", lambda b: b["pair_k1"]["bytes_ge"][1] / b["n"]),
        ("pair_k4r ≥4", lambda b: b["pair_k4r"]["bytes_ge"][1] / b["n"]),
        ("pair_k4r ≥16", lambda b: b["pair_k4r"]["bytes_ge"][3] / b["n"]),
        ("pair_k8r ≥4", lambda b: b["pair_k8r"]["bytes_ge"][1] / b["n"]),
        ("single_k16r ≥4", lambda b: b["single_k16r"]["bytes_ge"][1] / b["n"]),
        ("tandem ≤8", lambda b: b["tandem_le"][1] / b["n"]),
        ("tandem ≤64", lambda b: b["tandem_le"][4] / b["n"]),
        ("pair_k4r packets/B", lambda b: b["pair_k4r"]["packets"] / b["n"]),
        ("single_k8r packets/B", lambda b: b["single_k8r"]["packets"] / b["n"]),
    ]
    for c, bs in rows.items():
        for name, f in stats:
            qs = quantiles([f(b) for b in bs])
            print(f"| {c} | {name} | " + " | ".join(f"{q:.4f}" for q in qs) + " |")
    print("\n## Model (not measured): dependent loads a byte\n")
    print("One dependent load a packet when its descriptor is found from the row in one load")
    print("(cached paths); Σk lookups when each path is rediscovered cell by cell.\n")
    print(
        "| corpus | production (1 a byte) | pair walk | single k16r (cached) | pair k1 | pair k4r (cached) | pair k4r (uncached) | pair k8r (cached) | pair k4path (cached) |"
    )
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    for c, bs in rows.items():
        n = sum(b["n"] for b in bs)
        pw = sum(b["pair_walk_loads"] for b in bs) / n
        f = lambda m, key="packets": sum(b[m][key] for b in bs) / n
        print(
            f"| {c} | 1.0000 | {pw:.4f} | {f('single_k16r'):.4f} | {f('pair_k1'):.4f} | "
            f"{f('pair_k4r'):.4f} | {f('pair_k4r', 'lookups'):.4f} | {f('pair_k8r'):.4f} | {f('pair_k4path'):.4f} |"
        )


if __name__ == "__main__":
    main(sys.argv[1:])
