"""The frontier: claims, holds, blog cards, deltas, flags and the surface.

    frontier.py brief [--can shell,rust,...]        what to read, what is held, what you can take
    frontier.py status                              rewrite frontier/STATUS.md from the files alone
    frontier.py check                               validate the frontier's files
    frontier.py claim ID --by AGENT [--hours N]     acquire a hold, atomically, on origin
    frontier.py verify ID --session S               exit 0 iff S holds ID live and no open flag names it
    frontier.py renew ID --session S [--hours N]    extend your hold
    frontier.py release ID --session S              end your hold (after your PR has merged)
    frontier.py holds                               list the holds on origin
    frontier.py mark ID --status open|idea|done|dropped|blocked   edit a claim's state (goes in your PR)

A hold is the branch `hold/<id>` on origin: a chain of commits each carrying `lease.toml`. Taking,
renewing, taking over an expired hold and releasing are all pushes guarded by
`--force-with-lease`, so the remote compares and swaps atomically: of two agents claiming at
once, exactly one succeeds. Nothing deletes a branch (some agents' proxies refuse deletions);
a released hold is a commit saying so. Standard library only (Python 3.11+); nothing here needs
GitHub Actions.
"""

import datetime as dt
import pathlib
import re
import secrets
import subprocess
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPO = ROOT.parent
KINDS = {
    "probe",
    "build",
    "run",
    "review",
    "write",
    "ops",
    "decision",
    "blog",
    "explore",
    "synthesise",
}
ZONES = {"smooth", "striated"}
STATES = {"idea", "open", "review", "done", "dropped", "blocked"}
STAGES = ["idea", "outline", "draft", "review", "revise", "ready", "published"]
REQUIRED = ["id", "title", "kind", "zone", "needs", "priority", "status", "question", "stop_test"]
DELTA_HEAD = ["claim:", "by:", "outcome:", "evidence:"]
FLAG_HEAD = ["flags:", "invariant:", "by:", "status:"]
DEAD_COLS = ["claim", "verdict", "what it showed", "evidence"]
SURFACE_COLS = ["target", "part", "what", "room_1w", "room_8w", "state", "evidence", "claims"]
ME = {"GIT_AUTHOR_NAME": "Louis Maddox", "GIT_AUTHOR_EMAIL": "louismmx@gmail.com"}
ME |= {"GIT_COMMITTER_NAME": ME["GIT_AUTHOR_NAME"], "GIT_COMMITTER_EMAIL": ME["GIT_AUTHOR_EMAIL"]}


def config():
    """frontier/config.toml."""
    with open(ROOT / "config.toml", "rb") as f:
        return tomllib.load(f)


def git(*args, inp=None, env=None):
    """Run git in the repository; return (returncode, stdout)."""
    import os

    r = subprocess.run(
        ["git", "-C", str(REPO), *args],
        input=inp,
        capture_output=True,
        text=True,
        env={**os.environ, **(env or {})},
    )
    return r.returncode, r.stdout.strip() + ("\n" + r.stderr.strip() if r.returncode else "")


def overlaps(a, b):
    """Whether two repository paths contain one another, by path segment."""
    pa, pb = pathlib.PurePosixPath(a.strip("./")), pathlib.PurePosixPath(b.strip("./"))
    return pa == pb or pa in pb.parents or pb in pa.parents


def now():
    """The current UTC time, to the second."""
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0)


def load(dirname):
    """Every TOML item in a frontier directory, with its path."""
    items = []
    for p in sorted((ROOT / dirname).glob("*.toml")):
        with open(p, "rb") as f:
            d = tomllib.load(f)
        d["_path"] = p
        items.append(d)
    return items


def heads(dirname, head):
    """The `key: value` header lines of each Markdown file in a directory, newest name first."""
    out = []
    for p in sorted((ROOT / dirname).glob("*.md"), reverse=True):
        if p.name == "README.md":
            continue
        lines = p.read_text().splitlines()
        h = {}
        for line in lines[: len(head) + 2]:
            if ":" in line:
                k, v = line.split(":", 1)
                h[k.strip()] = v.strip()
        out.append((p, lines, h))
    return out


def holds(fetch=True):
    """{id: (sha, lease)} for every hold on origin; empty if origin can't be reached."""
    if fetch:
        git("fetch", "-q", "--prune", "origin", "+refs/heads/hold/*:refs/remotes/origin/hold/*")
    rc, out = git("for-each-ref", "--format=%(objectname) %(refname)", "refs/remotes/origin/hold")
    found = {}
    for line in out.splitlines() if rc == 0 else []:
        sha, ref = line.split()
        rc2, body = git("show", f"{sha}:lease.toml")
        if rc2 == 0:
            found[ref.rsplit("/", 1)[1]] = (sha, tomllib.loads(body))
    return found


def live(lease):
    """Whether a lease is in force: not released and not expired."""
    exp = dt.datetime.fromisoformat(lease.get("expires", "1970-01-01T00:00:00+00:00"))
    return not lease.get("released") and exp > now()


def write_lease(cid, lease, parent, expected):
    """Commit `lease` onto hold/<cid> (child of `parent`) and push it, guarded by `expected`."""
    body = "".join(
        f"{k} = {str(v).lower() if isinstance(v, bool) else repr(v).replace(chr(39), chr(34))}\n"
        for k, v in lease.items()
    )
    _, blob = git("hash-object", "-w", "--stdin", inp=body)
    _, tree = git("mktree", inp=f"100644 blob {blob}\tlease.toml\n")
    args = ["commit-tree", tree, "-m", f"hold {cid}: {lease['holder']} {lease['session']}"]
    args += ["-p", parent] if parent else []
    _, sha = git(*args, env=ME)
    ref = f"refs/heads/hold/{cid}"
    rc, out = git(
        "push",
        "--porcelain",
        f"--force-with-lease={ref}:{expected or ''}",
        "origin",
        f"{sha}:{ref}",
    )
    return rc == 0, out


def item(cid, ref="origin/master"):
    """A claim or card as the shared state has it (origin/master), or the local file."""
    git("fetch", "-q", "origin", "master")
    for d in ("claims", "blog"):
        rc, body = git("show", f"{ref}:frontier/{d}/{cid}.toml")
        if rc == 0:
            return tomllib.loads(body)
        p = ROOT / d / f"{cid}.toml"
        if p.exists():
            return tomllib.loads(p.read_text())
    sys.exit(f"no claim or card named {cid}")


def open_flags(cid=None):
    """Open flags, optionally only those naming `cid`."""
    return [
        (p, h)
        for p, _, h in heads("flags", FLAG_HEAD)
        if h.get("status") == "open" and (cid is None or cid in h.get("flags", "").split())
    ]


def row(c):
    """One Markdown line for a claim or card."""
    needs = ", ".join(c["needs"]) or "nothing"
    return (
        f"- **{c['id']}** (P{c['priority']}, {c['kind']}, {c['zone']}, needs {needs}): {c['title']}"
    )


def table(rows):
    """A Markdown table from rows of strings, the first row the header."""
    out = ["| " + " | ".join(rows[0]) + " |", "|" + "---|" * len(rows[0])]
    return out + ["| " + " | ".join(r) + " |" for r in rows[1:]]


def render():
    """The whole of STATUS.md, from the files alone (live holds are in `brief` and `holds`)."""
    claims, cards = load("claims"), load("blog")
    every = claims + cards
    _, head = git("rev-parse", "--short", "HEAD")
    out = ["# Frontier status", ""]
    out += [
        f"*Generated by `python3 frontier/tools/frontier.py status` at commit {head}. A local, uncommitted page; live holds aren't here: run `frontier.py holds` or `brief`.*",
        "",
    ]
    louis = [
        c for c in every if c.get("needs_louis") and c.get("status") not in ("done", "dropped")
    ]
    out += ["## Needs Louis", ""] + (
        [f"- **{c['id']}**: {c['needs_louis']}" for c in louis] or ["- nothing"]
    )
    flags = open_flags()
    out += ["", "## Open flags", ""] + (
        [f"- `{p.name}`: {h.get('flags')} ({h.get('invariant')})" for p, h in flags] or ["- none"]
    )
    for state, title in (
        ("review", "Awaiting review or merge"),
        ("open", "Open claims, by priority"),
        ("blocked", "Blocked"),
        ("idea", "Ideas, not yet admitted"),
    ):
        sel = sorted(
            [c for c in claims if c.get("status") == state], key=lambda c: (c["priority"], c["id"])
        )
        out += ["", f"## {title}", ""] + ([row(c) for c in sel] or ["- none"])
    cols = [[c["id"] for c in cards if c.get("stage") == s] for s in STAGES]
    out += ["", "## The blog board", ""]
    out += table(
        [STAGES]
        + [
            [x[i] if i < len(x) else "" for x in cols]
            for i in range(max(map(len, cols), default=0))
        ]
    )
    out += ["", "## Recent deltas", ""]
    out += [
        f"- `{p.name}`: {h.get('claim')}, {h.get('outcome')} ({h.get('by')})"
        for p, _, h in heads("deltas", DELTA_HEAD)[:12]
    ] or ["- none"]
    out += ["", "## Dead ends (`dead-ends.tsv`)", ""]
    out += table([line.split("\t") for line in (ROOT / "dead-ends.tsv").read_text().splitlines()])
    out += ["", "## The surface (`surface.tsv`)", ""]
    out += table([line.split("\t") for line in (ROOT / "surface.tsv").read_text().splitlines()])
    done = [c["id"] for c in every if c.get("status") in ("done", "dropped")]
    return "\n".join(out + ["", f"Closed: {', '.join(done) or 'none'}.", ""])


def check():
    """Validate the frontier's files (five checks); return 1 on any error."""
    errs, warn = [], []
    claims, cards = load("claims"), load("blog")
    # 1. claims and cards: fields and enums
    for c in claims + cards:
        n = c["_path"].name
        errs += [f"{n}: missing {k}" for k in REQUIRED if k not in c]
        errs += [f"{n}: id must equal the file name"] if c.get("id") != c["_path"].stem else []
        for key, allowed in (("kind", KINDS), ("zone", ZONES), ("status", STATES)):
            if c.get(key) not in allowed:
                errs.append(f"{n}: {key} {c.get(key)!r} not in {sorted(allowed)}")
        if c in cards and c.get("stage") not in STAGES:
            errs.append(f"{n}: stage {c.get('stage')!r} not in {STAGES}")
        late = c in cards and STAGES.index(c.get("stage", "idea")) > STAGES.index("review")
        if late and (not c.get("reviewed_by") or c.get("reviewed_by") == c.get("drafted_by")):
            errs.append(
                f"{n}: past review, it needs reviewed_by, by a different agent from drafted_by"
            )
    # 2. deltas and flags: their fixed headers, and deltas at most 40 lines
    for d, head in (("deltas", DELTA_HEAD), ("flags", FLAG_HEAD)):
        for p, lines, _ in heads(d, head):
            if not all(any(line.startswith(h) for line in lines[: len(head) + 2]) for h in head):
                errs.append(f"{d}/{p.name}: the first lines must be {' / '.join(head)}")
            if d == "deltas" and len(lines) > 40:
                errs.append(f"deltas/{p.name}: {len(lines)} lines, over 40")
    # 3. evidence is conserved: no delta changed or removed, no flag removed, against origin/master
    rc, diff = git(
        "diff", "--name-status", "origin/master", "--", "frontier/deltas", "frontier/flags"
    )
    for line in diff.splitlines() if rc == 0 else []:
        st, path = line.split("\t", 1)
        if path.endswith("README.md"):
            continue
        if st.startswith("D") or (st.startswith("M") and "/deltas/" in path):
            errs.append(f"{path}: deltas are append-only and flags are never deleted ({st})")
    # 4. the surface's shape
    rows = [r.split("\t") for r in (ROOT / "surface.tsv").read_text().splitlines()]
    errs += [f"surface.tsv: header must be {SURFACE_COLS}"] if rows[0] != SURFACE_COLS else []
    errs += [
        f"surface.tsv line {i + 1}: {len(r)} columns"
        for i, r in enumerate(rows)
        if len(r) != len(SURFACE_COLS)
    ]
    dead = [r.split("\t") for r in (ROOT / "dead-ends.tsv").read_text().splitlines()]
    errs += [f"dead-ends.tsv: header must be {DEAD_COLS}"] if dead[0] != DEAD_COLS else []
    errs += [
        f"dead-ends.tsv line {i + 1}: {len(r)} columns"
        for i, r in enumerate(dead)
        if len(r) != len(DEAD_COLS)
    ]
    # 5. the admission budget and STATUS.md's freshness (warnings)
    n_open = sum(c.get("status") == "open" for c in claims)
    if n_open > config().get("max_open_claims", 12):
        warn.append(f"{n_open} open claims, over max_open_claims: leave new ones at status idea")
    rc, touched = git("diff", "--name-only", "origin/master")
    hit = (
        [
            t
            for t in touched.splitlines()
            if any(t.startswith(p) for p in config().get("protected", []))
        ]
        if rc == 0
        else []
    )
    if hit:
        warn.append(
            f"protected files changed ({', '.join(hit)}): a different agent reviews and merges; RULES D needs Louis"
        )
    for w in warn:
        print("warning:", w)
    for e in errs:
        print("error:", e)
    print(f"{len(errs)} errors, {len(warn)} warnings")
    return 1 if errs else 0


def brief(can):
    """Print the loop, Needs Louis, flags, live holds, what can be taken, and recent deltas."""
    m = re.search(r"<!-- brief -->(.*?)<!-- /brief -->", (ROOT / "README.md").read_text(), re.S)
    print(m.group(1).strip())
    claims, cards = load("claims"), load("blog")
    held = {k: v for k, (_, v) in holds().items() if live(v)}
    louis = [
        c
        for c in claims + cards
        if c.get("needs_louis") and c.get("status") not in ("done", "dropped")
    ]
    print(
        "\n## Needs Louis\n"
        + ("\n".join(f"- {c['id']}: {c['needs_louis']}" for c in louis) or "- nothing")
    )
    print(
        "\n## Open flags\n"
        + ("\n".join(f"- {p.name}: {h.get('flags')}" for p, h in open_flags()) or "- none")
    )
    print(
        "\n## Held now (live leases on origin)\n"
        + (
            "\n".join(f"- {k}: {v.get('holder')} until {v.get('expires')}" for k, v in held.items())
            or "- nothing"
        )
    )
    review = [c for c in claims + cards if c.get("status") == "review"]
    print(
        "\n## Awaiting review or merge (finish before starting)\n"
        + ("\n".join(f"- {c['id']}: PR #{c.get('pr', '?')}" for c in review) or "- nothing")
    )
    git("fetch", "-q", "origin", "evidence")
    rc, ev = git("log", "-5", "--format=%h %cs %s", "origin/evidence")
    print(
        "\n## Recent workstation evidence (origin/evidence)\n"
        + (ev if rc == 0 and ev else "- unavailable")
    )
    take = [
        c
        for c in claims + cards
        if c.get("status") == "open"
        and c["id"] not in held
        and not c.get("needs_louis")
        and (not can or set(c["needs"]) <= can)
        and not open_flags(c["id"])
    ]
    take.sort(key=lambda c: (c["priority"], c["id"]))
    print("\n## What you can take (highest priority first)")
    for c in take:
        print(row(c) + (f", stage {c['stage']}" if "stage" in c else ""))
    other = [c for c in take if c["kind"] in ("explore", "synthesise")]
    if other:
        print(
            f"\n## Or explore or synthesise (about one session in five): {', '.join(c['id'] for c in other)}"
        )
    n_dead = len((ROOT / "dead-ends.tsv").read_text().splitlines()) - 1
    print(
        f"\n## Dead ends: {n_dead} in frontier/dead-ends.tsv. Read it before proposing a direction."
    )
    print("\n## Recent deltas")
    for p, _, h in heads("deltas", DELTA_HEAD)[:6]:
        print(f"- {p.name}: {h.get('claim')}, {h.get('outcome')} ({h.get('by')})")


def claim(cid, by, hours):
    """Acquire a hold on `cid`, atomically, on origin."""
    c = item(cid)
    if c.get("status") != "open":
        sys.exit(f"{cid} is {c.get('status')}, not open")
    if c.get("needs_louis"):
        sys.exit(f"{cid} needs Louis: {c['needs_louis']}")
    if open_flags(cid):
        sys.exit(f"{cid} has an open flag; resolve it first")
    all_holds = holds()
    if sum(live(v) for _, v in all_holds.values()) >= config().get("max_live_holds", 6):
        sys.exit(
            "the live-hold budget is full (config.toml max_live_holds); take nothing, or help finish something held"
        )
    sha, lease = all_holds.get(cid, (None, None))
    if lease and live(lease):
        sys.exit(f"{cid} is held by {lease['holder']} until {lease['expires']}")
    for k, (_, v) in all_holds.items():
        if k == cid or not live(v):
            continue
        same = sorted(set(c.get("mechanisms", [])) & set(v.get("mechanisms", [])))
        paths = c.get("owned_paths", []) if c.get("zone") == "striated" else []
        clash = [a for a in paths for b in v.get("owned_paths", []) if overlaps(a, b)]
        if same or clash:
            sys.exit(f"{cid} would share {same + clash} with {k}, held by {v.get('holder')}")
    t = now()
    new = {
        "id": cid,
        "holder": by,
        "session": secrets.token_hex(6),
        "claimed_at": t.isoformat(),
        "expires": (t + dt.timedelta(hours=hours)).isoformat(),
        "owned_paths": c.get("owned_paths", []),
        "mechanisms": c.get("mechanisms", []),
        "base": git("rev-parse", "origin/master")[1],
        "released": False,
    }
    ok, out = write_lease(cid, new, sha, sha)
    if not ok:
        sys.exit(f"not acquired (someone else moved hold/{cid} first, or the push failed):\n{out}")
    print(f"acquired {cid} until {new['expires']}; your session is {new['session']}")
    print(f"keep it: `verify`, `renew` and `release` need --session {new['session']}")


def mine(cid, session):
    """(sha, lease) if `session` holds `cid` live; exit otherwise."""
    sha, lease = holds().get(cid, (None, None))
    if not lease or lease.get("session") != session:
        sys.exit(f"{cid} is not held by session {session}")
    if not live(lease):
        sys.exit(
            f"{cid}'s hold by {session} has lapsed: re-claim it before any consequential action"
        )
    return sha, lease


def main(argv):
    """Dispatch a subcommand."""
    cmd = argv[1] if len(argv) > 1 else "brief"
    opt = lambda name, default=None: argv[argv.index(name) + 1] if name in argv else default  # noqa: E731
    hours = float(opt("--hours", "8"))
    if cmd == "brief":
        brief(set(filter(None, (opt("--can") or "").split(","))))
    elif cmd == "status":
        (ROOT / "STATUS.md").write_text(render())
        print("wrote frontier/STATUS.md")
    elif cmd == "check":
        return check()
    elif cmd == "holds":
        for k, (sha, v) in sorted(holds().items()):
            state = "live" if live(v) else ("released" if v.get("released") else "expired")
            print(
                f"{k}: {state}, {v.get('holder')} (session {v.get('session')}) until {v.get('expires')} [{sha[:8]}]"
            )
    elif cmd == "claim":
        claim(argv[2], opt("--by", "unknown"), hours)
    elif cmd == "verify":
        mine(argv[2], opt("--session"))
        if open_flags(argv[2]):
            sys.exit(f"{argv[2]} has an open flag")
        print(f"{argv[2]}: held live by this session; no open flag")
    elif cmd == "renew":
        sha, lease = mine(argv[2], opt("--session"))
        lease["expires"] = (now() + dt.timedelta(hours=hours)).isoformat()
        ok, out = write_lease(argv[2], lease, sha, sha)
        print(f"renewed until {lease['expires']}" if ok else f"not renewed:\n{out}")
    elif cmd == "release":
        sha, lease = holds().get(argv[2], (None, None))
        if not lease or lease.get("session") != opt("--session"):
            sys.exit(f"{argv[2]} is not held by session {opt('--session')}")
        lease["released"] = True
        ok, out = write_lease(argv[2], lease, sha, sha)
        print(f"released {argv[2]}" if ok else f"not released (someone took over):\n{out}")
    elif cmd == "mark":
        p = next(
            (
                ROOT / d / f"{argv[2]}.toml"
                for d in ("claims", "blog")
                if (ROOT / d / f"{argv[2]}.toml").exists()
            ),
            None,
        )
        if not p or opt("--status") not in STATES:
            sys.exit(f"usage: mark ID --status {'|'.join(sorted(STATES))}")
        p.write_text(
            re.sub(r'(?m)^status = ".*"$', f'status = "{opt("--status")}"', p.read_text(), count=1)
        )
        print(f"{argv[2]}: status {opt('--status')} (commit this in your PR)")
    else:
        print(__doc__)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
