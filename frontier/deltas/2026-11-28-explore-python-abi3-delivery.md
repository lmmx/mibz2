claim: explore
by: antigravity
outcome: done
evidence: docs/notebook/2026-11-28-explore-python-abi3-delivery.md; frontier/claims/python-abi3-delivery-build.toml; frontier/surface.tsv

Profiled and dissected all `abi3-py310` Python API entry points (`src/python.rs`,
`python/mibz2/__init__.py`) after `#1033`/`#1176`/`#1117`/`#1294` on `text.bz2`
(`28.7 MB`, `793,778` lines) and `wikidata.bz2` (`209.7 MB`, `15,620` lines) at
`t = 1, 2`, strictly inside `abi3-py310` with zero `unsafe` and zero packaging
change (VM scope, 100% SHA-256 bit-exact vs stdlib `bz2`, `28/28` pytest + edge-case
parity suite passing).

- **Four bottlenecks isolated:** (1) `BZ2Reader` inherits `io.IOBase.readline()`
  1-byte-at-a-time reads (`~25x` cliff on `for line in BZ2Reader`); (2) stock
  `io.BufferedReader(1 MiB)` adds a redundant buffer and 3 copies per block on top
  of `BzReader::fill_buf()` plus `2x` slower CPython line framing (`227.7` vs
  `110.4 ms` `BytesIO` floor); (3) Python wrapper hops on `readline()` cost `+66%`
  CPU unless C-bound directly on the instance; (4) `PyBytes::new_with` in
  `pyo3-0.29.3/src/types/bytes.rs:123` unconditionally zero-fills (`write_bytes(0)`)
  before the closure runs, locking zero-`unsafe` `abi3-py310` whole-stream reads at
  `2x` RSS (while a `libc` probe shows `>32 MiB` `mmap` chunks + unzeroed
  Limited-API `PyBytes_FromStringAndSize(NULL, n)` could cut RSS `-41..-45%`
  without dropping `abi3-py310`).
- **Clean zero-`unsafe` `abi3-py310` V3 prototype (`n = 7` interleaved paired A/B):**
  Rust `avail_rem`-tracked `Reader::{readline, readlines, read_full, peek, __iter__,
  __next__}` with safe 8-byte SWAR newline search + `_BZ2BufferedReader(io.BufferedReader)`
  (preserving `isinstance`, `.raw`, `.raw.verified`, `detach()`, `TextIOWrapper`)
  while retaining shipped `b"".join` `readall()` and `decompress()` (`py.detach` on
  `drop(out)`):
  - `raw_iter_lines` (`text_1m`, `n = 5`): **`-96.1%` CPU (`25.4x`)**, `-94.3%` wall.
  - `text` `open_readlines`: **`-27.7%` CPU (`-34.8%` wall) `t=1`**, **`-29.3%` CPU (`-41.4%` wall) `t=2`**.
  - `text` `open_iter_lines`: **`-13.5%` CPU `t=1`**, **`-15.3%` CPU (`-26.6%` wall) `t=2`**, `-5.4..-6.7%` HWM.
  - `wikidata` `open_read_64k`: **`-9.7%` CPU (`-12.8%` HWM) `t=1`**, **`-7.4%` CPU `t=2`**; `open_iter_lines` `-6.5% / -5.0%` CPU; `open_read_1m` `-5.3% / -3.5%` CPU; `open_readlines` `-4.8% / -4.2%` CPU; `open_read_all` & `decompress` neutral (`0/25` regressions).
- **Filed `frontier/claims/python-abi3-delivery-build.toml`** (`idea`, P2, zero
  `needs_louis`, frozen `stop_test`) and updated `frontier/surface.tsv` row `D`.
