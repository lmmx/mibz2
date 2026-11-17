import json,pathlib,math,statistics,importlib.util,re
from scipy.stats import t as student
root=pathlib.Path(__file__).resolve().parent
results={}
for name in ['window','low','cores']:
 p=next((root/name).glob('run*'));rows=[json.loads(l) for l in (p/'judged.jsonl').read_text().splitlines()]
 idx={(r['corpus'],r['threads'],r['arm'],r['round']):r for r in rows}
 ref='mibz2-occ-8-32' if name=='low' else 'mibz2'
 candidates={'window':['mibz2-occ-8-32','mibz2-occ-9-72','mibz2-occ-9-18'],'low':['mibz2-occ-7-28','mibz2-occ-6-24'],'cores':['mibz2-occ-9-36','mibz2-occ-8-32','mibz2-occ-7-28','mibz2-occ-6-24']}[name]
 counts=[8,10] if name=='cores' else [12,16,20]
 def cell(c,t,a,b):
  ar=[idx[c,t,a,i] for i in range(10)];br=[idx[c,t,b,i] for i in range(10)]
  logs=[math.log(x['wall']/y['wall']) for x,y in zip(ar,br)]
  mu=statistics.mean(logs);se=statistics.stdev(logs)/math.sqrt(10);h=student.ppf(.975,9)*se
  return dict(gm=math.exp(mu),lo=math.exp(mu-h),hi=math.exp(mu+h),p=float(student.cdf(mu/se,9)),rss=statistics.median(x['rss_kb'] for x in ar)/statistics.median(x['rss_kb'] for x in br),cpu=math.exp(statistics.mean(math.log(x['cpu']/y['cpu']) for x,y in zip(ar,br))))
 out={}
 for a in candidates:
  for t in counts:
   if name=='cores' and ((a in ['mibz2-occ-9-36','mibz2-occ-8-32'] and t==8) or (a=='mibz2-occ-6-24' and t==10)):continue
   for c in ['wikidata-x4','text']:out[f'{a}:{t}:{c}']=cell(c,t,a,ref)
 ps=sorted((v['p'],k) for k,v in out.items() if k.endswith('wikidata-x4'));reject=[]
 for i,(pval,k) in enumerate(ps):
  if pval>.05/(len(ps)-i):break
  reject.append(k)
 results[name]={'cells':out,'holm_reject':reject}
 if name=='low':
  results[name]['context_7_vs_9']={f'{t}:{c}':cell(c,t,'mibz2-occ-7-28','mibz2') for t in counts for c in ['wikidata-x4','text']}
 if name=='cores':
  shares={}
  for arm,w in [('mibz2',10),('mibz2-occ-9-36',9),('mibz2-occ-8-32',8),('mibz2-occ-7-28',7),('mibz2-occ-6-24',6)]:
   us=[];os=[]
   for x in rows:
    if x['arm']!=arm or x['threads']!=10 or x['corpus']!='wikidata-x4':continue
    cs=[(k,float(v)) for k,v in (s.split('=') for s in x['thread_cpu'].split())]
    worker=sum(v for k,v in cs if re.fullmatch(r'mibz2-\d+',k))
    other=sum(v for k,v in cs if not re.fullmatch(r'mibz2-\d+',k))
    us.append(worker/x['wall']/w);os.append(other/x['wall'])
   shares[arm]={'worker_share':statistics.median(us),'other_cpu_per_wall':statistics.median(os)}
  results[name]['thread_cpu_samples']=shares
 spec=importlib.util.spec_from_file_location(name,root/f'source-{name}'/'bench/two_arm.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 for tval in [-20,-5,-2,0,2,5,20]:assert abs(m.t_cdf(tval,9)-student.cdf(tval,9))<1e-12
 print(name,'independent Holm',reject)
print('same-run context 7/9',json.dumps(results['low']['context_7_vs_9'],indent=2))
(root/'independent.json').write_text(json.dumps(results,indent=2)+'\n')
