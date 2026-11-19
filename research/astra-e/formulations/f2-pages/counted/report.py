"""Check balanced five-arm rounds and report the complete and internally timed phases."""
import csv
from collections import Counter,defaultdict
from pathlib import Path
from statistics import median
import sys
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[4]/'bench/moonshot'))
from moonshot import paired_ratio

arms=['split64','seq_split64','seq44','counted','joint']
out=csv.writer(sys.stdout,lineterminator='\n')
out.writerow(['file','metric','arm','reference','ratio','ci_lo','ci_hi','sum_block_medians_ms'])
for path in map(Path,sys.argv[1:]):
    rows=list(csv.DictReader(path.open()))
    assert len(rows)==9*40*5 and len({(r['block'],r['arm'],r['round']) for r in rows})==len(rows)
    assert set(Counter((r['block'],r['arm'],r['position']) for r in rows).values())=={8}
    for metric in ['prep_mark_ns','discover_ns','merge_ns','residual_output_ns','inverse_cpu_ns','wall_ns']:
        laps=defaultdict(lambda:[0]*40)
        blocks=defaultdict(list)
        for r in rows:
            t=int(r[metric])
            laps[r['arm']][int(r['round'])-1]+=t
            blocks[r['arm'],r['block']].append(t)
        for ref in ['split64','counted','seq_split64']:
            for a in arms:
                ratio=paired_ratio(laps[a],laps[ref])
                typical=sum(median(samples) for (arm,b),samples in blocks.items() if arm==a)
                out.writerow([path.name,metric,a,ref,*[f'{ratio[k]:.5f}' for k in ['geomean','lo','hi']], f'{typical/1e6:.5f}'])
