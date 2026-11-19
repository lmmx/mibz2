"""Apply exactly one natural-to-trailing rotation to each source word at each section."""
import csv
from hashlib import sha256
from pathlib import Path
import struct
import sys
from verify import Reader,MAX,decode

def owner(mark,skip,previous,w):
    if mark==MAX:return MAX,0
    if skip:return mark,skip-1
    x=previous[mark]
    return x,len(w[x])-1

def one(row,src,dst):
    name=row['case']
    j=Reader(src/(name+'.jh'),b'JHSPv1\0\0',row['j_sha256'])
    ordinal,policy,h,n,m,jm,skip,nt,nr,nh=(j.u32() for _ in range(10))
    d=[j.take(j.u32()) for _ in range(nt)]
    runs=[(j.u32(),j.u32()) for _ in range(nr)]
    holes=[j.u32() for _ in range(nh)];j.end()
    incoming=[t for t,w in runs for _ in range(w)]
    succ=sorted(range(m),key=incoming.__getitem__);pred=[None]*m
    for i,x in enumerate(succ):pred[x]=i
    tfrom=[incoming[x] for x in succ]
    old=[d[t] for t in tfrom]
    c=bytes.fromhex(row['core'])[0]
    assert all(w and w[0]==c for w in d)
    rd=[w[1:]+w[:1] for w in d]
    assert len(set(rd))==len(rd)
    jm,skip=owner(jm,skip,pred,old)
    out=bytearray(b'JHSPv1\0\0'+struct.pack('<10I',ordinal,policy,h,n,m,jm,skip,nt,nr,nh))
    for w in rd:out+=struct.pack('<I',len(w))+w
    for t,w in runs:out+=struct.pack('<II',t,w)
    for x in holes:out+=struct.pack('<I',x)
    (dst/(name+'.jh')).write_bytes(out)
    o=Reader(src/(name+'.oracle'),b'JHORv1\0\0',row['h_sha256'])
    mh,hm,hs,on=(o.u32() for _ in range(4));assert on==n
    targets=[];hw=[];hp=[None]*mh
    for i in range(mh):
        x=o.u32();hp[x]=i;targets.append(x);hw.append(o.take(o.u32()))
    plain=o.take(n);o.end()
    assert all(w and w[0]==c for w in hw)
    hm,hs=owner(hm,hs,hp,hw)
    rw=[w[1:]+w[:1] for w in hw]
    assert decode(targets,rw,hm,hs,n)==decode(succ,[rd[t] for t in tfrom],jm,skip,n)==plain
    rec=bytearray()
    for x,w in zip(targets,rw):rec+=struct.pack('<II',x,len(w))+w
    data=b'JHORv1\0\0'+struct.pack('<4I',mh,hm,hs,n)+rec+plain
    (dst/(name+'.oracle')).write_bytes(data)
    return dict(row,j_mark=jm,j_skip=skip,h_mark=hm,h_skip=hs,j_sha256=sha256(out).hexdigest(),
        h_sha256=sha256(data).hexdigest(),h_map_words_sha256=sha256(rec).hexdigest(),word_convention='native_trailing')

def main():
    src=Path(sys.argv[1]);dst=Path(sys.argv[2]);dst.mkdir(parents=True,exist_ok=True)
    writer=None
    with Path(sys.argv[3]).open() as f:
        for row in csv.DictReader(f):
            r=one(row,src,dst)
            if writer is None:
                writer=csv.DictWriter(sys.stdout,r.keys(),lineterminator='\n');writer.writeheader()
            writer.writerow(r);sys.stdout.flush()
if __name__=='__main__':main()
