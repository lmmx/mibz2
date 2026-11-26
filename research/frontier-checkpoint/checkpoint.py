"""Read the shared frontier at a safe point; never change the working tree.

This is a shadow diagnostic, not an authority token or a running poller.
"""
import argparse
import json
import pathlib
import subprocess
import sys


CLIENT = "frontier/tools/frontier.py"
CONTROL = (CLIENT, "frontier/config.toml", "frontier/README.md", "frontier/flags/")


def git(repo, *args):
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"git {args[0]}: {result.stderr.strip()}")
    return result.stdout.strip()


def observe(repo, since=None):
    """Fetch trusted trunk and report changes without interpreting messages."""
    git(repo, "fetch", "-q", "--no-tags", "origin", "+refs/heads/master:refs/remotes/origin/master")
    observed = git(repo, "rev-parse", "origin/master")
    previous = since or git(repo, "merge-base", "HEAD", observed)
    git(repo, "merge-base", "--is-ancestor", previous, observed)
    expected = git(repo, "rev-parse", f"{observed}:{CLIENT}")
    actual = git(repo, "hash-object", CLIENT)
    changes = git(repo, "diff", "--name-only", previous, observed, "--", *CONTROL).splitlines()
    return {
        "observed_master": observed,
        "previous_master": previous,
        "client_current": actual == expected,
        "control_changes": changes,
        "next_action": "reread affected shared state" if actual == expected else "refresh client before mutation",
        "authority_granted": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=pathlib.Path, default=pathlib.Path.cwd())
    parser.add_argument("--since", help="last observed master commit; local cursor, not an acknowledgement")
    args = parser.parse_args()
    try:
        result = observe(args.repo, args.since)
    except (OSError, RuntimeError) as exc:
        print(json.dumps({"status": "unavailable", "error": str(exc), "authority_granted": False}))
        return 2
    print(json.dumps(result, indent=2))
    return 0 if result["client_current"] else 1


if __name__ == "__main__":
    sys.exit(main())
