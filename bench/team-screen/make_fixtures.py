"""The team screen's malformed and purpose-built fixtures, made from W and its block map.

docs/journal/2026-11-10-team-screen-design-draft.md. Deterministic: the same W
(`wikidata.bz2`, SHA-256 82f65ace...) and block map (`walk_count spans W`) give the same
bytes. Every edit is located structurally from the block's span, never by searching.

    python3 bench/team-screen/make_fixtures.py W SPANS OUT_DIR [WALK_COUNT]

Python only: the blocks are copied bit for bit and framed here (a header at their level,
their bits, the end marker and their combined CRC), as `walk_count extract` frames them,
so the screen's job needs no build for its fixtures. With WALK_COUNT, made at freeze only,
V-SS's block classes are verified by `walk_count spans`.

writes OUT_DIR/{E-trunc-team,E-trunc-solo,E-crc-team,E-crc-solo,V-SS}.bz2 and, for each
error fixture, OUT_DIR/prefix-NAME.bz2 (the blocks before the failed one, as a stream of
their own, for the oracle's prefix), and prints one JSON line per fixture: its digest, its
edit and its block.
"""

import hashlib
import json
import subprocess
import sys

# (name, edit, block): the blocks are W's, chosen from its frozen class map (T: a team's,
# S: the leader's solo), all in its first stream (level 9), early so that the prefixes are
# short. Block 2 is T, 3 is T, 4 is S between T 3 and T 5, 6 is S between T 5 and T 7.
ERRORS = [
    ("E-trunc-team", "truncate", 2, "T"),
    ("E-crc-team", "crc", 3, "T"),
    ("E-trunc-solo", "truncate", 4, "S"),
    ("E-crc-solo", "crc", 6, "S"),
]
# A valid stream with adjacent S blocks inside one stream (W's S,S pairs all cross a
# stream boundary): W's blocks 3 (T), 4 (S), 6 (S), 7 (T), bits untouched, framed as one
# stream with its combined CRC (`walk_count extract`)
SS = ("V-SS", [3, 4, 6, 7], "TSST")


def sha(b):
    return hashlib.sha256(b).hexdigest()


def extract(w, spans, blocks):
    """W's `blocks` as a stream of their own, bit for bit: `walk_count extract`'s framing"""
    level = spans[blocks[0]]["level"]
    bits = []
    crc = 0
    for i in blocks:
        s = spans[i]
        assert s["level"] == level
        for b in range(s["start_bit"], s["end_bit"]):
            bits.append(w[b // 8] >> (7 - b % 8) & 1)
        c = int(s["crc"], 16)
        crc = ((crc << 1 | crc >> 31) & 0xFFFFFFFF) ^ c
    for v, n in ((0x177245385090, 48), (crc, 32)):
        bits.extend(v >> (n - 1 - k) & 1 for k in range(n))
    bits.extend([0] * (-len(bits) % 8))
    body = bytes(int("".join(map(str, bits[k : k + 8])), 2) for k in range(0, len(bits), 8))
    return b"BZh" + str(level).encode() + body


def main():
    w_path, spans_path, out = sys.argv[1:4]
    walk_count = sys.argv[4] if len(sys.argv) > 4 else None
    w = bytearray(open(w_path, "rb").read())
    spans = [json.loads(line) for line in open(spans_path)]
    for name, edit, block, cls in ERRORS:
        s = spans[block]
        assert s["class"] == cls and s["stream"] == 0, (name, s)
        if edit == "truncate":
            byte = (s["start_bit"] + s["end_bit"]) // 2 // 8
            data = bytes(w[:byte])
            how = {"op": "truncate", "byte": byte}
        else:
            # the stored block CRC is the 32 bits after the 48-bit block magic; its last bit
            bit = s["start_bit"] + 48 + 31
            data = bytearray(w)
            data[bit // 8] ^= 0x80 >> (bit % 8)
            data = bytes(data)
            how = {"op": "xor", "bit": bit, "byte": bit // 8, "mask": 0x80 >> (bit % 8)}
        open(f"{out}/{name}.bz2", "wb").write(data)
        prefix = f"{out}/prefix-{name}.bz2"
        open(prefix, "wb").write(extract(w, spans, list(range(block))))
        print(
            json.dumps(
                dict(
                    id=name,
                    block=block,
                    cls=cls,
                    edit=how,
                    sha256=sha(data),
                    prefix_stream_sha256=sha(open(prefix, "rb").read()),
                )
            )
        )
    name, blocks, classes = SS
    path = f"{out}/{name}.bz2"
    open(path, "wb").write(extract(w, spans, blocks))
    cls = None
    if walk_count:
        got = subprocess.run(
            [walk_count, "spans", path], check=True, capture_output=True, text=True
        )
        cls = "".join(json.loads(line)["class"] for line in got.stdout.splitlines())
        assert cls == classes, (name, cls)
    print(
        json.dumps(
            dict(id=name, from_blocks=blocks, classes=cls, sha256=sha(open(path, "rb").read()))
        )
    )


if __name__ == "__main__":
    main()
