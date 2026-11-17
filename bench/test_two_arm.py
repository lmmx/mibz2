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

    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
