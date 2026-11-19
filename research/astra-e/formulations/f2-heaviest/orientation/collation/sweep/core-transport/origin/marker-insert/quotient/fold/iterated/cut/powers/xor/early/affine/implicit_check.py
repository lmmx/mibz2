"""Every row on first-level ten-tape maps and all small arbitrary interval exchanges."""
import csv,itertools,json,sys
from pathlib import Path
from check import old
from constructor import initial,step
from implicit import Implicit

def verify(bands,image,mark,colors):
    rules={};ref,ix,m,_=step(bands,image,mark,colors,rules)
    new={};v=Implicit(bands,image,mark,colors,new)
    assert rules==new and m==v.mark and v.n==sum(n for x,y,n,w in ref)
    for id in ix:
        x,y,n,w=ref[id]
        for offset in range(n):assert v.at(y+offset)==(x+offset,w)
    return v
def small():
    rows=cases=0
    for k in range(1,5):
        for widths in itertools.product((1,2),repeat=k):
            x=[0]
            for w in widths:x.append(x[-1]+w)
            for image in itertools.permutations(range(k)):
                ys=[0]*k;y=0
                for e in image:ys[e]=y;y+=widths[e]
                for families in (list(range(k)),[e//2 for e in range(k)]):
                    bands=[(x[e],ys[e],widths[e],families[e]) for e in range(k)]
                    for colors in itertools.product((0,1),repeat=max(families)+1):
                        for mark in range(y):
                            v=verify(bands,image,mark,colors);rows+=v.n;cases+=1
    print(json.dumps(dict(cases=cases,individual_successor_and_word_checks=rows)))
def panel(folder):
    f=csv.writer(sys.stdout,lineterminator="\n")
    f.writerow(["tape","old_bands","residual_rows","image_owner_records","cross_records","rows_no_secondary_lookup","rows_one_hit_slice","rows_larger_slice","largest_hit_slice","weighted_comparison_upper"])
    for path in sorted(folder.glob("0*/input.txt")):
        lines=path.read_text().splitlines()
        b,ix=initial([tuple(map(int,l.split()))for l in lines[1:]])
        col={e[3]:old['colour'](e[3],0)for e in b}
        v=verify(b,ix,int(lines[0]),col);j=v.work
        f.writerow([path.parent.name,len(b),v.n,len(v.image),len(v.lo),j.get(0,0),j.get(1,0),sum(n for c,n in j.items()if c>1),max(j),
                   sum(k.bit_length()*n for k,n in j.items())])
        sys.stdout.flush()
if __name__=="__main__":
    if sys.argv[1]=="tiny":small()
    else:panel(Path(sys.argv[1]))
