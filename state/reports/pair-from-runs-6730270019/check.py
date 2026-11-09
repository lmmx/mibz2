"""Independent untimed mathematical and evidence-envelope witnesses; no decoder code."""
import itertools, json, math, pathlib, subprocess, tempfile
ROOT = pathlib.Path(__file__).resolve().parent
out = {}
cases = 0
for alphabet, lengths in [(range(3), range(1,9)), (range(2),range(9,13))]:
    for n in lengths:
        for L in itertools.product(alphabet, repeat=n):
            order = sorted(range(n), key=lambda p:L[p])
            psi = order
            lf = [0]*n
            for q,p in enumerate(order): lf[p]=q
            runs=[]
            for p in range(n):
                if p==0 or L[p]!=L[p-1]: runs.append([p,p+1,lf[p]])
                else: runs[-1][1]=p+1
            D=[None]*n; cells=0
            for p,e,a in runs:
                for pj,ej,aj in runs:
                    x,y=max(p,aj),min(e,aj+ej-pj)
                    if x>=y: continue
                    cells+=1
                    for t in range(y-x):
                        q=a+x-p+t
                        assert D[q] is None
                        D[q]=L[q] | ((pj+x-aj+t)<<8)
            assert D==[L[q] | (psi[psi[q]]<<8) for q in range(n)]
            assert cells<=2*len(runs)-1
            cases+=1
out['formula_columns']=cases
# Deterministic sampling misses high-density positions: no timing or Rust execution.
L=[i%2 for i in range(128*128)]
for w in range(1,len(L)-7,128): L[w-1:w+8]=[0]*9
sample=[L[w+k]!=L[w+k-1] for w in range(1,len(L)-7,128) for k in range(8)]
out['admission_alias']={'length':len(L),'sample_density':sum(sample)/len(sample),
                      'actual_density':(1+sum(a!=b for a,b in zip(L,L[1:])))/len(L)}
# Execute the exact workflow's artifact/status predicate, in isolated fixture directories.
workflow=(ROOT/'.github/workflows/sweep.yml').read_text()
predicate=workflow[workflow.index('          for f in sweep-*.txt; do'):workflow.index('          { echo "head $(git rev-parse HEAD); corpus SHA-256 $got"')]
results={}
for name,children,raw in [('complete', ['1','4-cpu0','4-cpu1','4-cpu2','4-cpu3'],'file,round,arm,seconds\nx,0,P,1\n'),
                          ('missing_all_multicopy', ['1'],'file,round,arm,seconds\nx,0,P,1\n'),
                          ('header_only_raw', ['1','4-cpu0','4-cpu1','4-cpu2','4-cpu3'],'file,round,arm,seconds\n')]:
    with tempfile.TemporaryDirectory(dir=ROOT) as d:
        d=pathlib.Path(d)
        for c in children:
            (d/f'sweep-{c}.txt').write_text('VALID (this process\n')
            (d/f'status-{c}.txt').write_text('0\n')
            (d/f'raw-{c}.csv').write_text(raw)
        r=subprocess.run(['bash','-c','problems=""\n'+predicate+'\nprintf "%s" "$problems"'],cwd=d,text=True,capture_output=True,check=True)
        results[name]={'problems':r.stdout,'artifact_predicate_accepts':not r.stdout}
out['workflow_predicate']=results
# Literal diff()/required-interval logic, with sound positive raw ratios supplied.
def diff(d):
    if len(d)<2 or any(not math.isfinite(x) for x in d): return None
    m=sum(d)/len(d); var=sum((x-m)**2 for x in d)/(len(d)-1)
    h=12.706*math.sqrt(var/len(d))
    return (m,m-h,m+h)
out['m1_missing']={'ghz':'NaN','m1d':diff([math.nan,math.nan]),
                   'required_interval_labels':['Q1','M1 ratio','Q2','O/P','R/O'],
                   'difference_checked_by_validity':False}
out['zero_ghz_m1']=diff([0.,0.])
out['ipc_counterexample']={'one_I':6.2,'one_IPC':2,'run_I':12.4,'run_IPC':4,
                           'cycle_difference':12.4/4-6.2/2}
(ROOT/'checks.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
