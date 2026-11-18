"""Rewrap the public, hash-pinned Wikimedia windows. Only an input-preparation tool."""
import bz2
import csv
import hashlib
import json
import re
import sys
from pathlib import Path

BLOCK = f"{0x314159265359:048b}"
END = f"{0x177245385090:048b}"
HEADER = f"{int.from_bytes(b'BZh9', 'big'):032b}"


def envelope(parts):
    crc = 0
    for part in parts:
        crc = ((crc << 1 | crc >> 31) & 0xffffffff) ^ int(part[48:80], 2)
    bits = HEADER + "".join(parts) + END + f"{crc:032b}"
    bits += "0" * (-len(bits) % 8)
    return int(bits, 2).to_bytes(len(bits) // 8, "big")


def prepare(windows, out):
    spec = json.loads((windows / "wikidata.json").read_text())
    assert spec["corpus_sha256"] == "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
    corpus = bytearray()
    checksums = []
    for window in spec["windows"]:
        raw = (windows / (window["name"] + ".bin")).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == window["sha256"]
        bits = f"{int.from_bytes(raw, 'big'):0{len(raw)*8}b}"
        marks = list(re.finditer(BLOCK + "|" + END, bits))
        parts = [bits[a.start():b.start()] for a,b in zip(marks, marks[1:]) if a.group() == BLOCK]
        assert len(parts) == window["blocks"]
        for part in parts:
            raw_block = bz2.decompress(envelope([part]))
            checksums.append((len(checksums), len(raw_block), hashlib.sha256(raw_block).hexdigest()))
        if window["name"] == "w00":
            single = envelope([parts[1]])
            assert hashlib.sha256(single).hexdigest() == "e3bf5372b15885da138764a25bb2b7cd784ee3cbee1e664937647921ae3122b0"
            assert hashlib.sha256(bz2.decompress(single)).hexdigest() == "d2584c09f1d643d6abff8a25513b63af4e05969c15c30256aed3a05c74281210"
            (out / "frozen.bz2").write_bytes(single)
        corpus += envelope(parts)
    assert hashlib.sha256(corpus).hexdigest() == spec["corpus_sha256"]
    (out / "wikidata.bz2").write_bytes(corpus)
    with (out / "checksums.csv").open("w") as stream:
        writer = csv.writer(stream, lineterminator="\n")
        writer.writerow(("index", "bytes", "raw_sha256"))
        writer.writerows(checksums)


if __name__ == "__main__":
    prepare(Path(sys.argv[1]), Path(sys.argv[2]))
