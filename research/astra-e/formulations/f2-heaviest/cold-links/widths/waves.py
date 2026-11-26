"""Match the eight-lane native discovery DAG before and after grouped pruning."""
import csv
from pathlib import Path
import struct,sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[2]/"f2d/refined"))
from refine import Column,topology,schedule

def main(caps):
    w=csv.writer(sys.stdout,lineterminator="\n")
    w.writerow(["stream","threshold","hot_nodes","ordinary_queries","ordinary_waves","hot_queries","hot_waves","hot_critical"])
    with (HERE/"real.csv").open() as f:
        references={(int(r["stream"]),int(r["threshold"])):r for r in csv.DictReader(f)}
    for path in sorted(caps.glob("*.recs"),key=lambda p:int(p.stem)):
        s=int(path.stem);c=Column([v[0] for v in struct.iter_unpack("<H",path.read_bytes())])
        paths,_=c.actual();tree=topology(c,paths)
        costs=[len(path)+1 for path,_ in paths];orig,_=schedule(tree,costs)
        for threshold in (1,2,4,8):
            keep=[i for i,(_,(_,a,b)) in enumerate(paths) if b-a>threshold]
            ids={v:i for i,v in enumerate(keep)}
            collapsed=[[ids[x] for x in tree[v] if x in ids] for v in keep]
            hot=[costs[v] for v in keep]
            assert sum(hot)==int(references[s,threshold]["first_hot_queries"])
            waves,span=schedule(collapsed,hot)
            w.writerow([s,threshold,len(keep),sum(costs),orig,sum(hot),waves,span])
if __name__=="__main__":main(Path(sys.argv[1]))
