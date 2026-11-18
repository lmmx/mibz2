"""Independent full-stream byte comparison and bzip2 error propagation."""
import bz2
import hashlib
import subprocess
import sys
import tempfile
from pathlib import Path


def compare(reference, decoded):
    total = 0
    digest = hashlib.sha256()
    while part := reference.read(1 << 20):
        assert decoded.read(len(part)) == part
        digest.update(part)
        total += len(part)
    assert not decoded.read(1)
    return total, digest.hexdigest()


def verify(probe, frozen, wiki, prose, arm="switches"):
    with tempfile.TemporaryDirectory() as td:
        temp = Path(td)
        out = temp / "out"
        samples = dict(frozen=frozen, wikidata=wiki, prose=prose)
        # Includes the empty stream, periodic input, a long final RLE1 expansion
        # and concatenated streams with different block size settings.
        mixed = temp / "mixed.bz2"
        mixed.write_bytes(bz2.compress(b"") + bz2.compress(b"abc" * 45000, 1)
                          + bz2.compress(b"x" * 340000 + bytes(range(256)) * 30, 9))
        samples["mixed"] = mixed
        for label, path in samples.items():
            subprocess.run([probe, "write", path, arm, out], check=True)
            with bz2.open(path) as ref, out.open("rb") as decoded:
                print(arm, label, *compare(ref, decoded), flush=True)
        payload = Path(frozen).read_bytes()
        bits = bin(int.from_bytes(payload, "big"))[2:].zfill(8 * len(payload))
        end = bits.rindex(f"{0x177245385090:048b}")
        for label, bit in [("block CRC", 80), ("combined CRC", end + 48)]:
            bad = bytearray(payload)
            bad[bit // 8] ^= 128 >> (bit % 8)
            path = temp / "bad.bz2"
            path.write_bytes(bad)
            r = subprocess.run([probe, "write", path, arm, out], capture_output=True)
            assert r.returncode and b"crc" in r.stderr.lower(), (label, r.stderr)
            print(arm, label, "rejected")


if __name__ == "__main__":
    verify(*sys.argv[1:])
