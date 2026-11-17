# Independent baseline, round 2
Written before fetching or reading the revised post at 55b26bd7. This reviewer remembers round 1, so this is independence from the revised narrative, not a claim of an empty prior.

## Problem and evidence
Choose decoding workers, kernel and global in-flight block limit under the allowed CPU budget; more workers can increase cycles/byte enough to lose throughput. On the i9-10900K (10 physical cores, 20 logical, 20 MiB shared L3), measured choices are lazy7/28 at ten and past-core budgets, lazy6/24 at eight, lazy5/20 at six, and production eager4/16 at four. The low-count contrasts are whole-plan comparisons; lazy5/eager6 is not an isolated worker-count effect. Their direct paired comparisons are nevertheless valid plan evidence.

The existing shipped profile and a proposed transferable runtime rule are distinct. Admission on i9 does not demonstrate transfer to another cache topology or SMT arrangement. Cache discovery must intersect affinity and identify actual cache levels/domains.

## Model and its limits
Normalised c(W)=.422+.0068W² is relative to ten-worker production; a is an extrapolated intercept. At constant busy share the continuous optimum is 7.8777, and integer8 has .682% lower fitted wall than7. The measured winner7 is not obtained merely by flooring the minimiser. The corrected within1%-choose-smaller tie policy is fitted to these observations, not an independent prediction. For a+bW^p the contention share at continuous optimum is1/p, half only for p=2.

CPU supply is min(.93W,N-.16W-.125N); the separately estimated cache, CPU and serial limits compose a heuristic, not a solved joint optimum. Corrected cap uses logical N and per-domain N_d. At N4 its own formula selects3, disagreeing with measured eager4. Safe default must stay measured. Do not claim universal fit.

F~2.1MB is a calibrated reused hot footprint (~29.5bytes per letter), not total16.81MB buffers or measured temporal residency. -3/-9 total-buffer ratio .355 differs from letter/hot-footprint proxy .377; subsequent exponent fits must say which was used.

## Refutations and causal uncertainty
The -3 run keeps best7 rather than predicted9–10; the gain shrinks to .978 from .881. This rejects the simple homogeneity prediction; an interval spanning .98 is borderline for the separate threshold claim. A later serial ceiling is a hypothesis supported by a plateau and a flagged unpinned trace, not a clean timed occupancy measurement.

589 cycles per avoided LLC miss is an across-arm slope, not a measured individual DRAM penalty. Assumed80–100ns latency and predicted4–9GB/s are not measurements. Record51 adds derived hit91→65.2cycles and miss594→552cycles, while total IMC traffic is unavailable. These undermine a simple bandwidth story but do not identify a unique mechanism. Dividing mean miss latency by outstanding misses does not establish unhidden critical-path cost; overlapping stalls do not partition causal wall savings. W12 at seven has intervals spanning1, so its tiny point differences do not establish a gain. THP's ineffective intervention cannot prove all translation improvements useless.

## What should readers learn
1 Why fewer workers can finish sooner despite idle CPU.
2 How measurements, predictions and failure revise a model.
3 What is shipped, what is proposed, and what experiment permits transfer.
4 How to rerun paired, frozen comparisons with correctness, loss and memory checks.
5 Counts alone are insufficient: kernel and global window are part of the plan.
No detected slowdown is not a noninferiority proof. Percent less elapsed time is not the same number as percent more throughput.

Sources read: MONODROMY45–51 at c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e; LOG6945320222 and6945639925; worker calibration journal at55b26bd7. Its old pending status and superseded models are historical, not current authority.
