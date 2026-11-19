"""One-worker Chaingrind screen using the official scorer's W=7 latencies and metrics."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys


FROZEN_SCORER = "9c31461df7bef24cecf79348545b27ee26846835"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("tree", "binary", "corpus", "runtime", "scratch", "output"):
        parser.add_argument(name, type=Path)
    a = parser.parse_args()
    source_check = subprocess.run([
        "git", "-C", a.tree, "diff", "--quiet", FROZEN_SCORER, "--", "bench/moonshot",
    ])
    if source_check.returncode:
        raise SystemExit("This historical screen requires scorer 9c31461. See proxy/README.md.")
    sys.path.insert(0, str(a.tree / "bench/moonshot"))
    import moonshot

    scorer = moonshot.Scorer(argparse.Namespace(target_dir=a.scratch))
    scorer.bin = a.binary
    w = 7
    latencies = moonshot.latencies(scorer.cal, w)
    git = lambda *args: subprocess.check_output(["git", "-C", a.tree, *args], text=True).strip()
    info = scorer.info()
    manifest = dict(candidate=info, binary_sha256=digest(a.binary), corpus_sha256=digest(a.corpus),
                    head=git("rev-parse", "HEAD"), source_path_revision=FROZEN_SCORER,
                    refreshed_path_revision=git("log","-1","--format=%H","origin/master","--","bench/moonshot"),
                    master=git("rev-parse","origin/master"), latencies=latencies, workers=w,
                    valgrind=subprocess.check_output(["valgrind","--version"],text=True).strip(),
                    printed_scorer_version=info.get("scorer_version",info.get("version")),
                    calibration_sha256=digest(a.tree / "bench/moonshot/calibration.json"),
                    tool_sha256=digest(a.runtime / "chaingrind-amd64-linux"),
                    rustc=subprocess.check_output(["rustc","-vV"],text=True).strip())
    a.output.mkdir(parents=True, exist_ok=True)
    (a.output/"manifest.json").write_text(json.dumps(manifest,indent=2)+"\n")
    # Ordinal 1 is the frozen full block. 41 and 85 span opposite sides of
    # the density threshold in the earlier systematic large-block audit.
    for group, blocks in (("frozen", [1]), ("density-pair", [41,85])):
        symbols = a.scratch / f"{group}.sym"
        extract = subprocess.check_output([a.binary,"extract","--file",a.corpus,"--blocks",
                                           ",".join(map(str,blocks)),"--out",symbols],text=True)
        if json.loads(extract)["blocks"] != len(blocks):
            raise ValueError(f"missing blocks {blocks}")
        for arm in ("lazy","opend","candidate"):
            print(f"screen {group} {arm}",flush=True)
            raw = scorer.chaingrind(a.runtime, arm, symbols, latencies)
            row = dict(arm=arm,workers=w,ordinals=blocks,symbols_sha256=digest(symbols),
                       raw=raw,metrics=moonshot.proxy_metrics(raw))
            (a.output/f"{group}-{arm}.json").write_text(json.dumps(row,indent=2)+"\n")


if __name__ == "__main__":
    main()
