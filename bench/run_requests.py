#!/usr/bin/env python3
"""Run requests: workstation ready signals published as lightweight refs, reconciled by a
trusted job into the ready/<recipe> and hold/<recipe> tags the deployed runner already reads.

Any participant that can push a branch can ask for a run of a frozen, reviewed, green recipe
under the 9 November standing authorisation, with no interactive workflow dispatch, no pull
request and no master commit per signal:

    python3 bench/run_requests.py publish --recipe R --sha S --branch B \
        --claim C --session X --pr N --review ID [--supersedes OLD]
    python3 bench/run_requests.py publish --action hold --recipe R
    python3 bench/run_requests.py withdraw refs/heads/run-request/R/ID

A request is one commit on refs/heads/run-request/<recipe>/<12 hex> holding request.toml. It
confers no authority. The reconciler (`reconcile --apply`, run only from the default branch by
.github/workflows/run-signal.yml on a schedule) reads every request as untrusted data and sets
ready/<recipe> only when, at that moment, all of these hold:

  - the request parses strictly (known keys, bounded size, safe identifiers) and its ref names
    the same recipe; it is under 72 hours old and not withdrawn;
  - the default branch's bench/workstation-queue.txt lists the recipe on that branch;
  - the default branch's claim C is a queue claim (mechanism workstation-queue, or it owns the
    queue file), needs nothing from Louis, and hold/C is live, unreleased, with session X;
  - S is on origin's B, PR N's head is exactly S on B, review ID is at S and no review at S
    requests changes;
  - every check run on S passed or was skipped (a macOS job that never completed, was
    cancelled or timed out is not waited for, per CLAUDE.md; a macOS failure still holds),
    there is at least one, and the legacy commit status is success or absent;
  - hold/<recipe> is not set; ready/<recipe> is unset, already S, or exactly SUPERSEDES;
  - no receipt run-receipt/<recipe>/<S> exists.

Activation is one atomic push of ready/<recipe> (with a lease on its old value) and the
annotated receipt run-receipt/<recipe>/<S>. The receipt is the tombstone: once a run has been
signalled from a request, no request (old, replayed or new) signals that recipe at that commit
again, even after ready/<recipe> is cleared. A deliberate repeat at the same commit still goes
through the manual dispatch of run-signal.yml, with a delta saying why. A hold request needs
only a queued recipe: it can only stop work. Nothing here executes request content, checks
out a candidate or interpolates request text into a shell.
"""

from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import json
import os
import re
import secrets
import subprocess
import sys
import tomllib
import urllib.error
import urllib.request

RECIPE = re.compile(r"^[a-z0-9][a-z0-9-]{0,63}$")
SHA = re.compile(r"^[0-9a-f]{40}$")
BRANCH = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._/-]{0,127}$")
CLAIM = re.compile(r"^[a-z0-9][a-z0-9-]{0,79}$")
SESSION = re.compile(r"^[0-9a-f]{8,64}$")
REQUEST_REF = re.compile(r"^refs/heads/run-request/([a-z0-9][a-z0-9-]{0,63})/([0-9a-f]{12})$")
MAX_BYTES = 4096
MAX_AGE = dt.timedelta(hours=72)
MAX_SKEW = dt.timedelta(minutes=10)
READY_KEYS = {
    "version",
    "action",
    "recipe",
    "sha",
    "branch",
    "claim",
    "session",
    "pr",
    "review",
    "created",
}
OPTIONAL_KEYS = {"supersedes", "note"}
HOLD_KEYS = {"version", "action", "recipe", "created"}
WITHDRAW_KEYS = {"version", "action", "recipe", "created"}
PASSED = {"success", "neutral", "skipped"}
MACOS_OUTAGE = {"cancelled", "timed_out", "startup_failure", "action_required", "stale"}


class Unavailable(Exception):
    """A remote read failed: the request is refused this time and looked at again next time."""


class Refused(Exception):
    pass


@dataclasses.dataclass
class Decision:
    ref: str
    recipe: str
    outcome: str  # activated | eligible | done | withdrawn | refused
    reason: str = ""

    def line(self):
        return f"{self.outcome:<9} {self.ref}{': ' + self.reason if self.reason else ''}"


def render(fields: dict) -> str:
    """request.toml: one `key = value` line per field; strings JSON-quoted (valid TOML)."""
    return "".join(f"{k} = {json.dumps(v)}\n" for k, v in fields.items() if v is not None)


# ---------------------------------------------------------------- git and the GitHub API


def _git(cwd, *args, input=None, env=None, check=True):
    p = subprocess.run(
        ["git", *args],
        cwd=cwd,
        input=input,
        text=True,
        capture_output=True,
        env={**os.environ, **(env or {})},
    )
    if check and p.returncode:
        raise Unavailable(f"git {' '.join(args[:2])}: {p.stderr.strip()[:300]}")
    return p


class GitHubAPI:
    def __init__(self, token: str, base="https://api.github.com"):
        self.token, self.base = token, base

    def get(self, path):
        req = urllib.request.Request(
            f"{self.base}/{path}",
            headers={
                "Authorization": f"Bearer {self.token}",
                "Accept": "application/vnd.github+json",
                "X-GitHub-Api-Version": "2022-11-28",
                "User-Agent": "run-requests",
            },
        )
        try:
            with urllib.request.urlopen(req, timeout=30) as r:
                return json.load(r)
        except (urllib.error.URLError, TimeoutError, ValueError) as e:
            raise Unavailable(f"GitHub API {path.split('?')[0]}: {e}") from e


class World:
    """Everything the reconciler reads: origin's refs and default branch (git), and the GitHub
    API (pull requests, reviews, checks). Request content is only ever read as a blob."""

    def __init__(self, remote, repo, api, cwd, default="master"):
        self.remote, self.repo, self.api, self.cwd, self.default = remote, repo, api, cwd, default
        self._tags = None
        self._fetch(f"+refs/heads/{default}:refs/reconcile/default")

    def _fetch(self, *refspecs):
        _git(self.cwd, "fetch", "-q", "--no-tags", self.remote, *refspecs)

    def _show(self, rev, path, limit=1 << 20):
        p = _git(self.cwd, "cat-file", "-s", f"{rev}:{path}", check=False)
        if p.returncode or int(p.stdout) > limit:
            return None
        return _git(self.cwd, "cat-file", "blob", f"{rev}:{path}").stdout

    # requests
    def requests(self):
        out = _git(self.cwd, "ls-remote", self.remote, "refs/heads/run-request/*").stdout
        refs = [line.split("\t")[1] for line in out.splitlines() if "\t" in line]
        refs = sorted(r for r in refs if REQUEST_REF.match(r))
        if refs:
            self._fetch("+refs/heads/run-request/*:refs/reconcile/request/*")
        found = []
        for ref in refs:
            local = "refs/reconcile/request/" + ref[len("refs/heads/run-request/") :]
            meta = _git(self.cwd, "show", "-s", "--format=%H %ct", local).stdout.split()
            text = self._show(local, "request.toml", MAX_BYTES)
            found.append(
                (ref, meta[0], dt.datetime.fromtimestamp(int(meta[1]), dt.timezone.utc), text)
            )
        return sorted(found, key=lambda r: (r[2], r[0]))

    # the default branch
    def queue(self):
        text = self._show("refs/reconcile/default", "bench/workstation-queue.txt") or ""
        entries = {}
        for line in text.splitlines():
            parts = line.split()
            if parts and not line.startswith("#") and len(parts) >= 2:
                entries.setdefault(parts[0], parts[1])
        return entries

    def claim(self, claim_id):
        text = self._show("refs/reconcile/default", f"frontier/claims/{claim_id}.toml")
        return tomllib.loads(text) if text else None

    def lease(self, claim_id):
        p = _git(
            self.cwd,
            "fetch",
            "-q",
            "--no-tags",
            self.remote,
            f"+refs/heads/hold/{claim_id}:refs/reconcile/hold/{claim_id}",
            check=False,
        )
        if p.returncode:
            if "couldn't find remote ref" in p.stderr:
                return None
            raise Unavailable(f"hold/{claim_id}: {p.stderr.strip()[:200]}")
        text = self._show(f"refs/reconcile/hold/{claim_id}", "lease.toml", MAX_BYTES)
        return tomllib.loads(text) if text else None

    def on_branch(self, sha, branch):
        local = f"refs/reconcile/branch/{branch}"
        self._fetch(f"+refs/heads/{branch}:{local}")
        return (
            _git(self.cwd, "merge-base", "--is-ancestor", sha, local, check=False).returncode == 0
        )

    def default_head(self):
        return _git(self.cwd, "rev-parse", "refs/reconcile/default").stdout.strip()

    # tags: {name: commit}, annotated tags peeled
    def tags(self):
        if self._tags is None:
            out = _git(
                self.cwd,
                "ls-remote",
                "--tags",
                self.remote,
                "refs/tags/ready/*",
                "refs/tags/hold/*",
                "refs/tags/run-receipt/*",
            ).stdout
            plain, peeled = {}, {}
            for line in out.splitlines():
                sha, ref = line.split("\t")
                name = ref[len("refs/tags/") :]
                (peeled if name.endswith("^{}") else plain)[name.removesuffix("^{}")] = sha
            self._tags = {**plain, **peeled}
        return self._tags

    # the GitHub API
    def pr(self, n):
        return self.api.get(f"repos/{self.repo}/pulls/{n}")

    def reviews(self, n):
        return self.api.get(f"repos/{self.repo}/pulls/{n}/reviews?per_page=100")

    def checks(self, sha):
        return self.api.get(f"repos/{self.repo}/commits/{sha}/check-runs?per_page=100")

    def status(self, sha):
        return self.api.get(f"repos/{self.repo}/commits/{sha}/status")

    # the one write: an atomic push of tags, each with a lease on its old value
    def push_tags(self, updates):
        """updates: [(tag name, target commit, annotation or None, expected old commit or '')]"""
        args, specs = [], []
        for name, target, message, old in updates:
            if message is None:
                specs.append(f"{target}:refs/tags/{name}")
            else:
                _git(
                    self.cwd,
                    "-c",
                    "user.name=run-requests",
                    "-c",
                    "user.email=run-requests@users.noreply.github.com",
                    "tag",
                    "-f",
                    "-a",
                    "-m",
                    message,
                    name,
                    target,
                )
                specs.append(f"refs/tags/{name}:refs/tags/{name}")
            args.append(f"--force-with-lease=refs/tags/{name}:{old}")
        _git(self.cwd, "push", "-q", "--atomic", *args, self.remote, *specs)
        for name, target, _, _ in updates:
            self.tags()[name] = target


# ---------------------------------------------------------------- deciding


def parse(text):
    if text is None:
        raise Refused("no request.toml, or larger than 4096 bytes")
    try:
        req = tomllib.loads(text)
    except tomllib.TOMLDecodeError as e:
        raise Refused(f"not TOML: {e}") from None
    action = req.get("action")
    allowed = {"ready": READY_KEYS, "hold": HOLD_KEYS, "withdraw": WITHDRAW_KEYS}.get(action)
    if allowed is None:
        raise Refused(f"unknown action {str(action)[:20]!r}")
    keys = set(req)
    extra = keys - allowed - (OPTIONAL_KEYS if action == "ready" else {"note"})
    if extra or allowed - keys:
        raise Refused(f"keys: missing {sorted(allowed - keys)}, unknown {sorted(extra)}")
    if req["version"] != 1:
        raise Refused("version is not 1")
    checks = [("recipe", RECIPE)]
    if action == "ready":
        checks += [("sha", SHA), ("branch", BRANCH), ("claim", CLAIM), ("session", SESSION)]
        if "supersedes" in req:
            checks.append(("supersedes", SHA))
        for k in ("pr", "review"):
            if type(req[k]) is not int or req[k] <= 0:
                raise Refused(f"{k} is not a positive integer")
        if ".." in req["branch"] or req["branch"].endswith((".lock", "/")):
            raise Refused("bad branch")
    for k, pattern in checks:
        if not isinstance(req[k], str) or not pattern.match(req[k]):
            raise Refused(f"bad {k}")
    if "note" in req and (not isinstance(req["note"], str) or len(req["note"]) > 500):
        raise Refused("bad note")
    try:
        dt.datetime.fromisoformat(str(req["created"]))
    except ValueError:
        raise Refused("bad created") from None
    return req


def check_ci(world, sha):
    runs = world.checks(sha)
    got = runs.get("check_runs", [])
    if not got:
        raise Refused(f"no check runs on {sha[:12]}")
    if runs.get("total_count", 0) > len(got):
        raise Refused("more than 100 check runs: not all could be read")
    for run in got:
        name, status, concl = run.get("name", "?"), run.get("status"), run.get("conclusion")
        if "macos" in name.lower() and (status != "completed" or concl in MACOS_OUTAGE):
            continue  # CLAUDE.md: GitHub's macOS runners never hold a merge
        if status != "completed" or concl not in PASSED:
            raise Refused(f"check {name!r} is {status}/{concl}")
    st = world.status(sha)
    if st.get("total_count", 0) and st.get("state") != "success":
        raise Refused(f"commit status is {st.get('state')}")


def evaluate_ready(world, req, now):
    recipe, sha, tags = req["recipe"], req["sha"], world.tags()
    if f"run-receipt/{recipe}/{sha}" in tags:
        return "done", "already signalled from a request (receipt exists)"
    if f"hold/{recipe}" in tags:
        raise Refused(f"hold/{recipe} is set")
    if world.queue().get(recipe) != req["branch"]:
        raise Refused(f"the queue does not list {recipe} on {req['branch']}")
    claim = world.claim(req["claim"])
    if not claim:
        raise Refused(f"no claim {req['claim']} on the default branch")
    if claim.get("needs_louis") or claim.get("status") in ("done", "dropped", "idea", "blocked"):
        raise Refused(f"claim {req['claim']} is {claim.get('status')} or needs Louis")
    if "workstation-queue" not in claim.get(
        "mechanisms", []
    ) and "bench/workstation-queue.txt" not in claim.get("owned_paths", []):
        raise Refused(f"claim {req['claim']} is not a workstation queue claim")
    lease = world.lease(req["claim"])
    if (
        not lease
        or lease.get("released") is not False
        or lease.get("id") != req["claim"]
        or lease.get("session") != req["session"]
    ):
        raise Refused(f"no live lease on hold/{req['claim']} for session {req['session']}")
    try:
        if dt.datetime.fromisoformat(lease["expires"]) <= now:
            raise Refused(f"the lease on hold/{req['claim']} expired")
    except (KeyError, TypeError, ValueError):
        raise Refused("the lease has no valid expiry") from None
    if not world.on_branch(sha, req["branch"]):
        raise Refused(f"{sha[:12]} is not on {req['branch']}")
    pr = world.pr(req["pr"])
    if pr.get("head", {}).get("sha") != sha or pr.get("head", {}).get("ref") != req["branch"]:
        raise Refused(f"PR #{req['pr']}'s head is not {sha[:12]} on {req['branch']}")
    reviews = world.reviews(req["pr"])
    if len(reviews) >= 100:
        raise Refused("100 or more reviews: not all could be read")
    named = [r for r in reviews if r.get("id") == req["review"]]
    if (
        not named
        or named[0].get("commit_id") != sha
        or named[0].get("state") not in ("APPROVED", "COMMENTED")
        or not (named[0].get("body") or "").strip()
    ):
        raise Refused(f"review {req['review']} is not a review at {sha[:12]}")
    if any(r.get("commit_id") == sha and r.get("state") == "CHANGES_REQUESTED" for r in reviews):
        raise Refused(f"a review at {sha[:12]} requests changes")
    check_ci(world, sha)
    current = tags.get(f"ready/{recipe}")
    if current not in (None, sha) and req.get("supersedes") != current:
        raise Refused(f"ready/{recipe} is at {current[:12]}; a move needs supersedes = it")
    if current is None and "supersedes" in req:
        raise Refused(f"supersedes is set but ready/{recipe} is not")
    return "eligible", ""


def reconcile(world, now, apply=False):
    decisions = []
    queue_cache = None
    for ref, commit, when, text in world.requests():
        recipe = REQUEST_REF.match(ref).group(1)
        d = Decision(ref, recipe, "refused")
        try:
            if when > now + MAX_SKEW or now - when > MAX_AGE:
                raise Refused(f"expired: published {when.isoformat()}")
            req = parse(text)
            if req["recipe"] != recipe:
                raise Refused("the ref and request.toml name different recipes")
            if req["action"] == "withdraw":
                d.outcome = "withdrawn"
            elif req["action"] == "hold":
                queue_cache = queue_cache if queue_cache is not None else world.queue()
                if recipe not in queue_cache:
                    raise Refused(f"the queue does not list {recipe}")
                if f"hold/{recipe}" in world.tags():
                    d.outcome, d.reason = "done", f"hold/{recipe} is set"
                else:
                    d.outcome = "eligible"
                    if apply:
                        world.push_tags([(f"hold/{recipe}", world.default_head(), None, "")])
                        d.outcome = "activated"
            else:
                d.outcome, d.reason = evaluate_ready(world, req, now)
                if d.outcome == "eligible" and apply:
                    sha = req["sha"]
                    old = world.tags().get(f"ready/{recipe}", "")
                    receipt = json.dumps(
                        {
                            "request": ref,
                            "request_commit": commit,
                            "recipe": recipe,
                            "sha": sha,
                            "branch": req["branch"],
                            "claim": req["claim"],
                            "session": req["session"],
                            "pr": req["pr"],
                            "review": req["review"],
                            "previous": old or None,
                            "at": now.isoformat(),
                        },
                        indent=1,
                    )
                    updates = [(f"run-receipt/{recipe}/{sha}", sha, receipt, "")]
                    if old != sha:
                        updates.insert(0, (f"ready/{recipe}", sha, None, old))
                    world.push_tags(updates)
                    d.outcome = "activated"
        except Refused as e:
            d.outcome, d.reason = "refused", str(e)
        except Unavailable as e:
            d.outcome, d.reason = "refused", f"unavailable (retried next time): {e}"
        decisions.append(d)
    return decisions


# ---------------------------------------------------------------- publishing


def _commit_file(cwd, text, message, now, parent=None):
    env = {"GIT_AUTHOR_DATE": now.isoformat(), "GIT_COMMITTER_DATE": now.isoformat()}
    blob = _git(cwd, "hash-object", "-w", "--stdin", input=text).stdout.strip()
    tree = _git(cwd, "mktree", input=f"100644 blob {blob}\trequest.toml\n").stdout.strip()
    extra = ["-p", parent] if parent else []
    return _git(cwd, "commit-tree", tree, *extra, "-m", message, env=env).stdout.strip()


def publish(
    remote,
    cwd,
    now,
    recipe,
    action="ready",
    sha=None,
    branch=None,
    claim=None,
    session=None,
    pr=None,
    review=None,
    supersedes=None,
    note=None,
):
    fields = {"version": 1, "action": action, "recipe": recipe}
    if action == "ready":
        fields.update(
            sha=sha,
            branch=branch,
            claim=claim,
            session=session,
            pr=pr,
            review=review,
            supersedes=supersedes,
        )
    fields.update(created=now.isoformat(), note=note)
    text = render(fields)
    parse(text)  # refuse locally what the reconciler would refuse
    ref = f"refs/heads/run-request/{recipe}/{secrets.token_hex(6)}"
    commit = _commit_file(
        cwd, text, f"run request: {action} {recipe}" + (f" at {sha}" if sha else ""), now
    )
    _git(cwd, "push", "-q", remote, f"{commit}:{ref}")  # a new ref: never a force push
    return ref


def withdraw(remote, cwd, ref, now):
    m = REQUEST_REF.match(ref)
    if not m:
        raise SystemExit(f"not a request ref: {ref}")
    _git(cwd, "fetch", "-q", "--no-tags", remote, f"+{ref}:refs/reconcile/withdraw")
    parent = _git(cwd, "rev-parse", "refs/reconcile/withdraw").stdout.strip()
    text = render(
        {"version": 1, "action": "withdraw", "recipe": m.group(1), "created": now.isoformat()}
    )
    commit = _commit_file(cwd, text, f"run request: withdraw {m.group(1)}", now, parent)
    _git(cwd, "push", "-q", remote, f"{commit}:{ref}")  # a fast-forward of the request
    return commit


# ---------------------------------------------------------------- command line


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("reconcile", help="decide every request; --apply sets the tags")
    r.add_argument("--apply", action="store_true")
    r.add_argument("--remote", default="origin")
    r.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY", ""))
    r.add_argument("--default", default="master")
    p = sub.add_parser("publish", help="publish one request ref")
    p.add_argument("--action", choices=["ready", "hold"], default="ready")
    p.add_argument("--recipe", required=True)
    for k in ("sha", "branch", "claim", "session", "supersedes", "note"):
        p.add_argument(f"--{k}")
    p.add_argument("--pr", type=int)
    p.add_argument("--review", type=int)
    p.add_argument("--remote", default="origin")
    w = sub.add_parser("withdraw", help="withdraw a published request")
    w.add_argument("ref")
    w.add_argument("--remote", default="origin")
    a = ap.parse_args(argv)
    now = dt.datetime.now(dt.timezone.utc).replace(microsecond=0)
    cwd = _git(".", "rev-parse", "--show-toplevel").stdout.strip()
    if a.cmd == "publish":
        try:
            ref = publish(
                a.remote,
                cwd,
                now,
                a.recipe,
                a.action,
                a.sha,
                a.branch,
                a.claim,
                a.session,
                a.pr,
                a.review,
                a.supersedes,
                a.note,
            )
        except (Refused, Unavailable) as e:
            raise SystemExit(f"not published: {e}") from None
        print(
            f"published {ref}\nIt confers no authority: the scheduled reconciler checks it. "
            f"See its decision in the Run signal workflow's summary, or: "
            f"git ls-remote --tags origin 'ready/{a.recipe}' 'run-receipt/{a.recipe}/*'"
        )
        return 0
    if a.cmd == "withdraw":
        print(f"withdrawn {a.ref} at {withdraw(a.remote, cwd, a.ref, now)}")
        return 0
    token = os.environ.get("GITHUB_TOKEN", "")
    if not (token and a.repo):
        raise SystemExit("reconcile needs GITHUB_TOKEN and --repo (or GITHUB_REPOSITORY)")
    world = World(a.remote, a.repo, GitHubAPI(token), cwd, a.default)
    decisions = reconcile(world, now, apply=a.apply)
    lines = [d.line() for d in decisions] or ["no run requests"]
    print("\n".join(lines))
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as f:
            f.write("## Run requests\n\n```\n" + "\n".join(lines) + "\n```\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
