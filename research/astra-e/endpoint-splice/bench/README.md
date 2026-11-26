# Strict endpoint splices on the 233 actual scalar admissions

The exact batched constructor removed **6,145,662 of 19,961,308 explicit link assignments (30.79%)**. It took **1.0285× scalar CPU** [0.9950, 1.0631] to construct and emit the complete marked inverse across all admissions. Against the second same-code scalar label it took 1.0240× [1.0010, 1.0475]. The construction work count is a genuine reduction. The measured implementation has no demonstrated inclusive speed advantage. Scalar remains the direct control.

| Source-row orientation | Strict steps | Groups | Steps in groups of length ≥2 | Groups already in place |
|---|---:|---:|---:|---:|
| 0 | 632,144 | 298,084 | 445,649 | 9 |
| 1 | 1,049,954 | 359,746 | 839,967 | 0 |
| Total | 1,682,098 | 657,830 | 1,285,616 | 9 |

There were also 986,872 distinct equal-width steps and zero coincident tips. The 657,821 relocated strict groups cost six link assignments each. The 986,872 equal steps still cost ten each. This produced 13,815,646 assignments and placed 76.43% of strict steps in groups of length at least two. Savings per block ranged from 22.85% to 68.06%. Almost all savings came from grouping distinct losing nodes, with only nine already-positioned groups. Every loser still needs its width, word join and traversal. These counts alone do not quantify retired instructions, cache transfers or stalls. [Per-block counts](census/blocks.csv) · [Group length, orientation and physical width data](census/groups.csv) · [Registered threshold](../PLAN.md)

| Paid complete inverse | Blocks | Batched / scalar | Batched / replica | Replica / scalar |
|---|---:|---:|---:|---:|
| All admissions | 233 | 1.0285 [0.9950, 1.0631] | 1.0240 [1.0010, 1.0475] | 1.0044 [0.9897, 1.0193] |
| Original sparse admissions | 228 | 1.0297 [0.9933, 1.0673] | 1.0257 [1.0020, 1.0498] | 1.0039 [0.9884, 1.0197] |
| Additional dense admissions | 5 | 0.9980 [0.9569, 1.0407] | 0.9850 [0.9271, 1.0466] | 1.0132 [0.9272, 1.1070] |

The three medians of all-admission summed CPU were 478.45 ms for scalar, 474.42 ms for its replica and 485.96 ms for batched. Each of 24 rotations ran every arm on each block's true capped last column and primary, charging reconstruction of every temporary object, dense successor table and all output. Each arm reused its own normal buffers. Every arm occupied each of three slots eight times per block. The intervals use the four successive six-round means of paired log ratios, summing per-round CPU over the specified block set before division. The two identical scalar modules each own separate scratch, while the compiler emitted a single named scalar run/constructor pair. Their contrast measures position and scratch effects in this build. [Paired raw observations](price/times.csv) · [All block and group intervals](price/summary.json) · [Emitted symbols](price/arm-symbols.txt)

Across 90 fresh cap-only processes on the original ten systematic blocks, post-inverse VmHWM in KiB was 3,714 median and 5,052 maximum for batched, 3,720 and 5,052 for scalar, and 3,722 and 5,084 for the replica. Each process opened the byte oracle after sampling peak. Absolute memory includes that one cap representation and the complete cold inverse. It omits the decoder container, concurrency and preceding frontend. The CPU measurements ran at nice +8 on logical CPU0 of a Xeon Platinum 8573C from 08:30:32 through 08:31:24 UTC on 2026-11-19 including the initial input and warm-up checks. The host exposed CPUs 0–4 under a four-equivalent quota. Other research processes could use other logical CPUs, and a low-priority CPU2 compiler was visible. [Fresh samples](price/rss.csv) · [Measured build and flags](price/build.json)

The uninstrumented normal scalar and batched workers reproduced the libbzip2 plaintext on all 431 blocks, including the 198 short or column-fallback blocks. They checked block CRCs and all twelve stream trailers on 209,700,196 decoded bytes. For each admitted block the checker used the independent segmented inverse, expanded RLE1, verified CRC and compared against the same plaintext. All three measured kernels then reproduced the original marked inverse bytes in all 16,776 paid calls. The separate census compared all 15 public construction counters, retained mark, packed width, entire edge table, extra wide codes, spellings, offsets and all band triples between original and batched on 209,697,662 admitted rows. It verified the instrumented result and matched its counts to the original number of induction steps. The independent finite gate checked complete state and output against stable sorting on 28,741 marked/width cases, including arbitrary binary and ternary last columns, cap boundaries and inputs forcing the wide fallback. [Finite receipt](census/finite.txt) · [Structural checker](main.rs) · [Census build](census/build.json)

## Reproduce

Preregistration 67c756de and parent c3c43703, real geometry 62cde4e6, scalar ancestor d0a42980e896211906234c7679b38760ae44a46e. Use Rust 1.99.1. The independent complete plaintext SHA256 is 3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf. From the worktree root run

```bash
export PATH=/workspace/.mibz2-tools/rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin:$PATH
bench=research/astra-e/endpoint-splice/bench
scratch=/tmp/astra-endpoint-splice-native
plain=/tmp/astra-landmark-libbzip2.raw
bzip2 -dc corpora/wikidata.bz2 > "$scratch-plaintext"
cmp "$scratch-plaintext" "$plain"
for mode in census price; do
    taskset -c 0 nice -n 9 python3 -B "$bench/build.py" "$scratch" "$mode"
done
taskset -c 0 nice -n 9 "$scratch/census/panel" finite
taskset -c 0 nice -n 9 "$scratch/census/panel" census corpora/wikidata.bz2 "$plain" "$scratch/census"
python3 -B "$bench/report.py" geometry "$scratch/census"
taskset -c 0 nice -n 8 "$scratch/price/panel" price corpora/wikidata.bz2 "$plain" "$scratch/price"
taskset -c 0 nice -n 8 python3 -B "$bench/rss.py" "$scratch/price" /workspace/work/astra-powers-original/capped
python3 -B "$bench/report.py" price "$scratch/price"
```

The build checks out the frozen scalar's `src` into the specified scratch directory, generates matched candidate modules and compiles the metered and unmetered binaries separately. The two reports can also replay the archived raw rows directly. This bounded native screen supplies no independent CI, i9 or sustained worker wall-time result.
