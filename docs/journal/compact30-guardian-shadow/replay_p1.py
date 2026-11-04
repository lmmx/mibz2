#!/usr/bin/env python3
"""Static archive replay. No constructor execution, compilation, or new measurement.

python replay_p1.py --repo /path/to/mibz2 --out /scratch/p1
Reads only pinned Git objects; writes JSON results/manifest to --out.
"""
import argparse, collections, hashlib, io, json, math, pathlib, re, statistics, subprocess, tarfile

ap = argparse.ArgumentParser(description=__doc__)
ap.add_argument('--repo', type=pathlib.Path, required=True)
ap.add_argument('--out', type=pathlib.Path, required=True)
a = ap.parse_args()
if not __debug__: ap.error('Assertions must be enabled')
a.out.mkdir(parents=True, exist_ok=True)
PIN = '931640523226053b758ebc0c2c9fd4eac61128b3'
manifest = []
def blob(path, pin=PIN):
    b = subprocess.check_output(['git','-C',str(a.repo),'show',pin+':'+path])
    manifest.append(dict(pin=pin,path=path,bytes=len(b),sha256=hashlib.sha256(b).hexdigest(),
                         git_blob=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()))
    return b
def asm(path):
    return [(int(m[1],16),m[2],m[3]) for l in blob(path).decode().splitlines()
            if (m:=re.match(r'^\s*([0-9a-f]+):\s+(\S+)\s*(.*)',l))]
def memory(ins):
    reads=writes=0
    for _,op,args in ins:
        if op.startswith(('lea','nop')) or op in ('push','pop','ret','call'): continue
        operands=re.split(r',\s*(?![^()]*\))',args.split('#')[0])
        for i,x in enumerate(operands):
            if '(' not in x: continue
            dst=i==len(operands)-1 and op not in ('cmp','test','jmp')
            writes+=dst; reads+=not dst or not op.startswith('mov')
    return dict(reads=reads,writes=writes)
E='docs/journal/2026-11-04-compact30-epoch-tagged-finaliser/'
B='docs/journal/2026-11-03-compact30-constructor-v2-prototype/'
leaf=asm(E+'listings/ctor3-finish-false.asm')
old=asm(B+'ctor2-finish-false.asm')
ranges=[(0x61d30,0x61d93),(0x61e08,0x61e17),(0x61f1b,0x61fb8),(0x6231b,0x62329)]
fast=[i for lo,hi in ranges for i in old if lo<=i[0]<=hi]
assert len(leaf)==44 and len(fast)==81
assert memory(leaf)==dict(reads=12,writes=6)
assert not any(op in ('push','pop','call') or '%rsp' in x for _,op,x in leaf)
assert leaf[-1][1]=='ret'
sizes={n:len(asm(E+'listings/'+n+'.asm')) for n in ['ctor3-build-false','ctor3-finish-slow-false','ctor3-refill-false']}
rows=[json.loads(l) for l in blob(E+'corpora.jsonl').splitlines() if l]
for r in rows:
    assert r['leaf_fast']==r['v2_fast_full']
    assert r['fin_fast']==r['leaf_fast']+r['slow_ordinary_partial_same_epoch']
    assert r['slow_entries']==r['slow_ordinary_old_epoch']+r['slow_ordinary_partial_same_epoch']+r['slow_raw_preset']
p,=[r for r in rows if r['input']=='wikidata-20261026.bz2' and r['permille']==0 and not r['extra_starts']]
f,s,q,e=(p[k] for k in ['leaf_fast','slow_entries','slow_ordinary_partial_same_epoch','epochs'])
gross=(len(fast)-len(leaf))*f
center=gross-5*(s-q)-12*q-4*e
mc=blob(E+'miri.console').decode()
cases=[int(x) for x in re.findall(r'^ctor3-synthetic: .*: (\d+) cases exact$',mc,re.M)]
assert cases==[127,4,2,1] and mc.endswith('exit 0\n')
mutation=blob(E+'mutation.console').decode()
assert 'assertion failed: (d.raw as usize) < self.raw.len()' in mutation and mutation.endswith('exit 101\n')
blob(E+'mutation.diff')
rebuild={}
for n in ['corpora.console','adversarial.console','synthetic.console','synthetic-quick.console']:
    normal=lambda b:[x for x in b.decode().splitlines() if x!='exit 0']
    rebuild[n]=normal(blob(E+n))==normal(blob(E+'rebuild/'+n))
assert all(rebuild.values())
v3=dict(leaf_instructions=len(leaf),body_target=40,with_call=len(leaf)+1,call_target=41,
        leaf_memory=memory(leaf),read_target=10,baseline_fast_instructions=len(fast),
        counts={k:p[k] for k in ['blocks','rows','leaf_fast','slow_entries','slow_ordinary_partial_same_epoch','epochs','opens_ordinary']},
        gross=gross,tag_exit_cost=5*(s-q),length_exit_cost=12*q,epoch_cost=4*e,
        center=center,interval=[center-s,center+s],miri_classes=cases,miri_cases=sum(cases),
        mutation_caught=True,rebuild_consoles_equal=rebuild,whole_listing_sizes=sizes,
        historical_compiler_outcome='STOP')

def interval(ratios):
    logs=list(map(math.log,ratios)); assert len(logs)==20
    mu=statistics.mean(logs); h=2.093*statistics.stdev(logs)/math.sqrt(len(logs))
    return dict(point=math.exp(mu),lo=math.exp(mu-h),hi=math.exp(mu+h))

def capture(path, version):
    b=blob(path)
    with tarfile.open(fileobj=io.BytesIO(b),mode='r:gz') as t:
        members=t.getmembers(); names=[m.name for m in members if m.isfile()]
        root=next(n.rsplit('/',1)[0] for n in names if n.count('/')==1 and n.endswith('/MANIFEST.json'))
        read=lambda n:t.extractfile(root+'/'+n).read()
        man=json.loads(read('MANIFEST.json'))
        for row in man['files']:
            data=read(row['path'])
            assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256'],row['path']
        records=[json.loads(l) for l in read('c30-construction.jsonl').splitlines() if l]
        # The raw calculations below do not consume c30-analysis.json.
    meta,=[r for r in records if r['kind']=='meta']
    assert meta['rounds']==20 and not meta['dry_run']
    builds=[r for r in records if r['kind']=='build' and 0<=r['round']<20]
    by=collections.defaultdict(list)
    for r in builds: by[r['round'],r['regime'],r['mode'],r['arm']].append(r)
    per={}; skews=[]
    for rnd in range(20):
      for regime in ['warm','cold']:
       for arm in ['P','C']:
        metrics={}
        for mode in ['w1','w4']:
            batch=by[rnd,regime,mode,arm]
            assert len(batch)==meta['blocks'] and sum(r['rows'] for r in batch)==meta['rows']
            assert {r['block'] for r in batch}==set(range(meta['blocks']))
            metrics[mode+'_cpu_ns_row']=sum(r['cpu_end']-r['cpu_start'] for r in batch)/meta['rows']
            if mode=='w4':
                steps=collections.defaultdict(list)
                for r in batch: steps[r['step']].append(r)
                metrics['w4_completion_ns']=sum(max(x['end'] for x in rs)-min(x['start'] for x in rs) for rs in steps.values())
                for step,rs in steps.items():
                    if len(rs)!=4 or not all(x['rows']>=899000 for x in rs): continue
                    start=min(x['start'] for x in rs); span=max(x['end'] for x in rs)-start
                    gap=max(x['start'] for x in rs)-start
                    skews.append(dict(round=rnd,regime=regime,arm=arm,step=step,rows=sum(x['rows'] for x in rs),gap_ns=gap,span_ns=span,skew=gap/span))
        per[rnd,regime,arm]=metrics
    stability={}
    keys=['w1_cpu_ns_row','w4_cpu_ns_row','w4_completion_ns']
    for regime in ['warm','cold']:
      for k in keys:
        vals=[per[r,regime,'P'][k] for r in range(20)]
        gated=version==1 or regime=='warm' and k!='w1_cpu_ns_row'
        stability[regime+'_'+k]=dict(max_over_min=max(vals)/min(vals),min_round=vals.index(min(vals)),max_round=vals.index(max(vals)),gated=gated,passes=max(vals)/min(vals)<=1.05)
    ratios={regime+'_'+k:interval([per[r,regime,'C'][k]/per[r,regime,'P'][k] for r in range(20)]) for regime in ['warm','cold'] for k in keys}
    if version==1:
        bad=[s for s in skews if s['skew']>0.02]
        g2=dict(steps=len(skews),over_2pct=bad,max_skew=max(s['skew'] for s in skews),passes=not bad)
    else:
        g2={}
        for arm in ['P','C']:
            ss=[s for s in skews if s['regime']=='warm' and s['arm']==arm]
            weighted=sum(s['rows']*s['skew'] for s in ss)/sum(s['rows'] for s in ss)
            outliers=sum(s['skew']>0.02 for s in ss); maximum=max(s['skew'] for s in ss)
            g2[arm]=dict(steps=len(ss),weighted_skew=weighted,outliers=outliers,allowance=len(ss)//1000,max_skew=maximum,passes=weighted<=0.02 and outliers<=len(ss)//1000 and maximum<=0.10)
        g2['passes']=all(g2[x]['passes'] for x in ['P','C'])
    failed=[k for k,v in stability.items() if v['gated'] and not v['passes']]
    assert failed or not g2['passes']
    return dict(archive_sha256=hashlib.sha256(b).hexdigest(),manifest_files_verified=len(man['files']),records=len(records),source=man['source'],
                stability=stability,g2=g2,ratios_descriptive_only=ratios,failed_G1=failed,formal_outcome='INVALID')

c1=capture('docs/journal/2026-11-03-compact30-construction-capture/c30-construction-20261103T172906Z.tar.gz',1)
c2=capture('docs/journal/2026-11-03-compact30-construction-v2-capture/c30-construction-v2-20261103T222928Z.tar.gz',2)
for n in ['docs/journal/2026-11-03-compact30-construction-protocol.md','docs/journal/2026-11-03-compact30-construction-protocol-v2.md',
          'docs/journal/2026-11-04-compact30-research-charter.md','docs/journal/compact30-channel/GUARDIAN.md',
          'docs/journal/compact30-channel/PROTOCOL.md','docs/journal/compact30-channel/REVIEWER.md']:
    blob(n)
out=dict(v3=v3,capture_v1=c1,capture_v2=c2,limits=['Unblinded replay: required specification revealed expected outcomes.','Archive verification only; no new execution or economic qualification.'])
manifest=sorted(manifest,key=lambda x:(x['pin'],x['path']))
mb=(json.dumps(manifest,indent=2,sort_keys=True)+'\n').encode()
(a.out/'p1-manifest.json').write_bytes(mb)
out['evidence_digest']=hashlib.sha256(mb).hexdigest()
(a.out/'p1-results.json').write_text(json.dumps(out,indent=2,sort_keys=True)+'\n')
print(json.dumps(out,indent=2,sort_keys=True))
