"""Usage: python replay.py FROZEN_CHECKOUT. No timed execution.

Regenerates synthetics from checked blobs; Wikidata physical bytes are not rechecked.
For a full physical-file check run the pinned verifier with --wikidata separately.
"""
import sys, json, hashlib, importlib.util
from pathlib import Path
root=Path(sys.argv[1]).resolve();here=Path(__file__).resolve().parent
for f in json.loads((here/'source-blobs.json').read_text()):
    b=(root/f['path']).read_bytes()
    assert hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==f['sha'],f['path']
spec=importlib.util.spec_from_file_location('pinned',root/'bench/run_fed_stage.py')
v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
r=v.load(here/'artifact/run-fed-1.json')
expected=[v.wikidata_expected(root/'bench/wikidata.json')]+v.synthetic_expected(root)
problems=v.check(r,'aa409f6f2ebce9f41dc7902789f0ab8d6958b020',expected)
assert not problems,problems
print(json.dumps({'problems':problems,'physical_wikidata_checked':False,'readings':v.readings(r)},indent=2))
