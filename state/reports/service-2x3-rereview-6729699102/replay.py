"""Untimed independent replay: original mutations, repaired valid control, and boundary checks."""
import contextlib,io,json,pathlib,sys,copy
P=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(P))
import service_2x3 as s
import test_service_2x3 as t
src=(P/'original_probe.py').read_text()
ns={}
# Retain the old probe's actual cases; omit assertions that deliberately asserted the old bugs.
exec(src.split("assert result['control']")[0],ns)
old=copy.deepcopy(ns['result'])
original_fixture=ns['fixture']
def repaired_fixture():
    w,rows,trace=original_fixture()
    _,newtrace,files=t.fixture()
    trace.clear();trace.update(newtrace)
    (w/'corpus.sha256').write_text(s.REGISTERED_CORPUS_SHA256+'  wikidata.bz2\n')
    return w,rows,trace
ns['fixture']=repaired_fixture
fixed={name:ns['run'](fn) for name,fn in ns['cases'].items()}
assert fixed['control']['verdict']=='VALID'
assert all(v.get('verdict')=='INVALID' for k,v in fixed.items() if k!='control')
extras={}
for k,mut in {
 'boolean_wall':t.setall(wall=True),
 'nan_cpu':t.setall(cpu=float('nan')),
 'zero_rss':t.setall(rss_kb=0),
 'infinite_rss':t.setall(rss_kb=float('inf')),
 'nonhex_digest':t.setall(hash='g'*16),
 'trace_nan':t.set_trace(results=[{'threads':10,'timeline':t.timeline(**{'svc.util.0':float('nan')})}]),
 'round_list':t.setall(round=[]),
}.items():
    try:
        d=t.run(mut);extras[k]={'verdict':d['verdict'],'exit':d['exit']}
    except Exception as e: extras[k]={'exception':type(e).__name__,'detail':str(e)}
assert all(v.get('verdict')=='INVALID' for k,v in extras.items() if k!='round_list')
d=t.run(walls=t.ILLUSTRATIVE)
out={'old_cases_on_original_control':old,'old_cases_on_repaired_control':fixed,'additional_cases':extras,
 'illustrative_readings':d['readings'],'decomposition':d['decomposition'],
 'scope':'Synthetic report records only. No decoder or timing. git HEAD mocked to the fixture SHA; corrected control changes only registered corpus digest and full trace.'}
(P/'replay.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'repaired_control':fixed['control']['verdict'],'mutations_rejected':len(fixed)-1,'extra_cases':extras,'illustrative_overall':d['readings']['overall'],'decomposition':d['decomposition']},indent=2))

