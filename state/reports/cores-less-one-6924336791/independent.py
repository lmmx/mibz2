import json, math
import two_arm as ta

# Analytic Student-t CDF for df=1, independent of incomplete-beta implementation.
errs = [abs(ta.t_cdf(t,1)-(.5+math.atan(t)/math.pi)) for t in [-20,-5,-1,-.1,0,.1,1,5,20]]
assert max(errs)<1e-12
assert ta.holm({12:.017,16:.02,20:.5},.05)==set()
assert ta.holm({12:.016,16:.025,20:.05},.05)=={12,16,20}
rows=[]
for phys,t in [(8,16),(9,18),(10,12),(10,16),(10,20)]:
    old=phys
    new=phys-1 if phys>8 else phys
    rows.append(dict(physical_cores=phys,requested=t,old_workers=old,new_workers=new,
                     old_slots=4*t,new_slots=max((4*t*new)//t,new)))
print(json.dumps(dict(cauchy_cdf_max_error=max(errs),holm_boundary_checks='pass',
    control_false_void_independent_null=1-.95**4,topology=rows),indent=2))
