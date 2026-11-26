"""Read the shared frontier at a safe point; never change the working tree.

This is a shadow diagnostic, not an authority token or a running poller. One
observation fetches trusted master and projects only material state: whether
the client in this checkout is current, which control paths changed, and, for
an optional claim, which of its premises, owned paths and open flags changed.
"""
import argparse
import json
import pathlib
import subprocess
import sys
import time
import tomllib


CLIENT = "frontier/tools/frontier.py"
CONTROL = (CLIENT, "frontier/config.toml", "frontier/README.md", "frontier/flags/")
FLAGS = "frontier/flags"
LIMITS = (
    "a client that never calls observe() is not detected or fenced; rollout or server-side fencing is still needed",
    "a premise or flag match is a reason to reread, not proof of contradiction",
    "exit 0 means only that the client blob matches master; it grants no authority",
)


class Unavailable(RuntimeError):
    """Trusted master could not be read now; a cached ref is never used instead."""


def git(repo, *args, strip=True):
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"git {args[0]}: {result.stderr.strip()}")
    return result.stdout.strip() if strip else result.stdout


def overlaps(a, b):
    """Whether two repository paths contain one another, by path segment (as frontier.py)."""
    pa, pb = pathlib.PurePosixPath(a.lstrip("/")), pathlib.PurePosixPath(b.lstrip("/"))
    return pa == pb or pa in pb.parents or pb in pa.parents


def flag_head(body):
    head = {}
    for line in body.splitlines()[:6]:
        key, sep, value = line.partition(":")
        if sep and key.strip() in ("flags", "invariant", "by", "status", "resolved_by"):
            head[key.strip()] = value.strip()
    return head


def open_flags(repo, commit):
    """Open flags at an immutable commit: {file name: header}."""
    names = git(repo, "ls-tree", "-r", "--name-only", commit, "--", FLAGS).splitlines()
    found = {}
    for rel in names:
        name = pathlib.PurePosixPath(rel).name
        if name.endswith(".md") and name != "README.md":
            head = flag_head(git(repo, "show", f"{commit}:{rel}"))
            if head.get("status") == "open":
                found[name] = head
    return found


def claim_at(repo, commit, cid):
    try:
        return tomllib.loads(git(repo, "show", f"{commit}:frontier/claims/{cid}.toml"))
    except RuntimeError:
        return None


def scope(repo, previous, observed, changed, cid):
    """Project the change onto one claim's declared relations; no semantic inference."""
    card = claim_at(repo, observed, cid)
    if card is None:
        return {"claim": cid, "missing_at_master": True}
    before = claim_at(repo, previous, cid) or {}
    paths = list(card.get("premises", []) or []) + list(card.get("owned_paths", []) or [])
    relevant = lambda h: cid in h.get("flags", "").split() or any(
        overlaps(t, p) for t in h.get("flags", "").split() for p in paths
    )
    now = {n: h for n, h in open_flags(repo, observed).items() if relevant(h)}
    was = {n for n, h in open_flags(repo, previous).items() if relevant(h)}
    return {
        "claim": cid,
        "status": card.get("status"),
        "card_changed": f"frontier/claims/{cid}.toml" in changed,
        "stop_test_changed": before.get("stop_test") != card.get("stop_test") and bool(before),
        "premise_changes": sorted(f for f in changed if any(overlaps(f, p) for p in paths)),
        # Every restriction that is open now, not only new ones: a cursor never hides it.
        "open_flags": sorted(now),
        "new_flags": sorted(set(now) - was),
    }


def observe(repo, since=None, claim=None, cursor=None):
    """Fetch trusted trunk and report material changes since a local cursor."""
    started = time.perf_counter()
    try:
        git(repo, "fetch", "-q", "--no-tags", "origin", "+refs/heads/master:refs/remotes/origin/master")
    except RuntimeError as exc:
        raise Unavailable(str(exc)) from None
    observed = git(repo, "rev-parse", "origin/master")
    cursor = pathlib.Path(cursor) if cursor else None
    if since is None and cursor and cursor.exists():
        since = cursor.read_text().strip() or None
    previous = since or git(repo, "merge-base", "HEAD", observed)
    git(repo, "merge-base", "--is-ancestor", previous, observed)  # a rewritten trunk fails closed
    expected = git(repo, "rev-parse", f"{observed}:{CLIENT}")
    actual = git(repo, "hash-object", CLIENT)
    unchanged = previous == observed
    changed = [] if unchanged else git(repo, "diff", "--name-only", previous, observed).splitlines()
    status = iter(git(repo, "status", "--porcelain", "-z", "-uall", strip=False).split("\0"))
    dirty = []
    for entry in status:
        if entry:
            dirty.append(entry[3:])
            if entry[0] in "RC":
                dirty.append(next(status))  # -z puts a rename's source as a bare entry
    result = {
        "observed_master": observed,
        "previous_master": previous,
        "unchanged": unchanged,
        "commits_coalesced": 0 if unchanged else int(git(repo, "rev-list", "--count", f"{previous}..{observed}")),
        "client_current": actual == expected,
        "control_changes": sorted(f for f in changed if any(overlaps(f, c) for c in CONTROL)),
        "dirty_paths": len(dirty),
        "dirty_also_changed_on_master": sorted(set(dirty) & set(changed)),
        "next_action": [],
        "authority_granted": False,
        "limits": list(LIMITS),
    }
    if claim:
        result["scope"] = scope(repo, previous, observed, set(changed), claim)
    actions = result["next_action"]
    if not result["client_current"]:
        actions.append("refresh client before any mutation (separate clean worktree if work is dirty)")
    if result["dirty_also_changed_on_master"]:
        actions.append("integrate by hand or in a separate worktree; never reset dirty work")
    s = result.get("scope", {})
    if s.get("premise_changes") or s.get("card_changed") or s.get("new_flags") or s.get("missing_at_master"):
        actions.append("reread the claim's changed premises, card and flags before continuing")
    if s.get("open_flags"):
        actions.append("claim is flagged: verify will refuse until a different agent resolves it")
    if not actions:
        actions.append("continue current work")
    if cursor:
        cursor.write_text(observed + "\n")  # local, disposable; written only after success
    result["observation_seconds"] = round(time.perf_counter() - started, 4)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=pathlib.Path, default=pathlib.Path.cwd())
    parser.add_argument("--since", help="last observed master commit; local cursor, not an acknowledgement")
    parser.add_argument("--claim", help="project changes onto this claim's premises, owned paths and flags")
    parser.add_argument("--cursor", help="local file holding the last observed master (e.g. inside .git/)")
    args = parser.parse_args()
    try:
        result = observe(args.repo, args.since, args.claim, args.cursor)
    except (OSError, RuntimeError) as exc:
        print(json.dumps({"status": "unavailable", "error": str(exc), "authority_granted": False}))
        return 2
    print(json.dumps(result, indent=2))
    return 0 if result["client_current"] else 1


if __name__ == "__main__":
    sys.exit(main())
