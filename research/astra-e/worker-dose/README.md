# A fixed minority of eager workers in the edge decoder

The independent i9 leader admits five dense blocks to the combined writer. This experiment keeps that decoder and the existing shared work queue, assigning only the worker numbers divisible by four to eager Cells. The eager count is ⌈W/4⌉. The experiment changes the mix of block services while preserving worker and output-slot counts. [Target reference](../workstation/edge-i9.md) · [Frozen plan](PLAN.md)

## Sources and correctness

The pool creates persistent numbered threads and constructs its block scratch on each thread before fetching jobs. Each candidate reads that worker's index once and stores one eager or edge implementation in a Rust enum. The live heap for the unselected variant is empty. The enum's inline size can reach that of its largest variant, and individual high-water capacities depend on which blocks each worker receives. The bounded queue, scanner, output order, CRC combination, error cleanup and allocation routines are inherited from the measured workers. Both stock column paths and the validated run path delegate to the selected kernel. A non-worker constructor defaults to edge, so a calling-thread simulator does not model the mixed pool. [Candidate source](https://github.com/lmmx/mibz2/tree/77a3f8c6123bcb0b0c479789b46316da1778272a/research/astra-e/worker-dose) · [Existing pool launch and lifetime](https://github.com/lmmx/mibz2/blob/77a3f8c6123bcb0b0c479789b46316da1778272a/src/parallel.rs)

All 338 scorer checks passed, including Wikidata, real block and combined CRCs, multiple LF cycles, periodic inputs and mutated streams. The diagnostic executable recorded the exact worker roster and incoming run traffic in the scorer process at W1/2/4/7/8/9/10. The W10 observer assigned 108 of 383 run-route blocks to three eager members. Concurrency testing kept a W7 and a W10 pool alive in the same address space and verified both 1,272,843-byte outputs and all 17 numbered constructors. The timed executable compiles out diagnostic printing. [Scorer entry, exact build and all cases](../results/v1p2-dose-gate/) · [Worker roster and per-role rows](panel/roster.csv) · [Concurrent-pool check](concurrent.txt)

## What changes the prediction

On three fixed-affinity i9 runs with ten pairs per count, eager/Lazy wall moved from 0.911–0.931 at W4 to 1.027–1.038 at W8 and 1.105–1.121 at W10. In the final run, the sum of sampled worker CPU seconds averaged 0.383 versus 0.438 at W4, and 0.732 versus 0.667 at W10. The scheduler recorded per-thread CPU to 0.01 s. These are older one-copy runs with different build and affinity conditions from the four-copy unpinned edge sweep. The separate five-arm Xeon CI directly measured edge/eager wall at 1.253/1.219/1.053 on W1/2/4. On the eleven prespecified large admissions, 21 paired inverse-stage laps put eager at 0.705–0.900 of the combined/edge inverse CPU at the point estimates. Those eleven exclude the five extra dense blocks. This sample offers a shared-load hypothesis, with little evidence there for selecting the faster kernel by block alone. [Replayed i9 cells, source and archive hashes](historical.csv) · [Low-concurrency CI](../ci/evidence/edge-budget-44431030145.md) · [Local inverse comparisons](routing.csv)

For a long balanced workload, if eager and edge workers have constant independent per-worker service rates, the pool's service rate is the sum of those rates. To reach wall ratio 0.80 at the measured edge points while the other workers maintain their current rate, the eager subset must deliver 1.293/1.418/1.364/1.316 times that rate at W7/8/9/10. As a sensitivity check, substituting the favorable W1 full-process ratio 1.253 for this per-worker speed multiplier gives 0.809/0.831/0.827/0.814 of Lazy. A 5% paired reduction against edge would require multipliers 1.184/1.211/1.158/1.175. Scanner throughput, shared caches, finite queue effects and measurement conditions can change the actual result. [Model and exact inputs](ceiling.py) · [All values](ceiling.csv)

## Reproduction

The immutable kernel build is `77a3f8c6123bcb0b0c479789b46316da1778272a`. The panel driver is `569a091`. In a clone containing the research history, use a separate source checkout for the historical decoder. The runner refreshes the scorer from master and writes its exact selected hash before building edge, dose and the diagnostic executable. The observed panel used upstream path `75297a1d57b2bdf262c3a19f3064848dedf2c493`, printed v1 at `81585172f5fb9e7a59f330124779ba084fd6bf04`. Each original binary digest and the immutable compressed corpus digest are in the manifest. [Build manifest](panel/manifest.txt)

```sh
dose_driver="$(pwd)/research/astra-e/worker-dose"
git worktree add --detach ../astra-worker-dose 77a3f8c6123bcb0b0c479789b46316da1778272a
cd ../astra-worker-dose
bash bench/moonshot/corpus.sh
bash "$dose_driver/run.sh" corpora/wikidata.bz2 /tmp/astra-dose-new-results /tmp/astra-dose-new-build
```
