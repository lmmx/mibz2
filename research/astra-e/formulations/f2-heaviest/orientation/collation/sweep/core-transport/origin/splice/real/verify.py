"""Independently parse both binary streams and replay every token map, mark and word."""
from collections import Counter
import csv
from hashlib import sha256
from pathlib import Path
import struct
import sys

MAX=0xffffffff

class Reader:
    def __init__(self,path,magic,digest):
        self.data=path.read_bytes()
        assert sha256(self.data).hexdigest()==digest and self.data[:8]==magic
        self.pos=8
    def take(self,n):
        v=self.data[self.pos:self.pos+n];self.pos+=n
        assert len(v)==n
        return v
    def u32(self):
        return struct.unpack('<I',self.take(4))[0]
    def end(self):
        assert self.pos==len(self.data)

def decode(nexts,words,mark,skip,n):
    if mark==MAX:return None
    assert skip<len(words[mark])
    buf=bytearray(words[mark][skip:]);mark=nexts[mark]
    while len(buf)<n:
        buf.extend(words[mark]);mark=nexts[mark]
    return bytes(buf[:n])

def verify(folder,row,stats):
    stem=row['case']
    j=Reader(folder/(stem+'.jh'),b'JHSPv1\0\0',row['j_sha256'])
    ordinal,policy,h,n,m,jm,skip,ts,rs,ks=(j.u32() for _ in range(10))
    assert ordinal==int(row['ordinal']) and h==int(row['h']) and ['frequent','quote'][policy]==row['section']
    words=[j.take(j.u32()) for _ in range(ts)]
    runs=[(j.u32(),j.u32()) for _ in range(rs)]
    holes=[j.u32() for _ in range(ks)];j.end()
    assert sorted(set(holes))==holes and len(holes)==ks
    assert all(0<=x<m for x in holes) and all(q>0 and t<ts for t,q in runs)
    assert all(a[0]!=b[0] for a,b in zip(runs,runs[1:]))
    incoming=[t for t,q in runs for _ in range(q)]
    assert len(incoming)==m
    # Stable sorting physical destinations by their incoming symbol is the full
    # directed successor permutation in physical J source order.
    succ=sorted(range(m),key=incoming.__getitem__)
    source_words=[words[incoming[x]] for x in succ]
    pred=[0]*m
    for i,y in enumerate(succ):pred[y]=i
    forbidden=set(holes)
    keep=[x for x in range(m) if x not in forbidden]
    rank={k:i for i,k in enumerate(keep)}
    o=Reader(folder/(stem+'.oracle'),b'JHORv1\0\0',row['h_sha256'])
    hm_count,hm,hs,on=(o.u32() for _ in range(4))
    assert hm_count==len(keep) and on==n
    offsets=(jm,skip)
    if jm!=MAX:
        cursor=jm
        while cursor in forbidden:
            cursor=pred[cursor];skip+=len(source_words[cursor])
            if cursor==jm:break
        assert (hm,hs)==((MAX,0) if cursor in forbidden else (rank[cursor],skip))
    else:assert (hm,hs)==(MAX,0)
    ys=[];hw=[]
    map_start=o.pos
    for x in keep:
        y=succ[x];word=bytearray(source_words[x])
        while y in forbidden:
            word.extend(source_words[y]);y=succ[y]
        target=o.u32();v=o.take(o.u32())
        assert (rank[y],word)==(target,v)
        ys.append(target);hw.append(v)
    assert sha256(o.data[map_start:o.pos]).hexdigest()==row['h_map_words_sha256']
    original=o.take(n);o.end()
    assert sha256(original).hexdigest()==row['marked_sha256']
    raw_j=decode(succ,source_words,*offsets,n);raw_h=decode(ys,hw,hm,hs,n)
    assert (raw_j is None or raw_j==original) and (raw_h is None or raw_h==original)
    stats.update(objects=1,j_rows=m,h_rows=len(keep),holes=ks,
        marked_bytes=n,j_mark_on_hole=int(jm in forbidden),positive_holes=int(bool(holes)),
        h_spelling_bytes=sum(map(len,hw)),raw_j_fallback=int(raw_j is None),raw_h_fallback=int(raw_h is None))

def witnesses():
    from generate import direct,marked,output
    # Two disjoint fixed points. The raw-mark fallback tag is required for J
    # and H when the only source lies on the other cycle.
    last=b'ab'
    assert marked([0,1],[0],1)==(MAX,0)
    assert marked([0,1],[],0)==(MAX,0)
    # The original primary is J row 1, a hole. Its preceding retained source
    # spells "ab". Offset 1 must produce the three coded bytes "bab".
    last=b'bab';forward=[1,0,2];inverse=[1,0,2]
    sj,wj=direct(last,forward,[0,1]);sh,wh=direct(last,forward,[0])
    assert marked(inverse,[0,1],1)==(1,0)
    assert marked(inverse,[0],1)==(0,1)
    assert output(sj,wj,1,0,3)==output(sh,wh,0,1,3)==b'bab'
    assert marked(inverse,[0],2)==(MAX,0)

def main():
    folder=Path(sys.argv[1])
    stats=Counter()
    with Path(sys.argv[2]).open() as f:
        for row in csv.DictReader(f):verify(folder,row,stats)
    witnesses()
    for k in sorted(stats):print(k,stats[k])
    print("Replayed every physical row, original-digest marked bytes and fallback/offset witnesses.")
if __name__=='__main__':main()
