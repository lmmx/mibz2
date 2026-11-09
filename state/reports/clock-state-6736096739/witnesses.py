"""Algebraic counterexamples only; no workload timing."""
import json,math
# Ratios are candidate/reference elapsed time.
def run(n,f,m=0):
    t=n/f+m
    return {"wall":t,"cpu":t,"cycles":n+f*m}
fast=run(4,4);slow=run(4,2);extra=run(8,4)
assert slow["wall"]==extra["wall"]==2
assert slow["cycles"]==4 and extra["cycles"]==8
mf=run(4,4,1);ms=run(4,2,1)
assert ms["wall"]>mf["wall"] and ms["cycles"]<mf["cycles"]
# True candidate demand .9, mismatched frequency multiplies observed time ratio.
ratios=[.9*2,.9/2];assert abs(math.sqrt(math.prod(ratios))-.9)<1e-12
# Post-treatment low-state deletion selects away the candidate's harm.
all_pairs=[.9,.9,1.8,1.8]
assert math.prod(all_pairs)**.25>1 and math.sqrt(.9*.9)<1
print(json.dumps({"same_wall_cpu_different_cause":{"half_clock":slow,"double_work":extra},
"cycles_can_favour_slower_execution":{"high_frequency":mf,"low_frequency":ms},
"balanced_mismatch_ratios":ratios,
"selection":{"all_pairs_gmean":math.prod(all_pairs)**.25,"retained_fast_pairs_gmean":.9},
"limits":"Synthetic algebra, not #270 raw data or evidence of its cause."},indent=2))
