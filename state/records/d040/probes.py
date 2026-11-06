import json, os, pathlib, subprocess, tempfile
ROOT=pathlib.Path(__file__).resolve().parent
recipe=(ROOT/'recipe.sh').read_text()
block=recipe[recipe.index('built() {'):recipe.index('dir=$(python3')]
fail=recipe[recipe.index('fail() {'):recipe.index('# the commits')]
guard=recipe[recipe.index('overrides=$(env'):recipe.index('[ -z "$(git status')]
fake='''#!/usr/bin/env python3
import json,os,pathlib,sys
cwd=pathlib.Path.cwd(); base=cwd.name.startswith('build-'); case=os.environ['CASE']
if sys.argv[1:]==['-V']:
 print('cargo fixture 1');sys.exit(0)
with open(os.environ['EVENTS'],'a') as f:f.write(('base' if base else 'head')+'\\n')
target=pathlib.Path(os.environ['SHARED']) if case=='shared' else cwd/'target'/'configured-triple'/'release'/'examples'
target.mkdir(parents=True,exist_ok=True)
names=['ab'] if base else ['ab','windows']
if case=='missing' and base:names=[]
for name in names:
 p=target/name;p.write_text(('BASE' if base else 'HEAD')+':'+name)
 print(json.dumps({'reason':'compiler-artifact','executable':str(p),'target':{'name':name,'kind':['example']}}),flush=True)
if case=='malformed' and base:print('{',flush=True)
if case=='tamper' and not base:(pathlib.Path(os.environ['W'])/'bin/ab-base').write_text('TAMPER')
sys.exit(7 if (case=='base-fails' and base) or (case=='head-fails' and not base) else 0)
'''
rust='''#!/usr/bin/env python3
import os,pathlib
base=pathlib.Path.cwd().name.startswith('build-')
print('rustc fixture '+('2' if os.environ['CASE']=='toolchain-mismatch' and base else '1'))
'''
results={'method':'Exact extracted built/tool/build-copy block, no substitution edits. Synthetic cargo and rustc; no real Rust builds or decoder execution. No timing.'}
results['builds']=[]
for case in ['distinct','shared','base-fails','head-fails','missing','malformed','tamper','toolchain-mismatch']:
 with tempfile.TemporaryDirectory(prefix='guardian d040 ') as tmp:
  root=pathlib.Path(tmp);head=root/'head';head.mkdir();b=root/'builds';(b/'build-base').mkdir(parents=True);w=root/'attempt';w.mkdir();cmd=root/'commands';cmd.mkdir()
  for name,body in [('cargo',fake),('rustc',rust)]:
   (cmd/name).write_text(body);(cmd/name).chmod(0o755)
  events=root/'events'
  env=dict(os.environ,PATH=str(cmd)+':'+os.environ['PATH'],B=str(b),W=str(w),base='base',SHARED=str(root/'shared output'/'triple'/'release'/'examples'),CASE=case,EVENTS=str(events))
  p=subprocess.run(['bash','-c','set -euo pipefail\n'+fail+block],cwd=head,env=env,capture_output=True,text=True,timeout=15)
  copies={x.name:x.read_text() for x in (w/'bin').glob('*') if x.name!='ab-base.sha256'}
  r={'case':case,'exit':p.returncode,'events':events.read_text().splitlines() if events.exists() else [],'copies':copies,'stdout':p.stdout.strip(),'stderr':p.stderr.strip()}
  results['builds'].append(r)
  assert p.returncode==(0 if case in ['distinct','shared'] else 1),r
  if case in ['distinct','shared']:assert copies=={'ab-base':'BASE:ab','ab-head':'HEAD:ab','windows':'HEAD:windows'},r
  if case in ['base-fails','missing','malformed']:assert r['events']==['base'] and not copies,r
  if case=='head-fails':assert copies=={'ab-base':'BASE:ab'},r
  if case=='toolchain-mismatch':assert not r['events'] and not copies,r

results['environment']=[]
denied=['CARGO_BUILD_RUSTFLAGS','CARGO_TARGET_DIR','CARGO_BUILD_TARGET','CARGO_PROFILE_RELEASE_OPT_LEVEL','RUSTFLAGS','RUSTC_WRAPPER','RUSTUP_TOOLCHAIN','AB_CENSUS_MS','AB_SAMPLE_MS']
allowed=['CARGO_HOME','RUSTUP_HOME','RUST_BACKTRACE','RUST_LOG','CARGO_HTTP_TIMEOUT','CARGO_NET_OFFLINE','CARGO_TERM_COLOR']
clean={k:v for k,v in os.environ.items() if not (k.startswith('CARGO_') or k.startswith('RUST') or k in ['AB_CENSUS_MS','AB_SAMPLE_MS'])}
for key in denied+allowed:
 env=dict(clean);env[key]='fixture'
 p=subprocess.run(['bash','-c','set -euo pipefail\n'+fail+guard],env=env,capture_output=True,text=True,timeout=10)
 results['environment'].append({'variable':key,'exit':p.returncode})
 assert p.returncode==(1 if key in denied else 0),(key,p.returncode,p.stderr)

measured=next(x for x in recipe.splitlines() if x.startswith('measured=('))
results['frozen_paths']=[]
for name in ['build.rs','rust-toolchain','rust-toolchain.toml']:
 with tempfile.TemporaryDirectory() as tmp:
  root=pathlib.Path(tmp)
  def git(*args):return subprocess.run(['git',*args],cwd=root,capture_output=True,text=True,check=True)
  git('init','-q');git('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','-q','--allow-empty','-m','fixture baseline')
  baseline=git('rev-parse','HEAD').stdout.strip();(root/name).write_text('fixture\n')
  status=subprocess.run(['bash','-c',measured+'\ngit status --porcelain --untracked-files=all -- "${measured[@]}"'],cwd=root,capture_output=True,text=True,check=True).stdout.strip()
  git('add',name);git('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','-q','-m','fixture new input')
  p=subprocess.run(['bash','-c',measured+'\ngit diff --quiet "$cand" HEAD -- "${measured[@]}"'],cwd=root,env=dict(os.environ,cand=baseline),capture_output=True,text=True)
  assert status=='?? '+name and p.returncode==1,(name,status,p.returncode)
  results['frozen_paths'].append({'path':name,'untracked_detected':True,'committed_addition_detected':True})

(ROOT/'results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps(results,indent=2))
