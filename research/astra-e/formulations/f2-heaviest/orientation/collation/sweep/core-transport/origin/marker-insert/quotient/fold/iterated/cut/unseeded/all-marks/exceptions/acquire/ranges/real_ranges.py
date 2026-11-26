"""Compare complete canonical descriptors with an independent membership-packet implementation."""
import argparse,csv,hashlib,struct,subprocess,sys,time
from bisect import bisect_right
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent.parent))
from real import original,read_model
from rangemap import create

def run(a):
    out=None
    for row in csv.DictReader(a.sources.open()):
        i=int(row['index']);mark,rle,L,lf,order,raw_hash=original(a.original/row['tape'].split('.')[0]/'input.txt')
        for k in a.depths:
            n,p,K,start,modal,annotations,sha=read_model(a.models/str(i)/f"{k}.ctx");assert p==mark
            begin=time.thread_time_ns();ret,u,stats=create(rle,start,modal,mark);dt=time.thread_time_ns()-begin
            body=b''.join(struct.pack('<III',s,t,h)for s,t,w,h in ret)
            independent=subprocess.check_output([a.reference,a.wires/f'{i}.mask{k}'])
            assert body==independent
            starts=[s for s,t,w,h in ret];first=u.rank(mark);j=first;t=0;events=[]
            while True:
                r=u.select(j);context=bisect_right(start,r)-1
                if L[r]!=modal[context]:events.append((t,L[r],bisect_right(start,lf[r])-1))
                z=bisect_right(starts,j)-1;s,y,w,h=ret[z];j=y+j-s;t+=h
                if j==first:break
                assert t<n
            assert t==n
            pred=(a.wires/f'{i}.pred{k}').read_bytes()
            assert list(struct.iter_unpack('<IBI',pred[24+5*K:]))==events
            line=dict(block=i,k=k,n=n,mark=mark,context_sha256=sha,raw_sha256=raw_hash,
                      independent_band_sha256=hashlib.sha256(body).hexdigest(),marked_lap=t,marked_events=len(events),
                      python_thread_ns=dt,**stats)
            if out is None:out=csv.DictWriter(sys.stdout,line,lineterminator='\n');out.writeheader()
            out.writerow(line);sys.stdout.flush()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('original',type=Path);p.add_argument('models',type=Path)
    p.add_argument('sources',type=Path);p.add_argument('wires',type=Path);p.add_argument('reference',type=Path)
    p.add_argument('--depths',type=int,nargs='+',default=[4,8,16]);run(p.parse_args())
