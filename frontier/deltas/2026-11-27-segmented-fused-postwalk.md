claim: segmented-fused-postwalk
by: antigravity
outcome: review
evidence: src/bwt.rs; src/rle.rs; src/lib.rs; docs/journal/2026-11-27-explore/bwt-segmented-dissection.md

Implemented `segmented-fused-postwalk` in `src/bwt.rs`, `src/rle.rs`, and `src/lib.rs`, ready for independent B5 review (`status = "review"`, PR #1183).

### Changes
1. **Single-pass fused reverse-assembly + RLE1 + L2-hot CRC32 (`src/bwt.rs`, `src/rle.rs`, `src/lib.rs`):**
   - Added panic-safe thread-local scoped `FusedPostwalk` hook in `src/bwt.rs` (owned/append RAII guard cleared on `Drop`) wired into `src/lib.rs` `decode_block`, `decode_owned`, and `column` when `can_fuse_rle()` (`!ideal-rle1::Replay`). Public `Segmented::invert_column` / `invert_runs` and research replay paths remain unchanged.
   - Added `SegScratch::assemble_reversed_rle_crc` dispatching AVX2+BMI1 (`fused_rev_rle_crc_buf_avx2` / `fused_rev_rle_crc_cycle_avx2`, single `#[target_feature(enable = "avx2,bmi1")]` boundary, zero per-chunk `vzeroupper`) with an exact scalar mirror (`fused_rev_rle_crc_buf_scalar` / `fused_rev_rle_crc_cycle_scalar`) and `rle::decode_chunk_scalar`. Reverses each segment in 32 B AVX2 strides into a 4 KiB L1d stack buffer, decodes RLE1 directly into `out` (hoisted capacity guard, `#[cold]` linear-extrapolating `grow_out` with `reserve_exact` on `start == 0` capped at bzip2's 52x RLE1 ceiling, zero-call 32 B AVX2 stores for `v0` literals and `<= 32` B runs), and folds CRC32 every 64 KiB L2-hot tile.
   - Added private `walk_back_room_private` / `walk_segments_room_private` skipping redundant `START`-bit cleanup inside private `Segmented::tt`, plus `walk_back_fused_small` for short `< MIN_SEGMENTED` blocks.
2. **Scoped revision from PR #1183 first CI `bench` read (AMD EPYC 7763 Zen 3):**
   - Kept `#1080`'s proven `u32` `build_lf` rather than `u64` store-combining: on 2-store-pipe AMD Zen 3, `u64` packing added `shl 32 + or` and partial-register overhead on `random`/`records` (`+0.6..+2.8%`), whereas keeping `#1080`'s `build_lf` retains 100% of the fused post-walk win and reduces the B5 `unsafe` review surface.
   - Replaced fixed `1.25x` incremental `grow_out` steps (which caused 7–18 `realloc`s and `+7.5..+14.3%` RSS on expanding `runs`/`onebyte` in `decode_owned`) with linear-extrapolated `reserve_exact` (`produced * rem_in / consumed + 1% + 16 KiB`), resizing uniformly expanding blocks once to ~1.01x exact final size.

### B5 `unsafe` Safety Proofs Summary (all documented inline with `// SAFETY:` comments)
- `FusedPostwalk` (`src/bwt.rs`): `dst_ptr: *mut Vec<u8>` is only installed by `enter_append(&mut Vec<u8>, Kernels)` on the current thread, cleared on `Drop` (panic-safe), and dereferenced strictly inside the synchronous `run_back_fused` call with no concurrent or nested borrows.
- `assemble_reversed_rle_crc` (`src/bwt.rs`): AVX2 kernels gated on `k.avx2` (`Kernels` invariant guarantees AVX2+BMI1); every segment index in `chain` is validated `< bufs.len()` before `get_unchecked`; `reverse_copy_*` receives disjoint source (`b[pos..pos+chunk]`) and destination (`buf[buf_len..buf_len+chunk]`) with `pos + chunk <= seg_take <= b.len()` and `buf_len + chunk <= BUF_CAP`.
- `fused_rev_rle_crc_*_avx2` & `grow_out` (`src/bwt.rs`): `out` capacity invariant `out_len + need <= cap` is checked before every write (growing via `#[cold] grow_out` which commits `set_len(cur_len)` before `reserve_exact`/`reserve` and refreshes `out_ptr`/`out_cap`); 32 B AVX2 stores for `v0` literals and `run <= 32` fills strictly verify `out_len + 32 <= cap`; `out.set_len(out_len)` is only called on ranges fully initialized by prior contents + writes in `[start..out_len]`.

### Verification
- **Miri 8/8 PASS** (strict provenance & Stacked Borrows, including `bwt::tests::miri_fused_postwalk_edges_and_parity` covering segment/4 KiB/64 KiB straddling runs, `RunUnfinished`, short cycles `m < n`, repeated cycles, expansive runs, and `rle::tests::miri_decode_chunk_scalar_splits`).
- **`cargo clippy --all-targets --all-features -- -D warnings`** clean, `cargo fmt --check` clean, `phase-counters`, `tests/in_place`, `bwt::`, `rle::`, `trace::` pass.
- **Exact-head CI `bench` (PR #1183, 1st run AMD EPYC 7763 before `grow_out` + `build_lf` refinement):** primary `text` **-4.9% (1T) / -8.1% (2T) / -8.5% (4T)** wall & **-4.1..-4.4% RSS**; primary `wikidata` **-0.8% 4T CPU** (0 regressions); 2nd exact-head CI run verifying refined `grow_out` + `#1080` `build_lf` on all 7 corpora.

**What's next:** Exact-head green CI (`miri`, `rust`, `bench`, `occupancy`, `walk-stats`, `fuzz-smoke`, `pre-commit.ci`, `no-ai-attribution`) and independent B5 review of `src/bwt.rs` and `src/rle.rs` `unsafe` blocks. Hold retained (`session = 767202602b3a`); reviewer may transfer via `frontier.py take-review`.
