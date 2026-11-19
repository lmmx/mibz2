"""Twelve predeclared per-arm, per-block nominal scorer counter measurements."""
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

ARMS={"edge":("badb2aec3624bd65606c429b2e6f79e9d9d0050f","research/astra-e/edge-budget/candidate.rs"),
      "scalar":("d0a42980e896211906234c7679b38760ae44a46e","research/astra-e/front-fusion/cap-batch/worker/candidate.rs")}
SCORER="75297a1d57b2bdf262c3a19f3064848dedf2c493"
INPUT="82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
PLAN="bac52c22318ebeecdb198bb2b9a45b13686fef55"
BLOCKS=(1,85,165)


def utc():return datetime.now(timezone.utc).isoformat()
def sha(data):return hashlib.sha256(data).hexdigest()
def digest(path):return sha(path.read_bytes())
def git(root,*args):return subprocess.check_output(["git","-C",root,*args],text=True).strip()
def dump(path,record):path.write_text(json.dumps(record,indent=2)+"\n")


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for x in ["scorer","build","vlib","corpus","output"]:p.add_argument(x,type=lambda f:Path(f).resolve())
    a=p.parse_args()
    assert os.sched_getaffinity(0)=={4} and os.nice(0)==15
    sys.path.insert(0,str(a.scorer/"bench/moonshot"));import moonshot
    assert digest(a.corpus)==INPUT
    a.output.mkdir(parents=True,exist_ok=True)
    (a.output/"symbols").mkdir(exist_ok=True)
    local=a.build/"sym";local.mkdir(exist_ok=True)
    libs={}
    meta=dict(start=utc(),plan=PLAN,plan_committed=git(a.scorer,"show","-s","--format=%cI",PLAN),
              plan_public_ref_verified_before_samples=True,target_result_status_at_plan="queued as relayed; no scalar target receipt consulted",
              scorer_checkout=git(a.scorer,"rev-parse","HEAD"),fetched_master="3df195b10e9fb6f86f0c06ad9fad1feb992406d5",
              scorer_path=SCORER,scorer_version=moonshot.VERSION+" at "+moonshot.scorer_commit(),
              corpus_sha256=INPUT,files={},tools={},arms={},samples={},cpu=4,nice=15)
    for path in ["bench/moonshot/moonshot.py","bench/moonshot/chaingrind/chaingrind.c","bench/moonshot/chaingrind/build.sh","bench/moonshot/calibration.json","bench/moonshot/src/main.rs","bench/moonshot/src/check.rs"]:
        meta["files"][path]=digest(a.scorer/path)
        assert sha(subprocess.check_output(["git","-C",a.scorer,"show",SCORER+":"+path]))==meta["files"][path]
    prefix=Path(shutil.which("valgrind")); tool=a.vlib/"chaingrind-amd64-linux"
    for path in [prefix,Path(str(prefix)+".bin"),tool,a.vlib/"vgpreload_core-amd64-linux.so"]:
        meta["tools"][path.name]=digest(path)
    meta["valgrind"]=subprocess.check_output([prefix,"--version"],text=True,env=dict(os.environ,VALGRIND_LIB=str(a.vlib))).strip()
    meta["rustc"]=subprocess.check_output(["rustc","-Vv"],text=True).strip()
    for arm,(pin,entry) in ARMS.items():
        wt=a.build/arm; binary=a.build/(arm+"-build")/"release/moonshot"
        assert git(wt,"rev-parse","HEAD")==pin
        assert not git(wt,"diff","--name-only",pin,"--","src","Cargo.toml","Cargo.lock",entry)
        info=json.loads(subprocess.check_output([binary,"info"]))
        assert info["candidate"]==str(wt/entry)
        assert digest(wt/entry)==sha(subprocess.check_output(["git","-C",wt,"show",pin+":"+entry]))
        for path,h in meta["files"].items():assert digest(wt/path)==h
        deps=list((wt/entry).parent.glob("*.rs"))
        deps+=list((wt/"research/astra-e/narrow-short").glob("constructor_ahead.rs"))
        deps+=list((wt/"research/astra-e/short-words").glob("endpoint.rs"))
        deps+=list((wt/"docs/journal/2026-11-18-open-return").glob("segments.rs"))
        meta["arms"][arm]=dict(pin=pin,entry=entry,binary_sha256=digest(binary),info=info,
                src_tree=git(wt,"rev-parse",pin+":src"),research_source_sha256={str(z.relative_to(wt)):digest(z) for z in sorted(deps)})
        libs[arm]=moonshot.Scorer(argparse.Namespace(target_dir=binary.parents[1]))
        assert libs[arm].bin==binary
        for b in BLOCKS:
            target=local/f"{arm}-{b}.single"
            extracted=json.loads(subprocess.check_output([binary,"extract","--file",a.corpus,"--blocks",str(b),"--out",target]))
            assert extracted=={"blocks":1}
        for b in BLOCKS:
            # Every record is self-delimiting. Separate extraction prevents
            # want.contains(index) deduplication of warm=1 and measured=1.
            raw=(local/f"{arm}-1.single").read_bytes()+(local/f"{arm}-{b}.single").read_bytes()
            tape=local/f"{arm}-{b}.sym";tape.write_bytes(raw)
            if arm=="edge":
                (a.output/"symbols"/f"warm1-measure{b}.sym.xz").write_bytes(lzma.compress(raw,preset=6))
                meta["samples"][b]=dict(measured_single_sha256=digest(local/f"{arm}-{b}.single"),two_record_sha256=sha(raw),bytes=len(raw))
            else:
                assert raw==(local/f"edge-{b}.sym").read_bytes()
                assert meta["samples"][b]["measured_single_sha256"]==digest(local/f"{arm}-{b}.single")
    dump(a.output/"manifest.json",meta)
    with (a.output/"runs.jsonl").open("x") as out:
        for w in (7,10):
            for i,b in enumerate(BLOCKS):
                legs=["edge","scalar"] if (w+i)%2 else ["scalar","edge"]
                for arm in legs:
                    start=utc()
                    s=libs[arm];lat=moonshot.nominal_latencies(s.cal["model"],w)
                    counters=s.chaingrind(a.vlib,"candidate",local/f"{arm}-{b}.sym",lat)
                    assert counters["out_bytes"]=={1:900000,85:899988,165:900000}[b]
                    r=dict(block=b,workers=w,arm=arm,start=start,finish=utc(),latencies=lat,counters=counters)
                    out.write(json.dumps(r)+"\n");out.flush()
                    print(w,b,arm,counters["cycles"],flush=True)
    meta["finish"]=utc();dump(a.output/"manifest.json",meta)


if __name__=="__main__":main()
