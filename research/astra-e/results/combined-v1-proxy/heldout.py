"""Check prespecified v1 simulator estimates against a verified target i9 evidence bundle."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile

ROOT = Path(subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip())
STUDY = ROOT / "research/astra-e/results/combined-v1-proxy"
TARGET = "8c9640d8ab186f81a0500f48e606e0c21c2a6669"
SCORER = "604dac3f46569edd9ac9b6706b23a68d9d7d8f8d"


def source(commit, path):
    return subprocess.check_output(["git", "show", f"{commit}:{path}"], cwd=ROOT)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("tarball", type=Path)
    p.add_argument("sha256")
    args = p.parse_args()
    with args.tarball.open("rb") as f:
        assert hashlib.file_digest(f, "sha256").hexdigest() == args.sha256
    with tarfile.open(args.tarball) as bundle:
        files = {m.name.split("/", 1)[1]: m for m in bundle if m.isfile()}
        def read(name):
            return bundle.extractfile(files[name]).read().decode()
        assert read("mode.txt").strip() == "measure"
        assert read("commits.txt").splitlines()[0] == TARGET
        assert json.loads(read("scorer-refresh.json"))["selected_path_revision"] == SCORER
        assert "10900K" in read("lscpu.txt")
        gate = json.loads(read("candidate-gate/summary.json"))
        assert gate["gate"]["verdict"] == "PASS" and gate["gate"]["cases"] == 338
        assert gate["scorer"] == {"version": "v1", "commit": "9a9c169d70639c1c3e56cb85b3679fa7529e516c"}
        inputs = {line.split()[0] for line in read("inputs.sha256").splitlines()}
        assert inputs == {"82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915",
                          "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"}
        recorded = {path: h for h,path in (line.split() for line in read("sources.sha256").splitlines())}
        provenance = json.loads((STUDY/"checkpoint/provenance.json").read_text())
        for path in ("bench/moonshot/moonshot.py", "research/astra-e/narrow-short/ahead.rs"):
            assert recorded[path] == provenance["source"][path]
        rows = [json.loads(line) for line in read("runs.jsonl").splitlines()]
        official = json.loads(read("decision.json"))
    for path in ("research/astra-e/workstation/assess.py", "bench/moonshot/moonshot.py"):
        expected = source(TARGET, path)
        assert (ROOT/path).read_bytes() == expected, f"run from a checkout with frozen {path}"
    sys.path.insert(0, str(ROOT/"research/astra-e/workstation"))
    from assess import assess
    replay = json.loads(json.dumps(assess(rows)))
    assert replay == official
    proxy = json.loads(source("2337830","research/astra-e/results/combined-v1-proxy/score/summary.json"))
    prediction = proxy["proxy"]["predicted_over_lazy"]["candidate"]
    deposited = proxy["reference"]["predicted_over_lazy"]["opend"]
    out = csv.writer(sys.stdout, lineterminator="\n")
    out.writerow(("W", "v1_candidate_lazy", "v1_bootstrap_lo", "v1_bootstrap_hi",
                  "i9_candidate_lazy", "i9_paired_lo", "i9_paired_hi", "i9_point_inside_v1_interval",
                  "v1_candidate_deposited_opend", "i9_candidate_opend", "i9_vs_opend_lo", "i9_vs_opend_hi",
                  "i9_rss_median_ratio_vs_lazy", "i9_strict_target_met"))
    for w, r in official["workers"].items():
        model = prediction.get(w)
        obs = r["lazy"]["wall"]
        direct = r["opend"]["wall"]
        out.writerow((w,model["ratio"] if model else None, model["lo"] if model else None, model["hi"] if model else None,
                      obs["geomean"], obs["lo"], obs["hi"], (model["lo"]<=obs["geomean"]<=model["hi"]) if model else None,
                      (model["ratio"]/deposited[w]["ratio"]) if model else None, direct["geomean"], direct["lo"], direct["hi"],
                      r["lazy"]["median_peak_ratio"], r["target_met"]))


if __name__ == "__main__":
    main()
