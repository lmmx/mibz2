"""Name immutable MTF lookup storage and the tracer's two pointer stores."""

import subprocess
from pathlib import Path


def static_ranges(probe):
    """The static address ranges of a probe binary's symbols."""
    rows = []
    for s in subprocess.check_output(["nm", "-S", probe], text=True).splitlines():
        a = s.split()
        if len(a) != 4:
            continue
        if "runs5STEPS17h" in a[3] or "mtf5STEPS17h" in a[3]:
            name = "mtf_steps"
        elif "runs11ASTRA_COUNT17h" in a[3] or "runs10ASTRA_ROWS17h" in a[3]:
            name = "probe_metadata"
        else:
            continue
        rows.append(f"range {name} {int(a[0], 16):x} {int(a[1], 16)}\n")
    return "".join(rows)


def append(run, rows):
    """Append the named ranges to a run's ranges file."""
    p = run / "ranges"
    s = p.read_text()
    if "range mtf_steps " not in s:
        p.write_text(s + rows)


if __name__ == "__main__":
    import sys

    rows = static_ranges(sys.argv[1])
    for run in Path(sys.argv[2]).iterdir():
        if (run / "output").is_file():
            append(run, rows)
