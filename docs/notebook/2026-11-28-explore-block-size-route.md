# Explore, 28 Nov: Open's admission doesn't look at block size, and on bzip2 -1..-3 input it picks the slower inverse

Every route figure in the frontier is priced on 900k blocks: the bench corpus and the
off-wikidata panels are all `-9`. Here the same 209.7 MB of wikidata (decoded from
`corpora/wikidata.bz2`, SHA-256 checked) is re-encoded by bzip2 1.0.8 at levels 1, 2, 3, 5, 7 and 9
(SHA-256 prefixes 2ae39adffbfb, aa55dcfa36b9, 380cbfda6162, 78ee1f9e68a2, 58c8897f3da9,
817e6d89f4ab; the -9 matches the encoder-dependence explore's re-encoding). Then the decoder is
priced per level. VM: 4 vCPU Xeon @ 2.1 GHz, **2 MiB L2 a core**, 260 MiB L3. It is noisy and
bimodal, so every figure below is from interleaved rounds. Master 9280643, no `src/` edit.

## 1. Whole decode: U-shaped in level

`examples/bzcat` at production's 4 x threads slots, 5 interleaved rounds, median wall (ms):

| level | blocks | in MB | t1 | t2 | t4 | lbzip2 -d t4 |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 2098 | 19.0 | 551 | 259 | 220 | 545 |
| 2 | 1049 | 15.8 | 498 | 271 | 188 | 565 |
| 3 | 700 | 14.4 | 406 | 234 | 171 | 591 |
| 5 | 420 | 13.0 | 406 | 251 | 158 | 795 |
| 7 | 300 | 12.3 | 456 | 233 | 166 | 1056 |
| 9 | 234 | 11.9 | 492 | 261 | 158 | 1186 |

lbzip2's cost rises with level, as the inverse's cache misses grow with n. mibz2's falls, and at t4
level 1 is 1.39x level 9. Peak RSS rises with level (35 → 98 MiB at t4), as expected.
`station_balance` (3 passes): at -1 the scanner has 40.0M symbols against 25.0M at -9 (+60%, at
the same 2.2-2.5 ns a symbol), so S rises and k* = W/S falls from 7.2 to 5.1. That is the scanner's
share. The rest is in the worker.

## 2. The worker: Open loses to Segmented below ~300k rows

`examples/route_census` with two new switches. `ROUTE_FULL=90000` counts -1 blocks: the old
fixed 100,000 cut had silently dropped all 2,097 of them. `ROUTE_SHADOW=first` times
`Segmented` before the probe as well as after, to control order. Every block at every level is
the Open route (`OpenCellsOf::cap`, r/n < 0.21). Production's inverse over `Segmented`'s on the
same blocks, summed over the file, 3 rounds x 2 orders:

| level | n | Open/Seg (6 runs) | blocks Open wins |
|---|---:|---|---:|
| 1 | 99,981 | 1.167-1.200 | 20% |
| 2 | 199,981 | 1.062-1.091 | 35-38% |
| 3 | 299,981 | 0.980-1.021 | 53-55% |
| 5 | 499,981 | 0.712-0.763 | 90% |
| 7 | 699,981 | 0.540-0.580 | 98% |
| 9 | 899,981 | 0.481-0.514 | 98-100% |

The order made no difference: it moved no ratio by more than the round-to-round spread. The whole
file through `decode_with` (no shadow, best of 5, 3 rounds) shows the same thing: production over
`Segmented` alone was 1.06-1.19 at -1, about 1.0 at -3 and about 0.73 at -9.

**It is block size, not only r/n.** Smaller blocks have higher r/n (median 0.095 at -1, 0.068 at
-9), but within a fixed r/n bin the ratio still moves with n. Open/Seg summed per bin, pooled runs:

| r/n | n 100k | 200k | 300k | 500k | 900k |
|---|---:|---:|---:|---:|---:|
| 0.06-0.07 | 0.79 | 0.74 | 0.80 | 0.64 | 0.44 |
| 0.07-0.08 | 0.95 | 0.98 | 0.95 | 0.71 | 0.48 |
| 0.08-0.09 | 1.03 | 1.05 | 0.98 | 0.76 | 0.53 |
| 0.09-0.10 | 1.20 | 1.12 | 1.03 | 0.76 | 0.59 |
| 0.10-0.12 | 1.28 | 1.24 | 1.18 | 0.95 | 0.67 |
| 0.12-0.15 | 1.43 | 1.44 | 1.38 | 1.01 | 0.77 |

The curve is flat from 100k to 300k and drops between 300k and 500k. That is where
`Segmented`'s 4-byte-a-row vector passes this VM's 2 MiB L2 (1.2 MB at 300k rows, 2.0 MB at
500k). The inferred mechanism: Open's advantage is the cache misses it avoids, and a block that
fits in L2 has few to avoid. This was not measured with counters (no PMU on the VM).

## 3. What it is worth here

These are inverse-only totals for one file, averaged over the six runs. A block-size-aware rule
(`n < 400k`: Open only for r/n < 0.075, else `Segmented`; larger blocks unchanged) and the
per-block oracle min(Open, Seg) against production:

| level | production ms | rule | oracle |
|---|---:|---:|---:|
| 1 | 439 | -16.6% | -19.2% |
| 2 | 400 | -9.4% | -12.9% |
| 3 | 379 | -5.2% | -9.4% |
| 5 | 352 | 0.0% | -2.5% |
| 9 | 335 | 0.0% | -0.2% |

At -1 the rule's ~73 ms is about -11% of the whole 1T decode (best-of-5 `decode_with`, 617-737 ms,
scanner included; derived, not run). `Segmented` alone already measured 1.06-1.19x faster than
production on the whole -1 file. Wikidata -9 is untouched by construction.

## Caveats

- **The crossover is a cache property, and this machine's L2 is 8x the i9's.** On the i9-10900K
  (256 KiB L2, 20 MiB L3), `Segmented` never fits in L2, so the crossover may sit elsewhere or
  vanish at 1T. At 8 workers it is shared L3 that matters. CI's EPYC (1 MiB L2) is a third point. The
  rule's thresholds should come from those machines, not this VM.
- Memory: `OpenCells`' comment keeps Open's blocks off `Segmented`'s ~6.5-8.4 MiB/worker
  scratch at 8W (`<= 1.01x` RSS). For a 100k block that scratch is proportionally smaller, but a
  rule must still price 8W RSS on mixed-size streams.
- One corpus (wikidata text), one encoder (reference bzip2). A stream's last block is also short,
  but a single short block is negligible on large files.
- Single-thread kernel timings. There is no multi-thread A/B of a rule (that needs `src/open.rs`).

Follow-on claim: `open-admission-block-size` (idea, src/open.rs).
