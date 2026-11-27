claim: runs-front-and-fill-build
by: antigravity
outcome: partial
evidence: src/runs.rs; src/runs/tests.rs; docs/journal/2026-11-27-explore/runs-front-and-fill-dissection.md; frontier/claims/runs-front-and-fill-build.toml; frontier/surface.tsv

**What was asked:** Implement the `U1_u32` `front_avx2` redesign in `src/runs.rs` (64-symbol `u64` `vptest` Pass 1, branchless 256-bit `YMM_STEPS`, and `[u32; 256]` stack `cnt` with dead `Runs.per` compiled out in production), add word-boundary and `s >= 0x8000` unit/Miri tests, audit `objdump -d` and B5 safety invariants, verify 7-corpus SHA-256 parity and paired whole-decoder A/B, and submit PR `#1238` for independent B5 review without self-merging.
**What I found & built:**
- **Production changes (`src/runs.rs`):**
  1. **64-symbol `u64` `vptest` Pass 1:** processes 64 `u16` symbols per iteration (four 32B unaligned loads), writes full `u64` bitmap words directly to `f.lit` and `f.one` (eliminating 32→64 store-forwarding stalls), uses a single `_mm256_testz_si256` (`vptest`) on `eob_any` (zero `vpacksswb`/`vpermq`/`vpmovmskb` on non-EOB chunks), and writes the two trailing zero slack words inline (removing `memset` + `vzeroupper` from Pass 1).
  2. **Branchless 256-bit `YMM_STEPS` (`f01 = [f0 | f1]` in one YMM register):** replaces four 16B loads + 4×`vpshufb` + 2×`vpor` with two 32B aligned loads (`YmmStep`, `#[repr(C, align(64))]`) + `vpermq $0x4e` + 2×`vpshufb` + 1×`vpor` (`3` Port-5 uops vs `4`, `2` loads vs `4`, `1` YMM reg vs `2`). Legacy `STEPS` stays gated to `#[cfg(all(target_arch = "x86_64", feature = "mtf-fold"))]` for `src/runs/fold.rs`.
  3. **Zero-dead-`per` `[u32; 256]` stack `cnt`:** `Runs.per` has zero production readers in `src/` and is gated to `#[cfg(any(test, feature = "ideal-mtf"))]` (computed post-pass from `f.runs`), removing `movabs $0x100000000; or` on every literal, halving stack `cnt` (`2 KiB -> 1 KiB`) and `Runs` struct size (`-1 KiB`), and freeing `%r14` so LLVM hoists `YMM_STEPS` outside the hot Pass-2 loop.
  4. **Shipped `fill_avx2_trusted` untouched.**
- **Disassembly audit (`objdump -d` on release `ab`):**
  - Pass-2 hot loop: **`23 insns / 105 B`** baseline (`23 real insns + 1 align nop / 112 B` JCC-32B) vs shipped `27 insns / 147 B`; `%r14` holds hoisted `YMM_STEPS`; exact `3` Port-5 uops (`vpermq $0x4e` + 2×`vpshufb`) per literal; `0` hot-loop spills, `0` hot-loop `call`s, `0` hot-loop `vzeroupper`s.
  - Pass-1 hot loop: single `vptest %ymm11, %ymm11` per 64 symbols, exact unsigned `s <= 1` (`vpminuw` + `vpcmpeqw`), `0` hot spills/calls/`vzeroupper`.
- **Verification:**
  - `cargo fmt --check`, `cargo clippy -- -D warnings` (default `--all-targets`, `trace`, `forward-anchor-lanes`, `ideal-mtf`, `mtf-fold`, `scan-oracle`): **0 warnings**.
  - `cargo test --release --lib -- runs::`: **18/18 PASS** (including new `miri_runs_front_word_boundaries_and_edges` covering EOB at 0/1/31/32/62/63/64/65/126/127/128, runs 0..=22 crossing word boundaries, `s >= 0x8000`, `RunTooLong`/`BlockTooLong`/`no-EOB`, dirty `Runs` reuse, `per` parity, and `TT` parity).
  - `tests/decode.rs` (**14/14**), `tests/in_place.rs` (**7/7**), `tests/pipelined.rs` (**12/12**), `tests/service_default.rs` (**4/4**), `--features ideal-mtf --lib -- runs::tests::miri_runs_front` (**2/2**): all PASS.
  - Miri (`cargo +nightly miri test`): **PASS (0 UB)** on both default target AND `RUSTFLAGS="-C target-feature=+avx2,+bmi1,+bmi2,+lzcnt,+popcnt"` (directly interpreting `front_avx2` across all edge cases and `runs_front_kernel_selected`).
  - **7/7 corpora exact SHA-256 parity** vs `libbzip2` (`AB_SHA256=1`, baseline & JCC-32B, `mibz2` `t=1, 4` + `mibz2-workers-lazy` on `wikidata` `t=1, 4`).
- **Local whole-decoder paired A/B (`examples/ab.rs`, ABBA alternating, `559ca206` vs candidate):**
  - Baseline: `wikidata mibz2 t=1` wall `0.9614` (`-3.86%`, CPU `0.9831`, RSS `1.0061`), `t=4` wall `0.9940` (CPU `0.9960`, median RSS `1.0079`); `wikidata mibz2-workers-lazy t=1` wall `0.9276` (`-7.24%`, CPU `0.9390`), `t=4` wall `0.9468` (`-5.32%`, CPU `0.9394`).
  - JCC-32B: `wikidata mibz2 t=1` wall `0.9555` (`-4.45%`, CPU `0.9819`, RSS `1.0032`), `t=4` wall `0.9883` (CPU `0.9891`, median RSS `0.9851`); `text mibz2 t=1` wall `0.9363` (`-6.37%`, CPU `0.9647`), `t=4` wall `0.9967` (CPU `0.9939`); `wikidata mibz2-workers-lazy t=1` wall `0.9224 [0.8017..0.9448]` (`-7.76%`, 7/7 disjoint faster, CPU `0.9690`, median RSS `1.0016`), `t=4` wall `0.9550` (`-4.50%`, median RSS `0.9845`).
**What changed:** `src/runs.rs`, `src/runs/tests.rs`, `frontier/claims/runs-front-and-fill-build.toml`, `frontier/deltas/2026-11-27-runs-front-and-fill-build.md`, `frontier/surface.tsv`.
**What's next:** Await green CI (`miri`, `rust`, `bench`, `fuzz-smoke`, `pre-commit.ci`, `check`, `no-ai-attribution`) and independent B5 safety & evidence review on PR `#1238` before merge (live lease retained on `runs-front-and-fill-build`).
**Caveats:** Because `src/runs.rs` modifies `unsafe` AVX2/BMI2 intrinsics, B5 requires independent review before merge; CI paired `bench` provides the authoritative `PERFORMANCE-POLICY.md` §2 and `stop_test` verdict.
