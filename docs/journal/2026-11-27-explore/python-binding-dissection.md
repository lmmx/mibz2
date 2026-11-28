# Cold dissection of Python binding delivery overhead (`src/python.rs` and `python/mibz2/__init__.py`)

An `explore` session (27 November 2026), dissecting every Python delivery path after `#1033`
(`814d461b`, single-copy `Reader::read` via `BufRead::fill_buf`) against native Rust `BzReader` /
`decode_parallel` ceilings and a prototype PyO3 extension compiled in an isolated temporary directory
(cleaned up after measurement). Zero edits to `src/` or `python/`, zero frozen verdicts altered (C2).

**Hardware & environment:** 2-vCPU Haswell KVM VM, Linux, CPython 3.12.16, `rustc 1.99.1`, `pyo3 0.29.3`.
Because other sessions share the 2-vCPU VM, every A/B table reports **both** paired interleaved median
wall time and exact process CPU time (`time.process_time()`, immune to host preemption), plus isolated
fresh-subprocess peak RSS (`/proc/self/status` `VmHWM`).

**Corpora:**
- `wikidata.bz2` (`11,854,933 B` -> `209,700,196 B`, 17.69×, 431 blocks, SHA-256 `3f075e30f4f8c734...`)
- `text.bz2` (`1,847,399 B` -> `12,042,918 B`, 6.52×, 925 lines, SHA-256 `8fc42eaac0062fd1...`)

**Frozen discovery / stop bar (frozen before timing):**
- **Exactness:** 100% SHA-256 bit-identical across every workload and prototype on both corpora.
- **PASS (open `frontier/claims/python-zero-extra-copy.toml` at `idea`):** `>= 10%` wall reduction OR
  `>= 25%` peak RSS reduction at `t = 4` (or `>= 5%` wall reduction at `t = 1`) on real Python
  `mibz2.open` / `BZ2Reader` / `mibz2.decompress` workloads vs shipped `src/python.rs` +
  `python/mibz2/__init__.py`, with zero new unsafe-Rust risk.
- **STOP (dead-end row):** `< 5%` wall AND `< 15%` peak RSS across all entry points.

---

## Executive summary: **DECISIVE PASS (`-14%..-40%` wall, `-14%..-21%` CPU, `-36%..-46%` peak RSS on `open().read()`, `BZ2Reader.read()`, and `decompress()`)**

Auditing `src/python.rs` (221 lines) and `python/mibz2/__init__.py` (121 lines) and instrumenting real
Python workloads cleanly separates **two huge bottlenecks** from **two small/negligible ones**, and
exposes an essential **PyO3 `abi3-py310` ABI constraint**:

1. **Monster bottleneck #1 — `BZ2Reader` inherits `io.RawIOBase.readall()` (`25,829 × 8 KiB` calls on `wikidata.bz2`!):**
   - `mibz2.open(path).read()` (`io.BufferedReader.read(-1)`) delegates directly to `BZ2Reader.readall()`,
     which is NOT overridden in `python/mibz2/__init__.py`. CPython's default `RawIOBase.readall()` loops
     `self.read(8192)` **`25,829` times** on `wikidata.bz2` (over `103,000` times on `wikidata-x4`),
     allocating a temporary `PyBytes`, wrapping `memoryview(b).cast("B")`, copying into a growing
     `bytearray` (with doubling reallocs), and finally copying `210 MiB` again into `bytes` (`3` total
     copies, `440.8–479.8 MiB` peak RSS = `2.1–2.3×` output, `+484 ms` / `+62%` CPU above native Rust
     `fill_buf` at `t = 1`!).
2. **Monster bottleneck #2 — `mibz2.decompress(data, threads)` doubles peak RSS (`436–471 MiB` vs `228–285 MiB` Rust) and copies `210 MiB` twice:**
   - `src/python.rs:202-211` allocates a full `Vec<u8>` (`crate::decode`/`decode_parallel`, with doubling
     reallocs) and then copies all `210 MiB` a second time under the GIL via `PyBytes::new(py, &out)`.
3. **100% safe-Rust rescue via `PyBytes::new_with_writer` streaming (GIL released during `fill_buf`, single copy directly into one growing `PyBytes`):**
   - On `wikidata.bz2`, single-buffer `PyBytes::new_with_writer` streaming cuts:
     - **`mibz2.open(path).read()`:** wall **`-14.0%` (`t=1`) / `-28.0%` (`t=2`) / `-13.6%` (`t=4`)**,
       CPU **`-20.5% / -15.7% / -13.9%`**, peak RSS **`-45.9% / -42.5% / -35.8%`** (`441 -> 239 MiB` at `t=1`, `469 -> 301 MiB` at `t=4`)!
     - **`BZ2Reader(path).read()`:** wall **`-36.4%` (`t=1`) / `-37.6%` (`t=2`) / `-29.8%` (`t=4`)**,
       CPU **`-20.5% / -20.7% / -15.2%`**, peak RSS **`-45.9% / -40.7% / -37.5%`** (`480 -> 300 MiB` at `t=4`)!
     - **`mibz2.decompress(raw)`:** wall **`-21.8%` (`t=1`) / `-33.1%` (`t=2`) / `-40.4%` (`t=4`)**,
       CPU **`-15.2% / -18.0% / -16.4%`** (matching native Rust `decode` `1040 ms`!), peak RSS
       **`-42.7% / -40.9% / -37.1%`** (`471 -> 296 MiB` at `t=4`, matching native Rust `decode` `285 MiB`!)!
4. **Critical PyO3 `abi3-py310` ABI constraint (and two negative findings):**
   - **ABI constraint:** `Cargo.toml` currently enables `pyo3 = { features = ["abi3-py310"] }`. Under
     `Py_LIMITED_API`, PyO3 0.29's `PyBytes::new_with_writer` silently falls back to `Vec<u8>` +
     `PyBytes::new` (`2×` RSS), and `PyBuffer<u8>` requires `abi3-py311` or non-`abi3`! Unlocking the
     full `-36%..-46%` peak RSS and `-14%..-40%` wall win requires building non-`abi3` per-version
     CPython wheels (or waiting for Python 3.15 Limited API `PyBytesWriter`). Meanwhile, a 10-line
     pure-Python `BZ2Reader.readall()` (`_reader.read(1 << 20)` + `b"".join`) works immediately under
     `abi3-py310` and recovers **`-11.4%` (`t=1`) / `-7.1%` (`t=4`) CPU** and **`-5.9%` (`t=4`) wall** on
     `mibz2.open().read()` (still `~2×` RSS).
   - **Negative #1 — `chunks_into_pybytes_drop` (`abi3-py310` `PyBytes_FromStringAndSize(NULL, total)` + dropping each `900 KiB` `Vec<u8>` 1:1):** fails to cut RSS (`436.2 -> 434.9 MiB`, `-0.3%`, `+3.2%` CPU) because Linux glibc's dynamic `M_MMAP_THRESHOLD` ratchets above `900 KiB` after the first free, keeping freed blocks on the `brk` heap!
   - **Negative #2 — `Reader::readinto` alone (post-`#1033`) and `PyFile` `readinto`:** after `#1033`,
     `open_read_1m` / `raw_readinto_1m` double-copy overhead is only **`-0.8%..-3.7%` CPU** (`-0%..-10%`
     RSS at `t=1`); `PyFile` scratch-`PyByteArray` `readinto` on `io.BytesIO` saves only **`-0.4%..-2.0%`
     CPU** (only 3 × `4 MiB` reads on `wikidata.bz2`) while adding `+13.6%` RSS at `t=4`!

Opened `idea` claim [`python-zero-extra-copy`](../../../frontier/claims/python-zero-extra-copy.toml) and
recorded the scoped dead-end row in `frontier/dead-ends.tsv`.

---

## 1. Call-count audit of shipped `python/mibz2/__init__.py`

Instrumenting `BZ2Reader.readinto` on `wikidata.bz2` (`209,700,196 B`, 431 blocks) and `text.bz2`
(`12,042,918 B`, 925 lines):

| Python entry point | Output bytes | `BZ2Reader.readinto` calls | Buffer sizes passed to `readinto` |
|---|---:|---:|---|
| `mibz2.open(wiki).read()` | 209,700,196 | **25,829** | `[8192]` (`io.RawIOBase.readall` default!) |
| `BZ2Reader(wiki).read()` | 209,700,196 | **25,829** | `[8192]` |
| `mibz2.open(wiki)` `read(1 MiB)` loop | 209,700,196 | 632 | `3,248 .. 1,048,576` (partial tail fills) |
| `BZ2Reader(wiki).readinto(1 MiB)` loop | 209,700,196 | 432 | `[1,048,576]` (1 per block + EOF) |
| `mibz2.open(text)` `for line in f:` | 12,042,918 | 22 | `[1,048,576]` |

---

## 2. Paired benchmark: shipped `mibz2` vs `PyBytes::new_with_writer` + `PyBuffer` prototype vs native Rust (`wikidata.bz2`, 209.7 MB)

All arms verified SHA-256 identical (`3f075e30f4f8c734...`). Paired alternating order (`n = 5` repeats on
`wikidata`); `VmHWM` measured in isolated fresh subprocesses:

### `wikidata.bz2`, `t = 1`
- Native Rust ceilings: `fill_buf` `780 ms` CPU (`25.1 MiB` HWM), `read_1m` `830 ms` CPU (`25.9 MiB`), `decode` `980 ms` CPU (`227.6 MiB`).

| Workload | Shipped wall (ms) | Proto wall (ms) | Wall diff | Shipped CPU (ms) | Proto CPU (ms) | CPU diff | Shipped HWM (MiB) | Proto HWM (MiB) | RSS diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **`open_read_all`** | 2664.35 | 2291.27 | **-14.00%** | 1262.60 | 1003.24 | **-20.54%** | 440.8 | 238.6 | **-45.9%** |
| **`raw_readall`** | 2883.02 | 1834.75 | **-36.36%** | 1231.14 | 978.43 | **-20.53%** | 440.9 | 238.6 | **-45.9%** |
| **`decompress`** | 1825.00 | 1428.08 | **-21.75%** | 1103.51 | 935.57 | **-15.22%** | 436.4 | 249.9 | **-42.7%** |
| `open_read_1m` | 1792.20 | 1734.62 | -3.21% | 861.02 | 876.42 | +1.79% | 46.0 | 41.6 | -9.5% |
| `raw_readinto_1m` | 1291.97 | 1397.87 | +8.20% | 778.42 | 762.27 | -2.08% | 44.0 | 39.6 | -10.0% |
| `bytesio_open_read_1m` | 775.75 | 788.65 | +1.66% | 768.40 | 765.64 | -0.36% | 60.2 | 56.3 | -6.5% |

### `wikidata.bz2`, `t = 2`
- Native Rust ceilings: `fill_buf` `629.2 ms` wall / `790 ms` CPU (`49.8 MiB` HWM), `read_1m` `840 ms` CPU (`47.8 MiB`), `decode` `1030 ms` CPU (`255.2 MiB`).

| Workload | Shipped wall (ms) | Proto wall (ms) | Wall diff | Shipped CPU (ms) | Proto CPU (ms) | CPU diff | Shipped HWM (MiB) | Proto HWM (MiB) | RSS diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **`open_read_all`** | 2071.36 | 1490.83 | **-28.03%** | 1189.13 | 1002.78 | **-15.67%** | 450.0 | 258.7 | **-42.5%** |
| **`raw_readall`** | 1988.18 | 1240.39 | **-37.61%** | 1255.98 | 996.33 | **-20.67%** | 450.4 | 267.2 | **-40.7%** |
| **`decompress`** | 1565.09 | 1047.47 | **-33.07%** | 1183.62 | 970.50 | **-18.01%** | 452.0 | 267.0 | **-40.9%** |
| `open_read_1m` | 1336.34 | 1432.93 | +7.23% | 908.00 | 888.45 | -2.15% | 64.2 | 70.1 | +9.3% |
| `raw_readinto_1m` | 1536.25 | 1564.01 | +1.81% | 849.10 | 847.32 | -0.21% | 67.2 | 66.7 | -0.8% |
| `bytesio_open_read_1m` | 818.82 | 833.00 | +1.73% | 864.13 | 848.37 | -1.82% | 82.1 | 79.9 | -2.6% |

### `wikidata.bz2`, `t = 4`
- Native Rust ceilings: `fill_buf` `650.2 ms` wall / `880 ms` CPU (`70.7 MiB` HWM), `read_1m` `656.2 ms` wall / `880 ms` CPU (`92.2 MiB`), `decode` `777.0 ms` wall / `1040 ms` CPU (`284.7 MiB`).

| Workload | Shipped wall (ms) | Proto wall (ms) | Wall diff | Shipped CPU (ms) | Proto CPU (ms) | CPU diff | Shipped HWM (MiB) | Proto HWM (MiB) | RSS diff |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **`open_read_all`** | 1192.10 | 1029.80 | **-13.61%** | 1259.79 | 1084.31 | **-13.93%** | 468.8 | 301.1 | **-35.8%** |
| **`raw_readall`** | 1319.89 | 926.74 | **-29.79%** | 1258.59 | 1067.92 | **-15.15%** | 479.8 | 299.7 | **-37.5%** |
| **`decompress`** | 1689.55 | 1006.76 | **-40.41%** | 1266.60 | 1058.85 | **-16.40%** | 471.4 | 296.4 | **-37.1%** |
| `open_read_1m` | 916.91 | 941.05 | +2.63% | 939.44 | 932.40 | -0.75% | 89.9 | 93.1 | +3.5% |
| `raw_readinto_1m` | 845.79 | 844.66 | -0.13% | 882.64 | 869.92 | -1.44% | 92.3 | 96.9 | +5.0% |
| `bytesio_open_read_1m` | 884.71 | 869.95 | -1.67% | 934.89 | 915.73 | -2.05% | 107.6 | 122.3 | +13.6% |

### `abi3-py310` pure-Python `BZ2Reader.readall()` ablation (`wikidata.bz2`, `n = 5`)
Overriding `BZ2Reader.readall()` / `read(-1)` in pure Python with 1 MiB `_reader.read(1 << 20)` + `b"".join(chunks)` (zero ABI change, 10 LOC):
- `t = 1`: wall `2433.61 -> 2394.08 ms` (`-1.62%`), CPU `1245.95 -> 1104.23 ms` (**`-11.37%`**)
- `t = 4`: wall `1152.10 -> 1083.77 ms` (**`-5.93%`**), CPU `1275.74 -> 1185.74 ms` (**`-7.05%`**)

### `chunks_into_pybytes_drop` negative ablation (`wikidata.bz2`, `t = 1`, `n = 7`)
Decoding into `Vec<Vec<u8>>` block chunks and copying into `PyBytes_FromStringAndSize(NULL, total)` while dropping each `900 KiB` chunk 1:1 gave `decompress` CPU `1131.8 -> 1168.3 ms` (`+3.22%`) and peak RSS **`436.2 -> 434.9 MiB` (`-0.3%`)** because glibc dynamically raises `M_MMAP_THRESHOLD` past `900 KiB` after the first free, leaving freed chunks resident on the `brk` heap.

---

## 3. Paired benchmark on `text.bz2` (`12.04 MB`, 925 lines, `n = 7` interleaved)

| Workload | `t=1` CPU diff | `t=1` RSS diff | `t=2` CPU diff | `t=2` RSS diff | `t=4` CPU diff | `t=4` RSS diff |
|---|---:|---:|---:|---:|---:|---:|
| **`open_read_all`** | **-6.67%** | **-20.9%** (`51.8->41.0`) | **-2.12%** | **-11.3%** | **-5.76%** | **-9.1%** (`80.1->72.9`) |
| **`raw_readall`** | **-3.28%** | **-20.7%** (`51.6->41.0`) | **-5.03%** | **-7.9%** | **-5.76%** | **-9.3%** (`82.3->74.6`) |
| `open_read_1m` | -3.59% | -3.0% | +0.23% | -2.2% | -1.69% | -4.0% |
| `raw_readinto_1m` | -5.76% | -3.3% | -1.72% | +0.6% | -0.78% | -5.7% |
| `open_lines` | +0.49% | -3.4% | -1.68% | -2.7% | +0.75% | -4.5% |
| `decompress` | +4.08% | -2.2% | -3.01% | +13.4% | -0.92% | +9.8% |
| `bytesio_open_read_1m` | — | — | +0.87% | +6.2% | -2.37% | +3.7% |

(Note: `decompress` on `text.bz2` used `initial_cap = 4 * len(bz2) = 7.4 MiB`, triggering a grow to `14.8 MiB` while `text` is only `12.04 MiB`, causing a tiny `+6 MiB` RSS bump on the `12 MiB` corpus; starting at `min(4 * len(bz2), 1 MiB)` or `1 MiB` avoids oversizing small streams while still growing smoothly via `PyBytesWriter`.)

---

## 4. Recommended next step (`frontier/claims/python-zero-extra-copy.toml`)

1. **Immediate `abi3-py310`-compatible win (zero packaging change):** override `BZ2Reader.readall()` and `BZ2Reader.read(size=-1)` to eliminate the `25,829 × 8 KiB` loop (`-7%..-11%` CPU on `open().read()`).
2. **Full single-copy / 1.0× RSS win (`-14%..-40%` wall, `-14%..-21%` CPU, `-36%..-46%` peak RSS):** stream `BzReader::fill_buf()` directly into `PyBytes::new_with_writer` (100% safe Rust, GIL detached per block!) for `Reader::readall()` and `decompress()`, and add `Reader::readinto()` via `PyBuffer<u8>`. Because PyO3 0.29's `PyBytesWriter` uses CPython's native `_PyBytes_Resize` only when `not(Py_LIMITED_API)` (and `PyBuffer<u8>` requires `abi3-py311` or `not(Py_LIMITED_API)`), adopting the full version entails deciding whether to ship per-version CPython wheels instead of a single `abi3-py310` wheel (captured in the claim's `question` and `needs_louis` / packaging check).
