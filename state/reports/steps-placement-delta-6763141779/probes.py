import hashlib,json,math,statistics

# Arithmetic witnesses, not decoder measurements.
t19=2.093024054408263
d=[0.04,0.06]*10
mean=statistics.mean(d);half=t19*statistics.stdev(d)/math.sqrt(20)
false_dependence=0.60;other=-0.55
assert math.isclose(false_dependence+other,mean)
required=0.51*(0.0476/0.0476)
assert mean+half<required<false_dependence
weights=[0.5,-0.5,-0.5,0.5]
assert sum(w*x for w,x in zip(weights,[1,2,3,4]))==0
# Linear wall-clock drift is not necessarily linear slot-index drift.
unequal_starts=[0,1,3,6]
residual=sum(w*x for w,x in zip(weights,unequal_starts))
assert residual==1
# Iid bounded block effects: common small values, rare large positive values.
# Probability that the entire run misses the rare component, despite iid blocks.
rare=0.01;population_mean=10*rare
low_component_bound=0.001*(1+t19/math.sqrt(19))
assert low_component_bound<population_mean
bank_a=0x100000;bank_b=0x300000
out={
 'source_sha256':hashlib.sha256(json.load(open('r73-draft.json')).encode()).hexdigest(),
 'cancellation':{'mean_net':mean,'halfwidth':half,'upper':mean+half,'required':required,'pair_component':false_dependence,'other_component':other},
 'linear_drift':{'equally_spaced_residual':0,'unequal_starts':unequal_starts,'unit_slope_residual':residual},
 'iid_non_normal':{'mixture':'0.99 Uniform[-.001,.001] + 0.01 point mass at 10','population_mean':population_mean,'all_low_probability':(1-rare)**20,'upper_bound_if_all_low':low_component_bound},
 'aslr':{'lo_offset':0,'hi_offset':2048,'absolute_cross_process_difference':bank_b+2048-bank_a},
 'precision_max_sample_sd':0.10*math.sqrt(20)/t19,
 'native_measurements':False
}
print(json.dumps(out,indent=2))
