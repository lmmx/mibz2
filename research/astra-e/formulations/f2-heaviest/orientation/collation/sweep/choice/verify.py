"""Check the claimed integer optimum from the archived primal and rational dual without an optimizer."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import numpy as np

from optimize import Choice, Column, Oracle, follower_partition, heavy_set, here, objects
sys.path.insert(0,str(here.parents[7]/'docs/journal/2026-11-18-astra-c-output-first'))
from inputs import read_input


def verify(tape,record):
    receipt=json.loads(record.read_text())
    assert hashlib.sha256(tape.read_bytes()).hexdigest()==receipt['sha256']
    runs,_=read_input(tape)
    col=Column(runs)
    oracle=Oracle(b''.join(bytes([c])*n for c,n in runs))
    a,b=receipt['section']
    named=col.interval(b'"')
    assert (a,b) in (named,tuple(heavy_set(col,named,1)[0][0]))
    types,source,returns=objects(oracle,(a,b))
    groups,after=follower_partition(types,source,returns)
    model=Choice(types,groups,after)
    bits=list(map(int,receipt['class_bits']))
    assert len(bits)==model.n and set(bits)<={0,1}
    actual=model.exact(bits)
    assert actual==(receipt['Q'],receipt['D'])
    assert model.evaluate(bits)==actual[0]
    objective,arr,rhs=model.inequalities()
    weights=np.zeros(len(rhs),dtype=np.int64)
    for i,w in receipt['multipliers']:
        assert 0<=i<len(weights) and weights[i]==0 and w>0
        weights[i]=w
    unit=receipt['certificate_scale']
    lower=model.exact_lower(objective,arr,rhs,weights,unit)
    assert lower==receipt['certificate_lower_numerator']
    result=-(-lower//unit)
    assert result==receipt['certified_integer_lower']<=actual[0]
    return dict(tape=tape.name,section=[a,b],upper=actual[0],lower=result,exact=result==actual[0])


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('tape',type=Path)
    p.add_argument('record',type=Path)
    args=p.parse_args()
    print(json.dumps(verify(args.tape,args.record)))
