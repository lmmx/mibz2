# Fixed landed cylinders as independent native delimiters

Recognizing the landed property literal directly reduced the local complete inverse to 0.904 [0.870, 0.939] times the twice-heaviest quote-return policy. Its ratio to the faster original Compact quote control was 1.241 [1.203, 1.280]. The comma-quote policy measured 1.141 [1.108, 1.174] times Compact quote. Both miss the preregistered throughput criterion. [Registration](REGISTER.md) · [Paired stage receipts](phase.csv)

## Which set is reconstructed

Start at the old contiguous landing interval U and extend backward along C, requiring that no earlier positive step reaches U. When U is the literal cylinder I(P), the full candidate is I(CP). The spelling CP fixes every interior copy of P beginning at offsets 1 through |C|−1. An internal hit makes the entire candidate empty. If none occurs, I(CP) is already the desired exact cut. The two fixed strings meet the latter condition. Their original rank cylinders are hexadecimal 2c22 and 222c2270726f7065727479223a22. Each is decoded as its own section with its own positive returns and primary. [Proof, exact discrete tests and prior work](audit/README.md)

The earlier trailing comma-quote phrase selects quote-comma-quote sources. Its rotated selection covers 15,844 of the 30,581 full comma-quote rows on frozen. The original twice-heaviest property source has an eight-byte prefix preceding the new property cylinder. Its eight-step rotation equals the full cut on frozen and seven of ten systematic tapes. On the other three, the landed cut adds four, one and four rows. These original and rotated source rank ranges are distinct even when their maps are conjugate. Clipped one-byte-specialized Compact uses trailing-marker word spellings, so object equality is tested in that convention. Each byte primary receives the corresponding independent trailing-marker skip. [Pinned positive-hit/rotation comparisons](audit/verified.txt)

## Paid recognition and reconstruction

The quote control keeps Compact's existing whole-byte path. The fixed comma-quote and quote-colon-quote recognizers seed the landing quote from the supplied histogram, then extend each pair of endpoints using a forward scan through the necessary prefix of the original capped L runs. The landed property arm makes a single cap scan to store nine fixed-byte cumulative counts every 128 cap records. Each backward step locates an endpoint's block and scans at most 128 local cap records. It pays the sample construction and all 26 endpoint queries for the full landed pattern. Its raw rank search consumes neither a return-word dictionary nor a supplied original section. The named adaptive control builds an original LF run index and performs exact maximum-width branch-and-bound, breaking frequency ties by leftmost source. The twice-heaviest control shares that original LF index across both adaptive searches. Once U is selected, the native clipped constructor rebuilds its terminal-tagged descriptors and runs the same marked reconstruction. These measured adaptive costs pertain to that implementation. [Selectors](selector.rs) · [Adaptive and absent-block handling](dispatch.rs)

An absent section builds a current-block LF index for the raw marked-cycle fallback. A nonempty section disjoint from the marked cycle takes the same marked-cycle output from its newly built index. Reusing the subject across alternating absent, present and disjoint inputs exercises stale-epoch handling. The dense oracle explicitly walks original Ψ from candidate rows to determine each literal section, selects adaptive sections from full source counts, and checks the complete first dictionary, maximal incoming runs, marked row and byte offset. Every arm independently checks n original marked bytes. Small inputs include arbitrary byte columns, periodic text, doubled property spellings, bare unlanded properties, missing quotes and missing extensions. [Marked checks and source](main.rs) · [Adversarial receipt](small.txt)

## Local paired stage

The panel uses 36 balanced rotations of six arms, 2,160 checked native outputs. Each paired repetition sums ten systematic block CPU times on one pinned thread. All arms begin with the original capped runs, supplied byte histogram and original primary. Phases below are mean milliseconds per ten blocks. Ratio intervals are 95% t intervals over the paired log sums.

| Arm | Recognition | Index and mark | First discovery | First merge | Second discovery and spelling | Complete |
|---|---:|---:|---:|---:|---:|---:|
| Compact quote | 0.005 | 2.583 | 7.655 | 2.008 | 3.596 | 20.338 |
| Adaptive frequent | 3.116 | 3.645 | 9.687 | 2.424 | 2.975 | 23.636 |
| Literal quote-colon-quote | 1.002 | 4.086 | 9.966 | 2.524 | 3.034 | 22.339 |
| Comma-quote | 0.162 | 4.105 | 9.988 | 2.170 | 4.974 | 23.183 |
| Landed property | 0.707 | 4.155 | 17.176 | 0.924 | 1.363 | 25.266 |
| Twice-heaviest quote return | 3.194 | 4.155 | 17.402 | 0.902 | 1.293 | 27.923 |

The landed property and twice-heaviest objects have 31,367 and 31,358 total source rows on the ten blocks. They pay 3,911,432 and 4,030,238 first-discovery edges compared with 908,546 for Compact quote. The smaller section reduces later merge/writer work and shifts the cost toward first discovery. Subtracting both measured selector times gives property/twice-heaviest 0.992 [0.954, 1.032] and property/Compact quote 1.206 [1.169, 1.245]. These accounting substitutions explain the advantage over the conjugate control and the remaining deficit. The measured comma-quote/adaptive-frequent ratio was 0.982 [0.955, 1.010]. [Unrounded phase summaries](summary.txt)

A separate accounting bound subtracts all four observed successful second-stage phases. The landed property would still cost 1.129 [1.093, 1.166] times complete Compact quote. Even eliminating its entire second stage cannot compensate for the first discovery cost on this panel. The same bound for comma-quote is 0.808 [0.780, 0.837]. A first-only permutation writer would need to pay its own link setup and final emission from that budget. This counterfactual changes no registered policy or performance conclusion.

## Complete real admission and peak memory

All six policies on all 228 real inputs matched n original output bytes as well as exact literal/adaptive intervals, complete first-word spellings, maximal token runs and marked offsets. All 1,368 cases entered the second stage. The same six fresh persistent processes then checked 1,368 outputs against independent dense-reference hashes while recording VmHWM. Across 28,170 small marked cases, 11,888 exercised absent sections and 4,407 exercised live sections disjoint from the mark. [Admission objects](admissions.csv) · [Per-process receipts](rss.csv)

On all 228 real blocks, the fixed quote-colon-quote cylinder is exactly the one-level adaptive frequent section. The landed property interval and the twice-heaviest interval have equal mass on 191 blocks. On the remaining 37, the landed property has one to six extra rows, 90 in aggregate. Equal mass alone does not prove conjugacy on those 191 blocks. The direct property accumulates 125,948,034 first dictionary bytes and 447,462 incoming runs, compared with 23,393,313 bytes and 1,786,472 runs at the original quote.

| Policy | Single-worker process HWM, KiB | Maximum retained capacity with output, bytes |
|---|---:|---:|
| Named Compact quote | 8,232 | 8,473,542 |
| Adaptive frequent | 8,500 | 8,529,750 |
| Literal frequent | 8,496 | 8,528,726 |
| Comma-quote | 8,120 | 8,396,806 |
| Landed property | 7,916 | 8,047,470 |
| Twice-heaviest | 7,784 | 7,540,590 |

Direct literal recognition lowers the observed peak against the fast quote control for both candidates. The landed property trades 132 KiB of process peak for its timing improvement against twice-heaviest. The leading speed and memory results therefore belong to different local points. No scorer worker, CI or target-machine performance claim follows from this native stage.

## Reproduction

Protocol commits are 303887b and a64cb2c, from before the first timing round. The two fixed policies, current-block fallback, adaptive comparisons and native harness are at 8ec1f2e. Historical provenance and exhaustive recognition checks are at 9625476. The paired panel is stored at deb4dba. Use the research worktree's Rust 1.99.1 release library and the established [original-cap exporter and independent hash recipe](../../native/README.md). The generator checks the native source chain back to stock Compact and its second writer. The same ten-tape, 228-admission, and output-hash inputs apply.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/direct-native
out=$(mktemp -d)
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
python "$d/../../../../clipped/native/tapes.py" research/astra-e/dense/target/wikidata "$out/ten.bin"
"$out/check"
"$out/check" check "$out/ten.bin" > "$out/ten.csv"
"$out/check" check /tmp/astra-token-all/admissions.bin > "$out/admissions.csv"
nice -n 8 taskset -c 3 "$out/price" price "$out/ten.bin" 36 > "$out/phase.csv"
bash "$d/memory.sh" "$out" /tmp/astra-token-all/admissions.bin /tmp/astra-clipped-final/expected.csv 2 > "$out/rss.csv"
python "$d/report.py" "$out"
```
