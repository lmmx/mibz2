"""Which of mibz2's own changes moved its scaling knee (#39, #52).

On the i9-10900K mibz2 peaks at 4 workers and falls past them, the steepest of three
decoders while the fastest at every thread count. CI's runners (2 cores) measured every
change at 1 and 4 threads only, so a change that made a worker faster by keeping more of
its misses in flight could have sharpened the knee unseen. This builds variants of the
decoder and runs them side by side, back to back, across thread counts:

- released versions: v0.1.1 (before the flagged walk, #12), v0.1.2 (the flagged walk),
  v0.1.3 (32 lanes with each lane's next entry prefetched, #21, and the 2-symbol Huffman
  lookups, #24), v0.1.4 (the 16-wide run store, #7), v0.1.5 (the room walk's passes, #37);
- the walk's settings on this build: fewer lanes (`FLAGGED_LANES`: 16, 8) and another
  prefetch hint (`PREFETCH`: none, into L2, non-temporal), each one constant changed;
- research gates on this build (`gate2`-`gate6`, not in the default set): at most K
  workers in the T-vector phase (`invert`: the build and the walk) at once, the others free
  for MTF, RLE1 and the CRC.
- kept research branches, by commit (`pairwalk`, not in the default set): the pair walk
  (#6), v0.1.4 and one commit, so `--variants v0.1.4,pairwalk` measures that one change.

Each variant is examples/ab.rs built from its own worktree (target/variants/), checked to
give this build's bytes on the corpus, and run as an arm of this build's harness
(`mibz2@EXE`), every arm at every thread count each round in a rotated order. Reported:
each one's throughput, its scaling against its own 4-thread runs, and its throughput
against this build's at the same thread count, paired by round.

`just bench-variants [--variants v0.1.3,lanes16,...] [--threads 2,4,6,8,10] [--rounds 6]`
writes variants-<UTC time>.json (and ab's .jsonl).
"""

import argparse
import datetime
import json
import os
import re
import shutil
import subprocess
import sys

from imc import CORPUS, run
from scaling import AB, summarise

RELEASES = ["v0.1.1", "v0.1.2", "v0.1.3", "v0.1.4", "v0.1.5"]
# later releases a variant may name, outside the default set (the knee study's releases)
LATER = ["v0.2.0"]
# kept research branches, by commit (not in the default set): `pairwalk` is the pair walk
# (psi^2, two bytes a step: libsais's symbol-rows decode ported, #6), v0.1.4 and one commit,
# so `--variants v0.1.4,pairwalk` is that one change on the i9, and v0.1.5 beside them is
# today's walk (the room walk, #37) (2026-10-29-expert-review.md, E1)
COMMITS = {"pairwalk": "a3687d8fc9932d886490d65768a9bd1939edc66f"}
# a walk setting: the constant in src/bwt.rs and its value
SETTINGS = {
    "lanes16": ("FLAGGED_LANES: usize", "16"),
    "lanes8": ("FLAGGED_LANES: usize", "8"),
    "pf-off": ("PREFETCH: u8", "PF_OFF"),
    "pf-l2": ("PREFETCH: u8", "PF_L2"),
    "pf-nta": ("PREFETCH: u8", "PF_NTA"),
}
# a research variant (never on master): at most K of the decoder's workers in the T-vector
# phase (the build and the walk, `invert`) at once, the others free for MTF, RLE1 and the
# CRC; a process-wide counting semaphore around the one call
GATES = {f"gate{k}": k for k in (2, 3, 4, 5, 6)}
INVERT = "    s.inverse.invert(&s.ll, &s.counts, b.orig_ptr, &mut s.rle);\n"
GATED = """    {
        let (m, cv) = &WALK_GATE;
        let mut n = m.lock().unwrap_or_else(|e| e.into_inner());
        while *n >= WALK_PERMITS {
            n = cv.wait(n).unwrap_or_else(|e| e.into_inner());
        }
        *n += 1;
    }
    s.inverse.invert(&s.ll, &s.counts, b.orig_ptr, &mut s.rle);
    {
        let (m, cv) = &WALK_GATE;
        *m.lock().unwrap_or_else(|e| e.into_inner()) -= 1;
        cv.notify_one();
    }
"""
GATE_DEFS = """
/// bench/variants.py's gate (a research variant): at most WALK_PERMITS blocks in `invert`
static WALK_GATE: (std::sync::Mutex<usize>, std::sync::Condvar) =
    (std::sync::Mutex::new(0), std::sync::Condvar::new());
const WALK_PERMITS: usize = {k};
"""
DEFAULT = ",".join([*RELEASES, *SETTINGS])
KNOWN = ",".join([*RELEASES, *LATER, *SETTINGS, *GATES, *COMMITS])


def build(name, root, out):
    """examples/ab.rs of variant `name`, built in a worktree of its own, copied to `out`"""
    tree = os.path.join(root, "target", "variants", name)
    if os.path.exists(tree):
        run(["git", "worktree", "remove", "--force", tree])
        shutil.rmtree(tree, ignore_errors=True)
    ref = name if name in RELEASES or name in LATER else COMMITS.get(name, "HEAD")
    run(["git", "worktree", "add", "--detach", tree, ref], check=True)
    try:
        if name in GATES:
            path = os.path.join(tree, "src", "lib.rs")
            src = open(path).read()
            if src.count(INVERT) != 1:
                sys.exit(f"bench-variants: {name}: no single `invert` call in src/lib.rs")
            src = src.replace(INVERT, GATED) + GATE_DEFS.replace("{k}", str(GATES[name]))
            open(path, "w").write(src)
        if name in SETTINGS:
            const, value = SETTINGS[name]
            path = os.path.join(tree, "src", "bwt.rs")
            src = open(path).read()
            pat = re.compile(rf"^const {re.escape(const)} = [^;]+;$", re.M)
            if len(pat.findall(src)) != 1:
                sys.exit(f"bench-variants: {name}: no single `const {const}` in src/bwt.rs")
            open(path, "w").write(pat.sub(f"const {const} = {value};", src))
        env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(root, "target", "variants-build"))
        print(f"bench-variants: building {name} ({ref})", flush=True)
        r = subprocess.run(["cargo", "build", "--release", "--example", "ab"], cwd=tree, env=env)
        if r.returncode:
            sys.exit(f"bench-variants: {name} failed to build")
        exe = os.path.join(root, "target", "variants-build", "release", "examples", "ab")
        shutil.copy2(exe, out)
    finally:
        run(["git", "worktree", "remove", "--force", tree])
    return out


def output(exe, file):
    """one 2-thread decode by `exe`'s mibz2 arm: its output's size and hash, or None"""
    r = run([exe, "child", "mibz2", "2", file])
    try:
        rec = json.loads(r.stdout.strip().splitlines()[-1])
    except (ValueError, IndexError):
        return None
    return None if rec.get("err") else (rec.get("out"), rec.get("hash"))


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--variants", default=DEFAULT, help=f"any of {KNOWN}")
    ap.add_argument("--threads", default="2,4,6,8,10")
    ap.add_argument("--rounds", type=int, default=6)
    ap.add_argument("--ref", type=int, default=4, help="the thread count scaling is against")
    ap.add_argument("--file", default=CORPUS, help="the corpus (default the benchmark's)")
    ap.add_argument("--note", default="", help="what was closed or running, for the record")
    ap.add_argument("--keep", action="store_true", help="keep the variants' builds")
    a = ap.parse_args()
    root = run(["git", "rev-parse", "--show-toplevel"], check=True).stdout.strip()
    os.chdir(root)
    for p in [AB, a.file]:
        if not os.path.exists(p):
            sys.exit(f"bench-variants: no {p}; `just bench-variants` builds it")
    names = [v for v in a.variants.split(",") if v]
    unknown = [
        v
        for v in names
        if v not in RELEASES
        and v not in LATER
        and v not in SETTINGS
        and v not in GATES
        and v not in COMMITS
    ]
    if unknown:
        sys.exit(f"bench-variants: unknown variants {unknown}; any of {KNOWN}")
    threads = [int(t) for t in a.threads.split(",")]
    if a.ref not in threads:
        sys.exit(f"bench-variants: --threads must include --ref {a.ref}")
    bins = os.path.join(root, "target", "variants", "bin")
    os.makedirs(bins, exist_ok=True)
    want = output(os.path.abspath(AB), a.file)
    if not want:
        sys.exit("bench-variants: this build's harness could not decode the corpus")
    labels, arms, dropped = {"mibz2": "this build"}, ["mibz2"], {}
    for name in names:
        exe = build(name, root, os.path.join(bins, f"{name}-ab"))
        got = output(exe, a.file)
        if got != want:
            dropped[name] = f"its output {got} isn't this build's {want}"
            print(f"bench-variants: {name} left out: {dropped[name]}")
            continue
        arm = f"mibz2@{exe}"
        arms.append(arm)
        labels[arm] = name
    utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    raw = f"variants-{utc}.jsonl"
    argv = [AB, "run", raw, "--arms", ",".join(arms), "--threads", a.threads]
    argv += ["--rounds", str(a.rounds), "--settle", "0", a.file]
    if subprocess.run(argv).returncode:
        sys.exit("bench-variants: ab run failed")
    rows, same, out_bytes = summarise(raw, arms, threads, a.ref, labels, against="mibz2")
    print(f"(bench-variants {utc}, note: {a.note or '-'})")
    res = {
        "meta": {
            "utc": utc,
            "note": a.note,
            "commit": run(["git", "rev-parse", "HEAD"]).stdout.strip(),
            "argv": argv,
            "ref_threads": a.ref,
            "variants": {
                n: {
                    "ref": n if n in RELEASES or n in LATER else COMMITS.get(n, "HEAD"),
                    "setting": SETTINGS.get(n),
                    "walk_permits": GATES.get(n),
                }
                for n in names
            },
            "dropped": dropped,
        },
        "same_bytes": same,
        "output_bytes": out_bytes,
        "arms": rows,
        "records": raw,
    }
    with open(f"variants-{utc}.json", "w") as f:
        json.dump(res, f, indent=1)
    if not a.keep:
        shutil.rmtree(os.path.join(root, "target", "variants"), ignore_errors=True)
        shutil.rmtree(os.path.join(root, "target", "variants-build"), ignore_errors=True)
    print(f"\nbench-variants: wrote variants-{utc}.json and {raw}")


if __name__ == "__main__":
    main()
