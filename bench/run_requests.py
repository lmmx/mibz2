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
    queue file) whose text names the recipe, with status open or review and nothing needing
    Louis; hold/C is live, unreleased, with session X;
  - S is on origin's B; PR N is open or merged, its head exactly S on this repository's B;
    review ID is at S and no review at S requests changes;
  - green as CLAUDE.md defines it: every check run on S passed or was skipped, the required
    checks (check-commit, no-ai-attribution) and pre-commit.ci's status have reported success,
    and every Actions workflow on S has finished. Not waited for: a macOS job still waiting for
    a runner, and a cancelled run whose same-named twin passed. A macOS job that started,
    timed out or was cancelled holds (fail closed: use manual dispatch);
  - hold/<recipe> is not set; ready/<recipe> is unset, already S, or exactly SUPERSEDES;
  - no receipt run-receipt/<recipe>/<S> exists, and the request is not older than the
    recipe's latest receipt (so a request can't revive a run or hold after it was cleared).

Activation is one atomic push of ready/<recipe> (leased on its old value) and the annotated
receipt run-receipt/<recipe>/<S>. The receipt is the tombstone: once a run has been signalled
from a request, no request (old, replayed or new) signals that recipe at that commit again,
even after ready/<recipe> is cleared. A deliberate repeat at the same commit still goes
through the manual dispatch of run-signal.yml, with a delta saying why. A hold request needs
only a queued recipe (it can only stop work) and leaves run-receipt/<recipe>/hold-<request
commit>, so a manual clear is never undone by the same request. Withdrawal only stops a
request not yet acted on; an activated run is stopped by a hold. Nothing here executes request
content, checks out a candidate or interpolates request text into a shell.
"""

from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import json
import os
import pathlib
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
# CLAUDE.md: a macOS job waiting for a runner never holds a merge; one that started and then
# failed, timed out or was cancelled may be a real result, so it holds (fail closed)
MACOS_WAITING = {"queued", "waiting", "pending", "requested"}
# a head is green only once these have reported: absent is not green
REQUIRED_CHECKS = {"check-commit", "no-ai-attribution"}
REQUIRED_STATUSES = {"pre-commit.ci - pr"}
CLAIM_OK = {"open", "review"}


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


def _toml(text, what):
    """Parse TOML read from origin (a lease any agent can push, a claim): bad data refuses."""
    try:
        table = tomllib.loads(text)
    except (tomllib.TOMLDecodeError, TypeError, ValueError):
        raise Refused(f"{what} is not valid TOML") from None
    return table


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
        ls = _git(self.cwd, "ls-tree", "-l", rev, "--", path, check=False)
        if ls.returncode or not ls.stdout.strip():
            return None
        lines = ls.stdout.splitlines()
        if len(lines) != 1:
            return None
        head_meta, _, entry_path = lines[0].partition("\t")
        parts = head_meta.split()
        if len(parts) != 4 or parts[0] not in ("100644", "100755") or parts[1] != "blob":
            return None
        if entry_path != path or not parts[3].isdigit() or int(parts[3]) > limit:
            return None
        p = _git(self.cwd, "cat-file", "blob", parts[2], check=False)
        return p.stdout if p.returncode == 0 else None

    # requests
    def requests(self):
        out = _git(self.cwd, "ls-remote", self.remote, "refs/heads/run-request/*").stdout
        refs = [line.split("\t")[1] for line in out.splitlines() if "\t" in line]
        refs = sorted(r for r in refs if REQUEST_REF.match(r))
        refspecs = [
            f"+{r}:refs/reconcile/request/{r[len('refs/heads/run-request/') :]}" for r in refs
        ]
        for i in range(0, len(refspecs), 100):
            self._fetch(*refspecs[i : i + 100])
        found = []
        for ref in refs:
            local = "refs/reconcile/request/" + ref[len("refs/heads/run-request/") :]
            p_meta = _git(self.cwd, "show", "-s", "--format=%H %ct", local, check=False)
            meta = p_meta.stdout.split() if p_meta.returncode == 0 else []
            if len(meta) != 2 or not meta[1].isdigit():
                found.append((ref, "", dt.datetime.fromtimestamp(0, dt.timezone.utc), None))
                continue
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
        return _toml(text, f"claim {claim_id}") if text else None

    def open_flags(self):
        """Read open (or malformed-status) flags from the default branch's frontier/flags/."""
        p = _git(
            self.cwd,
            "ls-tree",
            "-r",
            "--name-only",
            "refs/reconcile/default",
            "--",
            "frontier/flags",
            check=False,
        )
        if p.returncode:
            raise Unavailable("cannot list frontier/flags on default branch")
        active = []
        for rel in sorted(p.stdout.splitlines()):
            name = pathlib.PurePosixPath(rel).name
            if not name.endswith(".md") or name == "README.md":
                continue
            body = self._show("refs/reconcile/default", rel, MAX_BYTES * 8)
            if body is None:
                raise Refused(f"cannot read frontier flag {name}")
            head = {}
            for line in body.splitlines()[:10]:
                if ":" in line and not line.startswith("**"):
                    k, v = line.split(":", 1)
                    head[k.strip()] = v.strip()
            if head.get("status") != "resolved":
                active.append((name, head.get("flags", "").split()))
        return active

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
        return _toml(text, f"hold/{claim_id}'s lease") if text else None

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
            self._raw = plain
        return self._tags

    def raw(self, name):
        """A tag's own value (the tag object, for an annotated tag): what a lease compares."""
        self.tags()
        return self._raw.get(name, "")

    def receipt_times(self, recipe, holds):
        """When each of the recipe's ready receipts (holds=False) or hold receipts (holds=True)
        was written: its tagger date."""
        if not any(n.startswith(f"run-receipt/{recipe}/") for n in self.tags()):
            return []
        local = f"refs/reconcile/receipt/{recipe}/"
        self._fetch(f"+refs/tags/run-receipt/{recipe}/*:{local}*")
        out = _git(
            self.cwd, "for-each-ref", "--format=%(refname:lstrip=-1) %(taggerdate:unix)", local
        ).stdout
        times = []
        for line in out.splitlines():
            name, _, stamp = line.partition(" ")
            if name.startswith("hold-") == holds and stamp.isdigit():
                times.append(dt.datetime.fromtimestamp(int(stamp), dt.timezone.utc))
        return times

    # the GitHub API
    def pr(self, n):
        return self.api.get(f"repos/{self.repo}/pulls/{n}")

    def reviews(self, n):
        return self.api.get(f"repos/{self.repo}/pulls/{n}/reviews?per_page=100")

    def checks(self, sha):
        return self.api.get(f"repos/{self.repo}/commits/{sha}/check-runs?per_page=100")

    def suites(self, sha):
        return self.api.get(f"repos/{self.repo}/commits/{sha}/check-suites?per_page=100")

    def status(self, sha):
        return self.api.get(f"repos/{self.repo}/commits/{sha}/status?per_page=100")

    # the one write: an atomic push of tags, each with a lease on its old value
    def push_tags(self, updates, now):
        """updates: [(tag name, target commit, annotation or None, expected old value or '')]"""
        args, specs = [], []
        when = {"GIT_COMMITTER_DATE": now.isoformat()}  # the receipt's tagger date
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
                    env=when,
                )
                specs.append(f"refs/tags/{name}:refs/tags/{name}")
            args.append(f"--force-with-lease=refs/tags/{name}:{old}")
        _git(self.cwd, "push", "-q", "--atomic", *args, self.remote, *specs)
        for name, target, message, _ in updates:
            self.tags()[name] = target
            if message is None:
                self._raw[name] = target


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
    if type(req["version"]) is not int or req["version"] != 1:
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
        # fullmatch, not match: `$` also matches before a final newline, so "id\n" would pass
        if not isinstance(req[k], str) or not pattern.fullmatch(req[k]):
            raise Refused(f"bad {k}")
    if "note" in req and (not isinstance(req["note"], str) or len(req["note"]) > 500):
        raise Refused("bad note")
    try:
        dt.datetime.fromisoformat(str(req["created"]))
    except ValueError:
        raise Refused("bad created") from None
    return req


def check_ci(world, sha):
    """Green as CLAUDE.md defines it: every check run and the legacy commit status, with every
    workflow finished and the required ones succeeded (absent or skipped is not green)."""
    runs = world.checks(sha)
    got = runs.get("check_runs", [])
    if runs.get("total_count", 0) > len(got):
        raise Refused("more than 100 check runs: not all could be read")

    # by name, across workflows: a superseded run's twin lives in another workflow run's suite
    # (on #1104, `mirror` was cancelled in one run and skipped in another), so keying on the
    # suite would hold every request on routine duplicates
    def key(r):
        return r.get("name")

    succeeded = {
        key(r) for r in got if r.get("status") == "completed" and r.get("conclusion") == "success"
    }
    missing = REQUIRED_CHECKS - succeeded
    if missing:
        raise Refused(f"required check(s) {sorted(missing)} have not succeeded on {sha[:12]}")

    passed = {
        key(r) for r in got if r.get("status") == "completed" and r.get("conclusion") in PASSED
    }

    def waiting_macos(r):
        return "macos" in str(r.get("name", "")).lower() and r.get("status") in MACOS_WAITING

    for run in got:
        name, status, concl = run.get("name", "?"), run.get("status"), run.get("conclusion")
        if name in REQUIRED_CHECKS:
            if status == "completed" and concl == "success":
                continue
            if status == "completed" and concl == "cancelled" and name in succeeded:
                continue
            raise Refused(f"required check {name!r} is {status}/{concl}")
        if status == "completed" and concl in PASSED:
            continue
        if waiting_macos(run):
            continue  # CLAUDE.md: a macOS job waiting for a runner never holds a merge
        if status == "completed" and concl == "cancelled" and key(run) in passed:
            continue  # a superseded duplicate of a job that passed or was skipped
        raise Refused(f"check {name!r} is {status}/{concl}")
    suites = world.suites(sha)
    listed = suites.get("check_suites", [])
    if suites.get("total_count", 0) > len(listed):
        raise Refused("more than 100 check suites: not all could be read")
    for suite in listed:
        if (suite.get("app") or {}).get("slug") != "github-actions":
            continue
        suite_runs = [r for r in got if (r.get("check_suite") or {}).get("id") == suite.get("id")]
        if suite.get("status") == "completed":
            concl = suite.get("conclusion")
            if concl in PASSED:
                continue
            if (
                concl == "cancelled"
                and suite_runs
                and all(
                    (r.get("status") == "completed" and r.get("conclusion") in PASSED)
                    or (
                        r.get("status") == "completed"
                        and r.get("conclusion") == "cancelled"
                        and key(r) in passed
                    )
                    for r in suite_runs
                )
            ):
                continue
            raise Refused(f"a workflow on {sha[:12]} completed with {concl!r}")
        pending = [r for r in suite_runs if r.get("status") != "completed"]
        if pending and all(waiting_macos(r) for r in pending):
            continue
        raise Refused(f"a workflow on {sha[:12]} is still {suite.get('status')}")
    st = world.status(sha)
    contexts = {x.get("context"): x.get("state") for x in st.get("statuses", [])}
    for context in sorted(REQUIRED_STATUSES):
        if contexts.get(context) != "success":
            raise Refused(f"status {context!r} is {contexts.get(context, 'absent')}")
    if st.get("state") != "success":
        raise Refused(f"commit status is {st.get('state')}")


def _overlaps(a, b):
    """Whether two repository paths contain one another, by path segment."""
    pa, pb = pathlib.PurePosixPath(a.lstrip("/")), pathlib.PurePosixPath(b.lstrip("/"))
    return pa == pb or pa in pb.parents or pb in pa.parents


def _matching_open_flag(world, claim_id, recipe, claim):
    """Return the first open flag filename matching claim_id, recipe, or owned/premise paths."""
    paths = list(claim.get("owned_paths", []) or []) + list(claim.get("premises", []) or [])
    for name, targets in world.open_flags():
        if claim_id in targets or recipe in targets:
            return name
        if any(_overlaps(tok, p) for tok in targets for p in paths):
            return name
    return None


def names_recipe(claim, recipe):
    text = " ".join(
        str(claim.get(k, "")) for k in ("title", "question", "why", "stop_test", "done_when")
    )
    return re.search(rf"(?<![a-z0-9-]){re.escape(recipe)}(?![a-z0-9-])", text) is not None


def older_than_a_signal(world, recipe, when, holds):
    """A request published before the recipe's latest request-made signal of its own kind is
    retired by it: after that signal is cleared, an older request can't revive the run, nor an
    older hold the hold. Only like retires like, so a ready receipt never retires a hold."""
    latest = max(world.receipt_times(recipe, holds), default=None)
    if latest is not None and when < latest:
        raise Refused(f"published before the latest signal for {recipe} ({latest.isoformat()})")


def evaluate_ready(world, req, now, when):
    recipe, sha, tags = req["recipe"], req["sha"], world.tags()
    if f"run-receipt/{recipe}/{sha}" in tags:
        return "done", "already signalled from a request (receipt exists)"
    older_than_a_signal(world, recipe, when, holds=False)
    if f"hold/{recipe}" in tags:
        raise Refused(f"hold/{recipe} is set")
    if world.queue().get(recipe) != req["branch"]:
        raise Refused(f"the queue does not list {recipe} on {req['branch']}")
    claim = world.claim(req["claim"])
    if not claim:
        raise Refused(f"no claim {req['claim']} on the default branch")
    if claim.get("needs_louis") or claim.get("status") not in CLAIM_OK:
        raise Refused(f"claim {req['claim']} is {claim.get('status')} or needs Louis")
    if "workstation-queue" not in claim.get(
        "mechanisms", []
    ) and "bench/workstation-queue.txt" not in claim.get("owned_paths", []):
        raise Refused(f"claim {req['claim']} is not a workstation queue claim")
    if not names_recipe(claim, recipe):
        raise Refused(f"claim {req['claim']} does not name {recipe}")
    open_flag = _matching_open_flag(world, req["claim"], recipe, claim)
    if open_flag:
        raise Refused(f"open flag {open_flag} blocks {req['claim']}/{recipe}")
    lease = world.lease(req["claim"])
    if (
        not lease
        or lease.get("released") is not False
        or lease.get("phase") not in (None, "integration")
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
    head = pr.get("head") or {}
    base = pr.get("base") or {}
    if (
        head.get("sha") != sha
        or head.get("ref") != req["branch"]
        or (head.get("repo") or {}).get("full_name") != world.repo
    ):
        raise Refused(f"PR #{req['pr']}'s head is not {sha[:12]} on {world.repo}'s {req['branch']}")
    if base.get("ref") != world.default:
        raise Refused(f"PR #{req['pr']} targets {base.get('ref')!r}, not {world.default!r}")
    if pr.get("draft") is not False:
        raise Refused(f"PR #{req['pr']} is draft or draft status is unknown")
    if not (pr.get("state") == "open" or pr.get("merged") is True):
        raise Refused(f"PR #{req['pr']} is closed without merging")
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


def _is_hold(text):
    try:
        return tomllib.loads(text or "").get("action") == "hold"
    except (tomllib.TOMLDecodeError, TypeError, ValueError):
        return False


def reconcile(world, now, apply=False):
    decisions = []
    queue_cache = None
    # holds first: a hold published in the same pass as a ready request always takes precedence
    found = world.requests()
    found.sort(key=lambda r: (not _is_hold(r[3]), r[2], r[0]))
    for ref, commit, when, text in found:
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
                receipt_name = f"run-receipt/{recipe}/hold-{commit}"
                queue_cache = queue_cache if queue_cache is not None else world.queue()
                if receipt_name in world.tags():
                    d.outcome, d.reason = "done", "already applied (receipt exists)"
                elif recipe not in queue_cache:
                    raise Refused(f"the queue does not list {recipe}")
                else:
                    older_than_a_signal(world, recipe, when, holds=True)
                    is_set = f"hold/{recipe}" in world.tags()
                    d.outcome = "eligible"
                    d.reason = f"hold/{recipe} is already set" if is_set else ""
                    if apply:
                        receipt = json.dumps(
                            {
                                "request": ref,
                                "request_commit": commit,
                                "recipe": recipe,
                                "action": "hold",
                                "at": now.isoformat(),
                            },
                            indent=1,
                        )
                        head = world.default_head()
                        updates = [(receipt_name, head, receipt, "")]
                        if not is_set:
                            updates.insert(0, (f"hold/{recipe}", head, None, ""))
                        world.push_tags(updates, now)
                        d.outcome = "activated"
            else:
                d.outcome, d.reason = evaluate_ready(world, req, now, when)
                if d.outcome == "eligible" and apply:
                    sha = req["sha"]
                    old = world.tags().get(f"ready/{recipe}", "")
                    receipt = json.dumps(
                        {
                            "request": ref,
                            "request_commit": commit,
                            "recipe": recipe,
                            "action": "ready",
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
                        updates.insert(
                            0, (f"ready/{recipe}", sha, None, world.raw(f"ready/{recipe}"))
                        )
                    world.push_tags(updates, now)
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
