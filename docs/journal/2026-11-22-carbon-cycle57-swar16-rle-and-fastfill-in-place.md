# Carbon Cycle 57 (19 / 20) — 16-Position Dual-`u64` Safe SWAR RLE1 (`0.1403x` vs Master), Overlapping-Endpoint `fill_run_fast`, and `TILE=16KiB` Kill

- **Date:** 2026-11-22
- **Branch:** `research/carbon-word-transport`
- **Base Candidate:** Cycle 56 (`0d580086`)
- **Saved Drop-In:** `docs/journal/2026-11-22-carbon-cycle57/rle.rs`
- **Safety:** `100%` safe Rust (`#![forbid(unsafe_code)]` harness `c57_rle_screen.rs`; `0` new `unsafe` blocks in `src/rle.rs`)

---

## 1. Algorithmic Improvements & Ablation

1. **`c57_swar16` (`src/rle.rs:unrle_scalar` & `find_runs_scalar` — `0.5791x` / `0.5269x` vs `swar_run5`, `0.1403x` vs `origin/master`):**
   - Cycle 48's `swar_run5` tested `5` byte offsets per `8`-byte load (`& 0x0000_00ff_ffff_ffff`).
   - Loading four overlapping `u64` slices `sub[0..8], sub[1..9], sub[2..10], sub[3..11]` from `rle[i..i+11]` (`swar_run8_at`) tests **all 8 byte positions** in a full `u64` without width masking (`d = (v0 ^ v1) | (v0 ^ v2) | (v0 ^ v3)`), and pairing two independent `swar_run8_at(rle, i)` + `swar_run8_at(rle, i + 8)` lanes (`if (z0 | z1) == 0 { i += 16; continue; }`) advances **`16` bytes per loop branch** (`3.2x` fewer branches than `swar_run5` with 2-way superscalar ILP), plus single-slice `&[c; 16][..run_len]` short-run appends in `unrle_scalar`.

2. **`c57_fast_fill` (`src/rle.rs:unrle_in_place` — `0.9879x` on `detect_clmul`):**
   - Replaced dynamic `buf[r_start..r_start + run_len].fill(c)` (`libc::memset` PLT call on every `4..32`-byte run) with safe overlapping-endpoint fixed-size copies (`2 x [u8; 4]` for `4..=8 B`, `2 x [u8; 8]` for `9..=16 B`, `2 x [u8; 16]` for `17..=32 B`) strictly bounded inside `[r_start, r_start + run_len)` so `src_{r+1}` at `j_r + 5` is never overwritten prematurely.

3. **Killed Candidate (`c57_tile16k`, `TILE = 16 KiB` vs `64 KiB` in `unrle_in_place`):**
   - **One-line kill:** Shrinking `unrle_in_place`'s CRC tile from `64 KiB` to `16 KiB` regressed `detect_clmul` to `1.0171x` (`2.82 -> 2.87 ms`) because `clmul_update`'s fixed fold prologue runs `4x` more often while `64 KiB` already stays hot in private L2. Kept `TILE = 64 KiB`.

---

## 2. Same-Binary A/B + A/A Results (`c57_rle_screen.rs` & `c48_scalar_rle_ab`)

| Kernel & Benchmark (`64` `wikidata-l9` blocks) | Baseline | `A/A` | Candidate | Ratio | Verdict |
| :--- | ---: | ---: | ---: | ---: | :---: |
| `unrle_scalar` vs Cycle 48 `swar_run5` (`25` rds) | `10.84 ms` | `0.9951x` | **`6.28 ms`** | **`0.5791x` (`1.73x` faster)** | **ADOPT** |
| `find_runs_scalar` vs Cycle 48 `swar_run5` (`25` rds) | `9.90 ms` | `0.9982x` | **`5.22 ms`** | **`0.5269x` (`1.90x` faster)** | **ADOPT** |
| `unrle_in_place` (`detect_clmul`, `FastFill64k`, `25` rds) | `2.82 ms` | `1.0002x` | **`2.78 ms`** | **`0.9879x`** | **ADOPT** |
| `unrle_in_place` (`detect_clmul`, `Tile16k`, `25` rds) | `2.82 ms` | `1.0002x` | `2.87 ms` | `1.0171x` | **KILL** |
| `rle::unrle` scalar vs `origin/master` (`15` rds) | `48.74 ms` | `0.9817x` | **`6.84 ms`** | **`0.1403x` (`7.13x` faster)** | **ADOPT** |
| Full `2T` scalar decode vs `origin/master` (`8` streams, `15` rds) | `783.40 ms` | `0.9996x` | **`596.52 ms`** | **`0.7615x` (`1.31x` faster)** | **ADOPT** |

All `rle::tests` (`4/4`) and `trace::tests` (`5/5`) pass with zero warnings.

Next cycle (`Cycle 58`, `20/20`): Final rebase check against `origin/master`, full qualification across all unit/integration suites + `1,500`-case fuzz + `examples/ab` + `1..20T` push-streaming A/B & peak RSS audit, and publication of **Final Handoff**, then **STOP**.
