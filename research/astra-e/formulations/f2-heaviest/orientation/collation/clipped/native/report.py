"""Audit exact geometry and summarize paired sums of ten blocks per repetition."""
from collections import Counter,defaultdict
import csv,math
from pathlib import Path
from scipy.stats import t
from statistics import mean,stdev
import sys
D=Path(__file__).resolve().parent
def ratios(file):
    sums=defaultdict(lambda:defaultdict(int))
    for r in csv.DictReader(file.open()):
        for f in ('first_ns','total_ns'):sums[r['arm'],f][int(r['rep'])]+=int(r[f])
    for arm,f in sums:
        if arm=='original':continue
        a,b=sums[arm,f],sums['original',f]
        x=[math.log(a[i]/b[i]) for i in a];m=mean(x);d=t.ppf(.975,len(x)-1)*stdev(x)/math.sqrt(len(x))
        print(f'{arm}.{f}/original = {math.exp(m):.4f} [{math.exp(m-d):.4f}, {math.exp(m+d):.4f}], n={len(x)}')
def check():
    oracle={(r['ordinal'],r['section']):r for r in csv.DictReader((D.parent/'wikidata.csv').open())}
    rows=list(csv.DictReader((D/'ten.csv').open()))
    seen=0
    for r in rows:
        if r['section']=='raw_quote':continue
        q=oracle[r['stream'],r['section']]
        for k,key in [('words','types'),('root_runs','token_runs'),('dict_bytes','dictionary_bytes'),('mixed','partial')]:
            assert r[k]==q[key],(r,q)
        seen+=1
    assert seen==40
    print('40/40 systematic Python oracle geometries match the native contracted builder')
    data=list(csv.DictReader((D/'admissions.csv').open()))
    for arm in ['quote','raw_quote','frequent','fifth','half']:
        rows=[r for r in data if r['section']==arm];assert len(rows)==228
        print(arm,'mixed',dict(sorted(Counter(int(r['mixed']) for r in rows).items())),
              'physical splits',dict(sorted(Counter(int(r['run_splits']) for r in rows).items())),
              'second-stage successes',sum(r['second']=='true' for r in rows))
if __name__=='__main__':
    if len(sys.argv)>1:ratios(Path(sys.argv[1]))
    else:check()
