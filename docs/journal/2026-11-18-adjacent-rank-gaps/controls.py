"""Build primitive bzip2 test blocks from a five-run family and seeded edits."""

import bz2
import random
import sys
from pathlib import Path

folder = Path(sys.argv[1])
folder.mkdir(parents=True, exist_ok=True)
for copies in (32, 512, 8192, 131072, 218750):
    text = b"abcd" * copies + b"!"
    (folder / f"power-{copies}.bz2").write_bytes(bz2.compress(text, 9))
text = bytearray(b"abcd" * 218750 + b"!")
rng = random.Random(182611)
sites = rng.sample(range(len(text) - 1), 16384)
done = 0
for edits in (1, 4, 16, 64, 256, 1024, 4096, 16384):
    for site in sites[done:edits]:
        text[site] = ord("e") + rng.randrange(16)
    (folder / f"edits-{edits}.bz2").write_bytes(bz2.compress(text, 9))
    done = edits
