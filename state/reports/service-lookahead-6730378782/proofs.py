"""Finite check of the front-pool admission bound, not scheduler or Rust verification."""
import json,pathlib
count=0;minimum=1000
for T in range(1,5):
 for K in range(1,5):
  for L in range(1,9):
   F=T+L+K
   for P in range(L):
    for A in range(K):
     for U in range(T+1):
      free=F-P-A-U
      assert free>=2
      minimum=min(minimum,free);count+=1
r={'states':count,'minimum_free_fronts_at_eligible_idle_worker':minimum,
   'premises':'Fronts held only by P prepared blocks, A<=K-1 other service workers preparing, U<=T teams; pool F=T+L+K; P<L.',
   'not_proved':'Rust ownership, scheduling, cancellations, cache costs, or an end-to-end speed bound.'}
pathlib.Path(__file__).with_name('proofs.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps(r))
