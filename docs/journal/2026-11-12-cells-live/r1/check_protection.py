"""Untimed R1 source/freeze checks; no generation, decoder or candidate clock."""

import ast
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
BASE = "7b53ded9eb139fdd69b4c29565e7373ac95e90d4"
FREEZE = "3b43af956d553905ea744f6cb6e1c785a0f8297a"
HISTORICAL = "a08277e1c36e276cfea7b3116d99ea5fd93f0e84"
EVIDENCE = ROOT / "docs/journal/2026-11-12-cells-live/evidence"


def git(*args):
    """Read immutable local Git objects."""
    return subprocess.check_output(["git", "-C", str(ROOT), *args])


def sha(data):
    """Digest bytes without changing the source."""
    return hashlib.sha256(data).hexdigest()


allowed = {
    "bench/cells-gap/live/driver.py",
    "bench/cells-gap/live/test_driver.py",
    "bench/cells-gap/live/README.md",
    "docs/journal/2026-11-12-cells-live/HANDOFF.md",
}
protected = []
for path in git("ls-tree", "-r", "--name-only", BASE).decode().splitlines():
    if path not in allowed:
        assert (ROOT / path).read_bytes() == git("show", f"{BASE}:{path}"), path
        protected.append(path)
historical = (
    git(
        "ls-tree",
        "-r",
        "--name-only",
        HISTORICAL,
        "bench/cells-gap",
        "docs/journal/monodromy/2026-11-10-cells-gap",
    )
    .decode()
    .splitlines()
)
for path in historical:
    assert (ROOT / path).read_bytes() == git("show", f"{HISTORICAL}:{path}"), path
manifest = json.loads((EVIDENCE / "SHA256.json").read_text())
for path, identity in manifest.items():
    assert sha((EVIDENCE / path).read_bytes()) == identity, path
source = json.loads((EVIDENCE / "source-correspondence.json").read_text())
for path, identity in source["original"].items():
    assert sha(git("show", source["pin"] + ":" + path)) == identity, path
plan = "docs/journal/2026-11-12-cells-live/PLAN.md"
assert (ROOT / plan).read_bytes() == git("show", f"{FREEZE}:{plan}")
driver = "bench/cells-gap/live/driver.py"
trees = [ast.parse(git("show", f"{BASE}:{driver}")), ast.parse((ROOT / driver).read_text())]
for name in ("schedule", "interval", "analyse"):
    nodes = [
        next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == name)
        for tree in trees
    ]
    assert ast.dump(nodes[0]) == ast.dump(nodes[1]), name
print(
    json.dumps(
        dict(
            reviewed_base=BASE,
            source_pin=source["pin"],
            plan_freeze=FREEZE,
            protected_files=len(protected),
            historical_files=len(historical),
            original_hashes=len(source["original"]),
            archived_digests=len(manifest),
            unchanged_functions=["schedule", "interval", "analyse"],
            changed_file_sha256={p: sha((ROOT / p).read_bytes()) for p in sorted(allowed)},
            plan_sha256=sha((ROOT / plan).read_bytes()),
            source_manifest_sha256=sha((EVIDENCE / "source-correspondence.json").read_bytes()),
            measurements=0,
        ),
        indent=2,
    )
)
