"""Authenticate an untimed or target scalar-cap workstation bundle and replay all decisions."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile

import scalar

RECIPE="cafcab595600d1430e67481863f92464b5544a4e"
RELEASE="530ed404a014a414ef0b62adfd947fa14d4533e6"
WIKI="82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
X4="0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"
def git(*args):return subprocess.check_output(["git",*args])
def verify(path):
    with tarfile.open(path) as z:
        items={}
        for r in z:
            assert r.isfile()
            name=r.name.split("/",1)[1]
            assert name not in items
            items[name]=z.extractfile(r).read().decode()
    mode=items["mode.txt"].strip()
    assert mode in ("dry","measure")
    assert items["toolchain.txt"].startswith("rustc 1.99.1 ")
    sources=items["matched-sources.txt"]
    for token in [f"recipe {RECIPE}",f"scalar {scalar.TREATMENT} {scalar.CANDIDATE}",f"measured edge {scalar.EDGE} {scalar.PREDECESSOR}"]:
        assert token in sources
    for rev,entry in [(scalar.TREATMENT,scalar.CANDIDATE),(scalar.EDGE,scalar.PREDECESSOR)]:
        assert hashlib.sha256(git("show",f"{rev}:{entry}")).hexdigest() in sources
    heads=items["commits.txt"].split()
    assert heads[0]==scalar.TREATMENT and heads[2:]==[scalar.EDGE,RELEASE]
    projected=json.loads(items["scorer-refresh.json"])
    revision=items["scorer-revision.txt"].strip()
    assert projected["proceed"] and projected["selected_path_revision"]==revision
    assert projected["selected_scorer_tree"]==git("rev-parse",revision+":bench/moonshot").decode().strip()
    assert items["scorer-blobs.txt"]==items["predecessor-scorer-blobs.txt"]
    for line in items["scorer-blobs.txt"].splitlines():
        descriptor,filename=line.split("\t")
        assert descriptor.split()[1]==git("rev-parse",f"{revision}:{filename}").decode().strip()
    for line in items["protocol-blobs.txt"].splitlines():
        descriptor,filename=line.split("\t")
        assert descriptor.split()[2]==git("rev-parse",f"{RECIPE}:{filename}").decode().strip()
    assert [s.split()[0]for s in items["inputs.sha256"].splitlines()]==[WIKI,X4]
    binaries=[s.split()[0]for s in items["binaries.sha256"].splitlines()]
    assert len(binaries)==len(set(binaries))==2
    for name,entry in [("scorer-info.json",scalar.CANDIDATE),("predecessor-info.json",scalar.PREDECESSOR)]:
        info=json.loads(items[name])
        assert info["candidate"].endswith("/"+entry) and set(scalar.ARMS)-{"predecessor"}<=set(info["arms"])
    report=next(s for s in items["scorer-report.txt"].splitlines() if s.startswith("scorer v"))
    assert report in items["predecessor-report.txt"].splitlines()
    assert json.loads(items["roster.json"])==list(map(list,scalar.roster()))
    def gate(name,arm):
        *cases,verdict=map(json.loads,items[name].splitlines())
        assert verdict==dict(arm=arm,verdict="PASS",failures=0)
        assert all(r["pass"] and r["failure"] is None for r in cases)
        assert any("wikidata" in r["case"] and r["bytes"]==209700196 for r in cases)
        return [(r["case"],r["bytes"] if r["case"]!="corpus text level 9" else None) for r in cases]
    full=gate("candidate-gate/check.jsonl","candidate")
    assert full==gate("predecessor-gate/check.jsonl","candidate")
    quick=gate("gate-lazy.jsonl","lazy")
    assert quick==gate("gate-opend.jsonl","opend")
    result=dict(recipe=RECIPE,mode=mode,sha256=hashlib.sha256(path.read_bytes()).hexdigest(),scorer_report=report,
                scorer_path_revision=revision,binaries=binaries,full_gate_each=len(full),reference_gate_each=len(quick),
                planned_samples=len(scalar.roster()),target_cpu="i9-10900K" in items["lscpu.txt"])
    if mode=="dry":
        evidence=json.loads(items["dry-coverage.json"])
        expected=[dict(label=a,compiled_arm="candidate" if a=="predecessor" else a,workers=w,slots=4*w,out_bytes=209700196)
                  for w in scalar.WORKERS for a in scalar.ARMS]
        assert evidence["explicit_workers"]==expected and evidence["target_machine"]==result["target_cpu"]
        if result["target_cpu"]:assert evidence["affinity_gate_worker_counts"]==scalar.WORKERS
        for w in evidence["affinity_gate_worker_counts"]:
            for a in scalar.ARMS:assert quick==gate(f"gate-{a}-w{w}.jsonl","candidate" if a=="predecessor" else a)
        assert "runs.jsonl" not in items
        result["explicit_worker_gates"]=len(expected)
        result["affinity_gate_worker_counts"]=evidence["affinity_gate_worker_counts"]
    else:
        assert result["target_cpu"]
        assert items["affinity.txt"].split()[-1]=="0-19"
        assert json.loads(items["scorer-info.json"])["cpus"]==json.loads(items["predecessor-info.json"])["cpus"]==20
        rows=list(map(json.loads,items["runs.jsonl"].splitlines()))
        assert len(rows)==768
        calc=scalar.decision(rows)
        assert json.loads(json.dumps(calc))==json.loads(items["decision.json"])
        result.update(timed_samples=len(rows),decision=calc)
    return result
if __name__=="__main__":print(json.dumps(verify(Path(sys.argv[1])),indent=2))
