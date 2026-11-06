#!/usr/bin/env python3
"""Read archived Callgrind files only. Usage: python replay.py /path/to/pinned/cg"""
import json
import re
import sys
from pathlib import Path

def read_arcs(path):
    text = path.read_text()
    assert re.search(r"^events:\s*Ir\s*$", text, re.M), path
    names = dict(re.findall(r"(?:^|\n)c?fn=\((\d+)\) ([^\n]+)", text))
    fn = cfn = ""
    pending = False
    arcs = []
    for line in text.splitlines():
        match = re.match(r"^fn=\((\d+)\)", line)
        if match:
            fn = match[1]
            continue
        match = re.match(r"^cfn=\((\d+)\)", line)
        if match:
            cfn = match[1]
            continue
        if line.startswith("calls="):
            pending = True
        elif pending and re.match(r"^\S+\s+\d+", line):
            arcs.append((names[fn], names[cfn], int(line.split()[1])))
            pending = False
    def arc(caller, callee):
        values = [ir for f, t, ir in arcs if caller in f and callee in t]
        return sum(values) if values else None
    result = {
        "measured": arc("c30cg::main", "c30cg::measured"),
        "decode": arc("c30cg::measured", "mibz2::decode_block"),
        "build": arc("mibz2::decode_block", "mibz2::bwt::build"),
        "walk": arc("mibz2::decode_block", "room_segments"),
        "assemble": arc("mibz2::decode_block", "SegScratch>::assemble"),
    }
    assert result["measured"] - result["decode"] == 16, path
    return result

def main():
    root = Path(sys.argv[1])
    recorded = json.loads(Path(__file__).with_name("arithmetic.json").read_text())
    output = []
    for row in recorded["profiles"]:
        path = root / Path(row["path"]).name
        actual = read_arcs(path)
        for field, value in actual.items():
            assert value == row[field], (path, field, value, row[field])
        output.append({"arm": row["arm"], "block": row["block"], **actual})
    print(json.dumps(output, indent=2))

if __name__ == "__main__":
    main()
