# Wormhole Cycle 58 (Neon) — Charge 4: Written Specification of the PR 4 `BlockSpan` Streaming Interface (Mechanisms ↔ `StreamDecoder`)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`, [`src/parallel.rs`](../../src/parallel.rs))  
**Coordination Target:** Mechanisms (`origin/research/carbon-word-transport`, `300a29976`, `C38a`/`C38b` Handoff 1)  
**Compile-Checked Reference Implementation:** [`c58_pr4_spec_ref.rs`](2026-11-22-wormhole-cycle58/c58_pr4_spec_ref.rs) (`#![forbid(unsafe_code)]`), [`c58_results.txt`](2026-11-22-wormhole-cycle58/c58_results.txt), [`c58_pr4_spec.toml`](2026-11-22-wormhole-cycle58/c58_pr4_spec.toml)

---

## 1. Design Goals & Hard Invariants

This specification defines the exact internal contract between **Mechanisms' parallel front end** (bounded-window magic anchor finder + fast block reader) and **Wormhole's `#739` `StreamDecoder`** so that Mechanisms' Handoff 1 drops into [`src/parallel.rs`](../../src/parallel.rs) with **zero rework** while preventing both of `#741`'s i9 rejections:

1. **Bounded Input Memory (`<= 2 × WIN_CAP = 8 MiB` even on `100 GB` streams):** Zero whole-input buffering; fixes `#741`'s `1.28×` RSS regression (`0.77×–0.95×` of `#739` RSS in C57).
2. **Zero Inter-Window Epoch Barrier:** Ping-pong `MAX_LIVE_WINS = 2` `Arc<WindowBuf>` windows overlap I/O + anchor-scan of window $k+1$ with BWT decoding of window $k$ (`-45%` `vcsw`, `-8 ms` vs single-window barrier in C57).
3. **Zero Regression at `<= 8T` Unpinned (fixes `#741`'s `1.11×` `6T` loss):** `StreamDecoder::new` activates PR 4 **only** when serial scanning actually binds (`threads >= physical_cores` or pinned `threads >= 8`, or explicit opt-in constructor for i9 grid benchmarking); below that crossover, `StreamDecoder` keeps its dedicated serial scanner thread (`W + 1` active threads).
4. **100% Reuse of `#739` Machinery:** Preserves `Ready` ring, `READY_BYTES = 12 MiB` cap (`W + 2` floor), `POOLED_FROM = 64 KiB`, `decode_owned` (`0 B` `scratch.ll`), `unrle_in_place` + `64 KiB` L2-tiled CRC, `StreamDecoder::read_block()`, `StreamDecoder::spare()`, `BzReader::fill_buf()`, `cancel()`, and exact `Error { kind, block, bit }` parity.

---

## 2. Exact Rust Interface Specification (Mechanisms ↔ `StreamDecoder`)

### 2.1 Mechanisms Front-End Surface (`src/scan.rs` & `src/block.rs`)

Mechanisms owns **only** these two stateless, `#![forbid(unsafe_code)]` primitives (no threads, no channels, no `BTreeMap`, no output buffer management):

```rust
/// Speculative block-start candidate inside one compressed `WindowBuf`.
/// Directly matches Mechanisms C38b's `MagicCand { bit, guessed_level }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanCand {
    /// Bit offset relative to `window.data[0]`.
    pub rel_bit: u64,
    /// Guessed bzip2 level (`1..=9`) from the most recent `BZh[1-9]` header
    /// seen at or before `rel_bit` (or `init_level` carried from the prior window).
    pub guessed_level: u8,
}

/// Stateless bounded-window magic finder implemented by Mechanisms (`C38b` disjoint 4-byte anchor scan).
///
/// Contract:
/// - Must scan `win[rel_start_bit / 8 ..]` and push all candidate `BLOCK_MAGIC` (`0x314159265359`)
///   positions in strictly increasing `rel_bit` order into `out` (after `out.clear()`).
/// - False positives (magic bytes occurring inside compressed payload) are 100% harmless:
///   `StreamDecoder`'s `ChainValidator` drops them in O(1) and recycles their output buffers.
/// - Missed candidates or wrong `guessed_level` (e.g. a false `BZh` inside payload) are 100% safe:
///   `ChainValidator` falls back to exact synchronous decode at the true `abs_bit` and `level`.
pub trait SpanFinder: Send + Sync {
    fn find_cands(&self, win: &[u8], rel_start_bit: u64, init_level: u8, out: &mut Vec<SpanCand>);
}

/// Mechanisms fast block reader (`#742` / `C38a`) called by each `StreamDecoder` worker
/// with its own thread-local `syms: Vec<u16>` and `scan_scratch`:
///
///   read_block_from_slice(
///       win: &[u8],
///       rel_bit: u64,
///       level: u8,
///       syms: Vec<u16>,
///       scratch: &mut BlockReadScratch,
///   ) -> Result<(block::Symbols, u64 /* end_rel_bit */), ErrorKind>
```

### 2.2 `StreamDecoder` Bounded Ping-Pong Window & Speculative Ring (`src/parallel.rs`)

`StreamDecoder` owns all threading, memory bounds, backpressure, buffer recycling, and stream validation:

```rust
pub const DEFAULT_WIN_CAP: usize = 4 << 20; // 4 MiB per window (tunable; swept in C62)
pub const MAX_LIVE_WINS: usize = 2;         // <= 8 MiB total live compressed input RAM

/// One immutable shared compressed input window.
/// Automatically recycled into `free_win` via `Arc::try_unwrap` when the last worker
/// AND `ChainValidator` drop their references (zero per-window allocation after warmup).
pub struct WindowBuf {
    pub abs_base_byte: u64, // stream byte offset of `data[0]`
    pub data: Vec<u8>,      // `len <= win_cap` (or single oversized block on pathological input)
    pub eof: bool,          // true iff `data` reaches EOF of the underlying input
}

/// One speculative work item claimed by workers (`0` scanner Huffman CPU, `1` TLS `syms` per worker).
struct SpanTask {
    win: Arc<WindowBuf>,
    cand_seq: usize,        // monotonic ring slot index (power-of-2 mask)
    abs_start_bit: u64,     // `win.abs_base_byte * 8 + rel_bit`
    rel_bit: u64,
    level: u8,
}

/// Stored in `#739`'s power-of-2 `Ready` ring (subject to `READY_BYTES = 12 MiB` + `W + 2` floor).
enum SpecOutcome {
    Block { abs_end_bit: u64, crc: u32, out: Vec<u8> },
    NeedMoreOrTrunc,        // straddling block past `win.data.len()`, or real EOF mid-block
    Err(ErrorKind),         // definitive malformed block at `abs_start_bit`
}

struct SpecEntry {
    abs_start_bit: u64,
    level: u8,
    outcome: SpecOutcome,
}
```

### 2.3 Ownership, Seam Carry-Over, & Early Window Reclamation Lifecycle

1. **Producer (`feed_windows`, feeds from `StreamDecoder::write` / `ChannelSource` or `Read`):**
   - Acquires a recycled `Vec<u8>` from `free_win` (blocks only if `MAX_LIVE_WINS = 2` buffers are currently pinned).
   - Prepends `tail_carry` (bytes starting at the **last** candidate byte `cut_byte` of window $k-1$, typically `~30–60 KiB`), then fills up to `win_cap` (`4 MiB`).
   - Calls `finder.find_cands(&buf, 0, last_level, &mut cands)` and retains `abs_base * 8 + c.rel_bit >= min_next_abs_bit` (deduplicating any candidate already published in window $k-1$).
   - If `!eof && cands.len() >= 2`: copies `buf[cut_byte..]` into `tail_carry` and publishes `cands[..cands.len() - 1]` — guaranteeing every published candidate's seam block is decoded **100% in parallel** in window $k+1$ (`0` seam fallback!).
   - Pushes `Arc::clone(&win_arc)` to `win_log` (for `ChainValidator`'s header/`END_MAGIC`/fallback checks) and enqueues `SpanTask`s without waiting for window $k-1$ to finish!
2. **Worker (`worker_parscan`, `W` threads):**
   - Holds `1` thread-local `syms: Vec<u16>` + `BlockReadScratch` + `Scratch<I>` (**zero `free_syms` mutex traffic**!).
   - Enforces `#739`'s backpressure (`cand_seq < taken_seq + slots` and `ready_bytes < READY_BYTES || cand_seq < taken_seq + floor`).
   - Calls Mechanisms' `read_block_from_slice(&win.data, rel_bit, level, syms, &mut bsc)` and **immediately calls `release_win(win)` BEFORE BWT inversion** — so window $k$ is reclaimed and refilled while window $k$'s last blocks are still finishing BWT!
   - Runs `#739`'s `decode_owned` (`0 B` `ll`, `POOLED_FROM = 64 KiB`, `unrle_in_place` + `64 KiB` L2-tiled CRC) and inserts `SpecEntry` into `#739`'s `Ready` ring.
3. **Reader (`ChainValidator` inside `StreamDecoder::advance` / `read_block`):**
   - Maintains exact `(first, in_stream, level, combined_crc, abs_bit, block_idx)`.
   - Validates `BZh[1-9]` stream headers and `END_MAGIC` + `StreamCrc` directly on `cur_win` (`win_log`), dropping `cur_win` via `release_win` as soon as `abs_bit` crosses into the next window.
   - At each `BLOCK_MAGIC`, pops speculative entries in `cand_seq` order:
     - `entry.abs_start_bit < abs_bit`: **false positive** → drop & recycle `out` to `free_out`.
     - `entry.abs_start_bit == abs_bit && entry.level == level`: **fast hit (`>99.5%`)** → update `combined_crc`, advance `abs_bit = abs_end_bit`, return `out` zero-copy!
     - `entry.abs_start_bit > abs_bit` or wrong `level` or `NeedMoreOrTrunc`: push `entry` back into `pending_spec` and decode the single block synchronously (advancing `cur_win` on `Truncated && !eof`), yielding **100% bit-exact parity** on all valid, multi-stream, truncated, and corrupt inputs!
4. **Unified Zero-Copy `decode_parallel(&[u8])`:**
   - When the input is already a `&[u8]` slice (`decode_parallel`), `WindowBuf` is skipped in favor of scoped `&[u8]` sub-slices (Cycle 66), reusing the exact same `SpanFinder` + `worker_parscan` + `ChainValidator` code path!

---

## 3. Verification Results (`c58_pr4_spec_ref.rs`, `#![forbid(unsafe_code)]`)

| Test Case | Window Cap | Output Parity | `windows` | `fast_hits` | `fp_dropped` | `straddle_fb` | Result |
|---|---:|---|---:|---:|---:|---:|---|
| `wikidata_431b_210m.bz2` (`431` blks) | `4 MiB` | `209,833,456 B` exact | `4` | **`430 / 431` (`99.8%`)** | `1` | `1` | **PASS** |
| `wikidata_431b_210m.bz2` (adversarial seams!) | `256 KiB` | `209,833,456 B` exact | **`65`** | **`430 / 431` (`99.8%`)** | `1` | `1` (`0` seam fbs!) | **PASS** |
| `wikidata_l1_36m.bz2` (`379` blks) | `512 KiB` | `37,799,566 B` exact | `8` | **`377 / 379` (`99.5%`)** | `2` | `2` | **PASS** |
| Empty stream (`0 B`) | `64 KiB` | `ErrorKind::Empty` | — | — | — | — | **PASS** |
| Bad header (`"NOTBZ2DATA"`) | `64 KiB` | `ErrorKind::NotBz2` | — | — | — | — | **PASS** |
| Truncated mid-block (`200 KB`) | `64 KiB` | `2,691,144 B` exact prefix + `Truncated` | — | — | — | — | **PASS** |
| Mid-block bit-flip corruption | `64 KiB` | `BlockCrc { stored, computed }` | — | — | — | — | **PASS** |

---

## 4. Cycle 58 Verdict & Next Cycle Plan

- **Charge 4 Written Spec COMPLETE & VERIFIED:** Mechanisms' Handoff 1 (`C38b` anchor finder + `C38a` block reader) drops directly behind `SpanFinder` + `read_block_from_slice` with zero rework to `StreamDecoder`.
- **Cycle 59 (next — Charge 4, Part 3):** Plug a fast `#![forbid(unsafe_code)]` disjoint 4-byte anchor `SpanFinder` (matching Mechanisms' `C38b` `~0.2 ms/4 MiB` finder cost!) into `decode_pr4_stream` and run a full **`12`-round `4W` / `6W` / `10W` / `16W` A/B across all 4 regimes** (`wikidata-l9`, `wikidata-l1`, `hetero-l9`, `random-l9`) against `3296195e3`!
