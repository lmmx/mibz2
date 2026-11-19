# Byte-width symbol transport between the scanner and workers

All 431 Wikidata blocks can carry every decoded Huffman symbol in one byte. The working prototype halves the symbol payload and lowers median peak process RSS by 1.67% at W2 and 2.62% at W4 against the i9-leading edge constructor. The complete worker decoder's W2 wall ratio was 1.1119 and W4 was 0.9986. Keep the lower-memory representation as an experimental result. The throughput evidence gives the incumbent edge priority for target testing. [Implementation at ae43fb7](generate.py) · [Complete paired results](results/comparison.csv)

## Scanner output and worker input

The scanner header defines the alphabet. If at most 254 bytes are in use, RUNA, RUNB, all literal ranks and EOB fit in u8. EOBs 256 and 257 select the original u16 implementation for the whole block. A pair's first code is never EOB, and both canonical Huffman paths validate their permutation index before emitting a symbol. A malformed prefix follows the existing error path even when the byte representation is selected. The parity and value of a symbol play no role in deciding the width. [Preregistered census and invariant](PLAN.md) · [Admitted bytes and true EOB values](blocks.csv)

In an isolated checkout, the regular worker pool's scanner opts into a separate method. It specializes the Huffman group loop once after reading the alphabet, keeping the original code and error decisions. The pooled `Vec<u16>` still owns the allocation. In the packed case its first word is 65534 or 65535, values outside the legitimate symbol range 0–257. The remaining words contain bytes in original decode order, with an initialized final padding byte for an odd count. The tag encodes that parity. The byte view excludes the tag and pad. The bounds-checked spare slice has room for all 900,100 positions, and the fast lookahead store remains guarded by the existing group-size invariant. Failed scans return the cleared carrier and preserve its capacity. A wide block reuses or grows the same allocation.

Workers classify the tape once and use specialized scalar or SIMD readers. The 32-symbol AVX2 bitmap step changes from two 32-byte loads of u16 lanes to one of u8 lanes. The original LZCNT/BMI2 run geometry and edge constructor then consume the same run records. Both plain-column and checkpoint-column readers accept bytes directly. Only the experimental worker launcher transports a packed tape. Ordinary public scanner consumers and the team launchers still use the original data contract. The generator specializes the fixed standard run interface used by edge, Lazy and eager. A future generic adapter for third-party research `invert_symbol_runs` overrides would need an explicit byte-tape interface. [Isolated source transformation](generate.py) · [Build script](build.sh)

The 233 large Wikidata blocks had EOB 80–103, and the 198 smaller blocks had EOB 2–61. Transport shrank from 49,958,598 to 24,980,382 bytes per corpus copy, including 862 tag bytes and 221 padding bytes. At W7–10 the maximum symbol payload saved over any contiguous 4W blocks was 2.61–3.28 MB before tag overhead. Independently of block retirement order, the largest possible payload savings over any 4W selected blocks were 4.79–6.41 MB. The nominal full-pool reserved capacity saving was 25.20–36.00 MB if all slots acquired separate buffers. High-water reuse and untouched reserved pages matter for actual RSS. [Exact window and capacity counts](geometry.txt)

## Correctness and local measurement

The refreshed scorer passed all 338 groups, including the byte-for-byte Wikidata comparison with libbzip2, CRC, periodic and multiple-cycle probes, and the group of 600 mutated streams. Its sequential verifier uses the regular scanner, and its worker checks use the new scanner. A separate sequential differential exercised the new scanner with both scalar and detected kernels. On every real block its individual symbol values, marked header, positions, byte output, and block result matched stock. The 431-block stream produced 209,700,196 output bytes on both routes. [Official gate](results/gate.jsonl) · [Direct real-stream differential](results/real-check.csv)

Another 203 input streams in each kernel mode produced identical data or identical first errors. The ten valid streams were independently decoded by Python's libbzip2, including EOB 255, 256 and 257, concatenated streams and an 800,000-byte stream switching between four narrow and five wide blocks. The 193 mutated streams covered block and combined stream CRC failures, truncation, unfinished RLE, bad Huffman codes, code lengths, marked pointers and magic. [Boundary checks](results/width-check.csv) · [Deterministic fixtures](fixtures.py)

Each of 16 rounds rotated stock LazyCells, stock edge and the packed scanner with edge on W2 and W4. Each process decoded two Wikidata copies with 4W slots, with zero settle on a four-CPU-quota Intel Xeon Platinum 8573C. The scorer printed `v1` at research receipt `81585172f5fb9e7a59f330124779ba084fd6bf04`. The latest master scoring commit was `75297a1` and both scorer trees were identical. Ratios and intervals use the scorer's 95% paired log method. [Scorer, compiler, binaries and input hashes](results/manifest.txt) · [96 raw timings](results/runs.jsonl)

| Workers | Wall / edge [95% interval] | Wall / Lazy [95% interval] | Median peak RSS / edge | Median peak RSS / Lazy |
|---:|---:|---:|---:|---:|
| 2 | 1.1119 [1.0008, 1.2354] | 0.9260 [0.8012, 1.0703] | 0.9833 | 0.8183 |
| 4 | 0.9986 [0.9446, 1.0556] | 0.8829 [0.8335, 0.9352] | 0.9738 | 0.7338 |

A separate 12-pair scanner-only run, alternating modes on CPU 3 after warmup, gave packed/wide CPU time 0.9170 [0.8343, 1.0080]. The marginal medians were 80.00 and 80.56 ms. This measurement leaves scanner speed unresolved and the W2 end-to-end regression unattributed. The main comparison includes handoff, reconstruction, RLE, CRC, stream ordering and process peak. The scorer's current symbol extractor uses the ordinary public scanner, so its inverse-stage proxy cannot see private scanner/worker transport. Hardware per-stage instruction counts, IPC and cache events remain unmeasured. [Raw scanner diagnostics](results/scanner.csv)

## Reproduce

Start with this study's checkout and the checksum-verified Wikidata corpus. The build driver creates two fresh detached worktrees from the pinned source, refreshes the scorer, regenerates the frozen edge kernel in both and applies the byte experiment to one. It leaves the research checkout intact.

```sh
study=research/astra-e/byte-symbols
wk=/tmp/astra-byte-replay
wiki=$(realpath corpora/wikidata.bz2)
bash "$study/build.sh" "$wk"
mkdir "$wk/results"
"$wk/candidate-build/release/moonshot" check --arm candidate --wikidata "$wiki" >"$wk/results/gate.jsonl"
python3 "$study/fixtures.py" "$wk/fixtures"
"$wk/check" "$wk/fixtures" >"$wk/results/width-check.csv"
"$wk/check" "$wiki" >"$wk/results/real-check.csv"
python3 "$study/panel.py" "$wk/candidate" "$wk/control-build/release/moonshot" "$wk/candidate-build/release/moonshot" "$wiki" "$wk/results"
taskset -c 3 "$wk/scanner" "$wiki" >"$wk/results/scanner.csv"
```

The prototype's implementation is reconstructible at `ae43fb76de758d8faae2e318ffff0022e0f0a0ae`. The control and candidate use identical scorer source and edge constructor. CI and the target i9 have not evaluated byte transport. For a future independent comparison, freeze both binaries at matching sources and measure complete workers with a wide-baseline control, paired peak RSS and block checks. The present W2/W4 results do not justify displacing a stronger target submission.
