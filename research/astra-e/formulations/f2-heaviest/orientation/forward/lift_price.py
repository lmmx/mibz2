"""Native three-arm crossover and exact hull-translation accounting."""
import argparse
import csv
import json
import math
from pathlib import Path
import statistics
import subprocess
from price import HEADER
from census import Column,heavy_set,paths,read_input

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("constructor",type=Path);p.add_argument("timer",type=Path);p.add_argument("tapes",type=Path);p.add_argument("out",type=Path)
    a=p.parse_args();structural=[]
    with (Path(__file__).parent/"hull.csv").open() as f:
        prediction={(r["input"],r["section"]):r for r in csv.DictReader(f)}
    with (a.out/"lifted-price.csv").open("w") as f:
        writer=csv.writer(f,lineterminator="\n");writer.writerow(("ordinal","section",*HEADER))
        for path in paths(a.tapes):
            ordinal=str(int(path.name.split(".")[0]));runs,mark=read_input(path);col=Column(runs)
            parent=col.interval(b'"')
            for section,ints in (("quote",[parent]),("frequent",heavy_set(col,parent,1)[0])):
                data=(f"{col.rows} {mark} {len(runs)} {len(ints)}\n"+"".join(f"{x} {y}\n" for x,y in [*runs,*ints])).encode()
                baseline,lift=(json.loads(subprocess.check_output([a.constructor.resolve(),mode],input=data))
                               for mode in ("RECYCLED_BANDS","HULL_BANDS"))
                for k in ("bands","expanded_hex","source_rows","covered_rows","groups","max_depth","trie_nodes","word_count","spelling_bytes"):
                    assert baseline[k]==lift[k],(ordinal,section,k)
                expected=prediction[ordinal,section]
                assert lift["lifted_groups"]==int(expected["hull_translates"])
                assert lift["lifted_packets"]-lift["lifted_groups"]==int(expected["hull_input_savings"])
                assert baseline["packet_inputs"]-lift["packet_inputs"]==lift["lifted_packets"]
                assert lift["hull_seeks"]==lift["lifted_groups"]+lift["reused_first"]
                assert lift["hull_seeks"]+lift["select_page_seeks"]==baseline["select_page_seeks"]
                assert lift["hull_increments"]+lift["select_page_run_increments"]==baseline["select_page_run_increments"]
                structural.append(dict(ordinal=ordinal,section=section,ordinary_packets=baseline["packet_inputs"],
                    ordinary_translations=baseline["select_translations"],explicit_packets=lift["packet_inputs"],
                    explicit_translations=lift["select_translations"],deferred_groups=lift["lifted_groups"],
                    logical_deferred_packets=lift["lifted_packets"],hull_seeks=lift["hull_seeks"],
                    rejected_hulls_reusing_first=lift["reused_first"],hull_increments=lift["hull_increments"],
                    remaining_page_seeks=lift["select_page_seeks"],remaining_page_increments=lift["select_page_run_increments"]))
                rows=subprocess.check_output([a.timer.resolve(),"24","TRIPLE"],input=data).decode().splitlines()
                assert len(rows)==72
                for line in rows:
                    fields=line.split(",");r,pos,arm=map(int,fields[:3]);expected=(pos+r+1)%3
                    assert arm==(2-expected if r//3%2 else expected)
                    writer.writerow((ordinal,section,*fields))
                f.flush()
    with (a.out/"lifted-verified.csv").open("w") as f:
        w=csv.DictWriter(f,fieldnames=structural[0],lineterminator="\n");w.writeheader();w.writerows(structural)
    with (a.out/"lifted-price.csv").open() as f: rows=list(csv.DictReader(f))
    results={}
    for section in ("quote","frequent"):
        g=[r for r in rows if r["section"]==section];out={}
        for new,ref,label in ((1,0,"recycled_over_direct"),(2,0,"lifted_over_direct"),(2,1,"lifted_over_recycled")):
            d={}
            for metric,fields in (("cpu",("cpu_ms",)),("wall",("wall_ms",)),("discover_spell_order",("discover_ms","dictionary_ms","order_ms")),("emit",("emit_ms",))):
                logs=[]
                for i in range(24):
                    value=[sum(sum(float(r[k]) for k in fields) for r in g if int(r["round"])==i and int(r["arm"])==a) for a in (new,ref)]
                    logs.append(math.log(value[0]/value[1]))
                m=statistics.mean(logs);h=2.06865761042*statistics.stdev(logs)/math.sqrt(24)
                d[metric]=dict(n=24,ratio=math.exp(m),lo=math.exp(m-h),hi=math.exp(m+h))
            out[label]=d
        results[section]=out
    (a.out/"lifted-price.json").write_text(json.dumps(results,indent=2)+"\n")

if __name__=="__main__":
    main()
