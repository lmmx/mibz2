"""Rewrap one block of the pinned input, preserving its entropy bits and CRC."""
from hashlib import sha256
from pathlib import Path
import bz2
import sys

data = Path(sys.argv[1]).read_bytes()
assert sha256(data).hexdigest() == "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
ordinal = int(sys.argv[3]) if len(sys.argv) > 3 else 1
bits = bin(int.from_bytes(data, "big"))[2:].zfill(8 * len(data))
block_magic = f"{0x314159265359:048b}"
end_magic = f"{0x177245385090:048b}"
start = -1
for _ in range(ordinal + 1):
    start = bits.index(block_magic, start + 1)
ends = [k for sig in (block_magic, end_magic) if (k := bits.find(sig, start + 48)) >= 0]
assert ends
envelope = "".join(f"{b:08b}" for b in b"BZh9")
envelope += bits[start:min(ends)] + end_magic + bits[start + 48:start + 80]
envelope += "0" * (-len(envelope) % 8)
block = int(envelope, 2).to_bytes(len(envelope) // 8, "big")
raw = bz2.decompress(block)
if ordinal == 1:
    assert start == 336
    assert sha256(block).hexdigest() == "e3bf5372b15885da138764a25bb2b7cd784ee3cbee1e664937647921ae3122b0"
    assert sha256(raw).hexdigest() == "d2584c09f1d643d6abff8a25513b63af4e05969c15c30256aed3a05c74281210"
Path(sys.argv[2]).write_bytes(block)
