import copy,json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent/'bench'))
import run_fed_stage as v
base=v.load(Path(__file__).parent/'bench/fixtures/run_fed_stage.json')
out=[]
for name,mut in [('ghz_negative',lambda r:r.update(ghz=-1)),('ghz_zero',lambda r:r.update(ghz=0)),('ghz_bool',lambda r:r.update(ghz=True)),('ghz_missing',lambda r:r.pop('ghz')),('output_one',lambda r:r['files'][0].update(output=1)),('accepted_rows_impossible',lambda r:r['files'][0].update(rows_runs=1,records=999999999,cells=999999999))]:
 r=copy.deepcopy(base);mut(r);row={'case':name,'problems':v.check(r,base['head'])}
 try:
  rr=v.readings(r);row.update(Q1=rr['Q1'],delta=rr['files'][0]['delta_direct'])
 except Exception as e:row['error']=repr(e)
 out.append(row)
print(json.dumps(out,indent=2))
