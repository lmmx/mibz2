"""D053: deterministic model of the permitted cross-case trace schedule.
This is a trace-ownership model, not a Rust runtime reproduction.
"""
done = []
late_a = {"name": "mibz2-coop-1", "marks": ["BuildStart", "WalkStart"]}
# A's reader is dropped while its feeder still owns Arc<StreamDecoder>.
# trace::take sees no finished helper buffer yet.
a_taken, done = done, []
assert not a_taken
# B starts: set_level(1) clears DONE, but does not clear other threads' TLS.
done.clear()
# A's old helper exits after B's reset, publishing its TLS buffer into DONE.
done.append(late_a)
# B is a tiny/early-error case with no helper; its own threads have ended.
b_taken, done = done, []
build = any(t["name"].startswith("mibz2-coop") and "BuildStart" in t["marks"] for t in b_taken)
walk = any(t["name"].startswith("mibz2-coop") and "WalkStart" in t["marks"] for t in b_taken)
assert build and walk
print("A: no attributed helper; B: build=true walk=true despite no B helper (permitted schedule)")
# Independent marginal counts cannot establish a joint count.
cases = [(True, False), (False, True)]
assert sum(x for x,y in cases)==sum(y for x,y in cases)==1
assert sum(x and y for x,y in cases)==0
print("Equal build/walk marginal counts do not establish the reported intersection")
