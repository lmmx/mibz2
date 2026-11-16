"""bench/two_arm.py on synthetic runs: each verdict where its rule puts it, Holm's arithmetic, a
moved control cell, and every defect the validity checks guard against leaves no verdict: the
Researcher's fabricated bundle (#145 REVIEW 6921090225) among them.

    python3 bench/test_two_arm.py
"""

import io
import json
import math
import shutil
import sys
import tempfile
from contextlib import redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import two_arm as ta  # noqa: E402

JUDGED, CONTROLS, R = [12, 16, 20], [9, 10], 10
REF_EXE = "/w/target/cores-less-one/build-master/target/release/examples/ab"
ARGS = ["--ref", "mibz2@build-master", "--cand", "mibz2", "--main", "wikidata-x4", "--guard", "text",
        "--judged", "12,16,20", "--controls", "9,10"]


def runs(main=None, guard=None, mem=1.0, noise=0.01, seed=7):
    """A complete run: `main` and `guard` map a thread count to the candidate's step over the
    reference (default level); `mem` its peak memory over the reference's. Rounds alternate
    a step's noise above and below it, so a level cell's interval holds 1"""
    import random
    main, guard = main or {}, guard or {}
    rng = random.Random(seed)
    outs = {"wikidata-x4": (838800784, "aa"), "text": (49067679, "bb")}
    rows = []
    for i in range(R):
        for corpus, steps in (("wikidata-x4", main), ("text", guard)):
            for t in JUDGED + CONTROLS:
                base = 0.5 * (1 + 0.02 * rng.uniform(-1, 1))
                wig = 1 + noise * (1 if i % 2 else -1)
                for arm, wall, rss in ((f"mibz2@{REF_EXE}", base, 100000 * t),
                                       ("mibz2", base * steps.get(t, 1.0) * wig, 100000 * t * (mem if t in JUDGED else 1.0))):
                    r = {"round": i, "corpus": corpus, "threads": t, "arm": arm, "termination": "success",
                         "wall": wall, "cpu": wall * t * 0.9, "rss_kb": rss, "out": outs[corpus][0],
                         "hash": outs[corpus][1], "cpus": "0-9"}
                    if t <= 9:
                        r["held_spinner"] = {"cpu": 19, "overlap": False}
                    rows.append(r)
    return rows


def run(rows, extra=()):
    d = Path(tempfile.mkdtemp())
    try:
        with open(d / "judged.jsonl", "w") as f:
            for r in rows:
                f.write((r if isinstance(r, str) else json.dumps(r)) + "\n")
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = ta.main(["two_arm.py", str(d / "judged.jsonl"), *ARGS, *extra])
        return code, buf.getvalue()
    finally:
        shutil.rmtree(d)


def expect(name, rows, code, phrase, extra=()):
    got, out = run(rows, extra)
    ok = got == code and phrase in out
    print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {got}, {'found' if phrase in out else 'missing'} {phrase!r}")
    if not ok:
        print(out)
    return ok


def main():
    results = []
    for df, t in ((9, 2.262), (4, 2.776), (29, 2.045)):
        p = ta.t_cdf(-t, df)
        results.append(abs(p - 0.025) < 5e-4)
        print(f"{'ok  ' if results[-1] else 'FAIL'} t cdf: P(T_{df} <= -{t}) = {p:.5f}, about 0.025")
    h = ta.holm({12: 0.009, 16: 0.02, 20: 0.5}, 0.05)
    results.append(h == {12, 16})
    print(f"{'ok  ' if results[-1] else 'FAIL'} Holm: rejects {sorted(h)} of 0.009, 0.02, 0.5 (12 and 16)")
    results.append(ta.canonical(f"mibz2@{REF_EXE}") == "mibz2@build-master" and ta.canonical("mibz2") == "mibz2")
    print(f"{'ok  ' if results[-1] else 'FAIL'} canonical arms: {ta.canonical(f'mibz2@{REF_EXE}')}, mibz2")

    results.append(expect("adopts", runs(main={12: 0.96, 20: 0.96}), 0, "VERDICT: ADOPTS. A gain at [12, 20] threads"))
    results.append(expect("held on memory", runs(main={12: 0.96}, mem=1.2), 0, "VERDICT: HELD ON MEMORY"))
    results.append(expect("a finding", runs(main={12: 0.96}, guard={20: 1.05}), 0,
                          "VERDICT: A FINDING, NOT ADOPTED. A gain at [12] threads (Holm), but SLOWER at [('text', 20)]"))
    results.append(expect("not adopted", runs(), 0, "VERDICT: NOT ADOPTED. No gain established"))
    results.append(expect("a control moved", runs(main={12: 0.96, 10: 0.97}), 1, "control cells moved [('wikidata-x4', 10)]"))
    results.append(expect("a guard control moved", runs(main={12: 0.96}, guard={9: 1.04}), 1, "control cells moved [('text', 9)]"))

    fx = HERE / "two_arm_fixtures" / "researcher-6920377932" / "runs.jsonl"
    results.append(expect("the Researcher's bundle", fx.read_text().splitlines(), 1, "VOID"))
    results.append(expect("the Researcher's bundle, at its own two rounds", fx.read_text().splitlines(), 1, "VOID",
                          extra=("--rounds", "2")))

    good = lambda: runs(main={12: 0.96})  # noqa: E731

    def mutate(fn):
        rows = good()
        fn(rows)
        return rows

    def set_field(rows, i, k, v):
        rows[i] = {**rows[i], k: v}

    cases = [
        ("a record duplicated", mutate(lambda r: r.append(r[3])), "duplicated"),
        ("a record missing", mutate(lambda r: r.pop(5)), "roster keys missing"),
        ("an unexpected arm", mutate(lambda r: set_field(r, 1, "arm", "mibz2-workers")), "not in the roster"),
        ("a corpus mislabelled", mutate(lambda r: set_field(r, 0, "corpus", "fake")), "not in the roster"),
        ("a failed run", mutate(lambda r: set_field(r, 2, "termination", "killed")), "did not succeed"),
        ("an error", mutate(lambda r: set_field(r, 2, "err", "exit 17")), "did not succeed"),
        ("peak memory missing", mutate(lambda r: r[4].pop("rss_kb")), "rss_kb None"),
        ("wall not finite", mutate(lambda r: set_field(r, 6, "wall", math.nan)), "wall nan"),
        ("wall a string", mutate(lambda r: set_field(r, 6, "wall", "0.5")), "wall '0.5'"),
        ("no hash", mutate(lambda r: r[7].pop("hash")), "hash None"),
        ("an output differs", mutate(lambda r: set_field(r, 9, "hash", "zz")), "outputs differ"),
        ("a run beside a shared spinner",
         mutate(lambda r: set_field(r, 0, "held_spinner", {"cpu": 19, "overlap": True})), "spinner sharing"),
        ("a line not JSON", mutate(lambda r: r.append("{not json")), "not JSON"),
    ]
    for name, rows, phrase in cases:
        results.append(expect(name, rows, 1, phrase))

    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
