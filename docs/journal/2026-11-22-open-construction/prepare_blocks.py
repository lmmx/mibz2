"""Authenticate the published columns, or make the five-block panel from the corpus check output."""

import hashlib
import json
import re
import sys
from pathlib import Path

mode, directory, *rest = sys.argv[1:]
directory = Path(directory)
if mode == "dense":
    captures = re.findall(r"DENSE (\d+) (\d+)", Path(rest[0]).read_text())
    assert [int(index) for index, _ in captures] == [41, 56, 165, 270, 278]
    blocks = []
    for index, primary in captures:
        name = f"dense-{index}.L"
        column = (directory / name).read_bytes()
        blocks.append(
            dict(
                file=name,
                n=len(column),
                primary_index=int(primary),
                sha256=hashlib.sha256(column).hexdigest(),
            )
        )
    (directory / "manifest.json").write_text(
        json.dumps({"blocks": blocks}, indent=2) + "\n"
    )
elif mode != "auth":
    sys.exit("usage: prepare_blocks.py auth DIRECTORY | dense DIRECTORY PARITY_LOG")

manifest = json.loads((directory / "manifest.json").read_text())
for block in manifest["blocks"]:
    column = (directory / block["file"]).read_bytes()
    assert len(column) == block["n"] and 0 <= block["primary_index"] < len(column)
    assert hashlib.sha256(column).hexdigest() == block["sha256"]
    print(block["file"], len(column), block["primary_index"], block["sha256"])
