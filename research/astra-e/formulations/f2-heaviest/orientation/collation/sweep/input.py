"""Convert the two original irec run arrays to the byte BWT for an offline sweep."""
import argparse
import hashlib
from pathlib import Path
import sys
root = Path(__file__).resolve().parents[7]
sys.path.insert(0, str(root / "docs/journal/2026-11-18-astra-c-output-first"))
from inputs import read_input

if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("irec", type=Path)
    p.add_argument("column", type=Path)
    a = p.parse_args()
    runs, mark = read_input(a.irec)
    data = b"".join(bytes([c])*length for c, length in runs)
    a.column.write_bytes(data)
    print(f"n={len(data)} mark={mark} tape_sha256={hashlib.sha256(a.irec.read_bytes()).hexdigest()} "
          f"column_sha256={hashlib.sha256(data).hexdigest()}")
