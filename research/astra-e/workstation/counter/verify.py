"""Validate a complete target or dry bundle against the caller's independently pinned recipe."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import tarfile
from tempfile import TemporaryDirectory

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
WIKI = "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
X4 = "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"
RELEASE = "530ed404a014a414ef0b62adfd947fa14d4533e6"
PROTOCOL = ["research/astra-e/workstation/counter", "research/astra-e/workstation/assess.py",
            "research/astra-e/workstation/refresh.py", "research/astra-e/workstation/test_protocol.py",
            "bench/bundle.sh", ".just/bench.just"]


def git(*args):
    return subprocess.check_output(["git","-C",ROOT,*args])


def digest(payload):
    return hashlib.sha256(payload).hexdigest()


def verify(path, expected):
    assert re.fullmatch(r"[0-9a-f]{40}", expected)
    with tarfile.open(path) as bundle:
        items = {}
        for member in bundle:
            assert member.isfile()
            parts = Path(member.name).parts
            assert len(parts)>=2 and ".." not in parts and not Path(member.name).is_absolute()
            name = "/".join(parts[1:])
            assert name not in items
            items[name] = bundle.extractfile(member).read()
    def text(name):
        return items[name].decode()
    def value(name):
        return json.loads(items[name])
    mode = text("mode.txt").strip()
    assert mode in ["dry","measure"] and text("registration.txt").strip()==expected
    assert text("toolchain.txt").startswith("rustc 1.99.1 ")
    assert git("ls-tree","-r",expected,"--",*PROTOCOL)==items["protocol-blobs.txt"]
    for filepath in [HERE/"panel.py",HERE.parent/"assess.py",HERE.parent/"refresh.py",HERE.parent/"test_protocol.py"]:
        assert filepath.read_bytes()==git("show",f"{expected}:{filepath.relative_to(ROOT)}")
    selected = value("scorer-refresh.json")
    projection = selected["selected_projection"]
    rev = text("scorer-revision.txt").strip()
    assert selected["proceed"] and rev==selected["selected_path_revision"]==git("log","-1","--format=%H",projection,"--","bench/moonshot").decode().strip()
    assert selected["selected_scorer_tree"]==git("rev-parse",projection+":bench/moonshot").decode().strip()==text("scorer-tree.txt").strip()
    assert selected["fetched_path_revision"]==git("log","-1","--format=%H",selected["fetched_master"],"--","bench/moonshot").decode().strip()
    attempts = selected["attempts"]
    assert attempts[-1]["failure"] is None and attempts[-1]["projection"]==projection
    assert attempts[0]["projection"]==selected["fetched_master"] and attempts[0]["build"]=="fresh"
    if selected["selected_build"]=="last-working":
        assert len(attempts)==2 and attempts[0]["failure"] and projection==selected["last_working_path_revision"]
    else:
        assert len(attempts)==1 and selected["selected_build"]=="fresh"
    # Load exactly the selected statistical implementation before importing panel/assess.
    with TemporaryDirectory() as tmp:
        projected = Path(tmp)/"moonshot.py"
        projected.write_bytes(git("show",projection+":bench/moonshot/moonshot.py"))
        spec = importlib.util.spec_from_file_location("moonshot", projected)
        module = importlib.util.module_from_spec(spec)
        sys.modules["moonshot"] = module
        spec.loader.exec_module(module)
    import panel
    assert selected["last_working_path_revision"]==panel.PIN
    assert value("roster.json")==list(map(list,panel.roster()))
    assert text("sources.txt").splitlines()==[" ".join([a,*s]) for a,s in panel.SOURCES.items()]
    assert [l.split()[0] for l in text("inputs.sha256").splitlines()]==[WIKI,X4]
    target = "i9-10900K" in text("lscpu.txt")
    def gate(name, arm):
        *cases, verdict = map(json.loads,text(name).splitlines())
        assert verdict==dict(arm=arm,verdict="PASS",failures=0)
        assert all(c["pass"] is True and c["failure"] is None for c in cases)
        assert any("wikidata" in c["case"] and c["bytes"]==panel.WIKI_BYTES for c in cases)
        return [(c["case"],None if c["case"]=="corpus text level 9" else c["bytes"]) for c in cases]
    signatures = {}
    binaries = {}
    versions = {}
    for arm,(pin,entry) in panel.SOURCES.items():
        assert items[arm+"-tree.txt"]==git("ls-tree",pin,"--","src","Cargo.toml","Cargo.lock")
        assert items[arm+"-against-v0.5.6.diff"]==git("diff",RELEASE,pin,"--","src","Cargo.toml","Cargo.lock")
        assert text(arm+"-entry.sha256").strip().split()==[digest(git("show",pin+":"+entry)),entry]
        binaries[arm] = text(arm+"-binary.sha256").split()[0]
        assert re.fullmatch("[0-9a-f]{64}",binaries[arm])
        info = value(arm+"-info.json")
        assert info["candidate"].endswith("/"+entry) and {"candidate","lazy","opend"} <= set(info["arms"])
        if mode=="measure":
            assert target and info["cpus"]==20
        assert items[arm+"-projection.txt"]==items["edge-projection.txt"]
        # Compare every index entry, including files added/removed when .just changes.
        expected_projection = git("ls-tree","-r",projection,"--","bench/moonshot",".just","Justfile").decode().splitlines()
        expected_projection = {s.split("\t")[1]:s.split("\t")[0].split()[::2] for s in expected_projection}
        actual_projection = {s.split("\t")[1]:s.split("\t")[0].split()[:2] for s in text(arm+"-projection.txt").splitlines()}
        assert expected_projection==actual_projection
        compiled = value(arm+"-compiled.json")
        assert entry in compiled and {"bench/moonshot/src/main.rs","src/bwt.rs","src/parallel.rs"} <= compiled.keys()
        # Regenerate the exact frozen include tree without building or running the decoder.
        with TemporaryDirectory() as tmp:
            tree = Path(tmp)/arm
            git("worktree","add","-q","--detach",str(tree),pin)
            try:
                subprocess.run(["bash","research/astra-e/prepare.sh",entry],cwd=tree,check=True,stdout=subprocess.DEVNULL)
                for name,h in compiled.items():
                    assert not Path(name).is_absolute() and ".." not in Path(name).parts
                    blob = git("show",projection+":"+name) if name.startswith("bench/moonshot/") else (tree/name).read_bytes()
                    if arm=="scalar" and name=="research/astra-e/front-fusion/cap-batch/worker/kernel.rs":
                        # d0a4298 writes three absolute Rust module attributes. Re-create the
                        # bytes actually hashed by rustc at the recorded checkout location.
                        recorded_root = info["candidate"].removesuffix("/"+entry).encode()
                        prefix = rb'(#\[path\s*=\s*")' + re.escape(str(tree).encode()) + rb'(?=/)'
                        blob, matches = re.subn(prefix, lambda m:m[1]+recorded_root, blob)
                        assert matches==3
                    assert digest(blob)==h, (arm,name)
            finally:
                git("worktree","remove","--force",str(tree))
        report = [l for l in text("report-"+arm+".txt").splitlines() if l.startswith("scorer v")]
        assert len(report)==1
        versions[arm] = report[0]
        signatures[arm] = gate("gate-"+arm+"/check.jsonl","candidate")
        summary = value("gate-"+arm+"/summary.json")
        assert summary["gate"]==dict(verdict="PASS",cases=len(signatures[arm]),failed=[],exit=0)
        assert summary["options"]["no_proxy"] and summary["options"]["no_timing"] and not summary["options"]["quick"]
        assert versions[arm]=="scorer "+summary["scorer"]["version"]+" at "+summary["scorer"]["commit"]
    assert len(set(versions.values()))==1 and len(set(binaries.values()))==3
    assert len(signatures["edge"])>=338 and signatures["edge"]==signatures["scalar"]==signatures["both"]
    quick = gate("gate-lazy.jsonl","lazy")
    assert len(quick)>=98 and quick==gate("gate-opend.jsonl","opend")
    dry = value("dry-coverage.json")
    expected_rows = [dict(label=a,build=panel.identity(a)[0],compiled_arm=panel.identity(a)[1],
                         workers=w,slots=4*w,out_bytes=panel.WIKI_BYTES) for w in panel.WORKERS for a in panel.ARMS]
    assert dry["explicit_workers"]==expected_rows and dry["target_machine"]==target
    if target:
        assert dry["affinity_gate_worker_counts"]==panel.WORKERS
    for w in dry["affinity_gate_worker_counts"]:
        assert w in panel.WORKERS
        for a in panel.ARMS:
            assert quick==gate(f"gate-{a}-w{w}.jsonl",panel.identity(a)[1])
    result = dict(registration=expected,mode=mode,bundle_sha256=digest(path.read_bytes()),binary_sha256=binaries,
                  target_cpu=target,scorer_path=rev,selected_build=selected["selected_build"],reports=versions,
                  full_gate_each=len(signatures["edge"]),quick_gate_each=len(quick),explicit_dry_rows=len(expected_rows),
                  affinity_gate_worker_counts=dry["affinity_gate_worker_counts"],planned_samples=len(panel.roster()))
    if mode=="measure":
        assert target and text("affinity.txt").strip().split()[-1]=="0-19"
        rows = list(map(json.loads,text("runs.jsonl").splitlines()))
        calc = panel.decision(rows)
        assert json.loads(json.dumps(calc,allow_nan=False))==value("decision.json")
        result.update(timed_samples=len(rows),decision=calc)
    else:
        assert "runs.jsonl" not in items and "decision.json" not in items
    return result


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle",type=Path)
    parser.add_argument("registration",help="independently recorded 40-character registration commit")
    options=parser.parse_args()
    print(json.dumps(verify(options.bundle,options.registration),indent=2,allow_nan=False))
