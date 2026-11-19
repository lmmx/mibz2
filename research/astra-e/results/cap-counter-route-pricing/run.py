"""Run the preregistered six diagnostic cells with exact archived controls and two freshly built counter arms."""
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
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
PREVIOUS=HERE.parent/"scalar-route-pricing"
PLAN="d05577e22526c5880fc6f683f4189b5b48377dc5"
SOURCE="50b8ecb8946a498601e3fd894ed84a2687df6cc8"
BLOCKS=(1,85,165)
SIZE={1:900000,85:899988,165:900000}
ARMS=("edge","counter","scalar","both")
FACTOR="research/astra-e/front-fusion/cap-counter/"

def utc():return datetime.now(timezone.utc).isoformat()
def sha(b):return hashlib.sha256(b).hexdigest()
def digest(p):return sha(p.read_bytes())
def git(w,*args):return subprocess.check_output(["git","-C",w,*args],text=True).strip()
def dump(path,data):path.write_text(json.dumps(data,indent=2)+"\n")

def main():
    p=argparse.ArgumentParser()
    for key in ("repo","scratch","vlib","corpus","output"):p.add_argument(key,type=lambda x:Path(x).resolve())
    p.add_argument("--historical-master",help="Use this immutable imported master only for replay")
    p.add_argument("--archived-new", action="store_true", help="Verify the exact stored counter executables")
    p.add_argument("--check-only", action="store_true", help="Verify toolchain, all binaries and all extracted inputs without collecting counters")
    args=p.parse_args()
    assert os.sched_getaffinity(0)=={4} and os.nice(0)==15
    repo=args.repo
    old=json.loads((PREVIOUS/"manifest.json").read_text())
    sys.path.insert(0,str(repo/"bench/moonshot"));import moonshot
    a=args.scratch
    wt=a/"source"
    assert git(wt,"rev-parse","HEAD")==SOURCE
    master=git(repo,"rev-parse",args.historical_master or "origin/master")
    revision=git(repo,"log","-1","--format=%H",master,"--","bench/moonshot")
    assert revision==old["scorer_path"]
    scorer=dict(version=moonshot.VERSION,commit=moonshot.scorer_commit())
    # Render the actual locally registered factorial score with the refreshed
    # scorer's version line. This diagnostic does not manufacture a gate result.
    prior_score=json.loads((HERE.parent/"v1p2-cap-counter-confirm/score/summary.json").read_text())
    assert prior_score["scorer"]==scorer
    header=moonshot.report(prior_score)
    version=next(line for line in header.splitlines() if line.startswith("scorer "))
    print(version,flush=True)
    assert version=="scorer "+old["scorer_version"]
    rust=subprocess.check_output(["rustc","-Vv"],text=True).strip()
    assert rust==old["rustc"]
    assert digest(args.corpus)==old["corpus_sha256"]
    tool=Path(shutil.which("valgrind"))
    hashes={f.name:digest(f) for f in (tool,Path(str(tool)+".bin"),args.vlib/"chaingrind-amd64-linux",
                                      args.vlib/"vgpreload_core-amd64-linux.so")}
    assert hashes==old["tools"]
    args.output.mkdir(exist_ok=True,parents=True)
    assert not (args.output/"runs.jsonl").exists()
    symbols=a/"symbols"
    symbols.mkdir(exist_ok=True)
    versions={f:digest(repo/f) for f in old["files"]}
    assert versions==old["files"]
    for path,expected in versions.items():
        assert digest(wt/path)==expected
        assert sha(subprocess.check_output(["git","-C",repo,"show",master+":"+path]))==expected
    meta=dict(start=utc(),plan=PLAN,plan_committed=git(repo,"show","-s","--format=%cI",PLAN),
              scorer_checkout=git(repo,"rev-parse","HEAD"),fetched_master=master,scorer_path=revision,
              scorer_version=version.removeprefix("scorer "),files=versions,tools=hashes,
              valgrind=subprocess.check_output([tool,"--version"],text=True,env=dict(os.environ,VALGRIND_LIB=str(args.vlib))).strip(),
              rustc=rust,corpus_sha256=old["corpus_sha256"],samples=old["samples"],cpu=4,nice=15,
              archive_source="research/astra-e/results/scalar-route-pricing",arms={},
              eligibility_scope="offline diagnostics; known local timing; independent factorial CI/i9 outcomes not consulted")
    runners={}
    for arm in ARMS:
        if arm in ("edge","scalar"):
            binary=a/"archive"/arm/"moonshot"; inf=old["arms"][arm].copy()
            assert digest(binary)==inf["binary_sha256"]
            assert json.loads(subprocess.check_output([binary,"info"]))==inf["info"]
            inf["origin"]="exact previous workers.tar.gz member"
        else:
            entry=FACTOR+("counter_edge.rs" if arm=="counter" else "both.rs")
            binary=a/(arm+"-build")/"release/moonshot"
            info=json.loads(subprocess.check_output([binary,"info"]))
            if args.archived_new:
                historical=json.loads((HERE/"manifest.json").read_text())["arms"][arm]
                assert historical["pin"]==SOURCE and historical["entry"]==entry
                assert info==historical["info"] and digest(binary)==historical["binary_sha256"]
            else:
                assert info["candidate"]==str(wt/entry)
            deps=[entry,FACTOR+"adapter.inc.rs",FACTOR+"kernel.rs",FACTOR+"narrow.rs",
                  "research/astra-e/edge-budget/gate.rs","research/astra-e/short-words/endpoint.rs",
                  "docs/journal/2026-11-18-open-return/segments.rs"]
            assert not git(wt,"diff","--name-only",SOURCE,"--","src","Cargo.toml","Cargo.lock",*deps)
            inf=dict(origin="fresh release build",pin=SOURCE,entry=entry,binary_sha256=digest(binary),info=info,
                     src_tree=git(wt,"rev-parse",SOURCE+":src"),research_source_sha256={f:digest(wt/f) for f in deps})
            if args.archived_new:
                inf["origin"]="exact counter-workers.tar.xz member"
        meta["arms"][arm]=inf
        runner=moonshot.Scorer(argparse.Namespace(target_dir=str(a/(arm+"-build"))))
        runner.bin=binary
        runners[arm]=runner
        # Re-extract each individual stream record, including its original checksum.
        singles={}
        for block in BLOCKS:
            dest=symbols/f"{arm}-{block}.single"
            result=json.loads(subprocess.check_output([binary,"extract","--file",args.corpus,"--blocks",str(block),"--out",dest]))
            assert result=={"blocks":1}
            singles[block]=dest.read_bytes()
            assert sha(singles[block])==old["samples"][str(block)]["measured_single_sha256"]
        for block in BLOCKS:
            raw=singles[1]+singles[block]
            assert raw==lzma.decompress((PREVIOUS/"symbols"/f"warm1-measure{block}.sym.xz").read_bytes())
            assert sha(raw)==old["samples"][str(block)]["two_record_sha256"]
            (symbols/f"{arm}-{block}.sym").write_bytes(raw)
    if args.check_only:
        print("All four executables, three extracted inputs per arm and the runtime verified")
        return
    # Keep actual official argv and outcome including under_chaingrind, CRC-checked
    # output size and the printed counter record without reimplementing its CLI.
    execute=subprocess.run
    dump(args.output/"manifest.json",meta)
    with (args.output/"runs.jsonl").open("x") as raw:
        for w in (7,10):
            for j,b in enumerate(BLOCKS):
                pos=(j+(w==10)*3)%4
                order=ARMS[pos:]+ARMS[:pos]
                for arm in order:
                    captured={}
                    def observe(cmd,**kwargs):
                        r=execute(cmd,**kwargs)
                        captured["argv"]=[str(v) for v in cmd]
                        captured["proxy"]=json.loads(r.stdout) if r.returncode==0 else r.stdout
                        if r.returncode:
                            print(r.stderr,file=sys.stderr)
                        return r
                    start=utc()
                    s=runners[arm];lat=moonshot.nominal_latencies(s.cal["model"],w)
                    with patch.object(moonshot.subprocess,"run",observe):
                        counters=s.chaingrind(args.vlib,"candidate",symbols/f"{arm}-{b}.sym",lat)
                    proxy=captured["proxy"]
                    assert proxy=={"arm":"candidate","blocks":2,"out_bytes":SIZE[b],"under_chaingrind":True}
                    assert counters["out_bytes"]==SIZE[b]
                    row=dict(block=b,workers=w,arm=arm,start=start,finish=utc(),latencies=lat,counters=counters,**captured)
                    raw.write(json.dumps(row)+"\n");raw.flush()
                    print(w,b,arm,counters["instructions"],flush=True)
    meta["finish"]=utc()
    dump(args.output/"manifest.json",meta)
if __name__=="__main__":main()
