"""ib.py THREADS FILE: indexed_bzip2 decoding FILE to stdout, for examples/ab.rs's
`indexed_bzip2` arm (as the bake-off's harness/ib.py: docs/journal/2026-10-27-bz2-bakeoff).

THREADS is its `parallelization` (1: its sequential reader). The decoded bytes are read
in 4 MiB chunks and written to stdout after a first line, "ready SECONDS": the CPU the
interpreter and the import took, which the harness leaves out of the decoder's, starting
its clock there. A decode error exits 2.
"""

import sys
import time

import indexed_bzip2

threads, path = int(sys.argv[1]), sys.argv[2]
out = sys.stdout.buffer
out.write(f"ready {time.process_time():.6f}\n".encode())
out.flush()
try:
    with indexed_bzip2.IndexedBzip2File(path, parallelization=threads) as f:
        while chunk := f.read(4 << 20):
            out.write(chunk)
    out.flush()
except Exception as e:  # noqa: BLE001 (any decode failure is reported the same way)
    print(f"decode error: {type(e).__name__}: {e}", file=sys.stderr)
    sys.exit(2)
