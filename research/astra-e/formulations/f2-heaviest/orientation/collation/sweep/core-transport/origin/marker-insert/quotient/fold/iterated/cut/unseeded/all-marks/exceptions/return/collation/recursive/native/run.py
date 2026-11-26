"""Price complete marked original bytes when the first sparse ordinary child has been provided."""
from pathlib import Path
import csv
import hashlib
import json
import math
import os
import statistics
import struct
import subprocess as sp
import sys
import tarfile
import tempfile
here=Path(__file__).resolve().parent
ret=here.parents[2]
publish=ret/'collation/publish'
repo=Path(sp.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
ref,contexts,work=map(lambda x:Path(x).resolve(),sys.argv[1:4])
action=sys.argv[4]
work.mkdir(exist_ok=True,parents=True)
sha=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
prior=json.loads((ret/'emitter/build.json').read_text())
inputs={q['block']:q for q in json.loads((ret/'emitter/inputs.json').read_text())}
objects={q['block']:q for q in json.loads((publish/'inputs.json').read_text())if q['k']==8}
ctx={int(q['index']):q for q in csv.DictReader((ret.parent/'contexts/blocks.csv').open())if q['k']=='8'}
caps={int(q['block']):int(q['cap_records']) for q in csv.DictReader((ret/'emitter/times.csv').open())}
corpus=repo/'corpora/wikidata.bz2'
assert sha(ref/'panel')==prior['rust']['exe_sha256'] and sha(corpus)==prior['corpus_sha256']
names=['prepared_bank','build_bank','jump_rules','scalar','lazy','opend']
labels=dict(zip(['stream','streamgaps','dense','scalar','lazy','opend'],names,strict=True))
if action=='build':
    inc='unsigned lookup(unsigned n,unsigned p,unsigned c){\n'+''.join(
        f"if(n=={q['n']}u&&p=={q['primary']}u&&c=={caps[b]}u)return {b}u;\n" for b,q in inputs.items())+'std::abort();}\n'
    (work/'ids.inc').write_text(inc);assert sha(work/'ids.inc')==prior['ids_sha256']
    flags=['-std=c++20','-O3','-g0','-Wall','-Wextra','-Werror']
    for source,opt,name in [('terminal.cpp',['-fPIC','-shared'],'libgrammar.so'),('small.cpp',[],'small')]:
        sp.run(['g++',*flags,*opt,'-I',work,here/source,'-o',work/name],check=True)
    files=work/'side';files.mkdir(exist_ok=True);manifest=[]
    with tarfile.open(publish/'columns.tar.xz','r:xz') as arc:
        for b,q in inputs.items():
            obj=files/f'{b}.col';obj.write_bytes(arc.extractfile(f'{b}-8.col').read())
            assert sha(obj)==objects[b]['column_sha256']
            src=contexts/str(b)/'8.ctx';assert sha(src)==ctx[b]['sha256']
            raw=src.read_bytes();assert raw[:8]==b'CTXMOD01'
            n,p,k,C,D,E,start=struct.unpack_from('<7I',raw,8)
            assert (n,p,k,C,E,start)==(q['n'],q['primary'],8,q['states'],q['events'],q['start_state'])
            assert len(raw)==36+9*C+13*D
            payload=bytearray(b'MODEDEF1'+struct.pack('<3I',n,start,C))
            for _,post,byte in struct.iter_unpack('<IIB',raw[36:36+9*C]):payload.extend(struct.pack('<I',(post<<8)|byte))
            model=files/f'{b}.model';model.write_bytes(payload)
            manifest.append(dict(block=b,input_ctx_sha256=sha(src),column_sha256=sha(obj),defaults_sha256=sha(model),
                                 column_bytes=obj.stat().st_size,defaults_bytes=len(payload),raw_sha256=q['raw_sha256'],
                                 libbzip2_plain_sha256=q['libbzip2_plain_sha256'],caps_sha256=q['caps_le_sha256'],block_crc=q['block_crc']))
    (work/'inputs.json').write_text(json.dumps(manifest,indent=2)+'\n')
    build=dict(compiler=sp.check_output(['g++','--version'],text=True).splitlines()[0],flags=flags,
               code={k:sha(here/k)for k in ['terminal.cpp','small.cpp','run.py']},
               binaries={k:sha(work/k)for k in ['libgrammar.so','small']},ids_sha256=sha(work/'ids.inc'),
               scalar_donor=prior['rust']['donor'],rust_panel=sha(ref/'panel'),rust_rss=sha(ref/'rss'))
    (work/'build.json').write_text(json.dumps(build,indent=2)+'\n')
elif action in ('gate','time','rss'):
    frozen=json.loads((work/'build.json').read_text())
    assert all(sha(here/k)==v for k,v in frozen['code'].items())
    assert all(sha(work/k)==v for k,v in frozen['binaries'].items())
    assert sha(ref/'rss')==frozen['rust_rss']
    side=work/'side';out=work/action;out.mkdir(exist_ok=True)
    files=json.loads((work/'inputs.json').read_text())
    for q in files:
        b=q['block'];assert sha(side/f'{b}.col')==q['column_sha256'] and sha(side/f'{b}.model')==q['defaults_sha256']
    env=dict(os.environ,LD_LIBRARY_PATH=str(work),ORD_INPUTS=str(side))
    exe=ref/('rss' if action=='rss' else 'panel')
    assert f'libgrammar.so => {work}/libgrammar.so' in sp.check_output(['ldd',exe],env=env,text=True)
    if action=='rss':
        with tempfile.TemporaryDirectory(dir=out) as tmp,(out/'rss.csv').open('w')as f:
            d=Path(tmp);(d/'sample.bz2').symlink_to(corpus)
            sp.run(['bzip2recover','sample.bz2'],cwd=d,stdout=sp.DEVNULL,stderr=sp.DEVNULL,check=True)
            w=csv.writer(f,lineterminator='\n');w.writerow('round block arm primary n caps before_kib after_kib'.split())
            for r in range(3):
                for b,q in inputs.items():
                    for t in range(6):
                        j=(r+b+t)%6;vals=list(map(int,sp.check_output([exe,d/f'rec{b+1:05d}sample.bz2',list(labels)[j],ref/f'block-{b}.raw'],env=env,text=True).split(',')))
                        assert vals[:3]==[q['primary'],q['n'],caps[b]];w.writerow([r,b,names[j],*vals])
    else:
        if action=='gate':(work/'small.csv').write_bytes(sp.check_output([work/'small']))
        env['ORD_STAMPS']=str(out/'phases')
        sp.run([exe,corpus,out,'0'if action=='gate'else'24'],env=env,check=True)
        for q in files:
            b=q['block'];assert sha(out/f'block-{b}.raw')==q['raw_sha256'] and sha(out/f'block-{b}.caps')==q['caps_sha256']
        for j in range(3):
            stamps=list(csv.DictReader((out/f'phases-{j}.csv').open()));assert len(stamps)==(10 if action=='gate'else 250)
            for i,z in enumerate(stamps):
                b=int(z['block']);assert int(z['visits'])==inputs[b]['events'] and int(z['marked_lap'])==inputs[b]['n']
                assert (int(z['setup_ns'])>0)==(i<10)
elif action=='report':
    tdir=work/'time';obs=list(csv.DictReader((tdir/'times.csv').open()));assert len(obs)==1440
    stamps={n:iter(list(csv.DictReader((tdir/f'phases-{j}.csv').open()))[10:])for j,n in enumerate(names[:3])}
    totals={}
    for q in obs:
        b,r,j=int(q['block']),int(q['round']),int(q['slot']);a=labels[q['arm']]
        assert a==names[((r+b)+(j if (r//6)%2==0 else 5-j))%6]
        def add(k,v):totals.setdefault(k,[0]*24)[r-1]+=v
        elapsed=int(q['thread_ns']);add(a,elapsed)
        if a in stamps:
            z=next(stamps[a]);assert b==int(z['block'])
            parts=[int(z[k+'_ns'])for k in ['scatter','order','prepare','emit']];assert sum(parts)<=elapsed
            for k,v in zip(['scatter','order','prepare','emit'],parts,strict=True):add(a+'_'+k,v)
            add(a+'_chronology',parts[0]+parts[1])
    assert all(next(i,None)is None for i in stamps.values())
    def ratio(a,b):
        logs=[math.log(x/y)for x,y in zip(totals[a],totals[b],strict=True)];g=[statistics.mean(logs[i:i+6])for i in range(0,24,6)]
        m=statistics.mean(g);h=3.182446305284263*statistics.stdev(g)/2
        return dict(ratio=math.exp(m),lo=math.exp(m-h),hi=math.exp(m+h),groups=[math.exp(v)for v in g])
    vs={a+'/scalar':ratio(a,'scalar')for a in totals if a!='scalar'}
    for x in names[:3]:
        vs[x+'/opend']=ratio(x,'opend')
    vs['build_bank/prepared_bank']=ratio('build_bank','prepared_bank')
    rss=list(csv.DictReader((work/'rss/rss.csv').open()));assert len(rss)==180
    memory={a:{'max_hwm_kib':max(int(z['after_kib'])for z in rss if z['arm']==a),
               'max_growth_kib':max(int(z['after_kib'])-int(z['before_kib'])for z in rss if z['arm']==a)}for a in names}
    print(json.dumps(dict(rounds=24,labels=labels,median_ms={k:statistics.median(v)/1e6 for k,v in totals.items()},
                          contrasts=vs,memory=memory),indent=2))
else:raise ValueError(action)
