"""The service core's starvation diagnostic: what an idle team was waiting on, through its
whole wait (research, untimed). Reads the event logs examples/svc_diag.rs writes (the
`svc-diag` feature's, src/diag.rs) and attributes every interval of every team's wait,
rather than labelling a whole wait by the state it began in, as `TeamIdleStart` does.

Asked for by the Researcher's REVIEW 6736071231 on #145
(state/reports/service-starvation-6735725456/report.md at d6f52bdd). Two mechanisms survive
there, and this tells them apart:

1. **Finish-first starvation:** with P = 1, Q = 2 and J > 0, both service workers choose a
   finish (rule 2 before rule 3), a team takes the last prepared block, and the other team
   waits while both services are inside their non-preemptive finishes.
2. **Window-credit hold:** the scanner admits while issued - taken < W; a ready result
   releases no credit, only the reader's ordered take does. So results ready behind an
   earlier missing block hold credits, the jobs run dry, and the teams wait.

**How a wait is attributed.** The log's events are replayed in their sequence order (each
taken under the lock whose state it reports). Between two consecutive events the state is
constant, so a team's wait is cut at every event into intervals, and each interval is
classified by the state in it, by the first rule that holds:

- `startup`: the team has not yet taken its first block;
- `drain`: the jobs are closed (the scanner has ended) and none is left (J = 0);
- if a prepared block is queued (P > 0): `other:handoff` with a back set free (the team is
  being woken), `other:no_back` without one (back pressure: finishing holds the backs);
- with P = 0:
  - `prep_in_progress`: a preparation is under way (A > 0);
  - `finish_first` (mechanism 1): a job is queued (J > 0) and every service worker is
    finishing (C = K);
  - `other:service_handoff`: a job is queued, no preparation under way, and a service
    worker not finishing (between tasks, or waking);
  - with no job (J = 0): `other:feeder_lag` if a block is issued but not yet in the hub's
    queue; else, the scanner waiting for a window credit, `window_credit` (mechanism 2)
    when a later result is ready while the reader's next block is missing,
    `other:reader_lag` when the reader's next block is itself ready, and
    `other:window_in_flight` when nothing is ready; and else `other:scan_or_input` (the
    scanner reading a block, or waiting for input).

Shares are of the teams' total wait, by the log's timestamps (indicative only: the log is
untimed, its run is not a benchmark, and this machine is not the i9), with the counts of
intervals and of waits beside them. Counts establish occurrence, not recovered wall time.
The deficit states (P = 0 or not, B = 0 or not) through each wait, and the wait's entry label
(`TeamIdleStart`'s), are kept beside the attribution.

    python3 bench/svc_diag.py LOG.tsv [LOG.tsv ...] [--json OUT]
"""

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

HUB = ("j", "p", "q", "a", "c", "u", "free_front", "free_back", "live", "closed")
WIN = ("issued", "taken", "ready", "ready_min", "ready_max", "end", "slots", "cancelled")
INT = ("seq", "ns", "place", "arg", "n") + HUB + WIN

CLASSES = (
    "finish_first",
    "window_credit",
    "prep_in_progress",
    "startup",
    "drain",
    "other",
)
OTHER = (
    "handoff",
    "no_back",
    "service_handoff",
    "feeder_lag",
    "reader_lag",
    "window_in_flight",
    "scan_or_input",
)

# svc_why (src/diag.rs): why a waiting service worker can't prepare
NO_JOB, NO_FRONT, THRESHOLD = 0, 1, 2


def read_tsv(path):
    """A log's events, as dicts, in its rows' order (the sequence's)"""
    rows = Path(path).read_text().splitlines()
    head = rows[0].split("\t")
    events = []
    for line in rows[1:]:
        e = {}
        for k, v in zip(head, line.split("\t"), strict=True):
            if k in INT:
                e[k] = None if v == "-" else int(v)
            else:
                e[k] = v
        e["hub"] = None if e["j"] is None else {k: e[k] for k in HUB}
        e["win"] = None if e["issued"] is None else {k: e[k] for k in WIN}
        events.append(e)
    return events


def role(thread):
    if thread == "mibz2-svc-feed":
        return "feed"
    if thread.startswith("mibz2-svc-"):
        return "svc"
    if thread == "mibz2-scan":
        return "scan"
    if thread.startswith("mibz2-") and thread[6:].isdigit():
        return "team"
    return "reader"


def classify(h, w, K, team_started, transit, scanner):
    """An interval's (class, detail) from the state in it: the hub's snapshot `h`, the
    window's `w`, the service workers K, whether this team has taken a block yet, the
    blocks issued but not yet in the hub's queue, and the scanner's state"""
    if not team_started:
        return "startup", None
    if h["closed"] and h["j"] == 0:
        return "drain", None
    if h["p"] > 0:
        return "other", ("handoff" if h["free_back"] > 0 else "no_back")
    if h["a"] > 0:
        return "prep_in_progress", None
    if h["j"] > 0:
        if h["c"] >= K:
            return "finish_first", None
        return "other", "service_handoff"
    if transit:
        return "other", "feeder_lag"
    if scanner == "slot_wait" and w is not None:
        if w["ready"] > 0 and w["ready_min"] is not None and w["ready_min"] > w["taken"]:
            return "window_credit", None
        if w["ready"] > 0:
            return "other", "reader_lag"
        return "other", "window_in_flight"
    return "other", "scan_or_input"


def no_front_defect(h, lookahead):
    """An eligible preparation blocked for want of a front set: a job, P below the
    threshold, and no free front (never, while the ownership identities hold)"""
    return h["j"] > 0 and h["p"] < lookahead and h["free_front"] == 0


def analyse(events):
    """One log's attribution (see the module's docstring)"""
    cfg = next(e for e in events if e["kind"] == "Config")
    T, K = cfg["place"], cfg["arg"]
    F, W = cfg["hub"]["free_front"], cfg["win"]["slots"]
    L = F - T - K
    h, w = dict(cfg["hub"]), dict(cfg["win"])
    stage = {}  # place -> its stage
    transit = set()  # issued, not yet in the hub's queue
    svc = {}  # service worker -> (state, place)
    svc_busy = defaultdict(int)  # service worker -> ns preparing or finishing
    scanner = "reading"
    scan_ns = Counter()
    started = set()  # teams that have taken a block
    waits = {}  # team -> its open wait
    done = []  # closed waits
    decisions = Counter()
    svc_waits = Counter()
    defects = Counter()
    finish_over_prep = 0
    first_ns, last_ns = events[0]["ns"], events[-1]["ns"]

    for i, e in enumerate(events):
        k, t, place = e["kind"], e["thread"], e["place"]
        r = role(t)
        if e["hub"] is not None:
            h = e["hub"]
            if h["free_front"] + h["a"] + h["p"] + h["u"] != F:
                defects["front_identity"] += 1
            if h["free_back"] + h["u"] + h["q"] + h["c"] != 2 * T:
                defects["back_identity"] += 1
        if e["win"] is not None:
            w = e["win"]
        # blocks' stages
        if k == "Issued":
            stage[place] = "transit"
            transit.add(place)
        elif k == "JobIn":
            stage[place] = "job"
            transit.discard(place)
        elif k == "SvcPrepare":
            stage[place] = "preparing"
            decisions["prepare_empty_p" if e["arg"] == 0 else "prepare_below_threshold"] += 1
        elif k == "Prepared":
            stage[place] = "prepared"
        elif k == "TeamGot":
            stage[place] = "walking"
            started.add(t)
        elif k == "Walked":
            stage[place] = "walked"
        elif k == "SvcFinish":
            stage[place] = "finishing"
            decisions["finish"] += 1
            # a finish chosen while a preparation below the threshold was eligible (rule 2
            # over rule 3): the decision mechanism 1 needs
            if h["j"] > 0 and 0 < h["p"] < L and h["free_front"] > 0:
                finish_over_prep += 1
        elif k == "Publish":
            stage[place] = "ready"
        elif k == "Taken":
            stage[place] = "taken"
        elif k in ("PrepRetire", "WalkRetire") and stage.get(place) not in ("ready", "taken"):
            stage[place] = "dropped"
        elif k == "RetireUnowned":
            defects["retire_unowned"] += 1
        # service workers' tasks
        if r == "svc":
            if k == "SvcPrepare":
                svc[t] = ("preparing", place, e["ns"])
            elif k == "SvcFinish":
                svc[t] = ("finishing", place, e["ns"])
            elif k in ("Prepared", "PrepRetire", "FinishRetire"):
                st = svc.get(t)
                if st and st[0] in ("preparing", "finishing"):
                    svc_busy[t] += e["ns"] - st[2]
                svc[t] = ("between", None, e["ns"])
            elif k == "SvcWait":
                svc[t] = ("idle", e["arg"], e["ns"])
                svc_waits[
                    {NO_JOB: "no_job", NO_FRONT: "no_front", THRESHOLD: "threshold"}[e["arg"]]
                ] += 1
                if e["arg"] == NO_FRONT:
                    defects["svc_wait_no_front"] += 1
            elif k == "SvcExit":
                svc[t] = ("exited", None, e["ns"])
        if e["hub"] is not None and no_front_defect(h, L):
            defects["eligible_prep_no_front"] += 1
        # the scanner
        if r == "scan":
            nxt = {
                "SlotWait": "slot_wait",
                "SlotGot": "reading",
                "InputWait": "input_wait",
                "InputGot": "reading",
                "Issued": "sending",
                "Enqueued": "slot_check",
                "ScanEnd": "ended",
            }.get(k)
            if nxt:
                scanner = nxt
        # teams' waits
        if r == "team":
            if k == "TeamWait":
                waits[t] = {
                    "team": t,
                    "start": e["ns"],
                    "label": "back" if e["arg"] else "prep",
                    "classes": Counter(),
                    "other": Counter(),
                    "deficit": Counter(),
                    "segments": Counter(),
                    "missing_stage": Counter(),
                }
            elif k in ("TeamGot", "TeamExit") and t in waits:
                wt = waits.pop(t)
                wt["end"] = e["ns"]
                done.append(wt)
        # the interval from this event to the next, for each team waiting
        if i + 1 < len(events):
            dt = max(0, events[i + 1]["ns"] - e["ns"])
            scan_ns[scanner] += dt
            for team, wt in waits.items():
                cls, detail = classify(h, w, K, team in started, bool(transit), scanner)
                wt["classes"][cls] += dt
                if dt > 0:
                    wt["segments"][cls] += 1
                if cls == "other":
                    wt["other"][detail] += dt
                d = ("P0" if h["p"] == 0 else "P1") + ("B0" if h["free_back"] == 0 else "B1")
                wt["deficit"][d] += dt
                if cls == "window_credit":
                    wt["missing_stage"][stage.get(w["taken"], "unissued")] += dt
    # a wait still open at the log's end (a cancelled decode) is closed there
    for wt in waits.values():
        wt["end"] = last_ns
        done.append(wt)
    return {
        "plan": {"teams": T, "services": K, "lookahead": L, "window": W, "fronts": F},
        "blocks": sum(1 for e in events if e["kind"] == "Issued"),
        "events": len(events),
        "span_ns": last_ns - first_ns,
        "waits": done,
        "decisions": dict(decisions),
        "finish_over_prep": finish_over_prep,
        "svc_waits": dict(svc_waits),
        "svc_busy_ns": dict(svc_busy),
        "scanner_ns": dict(scan_ns),
        "slot_waits": sum(1 for e in events if e["kind"] == "SlotWait"),
        "defects": dict(defects),
    }


def share(part, whole):
    return part / whole if whole else 0.0


def summarise(results):
    """The attribution's shares and counts over one or more logs (rounds)"""
    total = 0
    cls_ns, cls_waits, cls_segs = Counter(), Counter(), Counter()
    other_ns, deficit_ns, label_ns, missing = Counter(), Counter(), Counter(), Counter()
    excl_ns = Counter()  # the same, startup and drain apart
    decisions, svc_waits, scanner, defects = Counter(), Counter(), Counter(), Counter()
    n_waits, span, busy, blocks, finish_over_prep, slot_waits = 0, 0, [], 0, 0, 0
    team_span = 0
    for r in results:
        span += r["span_ns"]
        team_span += r["plan"]["teams"] * r["span_ns"]
        blocks += r["blocks"]
        finish_over_prep += r["finish_over_prep"]
        slot_waits += r["slot_waits"]
        decisions.update(r["decisions"])
        svc_waits.update(r["svc_waits"])
        scanner.update(r["scanner_ns"])
        defects.update(r["defects"])
        busy.append({t: share(v, r["span_ns"]) for t, v in sorted(r["svc_busy_ns"].items())})
        for wt in r["waits"]:
            n_waits += 1
            length = sum(wt["classes"].values())
            total += length
            label_ns[wt["label"]] += length
            for c, v in wt["classes"].items():
                cls_ns[c] += v
                if v > 0:
                    cls_waits[c] += 1
            cls_segs.update(wt["segments"])
            other_ns.update(wt["other"])
            deficit_ns.update(wt["deficit"])
            missing.update(wt["missing_stage"])
    steady = total - cls_ns["startup"] - cls_ns["drain"]
    for c in CLASSES:
        if c not in ("startup", "drain"):
            excl_ns[c] = cls_ns[c]
    return {
        "logs": len(results),
        "plan": results[0]["plan"] if results else None,
        "blocks": blocks,
        "team_waits": n_waits,
        "team_wait_ns": total,
        # (the teams' waits over the teams' whole time, first event to last: startup and
        # drain in both)
        "team_idle_share": share(total, team_span),
        "classes": {
            c: {
                "share": share(cls_ns[c], total),
                "share_excl_startup_drain": share(excl_ns[c], steady) if c in excl_ns else None,
                "ns": cls_ns[c],
                "intervals": cls_segs[c],
                "waits": cls_waits[c],
            }
            for c in CLASSES
        },
        "other": {d: share(other_ns[d], total) for d in OTHER},
        "deficit": {d: share(deficit_ns[d], total) for d in ("P0B0", "P0B1", "P1B0", "P1B1")},
        "entry_label": {d: share(label_ns[d], total) for d in ("prep", "back")},
        "window_credit_missing_stage": {
            s: share(v, sum(missing.values())) for s, v in missing.most_common()
        },
        "decisions": dict(decisions),
        "finish_over_prep": finish_over_prep,
        "svc_waits": dict(svc_waits),
        "svc_busy": busy,
        "scanner": {s: share(v, span) for s, v in scanner.most_common()},
        "slot_waits": slot_waits,
        "defects": dict(defects),
    }


def report(s):
    out = []
    p = s["plan"]
    out.append(
        f"{s['logs']} log(s), plan {p['teams']} teams + {p['services']} services, L={p['lookahead']}, "
        f"W={p['window']}; {s['blocks']} blocks; {s['team_waits']} team waits"
    )
    out.append("INDICATIVE ONLY: an untimed diagnostic run, not a benchmark, not the i9.")
    out.append(f"teams waiting: {s['team_idle_share']:.3f} of the teams' time, first event to last")
    out.append("")
    out.append(f"{'class':<20}{'share':>8}{'steady':>9}{'intervals':>11}{'waits':>7}")
    for c, v in s["classes"].items():
        st = "" if v["share_excl_startup_drain"] is None else f"{v['share_excl_startup_drain']:.3f}"
        out.append(f"{c:<20}{v['share']:>8.3f}{st:>9}{v['intervals']:>11}{v['waits']:>7}")
    out.append("other, by detail: " + ", ".join(f"{d} {v:.3f}" for d, v in s["other"].items()))
    out.append("deficit states: " + ", ".join(f"{d} {v:.3f}" for d, v in s["deficit"].items()))
    out.append(
        "entry label (TeamIdleStart's): "
        + ", ".join(f"{d} {v:.3f}" for d, v in s["entry_label"].items())
    )
    if s["window_credit_missing_stage"]:
        out.append(
            "window_credit, the reader's next block's stage: "
            + ", ".join(f"{d} {v:.3f}" for d, v in s["window_credit_missing_stage"].items())
        )
    out.append(
        f"decisions: {s['decisions']}; finish over an eligible prep: {s['finish_over_prep']}"
    )
    out.append(f"service waits: {s['svc_waits']}; scanner slot waits: {s['slot_waits']}")
    out.append(
        "service workers preparing or finishing, per log: "
        + "; ".join(", ".join(f"{v:.3f}" for v in b.values()) for b in s["svc_busy"])
    )
    out.append("scanner time: " + ", ".join(f"{d} {v:.3f}" for d, v in s["scanner"].items()))
    out.append(f"defects (all should be 0): {s['defects'] or 'none'}")
    return "\n".join(out)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("logs", nargs="+")
    ap.add_argument("--json")
    a = ap.parse_args(argv)
    s = summarise([analyse(read_tsv(p)) for p in a.logs])
    print(report(s))
    if a.json:
        Path(a.json).write_text(json.dumps(s, indent=2) + "\n")
    return 1 if s["defects"] else 0


if __name__ == "__main__":
    sys.exit(main())
