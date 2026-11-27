"""The frontier: claims, holds, blog cards, deltas, flags and the surface.

    frontier.py brief [--can shell,rust,...]        what to read, what is held, what you can take
    frontier.py status                              rewrite frontier/STATUS.md from files and open PRs
    frontier.py check                               validate the frontier's files
    frontier.py news                                list unposted default/experimental ledger changes
    frontier.py claim ID --by AGENT [--hours N]     acquire a hold, atomically, on origin
    frontier.py verify ID --session S               exit 0 iff S holds ID live and no open flag names it
    frontier.py renew ID --session S [--hours N]    extend your hold
    frontier.py release ID --session S              end your hold (after your PR has merged)
    frontier.py take-review ID --pr N --head SHA --by AGENT   transfer submitted reservation
    frontier.py yield-review ID --session S          park an unfinished integration
    frontier.py finish-review ID --pr N --head SHA   recover/release verified merged reservation
    frontier.py cancel-review ID --pr N --head SHA   release a verified closed-unmerged reservation
    frontier.py holds                               list the holds on origin
    frontier.py mark ID --status open|idea|done|dropped|blocked   edit a claim's state (goes in your PR)
    frontier.py arrive --as FAMILY [--can a,b] [--state S]   opt in to observational presence
    frontier.py touch [--state S] [--claim ID] [--can a,b] [--force]   refresh it (throttled)
    frontier.py depart                              publish a final departed record
    frontier.py population                          observed instances, apart from reservations

A hold is the branch `hold/<id>` on origin: a chain of commits each carrying `lease.toml`. Taking,
renewing, taking over an expired hold and releasing are all pushes guarded by
`--force-with-lease`, so the remote compares and swaps atomically: of two agents claiming at
once, exactly one succeeds. Nothing deletes a branch (some agents' proxies refuse deletions);
a released hold is a commit saying so.

Presence is optional and observational. `arrive` gives this checkout (per Git worktree) a random
opaque instance id for the participant's lifetime, distinct from any lease session, and publishes
one small `presence.toml` on `presence/<family>/<instance>`: self-declared family, capabilities
(for example `workflow-dispatch` where the harness can dispatch workflows), coarse state
(active, idle, integrating, departed), the current claim, and start and last-seen times. Nothing
else: no lease session, hostname, credential or conversation id. Once arrived, `brief`, `claim`,
`renew`, `release` and the review commands refresh it at most every 15 minutes unless something
changed; no background heartbeat. A record seen within 2 hours is "fresh"; older is "stale or
unknown", never dead or available. Arrival and departure prune at most three records departed for
2 hours or unseen for 7 days, each by compare-and-swap on its exact SHA, so a refresh wins a race.
Presence is untrusted: it never grants, reserves, admits or authorises anything, touches neither
hold/* nor the admission epoch, and a failure never fails the command it rides on.
FRONTIER_PRESENCE=off disables the checkpoint refresh. Standard library only (Python 3.11+); nothing here needs
GitHub Actions.
"""

import contextlib
import datetime as dt
import json
import os
import pathlib
import re
import secrets
import shutil
import subprocess
import sys
import tomllib
import urllib.error
import urllib.parse
import urllib.request

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
# The blog's admission test (Louis, 27 November 2026): a post tells the world a theoretical
# discovery or an advance in the software. Process narration and negative results alone go in
# the lab notebook (docs/notebook/). One post at a time between outline and ready.
NEWS_KINDS = {"discovery", "advance"}
IN_PROGRESS = ["outline", "draft", "review", "revise", "ready"]
REQUIRED = ["id", "title", "kind", "zone", "needs", "priority", "status", "question", "stop_test"]
DELTA_HEAD = ["claim:", "by:", "outcome:", "evidence:"]
FLAG_HEAD = ["flags:", "invariant:", "by:", "status:"]
DEAD_COLS = ["claim", "verdict", "what it showed", "evidence"]
SURFACE_COLS = ["target", "part", "what", "room_1w", "room_8w", "state", "evidence", "claims"]


def config(ref=None):
    """Read settings and the reservation ceiling from one immutable snapshot."""
    if ref:
        rc, sha = git("rev-parse", "--verify", f"{ref}^{{commit}}")
        if rc or not re.fullmatch(r"[0-9a-f]{40}", sha):
            raise ValueError(f"cannot resolve authoritative configuration at {ref}")

        def read(name):
            rc, body = git("show", f"{sha}:frontier/{name}")
            if rc:
                raise ValueError(f"missing authoritative frontier/{name}")
            return tomllib.loads(body)
    else:
        def read(name):
            return tomllib.loads((ROOT / name).read_text(encoding="utf-8"))

    cfg = read("config.toml")
    if "limits_file" in cfg:
        if cfg["limits_file"] != "limits.toml" or cfg.get("max_live_holds") != 0:
            raise ValueError("invalid operational-limits migration marker")
        limits = read("limits.toml")
        if set(limits) != {"max_live_holds"}:
            raise ValueError("limits.toml must contain only max_live_holds")
        cfg["max_live_holds"] = limits["max_live_holds"]
    # Legacy config is supported for pre-migration commits and hermetic fixtures.
    cap = cfg.get("max_live_holds")
    if type(cap) is not int or cap < 1:
        raise ValueError("max_live_holds must be a positive integer")
    return cfg


def reservation_limit():
    """Fail closed if authoritative operational settings cannot be read."""
    try:
        return config("origin/master")["max_live_holds"]
    except (OSError, ValueError) as exc:
        sys.exit(f"cannot read reservation limit: {exc}")


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
    return not lease.get("released") and (
        lease.get("phase") in ("integration", "parked") or exp > now()
    )


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


ADMISSION_REF = "refs/heads/frontier-admission"


def admission_snapshot():
    """Read the admission epoch before holds; failure is not absence."""
    rc, out = git("ls-remote", "origin", ADMISSION_REF)
    if rc:
        sys.exit(f"cannot read admission epoch: {out}")
    if not out:
        return ""
    fields = out.split()
    if len(fields) != 2 or fields[1] != ADMISSION_REF:
        sys.exit("invalid remote admission epoch")
    epoch = fields[0]
    if not re.fullmatch(r"[0-9a-f]{40}", epoch):
        sys.exit("invalid remote admission SHA")
    rc, out = git("fetch", "-q", "--no-tags", "--no-write-fetch-head", "origin", epoch)
    if rc:
        sys.exit(f"cannot fetch admission epoch: {out}")
    return epoch


def lease_identity():
    """Use the checkout's explicitly configured Git identity only for commit creation."""
    rc_name, name = git("config", "--local", "--get", "user.name")
    rc_email, email = git("config", "--local", "--get", "user.email")
    if rc_name or rc_email or not name or not email:
        return None
    return {
        "GIT_AUTHOR_NAME": name,
        "GIT_AUTHOR_EMAIL": email,
        "GIT_COMMITTER_NAME": name,
        "GIT_COMMITTER_EMAIL": email,
    }


def write_lease(cid, lease, parent, expected, admission=None):
    """Commit `lease` onto hold/<cid> (child of `parent`) and push it, guarded by `expected`."""
    identity = lease_identity()
    if identity is None:
        return False, "repository-local Git identity is not configured; no lease was written"
    body = "".join(
        f"{k} = {str(v).lower() if isinstance(v, bool) else repr(v).replace(chr(39), chr(34))}\n"
        for k, v in lease.items()
    )
    rc, blob = git("hash-object", "-w", "--stdin", inp=body)
    if rc:
        return False, blob
    rc, tree = git("mktree", inp=f"100644 blob {blob}\tlease.toml\n")
    if rc:
        return False, tree
    args = ["commit-tree", tree, "-m", f"hold {cid}: {lease['holder']} {lease['session']}"]
    args += ["-p", parent] if parent else []
    rc, sha = git(*args, env=identity)
    if rc:
        return False, sha
    ref = f"refs/heads/hold/{cid}"
    args = ["push", "--porcelain", f"--force-with-lease={ref}:{expected or ''}"]
    updates = [f"{sha}:{ref}"]
    if admission is not None:
        # Both refs advance or neither does. Distinct IDs share this epoch.
        epoch_args = ["commit-tree", tree, "-m", f"admission {secrets.token_hex(16)}"]
        if admission:
            epoch_args += ["-p", admission]
        rc, epoch = git(*epoch_args, env=identity)
        if rc:
            return False, epoch
        args += ["--atomic", f"--force-with-lease={ADMISSION_REF}:{admission}"]
        updates.append(f"{epoch}:{ADMISSION_REF}")
    rc, out = git(*args, "origin", *updates)
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


def claim_header(body):
    """Read the existing Claim header, not instructions elsewhere in a PR body."""
    lines = (body or "").splitlines()
    m = re.fullmatch(r"Claim: ([a-z0-9][a-z0-9_-]*)", lines[0].strip()) if lines else None
    return m.group(1) if m else None


def github_repository():
    """Identify a GitHub origin, including local GitHub git proxies; never query a test remote."""
    rc, remote = git("remote", "get-url", "origin")
    if rc:
        return None
    if remote.startswith("git@github.com:"):
        name = remote.split(":", 1)[1]
    else:
        url = urllib.parse.urlsplit(remote)
        if url.hostname == "github.com":
            name = url.path.lstrip("/")
        elif url.hostname in ("localhost", "127.0.0.1", "::1", "local_proxy"):
            if not url.path.startswith("/git/"):
                return None
            name = url.path.removeprefix("/git/")
        else:
            return None
    name = name.rstrip("/").removesuffix(".git")
    return name if re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", name) else None


def read_open_prs():
    """Return (open PRs to master, error), using live GitHub data rather than claim status."""
    repo = github_repository()
    if not repo:
        return None, "origin is not a recognised GitHub remote"
    endpoint = f"repos/{repo}/pulls?state=open&base=master&per_page=100"
    # gh can use the session's existing authentication; the standard-library fallback
    # also works for public repositories without gh. Never print credentials or response bodies.
    if shutil.which("gh"):
        try:
            proc = subprocess.run(
                ["gh", "api", "--hostname", "github.com", "--paginate", "--slurp", endpoint],
                cwd=REPO,
                capture_output=True,
                text=True,
                timeout=12,
            )
            if proc.returncode == 0:
                pages = json.loads(proc.stdout)
                if isinstance(pages, list) and all(isinstance(p, list) for p in pages):
                    return [pr for page in pages for pr in page], None
        except (OSError, subprocess.TimeoutExpired, ValueError):
            pass
    headers = {"Accept": "application/vnd.github+json", "User-Agent": "frontier-review"}
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    found = []
    try:
        for page in range(1, 1001):
            req = urllib.request.Request(
                f"https://api.github.com/{endpoint}&page={page}", headers=headers
            )
            with urllib.request.urlopen(req, timeout=8) as response:
                rows = json.load(response)
            if not isinstance(rows, list):
                break
            found.extend(rows)
            if len(rows) < 100:
                return found, None
    except (OSError, ValueError, urllib.error.URLError):
        pass
    return None, "GitHub open-PR query failed or was incomplete"


def read_pr(number):
    """Read a single PR, including merge state; never trust stale local review metadata."""
    repo = github_repository()
    if not repo or not isinstance(number, int) or number < 1:
        sys.exit("cannot identify pull request")
    endpoint = f"repos/{repo}/pulls/{number}"
    if shutil.which("gh"):
        try:
            proc = subprocess.run(
                ["gh", "api", "--hostname", "github.com", endpoint],
                cwd=REPO, capture_output=True, text=True, timeout=12,
            )
            if proc.returncode == 0:
                pr = json.loads(proc.stdout)
                if isinstance(pr, dict):
                    return pr
        except (OSError, subprocess.TimeoutExpired, ValueError):
            pass
    headers = {"Accept": "application/vnd.github+json", "User-Agent": "frontier-review"}
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    try:
        req = urllib.request.Request(f"https://api.github.com/{endpoint}", headers=headers)
        with urllib.request.urlopen(req, timeout=8) as response:
            pr = json.load(response)
        if isinstance(pr, dict):
            return pr
    except (OSError, ValueError):
        pass
    sys.exit(f"cannot read current PR #{number}; refusing integration transition")


def check_review_pr(cid, number, head, *, merged=False):
    """Validate exact PR lineage and state before an integration transition."""
    if not re.fullmatch(r"[0-9a-f]{40}", head or ""):
        sys.exit("integration requires the exact 40-character PR head")
    pr = read_pr(number)
    try:
        valid = (
            pr["number"] == number
            and pr["head"]["sha"] == head
            and pr["base"]["ref"] == "master"
            and claim_header(pr["body"]) == cid
            and (
                (pr["state"] == "closed" and pr["merged"] is True)
                if merged else (pr["state"] == "open" and pr["draft"] is False)
            )
        )
    except (KeyError, TypeError, ValueError):
        valid = False
    if not valid:
        sys.exit(f"PR #{number} is not the expected {'merged' if merged else 'open'} claim/head")
    return pr


def review_card(cid, number, head):
    """Fetch the immutable PR head and require the producer's existing review declaration."""
    rc, out = git("fetch", "-q", "--no-tags", "origin", f"refs/pull/{number}/head")
    if rc or git("rev-parse", "FETCH_HEAD")[1] != head:
        sys.exit(f"cannot fetch exact PR #{number} head: {out}")
    for dirname in ("claims", "blog"):
        rc, body = git("show", f"{head}:frontier/{dirname}/{cid}.toml")
        if rc == 0:
            c = tomllib.loads(body)
            if c.get("id") == cid and c.get("status") == "review" and c.get("pr") == number:
                return c
    sys.exit(f"PR #{number} has not declared this claim ready for handoff")


def _trunk_item(cid):
    """Read a claim or blog card strictly from origin/master (never local files)."""
    if not re.fullmatch(r"[A-Za-z0-9_-]+", cid):
        return {}
    for dirname in ("claims", "blog"):
        rc, body = git("show", f"origin/master:frontier/{dirname}/{cid}.toml")
        if rc == 0:
            try:
                return tomllib.loads(body)
            except tomllib.TOMLDecodeError:
                return {}
    return {}


def take_review(cid, number, head, by, hours):
    """Transfer a submitted claim's existing reservation, without acquiring another work slot."""
    if not re.fullmatch(r"[A-Za-z0-9_-]+", cid):
        sys.exit("invalid claim id")
    if not by or by == "unknown" or hours <= 0:
        sys.exit("integration requires a named participant and positive lease duration")
    check_review_pr(cid, number, head)
    c = review_card(cid, number, head)
    _fetch_master(strict=True)
    trunk = _trunk_item(cid)
    if not trunk or trunk.get("status") not in ("open", "review") or trunk.get("needs_louis"):
        sys.exit(f"{cid} is not an admitted, unreserved integration candidate")
    if c.get("needs_louis"):
        sys.exit(f"{cid} requires an owner decision")
    admission = admission_snapshot()
    all_holds = holds()
    sha, lease = all_holds.get(cid, (None, None))
    if not lease or lease.get("released"):
        sys.exit(f"{cid} has no existing reservation to transfer")
    if not live(lease) and sum(live(v) for _, v in all_holds.values()) >= reservation_limit():
        sys.exit("expired reservation cannot be revived at capacity")
    if lease.get("phase") == "integration" and dt.datetime.fromisoformat(lease["expires"]) > now():
        sys.exit(f"{cid} already has an integration holder; wait for yield or expiry")
    if lease.get("phase") in ("integration", "parked") and lease.get("pr") != number:
        sys.exit(f"{cid} is reserved for a different PR")
    producer = lease.get("producer", lease.get("holder"))
    if not producer or by == producer:
        sys.exit("integration handoff requires a participant different from the producer")
    if not lease.get("base") or git("merge-base", "--is-ancestor", lease["base"], head)[0]:
        sys.exit("PR head does not descend from the reserved work base")
    hit = conflicts(cid, lease.get("owned_paths", []), lease.get("mechanisms", []), all_holds)
    if hit:
        sys.exit(f"{cid} overlaps another live reservation: {hit[0]}")
    check_review_pr(cid, number, head)
    updated = {
        **lease, "holder": by, "producer": producer, "session": secrets.token_hex(6),
        "phase": "integration", "pr": number, "pr_head": head,
        "expires": (now() + dt.timedelta(hours=hours)).isoformat(), "released": False,
    }
    ok, out = write_lease(cid, updated, sha, sha, admission)
    if not ok:
        sys.exit(f"integration not acquired: {out}")
    print(f"acquired integration {cid} for PR #{number}; your session is {updated['session']}")
    print("Existing reservation transferred; verify exact head and all normal merge gates.")


def yield_review(cid, session):
    """Return an integration reservation to the environment without releasing its paths."""
    sha, lease = mine(cid, session)
    if lease.get("phase") != "integration":
        sys.exit("only an integration holder may yield its review")
    updated = {**lease, "phase": "parked", "session": secrets.token_hex(6)}
    ok, out = write_lease(cid, updated, sha, sha)
    if not ok:
        sys.exit(f"integration not yielded: {out}")
    print(f"{cid}: integration yielded; any independent participant can take it")


def finish_review(cid, number, head, *, cancelled=False):
    """Release exactly a verified terminal PR reservation, even after session loss."""
    sha, lease = holds().get(cid, (None, None))
    if (
        not lease or lease.get("phase") not in ("integration", "parked")
        or lease.get("pr") != number or lease.get("pr_head") != head
    ):
        sys.exit("no matching integration reservation")
    if cancelled:
        pr = read_pr(number)
        try:
            valid = (
                pr["number"] == number and pr["head"]["sha"] == head
                and pr["base"]["ref"] == "master" and claim_header(pr["body"]) == cid
                and pr["state"] == "closed" and pr["merged"] is False
            )
        except (KeyError, TypeError):
            valid = False
        if not valid:
            sys.exit("PR is not the exact closed-unmerged reservation")
        terminal = "closed-unmerged"
    else:
        pr = check_review_pr(cid, number, head, merged=True)
        merge = pr.get("merge_commit_sha")
        if not re.fullmatch(r"[0-9a-f]{40}", merge or ""):
            sys.exit("missing exact merge commit")
        _fetch_master(strict=True)
        if git("merge-base", "--is-ancestor", merge, "origin/master")[0]:
            sys.exit("merge commit is not on origin/master")
        terminal = merge
    if lease.get("released"):
        if lease.get("terminal") != terminal:
            sys.exit("release record disagrees with terminal PR")
        print(f"{cid}: already released for PR #{number}")
        return
    updated = {**lease, "released": True, "terminal": terminal}
    ok, out = write_lease(cid, updated, sha, sha)
    if not ok:
        sys.exit(f"integration not released: {out}")
    print(f"{cid}: terminal PR #{number}; reservation released")


def _release_terminal_ordinary(cid, sha, lease, trunk):
    """Release an ordinary live lease only on verified merged PR state on origin/master."""
    number = trunk.get("pr")
    base = lease.get("base")
    if (
        trunk.get("id") != cid
        or trunk.get("status") not in ("done", "dropped")
        or type(number) is not int
        or number < 1
        or not isinstance(base, str)
        or not re.fullmatch(r"[0-9a-f]{40}", base)
    ):
        return False
    pr = read_pr(number)
    try:
        merge = pr.get("merge_commit_sha")
        body = pr.get("body")
        valid = (
            type(pr["number"]) is int
            and pr["number"] == number
            and pr["base"]["ref"] == "master"
            and isinstance(body, str)
            and claim_header(body) == cid
            and pr["state"] == "closed"
            and pr["merged"] is True
            and isinstance(merge, str)
            and bool(re.fullmatch(r"[0-9a-f]{40}", merge))
        )
    except (AttributeError, KeyError, TypeError, ValueError):
        valid = False
    if not valid:
        sys.exit(f"PR #{number} is not a verified merged PR for {cid}")
    _fetch_master(strict=True)
    if git("merge-base", "--is-ancestor", merge, "origin/master")[0]:
        sys.exit("merge commit is not on origin/master")
    if git("merge-base", "--is-ancestor", base, merge)[0]:
        sys.exit("merge commit does not descend from the reserved work base")
    updated = {**lease, "released": True, "terminal": merge}
    ok, out = write_lease(cid, updated, sha, sha)
    if not ok:
        sys.exit(f"terminal hold not released: {out}")
    print(f"{cid}: terminal PR #{number}; reservation released")
    return True


def reconcile_integrations(all_holds, *, fetch_master=True):
    """Recover terminal reservations before admitting new work; never release on uncertainty."""
    refresh = False
    master_ok = True
    has_ordinary = any(live(v) and v.get("phase") is None for _, v in all_holds.values())
    if has_ordinary and fetch_master:
        master_ok, _ = _fetch_master(strict=False)
    for cid, (sha, lease) in sorted(all_holds.items()):
        if not live(lease):
            continue
        phase = lease.get("phase")
        if phase in ("integration", "parked"):
            refresh = True
            try:
                pr = read_pr(lease["pr"])
                if pr.get("state") == "closed" and isinstance(pr.get("merged"), bool):
                    finish_review(cid, lease["pr"], lease["pr_head"], cancelled=not pr["merged"])
            except (SystemExit, KeyError, TypeError, ValueError):
                print(
                    f"{cid}: integration reconciliation unavailable; reservation retained",
                    file=sys.stderr,
                )
        elif phase is None and master_ok:
            trunk = _trunk_item(cid)
            if trunk.get("status") not in ("done", "dropped"):
                continue
            try:
                if _release_terminal_ordinary(cid, sha, lease, trunk):
                    refresh = True
            except (SystemExit, KeyError, TypeError, ValueError):
                refresh = True
                print(
                    f"{cid}: terminal hold reconciliation unavailable; reservation retained",
                    file=sys.stderr,
                )
    return holds() if refresh else all_holds


def review_projection(items, prs, error):
    """Project review candidates from open PRs; this is visibility, never merge authority."""
    by_id = {c["id"]: c for c in items}
    linked, unclaimed, unlinked = {}, [], []
    # A partial or malformed response must not be reported as an empty queue.
    if prs is not None:
        try:
            for pr in prs:
                if (
                    not isinstance(pr["number"], int)
                    or pr["number"] < 1
                    or not re.fullmatch(r"[0-9a-f]{40}", pr["head"]["sha"])
                    or pr["state"] != "open"
                    or pr["base"]["ref"] != "master"
                    or not isinstance(pr["draft"], bool)
                    or (pr["body"] is not None and not isinstance(pr["body"], str))
                ):
                    raise ValueError("malformed PR metadata")
                cid = claim_header(pr["body"])
                if cid == "none":
                    unclaimed.append(pr["number"])
                elif cid is None:
                    unlinked.append(pr["number"])
                else:
                    linked.setdefault(cid, {})[pr["number"]] = pr
        except (KeyError, TypeError, ValueError):
            prs, error = None, "GitHub returned incomplete or malformed PR metadata"
            linked, unclaimed, unlinked = {}, [], []
    lines = []
    if prs is None:
        lines.append(f"- UNAVAILABLE: {error}; stored review pointers below are unverified.")
    else:
        for cid, group in sorted(
            linked.items(), key=lambda x: (by_id.get(x[0], {}).get("priority", 999), x[0])
        ):
            for number, pr in sorted(group.items()):
                state = "draft, not ready" if pr["draft"] else "open; integration gates apply"
                notes = []
                if cid not in by_id:
                    notes.append("claim not in this checkout")
                if len(group) > 1:
                    notes.append("multiple open PRs for claim")
                suffix = "; " + "; ".join(notes) if notes else ""
                lines.append(f"- {cid}: PR #{number} at {pr['head']['sha'][:12]} ({state}{suffix})")
        if unclaimed:
            refs = ", ".join(f"#{n}" for n in sorted(set(unclaimed)))
            lines.append(f"- Unclaimed contributions (Claim: none): {refs}")
        if unlinked:
            refs = ", ".join(f"#{n}" for n in sorted(set(unlinked)))
            lines.append(f"- Unclassified open PRs (no valid Claim header; may be legacy): {refs}")
    for cid, c in sorted(by_id.items()):
        if c.get("status") != "review":
            continue
        number = c.get("pr", "?")
        if prs is None:
            lines.append(f"- {cid}: stored PR #{number} (unverified)")
        elif number not in linked.get(cid, {}):
            lines.append(f"- {cid}: stored PR #{number} is not a linked open PR (stale pointer)")
    return lines or ["- none"], set(linked)


def check_pr_lineage(event_path, items):
    """Check the existing Claim header convention for PRs that change frontier data."""
    try:
        pr = json.loads(pathlib.Path(event_path).read_text())["pull_request"]
        base = pr["base"]["sha"]
        if not re.fullmatch(r"[0-9a-f]{40}", base):
            raise ValueError("invalid base SHA")
        rc, changed = git("diff", "--name-only", base, "HEAD", "--", "frontier/")
        if rc:
            return "cannot inspect PR's frontier diff for claim lineage"
        if not changed:
            return None
        cid = claim_header(pr.get("body"))
        if cid is None:
            return "frontier PR needs its existing Claim: <id> or Claim: none first line"
        if cid != "none" and cid not in {c["id"] for c in items}:
            return f"PR Claim: {cid} has no claim or blog card in the checked tree"
        for c in items:
            if (
                c.get("status") == "review"
                and c.get("pr") == pr["number"]
                and c["id"] != cid
            ):
                return f"PR Claim: {cid} does not match its review pointer for {c['id']}"
    except (OSError, KeyError, TypeError, ValueError):
        return "cannot read PR event for claim lineage"
    return None


def render():
    """STATUS.md from files and live PRs (live holds are in `brief` and `holds`)."""
    claims, cards = load("claims"), load("blog")
    every = claims + cards
    prs, pr_error = read_open_prs()
    review_lines, pending = review_projection(every, prs, pr_error)
    _, head = git("rev-parse", "--short", "HEAD")
    out = ["# Frontier status", ""]
    out += [
        f"*Generated by `python3 frontier/tools/frontier.py status` at commit {head}. A local, uncommitted page; the review queue uses live PRs. For live holds run `frontier.py holds` or `brief`.*",
        "",
    ]
    louis = [
        c for c in every if c.get("needs_louis") and c.get("status") not in ("done", "dropped")
    ]
    out += ["## Needs Louis", ""] + (
        [f"- **{c['id']}**: {c['needs_louis']}" for c in louis] or ["- nothing"]
    )
    due = release_due(every)
    out += ["", "## Release due (P1)", ""] + (
        [f"- {len(due[1])} software commit(s) on master since {due[0]}; no release claim yet"]
        if due
        else ["- nothing"]
    )
    flags = open_flags()
    out += ["", "## Open flags", ""] + (
        [f"- `{p.name}`: {h.get('flags')} ({h.get('invariant')})" for p, h in flags] or ["- none"]
    )
    out += ["", "## Population (observed presence, not reservations)", ""]
    out += presence_population(holds(strict=False))
    out += ["", "## Awaiting review or merge", ""] + review_lines
    for state, title in (
        ("open", "Open claims, by priority"),
        ("blocked", "Blocked"),
        ("idea", "Ideas, not yet admitted"),
    ):
        sel = sorted(
            [
                c
                for c in claims
                if c.get("status") == state and (state != "open" or c["id"] not in pending)
            ],
            key=lambda c: (c["priority"], c["id"]),
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


RELEASE_PATHS = ["src", "python", "Cargo.toml", "Cargo.lock", "pyproject.toml", "build.rs"]


def release_due(claims):
    """Software merged to master since the last release tag, and no live release claim.

    Returns (tag, commits) when a release is due, else None. Releases are P1 once recognised
    (Louis, 27 November 2026; docs/RELEASING.md section 1): this is how they are recognised.
    """
    rc, tag = git("describe", "--tags", "--abbrev=0", "--match", "v[0-9]*", "origin/master")
    if rc != 0 or not tag:
        return None
    rc, log = git("log", "--first-parent", "--format=%h %s", f"{tag}..origin/master", "--", *RELEASE_PATHS)
    commits = [line for line in log.splitlines() if line.strip()] if rc == 0 else []
    live_release = any(
        c["id"].startswith("release-") and c.get("status") not in ("done", "dropped") for c in claims
    )
    return (tag, commits) if commits and not live_release else None


LEDGER_STATES = {"default", "experimental", "awaiting-review", "prototype", "target"}
POST_DATE_RE = re.compile(r"^(\d{4}-\d{2}-\d{2})-")
PR_SUBJECT_RE = re.compile(r"\(#(\d+)\)")


def _ledger_rows():
    """Return parsed rows of frontier/ledger.tsv, or None if the file is absent."""
    p = ROOT / "ledger.tsv"
    if not p.is_file():
        return None
    lines = p.read_text(encoding="utf-8").splitlines()
    if not lines:
        return []
    cols = lines[0].split("\t")
    rows = []
    for raw in lines[1:]:
        if not raw.strip():
            continue
        parts = raw.split("\t")
        if len(parts) != len(cols):
            continue
        rec = dict(zip(cols, (x.strip() for x in parts)))
        pr_str = rec.get("pr", "")
        if not re.fullmatch(r"\d+", pr_str):
            continue
        rec["pr"] = int(pr_str)
        rows.append(rec)
    return rows


def _merged_prs_on_master():
    """Return (pr_set, err) for PR numbers merged on origin/master (first-parent)."""
    rc, _ = git("rev-parse", "--verify", "--quiet", "refs/remotes/origin/master")
    if rc != 0:
        ok, out = _fetch_master(strict=False)
        if not ok:
            return None, f"cannot verify merged PRs without origin/master: {out}"
    rc, log = git("log", "--first-parent", "--format=%s", "origin/master")
    if rc != 0:
        return None, f"git log origin/master failed: {log}"
    prs = set()
    for subject in log.splitlines():
        for m in PR_SUBJECT_RE.findall(subject):
            prs.add(int(m))
    return prs, None


def _newest_published_post_date(cards):
    """Return the newest YYYY-MM-DD date among published blog posts, or None."""
    dates = []
    blog_dir = REPO / "docs" / "site" / "blog"
    if blog_dir.is_dir():
        for entry in blog_dir.iterdir():
            if entry.is_file() and entry.suffix == ".md":
                m = POST_DATE_RE.match(entry.name)
                if m:
                    dates.append(m.group(1))
    for c in cards:
        if c.get("stage") == "published" and c.get("post"):
            m = POST_DATE_RE.match(pathlib.Path(c["post"]).name)
            if m:
                dates.append(m.group(1))
    return max(dates) if dates else None


def news():
    """List default/experimental ledger changes merged since the newest published post."""
    rows = _ledger_rows()
    if rows is None:
        print("error: frontier/ledger.tsv not found", file=sys.stderr)
        return 1
    cards = load("blog")
    since = _newest_published_post_date(cards)
    named = {
        pr
        for c in cards
        if c.get("status") != "dropped"
        for pr in c.get("changes", [])
        if type(pr) is int
    }
    fresh = [
        r
        for r in rows
        if r.get("state") in ("default", "experimental")
        and (since is None or r.get("merged", "") >= since)
        and r["pr"] not in named
    ]
    header = (
        f"Unposted ledger changes (default/experimental, merged >= {since}):"
        if since
        else "Unposted ledger changes (default/experimental, no published posts yet):"
    )
    print(header)
    if not fresh:
        print("- none")
        return 0
    for r in fresh:
        print(
            f"- #{r['pr']} ({r.get('merged', '?')}, {r.get('state', '?')}): "
            f"{r.get('effect', '')} [measured: {r.get('measured', 'not measured')}]"
        )
    return 0


def check(pr_event=None):
    """Validate the frontier's files (five checks); return 1 on any error."""
    errs, warn = [], []
    claims, cards = load("claims"), load("blog")
    ledger_rows = None
    merged_prs = None
    merged_err = None
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
        if c in cards and not c.get("legacy") and c.get("stage", "idea") != "idea":
            if not str(c.get("news", "")).strip():
                errs.append(
                    f"{n}: past idea, a blog card needs news: one plain sentence saying what a "
                    "reader outside the project learns (process and negative results go in docs/notebook/)"
                )
            kind = c.get("news_kind")
            if kind not in NEWS_KINDS:
                errs.append(
                    f"{n}: news_kind {kind!r} not in {sorted(NEWS_KINDS)}; "
                    "anything else is a lab-notebook entry, not a post"
                )
            elif kind == "advance":
                changes = c.get("changes")
                if (
                    not isinstance(changes, list)
                    or not changes
                    or not all(type(x) is int and x > 0 for x in changes)
                ):
                    errs.append(
                        f"{n}: news_kind 'advance' past idea requires non-empty changes = [<PR numbers>]"
                    )
                else:
                    if ledger_rows is None:
                        ledger_rows = _ledger_rows()
                    if ledger_rows is None:
                        errs.append(
                            f"{n}: cannot verify advance changes without frontier/ledger.tsv"
                        )
                    else:
                        ledger_prs = {r["pr"] for r in ledger_rows}
                        if merged_prs is None and merged_err is None:
                            merged_prs, merged_err = _merged_prs_on_master()
                        if merged_err is not None:
                            errs.append(f"{n}: {merged_err}")
                        for pr_num in changes:
                            if pr_num not in ledger_prs:
                                errs.append(
                                    f"{n}: change #{pr_num} has no row in frontier/ledger.tsv"
                                )
                            if merged_prs is not None and pr_num not in merged_prs:
                                errs.append(
                                    f"{n}: change #{pr_num} has no merge commit on origin/master"
                                )
            elif kind == "discovery":
                evidence = c.get("evidence")
                if (
                    not isinstance(evidence, list)
                    or not evidence
                    or not all(isinstance(p, str) and p.strip() for p in evidence)
                ):
                    errs.append(
                        f"{n}: news_kind 'discovery' past idea requires non-empty evidence = [paths]"
                    )
                else:
                    for ev_path in evidence:
                        if not (REPO / ev_path).exists():
                            errs.append(f"{n}: evidence path does not exist: {ev_path}")
        late = c in cards and STAGES.index(c.get("stage", "idea")) > STAGES.index("review")
        if late and (not c.get("reviewed_by") or c.get("reviewed_by") == c.get("drafted_by")):
            errs.append(
                f"{n}: past review, it needs reviewed_by, by a different agent from drafted_by"
            )
    for c in claims:
        if c["id"].startswith("release-") and c.get("status") not in ("done", "dropped") and c.get("priority") != 1:
            errs.append(f"{c['_path'].name}: a release claim is P1 (priority = 1), always")
    due = release_due(claims)
    if due:
        warn.append(
            f"release due: {len(due[1])} software commit(s) on master since {due[0]} and no live "
            "release claim; open release-<version> at priority 1 (docs/RELEASING.md section 1)"
        )
    busy = [c["id"] for c in cards if not c.get("legacy") and c.get("stage") in IN_PROGRESS]
    if len(busy) > 1:
        errs.append(
            f"blog: {len(busy)} posts in progress ({', '.join(busy)}); one at a time between "
            "outline and ready: publish or drop one first"
        )
    if pr_event:
        error = check_pr_lineage(pr_event, claims + cards)
        if error:
            errs.append(error)
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
    try:
        base_cfg = config("origin/master") if base else config()
        local_cfg = config()
    except (OSError, ValueError) as exc:
        print(f"error: invalid frontier configuration: {exc}")
        return 1
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
                f"protected files changed ({', '.join(hit)}): check actual authority and substantive review requirements; RULES D needs Louis"
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
    all_holds = holds(strict=False)
    held = {k: v for k, (_, v) in all_holds.items() if live(v)}
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
    due = release_due(claims)
    print(
        "\n## Release due (P1)\n"
        + (
            f"- {len(due[1])} software commit(s) on master since {due[0]}, and no release claim: "
            "open `release-<version>` at priority 1 and take it (docs/RELEASING.md)"
            if due
            else "- nothing"
        )
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
    print(
        "\n## Population (observed presence, not reservations; `frontier.py arrive` to appear)\n"
        + "\n".join(presence_population(all_holds))
    )
    prs, pr_error = read_open_prs()
    review_lines, pending = review_projection(claims + cards, prs, pr_error)
    print("\n## Awaiting review or merge (finish before starting)\n" + "\n".join(review_lines))
    print("Independent integration: take-review ID --pr N --head SHA --by AGENT; "
          "yield-review or finish-review. No additional work slot required.")
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
        and c["id"] not in pending
        and not c.get("needs_louis")
        and (not can or set(c["needs"]) <= can)
        and not any(_flag_matches(h, c["id"], c) for _, h in flags)
    ]
    take.sort(key=lambda c: (c["priority"], c["id"]))
    print("\n## What you can take (highest priority first)")
    if review_lines[0].startswith("- UNAVAILABLE:"):
        print("- Review discovery unavailable: these local suggestions are not a verified queue.")
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
    if github_repository():
        prs, error = read_open_prs()
        lines, pending = review_projection([c], prs, error)
        if lines[0].startswith("- UNAVAILABLE:"):
            sys.exit("cannot verify pending work before acquisition")
        if cid in pending:
            sys.exit(f"{cid} already has a pending PR; finish it before starting again")
    admission = admission_snapshot()
    all_holds = holds()
    if any(live(v) for _, v in all_holds.values()):
        all_holds = reconcile_integrations(all_holds, fetch_master=False)
    if sum(live(v) for _, v in all_holds.values()) >= reservation_limit():
        sys.exit(
            "the live-hold budget is full (max_live_holds); take nothing, or help finish something held"
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
    ok, out = write_lease(cid, new, sha, sha, admission)
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
    if not live(lease) or lease.get("phase") == "parked" or (
        dt.datetime.fromisoformat(lease["expires"]) <= now()
    ):
        sys.exit(
            f"{cid}'s hold by {session} has lapsed: re-claim it before any consequential action"
        )
    return sha, lease


# ---------------------------------------------------------------------------------------------
# Presence: optional, observational, self-declared. Never a credential, a lease or a slot.
#
# Each independent participant instance may publish one small `presence.toml` on its own ref,
# refs/heads/presence/<participant>/<instance>, outside master. The instance id is random and
# opaque, kept in the checkout's (per-worktree) Git directory for the participant's lifetime,
# and distinct from any per-claim lease session. Writes are throttled, compare-and-swapped per
# ref, never touch the admission epoch or hold/*, and never fail the command they ride on.
# Counts of observed instances are reported separately from live reservations; age past the
# TTL means "stale/unknown", never "dead" or "available".
# ---------------------------------------------------------------------------------------------

PRESENCE_PREFIX = "refs/heads/presence"
PRESENCE_FILE = "frontier-presence.toml"
PRESENCE_TTL = dt.timedelta(hours=2)  # seen within this: "fresh"; older: "stale/unknown"
PRESENCE_THROTTLE = dt.timedelta(minutes=15)  # an unchanged record is re-pushed at most this often
PRESENCE_PRUNE_AFTER = dt.timedelta(days=7)  # records this old (or departed past the TTL) may be pruned
PRESENCE_PRUNE_MAX = 3  # at most this many CAS deletions per write
PRESENCE_MAX_RECORDS = 256  # records read; the rest are counted as truncated
PRESENCE_MAX_BYTES = 2048
PRESENCE_MAX_CAPS = 16
PRESENCE_SKEW = dt.timedelta(minutes=10)
PRESENCE_STATES = ("active", "idle", "integrating", "departed")
PRESENCE_TIMEOUT = 30
_TOKEN = re.compile(r"[a-z0-9][a-z0-9-]{0,31}")
_INSTANCE = re.compile(r"[0-9a-f]{16}")
_CLAIM_ID = re.compile(r"[a-z0-9][a-z0-9._-]{0,79}")
_PRESENCE_KEYS = {
    "schema", "participant", "instance", "state", "claim", "capabilities", "started", "last_seen"
}


def _pgit(*args, inp=None, env=None, binary=False):
    """Run git for presence: bounded in time, never raising, never prompting."""
    try:
        r = subprocess.run(
            ["git", "-C", str(REPO), *args],
            input=inp,
            capture_output=True,
            text=not binary,
            timeout=PRESENCE_TIMEOUT,
            env={**os.environ, "GIT_TERMINAL_PROMPT": "0", **(env or {})},
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return 1, f"git {args[0]} failed: {exc}"
    if binary:
        return r.returncode, r.stdout
    return r.returncode, r.stdout.strip() + ("\n" + r.stderr.strip() if r.returncode else "")


def _presence_token(value, what):
    """A safe ref/record token, or ValueError."""
    if not isinstance(value, str) or not _TOKEN.fullmatch(value):
        raise ValueError(f"{what} must match {_TOKEN.pattern}: {value!r}")
    return value


def _presence_caps(caps):
    """Sorted unique capability tokens, bounded."""
    out = sorted({_presence_token(c, "capability") for c in caps})
    if len(out) > PRESENCE_MAX_CAPS:
        raise ValueError(f"at most {PRESENCE_MAX_CAPS} capabilities")
    return out


def _presence_time(value):
    """An aware ISO timestamp, or ValueError."""
    t = dt.datetime.fromisoformat(value) if isinstance(value, str) and len(value) <= 40 else None
    if t is None or t.tzinfo is None:
        raise ValueError(f"bad timestamp {value!r}")
    return t


def presence_validate(rec, participant=None, instance=None):
    """Return the record if it is a well-formed presence snapshot (optionally for this ref)."""
    if not isinstance(rec, dict) or set(rec) != _PRESENCE_KEYS or type(rec["schema"]) is not int \
            or rec["schema"] != 1:
        raise ValueError("unexpected keys or schema")
    _presence_token(rec["participant"], "participant")
    if not isinstance(rec["instance"], str) or not _INSTANCE.fullmatch(rec["instance"]):
        raise ValueError("bad instance")
    if rec["state"] not in PRESENCE_STATES:
        raise ValueError("bad state")
    if not isinstance(rec["claim"], str) or (rec["claim"] and not _CLAIM_ID.fullmatch(rec["claim"])):
        raise ValueError("bad claim")
    if not isinstance(rec["capabilities"], list) or rec["capabilities"] != _presence_caps(
        rec["capabilities"]
    ):
        raise ValueError("bad capabilities")
    if _presence_time(rec["last_seen"]) < _presence_time(rec["started"]):
        raise ValueError("last_seen precedes started")
    if participant is not None and (rec["participant"], rec["instance"]) != (participant, instance):
        raise ValueError("record does not match its ref")
    return rec


def presence_body(rec):
    """Serialise a validated record as TOML."""
    presence_validate(rec)
    lines = []
    for k in ("schema", "participant", "instance", "state", "claim", "capabilities", "started",
              "last_seen"):
        v = rec[k]
        if isinstance(v, int):
            lines.append(f"{k} = {v}")
        elif isinstance(v, list):
            lines.append(f"{k} = [" + ", ".join(f'"{x}"' for x in v) + "]")
        else:
            lines.append(f'{k} = "{v}"')
    return "\n".join(lines) + "\n"


def _presence_local_path():
    """The per-worktree file holding this checkout's instance; None if unavailable."""
    rc, out = _pgit("rev-parse", "--git-path", PRESENCE_FILE)
    if rc or not out:
        return None
    p = pathlib.Path(out)
    return p if p.is_absolute() else REPO / p


def presence_local():
    """Return the checkout's local presence state, or None (no arrival, or departed)."""
    p = _presence_local_path()
    if not p or not p.exists():
        return None
    try:
        d = tomllib.loads(p.read_text(encoding="utf-8"))
        presence_validate(d["record"])
    except (OSError, ValueError, KeyError, TypeError, tomllib.TOMLDecodeError):
        return None
    return None if d["record"]["state"] == "departed" else d


def _presence_save_local(record, pushed_at, sha):
    """Persist the instance, the last pushed record and the ref's last known SHA."""
    p = _presence_local_path()
    if not p:
        return
    text = f'pushed_at = "{pushed_at}"\nsha = "{sha}"\n\n[record]\n' + presence_body(record)
    tmp = p.with_name(p.name + ".tmp")
    tmp.write_text(text, encoding="utf-8")
    tmp.replace(p)


def presence_ref(record):
    """The record's own ref."""
    return f"{PRESENCE_PREFIX}/{record['participant']}/{record['instance']}"


def _presence_remote_sha(ref):
    """(ok, sha or '') for one presence ref on origin."""
    rc, out = _pgit("ls-remote", "origin", ref)
    if rc:
        return False, ""
    for line in out.splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[1] == ref and re.fullmatch(r"[0-9a-f]{40}", fields[0]):
            return True, fields[0]
    return True, ""


def _presence_push(record, expected):
    """Write `record` to its ref with CAS on `expected` (a SHA, or '' for absent)."""
    identity = lease_identity()
    if identity is None:
        return False, "repository-local Git identity is not configured"
    rc, blob = _pgit("hash-object", "-w", "--stdin", inp=presence_body(record))
    if rc:
        return False, blob
    rc, tree = _pgit("mktree", inp=f"100644 blob {blob}\tpresence.toml\n")
    if rc:
        return False, tree
    # Fast-forward chain, so no proxy has to accept a non-fast-forward update.
    args = ["commit-tree", tree, "-m", f"presence {record['participant']} {record['instance']}"]
    args += ["-p", expected] if expected else []
    rc, sha = _pgit(*args, env=identity)
    if rc:
        return False, sha
    if expected:
        _pgit("fetch", "-q", "--no-tags", "--no-write-fetch-head", "origin", expected)
    ref = presence_ref(record)
    rc, out = _pgit(
        "push", "--porcelain", "--no-verify", f"--force-with-lease={ref}:{expected}", "origin",
        f"{sha}:{ref}",
    )
    return rc == 0, sha if rc == 0 else out


def presence_write(record, local=None, *, force=False):
    """Publish `record` unless throttled; return (pushed, message). Never raises."""
    try:
        record = presence_validate(dict(record))
        t = now()
        if local and not force:
            prev = dict(local["record"])
            same = {k: v for k, v in prev.items() if k != "last_seen"} == {
                k: v for k, v in record.items() if k != "last_seen"
            }
            pushed = _presence_time(local.get("pushed_at", "1970-01-01T00:00:00+00:00"))
            if same and t - pushed < PRESENCE_THROTTLE:
                return False, "throttled"
        expected = (local or {}).get("sha", "")
        if local and local["record"]["instance"] != record["instance"]:
            expected = ""
        ok, out = _presence_push(record, expected)
        if not ok:
            # Our record was refreshed elsewhere or pruned: re-read once and retry.
            seen, expected = _presence_remote_sha(presence_ref(record))
            if not seen:
                return False, "origin unavailable"
            ok, out = _presence_push(record, expected)
            if not ok:
                return False, "not written (CAS lost twice)"
        _presence_save_local(record, t.isoformat(), out)
        return True, out
    except Exception as exc:  # noqa: BLE001 - presence must never break the command it rides on
        return False, f"presence error: {exc}"


def presence_records(fetch=True):
    """Read every presence record on origin.

    Returns {"error": str|None, "records": [(ref, sha, record|None, reason)], "truncated": n}.
    """
    out = {"error": None, "records": [], "truncated": 0}
    if fetch:
        rc, msg = _pgit(
            "fetch", "-q", "--prune", "--no-tags", "origin",
            f"+{PRESENCE_PREFIX}/*:refs/remotes/origin/presence/*",
        )
        if rc:
            out["error"] = "origin unavailable"
            return out
    rc, listing = _pgit(
        "for-each-ref", "--format=%(objectname) %(refname)", "refs/remotes/origin/presence"
    )
    if rc:
        out["error"] = "cannot list presence refs"
        return out
    entries = []
    for line in listing.splitlines():
        sha, ref = line.split(" ", 1)
        entries.append((sha, ref))
    out["truncated"] = max(0, len(entries) - PRESENCE_MAX_RECORDS)
    entries = entries[:PRESENCE_MAX_RECORDS]
    if not entries:
        return out
    # Size and type first, so an oversized or non-blob record is never read into memory.
    rc, checks = _pgit(
        "cat-file", "--batch-check",
        inp="".join(f"{sha}:presence.toml\n" for sha, _ in entries),
    )
    lines = checks.splitlines() if rc == 0 else []
    blobs = [None] * len(entries)
    small = []
    for i in range(len(entries)):
        fields = lines[i].split() if i < len(lines) else []
        if len(fields) == 3 and fields[1] == "blob" and fields[2].isdigit():
            if int(fields[2]) <= PRESENCE_MAX_BYTES:
                small.append((i, fields[0]))
            else:
                blobs[i] = "oversized"
    if small:
        rc, raw = _pgit(
            "cat-file", "--batch", inp="".join(f"{oid}\n" for _, oid in small).encode(),
            binary=True,
        )
        pos = 0
        for k, (i, _oid) in enumerate(small):
            try:
                if rc:
                    raise ValueError("cat-file failed")
                nl = raw.index(b"\n", pos)
                header = raw[pos:nl].split()
                pos = nl + 1
                if len(header) != 3:
                    raise ValueError("bad cat-file header")
                size = int(header[2])
                if header[1] != b"blob" or size > PRESENCE_MAX_BYTES or pos + size > len(raw):
                    raise ValueError("unexpected object")
                blobs[i] = raw[pos : pos + size]
                pos += size + 1
            except ValueError:
                # Out of step: everything from here on is unreadable, not misattributed.
                for j, _oid in small[k:]:
                    blobs[j] = None
                break
    for (sha, ref), blob in zip(entries, blobs):
        tail = ref[len("refs/remotes/origin/presence/") :].split("/")
        rec, reason = None, ""
        try:
            if len(tail) != 2:
                raise ValueError("unexpected ref shape")
            if blob is None:
                raise ValueError("no readable presence.toml blob")
            if blob == "oversized":
                raise ValueError("record too large")
            rec = presence_validate(tomllib.loads(blob.decode("utf-8")), tail[0], tail[1])
        except Exception as exc:  # noqa: BLE001 - untrusted input: RecursionError, TypeError...
            rec, reason = None, (str(exc) or type(exc).__name__)[:80]
        out["records"].append((f"{PRESENCE_PREFIX}/{'/'.join(tail)}", sha, rec, reason))
    return out


def presence_classify(rec, t=None):
    """Classify a record as fresh, stale, departed, expired or future by its self-declared times.

    "expired" (unseen past the prune horizon) keeps records that a proxy refused to delete from
    inflating the stale count for ever.
    """
    t = t or now()
    seen = _presence_time(rec["last_seen"])
    if t - seen > PRESENCE_PRUNE_AFTER:
        return "expired"
    if rec["state"] == "departed":
        return "departed"
    if seen - t > PRESENCE_SKEW:
        return "future"
    return "fresh" if t - seen <= PRESENCE_TTL else "stale"


def presence_prune(limit=PRESENCE_PRUNE_MAX):
    """Delete at most `limit` long-stale or departed records, each guarded by its exact SHA.

    A record refreshed meanwhile survives (the CAS fails). A proxy that refuses deletions just
    leaves the record, which readers still classify by age. Returns the number deleted.
    """
    found = presence_records()
    if found["error"]:
        return 0
    t, done = now(), 0
    for ref, sha, rec, _ in found["records"]:
        if done >= limit:
            break
        if rec is None:
            continue  # malformed records are reported, not deleted: someone should look
        age = t - _presence_time(rec["last_seen"])
        if age > PRESENCE_PRUNE_AFTER or (rec["state"] == "departed" and age > PRESENCE_TTL):
            rc, _ = _pgit("push", "--porcelain", "--no-verify",
                          f"--force-with-lease={ref}:{sha}", "origin", f":{ref}")
            done += rc == 0
    return done


def presence_population(all_holds=None, fetch=True):
    """Lines describing observed instances, kept apart from live reservations; never raises."""
    try:
        return _presence_population(all_holds, fetch)
    except Exception as exc:  # noqa: BLE001 - presence must never break brief or status
        return [f"- observed instances: unknown (error: {type(exc).__name__})"]


def _presence_population(all_holds, fetch):
    """Build presence_population's lines."""
    found = presence_records(fetch=fetch)
    lines = []
    if found["error"]:
        lines.append(f"- observed instances: unknown ({found['error']})")
    else:
        t = now()
        groups = {
            "fresh": [], "stale": [], "departed": [], "expired": [], "future": [], "malformed": []
        }
        for ref, _, rec, reason in found["records"]:
            if rec is None:
                groups["malformed"].append((ref, reason))
            else:
                groups[presence_classify(rec, t)].append(rec)
        ttl_h = PRESENCE_TTL.total_seconds() / 3600
        lines.append(
            f"- observed instances (self-declared, untrusted): {len(groups['fresh'])} fresh "
            f"(seen within {ttl_h:g} h), {len(groups['stale']) + len(groups['future'])} stale or "
            f"unknown, {len(groups['departed'])} departed, {len(groups['malformed'])} malformed"
            + (f", {len(groups['expired'])} expired (unseen 7+ days)" if groups["expired"] else "")
            + (f", {found['truncated']} not read" if found["truncated"] else "")
        )
        for rec in sorted(groups["fresh"], key=lambda r: (r["participant"], r["instance"])):
            claim_txt = f" on {rec['claim']}" if rec["claim"] else ""
            lines.append(
                f"  - {rec['participant']}/{rec['instance'][:8]}: {rec['state']}{claim_txt}; "
                f"can {','.join(rec['capabilities']) or 'unstated'}; seen {rec['last_seen']}"
            )
        caps = {}
        for rec in groups["fresh"]:
            for c in rec["capabilities"]:
                caps[c] = caps.get(c, 0) + 1
        if caps:
            lines.append(
                "  - fresh capabilities: "
                + ", ".join(f"{c} {n}" for c, n in sorted(caps.items()))
            )
        for ref, reason in groups["malformed"][:5]:
            lines.append(f"  - malformed {ref}: {reason}")
    if all_holds is not None:
        lv = [v for _, v in all_holds.values() if live(v)]
        phases = {}
        for v in lv:
            p = v.get("phase") or "work"
            phases[p] = phases.get(p, 0) + 1
        lines.append(
            f"- live reservations (hold/*, not instances): {len(lv)}"
            + (" (" + ", ".join(f"{n} {p}" for p, n in sorted(phases.items())) + ")" if lv else "")
        )
    return lines


def presence_arrive(participant, caps, state="idle"):
    """Start (or resume) this checkout's instance and publish it now."""
    _presence_token(participant, "participant")
    caps = _presence_caps(caps)
    if state not in PRESENCE_STATES or state == "departed":
        raise ValueError(f"state must be one of {', '.join(PRESENCE_STATES[:-1])}")
    local = presence_local()
    t = now().isoformat()
    if local and local["record"]["participant"] == participant:
        rec = {**local["record"], "capabilities": caps, "state": state, "last_seen": t}
    else:
        local = None
        rec = {
            "schema": 1, "participant": participant, "instance": secrets.token_hex(8),
            "state": state, "claim": "", "capabilities": caps, "started": t, "last_seen": t,
        }
    result = presence_write(rec, local, force=True)
    if result[0]:
        with contextlib.suppress(Exception):
            presence_prune()
    return rec, result


def presence_touch(state=None, claim_id=None, caps=None, *, force=False):
    """Refresh this checkout's record if it has arrived; (pushed, message)."""
    local = presence_local()
    if not local:
        return False, "no presence (run `frontier.py arrive --as FAMILY --can ...` to opt in)"
    rec = dict(local["record"])
    if state is not None:
        if state not in PRESENCE_STATES or state == "departed":
            return False, f"bad state {state!r}"
        rec["state"] = state
    if claim_id is not None:
        if claim_id and not _CLAIM_ID.fullmatch(claim_id):
            return False, f"bad claim {claim_id!r}"
        rec["claim"] = claim_id
    if caps is not None:
        try:
            rec["capabilities"] = _presence_caps(caps)
        except ValueError as exc:
            return False, str(exc)
    rec["last_seen"] = now().isoformat()
    return presence_write(rec, local, force=force)


def presence_depart():
    """Publish a final departed record and forget the local instance."""
    local = presence_local()
    if not local:
        return False, "no presence to depart"
    rec = {**local["record"], "state": "departed", "claim": "", "last_seen": now().isoformat()}
    ok, msg = presence_write(rec, local, force=True)
    if ok:
        with contextlib.suppress(Exception):
            presence_prune()
    else:  # forget locally anyway: the remote record ages into "stale/unknown"
        p = _presence_local_path()
        if p:
            with contextlib.suppress(OSError):
                p.unlink()
    return ok, msg


def presence_checkpoint(state=None, claim_id=None, caps=None, *, ended=None):
    """Opportunistic, throttled refresh at a normal frontier checkpoint, for arrived checkouts.

    `ended` names a claim whose work just ended: the record goes idle only if it was on it.
    Failures print one line to stderr and never change the command's outcome.
    """
    if os.environ.get("FRONTIER_PRESENCE") == "off":
        return
    try:
        local = presence_local()
        if local:
            if ended is not None:
                if local["record"]["claim"] != ended:
                    return
                state, claim_id = "idle", ""
            ok, msg = presence_touch(state, claim_id, caps)
            if not ok and msg not in ("throttled",):
                print(f"(presence not refreshed: {msg})", file=sys.stderr)
    except Exception as exc:  # noqa: BLE001
        print(f"(presence not refreshed: {exc})", file=sys.stderr)


def main(argv):
    """Dispatch a subcommand."""
    cmd = argv[1] if len(argv) > 1 else "brief"
    opt = lambda name, default=None: argv[argv.index(name) + 1] if name in argv else default  # noqa: E731
    hours = float(opt("--hours", "8"))
    can = [c for c in (opt("--can") or "").split(",") if c]
    if cmd == "brief":
        presence_checkpoint(caps=can or None)
        brief(set(can))
    elif cmd == "status":
        (ROOT / "STATUS.md").write_text(render())
        print("wrote frontier/STATUS.md")
    elif cmd == "check":
        return check(opt("--pr-event"))
    elif cmd == "news":
        return news()
    elif cmd == "holds":
        for k, (sha, v) in sorted(holds().items()):
            state = "live" if live(v) else ("released" if v.get("released") else "expired")
            print(
                f"{k}: {state}, {v.get('holder')} (session {v.get('session')}) until {v.get('expires')} [{sha[:8]}]"
            )
    elif cmd == "claim":
        claim(argv[2], opt("--by", "unknown"), hours)
        presence_checkpoint("active", argv[2])
    elif cmd == "take-review":
        take_review(argv[2], int(opt("--pr", "0")), opt("--head"), opt("--by", "unknown"), hours)
        presence_checkpoint("integrating", argv[2])
    elif cmd == "yield-review":
        yield_review(argv[2], opt("--session"))
        presence_checkpoint(ended=argv[2])
    elif cmd in ("finish-review", "cancel-review"):
        finish_review(argv[2], int(opt("--pr", "0")), opt("--head"), cancelled=cmd == "cancel-review")
        presence_checkpoint(ended=argv[2])
    elif cmd == "arrive":
        try:
            rec, (ok, msg) = presence_arrive(opt("--as", "unknown"), can, opt("--state", "idle"))
        except ValueError as exc:
            sys.exit(f"not arrived: {exc}")
        if not ok:
            print(f"presence not published ({msg}); claim work is unaffected", file=sys.stderr)
            return 1
        print(f"present as {rec['participant']}/{rec['instance']} ({rec['state']}); "
              "refreshed at checkpoints, `depart` when leaving")
    elif cmd == "touch":
        ok, msg = presence_touch(opt("--state"), opt("--claim"), can or None,
                                 force="--force" in argv)
        print("presence refreshed" if ok else f"presence not refreshed: {msg}")
        return 0 if ok or msg == "throttled" else 1
    elif cmd == "depart":
        ok, msg = presence_depart()
        print("departed" if ok else f"departure not published: {msg} (local instance forgotten)")
        return 0 if ok else 1
    elif cmd == "population":
        print("\n".join(presence_population(holds(strict=False))))
    elif cmd == "verify":
        cid = argv[2]
        all_holds = holds()
        _, lease = mine(cid, opt("--session"), all_holds)
        if lease.get("phase") == "integration":
            check_review_pr(cid, lease["pr"], lease["pr_head"])
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
        admission = admission_snapshot()
        sha, lease = mine(argv[2], opt("--session"))
        updated = {**lease, "expires": (now() + dt.timedelta(hours=hours)).isoformat()}
        ok, out = write_lease(argv[2], updated, sha, sha, admission)
        if not ok:
            print(f"not renewed:\n{out}", file=sys.stderr)
            return 1
        print(f"renewed until {updated['expires']}")
        presence_checkpoint()
    elif cmd == "release":
        sha, lease = holds().get(argv[2], (None, None))
        if not lease or lease.get("session") != opt("--session"):
            sys.exit(f"{argv[2]} is not held by session {opt('--session')}")
        if lease.get("phase") in ("integration", "parked"):
            sys.exit("use finish-review after merge, or yield-review before leaving")
        updated = {**lease, "released": True}
        ok, out = write_lease(argv[2], updated, sha, sha)
        if not ok:
            print(f"not released (someone took over):\n{out}", file=sys.stderr)
            return 1
        print(f"released {argv[2]}")
        presence_checkpoint(ended=argv[2])
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
