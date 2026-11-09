"""Research-only arithmetic and malformed-evidence probes; no decoder or timing."""
import importlib.util, sys, json, shutil, unittest
from pathlib import Path
ROOT = Path(__file__).resolve().parent
def module(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / file)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod
pd = module('plan_diagnostics', 'diagnostics.py')
t = module('fixtures', 'tests.py')
# A deterministic checkout identity substitutes only for the fixture's ambient git lookup.
t.HEAD = 'a' * 40
pd.head_commit = lambda: t.HEAD
result = unittest.TextTestRunner(verbosity=1).run(unittest.defaultTestLoader.loadTestsFromModule(t))
assert result.wasSuccessful()
out = {'existing_tests': result.testsRun, 'mutations': {}}
for name in ['missing_output', 'missing_wall', 'different_base_per_cell']:
    islevel = name == 'different_base_per_cell'
    w = t.attempt(pd.LEVELS if islevel else pd.BOUNDARIES, base=islevel)
    try:
        for f in w.glob('runs-*.jsonl'):
            rows = [json.loads(x) for x in f.read_text().splitlines()]
            for r in rows:
                if name == 'missing_output':
                    r.pop('hash'); r.pop('out')
                elif name == 'missing_wall': r.pop('wall')
                elif pd.arm_key(r['arm']) == 'base' and f.stem == 'runs-L20':
                    r['arm'] = 'mibz2@/different/base'
            f.write_text('\n'.join(map(json.dumps, rows)) + '\n')
        v = (pd.levels if islevel else pd.boundaries)(str(w))[0]
        out['mutations'][name] = {'verdict': v['verdict'], 'readings': v['readings']}
        assert v['verdict'] == 'VALID'
    finally: shutil.rmtree(w)
p = module('plans', 'plans.py')
def independent_a(n, cores, cache):
    if cores is None or min(n, cores) < 6: return ('workers', n)
    old = min(min(n, cores)//2, max(1, round(7*cache/36000000)))
    fast = cache//6000000
    feasible = [q for q in range(2, old) if q == fast and 3*q <= n and 3*q < cores]
    return ('teams', feasible[0], 3, True) if feasible else ('teams', old, 2, False)
count = 0
for cores in range(1, 65):
    for m in range(2, 129):
        for n in range(1, 2*cores+1):
            assert independent_a(n, cores, m*p.MIB) == p.rule_a(n, cores, m*p.MIB)
            count += 1
cliffs=[]
for cores in range(6,65):
    for m in range(3,129):
        a,b=independent_a(2*cores,cores,(m-1)*p.MIB), independent_a(2*cores,cores,m*p.MIB)
        if a[0]==b[0]=='teams' and a[2]==3 and b[2]==2:
            ratio=p.score(b[1],b[2],m*p.MIB)/p.score(a[1],a[2],m*p.MIB)
            if ratio<1: cliffs.append([cores,m-1,m,list(a),list(b),ratio])
out['independent_rule_a_cells']=count
out['cliffs']={'count':len(cliffs),'min':min(x[-1] for x in cliffs),'max':max(x[-1] for x in cliffs),'examples':cliffs[:3]}
out['published_low_level_wall_increase']={str(s):1/s-1 for s in [.763,.765,.776,.772]}
print(json.dumps(out,indent=2))
