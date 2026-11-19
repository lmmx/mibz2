"""Run the unseeded original-byte grammar and check the original libbzip2 block CRC."""
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
from decode import solve
from orient import real

tapes,corpus,passes=map(Path,sys.argv[1:])
traces=[]
with TemporaryDirectory() as tmp:
    folder=Path(tmp)
    (folder/'sample.bz2').symlink_to(corpus.resolve())
    subprocess.run(['bzip2recover','sample.bz2'],cwd=folder,check=True,capture_output=True)
    for tape in sorted(tapes.glob('*.irec.xz')):
        m,_,mark,raw,words=real.source(tape)
        history=[]
        out,counts=solve(raw,words,mark,0,m['n'],trace=history.append)
        plain=real.rle1(out)
        ref,crc=real.recovered(folder,m['index'])
        assert plain==ref and real.crc32_bzip2(plain)==crc
        traces.extend(dict(index=m['index'],**d) for d in history)
        print(json.dumps(dict(index=m['index'],mark=mark,crc=hex(crc),rows=m['n'],
                              pre_rle_sha=hashlib.sha256(out).hexdigest(),**counts)),flush=True)
with passes.open('x',newline='') as dest:
    w=csv.DictWriter(dest,fieldnames=traces[0],lineterminator='\n')
    w.writeheader()
    w.writerows(traces)
