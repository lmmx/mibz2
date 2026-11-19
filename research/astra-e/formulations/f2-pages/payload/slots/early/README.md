# Start survivor rank work before publishing retired lanes

When a wave retires jobs, every surviving job's next row was already computed from its immutable response. Child creation and dictionary publication can alter newly assigned jobs only. In the original 6758831 eight-slot run, 736,520 waves retired at least one job. Summing (active - retired) over those waves gives 4,897,286 next lookups (24.80% of all 19,750,839) that could move before publication. The original wave-count trace permits this calculation without a cache model.

The directory-hint variant prefetches both rank arrays for those next rows and uses the normal lookup when each row next advances. The early-index variant computes the actual rank before publication and saves it in that survivor's existing index slot. A ready bit excludes that lookup from the next wave. Newly assigned lanes always perform their normal lookup. A retiring job's index stays intact until its selectors are published. The third variant also issues a T0 hint for the compact payload when an early index becomes available. These alternative schedules retain the original heap capacities and byte output. On non-x86-64 targets the hint helper is empty. Moving a rank query preserves its result, but decreases independent work in the eight-lane lookup burst and adds live-mask bookkeeping.

The forced short checks applied all three policies. Every result on the 228 real admitted blocks matched Segmented. Two four-arm, 64-lap, position-balanced processes checked all nine systematic real block outputs. The table shows complete thread CPU relative to the frozen compact preload, with all recognition and publication costs included.

| Variant | CPU4 complete | CPU2 complete | CPU4 discovery | CPU2 discovery |
| --- | ---: | ---: | ---: | ---: |
| Directory hints | 1.0056 [0.9233,1.0952] | 0.9983 [0.9561,1.0423] | 1.0986 | 1.0128 |
| Compute early | 0.9893 [0.9278,1.0548] | 1.0039 [0.9728,1.0360] | 1.0953 [1.0144,1.1826] | 1.0519 [1.0079,1.0979] |
| Compute early and hint payload | 0.9388 [0.8672,1.0163] | 0.9861 [0.9497,1.0238] | 0.9914 | 1.0390 |

Computing indices early increased measured discovery cost on both pins. Adding the payload hint recovered some cost with no resolved complete advantage relative to the original. Every complete interval versus the frozen control spans parity. These data do not support promotion. Each reference query still paid its normal directory construction and payload consumption.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/early
bash "$d/run.sh" /tmp/astra-e-early-ranks
python3 "$d/report.py" /tmp/astra-e-early-ranks/{first,repeat}.csv
```
