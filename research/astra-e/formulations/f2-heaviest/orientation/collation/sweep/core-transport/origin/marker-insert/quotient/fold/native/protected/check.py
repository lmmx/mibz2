"""Recompute the anchor-conditioned optimum from the independent original type table."""
import csv,gzip,sys
from itertools import groupby
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]))
from fold import independent

def read(path):
    f=gzip.open(path,'rt') if path.suffix=='.gz' else path.open()
    with f: return list(csv.DictReader(f))

def check(path, reference):
    native=read(path)
    old=read(reference)
    assert len(native)==len(old)==35117
    actual={int(s):list(rr) for s,rr in groupby(native,key=lambda r:r['stream'])}
    print('stream,old_types,old_anchor_base,old_anchor_mass,eligible,selected,removed,old_rows,new_rows')
    for s,raw in groupby(old,key=lambda r:int(r['stream'])):
        rr=list(raw);freq={int(r['base']):int(r['mass']) for r in rr}
        d=max(freq,key=lambda k:(freq[k],-k))
        g={int(r['base']):int(r['succ']) for r in rr
           if int(r['succ'])!=4294967295 and int(r['base'])!=d and int(r['succ'])!=d}
        chosen=independent(g,freq)
        for r in actual[s]:
            k=int(r['base']);assert int(r['mass'])==freq[k]
            assert int(r['succ'])==g.get(k,4294967295)
            assert (r['canonical']=='1')==(k in chosen)
        mass=sum(freq[k] for k in chosen);m=sum(freq.values())
        print(f'{s},{len(rr)},{d},{freq[d]},{len(g)},{len(chosen)},{mass},{m},{m-mass}')
if __name__=='__main__':check(Path(sys.argv[1]),Path(sys.argv[2]))
