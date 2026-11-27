claim: scanner-coalesced-u8-decode
by: antigravity
outcome: partial
evidence: docs/journal/2026-11-26-scanner-coalesced-u8/registration.md; src/bits.rs; src/huff.rs; src/block.rs; corpora/wikidata.bz2 (82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915)

**What was asked:** Implement the four orthogonal serial-scanner levers (uncapped same-selector run coalescing, byte-aligned 8-bit `SYM1`/`SYM2` LUT with `SYM256` slow tag + low-byte `skip_short_shl_u8`, zero-skip `syms` reuse, and block-level BMI2 `shlx`/`shrx` dispatch) strictly inside `src/bits.rs`, `src/huff.rs`, and `src/block.rs`, with a frozen workstation i9-10900K registration and full correctness/safety verification.

**What I found:**
- **Correctness & safety (all PASS):** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --lib` (197/197 incl. `table_decodes_as_libbzip2`, `symbols_read_as_in_memory`, `alpha_258_sym256_eob_truncation_and_overrun`, `bmi2_and_scalar_decode_runs_match_exactly`, `miri_decode_runs_scalar_exact`, `trace::tests::ledger_matches_a_workers_buffers`), `tests/decode.rs` (14/14), `tests/service.rs` (10/10), `tests/mtf_kernels.rs` (4/4), `--profile fuzz` smoke (`classic` 12,240 cases / 0 failures, `pipeline` 113 cases / 0 failures), and exact byte parity across `wikidata` (all 431 blocks, 24,979,299 symbols) and all 6 `ab gen` corpora. `src/bits.rs` and `src/huff.rs` remain `#![forbid(unsafe_code)]`; `src/block.rs` uses `#![deny(unsafe_op_in_unsafe_fn)]` with a single `is_x86_feature_detected!("bmi2")`-guarded call to `decode_runs_bmi2`.
- **Disassembly (`objdump -d`, default baseline x86-64 release build):** `decode_runs_bmi2` hot 5-step loop is 15 instructions/step (`22/22 shlx`, `8/8 shrx`, `0 shl %cl`, `0 vzeroupper`, `0` bounds checks, `0` register spills, `0` `and $0x3f` on shift chain). Keeping `Reg` by value and persisting the `2 KiB` `[u16; 1042]` staging chunk cursor across runs eliminated per-step memory spills and cut `extend_from_slice` `memcpy` calls by ~8x.
- **VM timings (2-vCPU Haswell Xeon, 7 interleaved repeats, `scanner_dissection_probe`):** Live `mibz2::stream::Scanner` (`lib_scanner_seq`) ran **`71.994 ms`** on-CPU (`71.958..72.004`) / **`72.585 ms`** wall (`72.105..73.063`) vs `shipped_full_zero` **`79.992 ms`** on-CPU / **`81.249 ms`** wall (**-10.00% on-CPU / -10.66% wall, zero overlap**; **-13.8%** vs `pool16_shipped_zero` `84.217 ms`, zero overlap; **~-14.8%** vs `origin/master`'s `lib_scanner_seq` `83.98 ms`). Paired `ab run` (`head` vs `origin/master`, 3 rounds) improved `wikidata` 1T wall in 3/3 rounds (`0.764->0.744`, `0.793->0.770`, `0.789->0.763 s`, -20..-26 ms, RSS `27->27 MB`), `wikidata` 2T in 3/3 rounds, with neutral `text`/`runs`.

**What changed:**
- `src/bits.rs`: replaced `skip_short_shl` with `skip_short_shl_u8` (`live -= (e as u8) as u32`); removed unused `refill_win`.
- `src/huff.rs`: byte-aligned 8-bit `SYM1`/`SYM2` LUT + `SYM256` slow tag, runt-gated `build_into(..., build_pairs)`, persistent-chunk `decode_run` (`CHUNK_LEN = 1042`, `[u16; 18]` batch slices).
- `src/block.rs`: uncapped same-selector run coalescing with exact per-group overrun fallback (`safe_m` margin `125 B/group + 16 B`), `PAIR_GATE_SEL = 64`, zero-skip `syms.clear()`, block-level `decode_runs_bmi2` dispatch, and expanded unit/parity/Miri tests.
- `docs/journal/2026-11-26-scanner-coalesced-u8/registration.md`: frozen i9-10900K kill-test registration matching `scanner-coalesced-u8-decode.toml`.
- `frontier/claims/scanner-coalesced-u8-decode.toml`: moved to `status = "review"` with PR number (`stop_test` untouched). `frontier/surface.tsv` row `S` left untouched to avoid conflicting with open PR #1012 (`huff-bmi2-dispatch`).

**What's next:**
- Independent review + adversarial safety check of `decode_runs_bmi2` (`src/block.rs`), CI paired bench (`bench.yml` + `stages.py`) and MSRV/Miri verification, integration merge, and then the frozen i9-10900K workstation run under `docs/journal/2026-11-26-scanner-coalesced-u8/registration.md` to judge the `>= 8.0% S` / `>= 3.0% 10W` gate and update `surface.tsv` row `S`.

**Caveats:**
- Local timings are from a shared 2-vCPU Haswell Xeon VM, not the i9-10900K; final workstation verdict under the frozen stop test awaits i9 evidence.
- `cargo miri` is not installed in this VM; `miri_decode_runs_scalar_exact` ran natively here and awaits CI/reviewer Miri confirmation.
