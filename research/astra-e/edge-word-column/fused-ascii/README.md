# Direct marked ASCII pairing on the measured edge constructor

**Direct simultaneous pairing speeds the marked emitter on eligible blocks, but its constructor more than spends that gain.** In the packable subset of the frozen 17-block panel, emission took 0.7994× the measured edge worker's CPU [0.6929, 0.9223], construction took 1.3514× [1.2200, 1.4969], and the complete inverse took 1.1423× [1.0432, 1.2508]. The complete result against a second worker using the identical control code was 1.1483× [1.0428, 1.2644]. The intervals are paired round-level 95% t intervals on the log ratios. These are single-thread native measurements on the shared local host, not target W7–10 performance. [Frozen test design](PRICE-PLAN.md) · [All 1,632 native samples](paid.csv) · [Group and phase estimates](summary.json)

The kernel at 0507e3deb06e9a4ad2c3d3992755810efe40e754 produces the final packed successor directly from the unmodified endpoint bands and per-byte queues of the i9-tested edge worker badb2ae. It merges two consecutive ASCII literals when the old 14-bit pair alphabet fits. The second target comes from its original physical partner band and queue, and the lookup is fused into the single final table write. The incumbent creates one old single-hop entry per row. The previous pair implementation first created that old table, then read and rewrote it. Direct simultaneous acquisition uses immutable queue inputs, which permits overlapping eligible starts and attains the minimum possible number of marked hops under this packing policy. Its new directory and second-target loads cost more than the eliminated pass. [Exact construction and optimality](STRUCTURE.md) · [Original postpass](../../pairs/README.md) · [Self-contained scorer entry](candidate.rs)

| Full 17-block panel compared with original edge | Constructor | Marked emission | Complete inverse |
|---|---:|---:|---:|
| Identical code, second worker | 0.9909 [0.9474, 1.0364] | 1.0577 [0.9500, 1.1777] | 1.0247 [0.9542, 1.1003] |
| Dense row-ordered legacy postpass | 1.0592 [0.9895, 1.1339] | 0.9603 [0.8833, 1.0440] | 1.0140 [0.9618, 1.0691] |
| Direct simultaneous queue pairing | 1.1931 [1.0570, 1.3467] | 0.9679 [0.8858, 1.0576] | 1.0957 [0.9976, 1.2035] |

The timing panel contains 8 packable and 9 fallback blocks. The 233-block correctness scan includes every one of the 46 corpus fallbacks. Across the 187 packable inputs, the direct method needs 1,769,715 partner-range claims and 13,429,533 second-queue gathers. Marked hops drop from 18,658,864 to 11,573,452 on these inputs. Across all 233 they drop from 26,567,581 to 19,482,169. Of the 13,429,533 encoded shortcuts, a marked traversal consumes precisely 7,085,412. Overlapping starts account for the difference. Avoiding unnecessary compositions without discovering marked traversal parity in advance is a separate problem. [Full structural table](fused-pairs.csv)

Peak reserved successor/dictionary/directory capacity in the reused worker rose from 1,585,440 to 1,696,056 bytes. Summed initialized edge/directory bytes rose from 106,270,324 to 120,331,492 across admissions; this includes still-initialized directory storage from a previous eligible block. Twelve fresh processes per arm decoded the complete CRC-checked corpus. Median peak RSS in KiB was 19,878 [19,772, 19,880] for original edge, 19,796 [19,772, 19,880] for the older dense postpass, and 19,794 [19,784, 19,880] for directly fused pairs. The intervals here are observed minima and maxima. These figures establish neither an RSS benefit nor target-scale memory equivalence. [Each fresh process](memory.csv) · [Initialized and reserved arrays per input](checked.csv)

Every entry of the actual Rust-published table matched the independently computed original successor or its two-step composition on every admissible physical row. The 233 coded outputs matched Segmented, their RLE expansions matched libbzip2 and their block CRCs passed; the 431-block corpus passed all twelve combined stream CRCs. The actual Rust queue publisher and emitter also passed 640,196 exhaustive arbitrary marked labelled permutations through five rows, including 391,680 nondivisor horizons, plus 17,064 adversarial native stable-BWT comparisons per arm. The matched edge and fused binaries used rustc 1.99.1, optimization level 3 and one codegen unit. The native four-arm executable was SHA-256 517dc787f643d70b7b154c182529c346b21dd5d722709e10f4e96727b2ade9c4. [Corpus oracle receipt](checked.txt) · [Permutation and overlapping-start checks](property.txt) · [Stable BWT cases](adversarial.txt)

From the repository root with the canonical corpus, rustc 1.99.1 and libbzip2's CLI, reproduce the deterministic, single-thread price and RSS evidence as follows. Native controls are extracted from the pinned measured source. The generated candidate source is already committed. [Reproduction wrapper](native-replay.sh) · [Paired raw-data reduction](summary.py)

```bash
d=research/astra-e/edge-word-column/fused-ascii
t=/tmp/astra-fused-pair-replay
bash "$d/native-replay.sh" "$t" check
nice -n 12 taskset -c 3 bash "$d/native-replay.sh" "$t" price
nice -n 17 taskset -c 1 bash "$d/native-replay.sh" "$t" memory
python3 "$d/summary.py" "$t"
```

An endpoint certificate for replacing the second-queue load by arithmetic could cover only 1.42% of eligible gathers on complete original ASCII bands. Even free discovery of each actual claim's suffix boundary left only 6.54% removable after checking its endpoints. Full arithmetic subdivision increased directory size 34.85×. [Proof and all alternatives](AFFINE.md)
