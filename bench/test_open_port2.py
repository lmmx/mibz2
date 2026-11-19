#!/usr/bin/env python3
"""Checks of bench/open_port2.py, the open port's successor verdict, on made-up bundles: the
decision run, the sustained cell at its locked rounds, and every batch of the locked schedule; G2 (each
guard cell's mean log ratio against its locked critical value) catching a real slowdown, passing
symmetric spikes, and failing the Researcher's counterexample (REVIEW 7012181194); each VOID,
across files too; and the successor's own one attempt"""

import io
import json
import math
import os
import random
import shutil
import subprocess
import sys
import tempfile
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import open_port as op  # noqa: E402
import open_port2 as o2  # noqa: E402
import test_open_port as t1  # noqa: E402

PORT_SHA = t1.PORT_SHA
COUNTER = [-0.001] * 57 + [0.5] * 3  # the Researcher's counterexample: x1.02434 by geometric mean


def noise(seed, n, scale=0.01, spikes=0):
    """Symmetric noise for n rounds: pairs of +e and -e, shuffled, so a cell of no effect has a
    mean of 0 (n even); `spikes` adds that many pairs of +0.29 and -0.29"""
    rng = random.Random(seed)
    half = [abs(rng.gauss(0, scale)) for _ in range(n // 2 - spikes)] + [0.29] * spikes
    xs = [v for e in half for v in (e, -e)] + [0.0] * (n % 2)
    rng.shuffle(xs)
    return xs


def write(path, rows):
    path.write_text("".join(json.dumps(r) + "\n" for r in rows))


def build2(
    gstep=None,
    vectors=None,
    spikes=None,
    drop=(),
    stray=False,
    bedit=None,
    sustained=0.97,
    sus_n=None,
    **kw,
):
    """The first registration's decision and sustained bundle (test_open_port.build), and every
    batch of the locked schedule. `gstep` maps (corpus, t) to the port's true wall ratio on that
    guard (decision rounds too); `vectors` maps (corpus, t) to the extension's exact log ratios;
    `spikes` maps (corpus, t) to pairs of symmetric spikes; `bedit` edits extension records;
    `sustained` is S's true wall ratio, over `sus_n` rounds (S's locked rounds by default)."""
    gstep, vectors, spikes = gstep or {}, vectors or {}, spikes or {}
    step = dict(kw.pop("step", {}) or {})
    step.update(gstep)
    d = t1.build(step=step, **kw)
    srows = []
    for i, x in enumerate(noise(99, sus_n or o2.PLAN[o2.S][0], scale=0.005)):
        srows.append(t1.rec(op.SUSTAINED, 10, t1.REF, i, 1.6, 120000))
        srows.append(t1.rec(op.SUSTAINED, 10, t1.PORT, i, 1.6 * sustained * math.exp(x), 84000))
    write(d / "sustained.jsonl", srows)
    seed = 0
    for t, extra, cs in o2.batches():
        rows = []
        for c in cs:
            seed += 1
            xs = vectors.get((c, t)) or noise(seed, extra, spikes=spikes.get((c, t), 0))
            assert len(xs) == extra
            for i, x in enumerate(xs):
                base = 0.05
                for r in (
                    t1.rec(c, t, t1.REF, i, base, 100000),
                    t1.rec(c, t, t1.PORT, i, base * gstep.get((c, t), 1.0) * math.exp(x), 100000),
                ):
                    r = bedit(r, t, extra) if bedit else r
                    if r is not None:
                        rows.append(r)
        name = o2.batch_file(t, extra)
        if name not in drop:
            write(d / name, rows)
    if stray:
        write(d / "guards-t8-r999.jsonl", [])
    return d


def expect(name, d, code, phrases, absent=()):
    try:
        buf = io.StringIO()
        with redirect_stdout(buf):
            got = o2.main(["open_port2.py", "judge", str(d), "--port", PORT_SHA])
        out = buf.getvalue()
    finally:
        shutil.rmtree(d)
    ok = got == code and all(p in out for p in phrases) and not any(p in out for p in absent)
    print(
        f"{'ok  ' if ok else 'FAIL'} {name}: exit {got}, "
        + ", ".join(
            [f"{'found' if p in out else 'missing'} {p!r}" for p in phrases]
            + [f"{'present' if p in out else 'absent'} {p!r}" for p in absent]
        )
    )
    if not ok:
        print(out)
    return ok


def guard_checks():
    """The successor's one attempt: its own evidence path, failing closed"""

    def git(cwd, *args):
        subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True)

    def run(target, this, remote, cwd):
        buf = io.StringIO()
        old = Path.cwd()
        os.chdir(cwd)
        try:
            with redirect_stdout(buf):
                got = o2.main(
                    ["open_port2.py", "guard", str(target), "--this", str(this), "--remote", remote]
                )
        finally:
            os.chdir(old)
        return got, buf.getvalue()

    root = Path(tempfile.mkdtemp())
    out = []
    try:
        ident = ["-c", "user.name=t", "-c", "user.email=t@t"]
        origin = root / "origin.git"
        git(root, "init", "-q", "--bare", str(origin))
        seed = root / "seed"
        git(root, "init", "-q", str(seed))
        git(seed, "checkout", "-q", "--orphan", "evidence")
        (seed / "open-port").mkdir()
        (seed / "open-port" / "run-20261119T125538Z.tar.gz").write_bytes(b"first")
        git(seed, "add", "open-port")
        git(seed, *ident, "commit", "-q", "-m", "the first registration's attempt")
        git(seed, "push", "-q", str(origin), "evidence")
        work = root / "work"
        git(root, "clone", "-q", str(origin), str(work))
        target = work / "target" / "open-port-2"
        this = target / "run-20261120T000000Z"
        this.mkdir(parents=True)
        (this / "timing-started.txt").write_text("now\n")

        def check(name, want, phrase, remote="origin"):
            got, text = run(target, this, remote, work)
            ok = got == want and phrase in text
            print(
                f"{'ok  ' if ok else 'FAIL'} guard: {name}: exit {got}"
                + ("" if ok else f"\n{text}")
            )
            return ok

        out.append(
            check(
                "the first registration's open-port/ bundle is not this one's attempt: passes",
                0,
                "",
            )
        )
        out.append(
            check("an unreadable evidence branch: refuses", 1, "can't be fetched", remote="nowhere")
        )
        timed = target / "run-20261119T200000Z"
        timed.mkdir()
        (timed / "timing-started.txt").write_text("then\n")
        out.append(check("a timed attempt started here: refuses", 1, "already started here"))
        shutil.rmtree(timed)
        (seed / "open-port-2").mkdir()
        (seed / "open-port-2" / "run-20261121T000000Z.tar.gz").write_bytes(b"x")
        git(seed, "add", "open-port-2")
        git(seed, *ident, "commit", "-q", "-m", "archived")
        git(seed, "push", "-q", str(origin), "evidence")
        out.append(
            check(
                "an open-port-2 attempt archived on the evidence branch: refuses",
                1,
                "already archived",
            )
        )
    finally:
        shutil.rmtree(root)
    return out


def main():
    res = []
    plan = o2.PLAN
    ext = sum(2 * (n - op.ROUNDS) for k, (n, _) in plan.items() if k != o2.S)
    res.append(
        expect(
            "the port as hoped, guards with no effect: ADOPTS",
            build2(),
            0,
            [
                "VERDICT: ADOPTS",
                "1080 of 1080 (decision)",
                f"{2 * o2.PLAN[o2.S][0]} of {2 * o2.PLAN[o2.S][0]} (sustained)",
                f"{ext} of {ext}",
            ],
            absent=["VOID", "FAILS"],
        )
    )
    for c, t in (("ho-code-l9", 10), ("text", 8), ("records", 20)):
        res.append(
            expect(
                f"{c} at {t} threads truly 2% slower: G2 fails it",
                build2(gstep={(c, t): 1.02}),
                0,
                [f"G2: {c} at {t} threads", "VERDICT: NOT ADOPTED"],
                absent=["VERDICT: ADOPTS"],
            )
        )
    res.append(
        expect(
            "a guard truly 3% slower: G2 fails it",
            build2(gstep={("ho-prose-l1", 20): 1.03}),
            0,
            ["G2: ho-prose-l1 at 20 threads", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "a guard of no effect with symmetric spikes of 0.29: G2 passes",
            build2(spikes={("ho-code-l1", 10): 3, ("random", 8): 2}),
            0,
            ["VERDICT: ADOPTS"],
            absent=["G2: "],
        )
    )
    # the Researcher's counterexample (REVIEW 7012181194): 57 rounds at -0.001 and 3 at +0.5
    cell = next((c, t) for (c, t), (n, _) in sorted(plan.items()) if n - op.ROUNDS == 60)
    res.append(
        expect(
            f"Researcher 11: the counterexample as {cell[0]} at {cell[1]} threads' extension: G2 fails it",
            build2(vectors={cell: COUNTER}),
            0,
            [f"G2: {cell[0]} at {cell[1]} threads", "NOT ADOPTED"],
        )
    )
    m = sum(COUNTER) / len(COUNTER)
    ok = all(m > c for _, c in plan.values())
    print(
        f"{'ok  ' if ok else 'FAIL'} Researcher 11: the counterexample's mean {m:.5f} exceeds every cell's c (max {max(c for _, c in plan.values()):.4f})"
    )
    res.append(ok)
    # S, exactly as registered
    res.append(
        expect(
            "the sustained cell truly 2% slower: G2 fails it (S, its 40th cell)",
            build2(sustained=1.02),
            0,
            ["G2: sustained", "NOT ADOPTED"],
        )
    )
    # the decision conditions, as registered
    res.append(
        expect(
            "no gain at 20 threads: P1 fails",
            build2(step={("wikidata-x4", 20): 1.0}),
            0,
            ["P1: no gain on wikidata-x4 at 20 threads", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "Wikidata memory above 1.00: P2 fails",
            build2(mem={("wikidata-x4", 8): 1.01}),
            0,
            ["P2: peak RSS on wikidata-x4 at 8 threads", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "no held-out replication: H fails",
            build2(step={("ho-wikidata-l9", 10): 1.0}),
            0,
            ["H: ho-wikidata-l9 at 10 threads", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "a guard's memory above 1.10: G fails",
            build2(mem={("records", 20): 1.2}),
            0,
            ["G: records at 20 threads peak RSS 1.200, above 1.1", "NOT ADOPTED"],
        )
    )
    # VOID, within and across files
    first = o2.batch_file(*o2.batches()[0][:2])
    other_port = t1.PORT.replace("/w/target/", "/elsewhere/target/")
    for name, d, phrase in [
        ("a batch file missing", build2(drop=(first,)), f"{first}: missing"),
        ("a stray guards file", build2(stray=True), "files outside the schedule"),
        (
            "Researcher 12: text's output digest different in the extension",
            build2(bedit=lambda r, t, x: {**r, "sha256": "zz"} if r["corpus"] == "text" else r),
            "text: the outputs differ across files",
        ),
        (
            "Researcher 13: the port's binary another path in one batch",
            build2(
                bedit=lambda r, t, x: (
                    {**r, "arm": other_port}
                    if r["arm"] == t1.PORT and (t, x) == o2.batches()[0][:2]
                    else r
                )
            ),
            "the port build's arm differs across files",
        ),
        (
            "an extension record at threads 8.5",
            build2(
                bedit=lambda r, t, x: {**r, "threads": 8.5} if r["round"] == 3 and t == 8 else r
            ),
            "threads 8.5",
        ),
        (
            "an extension record with a NaN wall",
            build2(bedit=lambda r, t, x: {**r, "wall": float("nan")} if r["round"] == 7 else r),
            "not a finite positive number",
        ),
        (
            "a batch short by a round",
            build2(bedit=lambda r, t, x: None if r["round"] == x - 1 and t == 20 else r),
            "records missing",
        ),
        ("another port", build2(commits=f"port {'1' * 40}\nv0.5.6 {op.V056}\n"), "commits.txt"),
        ("an input off the manifest", build2(digests={"text.bz2": "5a" * 32}), "not the frozen"),
    ]:
        res.append(expect(f"{name}: VOID", d, 1, ["VOID", phrase], absent=["VERDICT"]))
    nS = o2.PLAN[o2.S][0]
    for name, d, phrase in [
        (
            f"the sustained cell with a round past its {nS}",
            build2(sus_n=nS + 1),
            "not a record of the roster",
        ),
        (
            "the sustained cell at the first registration's 8 rounds",
            build2(sus_n=8),
            "records missing",
        ),
    ]:
        res.append(expect(f"{name}: VOID", d, 1, ["VOID", phrase], absent=["VERDICT"]))
    d = build2()
    (d / "DRY-RUN.txt").write_text("dry\n")
    res.append(expect("a dry run: VOID", d, 1, ["VOID", "a dry run"], absent=["VERDICT"]))
    # the locked schedule
    ok = (
        len(plan) == 40
        and o2.S in plan
        and all(op.ROUNDS <= n and 0 < c < math.log(op.GUARD_MAX) for n, c in plan.values())
        and sorted(c for t, x, cs in o2.batches() for c in cs)
        == sorted(c for (c, t), (n, _) in plan.items() if (c, t) != o2.S and n > op.ROUNDS)
    )
    print(
        f"{'ok  ' if ok else 'FAIL'} the schedule: 39 guard cells and S once, rounds at least 12, every c below ln 1.02, every extension in one batch"
    )
    res.append(ok)
    res.extend(guard_checks())
    failed = res.count(False)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
