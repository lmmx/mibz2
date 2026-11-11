"""A phrase-level cycle need not be a byte-level dependency cycle.

A known seed T[0:p] plus the copy T[p:n] = T[0:n-p] has an overlapping
source/destination and hence a self-edge in a whole-phrase dependency graph.
Nevertheless every byte's dependency decreases its position by p, and terminates
at the seed. Expansion by doubling uses only initialized source ranges.
This is a counterexample to an over-strong SCC argument, NOT a Wikidata result.
"""
import json
from pathlib import Path


def expand(seed: bytes, n: int) -> tuple[bytes, int, int]:
    if not seed or n < len(seed):
        raise ValueError('Need a nonempty seed and n >= len(seed).')
    out=bytearray(n)
    out[:len(seed)]=seed
    initialized=len(seed); calls=0; copied=0
    while initialized<n:
        length=min(initialized,n-initialized)
        # Source [0,length) is fully initialized and disjoint from destination.
        out[initialized:initialized+length]=out[:length]
        initialized+=length; calls+=1; copied+=length
    return bytes(out), calls, copied


def run():
    cases=0
    for p in range(1,33):
        seed=bytes((17*j+3)%256 for j in range(p))
        for n in range(p,257):
            got,_,_=expand(seed,n)
            assert got==(seed*((n+p-1)//p))[:n]
            cases+=1
    got,calls,copied=expand(b'a',32768)
    report={'description':'Logical counterexample; no timing or real-corpus claim.',
            'correctness_cases':cases,'example_n':len(got),'bulk_copies':calls,
            'bytes_copied_after_seed':copied,'whole_phrase_graph_has_self_edge':True,
            'byte_dependency_edges': 'i -> i-1, terminating at 0'}
    Path('copy_dependency_results.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report))

if __name__=='__main__': run()
