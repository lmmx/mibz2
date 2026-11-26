"""Shadow view of new-work candidates; no runtime or notification dependencies."""

from replay import overlaps


def candidates(state, capabilities=None):
    """Project claimability from a complete observed trunk state, not session history."""
    keys = ("items", "live_holds", "pending", "flagged", "paths", "cap")
    if state.get("kind") != "committed" or any(state.get(k) is None for k in keys):
        return {"available": [], "blocked": {}, "unavailable": True}
    items, holds = state["items"], state["live_holds"]
    paths = set(state["paths"])
    available, blocked = [], {}
    for cid, item in sorted(items.items(), key=lambda x: (x[1].get("priority", 999), x[0])):
        if item.get("status") != "open":
            continue
        reasons = []
        if item.get("needs_louis"):
            reasons.append("reserved-decision")
        if cid in state["flagged"]:
            reasons.append("flag")
        if cid in state["pending"]:
            reasons.append("pending-integration")
        if cid in holds:
            reasons.append("held")
        if len(holds) >= state["cap"]:
            reasons.append("capacity")
        if capabilities is not None and not set(item.get("needs", [])) <= set(capabilities):
            reasons.append("capabilities")
        for premise in item.get("premises", []):
            if premise not in paths and not any(p.startswith(premise.rstrip("/") + "/") for p in paths):
                reasons.append("missing-premise:" + premise)
        for other, hold in sorted(holds.items()):
            if other == cid:
                continue
            same = set(item.get("mechanisms", [])) & set(hold.get("mechanisms", []))
            owned = item.get("owned_paths", []) if item.get("zone") == "striated" else []
            clash = any(overlaps(a, b) for a in owned for b in hold.get("owned_paths", []))
            if same or clash:
                reasons.append("overlap:" + other)
        if reasons:
            blocked[cid] = reasons
        else:
            available.append(cid)
    return {"available": available, "blocked": blocked, "unavailable": False}


def newly_available(before, after, capabilities=None):
    """Describe a change in availability, without creating work or consuming the event."""
    old, new = (candidates(s, capabilities) for s in (before, after))
    if old["unavailable"] or new["unavailable"]:
        return None
    return [cid for cid in new["available"] if cid not in old["available"]]
