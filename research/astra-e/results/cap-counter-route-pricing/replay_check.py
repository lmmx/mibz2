"""Compare recorded edge and scalar controls to their earlier same-binary runs."""
import argparse
import csv
from datetime import datetime
import json
from pathlib import Path
import sys
import tarfile

from report import ARMS, HERE, SAMPLES, WORKERS, digest, expected_config, geometry, require, scorer_and_model


def records(folder, arms):
    rows = [json.loads(line) for line in (folder / "runs.jsonl").read_text().splitlines()]
    cases = {(r["workers"], r["block"], r["arm"]): r for r in rows}
    wanted = {(w, b, a) for w in WORKERS for b in SAMPLES for a in arms}
    require(len(rows) == len(wanted) and set(cases) == wanted, f"unmatched or duplicate rows in {folder}")
    return cases


def compare(source, previous, current):
    old, new = (json.loads((p / "manifest.json").read_text()) for p in (previous, current))
    for key in ("files", "tools", "samples", "scorer_path", "scorer_version", "rustc",
                "valgrind", "corpus_sha256", "cpu", "nice"):
        require(old[key] == new[key], f"changed runtime or inputs: {key}")
    scorer, model = scorer_and_model(source, new)
    geometry(previous, old)
    with tarfile.open(previous / "workers.tar.gz", "r:gz") as archive:
        for arm in ("edge", "scalar"):
            before, after = old["arms"][arm], new["arms"][arm]
            require(all(after[key] == value for key, value in before.items()), f"changed binary identity: {arm}")
            with archive.extractfile(f"{arm}/moonshot") as binary:
                require(digest(binary.read()) == before["binary_sha256"], f"wrong archived binary: {arm}")

    historical = records(previous, ("edge", "scalar"))
    replay = records(current, ARMS)
    for case in sorted(historical):
        workers, block, arm = case
        before, after = historical[case], replay[case]
        require(datetime.fromisoformat(before["finish"]) < datetime.fromisoformat(after["start"]),
                f"missing independent repeat: {case}")
        lat = scorer.nominal_latencies(model, workers)
        p, q = before["counters"], after["counters"]
        require(before["latencies"] == after["latencies"] == lat and
                p["config"] == q["config"] == expected_config(model, lat), f"changed geometry: {case}")
        require(set(p) == set(q) and p["out_bytes"] == q["out_bytes"] == SAMPLES[block][0],
                f"changed event schema or measured length: {case}")
        prior, repeat = scorer.proxy_metrics(p), scorer.proxy_metrics(q)
        for name in sorted(p.keys() - {"config"}):
            prior[name], repeat[name] = p[name], q[name]
        for name in prior:
            x, y = prior[name], repeat[name]
            delta = y - x if x is not None and y is not None else None
            yield dict(workers=workers, block=block, arm=arm, metric=name, historical=x, replay=y,
                       delta=delta, abs_delta=abs(delta) if delta is not None else None,
                       replay_over_historical=y / x if delta is not None and x > 0 else None)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="checkout with the recorded scorer files")
    parser.add_argument("--previous", type=Path, default=HERE.parent / "scalar-route-pricing")
    parser.add_argument("--current", type=Path, default=HERE)
    args = parser.parse_args()
    rows = list(compare(args.source.resolve(), args.previous, args.current))
    writer = csv.DictWriter(sys.stdout, fieldnames=rows[0], lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
