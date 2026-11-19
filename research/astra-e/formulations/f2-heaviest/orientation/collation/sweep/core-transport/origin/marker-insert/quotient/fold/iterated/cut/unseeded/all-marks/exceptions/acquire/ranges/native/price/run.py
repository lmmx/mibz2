import csv,hashlib,json,os,struct,subprocess,sys,tempfile
from pathlib import Path
here=Path(__file__).resolve().parent
repo=Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
action=sys.argv[1];out,models,masks,ref=map(lambda p:Path(p).resolve(),sys.argv[2:])
corpus=repo/'corpora/wikidata.bz2'
capped=next(repo.glob('research/astra-e/**/cut/powers/xor/capped/receipt.json'))
receipt=json.loads(capped.read_text())
sha=lambda p:hashlib.file_digest(p.open('rb'),'sha256').hexdigest()
def dump(path,obj):path.write_text(json.dumps(obj,indent=2)+'\n')
assert sha(corpus)==receipt['corpus_sha256']
inputs=receipt['inputs'];exe=out/'panel';end=here.parents[1]
real={int(r['block']):r for r in csv.DictReader((end/'real.csv').open()) if r['k']=='8'}
if action=='prep':
    # Freeze both source translation units of the independent membership-packet induction.
    older='dd836e65'
    independent=out/'refsrc'/'acquire'/'ranges'/'independent.cpp'
    induce=out/'refsrc'/'return'/'induce.cpp'
    for src,dest in [(end/'independent.cpp',independent),(end.parents[1]/'return/induce.cpp',induce)]:
        dest.parent.mkdir(parents=True,exist_ok=True)
        dest.write_bytes(subprocess.check_output(['git','-C',repo,'show',older+':'+str(src.relative_to(repo))]))
    reference=out/'independent'
    subprocess.run(['g++','-std=c++20','-O3','-g0',independent,'-o',reference],check=True)
    results=[]
    for row in inputs:
        i=row['block'];q=real[i];raw=models/'raw'/f'{i}.txt';wire=models/'data'/str(i)/'8.ctx';mask=masks/f'{i}.mask8'
        caps=ref/f'block-{i}.caps';plain=ref/f'block-{i}.raw'
        assert sha(caps)==row['caps_le_sha256'] and sha(plain)==row['raw_sha256']==q['raw_sha256']
        assert sha(wire)==q['context_sha256']
        lines=raw.read_text().splitlines();assert int(lines[0])==row['mark']
        original=[tuple(map(int,line.split())) for line in lines[1:]]
        compact=[]
        for pair, in struct.iter_unpack('<H',caps.read_bytes()):
            b,n=divmod(pair,256)
            if compact and compact[-1][0]==b:compact[-1]=(b,compact[-1][1]+n)
            else:compact.append((b,n))
        assert compact==original and len(original)==row['maximal_runs']
        block=b''.join(bytes([b])*w for b,w in original)
        maskwire=mask.read_bytes()
        sig,n,mark=struct.unpack_from('<8sII',maskwire)
        assert sig==b'CLKMASK1' and n==row['rows'] and mark==row['mark'] and maskwire[16:16+n]==block
        exceptions=maskwire[16+n:];assert len(exceptions)==n
        ctx=wire.read_bytes()
        fields=struct.unpack_from('<7I',ctx,8)
        assert fields[:3]==(n,mark,8)
        count=fields[3];bound=0;expected=bytearray()
        for stop,post,modal in struct.iter_unpack('<IIB',ctx[36:36+9*count]):
            expected.extend(c!=modal for c in block[bound:stop]);bound=stop
        assert bytes(expected)==exceptions and bound==n
        bands=out/f'{i}.bands';result=subprocess.check_output([reference,mask]);bands.write_bytes(result)
        assert sha(bands)==q['independent_band_sha256'] and len(result)==12*int(q['canonical_return_pieces'])
        results.append(dict(block=i,rows=n,mark=mark,cap_records=row['records'],original_runs=len(original),
            band_records=len(result)//12,covered_events=int(q['marked_events']),caps_sha256=sha(caps),original_rle_txt_sha256=sha(raw),
            inverse_sha256=sha(plain),ctx_sha256=sha(wire),independent_mask_sha256=sha(mask),bands_sha256=sha(bands),
            block_crc=row['block_crc']))
    dump(out/'inputs.json',dict(corpus_sha256=sha(corpus),reference_commit=older,reference_binary=sha(reference),blocks=results))
elif action in ('gate','price'):
    manifest=json.loads((out/'build.json').read_text())
    assert manifest['executable']==sha(exe) and manifest['shared']==sha(out/'libranges.so')
    assert len(json.loads((out/'inputs.json').read_text())['blocks'])==10
    if action=='price':assert len(list(csv.DictReader((out/'gate.csv').open())))==20
    subprocess.run([exe,'panel',corpus,models,ref,out,'0' if action=='gate' else '24'],check=True)
    if action=='price':dump(out/'runtime.json',dict(affinity=sorted(os.sched_getaffinity(0)),nice=os.getpriority(os.PRIO_PROCESS,0),
        thread_clock='CLOCK_THREAD_CPUTIME_ID',wall_clock='Rust Instant',kernel=os.uname().release))
elif action=='rss':
    assert len(list(csv.DictReader((out/'gate.csv').open())))==20
    with tempfile.TemporaryDirectory(dir=out) as tmp,(out/'rss.csv').open('x',newline='') as f:
        t=Path(tmp);(t/'input.bz2').symlink_to(corpus)
        subprocess.run(['bzip2recover','input.bz2'],cwd=t,check=True,capture_output=True)
        w=csv.writer(f,lineterminator='\n');w.writerow(('round','block','arm','before_kib','post_kib'))
        for r in range(3):
            for h,row in enumerate(inputs):
                i=row['block'];sample=t/f'rec{i+1:05d}input.bz2'
                for j in range(2):
                    arm=('ranges','scalar')[(j+h+r)%2]
                    text=subprocess.check_output([exe,'rss',arm,str(i),models,ref,out,sample],text=True)
                    b,a=map(int,text.strip().split(','));assert a>=b>0
                    w.writerow((r,i,arm,b,a))
else:raise ValueError(action)
