"""bench/thread_split.py on synthetic records: the workers' busy share and the non-workers' CPU
for an occupancy arm (W from its name) and for production (W the count, `--workers mibz2=T`), a
record without thread_cpu left out, and a malformed --workers VOID.

    python3 bench/test_thread_split.py
"""

import io
import json
import shutil
import sys
import tempfile
from contextlib import redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import thread_split as ts  # noqa: E402


def rec(arm, t, rnd, workers, others, unattributed=0.0, wall=1.0):
    tc = " ".join([f"mibz2-{i}={w:.2f}" for i, w in enumerate(workers)] + [f"{n}={v:.2f}" for n, v in others])
    cpu = sum(workers) + sum(v for _, v in others) + unattributed
    return {"corpus": "wikidata-x4", "threads": t, "arm": arm, "round": rnd, "wall": wall, "cpu": cpu,
            "thread_cpu": tc}


def run(rows, extra=()):
    d = Path(tempfile.mkdtemp())
    try:
        with open(d / "judged.jsonl", "w") as f:
            for r in rows:
                f.write(json.dumps(r) + "\n")
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = ts.main(["thread_split.py", str(d / "judged.jsonl"), *extra])
        return code, buf.getvalue()
    finally:
        shutil.rmtree(d)


def main():
    results = []

    def check(name, code, out, want_code, phrases):
        ok = code == want_code and all(p in out for p in phrases)
        results.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {code}, "
              + ", ".join(f"{'found' if p in out else 'missing'} {p!r}" for p in phrases))
        if not ok:
            print(out)

    rows = []
    for rnd in range(3):
        # production at 6 threads: six workers 0.9 s each over a 1 s wall, main 0.6, scan 0.3,
        # 0.1 unattributed
        rows.append(rec("mibz2", 6, rnd, [0.9] * 6, [("main", 0.6), ("mibz2-scan", 0.3)], 0.1))
        # four lazy workers at 6 threads, each 0.95 s; main 0.5, scan 0.2, read 0.2
        rows.append(rec("mibz2-occ-4-16", 6, rnd, [0.95] * 4, [("main", 0.5), ("mibz2-scan", 0.2),
                                                                ("mibz2-read", 0.2)]))
    code, out = run(rows, ("--workers", "mibz2=T"))
    check("production (W the count) and an occupancy arm (W its name)", code, out, 0, [
        "mibz2: busy share a worker 0.900; non-worker CPU / wall 1.000 (named 0.900); worker threads seen 6",
        "mibz2-occ-4-16: busy share a worker 0.950; non-worker CPU / wall 0.900 (named 0.900); worker threads seen 4"])
    code, out = run(rows)
    check("production without its W: no busy share", code, out, 0,
          ["mibz2: busy share a worker n/a (W unknown); non-worker CPU / wall 1.000"])
    code, out = run(rows + [{**rows[0], "round": 9, "thread_cpu": ""}], ("--workers", "mibz2=T"))
    check("a record without thread_cpu left out", code, out, 0, ["1 records left out", "no worker threads"])
    code, out = run(rows, ("--workers", "mibz2=six"))
    check("--workers malformed", code, out, 1, ["VOID", "not ARM=N or ARM=T"])
    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
