"""The `walk-stats` records' checker: validity, then the engagement predicates (DRAFT, not frozen).

docs/journal/2026-11-10-staged-engagement-diagnostic-draft.md (§3 validity, §5
predicates), and the team screen's T2 probes (docs/journal/2026-11-10-team-screen-design-draft.md).
The records are `ab child`'s with the `walk-stats` feature and `AB_WALK_STATS=FILE`
(src/coop.rs, `walk_stats`): one JSON line each, of kind `call` (a walker call),
`share` (a helper's build share), `summary` (a cooperative block's, by the thread ending
its walk), `solo` (a block the leader walked alone), and a terminal `end`.

    python3 bench/walk_stats_check.py FILE --walker traverse_staged --classes CLASSES \\
        [--helpers 2] [--walked M] [--reference]

CLASSES is the input's block-class map (bench/team-screen/w-classes.txt: T a team's block,
S the leader's). Prints VALID or INVALID with every reason, then, for a valid file, each
predicate and the outcome: ENGAGED or NOT ENGAGED (`--reference`: the old arm's, its (c)
the analogue, reported and never an outcome). Exit 0 for ENGAGED or a valid reference, 1
for NOT ENGAGED, 2 for INVALID. No timing is read.
"""

import argparse
import json
import sys
from collections import defaultdict

U64 = 2**64
KINDS = {
    "call": {
        "team",
        "block",
        "thread",
        "role",
        "walker",
        "takes",
        "exhausts",
        "bytes",
        "tail",
        "fragments",
        "copied",
    },
    "share": {"team", "block", "thread"},
    "summary": {"team", "block", "rows", "starts", "seg_bytes", "entered", "joined", "built"},
    "solo": {"team", "block", "rows"},
    "end": {"records"},
}
WALKERS = {"traverse", "traverse_staged"}


def classes_of(path):
    text = "".join(line.strip() for line in open(path) if not line.startswith("#"))
    return {i: c for i, c in enumerate(text)}


def validate(lines, classes, helpers, walked=None):
    """(records by kind, a list of reasons it is INVALID: empty when VALID)"""
    bad = []
    recs = []
    for n, line in enumerate(lines):
        try:
            r = json.loads(line)
        except ValueError:
            bad.append(f"line {n + 1}: not JSON")
            continue
        kind = r.get("kind") if isinstance(r, dict) else None
        if kind not in KINDS or set(r) != KINDS[kind] | {"kind"}:
            bad.append(f"line {n + 1}: schema ({kind})")
            continue
        for k, v in r.items():
            if k in ("kind", "role", "walker", "joined"):
                continue
            if not isinstance(v, int) or isinstance(v, bool) or not 0 <= v < U64:
                bad.append(f"line {n + 1}: {k} = {v!r} is not a counter")
        recs.append(r)
    ends = [r for r in recs if r["kind"] == "end"]
    if len(ends) != 1 or recs[-1:] != ends:
        bad.append("no single terminal record at the end")
    elif ends[0]["records"] != len(recs) - 1:
        bad.append(f"the terminal record counts {ends[0]['records']}, the file has {len(recs) - 1}")
    by = defaultdict(list)
    for r in recs:
        by[r["kind"]].append(r)
    # unique keys
    keys = set()
    for r in recs:
        if r["kind"] == "end":
            continue
        key = (r["kind"], r["team"], r["block"], r.get("thread"))
        if key in keys:
            bad.append(f"duplicate record {key}")
        keys.add(key)
    for r in by["call"]:
        if r["walker"] not in WALKERS or r["role"] != ("leader" if r["thread"] == 0 else "helper"):
            bad.append(f"call {r['team']}/{r['block']}/{r['thread']}: walker or role")
    # exact block membership: every T block one summary, every S block one solo, no others
    want_t = {i for i, c in classes.items() if c == "T"}
    want_s = {i for i, c in classes.items() if c == "S"}
    got_t = [r["block"] for r in by["summary"]]
    got_s = [r["block"] for r in by["solo"]]
    if sorted(got_t) != sorted(want_t):
        bad.append(f"summaries for blocks {sorted(set(got_t) ^ want_t)[:8]}… differ from the T set")
    if sorted(got_s) != sorted(want_s):
        bad.append(
            f"solo records for blocks {sorted(set(got_s) ^ want_s)[:8]}… differ from the S set"
        )
    summary = {r["block"]: r for r in by["summary"]}
    team_of = {b: r["team"] for b, r in summary.items()}
    for kind in ("call", "share"):
        for r in by[kind]:
            if r["block"] not in summary or team_of[r["block"]] != r["team"]:
                bad.append(
                    f"{kind} {r['team']}/{r['block']}/{r['thread']} belongs to no summarised block of its team"
                )
    # rosters: each team's helpers the same, every block
    helpers_of = defaultdict(set)
    for r in by["share"]:
        helpers_of[r["team"]].add(r["thread"])
    for team, hs in helpers_of.items():
        if hs != set(range(1, helpers + 1)):
            bad.append(f"team {team}: helpers {sorted(hs)}, expected 1..{helpers}")
    # reconciliation and conservation, per block
    calls = defaultdict(dict)
    shares = defaultdict(set)
    for r in by["call"]:
        calls[r["block"]][r["thread"]] = r
    for r in by["share"]:
        shares[r["block"]].add(r["thread"])
    total = 0
    expected = sum(1 << h for h in range(1, helpers + 1))
    for b, s in summary.items():
        cs = calls[b]
        seen = sum(1 << t for t in cs)
        if seen != s["entered"]:
            bad.append(
                f"block {b}: call records {seen:b} are not the entered set {s['entered']:b} (a lost record)"
            )
        if (0 in cs) != s["joined"] or (s["entered"] & 1 != 0) != s["joined"]:
            bad.append(f"block {b}: the leader's call and its join bit disagree")
        if sum(1 << t for t in shares[b]) != s["built"]:
            bad.append(f"block {b}: share records are not the built set")
        # the roster, per T block (the Researcher's counterexample, REVIEW 6763089509 item 3):
        # every expected helper in the independently recorded built set AND in the share
        # records. A team-wide roster alone admits a block that some or all helpers never
        # built, and (a) then holds vacuously (no share, no call)
        if s["built"] != expected:
            bad.append(
                f"block {b}: built set {s['built']:b}, expected helpers 1..{helpers} ({expected:b})"
            )
        if shares[b] != set(range(1, helpers + 1)):
            bad.append(
                f"block {b}: share records from {sorted(shares[b])}, expected helpers 1..{helpers}"
            )
        if sum(c["takes"] for c in cs.values()) != s["starts"]:
            bad.append(f"block {b}: takes are not its starts")
        if sum(c["bytes"] for c in cs.values()) != s["seg_bytes"]:
            bad.append(f"block {b}: walked bytes are not its segments' bytes")
        for t, c in cs.items():
            if (
                c["exhausts"] != 1
                or c["tail"] + c["takes"] > c["bytes"]
                or c["copied"] > c["bytes"]
            ):
                bad.append(f"block {b}, thread {t}: a call's counts are inconsistent")
        total += s["seg_bytes"]
    if walked is not None and total != walked:
        bad.append(f"walked bytes {total:,}, expected M = {walked:,}")
    return by, bad


def predicates(by, walker, helpers):
    """The diagnostic's (a)-(d) on a VALID record set: {name: (held, detail)}"""
    calls = defaultdict(dict)
    for r in by["call"]:
        calls[(r["team"], r["block"])][r["thread"]] = r
    shares = defaultdict(set)
    for r in by["share"]:
        shares[(r["team"], r["block"])].add(r["thread"])
    teams = defaultdict(list)
    for s in by["summary"]:
        teams[s["team"]].append(s["block"])
    out = {}
    # (a) in every block of its team, each helper's walker calls equal its build shares
    a_fail = [
        (t, b, h)
        for t, blocks in teams.items()
        for b in blocks
        for h in range(1, helpers + 1)
        if (h in calls[(t, b)]) != (h in shares[(t, b)])
    ]
    # zero-work helper calls (present, found the queue exhausted at once): reported, not a fail
    zero = sum(1 for r in by["call"] if r["thread"] != 0 and r["bytes"] == 0)
    out["a"] = (
        not a_fail,
        (
            f"{len(a_fail)} helper-blocks without a matching call"
            if a_fail
            else "every helper walked every block it built"
        )
        + f"; {zero} zero-work helper calls",
    )
    # (b) helper-only, per team across its blocks: 3*max <= 2*sum, sum > 0
    b_detail, b_ok = [], True
    for t, blocks in sorted(teams.items()):
        hb = [
            sum(calls[(t, b)].get(h, {}).get("bytes", 0) for b in blocks)
            for h in range(1, helpers + 1)
        ]
        ok = sum(hb) > 0 and 3 * max(hb) <= 2 * sum(hb)
        b_ok &= ok
        b_detail.append(
            f"team {t}: helper bytes {hb}, largest share {max(hb) / sum(hb) if sum(hb) else 'n/a'}"
        )
    out["b"] = (b_ok, "; ".join(b_detail))
    # (c) at least one leader called the arm's walker
    leaders = sum(1 for cs in calls.values() if 0 in cs and cs[0]["walker"] == walker)
    out["c"] = (leaders > 0, f"{leaders} leader calls of {walker}")
    # (d) every call is the arm's walker
    other = [r for r in by["call"] if r["walker"] != walker]
    out["d"] = (not other, f"{len(other)} calls of another walker")
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("records")
    ap.add_argument("--walker", required=True, choices=sorted(WALKERS))
    ap.add_argument("--classes", required=True)
    ap.add_argument("--helpers", type=int, default=2)
    ap.add_argument("--walked", type=int)
    ap.add_argument(
        "--reference", action="store_true", help="the old arm: reported, never an outcome"
    )
    args = ap.parse_args()
    lines = [line for line in open(args.records).read().splitlines() if line.strip()]
    by, bad = validate(lines, classes_of(args.classes), args.helpers, args.walked)
    if bad:
        print("INVALID")
        for b in bad:
            print(" ", b)
        return 2
    print("VALID")
    preds = predicates(by, args.walker, args.helpers)
    for k, (held, detail) in preds.items():
        print(f"  ({k}) {'holds' if held else 'fails'}: {detail}")
    if args.reference:
        print("reference (the old arm): no outcome")
        return 0
    engaged = all(h for h, _ in preds.values())
    print("ENGAGED" if engaged else "NOT ENGAGED")
    return 0 if engaged else 1


if __name__ == "__main__":
    sys.exit(main())
