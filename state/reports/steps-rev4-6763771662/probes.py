"""Independent finite-trace oracle for pinned agg3 revision 4; no decoder or timing."""
import collections, hashlib, json, pathlib, random, re, subprocess
P=pathlib.Path(__file__).resolve().parent
REG='SDH'
def reg(a): return 0 if a>=0x1000000000 else 1 if 0x100000<=a<0x1000000 else 2
def oracle(events,named):
    ring=collections.deque(maxlen=48); loads=collections.Counter(); counts=collections.Counter(); pairs=collections.Counter(); weights=collections.Counter(); pops={}; anyc=collections.Counter()
    for pc,k,a,z in events:
        if k in 'LM' and 0x1000<=pc<0x2000:
            lr=reg(a); loads[lr]+=1; matches=collections.defaultdict(list)
            for spc,b,bz in ring:
                if max(a,b)<min(a+z,b+bz): continue
                for mi,m in enumerate((4096,1024)):
                    if {x%m for x in range(a,a+z)} & {x%m for x in range(b,b+bz)}:
                        matches[(lr,reg(b),mi)].append((spc,pc))
            for mi in (0,1): anyc[mi]+=any(t[2]==mi for t in matches)
            for t,pp in matches.items():
                counts[t]+=1
                n=any(x in named for x in pp); o=any(x not in named for x in pp)
                v=pops.setdefault(t,[0,0,0,0,0]); v[0]+=n;v[1]+=o;v[2]+=n and o
                v[3]+=sum(x in named for x in pp);v[4]+=sum(x not in named for x in pp)
                for pair in set(pp): pairs[(t,pair)]+=1
                for pair in pp: weights[(t,pair)]+=1
        if k in 'SM': ring.append((pc,a,min(z,64)))
    return loads,counts,pairs,weights,pops,anyc
def one(seed):
    r=random.Random(seed); events=[]; named={(0x2000,0x1100),(0x2001,0x1100),(0x2000,0x1101),(0x2fff,0x1100)}
    # Deliberate repeated same-PC and overlapping named/outside stores, then random ring churn.
    for pc in (0x2000,0x2000,0x2001,0x2002): events.append((pc,'S',0x2000100,4))
    events.append((0x1100,'L',0x100100,16))
    for j in range(1200):
        k=r.choice('SSLLM');pc=r.choice((0x1100,0x1101,0x1102,0x2000,0x2001,0x2002))
        a=r.choice((0x1000001000,0x100000,0x2000000))+r.choice((0,0x100,0x108,0x3fc,0x400,0xffc))
        events.append((pc,k,a,r.choice((1,4,8,16,32))))
    trace=''.join(f'I {pc:x},4\n {k} {a:x},{z}\n' for pc,k,a,z in events)
    args=['--named='+','.join(f'{a:x}:{b:x}' for a,b in sorted(named)),'mtf_avx2:1000:2000']
    p=subprocess.run([str(P/'agg3'),*args],input=trace,text=True,capture_output=True,check=True)
    assert not p.stderr and p.stdout.splitlines()[-1].startswith('END agg3 complete ')
    ll,cc,pp,ww,pops,aa=oracle(events,named)
    out=p.stdout.splitlines(); assert out[0]==f'mtf_avx2 loads S={ll[0]} D={ll[1]} H={ll[2]} | any4K={aa[0]} any1K={aa[1]}'
    for mi,mod in enumerate(('4K','1K')):
        expected='  mod '+mod+' (load<-store):'+''.join(f' {REG[lr]}<-{REG[sr]}={cc[lr,sr,mi]}' for lr in range(3) for sr in range(3) if cc[lr,sr,mi])
        assert out[mi+1]==expected
        for lr in range(3):
            for sr in range(3):
                t=(lr,sr,mi); n,o,b,nw,ow=pops.get(t,[0]*5)
                expected=f'NAMED mtf_avx2 mod{mod} {REG[lr]}<-{REG[sr]}: class={cc[t]} named={n} outside={o} both={b} | weight named={nw} outside={ow}'
                assert expected in out
                for sp,lp in named:
                    expected=f'NAMEDPAIR mtf_avx2 mod{mod} {REG[lr]}<-{REG[sr]}: store {sp:x} load {lp:x} loads={pp[t,(sp,lp)]} weight={ww[t,(sp,lp)]}'
                    assert expected in out
    expected=set()
    for (t,(sp,lp)),v in pp.items():
        lr,sr,mi=t
        expected.add(f'PAIR mtf_avx2 mod{("4K","1K")[mi]} {REG[lr]}<-{REG[sr]}: store {sp:x} load {lp:x} loads={v} weight={ww[t,(sp,lp)]} named={int((sp,lp) in named)}')
    assert set(x for x in out if x.startswith('PAIR '))==expected
    # Old source compiled in the previous review, SHA checked separately. Compare unchanged rows.
    old=subprocess.run([str(P/'agg3-old'),'mtf_avx2:1000:2000'],input=trace,text=True,capture_output=True,check=True).stdout.splitlines()
    legacy=[x for x in out if x.startswith(('mtf_avx2 loads','  mod','PAIRS '))]
    assert old==legacy
    return dict(seed=seed,events=len(events),pair_rows=len(expected),oracle='PASS',legacy='byte-identical')
results=[one(s) for s in range(8)]
history=json.loads((P/'history.json').read_text())
historical={}
for name,s in history.items():
    part=s.split('mtf.mtf_avx2 loads ',1)[1].splitlines()[1]
    historical[name]=int(re.search(r'D<-H=(\d+)',part)[1])
assert historical=={'sc-master-rand-0.txt':13829,'sc-head-rand-0.txt':56154}
# Inclusion-exclusion: K contrast = C + O contrast - overlap contrast.
# E1 implies N_hi <= C/9; S1 |O contrast| <= C and overlap_hi <= N_hi.
# Therefore K contrast <= (19/9)*C. With geometry N_hi=0, <=2*C.
small_checks=0
for nh in range(4):
  for nl in range(10*nh,41):
    c=nl-nh
    if c<=0: continue
    for ol in range(12):
      for oh in range(12):
        if abs(ol-oh)>c: continue
        for bl in range(min(nl,ol)+1):
          for bh in range(min(nh,oh)+1):
            kd=c+ol-oh-bl+bh
            assert 9*kd<=19*c
            if nh==0: assert kd<=2*c
            small_checks+=1
result={'source_pin':'b23897e89a292221836685eab0f1c8ce944802d2','hashes':{n:hashlib.sha256((P/n).read_bytes()).hexdigest() for n in ('agg3.c','test_agg3.sh')},'oracle_traces':results,'historical_class_counts':historical,'historical_excess_per_byte':(56154-13829)/890000,'rounded_reference':0.0476,'E2_loads':380800,'integer_named_contrast_lower_bound':(9*380800+18)//19,'if_named_hi_zero_lower_bound':190400,'inclusion_exclusion_cases':small_checks}
(P/'probes.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
