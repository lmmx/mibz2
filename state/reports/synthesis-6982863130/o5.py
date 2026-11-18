from collections import Counter
from hashlib import sha256
from itertools import accumulate
from pathlib import Path
import sys


def lf(column):
    count=Counter(column)
    at=list(accumulate([0]+[count[i] for i in range(256)]))
    out=[]
    for b in column:
        out.append(at[b])
        at[b]+=1
    return out


def contraction(column, primary):
    n=len(column)
    forward=lf(column)
    phase_order=[(i+primary)%n for i in range(n)]
    letters=[column[i] for i in phase_order]
    g={i:(forward[row]-primary)%n for i,row in enumerate(phase_order)}
    word_len=dict.fromkeys(g,1)
    cut={0}|{i for i in range(1,n) if letters[i]!=letters[i-1] or g[i]!=g[i-1]+1}
    out=[]
    while g[0]!=0:
        pre={y:x for x,y in g.items()}
        survivors={x for x in g if x<pre[x] and x<g[x]}
        assert 0 in survivors and len(survivors)*2<=len(g)
        compressed={x:i for i,x in enumerate(g)}
        atom_id={}
        atoms=0
        previous=None
        for x in g:
            if previous is None or x!=previous+1 or x in cut or word_len[x]!=word_len[previous] or compressed[g[x]]!=compressed[g[previous]]+1:
                atoms+=1
            atom_id[x]=atoms
            previous=x
        next_={}
        next_len={}
        next_cut=set()
        max_len, max_atom_passages, workload, repeats=0,0,0,0
        for x in sorted(survivors):
            y=x
            covered=[]
            while True:
                covered.append(y)
                y=g[y]
                if y in survivors:
                    break
            next_[x]=y
            next_len[x]=sum(word_len[a] for a in covered)
            if any(a in cut for a in covered):
                next_cut.add(x)
            max_len=max(max_len,len(covered))
            local_repeats=sum(atom_id[a]==atom_id[b] for a,b in zip(covered,covered[1:]))
            repeats+=local_repeats
            max_atom_passages=max(max_atom_passages,len(covered)-local_repeats)
            workload+=len(covered)
        assert workload==len(g)
        out.append((len(g),max_len,atoms,max_atom_passages,repeats))
        g,word_len,cut=next_,next_len,next_cut
    return out,len(g)


if __name__=="__main__":
    data=Path(sys.argv[1]).read_bytes()
    assert sha256(data).hexdigest()=="db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67"
    rows,terminal=contraction(data,264045)
    print("layers:",rows)
    print("rows,row depth,atoms (nonterminal),max atom passage sum,repeated visits",tuple(map(sum,zip(*rows))))
    print("terminal",terminal)
    for n in (10,33):
        events, _=contraction(bytes([1]+[0]*(n-1)),0)
        assert events[0][1]==n and len(events)==1
        print("long monotone excursion",n,events[0])
