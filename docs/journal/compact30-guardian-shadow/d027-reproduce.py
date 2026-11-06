"""D027: reproduce reporting predicates only; no decoder, timing or perf invocation.
Usage: python d027-reproduce.py /path/to/team_size.py > d027-reproduction.json
Reporter pin: b8b8ce473c7acde505c266768fcd38a7f1f1eacb
"""
import contextlib, copy, importlib.util, io, json, pathlib, sys, tempfile
from types import SimpleNamespace

spec = importlib.util.spec_from_file_location("reviewed_reporter", sys.argv[1])
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

def thread_names(leaders, helpers):
    return " ".join([f"mibz2-{i}=0.1" for i in range(leaders)] +
                    [f"mibz2-coop-{i}=0.05" for i in range(helpers)])

def fixture():
    out = []
    for t in [8, 10, 12, 16]:
        for arm in ["mibz2", "mibz2-coop2", "mibz2-coop3", "mibz2-coop4"]:
            k = 2 if arm == "mibz2" else int(arm[-1])
            leaders = 4 if arm == "mibz2" else t // k
            for i in range(20):
                primary = (arm,t) in [("mibz2-coop3",12),("mibz2-coop4",16)]
                out.append(dict(corpus="wikidata",arm=arm,threads=t,round=i,hash="same",
                    out=209700000,wall=1/1.05 if primary else 1.0,cpu=2.0,rss_kb=100000,
                    thread_cpu=thread_names(leaders,leaders*(k-1))))
    return out

def evaluate(rows):
    with tempfile.TemporaryDirectory() as td:
        path = pathlib.Path(td)
        raw = path/"runs.jsonl"
        raw.write_text("".join(json.dumps(r)+"\n" for r in rows))
        a = SimpleNamespace(runs=str(raw),corpus="wikidata",ref="mibz2@8",counters=None,
            out=str(path/"report"),rounds=20,ref_plan="4x2",
            control="mibz2-coop2@8=4x2",
            primary="mibz2-coop3@12=4x3,mibz2-coop4@16=4x4")
        with contextlib.redirect_stdout(io.StringIO()):
            mod.report(a)
        result=json.loads((path/"report.json").read_text())
        return dict(input_rows=len(rows),cells=len(result["cells"]),
                    verdict=result["verdict"])

base=fixture()
cases={"clean":base}
dup=copy.deepcopy(base)
dup.append(copy.deepcopy(next(r for r in dup if r["arm"]=="mibz2-coop2" and r["threads"]==8 and r["round"]==0)))
cases["duplicate_control_round"]=dup
cases["missing_secondary_cell"]=[r for r in base if (r["arm"],r["threads"])!=("mibz2-coop2",16)]
bad=copy.deepcopy(base)
next(r for r in bad if r["arm"]=="mibz2-coop4" and r["threads"]==16 and r["round"]==0)["thread_cpu"]=thread_names(4,11)
cases["wrong_primary_helpers"]=bad
cases["missing_primary_round"]=[r for r in base if (r["arm"],r["threads"],r["round"])!=("mibz2-coop3",12,0)]
bad=copy.deepcopy(base)
next(r for r in bad if r["arm"]=="mibz2-coop3" and r["threads"]==12 and r["round"]==0)["out"]=1
cases["inconsistent_output_length"]=bad
results={name:evaluate(rows) for name,rows in cases.items()}
print(json.dumps(results,indent=2,sort_keys=True))
