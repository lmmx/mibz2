"""Fixed six-round clustered comparison including the source-identical null."""
import csv
from collections import defaultdict,Counter
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,stdev
import sys

ARMS=('named_a','named_b','sparse4')
COLUMNS=('setup_ns','discover_ns','merge_ns','publication_ns','second_index_ns',
         'second_discover_ns','second_merge_ns','second_emit_ns','total_ns')
EXTRA=('common_first_ns','post_root_ns','second_total_ns')


def report(src,out):
    rows=list(csv.DictReader(src.open()))
    rounds=len(rows)//30
    assert rounds in [48,144] and rounds*30==len(rows)
    assert {int(r['rep']) for r in rows}==set(range(rounds))
    cells=set()
    orders=defaultdict(list)
    totals=defaultdict(float)
    for r in rows:
        c,rep,stream,pos=map(int,[r['cluster'],r['rep'],r['stream'],r['position']])
        arm=r['arm']
        assert c==rep//6 and (rep,stream,arm) not in cells
        cells.add((rep,stream,arm))
        orders[c,stream,rep].append((pos,arm))
        vals={k:int(r[k]) for k in COLUMNS}
        vals['common_first_ns']=sum(vals[k] for k in COLUMNS[:3])
        vals['post_root_ns']=vals['total_ns']-vals['common_first_ns']
        vals['second_total_ns']=sum(vals[k] for k in COLUMNS[4:8])
        for k,v in vals.items(): totals[rep,arm,k]+=v
    positions=Counter((c,s,tuple(a for _,a in sorted(o))) for (c,s,_),o in orders.items())
    assert set(positions.values())=={1} and len(positions)==rounds*10
    clusters=rounds//6
    tcrit=2.0686576104190406 if clusters==24 else 2.3646242515927844
    with (out/'clusters.csv').open('w') as f:
        w=csv.writer(f,lineterminator='\n');w.writerow(('cluster','arm','reference','metric','cluster_mean_log_ratio','ratio'))
        for a,b in [('sparse4','named_a'),('sparse4','named_b'),('named_b','named_a')]:
            for k in COLUMNS+EXTRA:
                if k=='publication_ns':continue
                cr=[]
                for c in range(clusters):
                    vals=[log(totals[r,a,k]/totals[r,b,k]) for r in range(6*c,6*c+6)]
                    v=mean(vals);cr.append(v);w.writerow((c,a,b,k,v,exp(v)))
                av=mean(cr);hw=tcrit*stdev(cr)/sqrt(clusters)
                print(f'{a}/{b} {k} {exp(av):.5f} [{exp(av-hw):.5f},{exp(av+hw):.5f}]')
                if a=='named_b' and k=='total_ns':
                    print(f'control_only_48_projected_half_width {2.0686576*stdev(cr[:8])/sqrt(24):.8f}')
    for a in ARMS:
        print(a,'mean_ms',dict((k,round(mean(totals[r,a,k] for r in range(rounds))/1e6,3)) for k in COLUMNS+EXTRA))


if __name__=='__main__': report(Path(sys.argv[1]),Path(sys.argv[2]))
