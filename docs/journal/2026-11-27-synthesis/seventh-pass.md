# Synthesis, seventh pass: worker-side rescues, 4-lane vectored convergence, joint L3 footprint, and post-dissection queue reordering

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), 27 November 2026. Follows the [sixth pass](sixth-pass.md) (`6f8a26f`, #1037). Analytical and read-only across open PRs: no `src/` edits, no new hardware runs, no frozen gate reopened (C2). **(derived)** marks cross-source arithmetic or conditional projections kept strictly separate from established i9 measurements per [`frontier/DECODER.md`](../../../frontier/DECODER.md).*

## Primary sources read (post-`6f8a26f` / #1037)

- **Living state:** `frontier/DECODER.md`, `frontier/surface.tsv`, `frontier/dead-ends.tsv`, `docs/team/PERFORMANCE-POLICY.md`, `docs/journal/2026-11-26-synthesis/worker-l3-footprint.md`.
- **Merged since sixth pass (`origin/master` @ `0eaf87f`):**
  - `#1038` (`3bb552a`): `frontier/deltas/2026-11-27-bwt-contraction-constructor-rescue.md` (`bwt-contraction-constructor-rescue` scoped STOP).
  - `#1040` (`2915d7e`): `frontier/deltas/2026-11-27-dense-cap-precheck.md` (`dense-cap-precheck` shipped in `src/open.rs`).
  - `#1042` (`9f89ba5`): `docs/journal/2026-11-27-explore/segmented-fallback-dissection.md` (`segmented-fallback-dissection` PASS → `segmented-fallback-build`).
  - `#1046` (`05c13b9`): `docs/journal/2026-11-26-explore/a6b-combined-redesign.md` (`a6b-combined-redesign` PASS → `a6b-endpoint-build`).
  - `#1049` (`87401be`), `#1050` (`f2845b8`), `#1051` (`0eaf87f`): claims `segmented-fallback-build.toml`, `forward-emission.toml`, `forward-4lane-anchor-probe.toml`, `a6b-endpoint-build.toml`.
- **Open PR snapshots inspected directly (read-only refs; unmerged and subject to review):**
  - `#1045` (`d31f328f08b8`): `docs/journal/2026-11-27-explore/forward-rle1-crc-fusion.md`, delta, `forward-rle1-crc-vectored-build.toml`.
  - `#1048` (`c59541a25a78`): `docs/journal/2026-11-27-explore/mtf-fold-and-outq-dissection.md`, delta, `mtf-fold-l1-batch-redesign.toml`.
  - `#1041` (`e33e54b5bc97`, all-green code commit `81ef7e6cc961`): `frontier/deltas/2026-11-27-mtf-fold-run-recipe.md`.
  - `#1047` (`f1ff20ca349e`): `frontier/deltas/2026-11-27-release-terminal-holds.md`.
  - Plus review-parked `#1031` (`93eab0930056`), `#1033` (`3fb0937e2ce0`), `#1034` (`d8b47e0ce236`) tracked from pass 6.

---

## 1. Anatomy of three worker-side rescues: why `A6b` passed, `F`-bucket contraction stopped, and `#1010` must be redesigned before its i9 run

Three results since `#1037` form a natural controlled comparison of rescue engineering (`CLAUDE.md`, 27 November):

| Line | Prior result | Cold-dissection finding | Outcome & structural reason |
|---|---|---|---|
| **`A6b` (`#1046`)** | 4 standalone STOPs (`rauzy-census/results-2`: induction order, LIMIT, SLP-in-emit, prefetch/inline7-alone) | Cost split across 3 co-dependent sub-phases (`setup` 19% random gathers, `steps` 51% rule alloc + bounds checks, `dict` 30% zero-fill + hash) | **PASS (`comb_nobc_soa_gen`, 0 `unsafe`):** co-design cuts `A6b` `−27.4%..−28.8%`, full `C+E` `−5.4%..−7.6%` (`+48.7..+67.7 ms` 1W i9-anchored, `+8.2..+9.1 ms/w` 8W), `A7` `−0.44%..+0.32%`. |
| **Post-`A6c` `F`-bucket contraction (`#1038`)** | `#1013` STOP (`1.14×` vs 1-lane, `1.29–1.33×` vs 4-lane) | Removing `72.4%` (`27.3 MiB`) redundant `c0 == LONG` copies cut `contr` `−9.0%..−13.9%`, but `67.7%` of 4-lane iters terminate a lane (`56.4%` K→K) and `A6c` fusion breaks SIMD `Reader::wire` | **Scoped STOP:** bare `prep_tag + pure_chase4` (`30.6 ms` VM) already exceeds the `~24–25 ms` total budget to beat 4-lane oracle by 3%. Structural obstruction on `edges[0..m]`. |
| **`mtf-fold` (`#1010` + `#1048`)** | `#1010` merged untimed; `#1041` built `bench-mtf-fold` | Real `wikidata` timing found `#1010` **regresses** `wkr_cpu_ms` `+1.61%` baseline (7/7 slower) / `+4.33%` native (`+1.71% / +3.81%` at `k=2`): direct per-run `FoldSink` overflows 15 GPRs (reloading `sink` 10×/lit base; spilling both MTF registers `f0`/`f1` every literal native), runs a `0 / 15.14M`-hit `c == self.c` check, and triggers `417,041` `vzeroupper` `push_slow` calls (`84.3%` pure `f+16 > 512` tail-overrun guards) | **Rescuable implementation pathology, not an algorithmic bound:** `#1048` proposes `mtf-fold-l1-batch-redesign` (`1 KiB` L1d `[u32; 256]` packed maximal-run batch decoupling `front_avx2` from `Pass::take + OutQ16`, plus `OutQ16` `496/512` or `1008/1024` guard zone cutting `push_slow` `6.2×–10.3×` with `0` overflow chunks). |

**Immediate consequence for `#1041` (`mtf-fold-run`):** `#1041` is 31/31 CI + Miri green (`81ef7e6cc961`) and correctly paused dispatching `ready/bench-mtf-fold` after `#1048` landed. Do **not** spend an i9 workstation slot testing `#1010`'s known-spilling loop. Merge `#1041` after independent B5 review to land the reusable zero-copy `S`/`M`/`C` oracles + `bench-mtf-fold` harness (and immediately release `src/runs.rs` for `segmented-fallback-build`), then run `mtf-fold-l1-batch-redesign` through its free local VM + `objdump` zero-spill gate before queuing the i9 kill test under `mtf-fold/registration.md` §3 (frozen K1–K4 untouched).

---

## 2. Revived upstream fusion and the three-claim convergence on 4-lane vectored emission

`single-pass-rle1-crc` (`#988` STOP) hit a `72.3%` L1d-hot compute floor when scanning and CRCing an already-emitted `879 KiB` block in a separate pass (`−3.5%..−6.1%` A9, `−20.8%` writes). Open PR `#1045` (`forward-rle1-crc-fusion`, `d31f328f08b8`) **revives RLE1+CRC fusion upstream inside 4-lane forward emission** and exposes two hard architectural constraints that directly govern `forward-emission`:

1. **Only 4-lane vectored tile fusion clears the gate:** `fused_4l_tile4096_vec` (chunked 4-lane forward emit + zero-scratch in-place tile RLE1 compaction + 64 KiB tiled PCLMULQDQ CRC32 + GF(2) `crc32_combine` + zero-stitch 4-slice delivery; `13,360/13,360` byte/CRC-exact; `0/431` boundary straddles on `wikidata`) cuts `wikidata ×4` logical `E + A9` writes **`−53.4%` (`1,718.2 → 799.9 MiB`)** and beats unfused 4-lane forward by **`+49.9 ms` (`+3.82%` of `1,306 ms` wall) warm / `+31.8 ms` (`+2.43%`) cold** on matched-AVX2 native (and `+46.4 ms` warm on baseline despite SSE2 handicap).
2. **Both 1-lane fusion and contiguous 4-lane stitching STOP:** 1-lane fused loses `−73..−186 ms` vs 4-lane unfused (MLP=4 dominates); contiguous 4-lane stitching triggers `memmove` on `74.9%` of lanes (`86.4%` change length), shrinking the write cut to `−18.6%` and flattening timing (`−23..+11 ms`).

**Architectural convergence across three claims:** `forward-4lane-anchor-probe` (live hold, pricing paid sampled-anchor `A6c + A7` in both vectored and stitched modes), `forward-emission` (open, `src/open.rs` + `src/open/outq.rs`), and `forward-rle1-crc-vectored-build` (`idea` in `#1045`, `src/open.rs` + `src/rle.rs` + `src/crc.rs` + `src/parallel.rs`) are **three stages of one 4-lane vectored pipeline**:

- If `forward-emission` hard-bakes contiguous stitching into a single `out: &mut Vec<u8>` buffer, `forward-rle1-crc-vectored-build` will immediately have to undo that stitch and re-touch `src/open.rs` and `src/parallel.rs`.
- **Clean vectored boundary (zero public API breakage):** `Open` emits up to 4 disjoint slices inside a single recycled block buffer (retaining `span/4` pre-allocated slack per lane so `text.bz2` sees `0` overflows). `BzReader::fill_buf` drains the 4 slices sequentially (lane 0 → 1 → 2 → 3) with zero extra copy and zero change to `BufRead` semantics; `parallel::decode` scatter-copies the 4 slices directly into the caller's output slice at their exact post-RLE1 offsets; small `no_lanes` blocks (`pattern.bz2`) bypass tile fusion onto the shipped path.
- Wait for `forward-4lane-anchor-probe` to report paid sampled-anchor cost (and vectored vs stitched delta) before starting `forward-emission`'s striated build on `src/open.rs`.

---

## 3. Joint L3 working-set and RSS accounting at `k = 6..10` **(derived)**

`worker-l3-footprint.md` established that on the 20 MiB L3 i9-10900K, `k · W_worker` (`W_active = 0.845 MiB`, `W_resident = 0.897 MiB`, `W_peak(A7) = 1.352 MiB`) drives zero-SMT `k = 6..8` IMC DRAM growth (`930 → 1,602 MiB`, `+672 MiB`, `R² = 0.991` vs `ready_hwm` `0.407`). Evaluating each new worker mechanism's L3 and RSS effect together reveals an offsetting balance that no single claim prices:

| Mechanism | Stage(s) affected | Per-worker touched L3 delta **(derived)** | `wikidata ×4` DRAM/L3 traffic delta **(derived)** | RSS / allocation caution |
|---|---|---|---|---|
| **Baseline (`worker-l3-footprint.md`)** | A5..A9 | `0.897 MiB` resident (`1.352 MiB` A7 peak); `k=8`: `7.18 MiB` res / `10.82 MiB` peak | `1,718.2 MiB` `E+A9` writes + `124.5 MiB` `runs` | `1.492 MiB` `Scratch` HWM/w (`4.120 MiB` incl. `out`+`syms`) |
| **`a6b-endpoint-build` (`#1046`, live hold)** | A6b (`16.5%` time) | **`+273 KiB` avg / `+420 KiB` max** unaliased SoA (`links`+`widths`+`words128`, vs shipped `282 KiB` aliased inside `edges`); stops pre-warming `edges` for `wire_xor` (0 ms drift on fwd `A6c`) | Small positive during A6b (`~+2.2 MiB` across 8W if synchronous; `~+0.36 MiB` time-weighted) | **`1.75 MiB/w` virtual (`14 MiB` @ 8W, `17.5 MiB` @ 10W):** avoid full-64K zero-fill on alloc (`vec![0; 65536]` faults all `1.75 MiB/w`; lazy/zero-page-clean init or touching only `0..nodes` keeps RSS `~420 KiB/w`, `~3.4 MiB` @ 8W) |
| **`mtf-fold-l1-batch-redesign` (`#1048` `idea`)** | A5 + A6a (`25%` time) | **`−138.8 KiB` mean / `−363.8 KiB` max** (`runs` + `counts`/`per` replaced by `1 KiB` L1d `[u32; 256]`) | **`−124.5 MiB`** L2-spilling `runs` writes/reads eliminated | Removes `361.8 KiB/w` persistent `runs.runs` HWM |
| **`forward-rle1-crc-vectored-build` (`#1045` `idea`)** | A7 + A9 (`46%` time) | **`+~220 KiB` transient slack** during A7 (`span/4` per lane → `~1.10 MiB` vs `0.88 MiB`), **eliminates entire `879 KiB` post-A7 `A9` pass** | **`−918.3 MiB`** (`−53.4%`) logical `E+A9` writes + removes L3-cold `A9` reread of `879 KiB/blk` | Must check `k=8..10` peak RSS with `1.25×` lane buffer (`+1.8 MiB` @ 8W; well under 10% / 64 MB policy cap) |

**Net takeaway:** Tested in isolation, `a6b-endpoint-build` slightly widens A6b's live cache footprint (`+273–420 KiB/w` unaliased, which its i9 registration must monitor at `k = 8..10`). Composed with `mtf-fold-l1-batch-redesign` (`−124.5 MiB`) and `forward-rle1-crc-vectored-build` (`−918.3 MiB` `E+A9` writes), the combined worker stack removes **`>1.0 GiB`** of L2/L3 traffic across `wikidata ×4` — directly attacking the `k · W_worker` L3-spill mechanism that drives `A9`'s `2.55×` blow-up at `k = 8..10`.

---

## 4. Updated coupled 1W and 8W trajectory: workers can drop below serial scan even without `mtf-fold`

Keeping established i9-10900K measurements (`wikidata ×4`, 839 MB) strictly separate from conditional VM-anchored projections **(derived)**:

- **Established i9-10900K baselines (`c71c4a56`, `cc8ba137`, `results-2`, `#1035`):**
  - 1W wall = `1,306 ms` (`C + E = 1,068.0 ms`: `A6a 200.3`, `A6b 216.0`, `A6c 161.4`, `A7 490.3`; `M = 132.4 ms`; `R = 66.6–74.4 ms`).
  - 8W shipped wall `none(8) = 192.9–194.6 ms`; scan-bound `C(8) = 183.2–184.4 ms`; worker-bound `S(8) = 184.9–185.0 ms`; post-`#1031` tie coupling gap `~7.5 ms` (`~4.5 ms` irreducible floor, `~5–6 ms` off-tie).
- **Conditional worker stack **(derived)** (every number below awaits its frozen i9 run; prebuilt 4-lane forward still omits paid sampled-anchor cost being measured in `forward-4lane-anchor-probe`):**
  - `a6b-endpoint-build` (`#1046`): `+48.7..+67.7 ms` 1W, `+8.2..+9.1 ms/w` 8W.
  - 4-lane `forward-emission` (prebuilt oracle `#992/#998`, before paid anchor tax): `+215.0..+270.0 ms` 1W, `+28.9..+36.5 ms/w` 8W.
  - `forward-rle1-crc-vectored-build` (`#1045`, matched-AVX2 cold..warm vs unfused 4-lane): `+31.8..+49.9 ms` 1W, `~+4.0..+6.7 ms/w` 8W (scaling against `A9`'s `12.8 ms/w` + `7.3%..11.4%` `E+A9` cut, before L3-spill relief).
  - **Subtotal WITHOUT `mtf-fold` **(derived)**:** `+295.5..+387.6 ms` 1W (`22.6%..29.7%` of `1,306 ms`), `+41.1..+52.3 ms/w` 8W → **`W'/8 ≈ 132.6–143.9 ms`** (minus paid anchor overhead).
  - **With `mtf-fold-l1-batch-redesign` **(derived)** (conditional on passing its local `>= 2.5%` + zero-spill gate and recovering the `40–85 ms` 1W / `6–13 ms/w` 8W census floor):** `+335.5..+472.6 ms` 1W (`25.7%..36.2%`), `+47.1..+65.3 ms/w` 8W → **`W'/8 ≈ 119.6–137.9 ms`**.
- **Coupled 8W consequence **(derived)**:**
  - Against `#1031 + #1034`'s projected serial scanner **`S' ≈ 143–150 ms`**, the worker side (`W'/8 ≈ 133–144 ms` even *without* `mtf-fold`, or `120–138 ms` with the L1d-batch redesign) **crosses cleanly below `S'`** and well below `DECODER.md`'s **`S'(8) <= 165 ms`** trigger for `pooled-scan-hybrid`.
  - Under serial scanning alone, 8W wall floors at `max(S', W'/8) + ~5–6 ms` ≈ **`148–156 ms`** (`~5.4–5.7 GB/s`), still `8–16 ms` above the `~140 ms` (`6 GB/s`) research milestone.
  - With `pooled-scan-hybrid` borrowing genuine worker slack (`(8 · W'/8 + S') / 9` work bound = **`133.8–144.5 ms`** without `mtf-fold`, **`122.2–139.1 ms`** with `mtf-fold-l1-batch-redesign`; `+ ~5 ms` coupling → **`~127–144 ms`**), the **`~140 ms` (`6 GB/s`) milestone becomes reachable** once paid 4-lane anchors and the scanner/worker i9 runs confirm these component bounds.

---

## 5. Primary `text` trajectory, updated workstation queue, and hold-unlock map

### 5a. Second primary corpus (`PERFORMANCE-POLICY.md` `text` & off-Wikidata)

Two milestones since `#1037` move the non-Wikidata axis from characterisation to execution:

1. **`#1040` (`dense-cap-precheck`, SHIPPED `2915d7e`):** Eliminated `25/25` wasted dense-band Open first-pass refusals (`1.536× → 1.027×` `Segmented`; `prose` `1.486× → 0.950×`), zero `wikidata` impact (`1.0001×` median quiet CPU, 233/233 Open unchanged).
2. **`#1042` (`segmented-fallback-dissection`, PASS → `segmented-fallback-build`):** Sequential `LF` column build + invariant-checked `runs::fill_avx2` + AVX2 `assemble_reversed` cuts `Segmented` inverse-BWT **`−15.44%` baseline / `−16.05%` native** on `prose/text + source` (`−11.7%..−15.9%` on `ab gen text`, `−18.4%` across all 6 corpora, `0` extra memory, `0` Wikidata blocks touched). Since inverse BWT is `64.4%–64.9%` of worker time on `text` (`52.5%` `prose`, `42.3%` `elf`), this projects **(derived)** **`−7.5%..−10.3%` worker time on `text`** (`−10.8%..−11.5%` `prose`, `−10.4%..−11.2%` `elf`) — composing cleanly with corpus-universal scanner gains (`#1031` + `#1034`) and requiring **no i9 workstation run** (judged directly by CI paired bench under `PERFORMANCE-POLICY.md` §2).

### 5b. Updated workstation-queue order & current review-unlock bottlenecks

Pass 6 placed `mtf-fold-run` (#2) ahead of `#1034`. Post-`#1048` and `#1046`, the highest-value order shifts:

| Priority | Item / PR | Bottleneck it clears | Recommended action |
|---:|---|---|---|
| **Unblock A** | `#1047` (`release-terminal-holds`, ALL GREEN) | Reconciles stale `hold/archive-team-strata` (#977 merged) → unblocks **`migrate-rules`** | Independent review + merge via `take-review` |
| **Unblock B** | `#1041` (`mtf-fold-run`, ALL GREEN, `e33e54b5bc97`) | Lands zero-copy `S`/`M`/`C` + `bench-mtf-fold` AND frees `src/runs.rs` → unblocks **`segmented-fallback-build`** (fast CI-only primary `text` win) | Independent B5 review (`scan_oracle.rs` FFI) + merge **without** queueing `#1010` on i9 |
| **Unblock C** | `#1033` (`python-read-single-copy`, ALL GREEN, 0 `unsafe`, 0 protected paths) | Ships `−3.1%..−7.0%` Python `mibz2.open()` delivery win; frees `delivery` mechanism | Merge directly (RULES O1; zero i9 slot needed) |
| **i9 Queue #1** | `#1031` (`bench-scanner-seam-window`, ALL GREEN) | Removes `11.2 ms` `in_maint` + `~2.15 ms` coupling bonus; cleans baseline | 1 independent C6 review + merge + queue (~15 min) |
| **i9 Queue #2** | `a6b-endpoint-build` (live hold, 0 `unsafe`) & `#1034` (`scanner-coalesced-u8-decode`) | `+48.7..+67.7 ms` 1W / `+8.2..+9.1 ms/w` 8W (`a6b`), `~25–30 ms` `S` (`#1034` once `#846`/`#879` citations, `Kernels.bmi2`, B5, and recipe are complete) | Queue ahead of unredesigned `#1010` |
| **Local gate first** | `mtf-fold-l1-batch-redesign` (`#1048` `idea`) & `forward-4lane-anchor-probe` (live hold) | Must clear free local VM gates (`>= 2.5%` + zero-spill `objdump` for fold; paid 4-lane anchor gate for `forward-emission`) before spending i9 time | Merge `#1048` + `#1045` (resolve `surface.tsv` rebase on `#1048`), then gate locally before i9 |

---

## Dead ends and claim annotations

- All 27 rows in `frontier/dead-ends.tsv` stand (including `#1038` `bwt-contraction-constructor-rescue` and `#1042` KKP `LF²`); `#1045` adds scoped STOPs for 1-lane fused, inline FSM, and 4-lane stitched RLE1+CRC upon merge. `single-pass-rle1-crc` (`#988` STOP) remains frozen for standalone post-emit RLE1+CRC; only *upstream* 4-lane vectored fusion (`#1045`) breaks its floor.
- Annotated `frontier/claims/forward-emission.toml` (`why` only; `stop_test` untouched) with `#1045`'s vectored 4-slice delivery requirement and `forward-4lane-anchor-probe` sequencing.
- Annotated `frontier/claims/pooled-scan-hybrid.toml` (`question` only; `stop_test` and `DECODER.md`'s `S'(8) <= 165 ms` trigger untouched) with Pass 7's updated post-`#1046`/`#1045`/`#1048` 8W balance.
- Left `frontier/surface.tsv` and `frontier/dead-ends.tsv` untouched to avoid merge conflicts with open PRs `#1045` and `#1048`, and opened no duplicate `idea` claims (`mtf-fold-l1-batch-redesign` and `forward-rle1-crc-vectored-build` are already proposed in `#1048` and `#1045`).
