#!/usr/bin/env python3
"""The outer guard's targeted mutants (Guardian D067), each applied alone to src/parallel.rs,
the outer-unwind suite run natively, and the file restored byte for byte.

Usage, from the repository's root, on a clean checkout of the pinned commit:
    python3 docs/journal/2026-11-09-service-guarded-scope/mutants.py OUT_DIR

Each mutant's log goes to OUT_DIR/<name>.log, headed by the command and the SHA-256 of
the mutated and restored file; its patch to OUT_DIR/<name>.patch. Then the restored
baseline runs once more (OUT_DIR/restored.log). A mutant is KILLED when the suite fails
and the failing test is one the mutant's obligation names; the script prints what failed,
and the logs carry the assertion or watchdog message. It never retries.
"""

import hashlib
import pathlib
import subprocess
import sys

SRC = pathlib.Path("src/parallel.rs")
CMD = ["cargo", "test", "--lib", "--", "service_unwind_tests"]

MUTANTS = {
    # M12's analogue: the guard disarmed on every path (its failure never runs)
    "G1-guard-removed": [
        (
            "        if std::thread::panicking() {\n"
            "            self.hub.fail(self.shared, self.held.take(), self.what);\n",
            "        if false && std::thread::panicking() {\n"
            "            self.hub.fail(self.shared, self.held.take(), self.what);\n",
        )
    ],
    # the hub's own wake-up missing from the failure (the shared one kept)
    "G2-no-hub-wake": [
        (
            "        shared.cv.notify_all();\n        self.cv.notify_all();\n    }\n}\n",
            "        shared.cv.notify_all();\n    }\n}\n",
        )
    ],
    # the terminal predicate missing: the hub's waiters wait for the old end alone
    "G3-no-terminal-predicate": [
        (
            "        self.failed || (self.closed && self.jobs.is_empty() && self.live == 0)\n",
            "        self.closed && self.jobs.is_empty() && self.live == 0\n",
        ),
        (
            "            let task = loop {\n                if st.failed {\n"
            "                    break None;\n                }\n",
            "            let task = loop {\n",
        ),
        (
            "    let got = loop {\n        if st.failed {\n            break None;\n        }\n",
            "    let got = loop {\n",
        ),
    ],
    # a double retirement: the guard retires a block whether or not it holds one
    "G4-double-retirement": [
        (
            "            let retired = queued + usize::from(held.is_some());\n",
            "            let retired = queued + 1;\n",
        )
    ],
    # the hold released at publication, not at retirement (the publish-to-retire gap)
    "G5-early-disarm": [
        (
            "    publish(shared, place, bit, res, symbols.syms);\n"
            "    #[cfg(test)]\n    outer_fault(hub, OuterAt::AfterPublish, Some(place));\n",
            "    o.held.set(None);\n"
            "    publish(shared, place, bit, res, symbols.syms);\n"
            "    #[cfg(test)]\n    outer_fault(hub, OuterAt::AfterPublish, Some(place));\n",
        )
    ],
}


def sha(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def run(out: pathlib.Path, name: str, src: bytes) -> str:
    env = {"RUST_BACKTRACE": "0"}
    import os

    env = {**os.environ, **env}
    p = subprocess.run(CMD, capture_output=True, text=True, env=env, timeout=1800)
    log = out / f"{name}.log"
    log.write_text(
        f"# {' '.join(CMD)}\n# src/parallel.rs sha256 {sha(src)}\n# exit {p.returncode}\n"
        + p.stdout
        + p.stderr
    )
    failed = [l for l in p.stdout.splitlines() if l.startswith("test ") and "FAILED" in l]
    summary = [l for l in p.stdout.splitlines() if l.startswith("test result")]
    return f"{name}: exit {p.returncode}; {summary[-1] if summary else 'no result'}\n" + "\n".join(
        f"    {l}" for l in failed
    )


def main() -> None:
    out = pathlib.Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    base = SRC.read_bytes()
    for name, edits in MUTANTS.items():
        s = base.decode()
        for old, new in edits:
            assert s.count(old) == 1, (name, old)
            s = s.replace(old, new)
        SRC.write_text(s)
        diff = subprocess.run(["git", "diff", "--", str(SRC)], capture_output=True, text=True)
        (out / f"{name}.patch").write_text(diff.stdout)
        try:
            print(run(out, name, s.encode()), flush=True)
        finally:
            SRC.write_bytes(base)
        assert SRC.read_bytes() == base
    print(run(out, "restored", base), flush=True)


if __name__ == "__main__":
    main()
