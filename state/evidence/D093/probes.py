import collections,json,random,re,subprocess,hashlib,pathlib
ROOT=pathlib.Path(__file__).resolve().parent
named={(0x2100,0x1100),(0x2104,0x1104),(0x21fc,0x1100)}
regs='SDH'
def region(a):return 'S' if a>=0x1000000000 else 'D' if 0x100000<=a<0x1000000 else 'H'
def run(seed):
 r=random.Random(seed);ring=[];trace=[];cl=collections.Counter();pop=collections.defaultdict(lambda:[0]*6);pairs=collections.defaultdict(lambda:[0,0]);ld=collections.Counter()
 for i in range(600):
  kind=r.choice('SSSLLM');pc=r.choice([0x2100,0x2104,0x2108]) if kind=='S' else r.choice([0x1100,0x1104,0x1108]);a=r.choice([0x1000002000,0x1000005000,0x100000,0x105000,0x2000000,0x2005000])+r.choice([0,0x100,0x104,0x3fc,0xffc]);sz=r.choice([1,4,16,32,64]);trace.extend([f'I {pc:x},4',f' {kind} {a:x},{sz}'])
  if kind in 'LM':
   lr=region(a);ld[lr]+=1
   for mod in [4096,1024]:
    hits=collections.defaultdict(list)
    for spc,b,z in ring:
     if a<b+z and b<a+sz:continue
     if set((a+j)%mod for j in range(sz)) & set((b+j)%mod for j in range(z)):hits[region(b)].append(spc)
    for sr,spcs in hits.items():
     key=(mod,lr,sr);cl[key]+=1;n=[s for s in spcs if(s,pc) in named];o=[s for s in spcs if(s,pc) not in named];inc=[1,bool(n),bool(o),bool(n and o),len(n),len(o)];pop[key]=[x+y for x,y in zip(pop[key],inc)]
     for spc in set(spcs):
      p=pairs[(mod,lr,sr,spc,pc)];p[0]+=1;p[1]+=spcs.count(spc)
  if kind in 'SM':ring=(ring+[(pc,a,sz)])[-48:]
 data='\n'.join(trace)+'\n';args=['--named='+','.join(f'{s:x}:{l:x}' for s,l in sorted(named)),'mtf_avx2:1000:2000'];p=subprocess.run([str(ROOT/'agg3')]+args,input=data,text=True,capture_output=True,check=True);out=p.stdout
 got={}
 for line in out.splitlines():
  m=re.fullmatch(r'NAMED mtf_avx2 mod(4K|1K) ([SDH])<-([SDH]): class=(\d+) named=(\d+) outside=(\d+) both=(\d+) \| weight named=(\d+) outside=(\d+)',line)
  if m:got[(4096 if m[1]=='4K' else 1024,m[2],m[3])]=list(map(int,m.groups()[3:]))
 assert len(got)==18
 for mod in [4096,1024]:
  for lr in regs:
   for sr in regs:assert got[(mod,lr,sr)]==pop[(mod,lr,sr)],(seed,mod,lr,sr,got[(mod,lr,sr)],pop[(mod,lr,sr)])
 gp={};gn={}
 for line in out.splitlines():
  m=re.fullmatch(r'(PAIR|NAMEDPAIR) mtf_avx2 mod(4K|1K) ([SDH])<-([SDH]): store ([0-9a-f]+) load ([0-9a-f]+) loads=(\d+) weight=(\d+)(?: named=(\d+))?',line)
  if m:
   key=(4096 if m[2]=='4K' else 1024,m[3],m[4],int(m[5],16),int(m[6],16));v=list(map(int,[m[7],m[8]]));(gp if m[1]=='PAIR' else gn)[key]=v
   if m[1]=='PAIR':assert int(m[9])==int((key[3],key[4]) in named)
 assert gp==dict(pairs),(seed,'pair mismatch')
 assert len(gn)==54
 for key,v in gn.items():assert v==pairs.get(key,[0,0])
 assert out.splitlines()[-1]==f'END agg3 complete lines={len(trace)} pairs={len(pairs)} named=3'
 old=subprocess.run([str(ROOT/'agg3-old'),'mtf_avx2:1000:2000'],input=data,text=True,capture_output=True,check=True).stdout
 legacy=lambda s:'\n'.join(l for l in s.splitlines() if l.startswith(('mtf_avx2 loads','  mod ','PAIRS ')))
 assert legacy(old)==legacy(out),(seed,'legacy changed')
 return {'seed':seed,'events':600,'named_class_checks':18,'named_pair_checks':54,'pairs':len(pairs),'legacy_equal':True,'trace_sha256':hashlib.sha256(data.encode()).hexdigest()}
result={'seeds':[run(s) for s in range(8)],'historical_excess_per_byte':(56154-13829)/890000,'E2_integer':380800,'scope':'Synthetic traces only; low PCs, configured image and store widths <=64; no decoder, trace capture or timing.'}
(ROOT/'probes.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
