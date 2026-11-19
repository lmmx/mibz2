import csv
from itertools import groupby
from pathlib import Path
import sys
def run(p):
    rs=[{k:int(v) for k,v in r.items()} for r in csv.DictReader(p.open())]
    def key(x):return x['k'],x['threshold']
    wr=csv.writer(sys.stdout,lineterminator='\n')
    wr.writerow(('k','threshold','states','exceptions','exception_fraction','jump8_byte_fraction','intersections','overlay_steps'))
    for par,items in groupby(sorted(rs,key=key),key):
        a=list(items)
        def total(k):return sum(v[k] for v in a)
        n=total('n')
        wr.writerow((*par,total('components'),total('exceptions'),f'{total("exceptions")/n:.7f}',
                     f'{8*total("chunks8")/n:.7f}',total('fragments'),total('overlay_steps')))
if __name__=='__main__':run(Path(sys.argv[1]))
