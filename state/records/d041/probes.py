import json,os,pathlib,subprocess,tempfile
ROOT=pathlib.Path(__file__).resolve().parent
recipe=(ROOT/'recipe.sh').read_text()
guard=recipe[recipe.index('configs() {'):recipe.index('# the toolchain each build')]
definition=guard.split('\nonly_base=')[0]
loop=recipe[recipe.index('  for f in "${CARGO_HOME'):recipe.index('  echo "census every')]
fail=recipe[recipe.index('fail() {'):recipe.index('# the commits')]
results={'method':'Exact new shell snippets, synthetic filesystem/config files only; no Cargo invocation, decoding or timing','guard':[],'inventory':[]}
for location in ['none','root','target','policy']:
 for basename in ['config','config.toml']:
  with tempfile.TemporaryDirectory(prefix='d041-') as tmp:
   root=pathlib.Path(tmp);head=root/'checkout';b=head/'target/three-team-policy';(b/'build-base').mkdir(parents=True)
   parent={'root':head,'target':head/'target','policy':b}.get(location)
   if parent:
    (parent/'.cargo').mkdir();(parent/'.cargo'/basename).write_text('# fixture\n')
   p=subprocess.run(['bash','-c','set -euo pipefail\n'+fail+guard],cwd=head,env=dict(os.environ,B=str(b),base='base'),capture_output=True,text=True,timeout=10)
   assert p.returncode==(0 if location=='none' else 1),(location,basename,p)
   results['guard'].append({'location':location,'file':basename,'exit':p.returncode})
for segment in ['plain','with space']:
 with tempfile.TemporaryDirectory(prefix='d041-') as tmp:
  root=pathlib.Path(tmp);ancestor=root/segment;head=ancestor/'checkout';head.mkdir(parents=True);(ancestor/'.cargo').mkdir();config=ancestor/'.cargo/config.toml';config.write_text('# fixture\n');home=root/'cargo-home';home.mkdir()
  p=subprocess.run(['bash','-c','set -euo pipefail\n'+definition+'\n'+loop+'\necho END'],cwd=head,env=dict(os.environ,CARGO_HOME=str(home)),capture_output=True,text=True,timeout=10)
  recorded=str(config) in p.stdout
  assert recorded==(segment=='plain') and p.returncode==0,(segment,p.stdout,p.stderr,p.returncode)
  results['inventory'].append({'ancestor':segment,'exit':p.returncode,'config_hash_recorded':recorded})
(ROOT/'results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps(results,indent=2))
