"""Untimed report validation fixtures. No decoder/build/workstation invocation."""
import contextlib, io, json, math, shutil, tempfile, types
from pathlib import Path
from unittest.mock import patch
import service_2x3 as s
HEAD='3e4477d035e96310eb4ac3d4014e01e13ffaa33c'
def fixture():
    w=Path(tempfile.mkdtemp(prefix='service2x3-review-'))
    (w/'commit.txt').write_text(HEAD+'\n')
    (w/'git-status.txt').write_text('')
    (w/'lscpu.txt').write_text('Socket(s): 1\nCore(s) per socket: 10\nL3 cache: 20 MiB (1 instance)\n')
    (w/'corpus.sha256').write_text('a'*64+'  wikidata.bz2\n')
    rows=[]
    for r in range(20):
        for cell,(leaders,helpers,svc) in s.CELLS.items():
            arm,budget=cell.split('@')
            names=[f'mibz2-{i}=.1' for i in range(leaders)]
            names += [f'mibz2-coop-{i}=.1' for i in range(helpers)]
            names += [f'mibz2-svc-{i}=.1' for i in range(svc)]
            rows.append(dict(arm=arm,threads=int(budget),round=r,wall=1.,cpu=6.,rss_kb=64000,hash='a'*64,out=209700196,err='',corpus='wikidata',thread_cpu=' '.join(names)))
    tl={'complete_rounds':6,'blocks':431,'svc.util.max':.7,'svc.team.idle_prep_share':.15,'svc.queue.finish.median_ms':.1,'svc.team.idle_back_share':.01}
    trace={'results':[{'timeline':tl}]}
    return w,rows,trace

def run(mutate):
    w,rows,trace=fixture()
    try:
        mutate(w,rows,trace)
        (w/'runs.jsonl').write_text('\n'.join(map(json.dumps,rows))+'\n')
        (w/(s.TRACE+'.json')).write_text(json.dumps(trace))
        a=types.SimpleNamespace(runs=str(w/'runs.jsonl'),occupancy=[str(w/(s.TRACE+'.json'))],out=str(w/'result'))
        try:
            with patch.object(s.subprocess,'run',return_value=types.SimpleNamespace(stdout=HEAD+'\n')), contextlib.redirect_stdout(io.StringIO()):
                status=s.report(a)
            d=json.loads((w/'result.json').read_text())
            return {'exit':status,'verdict':d['verdict'],'overall':d['readings']['overall'],'saturation':d['saturation']}
        except Exception as e:
            return {'error':type(e).__name__,'detail':str(e)}
    finally: shutil.rmtree(w)

def drop(key):
    return lambda w,rs,t:[r.pop(key) for r in rs]
def setall(**kw):
    return lambda w,rs,t:[r.update(kw) for r in rs]
cases={
 'control':lambda w,r,t:None,
 'deleted_hash':drop('hash'), 'deleted_size':drop('out'), 'deleted_wall':drop('wall'),
 'null_output':setall(hash=None,out=None), 'empty_output':setall(hash='',out=0),
 'negative_wall_and_cpu':setall(wall=-1.,cpu=-6.),
 'wrong_corpus':setall(corpus='different-workload'),
 'absent_corpus_digest':lambda w,r,t:(w/'corpus.sha256').unlink(),
 'absent_trace_metrics':lambda w,r,t:t.update(results=[{'timeline':{'complete_rounds':1}}]),
 'wrong_trace_plan':lambda w,r,t:t.update(results=[{'threads':1,'svc':1,'timeline':t['results'][0]['timeline']}]),
 'different_base_arm':lambda w,r,t:r[0].update(arm='mibz2@/other/base'),
 'one_round_only':lambda w,r,t:r.__setitem__(slice(None),[x for x in r if x['round']==0]),
 'dry_marker':lambda w,r,t:(w/'DRY-RUN.txt').write_text('dry'),
 'changed_freeze_marker':lambda w,r,t:(w/'NOT-FROZEN.txt').write_text('changed'),
 'duplicate_round':lambda w,r,t:r.append(dict(r[0])),
 'wrong_plan':lambda w,r,t:r[0].update(thread_cpu='mibz2-0=.1'),
}
result={name:run(fn) for name,fn in cases.items()}
assert result['control']['verdict']=='VALID'
for name in ['null_output','empty_output','negative_wall_and_cpu','wrong_corpus','absent_corpus_digest','absent_trace_metrics','wrong_trace_plan']:
    assert result[name]['verdict']=='VALID', (name,result[name])
for name in ['one_round_only','dry_marker','changed_freeze_marker','duplicate_round','wrong_plan']:
    assert result[name]['verdict']=='INVALID', (name,result[name])
def outcomes(speed='GAIN',cpu='NULL'):return {'speed_outcome':speed,'cpu_outcome':cpu}
# Both measured comparisons can improve, while the simpler pipelined control is faster.
result['both_mechanisms_gain']=s.readings({'P1':outcomes(),'C1':outcomes(),'P2':outcomes()})
result['cpu_inconclusive']=s.readings({'P1':outcomes(cpu='INCONCLUSIVE (the interval spans gain and harm)'),'C1':outcomes(),'P2':outcomes('REGRESSION')})
result['synthetic_mechanisms']={'wall':{'default':1.,'plain2':1.10,'pipe2':.90,'svc2':.95},'ratios':{'P1':1/.95,'C1':1.10/.95,'P2':1/.90,'S1':.90/.95},'meaning':'P1, C1, P2 gains coexist with svc2 slower than pipe2; illustrative values, not timing evidence.'}
print(json.dumps(result,indent=2))
