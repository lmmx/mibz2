"""Parse every reproducible ordinary weighted BWT wire and total the acquired fragment prices."""
import csv
import hashlib
import io
import json
from pathlib import Path
import struct
import sys
import tarfile
here=Path(__file__).resolve().parent
raw={ (int(v['block']),int(v['k'])):v for v in csv.DictReader((here/'blocks.csv').open()) }
root={ (int(v['block']),int(v['k'])):v for v in csv.DictReader((here.parent/'blocks.csv').open()) }
known=json.loads((here/'inputs.json').read_text())
totals={}
archive=tarfile.open(sys.argv[1],'r:*') if len(sys.argv)>1 else None
for q in known:
    ix,k=q['block'],q['k']
    r={a:int(b) for a,b in raw[ix,k].items()}
    baseline=root[ix,k]
    assert all(r[a]==int(baseline[b])for a,b in [('n','n'),('events','rows'),('classes','alphabet'),('incoming_runs','runs')])
    assert r['piece_bound']==r['true_bands']+r['source_ranges']+r['dest_byte_ranges']-2
    assert r['incoming_runs']<=r['intersection_pieces']<=r['piece_bound']
    assert r['verified_bytes']==r['n']
    if archive:
        path=f'{ix}-{k}.col';data=archive.extractfile(path).read()
        assert hashlib.sha256(data).hexdigest()==q['column_sha256'] and len(data)==q['column_bytes']
        f=io.BytesIO(data)
        assert f.read(8)==b'EXCCOL01'
        classes,runs,mark,skip,lap=struct.unpack('<5I',f.read(20))
        assert (classes,runs,mark,skip,lap)==(r['classes'],r['incoming_runs'],r['shifted_source'],r['skip'],0)
        prefix=0;weights={};keys=set()
        for _ in range(classes):
            base,width,d,context,start,end,post=struct.unpack('<IIIIBBI',f.read(22))
            assert width>0 and d>0 and base==prefix and context<r['states'] and post<r['states']
            key=(d,context,start,end);assert key not in keys;keys.add(key)
            weights[base]=width;prefix+=width
        assert prefix==r['events']
        prior=None
        for _ in range(runs):
            base,w=struct.unpack('<II',f.read(8));assert base!=prior and w>0
            weights[base]-=w;assert weights[base]>=0;prior=base
        assert not f.read(1) and not any(weights.values())
    t=totals.setdefault(k,{})
    for key in ['states','physical_payload_ranges','events','anchored_bands','true_bands','source_ranges',
                'dest_byte_ranges','intersection_pieces','piece_bound','classes','incoming_runs','dest_lookup_comparisons',
                'key_comparisons','base_sort_comparisons','prior_induce_packets','verified_bytes','primary_is_exception']:
        t[key]=t.get(key,0)+r[key]
    t['wire_bytes']=t.get('wire_bytes',0)+q['column_bytes']
assert len(raw)==len(known)==len(root)==30
if archive: assert len(archive.getmembers())==len(known)
print(json.dumps(totals,indent=2))
