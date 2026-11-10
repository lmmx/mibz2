"""The combined team-ports diagnostic's checker-step adjudicator (§8a; DRAFT, not frozen).

docs/journal/2026-11-10-team-ports-combined-registration.md, §2 (the checker pin), §3 (the
deadlines and the checker's one exit exception), §8 (the acceptance rule) and §8a (this
adjudication). It runs the pinned walk-stats checker on one E arm's records, archives the
checker's exit code, stdout and stderr verbatim, and decides the step by §8a's three ordered
rules; the first rule that applies decides:

1. execution failure, rejected: a timeout, a kill by a signal, or an exit other than 0 or 1
   (exit 2 is the checker's INVALID: a completed verdict that fails structural validity);
2. a complete, parsed verdict, or rejected: empty stderr; stdout exactly `VALID`, then the
   (a), (b), (c), (d) lines in that order, each `  (x) holds: …` or `  (x) fails: …`, then
   `ENGAGED` or `NOT ENGAGED`, and nothing else; the outcome ENGAGED exactly when all four
   hold; the exit 0 for ENGAGED and 1 for NOT ENGAGED;
3. the engagement component: accepted exactly when the verdict is VALID and (a), (c) and (d)
   each read `holds`. (b), the outcome and the exit code are reported as the checker gave them.

The checker's outcome is never relabelled: the decision is about this diagnostic's narrower
engagement component, recorded beside the checker's own ENGAGED or NOT ENGAGED. No exit
status is discarded, and `--reference` is never passed.

Before any run, §2's identity check: the checker is copied out of git with
`git cat-file -p <blob>`, and `git hash-object` of the copy must equal the pin. A mismatch is
a failed checker step: the checker is not run and no other checker is substituted.

    # materialise and verify the pinned checker (once, before the first E arm)
    python3 bench/team_ports_adjudicate.py checker --out DIR
    # run one E arm's checker step and adjudicate it (cwd DIR; paths relative to DIR)
    python3 bench/team_ports_adjudicate.py run --dir DIR --step E1 \\
        --records e1.jsonl --classes wm-classes.txt
    # adjudicate an archived outcome (exit code, stdout and stderr files)
    python3 bench/team_ports_adjudicate.py judge --exit-code N --stdout F --stderr F \\
        [--timed-out] [--signal N]

`run` exits 0 when the step is accepted, 1 when it is rejected, 2 on a usage error or an
identity mismatch. The decision record is written in every case.
"""

import argparse
import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

CHECKER_BLOB = "eb7c392d2d766e870de3e14a31b5fe940787dd3f"
CHECKER_NAME = "walk_stats_check.py"
DEADLINE_S = 600  # §3: count.py and the checker, each
KILL_AFTER_S = 30  # §3: timeout -s TERM -k 30
SCHEMA = "team-ports-combined/checker-step/1"
CLAIM_LIMIT = (
    "presence and call integrity only (§8, the claim limit): no claim that any helper did "
    "useful walk work, that the team ran balanced, or of any parallel speedup"
)
PREDICATE = re.compile(r"  \(([abcd])\) (holds|fails): (.+)")
OUTCOMES = ("ENGAGED", "NOT ENGAGED")


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def git_hash_object(path, repo):
    return subprocess.run(
        [
            "git",
            "-C",
            str(Path(repo).resolve()),
            "hash-object",
            "--no-filters",
            str(Path(path).resolve()),
        ],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def materialise_checker(out_dir, repo, blob=CHECKER_BLOB):
    """Copy the pinned blob out with `git cat-file -p` and check its `git hash-object`.

    Returns (path, record); the record's `matches` is False on a mismatch or a missing blob,
    and the path is then not to be run."""
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / CHECKER_NAME
    p = subprocess.run(["git", "-C", str(repo), "cat-file", "-p", blob], capture_output=True)
    record = dict(
        blob=blob, cat_file_exit=p.returncode, cat_file_stderr=p.stderr.decode(errors="replace")
    )
    if p.returncode != 0:
        record.update(hash_object=None, matches=False)
        return path, record
    path.write_bytes(p.stdout)
    got = git_hash_object(path, repo)
    record.update(hash_object=got, matches=got == blob, sha256=sha256(p.stdout))
    return path, record


def checker_command(checker, records, classes):
    """§8's checker command, exactly: never `--reference`"""
    return [
        "python3",
        str(checker),
        str(records),
        "--walker",
        "traverse",
        "--helpers",
        "2",
        "--classes",
        str(classes),
    ]


def _children(pid):
    out = []
    for task in Path("/proc").glob("[0-9]*"):
        try:
            fields = (task / "stat").read_text().rsplit(")", 1)[1].split()
        except OSError:
            continue
        if int(fields[1]) == pid:
            out.append(int(task.name))
    return out


def run_checker(cmd, cwd, deadline_s=DEADLINE_S, kill_after_s=KILL_AFTER_S, _inject_signal=None):
    """Run cmd in its own process group under `timeout -s TERM -k KILL DEADLINE` (§3).

    Returns the raw outcome: exit code (negative for a signal, as subprocess reports it),
    stdout and stderr as bytes, whether the deadline expired, and the signal if any.

    `_inject_signal` (the dry run's fault injection only, never the production path): a
    signal sent to the checker process, under `timeout`, once it has started."""
    full = ["timeout", "-s", "TERM", "-k", str(kill_after_s), str(deadline_s), *cmd]
    start = time.monotonic()
    proc = subprocess.Popen(
        full,
        cwd=cwd,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    if _inject_signal is not None:
        kids = []
        for _ in range(500):
            kids = _children(proc.pid)
            if kids:
                break
            time.sleep(0.01)
        time.sleep(0.2)  # let the interpreter start
        for k in kids:
            os.kill(k, _inject_signal)
    backstop = False
    try:
        out, err = proc.communicate(timeout=deadline_s + kill_after_s + 60)
    except subprocess.TimeoutExpired:  # `timeout` itself failed to end it: kill the group
        backstop = True
        os.killpg(proc.pid, signal.SIGKILL)
        out, err = proc.communicate()
    elapsed = time.monotonic() - start
    rc = proc.returncode
    # `timeout` exits 124 when the deadline expired and TERM ended the command, and 137
    # (128 + KILL) when KILL followed; a command killed by any other signal makes `timeout`
    # re-raise that signal on itself, which subprocess reports as a negative code
    timed_out = backstop or (rc in (124, 137) and elapsed >= deadline_s)
    sig = -rc if rc < 0 else None
    return dict(
        command=full,
        exit=rc,
        stdout=out,
        stderr=err,
        timed_out=timed_out,
        signal=sig,
        elapsed_s=round(elapsed, 3),
    )


def parse_verdict(stdout):
    """§8a rule 2's grammar. Returns (verdict, reasons): verdict None unless complete."""
    try:
        text = stdout.decode("utf-8")
    except UnicodeDecodeError:
        return None, ["stdout is not UTF-8"]
    if not text.endswith("\n"):
        return None, ["stdout does not end with a complete line (truncated)"]
    lines = text[:-1].split("\n")
    if any("\r" in line for line in lines):
        return None, ["stdout has a carriage return"]
    if len(lines) != 6:
        return None, [f"stdout has {len(lines)} lines, a complete verdict has 6"]
    reasons = []
    if lines[0] != "VALID":
        reasons.append(f"line 1 is {lines[0]!r}, not 'VALID'")
    preds = {}
    for letter, line in zip("abcd", lines[1:5]):
        m = PREDICATE.fullmatch(line)
        if not m or m[1] != letter:
            reasons.append(f"the ({letter}) line is {line!r}")
            continue
        preds[letter] = dict(holds=m[2] == "holds", detail=m[3])
    if lines[5] not in OUTCOMES:
        reasons.append(f"line 6 is {lines[5]!r}, not ENGAGED or NOT ENGAGED")
    if reasons:
        return None, reasons
    return dict(validity="VALID", predicates=preds, outcome=lines[5]), []


def _reported(verdict):
    """What §8 reports beside the decision: (b) and the zero-work count, never gated"""
    a = verdict["predicates"]["a"]["detail"]
    m = re.search(r"; (\d+) zero-work helper calls$", a)
    b = verdict["predicates"]["b"]
    return dict(
        b=dict(holds=b["holds"], detail=b["detail"]),
        undefined_ratio_teams=len(re.findall(r"largest share n/a", b["detail"])),
        zero_work_helper_calls=int(m[1]) if m else None,
    )


def adjudicate(exit_code, stdout, stderr, timed_out=False, sig=None):
    """§8a: the decision for one checker step, from its raw outcome.

    exit_code: int (negative for a signal, as subprocess reports it); stdout, stderr: bytes.
    Returns the decision's fields: decision (accepted or rejected), rule (1, 2 or 3),
    reasons, the parsed verdict (None unless rule 2 passed) and what is reported."""
    if sig is None and exit_code < 0:
        sig = -exit_code
    # rule 1: execution failure
    r1 = []
    if timed_out:
        r1.append("the step timed out (§3's deadline)")
    if sig is not None:
        r1.append(f"the step was killed by signal {sig}")
    if exit_code not in (0, 1):
        r1.append(
            "exit 2: the checker's INVALID, a verdict that fails structural validity"
            if exit_code == 2 and not timed_out and sig is None
            else f"exit {exit_code} is neither 0 nor 1"
        )
    if r1:
        return dict(decision="rejected", rule=1, reasons=r1, verdict=None, reported=None)
    # rule 2: a complete, parsed verdict
    r2 = []
    if stderr != b"":
        r2.append(f"stderr is not empty ({len(stderr)} bytes)")
    verdict, why = parse_verdict(stdout)
    r2 += why
    if verdict is not None:
        all_hold = all(p["holds"] for p in verdict["predicates"].values())
        if (verdict["outcome"] == "ENGAGED") != all_hold:
            r2.append(
                f"the outcome {verdict['outcome']} disagrees with the predicates "
                f"({'all hold' if all_hold else 'not all hold'})"
            )
        want = 0 if verdict["outcome"] == "ENGAGED" else 1
        if exit_code != want:
            r2.append(
                f"exit {exit_code} disagrees with the outcome {verdict['outcome']} (exit {want})"
            )
    if r2:
        return dict(decision="rejected", rule=2, reasons=r2, verdict=None, reported=None)
    # rule 3: the engagement component
    preds = verdict["predicates"]
    failing = [k for k in "acd" if not preds[k]["holds"]]
    reported = _reported(verdict)
    if failing:
        return dict(
            decision="rejected",
            rule=3,
            reasons=[f"required predicate ({k}) fails" for k in failing],
            verdict=verdict,
            reported=reported,
        )
    return dict(
        decision="accepted",
        rule=3,
        reasons=["VALID, and (a), (c) and (d) hold; (b), the outcome and the exit are reported"],
        verdict=verdict,
        reported=reported,
    )


def record(step, raw, decision, checker=None, inputs=None):
    """The machine-readable decision record (the raw bytes are archived beside it)"""
    return dict(
        schema=SCHEMA,
        step=step,
        checker=checker,
        inputs=inputs,
        command=raw.get("command"),
        status=dict(
            exit=raw["exit"],
            timed_out=raw["timed_out"],
            signal=raw["signal"],
            elapsed_s=raw.get("elapsed_s"),
        ),
        stdout=dict(
            bytes=len(raw["stdout"]), sha256=sha256(raw["stdout"]), file="checker-stdout.txt"
        ),
        stderr=dict(
            bytes=len(raw["stderr"]), sha256=sha256(raw["stderr"]), file="checker-stderr.txt"
        ),
        checker_outcome=decision["verdict"]["outcome"] if decision["verdict"] else None,
        checker_exit=raw["exit"],
        decision=decision["decision"],
        rule=decision["rule"],
        reasons=decision["reasons"],
        verdict=decision["verdict"],
        reported=decision["reported"],
        claim_limit=CLAIM_LIMIT if decision["decision"] == "accepted" else None,
    )


def archive(step_dir, raw, rec):
    """Write the raw outcome verbatim and the decision record into step_dir"""
    step_dir = Path(step_dir)
    step_dir.mkdir(parents=True, exist_ok=True)
    (step_dir / "checker-stdout.txt").write_bytes(raw["stdout"])
    (step_dir / "checker-stderr.txt").write_bytes(raw["stderr"])
    (step_dir / "checker-exit.txt").write_text(f"{raw['exit']}\n")
    (step_dir / "decision.json").write_text(json.dumps(rec, indent=2) + "\n")


def run_step(base, step, records, classes, deadline_s=DEADLINE_S, _inject_signal=None):
    """One E arm's checker step: identity check, run, archive, adjudicate.

    base holds the checker copy; records and classes are paths relative to base. The raw
    outcome and the decision go to base/<step>/. Returns the decision record."""
    base = Path(base)
    checker = base / CHECKER_NAME
    identity = dict(blob=CHECKER_BLOB, path=CHECKER_NAME)
    identity["hash_object"] = git_hash_object(checker, base) if checker.exists() else None
    identity["matches"] = identity["hash_object"] == CHECKER_BLOB
    inputs = {}
    for name, rel in (("records", records), ("classes", classes)):
        p = base / rel
        inputs[name] = dict(
            path=str(rel),
            sha256=sha256(p.read_bytes()) if p.is_file() else None,
            kind="file" if p.is_file() else ("fifo" if p.is_fifo() else "missing"),
        )
    if not identity["matches"]:
        raw = dict(command=None, exit=None, stdout=b"", stderr=b"", timed_out=False, signal=None)
        decision = dict(
            decision="rejected",
            rule=0,
            reasons=[f"§2: the checker copy is not blob {CHECKER_BLOB}; not run, none substituted"],
            verdict=None,
            reported=None,
        )
        rec = record(step, raw, decision, identity, inputs)
        rec["status"]["exit"] = None
        archive(base / step, dict(raw, exit="not run"), rec)
        return rec
    cmd = checker_command(CHECKER_NAME, records, classes)
    raw = run_checker(cmd, base, deadline_s, _inject_signal=_inject_signal)
    decision = adjudicate(
        raw["exit"], raw["stdout"], raw["stderr"], raw["timed_out"], raw["signal"]
    )
    identity["python3"] = shutil.which("python3")
    identity["environment"] = {k: v for k, v in os.environ.items() if k.startswith("PYTHON")}
    rec = record(step, raw, decision, identity, inputs)
    archive(base / step, raw, rec)
    return rec


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("checker", help="copy out and verify the pinned checker")
    c.add_argument("--out", required=True)
    c.add_argument("--repo", default=str(Path(__file__).resolve().parent.parent))
    r = sub.add_parser("run", help="run and adjudicate one checker step")
    r.add_argument("--dir", required=True, help="holds the verified checker copy")
    r.add_argument("--step", required=True, choices=["E1", "E2"])
    r.add_argument("--records", required=True, help="relative to --dir")
    r.add_argument("--classes", required=True, help="relative to --dir")
    j = sub.add_parser("judge", help="adjudicate an archived outcome")
    j.add_argument("--exit-code", type=int, required=True)
    j.add_argument("--stdout", required=True)
    j.add_argument("--stderr", required=True)
    j.add_argument("--timed-out", action="store_true")
    j.add_argument("--signal", type=int)
    args = ap.parse_args(argv)
    if args.cmd == "checker":
        path, rec = materialise_checker(args.out, args.repo)
        print(json.dumps(dict(rec, path=str(path)), indent=2))
        return 0 if rec["matches"] else 2
    if args.cmd == "run":
        rec = run_step(args.dir, args.step, args.records, args.classes)
        print(json.dumps(rec, indent=2))
        if rec["rule"] == 0:
            return 2
        return 0 if rec["decision"] == "accepted" else 1
    raw = dict(
        exit=args.exit_code,
        stdout=Path(args.stdout).read_bytes(),
        stderr=Path(args.stderr).read_bytes(),
        timed_out=args.timed_out,
        signal=args.signal,
    )
    decision = adjudicate(
        raw["exit"], raw["stdout"], raw["stderr"], raw["timed_out"], raw["signal"]
    )
    print(json.dumps(record("judge", raw, decision), indent=2))
    return 0 if decision["decision"] == "accepted" else 1


if __name__ == "__main__":
    sys.exit(main())
