"""Check recursive discovery, each joined entry's bytes, and independent statistical aggregation."""
from collections import Counter, defaultdict
import csv
import itertools
import json
import math
from pathlib import Path
import subprocess

from receipts import ROOT, OUT, DATA, SCORER, module, write

EVAL = Path("bench/moonshot/eval.py")


def git(*args):
    return subprocess.check_output(["git",*map(str,args)])


def metrics(rows,k):
    errors=[abs(math.log(r[k]/r["indep"]["ratio"])) for r in rows]
    correct=pairs=0
    for a,b in itertools.combinations(rows,2):
        if (a["w"],a["indep"]["host"]) == (b["w"],b["indep"]["host"]):
            pairs+=1
            correct+=(a[k]<b[k])==(a["indep"]["ratio"]<b["indep"]["ratio"])
    return dict(n=len(errors),male=sum(errors)/len(errors) if errors else None,right=correct,pairs=pairs)


def emit_csv(name,rows):
    with (OUT / name).open("w") as f:
        writer=csv.DictWriter(f,fieldnames=rows[0],lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def main():
    assert __debug__ and git("rev-parse","HEAD").decode().strip()==DATA
    assert EVAL.read_bytes()==git("show",f"{SCORER}:{EVAL}")
    reader=module("reader",EVAL)
    old=module("old_audit",ROOT/"retrospective/audit.py")
    forecasts=reader.forecasts(ROOT)
    results=reader.results(ROOT)
    rows=sorted(reader.join(forecasts,results),key=lambda r:(r["name"],int(r["w"])))
    records={r["source"]:old.receipt(r["source"]) for r in results}
    assert len({rid for rid,_ in records.values()})==len(records)
    assert len({(records[r["source"]][0],r["w"]) for r in results})==len(results)
    assert len([r for r in rows if r.get("indep")]) + sum(r.get("indep") is None for r in rows)==len(rows)
    joined=[]
    register=defaultdict(list)
    counts=Counter()
    for row in rows:
        res=row.get("indep")
        kind=row.get("how","no forecast" if res else "no independent result")
        counts[kind]+=1
        rid, measured=records[res["source"]] if res else (None,None)
        local=old.forecast_digest(row) if res and row.get("how") else None
        local_build=None
        if res and row.get("how"):
            assert local==measured and local
            entry="research/astra-e/"+row["candidate"].split("research/astra-e/",1)[1]
            local_build=row.get("commit") or (Path(row["dir"])/"manifest.txt").read_text().splitlines()[0]
            assert old.sha(git("show",f"{local_build}:{entry}"))==local
            assert old.sha(git("show",f"{res['head']}:{entry}"))==measured
        joined.append(dict(forecast=row["name"],version=row["version"],scorer_commit=row.get("scorer_commit"),
                           forecast_build=row.get("commit"),audited_build=local_build,forecast_source=row.get("source"),w=row["w"],
                           join=kind,forecast_entry_sha256=local,measurement_entry_sha256=measured,
                           run=rid,measurement_head=res and res["head"],host=res and res["host"],
                           receipt=res and res["source"],local=row.get("local"),proxy=row.get("proxy"),
                           measured=res and res["ratio"],lo=res and res["lo"],hi=res and res["hi"],
                           official_i9_calibration=bool(row.get("proxy_scored"))))
        if res:
            register[rid].append((row,kind))
    emit_csv("joins.csv",joined)
    coverage=[]
    for rid,data in sorted(register.items()):
        cells=sorted({int(r["w"]) for r,_ in data})
        matched=sorted({int(r["w"]) for r,k in data if k!="no forecast"})
        r=data[0][0]["indep"]
        coverage.append(dict(run=rid,host=r["host"],head=r["head"],source=r["path"],
                             receipt=r["source"],workers=" ".join(map(str,cells)),
                             joined_workers=" ".join(map(str,matched)),official_rows=len(data)))
    emit_csv("coverage.csv",coverage)
    official={}
    for version in sorted({r["version"] for r in rows if r.get("indep") and r["version"]!="-"}):
        subset=[r for r in rows if r["version"]==version and r.get("indep")]
        groups=[("proxy_i9","proxy",[r for r in subset if r.get("proxy_scored")])]
        for k in ("proxy","local"):
            groups.append((k+"_other",k,[r for r in subset if r.get(k) is not None and not r.get("proxy_scored")]))
        official[version]={}
        for label,k,group in groups:
            m=metrics(group,k)
            assert m==reader.accuracy(group,k)
            m["distinct_measurement_cells"]=len({(r["indep"]["source"],r["w"]) for r in group})
            m["distinct_prediction_measurement_tuples"]=len({(records[r["indep"]["source"]][1],r["w"],
                    r[k],r["indep"]["source"]) for r in group})
            m["i9_measurement_rows"]=sum(r["indep"]["i9"] for r in group)
            same_source=same_run=distinct=strict_right=0
            for a,b in itertools.combinations(group,2):
                if (a["w"],a["indep"]["host"]) != (b["w"],b["indep"]["host"]):
                    continue
                ai,ad=records[a["indep"]["source"]]; bi,bd=records[b["indep"]["source"]]
                same_source+=ad==bd
                same_run+=ai==bi
                if ad!=bd and a[k]!=b[k] and a["indep"]["ratio"]!=b["indep"]["ratio"]:
                    distinct+=1
                    strict_right+=(a[k]<b[k])==(a["indep"]["ratio"]<b["indep"]["ratio"])
            m.update(rank_pairs_same_entry=same_source,rank_pairs_same_run=same_run,
                     strict_cross_entry_pairs=distinct,strict_cross_entry_right=strict_right)
            official[version][label]=m
    heldout=[]
    indexed={int(r["w"]):r for r in results if "edge_i9_run" in r["source"]}
    with (ROOT/"results/edge-route-pricing/heldout-i9.csv").open() as f:
        for r in csv.DictReader(f):
            if r["comparison"] not in ("edge/lazy","edge/combined"):continue
            w=int(r["workers"]); src=indexed[w]
            recfile,key=src["source"].split(":")
            rec=json.loads(Path(recfile).read_text())[key]
            ref="master LazyCells" if r["comparison"]=="edge/lazy" else "frozen combined"
            obs=rec["candidate_over_reference"][ref][str(w)]
            assert all(float(r[a])==obs[b] for a,b in [("observed_wall","geomean"),("observed_lo","lo"),("observed_hi","hi")])
            pred=float(r["conditional_feedback"]) if r["conditional_feedback"] else None
            if pred: assert math.isclose(math.log(pred/obs["geomean"]),float(r["log_feedback_over_observed"]),abs_tol=1e-12)
            heldout.append(dict(w=w,comparison=r["comparison"],prediction=pred,
                                measured=obs["geomean"],lo=obs["lo"],hi=obs["hi"],
                                log_error=math.log(pred/obs["geomean"]) if pred else None))
    emit_csv("conditional-edge.csv",heldout)
    measured_cond={label:sum(abs(r["log_error"]) for r in heldout if r["comparison"]==label and r["prediction"])/3
                   for label in ("edge/lazy","edge/combined")}
    write("audit.json",dict(data=DATA,upstream_path_revision=SCORER,evaluator_sha256=old.sha(EVAL.read_bytes()),
           forecasts=len(forecasts),results=len(results),runs=len(records),output_rows=len(rows),joins=counts,
           joined_receipt_sha_matches=sum(r["how"] in ("directory","path only","sha256") for r in rows if r.get("how")),
           accuracy=official,combined_proxy=old.proxy_replay(reader,rows),edge_conditional_male=measured_cond))


if __name__=="__main__":
    main()
