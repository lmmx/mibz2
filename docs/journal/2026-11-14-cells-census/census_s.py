"""The Cells census's summary (rule: #145 LOG 6878326831).

    python3 census_s.py CENSUS.jsonl CALLGRIND_DIR [OUT_PHASES.jsonl]

CENSUS.jsonl is examples/cells_census.rs's records for a corpus. CALLGRIND_DIR holds one
callgrind run of the same binary on the same input (`--dump-before='*Kernel>::slots'`,
`--dump-instr=no --compress-strings=no --compress-pos=no`): one dump a Cells attempt. Each
dump's inclusive Ir of `Kernel::slots`, `Kernel::induce_cells` and `Kernel::write_cells` is
that attempt's (a dump is cut at the next attempt's entry to `slots`, and the three are
called once an attempt, in that order, on one thread).

s (serial share) of a block = (slots + induction) / (slots + induction + write_cells), over
admitted blocks; overall s weighted by the block size n, and pooled (sum over sum) too.
"""

import json
import re
import sys
from pathlib import Path

PHASES = {
    "slots": "<mibz2::cells::Kernel>::slots",
    "induce": "<mibz2::cells::Kernel>::induce_cells",
    "write": "<mibz2::cells::Kernel>::write_cells",
}


def inclusive(path: Path) -> tuple[int, dict]:
    """(the dump's part number, {phase: inclusive Ir}) of one callgrind dump."""
    part = 0
    names = {v: k for k, v in PHASES.items()}
    inc = dict.fromkeys(PHASES, 0)
    fn = None
    for line in path.read_text(errors="replace").splitlines():
        if line.startswith("part:"):
            part = int(line.split()[1])
        elif line.startswith("fn="):
            fn = names.get(line[3:].strip())
        elif line[:1].isdigit() and fn is not None:
            # a self cost line, or (after `calls=`) a call's inclusive cost: both the fn's
            inc[fn] += int(line.split()[1])
        # cfn=, cfi=, fl=, fi=, fe=, ob= ... change no function's cost
    return part, inc


def main() -> None:
    census = [json.loads(x) for x in open(sys.argv[1])]
    attempts = {r["attempt"]: r for r in census if "attempt" in r}
    dumps = sorted(
        (inclusive(p) for p in Path(sys.argv[2]).iterdir() if re.match(r"cg\.out", p.name)),
        key=lambda x: x[0],
    )
    with_slots = [d for d in dumps if d[1]["slots"] > 0]
    if len(with_slots) != len(attempts):
        sys.exit(f"{len(with_slots)} dumps with slots for {len(attempts)} attempts")
    rows = []
    for a, (part, inc) in enumerate(with_slots):
        r = attempts[a]
        row = {"attempt": a, "block": r["block"], "part": part, "n": r["n"],
               "outcome": r["outcome"], **inc}
        if r["outcome"] == "admitted":
            tot = inc["slots"] + inc["induce"] + inc["write"]
            row["s"] = (inc["slots"] + inc["induce"]) / tot
        rows.append(row)
    if len(sys.argv) > 3:
        with open(sys.argv[3], "w") as f:
            for row in rows:
                f.write(json.dumps(row) + "\n")
    adm = [x for x in rows if x["outcome"] == "admitted"]
    n_adm = sum(x["n"] for x in adm)
    tot = {k: sum(x[k] for x in adm) for k in PHASES}
    all_cells = sum(tot.values())
    out = {
        "attempts": len(rows),
        "admitted": len(adm),
        "admitted_n": n_adm,
        "ir_admitted": tot,
        "ir_refused_slots": sum(x["slots"] for x in rows if x["outcome"] != "admitted"),
        "ir_all_attempts": {k: sum(x[k] for x in rows) for k in PHASES},
        "s_weighted_n": sum(x["s"] * x["n"] for x in adm) / n_adm if adm else None,
        "s_pooled": (tot["slots"] + tot["induce"]) / all_cells if adm else None,
        "s_slots_only_pooled": tot["slots"] / all_cells if adm else None,
        "s_min": min(x["s"] for x in adm) if adm else None,
        "s_max": max(x["s"] for x in adm) if adm else None,
        "ir_per_row": {k: v / n_adm for k, v in tot.items()} if adm else None,
    }
    if adm:
        ss = sorted(x["s"] for x in adm)
        out["s_quartiles"] = [ss[len(ss) // 4], ss[len(ss) // 2], ss[3 * len(ss) // 4]]
    # the split side, from the census alone
    blocks = [r for r in census if r["outcome"] == "admitted"]
    for k in (2, 3, 4):
        ok = [r for r in blocks if r[f"k{k}"]["within_10pct"]]
        cross = sum(sum(r[f"k{k}"]["cross_span_copy_bytes"]) for r in blocks)
        later_n = sum(r["n"] - r[f"k{k}"]["cuts"][1] for r in blocks)
        out[f"k{k}"] = {
            "blocks_within_10pct": len(ok),
            "of": len(blocks),
            "n_share_within_10pct": sum(r["n"] for r in ok) / sum(r["n"] for r in blocks),
            "max_dev_median": sorted(r[f"k{k}"]["max_dev"] for r in blocks)[len(blocks) // 2],
            "cross_span_copy_bytes_share_of_later_spans": cross / later_n,
        }
    out["largest_top_factor_share_median"] = sorted(
        r["largest_top_factor"] / r["n"] for r in blocks)[len(blocks) // 2]
    out["blocks_largest_top_factor_over_0.4333n"] = sum(
        1 for r in blocks if r["largest_top_factor"] > 1.1 * r["n"] / 3)
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
