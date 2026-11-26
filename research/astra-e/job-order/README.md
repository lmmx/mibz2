# Can choosing a different complete block move the target?

The scanner and workers already had enough queued jobs to change hundreds of dispatch decisions on the W4 corpus. Choosing the longest available block by its symbol count **increased** median modeled makespan by 1.63% on the worker CPU shapes. Combining predicted service with the job's publication deadline gave **0.9999× FIFO** on those eight traces. The W7–10 models leave 4.19, 3.76, 1.22 and 0.65 ms between FIFO and a lower bound on *every* order in the central median-shape replay. The current i9 scalar worker needs approximately 22.15, 22.78, 22.15 and 16.66 ms on the same four-copy input to reach the stretch target. None of the evaluated policies has a consistent, sufficiently large benefit to justify a worker change on this evidence. No worker or allocator changed. [Local results](local.csv) · [Deadline-based local results](slack-local.csv) · [Target source data](rates.csv)

## What a worker can choose

The source scanner parses in order and takes a slot before reading a block. It learns EOB, the full symbol vector and its length before enqueuing the complete job. All workers receive from a single FIFO, finish MTF, inverse, RLE1 and block CRC on the same thread, and insert output or error into the ordered results map. Taking ordinal i frees a slot before its bytes finish copying to the caller. The worker keeps its own decode scratch. The queue path in the verified scalar source and release 0.5.6 has the same Git blob, `f9ab5e0b8f14b91d88e747f642fda8742be5f47e`. The earlier budget modeled advancing the release of a FIFO job; this experiment changes the order only after enqueue. [Frozen pipeline](https://github.com/lmmx/mibz2/blob/d0a42980e896211906234c7679b38760ae44a46e/src/parallel.rs) · [Earlier overlap analysis, f17e2101](../budget/overlap/README.md)

A conforming choice cannot split a block, start before EOB, increase the 4W cap, free a slot on worker completion or bypass an earlier result. Concatenated streams reset level and combined CRC in the scanner; each block's expected CRC, bit address and ordinal travel with the job. Scanner errors occupy the next ordered position after already issued blocks. Worker failures and cancellation must retain the original ordered-read behavior and termination semantics. Per-worker scratch resets after a panic. Bounded slots give bounded admission, while choosing jobs can still change which workers grow scratch, when the output and symbol buffer pools see large allocations, and peak RSS. This offline valid-input schedule changes no error behavior and measures no RSS. [Scanner contract](https://github.com/lmmx/mibz2/blob/d0a42980e896211906234c7679b38760ae44a46e/src/stream.rs)

## A lower bound for any job order

Let nonnegative sᵢ, dᵢ and cᵢ be fixed scan, worker and ordered reader service for job i. A worker may choose a fully enqueued job whenever it becomes available. Let Eᵢ be a lower bound on its enqueue time, Tᵢ a lower bound on its ordered take, and K=4W. Missing predecessors are zero. Induction in ordinal order gives

```text
Eᵢ = max(Eᵢ₋₁, Tᵢ₋K) + sᵢ
Tᵢ = max(Tᵢ₋₁ + cᵢ₋₁, Eᵢ + dᵢ,
         max over 0 ≤ j ≤ i of Eⱼ + (dⱼ + ... + dᵢ) / W)
makespan ≥ T_last + c_last
```

All jobs j through i must be complete before i can be taken. None can start before Eⱼ, and W workers can provide at most W units of combined service per unit elapsed time. The second term also prevents concentrating one unsplittable job on several workers. Ordered reads and slot return yield the first two recurrences. Each quantity on the right is itself a valid lower bound on the corresponding actual time. This proof applies even to a clairvoyant schedule that intentionally idles. A prefix sum and a running maximum evaluate all j in linear time, with units scaled by W to avoid downward rounding errors. The result is a lower bound, which can be strictly earlier than an attainable schedule. FIFO is separately calculated by an earliest-free-worker recurrence, then reproduced exactly by the independent event scheduler in all 16 W4 and 180 target cases. Brute force over 23,040 six-job priority schedules covered both improvements and regressions, attaining the floor on 14 of 32 instances. [Simulator and bound](model.py) · [Finite verification](verify.txt)

## W4 stock scanner and scalar worker

Eight archived level-2 timelines used the unchanged scanner and frozen scalar worker on 862 whole blocks (two Wikidata copies) per run. Scan service uses SlotGot to Enqueued and scanner loop housekeeping, including elapsed input waiting. Reader service uses Taken to the following ReadWait. Worker CPU uses DecodeStart to DecodeEnd on the thread CPU clock. The separate elapsed model uses JobGot to Ready, including descheduling and publishing. Early wake-up/queue residuals, extra locks, feeder contention, variable frequency and changed memory behavior are outside the model. Observed median time from first SlotGot to final ReadWait was 1.168 s, against 0.800 s modeled FIFO with CPU service or 1.061 s with elapsed service. The two service models have different meanings. [Original 862-block traces](../huffman-phrases/integrated/trace/) · [Instrumented experiment](../huffman-phrases/integrated/README.md) · [Hashes](manifest.json)

| Rule / FIFO | W4 CPU shapes, median [range] | W4 elapsed shapes, median [range] |
|---|---:|---:|
| Largest true service, oracle | 1.0109 [0.9945, 1.0641] | 1.0004 [0.9939, 1.0411] |
| Largest symbol count | 1.0163 [0.9999, 1.0570] | 1.0144 [0.9980, 1.0576] |
| Largest symbols among four oldest | 1.0086 [1.0017, 1.0331] | 1.0056 [0.9956, 1.0233] |
| Earliest predicted latest-start, true service oracle | 0.9915 [0.9858, 1.0178] | 0.9851 [0.9745, 0.9980] |
| Earliest predicted latest-start, known symbols | 0.9999 [0.9988, 1.0102] | 0.9969 [0.9910, 1.0046] |

The latest-start rule scores ordinal minus W times its service estimate divided by the arithmetic mean observed up to the current enqueue. Only completed jobs enter its choice. It broke FIFO at a median of 275 and 307 dispatches on the respective W4 models. The median all-order saving bound was 2.87% with CPU service and 4.30% with elapsed service, reaching 5.02% and 7.02% on their most permissive samples. The decision rule using true elapsed service has access to future descheduling. Its favorable W4 direction cannot be transferred to the known-symbol worker. [Registered longest-first rules](PLAN.md) · [Registered latest-start follow-up](SLACK.md)

## Conditional target projection

The original i9 role sweep recorded Lazy scanner/wall fractions .720, .755, .773 and .778 and average worker CPU/W/wall .904, .891, .870 and .826. The scalar i9 receipt measured complete wall/CPU/RSS and did not record a per-block timeline. For each W, scale the old Lazy scanner, main and non-scanner/non-worker residual by the ratio of the two Lazy median wall times. Subtract the scaled scanner and residual from measured scalar CPU to obtain modeled worker demand. Charging all main CPU as ordered copy is another transfer assumption. The scanner and copy shapes are the medians of the prior isolated measurements of each of the 431 blocks. The central worker shape is each block's median over both copies of all eight scalar W4 CPU traces. Repeat in source order four times. Eight additional cases use unaggregated W4 worker shapes individually, and four sensitivity cases separately rescale scan or workers by ±20%. No target scalar service trace supports more precise attribution. [Original role measurements](../budget/target.md) · [Verified scalar measurement](../workstation/scalar-independent-i9.md) · [Calibration](rates.csv)

| W | Measured scalar ms | Modeled FIFO ms | All-order lower bound ms | Max removable ms | Symbol latest-start / FIFO | Max removable ms across all 45 shape/rate cases |
|---:|---:|---:|---:|---:|---:|---:|
| 7 | 269.7 | 244.71 | 240.52 | 4.19 | 0.9973 | 16.74 |
| 8 | 263.4 | 240.78 | 237.02 | 3.76 | 0.9906 | 14.17 |
| 9 | 266.9 | 245.00 | 243.78 | 1.22 | 0.9976 | 8.57 |
| 10 | 275.2 | 256.05 | 255.41 | 0.65 | 1.0003 | 9.75 |

Across the eight separate unaggregated W4 shapes at central transport, the median known-symbol latest-start ratios were 0.99997, 0.99998, 1.00000 and 0.99996. The model omits 19–25 ms of the observed scalar wall in the central shape and gives no evidence assigning those milliseconds to job order. An independent per-block i9 trace showing a reproducible FIFO-specific delay, followed by a same-source uninstrumented direct worker comparison against scalar with identical slots and peak-memory checks, would change this decision. The current evidence favors keeping FIFO. [All modeled target schedules](target.csv) · [Latest-start target schedules](slack-target.csv)

From this directory, the following commands reproduce the complete bounded simulations using only tracked data. The paired worker scorer is reserved for actual candidate executables.

```sh
out=$(mktemp -d)
python3 -B verify.py | cmp - verify.txt
python3 -B study.py "$out/base"
python3 -B study.py "$out/slack" --slack
for file in local.csv target.csv rates.csv manifest.json; do cmp "$out/base/$file" "$file"; done
cmp "$out/slack/local.csv" slack-local.csv
cmp "$out/slack/target.csv" slack-target.csv
```
