# Carbon Cycle 47 (9/20): Zero-Shift Bipolar In-Place RLE1 & Auto-Vectorized `takes_runs`

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Base `origin/master`:** `8c1a73ea67dba7f2b97528f9ae0ad954d30f60e6` (`#739` + `#745`)  
**Constraint:** Zero new `unsafe` (`100%` safe Rust in `unrle_in_place` and `takes_runs`).

---

## 1. Mathematical Mechanism: Discrete IVT & Zero-Shift Bipolar In-Place RLE1 (`src/rle.rs`)

On `origin/master`, `rle::unrle_in_place` (`P4`, `5.31%` of total decode time on `wikidata-l9`) scans `buf[0..n]` for 4-byte runs `(j_r, c_r, extra_r)`, computes `shift = max_r d_r` where $d_r = \text{dst}_r - \text{src}_r$, and whenever `shift > 0`:
1. Calls `buf.reserve_exact(shift); buf.resize(n + shift, 0); buf.copy_within(first..n, first + shift);` — moving `buf[first..n]` up by `shift` bytes, and
2. Copies every stretch a **second time** (`buf.copy_within(src..from, dst)`) back down to its final offset (or swaps into `stage = &mut s.ll` when `shift > n`).

On `wikidata-l9`, **`59 / 383` runs-route blocks (`49.90 MB / 209.71 MB` = `23.8%` of all output bytes; `149.7 MB` on `wikidata_x3.bz2`)** have `shift > 0`, causing `99.8 MB` (`49.9 MB` read + `49.9 MB` write) of redundant L3/DRAM traffic across concurrent workers.

### Theorem (Discrete IVT & Monotone Zone Decomposition for In-Place RLE1)
Let $d_0 = 0$ and $d_{r+1} = d_r + (\text{extra}_r - 1)$ for $0 \le r < m$.
1. **Discrete Intermediate Value Property:** Because $\text{extra}_r \ge 0$, every step satisfies $d_{r+1} - d_r \ge -1$. Consequently, $d$ **cannot cross from positive ($> 0$) to negative ($< 0$) without hitting $d_v = 0$ ($\text{dst}_v = \text{src}_v$) in between**.
2. **Strict Output Monotonicity:** $\text{dst}_0 \le \text{dst}_0 + L_0 < \text{dst}_1 \le \dots \le \text{dst}_m + L_m = \text{len}$, so no stretch or run ever writes past $\text{len} = n + d_m$ (eliminating both `n + shift` over-reservation and the `shift > n` `stage` buffer swap).
3. **Single-Copy Bipolar Execution:**
   - Stretches with $d_r = 0$ stay in place (`0` bytes copied).
   - Stretches with $d_r < 0$ copy forward (`dst_r < src_r`).
   - Whenever run $r$ bridges $d_r \le 0 \to d_{r+1} > 0$, let $v \in \{r+1, \dots, m\}$ be the first index with $d_v = 0$ (or $v = m$ if $d > 0$ to the end). Walking $q = v$ down to $r + 1$ backward copies each stretch right (`dst_q > src_q`) and fills run $q$ in a **single pass** without overwriting unread input, followed by filling bridging run $r$ and streaming `64 KiB` L2-hot `crc::update_with` tiles up to the zone boundary.

We also replaced the scalar `usize` `.iter().map(...)` sum in `runs::takes_runs` (`src/runs.rs`) with a 16-wide `u32` chunked helper (`count_run_syms`) that auto-vectorizes cleanly to SSE2/AVX2 without `unsafe`.

---

## 2. Verification & Microbenchmark Results

- `c47_rle_screen`:
  - **`10,000 / 10,000` adversarial RLE1 fuzz cases** (alternating `extra = 0` shrinkages, `extra = 255` expansions, multi-zone zero crossings, `> 150 KiB` multi-tile blocks, `shift > n` bombs, and `RunUnfinished` errors): **PASS** (`100%` byte- and CRC-exact).
  - Standalone `wikidata-l9` `P4` (`unrle_in_place`, `15` interleaved rounds): `Master = 39.28 ms` (`A/A = 0.9879x`) -> **`Cand Bipolar = 38.14 ms` (`0.9712x`)**, eliminating `99.8 MB` of L3/DRAM copy traffic.
- `cargo clippy --all-targets -- -D warnings`: **PASS** (`0` warnings)
- `cargo test --release --lib rle::tests`: **PASS** (`4/4`)
- `cargo test --release --features trace --lib trace::tests`: **PASS** (`5/5` footprint-ledger invariants exact)
- `c41_bench_and_verify verify`: **PASS** (`PASS 1..5`, `1,500 / 1,500` streaming chunked mutation fuzz cases exact)

---

## 3. Subprocess `VmHWM` Peak RSS (`wikidata_x3.bz2`, `629.10 MB` decoded)

| Threads | `native` Master | `native` `c47` | `native` Ratio | `i9` Master | `i9` `c47` | `i9` Ratio | Gate (`<= 1.10x` at `6–20T`) |
|---:|---:|---:|---:|---:|---:|---:|---|
| `1` | `28.43 MiB` | `28.32 MiB` | `0.9963x` | `28.34 MiB` | `28.39 MiB` | `1.0018x` | — |
| `2` | `61.78 MiB` | `64.60 MiB` | `1.0456x` | `65.16 MiB` | `62.23 MiB` | `0.9551x` | — |
| `4` | `98.68 MiB` | `90.97 MiB` | `0.9219x` | `48.72 MiB` | `44.85 MiB` | **`0.9206x`** | — |
| `6` | `135.31 MiB` | `134.68 MiB` | `0.9954x` | `57.21 MiB` | `52.02 MiB` | **`0.9093x`** | **PASS** |
| `8` | `61.87 MiB` | `49.35 MiB` | **`0.7976x`** | `62.45 MiB` | `48.66 MiB` | **`0.7792x`** | **PASS** |
| `10` | `67.43 MiB` | `53.28 MiB` | **`0.7902x`** | `59.38 MiB` | `48.91 MiB` | **`0.8237x`** | **PASS** |
| `12` | `72.03 MiB` | `57.65 MiB` | **`0.8004x`** | `59.81 MiB` | `48.62 MiB` | **`0.8130x`** | **PASS** |
| `16` | `77.18 MiB` | `63.92 MiB` | **`0.8282x`** | `58.98 MiB` | `48.48 MiB` | **`0.8219x`** | **PASS** |
| `20` | `85.42 MiB` | `72.07 MiB` | **`0.8437x`** | `58.91 MiB` | `49.23 MiB` | **`0.8358x`** | **PASS** |

---

## 4. Same-Binary Push-Streaming A/B (`7` Interleaved Rounds, `-C opt-level=3 -C codegen-units=1`)

### `wikidata-l9` (`209.70 MB` out; Serial share: Master `14.18%` (`66.24 ms`) -> Cand `0.74%`; Modeled 10T crit = `0.7072x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `737.99` | `1.0020x` | **`0.9976x`** | **`0.9794x`** | `0.9977x` |
| `2` | `451.62` | `1.0152x` | **`0.9901x`** | **`0.9695x`** | `0.9964x` |
| `4` | `465.88` | `1.0152x` | **`0.9838x`** | **`0.9792x`** | `0.9971x` |
| `6` | `477.67` | `1.0071x` | `1.0135x` | `1.0002x` | `1.0034x` |
| `8` | `465.36` | `0.9864x` | **`0.9744x`** | **`0.9801x`** | **`0.9760x`** |
| `10` | `475.28` | `0.9760x` | **`0.9562x`** | **`0.9639x`** | **`0.9911x`** |
| `12` | `479.82` | `0.9973x` | **`0.9609x`** | **`0.9815x`** | `0.9929x` |
| `16` | `487.37` | `0.9874x` | **`0.9469x`** | **`0.9633x`** | `0.9899x` |
| `20` | `485.97` | `0.9979x` | **`0.9380x`** | **`0.9594x`** | **`0.9853x`** |

### `wikidata-l1` (`36.00 MB` out; Serial share: Master `18.45%` (`21.87 ms`) -> Cand `0.78%`; Modeled 10T crit = `0.6203x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `156.28` | `0.9918x` | **`0.9862x`** | **`0.9394x`** | `0.9861x` |
| `2` | `95.73` | `0.9730x` | **`0.9708x`** | **`0.9579x`** | `1.0006x` |
| `4` | `89.12` | `0.9790x` | **`0.9984x`** | **`0.9977x`** | `1.0066x` |
| `6` | `87.64` | `1.0109x` | **`0.9903x`** | **`0.9579x`** | **`0.9860x`** |
| `8` | `92.50` | `0.9846x` | `1.0475x` | `1.0177x` | `0.9943x` |
| `10` | `91.87` | `1.0163x` | `1.0615x` | `1.0248x` | `1.0070x` |
| `12` | `92.72` | `0.9897x` | `1.0465x` | `1.0144x` | `1.0165x` |
| `16` | `96.15` | `0.9789x` | `1.0150x` | `1.0157x` | `0.9761x` |
| `20` | `93.75` | `1.0214x` | `1.0254x` | `0.9703x` | `1.0007x` |

### `records-l9` (`9.60 MB` out; Serial share: Master `21.25%` (`33.41 ms`) -> Cand `1.37%`; Modeled 10T crit = `0.4734x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `170.27` | `1.0019x` | **`0.9945x`** | `0.9669x` | `0.9953x` |
| `2` | `91.04` | `1.0002x` | `1.0119x` | `1.0136x` | `1.0189x` |
| `4` | `93.41` | `1.0204x` | `1.0150x` | `0.9766x` | `1.0273x` |
| `6` | `99.09` | `0.9882x` | `1.0298x` | `1.0218x` | `1.0128x` |
| `8` | `104.02` | `1.0135x` | `1.0278x` | `0.9920x` | `1.0354x` |
| `10` | `106.85` | `0.9910x` | `1.0259x` | `1.0108x` | `1.0284x` |
| `12` | `107.12` | `0.9833x` | `1.0294x` | `1.0184x` | `1.0542x` |
| `16` | `106.18` | `1.0218x` | `1.0491x` | `1.0238x` | `1.0461x` |
| `20` | `102.24` | `1.0117x` | `1.0660x` | `1.0087x` | `1.0660x` |

---

## 5. Next Step: Cycle 48 (`10/20`) — Interim Handoff 2 Qualification & Publication

Fetch latest `origin/master`, rebase and qualify the complete `Cycles 44..48` candidate stack (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --release --lib --tests`, `trace::tests`, `examples/ab` 10T, `1,500`-case streaming fuzz, `VmHWM` peak RSS, `git apply --check` against pristine `origin/master`), and publish **Interim Handoff 2**.
