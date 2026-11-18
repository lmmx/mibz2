"""Split only the SHA-256-pinned concatenation into its 12 original streams and frozen block."""

from pathlib import Path
import bz2
import hashlib
import sys

p = Path(sys.argv[1]).read_bytes()
assert (
    hashlib.sha256(p).hexdigest()
    == "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
)
out = Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)
streams = out / "streams"
streams.mkdir(exist_ok=True)
header = bytes.fromhex("425A6839314159265359")
starts, at = [0], 1
while (k := p.find(header, at)) >= 0:
    starts.append(k)
    at = k + 1
assert len(starts) == 12
starts.append(len(p))
for i, (a, b) in enumerate(zip(starts, starts[1:])):
    (streams / f"w{i:02}.bz2").write_bytes(p[a:b])
# The first large block's published bit offset is 336.
bits = "".join(f"{v:08b}" for v in p[:180000])
magic = f"{0x314159265359:048b}"
s = 336
assert bits[s : s + 48] == magic
e = bits.index(magic, s + 1)
tail = f"{0x177245385090:048b}" + bits[s + 48 : s + 80]
block = "".join(f"{v:08b}" for v in b"BZh9") + bits[s:e] + tail
block += "0" * (-len(block) % 8)
single = int(block, 2).to_bytes(len(block) // 8, "big")
assert (
    hashlib.sha256(bz2.decompress(single)).hexdigest()
    == "d2584c09f1d643d6abff8a25513b63af4e05969c15c30256aed3a05c74281210"
)
(out / "frozen.bz2").write_bytes(single)
