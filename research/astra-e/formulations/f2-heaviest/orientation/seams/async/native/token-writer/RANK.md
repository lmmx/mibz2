# Publish a token object by ranking the sparse meeting starts

For any single contiguous section of a stable bzip2 column, each exact return word occupies one contiguous source bucket. The cyclic suffixes in that bucket carry the same seam triple. Their bases come from actual section rank order. Raw lexical word order can disagree with this order even for a one-byte cylinder. The independent proof covers arbitrary rank intervals, periodic ties, singletons, exact mark conventions, and the conditions on disconnected sections. [Proof and counterexamples](audit/README.md)

Mark source and destination band starts in two static bitsets over the m section ranks. A prefix count per 64 bits and one popcount gives the compact ordinal of a marked start. Scatter band indices into the two orders. Scan the source order and materialize a spelling only when the triple changes. The destination order now supplies maximal token runs, while an occurrence counter per token checks every known affine source/return match. The complete physical producer already sorts its published bands by source; that variant omits the source bitset and its scatter, paying the producer's sort. For general unions the native interface retains triple interning and an exact stable-LF certificate, with an exact fallback upon refusal.

The additional assembly work is O(m/64 + A + D), where A counts meeting bands and D counts the actual distinct spelling bytes. It stores O(m/64+A+K+D) words/bytes, with K token types. This charges building and initializing both sparse start directories. A single forward/backward path pair supplies all spelling bytes once per identified type. The second induction receives the existing Compact/preload word slices, bases, maximal token-column runs and marked row/skip.

## Complete and conditional prices

Twenty balanced paired repetitions on ten systematic blocks paid the real bounded-rank constructor, its group heap and source sort, the new token assembly and full second decode. Complete paid time was **17.18 [12.26, 24.07] × Compact on quote** and **27.01 [20.68, 35.28] × Compact on frequent**. Arc discovery dominates. All observations, including several CPU measurements over 100 times their arm's median, are retained. No whole-worker, CI, or i9 benchmark follows from this result. [Twenty paid paired repetitions](rank-paired.csv) · [Ratios](rank-summary.txt)

A separate 60-repetition confirmation prices the conditional writer with unsorted meetings and complete directional path nodes supplied. It omits the costly physical construction arm, balances five local arms through a ten-repetition order cycle, and compares identical marked output on all ten tapes. Reported observations are paired sums of all ten blocks per repetition, with geometric ratios and Student-t intervals on their logs. All 6,000 outputs matched.

| Conditional writer / complete Compact | Quote [95% interval] | Frequent [95% interval] |
|---|---:|---:|
| Sparse start ranks, dictionary and native second | 0.814 [0.756, 0.875] | 1.004 [0.947, 1.064] |
| Triple hash, dense destination scatter and native second | 0.870 [0.792, 0.957] | 0.966 [0.921, 1.012] |
| Triple hash, destination comparison sort and native second | 0.957 [0.864, 1.059] | 1.113 [1.020, 1.214] |
| Complete first token object supplied, native second only | 0.320 [0.293, 0.349] | 0.229 [0.213, 0.247] |

The no-hash/hash-scatter ratios were 0.935 [0.863, 1.012] and 1.040 [0.990, 1.091]. Neither resolves the time ranking. The scratch improvement is directly observable. Frozen quote uses 427,236 additional first-object assembly bytes from unsorted meetings, versus 1,191,932 for hash/scatter. With sorted sources already available, the extra assembly peak is 377,252 bytes. On frequent a complete first object handed over for free leaves substantial room for a constructor, but the measured materializers spend substantially all of it. The same retained geometry leads to different performance budgets between the two sections. [All 60-repetition observations](repeat.csv) · [Ratios](repeat-summary.txt)

## All 228 admissions

For both quote and its most frequent incoming type on **all 228 registered Wikidata admissions**, the rank-assembled second decoder matched the stable-sort inverse. An independent row walker checked every dictionary byte and true next-section hit for every occurrence, totaling **410,395,588 arc bytes**. Marked output totaled a further 410,395,588 verified bytes. Every object used the actual second decoder. For all 228 quote objects, the mark/skip, every first-level byte slice, each source base and every destination root run matched Compact's own first induction up to ID renaming. The selected frequent section uses maximum occurrence count, ties by lowest source base. [Per-block identity and capacity receipts](coverage.csv)

| Sum across admissions | Quote | Frequent |
|---|---:|---:|
| Section occurrences | 30,868,395 | 5,566,401 |
| Distinct word types per block, summed | 728,887 | 806,027 |
| Physical hit bands and maximal destination runs | 1,786,472 | 1,433,311 |
| Spell once per word, bytes | 23,393,313 | 41,433,719 |
| Spell once per band, bytes | 37,911,908 | 65,405,608 |

For frozen quote, retained physical geometry holds 4,918,996 tracked bytes, the completed C++ first-token object holds 259,104, and Rust retains 2,135,099 of adapter, second-stage, and output capacity. These coexist during output and already exceed the complete Compact control's 3,590,067 retained capacity. On frequent the corresponding figures are 4,892,596, 340,020 and 1,989,955 bytes. Capacity measurements include the n-byte output and represent final reserved payload; allocator metadata, Rust reallocation transients, caller input buffers and verification storage are excluded. These are not process peak RSS. The complete adverse time and capacity results rule out this physical combination as a production candidate.

The practical next target is a producer that publishes the token-column runs, source bases and one spelling per type directly. The measured second-only budgets quantify why producing that smaller object is worth investigating. Replaying the present rank ownership or enumerating the same per-band spellings would retain the priced work.

## Reproduction

Use the committed Compact and second sources, the repository's Rust toolchain, release library with benchmark internals, a C++20 compiler and Python with SciPy. The canonical compressed corpus SHA-256 is 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915. The exporter emits only original capped runs and the mark.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/seams/async/native/token-writer
out=/tmp/astra-token-writer
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
"$out/rank"
nice -n 10 taskset -c 4 python3 -B "$d/panel.py" "$out/rank" research/astra-e/dense/target/wikidata "$out/paid.csv" --reps 20
nice -n 10 taskset -c 4 python3 -B "$d/panel.py" "$out/envelope" research/astra-e/dense/target/wikidata "$out/repeat.csv" --reps 60
python3 -B "$d/summary.py" "$out/repeat.csv"
"$out/export" corpora/wikidata.bz2 > "$out/admissions.bin"
"$out/coverage" < "$out/admissions.bin" > "$out/coverage.csv"
```

[Native rank assembly](rank.hpp) · [Exhaustive 181,836 marked cases and fallbacks](rank-small.txt) · [All-admission independent arc checker](coverage.cpp)
