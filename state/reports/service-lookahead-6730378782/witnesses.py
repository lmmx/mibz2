"""Untimed counterexamples against the pinned report; no decoder execution."""
import json,pathlib,sys
ROOT=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'bench'))
import test_service_lookahead as t
def trace_edit(**kw):
    def mutate(rows,traces,files):
        for tr in traces.values(): tr['results'][0]['timeline'].update(kw)
    return mutate
def setting_one(rows,traces,files):
    for tr in traces.values(): tr['settings']['rounds']=1
def no_raw(rows,traces,files):
    for tr in traces.values(): tr['runs']=[]
def wrong_arm(rows,traces,files):
    traces[t.A4W8]['results'][0]['timeline']['svc.team.idle_prep_share']=.04
cases={
 'control':None,
 'one_complete_trace_round':trace_edit(complete_rounds=1),
 'trace_requests_one_round':setting_one,
 'trace_raw_runs_empty':no_raw,
 'prepared_exceeds_ceiling':trace_edit(**{'svc.backlog.prep_max':99}),
 'prepared_median_above_max':trace_edit(**{'svc.backlog.prep_max':2,'svc.backlog.prep_median':3}),
 'no_blocks_in_trace':trace_edit(blocks=0),
}
out={}
for name,mut in cases.items():
    d=t.run(mut)
    out[name]={k:d[k] for k in ['verdict','exit','readings','mechanism']}
    out[name]['failed_checks']=t.failed(d)
d=t.run(wrong_arm,walls={'mibz2-svc2-a3':.95})
out['gain_and_relief_on_different_arms']={k:d[k] for k in ['verdict','judged','mechanism']}
(ROOT/'witnesses.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({k:{'verdict':v['verdict'],'mechanism':v.get('mechanism')} for k,v in out.items()},indent=2))
