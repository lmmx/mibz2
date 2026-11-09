import sys,pathlib,json,copy,math,random
sys.path.insert(0,str(pathlib.Path(__file__).parent/'bench'))
import test_run_fed_leader_ws as t
results=[]
def check(label,mutate,code,interpretable=None,slow=None):
 c=t.capture('dir',slow=slow);mutate(c)
 rc,txt,out=t.run(t.records(),t.identity(),cmd='admit-occupancy',sub='dir',caps={'dir':c})
 assert rc==code,(label,rc,txt)
 if interpretable is not None: assert out['interpretable']==interpretable
 if code: assert out.get('interpretable') is not True
 results.append({'case':label,'exit':rc,'interpretable':out.get('interpretable'),'stdout':txt})
check('numeric baseline',lambda c:None,0,True)
check('full genuinely perturbs',lambda c:None,0,False,slow='full')
check('missing perturbation',lambda c:c['results'][0].pop('perturbation'),2)
check('unknown verdict',lambda c:c['results'][0]['perturbation']['full'].update(verdict='unknown'),2)
check('too few rounds',lambda c:c['results'][0]['perturbation']['full'].update(verdict='too few rounds',throughput_ratio={'ratio':None,'lo':None,'hi':None}),2)
check('18 copies of round zero off',lambda c:c.update(runs=[copy.deepcopy(c['runs'][0]) for _ in range(18)]),2)
check('NaN clocks',lambda c:[r.update(line=r['line'].replace('wall=','wall=NaN ignored=')) for r in c['runs']],2)
check('missing cell',lambda c:c['runs'].pop(),2)
check('incorrect interval',lambda c:c['results'][0]['perturbation']['full']['throughput_ratio'].update(hi=2),2)
# Separate implementation of the instrument's n=6 formula; no imported t-table.
rng=random.Random(27);largest=0
for i in range(100):
 xs=[math.exp(rng.uniform(-.7,.7)) for _ in range(6)]
 logs=[math.log(x) for x in xs];mean=sum(logs)/6
 h=2.571*math.sqrt(sum((x-mean)**2 for x in logs)/5)/math.sqrt(6)
 independent=tuple(math.exp(x) for x in (mean,mean-h,mean+h))
 actual=t.ws.rfl.geo_ci(xs)
 largest=max(largest,max(abs(a-b) for a,b in zip(independent,actual)))
assert largest<1e-12
print(json.dumps({'cases':results,'independent_formula_cases':100,'max_abs_error':largest},indent=2))
