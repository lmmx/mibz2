"""Matched-pool counts and width-one serial/independent work budgets."""
from collections import Counter
import csv,heapq
from pathlib import Path
HERE=Path(__file__).resolve().parent
def rows(p):
    with p.open() as f:return list(csv.DictReader(f))
data=rows(HERE/"census.csv");assert len(data)==10
native={int(r["stream"]):r for r in rows(HERE.parents[1]/"f2c/native.csv") if r["took"]=="1"}
wave={int(r["stream"]):r for r in rows(HERE.parents[1]/"f2-scheduler/native-waves.csv")}
hist=rows(HERE/"histograms.csv")
for tag,rs in (("frozen",data[:1]),("nine admissions",[r for r in data if int(r["stream"]) in native])):
    keys={r["stream"] for r in rs};print(tag)
    for r in rs:
        s=int(r["stream"])
        assert int(r["total_queries"])==int(wave[s]["lookups"])
        assert int(r["second_rows"])==int(native[s]["token_rows"])
        assert int(r["missing_sources"])==int(r["cutoff_nodes"])
        assert int(r["marked_serial_cold_queries"])==int(r["deferred_queries"])
        assert int(r["partial_barrier_rows"])==int(r["second_rows"])+int(r["cutoff_nodes"])
    for field in ("rows","token_rows","root_runs","words","singleton_types","cutoff_nodes","total_queries","retained_queries",
                   "deferred_queries","known_nonquote_bytes","cold_raw_bytes","total_dict_bytes","suppressed_edge_bytes",
                   "second_rows","partial_barrier_rows","partial_components","tainted_second_arcs","tainted_second_bytes",
                   "partial_regularity_queries","partial_fm_intersections","partial_hot_types","partial_hot_dict",
                   "partial_dangling_known_bytes","barrier_original_queries","hot_gap_zero","hot_cycle_rows"):
        print(field,sum(int(r[field]) for r in rs))
    print("standard_second_rank_lookups",sum(int(native[int(r["stream"])]["lookups"]) for r in rs))
    print("standard_second_dictionary_bytes",sum(int(native[int(r["stream"])]["dict_bytes"]) for r in rs))
    print("standard_first_lookup_waves",sum(int(wave[int(r["stream"])]["waves"]) for r in rs))
    for name in ("cold_post_cut_lookups","cold_complete_bytes","hot_gap_tokens","hot_gap_bytes","cut_known_nonquote_bytes"):
        freq=Counter()
        for t in hist:
            if t["stream"] in keys and t["measure"]==name: freq[int(t["value"])]+=int(t["frequency"])
        val=sorted(freq.elements());print(name,"p50 p95 max",val[(len(val)-1)//2],val[((len(val)-1)*95)//100],val[-1])
    sums=[0,0]
    for r in rs:
        ns=[]
        for t in hist:
            if t["stream"]==r["stream"] and t["measure"]=="cold_post_cut_lookups":
                ns.extend([int(t["value"])]*int(t["frequency"]))
        # All original cut seeds are known after the hot trie. This is a
        # favorable LPT price for a separate eager independent-tail pass.
        lanes=[0]*8
        for x in sorted(ns,reverse=True):heapq.heapreplace(lanes,lanes[0]+x)
        sums[0]+=max((sum(ns)+7)//8,max(ns));sums[1]+=max(lanes)
    print("separate_parallel_cold_pass_lower_and_lpt_waves",*sums)
