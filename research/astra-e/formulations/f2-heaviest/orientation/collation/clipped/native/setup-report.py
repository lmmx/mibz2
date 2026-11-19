"""Phase diagnostics and paired totals on the matched fast/full setup panel."""
from collections import defaultdict
from pathlib import Path
import csv,math,sys
from statistics import mean,stdev
from scipy.stats import t
file=Path(sys.argv[1]);sums=defaultdict(lambda:defaultdict(int));caps=defaultdict(list)
for r in csv.DictReader(file.open()):
    for f in ('first_ns','total_ns','prepare_mark_ns','discover_ns','merge_ns'):
        sums[r['arm'],f][int(r['rep'])]+=int(r[f])
    caps[r['arm']].append(int(r['reserved_with_output']))
for a,b in [('fast_quote','original'),('fast_quote','quote'),('fast_frequent','frequent'),
            ('fast_frequent','original'),('fast_fifth_or_custom','fifth_or_custom'),('fast_fifth_or_custom','original'),
            ('fast_raw_quote','original')]:
    for f in ('prepare_mark_ns','first_ns','total_ns'):
        time,base=sums[a,f],sums[b,f]
        logs=[math.log(time[k]/base[k]) for k in time];m=mean(logs);e=t.ppf(.975,len(logs)-1)*stdev(logs)/math.sqrt(len(logs))
        print(f'{a}/{b} {f} {math.exp(m):.4f} [{math.exp(m-e):.4f}, {math.exp(m+e):.4f}]')
for k,c in caps.items():print(k,'total retained bytes including output:',min(c),max(c))
