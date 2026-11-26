import argparse
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
from orient import real, small, solve

p=argparse.ArgumentParser()
p.add_argument('mode',choices=['small','real'])
p.add_argument('--tapes',type=Path,default=Path('research/astra-e/dense/target/wikidata'))
p.add_argument('--corpus',type=Path,default=Path('corpora/wikidata.bz2'))
p.add_argument('--passes',type=Path)
args=p.parse_args()
if args.mode=='small':
    print(json.dumps(small(),sort_keys=True))
else:
    traces=[]
    with tempfile.TemporaryDirectory() as tmp:
        d=Path(tmp)
        (d/'sample.bz2').symlink_to(args.corpus.resolve())
        subprocess.run(['bzip2recover','sample.bz2'],cwd=d,check=True,capture_output=True)
        for tape in sorted(args.tapes.glob('*.irec.xz')):
            m,_,mark,raw,words=real.source(tape)
            expected,crc=real.recovered(d,m['index'])
            for policy in ['ordinary','cheaper']:
                history=[]
                out,counts=solve(raw,words,mark,0,m['n'],policy,trace=history.append)
                plain=real.rle1(out)
                assert plain==expected and real.crc32_bzip2(plain)==crc
                traces.extend(dict(index=m['index'],policy=policy,**phase) for phase in history)
                print(json.dumps(dict(index=m['index'],policy=policy,crc=hex(crc),rows=m['n'],
                                      pre_rle_sha=hashlib.sha256(out).hexdigest(),**counts)),flush=True)
    with args.passes.open('x',newline='') as f:
        writer=csv.DictWriter(f,fieldnames=traces[0],lineterminator='\n')
        writer.writeheader()
        writer.writerows(traces)
