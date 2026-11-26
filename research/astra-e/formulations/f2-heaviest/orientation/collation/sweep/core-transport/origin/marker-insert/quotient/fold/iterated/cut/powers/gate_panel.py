"""Verify the fixed quarter gate at every marked byte and on the ten original tapes."""
import argparse
from functools import partial
import json
from pathlib import Path
import subprocess
import tempfile
import check
from backward import normalize
from real import original
from solve import solve


def run(args):
    args.scratch.mkdir()
    check.solve=partial(solve,normalizer=normalize,gated=True)
    tiny=check.small(6)
    sparse=[]
    for n in (17,128,2**32+7):
        out,r=check.solve([(1,1),(0,n)],[b'a',b'bB'],n,1,10,check=n<200)
        assert out==b'B'+b'a'*9 and r['normalized_rounds']==r['rounds']==1
        sparse.append(dict(n=n,gate_graphs=r['gate_graphs'],normalized_rounds=r['normalized_rounds'],
                           normalized_lookups=r['lookups'],normal_scatter=r['normal_written_rows']))
    print(json.dumps(dict(tiny=tiny,required=sparse)),flush=True)
    with tempfile.TemporaryDirectory() as td:
        folder=Path(td)
        (folder/'sample.bz2').symlink_to(args.corpus.resolve())
        subprocess.run(['bzip2recover','sample.bz2'],cwd=folder,stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL,check=True)
        for tape in sorted(args.tapes.glob('*.irec.xz')):
            tag=tape.name.split('.')[0]
            r=original(argparse.Namespace(tape=tape,corpus=args.corpus,recovered=folder,
                       differential=True,backward=True,gated=True,trace=args.scratch/(tag+'.csv')))
            print(json.dumps(r),flush=True)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('tapes',type=Path);p.add_argument('corpus',type=Path);p.add_argument('scratch',type=Path)
    run(p.parse_args())
