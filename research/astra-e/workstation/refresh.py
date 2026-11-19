"""Select one validated scorer before any decoder gate or timing is run."""

import json
from pathlib import Path
import os
import re
import subprocess
import sys

PATHS = ["bench/moonshot", ".just", "Justfile"]


def git(*args):
    return subprocess.check_output(["git", *args], text=True).strip()


def project(revision):
    # Remove the detached checkout's registration and files removed upstream.
    subprocess.run(
        ["git", "restore", "--source", revision, "--staged", "--worktree", "--", *PATHS], check=True
    )
    subprocess.run(["git", "diff", "--quiet", revision, "--", *PATHS], check=True)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def build_incompatibility(log):
    """Recognize source/manifest rejection, without retrying a resource or network failure."""
    output = log.read_text(errors="replace")
    for line in output.splitlines():
        if not line.startswith("{"):
            continue
        try:
            message = json.loads(line).get("message", {})
        except ValueError:
            continue
        if message.get("level") == "error" and message.get("spans"):
            return True
    return any(
        marker in output
        for marker in (
            "failed to parse manifest",
            "does not contain this feature",
            "does not have that feature",
            "but --locked was passed",
            "because --locked was passed",
            "needs to be updated but --locked",
            "manifest path `bench/moonshot/Cargo.toml` does not exist",
        )
    )


def compatibility(out, label):
    # These commands never decode a real block. A correctness or timing
    # observation cannot reach the fallback decision.
    target = Path(os.environ["CARGO_TARGET_DIR"]) / label
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    commands = [
        (
            "build",
            [
                "cargo",
                "build",
                "--release",
                "--locked",
                "--message-format=json",
                "--manifest-path",
                "bench/moonshot/Cargo.toml",
            ],
        ),
        (
            "interface",
            [
                sys.executable,
                "research/astra-e/workstation/test_protocol.py",
                "--interface",
                str(target),
                str(out / f"{label}-interface.json"),
            ],
        ),
    ]
    for stage, cmd in commands:
        log = out / f"{label}-{stage}.log"
        with log.open("w") as f:
            proc = subprocess.run(cmd, env=env, stdout=f, stderr=subprocess.STDOUT)
        if proc.returncode < 0:
            raise RuntimeError(f"{stage} process terminated by signal {-proc.returncode}; see {log}")
        if proc.returncode:
            if stage == "build" and not build_incompatibility(log):
                raise RuntimeError(f"build failed without a source/manifest diagnostic; see {log}")
            return {"stage": stage, "exit_code": proc.returncode, "command": cmd, "log": log.name}
    return None


def refresh(config, out, probe=compatibility):
    working = config["last_working_path_revision"]
    if not re.fullmatch(r"[0-9a-f]{40}", working):
        raise ValueError("last_working_path_revision must be a full commit ID")
    if git("log", "-1", "--format=%H", working, "--", "bench/moonshot") != working:
        raise ValueError("the pin must identify an actual scorer path revision")

    subprocess.run(["git", "fetch", "origin", "master"], check=True)
    checkout = subprocess.run(
        ["git", "checkout", "origin/master", "--", *PATHS], capture_output=True, text=True
    )
    upstream = git("rev-parse", "origin/master")
    fresh = git("log", "-1", "--format=%H", upstream, "--", "bench/moonshot")
    receipt = {
        "candidate_commit": git("rev-parse", "HEAD"),
        "fetched_master": upstream,
        "fetched_path_revision": fresh,
        "last_working_path_revision": working,
        "fresh_path_checkout_status": checkout.returncode,
        "fresh_path_checkout_stderr": checkout.stderr,
        "attempts": [],
        "proceed": False,
    }
    record = out / "scorer-refresh.json"
    write_json(record, receipt)
    # A checkout, permissions or network error is distinct from an observed
    # scorer interface/build failure. Preserve the receipt and stop.
    missing = [p for p in PATHS if not git("ls-tree", upstream, "--", p)]
    if checkout.returncode and not missing:
        raise RuntimeError(f"could not check out upstream paths: {checkout.stderr}")
    for label, revision in [("fresh", upstream), ("last-working", working)]:
        project(revision)
        attempt = {"build": label, "projection": revision}
        receipt["attempts"].append(attempt)
        write_json(record, receipt)
        failure = probe(out, label)
        attempt["failure"] = failure
        write_json(record, receipt)
        if failure is None:
            receipt.update(
                selected_build=label,
                selected_projection=revision,
                selected_path_revision=git("log", "-1", "--format=%H", revision, "--", "bench/moonshot"),
                selected_scorer_tree=git("rev-parse", f"{revision}:bench/moonshot"),
                proceed=True,
            )
            write_json(record, receipt)
            (out / "scorer-revision.txt").write_text(receipt["selected_path_revision"] + "\n")
            (out / "scorer-tree.txt").write_text(receipt["selected_scorer_tree"] + "\n")
            return label
        if fresh == working:
            break
    raise RuntimeError(f"no compatible projection; see {record}")


if __name__ == "__main__":
    pin = Path(__file__).with_name("scorer-pin.json")
    refresh(json.loads(pin.read_text()), Path(sys.argv[1]))
