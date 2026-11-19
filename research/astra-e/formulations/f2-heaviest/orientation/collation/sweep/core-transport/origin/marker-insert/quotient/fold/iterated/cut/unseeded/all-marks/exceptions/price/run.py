"""Exact capped-input scalar ABI and balanced prepared and provision timing panels."""
from pathlib import Path
import os,sys,csv,json,io,hashlib,struct,subprocess as sp,tempfile
here=Path(__file__).resolve().parent;repo=Path(sp.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
donor=here.parents[3]/'powers/xor/capped';receipt=json.loads((donor/'receipt.json').read_text())
ref,wires,base=map(lambda v:Path(v).resolve(),sys.argv[1:4]);action=sys.argv[4]
exe=ref/'panel';corpus=repo/'corpora/wikidata.bz2';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
build=json.loads((ref/'build.json').read_text());old=json.loads((donor/'build.json').read_text())
for k in ['rustc','donor','src_tree','source_sha256','rlib_sha256','rust_flags']:assert build[k]==old[k]
assert sha(exe)==build['exe_sha256'] and sha(corpus)==receipt['corpus_sha256']
rows=receipt['inputs'];base.mkdir(parents=True,exist_ok=True)
if action=='build':
    (base/'ids.inc').write_text('unsigned lookup(unsigned n,unsigned p,unsigned c){\n'+''.join(
         f"if(n=={q['rows']}u&&p=={q['mark']}u&&c=={q['records']}u)return {q['block']}u;\n" for q in rows)+'std::abort();}\n')
    args=['g++','-std=c++20','-O3','-g0','-Wall','-Wextra','-fPIC','-shared','-I',base,here/'clock.cpp','-o',base/'libgrammar.so']
    sp.run(args,check=True)
    (base/'manifest.json').write_text(json.dumps(dict(source_sha256=sha(here/'clock.cpp'),ids_sha256=sha(base/'ids.inc'),
                  so_sha256=sha(base/'libgrammar.so'),gxx=sp.check_output(['g++','--version'],text=True).splitlines()[0],
                  rust_executable=sha(exe),rust_build=build,flags=args[1:8]),indent=2)+'\n')
    sys.exit()
k=int(sys.argv[5]);mode=sys.argv[6];assert k in (8,16) and mode in ('prepared','provision')
out=base/f'k{k}-{mode}';out.mkdir(exist_ok=True)
env=dict(os.environ,CLOCK_K=str(k),CLOCK_MODE=mode,CLOCK_WIRES=str(wires),CLOCK_REFS=str(ref),LD_LIBRARY_PATH=str(base))
assert f'libgrammar.so => {base}/libgrammar.so' in sp.check_output(['ldd',exe],env=env,text=True)
labels=['unit_repeat','jump','jump_repeat'] if mode=='prepared' else ['parse_unit','parse_jump','build_jump']
old_labels=['stream','streamgaps','dense']
def verify_inputs():
    census={(int(q['index']),int(q['k'])):q for q in csv.DictReader((here.parent/'panel/blocks.csv').open())}
    inputs=[]
    for q in rows:
        b=q['block'];z=census[b,k];src=wires/f'{b}.pred{k}';data=src.read_bytes()
        assert data[:8]==b'PREDCLK1';n,state,C,E=struct.unpack_from('<4I',data,8)
        assert n==q['rows']==int(z['n']) and C==int(z['states']) and E==int(z['exceptions'])
        assert len(data)==24+5*C+9*E and 0<=state<C
        assert sha(ref/f'block-{b}.raw')==q['raw_sha256']
        inputs.append(dict(block=b,k=k,n=n,primary=q['mark'],start=state,states=C,events=E,wire_bytes=len(data),wire_sha256=hashlib.sha256(data).hexdigest(),raw_sha256=q['raw_sha256'],
                           decoded_sha256=q['decoded_sha256'],crc=q['block_crc'],jump8=int(z['chunk8']),unit_tail=int(z['tail8'])))
    return inputs
if action in ('gate','price'):
    inputs=verify_inputs();env['CLOCK_STAMPS']=str(out/'stamps.csv')
    sp.run([exe,corpus,out,'24' if action=='price' else '0'],env=env,check=True)
    for q in rows:
        b=q['block'];assert sha(out/f'block-{b}.raw')==q['raw_sha256'] and sha(out/f'block-{b}.caps')==q['caps_le_sha256']
    t=list(csv.DictReader((out/'stamps.csv').open()))
    assert all(int(q['once_ns'])>0 for q in t[:30])
    if action=='gate':
        assert len(t)==30;(out/'gate.json').write_text(json.dumps(dict(outputs=60,files=inputs,so=sha(base/'libgrammar.so')),indent=2)+'\n')
    else:
        assert len(t)==750;it=iter(t[30:]);obs=list(csv.DictReader((out/'times.csv').open()));assert len(obs)==1440
        for q in obs:
            if q['arm'] in old_labels:
                j=old_labels.index(q['arm']);p=next(it);assert p['block']==q['block'] and int(p['arm'])==j and int(p['once_ns'])==0 and int(p['k'])==k
                q.update(arm=labels[j],**{a:v for a,v in p.items() if a not in ('arm','block')})
                inside=sum(int(p[a]) for a in ['parse_ns','jump_ns','provision_ns','first_ns','verify_ns','second_ns'])
                assert inside<=int(q['thread_ns'])
        assert not list(it)
        keys=list(dict.fromkeys(a for q in obs for a in q))
        with (out/'paired.csv').open('w') as f:
            wr=csv.DictWriter(f,keys);wr.writeheader();wr.writerows(obs)
elif action=='rss':
    rss=ref/'rss';assert f'libgrammar.so => {base}/libgrammar.so' in sp.check_output(['ldd',rss],env=env,text=True)
    all6=labels+['scalar','lazy','opend'];donors=old_labels+all6[3:]
    with tempfile.TemporaryDirectory(dir=out) as tmp,(out/'rss.csv').open('w') as f:
        dest=Path(tmp);(dest/'sample.bz2').symlink_to(corpus)
        sp.run(['bzip2recover','sample.bz2'],cwd=dest,stdout=sp.DEVNULL,stderr=sp.DEVNULL,check=True)
        wr=csv.writer(f);wr.writerow(['round','block','arm','primary','rows','cap_records','pre_hwm_kib','post_hwm_kib'])
        for r in range(3):
            for q in rows:
                b=q['block']
                for i in range(6):
                    j=(r+i+b)%6
                    x=list(map(int,sp.check_output([rss,dest/f'rec{b+1:05d}sample.bz2',donors[j],ref/f'block-{b}.raw'],text=True,env=env).strip().split(',')))
                    assert x[:3]==[q['mark'],q['rows'],q['records']]
                    wr.writerow([r,b,all6[j],*x])
    (out/'rss-manifest.json').write_text(json.dumps({'rust_rss_executable':sha(rss),'so':sha(base/'libgrammar.so')},indent=2)+'\n')
else:raise ValueError(action)
