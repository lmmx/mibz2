"""Valid streams spanning the width cutoff, shared wide/narrow reuse, and mutations."""
import bz2
from pathlib import Path
import random
import sys

d=Path(sys.argv[1])
d.mkdir(parents=True,exist_ok=True)
streams=[]
for count in [2,17,128,253,254,255,256]:
    rng=random.Random(1201+count)
    # Long blocks with rare full alphabets are checked on both MTF routes.
    plain=bytes(range(count))*328+bytes(rng.randrange(count) for _ in range(8081))
    enc=bz2.compress(plain,compresslevel=1)
    assert bz2.decompress(enc)==plain
    (d/f"a{count}.bz2").write_bytes(enc)
    (d/f"a{count}.raw").write_bytes(plain)
    streams.append((enc,plain))
# Multiple blocks in a single stream reuse the exact carrier across alphabet changes.
widths=[254,256,17,255,253,256,254,255]
plain=b"".join((bytes(range(w))*(100000//w+1))[:100000] for w in widths)
enc=bz2.compress(plain,compresslevel=1)
(d/"alternate.bz2").write_bytes(enc)
(d/"alternate.raw").write_bytes(bz2.decompress(enc))
# Concatenated fresh stream headers and noninitial combined CRC errors.
for suffix,seq in [("both",[6,4,5,1,3,6,4]),("short",[0,5,1])]:
    enc=b"".join(streams[i][0] for i in seq)
    (d/f"concat-{suffix}.bz2").write_bytes(enc)
    (d/f"concat-{suffix}.raw").write_bytes(bz2.decompress(enc))
for i in range(192):
    rng=random.Random(91087+i)
    enc=bytearray(streams[i%len(streams)][0])
    if i%4==0:
        del enc[rng.randrange(0,len(enc)):]
    elif i%4==1:
        # Stored block CRC is byte-aligned after the first ten bytes.
        enc[10+i%4]^=1<<(i%8)
    else:
        idx=rng.randrange(len(enc));enc[idx]^=1<<rng.randrange(8)
    (d/f"mut-{i:03}.bz2").write_bytes(enc)
# The first single-block stream is valid. Corrupt only the combined checksum
# after the last stream's end marker, including the non-byte-aligned case.
tail=bytearray(streams[4][0])
bits=''.join(f'{b:08b}' for b in tail)
mark=f'{0x177245385090:048b}'
at=bits.rindex(mark)+48+9
tail[at//8]^=1<<(7-at%8)
broken=streams[6][0]+tail
try:
    bz2.BZ2Decompressor().decompress(tail)
except OSError:
    pass
else:
    raise AssertionError('the reference accepted the corrupt combined CRC')
(d/'bad-combined.bz2').write_bytes(broken)
