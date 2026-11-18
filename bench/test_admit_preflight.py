"""bench/admit_preflight.py on fabricated pre-flights: the registered one passes, and each way
the machine, the plans, the kernels, the census or the guards' digests can differ stops the run.

    python3 bench/test_admit_preflight.py
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
import admit_preflight as pf  # noqa: E402


def built():
    return [{"arm": arm, "threads": t, "workers": pf.PLANS[t][0], "window": pf.PLANS[t][1], "kernel": k,
             "lazy": arm == "mibz2", "measured_i9": True, "cpus_allowed": "0-19"}
            for arm, k in pf.KERNELS.items() for t in pf.PLANS]


def census():
    return [{"file": "wikidata.bz2", "blocks": 431, "column": 48, "both": 196, "wide_only": 32, "lazy_only": 0,
             "neither": 155, "differ": 0},
            {"file": "text.bz2", "blocks": 49, "column": 0, "both": 0, "wide_only": 0, "lazy_only": 0, "neither": 49,
             "differ": 0},
            {"file": "random.bz2", "blocks": 36, "column": 36, "both": 0, "wide_only": 0, "lazy_only": 0,
             "neither": 0, "differ": 0}]


def inputs():
    return [f"{d}  /w/target/admit/run-x/gen/{f}" for f, d in pf.DIGESTS.items()] + ["abc  /w/wikidata-x4.bz2"]


def run(b, c, i):
    d = Path(tempfile.mkdtemp())
    try:
        (d / "built.jsonl").write_text("".join(json.dumps(r) + "\n" for r in b))
        (d / "census.jsonl").write_text("".join(json.dumps(r) + "\n" for r in c))
        (d / "inputs.sha256").write_text("\n".join(i) + "\n")
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = pf.main(["admit_preflight.py", "--built", str(d / "built.jsonl"), "--census",
                            str(d / "census.jsonl"), "--inputs", str(d / "inputs.sha256")])
        return code, buf.getvalue()
    finally:
        shutil.rmtree(d)


def edit(rows, i, **kw):
    rows = [dict(r) for r in rows]
    rows[i].update(kw)
    return rows


def main():
    cases = [
        ("the registered pre-flight", built(), census(), inputs(), 0, "pre-flight: the measured i9"),
        ("not the measured i9", edit(built(), 0, measured_i9=False), census(), inputs(), 1, "not the measured i9"),
        ("a pinned affinity", edit(built(), 1, cpus_allowed="0-9"), census(), inputs(), 1, "the affinity '0-9'"),
        ("ten workers at 10 threads (the trim declined)", edit(built(), 1, workers=10, window=40), census(), inputs(),
         1, "10 workers and 40 in flight, not 7 and 28"),
        ("the wide arm's plan not production's", edit(built(), 3, workers=8, window=32), census(), inputs(), 1,
         "mibz2-wide at 10: 8 workers"),
        ("the wrong kernel", edit(built(), 2, kernel="mibz2::cells::CellsVariant<1>"), census(), inputs(), 1,
         "the kernel"),
        ("an arm's count missing", built()[:3], census(), inputs(), 1, "mibz2-wide at 10 threads missing"),
        ("the Wikidata census moved", built(), edit(census(), 0, wide_only=31), inputs(), 1,
         "wikidata.bz2 wide_only 31, not 32"),
        ("a text block admitted", built(), edit(census(), 1, wide_only=1), inputs(), 1, "text.bz2 wide_only 1"),
        ("outputs differing", built(), edit(census(), 0, differ=1), inputs(), 1, "differ 1, not 0"),
        ("a census file missing", built(), census()[:2], inputs(), 1, "random.bz2 missing"),
        ("a stale text corpus", built(), census(), ["0" * 64 + "  /w/gen/text.bz2"] + inputs()[1:], 1,
         "text.bz2 '000"),
        ("the guards' digests missing", built(), census(), inputs()[2:], 1, "random.bz2 None"),
    ]
    results = []
    for name, b, c, i, code, phrase in cases:
        got, out = run(b, c, i)
        ok = got == code and phrase in out
        results.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {got}, {'found' if phrase in out else 'missing'} {phrase!r}")
        if not ok:
            print(out)
    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
