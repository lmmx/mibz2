"""Read-only historical event projection; never launches, notifies or claims work."""

import argparse
import json
import subprocess
import tomllib


def git(*args):
    return subprocess.check_output(["git", *args])


def snapshot(ref):
    items = {}
    paths = git("ls-tree", "-r", "--name-only", "-z", ref, "frontier/claims", "frontier/blog")
    for path in paths.decode().split("\0"):
        if path.endswith(".toml"):
            item = tomllib.loads(git("show", f"{ref}:{path}").decode())
            cid = item["id"]
            if cid in items:
                raise ValueError(f"duplicate claim id: {cid}")
            items[cid] = item
    return items


def overlaps(a, b):
    a, b = a.rstrip("/"), b.rstrip("/")
    return bool(a and b and (a == b or a.startswith(b + "/") or b.startswith(a + "/")))


def project(before, after, changed, source=None):
    """Project observations; use pre-event dependencies to avoid hindsight leakage."""
    if source and source not in before and source not in after:
        raise ValueError(f"unknown source claim: {source}")
    origin = before.get(source, after.get(source, {}))
    events = []
    for cid in sorted(before.keys() | after.keys()):
        old, new = before.get(cid, {}), after.get(cid, {})
        previous, current = old.get("status"), new.get("status")
        reasons = []
        if previous != current:
            reasons.append({"kind": "state", "before": previous, "after": current})
        # Completed and dropped work is historical, not a new candidate.
        if current not in (None, "done", "dropped") and cid != source:
            for key in ("premises", "owned_paths"):
                paths = sorted({p for p in changed if any(overlaps(p, q) for q in old.get(key, []))})
                if paths:
                    reasons.append({"kind": key, "paths": paths})
            shared = sorted(set(old.get("mechanisms", [])) & set(origin.get("mechanisms", [])))
            if shared:
                reasons.append({"kind": "related-mechanism", "mechanisms": shared})
        if reasons:
            events.append({"claim": cid, "status": current, "reasons": reasons})
    return events


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base")
    parser.add_argument("head")
    parser.add_argument("--source", help="optional known source claim; not an agent identity")
    parser.add_argument("--kind", choices=("committed", "proposed"), required=True)
    args = parser.parse_args()
    base, head = (git("rev-parse", "--verify", ref + "^{commit}").decode().strip() for ref in (args.base, args.head))
    # The caller labels an event. This is not proof of merge or eligibility.
    changed = set(git("diff", "--name-only", "--no-renames", "-z", base, head).decode().split("\0")) - {""}
    result = project(snapshot(base), snapshot(head), changed, args.source)
    print(json.dumps({"kind": args.kind, "base": base, "head": head, "source": args.source, "observations": result}, indent=2))


if __name__ == "__main__":
    main()
