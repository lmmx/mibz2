import sys,json,copy,contextlib,io,tempfile,pathlib,argparse
sys.path.insert(0,str(pathlib.Path(__file__).parent/'bench'))
import test_run_fed_leader as t
import run_fed_leader as v
t.ROUNDS=15
def execute(label,change):
 r=t.clean()
 change(r)
 with tempfile.TemporaryDirectory() as d:
  p=pathlib.Path(d)/'r.jsonl';p.write_text(''.join(json.dumps(x)+'\n' for x in r))
  s=io.StringIO()
  try:
   with contextlib.redirect_stdout(s): code=v.report(argparse.Namespace(runs=str(p),threads=9,rounds=15,out=None))
   err=None
  except Exception as e:code=None;err=repr(e)
 return dict(case=label,exit=code,error=err,valid=s.getvalue().startswith('VALID:'),reading=next((l for l in s.getvalue().splitlines() if l.startswith('Primary')),None))
def each(r,fn):
 for x in r:fn(x)
def absent(r):each(r,lambda x:(x.pop('hash'),x.pop('out')))
def cpu(r):each(r,lambda x:x.update(cpu=-x['cpu']))
def phases(r):
 for x in r:
  if 'runfed' in x['arm']:x['phases']='runs=-1 team_records='+('-1' if x['arm'] in v.RECORD_ARMS else '0')+' fallbacks=0'
def ctrl(r):
 for x in r:
  if x['arm'].endswith('-ctl'):x['wall']*=.8
  if x['arm']==v.PRIMARY:x['wall']*=.90
def corpus(r):each(r,lambda x:x.update(corpus='different-'+str(x['round']),hash='same',out=100))
a=[execute('control',lambda r:None),execute('all output evidence missing',absent),execute('all CPU negative',cpu),execute('negative counts and absent timing phases',phases),execute('corpus varies by round',corpus),execute('control -20%, primary apparent gain against REF but loss against ctl',ctrl)]
print(json.dumps(a,indent=2))
