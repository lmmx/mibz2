"""Offline D033 migration checks. Place old_coord.py and coord.py beside this file.

old: b58cb628ccbb86b34a9ceac8e852f5d978a29a0d
new: 40709781a39758d14e037d70a9bb20a65fc0edf9
No network calls, writes to GitHub, or live migration.
"""
import ast
import copy
import json
from pathlib import Path
import coord
import old_coord

def msg(sender, kind, to=None, text=""):
    return f"FROM: {sender}\nTYPE: {kind}\n" + (f"TO: {to}\n" if to else "") + text

def done_cursor(role, ids):
    if role == "optimizer":
        return {"inbox_briefs": [{"comment_id": i, "status": "finished"} for i in ids]}
    key = "consumed_incoming_comment_ids" if role == "guardian" else "processed_comment_ids"
    return {key: list(ids)}

results = {"phase_comparisons": 0, "cursor_paths_checked": 0, "function_ast_equal": []}
for role, cfg in coord.ROLES.items():
    assert cfg["cursor"] == (f"team-state/{role}", "state/cursor.json")
    assert cfg["inbox"] == old_coord.ROLES[role]["inbox"]
    assert cfg["offset"] == old_coord.ROLES[role]["offset"]
    results["cursor_paths_checked"] += 1
    cid = cfg["offset"] + 1
    queued_id = 8_000_000_001
    inbox = [{"id": cid, "body": msg("coordinator", "CHECKPOINT", role)}]
    staged = {"id": queued_id, "body": msg("coordinator", "QUEUED", role)}
    review = {"id": 8_000_000_002, "body": msg(role, "REVIEW", text=str(cid))}
    for phase in ("unreceived", "in_progress", "complete", "reviewed", "finished"):
        cur = done_cursor(role, [cid] if phase == "finished" else [])
        cur["inbox_comments"] = [{"comment_id": cid, "status": phase}]
        channel = [staged] + ([review] if phase in ("reviewed", "finished") else [])
        moved = json.loads(json.dumps(cur))
        moved["migrated_from"] = {"branch": old_coord.ROLES[role]["cursor"][0], "commit": "a"*40}
        before = old_coord.state(role, channel=channel, inbox=inbox, cur=cur)
        after = coord.state(role, channel=channel, inbox=inbox, cur=moved)
        assert before == after, (role, phase, before, after)
        assert after["queued"] == [queued_id]
        results["phase_comparisons"] += 1

# A complete receipt plus REVIEW is still busy until Guardian consumes the ID.
cid = coord.ROLES["guardian"]["offset"] + 1
inbox = [{"id": cid, "body": msg("coordinator", "CHECKPOINT", "guardian")}]
channel = [{"id": 8_000_000_003, "body": msg("guardian", "REVIEW", text=str(cid))}]
cur = {"inbox_comments": [{"comment_id": cid, "status": "complete"}],
       "consumed_incoming_comment_ids": []}
assert coord.state("guardian", channel=channel, inbox=inbox, cur=cur)["incomplete"] == [cid]
results["complete_before_consumed_remains_busy"] = True

# A stale copied cursor with provenance does not acquire a late old-branch completion.
stale = copy.deepcopy(cur)
stale["migrated_from"] = {"branch": "review/compact30-guardian-shadow", "commit": "a"*40}
old_later = copy.deepcopy(cur)
old_later["consumed_incoming_comment_ids"] = [cid]
assert coord.state("guardian", channel=channel, inbox=inbox, cur=old_later)["incomplete"] == []
assert coord.state("guardian", channel=channel, inbox=inbox, cur=stale)["incomplete"] == [cid]
results["stale_copy_is_not_reconciled_by_router"] = True

def funcs(path):
    return {n.name: ast.dump(n, include_attributes=False)
            for n in ast.parse(Path(path).read_text()).body if isinstance(n, ast.FunctionDef)}
old, new = funcs(Path(__file__).with_name("old_coord.py")), funcs(Path(__file__).with_name("coord.py"))
for name in ("completed", "state", "route", "verify", "misrouted"):
    assert old[name] == new[name], name
    results["function_ast_equal"].append(name)
print(json.dumps(results, indent=2))
