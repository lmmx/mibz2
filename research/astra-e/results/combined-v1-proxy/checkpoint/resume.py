"""Resume a frozen v1 proxy using the official scorer with per-block memoisation."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import os
import sys

HERE = Path(__file__).resolve().parent


def sha(path):
    with path.open("rb") as f:
        return hashlib.file_digest(f, "sha256").hexdigest()


def key(arm, block, latencies):
    return arm, block, tuple(sorted(latencies.items()))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("checkout", type=Path)
    p.add_argument("target", type=Path)
    p.add_argument("output", type=Path, help="new directory for a continued score")
    p.add_argument("--check-only", action="store_true")
    args = p.parse_args()
    checkout = args.checkout.resolve()
    target = args.target.resolve()
    output = args.output.resolve()
    meta = json.loads((HERE / "provenance.json").read_text())
    os.chdir(checkout)
    frozen_paths = {**{repo_path: checkout / repo_path for repo_path in meta["source"]},
                    **{bin_path: target / bin_path for bin_path in meta["binary"]}}
    for filename, expected in {**meta["source"], **meta["binary"]}.items():
        if sha(frozen_paths[filename]) != expected:
            raise ValueError(f"frozen input differs: {filename}")
    sys.path.insert(0, str(checkout / "bench/moonshot"))
    import moonshot

    cal = json.loads(moonshot.CALIBRATION.read_text())
    model = cal["model"]
    sample = moonshot.proxy_sample(cal, "48+16")
    valid = {}
    for w in model["workers"]:
        base = moonshot.nominal_latencies(model, w)
        hi = dict(base, dram_lat=base["dram_lat"] + model["slope_delta"])
        for arm in ("lazy", "candidate"):
            for kind, blocks, lat in (("base", sample["full"] + sample["tiny"], base),
                                      ("hi", sample["slope"], hi)):
                for block in blocks:
                    valid[key(arm, block, lat)] = (w, kind)
    assert len(valid) == 960
    cache = {}
    with gzip.open(HERE / "proxy-runs.jsonl.gz", "rt") as f:
        for line in f:
            row = json.loads(line)
            k = key(row["arm"], row["block"], row["latencies"])
            if k in cache or valid.get(k) != (row["workers"], row["kind"]):
                raise ValueError(f"invalid or repeated row: {k}")
            cache[k] = row["counters"]
    if len(cache) != meta["rows"]:
        raise ValueError("incomplete checkpoint")
    print(f"{len(cache)} unique matching rows of {len(valid)}", flush=True)
    if args.check_only:
        return 0

    class Resumed(moonshot.Scorer):
        def chaingrind(self, vlib, arm, sym, lat):
            if sha(vlib / "chaingrind-amd64-linux") != meta["binary"]["chaingrind/chaingrind-amd64-linux"]:
                raise ValueError("rebuilt simulator differs from the frozen simulator")
            if sha(sym) != meta["symbols"][sym.name]:
                raise ValueError(f"regenerated warm/measured symbol tape differs: {sym.name}")
            cached = cache.get(key(arm, int(sym.stem), lat))
            if cached is not None:
                return cached
            return super().chaingrind(vlib, arm, sym, lat)

    moonshot.Scorer = Resumed
    return moonshot.main(["score", "research/astra-e/narrow-short/ahead.rs",
                          "--quick", "--no-build", "--no-timing", "--proxy-blocks", "48+16",
                          "--target-dir", str(target), "--out", str(output),
                          "--record", str(output / "register.jsonl")])


if __name__ == "__main__":
    raise SystemExit(main())
