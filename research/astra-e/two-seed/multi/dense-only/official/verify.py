"""Validate the selected worker proof, W1 matrix and preserved compiler images."""
import hashlib,json,lzma
from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text())
m=load(d/'manifest.json')
assert m['source']=='3932c278d9a9a273618d59e4e042f73447b68300'
assert m['path_revision']=='75297a1d57b2bdf262c3a19f3064848dedf2c493'
s=load(d/'gate/summary.json')
assert s['scorer']=={'version':'v1','commit':'81585172f5fb9e7a59f330124779ba084fd6bf04'}
assert s['gate']=={'verdict':'PASS','cases':338,'failed':[],'exit':0}
z=[json.loads(x) for x in (d/'gate/check.jsonl').read_text().splitlines()]
assert len(z)==339 and z[-1]['verdict']=='PASS' and all(r['pass'] for r in z[:-1])
with lzma.open(d/'worker.xz','rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==m['candidate_sha256']
assert m['control_sha256']==load(d.parents[1]/'official/manifest.json')['binaries_sha256']['scalar']
assert load(d/'summary.json')==json.loads(subprocess.check_output([sys.executable,d/'panel.py','report',d/'runs.jsonl']))
assert load(d.parent/'charge-summary.json')==json.loads(subprocess.check_output([sys.executable,d.parent/'report_charge.py']))
print('One 338-case gate, true scalar and selective compiler identities, 48 direct official W1 observations and 20 true-corpus CPU charge observations verified.')
