"""mkstream.py OUT.bz2 WINDOW... : a valid .bz2 of the whole blocks in byte-range windows
of a .bz2 (one stream per window), each block's encoded bits kept exactly as written.

A window is a raw byte range (curl -r). Every bit offset of the block magic
(0x314159265359) and the end-of-stream magic (0x177245385090) is found; a block runs from
its magic to the next magic of either kind. The blocks of each window are written as one
stream: "BZh9", the blocks' bits, the end-of-stream magic, the combined CRC (each block's
stored CRC, rotated in as bzip2 does) and padding to a byte. The decoders check every
block's CRC, so a false magic match (2^-48 a bit) cannot pass unnoticed.
"""
import sys

BLOCK, EOS = 0x314159265359, 0x177245385090


def magics(data: bytes, magic: int) -> list[int]:
    n = int.from_bytes(data, "big")
    total = len(data) * 8
    pat = magic.to_bytes(6, "big")
    found = []
    for s in range(8):
        # the window shifted left by s bits, so a magic at bit 8k+s is byte-aligned
        b = ((n << s) & ((1 << total) - 1)).to_bytes(len(data), "big")
        i = b.find(pat)
        while i >= 0:
            found.append(8 * i + s)
            i = b.find(pat, i + 1)
    return sorted(found)


def main() -> None:
    out_path, wins = sys.argv[1], sys.argv[2:]
    out = bytearray()
    nblocks = 0
    for w in wins:
        data = open(w, "rb").read()
        total = len(data) * 8
        n = int.from_bytes(data, "big")
        blocks, ends = magics(data, BLOCK), magics(data, EOS)
        marks = sorted([(b, 0) for b in blocks] + [(e, 1) for e in ends])
        bits, nbits, combined, k = 0, 0, 0, 0
        for (a, kind), (b, _) in zip(marks, marks[1:]):
            if kind != 0:
                continue
            seg = (n >> (total - b)) & ((1 << (b - a)) - 1)
            crc = (n >> (total - a - 80)) & 0xFFFFFFFF  # after the 48-bit magic
            combined = ((combined << 1) | (combined >> 31)) & 0xFFFFFFFF ^ crc
            bits = (bits << (b - a)) | seg
            nbits += b - a
            k += 1
        bits = (bits << 80) | (EOS << 32) | combined
        nbits += 80
        pad = (-nbits) % 8
        out += b"BZh9" + (bits << pad).to_bytes((nbits + pad) // 8, "big")
        nblocks += k
        print(f"{w}: {k} blocks, {len(ends)} end markers inside", file=sys.stderr)
    open(out_path, "wb").write(out)
    print(f"{out_path}: {len(wins)} streams, {nblocks} blocks, {len(out)} bytes", file=sys.stderr)


if __name__ == "__main__":
    main()
