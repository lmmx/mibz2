import ast,contextlib,io,json,math,tempfile
from pathlib import Path
import share_nothing as sn
tree=ast.parse(Path(__file__).with_name("test_share_nothing.py").read_text())
ns={"sn":sn,"json":json,"math":math}
for node in tree.body:
 if isinstance(node,ast.FunctionDef) and node.name in ("cpu_model","make_attempt"):
  exec(compile(ast.Module(body=[node],type_ignores=[]),"fixture","exec"),ns)
make=ns["make_attempt"]
out={}
def check(name,mut=None):
 with tempfile.TemporaryDirectory() as t:
  d=make(Path(t),mutate=mut)
  bad,_=sn.admission_problems(str(d),False)
  out[name]={"admitted":not bad,"problems":bad[:2]}
check("fixture")
check("missing_both_corpus_digests",lambda p:(p["corpora"]["files"]["wikidata-l2"].pop("sha256"),p["identity"]["corpora_after"].pop("wikidata-l2")))
check("empty_sysfs_and_null_timestamp",lambda p:[r.update(machine_pre={},t_spawn_unix_ns=None) for r in p["runs"]])
check("missing_head_frozen_sources",lambda p:None)
check("wrong_output_hash",lambda p:p["runs"][7].update(hash="bad"))
check("deadline_failure",lambda p:p["slots"][7].update(rc=124))
check("freeze_mismatch",lambda p:p["identity"].update(inputs_match_frozen=False))
check("altered_corpus_hash",lambda p:p["identity"]["corpora_after"].update({"wikidata-l2":"changed"}))
with tempfile.TemporaryDirectory() as t:
 d=make(Path(t))
 with contextlib.redirect_stdout(io.StringIO()):
  rc=sn.main(["judge",str(d)])
 j=json.loads((d/"judgement.json").read_text())
 out["baseline_full_judge"]={"rc":rc,"outcome":j["outcome"]}
with tempfile.TemporaryDirectory() as t:
 root=Path(t);(root/"proc/self").mkdir(parents=True)
 (root/"proc/self/cgroup").write_text("0::/missing.scope\n")
 out["absent_cgroup_mount"]=sn.cgroup_quota(t)
out["schedule_digest"]=sn.digest(sn.schedule())
out["ci_counterexample"]=sn.geo([math.log(.8)]*30+[math.log(1.2)]*30)
print(json.dumps(out,indent=2))
