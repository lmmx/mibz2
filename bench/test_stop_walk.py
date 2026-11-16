"""bench/stop_walk.py on synthetic bundles: each verdict where the registration puts it, the
Holm family's arithmetic, and every defect the validity checks guard against leaves no verdict:
the Researcher's fabricated bundle (REVIEW 6921090225) first among them.

    python3 bench/test_stop_walk.py
"""

import io
import json
import math
import random
import shutil
import sys
import tempfile
from contextlib import redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import stop_walk as sw  # noqa: E402

THREADS = [8, 9, 10, 12, 20]
R = 10


def bundle(main=None, guard=None, mem=1.0, seed=7, noise=0.01):
    """A complete run: `main` and `guard` map a thread count to mibz2-stop15's step over mibz2
    (default level); `mem` its peak memory over mibz2's. Rounds alternate a step's noise above
    and below it, so a level cell's interval holds 1"""
    main, guard = main or {}, guard or {}
    rng = random.Random(seed)
    files = {"judged": [], "forced": [], "overlap": [], "dose": []}
    outs = {sw.MAIN: (838800784, "aa"), sw.GUARD: (61440000, "bb"), sw.ONE: (209700196, "cc")}

    def rec(corpus, arm, t, i, wall, rss, overlap=None):
        r = {"round": i, "corpus": corpus, "threads": t, "arm": arm, "termination": "success",
             "wall": wall, "cpu": wall * max(t, 1) * 0.9, "rss_kb": rss,
             "out": outs[corpus][0], "hash": outs[corpus][1], "cpus": "0-9"}
        if overlap is not None:
            r["held_spinner"] = {"cpu": 19, "overlap": overlap}
        return r

    def wiggle(i, arm):
        return 1 + noise * (1 if (i + len(arm)) % 2 else -1)

    for i in range(R):
        for corpus, steps in ((sw.MAIN, main), (sw.GUARD, guard)):
            for t in THREADS:
                base = 0.5 * (1 + 0.02 * rng.uniform(-1, 1))
                files["judged"].append(rec(corpus, "mibz2", t, i, base, 100000 * t, overlap=False if t <= 9 else None))
                files["judged"].append(rec(corpus, "mibz2-stop15", t, i, base * steps.get(t, 1.0) * wiggle(i, "s"),
                                           100000 * t * mem, overlap=False if t <= 9 else None))
        for t in THREADS:
            base = 0.5 * (1 + 0.02 * rng.uniform(-1, 1))
            for arm in sw.FORCED_ARMS:
                files["forced"].append(rec(sw.MAIN, arm, t, i, base * wiggle(i, arm), 100000 * t))
        base = 0.5 * (1 + 0.02 * rng.uniform(-1, 1))
        for arm in sw.JUDGED_ARMS:
            files["overlap"].append(rec(sw.MAIN, arm, 10, i, base * wiggle(i, arm), 1000000, overlap=True))
        base = 2.0 * (1 + 0.02 * rng.uniform(-1, 1))
        for arm in sw.FORCED_ARMS:
            files["dose"].append(rec(sw.ONE, arm, 1, i, base * wiggle(i, arm), 37000))
            files["dose"].append(rec(sw.ONE, arm, 10, i, base * 1.5 / 9 * wiggle(i, arm), 300000))
    return files


def run(files, rounds=R):
    d = Path(tempfile.mkdtemp())
    try:
        for name, rows in files.items():
            with open(d / f"{name}.jsonl", "w") as f:
                for r in rows:
                    f.write((r if isinstance(r, str) else json.dumps(r)) + "\n")
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = sw.main(["stop_walk.py", str(d), "--rounds", str(rounds)])
        return code, buf.getvalue()
    finally:
        shutil.rmtree(d)


def expect(name, files, code, phrase, rounds=R):
    got, out = run(files, rounds)
    ok = got == code and phrase in out
    print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {got}, {'found' if phrase in out else 'missing'} {phrase!r}")
    if not ok:
        print(out)
    return ok


def main():
    results = []
    # the arithmetic: Student's t against the table, and Holm's step-down
    for df, t in ((9, 2.262), (4, 2.776), (29, 2.045)):
        p = sw.t_cdf(-t, df)
        results.append(abs(p - 0.025) < 5e-4)
        print(f"{'ok  ' if results[-1] else 'FAIL'} t cdf: P(T_{df} <= -{t}) = {p:.5f}, about 0.025")
    h = sw.holm({8: 0.009, 9: 0.011, 10: 0.04, 12: 0.5, 20: 0.9}, 0.05)
    results.append(h == {8, 9})
    print(f"{'ok  ' if results[-1] else 'FAIL'} Holm: rejects {sorted(h)} of 0.009, 0.011, 0.04, 0.5, 0.9 (8 and 9)")
    h = sw.holm({8: 0.011, 9: 0.2, 10: 0.3, 12: 0.4, 20: 0.5}, 0.05)
    results.append(h == set())
    print(f"{'ok  ' if results[-1] else 'FAIL'} Holm: rejects nothing when the least p is above alpha / 5")

    # the verdicts
    results.append(expect("adopts", bundle(main={8: 0.95, 9: 0.95}), 0, "VERDICT: ADOPTS. A gain at [8, 9] threads"))
    results.append(expect("held on memory", bundle(main={8: 0.95}, mem=1.2), 0, "VERDICT: HELD ON MEMORY"))
    results.append(expect("a finding", bundle(main={8: 0.95}, guard={20: 1.05}), 0,
                          "VERDICT: A FINDING, NOT ADOPTED. A gain at [8] threads (Holm), but SLOWER at [('text', 20)]"))
    results.append(expect("not adopted", bundle(), 0, "VERDICT: NOT ADOPTED. No gain established"))
    # a marginal gain at one cell that Holm does not carry: 1% faster against 1% noise
    results.append(expect("not adopted, marginal", bundle(main={8: 0.993}, noise=0.008), 0, "VERDICT: NOT ADOPTED"))

    # the Researcher's fabricated bundle (REVIEW 6921090225): two rounds of lazy and stop15,
    # stop14 and stop16 absent, an empty dose file. As the old reader's two files, and placed
    # as each of the new reader's
    fx = HERE / "stop_walk_fixtures" / "researcher-6920377932"
    runs = (fx / "runs.jsonl").read_text().splitlines()
    dose = (fx / "dose.jsonl").read_text().splitlines()
    results.append(expect("the Researcher's bundle, as judged and dose", {"judged": runs, "dose": dose}, 1, "VOID"))
    results.append(expect("the Researcher's bundle, as forced", {**bundle(), "forced": runs}, 1, "VOID"))
    results.append(expect("the Researcher's bundle, at its own two rounds",
                          {"judged": runs, "forced": runs, "overlap": runs, "dose": dose}, 1, "VOID", rounds=2))

    # each defect, alone, in an otherwise complete adopting bundle
    good = lambda: bundle(main={8: 0.95, 9: 0.95})  # noqa: E731

    def mutate(f, fn):
        b = good()
        fn(b[f])
        return b

    def set_field(rows, i, k, v):
        rows[i] = {**rows[i], k: v}

    cases = [
        ("a record duplicated", mutate("judged", lambda r: r.append(r[3])), "duplicated"),
        ("a record missing", mutate("judged", lambda r: r.pop(5)), "roster keys missing"),
        ("an unexpected arm", mutate("forced", lambda r: set_field(r, 0, "arm", "mibz2-workers-stop17")), "not in the roster"),
        ("a corpus mislabelled", mutate("judged", lambda r: set_field(r, 0, "corpus", "fake")), "not in the roster"),
        ("a failed run", mutate("judged", lambda r: set_field(r, 2, "termination", "killed")), "did not succeed"),
        ("an error", mutate("dose", lambda r: set_field(r, 2, "err", "exit 17")), "did not succeed"),
        ("no termination", mutate("judged", lambda r: r[1].pop("termination")), "did not succeed"),
        ("peak memory missing", mutate("judged", lambda r: r[4].pop("rss_kb")), "rss_kb None"),
        ("peak memory zero", mutate("forced", lambda r: set_field(r, 4, "rss_kb", 0)), "rss_kb 0"),
        ("wall not finite", mutate("judged", lambda r: set_field(r, 6, "wall", math.nan)), "wall nan"),
        ("wall negative", mutate("overlap", lambda r: set_field(r, 1, "wall", -0.5)), "wall -0.5"),
        ("wall a string", mutate("judged", lambda r: set_field(r, 6, "wall", "0.5")), "wall '0.5'"),
        ("no hash", mutate("judged", lambda r: r[7].pop("hash")), "hash None"),
        ("an output differs", mutate("judged", lambda r: set_field(r, 9, "hash", "zz")), "outputs differ"),
        ("an output's size differs", mutate("dose", lambda r: set_field(r, 9, "out", 7)), "outputs differ"),
        ("a dose round unpaired", mutate("dose", lambda r: r.pop(1)), "roster keys missing"),
        ("a judged run beside a shared spinner",
         mutate("judged", lambda r: set_field(r, 0, "held_spinner", {"cpu": 19, "overlap": True})), "spinner sharing"),
        ("a line not JSON", mutate("forced", lambda r: r.append("{not json")), "not JSON"),
    ]
    for name, b, phrase in cases:
        results.append(expect(name, b, 1, phrase))
    b = good()
    del b["overlap"]
    results.append(expect("a file missing", b, 1, "overlap.jsonl: missing"))

    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
