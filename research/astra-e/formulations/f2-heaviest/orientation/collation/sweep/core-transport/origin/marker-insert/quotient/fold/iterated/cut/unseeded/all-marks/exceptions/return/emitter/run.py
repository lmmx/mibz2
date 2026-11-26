"""Build and price the frozen physical-return emitters with the real Rust controls."""
import csv
import hashlib
import json
import math
import os
from pathlib import Path
import statistics
import subprocess as sp
import sys

here = Path(__file__).resolve().parent
repo = Path(sp.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
donor = here.parents[4]/'powers/xor/capped'
reference, wires, work = [Path(v).resolve() for v in sys.argv[1:4]]
stage = sys.argv[4]
work.mkdir(exist_ok=True,parents=True)
receipt = json.loads((donor/'receipt.json').read_text())
build = json.loads((reference/'build.json').read_text())
pinned = json.loads((donor/'build.json').read_text())
digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
for key in ('rustc','donor','src_tree','source_sha256','rlib_sha256','rust_flags'):
    assert build[key] == pinned[key]
assert digest(reference/'panel') == build['exe_sha256']
corpus = repo/'corpora/wikidata.bz2'
assert digest(corpus) == receipt['corpus_sha256']
rows = receipt['inputs']
rawdata = json.loads((here.parent/'inputs.json').read_text())
arms = ['physical','chronological','lap_layout','scalar','lazy','opend']
labels = dict(zip(['stream','streamgaps','dense','scalar','lazy','opend'],arms,strict=True))
if stage == 'build':
    source = here.parent.parent/'price/clock.cpp'
    assert digest(source)==json.loads((source.parent/'manifest.json').read_text())['source_sha256']
    (work/'clock-model.hpp').write_text(source.read_text().split('struct Stats{',1)[0])
    (work/'ids.inc').write_text('unsigned lookup(unsigned n,unsigned p,unsigned c){\n'+''.join(
        f"if(n=={q['rows']}u&&p=={q['mark']}u&&c=={q['records']}u)return {q['block']}u;\n"
        for q in rows)+'std::abort();}\n')
    flags=['-std=c++20','-O3','-g0','-Wall','-Wextra','-Werror']
    for options,name in [(['-fPIC','-shared'],'libgrammar.so'),(['-DSMALL_CHECK'],'small')]:
        sp.run(['g++',*flags,*options,'-I',work,here/'terminal.cpp','-o',work/name],check=True)
    manifest=dict(source_sha256=digest(here/'terminal.cpp'),
                  header_sha256=digest(work/'clock-model.hpp'),ids_sha256=digest(work/'ids.inc'),
                  so_sha256=digest(work/'libgrammar.so'),small_sha256=digest(work/'small'),
                  compiler=sp.check_output(['g++','--version'],text=True).splitlines()[0],
                  rust=build,flags=flags,corpus_sha256=digest(corpus))
    (work/'build.json').write_text(json.dumps(manifest,indent=2)+'\n')
elif stage in ('gate','price'):
    manifest=json.loads((work/'build.json').read_text())
    assert digest(work/'libgrammar.so')==manifest['so_sha256'] and digest(work/'small')==manifest['small_sha256']
    assert digest(here/'terminal.cpp')==manifest['source_sha256']
    inputs = []
    for q in rawdata:
        if q['k']!=8: continue
        b=q['index']
        model,mask=wires/f'{b}.pred8',wires/f'{b}.mask8'
        assert q['pred']==digest(model) and q['mask']==digest(mask)
        n,primary,states,events=__import__('struct').unpack_from('<IIII',model.read_bytes(),8)
        mr=mask.read_bytes();p=int.from_bytes(mr[12:16],'little')
        assert len(mr)==16+2*n
        artificial = 1-int(mr[16+n+p])
        inputs.append(dict(block=b,n=n,start_state=primary,primary=p,states=states,events=events,
                           artificial_primary=artificial,return_records=events+artificial,
                           default_bytes=4*states,jump_bytes=12*states,
                           chronological_bytes=8*events,physical_bytes=12*(events+artificial),
                           pred_sha256=q['pred'],mask_sha256=q['mask']))
    env=dict(os.environ,CLOCK_WIRES=str(wires),LD_LIBRARY_PATH=str(work))
    assert f'libgrammar.so => {work}/libgrammar.so' in sp.check_output(['ldd',reference/'panel'],env=env,text=True)
    out=work/stage;out.mkdir(exist_ok=True)
    if stage=='gate':
        (work/'small.csv').write_bytes(sp.check_output([work/'small']))
    sp.run([reference/'panel',corpus,out,'0' if stage=='gate' else '36'],env=env,check=True)
    for q,v in zip(rows,inputs,strict=True):
        b=q['block'];assert v['block']==b and v['n']==q['rows'] and v['primary']==q['mark']
        for ext,key in [('caps','caps_le_sha256'),('raw','raw_sha256')]:
            assert digest(out/f'block-{b}.{ext}')==q[key];v[key]=q[key]
        v['libbzip2_plain_sha256']=q['decoded_sha256'];v['block_crc']=q['block_crc']
    (out/'inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
elif stage=='report':
    f=work/'price/times.csv'
    observations=list(csv.DictReader(f.open()))
    by={}
    assert len(observations)==36*6*10
    for v in observations:
        k=(int(v['round']),labels[v['arm']],int(v['block']))
        assert k not in by
        r,arm,b=k;slot=int(v['slot'])
        assert arms[((r+b)+(slot if (r//6)%2==0 else 5-slot))%6]==arm
        by[k]=int(v['thread_ns'])
    totals={a:[sum(by[r,a,q['block']] for q in rows)for r in range(1,37)]for a in arms}
    def compare(a,b):
        logs=[math.log(x/y)for x,y in zip(totals[a],totals[b],strict=True)]
        group=[statistics.mean(logs[i:i+6])for i in range(0,36,6)]
        m=statistics.mean(group);half=2.570581835636305*statistics.stdev(group)/math.sqrt(len(group))
        return dict(ratio=math.exp(m),lo=math.exp(m-half),hi=math.exp(m+half),groups=[math.exp(x)for x in group])
    summary=dict(rounds=36,raw_arms=labels,
                 median_ten_block_ms={a:statistics.median(x)/1e6 for a,x in totals.items()},
                 comparisons={f'{a}/{b}':compare(a,b)for a,b in [('physical','scalar'),('chronological','scalar'),
                            ('physical','chronological'),('lap_layout','chronological'),('physical','lap_layout'),
                            ('physical','opend'),('physical','lazy')]})
    print(json.dumps(summary,indent=2))
else:
    raise ValueError(stage)
