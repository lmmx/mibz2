# Explore: `abi3-py310` Python Delivery Dissection (Zero `unsafe`, Zero Packaging Change)

- **Date:** 2026-11-28
- **Claim:** `explore`
- **Baseline:** `d20a71bb` (post-`#1033`, `#1176`, `#1117`, `#1294`; `pyo3 = 0.29.3`, `abi3-py310`, zero `unsafe` in `src/python.rs`)
- **Environment & scope:** Cloud Linux VM (Ubuntu 24.04, Python 3.12, glibc, `CARGO_PROFILE_RELEASE_LTO=off CARGO_PROFILE_RELEASE_CODEGEN_UNITS=4` matching incremental local builds). All performance timings are **VM CPU/wall exploratory measurements** (`n = 7` order-balanced interleaved pairs in fresh subprocesses for exact `VmHWM` peak RSS); hardware-sensitive wall decisions remain subject to CI / target verification. All prototypes verified **100% SHA-256 bit-exact** against stdlib `bz2` and shipped `mibz2`, with `28/28` existing `pytest` tests plus new edge-case tests passing.

---

## 1. Public API Audit & Baseline Profile (`mibz2` vs stdlib `bz2`)

`python/mibz2/__init__.py` exports `["BZ2Reader", "DecodeError", "decompress", "open"]`. There is no separate `BZ2File` or `BZ2Decompressor` class; `mibz2.open(source, threads, buffer_size=1 << 20)` returns `io.BufferedReader(BZ2Reader(source, threads), buffer_size)`.

### Stage 1 baseline on `text.bz2` (`28,706,026 B`, `793,778` lines, mean `36.2 B/line`) and `wikidata.bz2` (`209,700,196 B`, `15,620` lines, `431` blocks incl. `198` ~`45 B` runts)

| Workload | Corpus | Stdlib `bz2` CPU / HWM | Shipped `t=1` CPU / HWM / GIL gap | Shipped `t=2` CPU / HWM / GIL gap |
|---|---:|---:|---:|---:|
| `decompress` | `text` | `1,784.2 ms` / `75.6 MiB` | `364.9 ms` / `78.9 MiB` / `153.7 ms` | `368.2 ms` / `82.4 MiB` / `201.8 ms` |
| `open_read_all` | `text` | `1,859.7 ms` / `72.6 MiB` | `364.1 ms` / `88.8 MiB` / `22.2 ms` | `394.4 ms` / `97.0 MiB` / `27.7 ms` |
| `open_read_64k` | `text` | `1,840.4 ms` / `17.1 MiB` | `319.9 ms` / `37.1 MiB` / `23.4 ms` | `319.6 ms` / `54.6 MiB` / `34.1 ms` |
| `open_read_1m` | `text` | — | `320.4 ms` / `38.8 MiB` / `20.7 ms` | `320.8 ms` / `56.8 MiB` / `29.7 ms` |
| `open_read1_64k` | `text` | — | `301.4 ms` / `35.3 MiB` / `19.0 ms` | `311.8 ms` / `50.8 MiB` / `8.1 ms` |
| `open_iter_lines` | `text` | `3,230.0 ms` / `16.9 MiB` | **`645.0 ms`** (`+114%`) / `36.9 MiB` | **`635.7 ms`** (`+104%`) / `61.0 MiB` |
| `open_readline` | `text` | `3,139.2 ms` / `17.0 MiB` | **`665.1 ms`** (`+121%`) / `36.8 MiB` | **`619.5 ms`** (`+99%`) / `58.5 MiB` |
| `open_readlines` | `text` | — | `502.9 ms` / `95.6 MiB` | `520.9 ms` / `109.2 MiB` |
| `raw_iter_lines` (`1 MiB` slice) | `text_1m` | — | **`976.1 ms`** (`24.3x` vs `open_iter_lines` `41.0 ms`!) | — |
| `decompress` | `wikidata` | `13,848.9 ms` / `453.2 MiB` | `1,569.6 ms` / `436.6 MiB` / `1,801 ms` | `1,695.3 ms` / `450.4 MiB` / `2,543 ms` |
| `open_read_all` | `wikidata` | `13,026.0 ms` / `417.8 MiB` | `1,643.7 ms` / `437.7 MiB` / `54.1 ms` | `1,544.9 ms` / `444.0 MiB` / `78.3 ms` |
| `open_read_64k` | `wikidata` | `14,074.5 ms` / `20.5 MiB` | `1,279.2 ms` / `44.2 MiB` / `45.8 ms` | `1,211.3 ms` / `65.5 MiB` / `31.0 ms` |
| `open_read_1m` | `wikidata` | — | `1,353.2 ms` / `46.1 MiB` / `39.9 ms` | `1,269.2 ms` / `68.4 MiB` / `18.3 ms` |
| `open_read1_64k` | `wikidata` | — | `1,231.7 ms` / `39.4 MiB` / `29.1 ms` | `1,234.7 ms` / `61.9 MiB` / `8.1 ms` |
| `open_iter_lines` | `wikidata` | `14,808.1 ms` / `19.1 MiB` | `1,369.3 ms` / `41.8 MiB` / `37.1 ms` | `1,330.6 ms` / `65.3 MiB` / `28.6 ms` |
| `open_readline` | `wikidata` | `14,789.6 ms` / `19.1 MiB` | `1,286.9 ms` / `41.8 MiB` / `25.1 ms` | `1,350.2 ms` / `63.4 MiB` / `16.5 ms` |
| `open_readlines` | `wikidata` | — | `1,260.2 ms` / `234.6 MiB` | `1,227.6 ms` / `257.3 MiB` |

CPython line-iteration floor on already-decoded `text.bz2` (`28.7 MB`, `793,778` lines):
- `for line in io.BytesIO(raw_text)`: **`110.4 ms`** CPU
- `for line in io.BufferedReader(io.BytesIO(raw_text), 1 << 20)`: **`227.7 ms`** CPU (`2.06x` slower than `BytesIO`)
- `raw_text.splitlines(True)`: **`227.0 ms`** CPU

---

## 2. Four Structural Bottlenecks Diagnosed

1. **Catastrophic `BZ2Reader` line-iteration cliff (`~25x` slowdown):** `BZ2Reader` subclasses `io.RawIOBase` without overriding `readline()` / `readlines()` / `__iter__`, so iterating a `BZ2Reader` inherits `io.IOBase.readline()` which calls `self.read(1)` once per byte (`976 ms` CPU / `5,008 ms` wall on a `1 MiB` slice of `text.bz2`).
2. **Redundant 3-copy `io.BufferedReader` layer in `mibz2.open()`:** `BzReader` is already block-buffered (`fill_buf()` lends the decoded block up to ~`900 KiB`). Wrapping `BZ2Reader` in stock `io.BufferedReader(1 MiB)` adds a redundant `1 MiB` C buffer and forces **3 copies** per streaming read or line (`BzReader` -> temp `PyBytes` -> `memoryview(b).cast("B")[:n] = data` into `BufferedReader` -> returned `PyBytes`), plus `io.BufferedReader`'s `2x` slower per-line framing (`227.7 ms` vs `110.4 ms` on `text.bz2`).
3. **Why Python wrapper methods hurt `while f.readline():` (`+66%` in V1):** Delegating `readline()` through a Python method on a `BufferedReader` subclass executes `793,778` Python frame entries on `text.bz2` (`659.9 -> 1,098.2 ms` CPU). Binding the Rust C-method directly on the instance (`self.readline = reader.readline`, `self.readlines = reader.readlines`, `self.peek = reader.peek`, rebound on `detach()`) eliminates the Python wrapper hop completely (`659.9 -> 644.1 ms` / `-1.3..-4.0%` CPU) while keeping `isinstance(f, io.BufferedReader) == True`, `f.raw` as `BZ2Reader`, `f.raw.verified`, and `io.TextIOWrapper(f)` intact.
4. **Why zero-`unsafe` `abi3-py310` cannot break the `2x` peak-RSS floor on `readall()` / `decompress()` (and where the exact boundary lies):**
   - **Primary-source PyO3 `0.29.3` audit:**
     - `PyBytes::new_with` (`/opt/cargo/registry/src/.../pyo3-0.29.3/src/types/bytes.rs:117-126`) calls `ffi::PyBytes_FromStringAndSize(NULL, len)` + `ffi::PyBytes_AsString(pyptr)` (both 100% in `abi3-py310`), **then unconditionally calls `core::ptr::write_bytes(buffer, 0u8, len)`** before invoking the safe closure. That faults and zeroes all `209.7 MiB` while source chunks are still live (`2x` RSS = `437 MiB`) and double-writes the entire buffer under the GIL (`+3..+7%` CPU, `1.1-2.2 s` GIL stall).
     - `PyBytes::new_with_writer` / `PyBytesWriter` (`pyo3-0.29.3/src/byteswriter.rs:60,117`) falls back under `Py_LIMITED_API` to `Vec::with_capacity` + `PyBytes::new(py, &value.buffer)` (also `2x` RSS).
   - **glibc `M_MMAP_THRESHOLD` finding (relevant to `python-zero-extra-copy`):**
     - A direct `libc` `malloc`/`memmove`/`free` probe on `210 MiB` after freeing a single `1 MiB` buffer (reproducing `BzReader` startup) measured:
       - `chunk = 1 MiB`: `217.9 -> 427.6 MiB` peak RSS (`brk` heap; glibc raised `M_MMAP_THRESHOLD` to `1 MiB`).
       - `chunk = 16 MiB`: `217.9 -> 233.9 MiB` peak RSS (**`-45.3%`**).
       - `chunk = 33 MiB` (`> DEFAULT_MMAP_THRESHOLD_MAX = 32 MiB`): `217.9 -> 250.9 MiB` peak RSS (**`-41.3%`**, unconditionally `mmap`/`munmap` on every call in the same process).
     - Therefore, **zero-`unsafe` `abi3-py310`** should keep shipped `b"".join(chunks)` for `readall()` and `crate::decode`/`decode_parallel` + `py.detach(|| drop(out))` for `decompress()`. Breaking `2x` RSS on whole-stream reads requires either (a) dropping `abi3-py310` for safe `PyBytesWriter` (`python-zero-extra-copy`, `needs_louis`, which also saves the final copy), or (b) ~12 lines of `unsafe` Limited-API FFI (`PyBytes_FromStringAndSize(NULL, total)` + `PyBytes_AsString` without `write_bytes(0)`) combined with `> 32 MiB` `mmap`-backed chunks (no packaging/wheel-matrix change, still copies once).

---

## 3. Clean V3 Prototype Design (`abi3-py310`, Zero `unsafe`, Zero Packaging Change)

1. **`src/python.rs` (zero `unsafe`, zero `Cargo.toml` changes):**
   - Track `avail_rem: usize` inside `ReaderState` so any call while buffered bytes remain in `BzReader::fill_buf()` skips `py.detach()` (zero GIL release/reacquire per line; only detaches at block boundaries, `32` times on `text.bz2`).
   - Safe 8-byte SWAR `find_newline(&[u8]) -> Option<usize>` (zero `unsafe`).
   - `Reader::readline(size=None)`, block-batched `Reader::readlines(hint=None)` (single `consume()` per decoded block), `Reader::__iter__` / `Reader::__next__` (C-level `tp_iternext`), `Reader::read_full(size)` (single-copy fast path when `avail_rem >= size`, single GIL-detached gather across blocks otherwise + `py.detach(|| drop(out))`), and `Reader::peek(size=0)`.
   - `PyFile::read` uses `pyo3::intern!(py, "read")`; `decompress` detaches the GIL for `drop(out)`.
2. **`python/mibz2/__init__.py`:**
   - `BZ2Reader` binds `self.readline` / `self.readlines` directly to `self._reader` and delegates `__iter__` / `__next__` directly to `self._reader` (retaining shipped `1 MiB` chunk + `b"".join` `readall()`).
   - `mibz2.open()` returns `_BZ2BufferedReader(io.BufferedReader)` initialized with `super().__init__(raw, 1)` (preserving `isinstance(f, io.BufferedReader)`, `f.raw`, `f.raw.verified`, `detach()`, `io.TextIOWrapper(f)`), binding `readline`/`readlines`/`peek` directly to `raw._reader`, returning `raw._reader` from `__iter__`, delegating `read(size)` to `read_full(size)` / `_readall_from`, `read1(size)` to `read(size)`, and direct 2-copy `readinto` / `readinto1`.

---

## 4. Powered Interleaved Paired A/B Results (`n = 7`, VM Scope, 100% SHA-256 Parity)

Order-balanced alternating pairs (`base, V3, V3, base, ...`) in fresh subprocesses (`VmHWM` peak RSS). Intervals on CPU ratio `r = V3 / base` are the 2nd-order paired interval `[sorted[1], sorted[-2]]` (`n = 7`, ~87.5% sign-test CI; `n = 5` on `text_1m`).

### `text_1m.bz2` (`1 MiB`, `26,301` lines, `n = 5`)

| Workload | `t` | Base CPU | V3 CPU | Paired CPU ratio `[lo, hi]` | CPU delta | Wall delta | Base -> V3 HWM |
|---|---:|---:|---:|---:|---:|---:|---:|
| `raw_iter_lines` | 1 | `887.9 ms` | `35.1 ms` | `0.0394 [0.0390, 0.0396]` | **`-96.1%` (`25.4x`)** | **`-94.3%`** | `25.7 -> 25.8 MiB` (`+0.2%`) |

### `text.bz2` (`28,706,026 B`, `793,778` lines, `n = 7`)

| Workload | `t` | Base CPU | V3 CPU | Paired CPU ratio `[lo, hi]` | CPU delta | Wall delta | Base -> V3 HWM |
|---|---:|---:|---:|---:|---:|---:|---:|
| `open_readlines` | 1 | `550.7 ms` | `398.3 ms` | `0.7233 [0.6996, 0.7293]` | **`-27.7%`** | **`-34.8%`** | `97.9 -> 95.5 MiB` (`-2.4%`) |
| `open_iter_lines` | 1 | `756.8 ms` | `652.5 ms` | `0.8648 [0.8438, 0.8683]` | **`-13.5%`** | `-4.8%` | `35.0 -> 32.6 MiB` (`-6.7%`) |
| `open_readline` | 1 | `737.4 ms` | `713.1 ms` | `0.9603 [0.9437, 0.9914]` | `-4.0%` | `-2.4%` | `34.9 -> 32.8 MiB` (`-6.2%`) |
| `open_read_64k` | 1 | `266.2 ms` | `247.2 ms` | `0.9519 [0.9025, 0.9556]` | `-4.8%` | `-6.6%` | `35.1 -> 33.0 MiB` (`-5.7%`) |
| `open_read_1m` | 1 | `267.3 ms` | `257.0 ms` | `0.9616 [0.9540, 0.9670]` | `-3.8%` | `-8.0%` | `36.7 -> 36.4 MiB` (`-0.9%`) |
| `open_read_all` | 1 | `298.2 ms` | `296.3 ms` | `0.9917 [0.9850, 0.9968]` | `-0.8%` | `+0.4%` | `86.7 -> 86.8 MiB` (`+0.1%`) |
| `decompress` | 1 | `282.4 ms` | `284.7 ms` | `1.0106 [0.9966, 1.0165]` | `+1.1%` | `-5.5%` | `76.8 -> 76.8 MiB` (`+0.0%`) |
| `open_readlines` | 2 | `580.9 ms` | `408.2 ms` | `0.7070 [0.6984, 0.7264]` | **`-29.3%`** | **`-41.4%`** | `109.6 -> 104.5 MiB` (`-4.6%`) |
| `open_iter_lines` | 2 | `787.6 ms` | `663.1 ms` | `0.8475 [0.8257, 0.8551]` | **`-15.3%`** | **`-26.6%`** | `57.4 -> 54.3 MiB` (`-5.4%`) |
| `open_readline` | 2 | `744.7 ms` | `734.5 ms` | `0.9869 [0.9742, 0.9947]` | `-1.3%` | `-1.7%` | `57.5 -> 56.6 MiB` (`-1.5%`) |
| `open_read_64k` | 2 | `277.6 ms` | `264.2 ms` | `0.9518 [0.9422, 0.9634]` | `-4.8%` | `-3.0%` | `52.2 -> 48.9 MiB` (`-6.3%`) |
| `open_read_1m` | 2 | `286.3 ms` | `273.6 ms` | `0.9587 [0.9480, 0.9923]` | `-4.1%` | `-2.7%` | `52.9 -> 51.4 MiB` (`-2.9%`) |
| `open_read_all` | 2 | `318.3 ms` | `318.2 ms` | `1.0091 [0.9889, 1.0201]` | `+0.9%` | `+2.9%` | `93.6 -> 94.1 MiB` (`+0.6%`) |
| `decompress` | 2 | `315.3 ms` | `316.3 ms` | `1.0000 [0.9978, 1.0054]` | `0.0%` | `-4.0%` | `80.5 -> 80.4 MiB` (`-0.1%`) |

### `wikidata.bz2` (`209,700,196 B`, `15,620` lines, `n = 7`)

| Workload | `t` | Base CPU | V3 CPU | Paired CPU ratio `[lo, hi]` | CPU delta | Wall delta | Base -> V3 HWM |
|---|---:|---:|---:|---:|---:|---:|---:|
| `open_readlines` | 1 | `1,266.4 ms` | `1,198.8 ms` | `0.9519 [0.9222, 0.9683]` | `-4.8%` | **`-13.7%`** | `234.9 -> 234.8 MiB` (`0.0%`) |
| `open_iter_lines` | 1 | `1,144.1 ms` | `1,049.7 ms` | `0.9353 [0.8756, 0.9639]` | `-6.5%` | **`-11.5%`** | `37.6 -> 35.9 MiB` (`-4.3%`) |
| `open_read_64k` | 1 | `1,155.2 ms` | `1,060.7 ms` | `0.9028 [0.8992, 0.9228]` | **`-9.7%`** | **`-11.3%`** | `39.9 -> 34.8 MiB` (`-12.8%`) |
| `open_read_1m` | 1 | `1,119.8 ms` | `1,054.2 ms` | `0.9469 [0.9319, 0.9675]` | `-5.3%` | `-6.5%` | `41.9 -> 38.3 MiB` (`-8.5%`) |
| `open_read_all` | 1 | `1,394.7 ms` | `1,398.9 ms` | `1.0055 [0.9948, 1.0359]` | `+0.5%` | `+1.5%` | `433.6 -> 433.6 MiB` (`0.0%`) |
| `open_readlines` | 2 | `1,255.5 ms` | `1,203.5 ms` | `0.9578 [0.9338, 0.9815]` | `-4.2%` | `-0.8%` | `258.6 -> 257.0 MiB` (`-0.6%`) |
| `open_iter_lines` | 2 | `1,094.8 ms` | `1,040.4 ms` | `0.9501 [0.9398, 0.9645]` | `-5.0%` | `-3.0%` | `63.5 -> 59.9 MiB` (`-5.7%`) |
| `open_read_64k` | 2 | `1,098.1 ms` | `1,016.7 ms` | `0.9258 [0.9188, 0.9314]` | `-7.4%` | `-2.8%` | `61.6 -> 61.0 MiB` (`-1.0%`) |
| `open_read_1m` | 2 | `1,117.6 ms` | `1,077.7 ms` | `0.9655 [0.9611, 0.9819]` | `-3.5%` | `+0.1%` | `65.5 -> 64.7 MiB` (`-1.2%`) |
| `open_read_all` | 2 | `1,384.3 ms` | `1,384.2 ms` | `1.0019 [0.9751, 1.0122]` | `+0.2%` | `+3.1%` | `441.7 -> 437.8 MiB` (`-0.9%`) |

---

## 5. Disposition

- **Filed `frontier/claims/python-abi3-delivery-build.toml` (`idea`, P2, zero `needs_louis`, zero `unsafe`, `abi3-py310` intact):** delivers `25.4x` faster `BZ2Reader` line iteration, `-27.7..-29.3%` CPU (`-34.8..-41.4%` wall) on `open().readlines()`, `-13.5..-15.3%` CPU on `for line in open()`, `-7.4..-9.7%` CPU (`-12.8%` HWM at `t = 1`) on `wikidata` `open().read(64 KiB)`, and `0/25` regressions across both corpora and thread counts.
- **Preserved `python-zero-extra-copy` (`needs_louis`) intact:** documented the exact `pyo3-0.29.3` `PyBytes::new_with` zero-fill line (`bytes.rs:123`) and `glibc > 32 MiB` `mmap` chunking result (`-41..-45%` RSS) so future whole-stream RSS work has an exact map of both the zero-`unsafe` and Limited-API FFI boundaries.
