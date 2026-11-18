"""Rewrap the systematic sample and a density-refused Wikidata control."""

import bz2
import csv
import hashlib
import json
import re
import sys
from pathlib import Path

corpus, records, out = map(Path, sys.argv[1:])
out.mkdir(parents=True, exist_ok=True)
sha = lambda b: hashlib.sha256(b).hexdigest()
data = corpus.read_bytes()
assert sha(data) == "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
bits = f"{int.from_bytes(data, 'big'):0{len(data) * 8}b}"
block, end = (f"{m:048b}" for m in (0x314159265359, 0x177245385090))
markers = list(re.finditer(f"{block}|{end}", bits))
starts = [i for i, m in enumerate(markers) if m.group() == block]
sample = [json.loads(s) for s in (records / "blocks.jsonl").read_text().splitlines()]
sample += [{"index": 270, "block": "control"}]
w = csv.writer(sys.stdout, lineterminator="\n")
w.writerow(
    (
        "ordinal",
        "large_ordinal",
        "payload_bits",
        "envelope_bytes",
        "output_bytes",
        "compressed_sha256",
        "output_sha256",
    )
)
for row in sample:
    i = starts[row["index"]]
    a, b = markers[i].start(), markers[i + 1].start()
    raw = bits[a:b] + end + bits[a + 48 : a + 80]
    raw += "0" * (-len(raw) % 8)
    wrapped = b"BZh9" + int(raw, 2).to_bytes(len(raw) // 8, "big")
    decoded = bz2.decompress(wrapped)
    if row["index"] == 1:
        assert (
            a == 336
            and sha(wrapped) == "e3bf5372b15885da138764a25bb2b7cd784ee3cbee1e664937647921ae3122b0"
        )
    (out / f"{row['index']:03d}.bz2").write_bytes(wrapped)
    w.writerow(
        (row["index"], row["block"], b - a, len(wrapped), len(decoded), sha(wrapped), sha(decoded))
    )
