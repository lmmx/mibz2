"""The scanner diagnostic's inputs and their expected decoded sizes, anchored apart from the run
(bench-par-scan; docs/journal/2026-11-10-parallel-scan.md, section 5).

    python3 bench/par_scan_inputs.py MANIFEST CORPUS GEN_LOG RECORDS OUT_DIR > inputs.json

- `wikidata`: CORPUS, whose SHA-256 must be the manifest's `corpus_sha256`. Its decode by
  libbzip2 (Python's bz2) must be the manifest's `decoded_bytes` long.
- `wikidata-l1`: those decoded bytes recompressed by libbzip2 at level 1, written to OUT_DIR.
  The file is read back and decoded again by libbzip2: its bytes must have the corpus's decoded
  SHA-256 and the manifest's size.
- `records`: RECORDS, written by `ab gen`, whose log (GEN_LOG) states the records' size before
  compression and the file's size. libbzip2's decode of the file must be that size.

Each input's expected size is then passed to bench/par_scan_knee.py, which refuses cells of any
other size. The harness's own check compares mibz2::decode's bytes with libbzip2's.

Prints one JSON object: per input, its file's bytes and SHA-256, the expected decoded bytes and
their SHA-256 and where the size comes from; and the compressor's identity (Python's version and
libbzip2's). Exits 1, naming the fault, on any mismatch.
"""

import bz2
import ctypes
import hashlib
import json
import re
import shutil
import sys
from pathlib import Path


def die(msg: str) -> None:
    sys.exit(f"par_scan_inputs: {msg}")


def sha(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def libbzip2_version() -> str:
    try:
        import _bz2

        f = ctypes.CDLL(_bz2.__file__).BZ2_bzlibVersion
        f.restype = ctypes.c_char_p
        return f().decode()
    except Exception as e:  # recorded, not required
        return f"unknown ({type(e).__name__})"


def generated_size(log: str, name: str) -> tuple[int, int]:
    """`ab gen`'s line for NAME: its size before compression, and the file's."""
    hits = re.findall(rf"(?m)^.*/{re.escape(name)}: (\d+) -> (\d+) bytes$", log)
    if len(hits) != 1:
        die(f"the generator's log states {name}'s sizes {len(hits)} times, not once")
    return int(hits[0][0]), int(hits[0][1])


def main(manifest: str, corpus: str, gen_log: str, records: str, out_dir: str) -> dict:
    spec = json.loads(Path(manifest).read_text())
    want_bytes = spec["decoded_bytes"]
    if not isinstance(want_bytes, int) or want_bytes < 1:
        die(f"the manifest's decoded_bytes is {want_bytes!r}")
    out = Path(out_dir)
    inputs = {}

    z = Path(corpus).read_bytes()
    if sha(z) != spec["corpus_sha256"]:
        die(f"{corpus}'s SHA-256 is not the manifest's corpus_sha256")
    d = bz2.decompress(z)
    if len(d) != want_bytes:
        die(f"libbzip2 decodes {corpus} to {len(d)} bytes, not the manifest's {want_bytes}")
    digest = sha(d)
    shutil.copyfile(corpus, out / "wikidata.bz2")
    inputs["wikidata"] = {
        "file_bytes": len(z),
        "file_sha256": sha(z),
        "expected_bytes": want_bytes,
        "decoded_sha256": digest,
        "expected_from": f"{manifest}: decoded_bytes (libbzip2's decode agrees)",
    }

    (out / "wikidata-l1.bz2").write_bytes(bz2.compress(d, 1))
    del d
    z1 = (out / "wikidata-l1.bz2").read_bytes()
    d1 = bz2.decompress(z1)
    if len(d1) != want_bytes or sha(d1) != digest:
        die(
            "the generated level-1 input does not decode to the corpus's bytes "
            f"({len(d1)} bytes, SHA-256 {sha(d1)}; want {want_bytes}, {digest})"
        )
    del d1
    inputs["wikidata-l1"] = {
        "file_bytes": len(z1),
        "file_sha256": sha(z1),
        "expected_bytes": want_bytes,
        "decoded_sha256": digest,
        "expected_from": f"{manifest}: decoded_bytes; read back, libbzip2's decode has the "
        "corpus's decoded SHA-256",
    }

    rz = Path(records).read_bytes()
    raw, packed = generated_size(Path(gen_log).read_text(), "records.bz2")
    if packed != len(rz):
        die(f"{records} is {len(rz)} bytes, not the {packed} the generator wrote")
    rd = bz2.decompress(rz)
    if len(rd) != raw:
        die(f"libbzip2 decodes {records} to {len(rd)} bytes, not the generator's {raw}")
    shutil.copyfile(records, out / "records.bz2")
    inputs["records"] = {
        "file_bytes": len(rz),
        "file_sha256": sha(rz),
        "expected_bytes": raw,
        "decoded_sha256": sha(rd),
        "expected_from": "ab gen's stated size before compression (libbzip2's decode agrees)",
    }
    return {
        "inputs": inputs,
        "compressor": {"python": sys.version.split()[0], "libbzip2": libbzip2_version()},
    }


if __name__ == "__main__":
    if len(sys.argv) != 6:
        die("usage: par_scan_inputs.py MANIFEST CORPUS GEN_LOG RECORDS OUT_DIR")
    print(json.dumps(main(*sys.argv[1:]), indent=1))
