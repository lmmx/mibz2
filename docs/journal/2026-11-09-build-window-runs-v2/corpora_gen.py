# the bench corpora `records` and `random` at SCALE 4, as tests/common/mod.rs makes them
import struct, sys
scale = 4
M = (1 << 64) - 1
class Rng:
    def __init__(s, x): s.x = x
    def next(s):
        x = s.x
        x ^= (x << 13) & M; x ^= x >> 7; x ^= (x << 17) & M
        s.x = x; return x
rec = bytearray()
for i in range(600_000 * scale):
    v = (((i * 2654435761) & 0xffffffff) >> 12 | (i << 20)) & 0xffffffff
    rec += struct.pack('<I', v)
open('records', 'wb').write(rec)
r = Rng(0xfeed_beef_dead_cafe)
n = 0
while n < 2_500_000 * scale:
    k = 1 + r.next() % 600
    r.next()
    n += k
open('random', 'wb').write(bytes(r.next() & 0xff for _ in range(2_000_000 * scale)))
