"""Cross-check construction counts with independent full ordinary and Q certificate routes."""
import csv
from pathlib import Path
from collections import Counter
here=Path(__file__).resolve().parent
def rows(path):
    with path.open() as f:
        return list(csv.DictReader(f))
full={r['block']:r for r in rows(here.parent.parent/'cuts.csv') if r['model']=='physical' and r['depth']=='0'}
cert={r['block']:r for r in rows(here.parent/'choose.csv')}
ours=rows(here/'observations.csv')
assert len(ours)==len(full)==len(cert)==233
for r in ours:
    a,b=full[r['block']],cert[r['block']]
    for dest,source in [('m','old_rows'),('seeded_classes','old_classes'),('deleted','initial_ingress'),('aliases','new_classes'),('output_runs','new_runs')]:
        assert r[dest]==a[source]
    for dest,source in [('m','m'),('bands','bands'),('cert_frags','ordered_fragments'),('cert_queue_reads','queue_reads')]:
        assert r[dest]==b[source]
print('All 233 first-cut roots match the saved donor and certificate counts.')
print(' '.join(f'{k}={sum(int(r[k]) for r in ours)}' for k in ours[0] if k!='block'))
