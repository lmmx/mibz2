"""Measure the changed subset of the standard 48+16 sample on both frozen executables."""
import argparse
import csv
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    p = argparse.ArgumentParser(description=__doc__)
    for arg in ("checkout", "edge", "combined", "vlib", "corpus", "blocks", "output"):
        p.add_argument(arg, type=lambda s: Path(s).resolve())
    a = p.parse_args()
    root = a.checkout
    sys.path.insert(0, str(root / "bench/moonshot"))
    import moonshot
    def git(*args):
        return subprocess.check_output(["git", "-C", root, *args], text=True).strip()
    cal = json.loads(moonshot.CALIBRATION.read_text())
    sample = moonshot.proxy_sample(cal, "48+16")
    study = root / "research/astra-e"
    prev = json.loads((study / "results/combined-v1-proxy/checkpoint/provenance.json").read_text())
    census = study / "edge-budget/census.csv"
    with census.open() as f:
        changed = {int(r["ordinal"]) for r in csv.DictReader(f) if r["input"] == "wikidata" and r["extra_open"] == "1"}
    labels = {"base": sorted(changed.intersection(sample["full"] + sample["tiny"])),
              "hi": sorted(changed.intersection(sample["slope"]))}
    assert labels == {"base": [165], "hi": []}
    assert digest(a.corpus) == prev["source"]["corpora/wikidata.bz2"]
    assert digest(a.combined) == prev["binary"]["release/moonshot"]
    assert digest(a.vlib / "chaingrind-amd64-linux") == prev["binary"]["chaingrind/chaingrind-amd64-linux"]
    a.output.mkdir(parents=True, exist_ok=True)
    a.blocks.mkdir(parents=True, exist_ok=True)
    for b in labels["base"]:
        path = a.blocks / f"{b}.sym"
        if not path.exists():
            subprocess.run([a.edge, "extract", "--file", a.corpus, "--blocks", f"{sample['warm']},{b}", "--out", path], check=True)
        assert digest(path) == prev["symbols"][path.name]
    tools = {x: digest(root/x) for x in ("bench/moonshot/moonshot.py", "bench/moonshot/calibration.json",
                                           "bench/moonshot/chaingrind/chaingrind.c", "bench/moonshot/eval.py")}
    assert all(prev["source"][x] == h for x, h in tools.items() if x in prev["source"])
    info = {arm: json.loads(subprocess.check_output([binary, "info"])) for arm, binary in
            (("edge", a.edge), ("combined", a.combined))}
    assert info["edge"]["candidate"].endswith("/research/astra-e/edge-budget/candidate.rs")
    assert info["combined"]["candidate"].endswith("/research/astra-e/narrow-short/ahead.rs")
    meta = {"diagnostic_started": datetime.now(timezone.utc).isoformat(), "checkout": git("rev-parse", "HEAD"),
            "master": git("rev-parse", "origin/master"), "upstream_scorer": git("log", "-1", "--format=%H", "origin/master", "--", "bench/moonshot"),
            "scorer_tree": git("rev-parse", "origin/master:bench/moonshot"), "printed_scorer": f"scorer {moonshot.VERSION} at {moonshot.scorer_commit()}",
            "core_sha256": tools, "census_sha256": digest(census), "changed": sorted(changed), "sample": sample,
            "changed_in_sample": labels, "binary_sha256": {"edge": digest(a.edge), "combined": digest(a.combined),
            "chaingrind": digest(a.vlib / "chaingrind-amd64-linux")}, "bin_info": info,
            "sym_sha256": {str(b): digest(a.blocks / f"{b}.sym") for b in labels["base"]},
            "input_sha256": digest(a.corpus)}
    (a.output/"manifest.json").write_text(json.dumps(meta, indent=2)+"\n")
    file = a.output/"runs.jsonl"
    with file.open("x") as sink:
        for w in (7,8,10):
            for arm, binary in (("edge",a.edge),("combined",a.combined)):
                scorer = moonshot.Scorer(argparse.Namespace(target_dir=binary.parent))
                scorer.bin = binary
                for kind, blocks in labels.items():
                    lat=moonshot.nominal_latencies(cal["model"],w)
                    if kind=="hi":
                        lat["dram_lat"] += cal["model"]["slope_delta"]
                    for b in blocks:
                        counters = scorer.chaingrind(a.vlib,"candidate",a.blocks / f"{b}.sym",lat)
                        row={"workers":w,"arm":arm,"kind":kind,"block":b,"latencies":lat,"counters":counters}
                        sink.write(json.dumps(row)+"\n")
                        sink.flush()
                        print(w,arm,kind,b,counters["cycles"],flush=True)

if __name__ == "__main__":
    main()
