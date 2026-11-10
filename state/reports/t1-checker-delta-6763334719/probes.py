"""Research-only replay. Run from this report directory after fetching pinned sources.

bench/team_screen.py and test_team_screen.py plus manifest/team_gap_count from
9a37e9924e0b3fa0004344dcac45808aa1f9f733; bench/old_team_screen.py is
bench/team_screen.py from c077cbdc44a77a5e2f8ee76f7785c2f4240e34f6.
No decoder processes or timing are used.
"""
import copy, hashlib, json, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE/'bench'))
import test_team_screen as t
import team_screen as new
import old_team_screen as old

results=[]
def run(name,mutate,old_admits,new_admits):
    m,d,a=t.manifest(),t.delta(),t.artifacts()
    r=t.good_records(m,d,a)
    mutate(m,d,a,r)
    before=old.outputs_check(m,d,a,r)
    after=new.outputs_check(m,d,a,r)
    assert (not before)==old_admits,(name,before)
    assert (not after)==new_admits,(name,after)
    results.append(dict(case=name,old_admits=not before,new_admits=not after,new_failures=after))

run('complete_96_including_expected_decoder_errors',lambda *args:None,True,True)
for build in ('head','base'):
    def mutate(m,d,a,r,build=build):a[build]['sha']='9'*40
    run('wrong_artifact_'+build,mutate,True,False)
    def mutate(m,d,a,r,build=build):a[build].pop('sha')
    run('missing_artifact_sha_'+build,mutate,True,False)

for kind,statuses in [('reject',[None,-9,124,2,True,'1',1.0,0]),('arm',[False,0.0,None,-9,124,1,'0']),('oracle',[False,0.0,None,-9,124,1,'0'])]:
    for status in statuses:
        def mutate(m,d,a,r,kind=kind,status=status):
            w=next(w for w in r if w['kind']==kind and (kind!='arm' or w['fixture']=='E-crc-team'))
            w.update(exit_status=status,signal=None,timed_out=False)
        old_admits=(status!=0) if kind=='reject' else status==0
        run(f'{kind}_exit_{type(status).__name__}_{status}',mutate,old_admits,False)

for kind in ('reject','arm','oracle'):
    for field,value in [('signal',9),('timed_out',True)]:
        def mutate(m,d,a,r,kind=kind,field=field,value=value):
            next(w for w in r if w['kind']==kind)[field]=value
        run(f'{kind}_{field}',mutate,False,False)

files=['bench/team_screen.py','bench/old_team_screen.py','bench/test_team_screen.py','bench/team_gap_count.py','bench/team-screen/manifest.json']
print(json.dumps({'cases':results,'total':len(results),'source_sha256':{p:hashlib.sha256((HERE/p).read_bytes()).hexdigest() for p in files},'native_decoder_executed':False},indent=2))
