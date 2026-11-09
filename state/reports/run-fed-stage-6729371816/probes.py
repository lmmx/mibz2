"""Analytical counterexamples to source predicates; not execution of the Rust diagnostic.
No benchmark, decoder, unsafe witness, or workflow is executed.
"""
import json, math

def q1(w, synth):
    g,lo,hi=w
    # Transcription of filter_map + fold in run_fed_stage.rs:464-470.
    worst=max([0.0]+[r[0] for r in synth if r is not None])
    if g<=.90 and hi<1 and worst<=1.02:return 'FEASIBLE'
    if lo>.95:return 'STOP: no >=5% stage saving'
    if worst>1.02:return 'STOP: fallback label'
    return 'INCONCLUSIVE'

out={}
out['missing_synthetic_intervals']={'reading':q1((.89,.80,.99),[None]*6),'meaning':'All six absent intervals are silently omitted by the max predicate.'}
assert out['missing_synthetic_intervals']['reading']=='FEASIBLE'
out['uncertain_synthetic_cost']={'reading':q1((.89,.80,.99),[(1.01,.80,1.30)]*6),'meaning':'Point estimates pass although each upper bound permits >2% harm.'}
out['fallback_label_counterexample']={'A_accepted':1,'B_accepted':1.20,'A_fallback':1,'B_fallback':1,'R_stage':2.20/2,'R_fb':1.0,'reading':q1((.89,.80,.99),[(1.10,1.01,1.19)]),'meaning':'Whole-corpus regression can be entirely on accepted run-path blocks, with zero fallback regression.'}
assert out['fallback_label_counterexample']['reading']=='STOP: fallback label'
out['nominal_cycles']={'seconds':1e-6,'cells':100,'GHZ_header':2.8,'actual_GHZ_example':3.5,'displayed_cycles_per_cell':1e-6*2.8e9/100,'actual_cycles_per_cell_if_constant':1e-6*3.5e9/100,'meaning':'Displayed M3 uses wall seconds times a sampled/default frequency, not PMU cycles.'}
out['overshoot_bound']={'cell_lengths':[1,1,1],'n':3,'cells':3,'stored_u32_slots':sum(8*math.ceil(x/8) for x in [1,1,1]),'unique_table_slots':3,'meaning':'Each short cell emits a whole AVX2 store; identical final 4n-byte tables need not have identical store work.'}
assert out['overshoot_bound']['stored_u32_slots']==24
out['modelled_instruction_ratio']=2.5/2.6
print(json.dumps(out,indent=2))
