"""Summarize the native paired observations and recorded array capacities."""
import csv
import json
import math
from pathlib import Path
import statistics

HERE=Path(__file__).resolve().parent


def load(name):
    with (HERE/name).open() as f:
        return list(csv.DictReader(f))


def interval(ratios):
    logs=list(map(math.log,ratios))
    assert len(logs)==16
    center=statistics.mean(logs)
    half=2.13144954556*statistics.stdev(logs)/math.sqrt(len(logs))
    return dict(n=len(logs),ratio=math.exp(center),lo=math.exp(center-half),hi=math.exp(center+half))


def main():
    base,indexed,times=(load(f) for f in ("real.csv","indexed.csv","price.csv"))
    assert (len(base),len(indexed),len(times))==(20,20,1280)
    out={}
    for section in ("quote","frequent"):
        br=[r for r in base if r["section"]==section]; ir=[r for r in indexed if r["section"]==section]
        tr=[r for r in times if r["section"]==section]
        totals={k:sum(int(r[k]) for r in br) for k in (
            "groups","backward_nodes","packet_inputs","select_translations","backward_original_run_intersections",
            "backward_child_intervals","select_binary_searches","select_binary_comparisons","direct_hops","covered_rows","source_rows",
            "live_merges","bands_count","word_count","spelling_bytes","mark_seeks","output_lookups","sort_comparisons")}
        indexed_totals={k:sum(int(r[k]) for r in ir) for k in (
            "select_page_seeks","select_page_run_increments","source_directory_fills","sort_comparisons")}
        sizes={}
        for name,rows in (("binary_sorted",br),("paged_scatter",ir)):
            by_block={r["ordinal"]:sum(int(r.get(k,0)) for k in (
                "peak_frontier_capacity_bytes","trie_capacity_bytes","band_capacity_bytes","select_capacity_bytes",
                "word_index_capacity_bytes","dictionary_capacity_bytes","section_capacity_bytes","page_capacity_bytes",
                "source_directory_capacity_bytes")) for r in rows}
            sizes[name]={"retained_capacity_at_last_construction_phase":by_block,
                         "largest":max(by_block.values()),
                         "largest_page_directory":max(int(r.get("page_capacity_bytes",0)) for r in rows),
                         "largest_source_directory":max(int(r.get("source_directory_capacity_bytes",0)) for r in rows)}
        arms={}
        for i,name in enumerate(("binary_sorted","pages_sorted","binary_scatter","pages_scatter")):
            arm={"phase_median_per_block_ms_summed":{}}
            for field in ("cpu_ms","wall_ms","setup_ms","discover_ms","order_ms","dictionary_ms","emit_ms"):
                by_round=[[sum(float(r[field]) for r in tr if int(r["arm"])==a and int(r["round"])==lap)
                           for lap in range(16)] for a in (i,0)]
                if field in ("cpu_ms","wall_ms"): arm[field+"_over_control"]=interval(x/y for x,y in zip(*by_round))
                arm["phase_median_per_block_ms_summed"][field]=sum(
                    statistics.median(float(r[field]) for r in tr if int(r["arm"])==i and r["ordinal"]==o)
                    for o in {r["ordinal"] for r in tr})
            arms[name]=arm
        out[section]={"ten_tape_totals":totals,"indexed_constructor_totals":indexed_totals,
                      "capacity_bytes":sizes,"paired":arms}
    print(json.dumps(out,indent=2))


if __name__=="__main__":
    main()
