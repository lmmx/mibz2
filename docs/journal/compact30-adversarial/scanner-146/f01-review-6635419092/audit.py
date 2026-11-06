import ast, json, re, pathlib, difflib, subprocess, sys
root=pathlib.Path(__file__).parent
def read(name):
    if len(sys.argv)>1:
        manifest=json.loads((root/"inputs.json").read_text())
        return subprocess.check_output(["git","-C",sys.argv[1],"show",manifest[name]],text=True)
    return (root/name).read_text()
def ev(s,r):
    n=ast.parse(s,mode="eval").body
    def go(n):
        if isinstance(n,ast.Name): return r[n.id]
        if isinstance(n,ast.Constant) and type(n.value)==int: return n.value
        if isinstance(n,ast.BinOp):
            a,b=go(n.left),go(n.right)
            if isinstance(n.op,ast.Add): return a+b
            if isinstance(n.op,ast.Sub): return a-b
            if isinstance(n.op,ast.Mult): return a*b
            if isinstance(n.op,ast.FloorDiv): return a//b
        raise ValueError(ast.dump(n))
    return go(n)
out={}
for k in ["scan","desc"]:
    ledger=json.loads(read(f"{k}-ledger.json"))
    records=[json.loads(l) for l in read(f"{k}-counted.jsonl").splitlines() if l.strip()]
    r=next(r for r in records if r["input"]=="wikidata-20261026.bz2" and r["permille"]==0 and not r["extra_starts"])
    assert r==ledger["counted_record"]
    source=read(f"{k}-source.rs")
    old=read(f"{k}-old.rs")
    oldfields=set(re.findall(r"pub (\w+): u64",old))
    newfields=set(re.findall(r"pub (\w+): u64",source))-oldfields
    checks=[]
    for row in ledger["classes"]:
        es=row["counter"].split(" & ")
        vals=[ev(e,r) for e in es]
        assert all(v==row["frequency"] for v in vals),row
        assert row["delta"]==(row["candidate"]-row["baseline"])*row["frequency"]
        checks.append({"function":row["function"],"frequency":row["frequency"],"expressions":es,"values":vals})
    delta=sum(x["delta"] for x in ledger["classes"])
    lower=-delta-ledger["library_added"]
    upper=lower+ledger["library_removed"]
    assert [lower,upper]==ledger["interval_removed"]
    # Distinct coincident-frequency path must be checked separately.
    mutated=dict(r);mutated["bitmap_memsets"]+=1
    row=next(x for x in ledger["classes"] if x["frequency"]==230)
    assert not all(ev(e,mutated)==230 for e in row["counter"].split(" & "))
    source_lines=source.splitlines()
    sites={f:[i+1 for i,l in enumerate(source_lines) if re.search(r"\."+f+r"\s*\+=",l)] for f in sorted(newfields)}
    assert all(sites.values())
    out[k]={"classes":len(checks),"counter_expressions":sum(len(x["expressions"]) for x in checks),
            "new_counter_fields":len(newfields),"increment_lines":sites,"checks":checks,
            "interval": [lower,upper],"counter_supported_delta":delta,
            "distinct_bitmap_mutation_detected":True}
    (root/f"{k}-source.diff").write_text("".join(difflib.unified_diff(old.splitlines(True),source.splitlines(True))))
(root/"audit.json").write_text(json.dumps(out,indent=2)+"\n")
print(json.dumps({k:{x:v for x,v in d.items() if x not in ["checks","increment_lines"]} for k,d in out.items()},indent=2))
