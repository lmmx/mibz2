import csv
from pathlib import Path
import sys
def report(path):
    with path.open() as src:
        rows=[{k:int(v) for k,v in r.items()} for r in csv.DictReader(src)]
    out=csv.writer(sys.stdout,lineterminator='\n')
    out.writerow(('k','rows','contexts','exception_events','exception_fraction','coordinate_visits','visits_per_row','eight_byte_coverage','logical_wire_bytes'))
    for k in sorted({r['k'] for r in rows}):
        vs=[r for r in rows if r['k']==k]
        def total(name):return sum(r[name] for r in vs)
        n,c,e,q=map(total,['n','states','exceptions','coordinate_visits'])
        out.writerow((k,n,c,e,f'{e/n:.7f}',q,f'{q/n:.7f}',f'{8*total("chunk8")/n:.7f}',24*len(vs)+5*c+9*e))
if __name__=='__main__':report(Path(sys.argv[1]))
