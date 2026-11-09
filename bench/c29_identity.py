"""C29's run identity (bench/c29_checkpoints.py's IDENTITY_SCHEMA), as sweep.yml's `tool: c29`
writes it around the run: the head, the build, the digests, and the executed environment from
`ab env`'s line and `nproc`. The reporter's tests build their identities with `build`, so a
fixture is the workflow's own mapping, not a copy of it.

    python3 bench/c29_identity.py --env c29-env.json --nproc c29-nproc.txt --arms A,B \
        --rounds N --tree-clean true|false --bin BEFORE AFTER --corpus BEFORE AFTER \
        --model MODEL --run-status S > c29-identity.json

The cgroup CPU quota keeps its three states apart (`cpu_quota`): `{"state": "unlimited"}`
(read, `max`), `{"state": "finite", "cpus": Q}` (read, Q CPUs), or `{"state": "unknown"}`:
not read, or `ab env`'s line absent, unparsable, or without its `cpu_quota_state`. A null
quota is never taken for unlimited.
"""

import argparse
import json
import os
import subprocess

SCHEMA = "c29_checkpoints/identity/3"


def load_env(text):
    """`ab env`'s line as an object; {} if it is absent or not one"""
    try:
        env = json.loads(text) if text is not None else {}
    except ValueError:
        return {}
    return env if isinstance(env, dict) else {}


def quota(env):
    """The quota's state from `ab env`'s `cpu_quota_state` and `cpu_quota`: unknown unless
    the line says unlimited, or finite with a number"""
    state, q = env.get("cpu_quota_state"), env.get("cpu_quota")
    if state == "unlimited" and q is None:
        return {"state": "unlimited"}
    if state == "finite" and isinstance(q, (int, float)) and not isinstance(q, bool):
        return {"state": "finite", "cpus": q}
    return {"state": "unknown"}


def nproc_of(text):
    try:
        return int(text)
    except (TypeError, ValueError):
        return None


def build(
    env_text,
    nproc_text,
    *,
    head,
    tree_clean,
    rustc,
    toolchain,
    binary,
    corpus,
    arms,
    rounds,
    model,
    run_status,
):
    env = load_env(env_text)
    return {
        "schema": SCHEMA,
        "head": head,
        "tree_clean": tree_clean,
        # (by construction: the frozen build line has no --features, Cargo's default is [])
        "features": [],
        "profile": "release",
        "rustc": rustc,
        "toolchain": toolchain,
        "binary_sha256_before": binary[0],
        "binary_sha256_after": binary[1],
        "corpus_sha256_before": corpus[0],
        "corpus_sha256_after": corpus[1],
        "arms": arms,
        "threads": [9, 3],
        "rounds": rounds,
        "avx2": env.get("avx2"),
        "nproc": nproc_of(nproc_text),
        "cpus_allowed": env.get("cpus_allowed"),
        "available_parallelism": env.get("available_parallelism"),
        "cpu_quota": quota(env),
        "cpu_model": model,
        "ab_run_status": run_status,
        "env": env,
    }


def read(path):
    try:
        with open(path) as f:
            return f.read()
    except OSError:
        return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--env", required=True)
    ap.add_argument("--nproc", required=True)
    ap.add_argument("--arms", required=True)
    ap.add_argument("--rounds", type=int, required=True)
    ap.add_argument("--tree-clean", choices=["true", "false"], required=True)
    ap.add_argument("--bin", nargs=2, required=True)
    ap.add_argument("--corpus", nargs=2, required=True)
    ap.add_argument("--model", required=True)
    ap.add_argument("--run-status", type=int, required=True)
    a = ap.parse_args()

    def out(*cmd):
        return subprocess.check_output(cmd, text=True).strip()

    ident = build(
        read(a.env),
        read(a.nproc),
        head=out("git", "rev-parse", "HEAD"),
        tree_clean=a.tree_clean == "true",
        rustc=out("rustc", "--version"),
        toolchain={
            "rustc_vV": out("rustc", "-vV"),
            "cargo": out("cargo", "--version"),
            "rustflags": os.environ.get("RUSTFLAGS"),
            "cargo_encoded_rustflags": os.environ.get("CARGO_ENCODED_RUSTFLAGS"),
        },
        binary=a.bin,
        corpus=a.corpus,
        arms=a.arms.split(","),
        rounds=a.rounds,
        model=a.model,
        run_status=a.run_status,
    )
    print(json.dumps(ident, indent=1))


if __name__ == "__main__":
    main()
