"""Untimed counterexamples to 'unknown only if no level parses'; not a collector."""
import json
from math import inf
from pathlib import Path
cases=[]
# None means unreadable; inf is an explicit unlimited reading.
for observed, hidden in [([inf,None],2),([4,None],2),([8,None],2),([inf,None],8)]:
    parsed=[x for x in observed if x is not None]
    proposed=min(parsed) if parsed else None
    actual=min([x for x in observed if x is not None]+[hidden])
    cases.append(dict(observed=['max' if x==inf else x for x in observed],
                      hidden_parent=hidden,proposed='unlimited' if proposed==inf else proposed,
                      actual_cap=actual,proposed_admit=proposed is not None and proposed>=4,
                      actual_admit=actual>=4))
assert sum(c['proposed_admit'] and not c['actual_admit'] for c in cases)==3
Path(__file__).with_name('witnesses.json').write_text(json.dumps(cases,indent=2)+'\n')
print(json.dumps(cases,indent=2))
