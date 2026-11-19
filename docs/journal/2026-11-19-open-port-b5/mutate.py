"""Mutation check of the open kernel's port (src/open.rs, src/open/endpoint.rs and its
admission in src/cells_lazy.rs), with receipts: B5's evidence for #145 CHECKPOINT 7004759994.

Run from the repository root:

    python3 docs/journal/2026-11-19-open-port-b5/mutate.py RECEIPTS_DIR

The scorer is the one D113 asked for (#145 REVIEW 6988584322), as in the one-scan writer's:
- **The baseline first:** the unmutated source must build, and each test selection must run at
  least one test, with every test passing. The test names are recorded.
- **Each mutant alone:** its test binary must build. A mutant that fails to build is INVALID,
  never a kill. Then each selection runs. A kill is a runtime test failure: a selection whose
  test binary ran, reported `test result: FAILED`, and named its failing tests. Each mutant's
  failing tests and first panic message are recorded.
- **The receipts:** RECEIPTS_DIR/receipts.json, and each run's full cargo log beside it.

Every run ends: the port bounds `emit`'s walk by the retained rows and asserts that each band in
`induced` takes a row. (Before those two lines, two of these mutants made a loop grow until the
OOM killer stopped the test binary, which this scorer rightly does not count as a kill.)
The exit is 0 only if the baseline passes and every mutant builds and is killed at run time.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

SELECTIONS = ["open::"]
CARGO = ["rustup", "run", "1.99.1", "cargo"]
# (name, file, the line as shipped, the mutant)
MUTATIONS = [
    ("the primary's run loses its leading fragment", "src/open.rs",
     "            if pos < p {", "            if pos + 1 < p {"),
    ("each byte's first row one place on", "src/open.rs",
     "        let start = total;", "        let start = total + 1;"),
    ("the marked row's rank one place on", "src/open.rs",
     "mark = m + (p - x);", "mark = m + (p - x) + 1;"),
    ("a band's destination one place on", "src/open.rs",
     "let dst = k.wrapping_add(b.delta) as usize;", "let dst = k.wrapping_add(b.delta).wrapping_add(1) as usize;"),
    ("a short cycle over-repeated", "src/open.rs",
     "        let take = out.len().min(n - out.len());", "        let take = out.len();"),
    ("the open kernel's block truncated in OpenCells", "src/open.rs",
     "            self.open.run::<true, true>(r.records(), n, p, LIMIT, out);",
     "            self.open.run::<true, true>(r.records(), n, p, LIMIT, out);\n            out.pop();"),
    ("equal widths taken as the right side's cut", "src/open/endpoint.rs",
     "            if a.width <= b.width {", "            if a.width < b.width {"),
    ("a rule expanded as a copy on its first visit", "src/open/endpoint.rs",
     "                if rule.first == u32::MAX {", "                if rule.first != u32::MAX {"),
    ("a node's rows all placed at its first", "src/open/endpoint.rs",
     "let dst = e.bottom[id as usize] + in_node;", "let dst = e.bottom[id as usize];"),
    ("a band past its node's end", "src/open/endpoint.rs",
     "let width = (end - k).min(node.width - in_node);", "let width = (end - k).min(node.width - in_node + 1);"),
    ("the density line admitting r/n 0.15", "src/cells_lazy.rs",
     "records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n",
     "records * LAZY_DENSITY.1 <= n * LAZY_DENSITY.0 && records * 64 >= n"),
    ("the sparse gate refusing exactly one record in 64 rows", "src/cells_lazy.rs",
     "records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n",
     "records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 > n"),
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
    sources = sorted({m[1] for m in MUTATIONS})
    origs = {f: open(f).read() for f in sources}
    receipts = {"sources": sources, "selections": SELECTIONS, "mutants": []}
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
        for i, (name, src, old, new) in enumerate(MUTATIONS):
            assert origs[src].count(old) == 1, name
            open(src, "w").write(origs[src].replace(old, new.replace("\\n", "\n")))
            rec = check(f"mutant-{i}", out)
            sels = rec["selections"].values()
            kill = rec["built"] and any(s["ran"] > 0 and s["failed"] and "FAILED" in s["results"]
                                        and s["exit"] != 0 for s in sels)
            verdict = "INVALID (did not build)" if not rec["built"] else "KILLED" if kill else "SURVIVED"
            failing = sorted({t for s in sels for t in s["failed"]})
            panic = next((s["panic"] for s in sels if s["panic"]), None)
            open(src, "w").write(origs[src])
            receipts["mutants"].append({"name": name, "file": src, "verdict": verdict, "failing": failing,
                                        "panic": panic, **rec})
            killed += kill
            print(f"{verdict}: {name}" + (f": {', '.join(failing)}; {panic}" if kill else ""), flush=True)
    finally:
        for f, text in origs.items():
            open(f, "w").write(text)
    json.dump(receipts, open(out / "receipts.json", "w"), indent=1)
    print(f"{killed} of {len(MUTATIONS)} mutants killed at run time (baseline passed)")
    return 0 if killed == len(MUTATIONS) else 1


if __name__ == "__main__":
    sys.exit(main())
