"""Replay complete jobs in bounded slots at frozen trace and calibrated service rates."""
import argparse
import csv
import gzip
import hashlib
import json
import lzma
from pathlib import Path
from statistics import median
import tarfile
from model import Jobs,bounds,fifo_independent,simulate

HERE=Path(__file__).resolve().parent
RESEARCH=HERE.parent
POLICIES=('fifo','lpt','symbols','old4-lpt','old4-symbols')
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def read_csv(p):
    with (gzip.open if p.suffix=='.gz' else lzma.open if p.suffix=='.xz' else open)(p,'rt') as f:
        return list(csv.DictReader(f))
def dump(p,rows):
    with p.open('w') as f:
        w=csv.DictWriter(f,fieldnames=rows[0],lineterminator='\n')
        w.writeheader();w.writerows(rows)

def trace(path):
    rows=read_csv(path)
    items={}
    for kind in ('SlotGot','SlotWait','Issued','Enqueued','Taken','JobGot','DecodeStart','DecodeEnd','Ready','ReadWait'):
        items[kind]=sorted([r for r in rows if r['kind']==kind],
                           key=lambda r:int(r['place']) if r['kind'] not in ('ReadWait','SlotGot','SlotWait') else int(r['ns']))
    n=len(items['Issued']);assert n==862
    for key,rs in items.items():
        if key in ('ReadWait','SlotGot','SlotWait'):assert len(rs)==n+1
        else:assert [int(r['place']) for r in rs]==list(range(n))
    def ns(k,i,field='ns'):return int(items[k][i][field])
    # Scanner busy interval with fixed inter-job housekeeping, excluding waiting for a slot.
    scan=[ns('Enqueued',i)-ns('SlotGot',i)+(ns('SlotWait',i)-ns('Enqueued',i-1) if i else 0)
          for i in range(n)]
    copy=[ns('ReadWait',i+1)-ns('Taken',i) for i in range(n)]
    work=[ns('DecodeEnd',i,'cpu_ns')-ns('DecodeStart',i,'cpu_ns') for i in range(n)]
    elapsed=[ns('Ready',i)-ns('JobGot',i) for i in range(n)]
    symbols=[ns('Issued',i,'len')//2 for i in range(n)]
    assert all(2*s==ns('Issued',i,'len') for i,s in enumerate(symbols)) and symbols[:431]==symbols[431:]
    for vec in (scan,copy,work,elapsed):assert min(vec)>0
    # On a real trace Enqueued and Ready are stamped after releasing a message or lock.
    return Jobs(scan,work,copy,symbols),Jobs(scan,elapsed,copy,symbols),ns('ReadWait',n)-ns('SlotGot',0)

def record(rows,key,w,a,reference):
    cap=4*w
    floor=bounds(a,w,cap); fifo=fifo_independent(a,w,cap)
    for p in POLICIES:
        wall,choices,changed,largest,_=simulate(a,w,cap,p)
        assert floor<=wall and (p!='fifo' or wall==fifo)
        rows.append(dict(**key,workers=w,policy=p,blocks=len(a.work),slots=cap,reference_ns=reference,
                         paid_scan_ns=sum(a.scan),paid_work_ns=sum(a.work),paid_copy_ns=sum(a.copy),
                         floor_ns=floor,fifo_ns=fifo,wall_ns=wall,wall_over_fifo=wall/fifo,
                         upper_saving=1-floor/fifo,choice_points=choices,displaced_head=changed,
                         largest_queue=largest))

def target():
    role_path=RESEARCH/'budget/i9-roles.csv'
    arm_path=RESEARCH/'workstation/evidence/scalar-i9-1ac706d.tar.gz'
    with tarfile.open(arm_path) as f:
        samples=[json.loads(x) for x in f.extractfile(next(v for v in f if v.name.endswith('/runs.jsonl')))]
    hist=read_csv(role_path)
    out=[]
    for w in range(7,11):
        lazy=[r for r in samples if r['workers']==w and r['arm']=='lazy']
        scalar=[r for r in samples if r['workers']==w and r['arm']=='candidate']
        assert len(scalar)==len(lazy)==48 and all(r['out_bytes']==838800784 for r in scalar+lazy)
        l=median(r['wall_s'] for r in lazy)
        c=median(r['cpu_s'] for r in scalar)
        wall=median(r['wall_s'] for r in scalar)
        old=[r for r in hist if int(r['workers'])==w and r['arm']=='lazy']
        assert len(old)==12
        scale=l/median(float(r['wall']) for r in old)
        scan=scale*median(float(r['scanner']) for r in old)
        read=scale*median(float(r['main']) for r in old)
        # Preserve CPU mass outside scanner and worker from the older sweep.
        other=scale*median(float(r['cpu'])-float(r['worker_sum'])-float(r['scanner']) for r in old)
        work=c-other-scan
        assert min(scan,read,work)>0
        goal=.8*l
        out.append(dict(workers=w,scalar_s=wall,lazy_s=l,stretch_s=goal,scalar_cpu_s=c,
                        transported_scan_s=scan,transported_main_s=read,transported_other_s=other,
                        transported_worker_s=work))
    return out,[role_path,arm_path]

def scaled(vec,sec):
    unit=sec*1e9/sum(vec)
    return [max(1,round(x*unit)) for x in vec]

def main():
    arg=argparse.ArgumentParser();arg.add_argument('output',type=Path);p=arg.parse_args().output
    p.mkdir(parents=True,exist_ok=True)
    tr=RESEARCH/'huffman-phrases/integrated/trace'
    paths=sorted(tr.glob('marks-*-4-stock.csv.gz'))
    assert len(paths)==8
    local=[];data=[]
    for r,path in enumerate(paths):
        cpu,wall,ref=trace(path);data.append(cpu)
        for name,a in [('cpu',cpu),('elapsed',wall)]:
            record(local,dict(trace=r,shape=name),4,a,ref)
    dump(p/'local.csv',local)
    rates,src=target()
    dump(p/'rates.csv',rates)
    # Isolated reference has no scheduler waits or ordinal-specific output stalls.
    source={}
    for name,path in [('scan',RESEARCH/'budget/scanner.csv.xz'),('copy',RESEARCH/'budget/copy.csv.xz')]:
        src.append(path)
        rs=read_csv(path)
        source[name]=[median([int(x['ns']) for x in rs if int(x['block'])==b]) for b in range(431)]*4
    symbols=data[0].symbols*2
    assert all(d.symbols==data[0].symbols for d in data)
    a=[median([d.work[j] for d in data for j in (i,i+431)]) for i in range(431)]*4
    results=[]
    for rate in rates:
        w=rate['workers']
        for shape,work in [('median',a)]+[(str(i),d.work*2) for i,d in enumerate(data)]:
            for stress,sf,wf in [('central',1,1),('scan-20',0.8,1),('scan+20',1.2,1),('work-20',1,.8),('work+20',1,1.2)]:
                jobs=Jobs(scaled(source['scan'],sf*rate['transported_scan_s']),
                          scaled(work,wf*rate['transported_worker_s']),scaled(source['copy'],rate['transported_main_s']),symbols)
                record(results,dict(shape=shape,transport=stress),w,jobs,round(1e9*rate['scalar_s']))
    dump(p/'target.csv',results)
    manifest=dict(local_trace_pin='92684cc',scalar_pin='d0a42980e896211906234c7679b38760ae44a46e',
                  raw_inputs={str(path.relative_to(RESEARCH)):digest(path) for path in paths+src})
    (p/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')

if __name__=='__main__':main()
