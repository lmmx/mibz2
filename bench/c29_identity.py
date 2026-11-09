"""C29's run identity (bench/c29_checkpoints.py's IDENTITY_SCHEMA), as sweep.yml's `tool: c29`
writes it around the run: the head, the build, the digests, and the executed environment from
`ab env`'s lines (before and after the run) and `nproc`. The reporter's tests build their
identities with `build`, so a fixture is the workflow's own mapping, not a copy of it.

    python3 bench/c29_identity.py --env c29-env.json --env-after c29-env-after.json \
        --nproc c29-nproc.txt --arms A,B --rounds N --tree-clean true|false \
        --bin BEFORE AFTER --corpus BEFORE AFTER --model MODEL --run-status S > c29-identity.json

The cgroup CPU quota (`cpu_quota`) is derived here from `ab env`'s census (`cgroup`,
examples/cgroup/mod.rs), re-parsing each visited level's raw files, never from the producer's
summary alone (docs/journal/2026-11-09-c29-follow-up-registration.md, after the Researcher's
REVIEW 6735626248). Coverage is kept apart from the tightest limit:

- `{"state": "unlimited", "coverage": "complete"}`: every level from the leaf to the
  hierarchy's real root read, none finite;
- `{"state": "finite", "coverage": "complete", "quota": Q, "period": P, "path": L}`: the same,
  Q/P the tightest level's (exact integer comparison, no rounding);
- `{"state": "unknown", "coverage": "incomplete", "reasons": [...], "observed_tightest": ...}`:
  any level unread, malformed, hidden (a cgroup namespace, a mount-root offset, an overmount),
  a boundary that isn't the real root, the census absent, or the producer's summary disagreeing
  with this derivation. An observed limit is then an upper bound only, never a pass.

`cgroup_stable` compares the census's configuration before and after the run (membership, mount
layout, every level's limits and settings, the affinity's CPU IDs; never cpu.stat's counters).
"""

import argparse
import json
import os
import re
import subprocess
from fractions import Fraction

SCHEMA = "c29_checkpoints/identity/4"
CENSUS_SCHEMA = "ab/cgroup/1"
# the kernel's PROC_CGROUP_INIT_INO: the initial cgroup namespace
INIT_CGROUP_NS = "cgroup:[4026531835]"
# a level's files that are counters, not configuration (left out of `cgroup_stable`)
COUNTERS = {"cpu.stat"}


def load_env(text):
    """`ab env`'s line as an object; {} if it is absent or not one"""
    try:
        env = json.loads(text) if text is not None else {}
    except ValueError:
        return {}
    return env if isinstance(env, dict) else {}


def is_int(x):
    return isinstance(x, int) and not isinstance(x, bool)


def _text(files, name):
    """A recorded read: ("text", s), ("missing", None), ("error", e) or ("absent", None)"""
    g = files.get(name) if isinstance(files, dict) else None
    if not isinstance(g, dict):
        return "absent", None
    if isinstance(g.get("text"), str):
        return "text", g["text"]
    if g.get("missing") is True:
        return "missing", None
    if isinstance(g.get("error"), str):
        return "error", g["error"]
    return "absent", None


def _pos(s):
    """A positive decimal integer, exactly (ASCII digits only)"""
    return int(s) if isinstance(s, str) and re.fullmatch(r"[0-9]+", s) and int(s) > 0 else None


def classify(hierarchy, files, at_root):
    """One level's reading from its raw files: ("limit", Q, P), ("unlimited",),
    ("no_controller",), ("root",) or ("unknown", why)"""
    if hierarchy == "v2":
        kmax, vmax = _text(files, "cpu.max")
        if at_root:
            ktype, _ = _text(files, "cgroup.type")
            if kmax == "missing" and ktype == "missing":
                return ("root",)
            return ("unknown", f"claimed root: cpu.max {kmax}, cgroup.type {ktype}")
        if kmax == "text":
            f = vmax.split()
            if len(f) == 2 and f[0] == "max" and _pos(f[1]):
                return ("unlimited",)
            if len(f) == 2 and _pos(f[0]) and _pos(f[1]):
                return ("limit", _pos(f[0]), _pos(f[1]))
            return ("unknown", f"cpu.max {vmax!r} malformed")
        if kmax == "missing":
            kc, vc = _text(files, "cgroup.controllers")
            if kc == "text" and "cpu" not in vc.split():
                return ("no_controller",)
            return ("unknown", f"no cpu.max; cgroup.controllers {kc} {vc!r}")
        return ("unknown", f"cpu.max {kmax} {vmax!r}")
    if hierarchy == "v1":
        kq, vq = _text(files, "cpu.cfs_quota_us")
        kp, vp = _text(files, "cpu.cfs_period_us")
        if kq != "text" or kp != "text":
            return ("unknown", f"cfs quota {kq}, period {kp}")
        q, p = vq.strip(), _pos(vp.strip())
        if p and q == "-1":
            return ("unlimited",)
        if p and _pos(q):
            return ("limit", _pos(q), p)
        return ("unknown", f"cfs quota {vq!r} period {vp!r} malformed")
    return ("unknown", f"hierarchy {hierarchy!r}")


def _parent(path):
    return "/" + "/".join(path.strip("/").split("/")[:-1]) if path.strip("/") else None


def coverage(env):
    """The quota's state, derived from `ab env`'s census (module docstring)"""
    reasons = []
    cg = env.get("cgroup")
    if not isinstance(cg, dict) or cg.get("schema") != CENSUS_SCHEMA:
        return {"state": "unknown", "coverage": "incomplete", "reasons": ["no cgroup census"]}
    hierarchy = cg.get("hierarchy")
    levels = cg.get("levels")
    line = cg.get("membership_line")
    mount = cg.get("mount") if isinstance(cg.get("mount"), dict) else {}
    ns = (cg.get("ns") or {}).get("cgroup") if isinstance(cg.get("ns"), dict) else None
    if hierarchy not in ("v1", "v2"):
        reasons.append(f"hierarchy {hierarchy!r}")
    if not (isinstance(levels, list) and levels and all(isinstance(lv, dict) for lv in levels)):
        reasons.append("no visited levels")
        levels = []
    member_path = line.split(":", 2)[2] if isinstance(line, str) and line.count(":") >= 2 else None
    # the boundary: the real root only with the mount's root / and the initial namespace
    real_root = (
        mount.get("root") == "/"
        and isinstance(ns, dict)
        and ns.get("text") == INIT_CGROUP_NS
        and (cg.get("boundary") or {}).get("real_root") is True
    )
    if not real_root:
        reasons.append(
            f"the boundary is not shown to be the hierarchy's real root (mount root "
            f"{mount.get('root')!r}, cgroup namespace {ns!r})"
        )
    if levels and levels[0].get("path") != member_path:
        reasons.append(
            f"the leaf {levels[0].get('path')!r} is not the membership's {member_path!r}"
        )
    limits, kinds = [], {"limit": "limit", "unlimited": "unlimited"}
    for i, lv in enumerate(levels):
        path = lv.get("path")
        last = i == len(levels) - 1
        if i and path != _parent(levels[i - 1].get("path") or ""):
            reasons.append(f"level {path!r} is not the parent of {levels[i - 1].get('path')!r}")
        if last and real_root and path != "/":
            reasons.append(f"the last level {path!r} is not the root")
        c = classify(hierarchy, lv.get("files"), last and real_root)
        claimed = lv.get("kind")
        if c[0] == "unknown":
            reasons.append(f"{path}: {c[1]}")
            continue
        if kinds.get(c[0], c[0]) != claimed:
            reasons.append(f"{path}: the producer says {claimed!r}, the files {c[0]!r}")
            continue
        if c[0] == "limit":
            if (lv.get("quota"), lv.get("period")) != (c[1], c[2]):
                reasons.append(f"{path}: the producer's quota/period disagree with its files")
                continue
            limits.append({"quota": c[1], "period": c[2], "path": path})
    observed = min(limits, key=lambda x: Fraction(x["quota"], x["period"]), default=None)
    claimed = cg.get("state")
    if not reasons:
        state = "finite" if observed else "unlimited"
        legacy = env.get("cpu_quota")
        legacy_ok = (
            legacy is None
            if state == "unlimited"
            else (isinstance(legacy, (int, float)) and not isinstance(legacy, bool))
        )
        if claimed != state or env.get("cpu_quota_state") != state or not legacy_ok:
            reasons.append(
                f"the producer's state {claimed!r} / {env.get('cpu_quota_state')!r} "
                f"(cpu_quota {legacy!r}), not {state!r}"
            )
    if reasons:
        return {
            "state": "unknown",
            "coverage": "incomplete",
            "reasons": reasons,
            "observed_tightest": observed,
        }
    if observed is None:
        return {"state": "unlimited", "coverage": "complete"}
    return {"state": "finite", "coverage": "complete", **observed}


def _config(env):
    """The census's configuration, counters left out: what must not change across the run"""
    cg = env.get("cgroup") if isinstance(env.get("cgroup"), dict) else {}
    levels = [
        {
            "path": lv.get("path"),
            "dir": lv.get("dir"),
            "kind": lv.get("kind"),
            "quota": lv.get("quota"),
            "period": lv.get("period"),
            "files": {k: v for k, v in (lv.get("files") or {}).items() if k not in COUNTERS},
        }
        for lv in cg.get("levels") or []
        if isinstance(lv, dict)
    ]
    return {
        "membership": cg.get("membership"),
        "mount_lines": cg.get("mount_lines"),
        "ns": cg.get("ns"),
        "hierarchy": cg.get("hierarchy"),
        "mount": cg.get("mount"),
        "levels": levels,
        "cpuset_effective": cg.get("cpuset_effective"),
        "cpus_allowed_ids": env.get("cpus_allowed_ids"),
        "cpu_quota_state": env.get("cpu_quota_state"),
    }


def stability(before, after):
    """{"stable": bool, "differences": [...]}: the configuration before and after the run"""
    if not before or not after:
        return {"stable": False, "differences": ["no census before and after"]}
    a, b = _config(before), _config(after)
    diff = [k for k in a if a[k] != b[k]]
    if not isinstance(before.get("cgroup"), dict):
        diff.append("no census")
    return {"stable": not diff, "differences": diff}


def nproc_of(text):
    try:
        return int(text)
    except (TypeError, ValueError):
        return None


def environment(env_text, env_after_text, nproc_text):
    """The identity's environment facts, from `ab env`'s lines and `nproc` (also the
    preflight's, bench/c29_checkpoints.py's `preflight`)"""
    env, after = load_env(env_text), load_env(env_after_text)
    return {
        "avx2": env.get("avx2"),
        "nproc": nproc_of(nproc_text),
        "cpus_allowed": env.get("cpus_allowed"),
        "cpus_allowed_ids": env.get("cpus_allowed_ids"),
        "available_parallelism": env.get("available_parallelism"),
        "cpu_quota": coverage(env),
        "cpu_quota_after": coverage(after),
        "cgroup_stable": stability(env, after),
        "env": env,
        "env_after": after,
    }


def build(
    env_text,
    nproc_text,
    *,
    env_after_text=None,
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
    facts = environment(env_text, env_after_text, nproc_text)
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
        "cpu_model": model,
        "ab_run_status": run_status,
        **facts,
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
    ap.add_argument("--env-after", required=True)
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
        env_after_text=read(a.env_after),
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
