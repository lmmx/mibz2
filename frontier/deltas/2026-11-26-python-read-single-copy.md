claim: python-read-single-copy
by: antigravity
outcome: done
evidence: src/python.rs, python/tests/test_decode.py, frontier/claims/python-read-single-copy.toml

**What was asked:** Eliminate the redundant zero-filled `Vec<u8>` allocation and double copy in Python's `Reader::read(size)` (`src/python.rs`) by copying once directly from `BzReader`'s `BufRead::fill_buf` into `PyBytes`, with no public Rust API change and zero new `unsafe`.
**What I found:**
- `PyBytes::new(py, &avail[..n])` calls `PyBytes_FromStringAndSize(ptr, len)` directly (one copy, zero pre-zeroing; unlike `PyBytes::new_with`, which zeroes the buffer first via `ptr::write_bytes`).
- Parity: 100% byte-identical across 21 pytest tests (including doctests and 6 new edge-case tests: `read(0)`, closed `read(0)`/`read(16)`, `size = 1 << 40`, non-blocking `None` source, empty stream, corrupt-stream `read(0)`), 194 `cargo test --lib` tests, and `wikidata x4` (`838,800,784` B, SHA-256 `02949df2160357245d83e635cb155950ad0d31d2b4b8703d8e2d1be0bd3577bf` matching `mibz2::decode`). Zero new `unsafe`.
- Stop-test benchmark (`wikidata x4`, 2-vCPU Xeon VM, 1 warmup + 7 interleaved repeats alternating order, separate shipped `.so`s on `mibz2.open()`, default 1 MiB `BufferedReader`):
  - `open.read(1 MiB) 1T`: `5586.45` `[5089.32..6497.83]` -> `5197.58` `[4759.55..6421.77]` ms (**-6.96%** wall, paired -6.96%);
  - `open.read(64 KiB) 1T`: `5471.49` `[4982.15..7036.33]` -> `5302.12` `[4968.66..6050.32]` ms (**-3.10%** wall, paired -2.20%, 7/7 faster);
  - `open.read(1 MiB) 8T`: `2506.85` `[2398.80..2826.71]` -> `2412.39` `[2313.92..2613.21]` ms (**-3.77%** wall, paired -4.56%, 7/7 faster);
  - `open.read(64 KiB) 8T`: `2470.63` `[2206.98..2641.05]` -> `2351.19` `[2120.22..2581.96]` ms (**-4.83%** wall, paired -3.93%, 7/7 faster).
- Same-binary LTO-invariant paired A/B (`read_old` vs `read`, 7 interleaved repeats, wall + process CPU) confirmed the mechanism:
  - `open.read` (both chunk sizes call `Reader.read(1 MiB)`): 1T CPU `-5.48%` (1 MiB) / `-3.66%` paired (64 KiB); 8T wall `-3.98%` / `-5.34%` (CPU `-4.61%` / `-4.97%`, 7/7 faster) — removing 1,724 glibc `mmap` + soft page-fault + `munmap` TLB-shootdown cycles and redundant copies (~185-240 ms CPU saved).
  - Direct `Reader.read`: 1 MiB `-5.82%` wall / `-7.15%` CPU (1T), `-5.04%` wall / `-4.11%` CPU (8T); 64 KiB (below glibc's 128 KiB `mmap` threshold) `-2.46%` wall / `-4.06%` paired CPU (1T), `-2.58%` paired wall / `-2.08%` paired CPU (8T).
- Passes frozen `stop_test` (`>= 3.0%` wall across all 4 `mibz2.open()` cells, zero regressions > 0.5%).
**What changed:** `src/python.rs` (`Reader::read` uses `BufRead::fill_buf` + `PyBytes::new` + `consume`, preserving `size == 0`, closed, `Interrupted`, and error semantics), `python/tests/test_decode.py` (+6 edge-case tests), `frontier/claims/python-read-single-copy.toml` (`status = "review"`), and this delta.
**What's next:** Independent review and merge of this striated PR (hold stays live until merged). Follow-up opportunity: `BZ2Reader.readinto` in `python/mibz2/__init__.py` still allocates a temporary `PyBytes` via `_reader.read(len(b))` and copies it into `memoryview(b)`; adding a Rust `Reader::readinto` via `PyBuffer<u8>` would eliminate that remaining copy for `mibz2.open()`.
**Caveats:** Measurements ran on a shared 2-vCPU Xeon VM (load average 2.8-5.5), causing wall-spread overlap across repeats on 1T; process CPU time and same-binary paired deltas confirm the consistent ~4-7% win at 1 MiB and ~2-4% at direct 64 KiB.
