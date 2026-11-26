from pathlib import Path
from statistics import median
import csv,json,sys
p=Path(sys.argv[1]);raw=list(csv.DictReader((p/'heldout.csv').open()));manifest=list(csv.DictReader((p/'manifest.csv').open()))
held={int(x['admission']) for x in manifest if x['held_out']=='1'}
assert len(held)==219 and len(raw)==438
out={}
for h in (4,12):
    rr=[r for r in raw if int(r['h'])==h]
    assert len(rr)==219 and {int(x['stream']) for x in rr}==held
    share=[]
    for x in rr:
        base,lit,other=(int(x[k]) for k in ('root_only_records','literal_records','all_branch_records'))
        assert lit==int(x['actual_records'])
        if base==other:
            share.append(1 if lit==base else 0)
        else:share.append((base-lit)/(base-other))
    n=sum(x>=.5 for x in share)
    out[h]=dict(blocks=len(rr),fixed_present=sum(x['literal_present']=='true' for x in rr),contained=sum(x['literal_in_root']=='true' for x in rr),
                median_fraction=median(share),half_count=n,min_fraction=min(share),passes=median(share)>=.7 and 4*n>=3*len(rr))
print(json.dumps(out,indent=2))
