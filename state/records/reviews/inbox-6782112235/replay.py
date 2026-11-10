"""Replay archived data only; never executes a decoder or measures elapsed time."""
import json, math, statistics, pathlib, hashlib
p = pathlib.Path('cells-raw/20261110T175613Z')
c = pathlib.Path('cells-ci')
def rows(path):
    return [json.loads(s) for s in path.read_text().splitlines() if s.startswith('{')]
def route(x):
    b = x['n'] * 4 // 25
    return ('column' if not x['takes'] else 'density' if x['records'] * 10 >= x['n']
            else 'floor' if x['c_letters'] - 1 > b else 'budget' if x['c_cells'] > b else 'admitted')
out = {'scope':'archive arithmetic; admission predicted on census, not measured in CI', 'digests':{}, 'i9':[], 'ci':{}}
for f in ['cells-evidence.tar.gz','cells-final.zip']:
    out['digests'][f] = hashlib.sha256(pathlib.Path(f).read_bytes()).hexdigest()
for repeat in range(1,4):
    rr = rows(p / f'census-{repeat}.jsonl'); n = sum(x['n'] for x in rr)
    total = sum(x['ns_rec'] * x['n'] for x in rr)
    groups = {k:[x for x in rr if route(x)==k] for k in ['column','density','floor','budget','admitted']}
    a = groups['admitted']; budget = groups['budget']
    gain = sum((x['ns_rec']-x['c_whole'])*x['n'] for x in a)
    proxy = sum((x['c_order']+x['c_induce']*(x['n']*4//25)/x['c_cells'])*x['n'] for x in budget)
    high = 0; growths = []
    for i,x in enumerate(rr):
        if route(x) in ['column','density']: continue
        size = 48*x['records']+80*(x['n']*4//25)+4528
        if size > high: growths.append({'index':i,'requested_bytes':size}); high=size
    out['i9'].append({'repeat':repeat,'rows':n,'blocks':len(rr),
      'whole_ratio':sum(x['c_whole']*x['n'] for x in rr)/total,
      'phases_ns_row':{k:sum(x[k]*x['n'] for x in rr)/n for k in ['c_order','c_induce','c_write','c_whole','ns_rec','ns_front']},
      'groups':{k:{'blocks':len(v),'rows':sum(x['n'] for x in v)} for k,v in groups.items()},
      'admitted_row_share':sum(x['n'] for x in a)/n,
      'admitted_production_time_share':sum(x['ns_rec']*x['n'] for x in a)/total,
      'ideal_selected_ratio':1-gain/total,'linear_budget_proxy_ratio':1-(gain-proxy)/total,
      'gain_ns_per_all_rows':gain/n,'budget_proxy_ns_per_all_rows':proxy/n,
      'predicted_buffer_growths':growths,'sum_requested_growth_bytes':sum(x['requested_bytes'] for x in growths)})
ci=rows(c/'cells.jsonl')
for corpus in sorted({x['corpus'] for x in ci}):
    pairs={}
    for x in ci:
        if x['corpus']==corpus: pairs.setdefault(x['round'],{})[x['arm']]=x
    out['ci'][corpus]={'pairs':len(pairs),'wall_ratio':math.exp(statistics.mean(math.log(a['mibz2-cells']['wall']/a['mibz2@bin/ab']['wall']) for a in pairs.values())),
      'decoded_bytes':next(iter(pairs.values()))['mibz2-cells']['out'],
      'hashes_match':all(a['mibz2-cells']['hash']==a['mibz2@bin/ab']['hash'] for a in pairs.values())}
out['amdahl_scenarios']=[{'f':f,'R_s815':1-f*(1-.815),'R_s827':1-f*(1-.827)} for f in [.1,.25,.5,1.0]]
pathlib.Path('cells-gap-arithmetic.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'digests':out['digests'],'ci':out['ci'],'amdahl':out['amdahl_scenarios']},indent=2))
