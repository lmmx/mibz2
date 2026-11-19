"""Read true Ψ displacements directly from maximal L runs; cross-check Astra B's LF screen."""
import csv
from collections import Counter
from itertools import accumulate
from pathlib import Path
import subprocess,sys
pin="b174edaa17ad6dd90e5f7b7d563c95c60e3a0c8c"
ref={}
for name in ('frozen','sample'):
    raw=subprocess.check_output(['git','show',f'{pin}:docs/journal/2026-11-18-astra-b-collective/{name}.csv'],text=True)
    for row in csv.DictReader(raw.splitlines()):ref[int(row['block'])]=row
f=csv.writer(sys.stdout,lineterminator='\n');f.writerow(['tape','n','best_psi_shift','matching_rows','historical_lf_shift','historical_nonmatching','eligible_nontrivial_shifts'])
for p in sorted(Path(sys.argv[1]).glob('0*/input.txt')):
    runs=[tuple(map(int,x.split()))for x in p.read_text().splitlines()[1:]]
    n=sum(w for _,w in runs);cnt=[0]*256
    for c,w in runs:cnt[c]+=w
    cursor=[0,*accumulate(cnt)];mass=Counter();l=0
    for c,w in runs:
        mass[(l-cursor[c])%n]+=w;cursor[c]+=w;l+=w
    a,count=mass.most_common(1)[0];r=ref[int(p.parent.name)]
    assert count==n-int(r['rotation_errors']) and mass[-int(r['rotation'])%n]==count
    f.writerow([p.parent.name,n,a,count,r['rotation'],r['rotation_errors'],sum(a!=0 and 2*m>=n for a,m in mass.items())])
