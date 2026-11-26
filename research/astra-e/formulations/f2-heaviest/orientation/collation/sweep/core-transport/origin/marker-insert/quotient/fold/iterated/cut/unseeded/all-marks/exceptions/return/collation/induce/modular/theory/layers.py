"""Check the exact surgery identity and sharpened aggregate upper bound on frozen layers."""
from collections import defaultdict
from fractions import Fraction
from math import isqrt
from pathlib import Path
import csv
import hashlib
import json

donor=Path(__file__).resolve().parents[1]/'levels.csv'
groups=defaultdict(list)
for r in csv.DictReader(donor.open()):
    row={k:int(v) for k,v in r.items()}
    groups[row['block'],row['q']].append(row)
rows=[]
for (i,q),layers in sorted(groups.items()):
    n=layers[0]['n'];R=layers[0]['old_bands']
    ad=bd=total=raw=survivors=joins=0
    for r in layers:
        k=r['level'];M=q**k
        assert r['partial']==0 and r['final_bands']==r['residue_pieces']-r['equal']-r['self']
        assert r['old_bands']<=R*M
        assert r['residue_pieces']<=min(q*R*M,(n+M-1)//M)
        raw+=r['residue_pieces'];survivors+=r['final_bands']
        ad+=min(q*R*M,(n+M-1)//M)
        bd+=min(q*R*M,(n+q*M-1)//(q*M))
        joins+=r['equal']+r['strict']
        total+=r['equal']+r['strict']+r['self']
    assert survivors<=bd and raw<=ad and joins<=total<=n-1
    rows.append(dict(block=i,q=q,R=R,levels=len(layers),created_progressions=raw,
                     progression_bound=ad,surviving_bands=survivors,band_bound=bd,
                     all_steps=total,all_joins=joins))
print(json.dumps(dict(source_sha256=hashlib.sha256(donor.read_bytes()).hexdigest(),
                      layers=sum(map(len,groups.values())),groups=rows),indent=2))
