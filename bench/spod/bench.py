"""Build the fixed inputs, check the oracle, or run the paired Linux benchmark."""

import argparse
import bz2
import csv
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import tempfile

BASE = "8ff1187af6a920f7d57fd338680db3a5633a3ac5"
ARMS = ("workers", "aa", "teams", "spod")
THREADS = (2, 4, 8, 10)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def prepare(directory, wiki):
    directory.mkdir(exist_ok=True, parents=True)
    assert (
        sha(wiki.read_bytes())
        == json.loads(Path("bench/wikidata.json").read_text())["corpus_sha256"]
    )
    with tempfile.TemporaryDirectory() as tmp:
        src = Path(tmp) / "wiki.bz2"
        shutil.copyfile(wiki, src)
        subprocess.run(["bzip2recover", src], check=True, stderr=subprocess.DEVNULL)
        blocks = sorted(src.parent.glob("rec*.bz2"))
        assert len(blocks) == 431
        # Original compressed blocks 1, 2 and 3 (zero-based). Block 0 has 13 output bytes.
        chunks = [f.read_bytes() for f in blocks[1:4]]
        end_bytes = sum(len(bz2.decompress(f.read_bytes())) for f in blocks[-3:])
    # Same source set as tests/common::corpora, pinned to the base for both arms.
    paths = subprocess.check_output(
        ["git", "ls-tree", "-r", "--name-only", BASE, "src", "tests", "examples"], text=True
    ).splitlines()
    source = b"".join(
        subprocess.check_output(["git", "show", f"{BASE}:{p}"])
        for d in ("src", "tests", "examples")
        for p in sorted(paths)
        if Path(p).parent == Path(d)
    )
    source += subprocess.check_output(["git", "show", f"{BASE}:README.md"])
    text = (source * (2_400_000 // len(source) + 1))[:2_400_000]
    manifest = {}
    for size in (1, 2, 3):
        for name, data in [
            (f"wiki{size}", b"".join(chunks[:size])),
            (f"text{size}", bz2.compress(text[: size * 800_000])),
        ]:
            path = directory / (name + ".bz2")
            path.write_bytes(data)
            with tempfile.TemporaryDirectory() as tmp:
                copy = Path(tmp) / path.name
                shutil.copyfile(path, copy)
                subprocess.run(["bzip2recover", copy], check=True, stderr=subprocess.DEVNULL)
                assert len(list(copy.parent.glob("rec*.bz2"))) == size
            manifest[name] = {"path": path.name, "prefix": 0, "blocks": size}
    shutil.copyfile(wiki, directory / "tail.bz2")
    manifest["tail"] = {"path": "tail.bz2", "prefix": 209_700_196 - end_bytes, "blocks": 431}
    for info in manifest.values():
        path = directory / info["path"]
        decoded = subprocess.check_output(["bzip2", "-dc", path])
        info.update(
            compressed_sha256=sha(path.read_bytes()), sha256=sha(decoded), bytes=len(decoded)
        )
    (directory / "inputs.json").write_text(json.dumps(manifest, indent=2) + "\n")


def check(directory, binary, columns):
    manifest = json.loads((directory / "inputs.json").read_text())
    for name, info in manifest.items():
        for threads in (1, 4, 8, 10):
            for mode in ("default", "spod"):
                with (directory / info["path"]).open("rb") as inp:
                    result = subprocess.run(
                        [binary, str(threads)],
                        stdin=inp,
                        stdout=subprocess.PIPE,
                        check=True,
                        env=dict(os.environ, BZ2_INVERSE=mode),
                    )
                assert sha(result.stdout) == info["sha256"], (name, threads, mode)
        print(name, info["sha256"], flush=True)
    authenticated(columns)


def authenticated(columns, emit_csv=None):
    blocks = json.loads((columns / "manifest.json").read_text())["blocks"]
    assert len(blocks) == 6
    for b in blocks:
        path = columns / b["file"]
        data = path.read_bytes()
        assert sha(data) == b["sha256"] and len(data) == b["n"]
        assert 1 + sum(a != c for a, c in zip(data, data[1:])) == b["r"]
        args = [
            "cargo",
            "test",
            "--release",
            "--features",
            "spod",
            "--lib",
            "open::spod::tests::authenticated_column",
            "--",
            "--ignored",
            "--exact",
            "--nocapture",
        ]
        env = dict(os.environ, SPOD_COLUMN=str(path), SPOD_MARK=str(b["primary_index"]))
        if emit_csv is not None:
            env["SPOD_TIME_EMIT"] = "1"
        result = subprocess.run(args, env=env, check=True, stdout=subprocess.PIPE, text=True)
        assert "1 passed" in result.stdout
        if emit_csv is not None:
            rows = [
                line.split(",")[1:]
                for line in result.stdout.splitlines()
                if line.startswith("EMIT,")
            ]
            assert len(rows) == 8 * 4 * 3
            with emit_csv.open("a") as f:
                csv.writer(f, lineterminator="\n").writerows([[b["file"], *row] for row in rows])
        print(path.name, b["sha256"], flush=True)


def emit(columns, output):
    with output.open("x") as f:
        csv.writer(f, lineterminator="\n").writerow(["column", "round", "threads", "arm", "ns"])
    authenticated(columns, output)


def measure(binary, path, arm, threads, prefix, expected):
    with tempfile.TemporaryFile() as output:
        proc = subprocess.Popen(
            [binary, path, "workers" if arm == "aa" else arm, str(threads), str(prefix)],
            stdout=output,
        )
        _, status, usage = os.wait4(proc.pid, 0)
        proc.returncode = os.waitstatus_to_exitcode(status)
        if proc.returncode:
            raise RuntimeError((proc.args, proc.returncode))
        output.seek(0)
        wall, tail, ticks, rss, hits, size = map(int, output.read().split())
    assert size == expected
    return dict(
        wall_ms=wall / 1e6,
        tail_ms=tail / 1e6,
        cpu_ms=(usage.ru_utime + usage.ru_stime) * 1000,
        tail_cpu_ms=1000 * ticks / os.sysconf("SC_CLK_TCK"),
        rss_kib=rss,
        hits=hits,
    )


def run(directory, master, head, output):
    inputs = json.loads((directory / "inputs.json").read_text())
    with output.open("x") as f:
        writer = None
        # Balanced Williams order. Repeat twice, reversing the order of the cells.
        cells = [(n, t) for n in inputs for t in THREADS]
        for r in range(8):
            for name, threads in cells if r % 2 == 0 else cells[::-1]:
                info = inputs[name]
                for idx in (0, 1, 3, 2):
                    arm = ARMS[(idx + r) % 4]
                    stats = measure(
                        head if arm == "spod" else master,
                        directory / info["path"],
                        arm,
                        threads,
                        info["prefix"],
                        info["bytes"],
                    )
                    row = dict(round=r + 1, case=name, threads=threads, arm=arm, **stats)
                    if writer is None:
                        writer = csv.DictWriter(f, fieldnames=row, lineterminator="\n")
                        writer.writeheader()
                    writer.writerow(row)
                    f.flush()


def interval(rows, a, b, metric):
    logs = [math.log(float(x[a][metric]) / float(x[b][metric])) for x in rows]
    m = statistics.mean(logs)
    half = 2.365 * statistics.stdev(logs) / math.sqrt(8)
    return tuple(math.exp(t) for t in (m, m - half, m + half))


def report(raw):
    groups = {}
    for row in csv.DictReader(raw.open()):
        key = row["case"], row["threads"]
        groups.setdefault(key, {}).setdefault(row["round"], {})[row["arm"]] = row
    print("case t metric candidate/reference ratio lower95 upper95")
    for (name, threads), rounds in groups.items():
        rows = list(rounds.values())
        assert len(rows) == 8
        for metric in ["wall_ms", "cpu_ms", "rss_kib"] + (["tail_ms"] if name == "tail" else []):
            for a, b in [("spod", "workers"), ("spod", "teams"), ("aa", "workers")]:
                print(
                    name,
                    threads,
                    metric,
                    f"{a}/{b}",
                    *(f"{v:.3f}" for v in interval(rows, a, b, metric)),
                )


def emit_report(raw):
    groups = {}
    for row in csv.DictReader(raw.open()):
        key = row["column"], row["threads"]
        groups.setdefault(key, {}).setdefault(row["round"], {})[row["arm"]] = row
    print("column threads comparison speedup lower95 upper95")
    for (column, threads), rounds in groups.items():
        pairs = list(rounds.values())
        assert len(pairs) == 8
        for arm, label in (("2", "serial/SPOD"), ("1", "serial/serial")):
            print(column, threads, label, *(f"{v:.3f}" for v in interval(pairs, "0", arm, "ns")))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    for mode, args in {
        "prepare": ("directory", "wiki"),
        "check": ("directory", "binary", "columns"),
        "emit": ("columns", "output"),
        "run": ("directory", "master", "head", "output"),
        "report": ("raw",),
        "emit_report": ("raw",),
    }.items():
        p = sub.add_parser(mode)
        for arg in args:
            p.add_argument(arg, type=Path)
    opts = vars(parser.parse_args())
    globals()[opts.pop("mode")](**opts)
