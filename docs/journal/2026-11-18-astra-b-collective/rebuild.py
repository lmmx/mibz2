"""Reassemble the hashed corpus from the 12 pinned windows, leaving their block bits intact."""

from hashlib import sha256
import json
from pathlib import Path
import re
import sys


def pack(bits):
    bits += "0" * (-len(bits) % 8)
    return int(bits, 2).to_bytes(len(bits) // 8, "big")


def main(windows, dest):
    spec = json.loads((windows / "wikidata.json").read_text())
    start, stop = (format(x, "048b") for x in (0x314159265359, 0x177245385090))
    streams = []
    for window in spec["windows"]:
        raw = (windows / (window["name"] + ".bin")).read_bytes()
        assert sha256(raw).hexdigest() == window["sha256"]
        bits = bin(int.from_bytes(raw, "big"))[2:].zfill(8 * len(raw))
        magics = list(re.finditer(start + "|" + stop, bits))
        blocks = [
            bits[m.start() : e.start()] for m, e in zip(magics, magics[1:]) if m.group() == start
        ]
        assert len(blocks) == window["blocks"]
        crc = 0
        for block in blocks:
            crc = ((crc << 1 | crc >> 31) & 0xFFFFFFFF) ^ int(block[48:80], 2)
        stream = b"BZh9" + pack("".join(blocks) + stop + format(crc, "032b"))
        assert len(stream) == window["stream_bytes"]
        streams.append(stream)
    corpus = b"".join(streams)
    assert (
        sha256(corpus).hexdigest()
        == "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
    )
    dest.write_bytes(corpus)


if __name__ == "__main__":
    main(Path(sys.argv[1]), Path(sys.argv[2]))
