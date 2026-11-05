#!/usr/bin/env python3
"""Workstation runs: `examples/ab.rs` runs made off CI, kept with their machine.

    python3 bench/release_scaling.py record DIR --runs RUNS.jsonl [--env ENV.json]
        [--lscpu LSCPU.txt] [--machine NAME] [--ref ARM] [--arm ARM=LABEL ...]
        [--file NAME=PATH ...] [--note TEXT] [--date YYYY-MM-DD]

writes DIR/<date>-<machine>/: the harness's records as they were (runs.jsonl), the
machine as the runs saw it (env.json, from `ab env`), and meta.json: the date, the CPU
model and counts (from --lscpu, an `lscpu` output saved on that machine, else this one's),
the reference arm, each arm's label (a release or commit), each corpus's bytes and
SHA-256 (--file, when the files are at hand), and a note.

The site's Workstation page (docs/scripts/bench_release_scaling.py) reads every such
directory. CI's figures never depend on them: a pull request benches without any.
"""

import argparse
import hashlib
import json
import platform
import re
import shutil
import subprocess
import sys
from datetime import date
from pathlib import Path


def lscpu_fields(text: str) -> dict:
    """`lscpu`'s 'Key: value' lines, as a dict."""
    out = {}
    for line in text.splitlines():
        if ":" in line:
            key, _, value = line.partition(":")
            out[key.strip()] = value.strip()
    return out


def machine(lscpu: Path | None) -> dict:
    """The CPU model, its sockets, cores and threads, and the OS."""
    if lscpu is not None:
        text = lscpu.read_text()
    else:
        try:
            text = subprocess.run(["lscpu"], capture_output=True, text=True, check=True).stdout
        except (OSError, subprocess.CalledProcessError):
            text = ""
    f = lscpu_fields(text)
    num = lambda k: int(f[k]) if f.get(k, "").isdigit() else None  # noqa: E731
    sockets, per_socket, per_core = num("Socket(s)"), num("Core(s) per socket"), num("Thread(s) per core")
    return {
        "cpu": f.get("Model name"),
        "cpus": num("CPU(s)"),
        "cores": sockets * per_socket if sockets and per_socket else None,
        "threads_per_core": per_core,
        "max_mhz": f.get("CPU max MHz"),
        "os": None if lscpu is not None else f"{platform.system()} {platform.release()}",
    }


def slug(text: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-") or "machine"


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def record(a: argparse.Namespace) -> None:
    runs = [json.loads(line) for line in a.runs.read_text().splitlines() if line.strip()]
    if not runs:
        sys.exit(f"{a.runs}: no runs")
    arms = sorted({r["arm"] for r in runs})
    ref = a.ref or arms[0]
    if ref not in arms:
        sys.exit(f"--ref {ref}: not an arm of these runs ({', '.join(arms)})")
    labels = dict(x.split("=", 1) for x in a.arm)
    files = {}
    for spec in a.file:
        name, _, path = spec.partition("=")
        p = Path(path)
        files[name] = {"bytes": p.stat().st_size, "sha256": sha256(p)}
    m = machine(a.lscpu)
    day = a.date or date.today().isoformat()
    out = a.dir / f"{day}-{slug(a.machine or m['cpu'] or 'machine')}"
    out.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(a.runs, out / "runs.jsonl")
    if a.env:
        shutil.copyfile(a.env, out / "env.json")
    meta = {
        "date": day,
        "machine": a.machine,
        **m,
        "ref": ref,
        "arms": {arm: labels.get(arm, arm) for arm in arms},
        "files": files,
        "rounds": len({r["round"] for r in runs}),
        "threads": sorted({r["threads"] for r in runs}),
        "note": a.note,
    }
    (out / "meta.json").write_text(json.dumps(meta, indent=2) + "\n")
    print(out)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("record")
    r.add_argument("dir", type=Path)
    r.add_argument("--runs", type=Path, required=True)
    r.add_argument("--env", type=Path)
    r.add_argument("--lscpu", type=Path)
    r.add_argument("--machine")
    r.add_argument("--ref")
    r.add_argument("--arm", action="append", default=[], help="ARM=LABEL, e.g. mibz2=0.2.4")
    r.add_argument("--file", action="append", default=[], help="NAME=PATH, a corpus file")
    r.add_argument("--note")
    r.add_argument("--date")
    a = ap.parse_args()
    if a.cmd == "record":
        record(a)


if __name__ == "__main__":
    main()
