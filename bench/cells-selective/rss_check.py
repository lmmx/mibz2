"""Check two native controls, not decoder timing or production-memory qualification."""

import json
import subprocess
from pathlib import Path


def main():
    """Require exact child identity and responsive mutually consistent native peaks."""
    out = Path("rss-controls")
    out.mkdir()
    rows = []
    for mib in [1, 64]:
        with (out / f"{mib}.stderr").open("x") as err:
            subprocess.run(
                ["bin/rss-launcher", str(out / f"{mib}.json"), "bin/rss-control", str(mib)],
                stderr=err,
                check=True,
                timeout=35,
            )
        a = json.loads((out / f"{mib}.json").read_text())
        b = json.loads((out / f"{mib}.stderr").read_text())
        assert a["pid"] == a["waited_pid"] == b["pid"]
        assert a["exit"] == 0 and not a["timeout"]
        peaks = [a["wait4_hwm_kib"], b["self_hwm_kib"], b["proc_hwm_kib"]]
        assert min(peaks) > 0 and max(peaks) - min(peaks) <= 2048, peaks
        rows.append(a | b)
    for key in ["wait4_hwm_kib", "self_hwm_kib", "proc_hwm_kib"]:
        assert 48 * 1024 <= rows[1][key] - rows[0][key] <= 80 * 1024
    (out / "result.json").write_text(json.dumps(rows, indent=2) + "\n")


if __name__ == "__main__":
    main()
