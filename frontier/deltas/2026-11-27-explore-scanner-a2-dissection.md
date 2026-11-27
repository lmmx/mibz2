claim: none
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/scanner-a2-dissection.md, examples/scanner_a2_dissection.rs, frontier/claims/scanner-a2-fast-build.toml, frontier/surface.tsv

**What was asked:** Cold dissection and safe-Rust (`#![forbid(unsafe_code)]`) redesign probe of Serial Scanner `A2` (`A2a..d`: selectors, delta code-lengths, `Huff::build_into`) on top of post-`#1034` `S'`.
**What I found — VERDICT: PASS** across all three compiler builds (baseline x86-64, `-C target-cpu=native`, and `-C target-cpu=native` + Intel JCC-erratum 32B branch alignment control; 7 interleaved repeats, steal-free thread CPU):
- **Exactness:** `1,002/1,002` synthetic edge-case tables (`alpha=258`, `sym=256`, `eob=257`, incomplete/oversubscribed) + `2,009/2,009` real tables across 6 corpora bit-identical LUTs; 100% symbol-for-symbol and ending-bit-position exact vs `mibz2::stream::Scanner` across all 8 full `A2+A3` arms.
- **Three winning safe-Rust levers:**
  1. `A2b_direct_runs`: branchless 1-iter/run `leading_zeros()` selector MTF emitting `(group, run_len)` directly (cuts standalone `A2b` `-38.4%` base / `-33.1%` native / `-38.5%` JCC-32B disjoint, and eliminates `#1034`'s `501k`-byte `sel_buf` re-scan inside `A3`);
  2. `A2c_lut_multi`: `4 KiB` compile-time 8-bit multi-symbol delta LUT (`3.46-5.25` lens/lookup, O(1) range guard; cuts standalone `A2c` `-63.1%` base / `-66.7%` native / `-66.6%` JCC-32B disjoint);
  3. `A2d_direct_g64` / `A2d_direct_t16`: single-write fused `K=2` root fill + `split_at_mut` + tail-only (`1024 - filled_to`, mean `9.1` entries on wikidata) sub-LUT fill, dropping `d_arr[0..1024]` and redundant `K=1` overwrites (cuts standalone `A2d` `-24.0%` base / `-36.7%` native / `-24.8%` JCC-32B disjoint).
- **Combined `wikidata.bz2` (431 blks):** standalone `A2a..d` (`A2_cand_tot_g64`) **`-32.3%` base (`8.946->6.057 ms`), `-37.7%` native (`8.917->5.560 ms`), `-31.6%` JCC-32B (`7.434->5.082 ms`)**, all disjoint (`>= 25%` gate PASS). Joint end-to-end post-`#1034` `S'` (`S_all_g64` / `S_all_t16`): **`-5.71% / -6.40%` base**, **`-7.38% / -5.77%` native**, **`-4.44% / -4.98%` JCC-32B**, all disjoint (`>= 3.0%` gate PASS), projecting `~6.4-10.7 ms` off i9's `~145 ms` serial `S'`.
- **Off-wikidata guard corpora (`S_all_g64` across all 3 builds):** `text` `-2.47..-3.96%`, `records` `-0.20..-1.96%`, `random` `-0.37..-0.58%`, `runs` `-14.55..+0.70%`, `onebyte` exact (`0.00 ms`).
- **Ruled out:** `A2b_fast_mtf` (`+20..+63%` from 62/38% branch mispredictions); `A2c_lut1` (`+1..+66%` on uniform high-`1b` `records`/`random`); `A2d_split_vec_g64` (`-2.4..-18.2%`, superseded by single-write fused `K=2`); `sub_bits = 4` (neutral).
**What changed:** Added `examples/scanner_a2_dissection.rs`, `docs/journal/2026-11-27-explore/scanner-a2-dissection.md`, `frontier/claims/scanner-a2-fast-build.toml` (`status = "idea"`), and updated `frontier/surface.tsv` row `S`.
**What's next:** Admit `scanner-a2-fast-build` once PR `#1034` (`scanner-coalesced-u8-decode`) lands on `src/block.rs` and `src/huff.rs`.
**Caveats:** 2-core Haswell VM diagnostic (C5); i9-10900K numbers are projected pending workstation registration under `scanner-a2-fast-build`'s frozen stop test.
