"""The forced-team screen's checker (DRAFT stub: not frozen, not adopted, never run on a decode).

docs/journal/2026-11-10-team-screen-design-draft.md (the Guardian's D083; the Researcher's
REVISE, #145 6759658271). Every assertion is mandatory and comes from the versioned
manifest (bench/team-screen/manifest.json: the standing semantic roles, arms, fixtures,
limits) and the PR's pinned delta (bench/team-screen/deltas/PR.json: its selector and
walker), never from free-form arguments. Every failed assertion is named; the exit status
is 1 on any.

    python3 bench/team_screen.py outputs MANIFEST DELTA ARTIFACTS RECORDS_DIR
    python3 bench/team_screen.py check MANIFEST DELTA ARM PREFIX --binary AB [--table OUT.json]

- `outputs` (A1, an admission contract): RECORDS_DIR holds one completion record a run
  the manifest implies (`expected_runs`): every arm on every fixture and read buffer, one
  oracle decode a valid fixture and a prefix oracle an error fixture (`ab child lbzip2 1`),
  one lbzip2 rejection an error fixture. Each is bound to its command, its input's digest,
  its environment and its executable's digest (ARTIFACTS, the build step's record), and
  its process status is judged apart from any decoder error in its record: an ab child
  returns normally and reports a decoder error in the record (schema ab-child/2:
  `err_kind`, `err_block`, null on success; `sha256` beside the 64-bit hash), so a nonzero
  exit, a timeout or a signal is a process failure. Valid fixtures give their oracle's
  length, hash and SHA-256 with null error fields; error fixtures give the frozen kind and
  block and their prefix oracle's output; the matched arms agree.
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
    "kind",
    "run",
    "fixture",
    "command",
    "input_sha256",
    "env",
    "exe_sha256",
    "exit_status",
    "signal",
    "timed_out",
    "record",
}
ARM_WRAPPER = WRAPPER | {"arm", "build", "build_sha", "read_buffer"}
RECORD = {"schema", "out", "hash", "sha256", "err", "err_kind", "err_block"}
SCHEMA = "ab-child/2"
HASH = re.compile(r"^[0-9a-f]{16}$")
SHA = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")


def expected_runs(manifest):
    """Every completion record the screen must admit: {run id: (kind, arm or None,
    fixture, read buffer or None)}. Arm runs for every arm, fixture and read buffer the
    manifest names; one oracle decode per valid fixture and per error fixture's prefix;
    one lbzip2 rejection per error fixture"""
    runs = {}
    for f in manifest["fixtures"]:
        for a in manifest["arms"]:
            if a.get("only") and a["only"] != f["kind"]:
                continue
            for buf in manifest["read_buffers"][f["kind"]]:
                runs[f"{a['id']}__{f['id']}__{buf}"] = ("arm", a["id"], f["id"], buf)
        runs[f"oracle__{f['id']}"] = ("oracle", None, f["id"], None)
        if f["kind"] == "error":
            runs[f"reject__{f['id']}"] = ("reject", None, f["id"], None)
    return runs


def _file(f, kind):
    return f["prefix_oracle"]["file"] if kind == "oracle" and f["kind"] == "error" else f["file"]


def _frozen(f, artifacts):
    """A fixture's input digest and expected output: frozen in the manifest, or, for a
    per-head input (the text corpus is built from the head's own sources), the run's
    artifacts record"""
    if f["expect"] == "PER-HEAD":
        per = artifacts["per_head_inputs"][f["id"]]
        return per["sha256"], per["expect"]
    return f["sha256"], f["expect"]


def outputs_check(manifest, delta, artifacts, wrappers):
    """A1, the admission contract, on already-loaded completion records: a list of
    failure messages (empty: admitted).

    `artifacts` is the build step's record: {head: {sha, exe_sha256}, base: {...},
    lbzip2: {exe_sha256}, per_head_inputs: {fixture: {sha256, expect}}}"""
    failed = []
    arms = {a["id"]: a for a in manifest["arms"]}
    fixtures = {f["id"]: f for f in manifest["fixtures"]}
    want = expected_runs(manifest)
    common = manifest["common_env"]
    seen = {}
    if not wrappers:
        failed.append("no records")
    for w in wrappers:
        run = w.get("run") if isinstance(w, dict) else None
        if run not in want:
            failed.append(f"unexpected record {run!r}")
            continue
        if run in seen:
            failed.append(f"duplicate record {run}")
            continue
        kind, arm_id, fx_id, buf = want[run]
        need = ARM_WRAPPER if kind == "arm" else WRAPPER
        missing = need - set(w)
        if missing:
            failed.append(f"{run}: wrapper lacks {sorted(missing)}")
            continue
        seen[run] = w
        f = fixtures[fx_id]
        if w["kind"] != kind or w["fixture"] != fx_id:
            failed.append(f"{run}: labelled {w['kind']}/{w['fixture']}")
        in_sha, _ = _frozen(f, artifacts)
        if kind == "oracle" and f["kind"] == "error":
            in_sha = f["prefix_oracle"]["sha256"]
        if w["input_sha256"] != in_sha:
            failed.append(f"{run}: input {w['input_sha256']} is not the fixture's {in_sha}")
        # the process, apart from any decoder error the record reports
        if w["timed_out"] or w["signal"] not in (None, 0):
            failed.append(
                f"{run}: process failure (timed out {w['timed_out']}, signal {w['signal']})"
            )
            continue
        if kind == "reject":
            if w["command"] != ["lbzip2", "-dc", f["file"]]:
                failed.append(f"{run}: command {w['command']}")
            if w["exe_sha256"] != artifacts["lbzip2"]["exe_sha256"]:
                failed.append(f"{run}: not the recorded lbzip2")
            if w["exit_status"] == 0:
                failed.append(f"{run}: lbzip2 accepted a malformed fixture")
            continue
        # ab children (arms and oracles) return normally, a decoder error in the record
        if w["exit_status"] != 0:
            failed.append(f"{run}: process failure (exit {w['exit_status']})")
            continue
        if kind == "oracle":
            build, argv, env = "head", ["child", "lbzip2", "1", _file(f, kind)], dict(common)
        else:
            a = arms[arm_id]
            build = a["build"]
            argv = ["child", a["arm"], str(a["threads"]), f["file"]]
            env = dict(common, AB_READ_BUF=str(buf))
            env.update(delta["head_team_env"] if a["env"] == "delta.head_team_env" else a["env"])
            if w["arm"] != arm_id or w["build"] != build or w["read_buffer"] != buf:
                failed.append(
                    f"{run}: labelled arm {w['arm']}, build {w['build']}, buffer {w['read_buffer']}"
                )
            if w["build_sha"] != delta.get(build) or not SHA.match(str(w["build_sha"])):
                failed.append(f"{run}: built from {w['build_sha']}, not the delta's {build}")
        if w["command"][1:] != argv or not str(w["command"][0]).endswith("ab"):
            failed.append(f"{run}: command {w['command']}, expected ab {argv}")
        if w["exe_sha256"] != artifacts[build]["exe_sha256"] or not SHA256.match(
            str(w["exe_sha256"])
        ):
            failed.append(f"{run}: executable {w['exe_sha256']} is not the {build} build's")
        if w["env"] != env:
            failed.append(f"{run}: environment {w['env']}, expected {env}")
        r = w["record"]
        if not isinstance(r, dict) or RECORD - set(r):
            failed.append(f"{run}: record lacks {sorted(RECORD - set(r or {}))}")
            continue
        if r["schema"] != SCHEMA:
            failed.append(
                f"{run}: record schema {r['schema']!r}: a harness older than {SCHEMA} is INCOMPATIBLE"
            )
        if not HASH.match(str(r["hash"])) or not SHA256.match(str(r["sha256"])):
            failed.append(f"{run}: hash or sha256 malformed")
    for run in want:
        if run not in seen:
            failed.append(f"missing record {run}")
    # the outcomes: oracles against their frozen values, arms against their oracles
    oracle = {}
    for run, w in seen.items():
        kind, arm_id, fx_id, buf = want[run]
        r = w["record"]
        if kind != "oracle" or not isinstance(r, dict) or RECORD - set(r):
            continue
        f = fixtures[fx_id]
        _, exp = _frozen(f, artifacts)
        got = {k: r[k] for k in ("out", "hash", "sha256")}
        if r["err"] or r["err_kind"] is not None or r["err_block"] is not None:
            failed.append(f"{run}: the oracle failed: {r['err']!r}")
        elif f["kind"] == "valid" and got != {k: exp[k] for k in got}:
            failed.append(f"{run}: the oracle gave {got}, frozen {exp}")
        elif f["kind"] == "error" and got != {k: exp[k] for k in got}:
            failed.append(f"{run}: the prefix oracle gave {got}, frozen {exp}")
        oracle[fx_id] = got
    by_fixture = {}
    for run, w in seen.items():
        kind, arm_id, fx_id, buf = want[run]
        r = w["record"]
        if kind != "arm" or not isinstance(r, dict) or RECORD - set(r) or w["exit_status"] != 0:
            continue
        f = fixtures[fx_id]
        exp = _frozen(f, artifacts)[1]
        got = {k: r[k] for k in ("out", "hash", "sha256")}
        if fx_id in oracle and got != oracle[fx_id]:
            failed.append(f"{run}: output {got} is not the oracle's {oracle[fx_id]}")
        if f["kind"] == "valid":
            if r["err"] or r["err_kind"] is not None or r["err_block"] is not None:
                failed.append(
                    f"{run}: a valid fixture reported an error ({r['err']!r}, {r['err_kind']}, {r['err_block']})"
                )
        elif not r["err"] or r["err_kind"] != exp["kind"] or r["err_block"] != exp["block"]:
            failed.append(
                f"{run}: error {r['err_kind']} at block {r['err_block']}, expected {exp['kind']} at {exp['block']}"
            )
        by_fixture.setdefault(fx_id, {})[run] = (
            r["out"],
            r["hash"],
            r["sha256"],
            r["err_kind"],
            r["err_block"],
        )
    for fx_id, runs in by_fixture.items():
        if len(set(runs.values())) > 1:
            failed.append(f"{fx_id}: the matched arms differ: {runs}")
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
    o.add_argument(
        "artifacts", help="the build step's record of the builds, lbzip2 and per-head inputs"
    )
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
        failed = outputs_check(manifest, delta, json.load(open(args.artifacts)), wrappers)
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
