# Multi-symbol 10-bit root tables (`K = 3` in 4 B, `K = 4` in 8 B): STOP

Probe for claim `multi-symbol-huffman-table` (26 November 2026). Standalone research prototype in
`examples/huff_multi_symbol_probe.rs` (`#![forbid(unsafe_code)]`; zero `src/` changes) against
`corpora/wikidata.bz2` (431 blocks, 24,979,299 symbols, SHA-256 `82f65ace…9915` verified) on a
2-vCPU Haswell Xeon @ 2.30 GHz with **32 KiB L1d / 256 KiB L2** (matching the i9-10900K's L1d/L2
sizes).

**Verdict under the frozen stop test: STOP.** Neither 4-byte packed `K = 3` (24 KiB for 6 tables)
nor 8-byte `K = 4` (48 KiB for 6 tables) comes close to the frozen gate (≥ 8% cut to A3 = decode +
tables summed over all 431 blocks, 7 interleaved repeats, non-overlapping spread). No build claim
is admitted; a row is appended to `frontier/dead-ends.tsv`.

## 1. Design and verification

- **Baseline (`k2_shipped`):** Faithful mirror of `src/huff.rs` (`Huff::build_into` with Canonical
  Template-Clone pairs + `SubLut`, `decode_group_win` 264 B window with `STEPS = 5`, `decode_group_slice`
  fallback) and `src/block.rs` lines 210–262. Block headers, selectors, and code lengths are parsed
  once outside the clock.
- **4-byte packed `K = 3` (`k3_4b`, 4 KiB/table, 24 KiB for 6 tables):** Preserves the shipped L1d
  footprint by packing three 8-bit symbols (`0..=255`) into bits `6..29`, `total_bits` (`1..=10`,
  bits `4..5 == 0` so `skip_short_shl` stays mask-free) in bits `0..5`, and `cnt` (`0..=3`) in
  bits `30..31`. Symbol `256` (rare MTF-255 tail; occurs **0 times** in `corpora/wikidata.bz2`)
  uses a `SYM256` tag in `cnt == 0` alongside `EOB_FAST`, `SUB_PTR`, and `BAD`. Group-boundary
  overshoot at `k = 48` or `k = 49` reads `sym_len[s]` (`[u8; 258]`, cold outside tails), with a
  branch-lean tail handling `c >= 2` at `k = 48` in one path and any `c >= 1` at `k = 49` directly.
  Built via 3-pass Canonical Template-Clone (`K = 1 -> K = 2 -> K = 3`).
- **8-byte packed `K = 4` (`k4_8b`, 8 KiB/table, 48 KiB for 6 tables):** Same 8-bit symbol lanes in
  `u64` (`bits 6..37`), `total_bits` in `0..5`, `cnt` (`0..=4`) in `61..63`, built via 4-pass
  Canonical Template-Clone.
- **Runt gating (`*_gated`):** Across the 431 blocks, all 198 runts (`< 1,000` symbols; 8,295
  symbols total, 208 built tables) have `n_sel <= 20`, while all 233 full blocks (24,971,004
  symbols, 1,398 built tables) have `n_sel >= 1,413`. Gating at `n_sel >= 64` drops Pass 2+3+4 on
  every runt (leaving valid `cnt = 1` entries with zero decode-loop branches).
- **Exactness:** All 11 arms match a standalone canonical bit-by-bit decoder symbol-for-symbol and
  ending-bit-position on all 431 blocks (24,979,299 symbols) plus a synthetic `alpha = 258` edge
  case exercising `sym = 256` (len 3), `EOB` (len 4), `len = 12` `SUB_PTR`, and `k = 47..49`
  boundary straddling.

## 2. Lookup census (233 full blocks, 499,534 groups, 24,971,004 symbols)

Matches `examples/huff_lookup_census.rs` to four decimal places:

| K | root lookups | root/sym (+slow) | vs `K = 2` | `c = 0` (slow) | `c = 1` | `c = 2` | `c = 3` | `c = 4` | tail entry share |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 2 | 13,702,666 | 0.5487 (0.5556) | — | 170,331 | 2,263,997 | 11,268,338 | — | — | `k=49`: 43.9% |
| 3 | 12,004,970 | 0.4808 (0.4876) | −12.39% | 170,331 | 2,237,061 | 6,229,122 | 3,368,456 | — | `k=48`: 56.0%, `k=49`: 41.3% |
| 4 | 11,595,246 | 0.4643 (0.4712) | −15.38% | 170,331 | 2,253,140 | 6,187,878 | 1,763,811 | 1,220,086 | `k=47..49` |

## 3. Seven-repeat interleaved A3 timings (all 431 blocks)

Single `Instant::now()` over all 431 blocks (`build_tables + decode_syms`) after 3 warmup passes,
7 interleaved repeats with rotated arm order per repeat.

### 3a. Default release build (baseline x86-64, matching shipped binary)

| arm | min ms | med ms | max ms | vs `k2_shipped` | overlap? | ns/sym | tbl med ms | dec med ms | full ns/sym |
|---|---:|---:|---:|---:|:---:|---:|---:|---:|---:|
| `k2_shipped` | 75.457 | **76.852** | 78.500 | +0.00% | — | 3.077 | 5.744 | 67.726 | 2.710 |
| `k2_gated` | 75.727 | 76.153 | 77.267 | −0.91% | YES | 3.049 | 5.194 | 65.742 | 2.631 |
| `k2_batched` | 75.971 | 76.415 | 77.268 | −0.57% | YES | 3.059 | 5.586 | 67.190 | 2.689 |
| `k2_slice` | 77.123 | 77.516 | 80.950 | +0.86% | YES | 3.103 | 5.975 | 70.864 | 2.836 |
| `k3_4b` | 78.501 | **78.882** | 78.984 | +2.64% | NO (slower) | 3.158 | 7.919 | 67.249 | 2.691 |
| `k3_4b_gated` | 77.951 | **78.869** | 79.451 | +2.62% | YES | 3.157 | 7.450 | 67.063 | 2.683 |
| `k3_4b_batch5` | 77.380 | **77.692** | 78.824 | +1.09% | YES | 3.110 | 7.404 | 66.710 | 2.669 |
| `k3_4b_batch4` | 77.749 | 79.435 | 82.811 | +3.36% | YES | 3.180 | 7.254 | 65.424 | 2.618 |
| `k4_8b` | 90.971 | **91.432** | 92.242 | +18.97% | NO (slower) | 3.660 | 12.003 | 73.729 | 2.951 |
| `k4_8b_gated` | 89.838 | **90.421** | 93.172 | +17.66% | NO (slower) | 3.620 | 11.409 | 75.190 | 3.009 |
| `k4_8b_batched` | 90.057 | 91.138 | 93.247 | +18.59% | NO (slower) | 3.649 | 11.147 | 75.476 | 3.019 |

*(In an earlier single-pass-warmup run with `k3_4b_compact` refill-on-demand loop, `k3_4b_compact`
regressed +15.2% because per-lookup `r.live < 10` checks add a branch every step.)*

### 3b. Native build check (`RUSTFLAGS="-C target-cpu=native"`, BMI2 `shlx` + AVX2)

| arm | min ms | med ms | max ms | vs `k2_shipped` | overlap? | tbl med ms | dec med ms | full ns/sym |
|---|---:|---:|---:|---:|:---:|---:|---:|---:|
| `k2_shipped` | 65.720 | **67.699** | 69.863 | +0.00% | — | 5.474 | 61.583 | 2.465 |
| `k2_gated` | 64.334 | 65.197 | 67.592 | −3.70% | YES | 5.179 | 61.888 | 2.476 |
| `k3_4b` | 66.276 | 69.510 | 72.839 | +2.67% | YES | 7.705 | 61.932 | 2.478 |
| `k3_4b_gated` | 65.697 | 67.148 | 69.544 | −0.82% | YES | 7.050 | 60.923 | 2.437 |
| `k3_4b_batch5` | 65.122 | **66.970** | 70.476 | −1.08% | YES | 6.992 | 61.083 | 2.444 |
| `k4_8b` | 76.834 | 79.610 | 85.319 | +17.59% | NO (slower) | 9.203 | 70.401 | 2.818 |
| `k4_8b_gated` | 75.898 | **77.588** | 82.251 | +14.61% | NO (slower) | 8.602 | 69.349 | 2.775 |

## 4. Why −12.4% fewer lookups fails to move A3 (overhead decomposition)

Disassembly (`objdump -d`) confirms `dec_k3_win`'s inner step is textbook clean (19 instructions,
zero bounds checks, zero spills, single 4-byte LUT load). The full-block `nostore` diagnostic
(replacing `g_out` writes and symbol shifts with an in-register accumulator on the 233 full blocks)
exposes every piece of where the −12.4% lookup reduction goes:

| build | `k2` full dec | `k2` nostore | `k2` store+extract | `k3` full dec | `k3` nostore | `k3` store+extract | `k3` vs `k2` nostore |
|---|---:|---:|---:|---:|---:|---:|---:|
| baseline x86-64 | 65.617 ms (2.628 ns/s) | 62.283 ms (2.494 ns/s) | 0.134 ns/s (5.1%) | 64.943 ms (2.601 ns/s) | 58.507 ms (2.343 ns/s) | 0.258 ns/s (9.9%) | **−6.06%** |
| `target-cpu=native` | 62.213 ms (2.491 ns/s) | 59.264 ms (2.373 ns/s) | 0.118 ns/s (4.7%) | 60.306 ms (2.415 ns/s) | 55.598 ms (2.226 ns/s) | 0.189 ns/s (7.8%) | **−6.19%** |

Four mechanisms combine to extinguish the −12.4% lookup gain:

1. **Per-group tail amplification halves the gain even with zero stores (`−12.4%` lookups → `−6.1%` nostore):**
   With `K = 3`, the unrolled 5-step main loop must stop at `k > 35` (`GROUP - 3 * STEPS`, vs `40`
   for `K = 2`), pushing ~10 symbols/group into the branch-checked tail loop, and **97.3% of groups**
   enter `k = 48` (56.0%) or `k = 49` (41.3%) versus 43.9% in `K = 2`. That fixed per-group tail
   plus refill/selector overhead caps even a zero-store `K = 3` loop at **−6.1%** (already below
   the 8% stop test).
2. **Branchless 3-symbol writes double the store+extract cost (`+3.1 ms` baseline / `+1.8 ms` native):**
   Writing 3 `u16`s unconditionally every lookup issues `3 × 12.00M = 36.0M` stores + byte extracts
   vs `2 × 13.70M = 27.4M` for `K = 2` (+31% more store/extract uops), raising store+extract from
   `0.134` to `0.258 ns/sym` (5.1% → 9.9% of decode) and consuming nearly the entire `−3.78 ms`
   `nostore` saving.
3. **Pass-3 table build adds `+1.7–2.2 ms` (+30–38% of tables, +2.3–3.0% of A3):**
   Selector gating (`n_sel >= 64`) cuts runt table builds cleanly (`0.647 -> 0.144 ms`, saving
   `0.50 ms` on `K = 3` and `0.31 ms` on `K = 2`), but the 1,398 full-block tables still cost
   `+1.71 ms` more to build for `K = 3` (`7.450` vs `5.744 ms`) and `+5.67 ms` more for `K = 4`
   (`11.409 ms`).
4. **L1d spill dominates `K = 4` (`48 KiB > 32 KiB` L1d):**
   6 × 8 KiB tables spill 32 KiB L1d, slowing `K = 4` decode by **+8.9% to +14.3%** despite
   −15.4% fewer root lookups, for a net **+14.6% to +19.0%** A3 regression.

**Incidental observations (outside the gate, for the record):**
- **`k2_shipped` itself sped up −11.9%** (`76.85 -> 67.70 ms`) under `-C target-cpu=native` because
  BMI2 `shlx` replaces 3-uop/2-cycle `shl %cl` on the critical bit-buffer shift chain (shipped
  `src/huff.rs` compiles without BMI2 dispatch). Whether runtime BMI2 dispatch for `decode_group_win`
  pays on the i9 (where `shl %cl` vs `shlx` is also 1 vs 2 cycles on Comet Lake) is a separate
  question from multi-symbol tables.
- **`k2_slice` was +0.9% to +4.6% slower than `k2_shipped`**, confirming `decode_group_win`'s
  264-byte window pays for itself.
- **Runt gating on shipped `K = 2` (`k2_gated`)** saves ~`0.31 ms` on runt table builds (~0.4–0.9%
  of A3), well below the 3% line on its own.

## 5. Caveats

- Timed on a 2-vCPU Haswell Xeon VM (32 KiB L1d / 256 KiB L2, no PMU), not the i9-10900K.
  L1d/L2 capacities match the i9 exactly, and the result corroborates §82's negative result on #870
  (which tested 8-byte `K = 3` on both i9 and EPYC).
- Because even the zero-store `K = 3` ceiling is −6.1% on decode before adding 3-symbol stores or
  Pass-3 builds, and the best real `K = 3` is +1.1% (baseline) / −1.1% (native), there is no
  plausible path for `K = 3` or `K = 4` at `B = 10` to clear ≥ 8% of A3 on the i9.
