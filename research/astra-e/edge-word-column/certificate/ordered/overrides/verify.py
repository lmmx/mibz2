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

live=read(here/'live.csv')
assert len(live)==len(r)
for stream,old in zip(live,r):
    for a,b in [('block','block'),('nondefault_rows','nondefault_rows'),('exception_q_reads','exception_q_reads'),
                ('heap_comparisons','exception_heap_comparisons'),('heap_chunks','sorted_exception_fragments'),
                ('outer_claims','staged_outer_claims'),('cap_records','old_cap_records'),('output_runs','output_runs')]:
        assert stream[a]==old[b]
print('Live peaks across individual blocks:',
      max(int(v['peak_open_buckets']) for v in live),'simultaneous buckets;',
      max(int(v['peak_live_heads']) for v in live),'live queue heads.')
print('Live inner claims',sum(int(v['inner_claims']) for v in live))
