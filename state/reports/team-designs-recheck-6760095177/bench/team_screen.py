"""The forced-team screen's checker (DRAFT stub: not frozen, not adopted, never run on a decode).

docs/journal/2026-11-10-team-screen-design-draft.md (the Guardian's D083; the Researcher's
REVISE, #145 6759658271). Every assertion is mandatory and comes from the versioned
manifest (bench/team-screen/manifest.json: the standing semantic roles, arms, fixtures,
limits) and the PR's pinned delta (bench/team-screen/deltas/PR.json: its selector and
walker), never from free-form arguments. Every failed assertion is named; the exit status
is 1 on any.

    python3 bench/team_screen.py outputs MANIFEST DELTA RECORDS_DIR
    python3 bench/team_screen.py check MANIFEST DELTA ARM PREFIX --binary AB [--table OUT.json]

- `outputs` (A1, an admission contract): RECORDS_DIR holds one wrapper a (arm, fixture),
  `ARM__FIXTURE.json`: {arm, fixture, build, build_sha, exe_sha256, env, exit_status,
  signal, record}, `record` the harness child's JSON. Every expected pair is present
  exactly once; every wrapper and record has its schema; valid fixtures succeed with the
  oracle's length and hash; error fixtures fail with the manifest's kind and block, which
  the harness emits as structured fields (`err_kind`, `err_block`: examples/ab.rs's child,
  from the reader's `mibz2::Error`), and the promised prefix (length and hash); matched
  arms agree on (exit, out, hash, kind, block).
- `check` (the full tier's A2-A4: one team arm's callgrind profiles, matched to that arm's
  own binary and source, one file a thread): the leader and
  both helpers identified by role; the required events on those same threads; forbidden
  walkers absent; every named symbol attributable in the binary (an inlined or missing
  symbol is a failure, never a zero); conservation of walked bytes and claims; and, with
  `--table`, the per-thread Ir table with Ir/D (decoded bytes) and Ir/M (walked bytes) apart.

No timing is read, and nothing here is an acceptance figure.
"""

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import team_gap_count as tg  # noqa: E402

WRAPPER = {
    "arm",
    "fixture",
    "build",
    "build_sha",
    "exe_sha256",
    "env",
    "exit_status",
    "signal",
    "record",
}
RECORD = {"out", "hash", "err", "err_kind", "err_block"}
HASH = re.compile(r"^[0-9a-f]{16}$")
SHA = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")


def outputs_check(manifest, delta, wrappers):
    """A1 on already-loaded wrappers: a list of failure messages (empty: admitted)"""
    failed = []
    arms = {a["id"]: a for a in manifest["arms"]}
    fixtures = {f["id"]: f for f in manifest["fixtures"]}
    seen = {}
    for w in wrappers:
        missing = WRAPPER - set(w)
        if missing:
            failed.append(f"schema: a wrapper lacks {sorted(missing)}")
            continue
        key = (w["arm"], w["fixture"])
        if key[0] not in arms or key[1] not in fixtures:
            failed.append(f"unexpected record {key}")
            continue
        if key in seen:
            failed.append(f"duplicate record {key}")
            continue
        seen[key] = w
        rmiss = RECORD - set(w["record"] or {})
        if rmiss:
            failed.append(f"{key}: record lacks {sorted(rmiss)}")
        if not SHA.match(str(w["build_sha"])) or not SHA256.match(str(w["exe_sha256"])):
            failed.append(f"{key}: build identity malformed")
        if w["build_sha"] != delta.get(arms[key[0]]["build"]):
            failed.append(
                f"{key}: built from {w['build_sha']}, not the delta's {arms[key[0]]['build']}"
            )
        want_env = (
            delta["head_team_env"]
            if arms[key[0]]["env"] == "delta.head_team_env"
            else arms[key[0]]["env"]
        )
        if w["env"] != want_env:
            failed.append(f"{key}: environment {w['env']} is not {want_env}")
        if w["signal"] not in (None, 0):
            failed.append(f"{key}: killed by signal {w['signal']}")
    for a in arms:
        for f in fixtures:
            if (a, f) not in seen:
                failed.append(f"missing record {(a, f)}")
    tuples = {}
    for (a, f), w in seen.items():
        r = w["record"] or {}
        if not RECORD <= set(r):
            continue
        fx = fixtures[f]
        exp = fx["expect"]
        kind, block = r["err_kind"], r["err_block"]
        if not HASH.match(str(r["hash"])):
            failed.append(f"{(a, f)}: hash malformed")
        if fx["kind"] == "valid":
            if w["exit_status"] != 0 or r["err"]:
                failed.append(
                    f"{(a, f)}: a valid input failed (exit {w['exit_status']}, err {r['err']!r})"
                )
            if r["out"] != exp["out"] or r["hash"] != exp["hash"]:
                failed.append(f"{(a, f)}: not the oracle's output ({r['out']}, {r['hash']})")
        else:
            if not r["err"] or w["exit_status"] is None:
                failed.append(f"{(a, f)}: an error fixture succeeded")
            if kind != exp["kind"] or block is None or block != exp["block"]:
                failed.append(
                    f"{(a, f)}: error {kind} at block {block}, expected {exp['kind']} at {exp['block']}"
                )
            if r["out"] != exp["out"] or r["hash"] != exp["hash"]:
                failed.append(f"{(a, f)}: not the promised prefix ({r['out']}, {r['hash']})")
        tuples.setdefault(f, {})[a] = (w["exit_status"], r["out"], r["hash"], kind, block)
    for f, by_arm in tuples.items():
        if len(set(by_arm.values())) > 1:
            failed.append(f"{f}: the matched arms differ: {by_arm}")
    if not wrappers:
        failed.append("no records")
    return failed


def symbols(binary):
    out = subprocess.run(["nm", "-C", binary], capture_output=True, text=True, check=True).stdout
    return [ln.split(" ", 2)[-1] for ln in out.splitlines() if " t " in ln or " T " in ln]


def calls_to(profile, rx):
    return sum(c for (_, callee), c in profile.calls.items() if callee and re.search(rx, callee))


def check_profiles(manifest, delta, arm, profs, syms, walker_counts, fixture="W"):
    """A2 and A3 on one team arm's profiles: a list of failure messages (empty: admitted).
    `walker_counts(profile)` gives {walker regex: {calls, claims, takes, walked}}"""
    failed = []
    roles = manifest["roles"]
    counted = next(f for f in manifest["fixtures"] if f["id"] == fixture)["counted"]
    walker = delta["walker"][arm]
    forbid = delta["forbid"][arm]
    named = [
        roles["leader"]["identify_by"],
        roles["leader"]["solo"],
        roles["helper"]["identify_by"],
    ]
    named += [roles["helper"]["relay"], walker] + forbid
    for rx in named:
        if not any(re.search(rx, s) for s in syms):
            failed.append(f"A2: {rx} has no symbol in the binary: missing attribution, not zero")
    ids = [p.thread for p in profs]
    if len(set(ids)) != len(ids) or not profs:
        failed.append(f"A2: thread profiles missing or not unique: {ids}")
    helpers = [p for p in profs if calls_to(p, roles["helper"]["identify_by"]) > 0]
    leaders = [p for p in profs if calls_to(p, roles["leader"]["identify_by"]) > 0]
    if len(helpers) != roles["helper"]["count"]:
        failed.append(f"A2: {len(helpers)} helper threads, expected {roles['helper']['count']}")
    if len(leaders) != roles["leader"]["count"]:
        failed.append(f"A2: {len(leaders)} leader threads, expected {roles['leader']['count']}")
    if {p.thread for p in helpers} & {p.thread for p in leaders}:
        failed.append("A2: a thread is both leader and helper")
    blocks = counted["coop_blocks"]
    walked = claims = takes = calls = 0
    for p in profs:
        w = walker_counts(p)
        for rx in forbid:
            if w.get(rx, {}).get("calls", 0) or calls_to(p, rx):
                failed.append(f"A2: forbidden {rx} executed on thread {p.thread}")
        c = w.get(walker, {"calls": 0, "claims": 0, "takes": 0, "walked": 0})
        walked += c["walked"]
        claims += c["claims"]
        takes += c["takes"]
        calls += c["calls"]
    for h in helpers:
        c = walker_counts(h).get(walker, {"calls": 0, "takes": 0})
        shares = calls_to(h, roles["helper"]["identify_by"])
        relay = calls_to(h, roles["helper"]["relay"])
        if not (shares == c["calls"] == relay == blocks):
            failed.append(
                f"A3: helper {h.thread}: shares {shares}, {walker} calls {c['calls']}, relay {relay}; expected {blocks} each"
            )
        if c["takes"] < roles["helper"]["takes_min"]:
            failed.append(f"A3: helper {h.thread} claimed no segment in the whole run")
    for ld in leaders:
        solo = calls_to(ld, roles["leader"]["solo"])
        if solo != counted["solo_blocks"]:
            failed.append(
                f"A3: leader {ld.thread}: {solo} solo walks, expected {counted['solo_blocks']}"
            )
        joins = walker_counts(ld).get(walker, {"calls": 0})["calls"]
        if joins < roles["leader"]["join_walker_calls_min"]:
            failed.append(f"A3: leader {ld.thread} never joined a walk (the claimed join path)")
    if walked != counted["walked"]:
        failed.append(f"A3: walked bytes {walked:,}, expected {counted['walked']:,}")
    if takes != counted["takes"]:
        failed.append(f"A3: takes {takes:,}, expected {counted['takes']:,}")
    if claims != takes + calls:
        failed.append(f"A3: claims {claims:,} != takes {takes:,} + walker calls {calls:,}")
    return failed


def table(profs, walker_counts, decoded, walked):
    """A4: per thread and per stage, raw Ir, Ir/D and the walk's Ir/M; diagnostic only"""
    rows, tot = [], {}
    for p in profs:
        st, _ = tg.stages(p)
        rows.append(
            dict(thread=p.thread, total_ir=p.total, stages=dict(st), walkers=walker_counts(p))
        )
        for k, v in st.items():
            tot[k] = tot.get(k, 0) + v
    return dict(
        label="serialised (callgrind), schedule-dependent diagnostic; no threshold",
        D_decoded_bytes=decoded,
        M_walked_bytes=walked,
        threads=rows,
        stage_ir=tot,
        stage_ir_per_D={k: v / decoded for k, v in tot.items()},
        process_ir=sum(p.total for p in profs),
        process_ir_per_D=sum(p.total for p in profs) / decoded,
        inversion_ir_per_D=sum(tot.get(k, 0) for k in ("build", "walk", "assembly")) / decoded,
        walk_ir_per_M=tot.get("walk", 0) / walked,
        unattributed_ir=tot.get("other", 0),
    )


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    o = sub.add_parser("outputs")
    o.add_argument("manifest")
    o.add_argument("delta")
    o.add_argument("records")
    c = sub.add_parser("check")
    c.add_argument("manifest")
    c.add_argument("delta")
    c.add_argument("arm")
    c.add_argument("prefix")
    c.add_argument("--binary", required=True)
    c.add_argument(
        "--coop-src", required=True, help="the arm's own source's coop.rs, never another build's"
    )
    c.add_argument("--table")
    args = ap.parse_args()
    manifest = json.load(open(args.manifest))
    delta = json.load(open(args.delta))
    if args.cmd == "outputs":
        wrappers = []
        for name in sorted(os.listdir(args.records)):
            if name.endswith(".json"):
                with open(os.path.join(args.records, name)) as f:
                    wrappers.append(json.load(f))
        failed = outputs_check(manifest, delta, wrappers)
    else:
        profs = tg.load(args.prefix)
        dis = tg.disasm(args.binary, r"coop::traverse")
        lines = tg.coop_lines(args.coop_src)

        def walker_counts(p):
            out = {}
            for v, c in tg.walker_counts(p, dis, lines).items():
                rx = "coop::traverse_staged" if v == "staged" else manifest["default_walker"]
                out[rx] = dict(
                    calls=c["calls"],
                    claims=c["claims"],
                    takes=c["walked"]["take"],
                    walked=sum(c["walked"].values()),
                )
            return out

        failed = check_profiles(
            manifest, delta, args.arm, profs, symbols(args.binary), walker_counts
        )
        if args.table:
            counted = next(f for f in manifest["fixtures"] if f["id"] == "W")["counted"]
            with open(args.table, "w") as f:
                json.dump(
                    table(profs, walker_counts, counted["decoded"], counted["walked"]), f, indent=1
                )
    for msg in failed:
        print("FAIL", msg)
    print("team screen", args.cmd + ":", "FAIL" if failed else "admitted")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
