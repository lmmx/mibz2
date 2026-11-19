# All-block matched CRC follow-up

The nine-block initial screen had no repeated safe phrase at the 2,048-byte threshold. This run used every one of the 228 target-writer admissions. Thirty-six balanced nine-arm rounds gave 73,872 samples and four observations at each arm position for every block. All measured checksums matched the stored values and the decoded bytes matched libbzip2. Per-block tables and word observations exactly reproduced the frozen first experiment. Of the admissions, 196 contained a reusable visit of at least 512 bytes, 31 contained one of at least 1,024 bytes, and four contained one of at least 2,048 bytes.

Both empty-plan controls go through the exact incremental CRC code used for each ordinary gap. They carry the same paid or precomputed flag as the matched arm. A public stock CRC arm supplies a second control. Empty plans use a single ordinary call, bypassing digest-vector initialization. The paid descriptor is now eight bytes with the nonzero power of x serving as its validity indicator. Three static 256-word tables provide byte-length factors, with at most two carryless multiply/reductions needed to combine the length digits. The tables take 3,072 bytes total. Their values agreed with independent scalar exponentiation on 6,048 cross-range values. The reducer passed 32,768 additional pseudorandom polynomial products. Paid trials included digest-vector clearing, table lookup and CRC initialization of each encountered repeated word. Both arms still receive actual selected runs, mapped decoded positions, and the full emitted block for free.

| Minimum length | Paid / same-code empty [95%] | Free precomputed digest / same-code empty [95%] |
|---:|---|---|
| 512 | 1.0720 [1.0152,1.1321] | 1.1056 [1.0635,1.1493] |
| 1,024 | 1.0073 [0.9586,1.0585] | 1.0017 [0.9513,1.0548] |
| 2,048 | 0.9684 [0.9243,1.0146] | 0.9984 [0.9575,1.0411] |

The empty incrementally dispatched path/public stock ratio was 0.9679 [0.9077,1.0321]. At 2,048 bytes, even on the four blocks with actual selected spans, the paid and free intervals crossed parity. Mutually inconsistent close point estimates on a shared host do not establish a small advantage. Retain the production four-lane block CRC. The observed long-repeat coverage and fragmentation give no measured case for implementing a new worker CRC interface or seeking an i9 run.

```sh
d=research/astra-e/crc-phrases/followup
PAGE_DEPS=/path/to/release/deps bash "$d/run.sh" /tmp/astra-e-phrase-crc-all 2
python3 "$d/report.py"
```

The first command's times.csv is the decompressed version of the archived times.csv.xz. The independent 431-block correctness, source and transformed directory are the same as the [initial experiment](../README.md). The timing program keeps decoded blocks and plans live across trials, so its process footprint is inapplicable to production peak memory. A real implementation would pay span selection and count-hazard discovery in addition to these costs.

The independent frozen whole-worker budget also gives a generous conditional ceiling on these rare long returns. For each threshold, remove *the entire measured CRC-and-return time* from each of the 431 real blocks that has any eligible recurrence, retaining the complete measured worker time of the original control as denominator. Across ten full-corpus rotations, even a free CRC for the four blocks with a recurrence of 2,048 bytes would recover only 0.0247% of combined-worker CPU at the median (range 0.0137–0.0371%). For the 31 blocks at 1,024 the ceiling is 0.1948%, and at 512 it is 1.5613%. The real scheme still reads the gaps and verifies all output. These are computed local additive counterfactuals, excluding scanner/delivery overlap and any i9 throughput inference. [Per-threshold values](ceiling.txt) · [Recompute from the original stage receipts](ceiling.py)
