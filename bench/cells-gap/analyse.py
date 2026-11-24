"""Replay archived i9 figures against an untimed production census; no clocks.

Usage: analyse.py CENSUS.jsonl I9_CENSUS... > analysis.json
The hypothetical mixed inversion ratio excludes refusal overhead and assumes the
research timing survives the port. It is not a prediction of decoder speed.
"""

import json
import sys
from collections import defaultdict
from pathlib import Path


def load(path):
    return [json.loads(line) for line in Path(path).read_text().splitlines()]


def gate(row):
    if 10 * row["records"] >= row["n"]:
        return "density"
    if row["c_letters"] - 1 > 4 * row["n"] // 25:
        return "floor"
    if row["c_cells"] > 4 * row["n"] // 25:
        return "budget"
    return "admitted"


def analyse(census, historical):
    blocks = [r["data"] for r in census if r["kind"] == "block"]
    assert blocks and all(b["inversion_ns"] is None for b in blocks)
    grouped = defaultdict(lambda: {"blocks": 0, "rows": 0, "output_bytes": 0})
    for b in blocks:
        g = grouped[b["route"]]
        g["blocks"] += 1
        g["rows"] += b["rows"]
        g["output_bytes"] += b["bytes"]
    total = sum(b["rows"] for b in blocks)
    selected = [b for b in blocks if b["rows"] >= 65536]
    repeats = []
    for old in historical:
        # The historical tool uses the same ordered corpus with min=65536, every=1.
        assert len(old) == len(selected)
        for b, r in zip(selected, old):
            assert b["rows"] == r["n"] and r["takes"]
            assert b["route"] == gate(r), (b["block"], b["route"], gate(r))
        admitted = [r for r in old if gate(r) == "admitted"]
        prod = sum(r["ns_rec"] * r["n"] for r in old)
        cells = sum(r["c_whole"] * r["n"] for r in old)
        mixed = sum((r["c_whole"] if gate(r) == "admitted" else r["ns_rec"]) * r["n"] for r in old)
        weight = sum(r["n"] for r in admitted)
        repeats.append({
            "historical_all_cells_over_records": cells / prod,
            "hypothetical_mixed_over_records_excluding_refusal_cost": mixed / prod,
            "hypothetical_saved_ns": prod - mixed,
            "admitted_research_ns_per_row": {
                k: sum(r[k] * r["n"] for r in admitted) / weight
                for k in ["c_order", "c_induce", "c_write", "c_whole", "ns_rec"]
            },
        })
    return {
        "blocks": len(blocks), "rows": total,
        "output_bytes": sum(b["bytes"] for b in blocks),
        "routes": dict(grouped),
        "admitted_row_share": grouped["admitted"]["rows"] / total,
        "below_historical_minimum": {"blocks": len(blocks) - len(selected),
                                      "rows": sum(b["rows"] for b in blocks if b["rows"] < 65536)},
        "historical_repeats": repeats,
        "warning": "Historical counterfactual only. Different hardware/cache/port; refusal work omitted. No whole-decoder prediction.",
    }


if __name__ == "__main__":
    print(json.dumps(analyse(load(sys.argv[1]), [load(p) for p in sys.argv[2:]]), indent=2))
