"""Review fixtures only. Executes extracted shell logic with simulated Cargo, not a build."""
import json,os,subprocess,tempfile,textwrap
from pathlib import Path

HERE=Path(__file__).resolve().parent
recipe=(HERE/'inputs/bench.just').read_text()
measured=next(s.strip() for s in recipe.splitlines() if s.strip().startswith('measured='))
guard=next(s.strip() for s in recipe.splitlines() if s.strip().startswith('overrides=$('))
result={}
with tempfile.TemporaryDirectory() as tmp:
 p=Path(tmp)
 def git(*args): return subprocess.check_output(['git','-C',str(p),*args],text=True).strip()
 git('init','-q');(p/'Cargo.toml').write_text('[package]\nname="fixture"\nversion="0.1.0"\n')
 git('add','.');git('-c','user.name=Louis Maddox','-c','user.email=lmmx@users.noreply.github.com','commit','-qm','fixture')
 frozen=git('rev-parse','HEAD'); result['root_inputs']={}
 for name in ('build.rs','rust-toolchain','rust-toolchain.toml'):
  (p/name).write_text('review fixture\n')
  status=subprocess.check_output(['bash','-c',measured+'\ngit status --porcelain --untracked-files=all -- "${measured[@]}"'],cwd=p,text=True).strip()
  assert name in status
  git('add',name);git('-c','user.name=Louis Maddox','-c','user.email=lmmx@users.noreply.github.com','commit','-qm',name)
  rc=subprocess.run(['bash','-c',measured+'\ngit diff --quiet "$1" HEAD -- "${measured[@]}"','fixture',frozen],cwd=p,capture_output=True).returncode
  assert rc==1
  result['root_inputs'][name]={'untracked_refused':True,'committed_change_refused':True}
  git('reset','--hard',frozen)
 result['environment']={}
 for name in ('RUSTC_WORKSPACE_WRAPPER','CARGO_PROFILE_RELEASE_LTO','RUSTUP_TOOLCHAIN','AB_CENSUS_MS','AB_SAMPLE_MS','CARGO_HOME','CARGO_HTTP_PROXY'):
  env={'PATH':os.environ['PATH'],name:'fixture'}
  out=subprocess.check_output(['bash','-c',guard+'\nprintf "%s" "$overrides"'],env=env,text=True).strip()
  expected=name not in ('CARGO_HOME','CARGO_HTTP_PROXY')
  assert bool(out)==expected
  result['environment'][name]={'refused':bool(out)}

# Exercise the exact built()/mapfile/cp block against a Cargo-protocol fixture. A global
# build.target-dir can make both invocations report the same examples/ab output path.
with tempfile.TemporaryDirectory() as tmp:
 p=Path(tmp);head=p/'head';head.mkdir();b=p/'work';b.mkdir();base=b/'build-BASE';base.mkdir();w=p/'attempt';w.mkdir()
 mock=p/'tools';mock.mkdir();shared=p/'shared';shared.mkdir()
 cargo=mock/'cargo'
 cargo.write_text('''#!/usr/bin/env python3
import os,sys,json
from pathlib import Path
if sys.argv[1:]==['-V']:
 print('cargo fixture');sys.exit(0)
label='BASE' if Path.cwd().name=='build-BASE' else 'HEAD'
target=Path(os.environ['FIXTURE_TARGET'])
for i,a in enumerate(sys.argv):
 if a=='--example':
  name=sys.argv[i+1];out=target/name;out.write_text(label+' '+name+'\\n')
  print(json.dumps({'reason':'compiler-artifact','target':{'name':name,'kind':['example']},'executable':str(out)}))
''');cargo.chmod(0o755)
 rustc=mock/'rustc';rustc.write_text('#!/bin/sh\necho rustc-fixture\n');rustc.chmod(0o755)
 start=recipe.index('    built() {');end=recipe.index('    dir=$(python3',start)
 block=textwrap.dedent(recipe[start:end])
 # This sandbox has no /dev/fd for bash process substitution. For successful fixture
 # builds only, materialize the same path lists in temporary files; keep build/copy order.
 block=block.replace('mapfile -t base_built < <(built "$B/build-$base" ab)', 'built "$B/build-$base" ab > "$W/base-paths"\nmapfile -t base_built < "$W/base-paths"')
 block=block.replace('mapfile -t head_built < <(built "$PWD" ab windows)', 'built "$PWD" ab windows > "$W/head-paths"\nmapfile -t head_built < "$W/head-paths"')
 env=dict(os.environ,PATH=str(mock)+':'+os.environ['PATH'],B=str(b),W=str(w),base='BASE',FIXTURE_TARGET=str(shared))
 run=subprocess.run(['bash','-c','set -euo pipefail\nfail() { echo "$*" >&2; exit 1; }\n'+block],cwd=head,env=env,capture_output=True,text=True)
 assert run.returncode==0,(run.stdout,run.stderr)
 contents={n:(w/'bin'/n).read_text().strip() for n in ('ab-base','ab-head','windows')}
 assert contents['ab-base']==contents['ab-head']=='HEAD ab'
 result['shared_target_copy']={'shell_exit':run.returncode,'copies':contents,'baseline_preserved':False,'method':'recipe function/build/copy order; simulated Cargo messages/files; successful process substitutions expanded to temporary path lists because /dev/fd is unavailable; no compilation'}

result['census_source_review']={'refreshes_existing_tid':True,'guard_has_no_cpu_threshold':True,'rust_test_executed':False}
# A configuration ancestor of only the nested base checkout is outside both worktrees'
# clean checks and outside the two CARGO_HOME files recorded by the recipe.
with tempfile.TemporaryDirectory() as tmp:
 p=Path(tmp);head=p/'head';head.mkdir()
 def git(*args): return subprocess.check_output(['git','-C',str(head),*args],text=True,stderr=subprocess.DEVNULL).strip()
 git('init','-q');(head/'.gitignore').write_text('target/\n');(head/'Cargo.toml').write_text('[package]\nname="fixture"\nversion="0.1.0"\n')
 git('add','.');git('-c','user.name=Louis Maddox','-c','user.email=lmmx@users.noreply.github.com','commit','-qm','fixture')
 b=head/'target/three-team-policy';b.mkdir(parents=True);base=b/'build-BASE'
 git('worktree','add','--detach',str(base),'HEAD')
 (b/'.cargo').mkdir();conf=b/'.cargo/config.toml';conf.write_text('[build]\nrustflags=["-C", "opt-level=0"]\n')
 hs=git('status','--porcelain','--untracked-files=no')
 ms=subprocess.check_output(['bash','-c',measured+'\ngit status --porcelain --untracked-files=all -- "${measured[@]}"'],cwd=head,text=True).strip()
 bs=subprocess.check_output(['git','-C',str(base),'status','--porcelain','--untracked-files=all'],text=True).strip()
 assert hs==ms==bs==''
 # Cargo's documented cwd-and-ancestor search, checked as a path relation rather than
 # an invocation of unavailable Cargo. No environment override is needed.
 assert b in base.parents and b not in head.parents and b!=head
 result['base_only_config']={'head_status':hs,'measured_status':ms,'base_status':bs,'config_relative_to_head':str(conf.relative_to(head)),'ancestor_of_base_only':True,'recorded_by_CARGO_HOME_loop':False,'basis':'Git clean checks executed; configuration effect inferred from documented Cargo ancestor search'}
print(json.dumps(result,indent=2))
