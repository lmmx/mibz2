# Cost of the independently measured combined writer

The historical combined worker at 80d8409359214b4e5abfd4430213921e82e9cd30 reached wall ratios of .8982, .9117, .9031 and .8871 against LazyCells at W7–10 on the i9. This leaves 9.82–12.25% of its own elapsed time to remove. On the local Xeon the two largest paid inverse stages were the initial scan and retained-row queues (32.09% of inverse CPU on eleven blocks) and the residual emitter (38.39%). On the frozen block those two stages accounted for 42.8% and 27.8% of measured instructions. [Independent i9 receipt](../../workstation/combined-i9.md) · [Phase statistics](summary.txt) · [Raw phases](phases.csv)

The timing scope starts on already constructed run records at the actual combined endpoint and ends after all marked inverse bytes are ready. We rebuilt the frozen source, inserted eight stage boundaries in a temporary clone, and alternated it with the untouched original on the same eleven prespecified large blocks as the earlier 21-round study. Each of the 24 rounds contains both orders across blocks and across rounds every block gets 12 of each order. All 228 admissions first matched the released LazyCells inverse and the original constructor's result. Every timed output matched the independent saved inverse. The previous matched complete stage measured combined/opend thread CPU .9507 [.9171,.9854] on this same sample. Its controls include eager and Lazy. [Earlier stage receipt](../README.md) · [All 228 geometries and retained capacities](geometry.csv)

| Exact phase | Median summed CPU ms, 11 blocks | Median share per round | Frozen instructions | Frozen Ir share |
|---|---:|---:|---:|---:|
| Coalesce caps, classify widths, fill per-byte retained queues and thick-strip descriptors | 10.147 | 32.09% | 4,308,332 | 42.8% |
| Compute bucket prefixes and thick-strip image order | .109 | .37% | 57,680 | .6% |
| Initialize both endpoint lists and their gap nodes | 1.089 | 3.47% | 636,980 | 6.3% |
| Eliminate right endpoints and construct immediate words / grammar rules | 3.126 | 10.35% | 1,170,344 | 11.6% |
| Enumerate surviving bottom positions | .133 | .44% | 47,447 | .5% |
| Enumerate top bands, materialize words and publish offsets | 2.509 | 8.35% | 681,498 | 6.8% |
| Populate packed edges by projecting each band's queue slice | 1.544 | 5.11% | 367,179 | 3.6% |
| Follow the marked residual cycle and emit all inverse bytes | 11.948 | 38.39% | 2,803,424 | 27.8% |

The CPU times and shares are independently calculated medians and may not sum exactly. Stage proportions varied materially across rounds. The complete instrumented/untouched ratio was 1.0505 [1.0148,1.0874], using a paired log-ratio interval over the 24 complete sample rounds. Instruction counts come from eight Callgrind client-request dumps on warmed frozen block 1. These include boundary-call overhead. The CPU and instruction columns differ in input population and measurement mechanism; neither yields target IPC or target wall fractions. The existing run-span construction made the queue scan cheaper and the later projection dearer. Lower queue capacity alone did not improve its complete inverse. [Callgrind raw dumps](callgrind-baseline.tar.xz) · [Earlier queue alternatives](../../queue-spans/README.md)

## Geometry, array widths and remaining headroom

Across 228 admissions the constructor read 14,427,155 maximal runs, produced 744,649 thick-strip pieces, performed 2,617,469 joint-endpoint steps and emitted 25,484,201 packed retained rows. The largest per-block endpoint census was 12,835 nodes; every block took the u16-link endpoint. The narrow node's four u16 links, u32 width and u32 word total 16 bytes. A materialized rule occupies 16 bytes, a band 12, a thick strip 16, and a queued retained rank four. A dense edge occupies four bytes when its code and successor fit in one word and eight on the split layout. Across these 228 blocks the measured logical edge table total was 101,936,804 bytes, exactly four times the number of retained rows. On the frozen block the dense table contained 359,496 bytes. Actual live logical array capacity after each call includes previous high-water blocks. The largest per-category capacities observed were 1,569,472 bytes for queues, 158,656 for strips and their image order, 414,552 for endpoint links/orders/bottom positions, 131,200 for rules and spelling stack, 98,304 for bands, 147,456 for the byte dictionary and offsets and 1,437,984 for edges. These omit the shared run front, output, process overhead and fallback. [Per-block raw ledger](geometry.csv)

The earlier whole-worker phase diagnostic placed admitted complete inversion at median .79637 of the combined writer's summed single-thread block CPU on all 431 blocks. If the eleven-block phase shares applied to the entire admitted class, the initial scan would then account for .2555 of that additive worker budget and residual emission for .3057. For a denominator with phase fraction s, removing fraction p of that phase saves sp overall. The table gives the required reductions **if those same effective fractions applied to current worker wall**. The historical scanner, worker and consumer run concurrently. The i9 receipt contains no matching phase measurements.

| W | Required fraction of combined wall | Scan alone | Residual emitter alone | Both reduced by the same fraction |
|---:|---:|---:|---:|---:|
| 7 | 10.93% | 42.78% | 35.76% | 19.48% |
| 8 | 12.25% | 47.95% | 40.07% | 21.83% |
| 9 | 11.42% | 44.68% | 37.34% | 20.34% |
| 10 | 9.82% | 38.42% | 32.12% | 17.49% |

Elimination of the entire endpoint surgery phase would represent only .0824 of this modeled additive worker total. It cannot deliver the full remaining gap by itself under this application of local proportions. The local instruction and CPU receipts preserve the contrast between the emitter's lower instruction share on frozen and its large complete-sample CPU share. Neither measures DRAM traffic. [Paid denominator and i9 scope](../../budget/README.md)

## Recover the scan's counters from its existing queues

At any prefix of the true run sequence, the length of byte a's queue is exactly the number of retained rows of a seen so far. The original scan separately increments thin[a] and all[a] on each kept run. Store only omitted thick rows in all[a] during the scan. When a thick strip arrives, its occurrence rank is all[a] + queues[a].len(). At the end of the scan recover all 256 thin and total counts from queue lengths. This removes both indexed read-modify-writes on every thin run, preserves the original contiguous u32 queue, and adds one queue-length read to each thick strip plus a 256-bucket pass. The primary's exceptional retained singleton is enqueued exactly once, so the invariant survives splitting a thick run around the primary. Applying the transformation only when the dense-queue route is selected preserves the other compile-time routes. The queue contents, word and edge representations have the original dimensions. [Source transformation](queues.py)

The full profile and paired experiment can be rebuilt from the repository root with a moonshot-enabled release library, the manifest toolchain, Valgrind 3.24 and one available Linux x86-64 CPU. The script extracts the immutable original from git before generating either experiment.

```sh
d=research/astra-e/narrow-short/cost
cargo build --locked --release --features moonshot --lib
bash "$d/run.sh" /tmp/astra-combined-cost "$PWD/target/release/deps" 3
python3 "$d/report.py" /tmp/astra-combined-cost
```

## Paid screen of the queue-counter change

Across 32 rounds with four position-balanced arms, the new scan took **.9003 [.8493,.9544]** of the existing scan's CPU. The complete instrumented inverse ratio was .9744 [.9198,1.0323]. Direct timing of the two uninstrumented complete inverses gave **.9879 [.9187,1.0624]**. Ratios aggregate the same eleven block durations before pairing by round. Frozen scan instructions fell from 4,308,332 to 4,143,160 (−3.83%), and complete inverse instructions fell from 10,072,884 to 9,908,938 (−1.63%). These figures justify testing the integrated entry independently. The complete local CPU interval spans both a gain and a regression. [Raw paired timing](paired.csv) · [Frozen candidate Callgrind dumps](callgrind-queues.tar.xz) · [All statistics](summary.txt)

The corpus validation matched all 228 original and released inverses and preserved the counts of bands, words, induction steps and packed table bytes. Queue order and every pushed element are identical by induction over the run scan. The targeted candidate also passed 24,590 direct stable-LF marked comparisons, including periodic and multiple-cycle inputs, the 127-byte cap and two wide-link fallbacks. Its source uses the existing generation preflight. The first invocation of the following commands prepares a scorer-ready entry. Peak process RSS, whole-worker timing and the independent i9 target remain open. [Candidate](candidate.rs) · [Marked receipt](marked.txt)

```sh
d=research/astra-e/narrow-short/cost
bash research/astra-e/prepare.sh "$d/candidate.rs"
bash "$d/marked.sh" /tmp/astra-combined-cost-marked "$PWD/target/release/deps" 3
```
