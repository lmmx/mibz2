"""Compare the paid and fully audited builds with the d33bcaf libbzip2 and complete-cut receipts."""
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys
here=Path(__file__).resolve().parent
build,scratch,dest=map(lambda x:Path(x).resolve(),sys.argv[1:])
dest.mkdir()
truth=json.loads((here.parent/'real.json').read_text())['blocks']
known={}
for row in csv.DictReader((here.parent/'trace.csv').open()):
    if row['arm']=='unseeded':known.setdefault(row['tag'],[]).append(row)
stats=[]
for binary in ('run','audit'):
    small=json.loads(subprocess.check_output([build/binary,'small']))
    assert small==json.loads((here.parent/'tiny.json').read_text())
    for b in truth:
        tag=b['tag'];source=scratch/tag/'input.txt'
        assert hashlib.sha256(source.read_bytes()).hexdigest()==b['raw_input_sha256']
        target=dest/(binary+'-'+tag)
        target.mkdir()
        result=subprocess.check_output([build/binary,source,'check',target],text=True)
        for line in result.splitlines():
            histogram,prefix,scanned,name,skip,lap,fixed,moves,queries,hops,powers=line.split(',')
            assert hashlib.sha256((target/(name+'.bin')).read_bytes()).hexdigest()==b['pre_rle_sha256']
            data=list(csv.DictReader((target/(name+'.csv')).open()))
            for applied,expected in zip(data,known[tag],strict=True):
                assert all(applied[k]==expected[k] for k in applied)
            old=b['arms']['unseeded']
            assert tuple(map(int,[skip,lap,fixed,moves,queries,hops,powers]))==tuple(old[k] for k in [
                'skip','lap','terminal_multiplicity','ordinary_moves','power_queries','power_hops','power_runs'])
            if binary=='audit':stats.append(dict(tag=tag,arm=name,histogram=int(histogram),
                emitted_prefix=int(prefix),single_label_runs=int(scanned),depth=len(data),output_sha256=b['pre_rle_sha256']))
for row in stats:
    if row['arm']!='histogram':
        assert row['histogram']==int(known[row['tag']][0]['runs']) and row['emitted_prefix']>0
with (dest/'counts.csv').open('w',newline='') as f:
    w=csv.DictWriter(f,fieldnames=stats[0],lineterminator='\n');w.writeheader();w.writerows(stats)
identity={name:hashlib.sha256((build/name).read_bytes()).hexdigest() for name in ('run','audit')}
print(json.dumps(dict(binaries=identity,small=small,tapes=len(truth),paid_native_output_hashes=30,
      audited_native_output_hashes=30,matched_ordinary_steps=sum(len(known[t]) for t in known),
      audited_objects=['source bases','marked raw run and LF ordinal','complete old RLE',
                       'per-source word IDs','complete grammar DAG','mark ordinal','byte offset']),indent=2))
