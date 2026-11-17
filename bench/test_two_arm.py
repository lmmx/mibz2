"""bench/two_arm.py on synthetic runs: each verdict where its rule puts it, Holm's arithmetic, a
moved control cell, and every defect the validity checks guard against leaves no verdict: the
Researcher's fabricated bundle (#145 REVIEW 6921090225) among them, and a fabricated bundle for
each missing or malformed piece of the machine's and the runs' metadata (REVIEW 6924511372).

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
# the i9-10900K as `ab env` records it: 10 cores, 20 CPUs, CPU c and c + 10 on core c
ENV = {"cpus": 20, "cores": 10, "core_cpus": [f"{c},{c + 10}" for c in range(10)], "cpu_quota": None,
       "throttling": False}


def pinned(t):
    """`ab run --spread`'s CPUs at t threads on the i9: the first of each of the first t cores"""
    return ",".join(str(c) for c in range(t)) if t <= 10 else "all"
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
                         "hash": outs[corpus][1], "cpus": pinned(t)}
                    if t <= 9:
                        r["held_spinner"] = {"cpu": 19, "auto": True, "overlap": False,
                                             "decoder_cpus": pinned(t), "owner": "run", "cpu_s": 2.0}
                    rows.append(r)
    return rows


def run(rows, extra=(), env=ENV):
    d = Path(tempfile.mkdtemp())
    try:
        with open(d / "judged.jsonl", "w") as f:
            for r in rows:
                f.write((r if isinstance(r, str) else json.dumps(r)) + "\n")
        if env is not None:
            (d / "env.json").write_text(env if isinstance(env, str) else json.dumps(env))
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = ta.main(["two_arm.py", str(d / "judged.jsonl"), *ARGS, "--env", str(d / "env.json"), *extra])
        return code, buf.getvalue()
    finally:
        shutil.rmtree(d)


def expect(name, rows, code, phrase, extra=(), env=ENV):
    got, out = run(rows, extra, env)
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

    # the machine's and the runs' metadata (REVIEW 6924511372): each missing or malformed
    # piece, alone, leaves no verdict
    def at(rows, t):
        return next(i for i, r in enumerate(rows) if isinstance(r, dict) and r["threads"] == t)

    def spin(rows, t, **kw):
        i = at(rows, t)
        hs = {k: v for k, v in {**rows[i]["held_spinner"], **kw}.items() if v is not ...}
        rows[i] = {**rows[i], "held_spinner": hs}

    def drop(rows, t, k):
        i = at(rows, t)
        rows[i] = {k2: v for k2, v in rows[i].items() if k2 != k}

    meta = [
        ("no spinner where one was held", mutate(lambda r: drop(r, 9, "held_spinner")), "no spinner recorded"),
        ("a spinner not an object", mutate(lambda r: set_field(r, at(r, 9), "held_spinner", True)),
         "no spinner recorded"),
        ("the spinner's CPU missing", mutate(lambda r: spin(r, 9, cpu=...)), "the spinner's CPU None malformed"),
        ("the spinner's CPU a string", mutate(lambda r: spin(r, 9, cpu="19")), "the spinner's CPU '19' malformed"),
        ("the spinner's CPU off the machine", mutate(lambda r: spin(r, 9, cpu=40)), "the spinner's CPU 40 malformed"),
        ("the overlap flag missing", mutate(lambda r: spin(r, 9, overlap=...)), "the spinner's overlap None malformed"),
        ("the overlap flag a string", mutate(lambda r: spin(r, 9, overlap="false")),
         "the spinner's overlap 'false' malformed"),
        ("the owner missing", mutate(lambda r: spin(r, 9, owner=...)), "the spinner's owner None"),
        ("the decoder CPUs not the run's", mutate(lambda r: spin(r, 9, decoder_cpus="0,1")),
         "are not the run's"),
        ("a spinner on a decoder core, flagged clear",
         mutate(lambda r: spin(r, 9, cpu=10)), "ran beside a spinner sharing a decoder core (CPU 10)"),
        ("a spinner past the hold limit",
         mutate(lambda r: set_field(r, at(r, 10), "held_spinner", {"cpu": 19, "overlap": False})),
         "past the hold limit"),
        ("the run's CPUs missing", mutate(lambda r: drop(r, 10, "cpus")), "its CPUs None missing or malformed"),
        ("the run's CPUs a range", mutate(lambda r: set_field(r, at(r, 10), "cpus", "0-9")),
         "its CPUs '0-9' missing or malformed"),
        ("two of the run's CPUs on one core",
         mutate(lambda r: set_field(r, at(r, 10), "cpus", "0,1,2,3,4,5,6,7,8,18")), "are not one on each of 10 cores"),
        ("too few of the run's CPUs", mutate(lambda r: set_field(r, at(r, 10), "cpus", "0,1,2")),
         "are not one on each of 10 cores"),
        ("pinned past the cores", mutate(lambda r: set_field(r, at(r, 12), "cpus", "0,1,2,3,4,5,6,7,8,9")),
         "past the 10 cores, not unpinned"),
    ]
    for name, rows, phrase in meta:
        results.append(expect(name, rows, 1, phrase))
    envs = [
        ("no machine record", None, "no machine record"),
        ("a machine record not JSON", "{cores", "no machine record"),
        ("nine cores", {**ENV, "cores": 9}, "cores 9, not 10"),
        ("ten CPUs (no SMT)", {**ENV, "cpus": 10}, "cpus 10, not 20"),
        ("a CPU quota", {**ENV, "cpu_quota": 8.0}, "cpu_quota 8.0, not none"),
        ("the quota unrecorded", {k: v for k, v in ENV.items() if k != "cpu_quota"}, "cpu_quota 'missing'"),
        ("the cores' CPUs missing", {k: v for k, v in ENV.items() if k != "core_cpus"}, "core_cpus None"),
        ("a core's CPUs malformed", {**ENV, "core_cpus": ["0-10"] + ENV["core_cpus"][1:]}, "malformed"),
        ("a CPU on two cores", {**ENV, "core_cpus": ["0,10", "0,11"] + ENV["core_cpus"][2:]}, "CPU 0 on two cores"),
    ]
    for name, env, phrase in envs:
        results.append(expect(name, good(), 1, phrase, env=env))
    unheld = [{k: v for k, v in r.items() if k != "held_spinner"} for r in good()]
    results.append(expect("no spinner held, as registered", unheld, 0, "VERDICT: ADOPTS",
                          extra=("--hold-max-threads", "0")))

    # several candidates against one reference (`just bench-workers-window`): Holm over all their
    # main-corpus cells, a verdict each, and the choice among those that adopt
    cands = ["mibz2-occ-8-32", "mibz2-occ-9-72", "mibz2-occ-9-18"]
    occ_args = ["--ref", "mibz2", *sum((["--cand", c] for c in cands), []), "--main", "wikidata-x4",
                "--guard", "text", "--judged", "12,16,20", "--controls", "", "--hold-max-threads", "9"]

    def occ(steps=None, guard=None, drop=None):
        """occupancy-shaped runs: `steps` maps a candidate to its main-corpus step at every
        judged count (default level), `guard` a candidate to its text step"""
        import random
        steps, guard = steps or {}, guard or {}
        rng = random.Random(11)
        rows = []
        for i in range(R):
            for corpus, out in (("wikidata-x4", (838800784, "aa")), ("text", (49067679, "bb"))):
                for t in (12, 16, 20):
                    base = 0.5 * (1 + 0.02 * rng.uniform(-1, 1))
                    wig = 1 + 0.01 * (1 if i % 2 else -1)
                    for arm in ["mibz2", *cands]:
                        if arm == drop:
                            continue
                        f = (steps if corpus == "wikidata-x4" else guard).get(arm, 1.0)
                        wall = base * (f * wig if arm != "mibz2" else 1.0)
                        rows.append({"round": i, "corpus": corpus, "threads": t, "arm": arm,
                                     "termination": "success", "wall": wall, "cpu": wall * t * 0.9,
                                     "rss_kb": 100000 * t, "out": out[0], "hash": out[1], "cpus": "all"})
        return rows

    def run_occ(rows):
        d = Path(tempfile.mkdtemp())
        try:
            with open(d / "judged.jsonl", "w") as f:
                for r in rows:
                    f.write(json.dumps(r) + "\n")
            (d / "env.json").write_text(json.dumps(ENV))
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = ta.main(["two_arm.py", str(d / "judged.jsonl"), *occ_args, "--env", str(d / "env.json")])
            return code, buf.getvalue()
        finally:
            shutil.rmtree(d)

    def expect_occ(name, rows, code, phrases):
        got, out = run_occ(rows)
        ok = got == code and all(p in out for p in phrases)
        print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {got}, "
              + ", ".join(f"{'found' if p in out else 'missing'} {p!r}" for p in phrases))
        if not ok:
            print(out)
        return ok

    results.append(expect_occ("several: one adopts", occ({"mibz2-occ-9-18": 0.96}), 0, [
        "VERDICT for mibz2-occ-9-18: ADOPTS", "VERDICT for mibz2-occ-8-32: NOT ADOPTED",
        "VERDICT for mibz2-occ-9-72: NOT ADOPTED", "THE CHOICE: mibz2-occ-9-18"]))
    results.append(expect_occ("several: two adopt, the faster chosen",
                              occ({"mibz2-occ-8-32": 0.97, "mibz2-occ-9-72": 0.95}), 0, [
        "VERDICT for mibz2-occ-8-32: ADOPTS", "VERDICT for mibz2-occ-9-72: ADOPTS",
        "THE CHOICE: mibz2-occ-9-72, of those that ADOPT (mibz2-occ-8-32, mibz2-occ-9-72)"]))
    results.append(expect_occ("several: none", occ(), 0, ["THE CHOICE: none; the reference stays"]))
    results.append(expect_occ("several: a gain, its guard SLOWER",
                              occ({"mibz2-occ-8-32": 0.96}, guard={"mibz2-occ-8-32": 1.05}), 0, [
        "VERDICT for mibz2-occ-8-32: A FINDING, NOT ADOPTED", "THE CHOICE: none"]))
    results.append(expect_occ("several: one candidate's records missing", occ({"mibz2-occ-9-18": 0.96},
                              drop="mibz2-occ-9-72"), 1, ["VOID", "roster keys missing"]))

    # another arm as the reference, and a context arm (`just bench-workers-low`): 8 workers with
    # 32 in flight the reference, 7/28 and 6/24 the candidates, production's `mibz2` (9/36) run
    # and checked with them, its steps reported, not judged
    low_ref, low_cands, low_ctx = "mibz2-occ-8-32", ["mibz2-occ-7-28", "mibz2-occ-6-24"], "mibz2"

    def low(steps=None, guard=None, mem=None, drop=None, ctx=True):
        """bench-workers-low-shaped runs: `steps`, `guard` and `mem` map an arm (a candidate or
        the context arm) to its main-corpus step, its text step and its peak memory over the
        reference's (default level); `drop` an arm left out of the records"""
        import random
        steps, guard, mem = steps or {}, guard or {}, mem or {}
        rng = random.Random(13)
        rows = []
        for i in range(R):
            for corpus, out in (("wikidata-x4", (838800784, "aa")), ("text", (49067679, "bb"))):
                for t in (12, 16, 20):
                    base = 0.5 * (1 + 0.02 * rng.uniform(-1, 1))
                    wig = 1 + 0.01 * (1 if i % 2 else -1)
                    for arm in [low_ref, *low_cands, *([low_ctx] if ctx else [])]:
                        if arm == drop:
                            continue
                        f = (steps if corpus == "wikidata-x4" else guard).get(arm, 1.0)
                        wall = base * (f * wig if arm != low_ref else 1.0)
                        rows.append({"round": i, "corpus": corpus, "threads": t, "arm": arm,
                                     "termination": "success", "wall": wall, "cpu": wall * t * 0.9,
                                     "rss_kb": 100000 * t * mem.get(arm, 1.0), "out": out[0], "hash": out[1],
                                     "cpus": "all"})
        return rows

    def low_args(context=(low_ctx,)):
        return ["--ref", low_ref, *sum((["--cand", c] for c in low_cands), []),
                *sum((["--context", c] for c in context), []), "--main", "wikidata-x4", "--guard", "text",
                "--judged", "12,16,20", "--controls", "", "--hold-max-threads", "0"]

    def expect_low(name, rows, code, phrases, args=None, absent=()):
        d = Path(tempfile.mkdtemp())
        try:
            with open(d / "judged.jsonl", "w") as f:
                for r in rows:
                    f.write(json.dumps(r) + "\n")
            (d / "env.json").write_text(json.dumps(ENV))
            buf = io.StringIO()
            with redirect_stdout(buf):
                got = ta.main(["two_arm.py", str(d / "judged.jsonl"), *(low_args() if args is None else args),
                               "--env", str(d / "env.json")])
            out = buf.getvalue()
        finally:
            shutil.rmtree(d)
        ok = got == code and all(p in out for p in phrases) and not any(p in out for p in absent)
        print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {got}, "
              + ", ".join([f"{'found' if p in out else 'missing'} {p!r}" for p in phrases]
                          + [f"{'present' if p in out else 'absent'} {p!r}" for p in absent]))
        if not ok:
            print(out)
        return ok

    results.append(expect_low("another reference: one adopts, the context arm reported",
                              low({"mibz2-occ-7-28": 0.96, "mibz2": 1.05}), 0, [
        "Context, reported and not judged: mibz2 over mibz2-occ-8-32",
        "VERDICT for mibz2-occ-7-28: ADOPTS", "VERDICT for mibz2-occ-6-24: NOT ADOPTED",
        "THE CHOICE: mibz2-occ-7-28", "over the 6 main-corpus cells"],
        absent=["VERDICT for mibz2:", "mibz2 at 12 p"]))
    results.append(expect_low("another reference: none adopts, the reference stays",
                              low({"mibz2-occ-6-24": 1.08}), 0, [
        "VERDICT for mibz2-occ-6-24: NOT ADOPTED", "THE CHOICE: none; the reference stays"]))
    results.append(expect_low("a context arm SLOWER and over the memory gate judges nothing",
                              low({"mibz2-occ-6-24": 0.96}, guard={"mibz2": 1.05}, mem={"mibz2": 1.3}), 0, [
        "VERDICT for mibz2-occ-6-24: ADOPTS", "THE CHOICE: mibz2-occ-6-24", "SLOWER; memory 1.300"]))
    results.append(expect_low("a candidate over the memory gate against the new reference",
                              low({"mibz2-occ-7-28": 0.96}, mem={"mibz2-occ-7-28": 1.2}), 0, [
        "VERDICT for mibz2-occ-7-28: HELD ON MEMORY", "THE CHOICE: none"]))
    results.append(expect_low("the context arm's records missing", low({"mibz2-occ-7-28": 0.96}, drop="mibz2"), 1,
                              ["VOID", "roster keys missing", "'mibz2'"]))
    results.append(expect_low("a context arm run but not named", low({"mibz2-occ-7-28": 0.96}), 1,
                              ["VOID", "not in the roster"], args=low_args(context=())))
    results.append(expect_low("the context arm the reference", low(ctx=False), 1,
                              ["VOID", "two arms are one"], args=low_args(context=(low_ref,))))
    results.append(expect_low("the context arm a candidate", low(ctx=False), 1,
                              ["VOID", "two arms are one"], args=low_args(context=("mibz2-occ-7-28",))))
    results.append(expect_low("the context arm's pinning checked", [
        {**r, "cpus": "0,1,2"} if r["arm"] == "mibz2" and r["threads"] == 16 and r["round"] == 3 else r
        for r in low()], 1, ["VOID", "not unpinned"]))

    # candidates judged by count (`just bench-workers-at-cores`): production's `mibz2` the
    # reference at 8 and 10 threads asked, within the cores (pinned one CPU a core, a spinner on
    # a free core at 8, none at 10); 7/28, 8/32 and 9/36 judged at 10, 7/28 and 6/24 at 8; every
    # other cell reported, not judged; one Holm family of five cells, a verdict and a choice a count
    atc_cands = ["mibz2-occ-6-24", "mibz2-occ-7-28", "mibz2-occ-8-32", "mibz2-occ-9-36"]
    atc_at = {"mibz2-occ-6-24": "8", "mibz2-occ-7-28": "8,10", "mibz2-occ-8-32": "10", "mibz2-occ-9-36": "10"}

    def atc(steps=None, guard=None, mem=None, edit=None):
        """at-cores-shaped runs: `steps`, `guard` and `mem` map (arm, threads) to its main-corpus
        step, its text step and its peak memory over production's (default level); `edit` a
        function of a record returning it changed, or None to leave it out"""
        import random
        steps, guard, mem = steps or {}, guard or {}, mem or {}
        rng = random.Random(17)
        rows = []
        for i in range(R):
            for corpus, out in (("wikidata-x4", (838800784, "aa")), ("text", (49067679, "bb"))):
                for t in (8, 10):
                    base = 0.5 * (1 + 0.02 * rng.uniform(-1, 1))
                    wig = 1 + 0.01 * (1 if i % 2 else -1)
                    for arm in ["mibz2", *atc_cands]:
                        f = (steps if corpus == "wikidata-x4" else guard).get((arm, t), 1.0)
                        wall = base * (f * wig if arm != "mibz2" else 1.0)
                        r = {"round": i, "corpus": corpus, "threads": t, "arm": arm, "termination": "success",
                             "wall": wall, "cpu": wall * t * 0.9, "rss_kb": 100000 * t * mem.get((arm, t), 1.0),
                             "out": out[0], "hash": out[1], "cpus": pinned(t)}
                        if t <= 9:
                            r["held_spinner"] = {"cpu": 19, "auto": True, "overlap": False,
                                                 "decoder_cpus": pinned(t), "owner": "run", "cpu_s": 2.0}
                        r = edit(r) if edit else r
                        if r is not None:
                            rows.append(r)
        return rows

    def atc_args(at=None, cands=None):
        at = atc_at if at is None else at
        return ["--ref", "mibz2", *sum((["--cand", c] for c in (cands or atc_cands)), []),
                *sum((["--judge-at", f"{c}={ts}"] for c, ts in at.items()), []), "--main", "wikidata-x4",
                "--guard", "text", "--judged", "8,10", "--controls", "", "--hold-max-threads", "9"]

    def first(arm, t, corpus="wikidata-x4", rnd=3):
        return lambda r: r["arm"] == arm and r["threads"] == t and r["corpus"] == corpus and r["round"] == rnd

    results.append(expect_low("by count: 7 workers adopt at 10, the rest reported", atc({("mibz2-occ-7-28", 10): 0.95}), 0, [
        "VERDICT for mibz2-occ-7-28 at 10 threads: ADOPTS", "VERDICT for mibz2-occ-8-32 at 10 threads: NOT ADOPTED",
        "VERDICT for mibz2-occ-9-36 at 10 threads: NOT ADOPTED", "VERDICT for mibz2-occ-7-28 at 8 threads: NOT ADOPTED",
        "VERDICT for mibz2-occ-6-24 at 8 threads: NOT ADOPTED", "THE CHOICE at 10 threads: mibz2-occ-7-28",
        "THE CHOICE at 8 threads: none; the reference stays", "over the 5 main-corpus cells",
        "Context at 10 threads, reported and not judged:\n    mibz2-occ-6-24 wikidata-x4",
        "Context at 8 threads, reported and not judged:\n    mibz2-occ-8-32 wikidata-x4"],
        args=atc_args(), absent=["VERDICT for mibz2-occ-6-24 at 10", "VERDICT for mibz2-occ-9-36 at 8",
                                 "VERDICT for mibz2-occ-8-32 at 8", "mibz2-occ-6-24 at 10 p"]))
    results.append(expect_low("by count: a gain where an arm is not judged judges nothing",
                              atc({("mibz2-occ-6-24", 10): 0.9, ("mibz2-occ-9-36", 8): 0.9}), 0, [
        "THE CHOICE at 10 threads: none", "THE CHOICE at 8 threads: none", "mibz2-occ-6-24 wikidata-x4: 0.9"],
        args=atc_args()))
    results.append(expect_low("by count: two adopt at 10, the faster chosen; one at 8",
                              atc({("mibz2-occ-8-32", 10): 0.97, ("mibz2-occ-7-28", 10): 0.94,
                                   ("mibz2-occ-6-24", 8): 0.96}), 0, [
        "THE CHOICE at 10 threads: mibz2-occ-7-28, of those that ADOPT there (mibz2-occ-7-28, mibz2-occ-8-32)",
        "THE CHOICE at 8 threads: mibz2-occ-6-24"], args=atc_args()))
    results.append(expect_low("by count: each count read alone (a loss at 10 leaves 8's adoption)",
                              atc({("mibz2-occ-7-28", 10): 0.96, ("mibz2-occ-7-28", 8): 0.96},
                                  guard={("mibz2-occ-7-28", 10): 1.05}), 0, [
        "VERDICT for mibz2-occ-7-28 at 10 threads: A FINDING, NOT ADOPTED. A gain (Holm), but SLOWER on ['text']",
        "VERDICT for mibz2-occ-7-28 at 8 threads: ADOPTS", "THE CHOICE at 8 threads: mibz2-occ-7-28",
        "THE CHOICE at 10 threads: none"], args=atc_args()))
    results.append(expect_low("by count: held on memory at 8",
                              atc({("mibz2-occ-6-24", 8): 0.96}, mem={("mibz2-occ-6-24", 8): 1.2}), 0, [
        "VERDICT for mibz2-occ-6-24 at 8 threads: HELD ON MEMORY", "THE CHOICE at 8 threads: none"],
        args=atc_args()))
    results.append(expect_low("by count: a candidate without --judge-at", atc(), 1, ["VOID", "has no --judge-at"],
                              args=atc_args(at={k: v for k, v in atc_at.items() if k != "mibz2-occ-9-36"})))
    results.append(expect_low("by count: --judge-at for an arm not a candidate", atc(), 1,
                              ["VOID", "mibz2 is not a candidate"], args=atc_args(at={**atc_at, "mibz2": "10"})))
    results.append(expect_low("by count: a count not judged", atc(), 1, ["VOID", "[12] not judged counts"],
                              args=atc_args(at={**atc_at, "mibz2-occ-8-32": "10,12"})))
    results.append(expect_low("by count: a judged count with no candidate", atc(), 1,
                              ["VOID", "the judged count 8 judges no candidate"],
                              args=atc_args(at={**atc_at, "mibz2-occ-6-24": "10", "mibz2-occ-7-28": "10"})))
    results.append(expect_low("by count: --judge-at malformed", atc(), 1, ["VOID", "not ARM=COUNTS"],
                              args=atc_args(at={**atc_at, "mibz2-occ-9-36": ""})))
    results.append(expect_low("by count: unpinned at 10 threads", atc(edit=lambda r: {**r, "cpus": "all"}
                                                                       if first("mibz2-occ-8-32", 10)(r) else r), 1,
                              ["VOID", "its CPUs 'all' missing or malformed"], args=atc_args()))
    results.append(expect_low("by count: a spinner at 10 threads", atc(
        edit=lambda r: {**r, "held_spinner": {"cpu": 19, "overlap": False}} if first("mibz2", 10)(r) else r), 1,
        ["VOID", "past the hold limit"], args=atc_args()))
    results.append(expect_low("by count: no spinner at 8 threads", atc(
        edit=lambda r: {k: v for k, v in r.items() if k != "held_spinner"} if first("mibz2-occ-6-24", 8)(r) else r),
        1, ["VOID", "no spinner recorded"], args=atc_args()))
    results.append(expect_low("by count: a context arm's cell missing", atc(
        edit=lambda r: None if first("mibz2-occ-9-36", 8, "text")(r) else r), 1,
        ["VOID", "roster keys missing"], args=atc_args()))
    results.append(expect_low("by count: no --judge-at, the old reading", low({"mibz2-occ-7-28": 0.96}), 0,
                              ["THE CHOICE: mibz2-occ-7-28"], absent=["judged by count", "THE CHOICE at"]))

    # a sparse roster (`just bench-workers-sweep-low`): production's `mibz2` the reference at 8, 6
    # and 4 threads asked (pinned one CPU a core, a spinner on a free core at each); each count's
    # own candidates alone, run there and nowhere else: 6/24 and 5/20 at 8; 6/24 (the lazy
    # control), 5/20 and 4/16 at 6; 4/16 (the lazy control), 3/12 and 2/8 at 4. One Holm family
    # of eight cells, a verdict and a choice a count
    swp_at = {"mibz2-occ-6-24": "8,6", "mibz2-occ-5-20": "8,6", "mibz2-occ-4-16": "6,4",
              "mibz2-occ-3-12": "4", "mibz2-occ-2-8": "4"}
    swp_cands = list(swp_at)

    def swp(steps=None, guard=None, mem=None, edit=None, extra=()):
        """sweep-shaped runs, each count's arms alone; as `atc`, and `extra` (arm, threads)
        cells run beside them"""
        import random
        steps, guard, mem = steps or {}, guard or {}, mem or {}
        rng = random.Random(23)
        rows = []
        for t in (8, 6, 4):
            arms = ["mibz2", *(c for c in swp_cands if str(t) in swp_at[c].split(",")),
                    *(a for a, at in extra if at == t)]
            for i in range(R):
                for corpus, out in (("wikidata-x4", (838800784, "aa")), ("text", (49067679, "bb"))):
                    base = 0.5 * (1 + 0.02 * rng.uniform(-1, 1))
                    wig = 1 + 0.01 * (1 if i % 2 else -1)
                    for arm in arms:
                        f = (steps if corpus == "wikidata-x4" else guard).get((arm, t), 1.0)
                        wall = base * (f * wig if arm != "mibz2" else 1.0)
                        r = {"round": i, "corpus": corpus, "threads": t, "arm": arm, "termination": "success",
                             "wall": wall, "cpu": wall * t * 0.9, "rss_kb": 100000 * t * mem.get((arm, t), 1.0),
                             "out": out[0], "hash": out[1], "cpus": pinned(t),
                             "held_spinner": {"cpu": 19, "auto": True, "overlap": False,
                                              "decoder_cpus": pinned(t), "owner": "run", "cpu_s": 2.0}}
                        r = edit(r) if edit else r
                        if r is not None:
                            rows.append(r)
        return rows

    def swp_args(sparse=True, controls=""):
        return ["--ref", "mibz2", *sum((["--cand", c] for c in swp_cands), []),
                *sum((["--judge-at", f"{c}={ts}"] for c, ts in swp_at.items()), []), "--main", "wikidata-x4",
                "--guard", "text", "--judged", "8,6,4", "--controls", controls, "--hold-max-threads", "9",
                *(["--sparse"] if sparse else [])]

    results.append(expect_low("sparse: a choice a count, each count's arms alone",
                              swp({("mibz2-occ-5-20", 8): 0.95, ("mibz2-occ-5-20", 6): 0.94,
                                   ("mibz2-occ-4-16", 6): 0.96, ("mibz2-occ-4-16", 4): 0.95,
                                   ("mibz2-occ-3-12", 4): 0.92}), 0, [
        "sparse: each count's own arms alone", "over the 8 main-corpus cells",
        "VERDICT for mibz2-occ-6-24 at 8 threads: NOT ADOPTED", "VERDICT for mibz2-occ-5-20 at 8 threads: ADOPTS",
        "VERDICT for mibz2-occ-6-24 at 6 threads: NOT ADOPTED", "VERDICT for mibz2-occ-4-16 at 4 threads: ADOPTS",
        "VERDICT for mibz2-occ-2-8 at 4 threads: NOT ADOPTED",
        "THE CHOICE at 8 threads: mibz2-occ-5-20", "THE CHOICE at 6 threads: mibz2-occ-5-20, of those that ADOPT "
        "there (mibz2-occ-5-20, mibz2-occ-4-16)", "THE CHOICE at 4 threads: mibz2-occ-3-12, of those that ADOPT "
        "there (mibz2-occ-4-16, mibz2-occ-3-12)"],
        args=swp_args(), absent=["Context at", "VERDICT for mibz2-occ-2-8 at 8", "VERDICT for mibz2-occ-3-12 at 6",
                                 "VERDICT for mibz2-occ-4-16 at 8"]))
    results.append(expect_low("sparse: the lazy control's gain at 4 is judged, a loss at 6 stays at 6",
                              swp({("mibz2-occ-4-16", 4): 0.9, ("mibz2-occ-6-24", 6): 0.95},
                                  guard={("mibz2-occ-6-24", 6): 1.05}), 0, [
        "VERDICT for mibz2-occ-4-16 at 4 threads: ADOPTS", "THE CHOICE at 4 threads: mibz2-occ-4-16",
        "VERDICT for mibz2-occ-6-24 at 6 threads: A FINDING, NOT ADOPTED", "THE CHOICE at 6 threads: none",
        "THE CHOICE at 8 threads: none"], args=swp_args()))
    results.append(expect_low("sparse: an arm run where it is not judged", swp(extra=[("mibz2-occ-2-8", 8)]), 1,
                              ["VOID", "('wikidata-x4', 'mibz2-occ-2-8', 8, 0) not in the roster"], args=swp_args()))
    results.append(expect_low("sparse: a judged cell missing", swp(
        edit=lambda r: None if first("mibz2-occ-3-12", 4, "text")(r) else r), 1,
        ["VOID", "roster keys missing"], args=swp_args()))
    results.append(expect_low("sparse: the reference missing at a count", swp(
        edit=lambda r: None if r["arm"] == "mibz2" and r["threads"] == 6 else r), 1,
        ["VOID", "20 of 220 roster keys missing"], args=swp_args()))
    results.append(expect_low("not sparse: the crossed roster wants every arm at every count", swp(), 1,
                              ["VOID", "roster keys missing"], args=swp_args(sparse=False)))
    results.append(expect_low("sparse without --judge-at", low(), 1, ["VOID", "--sparse: with --judge-at"],
                              args=low_args() + ["--sparse"]))
    results.append(expect_low("sparse with control counts", swp(), 1, ["VOID", "--sparse: with --judge-at"],
                              args=swp_args(controls="10")))
    results.append(expect_low("sparse: the spinner checked at 4 threads", swp(
        edit=lambda r: {k: v for k, v in r.items() if k != "held_spinner"} if first("mibz2-occ-2-8", 4)(r) else r),
        1, ["VOID", "no spinner recorded"], args=swp_args()))

    # eager against lazy at trimmed counts (`just bench-eager-trim`): production's `mibz2` the
    # reference at 6, 4 and 10 threads asked (pinned one CPU a core; a spinner on a free core at
    # 6 and 4, none at 10); eager and lazy 5/20 at 6, eager 3/12 at 4, eager and lazy 7/28 at
    # 10. One Holm family of five cells, a verdict and a choice a count, and the eager arm over
    # its lazy twin at 6 and 10 reported, not judged
    ett_at = {"mibz2-oce-5-20": "6", "mibz2-occ-5-20": "6", "mibz2-oce-3-12": "4",
              "mibz2-oce-7-28": "10", "mibz2-occ-7-28": "10"}
    ett_cands = list(ett_at)
    ett_pairs = ["mibz2-oce-5-20/mibz2-occ-5-20", "mibz2-oce-7-28/mibz2-occ-7-28"]

    def ett(steps=None, guard=None):
        import random
        steps, guard = steps or {}, guard or {}
        rng = random.Random(29)
        rows = []
        for t in (6, 4, 10):
            arms = ["mibz2", *(c for c in ett_cands if ett_at[c] == str(t))]
            for i in range(R):
                for corpus, out in (("wikidata-x4", (838800784, "aa")), ("text", (49067679, "bb"))):
                    base = 0.5 * (1 + 0.02 * rng.uniform(-1, 1))
                    wig = 1 + 0.01 * (1 if i % 2 else -1)
                    for arm in arms:
                        f = (steps if corpus == "wikidata-x4" else guard).get((arm, t), 1.0)
                        wall = base * (f * wig if arm != "mibz2" else 1.0)
                        r = {"round": i, "corpus": corpus, "threads": t, "arm": arm, "termination": "success",
                             "wall": wall, "cpu": wall * t * 0.9, "rss_kb": 100000 * t, "out": out[0],
                             "hash": out[1], "cpus": pinned(t)}
                        if t <= 9:
                            r["held_spinner"] = {"cpu": 19, "auto": True, "overlap": False,
                                                 "decoder_cpus": pinned(t), "owner": "run", "cpu_s": 2.0}
                        rows.append(r)
        return rows

    def ett_args(pairs=None, judge=True):
        return ["--ref", "mibz2", *sum((["--cand", c] for c in ett_cands), []),
                *(sum((["--judge-at", f"{c}={ts}"] for c, ts in ett_at.items()), []) if judge else []),
                "--main", "wikidata-x4", "--guard", "text", "--judged", "6,4,10", "--controls", "",
                "--hold-max-threads", "9", *(["--sparse"] if judge else []),
                *sum((["--pair", p] for p in (ett_pairs if pairs is None else pairs)), [])]

    ett_steps = {("mibz2-oce-5-20", 6): 0.90, ("mibz2-occ-5-20", 6): 0.96, ("mibz2-oce-7-28", 10): 0.98}
    results.append(expect_low("eager trim: a choice a count, the pairs reported at 6 and 10", ett(ett_steps), 0, [
        "over the 5 main-corpus cells", "VERDICT for mibz2-oce-5-20 at 6 threads: ADOPTS",
        "VERDICT for mibz2-occ-5-20 at 6 threads: ADOPTS", "THE CHOICE at 6 threads: mibz2-oce-5-20",
        "THE CHOICE at 4 threads: none; the reference stays", "VERDICT for mibz2-occ-7-28 at 10 threads: NOT ADOPTED",
        "Pairs at 6 threads, reported and not judged",
        "mibz2-oce-5-20 over mibz2-occ-5-20 wikidata-x4: 0.9375", "mibz2-oce-5-20 over mibz2-occ-5-20 text: 1.0000",
        "Pairs at 10 threads, reported and not judged", "mibz2-oce-7-28 over mibz2-occ-7-28 wikidata-x4: 0.9800"],
        args=ett_args(), absent=["Pairs at 4 threads", "VOID"]))
    results.append(expect_low("eager trim: a pair SLOWER judges nothing",
                              ett({("mibz2-oce-7-28", 10): 1.05, ("mibz2-occ-7-28", 10): 0.95}), 0, [
        "mibz2-oce-7-28 over mibz2-occ-7-28 wikidata-x4: 1.10", "SLOWER",
        "VERDICT for mibz2-occ-7-28 at 10 threads: ADOPTS", "THE CHOICE at 10 threads: mibz2-occ-7-28"],
        args=ett_args()))
    results.append(expect_low("eager trim: without --pair, no pairs", ett(ett_steps), 0, [
        "THE CHOICE at 6 threads: mibz2-oce-5-20"], args=ett_args(pairs=[]), absent=["Pairs at"]))
    results.append(expect_low("eager trim: --pair without --judge-at", low(), 1, ["VOID", "--pair: with --judge-at only"],
                              args=low_args() + ["--pair", "mibz2-occ-7-28/mibz2-occ-6-24"]))
    results.append(expect_low("eager trim: --pair malformed", ett(), 1, ["VOID", "not ARM/ARM, two arms"],
                              args=ett_args(pairs=["mibz2-oce-5-20"])))
    results.append(expect_low("eager trim: --pair of one arm twice", ett(), 1, ["VOID", "not ARM/ARM, two arms"],
                              args=ett_args(pairs=["mibz2-oce-5-20/mibz2-oce-5-20"])))
    results.append(expect_low("eager trim: --pair naming an arm not in the run", ett(), 1,
                              ["VOID", "['mibz2-occ-3-12'] not an arm of the run"],
                              args=ett_args(pairs=["mibz2-oce-3-12/mibz2-occ-3-12"])))
    results.append(expect_low("eager trim: --pair of two arms never run together", ett(), 1,
                              ["VOID", "run together at no judged count"],
                              args=ett_args(pairs=["mibz2-oce-3-12/mibz2-oce-5-20"])))
    results.append(expect_low("eager trim: the lazy twin missing at 10", [
        r for r in ett() if not (r["arm"] == "mibz2-occ-7-28" and r["round"] == 2)], 1,
        ["VOID", "roster keys missing"], args=ett_args()))

    # unpinned, the reference by count (`just bench-unpinned`): every run on all 20 CPUs, no
    # spinner; at 10 threads lazy 7/28 over lazy 10/40, at 8 lazy 6/24 over lazy 8/32, at 6 lazy
    # 5/20 over eager 6/24. One Holm family of three cells, a verdict and a choice a count
    unp_ref = {10: "mibz2-occ-10-40", 8: "mibz2-occ-8-32", 6: "mibz2-oce-6-24"}
    unp_cand = {10: "mibz2-occ-7-28", 8: "mibz2-occ-6-24", 6: "mibz2-occ-5-20"}
    every = ",".join(str(c) for c in range(20))

    def unp(steps=None, guard=None, mem=None, edit=None):
        import random
        steps, guard, mem = steps or {}, guard or {}, mem or {}
        rng = random.Random(31)
        rows = []
        for t in (10, 8, 6):
            for i in range(R):
                for corpus, out in (("wikidata-x4", (838800784, "aa")), ("text", (49067679, "bb"))):
                    base = 0.5 * (1 + 0.02 * rng.uniform(-1, 1))
                    wig = 1 + 0.01 * (1 if i % 2 else -1)
                    for arm in (unp_ref[t], unp_cand[t]):
                        f = (steps if corpus == "wikidata-x4" else guard).get(t, 1.0)
                        wall = base * (f * wig if arm == unp_cand[t] else 1.0)
                        rss = 100000 * t * (mem.get(t, 1.0) if arm == unp_cand[t] else 1.0)
                        r = {"round": i, "corpus": corpus, "threads": t, "arm": arm, "termination": "success",
                             "wall": wall, "cpu": wall * t * 0.9, "rss_kb": rss, "out": out[0],
                             "hash": out[1], "cpus": every}
                        r = edit(r) if edit else r
                        if r is not None:
                            rows.append(r)
        return rows

    def unp_args(refs=None, unpinned=True, hold="0", ref=None):
        refs = [f"{a}={t}" for t, a in unp_ref.items()] if refs is None else refs
        return [*(["--ref", ref] if ref else []), *sum((["--ref-at", x] for x in refs), []),
                *sum((["--cand", c] for c in unp_cand.values()), []),
                *sum((["--judge-at", f"{c}={t}"] for t, c in unp_cand.items()), []), "--sparse",
                "--main", "wikidata-x4", "--guard", "text", "--judged", "10,8,6", "--controls", "",
                "--hold-max-threads", hold, *(["--unpinned"] if unpinned else [])]

    results.append(expect_low("unpinned: each count against its own reference", unp({10: 0.88, 8: 0.97}), 0, [
        "the reference by count: mibz2-occ-10-40 at 10, mibz2-occ-8-32 at 8, mibz2-oce-6-24 at 6",
        "unpinned: every run on every CPU", "over the 3 main-corpus cells",
        "10 threads: the judged cells, each candidate over mibz2-occ-10-40",
        "6 threads: the judged cells, each candidate over mibz2-oce-6-24",
        "VERDICT for mibz2-occ-7-28 at 10 threads: ADOPTS", "VERDICT for mibz2-occ-6-24 at 8 threads: ADOPTS",
        "VERDICT for mibz2-occ-5-20 at 6 threads: NOT ADOPTED", "THE CHOICE at 10 threads: mibz2-occ-7-28",
        "THE CHOICE at 6 threads: none; the reference stays"], args=unp_args(), absent=["VOID"]))
    results.append(expect_low("unpinned: a loss at 6, the memory gate at 8", unp({6: 1.05, 8: 0.95}, mem={8: 1.2}), 0, [
        "VERDICT for mibz2-occ-5-20 at 6 threads: NOT ADOPTED. No gain established (Holm does not reject); "
        "SLOWER on ['wikidata-x4']", "VERDICT for mibz2-occ-6-24 at 8 threads: HELD ON MEMORY"], args=unp_args()))
    results.append(expect_low("unpinned: a run pinned one CPU a core", unp(
        edit=lambda r: {**r, "cpus": ",".join(str(c) for c in range(r["threads"]))} if r["round"] == 4 else r), 1,
        ["VOID", "are not every one of the 20 CPUs (unpinned)"], args=unp_args()))
    results.append(expect_low("unpinned: a run recorded as 'all' (inherited, not set)", unp(
        edit=lambda r: {**r, "cpus": "all"} if r["threads"] == 8 and r["round"] == 0 else r), 1,
        ["VOID", "'all' missing or malformed, not every CPU listed"], args=unp_args()))
    results.append(expect_low("unpinned: a CPU missing", unp(
        edit=lambda r: {**r, "cpus": ",".join(str(c) for c in range(19))} if r["threads"] == 6 else r), 1,
        ["VOID", "not every one of the 20 CPUs"], args=unp_args()))
    results.append(expect_low("unpinned: a spinner recorded", unp(
        edit=lambda r: {**r, "held_spinner": {"cpu": 19, "auto": True, "overlap": True, "decoder_cpus": every,
                                              "owner": "run", "cpu_s": 2.0}} if r["threads"] == 10 else r), 1,
        ["VOID", "sharing a decoder core"], args=unp_args()))
    results.append(expect_low("unpinned: a spinner allowed", unp(), 1,
                              ["VOID", "--unpinned: with --hold-max-threads 0 only"], args=unp_args(hold="9")))
    results.append(expect_low("unpinned runs read as pinned ones", unp(), 1,
                              ["VOID", "are not one on each of 10 cores"], args=unp_args(unpinned=False)))
    results.append(expect_low("--ref-at: a count with none", unp(), 1, ["VOID", "the judged count 6 has no --ref-at"],
                              args=unp_args(refs=["mibz2-occ-10-40=10", "mibz2-occ-8-32=8"])))
    results.append(expect_low("--ref-at: a count with two", unp(), 1, ["VOID", "8 has two references"],
                              args=unp_args(refs=["mibz2-occ-10-40=10,8", "mibz2-occ-8-32=8", "mibz2-oce-6-24=6"])))
    results.append(expect_low("--ref-at: a count not judged", unp(), 1, ["VOID", "4 is not a judged count"],
                              args=unp_args(refs=[*(f"{a}={t}" for t, a in unp_ref.items()), "mibz2=4"])))
    results.append(expect_low("--ref-at: malformed", unp(), 1, ["VOID", "not ARM=COUNTS"],
                              args=unp_args(refs=[*(f"{a}={t}" for t, a in unp_ref.items()), "mibz2"])))
    results.append(expect_low("--ref-at and --ref together", unp(), 1, ["VOID", "--ref or --ref-at, exactly one"],
                              args=unp_args(ref="mibz2")))
    results.append(expect_low("neither --ref nor --ref-at", unp(), 1, ["VOID", "--ref or --ref-at, exactly one"],
                              args=unp_args(refs=[])))
    results.append(expect_low("--ref-at: a reference that is a candidate", unp(), 1, ["VOID", "two arms are one"],
                              args=unp_args(refs=["mibz2-occ-10-40=10", "mibz2-occ-8-32=8", "mibz2-occ-5-20=6"])))
    results.append(expect_low("--ref-at without --judge-at", low(), 1, ["VOID", "--ref-at: with --judge-at and --sparse"],
                              args=low_args()[2:] + ["--ref-at", "mibz2-occ-8-32=12,16,20"]))
    results.append(expect_low("unpinned: the reference missing at a count", unp(
        edit=lambda r: None if r["arm"] == "mibz2-oce-6-24" and r["round"] == 1 else r), 1,
        ["VOID", "roster keys missing"], args=unp_args()))

    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
