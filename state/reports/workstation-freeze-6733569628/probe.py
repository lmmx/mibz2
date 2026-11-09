import sys, json, copy, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent / 'bench'))
import test_run_fed_leader_ws as t
results=[]
def check(label, mutate):
    cap=t.capture('dir'); mutate(cap)
    code,text,out=t.run(t.records(),t.identity(),cmd='admit-occupancy',sub='dir',caps={'dir':cap})
    results.append(dict(case=label,exit=code,stdout=text,admission=out))
check('baseline',lambda c: None)
check('missing perturbation',lambda c:c['results'][0].pop('perturbation'))
check('unknown perturbation verdict',lambda c:c['results'][0]['perturbation']['full'].update(verdict='unknown: not measured'))
check('native too few rounds verdict',lambda c:c['results'][0]['perturbation']['full'].update(verdict='too few rounds',throughput_ratio={'ratio':None,'lo':None,'hi':None}))
check('duplicate only one round and mode',lambda c:c.update(runs=[copy.deepcopy(c['runs'][0]) for _ in range(18)]))
check('all capture wall clocks NaN',lambda c:[r.update(line=r['line'].replace('wall=0.027','wall=NaN')) for r in c['runs']])
print(json.dumps(results,indent=2))
