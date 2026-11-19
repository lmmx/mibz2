"""Fresh single-block processes with the same frontend and actual six treatment entry points."""
from pathlib import Path
from contextlib import ExitStack
import os, csv, json, subprocess as sp, sys, hashlib, tempfile
here=Path(__file__).resolve().parent
exe,corpus,stage=map(lambda x:Path(x).resolve(),sys.argv[1:4])
env=dict(os.environ,LD_LIBRARY_PATH=str(stage))
ldd=sp.check_output(['ldd',exe],env=env,text=True)
assert f'libgrammar.so => {stage}/libgrammar.so' in ldd
inputs=json.loads((here.parent/'powers/xor/capped/receipt.json').read_text())['inputs']
labels=['compact_stream','joint','original','scalar','lazy','opend']
donors=['stream','streamgaps','dense','scalar','lazy','opend']
(stage/'rss-manifest.json').write_text(json.dumps({'binary_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'bridge_sha256':hashlib.sha256((stage/'libgrammar.so').read_bytes()).hexdigest(),'original_rss_source_sha256':hashlib.sha256((here.parent/'powers/xor/capped/rss.rs').read_bytes()).hexdigest()},indent=2)+'\n')
with tempfile.TemporaryDirectory(dir=stage) as tmp,(stage/'fresh.csv').open('w') as f:
    root=Path(tmp);(root/'sample.bz2').symlink_to(corpus)
    sp.run(['bzip2recover','sample.bz2'],cwd=root,stdout=sp.DEVNULL,stderr=sp.DEVNULL,check=True)
    w=csv.writer(f);w.writerow(['round','block','arm','mark','rows','caps','pre_kernel_hwm_kib','post_kernel_hwm_kib'])
    for rnd in range(3):
        for src in inputs:
            b=src['block']
            for pos in range(6):
                a=(pos+rnd+b)%6
                q=sp.check_output([exe,root/f'rec{b+1:05d}sample.bz2',donors[a],stage/f'block-{b}.raw'],env=env,text=True)
                mark,n,num,before,after=map(int,q.strip().split(','))
                assert [mark,n,num]==[src['mark'],src['rows'],src['records']]
                w.writerow([rnd,b,labels[a],mark,n,num,before,after])
