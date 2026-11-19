"""Price observed gaps, global sorting, interchange and native named Compact in same laps."""
import csv
import json
import math
import statistics
import sys
from pathlib import Path

def main(folder):
    def read(name):
        with (folder/name).open() as inp:return list(csv.DictReader(inp))
    paid=read("paid.csv");real=read("real.csv")
    num=len(paid)//90
    assert num==36 and len(paid)==3240 and len(real)==170
    assert len({(r["ordinal"],r["round"],r["position"]) for r in paid})==len(paid)
    names={}
    data={}
    for h,layout in [(-1,2)]+[(h,layout) for h in (0,3,8,10) for layout in (0,1)]:
        name="compact" if layout==2 else f"{'observed' if layout else 'global'}_h{h}"
        names[h,layout]=name
        g=[r for r in paid if int(r["h"])==h and int(r["layout"])==layout]
        data[name]={key:[sum(float(r[key]) for r in g if int(r["round"])==i) for i in range(num)]
                    for key in ("cpu_s","wall_s","setup_cpu_s","discovery_mark_cpu_s","handoff_cpu_s","writer_cpu_s")}
        assert len(g)==360
        assert all(len([r for r in g if int(r["position"])==i])==40 for i in range(9))
    def paired(new,old,field):
        logs=[math.log(x/y) for x,y in zip(data[new][field],data[old][field])]
        m=statistics.mean(logs);d=2.030107928*statistics.stdev(logs)/math.sqrt(num)
        return [math.exp(x) for x in (m,m-d,m+d)]
    out={}
    for (h,layout),name in names.items():
        g=[r for r in real if int(r["h"])==h and int(r["layout"])==layout]
        item=dict(mean_ten_block_ms={k:1000*statistics.mean(v) for k,v in data[name].items()},
                  totals={k:sum(int(r[k]) for r in g) for k in ("sort_comparisons","history_slots","gapped_nodes","holes","posted_holes","link_reads","link_writes","lf_seeks","lf_pieces","selector_runs","logical_dict_bytes","words","root_runs","second")},
                  max_gap_capacity_bytes=max(int(r["gap_capacity_bytes"]) for r in g))
        if layout<2:
            refs=["compact",names[0,layout]]
            if layout==1:refs.extend([names[h,0],names[0,0]])
            item["paired"]={ref:{field:paired(name,ref,field) for field in ("cpu_s","wall_s")} for ref in dict.fromkeys(refs)}
        out[name]=item
    # A structural and raw semantic receipt for every attempted case, independent of timing.
    for ordinal in range(0,217,24):
        for h in (0,3,4,6,8,10,11,12):
            g=sorted((r for r in real if int(r["ordinal"])==ordinal and int(r["h"])==h),key=lambda r:r["layout"])
            assert len(g)==2
            for k in ("accepted","gapped_nodes","holes","lf_seeks","lf_pieces","selector_runs","logical_dict_bytes","words","root_runs","mark_steps","second"):
                assert g[0][k]==g[1][k],(ordinal,h,k)
            assert int(g[1]["history_slots"])==0 and g[1]["posted_holes"]==g[1]["holes"]
    (folder/"summary.json").write_text(json.dumps(out,indent=2)+"\n")
if __name__=="__main__":main(Path(sys.argv[1]))
