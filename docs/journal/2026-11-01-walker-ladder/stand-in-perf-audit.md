# Walker counter audit, with a stand-in perf (parsing check only; every value is fake)

Environment (numbers only): cpu_family 6, cpu_model 207, cpu_stepping 2, logical_cpus 4, physical_cores 4, threads_per_core 1, perf_event_paranoid 2, nmi_watchdog 0

| event | in perf list | status | counts on a run |
|---|---|---|---|
| `cycles` | True | counted | True |
| `instructions` | True | counted | True |
| `uops_issued.any` | True | counted | True |
| `uops_retired.retire_slots` | True | counted | True |
| `br_misp_retired.all_branches` | True | counted | True |
| `idq_uops_not_delivered.core` | True | <not supported> | False |
| `l1d_pend_miss.pending` | True | counted | True |
| `l1d_pend_miss.pending_cycles` | True | counted | True |
| `l1d_pend_miss.fb_full` | True | counted | True |
| `cycle_activity.stalls_l1d_miss` | True | counted | True |
| `cycle_activity.stalls_l2_miss` | True | counted | True |
| `cycle_activity.stalls_l3_miss` | True | counted | True |
| `cycle_activity.stalls_total` | True | counted | True |
| `resource_stalls.sb` | True | counted | True |
| `dtlb_load_misses.walk_active` | True | counted | True |
| `sw_prefetch_access.t0` | False | counted | True |
| `load_hit_pre.sw_pf` | True | counted | True |
| `mem_load_retired.l1_miss` | True | counted | True |
| `mem_load_retired.fb_hit` | True | counted | True |
| `mem_load_retired.l2_miss` | True | counted | True |
| `mem_load_retired.l3_miss` | True | counted | True |
| `resource_stalls.any` | True | counted | True |

| group | all counted | multiplexed |
|---|---|---|
| exec | False | False |
| mlp | True | False |
| stalls | True | False |
| pf_tlb | True | False |
| loads | True | True |
