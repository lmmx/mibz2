"""Exercise the admitted numerical route and the connected exact fallback for small columns."""
from collections import Counter
from itertools import groupby,product
import json
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
from audit_positions import verify_positions
from encode import serialize_parts
from decode import solve


def decode_exact(raw,words,mark,wire,plain,coords,executable):
    hist=serialize_parts(raw,words,mark,wire)
    if hist is None:
        return solve(raw,words,mark,0,sum(count for _,count in raw))[0],'power_or_multicycle'
    subprocess.run([executable,wire,plain,coords],check=True,stdout=subprocess.DEVNULL)
    return plain.read_bytes(),'numerical'


def run(executable):
    stats=Counter()
    examples={}
    with TemporaryDirectory() as td:
        root=Path(td)
        for n in range(1,8):
            for col in product(range(2),repeat=n):
                symbols=sorted(set(col))
                words=[bytes([k]) for k in symbols]
                runs=[(v,len(list(g))) for v,g in groupby(col)]
                raw=[(symbols.index(v),length) for v,length in runs]
                forward=sorted(range(n),key=col.__getitem__)
                for mark in range(n):
                    wire=root/'data'
                    wire.unlink(missing_ok=True)
                    coords=root/'offset'
                    out,route=decode_exact(raw,words,mark,wire,root/'plain',coords,executable)
                    stats[route]+=1
                    x=mark
                    expected=bytearray()
                    for _ in range(n):
                        x=forward[x]
                        expected.append(col[x])
                    assert bytes(expected)==out
                    if route=='numerical':
                        verify_positions(coords,runs,mark)
                        stats['positions_checked']+=n
                    if route=='power_or_multicycle':
                        period=1
                        x=forward[mark]
                        while x!=mark:
                            period+=1
                            x=forward[x]
                        key='primitive_with_power' if period==n else 'multiple_cycles'
                        if key not in examples and len(symbols)>1:
                            examples[key]=dict(last=col,mark=mark,period=period,output=list(out))
    assert set(examples)=={'primitive_with_power','multiple_cycles'}
    print(json.dumps(dict(counts=stats,examples=examples),indent=2))


if __name__=='__main__':
    run(Path(sys.argv[1]).resolve())
