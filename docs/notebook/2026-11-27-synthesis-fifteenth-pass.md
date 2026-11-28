# Synthesis, fifteenth pass: the shipped trim hides the scanner on i9 Wikidata, and the i9 has clock modes

Read across the i9 bundles of 27 Nov (`9de7b6e0` #1034, `77236472` #1119, `bce3914d` release-scaling,
`45ae3ff3` crossover) and the deltas merged since the fourteenth pass (#1158, `53f5c90`): #1161, #1162,
#1163, #1164, #1166, #1172, #1174–#1176, #1178, #1180–#1185, open #1179 and #1183. Raw `runs.jsonl`
re-read from the `evidence` branch; nothing re-run. Recurring claim `synthesise`.

**Not re-scored.** Every frozen verdict below stands (C2). Where a regrouping of raw runs gives a different
number from a recorded headline, it is labelled *regrouped* and is a reading of the bundle, not a new verdict.

## 1. The shipped trim puts i9 Wikidata deep in worker-bound territory: scanner levers paid nothing at 8T/10T

Three facts no single delta states together:

- **Measured i9 `k*`** (#1119 `station_balance`, pinned, post-#1034 default codegen, one Wikidata copy):
  `S = 35.24 ms`, `W = 333.48 ms`, `k* = 9.46`. `text 5.09`, `random 3.90`, `records 3.68`.
- **The shipped plan does not run 8 workers at 8T or 10T on the i9.** `trim_cell` gives 8T pinned → 6
  workers, ≥10T pinned → 7 (`TRIMMED_AT_EIGHT`, `TRIMMED_WORKERS`); release-scaling `thread_cpu` confirms
  `mibz2-0..5` at 8T and `mibz2-0..6` at 10T. Unpinned 12–20T runs 8.
- **Release-scaling 0.5.14 → 0.5.15 is flat at pinned 8T (1.002 [0.994, 1.010]) and 10T (0.996 [0.987, 1.005])**
  although 0.5.15 carries #1034's −10.5% serial scan.

Put together (*derived*): Wikidata worker demand at the shipped counts is `W/6 = 55.6 ms` (8T) and
`W/7 = 47.6 ms` (10T) against `S = 35.2 ms`. The scanner has **12–20 ms of slack per copy**. #1034's
−3.7 ms/copy landed inside the slack, so the bottleneck model predicts zero wall at 8T/10T, and the
release run measured zero. The +1.9%/+4.0% that #1034's recipe measured at "8/10" were in the bench
harness's untrimmed 8-worker configuration, which production does not use on the i9 at those counts.

Consequences:
- **Scanner levers on i9 Wikidata pay only at 1–2T** (serial share) until worker demand falls by about
  26% (re-tie at 7 workers: `W ≤ 7·S = 247 ms`) or 37% (6 workers: `212 ms`). That covers #1144
  (adaptive seam window), #1117 (A2 fast build), `scanner-a3-k3-pipe-u8`, and the slack-only hybrid.
  They still pay on text/random/records at production counts: `k* = 3.7–5.1` < 6–7 workers.
- **Worker levers pay one-for-one on Wikidata at shipped 8T/10T**, with 26–37% of headroom before the tie
  returns. That covers the block-tail cluster (§3), #1132 (MTF fold), `forward-emission-run`, and the
  A6c wire.
- **Recipe grid ≠ production plan.** Registrations that gate on "8 workers" or "10 workers" in an
  untrimmed harness measure a configuration that the i9 doesn't ship at those thread counts. Future
  registrations should state the plan (workers, kernel, pinning) they gate, and include the shipped trim
  as a cell.
- **The trim itself may be stale.** `TRIMMED_WORKERS = 7` predates #1034, OpenCells' calibration and the
  measured `k*`. At `k* = 9.46`, 10T with 8 workers would cut worker demand from 47.6 ms to 41.7 ms a copy,
  still above S, if contention lets per-worker cost hold. Nothing measures 8 workers on 10 pinned CPUs:
  the crossover's forced arm is `w = t`, which oversubscribes. That is an idea claim, `trim-at-measured-kstar`.
  It shares `src/parallel.rs` with `plan-schedule-and-lazy-threshold`, so it sequences behind that claim.

## 2. The i9 has clock modes, and they set the width of every hardware CI

Re-reading the raw rows by **pair position** and **mode**:

| Bundle, cell | Placement | Pattern in raw walls | Regrouped |
|---|---|---|---|
| #1119 `wikidata ×4`, 6T | unpinned (`cpus=all`) | **12/12 second-of-pair runs slow**: 0.353–0.366 s vs 0.241–0.250 s first; workers 0.32 vs 0.22 CPU-s, scanner 0.24 vs 0.16 (all threads ×~1.45) | position-matched aligned/default 1.008 (first), 1.001 (second); recorded 1.0064 [0.778, 1.302] |
| #1119, 8T and 10T | unpinned | mostly slow (0.28–0.33 s); first-of-pair 8T fast in 4/12 (0.195–0.208 s) | recorded CIs ±15% are this mixture |
| #1119, 1T | pinned CPU 0 | **11/12 first-of-pair runs slow** (1.448–1.461 vs 1.379–1.389 s, ×1.05) right after the previous round's 10T pair | position-matched aligned/default: second-of-pair **1.0054** (aligned 1.3844–1.3885 vs default 1.3789–1.3803, disjoint); recorded 1.0065 [0.975, 1.039] |
| #1034, 6/8/10T `wikidata ×4` | pinned (recipe) | unimodal, tight (8T 0.189–0.204 s) | — |
| release-scaling, pinned 4–10T | pinned | unimodal (8T 0.0652–0.0676 s) | — |
| release-scaling, **1T** | pinned CPU 1 | **bimodal**: fast ≈0.350–0.367 s, slow ≈0.390–0.405 s (×1.10). 0.5.14 slow in 8/10 runs, 0.5.15 in 3/10 | within-mode 0.5.14/0.5.15: fast **1.021**, slow **0.998**; recorded **1.057** [1.011, 1.104] |
| release-scaling, **12T** | unpinned | three clusters: ≈0.060, ≈0.065, ≈0.074–0.078 s; 0.5.14 has 3/10 in the slowest, 0.5.15 none | within-mode 1.006–1.02; recorded **1.095** [1.030, 1.164] |

What the regrouping shows:
- **The release's 1T and 12T headline gains are mostly mode counts.** The frequency-free measure in the same
  CHANGELOG entry, stage cycles a byte at 1T, is **−1.1% [−2.1%, −0.0%]**. That agrees with the within-mode
  ~0–2%, not the 5.7%. The expected serial sum (#1034's ~1.1% of 1T, #1080's 1.7% at 1T on Wikidata) is
  ~2–3%. 2T (1.034 [1.012, 1.055]) shows no clear modes and may be real. **The recorded 1T and 12T figures in
  `CHANGELOG.md` [0.5.15] and `docs/journal/release-attribution.md` should be restated** as mode mixtures
  with within-mode estimates. That belongs to the new claim `i9-clock-modes`, done by an agent other than the
  release's author. No flag is raised: a flag on `CHANGELOG.md` would block the P1 release now due.
- **Two phenomena, not one.** (a) A ~5–10% single-thread mode on pinned runs. Its size matches losing the
  top single-core turbo bin (5.3 → 4.8–5.0 GHz), and in #1119 it follows the heavy 10T pair. (b) A ~1.45×
  all-threads mode on *unpinned* ≥6T runs. That is the max-turbo/base ratio (5.3/3.7 = 1.43), and pinned
  bundles from the same day never show it. Both scale every thread's CPU equally. That is the signature left
  unresolved on 2026-10-28 (§4: slow after a 0.15 s gap) and 2026-11-09 (spin audit, ×2.2 all threads at
  10T). The new discriminators are **pinning** and **pair position**: #1119's harness uses `--settle 0.2` and
  `--prespin-ms 100`, and its unpinned 6T second runs are slow 12/12.
- **It sets the gate resolution.** A balanced-order paired design is unbiased in expectation, but the mode
  mixture widens 1T CIs to ±3% and 6–10T CIs to ±15–25%. Many registered gates are 1.5–3%. Stratifying by
  position, or classifying modes by cycles/APERF in the child, would have given #1119 a disjoint 1T answer
  (+0.5% slower, matching `station_balance`'s W +3.05% in direction).
- **Production relevance is open.** Users run unpinned. If mode (b) is a scheduler/frequency interaction and
  not a harness artefact (gap, prespin), the default decoder loses ~30% at 6–10T a good part of the time on
  this machine. That would be the largest lever on the board. The claim below makes the one discriminating
  run, with in-child cycles and placement, before anyone builds against it.

## 3. Four claims converge on the block tail (inverse RLE1 + CRC over the output)

| Claim | State | Paths | What it does to the tail | Measured (VM; JCC-32B column where the probe gave one) |
|---|---|---|---|---|
| `crc32-avx2-ring-barrett-build` | open, held, PR #1179 | `src/crc.rs` | 8-slot `vpshufb` ring + 8-lane CLMUL + table-free Barrett; frees the 16 KiB `TABLES` from L1d | CRC −16..−18% at 32–900 KiB; A9 −7.3% warm / −4.6% cold |
| `rle1-2load-filter-build` | idea | `src/rle.rs` | 2-load `find_runs` + raw-store `unrle` (0 `memcpy`/`vzeroupper`) | A9 −9.2% warm / −5.8% cold `wikidata ×4`; text A9 −16% |
| `segmented-fused-postwalk` | open, held, PR #1183 | `src/bwt.rs`, `src/rle.rs`, `src/lib.rs` | fused reverse + RLE1 + tiled CRC for the Segmented route | Segmented thread CPU −17..−19% (text) |
| `forward-rle1-crc-vectored-build` | idea | `src/open.rs`, `src/rle.rs`, `src/crc.rs`, `src/parallel.rs` | fuse RLE1 + tiled CRC into 4-lane forward emission | +3.8% warm / +2.4% cold of 1W (native build) |

Relations:
- **All four are worker-side and add no resident memory**, so by §1 they pay at shipped 8T/10T on i9
  Wikidata and pass the L3-footprint law (`≤ 1.01×` at 8W).
- **#1183 already calls `crate::crc::update_with`**, so it inherits #1179's ring CRC with no second CRC
  kernel. Keep it that way: fused paths call the crate CRC, never an inline copy.
- **`src/rle.rs` is the contention point.** #1183 edits it (+102 lines), and both idea claims own it.
  `rle1-2load-filter-build` should be claimed after #1183 merges and re-measured on top of it.
- **The A9 gains were each measured against the same pre-change A9**, and part of the CRC gain is L1d relief
  (`TABLES` eviction) that the RLE change also enjoys. Expect them to stack sub-additively. Whichever lands
  second re-prices A9 on top of the first; don't add the two figures.
- **Use the JCC-32B column as the i9 predictor.** #1119 showed the i9 has the erratum fixed in silicon, so VM
  base-build gains on branchy code include erratum relief the i9 never gets. Examples: #1175's mdjournal
  −24% (base) vs −13% (JCC), and #1183's `build_lf` store-combining −18.5..−20.9% (base) vs −3.9..−4.0% (JCC).
  The CRC and RLE probes hold up in the JCC column; `build_lf` mostly doesn't.

## 4. The 4-thread cell is EagerCells everywhere except the i9's own trim: one cause, three symptoms

`LAZY_FROM_THREADS = 8` (`plan-schedule-and-lazy-threshold`, #1182) means every non-i9 machine runs
EagerCells at 4–7T. Three records from today trace to it:
1. **#1034's CI acceptance flag** (`pattern` 4T +7.3%, resolved #1164). It sits in the EagerCells regime
   on the 4-vCPU CI runner, as does the bimodal `ready`-backlog RSS mode that #1137 found (`R² = 0.706`).
2. **The i9's lowest lbzip2 ratio is at 4T** (5.34× vs 6.2–8.5× elsewhere): `trim_cell(4) = (3, false)`
   is EagerCells too.
3. **The 5T/7T cliff** (#1182).

Consequence: once `plan-schedule-and-lazy-threshold` lands, CI's 4T arms switch kernel (OpenCells at
0.39–0.49× RSS). The CI acceptance bench's 4T baseline moves, and its noise model with it. One A/A run on CI
after that merge recalibrates PERFORMANCE-POLICY §2's 4T cells. Record it in that claim's delta, not as a
new process.

## 5. Smaller relations

- **#1161's STOP on standalone A6b, read with §1**, keeps the A6b door open only through the `edges`
  overlay (W1). That needs `unsafe` reinterpretation, so it is a B5 question, not a dead end: `C5` −1,752 KiB
  RSS and the best 8W cold result. Not opened here; #1161 recorded it.
- **Huffman self-sync (#1166) and the bench corpus:** `wikidata.bz2` is 12 streams (×4: 48), while the real
  dump is one stream. Scanner-side levers that use stream heads (seam windows, pooled scan starts) should
  keep a single-stream input in their guard set. This is a caveat for #1144 and the hybrid, not a claim.
- **Release due (P1):** #1175 and #1176 are on master since v0.5.15, and no `release-*` claim exists. The
  next release's scaling run is the natural first user of `i9-clock-modes`' estimator, and should report
  within-mode figures beside the pooled ones.

## Claims opened (idea)

- `i9-clock-modes` (P2): one pre-registered i9 run separating pinning × settle gap × pair position, with
  per-run cycles/APERF and placement; then a mode-robust estimator for `ab` comparisons and restated
  0.5.15 1T/12T records.
- `trim-at-measured-kstar` (P3, `src/parallel.rs`, after `plan-schedule-and-lazy-threshold`): test 8
  workers at 10T pinned against the shipped 7, using the measured `k* = 9.46`.
