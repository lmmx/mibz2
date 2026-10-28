"""Whether the scaling knee follows the block size: the same content at -1, -3, -6 and -9 (#39).

#39's counters fit L3 capacity first (live T-vectors outgrowing the cache, the demand hit
share falling about as C / W with an effective C of 6-7 MB), then queueing past the LLC.
A bzip2 file's block size is chosen when it is compressed (100k-900k bytes, `-1`-`-9`),
and a block's T-vector is 4 B a byte of it: so the same content recompressed at smaller
blocks shrinks every worker's hot set without touching the decoder.

- capacity dominates → the best worker count moves up as the blocks shrink, roughly as
  C / (4 B x block size) (at C = 6.5 MB: -9 about 2, -6 about 3, -3 about 5, -1 past 10);
- queueing past the LLC dominates → the knee stays near the same worker count;
- both → it moves, but less than inversely.

Recompressing also changes the block boundaries, the Huffman tables and the number of
blocks, so this tests the system, not cache size alone.

`just bench-blocks [--levels 1,3,6,9] [--threads 2,4,6,8,10] [--rounds 6]` decodes the
benchmark's corpus once (lbzip2), recompresses it at each level (lbzip2 -N), runs
examples/ab.rs's mibz2 arm on each back to back, and writes blocks-<UTC time>.json.
"""

import argparse
import datetime
import json
import os
import shutil
import subprocess
import sys

from imc import CORPUS, run
from scaling import AB, summarise

C_MB = 6.5  # #39's fitted effective L3 for the walk (a hypothesis this tests)


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--levels", default="1,3,6,9")
    ap.add_argument("--threads", default="2,4,6,8,10")
    ap.add_argument("--rounds", type=int, default=6)
    ap.add_argument("--ref", type=int, default=4, help="the thread count scaling is against")
    ap.add_argument("--file", default=CORPUS, help="the content (default the benchmark's)")
    ap.add_argument("--note", default="", help="what was closed or running, for the record")
    a = ap.parse_args()
    root = run(["git", "rev-parse", "--show-toplevel"], check=True).stdout.strip()
    os.chdir(root)
    for p in [AB, a.file]:
        if not os.path.exists(p):
            sys.exit(f"bench-blocks: no {p}; `just bench-blocks` builds it")
    if not shutil.which("lbzip2"):
        sys.exit("bench-blocks: needs lbzip2")
    levels = [int(x) for x in a.levels.split(",")]
    threads = [int(t) for t in a.threads.split(",")]
    if a.ref not in threads:
        sys.exit(f"bench-blocks: --threads must include --ref {a.ref}")
    work = os.path.join(root, "target", "blocks")
    os.makedirs(work, exist_ok=True)
    raw = os.path.join(work, "content")
    print("bench-blocks: decoding the content once", flush=True)
    with open(raw, "wb") as f:
        if subprocess.run(["lbzip2", "-dc", a.file], stdout=f).returncode:
            sys.exit("bench-blocks: lbzip2 could not decode the content")
    files = {}
    for lv in levels:
        out = os.path.join(work, f"wikidata-l{lv}.bz2")
        print(f"bench-blocks: recompressing at -{lv}", flush=True)
        with open(out, "wb") as f:
            if subprocess.run(["lbzip2", f"-{lv}", "-c", raw], stdout=f).returncode:
                sys.exit(f"bench-blocks: lbzip2 -{lv} failed")
        files[lv] = out
    utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    res = {"meta": {"utc": utc, "note": a.note, "levels": levels, "c_mb": C_MB}, "levels": {}}
    for lv, path in files.items():
        raw_jsonl = f"blocks-{utc}-l{lv}.jsonl"
        argv = [AB, "run", raw_jsonl, "--arms", "mibz2", "--threads", a.threads]
        argv += ["--rounds", str(a.rounds), "--settle", "0", path]
        print(f"\nbench-blocks: -{lv} ({os.path.getsize(path) / 1e6:.1f} MB)", flush=True)
        if subprocess.run(argv).returncode:
            sys.exit("bench-blocks: ab run failed")
        rows, same, out_bytes = summarise(raw_jsonl, ["mibz2"], threads, a.ref, {"mibz2": f"-{lv}"})
        res["levels"][lv] = {
            "file_bytes": os.path.getsize(path),
            "same_bytes": same,
            "output_bytes": out_bytes,
            "row": rows[0],
            "records": raw_jsonl,
            "predicted_best_workers": C_MB * 1e6 / (4 * lv * 100_000),
        }
    print(f"\n== bench-blocks {utc}  note: {a.note or '-'}")
    print(f"{'level':<7}{'T-vector':>10}{'model best':>12}  MB/s by threads (best *)")
    print(f"{'':<29}" + "".join(f"{t:>8}" for t in threads))
    for lv in levels:
        r = res["levels"][lv]["row"]
        best = r["best"]["threads"]
        cells = "".join(f"{r['mb_s'][t]:7.0f}{'*' if t == best else ' '}" for t in threads)
        print(
            f"-{lv:<6}{4 * lv * 0.1:8.1f}MB{res['levels'][lv]['predicted_best_workers']:11.1f}  "
            + cells
        )
    print("scaling against each level's own 4-thread runs (95% in the json):")
    for lv in levels:
        s = res["levels"][lv]["row"]["scaling"]
        print(f"  -{lv}: " + "  ".join(f"{t}: {s[t]['ratio']:.2f}" for t in threads))
    with open(f"blocks-{utc}.json", "w") as f:
        json.dump(res, f, indent=1)
    shutil.rmtree(work, ignore_errors=True)
    print(f"\nbench-blocks: wrote blocks-{utc}.json (and ab's blocks-{utc}-l*.jsonl)")


if __name__ == "__main__":
    main()
