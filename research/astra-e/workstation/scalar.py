"""Independent matched i9 measurement for the scalar cap shortcut."""
from contextlib import chdir, redirect_stderr
import io
import json
import math
import os
import re
from pathlib import Path
import statistics
import subprocess
import sys
from tempfile import TemporaryDirectory
from unittest.mock import patch

import assess
import edge
import refresh
import moonshot

HERE=Path(__file__).resolve().parent
ARMS=["lazy","opend","predecessor","candidate"]
WORKERS=[7,8,9,10]
ROUNDS=48
CANDIDATE="research/astra-e/front-fusion/cap-batch/worker/candidate.rs"
PREDECESSOR="research/astra-e/edge-budget/candidate.rs"
TREATMENT="d0a42980e896211906234c7679b38760ae44a46e"
EDGE="badb2aec3624bd65606c429b2e6f79e9d9d0050f"
PIN={"last_working_path_revision":"75297a1d57b2bdf262c3a19f3064848dedf2c493"}

def select(out,other):
    target=Path(os.environ["CARGO_TARGET_DIR"])
    counterpart=out/"predecessor-build"
    counterpart.mkdir()
    def both(directory,label):
        failure=refresh.compatibility(directory,label)
        if failure:return failure
        revision=refresh.git("rev-parse","origin/master") if label=="fresh" else PIN["last_working_path_revision"]
        with chdir(other),patch.dict(os.environ,CARGO_TARGET_DIR=str(target/"predecessor"),MOONSHOT_CANDIDATE=str(other/PREDECESSOR)):
            refresh.project(revision)
            failure=refresh.compatibility(counterpart,label)
        if failure:return dict(failure,log="predecessor-build/"+failure["log"])
        with (directory/f"{label}-matched-interface.log").open("w") as log:
            result=subprocess.run([sys.executable,str(HERE/"scalar.py"),"probe"],stdout=log,stderr=subprocess.STDOUT)
        if result.returncode:return {"stage":"matched-interface","exit_code":result.returncode}
        return None
    refresh.refresh(PIN,out,both)

def roster():
    return [(r,w,a) for r in range(ROUNDS) for w in WORKERS for a in moonshot.order(ARMS,r+w)]

def runner(treatment,predecessor):
    result=edge.paired_runner(treatment,predecessor)
    result.a.rounds=ROUNDS
    assert result.a.settle==2 and result.a.workers==WORKERS and result.a.repeat==1
    return result

def decision(rows):
    # Keep the registered 16-round edge implementation immutable. Its single-use validator
    # takes the round count from the module; all wall, RSS and maximum-peak rules remain exact.
    with patch.object(assess,"ROUNDS",ROUNDS):
        result=edge.decision(rows)
    by={(r["round"],r["workers"],r["arm"]):r["wall_s"] for r in rows}
    round_ratios=[math.exp(statistics.mean(math.log(by[i,w,"candidate"]/by[i,w,"predecessor"]) for w in WORKERS)) for i in range(ROUNDS)]
    pooled=moonshot.paired_ratio(round_ratios,[1.0]*ROUNDS)
    headroom={}
    for w in WORKERS:
        cell=result["workers"][w]
        lazy=cell["lazy"]
        peak_ok=(lazy["rss"]["hi"]<=1 and lazy["median_peak_ratio"]<=1 and lazy["max_peak_kb"]<=lazy["ref_max_peak_kb"])
        headroom[w]={"predecessor_wall_hi":cell["predecessor"]["wall"]["hi"],"peak_vs_lazy_ok":peak_ok}
    result.update(incremental_primary={"equal_weight_round_ratios":round_ratios,"wall":pooled,"per_w_guard":headroom,
        "criterion_met":pooled["hi"]<1 and all(x["predecessor_wall_hi"]<=1.02 and x["peak_vs_lazy_ok"] for x in headroom.values())})
    return result

def probe():
    source=Path("bench/moonshot/src/main.rs").read_text()
    # The scorer has no --slots option. Verify its actual compiled worker/slot arguments.
    if not re.search(r"parallel_workers_with::<K>\(\s*std::io::Cursor::new\(data\),\s*workers,\s*(?:4\s*\*\s*workers|workers\s*\*\s*4),",source):
        raise ValueError("scorer's paid invocation does not certify 4W slots")
    first,second=Path("scalar"),Path("edge")
    scorer=runner(first,second)
    def synthetic(s,arm,w,corpus,perf=None):
        wall,peak={(first,"candidate"):(.75,80),(second,"candidate"):(.78,85),
                   (second,"opend"):(.95,95),(second,"lazy"):(1,100)}[s.bin,arm]
        return dict(arm=arm,workers=w,out_bytes=assess.BYTES,wall_s=wall,cpu_s=1.,peak_rss_kb=peak)
    with TemporaryDirectory() as root,patch.object(moonshot.Scorer,"run_once",synthetic),patch.object(moonshot.time,"sleep",lambda _:None),redirect_stderr(io.StringIO()):
        scorer.timing(Path(root)/"unused",Path(root),ARMS)
        rows=edge.read_rows(Path(root))
    assert [(r["round"],r["workers"],r["arm"])for r in rows]==roster()
    good=decision(rows);assert good["i9_criterion_met"] and good["incremental_primary"]["criterion_met"]
    for r in rows:
        if r["arm"]=="candidate" and r["workers"]==9:r["wall_s"]=.819
    bad=decision(rows)
    assert bad["incremental_primary"]["wall"]["hi"]<1 and not bad["incremental_primary"]["criterion_met"] and not bad["i9_criterion_met"]
    for r in rows:
        if r["arm"]=="candidate" and r["workers"]==9:r.update(wall_s=.75,peak_rss_kb=101)
    bad=decision(rows);assert not bad["incremental_primary"]["criterion_met"] and not bad["i9_criterion_met"]
    print("768 ordered samples, 48 matched four-W rounds and four verified build/arm identities. A regressing W9 defeats a favorable pooled ratio; excess W9 peak defeats both decision rules.")

def dry_workers(first,second,corpus,out):
    cpus=sorted(os.sched_getaffinity(0))
    is_target="i9-10900K" in Path("/proc/cpuinfo").read_text()
    outputs=[]; gates=[]
    checks=[("lazy","lazy",second),("opend","opend",second),("predecessor","candidate",second),("candidate","candidate",first)]
    for w in WORKERS:
        for label,arm,binary in checks:
            # --workers launches W explicit decoder workers even on the quota-limited cloud.
            row=json.loads(subprocess.check_output([binary,"run","--arm",arm,"--workers",str(w),"--file",corpus]))
            assert row["arm"]==arm and row["workers"]==w and row["out_bytes"]==209700196
            outputs.append(dict(label=label,compiled_arm=arm,workers=w,slots=4*w,out_bytes=row["out_bytes"]))
        if len(cpus)<w:
            if is_target: raise RuntimeError(f"target dry cannot exercise capped availability W{w}")
            continue
        prefix=["taskset","-c",",".join(map(str,cpus[:w]))]
        available=json.loads(subprocess.check_output([*prefix,first,"info"]))["cpus"]
        if available!=w:
            if is_target: raise RuntimeError(f"target dry got {available} CPUs for W{w}")
            continue
        from test_protocol import check_gate
        for label,arm,binary in checks:
            path=out/f"gate-{label}-w{w}.jsonl"
            with path.open("w") as f:subprocess.run([*prefix,binary,"check","--quick","--arm",arm,"--wikidata",corpus],stdout=f,check=True)
            check_gate(path,arm)
        gates.append(w)
    (out/"dry-coverage.json").write_text(json.dumps({"target_machine":is_target,"explicit_workers":outputs,"affinity_gate_worker_counts":gates},indent=2)+"\n")

if __name__=="__main__":
    mode=sys.argv[1]
    if mode=="select":select(*map(Path,sys.argv[2:]))
    elif mode=="probe":probe()
    elif mode=="plan":print(json.dumps(roster()))
    elif mode=="dry":dry_workers(*map(Path,sys.argv[2:]))
    elif mode=="sweep":
        candidate,old,corpus,out=map(Path,sys.argv[2:])
        runner(candidate,old).timing(corpus,out,ARMS)
        (out/"decision.json").write_text(json.dumps(decision(edge.read_rows(out)),indent=2,allow_nan=False)+"\n")
    elif mode=="assess":print(json.dumps(decision(edge.read_rows(Path(sys.argv[2]))),indent=2,allow_nan=False))
    else:raise SystemExit("expected select, probe, plan, dry, sweep or assess")
