"""Carry the old source owner to the next implicit inverse transition."""
from bisect import bisect_right
import csv
from pathlib import Path
import sys
from check import old
from constructor import initial,step
from implicit import Implicit

class Hints:
    def __init__(self,c):
        self.c=c;self.lo=[];self.hi=[];j=0
        boundaries=c.source+[c.n];target=c.istart+[c.n]
        for x,end in zip(boundaries,boundaries[1:]):
            while target[j+1]<=x and j+1<len(c.image):j+=1
            lo=j
            if x<end:
                while target[j+1]<end:j+=1
                self.lo.append(lo);self.hi.append(j+1)
            else:self.lo.append(0);self.hi.append(0)
    def next(self,v,source_owner):
        lo,hi=self.lo[source_owner],self.hi[source_owner]
        assert lo<hi
        image_index=lo if hi==lo+1 else bisect_right(self.c.istart,v,lo+1,hi)-1
        return self.c.at_owner(v,image_index)

def examine(path):
    lines=path.read_text().splitlines();mark=int(lines[0]);raw=(path.parent/"cut.bin").read_bytes()
    bands,image=initial([tuple(map(int,x.split()))for x in lines[1:]])
    col={v[3]:old['colour'](v[3],0) for v in bands}
    rules={};b,ix,m,_=step(bands,image,mark,col,rules);c=Implicit(bands,image,mark,col,{})
    h=Hints(c);owner=0;owner_end=c.source+[c.n];expected_image_index=0
    totals=dict(tape=path.parent.name,rows=c.n,old_bands=len(bands),
        ownership_contacts=sum(u-l for l,u in zip(h.lo,h.hi)),largest_primary=0,
        primary_singleton=0,primary_two=0,secondary_zero=0,both_no_search=0,
        primary_le2_secondary_le1=0,primary_binary_comparisons_upper=0,
        secondary_binary_comparisons_upper=0)
    for id in ix:
        x,y,n,w=b[id]
        for q in range(y,y+n):
            while owner_end[owner+1]<=q:owner+=1
            while expected_image_index+1<len(c.image) and c.istart[expected_image_index+1]<=q:expected_image_index+=1
            e=c.image[expected_image_index];p=h.hi[owner]-h.lo[owner]
            s=c.end[e]-c.f[e] if c.f[e]>=0 else 0
            z,wo,src=h.next(q,owner)
            assert (z,wo)==(x+q-y,w) and owner_end[src]<=z<owner_end[src+1]
            totals["largest_primary"]=max(totals["largest_primary"],p)
            totals["primary_singleton"]+=p==1;totals["primary_two"]+=p==2;totals["secondary_zero"]+=s==0
            totals["both_no_search"]+=(p==1 and s==0);totals["primary_le2_secondary_le1"]+=(p<=2 and s<=1)
            totals["primary_binary_comparisons_upper"]+=(p-1).bit_length();totals["secondary_binary_comparisons_upper"]+=s.bit_length()
    source=bisect_right(c.source,m)-1
    W={k:bytes([k]) for k in col}
    for(a,b),v in rules.items():W[v]=bytes([a,b])
    row=m;at=steps=0
    while at<len(raw):
        row,w,source=h.next(row,source);n=min(len(W[w]),len(raw)-at)
        assert raw[at:at+n]==W[w][:n];at+=n;steps+=1
    totals["marked_queries"]=steps;totals["marked_horizon"]=at
    return totals
if __name__=="__main__":
    f=None
    for p in sorted(Path(sys.argv[1]).glob('0*/input.txt')):
        v=examine(p)
        if f is None:f=csv.DictWriter(sys.stdout,v,lineterminator="\n");f.writeheader()
        f.writerow(v);sys.stdout.flush()
