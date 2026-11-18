"""Cross-check identities across independently produced evidence and pin three small cache oracles."""
import argparse
import array
import csv
import json
import subprocess
import tempfile
from collections import OrderedDict
from pathlib import Path

from price import COHORT, SAMPLE


def rows(path):
    with path.open() as f:
        return list(csv.DictReader(f))


def cache_tests(binary):
    # Single-cache oracle, plus a two-worker hand calculation with a one-line shared LLC.
    with tempfile.TemporaryDirectory() as tmp:
        d = Path(tmp)
        for i, events in enumerate(([1, 0x80000002, 1, 0xffffffff, 3, 0x80000001],
                                     [0xffffffff, 0x80000002])):
            with (d/str(i)).open("wb") as f:
                array.array("I", events).tofile(f)
        seq = [1, 0x80000002, 1, 3, 0x80000001]
        for cap in (1,2,3):
            store = OrderedDict()
            counts = [0,0,0,0,0]
            for event in seq:
                key, dirty = event & 0x7fffffff, event >> 31
                counts[dirty] += 1
                if key not in store:
                    counts[2+dirty] += 1
                    if len(store) == cap:
                        counts[4] += store.popitem(last=False)[1]
                store[key] = store.get(key,0) | dirty
                store.move_to_end(key)
            counts[4] += sum(store.values())
            got = list(map(int,subprocess.check_output([binary,d/'0',str(cap)],text=True).split(",")))
            assert got[1:6] == counts
        # One cold write per worker to the same virtual line under two private identities.
        got = list(map(int,subprocess.check_output([binary,"--team","1",d/'1',d/'1'],text=True).split(",")))
        assert got == [2,1,2,2,2,2,2]


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("data", type=Path)
    ap.add_argument("--cache",type=Path)
    ap.add_argument("--historical-frozen",type=Path,help="also replay the unmodified Study 7 trace counters")
    args = ap.parse_args()
    folder = Path(__file__).resolve().parents[1]
    data = args.data
    ref = {int(r["block"]):r for r in rows(folder/"2026-11-18-cells-live-reuse"/"blocks.csv")
           if r["corpus"]=="wikidata.bz2"}
    arms = {k:rows(data/(k+".csv")) for k in ("shortcut","whole")}
    inp = rows(data/"input.csv")
    assert len(ref)==len(inp)==len(arms["whole"])==len(arms["shortcut"])==233
    for i, (a,b,c) in enumerate(zip(arms["whole"],arms["shortcut"],inp)):
        g = ref[i]
        assert i == int(a["block"]) == int(b["block"]) == int(c["block"])
        assert a["index"]==b["index"]==c["index"] and a["primary"]==b["primary"]==c["primary"]
        assert a["pre_sha256"]==b["pre_sha256"] and a["raw_sha256"]==b["raw_sha256"]
        assert a["capped"]==b["capped"]==g["records"] and a["n"]==g["n"]==g["period"]
        assert c["run_front"]=="1"
        assert int(a["roots"])-int(a["nodes"])==int(a["unit_visits"])
        for x in (a,b):
            assert int(x["copy_bytes"])+int(x["literals"])==int(x["n"])
            assert int(x["copies"])+int(x["literals"])==int(x["phrases"])
        assert int(b["nodes"])==int(b["roots"])+int(b["splits"])+int(b["new_bounds"])
    assert all(ref[i]["eager"]==ref[i]["lazy"]=="1" for i in COHORT)
    for r in rows(data/"pairs.csv"):
        assert ref[int(r["block"])]["steps"]==r["original_cuts"]
    for r in rows(data/"cache.csv"):
        i = int(r["block"])
        if r["arm"] in arms:
            s = arms[r["arm"]][i]
            extra = s["wide_visits"] if r["arm"]=="whole" else s["internal_reads"]
            assert int(r["lookup_reads"])==2*int(s["phrases"])+int(extra)
    assert len(rows(data/"teams.csv"))==16 and len(rows(data/"warm.csv"))==64
    assert tuple(int(r["block"]) for r in rows(data/"screens.csv"))==SAMPLE
    if args.cache:
        cache_tests(args.cache.resolve())
    if args.historical_frozen:
        from baseline import traffic
        rec=traffic.irec.load(str(args.historical_frozen))
        archive=rows(folder/"2026-11-18-cells-endpoint-sweep"/"traffic.csv")
        for mode in ("eager","lazy"):
            result, pool=traffic.baseline(rec,mode=="lazy")
            old=next(r for r in archive if r["block"]=="0" and r["model"]==mode)
            assert pool==int(old["pool"]) and all(int(old[k])==v for k,v in result.items())
    small=json.loads((data/"small.json").read_text())
    for mode in ("split","whole"):
        assert small[mode]["totals"]["cases"]==14209 and small[mode]["capped_cases"]["cases"]==30
    print("233 paired outputs and admissions, query and construction identities, samples and receipts checked")


if __name__=="__main__":
    main()
