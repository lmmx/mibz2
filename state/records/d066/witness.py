import json,re,subprocess,tempfile,pathlib
d=json.load(open('d066-inputs.json')); s=d['source']
recipe=d['recipe'].split('bench-telemetry-check:\n')[1].split('\n# The pipelined')[0]
recipe='\n'.join(l[4:] if l.startswith('    ') else l for l in recipe.splitlines())
finish=recipe[recipe.index('finish() {'):recipe.index('trap finish EXIT')]
out=[]
for word in ['CHECKED','FAILED','REFUSED','INCOMPLETE']:
 with tempfile.TemporaryDirectory() as p:
  root=pathlib.Path(p);(root/'bench').mkdir();(root/'bench/bundle.sh').write_text('exit 0\n')
  r=subprocess.run(['bash','-c','W=$PWD; outcome='+word+'\n'+finish+'\nfinish'],cwd=p,capture_output=True,text=True)
  out.append({'outcome':word,'exit':r.returncode})
assert [x['exit'] for x in out]==[0,1,1,3]
inherit=s[s.index('pub fn inherit_status('):s.index('/// One inheritance region:')]
assert '("UNAVAILABLE", e.clone())' in inherit
assert 'e.starts_with("its ")' in inherit
assert 'format!("reading the counter: {e}")' in s
mux=s[s.index('// multiplex-live: 24'):s.index('// user-only-fallback: the fallback')]
assert '.filter(|r| r.1.as_ref().is_ok_and(|x| !x.whole()))' in mux
assert 'Ok(()) if partial.is_empty()' in mux
# Source-checked branch models, not live PMU execution.
def inherited_error(e): return 'INCOMPLETE' if e.startswith('its ') else 'UNAVAILABLE'
def mux_status(readings):
 partial=[n for n,r in readings.items() if isinstance(r,dict) and not (r['enabled']>0 and r['running']==r['enabled'])]
 return 'OBSERVED' if not partial else 'PASS' # formatter consistently lists the partial readings
cases=[{'case':'inheritance read EIO','actual':inherited_error('reading the counter: Input/output error'),'required':'INCOMPLETE'},
 {'case':'all multiplex probe reads failed','actual':mux_status({'cycles':'its enable failed','task_clock':'reading the counter: EIO'}),'required':'INCOMPLETE'},
 {'case':'cycles failed, instructions partial','actual':mux_status({'cycles':'its enable failed','instructions':{'enabled':100,'running':50}}),'required':'INCOMPLETE'}]
print(json.dumps({'exit_fix':out,'remaining_status_counterexamples':cases},indent=2))
