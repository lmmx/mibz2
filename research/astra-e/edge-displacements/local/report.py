"""Reconcile native-region byte sizes with the unchanged width-16 spelling data."""
import csv
from collections import Counter
from pathlib import Path
import sys

D=Path(__file__).resolve().parent
E=D.parents[1]

def rows(path):
    with path.open() as f:
        for r in csv.DictReader(f):
            yield {k:int(v) for k,v in r.items()}

orig={r['block']:r['spell']+r['offset'] for r in rows(E/'private-cache/census.csv') if r['limit']==16}
alias={r['block']:r['maximal_order_arena']+4*r['bands']+4 for r in rows(E/'edge-spellings/subwords.csv')}
routes={r['ordinal']:r['output'] for r in rows(E.parents[1]/'docs/journal/2026-11-18-astra-d-memory/evidence/corpus-routes.csv')}
raw=list(rows(D/'census.csv'))
writer=None
for p in [16,32,64]:
    selected=[r for r in raw if r['page']==p]
    assert {r['block'] for r in selected}==set(orig)==set(alias)
    s=Counter()
    for r in selected:s.update(r)
    metrics={k:s[k] for k in ['original','residual24','global_both','global_row_bytes','code_bytes',
             'row_bytes','total_bytes','total_floor','pages','increasing_pages','adjacent','negative_steps','unit_steps',
             'raw_pages','constant_pages','unit_slope_pages','custom_slope_pages','exception_pages','exceptions','zero_bit_pages']}
    result={'page':p,'blocks':len(selected),'rows':s['m'],'coded':s['n'],**metrics}
    for kib in (192,256):
        for name,live in [
            ('full',lambda r:r['total_bytes']+14+orig[r['block']]),
            ('edge',lambda r:r['total_bytes']+14),
            ('raw_next_full',lambda r:40+r['global_row_bytes']+r['code_bytes']+14+orig[r['block']]),
            ('possible_full',lambda r:r['total_floor']+14+orig[r['block']]),
            ('subwords_same_numeric',lambda r:r['total_bytes']+14+alias[r['block']])]:
            ids=[r['block'] for r in selected if live(r)<=1024*kib]
            result[f'{name}_{kib}KiB_blocks']=len(ids)
            result[f'{name}_{kib}KiB_output']=sum(routes[i] for i in ids)
    if writer is None:
        writer=csv.DictWriter(sys.stdout,fieldnames=list(result),lineterminator='\n')
        writer.writeheader()
    writer.writerow(result)
