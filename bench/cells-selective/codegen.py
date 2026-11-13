"""Archive exact optimized induction bodies and verify untimed route correspondence."""

import hashlib
import json
import re
import subprocess
from pathlib import Path


def main():
    """Extract symbol sizes/disassembly from the two actual measured executables."""
    out = Path("selective-codegen")
    out.mkdir()
    rows = []
    for name, binary in [("original", "bin/original-cells-ab"), ("selective", "bin/cells-ab")]:
        symbols = subprocess.check_output(["nm", "-S", "-C", binary], text=True)
        matches = [
            s
            for s in symbols.splitlines()
            if "mibz2::cells::Kernel" in s and s.endswith("::induce_cells")
        ]
        assert len(matches) == 1, matches
        fields = matches[0].split()
        address, size = int(fields[0], 16), int(fields[1], 16)
        assembly = subprocess.check_output(
            [
                "objdump",
                "-Cd",
                "--no-show-raw-insn",
                f"--start-address={address}",
                f"--stop-address={address + size}",
                binary,
            ],
            text=True,
        )
        (out / f"{name}.asm").write_text(assembly)
        comments = subprocess.check_output(["readelf", "--string-dump=.comment", binary], text=True)
        compiler = re.findall(r"rustc version [^\n]+", comments)
        assert len(compiler) == 1, comments
        rows.append(
            {
                "arm": name,
                "binary_sha256": hashlib.sha256(Path(binary).read_bytes()).hexdigest(),
                "symbol": matches[0],
                "symbol_bytes": size,
                "compiler": compiler[0],
            }
        )
    assert rows[0]["compiler"] == rows[1]["compiler"], "compiler mismatch"
    routes = []
    for name in ["original", "selective"]:
        log = Path(f"{name}-parity.log").read_text()
        found = sorted(re.findall(r"CELLS_ROUTE [^\n]+", log))
        assert len(found) == 7 and "1 passed" in log, (name, found)
        routes.append(found)
    assert routes[0] == routes[1], "admission/refusal/cell counts changed"
    (out / "manifest.json").write_text(
        json.dumps({"binaries": rows, "routes": routes[0]}, indent=2) + "\n"
    )


if __name__ == "__main__":
    main()
