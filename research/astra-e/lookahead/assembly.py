"""Record the baseline and candidate emitter loops in a single release harness."""
import bisect
import re
import subprocess
import sys
from pathlib import Path

binary = Path(sys.argv[1])
def run(*args):
    return subprocess.check_output(args, text=True)

print(run("sha256sum", str(binary)).strip())
print(run("readelf", "-n", str(binary)).split("Build ID: ")[1].splitlines()[0])
decoded = run("objdump", "--dwarf=decodedline", str(binary))
# Locate the first source-line entries of the loop. Use only a narrow fragment of
# the disassembly; absolute PCs vary with compiler versions and the checkout.
line_pcs = sorted(int(m, 16) for m in re.findall(r"\s(0x[0-9a-f]+)\s*(?:x\s*)?$", decoded, re.MULTILINE))
for name, filename, source_line, before, after in (
    ("open", "open_return.rs", 449, 0x40, 0x150),
    ("lookahead", "writer.rs", 22, 0x83, 0xAA),
):
    pattern = rf"^{re.escape(filename)}\s+{source_line}\s+(0x[0-9a-f]+)"
    pc = int(re.search(pattern, decoded, flags=re.MULTILINE).group(1), 16)
    print(name)
    start = line_pcs[bisect.bisect_right(line_pcs, pc-before) - 1]
    end = line_pcs[bisect.bisect_left(line_pcs, pc+after)]
    print(run("objdump", "-Cd", f"--start-address={start}",
              f"--stop-address={end}", str(binary)))
