"""The combined team-ports diagnostic's §12a adjudication dry run (untimed, synthetic only).

docs/journal/2026-11-10-team-ports-combined-registration.md §12a: §8a's adjudicator
(bench/team_ports_adjudicate.py) on synthetic walk-stats records, against the pinned checker
blob, with no decoder build or run and no timing. Each case's checker output, exit code and
§8a decision are archived under OUT/cases/<case>/, and OUT/summary.json lists every case's
expected and actual decision.

Two kinds of case:
- **real:** the pinned checker runs on a synthetic ledger through the adjudicator's own `run`
  path (deadline, process group, verbatim capture). The timeout reads a FIFO no one writes;
  the signals are sent to the checker process once it has started (the dry run's fault
  injection, which the production path never uses);
- **simulated:** an outcome no correct checker run produces (a truncated report, an exit
  that disagrees with its outcome, a stderr warning, an extra line), built from a real case's
  archived stdout and adjudicated by the same `adjudicate`.

The fixtures follow §12a: one T block, both helpers in the built set and share records, the
leader 100 walked bytes, exact conservation and terminal count.

    python3 bench/team_ports_dry_run.py OUT
"""

import json
import os
import signal
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import team_ports_adjudicate as adj  # noqa: E402

TRAVERSE = "traverse"


def ledger(calls, shares=(1, 2), terminal=True):
    """calls: {thread: (walker, bytes)}. One T block of team 0, built by helpers 1 and 2."""
    recs = [dict(kind="share", team=0, block=0, thread=h) for h in shares]
    for t, (walker, n) in sorted(calls.items()):
        recs.append(
            dict(
                kind="call",
                team=0,
                block=0,
                thread=t,
                role="leader" if t == 0 else "helper",
                walker=walker,
                takes=int(n > 0),
                exhausts=1,
                bytes=n,
                tail=0,
                fragments=0,
                copied=0,
            )
        )
    total = sum(n for _, n in calls.values())
    recs.append(
        dict(
            kind="summary",
            team=0,
            block=0,
            rows=total,
            starts=sum(n > 0 for _, n in calls.values()),
            seg_bytes=total,
            entered=sum(1 << t for t in calls),
            joined=0 in calls,
            built=sum(1 << h for h in shares),
        )
    )
    lines = [json.dumps(r) for r in recs]
    if terminal:
        lines.append(json.dumps(dict(kind="end", records=len(recs))))
    return "\n".join(lines) + "\n"


def team(leader, h1, h2, w1=TRAVERSE):
    return {0: (TRAVERSE, leader), 1: (w1, h1), 2: (TRAVERSE, h2)}


# (name, §12a row, description, how, expected decision, expected rule)
# how: ("ledger", text) | ("bytes", data) | ("fifo",) | ("signal", signum) | ("sim", fn)
def real_cases():
    return [
        (
            "balanced",
            "balanced, helpers 100/100",
            "VALID, all hold, ENGAGED, exit 0",
            ("ledger", ledger(team(100, 100, 100))),
            "accepted",
            3,
        ),
        (
            "balance-only-1-99",
            "balance-only, helpers 1/99",
            "VALID, only (b) fails, NOT ENGAGED, exit 1",
            ("ledger", ledger(team(100, 1, 99))),
            "accepted",
            3,
        ),
        (
            "zero-work-0-0",
            "zero-work, helpers 0/0",
            "VALID, (b) fails (ratio n/a), NOT ENGAGED, exit 1; the claim limit",
            ("ledger", ledger(team(100, 0, 0))),
            "accepted",
            3,
        ),
        (
            "a-fails",
            "a required predicate fails",
            "helper 1 built (share) but never called: (a) fails, VALID, exit 1",
            ("ledger", ledger({0: (TRAVERSE, 100), 2: (TRAVERSE, 50)})),
            "rejected",
            3,
        ),
        (
            "c-fails",
            "a required predicate fails",
            "the leader never joined (no leader call): (c) fails, VALID, exit 1",
            ("ledger", ledger({1: (TRAVERSE, 50), 2: (TRAVERSE, 50)})),
            "rejected",
            3,
        ),
        (
            "d-fails",
            "a required predicate fails",
            "helper 1 called traverse_staged, 50/50: (d) fails, VALID, exit 1",
            ("ledger", ledger(team(100, 50, 50, w1="traverse_staged"))),
            "rejected",
            3,
        ),
        (
            "invalid-no-terminal",
            "INVALID",
            "the 1/99 ledger without its terminal record: INVALID, exit 2",
            ("ledger", ledger(team(100, 1, 99), terminal=False)),
            "rejected",
            1,
        ),
        (
            "timeout",
            "a timeout",
            "records are a FIFO no one writes; a 2 s deadline (TERM, then KILL 30 s later)",
            ("fifo",),
            "rejected",
            1,
        ),
        (
            "signal-kill",
            "a kill by signal",
            "the checker, blocked on a FIFO, is sent SIGKILL",
            ("signal", signal.SIGKILL),
            "rejected",
            1,
        ),
        (
            "signal-term",
            "a kill by signal",
            "the checker, blocked on a FIFO, is sent SIGTERM",
            ("signal", signal.SIGTERM),
            "rejected",
            1,
        ),
        (
            "crash-traceback-exit1",
            "a crash: non-zero exit, a traceback, no verdict",
            "records that are not UTF-8: the checker's own UnicodeDecodeError traceback, exit 1",
            ("bytes", b"\xff\xfe not utf-8\n"),
            "rejected",
            2,
        ),
    ]


def _lines(text):
    return text.split("\n")[:-1]


def _join(lines):
    return ("\n".join(lines) + "\n").encode()


def simulated_cases():
    """Each fn takes {case: (exit, stdout bytes, stderr bytes)} of the real cases and returns
    (exit, stdout, stderr, timed_out, signal)."""

    def good(o):
        return o["balanced"][1]

    def bad(o):
        return o["balance-only-1-99"][1]

    def g(o):
        return _lines(good(o).decode())

    def b(o):
        return _lines(bad(o).decode())

    return [
        # §12a: truncated or partial
        (
            "truncated-no-outcome-exit0",
            "a truncated report",
            "balanced stdout cut after the predicates, exit 0",
            lambda o: (0, _join(g(o)[:-1]), b"", False, None),
            "rejected",
            2,
        ),
        (
            "truncated-no-outcome-exit1",
            "a truncated report",
            "1/99 stdout cut after the predicates, exit 1",
            lambda o: (1, _join(b(o)[:-1]), b"", False, None),
            "rejected",
            2,
        ),
        (
            "truncated-mid-line",
            "a truncated report",
            "1/99 stdout cut inside the (c) line, exit 1",
            lambda o: (1, bad(o)[: bad(o).index(b"  (c)") + 7], b"", False, None),
            "rejected",
            2,
        ),
        (
            "truncated-empty",
            "a truncated report",
            "no stdout at all, exit 0",
            lambda o: (0, b"", b"", False, None),
            "rejected",
            2,
        ),
        # §12a: crash
        (
            "crash-no-output-exit1",
            "a crash: non-zero exit, a traceback, no verdict",
            "no stdout, a traceback on stderr, exit 1",
            lambda o: (
                1,
                b"",
                b"Traceback (most recent call last):\n  ...\nMemoryError\n",
                False,
                None,
            ),
            "rejected",
            2,
        ),
        (
            "crash-partial-verdict-exit1",
            "a crash: non-zero exit, a traceback, no verdict",
            "VALID and (a) printed, then a traceback, exit 1",
            lambda o: (
                1,
                _join(b(o)[:2]),
                b"Traceback (most recent call last):\n  ...\nKeyError: 'bytes'\n",
                False,
                None,
            ),
            "rejected",
            2,
        ),
        (
            "crash-exit3",
            "a crash: non-zero exit, a traceback, no verdict",
            "no stdout, exit 3",
            lambda o: (3, b"", b"fatal\n", False, None),
            "rejected",
            1,
        ),
        # §12a: exit disagrees with the outcome
        (
            "engaged-exit1",
            "a verdict whose exit disagrees with its outcome",
            "balanced (ENGAGED) stdout, exit 1",
            lambda o: (1, good(o), b"", False, None),
            "rejected",
            2,
        ),
        (
            "not-engaged-exit0",
            "a verdict whose exit disagrees with its outcome",
            "1/99 (NOT ENGAGED) stdout, exit 0",
            lambda o: (0, bad(o), b"", False, None),
            "rejected",
            2,
        ),
        (
            "outcome-contradicts-predicates",
            "a verdict whose exit disagrees with its outcome",
            "1/99 stdout with its outcome line made ENGAGED ((b) fails), exit 0",
            lambda o: (0, _join(b(o)[:-1] + ["ENGAGED"]), b"", False, None),
            "rejected",
            2,
        ),
        # §12a (D094): non-empty stderr
        (
            "stderr-warning",
            "non-empty stderr",
            "balanced stdout, exit 0, a warning on stderr",
            lambda o: (0, good(o), b"warning\n", False, None),
            "rejected",
            2,
        ),
        # the Researcher's controls (6764231210 item 2 / report 8d3d9004 §2)
        (
            "duplicate-line",
            "duplicate output",
            "1/99 stdout with its (b) line repeated, exit 1",
            lambda o: (1, _join(b(o)[:3] + b(o)[2:]), b"", False, None),
            "rejected",
            2,
        ),
        (
            "reordered-lines",
            "reordered output",
            "1/99 stdout with (a) and (b) swapped, exit 1",
            lambda o: (1, _join([b(o)[0], b(o)[2], b(o)[1]] + b(o)[3:]), b"", False, None),
            "rejected",
            2,
        ),
        (
            "reference-line",
            "extra output",
            "balanced stdout with the checker's --reference line appended, exit 0",
            lambda o: (0, _join(g(o) + ["reference (the old arm): no outcome"]), b"", False, None),
            "rejected",
            2,
        ),
        (
            "invalid-listing-exit1",
            "INVALID",
            "the real INVALID listing, exit 1 instead of 2",
            lambda o: (1, o["invalid-no-terminal"][1], b"", False, None),
            "rejected",
            2,
        ),
        (
            "invalid-listing-exit0",
            "INVALID",
            "the real INVALID listing, exit 0",
            lambda o: (0, o["invalid-no-terminal"][1], b"", False, None),
            "rejected",
            2,
        ),
        # the Guardian's eight output/status controls (D094 §3, state/evidence/D094/review_probes.py)
        (
            "D094-truncated",
            "D094 control",
            "balanced stdout without its last line, exit 0",
            lambda o: (0, _join(g(o)[:-1]), b"", False, None),
            "rejected",
            2,
        ),
        (
            "D094-crash",
            "D094 control",
            "no stdout, 'Traceback: fixture' on stderr, exit 1",
            lambda o: (1, b"", b"Traceback: fixture\n", False, None),
            "rejected",
            2,
        ),
        (
            "D094-timeout",
            "D094 control",
            "balanced stdout, exit 0, but the deadline expired",
            lambda o: (0, good(o), b"", True, None),
            "rejected",
            1,
        ),
        (
            "D094-signal",
            "D094 control",
            "balanced stdout, exit -9 (SIGKILL)",
            lambda o: (-9, good(o), b"", False, None),
            "rejected",
            1,
        ),
        (
            "D094-exit-disagrees-engaged",
            "D094 control",
            "balanced stdout, exit 1",
            lambda o: (1, good(o), b"", False, None),
            "rejected",
            2,
        ),
        (
            "D094-exit-disagrees-not-engaged",
            "D094 control",
            "1/99 stdout, exit 0",
            lambda o: (0, bad(o), b"", False, None),
            "rejected",
            2,
        ),
        (
            "D094-stderr-warning",
            "D094 control",
            "balanced stdout, exit 0, 'warning' on stderr",
            lambda o: (0, good(o), b"warning\n", False, None),
            "rejected",
            2,
        ),
        (
            "D094-extra-stdout",
            "D094 control",
            "balanced stdout + 'reference', exit 0",
            lambda o: (0, good(o) + b"reference\n", b"", False, None),
            "rejected",
            2,
        ),
    ]


def dry_run(out, deadline_s=2):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=False)
    _, ident = adj.materialise_checker(out, HERE.parent)
    (out / "checker.json").write_text(json.dumps(ident, indent=2) + "\n")
    if not ident["matches"]:
        raise SystemExit(f"the pinned checker blob is not available or does not match: {ident}")
    rows, outcomes = [], {}
    for name, row, desc, how, want, want_rule in real_cases():
        d = out / "cases" / name
        d.mkdir(parents=True)
        (d / "classes.txt").write_text("T\n")
        rel = Path("cases") / name
        inject, limit = None, adj.DEADLINE_S
        if how[0] == "ledger":
            (d / "records.jsonl").write_text(how[1])
            records = rel / "records.jsonl"
        elif how[0] == "bytes":
            (d / "records.jsonl").write_bytes(how[1])
            records = rel / "records.jsonl"
        else:  # a FIFO no one writes: the checker blocks in open()
            os.mkfifo(d / "records.fifo")
            records = rel / "records.fifo"
            if how[0] == "fifo":
                limit = deadline_s
            else:
                inject = how[1]
        rec = adj.run_step(
            out, str(rel), str(records), str(rel / "classes.txt"), limit, _inject_signal=inject
        )
        if (d / "records.fifo").exists():
            (d / "records.fifo").unlink()
        outcomes[name] = (
            rec["status"]["exit"],
            (d / "checker-stdout.txt").read_bytes(),
            (d / "checker-stderr.txt").read_bytes(),
        )
        rows.append(_row(name, row, desc, "real", want, want_rule, rec))
    for name, row, desc, fn, want, want_rule in simulated_cases():
        code, so, se, timed_out, sig = fn(outcomes)
        raw = dict(
            exit=code,
            stdout=so,
            stderr=se,
            timed_out=timed_out,
            signal=sig if sig else (-code if code < 0 else None),
        )
        decision = adj.adjudicate(code, so, se, timed_out, sig)
        rec = adj.record(f"cases/{name}", raw, decision)
        rec["simulated"] = desc
        adj.archive(out / "cases" / name, raw, rec)
        rows.append(_row(name, row, desc, "simulated", want, want_rule, rec))
    summary = dict(
        schema="team-ports-combined/dry-run/1",
        registration="docs/journal/2026-11-10-team-ports-combined-registration.md §12a",
        checker=ident,
        cases=rows,
        all_match=all(r["match"] for r in rows),
    )
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return summary


def _row(name, row, desc, kind, want, want_rule, rec):
    return dict(
        case=name,
        row=row,
        kind=kind,
        description=desc,
        checker_exit=rec["status"]["exit"],
        timed_out=rec["status"]["timed_out"],
        signal=rec["status"]["signal"],
        checker_outcome=rec["checker_outcome"],
        expected=dict(decision=want, rule=want_rule),
        actual=dict(decision=rec["decision"], rule=rec["rule"]),
        reasons=rec["reasons"],
        zero_work_helper_calls=(rec["reported"] or {}).get("zero_work_helper_calls"),
        match=rec["decision"] == want and rec["rule"] == want_rule,
    )


def table(summary):
    out = [
        "| case | kind | checker exit | checker outcome | expected | actual | match |",
        "|---|---|--:|---|---|---|---|",
    ]
    for r in summary["cases"]:
        status = str(r["checker_exit"])
        if r["timed_out"]:
            status += " (timed out)"
        elif r["signal"]:
            status += f" (signal {r['signal']})"
        out.append(
            f"| `{r['case']}` | {r['kind']} | {status} | {r['checker_outcome'] or '—'} | "
            f"{r['expected']['decision']} (rule {r['expected']['rule']}) | "
            f"{r['actual']['decision']} (rule {r['actual']['rule']}) | {'yes' if r['match'] else '**NO**'} |"
        )
    return "\n".join(out)


if __name__ == "__main__":
    s = dry_run(sys.argv[1])
    print(table(s))
    sys.exit(0 if s["all_match"] else 1)
