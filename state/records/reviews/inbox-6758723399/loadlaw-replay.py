import json
# Match examples/coop.rs: sorted[n//2], the upper middle for even samples.
def median(values):
 values=sorted(values)
 return values[len(values)//2]
from pathlib import Path
rows=[json.loads(s) for s in Path("loadlaw-grid.jsonl").read_text().splitlines() if s.strip()]
rows=[r for r in rows if r["round"]>=0]
arms={a:[r for r in rows if r["arm"]==a] for a in sorted(set(r["arm"] for r in rows))}
med={a:median(r["mb_s"] for r in rr) for a,rr in arms.items()}
Ws=[1,2,4,5,6,8,10];p1=med["p1"];g=p1/med["u1"]
xs=[3.6*w for w in Ws];ys=[w*p1/med["p"+str(w)]-1 for w in Ws]
def X(x):
 if x<=xs[0]:return ys[0]
 for i in range(1,len(xs)):
  if x<=xs[i]:
   return ys[i-1]+(ys[i]-ys[i-1])*(x-xs[i-1])/(xs[i]-xs[i-1])
 raise ValueError("extrapolation")
u=[dict(W=w,measured=med["u"+str(w)],predicted=w*p1/(g+X(w*2.7)),ratio=med["u"+str(w)]/(w*p1/(g+X(w*2.7)))) for w in Ws]
c=[dict(W=w,L_MB=1.95*w,X=X(1.95*w),prediction=w*p1/(1.18+X(1.95*w))) for w in range(2,11)]
# Effective phase inflation, within each phase's same-harness one-worker boundary.
def phase_t(a,p):return median(r[p+"_s_a_worker"] for r in arms[a])
phase=[dict(W=w,build=w*phase_t("p"+str(w),"build")/phase_t("p1","build"),walk=w*phase_t("p"+str(w),"walk")/phase_t("p1","walk")) for w in Ws]
loo=[]
for i in range(1,len(xs)-1):
 pred=ys[i-1]+(ys[i+1]-ys[i-1])*(xs[i]-xs[i-1])/(xs[i+1]-xs[i-1])
 loo.append(dict(W=Ws[i],observed_X=ys[i],neighbor_interpolated_X=pred))
out=dict(rows=len(rows),rounds={a:len(rr) for a,rr in arms.items()},p1=p1,g_u=g,X=list(zip(xs,ys)),u=u,c30_assuming_g1_18_a1=c,production_phase_inflation=phase,leave_one_p_knot_out=loo,
g_from_CI=[1+.114/.7,1+.137/.7],
g7_to_tie={str(D):7*p1/D-X(7*1.95) for D in [2350,2430,2584]},
g7_sensitivity=[dict(g=gg,predicted=7*p1/(gg+X(7*1.95))) for gg in [1.0,1.18,1.3,1.5,2]],
schedule_cost={"original":61*12,"one_extra_level9_grid":61*5,"two_levels_extra_grid":61*10})
print(json.dumps(out,indent=2))
