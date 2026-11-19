"""Fingerprint the N,p,physical-cap tape and lock timing positions before evaluating their pair."""
import csv,hashlib,struct,sys
from pathlib import Path

def stream(path):
    with path.open('rb') as f:
        while head:=f.read(16):
            i,n,p,m=struct.unpack('<4I',head);caps=f.read(2*m);assert len(caps)==2*m
            yield i,hashlib.sha256(head[4:]+caps).hexdigest(),head+caps

def split(train,all_inputs,out):
    training=dict((h,i) for i,h,_ in stream(train));assert len(training)==10
    all_rows=list(stream(all_inputs));assert len(all_rows)==228 and len({x[0] for x in all_rows})==228
    held=sorted((i,h,b) for i,h,b in all_rows if h not in training)
    n=len(held);positions={(2*i+1)*n//24 for i in range(12)};assert len(positions)==12
    panel={held[p][0] for p in positions};out.mkdir(exist_ok=True)
    with (out/'held.bin').open('wb') as a,(out/'panel.bin').open('wb') as p:
        for i,h,b in held:
            a.write(b)
            if i in panel:p.write(b)
    with (out/'split.csv').open('w') as f:
        w=csv.writer(f,lineterminator='\n');w.writerow(('source','stream','sha256_count_primary_caps','split','timed'))
        for h,i in sorted(training.items(),key=lambda x:x[1]):w.writerow(('ten',i,h,'training',0))
        for i,h,b in sorted(all_rows):w.writerow(('admissions',i,h,'training_overlap' if h in training else 'heldout',int(i in panel)))
if __name__=='__main__':split(*(Path(a) for a in sys.argv[1:]))
