"""Independent deterministic counterexamples, exact PR318 source; no hardware timing."""
import contextlib, io, json, tempfile, math
from pathlib import Path
import two_modes as tm
import test_two_modes as T

out = {}

# Fixed work per sample: one long stall is only one item in an unweighted median.
k, hz, n, base = 1 << 20, 3.7e9, 700, 808277
normal = [base] * n
stalled = normal.copy()
stalled[n//2] += n*base
def read(ds):
    p = {'first_tsc':0,'iters_per_sample':k,'dtsc':ds}
    pairs = tm.probe_rates(p,0,sum(ds),hz)
    return {'median_giter_s':tm.med(v for _,v in pairs),
            'total_work_over_elapsed_giter_s':n*k*hz/sum(ds)/1e9,
            'span_s':sum(ds)/hz,'samples':len(pairs)}
out['one_stall'] = {'uninterrupted':read(normal),'stalled':read(stalled)}
assert out['one_stall']['uninterrupted']['median_giter_s']==out['one_stall']['stalled']['median_giter_s']
assert out['one_stall']['stalled']['total_work_over_elapsed_giter_s'] < 2.401

# The matched occupancy row counts sleepers on their last CPU as a busy sibling.
def sleeping_siblings(pass_no, recs, probes):
    for p in probes:
        for _,tasks in p['placement']['samples']:
            for task in tasks:
                if task[1] >= 10:
                    task[2] = 'S'
with contextlib.redirect_stdout(io.StringIO()):
    rc,j = T.judged('H2',mutate=sleeping_siblings)
out['sleeping_siblings'] = {'rc':rc,'result':j['result'],
    'slow_all_tasks':j['pass1']['table']['slow']['sibling_share'],
    'slow_running_tasks':j['pass1']['table']['slow']['sibling_share_running'],
    'fast_running_tasks':j['pass1']['table']['fast']['sibling_share_running']}
assert rc==0 and j['result'].startswith('MATCH H2')
assert out['sleeping_siblings']['slow_running_tasks']==0

# Constant input-equivalent progress can alternate at the 500ms threshold after chunking.
out['constant_sustained'] = []
for rate in [0.98,1.00,1.01,1.02,1.05,1.10]:
    rec,_ = T.sustained(lambda t:rate,chunk=4*1024*1024)
    s=tm.sustained_reading(rec,50)
    out['constant_sustained'].append({'true_constant_gbps':rate,**{k:s[k] for k in ['signature','fast_share','switches','blocks_gbps']}})

# Evidence flags are recorded by the recipe but ignored by the standalone judge.
with tempfile.TemporaryDirectory() as d:
    T.attempt(d,'H3')
    ip=Path(d)/'identity.json'; ident=json.loads(ip.read_text())
    ident.update({'inputs_match_frozen':False,'frozen':None,'preflight_exit':1,
                  'sustained_exit':124,'ab_sha256_after':'0'*64,
                  'probe_sha256':'a'*64,'probe_sha256_after':'b'*64,
                  'corpus_sha256':'c'*64,'corpus_sha256_after':'d'*64})
    ip.write_text(json.dumps(ident))
    with contextlib.redirect_stdout(io.StringIO()): rc=tm.main(['judge',d])
    j=json.loads((Path(d)/'judgement.json').read_text())
    out['contradictory_identity']={'rc':rc,'label':j['label'],'problems':j['problems']}
    assert rc==0 and not j['problems']

# Dropping a decisive reading does not just report unread: it can manufacture a unique hit.
v=tm.BASELINE.copy();v['clock']=tm.UNREAD
out['only_clock_missing']={'values':v,'outcome':tm.outcome({'fast':{'n':20},'slow':{'n':20}},v)}
assert out['only_clock_missing']['outcome'].startswith('MATCH H1')

# No treatment effect; a deterministic three-period nuisance aliases with fixed PPN.
slots=[tm.SCHEDULE[1][i%3]=='P' for i in range(60)]
fast=[i%3!=2 for i in range(60)]
out['ppn_alias_without_causal_effect']={'probed_fast':sum(f for f,p in zip(fast,slots) if p),
    'probed_n':sum(slots),'unprobed_fast':sum(f for f,p in zip(fast,slots) if not p),'unprobed_n':60-sum(slots)}
print(json.dumps(out,indent=2))
