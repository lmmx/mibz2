"""Generate only frozen pattern/onebyte inputs; no decode or timing.

Wikidata must be supplied with driver.DIGESTS identity; reuse ../corpus.py and
its pinned windows only if reconstruction is needed for a later authorised run.
"""

import bz2
import sys
from pathlib import Path

from driver import DIGESTS, digest


def generate(directory):
    directory.mkdir()
    for name, raw in (("pattern", b"0123456" * 2_800_000), ("onebyte", b"a" * 20_000_000)):
        path = directory / f"{name}.bz2"
        path.write_bytes(bz2.compress(raw, compresslevel=9))
        assert digest(path) == DIGESTS[name], "frozen compressed identity mismatch"


if __name__ == "__main__":
    generate(Path(sys.argv[1]))
