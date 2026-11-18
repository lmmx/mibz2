"""Resolve the actual measured buffer alias after a cold-lap fallback."""
from pathlib import Path

def resolve(run):
    f = run/"ranges"
    lines = f.read_text().splitlines()
    if "info direct 0" not in lines:
        return
    phases = {int(x.split(",")[0]) for x in (run/"instructions").read_text().splitlines()}
    if 12 not in phases:
        return
    # Warm finish may have left a copy in scratch.ll. Phase 12 in lap two proves
    # that inversion used the final slot. Preserve measured offsets within it.
    fields = [s.split() for s in lines]
    starts = {a[1]:int(a[2],16) for a in fields if a[0]=="range"}
    delta = starts["final"] - starts["inverse"]
    for a in fields:
        if a[0]=="range" and a[1] in {"entries","front_bytes","bottom"}:
            a[2] = f"{int(a[2],16)+delta:x}"
        if a[:2]==["info","direct"]: a[2] = "1"
    f.write_text("".join(" ".join(a)+"\n" for a in fields))

if __name__=="__main__":
    import sys
    for run in Path(sys.argv[1]).iterdir(): resolve(run)
