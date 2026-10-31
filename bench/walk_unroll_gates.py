"""The walk-unroll trial's structural gates (docs/journal/2026-10-31-walk-unroll-protocol.md
§4, as amended by 2026-10-31-walk-unroll/amendment-1.md) on a built binary, before timing.

    walk_unroll_gates.py BINARY [--out FILE]

Exits 1 if any gate fails. Reuses bench/walk_asm_audit.py's hot-path tracing.
"""
import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import walk_asm_audit as w  # noqa: E402

STEP = ["mov", "mov", "test", "js", "mov", "shr", "mov", "prefetcht0", "mov", "mov"]
STACK = re.compile(r"\[(rsp|rbp)\+0x[0-9a-f]+\]$|\[(rsp|rbp)\]$")


def step_cores(path):
    """Each lane-step on the round's path (from a row load to the next): whether it has
    exactly its core: one row load and one row store at the same constant stack
    displacement, one T-vector load, one `js`, one `prefetcht0`, one byte store at a base
    plus the pass's offset, and at most one base load. Returns (cores ok, exact 10-step)"""
    tl = [i for i, (_, m, o) in enumerate(path) if m == "mov" and w.TLOAD.search(o)]
    ok, exact = [], 0
    for n, i in enumerate(tl):
        start = i - 1
        end = tl[n + 1] - 1 if n + 1 < len(tl) else len(path)
        seg = path[start:end]
        mns = [m for _, m, _ in seg]
        ops = [o for _, _, o in seg]
        row = ops[0].split(",")[-1].split("PTR")[-1].strip() if mns[0] == "mov" else ""
        row_stores = sum(1 for m, o in zip(mns, ops) if m == "mov" and o.split(",")[0].strip().endswith(row) and "DWORD" in o.split(",")[0])
        # the byte store's base register, and the loads of it in this step
        bstore = [o for m, o in zip(mns, ops) if m == "mov" and w.BYTE_AT_OFFSET.match(o)]
        breg = re.match(r"BYTE PTR \[(r\w+)\+", bstore[0]).group(1) if len(bstore) == 1 else None
        base_loads = sum(1 for m, o in zip(mns, ops) if m == "mov" and o.split(",")[0].strip() == breg and "PTR" in o)
        good = (bool(STACK.search(row)) and sum(1 for m, o in zip(mns, ops) if m == "mov" and w.TLOAD.search(o)) == 1
                and mns.count("js") == 1 and mns.count("prefetcht0") == 1 and row_stores == 1
                and sum(1 for m, o in zip(mns, ops) if m == "mov" and w.BYTE_AT_OFFSET.match(o)) == 1
                and base_loads <= 1 and not any(m.startswith("call") for m in mns))
        ok.append(good)
        exact += mns[:10] == STEP and len([m for m in mns if m not in ("jmp",)]) == 10
    return ok, exact


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("binary")
    ap.add_argument("--out")
    a = ap.parse_args()
    f = w.functions(a.binary, "bwt::room_segments")
    arm = [v for k, v in f.items() if "room_segments_unrolled::<32, 1>" in k]
    shipped = [v for k, v in f.items() if "room_segments::<32, 1>" in k and "unrolled" not in k]
    assert len(arm) == 1 and len(shipped) == 1, f"instantiations found: {list(f)}"
    pa, ps = w.hot_loop(arm[0]), w.hot_loop(shipped[0])
    ca, per_a = w.census(pa)
    cores, exact = step_cores(pa)
    cs, per_s = w.census(ps)
    # the shipped walk's stack traffic counted alike: its row load and store go through a
    # pointer into the lane array (1 load, 1 store a step), its outer loop stores once a round
    ship_loads = per_s["stack_loads"] + 1.0
    ship_stores = per_s["stack_stores"] + 1.0 + 1 / 32
    g = {
        "S1_unrolled": ca["t_loads"] == 32,
        "S2_instructions": per_a["instructions"] <= 11.0,
        "S3p_memory": 2.9 <= per_a["loads"] <= 3.25 and 1.9 <= per_a["stores"] <= 2.25
        and ca["stores"] - 64 <= 8 and per_a["prefetches"] == 1.0,
        "S4_branches": per_a["conditional_branches"] <= 1.1,
        "S5p_calls_stack": ca["calls"] == 0 and per_a["stack_loads"] <= ship_loads + 0.25
        and per_a["stack_stores"] <= ship_stores + 0.25,
        "S6_step_cores": len(cores) == 32 and all(cores),
    }
    report = dict(arm=dict(totals=ca, per_lane_step=per_a, steps_with_core=sum(cores),
                           steps_exactly_10=exact),
                  shipped=dict(totals=cs, per_lane_step=per_s,
                               stack_counted_alike=dict(loads=ship_loads, stores=round(ship_stores, 4))),
                  gates=g, pass_=all(g.values()))
    print(json.dumps(report, indent=1))
    if a.out:
        Path(a.out).write_text(json.dumps(report, indent=1) + "\n")
    sys.exit(0 if all(g.values()) else 1)


if __name__ == "__main__":
    main()
