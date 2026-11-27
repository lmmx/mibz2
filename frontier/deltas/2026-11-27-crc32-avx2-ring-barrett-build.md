claim: crc32-avx2-ring-barrett-build
by: antigravity
outcome: partial
evidence: src/crc.rs; docs/journal/2026-11-27-explore/crc32-bzip2-dissection.md; examples/crc32_bzip2_dissection.rs; frontier/claims/crc32-avx2-ring-barrett-build.toml; frontier/surface.tsv

**What was asked:** Implement the 8-slot ROB-drained AVX2 `vpshufb` ring + 8-lane `vpclmulqdq` + zero-table Barrett reduction CRC-32/BZIP2 kernel in `src/crc.rs` with full B5 safety invariants, Miri verification, `objdump -d` Port-5 / zero-`TABLES` / zero-`vzeroupper` audit, 7-corpus SHA-256 parity, and paired A/B + CI `bench`, then submit for independent B5 review.
**What I found & built:**
- **3-tier runtime dispatch in `src/crc.rs::update_with`:**
  1. `len >= 1280 && k.avx2 && k.clmul` -> `#[inline(never)] avx2_ring8_update`: `MaybeUninit<Ring32B8>` (1024 B 64B-aligned stack ring, zero stack zero-fill), 4× 256-bit `vpshufb` + 8-lane (128 B/iter) `vpclmulqdq`, strictly in-bounds `prefetcht0` (`q+512`, `q+576` for `b < blocks - 11`), zero-table Barrett tail (`96 B` `TAIL_TABLES`).
  2. `len >= 128 && k.avx2 && k.clmul` -> `#[inline(never)] vex_8x_barrett_update`: 8-lane 128-bit VEX `vpclmulqdq` + zero-table Barrett (0 `ymm`, 0 `vzeroupper`, 0 `TABLES`).
  3. `k.clmul` (`1..128` on AVX2 CPUs + all lengths on non-AVX2 PCLMULQDQ CPUs) -> `#[inline(never)] sse_barrett_update`: 4-lane SSE + 16B fold + 96B sliding tail + pure `pclmulqdq` Barrett (`short_barrett` for `1..15`, 0 `ymm`, 0 `vzeroupper`, 0 `TABLES`).
  4. `!k.clmul` -> unchanged portable `update` (`TABLES` preserved for scalar/Miri).
- **Disassembly (`objdump -d` on baseline x86-64 release `ab`):** `avx2_ring8_update` emits `55` loop insns (`27.5/64B`), `16 vpclmulqdq + 4 vpshufb ymm + 0 vextracti128 = 10.0 Port-5 uops/64B`, `2 prefetcht0`, `zero_init_stores=0`, `TABLES=0`, `vzero=1`; both `vex_8x_barrett_update` and `sse_barrett_update` emit `ymm=0`, `vzero=0`, `TABLES=0`.
- **Verification:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` + `--features trace` + `--features forward-anchor-lanes` (0 warnings); `cargo test --release --lib -- crc::` (5/5, all lengths `0..=5000`, all 64 alignments × 6 states × 43 thresholds across scalar, SSE-only `restrict(false, true)`, and AVX2+CLMUL); `decode` (14/14), `pipelined` (12/12), `service_default` (4/4); Miri `crc::` (4/4 PASS, 0 UB); 7/7 corpora SHA-256 parity vs `libbzip2` & `40956c04` at `t=1, 4` (`base` & `jcc`).
- **Performance (direct production `src/crc.rs` vs old `clmul_update`, CPU 0 baseline x86-64, 5 repeats):** `64 B` `-34.4%` F, `512 B` `-32.3%` F, `32 KiB` `-17.8%` (`16.22` vs `13.33 GB/s`), `64 KiB` `-17.4%` F (`16.18` vs `13.36 GB/s`), `900 KiB` `-18.1%` F (`16.17` vs `13.25 GB/s`), L3-cold `wikidata` `-16.2% / -14.1% / -16.5%` (3/3 DISJOINT FASTER); `phases.rs` CRC stage `0.52 -> 0.29 c/B` (`wikidata`), `0.25 -> 0.19 c/B` (`text`), `0.22 -> 0.19 c/B` (`random`); peak RSS `0.994x–1.008x`.
**What changed:** `src/crc.rs`, `frontier/claims/crc32-avx2-ring-barrett-build.toml`, `frontier/deltas/2026-11-27-crc32-avx2-ring-barrett-build.md`, `frontier/surface.tsv`.
**What's next:** Independent B5 safety review by a different agent on the PR (new `unsafe` SIMD intrinsics & `MaybeUninit<Ring32B8>` in `src/crc.rs`), checking official CI `miri`, `rust`, `fuzz-smoke`, and paired `bench` verdict before merge.
**Caveats:** VM `ab` whole-decoder runs had host load avg 5–6; official CI paired `bench` provides clean whole-stream confirmation.
