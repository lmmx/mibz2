"""Run the unchanged Rust capped harness against three supplied smooth indices."""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import subprocess as sp
import tempfile

HERE = Path(__file__).resolve().parent
REPO = Path(sp.check_output(["git","-C",HERE,"rev-parse","--show-toplevel"],text=True).strip())
DONOR = HERE.parents[6] / "powers/xor/capped"
CORPUS = REPO / "corpora/wikidata.bz2"
OLD = ["stream","streamgaps","dense","scalar","lazy","opend"]
NEW = ["k4","k8","k16",*OLD[3:]]

def sha(path):
    with path.open("rb") as f:
        return hashlib.file_digest(f,"sha256").hexdigest()

def dump(path, val):
    path.write_text(json.dumps(val,indent=2)+"\n")

def read(path):
    with path.open() as f:
        return list(csv.DictReader(f))

def write(path, rows):
    with path.open("w") as f:
        writer=csv.DictWriter(f,list(dict.fromkeys(k for q in rows for k in q)),lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)

def build(d, manifest, rss):
    flags=["-std=c++20","-O3","-g0","-Wall","-Wextra","-fPIC","-shared"]
    sp.run(["g++",*flags,HERE/"bridge.cpp","-o",d/"libgrammar.so"],check=True)
    dependencies=sp.check_output(["g++","-std=c++20","-MM",HERE/"bridge.cpp"],text=True)
    sources=[Path(x) for x in dependencies.replace("\\\n","").split()[1:]]
    dump(d/"build.json",dict(bridge_sources={str(p.resolve().relative_to(REPO)):sha(p) for p in sources},
         so=sha(d/"libgrammar.so"),rss=sha(rss),rust=manifest,flags=flags,
         gxx=sp.check_output(["g++","--version"],text=True).splitlines()[0],
         cpu=next(s.split(":",1)[1].strip() for s in Path("/proc/cpuinfo").read_text().splitlines() if s.startswith("model name"))))

def panel(ref, dest, inputs, env, reps):
    stamps=dest/"stamps.csv"
    sp.run([ref/"panel",CORPUS,dest,str(reps)],env=dict(env,SMOOTH_STAMPS=str(stamps)),check=True)
    for q in inputs:
        b=q["block"]
        assert sha(dest/f"block-{b}.raw")==q["raw_sha256"] and sha(dest/f"block-{b}.caps")==q["caps_le_sha256"]
    keys={(q["rows"],q["mark"],q["records"]):str(q["block"]) for q in inputs}
    assert len(keys)==len(inputs)==10
    calls=read(stamps)
    assert len(calls)==30*(1+reps)
    verified=set()
    for i,c in enumerate(calls):
        c["block"]=keys[tuple(int(c[k]) for k in ("n","primary","records"))]
        a=int(c["arm"])
        assert int(c["depth"])==[4,8,16][a] and (int(c["supply_ns"])>0)==(i<30)
        if i<30: verified.add((a,c["block"]))
    assert len(verified)==30
    if not reps:
        dump(dest/"gate.json",dict(output_checks=60,inputs=inputs,so=sha(dest/"libgrammar.so")))
        return
    times=read(dest/"times.csv")
    assert len(times)==24*60
    it=iter(calls[30:])
    for t in times:
        a=OLD.index(t["arm"]);t["arm"]=NEW[a]
        if a>=3: continue
        c=next(it)
        assert t["block"]==c["block"] and int(c["arm"])==a and int(c["emit_ns"])<=int(t["thread_ns"])
        t.update({k:c[k] for k in ("depth","repair_steps","supply_ns","emit_ns","index_bytes","output_bytes")})
    assert not list(it)
    write(dest/"paired.csv",times)
    dump(dest/"runtime.json", dict(affinity=sorted(os.sched_getaffinity(0)),nice=os.getpriority(os.PRIO_PROCESS,0),
                                  quota=Path("/sys/fs/cgroup/cpu.max").read_text().strip(),so=sha(dest/"libgrammar.so")))

def rss(ref,d,inputs,env):
    rows=[]
    with tempfile.TemporaryDirectory(dir=d) as tmp:
        src=Path(tmp)
        (src/"sample.bz2").symlink_to(CORPUS)
        sp.run(["bzip2recover","sample.bz2"],cwd=src,check=True,stdout=sp.DEVNULL,stderr=sp.DEVNULL)
        for q in inputs: assert sha(ref/f'block-{q["block"]}.raw')==q["raw_sha256"]
        for r in range(3):
            for q in inputs:
                b=q["block"]
                for i in range(6):
                    a=(r+b+i)%6
                    cmd=[ref/"rss",src/f"rec{b+1:05d}sample.bz2",OLD[a],ref/f"block-{b}.raw"]
                    x=list(map(int,sp.check_output(cmd,env=env,text=True).strip().split(",")))
                    assert x[:3]==[q["mark"],q["rows"],q["records"]]
                    rows.append(dict(round=r,block=b,arm=NEW[a],primary=x[0],rows=x[1],records=x[2],
                                     pre_hwm_kib=x[3],post_hwm_kib=x[4]))
    write(d/"rss.csv",rows)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("donor",type=Path)
    p.add_argument("base",type=Path)
    p.add_argument("action",choices=["build","gate","price","rss"])
    args=p.parse_args()
    ref,d=args.donor.resolve(),args.base.resolve()
    d.mkdir(parents=True,exist_ok=True)
    receipt=json.loads((DONOR/"receipt.json").read_text())
    frozen=json.loads((DONOR/"build.json").read_text())
    local=json.loads((ref/"build.json").read_text())
    for k in ("rustc","donor","source_sha256","src_tree","rlib_sha256","rust_flags","exe_sha256"):assert frozen[k]==local[k]
    assert sha(ref/"panel")==frozen["exe_sha256"] and sha(CORPUS)==receipt["corpus_sha256"]
    assert sha(ref/"rss")=="20ec2f55e18dd6893e2cc466a30e584e70e8dab647d7ac7d9ae45ada4e45c289"
    if args.action=="build":
        return build(d,local,ref/"rss")
    env=dict(os.environ,LD_LIBRARY_PATH=str(d))
    env.pop("SMOOTH_STAMPS",None)
    for exe in ("panel","rss"):
        assert f"libgrammar.so => {d}/libgrammar.so" in sp.check_output(["ldd",ref/exe],text=True,env=env)
    if args.action!="gate":assert json.loads((d/"gate.json").read_text())["so"]==sha(d/"libgrammar.so")
    if args.action=="rss":return rss(ref,d,receipt["inputs"],env)
    return panel(ref,d,receipt["inputs"],env,24 if args.action=="price" else 0)

if __name__=="__main__":
    main()
