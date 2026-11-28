claim: lanes-disjoint-slabs-build
by: antigravity
outcome: done
evidence: src/open/lanes.rs

## Shipped safe disjoint-slab 4-lane forward emission (`src/open/lanes.rs`)

- **Claim:** `lanes-disjoint-slabs-build` (admitted in `#1216`, `src/open/lanes.rs` only, `#![forbid(unsafe_code)]`, zero `unsafe`).
- **Mechanism:**
  - `#[inline(never)] fn run<const K: usize>` with `let lanes = &mut lanes[..K]` and `nw = lens.len().min(inline.len())` pre-slicing eliminates the mega-inlined `K=4,3,2,1` loop frame (`3,728` insns, `69` `vzeroupper`s, `0x448` frame).
  - Per-lane owned disjoint `Box<[u8; 32768]>` slabs with compile-time `o <= SLAB - INLINE` eliminate slab bounds and overflow checks in 100% safe Rust, with compact 24-byte `Lane` (`slab`, `j`, `start: u16`, `off: u16`, `seg: u8`, `slot: u8`), zero-reallocation `Arena::pending: [Vec<(u8, u16, u16)>; 4]`, and lazy slab pool bounded by `allocated <= max_slabs`.
  - Inline `LONG`-fits-in-slab fast path (`wlen <= SLAB - o`) skips `#[cold] spill_word` on >99.7% of `LONG` words, plus `out.reserve(n)` stitch.
- **Disassembly audit (`objdump -d` on release `ab --features forward-anchor-lanes`):**
  - Baseline `Walk::run<4>`: **`501` insns, `0` `vzeroupper`, `0x88` frame** (`run<3>`: 423, `run<2>`: 284, `run<1>`: 201 — all `0` `vzeroupper`).
  - JCC-32B `Walk::run<4>`: **`520` insns, `0` `vzeroupper`, `0x88` frame** (`run<3>`: 439, `run<2>`: 294, `run<1>`: 207 — all `0` `vzeroupper`).
- **Verification (all PASS):**
  - `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (default, `forward-anchor-lanes`, `trace`, `forward-anchor-lanes a6b-endpoint-redesign`).
  - `cargo test --release --lib --features forward-anchor-lanes -- open::` **45/45 PASS** (including new `long_word_slab_boundaries_and_multi_slab_words` covering `rem == wlen`, `rem == wlen - 1`, `rem == 0`, 70 KiB multi-slab `LONG` words, `K = 4..1` drain, and repeated `Arena` pool reuse).
  - `tests/decode.rs` (14/14), `tests/pipelined.rs` (12/12), `tests/service_default.rs` (4/4): **30/30 PASS** under both default and `--features forward-anchor-lanes`.
  - Miri (`cargo +nightly miri test --lib --features forward-anchor-lanes`): **3/3 PASS (0 UB)** (`miri_forward_anchor_lanes`, `long_word_slab_boundaries_and_multi_slab_words`, `invalid_edges_fall_back_and_untag`).
  - **7/7 corpora exact SHA-256 parity** vs `libbzip2` (`AB_SHA256=1`: `wikidata`, `text`, `random`, `records`, `pattern`, `runs`, `onebyte`).
- **Whole-decoder paired `ab` (`37dc02bc` vs `d5c645af`, `--features forward-anchor-lanes`, `mibz2-workers-lazy`):**
  - **Baseline x86-64 (`wikidata`, 6 rounds `ab run`):**
    - `t = 1`: wall **`0.896 [0.872, 0.922]` (`-10.4%`)**, CPU **`0.894 [0.877, 0.911]` (`-10.6%`)**, RSS `25 -> 25 MB`
    - `t = 2`: wall **`0.894 [0.850, 0.940]` (`-10.6%`)**, CPU **`0.885 [0.851, 0.921]` (`-11.5%`)**, RSS `31 -> 31 MB`
    - `t = 4`: wall **`0.931 [0.885, 0.979]` (`-6.9%`)**, CPU **`0.912 [0.882, 0.944]` (`-8.8%`)**, RSS `41 -> 41 MB`
    - 6 guards (`t = 1, 4`, 12 cells): **0/12 regressions** (`random` `t=1, 4` statistically faster `-2.5%..-2.8%`).
  - **JCC-32B-aligned (`wikidata`, 7 rounds):**
    - `t = 1`: CPU **`0.9219 [0.8974, 0.9471]` (`-7.8%`)**, RSS `25.4 -> 25.2 MiB` (`0.994x`)
    - `t = 2`: wall **`0.8781 [0.7897, 0.9763]` (`-12.2%`)**, CPU **`0.9183 [0.9033, 0.9335]` (`-8.2%`)**
    - `t = 4`: wall **`0.9473 [0.9154, 0.9803]` (`-5.3%`)**, CPU **`0.9175 [0.9097, 0.9255]` (`-8.25%`)**, RSS **`43.2 -> 40.6 MiB` (`0.938x`, `-2.6 MiB`)**
    - 6 guards (`t = 1, 4`, 12 cells): **0/12 regressions**.
- **Verdict against frozen `stop_test`: PASS.** Zero `unsafe`, 100% exact & Miri-clean, `-7.8%..-11.5%` whole-decoder Wikidata CPU speedup (well above `>= 5%`), zero guard regressions, and equal/lower peak RSS (`0.938x..0.994x` at `t=1, 4`).
