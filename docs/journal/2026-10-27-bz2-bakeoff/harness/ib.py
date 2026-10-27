"""ib.py THREADS FILE: indexed_bzip2 (PyPI 1.7.0) decoding FILE to stdout, read in 4 MiB
chunks; THREADS is its `parallelization`. A decode error exits 2."""
import sys

import indexed_bzip2

threads, path = int(sys.argv[1]), sys.argv[2]
out = sys.stdout.buffer
try:
    with indexed_bzip2.IndexedBzip2File(path, parallelization=threads) as f:
        while chunk := f.read(4 << 20):
            out.write(chunk)
    out.flush()
except Exception as e:  # noqa: BLE001 (any decode failure is reported the same way)
    print(f"decode error: {type(e).__name__}: {e}", file=sys.stderr)
    sys.exit(2)
