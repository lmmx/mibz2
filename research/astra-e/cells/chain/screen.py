"""Run the official W=7 model on frozen and on two verified width regimes."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys

def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ("tree", "corpus", "candidate", "runtime", "scratch", "out"):
        parser.add_argument(key, type=Path)
    a = parser.parse_args()
    tree, scratch = a.tree.resolve(), a.scratch.resolve()
    sys.path.insert(0, str(tree / "bench/moonshot"))
    import moonshot

    def git(*args):
        return subprocess.check_output(["git", "-C", tree, *args], text=True).strip()

    # Refresh in the dedicated research worktree at the start of each comparison.
    git("fetch", "origin", "master")
    git("checkout", "origin/master", "--", "bench/moonshot", ".just", "Justfile")
    scorer = moonshot.Scorer(argparse.Namespace(target_dir=scratch))
    scorer.build(a.candidate.resolve())
    info = scorer.info()
    latencies = moonshot.latencies(scorer.cal, 7)
    audit_path = a.candidate.parent / "geometry.csv"
    with audit_path.open() as f:
        inputs = {int(row["stream_index"]): row for row in csv.DictReader(f)}
    assert [int(inputs[i]["letters"]) for i in (1,85,269)] == [51184,62533,124940]
    assert all(inputs[i]["own"] == "1" for i in (1,85,269))
    manifest = {
        "candidate":info, "candidate_sha256":digest(a.candidate),"binary_sha256":digest(scorer.bin),
        "source_commit":git("rev-parse","HEAD"), "master":git("rev-parse","origin/master"),
        "scorer_path_revision":git("log","-1","--format=%H","origin/master","--","bench/moonshot"),
        "printed_scorer_version":info.get("scorer_version", info.get("version")),
        "calibration_sha256":digest(tree / "bench/moonshot/calibration.json"),
        "chaingrind_sha256":digest(a.runtime / "chaingrind-amd64-linux"),
        "valgrind":subprocess.check_output(["valgrind","--version"],text=True).strip(),
        "rustc":subprocess.check_output(["rustc","-Vv"],text=True).strip(),
        "workers":7,"latencies":latencies,"corpus_sha256":digest(a.corpus),
        "geometry_sha256":digest(audit_path), "identities":[inputs[i] for i in (1,85,269)],
    }
    a.out.mkdir(parents=True,exist_ok=False)
    (a.out / "manifest.json").write_text(json.dumps(manifest,indent=2)+"\n")
    for name, group in [("frozen",[1]), ("small-large-slots",[85,269])]:
        symbols = scratch / (name + ".sym")
        meta = subprocess.check_output([scorer.bin,"extract","--file",a.corpus,"--blocks",
            ",".join(map(str,group)),"--out",symbols],text=True)
        assert json.loads(meta)["blocks"] == len(group)
        for arm in ("lazy","cells","opend","open","candidate"):
            print(name,arm,flush=True)
            raw = scorer.chaingrind(a.runtime, arm, symbols, latencies)
            body = {"stream_indices":group,"symbols_sha256":digest(symbols),
                "arm":arm,"raw":raw,"metrics":moonshot.proxy_metrics(raw)}
            (a.out / f"{name}-{arm}.json").write_text(json.dumps(body,indent=2)+"\n")

if __name__ == "__main__":
    main()
