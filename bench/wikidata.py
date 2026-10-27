"""The benchmark's real-data corpus: Wikimedia's own bzip2 blocks, from byte-range windows
of a dated Wikidata JSON dump (bench/wikidata.json pins the dump, each window's offset,
length and SHA-256, and the corpus's SHA-256).

- `fetch SPEC DIR`: each window into DIR as <name>.bin, kept if already there with its
  SHA-256, else read from the dump by an HTTP byte range and checked. Prints where the
  windows came from ("kept" or "dumps.wikimedia.org").
- `readme SPEC`: the README of the `bench-data` branch, where the windows are kept.
- `build SPEC DIR WINDOWS_EXE OUT`: each window's whole blocks as a stream of their own
  (examples/windows.rs, which also checks them against libbzip2 and lbzip2), the streams
  one after another written to OUT, and OUT's SHA-256 checked against the spec's.

Wikimedia serves a dated dump for about six weeks; the Benchmark workflow keeps the
windows on the repository's `bench-data` branch once master's run has read them.
"""

import hashlib
import json
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fetch(spec: dict, out: Path) -> None:
    out.mkdir(parents=True, exist_ok=True)
    fetched = 0
    for w in spec["windows"]:
        path = out / f"{w['name']}.bin"
        if path.exists() and sha256(path) == w["sha256"]:
            continue
        end = w["offset"] + w["length"] - 1
        req = urllib.request.Request(
            spec["url"],
            headers={
                "Range": f"bytes={w['offset']}-{end}",
                "User-Agent": "mibz2 benchmark (https://github.com/lmmx/mibz2)",
            },
        )
        for attempt in range(4):
            try:
                with urllib.request.urlopen(req, timeout=60) as r:
                    data = r.read()
                break
            except OSError as e:
                if attempt == 3:
                    sys.exit(f"{w['name']}: {spec['url']} bytes {w['offset']}-{end}: {e}")
                time.sleep(5 * (attempt + 1))
        if hashlib.sha256(data).hexdigest() != w["sha256"]:
            sys.exit(f"{w['name']}: the bytes read do not have the window's SHA-256")
        path.write_bytes(data)
        fetched += 1
    print("dumps.wikimedia.org" if fetched else "kept")


def build(spec: dict, windows: Path, exe: str, out: Path) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        paths = [str(windows / f"{w['name']}.bin") for w in spec["windows"]]
        subprocess.run([exe, tmp, *paths], check=True, stdout=sys.stderr)
        with open(out, "wb") as f:
            for w in spec["windows"]:
                f.write((Path(tmp) / f"{w['name']}.blocks.bz2").read_bytes())
    if sha256(out) != spec["corpus_sha256"]:
        sys.exit(f"{out}: not the corpus's SHA-256 ({spec['corpus_sha256']})")
    print(f"{out}: {out.stat().st_size} bytes, SHA-256 checked", file=sys.stderr)


def readme(spec: dict) -> None:
    print(f"""# Benchmark data

Data only, never merged: the input of mibz2's benchmark (`.github/workflows/bench.yml` on
master) that its sources no longer serve. The Benchmark workflow adds to it from master's
runs.

## {spec["windows_dir"]}

{len(spec["windows"])} byte-range windows of `{spec["dump"]}` ({spec["dump_bytes"]:,} bytes,
SHA-1 `{spec["dump_sha1"]}`, from {spec["url"]}): {spec["about"]} `wikidata.json` beside
them (the repository's `bench/wikidata.json`) holds each window's offset, length and
SHA-256, and the corpus's SHA-256.

Wikidata's data is CC0.""")


def main() -> None:
    mode, spec = sys.argv[1], json.loads(Path(sys.argv[2]).read_text())
    if mode == "fetch":
        fetch(spec, Path(sys.argv[3]))
    elif mode == "readme":
        readme(spec)
    elif mode == "build":
        build(spec, Path(sys.argv[3]), sys.argv[4], Path(sys.argv[5]))
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
