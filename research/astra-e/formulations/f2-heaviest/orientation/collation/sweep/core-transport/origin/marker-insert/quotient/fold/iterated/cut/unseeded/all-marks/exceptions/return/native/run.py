"""Verify original caps/range wires, time both complete clock builders, and sample fresh HWM."""
import csv
import hashlib
import json
import math
from pathlib import Path
import os
import re
import statistics
import struct
import subprocess
import sys

HERE=Path(__file__).resolve().parent
WORK,CAPS,MASKS=map(lambda p:Path(p).resolve(),sys.argv[1:4])
PHASE=sys.argv[4]
WORK.mkdir(parents=True,exist_ok=True)
EXE=WORK/'native'
MANIFEST=WORK/'manifest.json'
DIGEST={ (r['index'],r['k']):r for r in json.loads((HERE.parent/'inputs.json').read_text())}
IDS=sorted({i for i,k in DIGEST})
PREFIX=str(MASKS)
HASH=lambda b:hashlib.sha256(b).hexdigest()
def csv_writer(f):
    return csv.writer(f,lineterminator='\n')
def args(*other):
    return [EXE,*[str(x)for x in other]]
if PHASE=='build':
    subprocess.run(['g++','-std=c++20','-O3','-Wall','-Wextra',HERE/'native.cpp','-o',EXE],check=True)
    inputs=[]
    for (idx,k),h in sorted(DIGEST.items()):
        caps=(CAPS/f'block-{idx}.caps').read_bytes()
        buf=(MASKS/f'{idx}.mask{k}').read_bytes()
        assert HASH(buf)==h['mask'] and buf[:8]==b'CLKMASK1'
        n,p=struct.unpack_from('<II',buf,8)
        last,mask=buf[16:16+n],buf[16+n:]
        assert len(mask)==n
        expected=b''.join(bytes((x>>8,))*(x&255)for x, in struct.iter_unpack('<H',caps))
        assert expected==last and set(mask)<={0,1}
        base=0;segments=bytearray()
        for m in re.finditer(b'\x01+',mask):
            segments.extend(struct.pack('<III',m.start(),m.end(),base));base+=m.end()-m.start()
        payload=struct.pack('<III',n,p,len(caps)//2)+caps+struct.pack('<I',len(segments)//12)+segments
        (WORK/f'{idx}.{k}').write_bytes(payload)
        inputs.append(dict(index=idx,k=k,cap_sha256=HASH(caps),wire_sha256=HASH(payload),exception_ranges=len(segments)//12))
    MANIFEST.write_text(json.dumps(dict(exe_sha256=HASH(EXE.read_bytes()),cpp_sha256=HASH((HERE/'native.cpp').read_bytes()),
        compiler=subprocess.check_output(['g++','--version'],text=True).splitlines()[0],inputs=inputs),indent=2)+'\n')
elif PHASE=='price':
    identity=json.loads(MANIFEST.read_text())
    assert HASH(EXE.read_bytes())==identity['exe_sha256'] and identity['cpp_sha256']==HASH((HERE/'native.cpp').read_bytes())
    assert all(HASH((WORK/f"{v['index']}.{v['k']}").read_bytes())==v['wire_sha256'] for v in identity['inputs'])
    for stage in ('check','time','repeat-check'):
        with (WORK/(stage+'.csv')).open('x') as f:
            test=stage!='time'
            csv_writer(f).writerow(['block','k','lap','events'] if test else ['round','k','position','arm','block','cpu_ns','events','lap'])
            f.flush()
            subprocess.run(args('check' if test else stage,WORK,MASKS,*IDS),stdout=f,check=True)
    assert (WORK/'check.csv').read_bytes()==(WORK/'repeat-check.csv').read_bytes()
    with (WORK/'rss.csv').open('x') as f:
        csv_writer(f).writerow(['block','k','arm','pre_hwm_kib','post_hwm_kib','retained_bytes']);f.flush()
        for b in IDS:
            for k in (4,8,16):
                for arm in ('direct','induced'):
                    subprocess.run(args('rss',WORK,MASKS,b,k,arm),stdout=f,check=True)
elif PHASE=='report':
    samples=list(csv.DictReader((WORK/'time.csv').open()));assert len(samples)==24*3*2*len(IDS)
    by={};order=[]
    for r in range(24):
        for j in range(3):
            i=(r+j)%3;k=(4,8,16)[i]
            for pos in range(2):
                arm=('direct','induced')[(r+i+pos)%2]
                order.extend((r,k,pos,arm,b)for b in IDS)
    assert order==[(int(s['round']),int(s['k']),int(s['position']),s['arm'],int(s['block'])) for s in samples]
    check={(int(s['block']),int(s['k'])):(int(s['events']),int(s['lap']))for s in csv.DictReader((WORK/'check.csv').open())}
    for s in samples:
        key=(int(s['round']),int(s['k']),s['arm'])
        assert check[int(s['block']),int(s['k'])]==(int(s['events']),int(s['lap']))
        by[key]=by.get(key,0)+int(s['cpu_ns'])
    rss=list(csv.DictReader((WORK/'rss.csv').open()))
    assert len(rss)==60
    summary={}
    for k in (4,8,16):
        x={a:[by[r,k,a]/1e6 for r in range(24)] for a in ('direct','induced')}
        logs=[math.log(v/w)for v,w in zip(x['induced'],x['direct'],strict=True)]
        m=statistics.mean(logs);h=2.06865761041904*statistics.stdev(logs)/math.sqrt(24)
        mem={}
        for a in x:
            rs=[v for v in rss if v['arm']==a and int(v['k'])==k]
            mem[a]=dict(max_process_kib=max(int(v['post_hwm_kib'])for v in rs),
                        max_growth_kib=max(int(v['post_hwm_kib'])-int(v['pre_hwm_kib'])for v in rs),
                        max_reused_vector_bytes=max(int(v['retained_bytes'])for v in rs))
        summary[k]=dict(rounds=24,median_cpu_ms={a:statistics.median(v)for a,v in x.items()},
            ratio=math.exp(m),lo=math.exp(m-h),hi=math.exp(m+h),memory=mem)
    print(json.dumps(summary,indent=2))
else:raise ValueError(PHASE)
