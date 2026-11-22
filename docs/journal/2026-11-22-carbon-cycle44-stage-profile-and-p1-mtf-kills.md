# Carbon Cycle 44 (`6/20`) — Post-`#745` Stage Profile on `origin/master` (`a3f26aa0`), `mtf_ring16` Kill, and `LUT10` Pareto Proof

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Base (`origin/master`):** `47b37019` (`src/` at `a3f26aa0` / `4a79f063`, Wormhole `#739` + Carbon `#745`)  
**Safety:** `#![forbid(unsafe_code)]` on all candidate code

---

## 1. Per-Stage CPU Profile on `origin/master` (`a3f26aa0`, Post-`#739 + #745`)

Measured across 3 steady-state rounds with reused `Scratch<OpenCells>` (`CLOCK_THREAD_CPUTIME_ID`):

| Stage | `wikidata-l9` (`431` blks, `383` runs) | `wikidata-l1` (`361` blks, `361` runs) | `records-l9` (`11` blks, `0` runs) |
|---|---:|---:|---:|
| **`P1` (`block + huff -> syms`)** | `66.53 ms` (**`14.70%`**) | `22.37 ms` (**`19.69%`**) | `33.45 ms` (**`21.88%`**) |
| **`P2a` (`runs::takes_runs`)** | `0.21 ms` (`0.05%`) | `0.14 ms` (`0.12%`) | `0.00 ms` (`0.00%`) |
| **`P2b` (`runs::front` / `mtf::mtf`)** | `41.09 ms` (**`9.08%`**) | `10.95 ms` (**`9.64%`**) | `76.49 ms` (**`50.03%`**) |
| **`P3` (`OpenCells` inverse BWT)** | `320.57 ms` (`70.85%`) | `79.75 ms` (`70.20%`) | `41.71 ms` (`27.28%`) |
| **`P4` (`unrle_in_place` + CRC)** | `24.04 ms` (`5.31%`) | `0.40 ms` (`0.35%`) | `1.22 ms` (`0.80%`) |
| **Pre-BWT Total (`P1 + P2`)** | **`107.83 ms` (`23.83%`)** | **`33.46 ms` (`29.45%`)** | **`109.94 ms` (`71.92%`)** |

Key structural finding on `wikidata-l9`: the `48` non-runs blocks are tiny stream-tail blocks totaling **`< 0.01 ms`** of `mtf::mtf`; **`99.99%+` (`41.09 ms`) of `P2b` on `wikidata-l9` is `runs::front`**, executed **inside the BWT workers** on cold `Vec<u16>` `syms` migrated across cores!

---

## 2. Cycle 44 Same-Binary A/B + A/A Experiments & One-Line Kills

### 2.1 Safe Register-`u128` + Moving-Base `16 x 16` MTF (`mtf_ring16`, `#![forbid(unsafe_code)]`)

Tested against `origin/master`'s `mtf::mtf` (`mtf_avx2`, 9 interleaved rounds, exact byte + count parity verified across all blocks):

| Corpus | Master `mtf_avx2` (ms) | A/A | Safe `mtf_ring16` (ms) | Ratio (`Cand / Master`) | Verdict |
|---|---:|---:|---:|---:|---|
| `records-l9` (`11` blocks) | `78.96` | `1.0049x` | `191.32` | `2.4230x` | **KILL** |
| `wikidata-l9` (`431` blocks forced MTF) | `115.73` | `0.9933x` | `213.01` | `1.8406x` | **KILL** |

- **One-line kill (`mtf_ring16`):** Variable-trip `1..g` bucket-cascade branch mispredicts + unaligned `u64` store-to-load forwarding stalls ran `2.423x` slower than `mtf_avx2`'s branchless 7-row `32 B` shift on `records-l9`.

### 2.2 Isolated Phase-1 (`bits + huff + block`) Ablations vs `#745` (`11` Interleaved Rounds)

| Candidate | `wikidata-l9` (`61.61 ms`, A/A `0.9976x`) | `wikidata-l1` (`20.57 ms`, A/A `1.0034x`) | `records-l9` (`32.64 ms`, A/A `0.9987x`) | Verdict |
|---|---:|---:|---:|---|
| Inline `SUB_PTR` inside `5x` `step_win!` | `1.2600x` | `1.2329x` | `1.3767x` | **KILL** |
| `LUT_BITS = 11` (`8 KiB`/table, `#[cold]` kept) | `1.0077x` | `1.0835x` | `1.1355x` | **KILL** |
| Branchless `refill_win` (`pos & 255`) | `1.0044x` | `1.0035x` | `1.0138x` | **KILL** |
| `LUT10` + `stack_extend` (`[u16; 66]`) | `0.9924x` | `1.0121x` | `1.0091x` | **KILL** |

- **One-line kills:**
  1. **Inline `SUB_PTR` in `step_win!`:** Unrolling `5` copies of `sub_lut` branches inside `decode_group_win` spilled `Reg` to the stack (`+23%` to `+38%` regression).
  2. **`LUT_BITS = 11` (`6 x 8 KiB = 48 KiB`):** Overflowed L1d alongside `selectors` + `syms` and doubled `build_into` cost (`+8.4%` on `-1`, `+13.6%` on `records-l9`), proving `#745`'s `LUT_BITS = 10` (`6 x 4 KiB = 24 KiB`) is L1d-optimal.
  3. **Branchless `refill_win` (`pos & 255`):** `win.get(pos..pos+8)` already predicts at `100%`, whereas `pos & 255` adds a 1-cycle address-generation dependency (`1.004x–1.014x`).
  4. **Standalone `stack_extend`:** Within A/A noise (`0.992x–1.012x`) when `syms` capacity stays warm on one thread.

---

## 3. Cycle 45 (`7/20`) Structural Target: Scanner-Resident `P1 + P2` (`runs::front`) Fusion

In Interim Handoff 1's `ab-child/2` 10-thread profile (`wikidata_x3.bz2`), the 3 parallel front-end scanners used only `0.08 + 0.06 + 0.06 = 0.20 s` while each BWT worker took `0.12–0.16 s` — because every BWT worker still executes `P2` (`takes_runs + runs::front`, **`41.3 ms` = `10.7%` of worker CPU**) on cross-core migrated `Vec<u16>` `syms`!

Moving `takes_runs + runs::front` onto the parallel scanners **immediately after `read_item_tight` while `syms` is L2-hot on the same core**:
1. Cuts every BWT worker's critical path by **`-10.7%`** (`385.9 ms -> 344.6 ms` across 8 workers = `43.1 ms`, vs `104.5 ms / 3 = 34.8 ms` across 3 scanners -> **`0.648x` modeled 10T critical path vs `a3f26aa0`**!).
2. Keeps `Vec<u16>` `syms` **100% private to the 3 scanners** (`3` `Vec<u16>`s total instead of `12+` in `Jobs`/`free_syms`, cutting peak RSS by another `-3 to -5 MiB`!) and eliminates cross-core `50 MB` `syms` cache-line migration.
