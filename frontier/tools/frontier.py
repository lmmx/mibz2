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


def config(ref=None):
    """frontier/config.toml, optionally from a git ref such as origin/master."""
    if ref:
        rc, body = git("show", f"{ref}:frontier/config.toml")
        if rc == 0:
            return tomllib.loads(body)
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
    pa, pb = pathlib.PurePosixPath(a.lstrip("/")), pathlib.PurePosixPath(b.lstrip("/"))
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


def _parse_head(lines, head):
    """Parse `key: value` header lines from a delta or flag Markdown file."""
    h = {}
    for line in lines[: len(head) + 3]:
        if ":" in line and not line.startswith("**"):
            k, v = line.split(":", 1)
            h[k.strip()] = v.strip()
    if "resolved_by" not in h:
        for line in lines:
            if line.startswith("resolved_by:"):
                h["resolved_by"] = line.split(":", 1)[1].strip()
                break
    return h


def heads(dirname, head):
    """The `key: value` header lines of each Markdown file in a directory, newest name first."""
    out = []
    for p in sorted((ROOT / dirname).glob("*.md"), reverse=True):
        if p.name == "README.md":
            continue
        lines = p.read_text().splitlines()
        out.append((p, lines, _parse_head(lines, head)))
    return out


def _fetch_master(strict=False):
    """Fetch origin's master ref; exit on failure when strict."""
    rc, out = git(
        "fetch", "-q", "--no-tags", "origin", "+refs/heads/master:refs/remotes/origin/master"
    )
    if rc != 0 and strict:
        sys.exit(f"cannot fetch origin/master:\n{out}")
    return rc == 0, out


def holds(fetch=True, strict=True):
    """{id: (sha, lease)} for every hold on origin; fail closed on fetch error when strict."""
    if fetch:
        rc, out = git(
            "fetch", "-q", "--prune", "origin", "+refs/heads/hold/*:refs/remotes/origin/hold/*"
        )
        if rc != 0:
            if strict:
                sys.exit(f"cannot fetch holds from origin:\n{out}")
            return {}
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


def conflicts(cid, owned_paths, mechanisms, all_holds):
    """Return (other_id, holder, shared) for any other live hold overlapping paths or mechanisms."""
    mechs = set(mechanisms or [])
    paths = list(owned_paths or [])
    found = []
    for k, (_, v) in all_holds.items():
        if k == cid or not live(v):
            continue
        same = sorted(mechs & set(v.get("mechanisms", [])))
        clash = [a for a in paths for b in v.get("owned_paths", []) if overlaps(a, b)]
        if same or clash:
            found.append((k, v.get("holder"), same + clash))
    return found


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


def item(cid, ref="origin/master", fetch=True, required=True):
    """A claim or card as the shared state has it (origin/master), or the local file."""
    if fetch:
        _fetch_master(strict=False)
    for d in ("claims", "blog"):
        rc, body = git("show", f"{ref}:frontier/{d}/{cid}.toml")
        if rc == 0:
            return tomllib.loads(body)
        p = ROOT / d / f"{cid}.toml"
        if p.exists():
            return tomllib.loads(p.read_text())
    if required:
        sys.exit(f"no claim or card named {cid}")
    return {}


def _flag_matches(h, cid, claim_data=None):
    """Whether flag header `h` applies to `cid` by id or by premise/owned path."""
    targets = h.get("flags", "").split()
    if cid in targets:
        return True
    if not claim_data:
        return False
    paths = list(claim_data.get("premises", []) or []) + list(
        claim_data.get("owned_paths", []) or []
    )
    return any(overlaps(tok, p) for tok in targets for p in paths)


def open_flags(cid=None, *, claim_data=None, fetch=False, strict=False):
    """Open flags from origin/master merged with local flags, optionally filtered to a claim."""
    if fetch:
        _fetch_master(strict=strict)
    merged = {p.name: (p, h) for p, _, h in heads("flags", FLAG_HEAD)}
    rc, listing = git("ls-tree", "-r", "--name-only", "origin/master", "--", "frontier/flags")
    if rc == 0:
        for rel in listing.splitlines():
            name = pathlib.PurePosixPath(rel).name
            if not name.endswith(".md") or name == "README.md":
                continue
            rc2, body = git("show", f"origin/master:{rel}")
            if rc2 == 0:
                merged[name] = (ROOT / "flags" / name, _parse_head(body.splitlines(), FLAG_HEAD))
    elif strict:
        sys.exit(f"cannot read flags from origin/master:\n{listing}")
    found = [
        (p, h) for _, (p, h) in sorted(merged.items(), reverse=True) if h.get("status") == "open"
    ]
    if cid is not None:
        found = [(p, h) for p, h in found if _flag_matches(h, cid, claim_data)]
    return found


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


def _merge_base():
    """Return (base_sha, err) for origin/master...HEAD, fetching origin/master if absent."""
    rc, _ = git("rev-parse", "--verify", "--quiet", "refs/remotes/origin/master")
    if rc != 0:
        ok, out = _fetch_master(strict=False)
        if not ok:
            return (
                None,
                f"origin/master is absent and fetch failed (refusing to skip diff checks): {out}",
            )
    rc, base = git("merge-base", "origin/master", "HEAD")
    if rc != 0 or not base:
        git("fetch", "-q", "--unshallow", "origin")
        rc, base = git("merge-base", "origin/master", "HEAD")
    if rc != 0 or not base:
        return None, "cannot compute merge-base with origin/master (refusing to skip diff checks)"
    return base, None


def check():
    """Validate the frontier's files (five checks); return 1 on any error."""
    errs, warn = [], []
    claims, cards = load("claims"), load("blog")
    # 1. claims and cards: fields, enums, and premises
    for c in claims + cards:
        n = c["_path"].name
        errs += [f"{n}: missing {k}" for k in REQUIRED if k not in c]
        errs += [f"{n}: id must equal the file name"] if c.get("id") != c["_path"].stem else []
        for key, allowed in (("kind", KINDS), ("zone", ZONES), ("status", STATES)):
            if c.get(key) not in allowed:
                errs.append(f"{n}: {key} {c.get(key)!r} not in {sorted(allowed)}")
        for prem in c.get("premises", []):
            if not (REPO / prem).exists():
                errs.append(f"{n}: premise path does not exist: {prem}")
        if c in cards and c.get("stage") not in STAGES:
            errs.append(f"{n}: stage {c.get('stage')!r} not in {STAGES}")
        late = c in cards and STAGES.index(c.get("stage", "idea")) > STAGES.index("review")
        if late and (not c.get("reviewed_by") or c.get("reviewed_by") == c.get("drafted_by")):
            errs.append(
                f"{n}: past review, it needs reviewed_by, by a different agent from drafted_by"
            )
    # 2. deltas and flags: fixed headers, 40-line cap, and independent flag resolution
    for d, head in (("deltas", DELTA_HEAD), ("flags", FLAG_HEAD)):
        for p, lines, h in heads(d, head):
            if not all(any(line.startswith(k) for line in lines[: len(head) + 2]) for k in head):
                errs.append(f"{d}/{p.name}: the first lines must be {' / '.join(head)}")
            if d == "deltas" and len(lines) > 40:
                errs.append(f"deltas/{p.name}: {len(lines)} lines, over 40")
            if d == "flags":
                st = h.get("status")
                if st not in ("open", "resolved"):
                    errs.append(f"flags/{p.name}: status {st!r} must be 'open' or 'resolved'")
                elif st == "resolved":
                    rb = h.get("resolved_by", "")
                    if not rb or rb == h.get("by"):
                        errs.append(
                            f"flags/{p.name}: resolved flag needs resolved_by: by a different agent from by:"
                        )
    # 3. evidence is conserved: append-only deltas/flags/dead-ends and frozen stop_test vs origin/master
    base, base_err = _merge_base()
    if base_err:
        errs.append(base_err)
    else:
        rc, diff = git(
            "diff",
            "--name-status",
            "--no-renames",
            base,
            "--",
            "frontier/deltas",
            "frontier/flags",
            "frontier/claims",
            "frontier/blog",
        )
        if rc != 0:
            errs.append(f"git diff against merge-base {base[:8]} failed: {diff}")
        for line in diff.splitlines() if rc == 0 else []:
            st, path = line.split("\t", 1)
            if path.endswith("README.md"):
                continue
            if "/deltas/" in path or "/flags/" in path:
                if st.startswith("D") or (st.startswith("M") and "/deltas/" in path):
                    errs.append(
                        f"{path}: deltas are append-only and flags are never deleted ({st})"
                    )
                elif "/flags/" in path and st.startswith("M"):
                    rc_old, old_body = git("show", f"{base}:{path}")
                    if rc_old == 0 and (REPO / path).exists():
                        old_h = _parse_head(old_body.splitlines(), FLAG_HEAD)
                        cur_h = _parse_head((REPO / path).read_text().splitlines(), FLAG_HEAD)
                        if old_h.get("by") and cur_h.get("by") != old_h.get("by"):
                            errs.append(f"{path}: flag provenance `by:` cannot be rewritten")
            elif path.endswith(".toml") and st.startswith("M"):
                rc_old, old_body = git("show", f"{base}:{path}")
                if rc_old == 0 and (REPO / path).exists():
                    old_stop = tomllib.loads(old_body).get("stop_test", "").strip()
                    cur_stop = tomllib.loads((REPO / path).read_text()).get("stop_test", "").strip()
                    if old_stop != cur_stop:
                        errs.append(f"{path}: stop_test is frozen once committed (C2)")
        rc_dead, old_dead = git("show", f"{base}:frontier/dead-ends.tsv")
        if rc_dead == 0:
            cur_dead = {
                line
                for line in (ROOT / "dead-ends.tsv").read_text().splitlines()[1:]
                if line.strip()
            }
            for row_line in old_dead.splitlines()[1:]:
                if row_line.strip() and row_line not in cur_dead:
                    errs.append(
                        f"frontier/dead-ends.tsv: existing verdict {row_line.split(chr(9), 1)[0]!r} was removed or rewritten"
                    )
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
    # 5. the admission budget and protected files (warnings, using origin/master config)
    base_cfg = config("origin/master") if base else config()
    local_cfg = config()
    n_open = sum(c.get("status") == "open" for c in claims)
    if n_open > base_cfg.get("max_open_claims", 12):
        warn.append(f"{n_open} open claims, over max_open_claims: leave new ones at status idea")
    if base:
        rc, touched = git("diff", "--name-only", base)
        rc_u, untracked = git("ls-files", "--others", "--exclude-standard")
        changed = (touched.splitlines() if rc == 0 else []) + (
            untracked.splitlines() if rc_u == 0 else []
        )
        protected = list(
            dict.fromkeys(base_cfg.get("protected", []) + local_cfg.get("protected", []))
        )
        hit = [t for t in changed if any(overlaps(t, p) for p in protected)]
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
    held = {k: v for k, (_, v) in holds(strict=False).items() if live(v)}
    flags = open_flags(fetch=True, strict=False)
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
        + ("\n".join(f"- {p.name}: {h.get('flags')}" for p, h in flags) or "- none")
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
        and not any(_flag_matches(h, c["id"], c) for _, h in flags)
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
    _fetch_master(strict=True)
    c = item(cid, fetch=False)
    if c.get("status") != "open":
        sys.exit(f"{cid} is {c.get('status')}, not open")
    if c.get("needs_louis"):
        sys.exit(f"{cid} needs Louis: {c['needs_louis']}")
    if open_flags(cid, claim_data=c, fetch=False, strict=True):
        sys.exit(f"{cid} has an open flag; resolve it first")
    all_holds = holds()
    if sum(live(v) for _, v in all_holds.values()) >= config("origin/master").get(
        "max_live_holds", 6
    ):
        sys.exit(
            "the live-hold budget is full (config.toml max_live_holds); take nothing, or help finish something held"
        )
    sha, lease = all_holds.get(cid, (None, None))
    if lease and live(lease):
        sys.exit(f"{cid} is held by {lease['holder']} until {lease['expires']}")
    paths = c.get("owned_paths", []) if c.get("zone") == "striated" else []
    hit = conflicts(cid, paths, c.get("mechanisms", []), all_holds)
    if hit:
        k, holder, shared = hit[0]
        sys.exit(f"{cid} would share {shared} with {k}, held by {holder}")
    t = now()
    new = {
        "id": cid,
        "holder": by,
        "session": secrets.token_hex(6),
        "claimed_at": t.isoformat(),
        "expires": (t + dt.timedelta(hours=hours)).isoformat(),
        "owned_paths": paths,
        "mechanisms": c.get("mechanisms", []),
        "base": git("rev-parse", "origin/master")[1],
        "released": False,
    }
    ok, out = write_lease(cid, new, sha, sha)
    if not ok:
        sys.exit(f"not acquired (someone else moved hold/{cid} first, or the push failed):\n{out}")
    print(f"acquired {cid} until {new['expires']}; your session is {new['session']}")
    print(f"keep it: `verify`, `renew` and `release` need --session {new['session']}")
    if c.get("premises"):
        print(f"premises to re-read first: {', '.join(c['premises'])}")


def mine(cid, session, all_holds=None):
    """(sha, lease) if `session` holds `cid` live; exit otherwise."""
    src = holds() if all_holds is None else all_holds
    sha, lease = src.get(cid, (None, None))
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
        cid = argv[2]
        all_holds = holds()
        _, lease = mine(cid, opt("--session"), all_holds)
        hit = conflicts(cid, lease.get("owned_paths", []), lease.get("mechanisms", []), all_holds)
        if hit:
            k, holder, shared = hit[0]
            sys.exit(f"{cid} conflicts on {shared} with live hold {k} (held by {holder})")
        _fetch_master(strict=True)
        c = item(cid, fetch=False, required=False)
        claim_data = {
            "premises": c.get("premises", []),
            "owned_paths": list(set(c.get("owned_paths", [])) | set(lease.get("owned_paths", []))),
        }
        if open_flags(cid, claim_data=claim_data, fetch=False, strict=True):
            sys.exit(f"{cid} has an open flag")
        print(f"{cid}: held live by this session; no open flag")
    elif cmd == "renew":
        sha, lease = mine(argv[2], opt("--session"))
        updated = {**lease, "expires": (now() + dt.timedelta(hours=hours)).isoformat()}
        ok, out = write_lease(argv[2], updated, sha, sha)
        if not ok:
            print(f"not renewed:\n{out}", file=sys.stderr)
            return 1
        print(f"renewed until {updated['expires']}")
    elif cmd == "release":
        sha, lease = holds().get(argv[2], (None, None))
        if not lease or lease.get("session") != opt("--session"):
            sys.exit(f"{argv[2]} is not held by session {opt('--session')}")
        updated = {**lease, "released": True}
        ok, out = write_lease(argv[2], updated, sha, sha)
        if not ok:
            print(f"not released (someone took over):\n{out}", file=sys.stderr)
            return 1
        print(f"released {argv[2]}")
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
