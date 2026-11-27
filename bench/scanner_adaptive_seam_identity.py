"""Untimed decoder identity checks for the adaptive seam window workstation experiment (scanner-adaptive-seam-window)."""

import argparse
import bz2
import hashlib
import json
import os
import re
import subprocess
from pathlib import Path

OUTPUT_BYTES = 4 * 209_700_196
ARM_COUNTS = {"mibz2": (6, 8, 9, 10, 12), "mibz2-workers-lazy": (8, 9, 10)}


def reference(corpus, base, head):
    digest = hashlib.sha256()
    size = 0
    with bz2.open(corpus, "rb") as stream:
        while chunk := stream.read(1 << 20):
            size += len(chunk)
            digest.update(chunk)
    if size != OUTPUT_BYTES:
        raise ValueError(f"reference decoded {size} bytes, expected {OUTPUT_BYTES}")
    expected = digest.hexdigest()
    short_hash = None
    samples = []
    for build, executable in (("base", base), ("head", head)):
        for arm, counts in ARM_COUNTS.items():
            for count in counts:
                result = subprocess.run(
                    [executable, "child", arm, str(count), corpus],
                    env={**os.environ, "AB_SHA256": "1"},
                    check=True, capture_output=True, text=True,
                )
                child = json.loads(result.stdout)
                if (child.get("err") != "" or child.get("out") != size
                        or child.get("sha256") != expected
                        or not re.fullmatch("[0-9a-f]{16}", str(child.get("hash")))):
                    raise ValueError(f"{build} {arm} {count}: output differs from reference")
                if short_hash is None:
                    short_hash = child["hash"]
                if child["hash"] != short_hash:
                    raise ValueError(f"{build} {arm} {count}: inconsistent streaming hash")
                samples.append({"build": build, "arm": arm, "threads": count, "record": child})
    return {"sha256": expected, "hash": short_hash, "out": size, "checks": samples}


def check_trace(path, identity, rounds):
    report = json.loads(path.read_text())
    if report.get("parity") is not True or report.get("output_bytes") != identity["out"]:
        raise ValueError(f"{path}: trace failed output parity")
    want = {(r, mode) for r in range(rounds) for mode in ("off", "timeline", "full")}
    seen = set()
    for run in report["runs"]:
        fields = dict(word.split("=", 1) for word in run["line"].split() if "=" in word)
        key = (run["round"], run["mode"])
        if (key not in want or key in seen or run["threads"] != 10
                or fields.get("err") != "-" or fields.get("hash") != identity["hash"]
                or fields.get("out") != str(identity["out"])):
            raise ValueError(f"{path}: invalid trace output for {key}")
        seen.add(key)
    if seen != want:
        raise ValueError(f"{path}: missing trace records {sorted(want - seen)}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    cmd = parser.add_subparsers(dest="mode", required=True)
    capture = cmd.add_parser("reference")
    capture.add_argument("corpus")
    capture.add_argument("base")
    capture.add_argument("head")
    trace = cmd.add_parser("trace")
    trace.add_argument("identity", type=Path)
    trace.add_argument("rounds", type=int)
    trace.add_argument("reports", type=Path, nargs="+")
    args = parser.parse_args()
    if args.mode == "reference":
        print(json.dumps(reference(args.corpus, args.base, args.head), indent=2))
    else:
        for report_path in args.reports:
            check_trace(report_path, json.loads(args.identity.read_text()), args.rounds)
