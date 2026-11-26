# Synthesis, fourth pass: the hardware-floor law, the post-K=3 scan path to 6 GB/s, and forward emission as a four-way keystone

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), the fourth pass of 26 November. Follows [the first](README.md) (#954), [the second](second-pass.md) (#965), and [the third](third-pass.md) (#983). Analytical only: no new runs, no `src/` edits, no frozen gate altered or reopened (C2). Every measured figure is cited to its primary file; arithmetic done here is marked **(derived)**; VM-to-i9 ratios are marked **(projected)**.*

## What was read

- **Deltas since #983 (plus #982, unread by #983):**
  - `#982` `explore-fold-chain-and-pipeline-edges` (falsification try on `mtf-fold-build`; pipeline fill/drain null);
  - `#984` `floor-throughput-overview` (4.31 GB/s shipped vs 6 GB/s ≈ 140 ms milestone);
  - `#985` / `#992` `ideal-permutation-structures` (product-value PASS for `F1a`/`F1b`/`F4a`; conjugation & post-Open row contraction STOP; opened `bwt-digram-return-map`);
  - `#986` `multi-symbol-huffman-table` (STOP on `B=10` `K=3`/`K=4`; incidental `-11.9%` native BMI2 speedup on shipped `K=2`);
  - `#988` `single-pass-rle1-crc` (STOP on standalone RLE1+CRC; 72.3% compute floor; points to upstream fusion into `forward-emission`);
  - `a9bfd00` / `#998` `permutation-patterns` (SCOPED FAMILY STOP on uncontracted encodings of Open's `LIMIT=16` map $T$);
  - Tooling & governance: `#974` (`3ced2be`), `tooling-flags-resolved` (`c9d25bb`), `max_live_holds = 6` (`1de637f`), `swarm-reviews-968-977-984-985`, `#977` / `#996` / `#997` (`archive-team-strata`, lease identity), `#999` (`admit-p2p3-probes`: admitted `bwt-digram-return-map` and `worker-l3-footprint`).
- **Primary files checked:** `multi-symbol-huffman-table.md`; `rle1-crc-census.md`; `ideal-permutation-structures/results.md`; `permutation-patterns/results.md`; `scan-lookups-and-runts.md`; `ideal-parts/results/floor/README.md`; `pooled-scan/results/report.txt`; `src/huff.rs`; `src/lib.rs:156-193`; `frontier/surface.tsv`; `frontier/dead-ends.tsv`; every claim in `frontier/claims/` and `frontier/blog/`.

---

## 1. One structural law across four consecutive STOPs (`#958`, `#986`, `#988`, `#992`, `#998`)

Read individually, the recent verdicts look like four unrelated negative probes on four separate stages. Put side by side, they all hit the **same physical barrier**: shipped `mibz2`'s inner loops (`K=2` 4-byte LUT in `src/huff.rs`, 1-load `edges[j]` in `src/open.rs`, AVX2 + 64K-tiled PCLMULQDQ in `src/rle.rs`) already sit close to their single-pass / single-load instruction floor for their current work count. Re-encoding tables, reordering passes, or compressing navigation without reducing the underlying count of symbols, visits, or compacted bytes fails every time; only **eliminating an intermediate pass or reducing visit count upstream** (or removing a critical-path ALU cycle / offloading to idle cores) clears the 3% wall bar:

| stage | work-preserving attempt | why it STOPPED (measured floor) | upstream / orthogonal lever that breaks the floor |
|---|---|---|---|
| **S (`#986`)** | `B=10` `K=3`/`K=4` root tables (same 50-sym groups & 24.98M writes) | Lookups `-12.4%`, but group-tail amplification (`97.3%` vs `43.9%`) caps zero-store ceiling at `-6.1%` (`< 8%` gate) and 3-sym writes add `+31%` store/extract uops (`+1.1%` base / `-1.1%` nat) | Remove 1 critical-path cycle/lookup via runtime BMI2 `shlx` dispatch (`-11.9%` native on shipped `K=2`, unclaimed) OR offload scan CPU to worker slack (`pooled-scan-hybrid`) |
| **C-A6b (`#958`)** | Induction order, `LIMIT` sweep, SLP-in-emit, prefetch/interleaving | `LIMIT=16` is already optimal; `lit%` stays `81.6-82.2%` down to `LIMIT=2`; `50%` `join()`, not pointer-chase bound | Eliminate the `124.5 MiB` `runs` intermediate (`mtf-fold-build`, subject to `#982`'s AVX2 overlap check) |
| **E-walk (`#992` F2, `#998`)** | Row conjugation (`F2a`/`F2b`) & pattern encodings of unchanged $T$ (`P1_strict_run`, `P2_mono_B16`, Barbay–Navarro) | Same `13.53M` visits & `81.2%` literals; shrinking `nav` `22-40%` cannot repay `+0.16..+1.00` dependent loads/visit (`1.02-2.32×` slower in every regime) | **Contract the labelled return map** (`F1a`/`F1b`/`F4a` → `bwt-digram-return-map`): cuts visits `0.21-0.34×`, collapses `lit%` `81.2% → 6.5-8.3%`, AND pulls `edges` into 256 KiB L2 (`93.6-152.7 KiB/b`) on the unchanged 1-load consumer (`2.8-4.4×` faster) |
| **R (`#988`)** | Standalone single-pass in-place / backward / OOP RLE1 + CRC | Same `800 MiB` scan + `726 MiB` unavoidable compaction writes (`89.4%` `extra==0`) + `800 MiB` CRC; L1d-hot `crc32 + scan` alone is `72.3%` of A9 (`-27.7%` ceiling `< 35%` gate; best `-3.5%` warm / `-6.1%` cold) | **Fuse RLE1 + tiled CRC32 upstream into `forward-emission`** (eliminating post-BWT scan + `726 MiB` compaction; ceiling `-57.0%` of A9 = `38-42 ms` 1W, `~7.3 ms/w` 8W **(derived)**) |

---

## 2. The 8-worker path to 6 GB/s (~140 ms) after `#986`'s scan STOP and `#982`'s fold caveat

`second-pass.md` §1 counted on `multi-symbol-huffman-table` for `17-21 ms` off the `177-179 ms` serial scan, and on `mtf-fold` (`6-13 ms/w`) + 1-lane `forward-emission` (`18-20 ms/w`) for `S′(8) = 152-161 ms`. Three results since `#983` rewrite that roadmap:

1. **Row `S` lost its only serial-algorithmic claim (`#986`), leaving an unclaimed ISA-dispatch lever.**
   - `#986` closes `B=10` multi-symbol tables beyond `K=2`, leaving `pooled-scan-hybrid` as the only claim on `S`.
   - Yet `#986` §3b/§4 observed incidentally that compiling **shipped `k2_shipped`** with `-C target-cpu=native` cut A3 by **`-11.9%`** (`76.852 → 67.699 ms`, non-overlapping `[75.46..78.50]` vs `[65.72..69.86]`). Inspection of `src/lib.rs:189-193` and `src/huff.rs:290-390` confirms `Kernels::detect()` checks `avx2 && bmi1` and `pclmulqdq`, **not `bmi2`**, and `src/huff.rs` has zero `#[target_feature]` dispatch: every `r.skip_short_shl(e)` in `decode_group_win` compiles to baseline x86-64 `shl %cl` (3 uops, 2-cycle latency on Intel Haswell through Comet Lake) on the critical ~7-8-cycle dependent load+shift chain (`~13.2` cycles/lookup on i9, `scan-lookups-and-runts.md`).
   - With A3 ≈ 81% of the i9 scan (`~144-145 ms`), saving 1 cycle/lookup (`7.6%`) to `#986`'s native ratio (`11.9%`) bounds a pure BMI2 dispatch cut at **`10.9-17.3 ms`** off the `177.5 ms` serial scan (`scan′ ≈ 160.2-166.6 ms`, **projected/derived**), with zero table bloat or algorithmic risk. Because `#986` enabled all native flags at once, an ablation separating isolated `bmi2` (`shlx`) from `bmi1`/AVX2/scheduling is needed first → opened `huff-bmi2-dispatch` at `status = "idea"`.
2. **`#982` turns `mtf-fold`'s 6-13 ms/w into an overlap bet — while `#992`/`#998` decouple the 8W roadmap from it via 4-lane `forward-emission`.**
   - `#982` showed `mtf_fold_census`'s `27.6%` (`6-13 ms/w`) relied on hiding `OutQ16` behind `MtfU128`, which runs `3.0-3.1×` slower than production `prod_a5_avx2`; without that overlap, the conservative floor is `1.5-3.7%` of 1W wall (`~3-6 ms/w` at 8W), straddling K1's `2.5%` bar. Because `#968` (`mtf-fold-build`) already implements the AVX2 sink fold behind `cfg(feature = "mtf-fold")`, it should measure folded AVX2 vs `prod_a5_avx2_plus_a6a` locally before queueing an i9 run.
   - Meanwhile, `#992` and `#998` both measured **prebuilt 4-lane forward emission** (`ctrl_shipped_fwd_4lane`, `MLP=4`) on held-out blocks: `A6c_fwd` saves `97.2-105.1 ms` at 1W (`12.2-13.1 ms/w` at 8W) and 4-lane `E` saves `+117.9..+165.1 ms` warm over `ctrl_shipped_bidi` (`+16.7..+23.4 ms/w` at 8W via `69.5/490.3`), totalling **`215.1-270.2 ms` at 1W (`16.5-20.7%` of 1,306 ms) and `28.9-36.5 ms/w` at 8W** before sampled-anchor overhead and lane imbalance **(projected/derived)**.
3. **Updated work-conservation table (`third-pass.md` §2: `S(8) = 184.9 ms`, `scan = 177.5 ms`, `hybrid = (8·S′ + scan)/9`, `P′ = S′ + scan/8`; `838.8 MB / wall`):**

| worker-side stack (`k = 8`, **derived/projected**) | `S′(8)` ms | hybrid (`scan = 177.5`) | hybrid (`scan′ = 160-167`, BMI2) | built `P′(8)` ms |
|---|---:|---:|---:|---:|
| Today (shipped) | 184.9 | 184.1 ms (4.56 GB/s) | 182.2-182.9 ms (4.59-4.60 GB/s) | 207.1 |
| `mtf-fold` (`3-13`) + 1-lane `fwd` (`18-20`) | 151.9-163.9 | 154.7-165.4 ms (5.07-5.42 GB/s) | 152.8-164.2 ms (5.11-5.49 GB/s) | 174.1-186.1 |
| **4-lane `fwd` alone** (prebuilt, pre-anchor: `28.9-36.5`) | **148.4-156.0** | **151.6-158.4 ms** (5.30-5.53 GB/s) | **149.7-157.2 ms** (5.34-5.60 GB/s) | 170.6-178.2 |
| **`mtf-fold` (`3-13`) + 4-lane `fwd`** (prebuilt) | **135.4-153.0** | **140.1-155.7 ms** (5.39-5.99 GB/s) | **138.2-154.5 ms** (5.43-6.07 GB/s) | 157.6-175.2 |
| Above + `bwt-digram-return-map` (`+20..+35 ms/w` net) | 100.4-133.0 | 109.0-137.9 ms (6.08-7.70 GB/s) | 107.0-136.7 ms (6.13-7.84 GB/s) | 122.6-155.2 |

**Three immediate consequences:**
- **4-lane `forward-emission` ALONE crosses `third-pass.md` §2's `158-165 ms` trigger for `pooled-scan-hybrid`**, even if `mtf-fold` contributes zero.
- **Reaching `#984`'s 6 GB/s (`~140 ms`) milestone requires BOTH worker-side contraction/4-lane MLP AND `pooled-scan-hybrid`** (4-lane `fwd` + `mtf-fold` + hybrid reaches the `140 ms` edge; adding `bwt-digram-return-map` or `huff-bmi2-dispatch` clears it with margin).
- **Without `pooled-scan-hybrid`, `huff-bmi2-dispatch` is the only known lever that moves the serial `177-179 ms` wall itself** once `S′(8) < 160 ms`.

---

## 3. `forward-emission` is now a four-way keystone — with an architectural trade-off between 4-lane MLP and inline RLE1+CRC fusion

Putting `#954`, `#988`, `#992`, and `#998` together reveals both a major upgrade and a subtle design tension inside `forward-emission`:

1. **Multi-lane is mandatory on uncontracted Open (`#992` & `#998`):**
   - `forward-emission.toml` still reads "the multi-lane share is still unmeasured." Both `#992` and `#998` have now measured it on held-out blocks: on uncontracted Open (`edges = 455.5 KiB/blk`, spilling 256 KiB L2), **single-lane forward (`MLP = 1`) actually regresses vs shipped bidirectional (`MLP = 2`) in cold/L3-pressured mode** (`187.6 vs 181.4 ms` base / `201.8 vs 173.2 ms` nat in `#992`; `185.0 vs 176.5` / `191.5 vs 169.8 ms` in `#998`) and is flat on native warm `E` (`117.8 vs 120.4` / `114.7 vs 114.0 ms`). Almost all of `#958`'s `+142-157 ms` single-lane `prod_forward_emit` win came from `A6c` (`~100 ms`) and baseline x86-64 `E`.
   - **4-lane forward (`MLP = 4`)** reverses that completely (`0.731-0.818×` warm and `0.643-0.697×` cold vs 1-lane fwd), delivering `+118-165 ms` warm (`+121-151 ms` cold) on `E` alone on top of `A6c`.
2. **Fourth convergence (`#988` → `forward-emission`) and the lane-boundary tension:**
   - `#988` proves standalone `R` (`A9`) cannot break its 72.3% `crc32 + scan` floor; eliminating `find_runs` (`40.0%`), `shift-up` (`5.3%`), and `726 MiB` compaction (`22.1%`) leaves `crc32_only_floor` (`61.75 vs 143.49 ms` = `-57.0%` of A9, ceiling `~38-42 ms` at 1W / `~7.3 ms/w` at 8W, plus removing `726-916 MiB` of L3 write traffic on `×4` **(derived)**).
   - **The hidden tension:** inline RLE1 expansion + sequential CRC32 is trivial in **1-lane forward** emission (one sequential output cursor `0..n`, zero shifting), whereas in **4-lane forward** emission each lane's RLE1 output length differs from its pre-RLE1 span (`89.4%` of runs shrink by 1 B; `total_grown` spans `-1,183..+191 B`) and a 4-byte run can straddle any of the 3 lane boundaries! Stitching 4 expanded slices back into one contiguous buffer would re-introduce `~75%` of compaction traffic unless either (a) `StreamDecoder` lends per-lane slices, (b) lanes expand only when run-free / track boundary run state + GF(2) `crc32_combine` (`#988`), or (c) **contracted return maps (`F1a`/`F1b`) make 1-lane emission nearly as fast as 4-lane**: in `#992`, once `edges` shrinks into 256 KiB L2 (`93.6 KiB/blk`), 4-lane adds only `7-8%` warm (`+8-10 ms` i9) over 1-lane `F1b` — far smaller than the `~38-42 ms` 1W upstream RLE1+CRC fusion prize! Moreover, in a contracted dictionary (`118-337 KiB/blk`), multi-byte words can be pre-flagged for internal RLE1 runs once at dictionary build time, leaving only 3-byte boundary carry checks between emitted words.
   - **Action:** Sequence RLE1+CRC fusion cleanly as its own smooth-zone follow-on probe (`forward-rle1-crc-fusion`, `status = "idea"`) rather than complicating `forward-emission`'s initial `src/open.rs` build.

---

## 4. A mathematical bridge from `#998` to `bwt-digram-return-map` (currently open)

`#998` stopped uncontracted encodings of Open's `LIMIT=16` map $T = \Psi_R$, while `#992` opened `bwt-digram-return-map` to construct visit-reducing contractions (`F1a`/`F1b`) in BWT/bucket space (after proving post-Open row-graph contraction `mat_f3b_post_open` costs `394-408 ms` = `3.26-3.42×` Open `C`). Putting `#998`'s exact census beside `#992` directly narrows the constructor design space:

- **Why forward (`Ψ_R` / band) space beats backward (`LF_R`) space by 33×:** `#998` measured `nRuns(T)/m = 0.00835` (`857` runs) and `nLRuns(T)/m = 0.01581` (`1,628` labelled ascending runs, mean **63 rows**, `98.35%` contiguous 3-grams = `123` and `98.68%` label-homogeneous) across `3,548` bands — **`33×` fewer ascending runs than `T⁻¹ = LF_R` (`0.27582`)**.
- **Why `#958`'s `LIMIT` sweep stalled while `F1a`/`F1b` collapsed `lit%` `81.2% → 6.5-8.3%`:** `#998` found `corr(h1/m, max_prime/m) = +0.989` (`h1/m = 0.4118` width-1 thin runs form the irreducible prime core). Open's width-`LIMIT` induction eliminates only *thick L-runs* (`width > LIMIT`), leaving the thin-run literal core untouched. Conversely, eliminating **high-frequency literal `F`-buckets** (a contiguous interval `[F_R[b], F_R[b+1])` in `Ψ_R`, where top bytes of `σ = 87` generate most of the `81.2%` 1-byte literal visits) operates directly on the `~1,628` labelled ascending runs / `~3,548` bands (`~30-60×` fewer objects than `m = 98,905` rows!), bypassing `mat_f3b_post_open`'s `13.53M`-edge bottleneck.
- **Note for the live holder of `bwt-digram-return-map`:** not editing the held claim's TOML here; this bridge is recorded here for its constructor phase.

---

## 5. Cross-cutting methodology finding: baseline x86-64 vs `-C target-cpu=native` flips probe rankings

Across all four probes since `#983`, compiling with baseline x86-64 vs `-C target-cpu=native` changed magnitudes or signs by `10-28%`:
- **`#986` (`S`):** native cut shipped `k2_shipped` by `-11.9%` (`76.85 → 67.70 ms`, BMI2 `shlx`) — larger than the 8% gate `K=3` failed.
- **`#988` (`R`):** `shipped_oop_avx2_sep_crc` appeared `-14.5%` faster on baseline (runtime AVX2 vs SSE2 safe mirror) and flipped to `+13.6%` slower once both ran AVX2 under native.
- **`#992` & `#998` (`E`):** single-lane `ctrl_shipped_fwd` beat `ctrl_shipped_bidi` by `-9.1%..-11.7%` warm on baseline, shrinking to `-0.7%..+0.6%` under native; hardware `popcnt`/`lzcnt` cut `P1_strict_run_fwd` by `-25.6%` (`245.9 → 183.0 ms`).

**Consequence:** Because shipped `mibz2` compiles for baseline x86-64 with runtime `Kernels::detect()` covering only `avx2+bmi1` and `pclmulqdq`, whereas `#![forbid(unsafe_code)]` probes cannot use `#[target_feature]` directly, **every future smooth probe should report both baseline x86-64 and `-C target-cpu=native`** (as `#986`, `#988`, `#992`, `#998` did), and any `-10%+` native-only win on a shipped kernel is itself a candidate runtime-dispatch lead (`huff-bmi2-dispatch`).

---

## 6. Dead ends re-checked & claim hygiene

- **Dead ends (`frontier/dead-ends.tsv`, rows 1-21):**
  - **Rows 15-21 (added since #983)** all stand strictly under C2: pipeline fill/drain (null), `multi-symbol-huffman-table` (STOP — `huff-bmi2-dispatch` is an orthogonal ISA-dispatch lead on shipped `K=2`, not a revival of `K=3`/`K=4`), `single-pass-rle1-crc` (STOP standalone — upstream fusion is a separate `forward-rle1-crc-fusion` claim), row-renumbering conjugation & post-Open row-graph contraction (STOP, reinforced by `#998`), git-global lease identity (STOP), and uncontracted permutation-pattern encodings of Open's `T` (STOP, leaving `bwt-digram-return-map` open).
  - **Row 6 (`pooled scan, as built`, KILL)** stands (C2); §2 sharpens its quantitative revival condition (`P′(8) = 158-175 ms` once 4-lane `fwd` + `mtf-fold` land).
  - **Row 12 (`rauzy-output LIMIT sweep`, STOP)** is now structurally explained by `#992` Part A + `#998`'s `corr(h1/m, max_prime/m) = +0.989`.
- **Claim hygiene (unheld items only; live holds left untouched):**
  - Opened two smooth `idea` claims with frozen stop tests: `huff-bmi2-dispatch` (P2) and `forward-rle1-crc-fusion` (P3).
  - Annotated `forward-emission.toml` (`why` + `premises`: `#992`/`#998` 4-lane measurements & cold-mode `MLP=1` warning) and `pooled-scan-hybrid.toml` (`question` note + `premises`: `#986` STOP, `#982` fold floor, 4-lane `S′(8)` trajectory). Zero `stop_test` fields modified.
  - Updated `frontier/surface.tsv` row `S` only (adding `huff-bmi2-dispatch` and noting BMI2 `shlx` `-11.9%` native on shipped `K=2`), leaving rows touched by live holds (`C-A6a`, `M`, `E-walk`, `E-spelling`, `R`, `D`) untouched to avoid merge conflicts.
  - **Stale operational pointers noted for next ops/swarm pass (not edited here):** `harden-frontier-tooling.toml` still reads `status = "review"`, `pr = 974` (merged as `3ced2be`); `hold/archive-team-strata` awaits `finish-review` (#977 merged as `db83e59`); `#968` (`mtf-fold-build`) awaits `surface.tsv` conflict resolution + `#982` local AVX2 check; `bottlenecks-floor-correction` (`#975` live) awaits an independent post-publication review to close to `published`/`done`.
