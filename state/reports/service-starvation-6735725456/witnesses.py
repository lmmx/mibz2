"""Untimed abstract witnesses; no decoder execution or performance measurement."""
import json
from fractions import Fraction
from itertools import product
states=0
for t,l,k in product(range(1,5),range(1,9),range(1,5)):
    for p,a,u in product(range(l),range(k),range(t+1)):
        free=t+l+k-p-a-u
        assert free>=2
        states+=1
# Source priority: empty-P prepare, otherwise finish, otherwise below-L prepare.
def choose(p,j,front,q,l):
    if j and front and p==0:return "prepare"
    if q:return "finish"
    if j and front and p<l:return "prepare"
    return "wait"
p,j,front,q,l=1,1,4,2,2
first=choose(p,j,front,q,l);q-=1
second=choose(p,j,front,q,l);q-=1
assert (first,second)==("finish","finish")
p-=1 # team 0 takes the one prepared block; team 1 now cannot take one
assert p==0
# Constructed event units, not times from a run:
# t0: P=B=0; t1: P=1,B=0; t9: B=1, TeamGot.
recorded_prep=9
actual_no_prep=1
prepared_but_no_back=8
assert recorded_prep==actual_no_prep+prepared_but_no_back
# Equal marginal service utilization does not fix joint overlap with team starvation.
starvation=set(range(1))
busy_a=set(range(7));busy_b=set(range(3,10))
assert len(busy_a)==len(busy_b)==7
# A rigid 1-prep/1-finish split cannot improve the ideal shared-pool capacity bound.
for p,f in product(range(1,21),repeat=2):
    assert min(Fraction(1,p),Fraction(1,f))<=Fraction(2,p+f)
out={
"front_availability":{"states":states,"minimum_free_at_eligible_prep_decision":2,
"assumptions":"T=1..4,L=1..8,K=1..4; P<L, other preparations A<K, walking teams U<=T; disjoint ownership, no lost buffers"},
"finish_priority_witness":{"initial":"P=1,J=1,Q=2; free front and back sets","choices":[first,second],"after_one_team_take":"P=0, both service workers nonpreemptively finishing"},
"wait_label_witness":{"recorded_prep_units":recorded_prep,"actual_P_zero_units":actual_no_prep,"P_positive_B_zero_units":prepared_but_no_back},
"equal_marginals":{"service_busy_fraction":[0.7,0.7],"starvation_overlap_fraction_case_a":len(starvation&busy_a)/len(starvation),"case_b":len(starvation&busy_b)/len(starvation)},
"fixed_split_bound":{"positive_integer_cost_pairs_checked":400,"relation":"min(1/p,1/f) <= 2/(p+f)"},
"scope":"Abstract witnesses and finite checks only. No claim these schedules occurred in #271; no Rust safety proof; no speedup estimate."
}
print(json.dumps(out,indent=2))
