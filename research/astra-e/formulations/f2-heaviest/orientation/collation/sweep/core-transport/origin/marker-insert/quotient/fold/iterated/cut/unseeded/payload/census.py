"""Exact sharing oracles on the unchanged ordered cut schedule."""
from collections import Counter
from pathlib import Path
from unittest.mock import patch
import argparse
import hashlib
import json
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
import decode
from orient import real
from grammar import Node,output


class Pool:
    def __init__(self,limit):
        self.limit=limit
        self.by_pair={}
        self.by_text={}
        self.texts={}
        self.stats=Counter()
        pool=self

        class Handle(Node):
            def __init__(self,**kwargs):
                super().__init__(**kwargs)
                if self.literal is not None:
                    pool.stats['literal_nodes']+=1
                    if pool.collect(self.n):
                        pool.texts[self]=self.literal
                        pool.by_text[self.literal]=self

            def __add__(self,b):
                return self if b==b'' else pool.request(self,b,None)

            def power(self,d):
                return self if d==1 else pool.request(self,None,d)
        self.handle=Handle

    def collect(self,n):
        return self.limit is None or n<=self.limit

    def request(self,a,b,d):
        n=a.n*d if d is not None else a.n+b.n
        self.stats['requests']+=1
        self.stats['all_requested_length']+=n
        key=(a,b,d)
        if key in self.by_pair:
            self.stats['structural_hits']+=1
            return self.by_pair[key]
        spelling=None
        if self.collect(n):
            spelling=self.texts[a]*d if d is not None else self.texts[a]+self.texts[b]
            self.stats['constructed_text_bytes']+=len(spelling)
            if spelling in self.by_text:
                self.stats['extra_text_hits']+=1
                self.by_pair[key]=self.by_text[spelling]
                return self.by_text[spelling]
        z=self.handle(left=a,right=b,repeat=d or 1)
        if spelling is not None:
            self.texts[z]=spelling
            self.by_text[spelling]=z
        self.by_pair[key]=z
        self.stats['internal_nodes']+=1
        return z


def summary(root):
    levels={}
    def height(v):
        if v not in levels:
            if v.literal is not None:
                h=0
            else:
                h=1+max(height(v.left),height(v.right) if v.right is not None else 0)
            levels[v]=h
        return levels[v]
    h=height(root)
    return dict(reachable=len(levels),height=h)


def check(tape,control,limit):
    meta,_,mark,raw,words=real.source(tape)
    pool=Pool(limit)
    def measured_output(node,skip,horizon):
        # Snapshot construction before the output-only cyclic horizon wrapper.
        report=dict(pool.stats,**summary(node),ordered_pair_entries=len(pool.by_pair),
                    retained_text_bytes=sum(map(len,pool.by_text)))
        out,c=output(node,skip,horizon)
        report.update(emitter=c)
        return out,report
    capture={}
    def collect(node,skip,horizon):
        b,d=measured_output(node,skip,horizon)
        capture.update(d)
        return b,d['emitter']
    with patch.object(decode,'Node',pool.handle),patch.object(decode,'output',collect):
        out,counts=decode.solve(raw,words,mark,0,meta['n'])
    assert hashlib.sha256(out).hexdigest()==control['pre_rle_sha']
    for metric in ('rounds','old_rows','refined_ids','refined_runs','moved_marks','joins','final_skip','lap'):
        assert counts[metric]==control[metric]
    assert capture['requests']==control['joins']
    return dict(index=meta['index'],text_limit=limit,fresh_joins=control['joins'],baseline_visits=control['output_visits'],
                pre_rle_sha=control['pre_rle_sha'],**capture)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('tapes',type=Path)
    args=p.parse_args()
    controls={row['index']:row for row in map(json.loads,(HERE.parent/'real.jsonl').read_text().splitlines())}
    for tape in sorted(args.tapes.glob('*.irec.xz')):
        m,_,_,_,_=real.source(tape)
        for limit in (0,16,None):
            print(json.dumps(check(tape,controls[m['index']],limit)),flush=True)
