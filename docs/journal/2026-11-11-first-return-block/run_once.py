"""Enforce the frozen one-attempt, ten-minute, 256-MiB process envelope."""

import argparse
import hashlib
import json
import resource
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parent
MEMORY = 256 * 1024 * 1024
SECONDS = 600


def limits():
    """Set hard address-space and CPU bounds before executing the candidate."""
    resource.setrlimit(resource.RLIMIT_AS, (MEMORY, MEMORY))
    resource.setrlimit(resource.RLIMIT_CPU, (SECONDS, SECONDS))


def digest(path):
    """Compute a frozen source/input identity."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_limits():
    """Test enforcement without executing the probe or reading candidate data."""
    code = """import json,resource
assert resource.getrlimit(resource.RLIMIT_AS)==(268435456,268435456)
assert resource.getrlimit(resource.RLIMIT_CPU)==(600,600)
try:
    x=bytearray(300*1024*1024)
except MemoryError:
    print(json.dumps({'address_space_limit':268435456,'cpu_seconds':600,'oversized_allocation_refused':True}))
else:
    raise AssertionError('memory limit not enforced')
"""
    p = subprocess.run(
        [sys.executable, "-c", code],
        preexec_fn=limits,
        capture_output=True,
        text=True,
        check=True,
        timeout=5,
    )
    result = json.loads(p.stdout)
    try:
        subprocess.run(
            [sys.executable, "-c", "import time;time.sleep(1)"], timeout=0.05, check=True
        )
    except subprocess.TimeoutExpired:
        result["wall_timeout_enforced"] = True
    else:
        raise AssertionError("timeout not enforced")
    print(json.dumps(result, sort_keys=True))


def main():
    """Refuse changed inputs or an existing attempt directory; never auto-retry."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify-limits", action="store_true")
    parser.add_argument("export", nargs="?", type=Path)
    parser.add_argument("destination", nargs="?", type=Path)
    args = parser.parse_args()
    if args.verify_limits:
        verify_limits()
        return
    assert args.export and args.destination
    freeze = json.loads((ROOT / "FREEZE.json").read_text())
    for name, sha in freeze["source_sha256"].items():
        assert digest(ROOT / name) == sha, name
    identities = ROOT / "evidence/identities.json"
    assert digest(identities) == freeze["identities_sha256"]
    meta = json.loads(identities.read_text())
    for name, record in meta["artifacts"].items():
        assert digest(args.export / name) == record["sha256"], name
    args.destination.mkdir(parents=True, exist_ok=False)
    (args.destination / "attempt.json").write_text(
        json.dumps(
            dict(
                freeze_sha256=digest(ROOT / "FREEZE.json"),
                block=meta["block"],
                delimiter=meta["delimiter"],
                max_seconds=SECONDS,
                address_space_bytes=MEMORY,
                attempts=1,
            ),
            indent=2,
        )
        + "\n"
    )
    command = [
        sys.executable,
        str(ROOT / "probe.py"),
        str(args.export),
        str(identities),
        str(args.destination / "result.json"),
    ]
    state = dict(command=command, retries=0)
    with (
        (args.destination / "progress.jsonl").open("w") as out,
        (args.destination / "stderr.txt").open("w") as err,
    ):
        try:
            completed = subprocess.run(
                command, stdout=out, stderr=err, preexec_fn=limits, timeout=SECONDS
            )
            state["returncode"] = completed.returncode
        except subprocess.TimeoutExpired:
            state["timeout"] = True
    if not (args.destination / "result.json").exists():
        outcome = "decline_wall_limit" if state.get("timeout") else "stop_process_failure"
        (args.destination / "result.json").write_text(
            json.dumps(
                dict(
                    outcome=outcome,
                    complete=False,
                    supervisor=state,
                    unfinished_checks="No result inferred; retain progress and stderr",
                ),
                indent=2,
            )
            + "\n"
        )
    state["child_peak_rss_kib"] = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    (args.destination / "supervisor.json").write_text(
        json.dumps(state, indent=2, sort_keys=True) + "\n"
    )
    print((args.destination / "result.json").read_text())


if __name__ == "__main__":
    main()
