# Carbon Cycle 54 (16 / 20) — `decode_group_win` 6-Cycle Critical-Path Latency Floor & Clean Kills

- **Date:** 2026-11-22
- **Branch:** `research/carbon-word-transport`
- **Base Candidate:** Interim Handoff 3 (`a1c04a1a` over `origin/master` `dd6450e6`)
- **Safety:** `#![forbid(unsafe_code)]` across all screened harnesses (`c54_huff_screen.rs`, `c54_huff_screen2.rs`)

---

## 1. Objective & Critical-Path Hypothesis

Following Cycle 49's sub-phase profile showing `huff::decode_group_win` accounts for `90.0%` (`wikidata-l9`), `73.0%` (`wikidata-l1`), and `95.4%` (`records-l9`) of `P1`, Cycle 54 tested whether eliminating off-critical-path ALU/mask instructions inside `step_win!` (`r.live -= e & 63`, `((e >> SYM) & 0x1ff)` masks, `win.get(pos..pos+8)`) can further reduce `P1` below Cycle 49's `59.98 ms` (`wikidata-l9`).

Note first an exact bit-level invariant check on **LSB Sentinel Bitbuffers** (`1 << (63 - live)`):
- In `bits::Reg::refill_win`, `self.buf |= v >> self.live` populates **all 64 bits** of `self.buf` with valid stream bits even when `self.live == 56` (`take = (63 - live) >> 3` leaves `1..8` valid lookahead bits below `live`), which is required for `STEPS = 5` (`5 * 10 = 50` consumed bits + `10`-bit 5th peek = up to `60 > 56` bits). Placing a `1` sentinel at bit `63 - live` (`bit 7` when `live == 56`) would clobber lookahead bits `0..7` and corrupt the 5th unrolled step whenever `4` steps consume `> 46` bits.

Instead, because `e` has `bits 4..5 == 0` and `5 * 10 = 50 < 64`, `used = used.wrapping_add(e)` accumulates the exact 5-step bit length mod `64` (`used & 63`) without per-step `& 63` masks (`DeferLive`). We screened four `100%` bit- and error-exact variants (verified across all 3 corpora + `1,500` mutated/truncated cases):

1. **`c54_refill_fast`:** Branchless `let p = self.pos & 255; win[p..p + 8]` inside `decode_group_win` (`pos <= 140 < 256`).
2. **`c54_defer_live` (`+ RefillFast` and isolated `DeferLiveOnly`):** Accumulate `used = used.wrapping_add(e)` across `STEPS = 5` and apply `r.live -= used & 63` once per 5-step unroll (flushing before `c == 0`).
3. **`c54_lut64` (`[u64; 1024]`, `8 KiB`/table):** Pre-pack `[sym1: u16, sym2: u16]` in `e as u32` for zero-mask single-`movl` symbol pair stores + `DeferLive`.
4. **`c54_u8pair32` (`[u32; 1024]`, `4 KiB`/table):** Keep `6 x 4 KiB = 24 KiB` L1d residency by storing `sym1: u8` (`bits 16..23`) and `sym2: u8` (`bits 24..31`) for `< 256` symbols (routing rare `s == 256` on full-256-byte alphabets via `SYM256_FAST_8`), removing both `& 0x1ff` masks.

---

## 2. Same-Binary `P1` A/B + A/A Results (`21–25` Interleaved Rounds)

### Run 1 (`c54_huff_screen.rs`, `21` Interleaved Rounds)

| Corpus | `Base` (ms) | `A/A` | `c54_refill_fast` | `c54_refill + DeferLive` | `c54_lut64` (`8 KiB`/tbl) |
| :--- | ---: | ---: | ---: | ---: | ---: |
| `wikidata-l9` | `62.71` | `1.0026x` | `1.0044x` | `0.9962x` | `1.0745x` |
| `wikidata-l1` | `21.08` | `0.9985x` | `1.0024x` | `0.9879x` | `1.0572x` |
| `records-l9` | `31.39` | `1.0101x` | `1.0143x` | `1.0171x` | `1.1656x` |

### Run 2 (`c54_huff_screen2.rs`, `25` Interleaved Rounds — Isolated `4 KiB` Variants)

| Corpus | `Base` (ms) | `A/A` | `DeferLiveOnly` | `c54_u8pair32` (`4 KiB`/tbl) |
| :--- | ---: | ---: | ---: | ---: |
| `wikidata-l9` | `59.98` | `1.0002x` | `1.0013x` | `0.9980x` |
| `wikidata-l1` | `20.11` | `1.0046x` | `0.9970x` | `1.0046x` |
| `records-l9` | `31.29` | `1.0065x` | `1.0086x` | `1.0545x` |

---

## 3. One-Line Kills & Hardware Floor Diagnosis

- **KILL `c54_lut64` (`1.0572x–1.1656x`):** Growing `lut` from `[u32; 1024]` (`4 KiB`) to `[u64; 1024]` (`8 KiB`) expands the 6-group table set from `24 KiB` (`<= 32 KiB` L1d) to `48 KiB` (`> 32 KiB` L1d), causing L1d misses across 50-symbol selector switches.
- **KILL `c54_DeferLiveOnly` (`1.0013x` / `0.9970x` / `1.0086x`):** `decode_group_win` is latency-bound at `5.8` cycles/step on the serial chain `r.buf -> (>> 54, 1 cyc) -> lut[idx] (L1d, 4 cyc) -> (shlq %cl, 1 cyc) -> r.buf`, so `r.live -= e & 63` executes for free in the 4-cycle L1d load shadow while flushing `used` on `c == 0` slows `records-l9` (`+0.86%`).
- **KILL `c54_u8pair32` (`0.9980x` / `1.0046x` / `1.0545x`):** Moving `c` from bits `30..31` (`e >> 30`) to bits `14..15` (`(e >> 14) & 3`) adds an `& 3` mask on the `c == 0` branch test (`+5.45%` on `records-l9`) while symbol extraction masks already hide in the L1d load shadow.
- **KILL `c54_refill_fast` (`1.0024x–1.0143x`):** Reconfirmed Cycle 44's finding that `pos & 255` adds a 1-cycle address dependency versus a `100%`-predicted `win.get(pos..pos+8)` bounds check.

**Conclusion:** Stop single-bitstream `decode_group_win` micro-ALU tuning (`5.8` cycles/step is at the x86-64 `1 + 4 + 1 = 6`-cycle L1d + variable-shift serial latency floor). Candidate codebase remains clean at Handoff 3 (`work_cand_c53`). Next cycle (`Cycle 55`) attacks multi-thread cold-start pipeline starvation in `BzReader::parscan` (`src/read.rs`).
