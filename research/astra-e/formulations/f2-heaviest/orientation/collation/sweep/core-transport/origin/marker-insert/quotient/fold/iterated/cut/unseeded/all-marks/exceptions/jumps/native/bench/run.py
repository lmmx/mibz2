"""Rebuild the C++ bridge and price it in the unmodified capped Rust donor."""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import subprocess as sp
import tempfile

HERE = Path(__file__).resolve().parent
REPO = Path(sp.check_output(["git", "-C", HERE, "rev-parse", "--show-toplevel"], text=True).strip())
DONOR = HERE.parents[5] / "powers/xor/capped"
CORPUS = REPO / "corpora/wikidata.bz2"
OLD = ["stream", "streamgaps", "dense", "scalar", "lazy", "opend"]
NEW = ["prepared", "geometry", "complete", *OLD[3:]]

def sha(path):
    return hashlib.file_digest(path.open("rb"), "sha256").hexdigest()

def dump(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")

def read_csv(path):
    with path.open() as f:
        return list(csv.DictReader(f))

def write_csv(path, rows):
    keys = dict.fromkeys(k for row in rows for k in row)
    with path.open("w") as f:
        wr = csv.DictWriter(f, fieldnames=keys)
        wr.writeheader()
        wr.writerows(rows)

def build(base, ref):
    sp.run(["python3", HERE.parent / "generate.py", base], cwd=REPO, check=True)
    flags = ["-std=c++20", "-O3", "-g0", "-mpopcnt", "-mbmi2", "-Wall", "-Wextra", "-fPIC", "-shared"]
    sp.run(["g++", *flags, "-I", base, HERE / "bridge.cpp", "-o", base / "libgrammar.so"], check=True)
    sources = [HERE / "bridge.cpp", HERE.parent / "generate.py", HERE.parent / "jump.hpp", HERE.parent / "model.hpp",
               HERE.parents[2] / "contexts/contexts.hpp"]
    dump(base / "build.json", dict(flags=flags, rust=json.loads((ref / "build.json").read_text()),
             gxx=sp.check_output(["g++", "--version"], text=True).splitlines()[0],
             sources={str(p.relative_to(REPO)):sha(p) for p in sources}, selector=sha(base / "selector.inc"),
             shared_object=sha(base / "libgrammar.so"), rss=sha(ref / "rss"),
             cpu=next(s.split(":",1)[1].strip() for s in Path("/proc/cpuinfo").read_text().splitlines() if s.startswith("model name"))))

def measurements(ref, dest, rows, env, action):
    warm = action == "gate"
    env["JUMP_STAMPS"] = str(dest / "stamps.csv")
    sp.run([ref / "panel", CORPUS, dest, "0" if warm else "24"], env=env, check=True)
    for q in rows:
        for ext, name in (("caps", "caps_le_sha256"), ("raw", "raw_sha256")):
            assert sha(dest / f"block-{q['block']}.{ext}") == q[name]
    stamps = read_csv(dest / "stamps.csv")
    by_key = {(q["rows"], q["mark"], q["records"]):q["block"] for q in rows}
    assert len(by_key) == 10
    for s in stamps:
        key = tuple(int(s[k]) for k in ("n", "primary", "records"))
        s["block"] = str(by_key[key])
    expected = {(str(a), str(q["block"])) for a in range(3) for q in rows}
    assert expected == {(s["arm"], s["block"]) for s in stamps[:30]}
    assert all((int(s["supply_ns"]) > 0) == (s["arm"] != "2") for s in stamps[:30])
    if warm:
        assert len(stamps) == 30
        dump(dest / "gate.json", dict(exact_outputs=60, block_inputs=rows, so=sha(dest.parent / "libgrammar.so"),
             mode=env["JUMP_MODE"], cpu_affinity=sorted(os.sched_getaffinity(0))))
        return
    assert len(stamps) == 750
    it = iter(stamps[30:])
    obs = read_csv(dest / "times.csv")
    assert len(obs) == 1440
    for q in obs:
        arm = OLD.index(q["arm"])
        q["arm"] = NEW[arm]
        if arm >= 3:
            continue
        s = next(it)
        assert (s["arm"], s["block"]) == (str(arm), q["block"]) and int(s["supply_ns"]) == 0
        clocks = [int(s[k]) for k in ("caps_ns", "geometry_ns", "index_ns", "emit_ns")]
        assert sum(clocks) <= int(q["thread_ns"])
        assert tuple(v > 0 for v in clocks) == ((arm==2), (arm==2), (arm>0), True)
        q.update({k:v for k,v in s.items() if k not in {"arm","block","n","primary","records"}})
    assert list(it) == []
    write_csv(dest / "paired.csv", obs)
    dump(dest / "runtime.json", dict(cpu_affinity=sorted(os.sched_getaffinity(0)),
             nice=os.getpriority(os.PRIO_PROCESS,0), mode=env["JUMP_MODE"], so=sha(dest.parent / "libgrammar.so")))

def rss(ref, dest, rows, env):
    result = []
    with tempfile.TemporaryDirectory(dir=dest.parent) as temp:
        tmp = Path(temp)
        (tmp / "sample.bz2").symlink_to(CORPUS)
        sp.run(["bzip2recover", "sample.bz2"], cwd=tmp, stdout=sp.DEVNULL, stderr=sp.DEVNULL, check=True)
        for r in range(3):
            for q in rows:
                b = q["block"]
                for i in range(6):
                    j = (r+i+b) % 6
                    cmd = [ref / "rss", tmp / f"rec{b+1:05d}sample.bz2", OLD[j], ref / f"block-{b}.raw"]
                    p,n,c,pre,post = map(int,sp.check_output(cmd,text=True,env=env).strip().split(","))
                    assert (p,n,c) == (q["mark"], q["rows"], q["records"])
                    result.append(dict(round=r,block=b,arm=NEW[j],primary=p,rows=n,records=c,pre_hwm_kib=pre,post_hwm_kib=post))
    write_csv(dest / "rss.csv", result)

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("donor", type=Path)
    p.add_argument("base", type=Path)
    p.add_argument("action", choices=("build","gate","price","rss"))
    p.add_argument("mode", nargs="?", choices=("fixed","variable"))
    args = p.parse_args()
    ref, base = args.donor.resolve(), args.base.resolve()
    base.mkdir(parents=True, exist_ok=True)
    frozen = json.loads((DONOR / "build.json").read_text())
    local = json.loads((ref / "build.json").read_text())
    receipt = json.loads((DONOR / "receipt.json").read_text())
    for key in ("rustc","donor","src_tree","source_sha256","rlib_sha256","rust_flags","exe_sha256"):
        assert local[key] == frozen[key]
    assert sha(ref / "panel") == frozen["exe_sha256"] and sha(CORPUS) == receipt["corpus_sha256"]
    assert sha(ref / "rss") == "20ec2f55e18dd6893e2cc466a30e584e70e8dab647d7ac7d9ae45ada4e45c289"
    if args.action == "build":
        return build(base, ref)
    assert args.mode is not None
    dest = base / args.mode
    dest.mkdir(exist_ok=True)
    env = dict(os.environ, JUMP_MODE=args.mode, LD_LIBRARY_PATH=str(base))
    env.pop("JUMP_STAMPS", None)
    for exe in (ref / "panel", ref / "rss"):
        assert f"libgrammar.so => {base}/libgrammar.so" in sp.check_output(["ldd", exe], env=env, text=True)
    if args.action != "gate":
        gate = json.loads((dest / "gate.json").read_text())
        assert gate["so"] == sha(base / "libgrammar.so")
    if args.action == "rss":
        for row in receipt["inputs"]:
            assert sha(ref / f'block-{row["block"]}.raw') == row["raw_sha256"]
        return rss(ref, dest, receipt["inputs"], env)
    return measurements(ref, dest, receipt["inputs"], env, args.action)

if __name__ == "__main__":
    main()
