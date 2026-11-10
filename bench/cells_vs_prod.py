"""Summarise `rauzy` census runs: the cells kernel (c_whole) against production's records path (ns_rec).

Both read the same run records; ns_rec is production's inversion stage since 0.2.8 for a block
that `takes` the records path. Pooled ratio = sum(c_whole * n) / sum(ns_rec * n), per repeat,
over all blocks and over the blocks production sends down the records path.

    python3 bench/cells_vs_prod.py OUT.json census-1.jsonl census-2.jsonl ...
"""
import json
import sys


def pool(rows):
    c = sum(r["c_whole"] * r["n"] for r in rows)
    p = sum(r["ns_rec"] * r["n"] for r in rows)
    return {"blocks": len(rows), "rows": sum(r["n"] for r in rows),
            "cells_ns_row": c / max(1, sum(r["n"] for r in rows)),
            "prod_ns_row": p / max(1, sum(r["n"] for r in rows)),
            "ratio": c / p if p else None}


def main(out, paths):
    reps = []
    for p in paths:
        rows = [json.loads(l) for l in open(p) if l.startswith("{")]
        reps.append({"file": p, "all": pool(rows), "takes": pool([r for r in rows if r.get("takes")])})
    res = {"repeats": reps,
           "all_below_1": all(r["all"]["ratio"] is not None and r["all"]["ratio"] < 1 for r in reps),
           "takes_below_1": all(r["takes"]["ratio"] is not None and r["takes"]["ratio"] < 1 for r in reps)}
    json.dump(res, open(out, "w"), indent=1)
    for r in reps:
        a, t = r["all"], r["takes"]
        print(f"{r['file']}: all {a['blocks']} blocks ratio {a['ratio']:.3f} "
              f"({a['cells_ns_row']:.3f} vs {a['prod_ns_row']:.3f} ns/row); "
              f"records-path blocks {t['blocks']} ratio {t['ratio'] if t['ratio'] is None else round(t['ratio'], 3)}")
    print("cells below production in every repeat (all blocks):", res["all_below_1"])
    print("cells below production in every repeat (records-path blocks):", res["takes_below_1"])


main(sys.argv[1], sys.argv[2:])
