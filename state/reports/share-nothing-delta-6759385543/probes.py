import ast,contextlib,io,json,math,os,shutil,sys,tempfile
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
import share_nothing as sn

# Use the pinned suite's fixture builders, not a substitute judge; no pytest dependency.
source=Path(__file__).with_name('test_share_nothing.py').read_text()
tree=ast.parse(source)
names={'i9_static','samples','cgroup_root','cpu_model','state','write_preflight','make_attempt'}
nodes=[]
for n in tree.body:
    if isinstance(n,ast.Assign) and all(isinstance(t,(ast.Name,ast.Tuple)) for t in n.targets):
        if any(isinstance(x,ast.Name) and x.id in {'LSCPU','HEAD','FROZEN','AB','L2','L9'} for t in n.targets for x in ast.walk(t)):
            nodes.append(n)
    if isinstance(n,ast.FunctionDef) and (n.name in names or n.name.startswith('_') or n.name.startswith('test_cgroup_')):
        n.decorator_list=[];nodes.append(n)
ns=dict(globals());exec(compile(ast.Module(body=nodes,type_ignores=[]),'pinned-fixtures','exec'),ns)
out={}
def admission(name,mutation=None):
    with tempfile.TemporaryDirectory() as p:
        d=ns['make_attempt'](Path(p),mutate=mutation)
        bad,rows,pf=sn.admission_problems(str(d),False)
        out[name]={'valid':not bad,'problems':bad[:4]}
        return rows,bad
rows,bad=admission('clean');assert not bad
for name in sorted(ns):
    if name.startswith('_') and callable(ns[name]) and name in {n.name for n in nodes if isinstance(n,ast.FunctionDef)}:
        _,bad=admission(name,ns[name]);assert bad,name
for name in sorted(ns):
    if name.startswith('test_cgroup_'):
        with tempfile.TemporaryDirectory() as p:ns[name](Path(p))
        out[name]='PASS'
with tempfile.TemporaryDirectory() as p:
    d=ns['make_attempt'](Path(p));shutil.rmtree(d/'preflight')
    bad,_,_=sn.admission_problems(str(d),False)
    assert bad;out['missing_preflight']=bad
with tempfile.TemporaryDirectory() as p:
    d=ns['make_attempt'](Path(p),preflight=False,mutate=lambda v:v['identity'].update(preflight_exit=0))
    bad,_,_=sn.admission_problems(str(d),False)
    assert bad;out['preflight_disagreement']=bad[:2]
# Independent full-round reordering must fail before statistics.
def reorder(v):
    v['slots']=v['slots'][12:24]+v['slots'][:12]+v['slots'][24:]
    v['runs']=v['runs'][12:24]+v['runs'][:12]+v['runs'][24:]
_,bad=admission('whole_round_swap',reorder);assert bad
# Regenerate original interpretation counterexample against the revised analysis.
for r in rows:
    if r['round']>=1 and r['corpus']==sn.PRIMARY and r['threads']==10:
        f=.8 if r['round']<=30 else 1.2
        r['cpu']=ns['cpu_model'](sn.PRIMARY,4,False)*3.5*f
result=sn.analyse(rows,False)
q=result['primary']['cpu']['ratio_of_ratios']
assert q['lo']<1<q['hi']
assert result['model_check']['ratio_of_ratios'].startswith('includes 1: INCONCLUSIVE')
assert len(result['chronological'])==60
for c in result['modes']['classes'].values():
    assert 'lo' not in c['cpu']['ratio_of_ratios']
out['interpretation']={'ratio':q['ratio'],'lo':q['lo'],'hi':q['hi'],'class':result['model_check']['ratio_of_ratios'],'chronological_rounds':len(result['chronological']),'mode_intervals':False}
out['scope']='Synthetic fixtures and pinned functions; no decoder execution or timing; not the 91-test pytest suite.'
Path(__file__).with_name('probes.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
