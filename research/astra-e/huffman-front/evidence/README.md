# Can the i9 supply a fused Huffman and run front?

The paid native comparison gives each worker a verified block start and level. It can establish whether parsing straight to capped run records lowers CPU for a known envelope. A production reader must also discover, validate and schedule that envelope. Putting the joined code on the existing serial scanner has a different break-even point. At W10 the old measured scanner used 0.778 of Lazy wall, leaving only about **7 ms per Wikidata×4** of added scanner time at the 0.80 target. Moving both stages to workers eliminates that particular serial deadline but makes the W workers pay for parsing and exact discovery. [Raw derivation](bounds.json) · [Executable](bounds.py)

## Earlier paid and proposed work

- On the i9-10900K, #10 compared the scanner with a worker that finds candidate magic markers, calls `read_item` into a `Vec<u16>`, then calls `decode_block`. This paid for discovery, chain validation, speculative work, scheduling, symbol storage and complete decode. Its five paired rounds on level-9 Wikidata gave median **find/scan throughput** 0.892/0.927/0.978/1.006/1.006/0.981 at W1/2/4/6/8/10. W10 ranged from 0.880 to 1.028. Level-1 Wikidata and binary records gave 1.615 and 1.780 at W10. The older inverse, one table per worker, in-memory harness and candidate list that grew with input differ from the present pipeline. The worker’s returned symbols disambiguate what this earlier result covers. [Frozen implementation and registration](https://github.com/lmmx/mibz2/tree/efc1bd88c73b07d917617eddfdf008dffc691e3d) · [Measured report](https://github.com/lmmx/mibz2/blob/4fe9f12004f730752353edcd49f25496d2b15ef6/docs/journal/2026-11-10-par-scan-result.md) · [Original target evidence](https://github.com/lmmx/mibz2/tree/727674e185744b91077dd4a862f2cd9e1485f0d0/par-scan)
- The compact30 channel proposed speculative block discovery with worker-side Huffman+MTF, overlapping dependency chains and removing the symbol array. Its log records the proposal, without a same-record-return throughput result. The earlier pbz2 0.2.0 decoder did fuse Huffman/RLE2/MTF into T-vector words. Against the then-current mibz2 in the five-round, 79.2 MB bake-off its complete wall ratios were 0.98 [0.90,1.06] at one thread and 1.00 [0.86,1.16] at four. On separate real dump blocks at four threads it measured 1.06 [0.77,1.46]. Its scheduling, arena, inverse and output paths also differed. I found no earlier paid comparison of the same modern capped-record result with and without the full symbol tape in the examined repository records. [Compact30 log](https://github.com/lmmx/mibz2/blob/44d4f34f0dc3f7865e4c28143fb58159e7d8e276/docs/journal/compact30-channel/log.md) · [pbz2 design](https://github.com/lmmx/mibz2/blob/09984cf9c1bac313523a466be9752b9a2ef877e8/docs/journal/2026-10-27-mibz2-v2.md) · [Paid bake-off](https://github.com/lmmx/mibz2/blob/734f9da49a330eef1be4cd0aa41299c9cb742926/docs/journal/2026-10-27-bz2-bakeoff.md)
- The proposed sibling front retained serial Huffman parsing and would have moved MTF to a sibling hardware thread. Its 9–16% W10 share is an estimate from a separate CI stage price and assumed inflation, since the i9 inverse scopes did not measure MTF alone. The worker-bound ideal gain of 1.10–1.19 and estimated 5–10% after SMT sharing did not qualify a built arm. The i9 archive named `fused/` tested a **post-walk** pass at source `27c39f9f`. The later Astra precounted front started from already decoded symbols. Its independent whole-worker CI ratios against its measured predecessor were 1.0110/1.0212/1.0122 at W1/W2/W4. Each is a different change of phase boundary. [Sibling design and assumptions](https://github.com/lmmx/mibz2/blob/cc2147679e3aa86d7ef6af95eb5bcef378f27771/docs/journal/2026-11-16-sibling-front-end.md) · [Post-walk archive](https://github.com/lmmx/mibz2/tree/0026923046a02598bc7ce9607902e02ce01ee1d3/fused) · [Precounted front CI](../../ci/evidence/front-fusion-44445335323.md)

## Serial producer

Normalize all demands to the wall of the paired 0.5.6 LazyCells process, L. Let S be the scanner’s CPU demand, Q the sum of worker CPU demands, F the worker front CPU removed, and P the net change to scanner CPU from joining the two stages. The measured serial and worker capacity floors give a necessary condition for the new wall ratio T:

```text
T ≥ max(S + P, (Q − F + worker retry and conversion CPU) / W, other serial stages).
```

This inequality uses CPU seconds per logical thread and consistent demands. It omits waits and assumes the demands still hold after moving work. Necessary serial headroom is therefore P ≤ 0.80 − S. The table uses the corresponding medians of individual Lazy role readings from 12 target rounds. The last column converts dimensionless headroom with median Lazy wall from the newer 40-round experiment. The measured scalar ratios come from the 48-round scalar registration.

| Workers | Scalar / Lazy | Historical S / Lazy wall | Max extra scanner demand to match scalar, units of L | Max extra scanner demand for stretch, units of L | Allowed scanner growth | Extra scanner ms for stretch, ×4 |
|---:|---:|---:|---:|---:|---:|---:|
| 7 | .8702 | .7200 | .1502 | .0800 | 11.11% | 24.8 |
| 8 | .8797 | .7554 | .1243 | .0446 | 5.91% | 13.4 |
| 9 | .8763 | .7732 | .1032 | .0268 | 3.47% | 8.2 |
| 10 | .8590 | .7776 | .0813 | .0224 | 2.88% | 7.2 |

Historical opend scanner shares measured against their own faster wall were .771/.812/.829/.824. Against the correctly paired Lazy denominator those same samples gave .7200/.7663/.7729/.7744. Scanner service can itself change with cache residency, concurrent workers, memory placement and frequency. No target scalar or cap-counter run captured per-role time. The 40-round scalar comparator reproduced a whole-CPU near the 48-round release, with separate noise and scheduling. [Historical role rows](../../budget/i9-roles.csv) · [Verified scalar run](../../workstation/scalar-independent-i9.md) · [Newer counter and scalar run](../../workstation/counter-independent-i9.md)

## Known boundaries on workers

Let A be the aggregate CPU saved by the joined front against the old scanner plus old worker front, including table setup, all real routes, recognizers and fallback replay. Let D be candidate discovery plus any additional decoding on **the workers**, and G the serial finder/joiner demand. Then

```text
T ≥ max(G, (Q + S − A + D) / W, other serial stages).
A − D ≥ Q + S − .80 W   is necessary at the target.
```

Removing the scanner’s bottleneck does not remove the Huffman work. If an ordinary scanner is also retained to identify boundaries, its parsing and supply still have to be paid beside this worker demand. Even with free boundaries, workers need actual CPU savings or idle capacity. T can exceed this service floor through imbalance, the longest block, queue stalls, in-order delivery and memory contention. Thread CPU can overlap on the i9's twenty logical processors. Dividing total process CPU by ten would impose an unjustified service model.

The target receipts contain whole-process CPU C, with no scanner/worker breakdown. For a conditional numerical bound put Q+S = C−O, where O is CPU outside those two roles. The table transfers O from the historical Lazy run using its measured process CPU minus named worker and scanner CPU. It includes role-sampler quantization and interval mismatch. Rows use the median of each new scalar CPU/paired-Lazy-wall ratio. No cross-epoch pairs enter an interval. The two columns per quantity retain the independent 48-round and 40-round scalar data.

| W | Scalar CPU/L, 48 / 40 rounds | Assumed O/L | Worker service floor with zero saved CPU, 48 / 40 | Net CPU saving required for stretch, ms per ×4, 48 / 40 |
|---:|---:|---:|---:|---:|
| 7 | 6.916 / 6.900 | .754 | .8802 / .8780 | 173.6 / 169.1 |
| 8 | 7.781 / 7.776 | .912 | .8586 / .8580 | 141.1 / 139.0 |
| 9 | 8.469 / 8.522 | .972 | .8331 / .8389 | 91.0 / 106.7 |
| 10 | 8.953 / 9.070 | 1.028 | .7925 / .8042 | 0 / 13.4 |

A zero means the modeled worker-capacity inequality allows the target without net CPU savings. It says nothing about the rest of the critical path. W10 crosses that boundary between the two data sets. Reassigning the historical residual changes the budget further. Charging only **named main and feeder** CPU to O makes the corresponding 40-round floors .9363/.9149/.8958/.8575. Transferring the measured historical opend residual gives .8819/.8615/.8276/.8035. The true scalar split is unmeasured. Applying the historical W10 estimate F/Q=.09–.16 in addition to the 40-round Lazy-residual assumptions gives S+F≈1.43–1.94 in units of L. A scanner doing all of that would need to consume at most .80 of L, or about 41–56% of the original sum. This illustration depends on two independent transfers. None of these conditional values qualifies a candidate for the i9 target. [All alternatives and original archive hashes](bounds.py) · [Numerical values](bounds.json)

## State and memory

The four-copy input supplies **99,917,196 symbols**, or 199,834,392 bytes of logical u16 payload. Its 1,532 run-route blocks produced 65,307,816 capped records, or 130,615,632 bytes of record payload. The other 192 tiny column routes contained only 2,052 symbols and produced 1,860 BWT rows. A whole-corpus word-aligned one-run-bit-per-symbol bitmap needs 12,497,344 logical bytes if every block is built once. The largest individual block in the corpus holds 199,770 symbols and the largest run tape 185,251 records. These are counts of initialized data. [Per-block run census](../../front-fusion/geometry.csv) · [All-block symbol census](../../mtf-pairs/geometry/blocks.csv)

The original sequential scanner reserves space for 900,100 u16 symbols in a reusable vector. Its free-vector pool, 4W block window and worker scratch have different lifetimes. With one full-size vector per slot, the **reserved** payload scale is 50.4/57.6/64.8/72.0 decimal MB across W7–10. The committed pages depend on actual writes, slot reuse and allocator behavior. Worker fusion can avoid this queue of full symbol arrays and use bounded groups, a run bitmap and the capped records already required downstream. It may still retain a full symbol vector and a stock run or column workspace for a rejected or malformed block. A producer emitting records instead changes their ownership and cache handoff. Candidate discovery, validated compressed spans, out-of-order results, CRCs, and the final output slots need separate accounting. Stock #10 accumulated found candidates with input size. Counts of bytes written or saved do not directly determine DRAM line transfers, peak RSS or throughput. [Original queue](https://github.com/lmmx/mibz2/blob/530ed404a014a414ef0b62adfd947fa14d4533e6/src/parallel.rs) · [Original scanner reservation](https://github.com/lmmx/mibz2/blob/530ed404a014a414ef0b62adfd947fa14d4533e6/src/block.rs) · [Previous handoff and critical-path study](../../budget/overlap/README.md)

Reproduce from the campaign checkout with its reference and evidence commits fetched.

```sh
python3 -B research/astra-e/huffman-front/evidence/bounds.py > /tmp/astra-huffman-supply.json
cmp /tmp/astra-huffman-supply.json research/astra-e/huffman-front/evidence/bounds.json
```
