from pathlib import Path
import csv,hashlib,json,math,os,statistics,sys,subprocess as sp
here=Path(__file__).resolve().parent
base=here.parent
ref,work=map(lambda s:Path(s).resolve(),sys.argv[1:3]);stage=sys.argv[3]
work.mkdir(exist_ok=True,parents=True)
pub=base.parent/'recursive/native'
known=json.loads((pub/'inputs.json').read_text())
build=json.loads((pub/'build.json').read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(ref/'panel')==build['rust_panel']
repo=Path(sp.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
corpus=repo/'corpora/wikidata.bz2'
prior=json.loads((base.parents[1]/'emitter/build.json').read_text());assert sha(corpus)==prior['corpus_sha256']
ids=list(csv.DictReader((pub/'raw/times.csv').open()))
geom={int(v['block']):v for v in csv.DictReader((base/'seed-blocks.csv').open())if v['k']=='8'}
forest={int(v['block']):v for v in csv.DictReader((base/'forest-blocks.csv').open())if v['k']=='8'}
names=['seed_only','forest_supplied','complete','scalar','lazy','opend']
labels=dict(zip(['stream','streamgaps','dense','scalar','lazy','opend'],names,strict=True))
if stage=='build':
 caps={int(v['block']):v for v in ids}
 inc='unsigned lookup(unsigned n,unsigned p,unsigned c){\n'
 for q in known:
  b=q['block'];v=caps[b];inc+=f"if(n=={v['rows']}u&&p=={v['mark']}u&&c=={v['cap_records']}u)return {b}u;\n"
 (work/'ids.inc').write_text(inc+'std::abort();}\n');assert sha(work/'ids.inc')==build['ids_sha256']
 flags=['-std=c++20','-O3','-g0','-Wall','-Wextra','-Werror','-fPIC','-shared']
 sp.run(['g++',*flags,'-I',work,here/'worker.cpp','-o',work/'libgrammar.so'],check=True)
 code={p.name:sha(p)for p in [here/'worker.cpp',here/'run.py',base/'seeds.hpp',base/'forest.hpp']}
 (work/'build.json').write_text(json.dumps(dict(code=code,flags=flags,compiler=sp.check_output(['g++','--version'],text=True).splitlines()[0],
                                     rust=sha(ref/'panel'),lib=sha(work/'libgrammar.so')),indent=2)+'\n')
elif stage in ('gate','time'):
 out=work/stage;out.mkdir(exist_ok=True)
 manifest=json.loads((work/'build.json').read_text());assert sha(work/'libgrammar.so')==manifest['lib']
 assert all(sha(x)==manifest['code'][x.name]for x in [here/'worker.cpp',here/'run.py',base/'seeds.hpp',base/'forest.hpp'])
 env=dict(os.environ,LD_LIBRARY_PATH=str(work),SEED_REF=str(ref),SEED_STAMPS=str(out))
 assert f'libgrammar.so => {work}/libgrammar.so' in sp.check_output(['ldd',ref/'panel'],env=env,text=True)
 sp.run([ref/'panel',corpus,out,'0'if stage=='gate'else'6'],env=env,check=True)
 for q in known:
  b=q['block'];assert sha(out/f'block-{b}.raw')==q['raw_sha256'] and sha(out/f'block-{b}.caps')==q['caps_sha256']
 for j in range(3):
  stamps=list(csv.DictReader((out/f'phases-{j}.csv').open()));assert len(stamps)==(10 if stage=='gate'else 70)
  for v in stamps:
   b=int(v['block']);g=geom[b];f=forest[b]
   assert all(int(v[a])==int(g[c])for a,c in [('mixed','mixed_contexts'),('seeds','seed_cylinders'),('events','exception_rows')])
   assert (int(v['classes']),int(v['runs']))==((0,0)if j==0 else (int(f['classes']),int(f['output_runs'])))
elif stage=='report':
 obs=list(csv.DictReader((work/'time/times.csv').open()));assert len(obs)==360
 it={a:iter(list(csv.DictReader((work/f'time/phases-{j}.csv').open()))[10:])for j,a in enumerate(names[:3])};tot={}
 for v in obs:
  r,b,j=int(v['round']),int(v['block']),int(v['slot']);a=labels[v['arm']]
  assert a==names[((r+b)+(j if (r//6)%2==0 else 5-j))%6]
  def add(key,ns):tot.setdefault(key,[0]*6)[r-1]+=ns
  ns=int(v['thread_ns']);add(a,ns)
  if a in it:
   z=next(it[a]);assert int(z['block'])==b
   phases=[int(z[k+'_ns'])for k in ['index','classify','forest','terminal']];assert sum(phases)<=ns
   for k,u in zip(['index','classify','forest','terminal'],phases,strict=True):add(a+'_'+k,u)
 assert all(next(g,None)is None for g in it.values())
 res={}
 for a,v in tot.items():
  if a=='scalar':continue
  raw=[x/y for x,y in zip(v,tot['scalar'],strict=True)];logs=list(map(math.log,raw))
  m=statistics.mean(logs);h=2.5705818356363*statistics.stdev(logs)/math.sqrt(6)
  res[a]=dict(ratio=math.exp(m),lo=math.exp(m-h),hi=math.exp(m+h),rounds=raw)
 print(json.dumps(dict(labels=labels,median_ms={a:statistics.median(v)/1e6 for a,v in tot.items()},vs_scalar=res),indent=2))
else:raise ValueError(stage)
