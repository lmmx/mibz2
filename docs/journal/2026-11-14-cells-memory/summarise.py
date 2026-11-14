"""The Cells memory census's summary.

    python3 summarise.py DIR > DIR/summary.json

DIR holds trace-{cells,segmented}.jsonl (memtrace/consumer.rs, one line per kernel attempt)
and blocks-{cells,segmented}.jsonl (examples/cells_memory.rs, one line per block). Every
figure is over the blocks Cells admitted, both kernels on the same blocks, pooled (sums over
blocks, then divided by the sum of the blocks' n, the kernel's output bytes), unless named
otherwise. "lines" are distinct 64-byte lines of one attempt, summed over attempts.

Modelled misses: a line access misses a fully associative LRU cache of C lines exactly when
its stack distance d (distinct lines since its last access in the attempt) is at least C,
or it is the line's first access in the attempt (cold). The consumer keeps d in buckets
floor(log2(d + 1)), so C is a power of two and the boundary is d + 1 >= C (one line off).
"""

import json
import sys
from collections import defaultdict
from pathlib import Path

PHASE = {
    0: "pre", 10: "slots.count", 11: "slots.alloc", 12: "slots.letters", 20: "induce_cells",
    30: "write.fill", 31: "write.place", 32: "write.spine", 33: "write.descend",
    34: "write.expand", 35: "write.respine", 36: "write.repeat",
    50: "seg.fill", 51: "seg.walk", 52: "seg.assemble",
}
GROUP = {10: "slots", 11: "slots", 12: "slots", 20: "induce_cells", 0: "pre",
         50: "seg.fill", 51: "seg.walk", 52: "seg.assemble"}
for p in range(30, 37):
    GROUP[p] = "write_cells"
REGION = {
    0: "other", 99: "stack", 1: "recs", 2: "buf.pad", 3: "buf.tops", 4: "buf.bots",
    5: "buf.arena", 6: "buf.rules", 7: "buf.spare", 8: "pos", 9: "out", 20: "tt",
    21: "seg.starts", 22: "seg.bitmap", 23: "seg.next", 24: "seg.bufs", 25: "seg.bufs.outer",
    40: "fallback.tt", 41: "fallback.seg",
}
CACHES = {"32KiB": 9, "64KiB": 10, "256KiB": 12, "1MiB": 14, "2MiB": 15, "32MiB": 19}


def misses(reuse, k):
    """cold + accesses with d + 1 >= 2^k"""
    return reuse[25] + sum(reuse[k:25])


def retouch(reuse, k):
    """non-cold accesses with d + 1 >= 2^k: a line touched again after an eviction-sized gap"""
    return sum(reuse[k:25])


def pattern_phases(pat):
    w, r = pat.split("|")
    ws = {int(x) for x in w[1:].split("+") if x}
    rs = {int(x) for x in r[1:].split("+") if x}
    return ws, rs


def per_path(rows, nsum):
    out = {}
    tot = defaultdict(int)
    by_phase = defaultdict(lambda: defaultdict(float))
    by_region = defaultdict(lambda: defaultdict(float))
    by_pr = defaultdict(lambda: defaultdict(float))
    for a in rows:
        tot["distinct_lines"] += a["distinct_lines"]
        tot["events"] += a["events"]
        tot["line_events"] += a["line_events"]
        for s in a["stats"]:
            rw = s["rw"]
            g = GROUP[s["phase"]]
            reg = REGION[s["region"]]
            for d in (by_phase[g], by_region[reg], by_pr[(PHASE[s["phase"]], reg)]):
                d[f"{rw}_bytes"] += s["bytes"]
                d[f"{rw}_accesses"] += s["accesses"]
                d[f"{rw}_lines"] += s["lines"]
                for c, k in CACHES.items():
                    d[f"miss_{c}"] += misses(s["reuse"], k)
                    d[f"retouch_{c}"] += retouch(s["reuse"], k)
                d["cold"] += s["reuse"][25]
            tot[f"{rw}_bytes"] += s["bytes"]
            if s["region"] not in (0, 99):
                tot[f"{rw}_bytes_heap"] += s["bytes"]
            for c, k in CACHES.items():
                tot[f"miss_{c}"] += misses(s["reuse"], k)
        for reg, v in a["region_lines"].items():
            name = REGION[int(reg)]
            by_region[name]["lines"] += v["lines"]
            by_region[name]["multi_phase_lines"] += v["multi_phase"]
            by_region[name]["new_pages"] += v["new_pages"]
            if int(reg) not in (0, 99):
                tot["distinct_lines_heap"] += v["lines"]
            # lines touched by each phase group (from the patterns), and the patterns
            for pat, c in v["patterns"].items():
                ws, rs = pattern_phases(pat)
                for g in {GROUP[p] for p in ws | rs}:
                    by_phase[g]["lines_touched"] += c
                    by_pr[("group:" + g, name)]["lines_touched"] += c
                gw = "+".join(sorted({GROUP[p] for p in ws}))
                gr = "+".join(sorted({GROUP[p] for p in rs}))
                by_region[name][f"pattern W[{gw}] R[{gr}]"] += c
                by_region[name][f"fine W[{'+'.join(PHASE[p] for p in sorted(ws))}] R[{'+'.join(PHASE[p] for p in sorted(rs))}]"] += c
    per_byte = {k: v / nsum for k, v in tot.items() if k not in ("events", "line_events")}
    per_byte["touched_bytes_per_output_byte"] = 64 * tot["distinct_lines"] / nsum
    out["totals"] = dict(tot)
    out["per_output_byte"] = per_byte
    out["by_phase_per_output_byte"] = {
        g: {k: v / nsum for k, v in d.items()} for g, d in sorted(by_phase.items())}
    out["by_region_per_output_byte"] = {
        r: {k: v / nsum for k, v in sorted(d.items()) if not k.startswith(("pattern", "fine"))}
        for r, d in sorted(by_region.items())}
    out["by_region_patterns_share_of_region_lines"] = {
        r: {k: round(v / d["lines"], 4) for k, v in sorted(d.items(), key=lambda x: -x[1])
            if k.startswith(("pattern", "fine")) and v / d["lines"] >= 0.005}
        for r, d in sorted(by_region.items()) if d["lines"]}
    out["by_phase_region_per_output_byte"] = {
        f"{p} / {r}": {k: v / nsum for k, v in sorted(d.items())}
        for (p, r), d in sorted(by_pr.items())}
    return out


def capacity(rows):
    """per region: reserved this block, capacity after it, touched (lines * 64): medians,
    max, and pooled ratios"""
    res = defaultdict(lambda: defaultdict(list))
    for a in rows:
        lines = {REGION[int(k)]: v["lines"] for k, v in a["region_lines"].items()}
        # a region may have several markers (seg.bufs: one per inner buffer): summed per block
        blk = defaultdict(lambda: [0, 0])
        for rid, reserved, cap in a["regions"]:
            blk[REGION[rid]][0] += reserved
            blk[REGION[rid]][1] += cap
        for name, (reserved, cap) in blk.items():
            res[name]["reserved"].append(reserved)
            res[name]["capacity"].append(cap)
            res[name]["touched"].append(64 * lines.get(name, 0))
            res[name]["n"].append(a["n"])
    out = {}
    for name, d in res.items():
        # seg.bufs: one region marker per inner buffer, summed per block below
        out[name] = {
            k: {"median": sorted(v)[len(v) // 2], "max": max(v), "sum": sum(v)}
            for k, v in d.items() if k != "n"}
    return out


def capacity_per_block(rows):
    """each block's total reserved, capacity, touched over the kernel's regions"""
    tot = []
    for a in rows:
        reserved = sum(r[1] for r in a["regions"] if r[0] not in (40, 41))
        cap = sum(r[2] for r in a["regions"])
        touched = 64 * sum(v["lines"] for k, v in a["region_lines"].items()
                           if int(k) not in (0, 99))
        tot.append({"block": a["block"], "n": a["n"], "reserved": reserved,
                    "capacity": cap, "touched": touched})
    def q(k):
        v = sorted(x[k] / x["n"] for x in tot)
        return {"min": v[0], "median": v[len(v) // 2], "max": v[-1]}
    return {
        "per_n": {k: q(k) for k in ("reserved", "capacity", "touched")},
        "pooled_per_n": {k: sum(x[k] for x in tot) / sum(x["n"] for x in tot)
                         for k in ("reserved", "capacity", "touched")},
        "max_bytes": {k: max(x[k] for x in tot) for k in ("reserved", "capacity", "touched")},
    }


def lru_shares(rows, phase, region, rw):
    """the line accesses of (phase, region, rw) by LRU stack distance, as shares"""
    h = [0] * 26
    for a in rows:
        for st in a["stats"]:
            if st["phase"] == phase and st["region"] == region and st["rw"] == rw:
                for i, v in enumerate(st["reuse"]):
                    h[i] += v
    tot = sum(h)
    out = {"line_accesses": tot, "cold": h[25] / tot}
    for c, k in CACHES.items():
        out[f"within_{c}"] = sum(h[:k]) / tot
    return out


def copies(blocks):
    adm = [b for b in blocks if b.get("outcome") == "admitted"]
    nb = [0] * 25
    nc = [0] * 25
    for b in adm:
        for i in range(25):
            nb[i] += b["back_log2_bytes"][i]
            nc[i] += b["back_log2_count"][i]
    tb = sum(nb)
    nsum = sum(b["n"] for b in adm)
    cum = 0
    under = {}
    for i in range(25):
        cum += nb[i]
        under[f"back_lt_2^{i + 1}"] = round(cum / tb, 5)
    return {
        "blocks": len(adm),
        "copy_bytes": tb,
        "copy_bytes_per_output_byte": tb / nsum,
        "small_bytes_per_output_byte": sum(b["small_bytes"] for b in adm) / nsum,
        "copies": sum(nc),
        "mean_copy_len": tb / sum(nc),
        "overlapping_copies": sum(b["overlapping"] for b in adm),
        "share_of_copy_bytes_with_back_distance_below": under,
        "back_log2_bytes": nb,
        "back_log2_count": nc,
    }


def main():
    d = Path(sys.argv[1])
    cells = [json.loads(x) for x in open(d / "trace-cells.jsonl")]
    seg = [json.loads(x) for x in open(d / "trace-segmented.jsonl")]
    blocks_c = [json.loads(x) for x in open(d / "blocks-cells.jsonl")]
    blocks_s = [json.loads(x) for x in open(d / "blocks-segmented.jsonl")]
    # the drivers decoded the same bytes
    assert [b["decoded"] for b in blocks_c] == [b["decoded"] for b in blocks_s]
    adm = {a["block"]: a for a in cells if a["outcome"] == 0}
    segs = {a["block"]: a for a in seg}
    assert set(adm) <= set(segs), "a block Cells admitted that Segmented did not attempt"
    rows_c = [adm[b] for b in sorted(adm)]
    rows_s = [segs[b] for b in sorted(adm)]
    assert all(c["n"] == s["n"] for c, s in zip(rows_c, rows_s))
    nsum = sum(a["n"] for a in rows_c)
    refused = [a for a in cells if a["outcome"] != 0]
    out = {
        "corpus": "wikidata (bench/wikidata.json, 431 blocks)",
        "cells_attempts": len(cells),
        "admitted_blocks": len(rows_c),
        "admitted_n": nsum,
        "segmented_attempts": len(seg),
        "cells": per_path(rows_c, nsum),
        "segmented_same_blocks": per_path(rows_s, nsum),
        "capacity_cells": capacity(rows_c),
        "capacity_segmented": capacity(rows_s),
        "capacity_per_block_cells": capacity_per_block(rows_c),
        "capacity_per_block_segmented": capacity_per_block(rows_s),
        "copies_cells_writer": copies(blocks_c),
        "copy_source_reads_lru": {
            "note": "out-region loads in write.expand (rules' long copies) and write.respine "
                    "(the spine's), by LRU stack distance in lines over the attempt's whole "
                    "address stream; within_C: distance + 1 < C",
            "write.expand": lru_shares(rows_c, 34, 9, "r"),
            "write.respine": lru_shares(rows_c, 35, 9, "r"),
        },
        "new_pages_total": {
            "cells": {REGION[int(k)]: sum(a["region_lines"].get(k, {}).get("new_pages", 0) for a in cells)
                      for k in {k for a in cells for k in a["region_lines"]}},
            "segmented": {REGION[int(k)]: sum(a["region_lines"].get(k, {}).get("new_pages", 0) for a in seg)
                          for k in {k for a in seg for k in a["region_lines"]}},
        },
        "output_fill": {
            "admitted_blocks_writing_over_half_n_of_zeros": sum(
                1 for a in rows_c for st in a["stats"]
                if st["phase"] == 30 and st["region"] == 9 and st["bytes"] > a["n"] / 2),
            "zero_bytes_written_per_output_byte": sum(
                st["bytes"] for a in rows_c for st in a["stats"]
                if st["phase"] == 30 and st["region"] == 9) / nsum,
        },
        "refused_attempts": {
            "count": len(refused),
            "n": sum(a["n"] for a in refused),
            "distinct_lines": sum(a["distinct_lines"] for a in refused),
            "bytes_read": sum(s["bytes"] for a in refused for s in a["stats"] if s["rw"] == "r"),
            "bytes_written": sum(s["bytes"] for a in refused for s in a["stats"] if s["rw"] == "w"),
        },
        "first_block": {
            "cells": rows_c[0]["block"],
            "note": "the first admitted block allocates; its fills include the output's growth",
        },
    }
    # the steady state: the admitted blocks after the first (no growth of `out` or `buf`)
    out["cells_steady"] = {"per_output_byte": per_path(rows_c[1:], sum(a["n"] for a in rows_c[1:]))["per_output_byte"]}
    out["segmented_steady"] = {"per_output_byte": per_path(rows_s[1:], sum(a["n"] for a in rows_s[1:]))["per_output_byte"]}
    json.dump(out, sys.stdout, indent=1)
    print()


if __name__ == "__main__":
    main()
