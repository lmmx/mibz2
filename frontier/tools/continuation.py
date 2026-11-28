#!/usr/bin/env python3
"""Decentralised dependency continuations: task-local wait/resume for one chain at a time.

A *continuation* is one gate of one claim's own dependency chain: (claim, action, exact pin),
for example `release-0-5-18--merge-preflight` waiting on `merged:1256@<40-hex head>`. Its state
lives on its own ref, `refs/heads/wait/<claim>--<action>`, written only by compare-and-swap, so:

- there is no global lease, watcher role or scheduler: each key is locked on its own, and
  distinct chains wait and advance independently;
- at most one session actively waits on a key (`waiter_*` fields, with a TTL), and at most one
  peer acquires its execution (`acquired_*`, with a TTL), so an abandoned waiter or executor
  never strands the chain: a peer resumes it from shared state once the TTL lapses;
- a poll writes nothing. A wait writes once to start, once per half-TTL to renew, and once on a
  meaningful change; it never touches master, the admission epoch, a hold or the queue lock;
- `ready` means the pinned dependency is observable (CI green at that head, PR merged at that
  head, a bundle archived for that pin, a ref at that commit). It is never a PASS: a returned
  bundle is `returned-unread` until a session reads it and records a disposition.

The consequential action itself still runs under the ordinary claim hold (`frontier.py claim`
or `take-review`) and its gates; `resume` prints that next step rather than doing it.

Usage (see frontier/CONTINUATIONS.md):
  continuation.py declare <claim> <action> <dep> [--next TEXT] [--priority N] [--critical]
                  [--independent] --by AGENT
  continuation.py watch <key> --session S --by AGENT [--timeout SEC]   # blocks; exits on change
  continuation.py resume <key> --session S --by AGENT [--hours H]
  continuation.py done <key> --session S --disposition TEXT
                  [--next-action A --next-dep DEP [--next TEXT]]
  continuation.py view [--can caps] [--probe] [--all]                  # read-only
  continuation.py show <key>
Dependency pins: pr:<N>@<sha>  merged:<N>@<sha>  evidence:<recipe>@<sha>  ref:<refname>@<sha>
                 claim:<id>=<status>[,<status>]  hold-free:<id>  manual:<text>
"""

from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import importlib.util
import json
import os
import pathlib
import re
import secrets
import shutil
import subprocess
import sys
import threading
import time
import tomllib
import urllib.error
import urllib.request

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[1]
WAIT_PREFIX = "refs/heads/wait/"
RECORD_FILE = "continuation.toml"
SHA_RE = re.compile(r"[0-9a-f]{40}")
TOKEN = r"[a-z0-9][a-z0-9._-]{0,79}"
KEY_RE = re.compile(rf"({TOKEN})--({TOKEN})")
TERMINAL = ("done", "superseded", "failed")
WAITER_TTL = 900  # an active wait is visible this long without renewal
ACQUIRE_TTL = 1800  # an acquired execution lapses back to ready after this, unless done
CLASS_RANK = {
    "ready": 0,
    "waiting": 1,
    "unknown": 1,
    "held": 2,
    "owner-blocked": 3,
    "capability-blocked": 3,
    "failed": 4,
    "superseded": 5,
    "done": 6,
}


class ContinuationError(Exception):
    """A refused continuation transition (bad pin, duplicate key, missing disposition...)."""


@dataclasses.dataclass
class Observation:
    """What a probe saw: waiting, ready, failed, superseded or unknown."""

    state: str
    authoritative_at: int | None = None
    detail: str = ""
    value: str | None = None


@dataclasses.dataclass
class Result:
    """The outcome of a watch or resume, for the caller (and the model) to act on once."""

    outcome: str
    detail: str = ""
    polls: int = 0
    record: dict | None = None

    @property
    def ok(self):
        """Whether the outcome lets the caller proceed."""
        return self.outcome in ("ready", "acquired")


class SystemClock:
    """Wall-clock time and real sleep."""

    def now(self):
        """Current epoch second."""
        return int(time.time())

    def sleep(self, s):
        """Sleep for `s` seconds."""
        time.sleep(s)


def _t(clock):
    return int(clock.now())


def iso(t):
    """Format an epoch second as UTC ISO, or 'unknown'."""
    if t is None:
        return "unknown"
    return dt.datetime.fromtimestamp(int(t), dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


# ---------------------------------------------------------------- pins


def parse_dep(dep):
    """Parse a dependency pin into (kind, args); exact 40-hex SHAs only."""
    if not isinstance(dep, str) or ":" not in dep:
        raise ContinuationError(f"dependency must be kind:args, got {dep!r}")
    kind, rest = dep.split(":", 1)

    def at(rest):
        if "@" not in rest:
            raise ContinuationError(f"{kind} pin needs @<40-hex sha>: {dep!r}")
        left, sha = rest.rsplit("@", 1)
        if not SHA_RE.fullmatch(sha):
            raise ContinuationError(f"pin must be the exact 40-character SHA: {dep!r}")
        return left, sha

    if kind in ("pr", "merged"):
        n, sha = at(rest)
        if not n.isdigit() or int(n) < 1:
            raise ContinuationError(f"bad PR number in {dep!r}")
        return kind, {"pr": int(n), "sha": sha}
    if kind == "evidence":
        recipe, sha = at(rest)
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", recipe):
            raise ContinuationError(f"bad recipe in {dep!r}")
        return kind, {"recipe": recipe, "sha": sha}
    if kind == "ref":
        ref, sha = at(rest)
        if not re.fullmatch(r"refs/[A-Za-z0-9._/-]+", ref) or ".." in ref:
            raise ContinuationError(f"bad ref in {dep!r}")
        return kind, {"ref": ref, "sha": sha}
    if kind == "claim":
        if "=" not in rest:
            raise ContinuationError(f"claim pin is claim:<id>=<status>[,<status>]: {dep!r}")
        cid, statuses = rest.split("=", 1)
        if not re.fullmatch(TOKEN, cid):
            raise ContinuationError(f"bad claim id in {dep!r}")
        return kind, {"claim": cid, "statuses": [s for s in statuses.split(",") if s]}
    if kind == "hold-free":
        if not re.fullmatch(TOKEN, rest):
            raise ContinuationError(f"bad claim id in {dep!r}")
        return kind, {"claim": rest}
    if kind == "manual":
        return kind, {"text": rest}
    raise ContinuationError(f"unknown dependency kind {kind!r}")


def make_key(claim, action):
    """The continuation key `<claim>--<action>`, validated."""
    key = f"{claim}--{action}"
    if not KEY_RE.fullmatch(key) or "--" in claim or "--" in action:
        raise ContinuationError(f"claim and action must be lowercase tokens without '--': {key!r}")
    return key


# ---------------------------------------------------------------- stores


class MemoryStore:
    """In-process compare-and-swap store (tests, and a model of the ref store)."""

    def __init__(self):
        self._d = {}
        self._lock = threading.Lock()
        self._n = 0

    def get(self, key):
        """Return (token, record) for `key`, or (None, None)."""
        with self._lock:
            if key not in self._d:
                return None, None
            tok, rec = self._d[key]
            return tok, dict(rec)

    def cas(self, key, expected, record):
        """Write `record` only if the current token is `expected`; return (ok, token)."""
        with self._lock:
            cur = self._d.get(key, (None, None))[0]
            if cur != expected:
                return False, cur
            self._n += 1
            tok = f"m{self._n}"
            self._d[key] = (tok, dict(record))
            return True, tok

    def list(self):
        """Every continuation as {key: (token, record)}."""
        with self._lock:
            return {k: (t, dict(r)) for k, (t, r) in self._d.items()}


def _toml_value(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, float):
        return str(int(v))
    if isinstance(v, (list, tuple)):
        return "[" + ", ".join(_toml_value(x) for x in v) + "]"
    return json.dumps(str(v))


def dump_record(rec):
    """Serialise a flat record as TOML."""
    return "".join(f"{k} = {_toml_value(v)}\n" for k, v in sorted(rec.items()) if v is not None)


class GitRefStore:
    """Each continuation on its own ref, refs/heads/wait/<key>, moved only with force-with-lease.

    The token is the ref's commit SHA. Nothing here reads or writes any other ref; it reads refs/heads/hold/
    never (holds come from frontier.py, read-only).
    """

    def __init__(self, repo=REPO, remote="origin"):
        self.repo = pathlib.Path(repo)
        self.remote = remote

    def _git(self, *args, inp=None, env=None):
        r = subprocess.run(
            ["git", "-C", str(self.repo), *args],
            input=inp,
            capture_output=True,
            text=True,
            env={**os.environ, **(env or {})},
            timeout=60,
        )
        return r.returncode, r.stdout.strip(), r.stderr.strip()

    def _identity(self):
        rc1, name, _ = self._git("config", "--get", "user.name")
        rc2, email, _ = self._git("config", "--get", "user.email")
        if rc1 or rc2 or not name or not email:
            raise ContinuationError(
                "git user.name/user.email must be configured to write a continuation"
            )
        return {
            "GIT_AUTHOR_NAME": name,
            "GIT_AUTHOR_EMAIL": email,
            "GIT_COMMITTER_NAME": name,
            "GIT_COMMITTER_EMAIL": email,
        }

    def _remote_refs(self, pattern):
        rc, out, err = self._git("ls-remote", self.remote, pattern)
        if rc:
            raise ContinuationError(f"cannot read {pattern} from {self.remote}: {err}")
        found = {}
        for line in out.splitlines():
            sha, ref = line.split()
            found[ref] = sha
        return found

    def _read(self, shas):
        need = [s for s in shas if self._git("cat-file", "-e", f"{s}^{{commit}}")[0]]
        if need:
            rc, _, err = self._git(
                "fetch", "-q", "--no-tags", "--no-write-fetch-head", self.remote, *need
            )
            if rc:
                raise ContinuationError(f"cannot fetch continuation state: {err}")
        out = {}
        for s in shas:
            rc, body, err = self._git("show", f"{s}:{RECORD_FILE}")
            if rc:
                raise ContinuationError(f"unreadable continuation {s[:8]}: {err}")
            out[s] = tomllib.loads(body)
        return out

    def get(self, key):
        """Return (token, record) for `key`, or (None, None)."""
        ref = WAIT_PREFIX + key
        sha = self._remote_refs(ref).get(ref)
        if not sha:
            return None, None
        return sha, self._read([sha])[sha]

    def list(self):
        """Every continuation as {key: (token, record)}."""
        refs = self._remote_refs(WAIT_PREFIX + "*")
        recs = self._read(list(refs.values())) if refs else {}
        return {ref.removeprefix(WAIT_PREFIX): (sha, recs[sha]) for ref, sha in refs.items()}

    def cas(self, key, expected, record):
        """Write `record` only if the current token is `expected`; return (ok, token)."""
        if not KEY_RE.fullmatch(key):
            raise ContinuationError(f"bad key {key!r}")
        env = self._identity()
        rc, blob, err = self._git("hash-object", "-w", "--stdin", inp=dump_record(record))
        if rc:
            raise ContinuationError(err)
        rc, tree, err = self._git("mktree", inp=f"100644 blob {blob}\t{RECORD_FILE}\n")
        if rc:
            raise ContinuationError(err)
        # The nonce keeps two writers' commits distinct even for identical records: git treats a
        # push of the value already on the remote as up to date without checking the lease.
        msg = f"wait {key}: {record.get('state')} {secrets.token_hex(8)}"
        args = ["commit-tree", tree, "-m", msg]
        if expected and not self._git("cat-file", "-e", f"{expected}^{{commit}}")[0]:
            args += ["-p", expected]
        rc, sha, err = self._git(*args, env=env)
        if rc:
            raise ContinuationError(err)
        ref = WAIT_PREFIX + key
        rc, _, _ = self._git(
            "push",
            "-q",
            "--porcelain",
            f"--force-with-lease={ref}:{expected or ''}",
            self.remote,
            f"{sha}:{ref}",
        )
        return (True, sha) if rc == 0 else (False, None)


# ---------------------------------------------------------------- transitions


def declare(
    store,
    claim,
    action,
    dep,
    *,
    next_text="",
    critical=False,
    priority=3,
    by="",
    independent=False,
    clock=None,
):
    """Create the continuation for one gate. Refuses a key that already exists (dedupe)."""
    clock = clock or SystemClock()
    parse_dep(dep)
    key = make_key(claim, action)
    rec = {
        "key": key,
        "claim": claim,
        "action": action,
        "dep": dep,
        "next": next_text,
        "critical": bool(critical),
        "priority": int(priority),
        "by": by,
        "independent": bool(independent),
        "declared_at": _t(clock),
        "state": "waiting",
    }
    ok, _ = store.cas(key, None, rec)
    if not ok:
        raise ContinuationError(f"continuation {key} already exists; watch or resume it instead")
    return key


def waiter_live(rec, t):
    """Whether a live session is actively waiting on this record."""
    return bool(rec.get("waiter_session")) and int(rec.get("waiter_until", 0)) > t


def acquired_live(rec, t):
    """Whether a peer's execution acquisition is still in force."""
    return rec.get("state") == "acquired" and int(rec.get("acquired_until", 0)) > t


def acquire_waiter(store, key, session, holder, clock, ttl=WAITER_TTL):
    """Become the one active waiter for `key`; (False, holder) if another live waiter exists."""
    for _ in range(4):
        tok, rec = store.get(key)
        if rec is None:
            raise ContinuationError(f"no continuation {key}")
        t = _t(clock)
        if waiter_live(rec, t) and rec["waiter_session"] != session:
            return False, rec.get("waiter_holder", "?")
        rec.update(waiter_session=session, waiter_holder=holder, waiter_until=t + ttl)
        ok, new = store.cas(key, tok, rec)
        if ok:
            return True, new
    return False, "contended"


def _release_waiter(store, key, session):
    for _ in range(4):
        tok, rec = store.get(key)
        if rec is None or rec.get("waiter_session") != session:
            return
        for k in ("waiter_session", "waiter_holder", "waiter_until"):
            rec.pop(k, None)
        if store.cas(key, tok, rec)[0]:
            return


def _apply(rec, o, t):
    """Fold a meaningful observation into the record without overwriting first observations."""
    rec["state"] = o.state
    rec["last_detail"] = o.detail
    if o.state == "ready":
        rec.setdefault("ready_observed", t)
        if o.authoritative_at is not None:
            rec.setdefault("ready_authoritative", int(o.authoritative_at))
    elif o.state in ("failed", "superseded"):
        rec.setdefault("ended_observed", t)


def watch(
    store,
    key,
    *,
    session,
    holder,
    probe,
    clock=None,
    timeout=3300,
    interval=45,
    max_interval=300,
    backoff=1.5,
    waiter_ttl=WAITER_TTL,
):
    """Wait in place for this one continuation's pin; return only on change, timeout or refusal.

    Polls cost nothing shared: the record is written to become the waiter, to renew that
    every half-TTL, and once when the dependency changes state. On timeout the waiter is
    released and the record keeps its pin and next action for any peer.
    """
    clock = clock or SystemClock()
    tok, rec = store.get(key)
    if rec is None:
        raise ContinuationError(f"no continuation {key}")
    if rec["state"] in TERMINAL:
        return Result(rec["state"], rec.get("last_detail", ""), 0, rec)
    if acquired_live(rec, _t(clock)):
        return Result(
            "held",
            f"acquired by {rec.get('acquired_by')} until {iso(rec.get('acquired_until'))}",
            0,
            rec,
        )
    ok, who = acquire_waiter(store, key, session, holder, clock, waiter_ttl)
    if not ok:
        return Result("duplicate", f"already watched by {who}", 0, rec)
    dep = parse_dep(rec["dep"])
    start = _t(clock)
    polls = 0
    last = ""
    pause = float(interval)
    while True:
        o = probe(dep)
        polls += 1
        if o.state in ("ready", "failed", "superseded"):
            for _ in range(4):
                tok, rec = store.get(key)
                if rec.get("waiter_session") != session:
                    return Result("lost", "another session took over this wait", polls, rec)
                _apply(rec, o, _t(clock))
                for k in ("waiter_session", "waiter_holder", "waiter_until"):
                    rec.pop(k, None)
                if store.cas(key, tok, rec)[0]:
                    return Result(o.state, o.detail, polls, rec)
            return Result("lost", "could not record the change", polls, rec)
        if o.state == "unknown":
            last = f"unknown: {o.detail}" if o.detail else "unknown"
        else:
            last = o.detail or "waiting"
        t = _t(clock)
        remaining = timeout - (t - start)
        if remaining < 1:
            break
        step = min(pause, remaining)
        tok, rec = store.get(key)
        if rec.get("waiter_session") != session:
            return Result("lost", "another session took over this wait", polls, rec)
        if int(rec.get("waiter_until", 0)) - t < max(waiter_ttl / 2, step + 30):
            rec["waiter_until"] = t + waiter_ttl
            if not store.cas(key, tok, rec)[0]:
                return Result("lost", "waiter renewal lost", polls, rec)
        clock.sleep(step)
        pause = min(float(max_interval), pause * backoff)
    _release_waiter(store, key, session)
    return Result(
        "timeout",
        f"still {last} after {timeout}s; pin and next action kept",
        polls,
        store.get(key)[1],
    )


def _next_step(rec, by):
    nxt = rec.get("next") or "(see the claim's runbook)"
    return (
        f"acquired {rec['key']}. Next: {nxt}. Take the ordinary reservation before acting: "
        f"python3 frontier/tools/frontier.py claim {rec['claim']} --by {by} (or take-review for "
        "a review handoff), then frontier.py verify before anything consequential. Ready means "
        "the pinned dependency is observable, not that it passed: read and judge it, then record "
        f"`continuation.py done {rec['key']} --disposition ...`."
    )


def resume(store, key, *, session, by, probe=None, clock=None, ttl=ACQUIRE_TTL):
    """Acquire execution of a ready continuation (one peer wins); revalidate its pin first."""
    clock = clock or SystemClock()
    for _ in range(4):
        tok, rec = store.get(key)
        if rec is None:
            return Result("missing", f"no continuation {key}")
        t = _t(clock)
        state = rec["state"]
        if state in TERMINAL:
            return Result(state, rec.get("last_detail", ""), 0, rec)
        if acquired_live(rec, t) and rec.get("acquired_session") != session:
            return Result(
                "held",
                f"acquired by {rec.get('acquired_by')} until {iso(rec['acquired_until'])}",
                0,
                rec,
            )
        if rec.get("independent") and by == rec.get("by"):
            return Result("refused", f"{key} needs an agent other than its author {by}", 0, rec)
        if probe is not None:
            o = probe(parse_dep(rec["dep"]))
            if o.state in ("failed", "superseded"):
                _apply(rec, o, t)
                store.cas(key, tok, rec)
                return Result(o.state, o.detail, 1, rec)
            if o.state != "ready":
                return Result("not-ready", f"{o.state}: {o.detail}", 1, rec)
            if state == "waiting":
                _apply(rec, o, t)
        elif state not in ("ready", "acquired"):
            return Result("not-ready", state, 0, rec)
        rec["state"] = "acquired"
        rec.setdefault("first_acquired_at", t)
        rec.update(acquired_at=t, acquired_by=by, acquired_session=session, acquired_until=t + ttl)
        ok, _ = store.cas(key, tok, rec)
        if ok:
            return Result("acquired", _next_step(rec, by), 0, rec)
    return Result("held", "contended", 0, store.get(key)[1])


def done(
    store,
    key,
    *,
    session,
    disposition,
    clock=None,
    next_action=None,
    next_dep=None,
    next_text="",
    critical=None,
    priority=None,
):
    """Record the acquirer's disposition; optionally declare the chain's next gate."""
    clock = clock or SystemClock()
    if not disposition or not disposition.strip():
        raise ContinuationError("a disposition (what was read and decided) is required")
    tok, rec = store.get(key)
    if rec is None:
        raise ContinuationError(f"no continuation {key}")
    if rec.get("state") != "acquired" or rec.get("acquired_session") != session:
        raise ContinuationError(f"{key} is not acquired by session {session}; resume it first")
    t = _t(clock)
    if next_action:
        if not next_dep:
            raise ContinuationError("--next-action needs --next-dep")
        nk = declare(
            store,
            rec["claim"],
            next_action,
            next_dep,
            next_text=next_text,
            critical=rec.get("critical") if critical is None else critical,
            priority=rec.get("priority", 3) if priority is None else priority,
            by=rec.get("acquired_by", ""),
            independent=False,
            clock=clock,
        )
        rec.update(next_key=nk, next_started_at=t)
    rec.update(state="done", disposition=disposition.strip(), disposed_at=t)
    ok, _ = store.cas(key, tok, rec)
    if not ok:
        raise ContinuationError(f"{key} changed underneath; re-read and retry")
    return rec


def timings(rec):
    """Ready (authoritative or first observed), acquisition, disposition, next start; never invented."""
    auth = rec.get("ready_authoritative")
    seen = rec.get("ready_observed")
    ready_at = auth if auth is not None else seen
    acq = rec.get("first_acquired_at", rec.get("acquired_at"))
    disp = rec.get("disposed_at")
    nxt = rec.get("next_started_at")

    def d(a, b):
        return None if a is None or b is None else int(b) - int(a)

    return {
        "declared_at": rec.get("declared_at"),
        "ready_at": ready_at,
        "ready_source": "authoritative"
        if auth is not None
        else ("observed" if seen is not None else "unknown"),
        "ready_observed": seen,
        "acquired_at": acq,
        "ready_to_acquire": d(ready_at, acq),
        "observed_to_acquire": d(seen, acq),
        "disposed_at": disp,
        "acquire_to_disposition": d(acq, disp),
        "next_started_at": nxt,
        "disposition_to_next_start": d(disp, nxt) if nxt is not None else None,
    }


def view(store, *, clock=None, claims=None, holds=None, can=None, probe=None, include_done=False):
    """A read-only ordered observation of continuations. Writes nothing; assigns nothing."""
    clock = clock or SystemClock()
    t = _t(clock)
    claims = claims or {}
    holds = holds or {}
    rows = []
    for key, (_, rec) in store.list().items():
        state = rec.get("state", "unknown")
        cls, detail = state, rec.get("last_detail", "")
        c = claims.get(rec.get("claim"), {})
        h = holds.get(rec.get("claim"))
        if state in TERMINAL:
            cls = state
        elif c.get("needs_louis"):
            cls, detail = "owner-blocked", c["needs_louis"]
        elif can is not None and not set(c.get("needs", [])) <= set(can):
            cls, detail = (
                "capability-blocked",
                "needs " + ",".join(sorted(set(c["needs"]) - set(can))),
            )
        elif acquired_live(rec, t):
            cls, detail = "held", f"acquired by {rec.get('acquired_by')}"
        elif h and h.get("live") and h.get("holder") != rec.get("acquired_by"):
            cls, detail = "held", f"claim held by {h.get('holder')}"
        elif state == "acquired":
            cls, detail = "ready", f"acquisition by {rec.get('acquired_by')} lapsed"
        elif state == "waiting":
            if probe is not None:
                try:
                    o = probe(parse_dep(rec["dep"]))
                except Exception as exc:  # a probe failure is unknown, never absence
                    o = Observation("unknown", detail=str(exc))
                cls = {
                    "ready": "ready",
                    "unknown": "unknown",
                    "failed": "failed",
                    "superseded": "superseded",
                }.get(o.state, "waiting")
                detail = o.detail
            if waiter_live(rec, t):
                detail = (
                    detail + "; " if detail else ""
                ) + f"watched by {rec.get('waiter_holder')}"
        if cls == "done" and not include_done:
            continue
        prio = min(int(rec.get("priority", 3)), int(c.get("priority", 9)))
        tm = timings(rec)
        rows.append(
            {
                "key": key,
                "class": cls,
                "claim": rec.get("claim"),
                "action": rec.get("action"),
                "dep": rec.get("dep"),
                "critical": bool(rec.get("critical")),
                "priority": prio,
                "since": tm["ready_at"]
                if tm["ready_at"] is not None
                else rec.get("declared_at", 0),
                "detail": detail,
                "next": rec.get("next", ""),
            }
        )
    rows.sort(
        key=lambda r: (
            CLASS_RANK.get(r["class"], 9),
            not r["critical"],
            r["priority"],
            r["since"],
            r["key"],
        )
    )
    return rows


# ---------------------------------------------------------------- live probes (read-only)


def pr_probe_from(read_pr, read_checks):
    """Probe for pr: and merged: pins, from injectable readers (tests) or GitHub (live)."""

    def probe(dep):
        kind, a = dep
        try:
            pr = read_pr(a["pr"])
        except Exception as exc:
            return Observation("unknown", detail=f"PR #{a['pr']} unreadable: {exc}")
        head = (pr.get("head") or {}).get("sha")
        if pr.get("merged"):
            if head != a["sha"]:
                return Observation(
                    "superseded", detail=f"#{a['pr']} merged at {str(head)[:8]}, not the pin"
                )
            return Observation(
                "ready", _parse_time(pr.get("merged_at")), f"#{a['pr']} merged at pin", head
            )
        if pr.get("state") != "open":
            return Observation("superseded", detail=f"#{a['pr']} closed unmerged")
        if head != a["sha"]:
            return Observation(
                "superseded", detail=f"#{a['pr']} head moved to {str(head)[:8]}", value=head
            )
        if kind == "merged":
            return Observation("waiting", detail=f"#{a['pr']} open at pin")
        try:
            state, at = read_checks(a["sha"])
        except Exception as exc:
            return Observation("unknown", detail=f"checks unreadable: {exc}")
        if state == "success":
            return Observation("ready", at, f"#{a['pr']} checks and statuses green at pin", head)
        if state == "failure":
            return Observation(
                "failed", at, f"#{a['pr']} has a failed check or status at pin", head
            )
        if state == "pending":
            return Observation("waiting", detail=f"#{a['pr']} checks pending")
        return Observation("unknown", detail=f"checks {state}")

    return probe


def evidence_probe_from(list_bundles):
    """Probe for evidence:<recipe>@<sha>: ready (returned-unread) only for a bundle of that pin."""

    def probe(dep):
        _, a = dep
        try:
            bundles = list_bundles(a["recipe"])
        except Exception as exc:
            return Observation("unknown", detail=f"evidence unreadable: {exc}")
        hits = [b for b in bundles if b.get("pin") == a["sha"]]
        if not hits:
            return Observation("waiting", detail=f"no {a['recipe']} bundle for {a['sha'][:8]} yet")
        first = min(hits, key=lambda b: b.get("at") or 0)
        return Observation(
            "ready",
            first.get("at"),
            f"returned-unread: {first.get('rel')} (evidence {str(first.get('commit'))[:8]}); "
            "read and judge it, a bundle is not a PASS",
            first.get("commit"),
        )

    return probe


def _parse_time(s):
    if not s:
        return None
    try:
        return int(dt.datetime.fromisoformat(s.replace("Z", "+00:00")).timestamp())
    except ValueError:
        return None


def _frontier():
    spec = importlib.util.spec_from_file_location("frontier_ro", HERE / "frontier.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _gh_json(endpoint):
    """GET a GitHub API endpoint: `gh api` if present, else urllib with any GH_TOKEN."""
    if shutil.which("gh"):
        try:
            p = subprocess.run(
                ["gh", "api", "--hostname", "github.com", endpoint],
                cwd=REPO,
                capture_output=True,
                text=True,
                timeout=15,
            )
            if p.returncode == 0:
                return json.loads(p.stdout)
        except (OSError, subprocess.TimeoutExpired, ValueError):
            pass
    headers = {"Accept": "application/vnd.github+json", "User-Agent": "frontier-continuation"}
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(f"https://api.github.com/{endpoint}", headers=headers)
    with urllib.request.urlopen(req, timeout=10) as r:
        return json.load(r)


def _repo_name():
    rc, out, _ = GitRefStore()._git("remote", "get-url", "origin")
    m = re.search(r"(?:github\.com[:/]|/git/)([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+?)(?:\.git)?/?$", out)
    if rc or not m:
        raise ContinuationError("origin is not a recognised GitHub remote")
    return m.group(1)


def _live_read_pr(n):
    return _gh_json(f"repos/{_repo_name()}/pulls/{n}")


def _live_read_checks(sha):
    repo = _repo_name()
    runs = _gh_json(f"repos/{repo}/commits/{sha}/check-runs?per_page=100").get("check_runs", [])
    status = _gh_json(f"repos/{repo}/commits/{sha}/status")
    times = [_parse_time(r.get("completed_at")) for r in runs]
    times += [_parse_time(s.get("updated_at")) for s in status.get("statuses", [])]
    at = max([x for x in times if x is not None], default=None)
    if any(r.get("status") != "completed" for r in runs):
        return "pending", None
    bad = {"failure", "timed_out", "action_required", "startup_failure", "cancelled"}
    if any(r.get("conclusion") in bad for r in runs) or status.get("state") in ("failure", "error"):
        return "failure", at
    if status.get("statuses") and status.get("state") == "pending":
        return "pending", None
    if not runs and not status.get("statuses"):
        return "pending", None
    return "success", at


def _live_bundles(recipe):
    fr = _frontier()
    qs = fr.read_workstation_queue_state(fetch=True)
    if qs.get("state") == "unknown":
        raise ContinuationError(qs.get("reason", "queue/evidence state unknown"))
    out = []
    for d in dict.fromkeys([recipe, recipe.removeprefix("bench-")]):
        for rel in (qs.get("bundles_by_dir") or {}).get(d, []):
            info = fr._bundle_inspect(qs["ev_sha"], rel, qs.get("tree_files"))
            if info["sidecar"] == "mismatch":
                raise ContinuationError(f"SHA-256 sidecar mismatch for {rel}")
            commit = fr._evidence_bundle_commit(qs["ev_sha"], rel)
            rc, ct = fr.git("log", "-1", "--format=%ct", commit)
            for pin in info["commits"]:
                out.append(
                    {
                        "pin": pin,
                        "rel": rel,
                        "commit": commit,
                        "at": int(ct) if rc == 0 and ct.isdigit() else None,
                    }
                )
    return out


def _live_ref(a):
    store = GitRefStore()
    refs = store._remote_refs(a["ref"])
    refs.update(store._remote_refs(a["ref"] + "^{}"))
    peeled = refs.get(a["ref"] + "^{}") or refs.get(a["ref"])
    if not peeled:
        return Observation("waiting", detail=f"{a['ref']} not set")
    if peeled != a["sha"]:
        return Observation(
            "superseded", detail=f"{a['ref']} is at {peeled[:8]}, not the pin", value=peeled
        )
    return Observation("ready", None, f"{a['ref']} at pin (set time not authoritative)", peeled)


def live_probe(dep):
    """Read-only probe of real shared state for any pin kind; failures are `unknown`."""
    kind, a = dep
    try:
        if kind in ("pr", "merged"):
            return pr_probe_from(_live_read_pr, _live_read_checks)(dep)
        if kind == "evidence":
            return evidence_probe_from(_live_bundles)(dep)
        if kind == "ref":
            return _live_ref(a)
        if kind == "claim":
            fr = _frontier()
            c = fr.item(a["claim"], required=False)
            st = c.get("status")
            if st in a["statuses"]:
                return Observation("ready", None, f"{a['claim']} is {st} on master")
            return Observation("waiting", detail=f"{a['claim']} is {st}")
        if kind == "hold-free":
            fr = _frontier()
            h = fr.holds(fetch=True, strict=False).get(a["claim"])
            if h and fr.live(h[1]):
                return Observation("waiting", detail=f"held by {h[1].get('holder')}")
            return Observation("ready", None, f"no live hold on {a['claim']}")
        return Observation("waiting", detail=f"manual: {a.get('text', '')}; resume by hand")
    except SystemExit as exc:
        return Observation("unknown", detail=str(exc))
    except Exception as exc:
        return Observation("unknown", detail=f"{type(exc).__name__}: {exc}")


def _live_context():
    """Claims (master) and holds for view(); unavailable is reported, never treated as absence."""
    notes = []
    claims, holds = {}, {}
    try:
        fr = _frontier()
        fr._fetch_master(strict=False)
        for p in sorted((fr.ROOT / "claims").glob("*.toml")):
            rc, body = fr.git("show", f"origin/master:frontier/claims/{p.name}")
            c = tomllib.loads(body) if rc == 0 else tomllib.loads(p.read_text())
            claims[c["id"]] = c
        for cid, (_, lease) in fr.holds(fetch=True, strict=False).items():
            holds[cid] = {"holder": lease.get("holder"), "live": fr.live(lease)}
    except SystemExit as exc:
        notes.append(f"unknown: {exc}")
    except Exception as exc:
        notes.append(f"unknown: {type(exc).__name__}: {exc}")
    return claims, holds, notes


# ---------------------------------------------------------------- CLI


EXIT = {
    "ready": 0,
    "acquired": 0,
    "timeout": 10,
    "duplicate": 11,
    "superseded": 12,
    "failed": 12,
    "held": 13,
    "lost": 14,
    "not-ready": 15,
    "refused": 16,
    "missing": 17,
    "done": 0,
}


def _print_result(r):
    print(f"{r.outcome}: {r.detail}" + (f" (polls {r.polls})" if r.polls else ""))
    return EXIT.get(r.outcome, 1)


def format_view(rows, notes=()):
    """Render view() rows as Markdown list lines."""
    lines = [f"- {n}" for n in notes]
    if not rows:
        lines.append("- none")
    for r in rows:
        crit = " critical" if r["critical"] else ""
        lines.append(
            f"- [{r['class']}] P{r['priority']}{crit} `{r['key']}` on `{r['dep']}` since "
            f"{iso(r['since'])}"
            + (f": {r['detail']}" if r["detail"] else "")
            + (f"; next: {r['next']}" if r["next"] else "")
        )
    return "\n".join(lines)


def main(argv=None):
    """Command-line entry point."""
    ap = argparse.ArgumentParser(prog="continuation.py", description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    d = sub.add_parser("declare")
    d.add_argument("claim")
    d.add_argument("action")
    d.add_argument("dep")
    d.add_argument("--next", default="")
    d.add_argument("--priority", type=int, default=3)
    d.add_argument("--critical", action="store_true")
    d.add_argument("--independent", action="store_true")
    d.add_argument("--by", required=True)
    w = sub.add_parser("watch")
    w.add_argument("key")
    w.add_argument("--session", default=None)
    w.add_argument("--by", required=True)
    w.add_argument("--timeout", type=int, default=3300)
    w.add_argument("--interval", type=int, default=45)
    w.add_argument("--max-interval", type=int, default=300)
    r = sub.add_parser("resume")
    r.add_argument("key")
    r.add_argument("--session", required=True)
    r.add_argument("--by", required=True)
    r.add_argument("--hours", type=float, default=ACQUIRE_TTL / 3600)
    f = sub.add_parser("done")
    f.add_argument("key")
    f.add_argument("--session", required=True)
    f.add_argument("--disposition", required=True)
    f.add_argument("--next-action")
    f.add_argument("--next-dep")
    f.add_argument("--next", default="")
    v = sub.add_parser("view")
    v.add_argument("--can", default=None)
    v.add_argument("--probe", action="store_true", help="probe waiting pins now (read-only)")
    v.add_argument("--all", action="store_true")
    s = sub.add_parser("show")
    s.add_argument("key")
    a = ap.parse_args(argv)
    store = GitRefStore()
    try:
        if a.cmd == "declare":
            print(
                declare(
                    store,
                    a.claim,
                    a.action,
                    a.dep,
                    next_text=a.next,
                    critical=a.critical,
                    priority=a.priority,
                    by=a.by,
                    independent=a.independent,
                )
            )
            return 0
        if a.cmd == "watch":
            session = a.session or secrets.token_hex(6)
            print(f"watching {a.key} as session {session}", flush=True)
            return _print_result(
                watch(
                    store,
                    a.key,
                    session=session,
                    holder=a.by,
                    probe=live_probe,
                    timeout=a.timeout,
                    interval=a.interval,
                    max_interval=a.max_interval,
                )
            )
        if a.cmd == "resume":
            return _print_result(
                resume(
                    store,
                    a.key,
                    session=a.session,
                    by=a.by,
                    probe=live_probe,
                    ttl=int(a.hours * 3600),
                )
            )
        if a.cmd == "done":
            rec = done(
                store,
                a.key,
                session=a.session,
                disposition=a.disposition,
                next_action=a.next_action,
                next_dep=a.next_dep,
                next_text=a.next,
            )
            print(
                f"done {a.key}" + (f"; next gate {rec['next_key']}" if rec.get("next_key") else "")
            )
            return 0
        if a.cmd == "view":
            claims, holds, notes = _live_context()
            can = set(a.can.split(",")) if a.can else None
            rows = view(
                store,
                claims=claims,
                holds=holds,
                can=can,
                probe=live_probe if a.probe else None,
                include_done=a.all,
            )
            print(format_view(rows, notes))
            return 0
        if a.cmd == "show":
            tok, rec = store.get(a.key)
            if rec is None:
                print(f"no continuation {a.key}")
                return 17
            print(dump_record(rec), end="")
            print(f"# token {tok}")
            for k, val in timings(rec).items():
                print(f"# {k}: {val}")
            return 0
    except ContinuationError as exc:
        print(f"refused: {exc}", file=sys.stderr)
        return 2
    return 1


if __name__ == "__main__":
    sys.exit(main())
