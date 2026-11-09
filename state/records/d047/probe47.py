import json,re,ast,hashlib
d=json.load(open("probe47-input.json"))
total=pipe=reach=countable=errors=jobs=0
candidates={}
for name,log in d["logs"].items():
    total+=int(re.search(r"^(\d+) cases",log)[1])
    for line in log.splitlines():
        m=re.match(r"\s*(\d+)\s+(mibz2 .*?): (.*)",line)
        if not m:continue
        n,label,rest=int(m[1]),m[2],m[3]
        pipe+=n;candidates[label]=candidates.get(label,0)+n
        v=re.search(r"(\d+) of (\d+) reached, (\d+) with an overlapped error, (\d+) overlapped jobs",rest)
        if v:
            a,b,c,e=map(int,v.groups());reach+=a;countable+=b;errors+=c;jobs+=e
defs={}
for node in ast.parse(d["script"]).body:
    if isinstance(node,ast.Assign):
        for target in node.targets:
            if isinstance(target,ast.Name) and target.id in ("M","M2_EDITS"):
                defs[target.id]=ast.literal_eval(node.value)
verified=[]
for key,(_,path,old,new) in defs["M"].items():
    if key=="M0":continue
    src=d["src"][path]
    edits=defs["M2_EDITS"] if key=="M2" else [(old,new)]
    for old,new in edits:
        assert src.count(old)==1,(key,"ambiguous replacement")
        src=src.replace(old,new)
    h=hashlib.sha1(b"blob "+str(len(src.encode())).encode()+b"\0"+src.encode()).hexdigest()
    expected=re.search(r"index [a-f0-9]+\.\.([a-f0-9]+)",d["patches"][key])[1]
    assert h.startswith(expected),(key,h,expected)
    verified.append(key)
survivors=[x["id"] for x in d["results"] if all(x.get(k,[0])[0]==0 for k in ("pipelined","coop_unit","miri","fuzz"))]
# Read's ordered-ready invariant after scanner end and all normal workers have finished.
# Mutant removes s and next; previous inverted, if any, still gets an error.
held={"s":5,"next":6}
ready_baseline={5:"Internal",6:"Internal",7:"success"}
ready_mutant={7:"success"}
def read_step(ready,taken,end):
    if taken in ready:return "result"
    if taken==end:return "end"
    return "wait"
assert read_step(ready_baseline,5,8)=="result"
assert read_step(ready_mutant,5,8)=="wait"
print(json.dumps(dict(total_cases=total,pipeline_cases=pipe,countable=countable,reached=reach,overlap_error_cases=errors,overlapped_jobs=jobs,candidates=candidates,mutation_blob_checks=verified,reported_survivors=survivors,outer_panic_model={"baseline":"result","M12":"wait at missing place 5 despite later results and end 8"},limits="Archived-log arithmetic, exact mutation blob reconstruction, and abstract read predicate only; not a Rust/Miri/fuzz/mutation execution."),indent=2))
