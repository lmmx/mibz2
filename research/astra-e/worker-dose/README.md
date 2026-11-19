# A fixed minority of eager workers in the edge decoder

The independent i9 leader admits five dense blocks to the combined writer. This experiment keeps that decoder and the existing shared work queue, assigning only the worker numbers divisible by four to eager Cells. The eager count is ⌈W/4⌉. The experiment changes the mix of block services while preserving worker and output-slot counts. [Target reference](../workstation/edge-i9.md) · [Frozen plan](PLAN.md)

## Sources and correctness

The pool creates persistent numbered threads and constructs its block scratch on each thread before fetching jobs. Each candidate reads that worker's index once and stores one eager or edge implementation in a Rust enum. The live heap for the unselected variant is empty. The enum's inline size can reach that of its largest variant, and individual high-water capacities depend on which blocks each worker receives. The bounded queue, scanner, output order, CRC combination, error cleanup and allocation routines are inherited from the measured workers. Both stock column paths and the validated run path delegate to the selected kernel. A constructor on a thread without the numbered pool name defaults to edge. The calling-thread simulator therefore cannot model the mixed pool. The selector depends on the frozen pool naming convention; the observed roster provides a regression check for future changes. [Candidate source](https://github.com/lmmx/mibz2/tree/77a3f8c6123bcb0b0c479789b46316da1778272a/research/astra-e/worker-dose) · [Existing pool launch and lifetime](https://github.com/lmmx/mibz2/blob/77a3f8c6123bcb0b0c479789b46316da1778272a/src/parallel.rs)

All 338 scorer checks passed, including Wikidata, real block and combined CRCs, multiple LF cycles, periodic inputs and mutated streams. The diagnostic executable recorded the exact worker roster and incoming run traffic in the scorer process at W1/2/4/7/8/9/10. The W10 observer assigned 108 of 383 run-route blocks to three eager members. Concurrency testing kept a W7 and a W10 pool alive in the same address space and verified both 1,272,843-byte outputs and all 17 numbered constructors. The timed executable compiles out diagnostic printing. [Scorer entry, exact build and all cases](../results/v1p2-dose-gate/) · [Worker roster and per-role rows](panel/roster.csv) · [Concurrent-pool check](concurrent.txt)

## What changes the prediction

On three fixed-affinity i9 runs with ten pairs per count, eager/Lazy wall moved from 0.911–0.931 at W4 to 1.027–1.038 at W8 and 1.105–1.121 at W10. In the final run, the sum of sampled worker CPU seconds averaged 0.383 versus 0.438 at W4, and 0.732 versus 0.667 at W10. The scheduler recorded per-thread CPU to 0.01 s. These are older one-copy runs with different build and affinity conditions from the four-copy unpinned edge sweep. The separate five-arm Xeon CI directly measured edge/eager wall at 1.253/1.219/1.053 on W1/2/4. On the eleven prespecified large admissions, 21 paired inverse-stage laps put eager at 0.705–0.900 of the combined/edge inverse CPU at the point estimates. Those eleven exclude the five extra dense blocks. This sample offers a shared-load hypothesis, with little evidence there for selecting the faster kernel by block alone. [Replayed i9 cells, source and archive hashes](historical.csv) · [Low-concurrency CI](../ci/evidence/edge-budget-44431030145.md) · [Local inverse comparisons](routing.csv)

For a long balanced workload, if eager and edge workers have constant independent per-worker service rates, the pool's service rate is the sum of those rates. To reach wall ratio 0.80 at the measured edge points while the other workers maintain their current rate, the eager subset must deliver 1.293/1.418/1.364/1.316 times that rate at W7/8/9/10. As a sensitivity check, substituting the favorable W1 full-process ratio 1.253 for this per-worker speed multiplier gives 0.809/0.831/0.827/0.814 of Lazy. A 5% paired reduction against edge would require multipliers 1.184/1.211/1.158/1.175. Scanner throughput, shared caches, finite queue effects and measurement conditions can change the actual result. [Model and exact inputs](ceiling.py) · [All values](ceiling.csv)

## Independent comparison

Twenty complete five-arm rotations on AMD EPYC 7763 verified a W2 improvement over the frozen edge source. One eager and one edge worker took **0.9095 [0.8977,0.9215]** of two edge workers' time, with 1.0789 times their median peak. Two homogeneous eager workers were faster still, with mixed/eager 1.0969 [1.0781,1.1160]. The W4 mixed/edge result was **0.9881 [0.9537,1.0238]** and median peak was 1.0701 of edge. W4 mixed/eager was 1.0457 [1.0208,1.0711]. Each of the two mixed counts used less peak memory than Lazy in all 20 pairs. W1 selected eager throughout and matched the independent master eager control within its interval. All five arms passed 338 correctness records. [CI report and paired archive](../ci/evidence/worker-dose-44448175373.md) · [Maximum peaks and pair counts](ci-replay/maxima.csv)

Mixed/harmonic was 1.0032 [0.9883,1.0184] at W2 and 1.0028 [0.9728,1.0337] at W4. The mixed kernel took longer than homogeneous eager in every one of the 40 W2/W4 pairs. The independently established W2 gain versus edge is consistent with combining different homogeneous service rates. The registered W7–10 i9 reference remains the dense-block edge writer. [Same-round service envelope](ci-replay/envelope.csv) · [Official scorer receipt](../results/v1p2-dose-gate/ci-receipt.json)

A second CI job independently rebuilt the same selected inputs on an AMD EPYC 7763 and completed another 300 samples. At W2 mixed/edge was **0.8861 [0.8714,0.9010]** and mixed/eager was **1.0891 [1.0770,1.1013]**. At W4, mixed/edge was **0.9876 [0.9836,0.9916]**, resolving a 1.24% improvement with median peak RSS 1.0578 times edge and 0.7750 times Lazy. Its W4 point agrees with the first CI's 0.9881, whose wider interval included parity. Across the two runs mixed beat homogeneous eager at W4 in only one of 40 rounds. In the replication W4 mixed/harmonic was 0.9956 [0.9892,1.0021]. Its W1 comparison between the two eager implementations was 0.9764 [0.9548,0.9984], demonstrating a small single-worker build/measurement contrast. Each run's raw rows, baselines, scorer and intervals remain separate. [Replication and archive](../ci/evidence/worker-dose-44448785057.md) · [Compilation-input audit](replication-audit.json) · [Replication peak/CPU](ci-replication/maxima.csv)

## Complete-worker local result

The 16-round schedule produced 128 measured processes on the four-CPU Xeon quota. Each process used 4W slots and decoded two corpus copies, 419,400,392 bytes. The eager and Lazy cells and the edge control passed the 98-case control gates, while the mixed candidate passed its independent 338-case gate. Matched round ratios use the scorer's 95% Student interval on the log scale. [All raw rows](panel/runs.jsonl) · [Direct wall, CPU, memory and maximum peaks](panel/comparison.csv)

| W | Mixed / edge wall [95%] | Mixed / eager wall [95%] | Mixed / Lazy wall [95%] | Median peak / edge, eager, Lazy | Mixed / constant-rate wall [95%] |
|---:|---|---|---|---|---|
| 2 | 0.8857 [0.6852,1.1448] | 1.0850 [0.6680,1.7622] | 0.8749 [0.5461,1.4014] | 1.0829, 0.8748, 0.8809 | 1.0374 [0.6919,1.5554] |
| 4 | 0.7854 [0.4585,1.3456] | 0.9988 [0.8332,1.1973] | 0.8497 [0.7095,1.0175] | 1.0546, 0.7709, 0.7669 | 0.9118 [0.6851,1.2135] |

The constant-rate column uses the harmonic blend of the two homogeneous **wall times in that same round**. It assumes the delivered per-worker rates stay constant when workers share the pool. Scanner and ordered-delivery bottlenecks, OS scheduling, shared caches and finite job placement can affect this diagnostic. [Model replay](interaction.py) · [All derived interaction cells](panel/interaction.csv) · [Envelope with a shared additive or overlapped scanner](serial.md)

The preregistered mixed/edge speed condition did not pass at either worker count. All 16 pairs and both complete-worker outputs remain in the result. Round 14 recorded 16.21 s for W2 mixed and 32.58 s for W4 edge. In W4 that latter sample alone produced a ratio of 0.02384; the median of all 16 pair ratios was 1.0100 and mixed finished faster in six. W2 mixed finished faster in ten of 16 pairs and its median matched ratio was 0.8632. Removing each possible single W2 round in turn gives geomeans from 0.8288 to 0.9357, still with unresolved full-sample intervals. Median mixed RSS exceeded edge by 8.29%/5.46% at W2/W4, while the observed maxima and paired upper RSS limits stayed below Lazy in both cells. The current target reference remains edge at `badb2ae`. A five-arm full-worker CI against that frozen edge source, eager, opend and Lazy is the appropriate independent test of the W2 direction and crossover if this dose is taken further. The local evidence provides no resolved W4 advantage against either homogeneous kernel and no independent basis to queue an i9 sweep. [Sensitivity for all arms](panel/sensitivity.csv) · [Original receipt and scope](panel/manifest.txt)

## Reproduction

The immutable kernel build is `77a3f8c6123bcb0b0c479789b46316da1778272a`. The panel driver is `569a091`. In a clone containing the research history, use a separate source checkout for the historical decoder. The runner refreshes the scorer from master and writes its exact selected hash before building edge, dose and the diagnostic executable. The observed panel used upstream path `75297a1d57b2bdf262c3a19f3064848dedf2c493`, printed v1 at `81585172f5fb9e7a59f330124779ba084fd6bf04`. Each original binary digest and the immutable compressed corpus digest are in the manifest. [Build manifest](panel/manifest.txt)

```sh
dose_driver="$(pwd)/research/astra-e/worker-dose"
git worktree add --detach ../astra-worker-dose 77a3f8c6123bcb0b0c479789b46316da1778272a
cd ../astra-worker-dose
bash bench/moonshot/corpus.sh
bash "$dose_driver/run.sh" corpora/wikidata.bz2 /tmp/astra-dose-new-results /tmp/astra-dose-new-build
```


The raw panel replays offline from any checkout with the frozen scorer Python module.

```sh
d=research/astra-e/worker-dose
python3 "$d/panel.py" /dev/null /dev/null /dev/null "$d/panel" --replay
python3 "$d/interaction.py" "$d/panel/runs.jsonl" | diff - "$d/panel/interaction.csv"
python3 "$d/sensitivity.py" "$d/panel/runs.jsonl" | diff - "$d/panel/sensitivity.csv"
python3 "$d/check_roster.py" "$d/panel" | diff - "$d/panel/roster.csv"
```
