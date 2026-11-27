# Coalesced byte-aligned Huffman scanner with BMI2 dispatch: design and i9-10900K kill test (registration)

*Frozen 26 November 2026 before any production timing or workstation run, under claim [`scanner-coalesced-u8-decode`](../../../frontier/claims/scanner-coalesced-u8-decode.toml). Decision rule under RULES C1a and `docs/team/PERFORMANCE-POLICY.md`; C2 is never relaxed after data is seen.*

---

## 1. Mechanism and motivation

Once `mtf-fold-build` and `forward-emission` cut 8-worker busy time below the serial scanner (`~152–161 ms` vs `177–179 ms` on i9-10900K, `docs/journal/2026-11-26-synthesis/second-pass.md` §1), serial Huffman scan (`S = A2 + A3`) becomes the direct 8-worker and 10-worker bottleneck. Cold dissection (`docs/journal/2026-11-26-explore/scanner-full-dissection.md`) and the isolated BMI2 ablation (`docs/journal/2026-11-26-explore/huff-bmi2-dispatch.md`, PR #1012, PASS) identified four orthogonal levers inside the shipped 24 KiB (`B = 10`, `K = 2`) L1d table footprint (`src/bits.rs`, `src/huff.rs`, `src/block.rs` only):

1. **Uncapped consecutive same-selector run coalescing (`src/block.rs`, `src/huff.rs`):** On `wikidata`, 62.4% of selectors have MTF index `j = 0`, collapsing 499,534 50-symbol groups into 188,898 same-table runs (`-62.2%` group boundaries, tail loops, and `k == 49` branches). Per-group `Truncated` / `overran` parity is preserved mathematically: each symbol consumes at most 20 bits (125 B per 50-symbol group), so coalescing `safe_m = (data_len - (r.pos + 16)) / 125` groups makes overrun impossible inside the run; any trailing groups within 125 B of EOF fall back to 1-group-at-a-time decoding with exact per-group `r.overran(data)` checks.
2. **Byte-aligned 8-bit `SYM1`/`SYM2` root LUT + low-byte `skip_short_shl_u8` (`src/huff.rs`, `src/bits.rs`):** Packs `total_bits` in bits `0..7` (bits `4..7 == 0`), `SYM1` in `8..15`, `SYM2` in `16..23`, `LEN1` in `24..28`, `CNT` in `30..31`, and routes literal symbol `256` (`alpha == 258` only; 0 occurrences in `wikidata.bz2`) to a `SYM256` tag in `cnt == 0`. Reading `self.live -= (e as u8) as u32` breaks LLVM's `and $0x3f` CSE off the `buf` shift chain, replaces both `& 0x1ff` masks with zero-extending byte extracts (`movzbl`), and pairs with runt-gated pair building (`n_sel >= 64`).
3. **Zero-skip `syms` reuse via an L1d-hot `[u16; 1042]` (`2 KiB`) persistent chunk buffer (`src/block.rs`, `src/huff.rs`):** Replaces `syms.resize(max_out, 0)` + `syms.truncate(n)` (which zeroed 40.23 MiB across 235/431 blocks in an 8-worker LIFO pool after runt truncations) with `syms.clear()` + chunked `syms.extend_from_slice(&chunk[..ck])` where `ck` persists across coalesced runs and flushes only when `ck > 1000` or on `EOB` (`[u16; 18]` batch slices in the 5-step hot loop, `ck & 1023` in the tail, zero per-step bounds checks, zero `unsafe`). Preserves `syms.capacity() == MAX_SYMS` (`src/trace.rs` memory ledger invariant).
4. **Block-level runtime BMI2 (`shlx`/`shrx`) dispatch (`src/block.rs`):** Hoists `#[cfg(target_arch = "x86_64")] if std::is_x86_feature_detected!("bmi2")` to the block level (431 calls on `wikidata`, avoiding the +3.26% per-group dispatch penalty found in PR #1012). `src/bits.rs` and `src/huff.rs` remain `#![forbid(unsafe_code)]`; `src/block.rs` uses `#![deny(unsafe_op_in_unsafe_fn)]` with a single `unsafe` call to `decode_runs_bmi2`, whose body executes the exact same safe `decode_runs_inner` helper as `decode_runs_scalar`.

---

## 2. Correctness and safety obligations (VOID if any fail)

1. **Byte- and error-identical output:**
   - Full `cargo test` (including `table_decodes_as_libbzip2`, `symbols_read_as_in_memory`, `trace::tests::ledger_matches_a_workers_buffers`, and `tests/decode.rs` / `tests/service.rs`);
   - Dedicated unit tests exercising `alpha = 258` (`sym = 256` in both root LUT and `SubLut`, `eob = 257`), `eob = 256` (`alpha = 257`), runt blocks (`n_sel < 64`), multi-group coalesced runs (`M > 10`), mid-run `Truncated` / `BadHuffmanCode` / `SelectorOverrun` exact bit positions, and exact BMI2-vs-scalar parity on random + edge-case blocks;
   - `fuzz-smoke` clean;
   - Exact SHA-256 byte parity on `corpora/wikidata.bz2` (all 431 blocks, 1W and multi-thread) and all `ab gen` corpora (`text`, `runs`, `records`, `random`, `pattern`, `onebyte`).
2. **Memory safety:**
   - `src/bits.rs` and `src/huff.rs` retain `#![forbid(unsafe_code)]`.
   - `src/block.rs` enforces `#![deny(unsafe_op_in_unsafe_fn)]`; the sole `unsafe` expression is the `is_x86_feature_detected!("bmi2")`-guarded call to `decode_runs_bmi2`, documented with a `// SAFETY:` invariant and independently reviewed before merge.
   - Scalar path (`decode_runs_scalar` / `read_item`) passes Miri under `cargo miri test`.
3. **Codegen verification:**
   - Default baseline x86-64 release `objdump -d` confirms `decode_runs_bmi2` emits `shlx` / `shrx` with zero `and $0x3f` on the shift chain, zero `& 0x1ff`, zero bounds checks in the 5-step hot loop, and zero `vzeroupper`.

---

## 3. Workstation i9-10900K decision rule (frozen; C2 applies)

### 3.1 Run design

- **Machine & corpora:** i9-10900K, pinned scanner, `wikidata ×4` (`0b820d14...ce08e`) plus `ab gen` `text` and `runs` (and full `ab` corpora under `PERFORMANCE-POLICY.md`), interleaved head vs baseline (`origin/master` merge-base) across `k ∈ {1, 2, 6, 8, 10}` threads (10–12 randomised paired rounds), plus single-thread `examples/phases.rs` (`read` = `A2 + A3`) on `wikidata ×4` and `C`-ideal paired 8W/10W if worker improvements have not yet landed.
- **Estimated workstation time:** < 15 minutes (below the 30-minute threshold of `PERFORMANCE-POLICY.md` §4a).

### 3.2 Pass / STOP criteria (exact transcription of `scanner-coalesced-u8-decode.toml`)

**PASS (adopt / keep merged)** only if ALL of the following hold:
1. **Serial scan `S` gain:** Cuts serial scan `S` (`A2 + A3` on `wikidata ×4`, 1W scan or paired `C`-ideal 8W/10W) by **≥ 8.0%** (95% CI upper bound ≤ `0.920` / non-overlapping `[min..max]` spreads).
2. **Wall-clock capture:** Cuts **10W wall** (or paired post-worker-improvement 8W wall) by **≥ 3.0%** (95% CI upper bound ≤ `0.970` / non-overlapping spreads).
3. **No regression:** Zero regression beyond noise on `runs`, `text`, or `1W–10W` `wikidata` (95% CI lower bound ≤ `1.005`, and secondary corpora within `PERFORMANCE-POLICY.md` §2 rule 3).
4. **Memory:** Peak RSS ≤ **1.05×** baseline at every thread count and corpus.

**Otherwise STOP:** Revert or do not adopt, and record a scoped dead-end row in `frontier/dead-ends.tsv`.

### 3.3 VOID conditions

The workstation run is **VOID** if:
- Any correctness, parity, or safety check in §2 fails;
- The machine is not the i9-10900K or any check pass fails byte-parity;
- Any judged 95% confidence interval is undefined or the run fails to exercise `decode_runs_bmi2` on the BMI2-capable i9-10900K.
