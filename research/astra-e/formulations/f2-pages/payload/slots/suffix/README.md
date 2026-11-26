# Reuse an already spelled terminal suffix

The eight-slot F2 constructor publishes a word by traversing its contracted parent edges in reverse. Every completed, terminal ancestor owns a contiguous spelling of the entire remaining suffix. Node labels are assigned when its spelling begins. At the current node the label equals the current word count. An unassigned ancestor has sentinel u32::MAX. A prior terminal has a label strictly below the count and a complete dictionary entry. Copying that earlier range into the current suffix and ending the walk gives the same byte string. First-level scheduling assigns a child only after its parent publishes. The second-level traversal visits parents before children. This observation needs no extra persistent index or vector.

The experiment applies the test to first-level original-byte spelling, second-level token spelling, and both, keeping every published dictionary byte. Four arms produced the Segmented inverse on all 228 canonical admissions. Marked stable-sort and run-geometry checks passed. Per-block checks in the aggregate verified that all arms materialized the same total dictionary bytes at each level.

| Pooled count across 228 admissions | Original spans | Remaining original spans | Replacement tail copies | Bytes from cached tails | All bytes |
|---|---:|---:|---:|---:|---:|
| First-level edge slices | 3,466,864 | 2,394,767 | 500,503 | 1,256,860 | 23,393,313 |
| Second-level token slices | 4,418,276 | 4,415,048 | 3,177 | 6,753 | 41,433,719 |

After charging the replacement copies, 571,594 first-level splice operations disappear, about 16.5%. Their average copied tail is only 2.51 bytes. Almost all second-level spelling still needs its original token slices. In the two 64-lap balanced, pinned four-arm processes the complete inverse thread-CPU ratios against the unchanged preload were as follows.

| Variant | CPU 4 | CPU 2 |
|---|---|---|
| First only | 1.0261 [0.9924,1.0609] | 1.0089 [0.9818,1.0368] |
| Second only | 1.0181 [0.9767,1.0612] | 1.0021 [0.9704,1.0349] |
| Both | 1.0337 [0.9955,1.0733] | 1.0070 [0.9749,1.0400] |

First-only discovery cost 1.0973 [1.0450,1.1524] and 1.0115 [0.9689,1.0560]. The complete comparison provides no measured reason to enable these checks. The second-level test costs a lookup per visited ancestor with negligible reuse on this corpus. No whole-process RSS, independent CI, or workstation timing was collected.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/suffix
bash "$d/run.sh" /tmp/astra-e-terminal-suffix
python3 "$d/report.py" /tmp/astra-e-terminal-suffix/{first,repeat}.csv
```

The runner verifies the canonical corpus hash. Set PAGE_DEPS to the released moonshot deps from 252646a to match the measured library. The measured binary excludes all structural counters. The separate observation binary records counts.csv and checks every full inverse. Standalone scorer wrappers are first.rs, second-only.rs and both.rs.
