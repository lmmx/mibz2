"""Run the native oracle against sampled archives and a high-entropy run corpus."""
import random
import subprocess
import sys
from pathlib import Path
from screen import read_input, paths

archive, scratch, binary = map(Path, sys.argv[1:])
scratch.mkdir(exist_ok=True)
args = []
for tape in paths(archive):
    runs, p = read_input(tape)
    column = scratch / (tape.stem.split(".")[0] + ".column")
    column.write_bytes(b"".join(bytes([c]) * w for c, w in runs))
    args.append(f"{p}:{column}")
rng = random.Random(0x41E)
for i in range(3):
    out = bytearray()
    while len(out) < 900_000:
        c = rng.randrange(128)
        w = rng.choice([1, 2, 3, 9, 16, 17, 19, 23, 29, 34, 77, 179])
        out += bytes([c]) * w
    column = scratch / f"random-runs-{i}.column"
    column.write_bytes(out[:900_000])
    args.append(f"{rng.randrange(900_000)}:{column}")
subprocess.run([binary.resolve(), *args], check=True)
