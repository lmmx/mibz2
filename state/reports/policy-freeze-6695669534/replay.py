"""Review-only shell boundary fixtures; simulated Cargo, no compilation or timing."""
import json,os,subprocess,tempfile,textwrap
from pathlib import Path

HERE=Path(__file__).resolve().parent
recipe=(HERE/'inputs/bench.just').read_text()
start=recipe.index('    built() {');end=recipe.index('    dir=$(python3',start)
block=textwrap.dedent(recipe[start:end])
configfn=next(s.strip() for s in recipe.splitlines() if s.strip().startswith('configs()'))
result={'build_cases':{},'config_cases':{}}
with tempfile.TemporaryDirectory() as tmp:
 p=Path(tmp);head=p/'head';head.mkdir();b=head/'target/three-team-policy';b.mkdir(parents=True);base=b/'build-BASE';base.mkdir()
 mock=p/'tools';mock.mkdir();shared=p/'shared';shared.mkdir()
 cargo=mock/'cargo';cargo.write_text('''#!/usr/bin/env python3
import os,sys,json
from pathlib import Path
if sys.argv[1:]==['-V']:
 print('cargo fixture');sys.exit(0)
label='BASE' if Path.cwd().name=='build-BASE' else 'HEAD'
with open(os.environ['FIXTURE_LOG'],'a') as f: f.write(label+'\\n')
target=Path(os.environ['FIXTURE_TARGET'])
for i,a in enumerate(sys.argv):
 if a=='--example':
  name=sys.argv[i+1];out=target/name;out.write_text(label+' '+name+'\\n')
  print(json.dumps({'reason':'compiler-artifact','target':{'name':name,'kind':['example']},'executable':str(out)}))
if os.environ.get('FIXTURE_FAIL')==label: sys.exit(7)
''');cargo.chmod(0o755)
 rustc=mock/'rustc';rustc.write_text('#!/bin/sh\necho rustc-fixture\n');rustc.chmod(0o755)
 for case in ('success','BASE','HEAD'):
  w=p/('attempt-'+case);w.mkdir();log=p/('log-'+case)
  env=dict(os.environ,PATH=str(mock)+':'+os.environ['PATH'],B=str(b),W=str(w),base='BASE',FIXTURE_TARGET=str(shared),FIXTURE_LOG=str(log),FIXTURE_FAIL=case)
  run=subprocess.run(['bash','-c','set -euo pipefail\nfail() { echo "REFUSED: $*" >&2; exit 1; }\n'+block],cwd=head,env=env,capture_output=True,text=True)
  copies={n:(w/'bin'/n).read_text().strip() for n in ('ab-base','ab-head','windows') if (w/'bin'/n).exists()}
  if case=='success':
   assert run.returncode==0,(run.stdout,run.stderr)
   assert copies=={'ab-base':'BASE ab','ab-head':'HEAD ab','windows':'HEAD windows'}
  else:
   assert run.returncode!=0 and ('the '+case.lower()+' build failed') in run.stderr
   assert 'ab-head' not in copies
   if case=='BASE': assert not copies and log.read_text()=='BASE\n'
  result['build_cases'][case]={'exit':run.returncode,'copies':copies,'builds':log.read_text().splitlines(),'stderr':run.stderr.strip()}
 # Exact guard from the recipe, before any build.
 a=recipe.index('    only_base=$(configs');z=recipe.index('    # the toolchain',a)
 guard=textwrap.dedent(recipe[a:z])
 env=dict(os.environ,B=str(b),base='BASE')
 def check():
  return subprocess.run(['bash','-c','set -euo pipefail\nfail() { echo "$*" >&2; exit 1; }\n'+configfn+'\n'+guard],cwd=head,env=env,capture_output=True,text=True)
 assert check().returncode==0
 result['config_cases']['clean']=True
 for folder in (b,head/'target',head):
  for name in ('config','config.toml'):
   (folder/'.cargo').mkdir(exist_ok=True)
   f=folder/'.cargo'/name;f.write_text('[build]\nrustflags=["-C", "opt-level=0"]\n')
   run=check();assert run.returncode!=0 and str(f) in run.stderr
   result['config_cases'][str(f.relative_to(head))]='refused'
   f.unlink()
 # The common ancestor and Cargo-home files must be discovered by the actual hash loop.
 cargo_home=p/'cargo-home';cargo_home.mkdir();homeconfig=cargo_home/'config.toml';homeconfig.write_text('[net]\noffline=true\n')
 (p/'.cargo').mkdir();common=p/'.cargo/config.toml';common.write_text('[build]\njobs=2\n')
 a=recipe.index('      for f in "${CARGO_HOME:');z=recipe.index('      echo "census',a)
 loop=textwrap.dedent(recipe[a:z])
 run=subprocess.run(['bash','-c','set -euo pipefail\n'+configfn+'\n'+loop],cwd=head,env=dict(os.environ,CARGO_HOME=str(cargo_home)),capture_output=True,text=True)
 assert run.returncode==0,(run.stdout,run.stderr)
 assert str(common) in run.stdout and str(homeconfig) in run.stdout
 result['common_configuration_record']={'ancestor_hashed':True,'cargo_home_hashed':True,'exit':run.returncode}
print(json.dumps(result,indent=2))
