import json,importlib.util,re
from pathlib import Path
p=Path('d059')
def mod(n,f):
 s=importlib.util.spec_from_file_location(n,p/f);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
c=mod('coord','coord.py');snap=json.loads((p/'inboxes.json').read_text());c.comments=lambda n:snap[str(n)]
audit=c.audit(c.CUTOVER);assert not [x for x in audit if not x['resolved']]
old=(p/'old.rs').read_text();new=(p/'run_team.rs').read_text()
assert 'self.arrived.fetch_add(1, Ordering::AcqRel) + 1 == of' in old and 'self.arrived.fetch_add(1, Ordering::AcqRel) + 1 == of' in new
def helper(s):return s.split('let helper = |j: usize| {',1)[1].split('let leader = ||',1)[0]
a=helper(old);b=helper(new)
a=a.replace('relay.built.pass(helpers, || unsafe { relay.end_build() });','if relay.built.arrive(helpers) { unsafe { relay.end_build() }; relay.built.set(); } else { relay.built.wait(); }')
def norm(s):
 s=re.sub(r'tmark!\([^;]+;','',s);s=re.sub(r'//[^\n]*','',s);return re.sub(r'\s+','',s)
assert norm(a)==norm(b)
import ast,types
tree=ast.parse((p/'run_fed_leader.py').read_text());fn=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='plan_of');ns={'re':re};exec(compile(ast.Module(body=[fn],type_ignores=[]),'plan_of','exec'),ns);r=types.SimpleNamespace(plan_of=ns['plan_of'])
correct=['mibz2-'+str(i) for i in range(3)]+['mibz2-coop-p'+str(i) for i in range(6)]
bad=correct.copy();bad[-1]='mibz2-2'
assert r.plan_of(' '.join(correct))==(3,0,6)
assert r.plan_of(' '.join(bad))==(4,0,5)
# Latest-name overwrite repairs only a rename that is actually sampled.
seen={i:n for i,n in enumerate(bad)};before=r.plan_of(' '.join(seen.values()));seen[8]=correct[8];after=r.plan_of(' '.join(seen.values()))
out={'helper_body_equal_after_expanding_old_pass_and_removing_trace':True,'same_arrival_AcqRel':True,'census_inherited_name':before,'after_observed_rename':after,'unsampled_rename_retains_wrong_plan':True,'scope':'Source expansion and reporter predicate probes; not a Rust memory model or scheduling execution','audit':audit,'route':c.route('coordinator','REVIEW')}
(p/'results.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))

