"""Mutation check of src/cells_owned.rs, the one-scan writer, with receipts.

Run from the repository root:

    python3 docs/journal/2026-11-18-one-scan-writer/mutate.py RECEIPTS_DIR

The Guardian's D113 asked for a sound scorer (#145 REVIEW 6988584322): an earlier version
counted any nonzero cargo exit as a kill, so a build or dependency failure would have scored as
one. This version:
- **The baseline first:** the unmutated source must build, and each test selection must run at
  least one test, with every test passing. The test names are recorded.
- **Each mutant alone:** its test binary must build. A mutant that fails to build is INVALID,
  never a kill. Then each selection runs. A kill is a runtime test failure: a selection whose
  test binary ran, reported `test result: FAILED`, and named its failing tests. Each mutant's
  failing tests and first panic message are recorded.
- **The receipts:** RECEIPTS_DIR/receipts.json (the baseline, each mutant's verdict, failing
  tests and panic message), and each run's full cargo log beside it.

The exit is 0 only if the baseline passes and every mutant builds and is killed at run time.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

SRC = "src/cells_owned.rs"
SELECTIONS = ["owned_writer", "miri_cells_owned"]
CARGO = ["rustup", "run", "1.99.1", "cargo"]
MUTATIONS = [
    ("the survival certificate inverted", "} else if w == later {", "} else if w != later {"),
    ("no wrap on the marked spine",
     "                if right >= p {\n                    right -= p;\n                }",
     "                if right >= p + p {\n                    right -= p;\n                }"),
    ("the commands run last parent first", "        for i in tail..nr {", "        for i in (tail..nr).rev() {"),
    ("no command stored", "                        if tail > j {", "                        if tail > j + nr {"),
    ("an owned child's position not set", "                        *pos.add((r - ro) / 16) = d;",
     "                        let _ = (r, d);"),
    ("the root one place off", "*pos.add(nr - 1) = ((p - start) % p) as u32;",
     "*pos.add(nr - 1) = ((p - start + 1) % p) as u32;"),
    ("the carried descriptors not updated", "            (na, nb) = (a, b);", "            let _ = (na, nb);"),
    ("the guard admits a shorter cycle", "if p != n || nr == 0 || word as usize != self.rule_end - 16 {",
     "if p > n || nr == 0 || word as usize != self.rule_end - 16 {"),
]


def cargo(args, log):
    p = subprocess.run(CARGO + args, capture_output=True, text=True, env=dict(os.environ))
    Path(log).write_text(f"$ cargo {' '.join(args)}\nexit {p.returncode}\n\n{p.stdout}\n{p.stderr}")
    return p.returncode, p.stdout + "\n" + p.stderr


def tests(out):
    """(tests run, passed names, failed names, summary results, first panic message)"""
    ran = sum(int(n) for n in re.findall(r"^running (\d+) tests?$", out, re.M))
    passed = re.findall(r"^test (\S+) \.\.\. ok$", out, re.M)
    failed = re.findall(r"^test (\S+) \.\.\. FAILED$", out, re.M)
    results = re.findall(r"^test result: (ok|FAILED)\.", out, re.M)
    m = re.search(r"panicked at ([^\n]+)\n([^\n]+)", out)
    return ran, passed, failed, results, (f"{m.group(1)}: {m.group(2)}" if m else None)


def check(tag, logs):
    """Build, then each selection: the receipt"""
    rc, _ = cargo(["test", "--release", "--lib", "--no-run"], logs / f"{tag}-build.log")
    rec = {"built": rc == 0, "selections": {}}
    if rc != 0:
        return rec
    for sel in SELECTIONS:
        rc, out = cargo(["test", "--release", "--lib", sel], logs / f"{tag}-{sel}.log")
        ran, passed, failed, results, panic = tests(out)
        rec["selections"][sel] = {"exit": rc, "ran": ran, "passed": passed, "failed": failed,
                                  "results": results, "panic": panic}
    return rec


def main():
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "mutation-receipts")
    out.mkdir(parents=True, exist_ok=True)
    orig = open(SRC).read()
    receipts = {"source": SRC, "selections": SELECTIONS, "mutants": []}
    base = check("baseline", out)
    receipts["baseline"] = base
    ok = base["built"] and all(s["exit"] == 0 and s["ran"] > 0 and not s["failed"] and s["results"]
                               and all(r == "ok" for r in s["results"]) for s in base["selections"].values())
    print(f"baseline: {'PASS' if ok else 'FAIL'}: "
          + "; ".join(f"{k} ran {v['ran']}, passed {len(v['passed'])}" for k, v in base["selections"].items()),
          flush=True)
    if not ok:
        json.dump(receipts, open(out / "receipts.json", "w"), indent=1)
        return 1
    killed = 0
    try:
        for i, (name, old, new) in enumerate(MUTATIONS):
            assert orig.count(old) == 1, name
            open(SRC, "w").write(orig.replace(old, new))
            rec = check(f"mutant-{i}", out)
            sels = rec["selections"].values()
            kill = rec["built"] and any(s["ran"] > 0 and s["failed"] and "FAILED" in s["results"]
                                        and s["exit"] != 0 for s in sels)
            verdict = "INVALID (did not build)" if not rec["built"] else "KILLED" if kill else "SURVIVED"
            failing = sorted({t for s in sels for t in s["failed"]})
            panic = next((s["panic"] for s in sels if s["panic"]), None)
            receipts["mutants"].append({"name": name, "verdict": verdict, "failing": failing,
                                        "panic": panic, **rec})
            killed += kill
            print(f"{verdict}: {name}" + (f": {', '.join(failing)}; {panic}" if kill else ""), flush=True)
    finally:
        open(SRC, "w").write(orig)
    json.dump(receipts, open(out / "receipts.json", "w"), indent=1)
    print(f"{killed} of {len(MUTATIONS)} mutants killed at run time (baseline passed)")
    return 0 if killed == len(MUTATIONS) else 1


if __name__ == "__main__":
    sys.exit(main())
