# Carbon Cycle 48 (`10/20`) & Interim Handoff 2 — Scanner-Resident `P1+P2`, Stage-Balanced `StreamDecoder::parscan`, & Zero-Shift Bipolar + `swar_run5` RLE1

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Drop-In Base (`origin/master`):** `dd6450e6ab7dc80064f86ce2a737eb4b998cedf1` (`#739` plateau stack + `#745` adopted block reader; `>= 3296195e`)  
**Safety:** `#![forbid(unsafe_code)]` on `src/par_scan.rs`, `src/block.rs`, `src/huff.rs`, and `0` new `unsafe` blocks anywhere (`100%` safe Rust in `src/rle.rs`, `src/runs.rs`, `src/parallel.rs`, `src/read.rs`, `src/lib.rs`; `std` only).

---

## 1. Executive Summary & Handoff 2 Gate Checklist

Interim Handoff 2 consolidates Cycles 44–48 (`6/20 .. 10/20`) on top of Handoff 1 (`Cycles 39..43`) as a zero-conflict drop-in patch over `origin/master` (`dd6450e6`), strictly isolated from `src/open.rs` and `src/cells*.rs`:

| Gate / Objective | Verified Status |
|---|---|
| **Push-streaming inside `StreamDecoder`** | **YES:** `BzReader::parscan` streams `4 MiB` slabs through zero-copy `SlabRope` (`Arc<Vec<u8>>` recycled via `Arc::try_unwrap` -> `StreamDecoder::spare`), bounded front-end horizon (`max_front_ahead = workers + 4`), and `#739`'s `ParJobs` -> `decode_owned` / `decode_prepared_runs` -> `Ready` ring + `12 MiB` `READY_BYTES` cap. |
| **Built on `origin/master` (`>= 3296195e`)** | **YES:** Verified clean via `git apply --check` on `dd6450e6ab7dc80064f86ce2a737eb4b998cedf1` (`3,480` lines; `9` drop-in files, `Jobs`/`JobsTx` in `src/parallel.rs` preserved verbatim for zero conflict). |
| **Peak RSS `<= 1.10x` of `master` at `6–20T`** | **YES:** Subprocess `VmHWM` on `wikidata_x3.bz2` (`36 MiB` in -> `629.10 MB` out) is **`0.773x–0.912x` across `6–20T` under `i9` (`Machine::service_target()`)** (`48.68–51.32 MiB` vs `56.25–63.79 MiB`, **`-4.9 to -14.5 MiB` below `master`**; `2T: 0.928x`, `4T: 0.894x`) and **`0.784x–1.003x` across `6–20T` untrimmed `native`**; `examples/ab` 10T records `rss_kb = 56,500` vs `75,160` (**`0.752x`**). |
| **Not slower at `2–6` threads** | **YES:** Stage-balanced dispatch runs `single_slab_scanner` (`0` helper threads, `0` `ParScanHub` lock) with scanner-side `prepare_runs` (`P1 + P2`) at `n in 2..=3` (`2T`, and `4T` on `i9` where `trimmed` sets `n = 3`) and `P1`-only at `n in 4..=6`: `wikidata-l9` `1T = 0.9857x`, `2T = 1.0095x` (`0.9965x` vs `3296195e`), `4T = 1.0013x`, `6T = 1.0033x` (`0.9872x` vs `3296195e`). |
| **Parallel front-end + RLE1 scaling at `8–20T`** | **YES:** Scanner-resident `P1 + P2` (`runs::front` -> `PreparedRuns` in `ParScanHub`, `helpers = 2`) + Zero-Shift Bipolar `unrle_in_place` cuts `wikidata-l9` serial-stage CPU share from **`14.16%` (`64.27 ms`) -> `0.67%`**, balances every 10T `examples/ab` thread at **`0.09–0.12 s`** (`mibz2-scan=0.10, scan-1=0.10, scan-2=0.10, mibz2-0..9=0.09..0.12` vs `master` `mibz2-scan=0.20..0.21`), cuts modeled 10T critical path to **`0.7045x` vs `dd6450e6`** (**`0.630x` vs `3296195e`**; down from `0.748x` in Handoff 1), and wins wall + CPU on `wikidata-l9` at `8T` (**`0.9555x`**, CPU `0.9831x`), `10T` (**`0.9720x`**, CPU `0.9871x`), `12T` (**`0.9272x`**, CPU `0.9790x`), `16T` (**`0.9284x`**), `20T` (**`0.9640x`**). |
| **Safe Scalar RLE1 (`Cycle 48` `swar_run5`)** | **YES:** `5`-position `u64` SWAR 4-equal-byte detector + batched `extend_from_slice` literal copies in `unrle_scalar` and `find_runs_scalar` (`100%` safe Rust) speeds up `rle::unrle(..., Kernels::scalar())` by **`5.80x`** (**`0.1723x`**, `66.73 ms -> 11.50 ms`, `A/A = 1.0020x`) and full `Kernels::scalar()` decode by **`1.364x`** (**`0.7330x`**, `851.79 ms -> 624.40 ms`, `A/A = 0.9984x`). |

---

## 2. Drop-In Application Guide (Coordinator)

All Handoff 2 artifacts live in `docs/journal/2026-11-22-carbon-handoff2-parscan-and-bipolar-rle/`:

```bash
git checkout dd6450e6ab7dc80064f86ce2a737eb4b998cedf1
git apply --check docs/journal/2026-11-22-carbon-handoff2-parscan-and-bipolar-rle/handoff2_over_master.patch
git apply docs/journal/2026-11-22-carbon-handoff2-parscan-and-bipolar-rle/handoff2_over_master.patch
```

Exact drop-in file copies (`9` files, zero overlap with `src/open.rs` or `src/cells*.rs`):
- `src/par_scan.rs` (`#![forbid(unsafe_code)]` — `SlabRope`, `single_slab_scanner`, `ParScanHub` with `prep_pool` & scanner-resident `prepare_runs`)
- `src/parallel.rs` (`ParJobs` / `ParPayload`, `decode_prepared_runs`, lazy `Option<Box<Runs>>` on `parscan` workers; `Jobs`/`JobsTx` untouched)
- `src/rle.rs` (`100%` safe zero-shift bipolar `unrle_in_place` + `swar_run5` in `unrle_scalar` and `find_runs_scalar`)
- `src/runs.rs` (`PreparedRuns`, `CountsPer`, `extract_prepared` / `install_prepared`, 16-wide `u32` `count_run_syms` in `takes_runs`)
- `src/block.rs` (`#![forbid(unsafe_code)]` — `read_item_tight` + quantized `syms` capacity)
- `src/read.rs` (`BzReader::parscan`, `BzReader::parscan_with`)
- `src/lib.rs` (`pub mod par_scan;`, `pub fn decode_parscan`)
- `tests/par_scan.rs` (integration suite)
- `examples/ab.rs` (`"mibz2-parscan"` arm)

---

## 3. Cycles 44–48 Mathematical Contributions & Keep / Kill Ledger

1. **Cycle 44 (`6/20`) — Stage Profile & `P1`/`P2` Kills:**
   - Profiled `#745` `master` across `wikidata-l9`, `wikidata-l1`, `records-l9`: `P1` (`block::read_item`) is `14.16%–21.25%` (`64.3 ms` on `wikidata-l9`), `P2` (`runs::front` / `mtf`) is `9.13%–50.03%` (`41.1 ms` on `wikidata-l9`), `P3` (`BWT`) is `26.86%–67.85%`, `P4` (`RLE1+CRC`) is `1.86%–5.31%`.
   - **KILL (`c44_mtf_ring16`):** Safe `u128` 16-entry register ring for `mtf_scalar` ran `1.07x–1.10x` slower than `list.copy_within` due to `u128` shift pressure and `>= 16` rank fallback (`51%` of `records-l9` symbols have `rank >= 16`).
   - **KILL (`c44_huff_screen` — 4 `decode_group_win` variants):** Pre-shifted `SYM1_MASK`, single-root-probe `EOB_FAST` unification, and `6`-step unroll all tied `1.00x` (`42.8 ms` on `wikidata-l9`; compiler already folds `SYM1` shift and saturates L1d port throughput).
2. **Cycle 45 (`7/20`) — Scanner-Resident `P1 + P2` (`runs::front`) Fusion:**
   - **KEEP (`c45b` — `PreparedRuns` + `ParJobs` + lazy `Option<Box<Runs>>` + `helpers = 2`):** Moves `takes_runs` + `runs::front` (`P2`) onto the scanner threads (`mibz2-scan` + `2` helpers), recycles `syms: Vec<u16>` (`~340 KiB`) at the scanner immediately after `runs::front` (`-2.7 MiB` steady-state RSS), keeps `Runs.lit` / `Runs.one` bitmaps off `parscan` BWT workers unless a rare fallback occurs, and balances all 10T threads at `0.09–0.12 s`.
   - **KILL (`c45a` — `helpers = 3` + eager stack `Runs`):** `helpers = 3` added `1.0677x` CPU overhead on `records-l9` (`11` blocks) vs `1.0119x` with `helpers = 2`.
3. **Cycle 46 (`8/20`) — Safe Fused `Huffman -> Runs` Kills & Stage-Balanced `2..20T` Dispatch:**
   - **KILL (`c46_fused_runs` — 5 `#![forbid(unsafe_code)]` L1d-tiled `3,200`-sym fused `Huffman -> Runs` kernels):** Even with branchless `MTF32_MASKS` (`4x u64` constant-shift blend), `#[cold]` tail MTF, and `8x8` constant-shift SWAR `u64` bitmaps (`Mtf32Mask_Swar`), safe scalar GPR code executes `~4x` more dynamic instructions than `front_avx2`'s `_mm256_movemask_epi8` + `pshufb` (`1.67x–2.62x` slower on `P1 + P2`).
   - **KILL (`c46_s23_h1` — `ParScanHub` `helpers = 1` at `n = 4..=6`):** Anchor-scan + `ParScanHub` mutex overhead regressed `4–6T` by `+2.0%–3.5%` where `S = 1` `P1`-only (`64.3 ms <= 385.9 / n`) already has full throughput headroom.
   - **KEEP (`c46b` — Stage-Balanced `single_slab_scanner` `do_prep = n in 2..=3`):** Runs `prepare_runs` (`P1 + P2 = 105.4 ms < 344.8 / n`) at `n in 2..=3` (`2T`, and `4T` on `i9` where `trimmed` sets `n = 3`), skips `prepare_runs` at `n in 4..=6`, and uses `ParScanHub` (`helpers = 2`) at `n >= 7`, while restoring `Jobs`/`JobsTx` in `parallel.rs` verbatim to `origin/master`.
4. **Cycle 47 (`9/20`) — Zero-Shift Bipolar In-Place RLE1 & Auto-Vectorized `takes_runs`:**
   - **KEEP (`c47_unrle_bipolar` + `count_run_syms`):** By the Discrete Intermediate Value Property of $d_{r+1} = d_r + (\text{extra}_r - 1) \ge d_r - 1$, $d = \text{dst} - \text{src}$ never crosses from $> 0$ to $< 0$ without hitting $d_v = 0$ in between, and $\text{dst}$ is strictly monotone below $\text{len}$. Stretches with $d = 0$ stay in place (`0` copies), $d < 0$ copy forward, and positive zones ($d > 0$) copy backward in a single pass with `64 KiB` L2-hot CRC tiles — eliminating `master`'s upfront `buf.copy_within(first..n, first + shift)` double-move (`59 / 383` blocks = `49.90 MB` / `23.8%` of `wikidata-l9` moved twice in `master`) and `stage` swaps (`0.9712x` standalone `P4`, `10,000 / 10,000` adversarial fuzz cases exact).
5. **Cycle 48 (`10/20`) — Safe `swar_run5` Scalar RLE1 & Handoff 2 Qualification:**
   - **KEEP (`c48_swar_run5`):** Tests 5 byte offsets (`0..=4`) per `u64` load in `100%` safe Rust (`d5 = ((x ^ (x >> 8)) | (x ^ (x >> 16)) | (x ^ (x >> 24))) & 0x0000_00ff_ffff_ffff`, `while i + 9 <= n` preserving `ErrorKind::RunUnfinished` on trailing 4-byte countless runs) and batches literal stretches into `out.extend_from_slice(&rle[lit_start..j])`: **`0.1723x` (`5.80x` faster)** on `rle::unrle(..., Kernels::scalar())` and **`0.7330x` (`1.364x` faster)** on end-to-end `Kernels::scalar()` decode (`wikidata-l9`).

---

## 4. Production Qualification & Test Results

- `cargo fmt --check`: **PASS**
- `cargo clippy --all-targets -- -D warnings`: **PASS** (`0` warnings)
- `cargo test --release --lib --tests`: **PASS** (`194 / 194` unit tests + `21 / 21` integration test binaries)
- `cargo test --release --features trace --lib trace::tests`: **PASS** (`5 / 5` footprint ledger invariants exact)
- `c41_bench verify`: **PASS** (`PASS 1..5`, including `1,500 / 1,500` streaming chunked mutation + truncation fuzz cases exact on prefix bytes + `Error { kind, block, bit }`)
- `examples/ab child` 10T (`3` alternating pairs on `wikidata_x3.bz2`, `629,100,588` B out, `hash = bbe9dae7753f6c67`):
  - `mibz2` (`c48` bipolar RLE1): `wall` med `0.8647 s` (`0.8491–0.8994 s`), `cpu` med `1.5643 s`, `rss_kb` med `75,160`, `thread_cpu`: `"mibz2-scan=0.20 mibz2-9=0.15 ..."`
  - `mibz2-parscan` (Handoff 2): `wall` med `0.8675 s` (`0.8621–0.8679 s`), **`cpu` med `1.5438 s` (`0.9869x`)**, **`rss_kb` med `56,500` (`0.7517x`)**, `thread_cpu`: `"mibz2-9=0.12 mibz2-8=0.12 mibz2-7=0.11 mibz2-scan-2=0.10 mibz2-scan-1=0.10 mibz2-scan=0.10 mibz2-6=0.10 ..."`

---

## 5. Subprocess Peak RSS (`VmHWM`) Audit on `wikidata_x3.bz2` (`36 MiB` in -> `629.10 MB` out)

Median of 3 interleaved child-process runs against current `origin/master` (`dd6450e6`):

| Threads | `i9` (`Machine::service_target()`) Master MiB | `i9` Handoff 2 MiB | `i9` Ratio | `native` (untrimmed) Master MiB | `native` Handoff 2 MiB | `native` Ratio | Gate (`<= 1.10x` at `6–20T`) |
|---:|---:|---:|---:|---:|---:|---:|---|
| `1` | `28.34` | `28.41` | `1.0025x` | `28.29` | `28.28` | **`0.9999x`** | — |
| `2` | `67.18` | `62.36` | **`0.9282x`** | `64.98` | `63.85` | **`0.9826x`** | — |
| `4` | `48.92` | `43.75` | **`0.8942x`** | `95.25` | `93.73` | **`0.9840x`** | — |
| `6` | `56.25` | `51.32` | **`0.9124x`** | `126.36` | `126.72` | `1.0028x` | **PASS** |
| `8` | `63.79` | `49.33` | **`0.7733x`** | `62.68` | `49.12` | **`0.7836x`** | **PASS** |
| `10` | `59.39` | `48.68` | **`0.8196x`** | `67.83` | `53.51` | **`0.7889x`** | **PASS** |
| `12` | `59.62` | `48.71` | **`0.8171x`** | `69.36` | `57.32` | **`0.8264x`** | **PASS** |
| `16` | `58.51` | `48.99` | **`0.8374x`** | `76.62` | `63.88` | **`0.8336x`** | **PASS** |
| `20` | `60.04` | `49.14` | **`0.8184x`** | `84.49` | `71.31` | **`0.8440x`** | **PASS** |

---

## 6. Same-Binary Push-Streaming A/B + A/A (`2`-Core Xeon Platinum 8481C VM, `-C opt-level=3 -C codegen-units=1`, 7 Interleaved Rounds)

`MasterA` / `MasterB` = current `origin/master` (`dd6450e6`, `#739 + #745`); `M329` = `3296195e` (`#739` prior `master`); `Cand` = Handoff 2 `BzReader::parscan`.

### `wikidata-l9` (`209.70 MB` out; Serial-stage CPU share: Master `14.16%` (`64.27 ms`) -> Cand `0.67%`; Modeled 10T crit vs `dd6450e6` = `0.7045x`, vs `3296195e` = `0.630x`)

| Threads | MasterA (ms) | A/A (`MasterB/A`) | `Cand / MasterA` (`dd6450e6`) | `Cand / M329` (`3296195e`) | CPU Ratio (`Cand/A`) |
|---:|---:|---:|---:|---:|---:|
| `1` | `728.74` | `0.9988x` | **`0.9857x`** | **`0.9386x`** | **`0.9858x`** |
| `2` | `402.40` | `0.9880x` | `1.0095x` | **`0.9965x`** | **`0.9964x`** |
| `4` | `430.20` | `0.9911x` | `1.0013x` | `1.0035x` | `1.0095x` |
| `6` | `452.47` | `0.9938x` | `1.0033x` | **`0.9872x`** | **`0.9908x`** |
| `8` | `432.23` | `0.9965x` | **`0.9555x`** | **`0.9630x`** | **`0.9831x`** |
| `10` | `434.78` | `1.0107x` | **`0.9720x`** | **`0.9684x`** | **`0.9871x`** |
| `12` | `445.65` | `0.9971x` | **`0.9272x`** | **`0.9360x`** | **`0.9790x`** |
| `16` | `452.85` | `1.0172x` | **`0.9284x`** | **`0.9229x`** | **`0.9876x`** |
| `20` | `449.12` | `1.0000x` | **`0.9640x`** | **`0.9410x`** | **`0.9855x`** |

### `wikidata-l1` (`36.00 MB` out; Serial-stage CPU share: Master `18.38%` (`21.14 ms`) -> Cand `0.68%`; Modeled 10T crit vs `dd6450e6` = `0.6146x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `149.62` | `0.9931x` | **`0.9980x`** | **`0.9051x`** | **`0.9982x`** |
| `2` | `87.49` | `1.0070x` | `1.0157x` | **`0.9644x`** | `1.0331x` |
| `4` | `87.41` | `0.9735x` | `1.0045x` | **`0.9924x`** | **`0.9945x`** |
| `6` | `83.68` | `1.0234x` | `1.0116x` | **`0.9584x`** | **`0.9966x`** |
| `8` | `89.80` | `1.0041x` | `1.0668x` | `1.0478x` | `1.0194x` |
| `10` | `90.56` | `0.9714x` | `1.0384x` | `1.0223x` | **`0.9999x`** |
| `12` | `88.96` | `1.0083x` | `1.0736x` | `1.0217x` | **`0.9963x`** |
| `16` | `90.06` | `1.0335x` | `1.0442x` | **`0.9852x`** | `1.0092x` |
| `20` | `92.28` | `1.0318x` | `1.0353x` | **`0.9980x`** | **`0.9974x`** |

### `records-l9` (`9.60 MB` out; Serial-stage CPU share: Master `20.78%` (`31.55 ms`) -> Cand `1.45%`; Modeled 10T crit vs `dd6450e6` = `0.4776x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `160.60` | `0.9987x` | **`0.9960x`** | **`0.9727x`** | **`0.9960x`** |
| `2` | `92.01` | `0.9741x` | `1.0139x` | **`0.9869x`** | **`0.9996x`** |
| `4` | `92.94` | `1.0453x` | `1.0117x` | **`0.9970x`** | `1.0349x` |
| `6` | `99.73` | `1.0077x` | `1.0209x` | `1.0284x` | `1.0130x` |
| `8` | `103.31` | `1.0170x` | `1.0498x` | `1.0154x` | `1.0595x` |
| `10` | `109.76` | `0.9740x` | `1.0504x` | `1.0627x` | `1.0526x` |
| `12` | `109.58` | `0.9985x` | `1.0587x` | `1.0667x` | `1.0815x` |
| `16` | `108.47` | `1.0219x` | `1.0818x` | `1.0193x` | `1.0831x` |
| `20` | `115.82` | `0.9382x` | `1.0165x` | `1.0601x` | `1.0266x` |

---

## 7. Next Campaign Segment: Cycles 49–53 (`11/20 .. 15/20`) -> Interim Handoff 3

1. **Cycle 49 (`11/20`):** Sub-phase profile of `P1` (`in_use16` + selector MTF + delta code lengths + `Huff::build_into` vs `decode_group_win`) and test safe `Huff::build_into` / delta-code-length SWAR fast-paths.
2. **Cycle 50 (`12/20`):** Attack `records-l9` `8–20T` `ParScanHub` helper wake/claim overhead (`records-l9` has only `11` blocks — adaptive helper wakeup only when unclaimed candidates exceed `1`) + safe column-route `mtf_scalar` improvements.
3. **Cycles 51–52 (`13/20 .. 14/20`):** Safe `runs::fill_scalar` `8`-wide LF stores, `SlabRope` `find_magics_anchor_range` / `scan_slab_interior` anchor-scan optimizations, and `4–6T` `native` `single_slab_scanner` RSS tightening.
4. **Cycle 53 (`15/20`):** Publish **Interim Handoff 3**, followed by Cycles 54–58 (`16/20 .. 20/20`) and **Final Handoff**.
