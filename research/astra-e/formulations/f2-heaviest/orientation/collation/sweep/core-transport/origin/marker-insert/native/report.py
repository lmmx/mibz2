"""Paired duration ratios across the same ten-block complete rounds."""
import csv
from collections import Counter,defaultdict
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,stdev
import sys


def main(path):
    rows=list(csv.DictReader(path.open()))
    durations=defaultdict(float)
    cells=set()
    positions=Counter()
    phases=['setup_ns','discover_ns','merge_ns','publication_ns','second_index_ns','second_discover_ns','second_merge_ns','second_emit_ns','total_ns']
    for x in rows:
        i,r,arm=int(x['stream']),int(x['rep']),x['arm']
        cell=(i,r,arm)
        assert cell not in cells
        cells.add(cell);positions[i,arm,int(x['position'])]+=1
        for p in phases:
            durations[r,arm,p]+=int(x[p])
    assert len(cells)==2560 and set(positions.values())=={4}
    arms=['named_quote','full_word_marker','sparse4','sparse8','sparse12','lift4','lift8','lift12']
    for arm in arms:
        values=[mean(durations[r,arm,p] for r in range(32))/1e6 for p in phases]
        print(arm,'mean_ms',dict(zip(phases,[round(x,3) for x in values])))
    comparisons=[(a,'named_quote') for a in arms[1:]]
    comparisons.extend([(f'sparse{h}',f'lift{h}') for h in (4,8,12)])
    comparisons.extend([(f'sparse{h}','full_word_marker') for h in (4,8,12)])
    for arm,ref in comparisons:
        for p in ['total_ns','second_discover_ns','second_emit_ns']:
            logs=[log(durations[r,arm,p]/durations[r,ref,p]) for r in range(32)]
            a=mean(logs);width=2.0395134463964077*stdev(logs)/sqrt(len(logs))
            print(f'{arm}/{ref} {p} {exp(a):.5f} [{exp(a-width):.5f},{exp(a+width):.5f}]')


if __name__=='__main__':
    main(Path(sys.argv[1]))
