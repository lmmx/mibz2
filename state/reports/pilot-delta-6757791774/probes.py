"""Independent replay of the prior pilot counterexamples; synthetic records only."""
import copy,json,tempfile,statistics,itertools
from pathlib import Path
import service_tg as tg
import test_service_tg as T
import test_tg_ni_pilot as P
import tg_ni_pilot as pilot
import tg_ni_gate as gate
T.R=600
out={}
with tempfile.TemporaryDirectory() as root:
    man=T.build(root,label='pilot')
    for a in man['artifacts']:
        rows=tg.read_jsonl(str(Path(root)/a['name']/a['shard']/'runs.jsonl'))[0]
        P.add_times(root,a['shard'],rows,step=1.5)
    man=T.remanifest(root,label='pilot')
    for j in man['jobs']:j['started_at']='2026-11-09T23:50:00Z'
    def analyse(m=man):
        return pilot.analyse(root,m,T.HEAD,T.BASESHA,T.REG['frozen']['ni_rustc'],T.REG['frozen']['corpus_text'],rounds=600)
    v=analyse();assert v['valid'],v['problems']
    out['clean']={'valid':v['valid'],'cells':len(v['cells']),'menu':v['cells']['runs/1/wall']['batches']}
    for b in out['clean']['menu']:
        assert abs(b['lrv']-b['rounds']*b['se']**2)<1e-12
    a=next(a for a in man['artifacts'] if a['shard']=='runs')
    d=Path(root)/a['name']/'runs'
    def mutate(name,file,value,rehash=False):
        p=d/file;original=p.read_text();p.write_text(value)
        m=copy.deepcopy(man)
        if rehash:
            next(x for x in m['artifacts'] if x['shard']=='runs')['files'][file]=tg.sha256_file(p)
        v=analyse(m);out[name]={'valid':v['valid'],'problems':v['problems']};assert not v['valid'];p.write_text(original)
    mutate('missing_times','row-times.txt','',True)
    rows=[json.loads(l) for l in (d/'runs.jsonl').read_text().splitlines()];rows[0]['wall']*=1.01
    mutate('changed_rows','runs.jsonl',''.join(json.dumps(x)+'\n' for x in rows))
    times=(d/'row-times.txt').read_text().splitlines();parts=times[2].split();parts[1]=str(T.T0-10);times[2]=' '.join(parts)
    mutate('backwards_times','row-times.txt','\n'.join(times)+'\n',True)
    mutate('stamper_failed','stamp.exit','1\n',True)
    m=copy.deepcopy(man);m['unlisted']=['unexpected'];v=analyse(m);assert not v['valid'];out['unlisted']=v['problems']
    m=copy.deepcopy(man);m['jobs'][0].pop('started_at');v=analyse(m);assert not v['valid'];out['missing_job_start']=v['problems']
    for label in ['smoke','pilot']:
        refused=tg.ni_judge(root,{'label':label,'artifacts':[]},T.HEAD,reg={'frozen':{},'bench':None},check_git=False)
        out['empty_'+label]=gate.gate(label,refused,2);assert out['empty_'+label][0]==1
ends=list(itertools.accumulate(([1]*29+[100])*20))
out['batch_span']=pilot.tail(pilot.spans(0,ends,30));assert out['batch_span']['median']==129
try:pilot.batches(list(range(600)),48)
except ValueError:out['unregistered_tail']='refused'
else:raise AssertionError('silently dropped tail')
print(json.dumps(out,indent=2))
