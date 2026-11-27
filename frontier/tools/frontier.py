"""The frontier: claims, holds, blog cards, deltas, flags and the surface.

    frontier.py brief [--can shell,rust,...]        what to read, what is held, what you can take
    frontier.py status                              rewrite frontier/STATUS.md from files and open PRs
    frontier.py check                               validate the frontier's files
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

A hold is the branch `hold/<id>` on origin: a chain of commits each carrying `lease.toml`. Taking,
renewing, taking over an expired hold and releasing are all pushes guarded by
`--force-with-lease`, so the remote compares and swaps atomically: of two agents claiming at
once, exactly one succeeds. Nothing deletes a branch (some agents' proxies refuse deletions);
a released hold is a commit saying so. Standard library only (Python 3.11+); nothing here needs
GitHub Actions.
"""

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
    flags = open_flags()
    out += ["", "## Open flags", ""] + (
        [f"- `{p.name}`: {h.get('flags')} ({h.get('invariant')})" for p, h in flags] or ["- none"]
    )
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


def check(pr_event=None):
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
        return check(opt("--pr-event"))
    elif cmd == "holds":
        for k, (sha, v) in sorted(holds().items()):
            state = "live" if live(v) else ("released" if v.get("released") else "expired")
            print(
                f"{k}: {state}, {v.get('holder')} (session {v.get('session')}) until {v.get('expires')} [{sha[:8]}]"
            )
    elif cmd == "claim":
        claim(argv[2], opt("--by", "unknown"), hours)
    elif cmd == "take-review":
        take_review(argv[2], int(opt("--pr", "0")), opt("--head"), opt("--by", "unknown"), hours)
    elif cmd == "yield-review":
        yield_review(argv[2], opt("--session"))
    elif cmd in ("finish-review", "cancel-review"):
        finish_review(argv[2], int(opt("--pr", "0")), opt("--head"), cancelled=cmd == "cancel-review")
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
