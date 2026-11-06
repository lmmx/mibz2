import json, os, pathlib, subprocess, tempfile

ROOT=pathlib.Path(__file__).resolve().parent
recipe=(ROOT/'recipe.sh').read_text()
built=recipe[recipe.index('built() {'):recipe.index('dir=$(python3')]
finish=recipe[recipe.index('finish() {'):recipe.index('fail() {')]
results={'method':'Untimed Bash fixtures extracted from pinned recipe. /dev/fd unavailable: two process substitutions replaced by named FIFOs, keeping background producers and mapfile status behavior. Cargo is a synthetic producer; no Rust build or decoder run.'}

fake='''#!/usr/bin/env python3
import json,os,pathlib,sys
cwd=pathlib.Path.cwd(); base=cwd.name.startswith('build-')
target=pathlib.Path(os.environ['SHARED']) if os.environ['CASE']=='shared' else cwd/'target'/'configured-triple'/'release'/'examples'
target.mkdir(parents=True,exist_ok=True)
names=['ab'] if base else ['ab','windows']
for name in names:
 p=target/name;p.write_text(('BASE' if base else 'HEAD')+':'+name)
 print(json.dumps({'reason':'compiler-artifact','executable':str(p),'target':{'name':name,'kind':['example']}}),flush=True)
sys.exit(7 if base and os.environ['CASE']=='failed-build' else 0)
'''
results['artifacts']=[]
for case in ['distinct','shared','failed-build']:
 with tempfile.TemporaryDirectory() as tmp:
  root=pathlib.Path(tmp);head=root/'head';head.mkdir();b=root/'builds';(b/'build-base').mkdir(parents=True);w=root/'attempt';w.mkdir();cmd=root/'commands';cmd.mkdir();(cmd/'cargo').write_text(fake);(cmd/'cargo').chmod(0o755)
  body=built
  for label,call in [('base','built "$B/build-$base" ab'),('head','built "$PWD" ab windows')]:
   old=f'mapfile -t {label}_built < <({call})'
   new=f'mkfifo "$W/{label}.fifo"\n({call}) > "$W/{label}.fifo" &\nproducer=$!\nmapfile -t {label}_built < "$W/{label}.fifo"\nif wait "$producer"; then producer_status=0; else producer_status=$?; fi\nprintf "%s\\n" "$producer_status" > "$W/{label}-producer-status"'
   assert old in body;body=body.replace(old,new)
  script='set -euo pipefail\nfail(){ echo "$*" >&2; exit 1; }\n'+body
  env=dict(os.environ,PATH=str(cmd)+':'+os.environ['PATH'],B=str(b),W=str(w),base='base',SHARED=str(root/'shared'/'triple'/'release'/'examples'),CASE=case)
  p=subprocess.run(['bash','-c',script],cwd=head,env=env,capture_output=True,text=True,timeout=15)
  r={'case':case,'shell_exit':p.returncode,'stderr':p.stderr,'base_producer_exit':(w/'base-producer-status').read_text().strip(),'head_producer_exit':(w/'head-producer-status').read_text().strip(),'copies':{x.name:x.read_text() for x in (w/'bin').iterdir()}}
  results['artifacts'].append(r)
assert results['artifacts'][0]['copies']['ab-base']=='BASE:ab'
assert results['artifacts'][1]['copies']['ab-base']=='HEAD:ab'
assert results['artifacts'][2]['shell_exit']==0 and results['artifacts'][2]['base_producer_exit']=='7'

results['finish']=[]
for outcome,status in [('VALID PASS',0),('VALID FAIL',0),('INVALID: fixture',0),('REPORT ERROR: fixture',0),('REFUSED: fixture',1),('INCOMPLETE: fixture',7)]:
 with tempfile.TemporaryDirectory() as tmp:
  root=pathlib.Path(tmp);(root/'bench').mkdir();w=root/'attempt';w.mkdir()
  (root/'bench/bundle.sh').write_text((ROOT/'bundle.sh').read_text())
  p=subprocess.run(['bash','-c','set -euo pipefail\n'+finish+'\nexit '+str(status)],cwd=root,env=dict(os.environ,W=str(w),outcome=outcome),capture_output=True,text=True,timeout=15)
  results['finish'].append({'outcome':outcome,'incoming_status':status,'shell_exit':p.returncode,'recorded':(w/'outcome.txt').read_text().strip(),'bundle_exists':pathlib.Path(str(w)+'.tar.gz').exists()})
assert [r['shell_exit'] for r in results['finish']]==[0,0,1,2,1,7]
assert all(r['bundle_exists'] and r['recorded']==r['outcome'] for r in results['finish'])

seen={}
for tid,name in [(7,'ab\n'),(8,'mibz2-0\n'),(7,'mibz2-coop-1\n'),(8,'mibz2-0\n')]:seen[tid]=name.strip()
assert list(seen.values())==['mibz2-coop-1','mibz2-0']
results['census']={'source_review':'census_saw uses unconditional BTreeMap.insert; watcher invokes it on each successful comm read; keyed by tid, preserving multiplicity','transition_model':seen,'rust_test_run':False,'limitation':'cargo/rustc unavailable; deterministic Python model, not execution of Rust test or live /proc sampling'}
(ROOT/'results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps(results,indent=2))
