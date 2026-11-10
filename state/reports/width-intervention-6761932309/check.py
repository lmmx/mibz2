import math,json
n=20
out={'dependence':[]}
for rho in [0,.5,.8]:
    d=1+2*sum((1-h/n)*rho**h for h in range(1,n))
    out['dependence'].append(dict(rho=rho,variance_multiplier=d,effective_n=n/d,oracle_iid_interval_coverage=math.erf(1.959963984540054/math.sqrt(2*d))))
out['additive_cost_interaction']=((.7929+.5)/(.6825+.5))/(.7929/.6825)
out['scenario_factors']=[((.11+.683*b)/(.1+.582*a))/(.7929/.6825) for a,b in [(1.12,1.65),(1.12,3.49),(1.30,6.27)]]
assert out['dependence'][1]['oracle_iid_interval_coverage']<.76
assert out['additive_cost_interaction']<1
print(json.dumps(out,indent=2))
