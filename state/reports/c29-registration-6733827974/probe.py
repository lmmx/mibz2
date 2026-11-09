import sys, pathlib, json, math
sys.path.insert(0,str(pathlib.Path(__file__).parent/'bench'))
import test_c29_checkpoints as t
cases={}
for label,kw in [('registered_fixture',{}),('explicit_non_AVX2_two_cpu',{'avx2':False,'cpu_count':2})]:
    code,out=t.report(t.runs(ratio=1.0),t.identity(**kw));cases[label]={'exit':code,'stdout':out}
ratios=[r['wall']/2 for r in t.runs() if r['threads']==9 and r['arm']==t.c.C29]
cases['interval']=dict(zip(['mean','lo','hi'],t.c.geo_ci(ratios)))
cases['threshold_boundary']=t.c.reading_of((.98,.975,.985),(1,.995,1.005))
print(json.dumps(cases,indent=2))
