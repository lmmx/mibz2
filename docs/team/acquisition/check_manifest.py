"""Check a proposal's acquisition manifest against the registry (ACQUISITION-FIRST.md §5).

The manifest names the registry edges it uses (``uses``) and its ``target`` object. It may
also declare new edges inline, as ``[[edge]]`` tables with the registry's fields.

The check fails (exit 1) unless every used edge's inputs are in the entry state I0 or are
produced by another used edge, the graph is acyclic and reaches the target, and no edge on the
path to the target has status "assumed" or "open", and unless no edge before the final one
already yields the text or an object that determines it (the registry's [determines] table). It
then reports the dependent edges (D or Q
of n-scale) on that path, the cost class that must never pass as "preprocessing".

Run: python3 docs/team/acquisition/check_manifest.py <manifest.toml> [--registry PATH]
"""

import argparse
import sys
from pathlib import Path

if sys.version_info >= (3, 11):
    import tomllib
else:  # pragma: no cover - the repository targets 3.10+, the checker needs 3.11's tomllib
    sys.exit("check_manifest.py needs Python 3.11 or later (tomllib)")

HERE = Path(__file__).resolve().parent
BLOCKING = {"assumed", "open"}
FIELDS = ("id", "inputs", "output", "status", "cost", "dependent", "evidence")
COST_TERMS = ("W", "D", "B", "Q", "M")


def load(path):
    """Return the parsed TOML file at ``path``."""
    with open(path, "rb") as f:
        return tomllib.load(f)


def n_scale(expr):
    """Say whether a cost expression grows with n (a crude, conservative textual test)."""
    return "n" in expr.replace("none", "")


def check(manifest, registry):
    """Return (ok, report lines) for a manifest against a registry."""
    lines, ok = [], True
    edges = {e["id"]: e for e in registry.get("edge", [])}
    for e in manifest.get("edge", []):
        missing = [f for f in FIELDS if f not in e]
        if missing:
            return False, [f"FAIL: inline edge {e.get('id', '?')} lacks {missing}"]
        edges[e["id"]] = e
    for e in edges.values():
        absent = [t for t in COST_TERMS if t not in e["cost"]]
        if absent:
            return False, [f"FAIL: edge {e['id']} has no cost term(s) {absent}"]
    used = list(manifest.get("uses", [])) + [e["id"] for e in manifest.get("edge", [])]
    unknown = [u for u in used if u not in edges]
    if unknown:
        return False, [f"FAIL: unknown edge(s) {unknown}: define them inline with a cost"]
    known = set(registry["entry"]["objects"])
    producer, pending = {}, [edges[u] for u in dict.fromkeys(used)]
    while pending:
        ready = [e for e in pending if set(e["inputs"]) <= known]
        if not ready:
            for e in pending:
                need = sorted(set(e["inputs"]) - known)
                lines.append(f"FAIL availability: {e['id']} needs {need}, never derived from I0")
            return False, lines
        for e in ready:
            pending.remove(e)
            known.add(e["output"])
            producer.setdefault(e["output"], e)
    target = manifest["target"]
    if target not in producer:
        return False, [f"FAIL: nothing used produces the target {target!r}"]
    path, stack, seen = [], [target], set()
    while stack:
        obj = stack.pop()
        if obj in seen or obj not in producer:
            continue
        seen.add(obj)
        e = producer[obj]
        path.append(e)
        stack.extend(e["inputs"])
    for e in path:
        if e["status"] in BLOCKING:
            ok = False
            lines.append(
                f"FAIL availability: {e['id']} ({'+'.join(e['inputs'])}"
                f" -> {e['output']}) is {e['status'].upper()}: {e['evidence']}"
            )
    # Record 84's gap: a path that already holds the output (the text, or an object that
    # determines it) before its final edge passes availability but pays twice. Fail it.
    determines = set(registry.get("determines", {}).get("text", []))
    determines |= set(manifest.get("determines", {}).get("text", []))
    determines.add("text")
    final = producer[target]
    for e in path:
        if e is not final and e["output"] in determines:
            ok = False
            lines.append(
                f"FAIL cost: {e['id']} already yields {e['output']!r}, which determines the text, "
                f"before the final edge {final['id']}: everything after it is paid twice"
            )
    dep = [e for e in path if e["dependent"] or n_scale(e["cost"]["D"]) or n_scale(e["cost"]["Q"])]
    lines.append(f"path to {target!r}: {' <- '.join(e['id'] for e in path)}")
    for e in dep:
        c = e["cost"]
        lines.append(f"  dependent/n-scale: {e['id']}: D = {c['D']}, Q = {c['Q']} ({e['status']})")
    lines.insert(0, ("PASS" if ok else "FAIL") + f": {manifest.get('name', '(unnamed)')}")
    return ok, lines


def main():
    """Check the manifest named on the command line; exit 1 if it fails."""
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("manifest")
    ap.add_argument("--registry", default=HERE / "primitives.toml")
    a = ap.parse_args()
    ok, lines = check(load(a.manifest), load(a.registry))
    print("\n".join(lines))
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
