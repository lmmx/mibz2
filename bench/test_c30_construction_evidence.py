"""The collector's own rejection of bad dry-run evidence (G9, report 197), on the real
binary, by direct invocation, without any timing: every case either stops at the evidence
check, before the decode or any build, or (the positive control) stops after verifying it
with --verify-evidence-only.

    python3 -I bench/test_c30_construction_evidence.py BINARY CORPUS DRY_RUN_DIR CPUS WORK_DIR
        [SUBCOMMAND]

DRY_RUN_DIR is a completed dry run's directory, as the recipe leaves it (its records,
analysis and manifest). Each case runs `cbuild ... --rounds 20` (a capture's arguments) on
a copy of it, mutated, and requires exit status 3, a final `failure` record whose `what` is
`evidence`, and no build record. Mutations marked "resealed" also recompute the manifest's
checksums, as a consistent forger would.

SUBCOMMAND is `cbuild` (draft 4, the default) or `cbuild-v2` (constructor v2's protocol,
report 211), whose DRY_RUN_DIR is a v2 dry run's. With `cbuild-v2`, three more cases: the
dry run's records or analysis without v2's identifier, and another protocol's, must each be
refused (a v1-mode dry run of the same binary is not v2's evidence)."""

import hashlib
import json
import os
import shutil
import subprocess
import sys

BINARY, CORPUS, DRY, CPUS, WORK = sys.argv[1:6]
SUB = sys.argv[6] if len(sys.argv) > 6 else "cbuild"
JL, AN, MF = "c30-construction.jsonl", "c30-analysis.json", "MANIFEST.json"


def records(path):
    try:
        return [json.loads(x) for x in open(path) if x.strip()]
    except OSError:
        return []


def run(name, evidence, extra=(), cpus=CPUS, corpus=CORPUS, capture=True):
    out = os.path.join(WORK, f"{name}.jsonl")
    args = [BINARY, SUB, out, corpus, "--rounds", "20", "--cpus", cpus, *extra]
    if not capture:
        args.append("--dry-run")
    if evidence is not None:
        args += ["--dry-run-evidence", evidence]
    p = subprocess.run(args, capture_output=True, text=True)
    return p.returncode, records(out), p.stdout + p.stderr


def copy(name):
    d = os.path.join(WORK, name)
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    for f in (JL, AN, MF):
        shutil.copy(os.path.join(DRY, f), d)
    return d


def reseal(d):
    m = json.load(open(os.path.join(d, MF)))
    for f in m["files"]:
        p = os.path.join(d, f["path"])
        if os.path.exists(p):
            f["sha256"] = hashlib.sha256(open(p, "rb").read()).hexdigest()
    json.dump(m, open(os.path.join(d, MF), "w"), indent=1)


def edit_jsonl(d, fn):
    p = os.path.join(d, JL)
    rs = [json.loads(x) for x in open(p)]
    rs = [r for r in (fn(r) for r in rs) if r is not None]
    open(p, "w").write("".join(json.dumps(r) + "\n" for r in rs))


def edit_json(d, f, fn):
    p = os.path.join(d, f)
    v = json.load(open(p))
    fn(v)
    json.dump(v, open(p, "w"), indent=1)


def set_kind(kind, **kw):
    def fn(r):
        if r.get("kind") == kind:
            r.update(kw)
        return r

    return fn


def rejected(name, evidence, **kw):
    code, rs, log = run(name, evidence, **kw)
    last = rs[-1] if rs else {}
    ok = (
        code == 3
        and last.get("kind") == "failure"
        and last.get("what") == "evidence"
        and not any(r.get("kind") == "build" for r in rs)
        and not any(r.get("kind") == "meta" for r in rs)
    )
    assert ok, (name, code, last, log[-400:])
    print(f"ok {name}: rejected ({last.get('detail', '')[-110:]})")


def main():
    os.makedirs(WORK, exist_ok=True)
    peak = next(r for r in records(os.path.join(DRY, JL)) if r["kind"] == "postflight")["vm_hwm"]

    # the positive control: the unmodified evidence verifies, and nothing is built
    code, rs, log = run("valid", DRY, extra=["--verify-evidence-only"])
    kinds = [r["kind"] for r in rs]
    pre = next(r for r in rs if r["kind"] == "preflight")
    assert code == 0 and kinds == ["preflight", "evidence_verified"], (code, kinds, log[-400:])
    assert pre["verified_peak"] == peak and pre["required"] == max(pre["estimate"], peak)
    print(
        f"ok valid: verified, peak {peak} recorded, required = max(estimate, peak), nothing built"
    )

    rejected("missing", None)
    rejected("missing-dir", os.path.join(WORK, "no-such-dir"))
    code, rs, _ = run("dry-run-given-evidence", DRY, capture=False)
    assert code == 3 and rs[-1].get("what") == "evidence", code
    print("ok dry-run-given-evidence: rejected")

    d = copy("tiny-peak")
    edit_jsonl(d, set_kind("postflight", vm_hwm=1))
    edit_json(d, AN, lambda v: v["memory"].update(peak_rss_vm_hwm=1))
    reseal(d)
    rejected("tiny-peak (resealed)", d)

    d = copy("peak-disagrees")
    edit_json(d, AN, lambda v: v["memory"].update(peak_rss_vm_hwm=peak + 1))
    reseal(d)
    rejected("peak-disagrees (resealed)", d)

    d = copy("edited-records")
    edit_jsonl(d, set_kind("postflight", vm_hwm=peak * 2))
    rejected("edited-records (checksum)", d)

    d = copy("stale-binary")
    edit_json(d, MF, lambda v: v["binary"].update(sha256="0" * 64))
    rejected("stale-binary", d)

    d = copy("recorded-binary")
    edit_jsonl(d, set_kind("meta", binary_sha256="0" * 64))
    reseal(d)
    rejected("recorded-binary (resealed)", d)

    d = copy("other-machine")
    edit_jsonl(
        d, lambda r: (r["machine"].update(hostname="i9"), r)[1] if r["kind"] == "meta" else r
    )
    reseal(d)
    rejected("other-machine (resealed)", d)

    rejected("other-cpus", DRY, cpus=",".join(reversed(CPUS.split(","))))

    d = copy("failed-gate")
    edit_json(d, AN, lambda v: v["gates"].update(G6_exact_output=False))
    reseal(d)
    rejected("failed-gate (resealed)", d)

    d = copy("failed-schedule")
    edit_json(d, AN, lambda v: v["schedule"].update(S7_start_barrier=False))
    reseal(d)
    rejected("failed-schedule (resealed)", d)

    d = copy("injected")
    edit_jsonl(d, set_kind("meta", inject="pin:w4:1"))
    reseal(d)
    rejected("injected (resealed)", d)

    d = copy("not-a-dry-run")
    edit_jsonl(d, set_kind("meta", dry_run=False))
    reseal(d)
    rejected("not-a-dry-run (resealed)", d)

    d = copy("incomplete")
    edit_jsonl(d, lambda r: None if r["kind"] == "postflight" else r)
    reseal(d)
    rejected("incomplete (resealed)", d)

    d = copy("failure-record")
    p = os.path.join(d, JL)
    open(p, "a").write(json.dumps({"kind": "failure", "what": "output", "detail": "x"}) + "\n")
    reseal(d)
    rejected("failure-record (resealed)", d)

    d = copy("exit-nonzero")
    edit_json(d, MF, lambda v: v.update(exit_status=3))
    rejected("exit-nonzero", d)

    d = copy("tracked-changes")
    edit_json(d, MF, lambda v: v["source"].update(tracked_changes=["M src/bwt.rs"]))
    rejected("tracked-changes", d)

    d = copy("no-analysis")
    os.remove(os.path.join(d, AN))
    rejected("no-analysis", d)

    d = copy("other-corpus-hash")
    edit_json(d, MF, lambda v: v["corpora"][0].update(sha256="0" * 64))
    rejected("other-corpus-hash", d)

    d = copy("malformed")
    open(os.path.join(d, AN), "w").write("{not json")
    reseal(d)
    rejected("malformed (resealed)", d)

    if SUB == "cbuild-v2":
        d = copy("v1-records")
        edit_jsonl(
            d, lambda r: ({k: v for k, v in r.items() if k not in ("protocol", "candidate")}
                          if r["kind"] == "meta" else r)
        )  # fmt: skip
        reseal(d)
        rejected("v1-records (resealed)", d)

        d = copy("v1-analysis")
        edit_json(d, AN, lambda v: v.pop("protocol"))
        reseal(d)
        rejected("v1-analysis (resealed)", d)

        d = copy("other-protocol")
        edit_jsonl(d, set_kind("meta", protocol="c30-construction-v1"))
        reseal(d)
        rejected("other-protocol (resealed)", d)

    print("evidence tests passed")


if __name__ == "__main__":
    main()
