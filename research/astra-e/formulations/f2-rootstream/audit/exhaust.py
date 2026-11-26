"""All ordered-by-creation trees to five nodes and short legal stable selector tapes."""
from collections import Counter
from itertools import product
import json
from tree import audit

total=Counter();coverage=Counter(); witnesses={}
for n,limit in ((1,7),(2,7),(3,7),(4,6),(5,5)):
    for tail in product(*(range(i) for i in range(1,n))):
        parent=(-1,)+tail
        for m in range(1,limit+1):
            for labels in product(range(n),repeat=m):
                counts=audit(labels,parent)
                if counts is None: continue
                total.update(counts);coverage[n]+=1
                if counts.get("nonleaf_bottlenecks") and "partial_child" not in witnesses:
                    witnesses["partial_child"]=dict(parents=parent,root=labels,counts=counts)
                if counts["visits"]>counts["materialized_runs"] and "reentry" not in witnesses:
                    witnesses["reentry"]=dict(parents=parent,root=labels,counts=counts)
# Relax either of the two hypotheses and aligned selector boundaries can falsely
# split a maximal terminal run. These literal words exhibit the failure.
witnesses["uncoalesced"]={"selector_tapes":[[[1,1],[1,1]],[[1,2]]],"descent_output":[[1,1],[1,1]],"maximal_runs":[[1,2]]}
witnesses["shared_label"]={"selector_tapes":[[[1,1],[2,1]],[[1,1]],[[2,1]]],"terminal_label":{"1":"a","2":"a"},"descent_output":[["a",1],["a",1]],"maximal_runs":[["a",2]]}
print(json.dumps(dict(cases_by_nodes=coverage,totals=total,witnesses=witnesses),indent=2))
