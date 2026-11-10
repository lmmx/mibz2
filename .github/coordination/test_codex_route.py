#!/usr/bin/env python3
"""Offline sanity checks for Codex issue routing; run: python3 .github/coordination/test_codex_route.py.

No network or GitHub token required. The real route/verify/audit commands still
must be checked against #363 after this setup PR merges.
"""
import importlib.util
from pathlib import Path

p = Path(__file__).with_name("coord.py")
spec = importlib.util.spec_from_file_location("coord", p)
assert spec and spec.loader
coord = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coord)

assert coord.ROLES["codex"]["inbox"] == 363
# Adding a fresh inbox must not re-audit old channel traffic.
assert coord.ROLES["codex"]["offset"] > coord.CUTOVER
assert coord.CUTOVER == min(r["offset"] for r in coord.ROLES.values())
assert coord.CUTOVER >= 6646249200
assert coord.ROLES["codex"]["cursor"] == ("team-state/codex", "state/cursor.json")
assert coord.canonical("codex") == "codex"
assert coord.roles_named("codex") == ["codex"]
assert coord.completed("codex", {"processed_comment_ids": [100, 101]}) == {100, 101}
incoming = "FROM: coordinator\nTO: codex\nRE: cells @ abcdef\nTYPE: CHECKPOINT\n\nACTOR: ChatGPT Work (acting)"
outgoing = "FROM: codex\nTO: coordinator\nRE: inbox 123\nTYPE: REVIEW"
assert coord.misrouted(363, incoming, 9000000000) is None
assert coord.misrouted(145, outgoing, 9000000001) is None
assert coord.misrouted(363, outgoing, 9000000002) is not None
assert coord.misrouted(145, incoming, 9000000003) is not None
assert coord.misrouted(363, incoming.replace("TO: codex", "TO: monodromy"), 9000000004) is not None
assert coord.misrouted(363, incoming.replace("TYPE: CHECKPOINT", "TYPE: QUEUED"), 9000000005) is not None
print("Codex routing static checks passed; real GitHub delivery remains untested.")
