"""Reconcile full gates, scorer versions, frozen binaries and all paired timing rows."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile

d=Path(__file__).resolve().parent
m=json.loads((d/'manifest.json').read_text())
version={'version':'v1','commit':'81585172f5fb9e7a59f330124779ba084fd6bf04'}
def git(*args):return subprocess.check_output(['git','-C',d.parents[3],*args])
assert m['scorer_path']==git('log','-1','--format=%H',m['origin_master'],'--','bench/moonshot').decode().strip()
assert m['scorer_blobs']=={x.split(maxsplit=3)[3]:x.split()[2] for x in git('ls-tree','-r',m['origin_master'],'--','bench/moonshot').decode().splitlines()}
for arm in ['candidate','control','score']:
    s=json.loads((d/arm/'summary.json').read_text())
    assert s['scorer']==version and s['gate']=={'verdict':'PASS','cases':338,'failed':[],'exit':0}
    c=[json.loads(v) for v in (d/arm/'check.jsonl').read_text().splitlines()]
    assert len(c)==339 and c[-1]=={'arm':'candidate','verdict':'PASS','failures':0}
    assert all(r['pass'] for r in c[:-1])
    assert len([r for r in c[:-1] if r['bytes']==209700196 and r['case'].startswith('wikidata ')])==1

with tarfile.open(d/'workers.tar.xz') as f:
    assert set(f.getnames())==set(m['worker_sha256'])
    for arm,digest in m['worker_sha256'].items():
        assert hashlib.file_digest(f.extractfile(arm),'sha256').hexdigest()==digest

raw=[json.loads(x) for x in (d/'score/runs.jsonl').read_text().splitlines()]
assert len(raw)==54
assert {(r['round'],r['workers'],r['arm']) for r in raw}=={(k,w,a) for k in range(9) for w in [1,4] for a in ['lazy','opend','candidate']}
assert all(r['out_bytes']==419400392 for r in raw)
rebuilt=subprocess.check_output([sys.executable,d/'direct.py','report',d/'direct.jsonl'])
assert json.loads(rebuilt)==json.loads((d/'direct-summary.json').read_text())
print('Three 338-case correctness runs, two compiled SHA-256s, refreshed scorer tree, 54 official and 80 direct full-worker runs verified.')
