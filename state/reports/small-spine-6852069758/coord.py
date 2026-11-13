#!/usr/bin/env python3
"""Deterministic routing and delivery checks for the mibz2 agent team's channel (docs/team/PROTOCOL.md).

The coordinator's messages have one correct place each (PROTOCOL.md §1):

    an assignment for a role   -> the role's inbox (#160, #161, #162, #222), one in flight
    the same, role busy        -> the channel PR #145 as TYPE: QUEUED
    findings, ACKs, decisions  -> the channel PR #145

Subcommands (stdlib only; GitHub reads go through the public REST API, unauthenticated):

    route    --to ROLE --type TYPE         where a message goes now, and why
    verify   --comment-id ID --to ROLE --type TYPE
                                           fetch a posted comment and check it is where
                                           `route` said, with the header it should have
    check    --issue N  (body on stdin)    the tripwire: is this comment misrouted?
    audit    [--since ID]                  every misrouted comment on #145 and the inboxes
    head-sha --pr N --sha SHA [--merged]   exit 0 only if SHA is the PR's exact full head;
                                           with --merged, only if the PR is merged at that head
    warn     --issue N --comment-id ID     the tripwire's warning text (used by the workflow)

Exit status: 0 when the action checked is correct, 1 when it is not, 2 on a usage or
network error. Nothing here posts, edits or deletes anything; the workflow that calls
`warn` raises its one warning itself, as a failing check run on #145.
"""

import argparse
import json
import os
import re
import sys
import urllib.request

REPO = "lmmx/mibz2"
API = f"https://api.github.com/repos/{REPO}"
CHANNEL = 145

# Each role's inbox, offset and cursor (docs/team/SPEC.<ROLE>.md). Comments after the offset are
# the role's queue. Each cursor is state/cursor.json on the role's team-state/<role> branch.
ROLES = {
    "optimizer": {
        "aliases": ("optimizer", "optimiser", "astra"),
        "inbox": 160,
        "offset": 6646251725,
        "cursor": ("team-state/optimizer", "state/cursor.json"),
    },
    "guardian": {
        "aliases": ("guardian",),
        "inbox": 161,
        "offset": 6646249200,
        "cursor": ("team-state/guardian", "state/cursor.json"),
    },
    "researcher": {
        "aliases": ("researcher",),
        "inbox": 162,
        "offset": 6646250444,
        "cursor": ("team-state/researcher", "state/cursor.json"),
    },
    # (the Oracle until 2026-11-06: `oracle` stays an alias, so its earlier messages and
    # any reply from a task still pinned to SPEC.ORACLE.md route and audit as the same role)
    "observer": {
        "aliases": ("observer", "oracle"),
        "inbox": 222,
        "offset": 6683737090,
        # (its state branch keeps the old name, an address like #222: moving it would need a
        # migration manifest that SPEC.md's check fails closed on, for no reader's benefit)
        "cursor": ("team-state/oracle", "state/cursor.json"),
    },
}
INBOXES = {r["inbox"]: name for name, r in ROLES.items()}
# The first inbox comment: before it, assignments on #145 with a TO: line were correct.
CUTOVER = min(r["offset"] for r in ROLES.values())
# Assignments: what wakes a role in its inbox. Everything else is a publication.
ASSIGNMENTS = {"CHECKPOINT", "PROPOSAL", "CORRECTION"}
MARKER = "<!-- routing-tripwire: {id} -->"
# Misroutes already handled, and how: `audit` reports them apart and doesn't fail on them.
RESOLVED = {
    6652503445: "re-dispatched to #160 as 6653416078 (logged on the channel)",
    6653242752: "re-dispatched to #160 as 6653416078 (logged on the channel)",
    6654146350: "a deliberate tripwire test; its first run could not comment on locked #145 (#187)",
    6654371320: "a deliberate tripwire test; its check run did not reopen the finished suite (#188)",
    6654531225: "a deliberate tripwire test; an in-progress check run did not reopen it either",
}


def header(body):
    """The FROM/TO/RE/TYPE/DECISION lines at the top of a message (PROTOCOL.md §2)."""
    h = {}
    for line in (body or "").splitlines()[:8]:
        m = re.match(r"^\s*(FROM|TO|RE|TYPE|DECISION)\s*:\s*(.*?)\s*$", line)
        if m:
            h[m[1]] = m[2]
    return h


def canonical(name):
    """The role a name or alias means (`oracle` → `observer`), or the name unchanged"""
    name = name.strip().lower()
    return next((r for r, v in ROLES.items() if name in v["aliases"]), name)


def roles_named(to):
    """The inbox roles a TO: line names, by their canonical names."""
    names = {t.strip().lower() for t in re.split(r"[,&/]| and ", to or "") if t.strip()}
    return sorted(r for r, v in ROLES.items() if names & set(v["aliases"]))


def misrouted(issue, body, comment_id=None):
    """Why this comment is in the wrong place, or None if it is where it belongs."""
    if MARKER.split("{")[0] in (body or ""):
        return None  # the tripwire's own warning
    h = header(body)
    sender = h.get("FROM", "").strip().lower()
    kind = h.get("TYPE", "").strip().upper()
    to = roles_named(h.get("TO", ""))
    if issue == CHANNEL:
        if comment_id is not None and comment_id < CUTOVER:
            return None
        if sender == "coordinator" and kind in ASSIGNMENTS and to:
            inboxes = ", ".join(f"{r}'s inbox #{ROLES[r]['inbox']}" for r in to)
            return (f"a {kind} from the coordinator to {', '.join(to)} on #{CHANNEL}, which "
                    f"wakes only the coordinator. It belongs in {inboxes}, or here as "
                    f"TYPE: QUEUED while the role is busy.")
        return None
    if issue in INBOXES:
        role = INBOXES[issue]
        if comment_id is not None and comment_id <= ROLES[role]["offset"]:
            return None  # the queue's opening comment, or history before it
        if sender != "coordinator":
            return (f"a message from {sender or 'an unnamed sender'} in {role}'s inbox "
                    f"#{issue}. Only the coordinator writes in an inbox; replies go to "
                    f"#{CHANNEL}.")
        if kind == "QUEUED":
            return f"TYPE: QUEUED in {role}'s inbox #{issue}. Staged work goes on #{CHANNEL}."
        if kind not in ASSIGNMENTS:
            return (f"a {kind or 'untyped'} message in {role}'s inbox #{issue}, which wakes "
                    f"{role} for nothing. An inbox takes only assignments "
                    f"({', '.join(sorted(ASSIGNMENTS))}); publications go on #{CHANNEL}.")
        if role not in to:
            named = ", ".join(to) or h.get("TO", "nobody")
            return (f"a message to {named} in {role}'s inbox #{issue}, where only {role} "
                    f"reads it.")
        if len(to) > 1:
            return (f"a message to {', '.join(to)} in {role}'s inbox #{issue} alone. The "
                    f"other roles read only their own inboxes.")
    return None


def warning(issue, comment_id, reason):
    return (f"{MARKER.format(id=comment_id)}\n**Routing warning:** comment "
            f"[{comment_id}](https://github.com/{REPO}/pull/{issue}#issuecomment-{comment_id}) "
            f"on #{issue} is {reason}\n\nNo action taken: nothing was edited, moved or "
            f"re-posted. The coordinator corrects it.")


# --- GitHub reads -------------------------------------------------------------------------

def get(url):
    req = urllib.request.Request(url, headers={"Accept": "application/vnd.github+json",
                                               "User-Agent": "mibz2-coordination"})
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        req.add_header("Authorization", f"Bearer {token}")
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.load(r)


def comments(issue):
    out, page = [], 1
    while True:
        batch = get(f"{API}/issues/{issue}/comments?per_page=100&page={page}")
        out += batch
        if len(batch) < 100:
            return out
        page += 1


def cursor(role):
    branch, path = ROLES[role]["cursor"]
    import base64
    blob = get(f"{API}/contents/{path}?ref={branch}")
    return json.loads(base64.b64decode(blob["content"]))


def completed(role, cur):
    """The inbox comments the role's cursor marks complete (each role's cursor differs)."""
    if role == "optimizer":
        return {b["comment_id"] for b in cur.get("inbox_briefs", [])
                if b.get("completed") or b.get("status") in ("completed", "finished")}
    if role == "guardian":
        return set(cur.get("consumed_incoming_comment_ids", []))
    if role in ("researcher", "observer"):
        return set(cur.get("processed_comment_ids", []))
    raise ValueError(role)


def state(role, channel=None, inbox=None, cur=None):
    """The role's queue: incomplete inbox items, items awaiting a REVIEW, staged items."""
    r = ROLES[role]
    inbox = comments(r["inbox"]) if inbox is None else inbox
    channel = comments(CHANNEL) if channel is None else channel
    done = completed(role, cursor(role) if cur is None else cur)
    mine = [c for c in inbox if c["id"] > r["offset"] and not is_bot(c)]
    reviews = [c for c in channel if header(c["body"]).get("TYPE", "").upper() == "REVIEW"
               and header(c["body"]).get("FROM", "").lower() in r["aliases"]]
    unreviewed = [c["id"] for c in mine
                  if not any(str(c["id"]) in v["body"] for v in reviews)]
    # A QUEUED item is dispatched only by an inbox message whose RE line cites it
    # ("dispatching QUEUED <id>"). A mention in a body ("next in your queue: …") is not a
    # dispatch: counting it once dropped #271's queued re-review (6731064440) unsent.
    cited = set(re.findall(r"\d{6,}", " ".join(header(c["body"]).get("RE", "") for c in inbox)))
    queued = [c["id"] for c in channel
              if header(c["body"]).get("TYPE", "").upper() == "QUEUED"
              and role in roles_named(header(c["body"]).get("TO", ""))
              and str(c["id"]) not in cited]
    return {"incomplete": [c["id"] for c in mine if c["id"] not in done],
            "unreviewed": unreviewed, "queued": queued}


def is_bot(c):
    u = c.get("user") or {}
    return u.get("type") == "Bot" or u.get("login", "").endswith("[bot]")


# --- Subcommands --------------------------------------------------------------------------

def route(role, kind):
    kind = kind.upper()
    if role not in ROLES or kind not in ASSIGNMENTS:
        return {"issue": CHANNEL, "type": kind, "why": "a publication: the channel PR"}
    s = state(role)
    if s["incomplete"] or s["unreviewed"]:
        return {"issue": CHANNEL, "type": "QUEUED", "busy": s,
                "why": f"{role} is busy: stage it on #{CHANNEL} as TYPE: QUEUED"}
    if s["queued"]:
        return {"issue": ROLES[role]["inbox"], "type": kind, "queued": s["queued"],
                "why": f"{role} is free, but QUEUED {s['queued'][0]} is older: dispatch it "
                       f"first, citing its ID"}
    return {"issue": ROLES[role]["inbox"], "type": kind,
            "why": f"{role} is free: its inbox #{ROLES[role]['inbox']}"}


def resumes(re_line):
    """The inbox item a resume request names (`RE: resume 123`), or None."""
    m = re.match(r"^\s*resume(?:\s+request\s+for)?\s+(\d+)\s*$", re_line or "", re.I)
    return int(m[1]) if m else None


def verify(comment_id, role, kind):
    c = get(f"{API}/issues/comments/{comment_id}")
    issue = int(c["issue_url"].rsplit("/", 1)[1])
    h = header(c["body"])
    problems = []
    reason = misrouted(issue, c["body"], comment_id)
    if reason:
        problems.append(reason)
    if h.get("FROM", "").lower() != "coordinator":
        problems.append(f"FROM is {h.get('FROM')!r}, not coordinator")
    if h.get("TYPE", "").upper() != kind.upper():
        problems.append(f"TYPE is {h.get('TYPE')!r}, not {kind.upper()}")
    if role in ROLES:
        if role not in roles_named(h.get("TO", "")):
            problems.append(f"TO does not name {role}")
        if kind.upper() == "QUEUED" and issue != CHANNEL:
            problems.append(f"QUEUED posted on #{issue}, not the channel PR #{CHANNEL}")
        if kind.upper() in ASSIGNMENTS:
            if issue != ROLES[role]["inbox"]:
                problems.append(f"posted on #{issue}, not {role}'s inbox "
                                f"#{ROLES[role]['inbox']}")
            else:
                s = state(role)
                others = sorted({i for i in s["incomplete"] + s["unreviewed"]
                                 if i != comment_id})
                # A resume request (PROTOCOL.md §2) re-wakes the one item in flight: it
                # names that item and adds no second assignment.
                resumed = resumes(h.get("RE", ""))
                if resumed is not None and others == [resumed]:
                    others = []
                if others:
                    problems.append(f"{role} already had {others} in flight: one at a time")
    return {"comment": comment_id, "issue": issue, "ok": not problems, "problems": problems}


def audit(since):
    found = []
    for issue in (CHANNEL, *INBOXES):
        for c in comments(issue):
            if c["id"] >= since and not is_bot(c):
                reason = misrouted(issue, c["body"], c["id"])
                if reason:
                    found.append({"issue": issue, "comment": c["id"], "at": c["created_at"],
                                  "reason": reason, "resolved": RESOLVED.get(c["id"])})
    return found


def head_sha(pr, sha, merged=False):
    """A merge's expected head must be the PR's exact 40-hex head, never a prefix or guess.

    Before a merge this checks the value; it cannot check that the value reached the merge.
    The merge tool takes it as `expectedHeadSha` and silently drops an unknown name such as
    `sha`, which is how #167, #170 and #184-#189 merged with no head check at all. So the
    check runs again after the merge with `merged=True`: it passes only if the PR is merged
    and its head, which GitHub keeps as the merged head, is still exactly `sha`.
    """
    pull = get(f"{API}/pulls/{pr}")
    head = pull["head"]["sha"]
    ok = bool(re.fullmatch(r"[0-9a-f]{40}", sha)) and sha == head
    out = {"pr": pr, "head": head, "given": sha}
    if merged:
        out["merged"] = bool(pull.get("merged"))
        ok = ok and out["merged"]
    out["ok"] = ok
    return out


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("route"); a.add_argument("--to", required=True); a.add_argument("--type", required=True)
    a = sub.add_parser("verify"); a.add_argument("--comment-id", type=int, required=True)
    a.add_argument("--to", required=True); a.add_argument("--type", required=True)
    a = sub.add_parser("check"); a.add_argument("--issue", type=int, required=True)
    a.add_argument("--comment-id", type=int)
    a = sub.add_parser("warn"); a.add_argument("--issue", type=int, required=True)
    a.add_argument("--comment-id", type=int, required=True)
    a = sub.add_parser("audit"); a.add_argument("--since", type=int, default=CUTOVER)
    a = sub.add_parser("head-sha"); a.add_argument("--pr", type=int, required=True)
    a.add_argument("--sha", required=True)
    a.add_argument("--merged", action="store_true",
                   help="after the merge: the PR must be merged, at exactly this head")
    args = p.parse_args(argv)
    try:
        if args.cmd == "route":
            out = route(canonical(args.to), args.type); ok = True
        elif args.cmd == "verify":
            out = verify(args.comment_id, canonical(args.to), args.type); ok = out["ok"]
        elif args.cmd in ("check", "warn"):
            body = sys.stdin.read()
            reason = misrouted(args.issue, body, args.comment_id)
            if args.cmd == "warn":
                if reason:
                    print(warning(args.issue, args.comment_id, reason))
                return 0
            out = {"misrouted": bool(reason), "reason": reason}; ok = not reason
        elif args.cmd == "audit":
            out = audit(args.since); ok = not [f for f in out if not f["resolved"]]
        else:
            out = head_sha(args.pr, args.sha, args.merged); ok = out["ok"]
    except (OSError, ValueError, KeyError) as e:
        print(json.dumps({"error": str(e)}), file=sys.stderr)
        return 2
    print(json.dumps(out, indent=2))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
