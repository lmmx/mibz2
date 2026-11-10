"""Exact binomial policy checks and constructed risk witnesses; no measured runs."""
import json, math
from pathlib import Path
from scipy.stats import t

def tail(n, r, p):
    return math.fsum(math.comb(n,j)*p**j*(1-p)**(n-j) for j in range(r,n+1))

size=tail(24,17,.5)
power=tail(24,17,.9)
out={'sign': {'k':24,'threshold':17,'null_size':size,'individual_power_p09':power,
              'all_42_union_lower_power_p09':max(0,1-42*(1-power))}, 'mean_t_witness':{}}
for n in (12,24,56,64,128,256):
    critical=float(t.ppf(.95,n-1))
    admitted=[m for m in range(n+1) if -1+19*m/n+critical*19*math.sqrt(m*(n-m)/(n*n*(n-1)))<=0]
    prob=math.fsum(math.comb(n,m)*(1/19)**m*(18/19)**(n-m) for m in admitted)
    out['mean_t_witness'][n]={'accepted_rare_counts':admitted,'probability':prob}

# All 42 cells share the same draw; this is allowed cross-cell dependence.
# 90% of dispatches have ratio .999, 10% ratio 10. Sign success is .9,
# while both expected log ratio and arithmetic expected ratio show material harm.
ratio_good=.999; ratio_bad=10
out['default_risk_witness']={
    'good_ratio':ratio_good,'good_probability':.9,'bad_ratio':ratio_bad,'bad_probability':.1,
    'geometric_population_ratio':math.exp(.9*math.log(ratio_good)+.1*math.log(ratio_bad)),
    'arithmetic_population_ratio':.9*ratio_good+.1*ratio_bad,
    'all42_pass_probability_identical_cells':power,
    'two_independent_programmes_both_sign_pass_probability':power**2,
    'hypothetical_sign_harm_flag_probability_17of24_above1':tail(24,17,.1),
    'joint_primary_sign_pass_and_harm_flag_probability':0,
}
# The two events are mutually exclusive: primary pass requires >=17 good draws,
# a sign harm flag requires >=17 bad draws, and there are only 24 draws.
assert size < .05 and abs(size-.03195732831954956)<1e-14
assert out['default_risk_witness']['geometric_population_ratio']>1.25
assert max(0,1-42*(1-power))>.929
out['rare_tail_missing_probability']={'q01_k24':.99**24,'q01_k48':.99**48}
out['cost']={'primary_runner_minutes_at_stated_pace':2*2583,
             'with_full_confirmation_runner_minutes':4*2583,
             'primary_shard_jobs':24*9,'with_confirmation_shard_jobs':48*9,
             'all_valid_q002_primary':.998**216,'all_valid_q002_both':.998**432,
             'all_valid_q01_primary':.99**216,'all_valid_q01_both':.99**432}
Path('d095-policy-results.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
