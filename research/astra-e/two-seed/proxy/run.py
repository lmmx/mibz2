"""Replay the registered six matched chaingrind pairs with the two archived gate binaries."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import lzma
import os
from pathlib import Path
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
OLD = ROOT/"research/astra-e/results/scalar-route-pricing"
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def git(path, *args):
    return subprocess.check_output(["git", "-C", path, *args], text=True).strip()
def now():
    return datetime.now(timezone.utc).isoformat()

def main():
    scorer, build, vlib, output = map(lambda x:Path(x).resolve(), sys.argv[1:])
    output.mkdir(parents=True, exist_ok=True)
    if (output/"runs.jsonl").exists():
        raise FileExistsError("use an empty destination to preserve existing observations")
    assert os.sched_getaffinity(0)=={4} and os.nice(0)==15
    sys.path.insert(0,str(scorer/"bench/moonshot"))
    import moonshot
    ref=json.loads((OLD/"manifest.json").read_text())
    path=git(scorer,"log","-1","--format=%H","origin/master","--","bench/moonshot")
    assert path==ref["scorer_path"]
    meta=dict(start=now(),plan=git(ROOT,"log","-1","--format=%H","--",str(HERE/"PLAN.md")),fetched_master=git(scorer,"rev-parse","origin/master"),
              scorer_path=path,scorer_version=moonshot.VERSION+" at "+moonshot.scorer_commit(),files=ref["files"],tools=ref["tools"],
              samples=ref["samples"],corpus_sha256=ref["corpus_sha256"],cpu=4,nice=15,arms={})
    for name,h in ref["files"].items(): assert sha(scorer/name)==h
    prefix=Path(shutil.which("valgrind"))
    for tool in [prefix,Path(str(prefix)+".bin"),vlib/"chaingrind-amd64-linux",vlib/"vgpreload_core-amd64-linux.so"]:
        assert sha(tool)==meta["tools"][tool.name]
    decoders={}
    for arm in ("control","candidate"):
        target=build/(arm+"-target")
        binary=target/"release/moonshot"
        gate=build/"official"/arm
        summary=json.loads((gate/"summary.json").read_text())
        entry=Path(summary["info"]["candidate"])
        info=json.loads(subprocess.check_output([binary,"info"]))
        assert info["candidate"]==str(entry) and summary["gate"]==dict(verdict="PASS",cases=338,failed=[],exit=0)
        assert meta["scorer_version"]==summary["scorer"]["version"]+" at "+summary["scorer"]["commit"]
        meta["arms"][arm]=dict(executable=str(binary),binary_sha256=sha(binary),entry=str(entry),entry_sha256=sha(entry),
                               source_checkout=git(entry.parent,"rev-parse","HEAD"),gate_sha256=sha(gate/"check.jsonl"))
        decoders[arm]=moonshot.Scorer(argparse.Namespace(target_dir=str(target)))
    def persist():
        (output/"manifest.json").write_text(json.dumps(meta,indent=2)+"\n")
    persist()
    with TemporaryDirectory() as tmp,(output/"runs.jsonl").open("x") as out:
        for i,b in enumerate([1,85,165]):
            tape=Path(tmp)/f"{b}.sym"
            tape.write_bytes(lzma.decompress((OLD/"symbols"/f"warm1-measure{b}.sym.xz").read_bytes()))
            assert sha(tape)==ref["samples"][str(b)]["two_record_sha256"]
            for j,w in enumerate([7,10]):
                for arm in (["control","candidate"] if (i+j)%2==0 else ["candidate","control"]):
                    start=now()
                    s=decoders[arm]
                    lat=moonshot.nominal_latencies(s.cal["model"],w)
                    raw=s.chaingrind(vlib,"candidate",tape,lat)
                    assert raw["out_bytes"]=={1:900000,85:899988,165:900000}[b]
                    out.write(json.dumps(dict(workers=w,block=b,arm=arm,start=start,finish=now(),latencies=lat,counters=raw))+"\n")
                    out.flush()
                    print(w,b,arm,flush=True)
    meta["finish"]=now()
    persist()

if __name__=="__main__":main()
