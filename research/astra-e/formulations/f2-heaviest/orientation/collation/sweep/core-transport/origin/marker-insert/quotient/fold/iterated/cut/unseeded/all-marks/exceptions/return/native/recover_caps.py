"""Recover the exact original 127-capped records from an authenticated physical column."""
import argparse
import hashlib
import itertools
import json
import struct
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("masks", type=Path)
parser.add_argument("output", type=Path)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
manifest = json.loads(Path(__file__).with_name("manifest.json").read_text())
for record in manifest["inputs"]:
    if record["k"] != 8:
        continue
    block = record["index"]
    data = (args.masks / f"{block}.mask8").read_bytes()
    assert data[:8] == b"CLKMASK1"
    n, mark = struct.unpack_from("<II", data, 8)
    assert len(data) == 16 + 2*n and mark < n
    caps = bytearray()
    for byte, values in itertools.groupby(data[16:16+n]):
        width = sum(1 for _ in values)
        full, tail = divmod(width, 127)
        caps.extend(bytes((127, byte)) * full)
        if tail:
            caps.extend((tail, byte))
    assert hashlib.sha256(caps).hexdigest() == record["cap_sha256"]
    with (args.output / f"block-{block}.caps").open("xb") as f:
        f.write(caps)
