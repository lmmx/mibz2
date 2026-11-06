#!/usr/bin/env python3
"""Replay frozen archive counters/exposures. No live measurement or project execution."""
import argparse
import collections
import hashlib
import json
from pathlib import Path
import statistics
import tarfile

ARCHIVE = "docs/journal/2026-11-03-compact30-construction-v2-capture/c30-construction-v2-20261103T222928Z.tar.gz"
BLOB = "c0e5ddbac2e60eba0d6e3123ad8b22d7b0fce7fd"
PREFIX = "c30-construction-v2-20261103T222928Z/"


def replay(repo):
    raw = (repo / ARCHIVE).read_bytes()
    assert hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest() == BLOB
    with tarfile.open(repo / ARCHIVE) as archive:
        manifest = json.load(archive.extractfile(PREFIX + "MANIFEST.json"))
        data = archive.extractfile(PREFIX + "c30-construction.jsonl").read()
    expected = next(x for x in manifest["files"] if x["path"] == "c30-construction.jsonl")
    assert len(data) == expected["bytes"]
    assert hashlib.sha256(data).hexdigest() == expected["sha256"]
    records = [json.loads(line) for line in data.splitlines()]
    meta = next(r for r in records if r["kind"] == "meta")
    counters = {r["block"]: r for r in records if r["kind"] == "counters"}
    assert len(counters) == 434 == meta["blocks"]
    per_block = []
    for i, c in sorted(counters.items()):
        n = c["rows"]
        assert c["fin_forced"] == 0 and c["rows_bulk_raw"] == 0
        assert n == meta["block_rows"][i]
        assert n == c["rows_scalar"] + c["rows_fragment"] + c["rows_bulk"]
        assert c["run_rows"] == c["rows_fragment"] + c["rows_bulk"]
        assert c["rows_bulk"] == 30*c["complete_groups"]
        assert c["fin_fast"] + c["fin_ordinary"] == c["norm_full"] + c["norm_partial"]
        portions = sum(c[k] for k in ("fin_fast", "fin_ordinary", "fin_wide", "fin_crossing", "fin_forced"))
        # Each run closes at most one nonempty prefix; final sweep <=256.
        refill_min = max(0, portions - c["runs"] - 256)
        # A noncrossing partial successor can only be the global terminal record.
        terminal_partial_max = int(n % 30 != 0)
        eligible_min = max(0, refill_min - c["fin_crossing"] - terminal_partial_max)
        c["eligible_min"] = eligible_min
        per_block.append({"block": i, "rows": n, "portions": portions,
                          "refill_min": refill_min, "eligible_min": eligible_min,
                          "scalar_rows": c["rows_scalar"], "fast_finishes": c["fin_fast"],
                          "nonuniform_tiles": (c["rows_scalar"] - n % 16)//16})
    total = lambda field: sum(c[field] for c in counters.values())
    t = sum(c["rows"]//16 for c in counters.values())
    u = total("run_rows")//16
    s = t-u
    qmax = sum(min(c["epochs"], c["runs"]) for c in counters.values())
    emin = sum(b["eligible_min"] for b in per_block)
    p = sum(b["portions"] for b in per_block)
    assert (total("rows"), t,u,s,qmax,emin)==(207006515,12937790,10544477,2393313,3450,1084913)
    # Descriptive exposure on the existing INVALID capture only. All20 rounds;
    # completion-critical = latest absolute end, not longest worker duration.
    steps = collections.defaultdict(list)
    for r in records:
        if r["kind"] == "build" and r["regime"] == "warm" and r["mode"] == "w4" and r["arm"] == "C" and r["round"] >= 0:
            steps[r["round"], r["step"]].append(r)
    full = [rs for rs in steps.values() if len(rs)==4 and all(r["rows"]>=899000 for r in rs)]
    assert len(full)==1140
    fields=("eligible_min", "rows_scalar", "fin_fast")
    exposures={}
    for f in fields:
        assigned=[counters[r["block"]][f] for rs in full for r in rs]
        latest=[counters[max(rs,key=lambda r:r["end"])["block"]][f] for rs in full]
        longest=[counters[max(rs,key=lambda r:r["end"]-r["start"])["block"]][f] for rs in full]
        exposures[f]={"assigned_mean":statistics.mean(assigned),
                      "latest_end_mean":statistics.mean(latest),
                      "longest_duration_mean":statistics.mean(longest),
                      "latest_end_vs_assigned":statistics.mean(latest)/statistics.mean(assigned)}
    result={"scope":"structural replay and descriptive exposure; capture remains INVALID; no timing inference",
            "archive_git_blob":BLOB,"archive_sha256":hashlib.sha256(raw).hexdigest(),
            "jsonl_sha256":expected["sha256"],"archive_source_commit":manifest["source"]["commit"],
            "blocks":len(counters),"rows":total("rows"),"large_blocks":sum(c["rows"]>=899000 for c in counters.values()),
            "full_tiles":t,"uniform_tiles":u,"nonuniform_tiles":s,
            "tail_rows":sum(c["rows"]%16 for c in counters.values()),"Q_max":qmax,
            "nonempty_refill_min":sum(b["refill_min"] for b in per_block),
            "fast_full_min":sum(max(0,c["fin_fast"]-c["norm_partial"]) for c in counters.values()),
            "fast_full_max":sum(min(c["fin_fast"],c["norm_full"]) for c in counters.values()),
            "scanner_gross_interval":[5*t+10*s,5*t+10*s+3*qmax],
            "refill_eligible_min":emin,
            "refill_eligible_min_large_blocks":sum(b["eligible_min"] for b in per_block if b["rows"]>=899000),
            "refill_eligible_positive_blocks":sum(b["eligible_min"]>0 for b in per_block),
            "refill_conditional_net_before_unbounded_H":27*emin-5*p,
            "refill_transient_store_bytes_before_shifted_costs":24*emin,
            "archived_all_full_steps":len(full),"exposures":exposures,"per_block":per_block}
    return result


if __name__ == "__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo",type=Path,required=True)
    parser.add_argument("--output",type=Path)
    args=parser.parse_args()
    text=json.dumps(replay(args.repo),indent=2,sort_keys=True)+"\n"
    if args.output: args.output.write_text(text)
    else: print(text,end="")
