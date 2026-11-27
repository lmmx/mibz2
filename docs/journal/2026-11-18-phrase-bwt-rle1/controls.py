"""Generate the phrase-alphabet and byte-level controls for the native experiment."""

from pathlib import Path
import bz2
import random
import sys


# Three dictionaries differing only in word length, with random token order.
def make(folder):
    """Write fixed-seed compressed controls with known dictionaries and RLE1 edge cases."""
    folder.mkdir(exist_ok=True)
    for width in (16, 128, 1024):
        rng = random.Random(847321)
        alphabet = b"abcdefghijklmnopqrstuvwxyz0123456789"
        words = [bytes(rng.choices(alphabet, k=width - 1)) + b'"' for _ in range(64)]
        text = b"".join(rng.choices(words, k=(12 * 1024**2) // width))
        (folder / f"lex{width}.bz2").write_bytes(bz2.compress(text, 9))
    rng = random.Random(8491)
    raw = rng.randbytes(900000)
    samples = {
        "random": raw,
        "noquote": b"abcd" * 225000,
        "countequalsquote": (b"a" * 38 + b'x"y"') * 24000,
        "fourquotes": (b'"' * 4 + b'a"*') * 140000,
    }
    for name, data in samples.items():
        (folder / f"{name}.bz2").write_bytes(bz2.compress(data, 9))


if __name__ == "__main__":
    make(Path(sys.argv[1]))
