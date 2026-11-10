"""The target-gated noninferiority workflow's verdict gate (.github/workflows/target-gated-ni.yml,
its `aggregate` job's last step; the Guardian's D078): the job's status follows the judge's
verdict, after the evidence is uploaded.

    python3 bench/tg_ni_gate.py --label LABEL --verdict ni.json --exit JUDGE_EXIT [--summary FILE]

- **qualification:** passes (exit 0) only if the judge exited 0 and its verdict is valid and
  passes condition 2. A valid NOT SHOWN (judge exit 1), invalid evidence (exit 2), a missing or
  unreadable verdict, or any disagreement between the exit and the verdict fails (exit 1).
- **smoke** and **pilot** (the registration's §2.8 design pilot): never evidence. The judge must have refused it as smoke: its
  verdict present, not valid, and among its problems the label's refusal. And the run must
  pass the structural admission (`service_tg.py structure`, `--structure`): every shard present,
  its artifacts listed and downloaded, the manifest valid, the history reconciled, the files
  whole. Only then is it success (exit 0), and the summary says "smoke: not evidence" (or
  "pilot: not evidence"). A refused label with nothing behind it fails (exit 1). The pilot's own analysis is bench/tg_ni_pilot.py's, offline.
- Any other label fails.

It writes one line to `--summary` (the step summary) either way.
"""

import argparse
import json
import sys

def gate(label, verdict, judge_exit, structure=None):
    """(exit status, one-line summary)"""
    if label in ("smoke", "pilot"):
        if not isinstance(verdict, dict):
            return 1, f"{label}: not evidence; FAILED: the judge wrote no verdict (the plumbing is broken)"
        if verdict.get("valid"):
            return 1, f"{label}: not evidence; FAILED: the judge admitted a {label} run"
        if not any(f"labelled '{label}'" in str(p) for p in verdict.get("problems", [])):
            return 1, f"{label}: not evidence; FAILED: the judge did not refuse it as {label}"
        if judge_exit != 2:
            return 1, f"{label}: not evidence; FAILED: the judge exited {judge_exit}, not 2 (refused)"
        # (the refusal alone shows only that the label is refused: the plumbing is shown by the
        # structural admission, every shard present, listed, downloaded, reconciled and whole)
        if not isinstance(structure, dict) or structure.get("label") != label:
            return 1, f"{label}: not evidence; FAILED: no structural admission for a {label} run"
        if not structure.get("valid"):
            why = "; ".join(map(str, structure.get("problems", [])[:3])) or "not admitted"
            return 1, f"{label}: not evidence; FAILED: the structural admission failed: {why}"
        return 0, (f"{label}: not evidence (the judge refused it as a {label} run, as it must); "
                   f"structurally admitted: all {len(structure.get('shards', []))} shards, R {structure.get('rounds')}")
    if label == "qualification":
        if not isinstance(verdict, dict):
            return 1, "qualification: FAILED: no readable verdict (INVALID)"
        valid, passes = bool(verdict.get("valid")), bool(verdict.get("passes"))
        if judge_exit == 0 and valid and passes:
            return 0, "qualification: condition 2 met at every cell (PASS)"
        if judge_exit == 1 and valid and not passes:
            failing = "; ".join(verdict.get("failing", []) + [f"RSS {c}" for c in verdict.get("rss_regression", [])])
            return 1, f"qualification: condition 2 NOT SHOWN: {failing}"
        if judge_exit == 2 and not valid:
            return 1, "qualification: INVALID: " + "; ".join(map(str, verdict.get("problems", [])[:5]))
        return 1, f"qualification: FAILED: the judge's exit {judge_exit} disagrees with its verdict (valid {valid}, passes {passes})"
    return 1, f"FAILED: unknown label {label!r}"


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--label", required=True)
    p.add_argument("--verdict", required=True)
    p.add_argument("--exit", type=int, required=True)
    p.add_argument("--structure", help="smoke and pilot: service_tg.py structure's JSON")
    p.add_argument("--summary")
    a = p.parse_args(argv)
    try:
        with open(a.verdict) as f:
            verdict = json.load(f)
    except (OSError, json.JSONDecodeError):
        verdict = None
    structure = None
    if a.structure:
        try:
            with open(a.structure) as f:
                structure = json.load(f)
        except (OSError, json.JSONDecodeError):
            structure = None
    status, line = gate(a.label, verdict, a.exit, structure)
    print(line)
    if a.summary:
        with open(a.summary, "a") as f:
            f.write(f"\n**{line}**\n")
    return status


if __name__ == "__main__":
    sys.exit(main())
