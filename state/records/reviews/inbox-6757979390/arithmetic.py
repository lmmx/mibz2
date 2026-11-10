import json
n=209700196
d={
 "units":"Arithmetic scenarios only; no candidate executed and no new timing.",
 "copy":{"output_bytes":n,"logical_copy_read_plus_write":2*n,
 "invalid_bytes_over_Dw_pipe3":n/1182227023,
 "reported_copy_Ir_share":1.3/26.7,
 "ideal_instruction_speedup_if_entire_term_removed":26.7/(26.7-1.3),
 "instruction_based_consumer_ceiling_at_3GHz_4Ir":3e9*4/1.3},
 "pair":{"rows":900000,"old_zero_plus_overlap_store_bytes":13*900000,
 "new_split_layout_fully_initialised_store_bytes":5*900000,
 "final_table_bytes":5*900000,
 "ideal_build_ratio_breakeven":{k:1+0.5*w/b for k,b,w in [("wikidata",1.75,6.54),("text",2.50,7.68),("records",2.96,4.63)]},
 "text_ideal_decode_reduction_if_build_2x_walk_half":(0.5*7.68-2.50)/20.30},
 "scanner":{"text_share":3.17/20.30,"hypothetical_1p5x_stage_speedup_whole_speedup":1/(1-(3.17/20.30)*(1-1/1.5))},
 "phrase":{"first_only_events":0.1050,"nearer_events":0.0972,"relative_event_increase":0.105/0.0972-1,
 "first_only_40Ir_event_cost_per_row":0.105*40,
 "first_only_100Ir_event_cost_per_row":0.105*100,
 "naive_150Ir_event_cost_per_row":0.105*150},
 "contraction":{"one_round_removed_fraction":[1/3,1/2],"cost_condition":"build_per_remaining_row + expansion_cost < removed_fraction * walk_per_remaining_row; phase assignment and traffic included"},
 "mtf":{"K4_initial_256_entry_lists":4*256,"three_prefix_compositions_entries":3*256,
 "text_original_ideal_total_fraction_saved":(5.47*(1-1/4))/20.30,
 "text_emitter_model_ideal_remaining_stage_saving":1.8*(1-1/4)}
}
print(json.dumps(d,indent=2))
