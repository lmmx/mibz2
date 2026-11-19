"""Matched native first-normalization price with a complete dense byte inverse control."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from real import source
from grammar import Node
from cut import seed,geometry,overlay
from normalize import normalize


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args):
    here=Path(__file__).resolve().parent
    temp=args.scratch
    temp.mkdir()
    meta,runs,mark,root,letters=source(args.tape)
    raw=temp/'input.txt'
    raw.write_text(str(mark)+'\n'+''.join(f'{k} {n}\n' for k,n in runs))
    binary=temp/'phase'
    compile_args=['g++','-std=c++20','-O3','-g0','-Wall','-Wextra',str(here/'native.cpp'),'-o',str(binary)]
    subprocess.run(compile_args,check=True)
    subprocess.run([binary,raw,'check',temp],check=True)
    fresh,weights,a=seed(root,[Node(literal=w) for w in letters],mark)
    _,bounds=geometry(fresh,len(weights))
    slices,_=overlay(fresh,bounds)
    normalized,new,na,c=normalize(fresh,weights,a,bounds,slices,check=False)
    expected=''.join(f'{k} {n}\n' for k,n in normalized)
    assert (temp/'native.rle').read_text()==expected
    expected_words=[str(na)]
    for x in new:
        expected_words.append(f'{(x.left if x.literal is None else x).literal[0]} {x.n}')
    assert (temp/'native.words').read_text()=='\n'.join(expected_words)+'\n'
    # The complete byte inverse control is compared with the independent
    # marked byte reference accepted by the Python full-grammar experiment.
    data=json.loads(args.accepted.read_text())
    assert digest(temp/'direct.bin')==data['original_inverse_sha256']
    for arm in ('sparse','dense','interval','direct'):
        with (temp/(arm+'-rss.csv')).open('w') as f:
            subprocess.run([binary,raw,arm,'1'],stdout=f,check=True)
    with (temp/'phase.csv').open('w') as f:
        subprocess.run([binary,raw,'panel','24'],stdout=f,check=True)
    print(json.dumps(dict(compiler=subprocess.check_output(['g++','--version'],text=True).splitlines()[0],
        flags=compile_args[1:-3],cpp_sha256=digest(here/'native.cpp'),interval_header_sha256=digest(here/'interval.hpp'),binary_sha256=digest(binary),
        input_sha256=digest(raw),source_sha256=digest(args.tape),root_rows=meta['n'],root_runs=len(runs),
        original_mark=mark,normalized_rle_sha256=digest(temp/'native.rle'),
        normalized_words_sha256=digest(temp/'native.words'),direct_sha256=digest(temp/'direct.bin'),
        costs=c),indent=2))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('tape',type=Path)
    p.add_argument('accepted',type=Path)
    p.add_argument('scratch',type=Path)
    run(p.parse_args())
