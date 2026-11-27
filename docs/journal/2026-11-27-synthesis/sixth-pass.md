# Synthesis, sixth pass: scanner stack reconciliation, post-seam 8W coupling trajectory, and the non-wikidata axis

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), 27 November 2026. Follows the [fifth pass](../2026-11-26-synthesis/fifth-pass.md) (#1021). Analytical and read-only across open PRs: no `src/` edits, no new hardware runs, no frozen gate reopened (C2). **(derived)** marks cross-source arithmetic or projections distinguished from established i9 measurements per [`frontier/DECODER.md`](../../../frontier/DECODER.md).*

## Primary sources read

- **Living state:** `frontier/DECODER.md`, `frontier/surface.tsv`, `frontier/dead-ends.tsv`, `docs/team/PERFORMANCE-POLICY.md`, `bench/workstation-queue.txt`, `src/lib.rs:168-227` (`Kernels`).
- **Merged analyses:** [fifth pass](../2026-11-26-synthesis/fifth-pass.md) (#1021), [BMI2 dispatch ablation](../2026-11-26-explore/huff-bmi2-dispatch.md) (#1012), [off-wikidata route census](../2026-11-27-explore/route-census.md) (#1023), [8-worker coupling dissection](../2026-11-27-explore/eight-worker-coupling.md) (#1035).
- **Open PRs inspected directly (read-only refs):**
  - `#1034` (`scanner-coalesced-u8-decode`): `src/bits.rs`, `src/huff.rs`, `src/block.rs`, `docs/journal/2026-11-26-scanner-coalesced-u8/registration.md`, delta.
  - `#1031` (`build/scanner-seam-window`): `docs/journal/2026-11-27-scanner-seam-window/registration.md`, delta, diff stat (`src/parallel.rs`, `src/trace.rs`, `.just/bench.just`, `bench/scanner_seam_window.py`).
  - `#1033` (`python-read-single-copy`): `src/python.rs` diff, delta.
- **Live claims:** `scanner-coalesced-u8-decode.toml`, `scanner-seam-window.toml`, `huff-bmi2-build.toml`, `mtf-fold-run.toml`, `delivery-l3-working-set.toml`, `dense-cap-precheck.toml` (#1030), `segmented-fallback-dissection.toml` (#1032).

---

## 1. Scanner stack reconciliation: `#1034` vs `huff-bmi2-build` vs `#1031`

Three scanner items now overlap conceptually or by file ownership: `#1034` (`scanner-coalesced-u8-decode`, open PR), `huff-bmi2-build` (unheld `idea`), and `#1031` (`scanner-seam-window`, open PR).

### 1a. Audit of `#1034`'s BMI2 implementation against `#1012`'s criteria

`frontier/DECODER.md` warns: *"Do not duplicate BMI2 implementation inside a combined scanner build."* Direct inspection of `#1034`'s `src/block.rs` and `src/huff.rs` against `#1012`'s disassembly criteria (`huff-bmi2-dispatch.md` §3) and `huff-bmi2-build.toml`:

| Criterion (`#1012` / `huff-bmi2-build.toml`) | What `#1034` actually implements | Match / gap |
|---|---|---|
| Block-level `#[target_feature(enable = "bmi2")]` (not per-group, which regressed +3.3..+6.0%) | `src/block.rs:157-168` defines `#[cfg(target_arch = "x86_64")] #[target_feature(enable = "bmi2")] unsafe fn decode_runs_bmi2`, called once per block; both `decode_runs_inner` (`src/block.rs:104`) and `Huff::decode_run` (`src/huff.rs:291`) are `#[inline(always)]`. | **MATCH** |
| `shlx`/`shrx` with `0 shl %cl`, `0 vzeroupper`, `0` YMM pollution | `#1034` delta reports `objdump -d` on baseline x86-64 release: `22/22 shlx`, `8/8 shrx`, `0 shl %cl`, `0 vzeroupper`, `0` bounds checks, `0` spills, plus `0 and $0x3f` on shift chain (resolving `#1012` §3.4 via `skip_short_shl_u8`). | **MATCH** (and improves on `#1012` §3.4) |
| Minimal `unsafe` surface | `src/bits.rs` and `src/huff.rs` keep `#![forbid(unsafe_code)]`; `src/block.rs` uses `#![deny(unsafe_op_in_unsafe_fn)]` with one `unsafe` call to `decode_runs_bmi2` calling the same safe `decode_runs_inner`. | **MATCH** |
| Optional AVX2+BMI2 `build_into` (`#1012`: −9.60% vs −9.01% on A3) | `Huff::build_into` (`src/huff.rs:84`) stays scalar (runt-gated via `PAIR_GATE_SEL = 64`). | Minor (~0.6% of A3); acceptable |
| **Runtime gating through `Kernels::detect()` (`src/lib.rs:168-224`)** | **`#1034` does NOT touch `src/lib.rs`:** `src/block.rs:190` calls `std::is_x86_feature_detected!("bmi2")` directly. `Kernels::scalar()` / `Kernels::restrict()` cannot disable BMI2. | **GAP** — breaks scalar-forced parity / `Kernels`-gated paired BMI2 A/B |
| **Citations requested by `fifth-pass.md` §2** | `git diff origin/master...pr-1034` has **zero mentions** of `#846` (zero-skip `syms` i9 runs in §77/§79), `#879` (batched-bits i9 run: `0.978–1.001`, null), or §77/§79. | **GAP** — must be cited in `#1034`'s `registration.md` before queuing |

### 1b. Resolution and i9 composition

- **`#1034` supersedes `huff-bmi2-build`'s decode-loop build if `#1034` lands:** do **not** implement a second BMI2 decode path. Before merging `#1034` (or immediately after, if `src/lib.rs` stays outside `#1034`'s claim paths), either thread a `bmi2: bool` flag through `Kernels` (`detect()` sets it, `scalar()` and `new_unchecked()` set `false`, preserving soundness) so `Kernels::scalar()` truly disables all x86 ISA specialisation, or record explicitly why scanner BMI2 bypasses `Kernels`. If `#1034` fails its frozen i9 gate and is reverted, `huff-bmi2-build` remains the unheld standalone fallback on shipped `Huff`.
- **`#1031` and `#1034` are file-disjoint and mechanism-orthogonal on the i9:**
  - `#1031` (`src/parallel.rs`, `src/trace.rs`) removes `ChannelSource::view`'s `in_maint` (`11.2 ms` at 8W, `12.8 ms` at 10W) and is already C6-registered with a runnable workstation recipe (`bench-scanner-seam-window`, `bench/scanner_seam_window.py`, 8/8 unit tests).
  - `#1034` (`src/bits.rs`, `src/huff.rs`, `src/block.rs`) attacks `A2 + A3` (`170.8 ms` at 8W) plus `syms` zero-skip (lever 3 = `#846`'s mechanism). It still needs `#846`/`#879` citations in `registration.md`, adversarial B5 review of `decode_runs_bmi2`, CI Miri verification on `decode_runs_scalar`, and a workstation recipe before queuing.
  - Because `#1031` removes the non-stationary `in_maint` stalls that distort both `S` and the 8W coupling gap (see §2), running `#1031` first (or stacking `#1034` on post-`#1031` master) gives `#1034` a cleaner baseline.
- **Disjoint Python delivery win (`#1033`, `src/python.rs` only):** eliminates the redundant zero-filled `Vec<u8>` + second copy in `Reader::read` via `BufRead::fill_buf` + `PyBytes::new` (`−3.1%..−7.0%` wall across all four `mibz2.open()` cells on VM, zero new `unsafe`, zero overlap with Rust core or the i9 queue). Ready for independent review/merge now; follow-up `Reader::readinto` via `PyBuffer<u8>` remains unowned.

---

## 2. Coupled 8W trajectory and workstation-queue roadmap

### 2a. Established i9 facts vs conditional projections

Per `frontier/DECODER.md`, established i9 measurements and conditional projections are kept strictly apart:

- **Established on i9-10900K (`wikidata ×4`, 839 MB, 8 workers):**
  - Shipped wall `none(8) = 192.9–194.6 ms` (`c71c4a56`, `cc8ba137`, `character-stage1`), at a tie between scan-bound `C(8) = 183.2–184.4 ms` and worker-bound `S(8) = 184.9–185.0 ms`.
  - `#1035` decomposed the `7.57–9.13 ms` coupling gap (`wall − max(S, W/8)`) via exact `src/parallel.rs` discrete-event replay (validated within `+0.94..+1.36 ms`, mean `+1.15 ms` matching `S`/`SC`'s `1.12–1.16 ms` unoverlapped handoff residual):
    1. **`~2.15 ms` coupling bonus already owned by `#1031` (`scanner-seam-window`):** `in_maint`'s 11 stop-the-world stalls (`0.53–1.54 ms`) are front-loaded across copies `c0..c3` (`3.81 → 3.09 → 1.86 → 1.40 ms`), causing a mid-decode phase inversion. Removing `in_maint` drops the exact-1.00-tie mid-stream gap from `6.61` to `4.46 ms` *on top of* its `~10–11 ms` direct scan reduction.
    2. **Post-`#1031` coupling floor at a 150 ms exact tie:** total gap is **`~7.5 ms` (4.8%)**, of which **`~4.5 ms` is irreducible** (`2.19 ms` indivisible-block variance/drain + `1.22 ms` thread spawn + `~1.1 ms` handoff), leaving only `~2.5 ms` (`1.6%`) combined unowned ceiling (`1.21 ms` runt-exempt slots/slots=64 + `1.35 ms` smaller initial 4 MiB feeder chunk).
    3. **Direct constraint on `delivery-l3-working-set`:** `READY_BYTES = 12 MiB` never fires at `k = 8` (`slot_cap = 0.00 ms`, median `hwm = 6.01 MiB`), whereas `SLOTS_PER_THREAD = 4` (32 slots) already incurs `3.25–6.10 ms` of HOL `slot_wait` because 15–16 of 32 slots hold tiny 8–30 B runts (`wikidata` alternates `F.F.F.F`). Tightening `SLOTS_PER_THREAD` to 2 or 3 will worsen `k = 8` backpressure unless runts are excluded from the slot count; tightening `READY_BYTES` alone leaves `k = 8` untouched while bounding `k = 9–10` L3 residency.
- **Conditional projections **(derived)** (NOT measurements; every number below awaits its frozen i9 run):**
  - **Scanner path (`#1031` + `#1034`):** `#1031` cuts `in_maint` (`~10–11 ms`), taking `S` from `~185 ms` toward `~174 ms`; `#1034` (VM `−10.8%` safe + `−4.5%` BMI2 = `−14.8%..−17.9%` on `S`, projected `~25–30 ms` on i9) would then push `S'` toward **`~145–150 ms`**.
  - **Worker path (`mtf-fold-run` + `forward-emission`):** `mtf-fold` (`#1010`, merged on master) has a VM census expectation of `~6–13 ms/w` behind ideal `S`, with the caveat (`2026-11-26-explore-fold-chain-and-pipeline-edges.md`) that its zero-overlap floor straddles K1's 2.5% gate. Combined with `forward-emission` (unbuilt; queued on `src/open.rs`), fourth-pass projections place `W'/8` near **`~135–153 ms`**.
  - **Coupled 8W wall after both paths **(derived)**:** replacing fifth-pass's unadjusted `+8 ms` tie estimate with `#1035`'s post-`#1031` gap (`~7.5 ms` at an exact 1.00 tie, falling to `~5–6 ms` off-tie) puts the projected 8W wall at **`~152–160 ms`** (down slightly from fifth-pass's `153–163 ms`, still ~12–20 ms above the `~140 ms` 6 GB/s milestone). Reaching `~140 ms` requires pushing *both* `S'` and `W'/8` to `<= ~133–135 ms`, which is why `pooled-scan-hybrid` (triggered under `DECODER.md` once measured `S'(8) <= 165 ms`) remains relevant once worker wins land.

### 2b. Workstation-queue ordering (three runs preparing)

| Priority | Run / PR | Current state | What it decides & why this order |
|---:|---|---|---|
| **1** | `bench-scanner-seam-window` (`#1031`) | Code, tests, recipe, and C6 registration complete; awaits 1 independent C6 review + green CI | Lowest risk, ~15 min. Decides `#1031` adoption at production 10/12T (8W) — where removing `11.2 ms` `in_maint` + `#1035`'s `~2.15 ms` coupling bonus captures the 8W tie excess (`none 193 → ~185 ms`, matching `#861`'s measured `0.982/0.977` FASTER). Clears `in_maint` before `#1034` runs. |
| **2** | `bench-mtf-fold` (`mtf-fold-run`) | Prototype `#1010` merged on master; claim held; needs `src/scan_oracle.rs` port + recipe | Decides K1/K2 for `mtf-fold` on the worker station. Unblocks `forward-emission` on `src/open.rs` and tests whether `S'(8)` moves toward the `<= 165 ms` hybrid trigger. |
| **3** | `scanner-coalesced-u8-decode` (`#1034`) | Code + prose registration open; needs `#846`/`#879` citations, B5 review on `decode_runs_bmi2`, `Kernels.bmi2` reconciliation, and workstation recipe | Decides `>= 8.0% S` / `>= 3.0% 10W` (or post-worker 8W). Running after `#1031` avoids attributing `in_maint` coupling noise to `#1034`; supersedes `huff-bmi2-build` on PASS. |

*(Non-workstation PRs `#1033` `python-read-single-copy` and `#1030` `dense-cap-precheck` need no i9 queue slot and can integrate independently.)*

---

## 3. The non-wikidata (`PERFORMANCE-POLICY.md` `text`) axis

`docs/team/PERFORMANCE-POLICY.md` §2 designates **two** primary corpora — `wikidata` and `text` — yet `frontier/surface.tsv` rows `C` and `E` price only `OpenCells` on `wikidata`, where 233/233 (**100%**) of full blocks enter `Open` (`r/n` median `0.068`).

`docs/journal/2026-11-27-explore/route-census.md` (#1023) measured what production actually executes across seven non-wikidata corpora (107 full blocks: `prose`, `pysrc`, `rustsrc`, `json`, `elf`, `dna`, `random`):

- **84% of off-wikidata full blocks bypass `OpenCells`:** only **17/107 (16%)** enter `Open` (8/16 `rustsrc`, 7/16 `json`, 1/8 `prose`, 1/11 `pysrc`, 0/56 `elf`+`dna`+`random`). The remaining **90/107 (84%)** execute `Segmented`: 55 via the column route and 35 via the run-route fallback.
- **Two distinct consequences now live on the frontier:**
  1. **Active regression (`dense-cap-precheck`, `#1030`, open on `src/open.rs`):** `Cap::dense` (`r/n ∈ [0.15, 0.28]`) was fitted to wikidata's 5 dense blocks; off wikidata, **25/25** dense-band blocks (7 `prose`, 8 `rustsrc`, 4 `json`, 4 `elf`, 2 `pysrc`) run Open's first pass to the cap and are then refused, making production **1.13–1.48× slower than `Segmented`** on those blocks (**1.09–1.32×** slower on the whole of `prose`). A one-pass thin-run record count predicts every refusal cleanly before Open starts, fixing the regression with zero wikidata impact.
  2. **Unmapped 84% workload (`segmented-fallback-dissection`, `#1032`, open smooth probe):** Even after `#1030` eliminates wasted dense-band refusals, `Segmented` carries **84%** of non-wikidata full blocks (including 7/8 `prose`) and has no `surface.tsv` row and no build claim.
- **Cross-corpus asymmetry across the active roadmap:**
  - **Scanner levers (`#1031`, `#1034`) are corpus-universal:** every route (`Open`, `column`, `run-route Segmented`) executes the identical Huffman scanner, so scanner gains improve `wikidata`, `text`, source code, and binaries alike.
  - **`OpenCells` worker levers (`mtf-fold`, `forward-emission`, `bwt-contraction-constructor-rescue`)** cover 100% of `wikidata` full blocks but only ~16% off-wikidata (and 0% of `elf`/`dna`/`random`).
  - **`dense-cap-precheck` (`#1030`) + `segmented-fallback-dissection` (`#1032`)** are the complementary worker-side axis for `PERFORMANCE-POLICY.md`'s second primary corpus (`text` / prose). Once `#1032` reports its phase breakdown, `surface.tsv` should gain a `Segmented` row (or explicit off-wikidata column) so non-wikidata room is no longer invisible.

---

## Dead ends and claim annotations

- All 25 rows in `frontier/dead-ends.tsv` stand (including `#1035`'s closed row for the 8-worker coupling gap as a standalone `>= 3%` lever); none is revived.
- Annotated `frontier/claims/delivery-l3-working-set.toml` (`question` only; `stop_test` untouched) with `#1035`'s `READY_BYTES` vs `SLOTS_PER_THREAD < 4` runt-HOL constraint.
- Annotated `frontier/claims/huff-bmi2-build.toml` (`why` only; `stop_test` untouched) with `#1034`'s BMI2 overlap, `Kernels.bmi2` gap, and reconciliation rule (no duplicate BMI2 build).
- No new `idea` claims opened (`dense-cap-precheck`, `segmented-fallback-dissection`, `scanner-seam-window`, `scanner-coalesced-u8-decode`, and `mtf-fold-run` already cover every actionable mechanism identified here).
