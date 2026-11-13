"""Rebuild only the SHA-pinned CI corpus from its archived byte windows.

No network or timing. This deliberately narrow extractor accepts only windows
whose magic count, hashes and final corpus hash match the supplied manifest.
libbzip2 (Python bz2) independently validates the rebuilt streams and output size.
Usage: python3 corpus.py SPEC WINDOW_DIR OUTPUT
"""

import bz2
import hashlib
import json
import re
import sys
from pathlib import Path


def bits(n, width):
    return f"{n:0{width}b}"


def rebuild(spec, root):
    streams = []
    magic = bits(0x314159265359, 48)
    eos = bits(0x177245385090, 48)
    for w in spec["windows"]:
        data = (root / (w["name"] + ".bin")).read_bytes()
        assert hashlib.sha256(data).hexdigest() == w["sha256"], w["name"]
        text = "".join(bits(b, 8) for b in data)
        starts = [m.start() for m in re.finditer(magic, text)]
        # These pinned windows contain consecutive whole blocks and no stream end.
        assert len(starts) == w["blocks"] + 1, w["name"]
        body = text[starts[0] : starts[-1]]
        assert eos not in body, w["name"]
        crc = 0
        for start in starts[:-1]:
            crc = ((crc << 1) | (crc >> 31)) & 0xFFFFFFFF
            crc ^= int(text[start + 48 : start + 80], 2)
        stream = "".join(bits(b, 8) for b in b"BZh9") + body + eos + bits(crc, 32)
        stream += "0" * (-len(stream) % 8)
        encoded = bytes(int(stream[i : i + 8], 2) for i in range(0, len(stream), 8))
        assert len(encoded) == w["stream_bytes"], w["name"]
        assert len(bz2.decompress(encoded)) == w["decoded_bytes"], w["name"]
        streams.append(encoded)
    corpus = b"".join(streams)
    assert len(corpus) == spec["corpus_bytes"]
    assert hashlib.sha256(corpus).hexdigest() == spec["corpus_sha256"]
    return corpus


if __name__ == "__main__":
    spec_path, root, out = map(Path, sys.argv[1:])
    spec = json.loads(spec_path.read_text())
    data = rebuild(spec, root)
    out.write_bytes(data)
    print(json.dumps({"corpus_sha256": hashlib.sha256(data).hexdigest(),
                      "compressed_bytes": len(data), "libbzip2_parity": True}))
