import bz2
import hashlib
from pathlib import Path


FROZEN = "e3bf5372b15885da138764a25bb2b7cd784ee3cbee1e664937647921ae3122b0"
RAW = "d2584c09f1d643d6abff8a25513b63af4e05969c15c30256aed3a05c74281210"
PRE = "f87e5caa2d5f0df9d5958fe60901e2023ae04ac406603d4c1584c8bb4017fbd6"


def sha(data):
    return hashlib.sha256(data).hexdigest()


w = Path("w00.bin").read_bytes()
assert sha(w) == "b03bd32b1e03616c73c10e12981a500ec4829f46b1040a48dcd4a55cb1cc7024"
bits = bin(int.from_bytes(w, "big"))[2:].zfill(8*len(w))
block = f"{0x314159265359:048b}"
end = f"{0x177245385090:048b}"
header = "".join(f"{b:08b}" for b in b"BZh9")
prev = bits.index(block)
while True:
    nxt = bits.index(block, prev+48)
    seq = header + bits[prev:nxt] + end + bits[prev+48:prev+80]
    seq += "0" * (-len(seq) % 8)
    compressed = int(seq,2).to_bytes(len(seq)//8, "big")
    if sha(compressed) == FROZEN:
        break
    prev = nxt
raw = bz2.decompress(compressed)
assert sha(raw) == RAW
from itertools import groupby
pre = bytearray()
for b, repeat in groupby(raw):
    count = sum(1 for _ in repeat)
    while count:
        used = min(255,count)
        pre.extend([b]*min(4,used))
        if used>=4:
            pre.append(used-4)
        count-=used
assert sha(pre) == PRE, (len(pre), sha(pre))
Path("frozen.pre").write_bytes(pre)
Path("frozen.bz2").write_bytes(compressed)
print("pinned input", prev, nxt, len(compressed), len(raw), len(pre))
