"""Full-stream byte and CRC controls against Python's independent libbz2."""
import bz2
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile

probe, frozen, wiki, prose = map(Path, sys.argv[1:])
probe = probe.resolve()


def digest(stream):
    h = hashlib.sha256()
    n = 0
    while part := stream.read(1 << 20):
        h.update(part)
        n += len(part)
    return n, h.hexdigest()


with tempfile.TemporaryDirectory() as td:
    out = Path(td) / "out"
    for label, path, arms in [
        ("frozen", frozen, ["16", "q16", "i16", "qi16", "s16", "ranked"]),
        ("wikidata", wiki, ["16", "qi16", "s16", "ranked"]),
        ("prose", prose, ["16", "qi16", "s16", "ranked"]),
    ]:
        with bz2.open(path) as f:
            reference = digest(f)
        for arm in arms:
            subprocess.run([probe, "write", path, arm, out], check=True)
            with out.open("rb") as f:
                assert digest(f) == reference, (label, arm)
        print(label, *reference, ",".join(arms))

    payload = frozen.read_bytes()
    bits = bin(int.from_bytes(payload, "big"))[2:].zfill(8 * len(payload))
    tail = bits.rindex(f"{0x177245385090:048b}")
    for label, bit in [("block CRC", 80), ("stream CRC", tail + 48)]:
        bad = bytearray(payload)
        bad[bit // 8] ^= 128 >> (bit % 8)
        path = Path(td) / "bad.bz2"
        path.write_bytes(bad)
        for arm in ["16", "i16", "qi16", "s16", "ranked"]:
            run = subprocess.run([probe, "write", path, arm, out], capture_output=True)
            assert run.returncode and b"crc" in run.stderr.lower(), (arm, run.stderr)
        print(label, "rejected by all five arms")
