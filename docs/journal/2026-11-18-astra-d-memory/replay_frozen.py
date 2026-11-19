"""Replay the archived frozen references without the Rust toolchain or Valgrind."""

import csv
import hashlib
import io
import os
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

study = Path(__file__).resolve().parent
evidence = study / "evidence"
arms = ("stock", "direct", "tiles")
members = ("events", "ranges", "instructions")


def table(path):
    """Read a CSV file as a list of rows."""
    with path.open() as data:
        return list(csv.DictReader(data))


def read_traces(work):
    """Extract the archived traces for every arm into the work directory."""
    expected = {f"{arm}/{name}" for arm in arms for name in members}
    for arm in arms:
        (work / arm).mkdir()
    with tarfile.open(evidence / "frozen-native.tar.xz", "r|xz") as archive:
        for member in archive:
            if member.name not in expected or not member.isfile():
                raise ValueError(f"Unexpected archive member {member.name}")
            expected.remove(member.name)
            with archive.extractfile(member) as source, (work / member.name).open("wb") as dest:
                shutil.copyfileobj(source, dest)
    assert not expected, expected
    for row in table(evidence / "trace-digests.csv"):
        if row["ordinal"] == "1":
            for name in members:
                with (work / row["arm"] / name).open("rb") as data:
                    assert hashlib.file_digest(data, "sha256").hexdigest() == row[name]


def verify(work, arm):
    """Check one arm's replay against its stored block rows."""
    local = work / arm
    stored = [r for r in table(evidence / f"{arm}-blocks.csv") if r["ordinal"] == "1"]
    with (local / "instructions").open() as data:
        events = sum(int(n) for _, n in csv.reader(data))
    for block in stored:
        capacity = block["llc_KiB"]
        result = subprocess.check_output(
            [work / "replay", local / "ranges", local / "events", capacity], text=True
        )
        rows = list(csv.DictReader(io.StringIO(result)))
        assert events == int(block["instructions"])
        for metric in rows[0].keys() - {"phase", "region"}:
            assert sum(int(r[metric]) for r in rows) == int(block[metric]), (arm, capacity, metric)
        if capacity == "2048":
            expected = [
                {k: v for k, v in r.items() if k not in {"ordinal", "llc_KiB"}}
                for r in table(evidence / f"{arm}-regions.csv")
                if r["ordinal"] == "1"
            ]
            assert rows == expected, (arm, "phase and region")
        transfers = sum(int(block[k]) for k in ("dram_read", "dram_rfo", "dram_wb"))
        rate = 64 * transfers / int(block["output"])
        print(f"{arm:6} {capacity} KiB LLC  {transfers:7,} DRAM lines  {rate:.6f} B/B")


if __name__ == "__main__":
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        read_traces(work)
        subprocess.run(
            [
                os.environ.get("CXX", "c++"),
                "-std=c++17",
                "-O3",
                study / "replay.cc",
                "-o",
                work / "replay",
            ],
            check=True,
        )
        for arm in arms:
            verify(work, arm)
