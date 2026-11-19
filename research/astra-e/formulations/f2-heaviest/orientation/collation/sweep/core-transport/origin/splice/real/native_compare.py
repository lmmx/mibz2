"""Compare actual Compact first-object snapshots with independently rotated dense truth."""
import csv
from collections import Counter
from hashlib import sha256
from pathlib import Path
import sys
from verify import Reader,decode
from rotate import owner

def main():
    captured=Path(sys.argv[1]);fixtures=Path(sys.argv[2])
    writer=None
    for row in csv.DictReader(Path(sys.argv[3]).open()):
        name=row['case'];p=captured/(name+'.native')
        nd=sha256(p.read_bytes()).hexdigest()
        native=Reader(p,b'JNATv1\0\0',nd);j=Reader(fixtures/(name+'.jh'),b'JHSPv1\0\0',row['j_sha256'])
        i,policy,h,n,lo,hi,at,skip,types,nruns=[native.u32() for _ in range(10)]
        assert [i,policy,h,n,hi-lo,at,skip,types,nruns,int(row['holes'])]==[j.u32() for _ in range(10)]
        assert [lo,hi]==[int(row['j_lo']),int(row['j_hi'])]
        bases=[]
        for _ in range(types):
            bases.append(native.u32())
            assert native.take(native.u32())==j.take(j.u32())
        freq=Counter()
        for _ in range(nruns):
            t,q=j.u32(),j.u32()
            assert (native.u32(),native.u32())==(bases[t],q)
            freq[t]+=q
        b=0
        for t in range(types):
            assert bases[t]==b;b+=freq[t]
        assert b==hi-lo
        for _ in range(int(row['holes'])):j.u32()
        j.end()
        actual=native.take(n);native.end()
        assert sha256(actual).hexdigest()==row['marked_sha256']
        assert actual==(fixtures/(name+'.oracle')).read_bytes()[-n:]
        obs=dict(case=name,native_sha256=nd,source_rows=b,source_types=types,dict_bytes=int(row['dict_bytes']),
            incoming_runs=nruns,native_mark=at,trailing_skip=skip,full_bytes=n)
        if writer is None:
            writer=csv.DictWriter(sys.stdout,obs.keys(),lineterminator='\n');writer.writeheader()
        writer.writerow(obs)
    # All words start with 'X'. A mark at a natural word start has skip zero;
    # in the trailing convention its byte lies at the end of the previous word.
    natural=[b'XY',b'Xabc'];rotated=[w[1:]+w[:1] for w in natural];succ=pred=[1,0]
    assert owner(0,0,pred,natural)==(1,3)
    assert decode(succ,natural,0,0,13)==decode(succ,rotated,1,3,13)
    assert owner(0xffffffff,0,pred,natural)==(0xffffffff,0)
if __name__=='__main__':main()
