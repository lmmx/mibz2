"""Untimed review probes; run beside the reviewed bw_titrate.py and test_bw_titrate.py."""
import json, tempfile
from pathlib import Path
import test_bw_titrate as t

out = {}
with tempfile.TemporaryDirectory() as d:
    p = Path(d)
    for f in (t.test_readings, t.test_figures, t.test_admission):
        f(p)
    out['existing_tests'] = '3 passed'
    for name, mutate in (
        ('negative_zero_thread_bytes', lambda rs: [r.update(bytes=-1) for r in rs if r['threads']==0]),
        ('uniform_wrong_size', lambda rs: [r.update(bytes=1) for r in rs if r['threads']>0]),
    ):
        r,v=t.run(p,t.grid(mutate=mutate))
        assert r.returncode==0
        out[name]={'exit':r.returncode,'reading':v['reading']}
# Old/new census predicates on the independently derived aab BWT witness.
traj=[0,1,2]; cell=[0,0,1]; q,t0=0,1
old=new=0
while t0+old<3 and traj[t0+old]-traj[q+old]==1: old+=1
while t0+new<3 and cell[traj[t0+new]]==cell[traj[q+new]]: new+=1
assert (old,new)==(2,1)
out['aab_certificate']={'old':old,'revised':new,'native_rust_executed':False}
# Under the ideal fixed-W model, near-zero sensitivity rejects only sufficient dose.
W,H=28.5,8
out['fixed_capacity_saturated_model']={'W_GBs':W,'H_GBs':H,'predicted_ratio':1-H/W}
# Removing a core can move an originally saturated decoder below its bandwidth ceiling.
bandwidth_capacity=1.5; reduced_cpu_capacity=1.0; beta=19; h=8
out['baseline_bottleneck_shift']={'original_GBs':bandwidth_capacity,
 'hog_spin_baseline_GBs':reduced_cpu_capacity,
 'hog_store_GBs':min(reduced_cpu_capacity,(28.5-h)/beta),
 'ratio':min(reduced_cpu_capacity,(28.5-h)/beta)/reduced_cpu_capacity}
print(json.dumps(out,indent=2))
