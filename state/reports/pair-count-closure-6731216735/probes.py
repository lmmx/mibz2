"""Independent untimed replay for ae62ea75a0e9e75a6d300264daffb99c6763e9f9."""
import sys,tempfile,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent/'bench'))
import test_pair_runs_validity as t
import pair_runs_validity as v

def rows(d,c,field,value):
    t.set_counts(d,c,t.CORPUS,field,value)
def allrows(d,field,value):
    for c in v.copies_for(2): rows(d,c,field,value)
def both_census(d):
    allrows(d,5,199000000)
    for c in v.copies_for(2):
        t.edit_log(d,c,lambda ls:[l.replace('(200000000 rows)','(199000000 rows)') if l.startswith(t.CORPUS+':') else l for l in ls])
def duplicate(d):
    t.edit_log(d,'1',lambda ls:ls+[next(x for x in ls if x.startswith(t.CORPUS+':'))])
cases=[
 ('control',lambda d:None,True),
 ('old_child_400M',lambda d:rows(d,'2-cpu1',5,400000000),False),
 ('old_M1_20B',lambda d:rows(d,'1',5,20000000000),False),
 ('old_negative_blocks',lambda d:allrows(d,4,-233),False),
 ('old_child_output_1',lambda d:rows(d,'2-cpu1',6,1),False),
 ('child_rows_within_domain',lambda d:rows(d,'2-cpu1',5,199000000),False),
 ('common_mode_rows_within_domain',lambda d:allrows(d,5,199000000),False),
 ('common_mode_zero_rows',lambda d:allrows(d,5,0),False),
 ('duplicate_census',duplicate,False),
 ('consistent_common_mode_rows_not_independently_pinned',both_census,True),
]
out=[]
for name,mutate,want in cases:
 with tempfile.TemporaryDirectory() as tmp:
  d=t.make_run(tmp);mutate(d);ok,lines=v.validate(d)
  assert ok==want,(name,lines)
  if not ok:
   assert not any('readings (UNJUDGED' in l for l in lines),lines
   assert any('NOT VALID (no readings)' in l for l in lines),lines
  out.append({'case':name,'valid':ok,'lines':lines})
print(json.dumps(out,indent=2))
