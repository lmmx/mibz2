"""Refresh the detached measurement worktree and enforce the registered scorer identity."""
import json
from pathlib import Path
import re
import subprocess
import sys

PATHS = ["bench/moonshot", ".just", "Justfile"]

def git(*args):
    return subprocess.check_output(["git", *args], text=True).strip()

def project(revision):
    # A plain checkout overlays paths. Remove files unique to the other tree
    # too, including a Coordinator's research-only .just registration.
    subprocess.run(["git", "restore", "--source", revision, "--staged", "--worktree", "--", *PATHS],
                   check=True)
    subprocess.run(["git", "diff", "--quiet", revision, "--", *PATHS], check=True)

def refresh(config, out):
    working = config["last_working_path_revision"]
    if not re.fullmatch(r"[0-9a-f]{40}", working):
        raise ValueError("last_working_path_revision must be a full commit ID")
    if git("log", "-1", "--format=%H", working, "--", "bench/moonshot") != working:
        raise ValueError("the registered pin must identify an actual scorer path revision")

    subprocess.run(["git", "fetch", "origin", "master"], check=True)
    fresh_checkout = subprocess.run(["git", "checkout", "origin/master", "--", *PATHS],
                                    capture_output=True, text=True)
    # A renamed or removed scorer path may break the required checkout itself.
    # Preserve the last working tree and report the failure in the same receipt.
    upstream = git("rev-parse", "origin/master")
    if fresh_checkout.returncode == 0:
        project(upstream)
    fresh = git("log", "-1", "--format=%H", upstream, "--", "bench/moonshot")
    selected = upstream
    reason = "registered upstream scorer is current"
    exception = config.get("incompatible_successor")
    evidence = None
    permitted = True
    if fresh != working or fresh_checkout.returncode:
        selected = working
        project(working)
        permitted = False
        reason = "upstream scorer changed; re-score baseline and preserved candidate before measuring"
        if exception and exception["path_revision"] == fresh:
            evidence = exception["evidence"]
            # An exception applies to exactly one observed broken update. Its
            # diagnosis and the working recipe must already belong to this freeze.
            evidence_blob = git("rev-parse", f"HEAD:{evidence}")
            if git("cat-file", "-t", evidence_blob) != "blob" or int(git("cat-file", "-s", evidence_blob)) == 0:
                raise ValueError("registered interface failure needs a tracked, nonempty evidence file")
            permitted = True
            reason = "using registered last-working scorer for this documented incompatible update"

    receipt = {
        "candidate_commit": git("rev-parse", "HEAD"),
        "fetched_master": upstream,
        "fetched_path_revision": fresh,
        "fresh_path_checkout_status": fresh_checkout.returncode,
        "fresh_path_checkout_stderr": fresh_checkout.stderr,
        "last_working_path_revision": working,
        "selected_projection": selected,
        "selected_path_revision": git("log", "-1", "--format=%H", selected, "--", "bench/moonshot"),
        "selected_scorer_tree": git("rev-parse", f"{selected}:bench/moonshot"),
        "reason": reason,
        "incompatibility_evidence": evidence,
        "proceed": permitted,
    }
    (out / "scorer-refresh.json").write_text(json.dumps(receipt, indent=2) + "\n")
    (out / "scorer-revision.txt").write_text(receipt["selected_path_revision"] + "\n")
    (out / "scorer-tree.txt").write_text(receipt["selected_scorer_tree"] + "\n")
    print(reason, flush=True)
    return permitted

if __name__ == "__main__":
    pin = Path(__file__).with_name("scorer-pin.json")
    if not refresh(json.loads(pin.read_text()), Path(sys.argv[1])):
        sys.exit(2)
