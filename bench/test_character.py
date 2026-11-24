#!/usr/bin/env python3
"""Checks of bench/character.py, bench-character's report (stage 1), on synthetic records:
k_balance and its absence, each count's limit (SCANNER-, WORKER-, ADMISSION-LIMITED, MIXED), the ledgers'
cost bound (PERTURBED), INCOMPLETE, and each VOID. The harness's own identities are checked on
real decodes by tests/character.rs and CI's smoke run."""

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import character as cr  # noqa: E402

I9 = "Model name:                           Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
XEON = "Model name:                           Intel(R) Xeon(R) Processor @ 2.10GHz\n"
ROUNDS, BAND_ROUNDS, REPS, API_ROUNDS = 4, 6, 2, 1


def args():
    return argparse.Namespace(
        rounds=ROUNDS,
        band_rounds=BAND_ROUNDS,
        reps=REPS,
        api_rounds=API_ROUNDS,
        out_bytes=cr.OUT_BYTES,
    )


def proc(wall=0.3):
    return {"wall_s": wall, "bytes": cr.OUT_BYTES, "base_kb": 50000, "peak_kb": 120000, "mean_kb": 100000.0,
            "rss_kb_s": 30000.0, "user_s": 1.0, "sys_s": 0.1, "minflt": 1000, "majflt": 0, "nvcsw": 100,
            "nivcsw": 10}  # fmt: skip


def records():
    recs = []
    for r in range(BAND_ROUNDS):
        for o, k in enumerate(cr.GRID if r < ROUNDS else cr.BAND):
            for rep in range(REPS):
                x = {"kind": "decode", "section": "grid", "round": r, "order": o, "rep": rep, "k": k,
                     "on": 1, "window": 4 * k, "identities_ok": True, "identities": []}  # fmt: skip
                x.update(proc())
                recs.append(x)
    for p in range(ROUNDS // 2):
        for k in cr.COST_COUNTS:
            for arm in (0, 1):
                for rep in range(REPS):
                    x = {"kind": "decode", "section": "cost", "pair": p, "round": 2 * p + 1, "rep": rep,
                         "arm": arm, "k": k, "on": arm, "window": 4 * k}  # fmt: skip
                    x.update(proc())
                    recs.append(x)
    for r in range(API_ROUNDS):
        for o, t in enumerate(cr.GRID):
            x = {"kind": "api", "section": "api", "round": r, "order": o, "requested": t, "threads": t,
                 "window": 4 * t, "lazy": t >= 8, "service": False}  # fmt: skip
            x.update(proc())
            recs.append(x)
    return recs


def main():
    res = []

    def check(name, ok, why=""):
        res.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}{'' if ok else ': ' + str(why)}")

    # k*
    gaps = {k: [0.01 * (k - 6.5)] * 5 for k in cr.GRID}
    check("k* is the first count whose median gap is positive", cr.kstar(gaps) == 7, cr.kstar(gaps))
    gaps = {k: [-0.1, -0.2, 0.3] if k == 3 else [-0.1] * 3 for k in cr.GRID}
    check("k* by the median, not the mean", cr.kstar(gaps) is None, cr.kstar(gaps))
    gaps = {k: [-0.1] * 3 for k in cr.GRID}
    check("no k* where the gap is never positive", cr.kstar(gaps) is None)
    gaps = {k: ([0.1] * 3 if k in (4, 9) else [-0.1] * 3) for k in cr.GRID}
    check("k* the first, even where the gap turns back", cr.kstar(gaps) == 4)
    # the limit rule
    m = {"u_scanner": 0.9, "slot_share": 0.03, "u_worker": 0.5, "starve_share": 0.4}
    check("SCANNER-LIMITED", cr.limit(m) == "SCANNER-LIMITED", cr.limit(m))
    m = {"u_scanner": 0.3, "slot_share": 0.6, "u_worker": 0.97, "starve_share": 0.01}
    check("WORKER-LIMITED", cr.limit(m) == "WORKER-LIMITED", cr.limit(m))
    m = {"u_scanner": 0.7, "slot_share": 0.2, "u_worker": 0.4, "starve_share": 0.5}
    check("ADMISSION-LIMITED", cr.limit(m) == "ADMISSION-LIMITED", cr.limit(m))
    m = {"u_scanner": 0.8, "slot_share": 0.08, "u_worker": 0.6, "starve_share": 0.3}
    check("MIXED", cr.limit(m) == "MIXED", cr.limit(m))
    m = {"u_scanner": 0.85, "slot_share": 0.05, "u_worker": 0.9, "starve_share": 0.0}
    check("the scanner's rule first at its bounds", cr.limit(m) == "SCANNER-LIMITED", cr.limit(m))
    # the cost bound
    check("cost: 1.005 [0.99, 1.02] within", not cr.cost_flag(1.005, 0.99))
    check("cost: 1.011 PERTURBED (the point)", cr.cost_flag(1.011, 0.99))
    check("cost: 1.008 [1.006, 1.010] PERTURBED (the interval)", cr.cost_flag(1.008, 1.006))
    check("cost: 1.008 [1.004, 1.012] within", not cr.cost_flag(1.008, 1.004))
    # the records: whole, and each VOID
    a = args()
    void, inc = cr.validate(records(), a, I9)
    check("whole records: no VOID, nothing INCOMPLETE", not void and not inc, void[:3] + inc[:3])
    void, _ = cr.validate(records(), a, XEON)
    check("VOID: not the i9", any("i9-10900K" in v for v in void), void[:3])

    def mutated(fn):
        recs = records()
        fn(recs)
        return cr.validate(recs, a, I9)

    cases = [
        ("a grid decode missing", lambda r: r.remove(next(x for x in r if x.get("section") == "grid" and x["k"] == 7)), "decodes"),
        ("a grid decode duplicated", lambda r: r.append(dict(next(x for x in r if x.get("section") == "grid"))), "2 decodes"),
        ("a failed child", lambda r: r.append({"kind": "failed", "section": "grid", "cmd": ["--one"], "status": 101}), "failed"),
        ("wrong bytes", lambda r: next(x for x in r if x.get("section") == "grid").update(bytes=1), "bytes"),
        ("a window not 4k", lambda r: next(x for x in r if x.get("section") == "grid" and x["k"] == 5).update(window=21), "window"),
        ("no scheduling figures", lambda r: next(x for x in r if x.get("section") == "cost").pop("nivcsw"), "no"),
        ("a cost arm short", lambda r: r.remove(next(x for x in r if x.get("section") == "cost" and x["k"] == 10)), "cost pair"),
        ("an API count missing", lambda r: r.remove(next(x for x in r if x.get("section") == "api" and x["requested"] == 13)), "api round"),
        ("the ledgers off in the grid", lambda r: next(x for x in r if x.get("section") == "grid").update(on=0), "ledgers off"),
        ("a major fault (the corpus not resident)", lambda r: next(x for x in r if x.get("section") == "grid" and x["k"] == 9).update(majflt=3), "major faults"),
        ("a band round missing a count", lambda r: r.remove(next(x for x in r if x.get("section") == "grid" and x["round"] == ROUNDS and x["k"] == 12 and x["rep"] == 0)), "decodes"),
        ("a count outside the band in a band round", lambda r: r.append(dict(next(x for x in r if x.get("section") == "grid" and x["k"] == 3), round=ROUNDS + 1, order=10)), "counts"),
    ]  # fmt: skip
    for name, fn, phrase in cases:
        void, _ = mutated(fn)
        check(f"VOID: {name}", any(phrase in v for v in void), void[:3])

    def bad_identity(r):
        x = next(x for x in r if x.get("section") == "grid" and x["k"] == 12)
        x["identities_ok"] = False
        x["identities"] = [{"name": "Little's law", "ok": False, "detail": ""}]

    void, inc = mutated(bad_identity)
    check(
        "INCOMPLETE, not VOID: an identity failing",
        not void and len(inc) == 1 and "k 12" in inc[0],
        (void[:2], inc),
    )
    failed = sum(1 for x in res if not x)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
