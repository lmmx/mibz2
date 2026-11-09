import json, pathlib
s=(pathlib.Path(__file__).parent/'ab.rs').read_text()
block=s[s.index('// task-clock-modes\n        {'):s.index('// inheritance\n        for')]
assert '.map_or(0, |_| u64::from_ne_bytes' in block
assert '"OBSERVED"' in block and 'let (nu, na) = (read(u), read(a));' in block
inh=s[s.index('// inheritance\n        for'):s.index('i32::from(fails > 0)')]
assert 'let tc = num(&f, "task_clock_s").unwrap_or(f64::NAN);' in inh
assert 'let ok = (0.090..0.130).contains(&tc);' in inh
record={'task_clock_s':0.1,'cycles':None,'cycles_why':'its enable failed (errno 5)','perf_ctl_errors':[{'counter':'cycles','op':'enable','errno':5}]}
# Exact acceptance inequality, modelled independently; no PMU/Rust execution.
print(json.dumps({'hardware_inheritance_counterexample':record,'current_predicate_passes':0.090 <= record['task_clock_s'] < 0.130,'mode_read_error_current_serialization':{'user_only_ns':0,'both_ns':0,'status':'OBSERVED'}},indent=2))
