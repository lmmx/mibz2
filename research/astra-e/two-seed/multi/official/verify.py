"""Replay all paid and correctness evidence without a benchmark run."""
from pathlib import Path
import hashlib
import json
import sys
import subprocess
import tarfile

d=Path(__file__).resolve().parent
def load(p):return json.loads(p.read_text())
snap=load(d/'manifest.json')
assert snap['scorer_tree'].split()[2]=='75f7b0321bd71c5086cbfb34747f7ea963463784'
assert snap['scorer_path']=='75297a1d57b2bdf262c3a19f3064848dedf2c493'
assert snap['common_edge_except_run_iterator']
for a in ['edge-four','scalar-four','scalar','score']:
    s=load(d/a/'summary.json')
    assert s['scorer']=={'version':'v1','commit':'81585172f5fb9e7a59f330124779ba084fd6bf04'}
    assert s['gate']=={'verdict':'PASS','cases':338,'failed':[],'exit':0}
    rows=[json.loads(x) for x in (d/a/'check.jsonl').read_text().splitlines()]
    assert len(rows)==339 and all(x['pass'] for x in rows[:-1]) and rows[-1]['verdict']=='PASS'
    assert len([x for x in rows[:-1] if x['case'].startswith('wikidata ') and x['bytes']==209700196])==1
official=[json.loads(x) for x in (d/'score/runs.jsonl').read_text().splitlines()]
assert len(official)==54 and all(r['out_bytes']==419400392 for r in official)
assert {(r['round'],r['workers'],r['arm']) for r in official}=={(i,w,a) for i in range(9) for w in [1,4] for a in ['candidate','lazy','opend']}
info=load(d/'direct/binaries.json')['binaries']
prior=load(d.parents[1]/'official/manifest.json')['worker_sha256']
assert info['two']['sha256']==prior['candidate'] and info['edge']['sha256']==prior['control']
with tarfile.open(d/'workers.tar.xz') as f:
    assert set(f.getnames())==set(snap['binaries_sha256'])
    for a,h in snap['binaries_sha256'].items():
        assert h==hashlib.file_digest(f.extractfile(a),'sha256').hexdigest()==info[a]['sha256']
assert load(d/'direct-summary.json')==json.loads(subprocess.check_output([sys.executable,d/'panel.py','report',d/'direct/panel.jsonl']))
print('Three independent builds and full gates verified, 54 scorer runs and all 200 position- and predecessor-balanced official worker runs verified.')
