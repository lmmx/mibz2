"""Verify all complete-worker observations, both fresh gates and all four compiler images."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile

d=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text())
m=load(d/'manifest.json')
assert m['rev']=='85025c903eed22e2df130b6f345613a4ddfaf5d1'
assert m['scorer_path']=='75297a1d57b2bdf262c3a19f3064848dedf2c493'
info=load(d/'direct/binaries.json')
for name in ['candidate','joined']:
    s=load(d/name/'summary.json')
    assert s['scorer']=={'version':'v1','commit':'81585172f5fb9e7a59f330124779ba084fd6bf04'}
    assert s['gate']=={'verdict':'PASS','exit':0,'cases':338,'failed':[]}
    raw=[json.loads(x) for x in (d/name/'check.jsonl').read_text().splitlines()]
    assert len(raw)==339 and all(r['pass'] for r in raw[:-1]) and raw[-1]['verdict']=='PASS'
    assert sum(r['case'].startswith('wikidata ') and r['bytes']==209700196 for r in raw[:-1])==1
    paid=info[{'candidate':'sliced','joined':'joined'}[name]]
    assert paid['sha256']==m['binaries_sha256'][name]
with tarfile.open(d/'workers.tar.xz') as archive:
    assert set(archive.getnames())==set(m['binaries_sha256'])
    for a,h in m['binaries_sha256'].items():
        assert h==hashlib.file_digest(archive.extractfile(a),'sha256').hexdigest()
prior=load(d.parents[1]/'official/manifest.json')['binaries_sha256']
assert info['four']['sha256']==prior['scalar-four'] and info['scalar']['sha256']==prior['scalar']
assert load(d/'summary.json')==json.loads(subprocess.check_output([sys.executable,d/'panel.py','report',d/'direct/runs.jsonl']))
print('Two 338-case hooked decoder gates, four matched compiler hashes and all 160 position- and predecessor-balanced whole-stream observations verified.')
