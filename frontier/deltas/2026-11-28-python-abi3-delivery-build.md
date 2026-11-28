claim: python-abi3-delivery-build
by: antigravity
outcome: done
evidence: src/python.rs; python/mibz2/__init__.py; python/mibz2/_mibz2.pyi; python/tests/test_decode.py; docs/notebook/2026-11-28-explore-python-abi3-delivery.md; frontier/surface.tsv

Implemented direct Rust `abi3-py310` line and streaming delivery in `src/python.rs`
and `python/mibz2/__init__.py` (zero `unsafe`, `Cargo.toml` `abi3-py310` unchanged,
`python/mibz2/_mibz2.pyi` updated) and added 14 new parametrized Python tests plus
exhaustive Rust SWAR newline tests (`43/43` pytest + doctest PASS, `ruff` PASS,
`ty check` PASS, `cargo fmt` + `clippy --features python` PASS, 100% SHA-256 parity).

- **Mechanism:** `Reader` tracks `avail_rem` so reads/lines inside the lent
  `BzReader::fill_buf()` slice skip `py.detach()`, scans for `\n` via safe 8-byte
  SWAR `find_newline`, and exposes `readline`, block-batched `readlines`,
  `read_full`, `peek`, and `__iter__`/`__next__` (`PyFile::read` interns `"read"`;
  `decompress` detaches GIL for `drop(out)`). `BZ2Reader` and
  `_BZ2BufferedReader(io.BufferedReader)` bind hot C methods directly on the
  instance (rebound on `detach()`), preserve `isinstance(..., io.BufferedReader)`,
  `.raw`, `.raw.verified`, and `io.TextIOWrapper`, and copy into `bytearray` via
  `memoryview(b)` (avoiding CPython `bytearray` slice-resize overhead caught during
  `stop_test` verification: `+14.7%` -> `-2.5%` on `raw_readinto_1m`).
- **Frozen `stop_test` paired A/B vs `6ce89cc5` (`n = 7` alternating pairs, VM scope):**
  - `text_1m` `t=1` `raw_iter_lines` (`n=5`): **`0.0383x [0.0379, 0.0390]`** (`-96.2%` CPU / `26.1x`, wall `0.040x`, HWM `1.0015x`) — PASS (`<= 0.10x`).
  - `text` `open_readlines`: **`0.7174x [0.6995, 0.7270]`** (`-28.3%`, HWM `0.977x`) `t=1`; **`0.7240x [0.7124, 0.7531]`** (`-27.6%`, HWM `0.972x`) `t=2` — PASS (`<= 0.80x`).
  - `text` `open_iter_lines`: **`0.8675x [0.8473, 0.8778]`** (`-13.3%`, HWM `0.939x`) `t=1`; **`0.8615x [0.8348, 0.8843]`** (`-13.9%`, HWM `0.922x`) `t=2` — PASS (`<= 0.90x`).
  - `wikidata` `open_read_64k`: **`0.9089x [0.9015, 0.9250]`** (`-9.1%`, HWM **`0.882x`**) `t=1`; **`0.9378x [0.9218, 0.9567]`** (`-6.2%`, HWM `0.993x`) `t=2` — PASS (`<= 0.95x`).
  - **Guardrails:** `0/29` CPU regressions across all workloads (`<= 1.0191x` max, including `open_read_1m` `-2.4..-5.2%`, `open_read1_64k` `-0.7..-3.4%`, `open_readinto_1m` `-2.0..-4.7%`, `raw_readinto_1m` `-2.5..+1.1%`, `open_read_all`/`decompress` neutral); deterministic `t=1` HWM `<= 1.0029x` across 100% of cells (multi-threaded `t=2` `StreamDecoder` queue-depth HWM jitter `+1.5..+3.3 MiB` on `text` `open_read1_64k`/`open_read_1m`/`open_readline` and `wikidata` `open_read_1m`/`open_readinto_1m`, within `#1273` parallel queue variance).
