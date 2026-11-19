from pathlib import Path
import csv
here=Path(__file__).resolve().parent
def read(p):
    with p.open() as f:return list(csv.DictReader(f))
r=read(here/'observations.csv')
base={x['block']:x for x in read(here.parent/'observations.csv')}
geo={x['block']:x for x in read(here.parents[2]/'geometry.csv')}
assert len(r)==233
for o in r:
    b=base[o['block']]
    assert o['m']==b['m'] and o['variable_sources']==b['mixed_buckets'] and o['source_rows']==b['pulled_mass']
    assert o['old_cap_records']==geo[o['block']]['cap_records'] and o['output_runs']==b['output_runs']
    assert o['exception_q_reads']==o['nondefault_rows']
print('233 capped and Q-override publications passed; all donor RLE sizes and cap counts agree.')
print(' '.join(f'{k}={sum(int(v[k]) for v in r)}' for k in r[0] if k!='block'))
