# Wormhole Cycle 61 (Neon): PR 4 Adversarial Stress & Edge-Case Hardening Suite (Invariants #3–#5)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`)  
**Suite:** [`c61_pr4_adversarial_suite.rs`](2026-11-22-wormhole-cycle61/c61_pr4_adversarial_suite.rs) (`#![forbid(unsafe_code)]`, 9 adversarial categories verified byte-exact & error-kind-exact against `mibz2::BzReader`)  
**Output:** [`c61_results.txt`](2026-11-22-wormhole-cycle61/c61_results.txt)  
**Manifest:** [`c61_pr4_adversarial.toml`](2026-11-22-wormhole-cycle61/c61_pr4_adversarial.toml) (`check_manifest.py`: `PASS`)

---

## 1. Objective

Before Mechanisms and the Coordinator port PR 4 into `src/stream.rs`, Cycle 61 tries to **break** our Cycle 58/59 PR 4 `BlockSpan` streaming engine with 9 hostile test categories — multi-stream level switches, empty streams, oversized blocks `> win_cap`, false-positive storms (`> 12× slots` bogus candidates/window), mid-window early `cancel()`, `1..13 B` trickle+`EINTR` readers, mid-stream `Io` errors, all bzip2 error classes, and an exhaustive byte-by-byte truncation sweep (`0..=24` bytes).

---

## 2. Three Real Production Spec Invariants Caught & Fixed

The adversarial suite caught **three** concrete edge-case bugs that would have broken production `StreamDecoder` if ported directly from Cycle 59:

1. **Mandatory PR 4 Invariant #3 — Livelock-Free Oversized-Block Guard + Window-Spanning Fallback:**
   - **Hazard:** If a single compressed block exceeds `win_cap` (e.g., a `904 KiB` incompressible `BZh9` block under `win_cap = 256 KiB` or `512 KiB`, or a corrupt/pathological `> 4 MiB` gap between magics), carrying from `cands[0].rel_bit / 8` makes `tail_carry.len() -> win_cap`, advancing `abs_base` by `0` bytes (**infinite livelock**).
   - **Fix:** Cap `tail_carry` at `max_carry = (win_cap / 2).max(16)` on every non-EOF window so `advance = buf.len() - tail_carry.len() >= win_cap / 2 > 0` strictly (provably livelock-free). When the `ChainValidator` fallback encounters `Truncated` on a non-EOF window, it accumulates the non-overlapping bytes into `fb_buf`, releases the finished `cur_win` immediately via `release_win` (keeping `peak_lw <= MAX_LIVE_WINS = 2` with `0` deadlock), and pops the next window from `win_log`. Verified on `904 KiB` blocks inside `256 KiB` windows (`3.5×` larger than `win_cap`, `42` windows, `peak_lw = 2`, 100% byte-exact).
2. **Mandatory PR 4 Invariant #4 — `magic_over` Check Before `BadBlockMagic`:**
   - **Hazard:** On a stream truncated at `4..9` bytes (e.g., `b"BZh1"` with `0..5` trailing bytes), `Bits::at` zero-pads past end-of-slice and returns `magic != BLOCK_MAGIC` with `br.overran() == true`. Checking `over` only after `magic == EOS` caused `Trunc_4B_hdr_only` to misreport `BadBlockMagic` instead of `Truncated`.
   - **Fix:** Check `br.overran()` immediately after reading the 48-bit magic, BEFORE dispatching on `EOS` or `BLOCK_MAGIC`.
3. **Mandatory PR 4 Invariant #5 — `br.overran()` Precedes `ErrorKind` Classification (Worker & Fallback):**
   - **Hazard:** Caught by our byte-by-byte truncation sweep at `len = 10` (`"BZh1" + BLOCK_MAGIC`, 0 bytes after magic): `read_item` reads zero-padded `in_use_16 == 0` and returns `Err(NoSymbols)` before checking `overran()`. Worse, on a **valid** stream where a block happens to start `10` bytes before a non-EOF window seam, a worker returning `SpecOutcome::Err(NoSymbols)` instead of `NeedMoreOrTrunc` would **falsely abort a valid stream**!
   - **Fix:** Both worker and fallback MUST map any `Err(_)` (or `Ok(_)`!) with `br.overran() == true` to `SpecOutcome::NeedMoreOrTrunc` (worker) / `ErrorKind::Truncated` (fallback -> advance window if `!eof`, else return `Truncated`). Also indexed `b"BZ"` / `b"Zh"` in `FastAnchorFinder` so mid-window `BZh` level transitions refresh `guessed_level` immediately (`770/770` = `100%` fast hits across 5 concatenated mixed-level streams!).

---

## 3. Full 9-Category Hardening Scorecard (`c61_results.txt`)

| # | Adversarial Category | Configuration | Result | Key Metrics |
|---:|---|---|---|---|
| 1 | Multi-stream concat (`BZh1 + empty5 + BZh9 + empty1 + BZh1`) | `4 MiB` & `512 KiB`, `4W` | **PASS** | `770/770` (`100%` fast hits, `fb=0`) at `4 MiB`; `758 fh / 12 fb` at `512 KiB`, `peak_lw = 2` |
| 2 | Oversized single block `> win_cap` (`904 KiB` `random-l9`) | `256 KiB`, `512 KiB`, `1024 KiB`, `4W` | **PASS** | `12/12` exact at `256K` (`42` wins) & `512K` (`24` wins); `12/12 fh` at `1024K`; `peak_lw = 2`, `0` livelock |
| 3 | False-positive storm (`SpamFinder`, `200` bogus cands/win `> 12× slots`) | `wikidata-l1`, `512 KiB`, `4W` | **PASS** | `379/379` exact, **`1,918` FP dropped**, `0` deadlock, `0` leak |
| 4 | Mid-window early `cancel()` (`on_block -> false`) | `wikidata-l9`, `k ∈ {0, 1, 5, 100, 430}`, `4W` | **PASS** | `10.6–14.2 ms` teardown on early stop, `0` deadlock, `peak_lw = 2` |
| 5 | Hostile `TrickleReader` (`1..13 B`/read + `EINTR` every 7th call) | `200 KB` trunc `wikidata-l1`, `64 KiB`, `4W` | **PASS** | Exact `2,399,547 B` prefix + exact `Truncated` parity |
| 6 | Mid-stream `Io(BrokenPipe)` error propagation | Fail at `150 KB`, `64 KiB`, `4W` | **PASS** | Exact `ErrorKind::Io(BrokenPipe)` propagated |
| 7 | `13` bzip2 error-kind & prefix parity cases | `Empty`, `NotBz2`, `Trunc_*`, `TrailingData`, `BadBlockMagic`, `BlockCrc`, `StreamCrc`, `EmptyValid` | **PASS** | `13/13` exact `ErrorKind` + exact decoded prefix vs `BzReader` |
| 8 | Level-flip concat (`empty BZh1 + hetero BZh9`) | `4 MiB`, `4W` | **PASS** | `60/60` exact (`59 fh / 1 fb / 1 fp`) |
| 9 | Exhaustive byte-by-byte truncation sweep (`0..=14` `empty9`, `0..=24` `wiki_l1`) | `40` prefixes, `64 KiB`, `4W` | **PASS** | `40/40` exact `ErrorKind` + exact prefix vs `BzReader` |

---

## 4. Kill / Keep Verdict & Next Step

- **PROMOTED Hardened PR 4 Spec (Invariants #1–#5):** All 5 production invariants (`published_seq` EOF guard, zero-`memset` refill, livelock-free `max_carry <= win_cap/2` + window-spanning fallback, `magic_over` before `BadBlockMagic`, `br.overran()` preceding error classification) are now verified across all 9 adversarial categories with `0` deadlocks, `0` livelocks, `peak_lw <= 2`, and `100%` `BzReader` error/prefix parity.
- **Next (Cycle 62):** Full window-size × `MAX_LIVE_WINS` Pareto sweep (`{1, 2, 4, 8 MiB} × {1, 2, 3}`) across `4W/10W/16W` on `wikidata-l9`, `wikidata-l1`, and `hetero-l9` to prove `(DEFAULT_WIN_CAP, MAX_LIVE_WINS) = (4 MiB, 2)` is Pareto-optimal.
