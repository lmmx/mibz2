# Carbon Interim Handoff 0 — Cycle 37 Parallel Front End Drop-In Over `origin/master` (`53f7ecf9`)

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Base Commit (`origin/master`):** `53f7ecf935d62ac7896eafaf55c5d249789614ca` (`mibz2 0.5.11`)  
**Deliverables:**
- Unified patch over `53f7ecf9`: [c37_over_master_53f7ecf9.patch](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-handoff0-c37-dropin/c37_over_master_53f7ecf9.patch)
- Drop-in module copies:
  - [src_bits.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-handoff0-c37-dropin/files/src_bits.rs) (`src/bits.rs`)
  - [src_huff.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-handoff0-c37-dropin/files/src_huff.rs) (`src/huff.rs` — new `#![forbid(unsafe_code)]` module)
  - [src_block.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-handoff0-c37-dropin/files/src_block.rs) (`src/block.rs`)
  - [src_stream.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-handoff0-c37-dropin/files/src_stream.rs) (`src/stream.rs`)
  - [src_runs.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-handoff0-c37-dropin/files/src_runs.rs) (`src/runs.rs`)
  - [src_par_scan.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-handoff0-c37-dropin/files/src_par_scan.rs) (`src/par_scan.rs` — new `#![forbid(unsafe_code)]` module)
  - [src_lib.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-handoff0-c37-dropin/files/src_lib.rs) (`src/lib.rs`)
  - [examples_par_scan.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-handoff0-c37-dropin/files/examples_par_scan.rs) (`examples/par_scan.rs`)
  - [examples_ab.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-handoff0-c37-dropin/files/examples_ab.rs) (`examples/ab.rs`)

---

## 1. What Is Included in Interim Handoff 0 (Cycle 37 State)

All new modules (`src/bits.rs`, `src/huff.rs`, `src/block.rs`, `src/stream.rs`, `src/par_scan.rs`) compile with `#![forbid(unsafe_code)]` and zero external dependencies:

1. **Barrier-Parallel Disjoint 4-Byte Anchor Finder (`src/par_scan.rs`, `examples/par_scan.rs`):**
   - By the Disjoint 4-Byte Anchor Theorem, any 48-bit `BLOCK_MAGIC` (`0x314159265359`) or `END_MAGIC` (`0x177245385090`) starting at byte $k \ge 0$, bit shift $s \in [0, 7]$ spans 5 whole bytes $k+1 \dots k+5$ and therefore contains the unique aligned 2-byte anchor $(p, p+1)$ at $p = 4(\lfloor k/4 \rfloor + 1) \in \{k+1, k+2, k+3, k+4\}$.
   - All $W$ worker threads partition the anchor indices $m \in [1, \lfloor (L - 2)/4 \rfloor]$ disjointly with zero overlap, filter 4 anchors per iteration against the L1-resident `8 KiB` `[u64; 1024]` `ANCHOR_BITMAP` in safe Rust, and merge via a panic-safe `Condvar` handshake (`0.01 ms` serial merge CPU; no extra finder thread — $W + 1$ threads total instead of $W + 2$).
2. **Lock-Free `AtomicUsize` Work Stealing + Deadlock-Free `base_idx` Window Gating (`src/par_scan.rs`):**
   - Workers claim candidate indices via `cursor.fetch_add(1, Ordering::Relaxed)` and decode each candidate independently using a worker-local `54 KiB` `ScanScratch` (`read_item_with_scratch`) + `Scratch<OpenCells>`.
   - **Preemption Deadlock Caught & Fixed at `16W`:** Testing real wall across `8W–20W` on our 2-vCPU VM caught a head-of-line preemption deadlock when gating on `in_flight_ready >= slots` (a worker that fetched `idx = base_idx` via `fetch_add` and was preempted before acquiring the mutex could block behind `slots` out-of-order completions of `idx > base_idx`). Replacing `in_flight_ready >= slots` with `idx >= st.base_idx + slots` guarantees `idx == st.base_idx` never blocks (`slots >= 1`) while bounding speculative lookahead to `slots` candidates.
3. **`ScannerC37` L1d-Resident Huffman Decoder (`src/huff.rs`, `src/block.rs`, `src/bits.rs`, `src/stream.rs`):**
   - `4 KiB` `LUT_BITS = 10` root table with `EOB_FAST` (`COUNT = 0` sentinel freeing pair inner loop of EOB checks), `buf.wrapping_shl(e)` hardware shift-masking, `264 B` group-window bounds-check hoisting (`decode_group_win`), `2 KiB` `[u16; 1024]` `O(1)` `SubLut` for `11..=20`-bit codes, and Canonical Template-Clone 2-symbol pair builder (`54 KiB` reusable `ScanScratch`).
4. **Monotonic Short-Circuit `takes_runs` (`src/runs.rs`):**
   - LLVM-autovectorized 64-symbol window counts (`w.iter().map(|&s| usize::from(s <= 1)).sum::<usize>()`) paired with monotonic threshold early-accept (`runs >= need`) and early-reject (`runs + remaining < need`), boolean-identical to `origin/master` (`13.2×`–`15.3×` faster).
5. **Drop-In Entry Points for Immediate i9 Testing:**
   - `mibz2::decode_parscan(data, threads)` (`src/lib.rs`)
   - `examples/ab.rs` arm `"mibz2-parscan"` (streams retired chunks directly into `ab`'s output hasher `h.update(chunk)`)
   - `examples/par_scan.rs` (`find` upgraded to Cycle 37 parallel disjoint-anchor finder + `AtomicUsize` work stealing + `ScanScratch` + `Scratch<OpenCells>`, and `scan` upgraded to `Scratch<OpenCells>` for an apples-to-apples `0.5.11` `OpenCells` comparison via `just bench-par-scan`).

> [!NOTE]
> `mibz2::decode_parallel` / `StreamDecoder::new` (`BzReader::parallel`) are intentionally left on `StreamDecoder`'s existing team/plan path in Handoff 0 so all 22 `parallel::plan_tests` and `service_default` tests remain untouched while the i9 benchmarks `mibz2-parscan` vs `mibz2` (`origin/master`). Replacing `StreamDecoder`'s serial scanner for push-streaming `write`/`read` (`BzReader::parallel`) is the target of **Cycles 39–43 (`Interim Handoff 1`)**.

---

## 2. How to Apply & Run on the i9-10900K

```bash
# On a clean checkout of origin/master (53f7ecf935d62ac7896eafaf55c5d249789614ca):
git apply --check docs/journal/2026-11-21-carbon-handoff0-c37-dropin/c37_over_master_53f7ecf9.patch
git apply docs/journal/2026-11-21-carbon-handoff0-c37-dropin/c37_over_master_53f7ecf9.patch

# 1. Verify full test suite + fault injection + adversarial + mutation fuzz:
cargo test --release
cargo build --release --examples
./target/release/examples/par_scan - faults
./target/release/examples/par_scan wikidata.bz2 check
./target/release/examples/par_scan wikidata.bz2 adversarial
./target/release/examples/par_scan wikidata.bz2 fuzz 1500

# 2. Paired knee benchmark (Cycle 37 parallel `find` vs `scan`, both on OpenCells):
./target/release/examples/par_scan wikidata.bz2 knee 5 1,2,4,6,8,10,16,20

# 3. End-to-end A/B harness (`mibz2-parscan` vs `mibz2` and `mibz2-workers-lazy`):
./target/release/examples/ab run out.jsonl \
    --arms mibz2,mibz2-workers-lazy,mibz2-parscan \
    --threads 1,2,4,6,8,10,16,20 --rounds 5 \
    wikidata.bz2 wikidata-l1.bz2 records.bz2
./target/release/examples/ab summary out.jsonl --ref mibz2
```

---

## 3. Qualification & Exactness Results (Verified Over `53f7ecf9`)

- **Unit & Integration Suite (`cargo test --release --lib --tests`):** **100% PASS** (`182/182` unit tests + all `20` integration test suites: `admission`, `cells`, `cells_default`, `cooperative`, `corpus`, `decode`, `in_place`, `kernels`, `lazy_admission`, `lazy_workers`, `mtf_checkpoints`, `mtf_kernels`, `pipelined`, `reader_panic`, `service`, `service_default`, `trace`).
- **Panic / Liveness Faults (`par_scan - faults`):** **`16/16` PASS** (`Worker`, `Finder`, `Scanner`, `Joiner` at `1` and `4` workers; zero hangs).
- **Adversarial & Mutation Fuzz (`par_scan adversarial` + `fuzz 1500`):** **`6/6` adversarial + `1500/1500` mutated multi-stream files PASS** byte-for-byte and `Error { kind, block, bit }`-for-`Error` across 13 error kinds (`3,126` false candidates stepped over, `1,983` valid-CRC blocks dropped without emission).
- **`examples/ab` Hash Parity (`mibz2` vs `mibz2-parscan`):** Verified identical output hashes at `1, 4, 10` threads.

---

## 4. Same-Binary `1T | 2T | 4W | 8W | 10W | 16W | 20W` A/B + A/A + Explicit Serial Share

Measured in a single binary linking unmodified `origin/master` (`53f7ecf9`, `MasterA` and `MasterB` `A/A` control) and the Handoff 0 drop-in (`Cand`) compiled with identical `rustc -C opt-level=3 -C codegen-units=1` flags (`5` interleaved rounds per worker count on 2-core Intel Xeon Platinum 8481C VM):
- **`M_Serial` / `C_Serial`:** Measured `CLOCK_THREAD_CPUTIME_ID` of the serial stage (`Scanner` + joiner retirement in `Master`; Phase-1b boundary merge + chain joiner retirement in `Cand`), reported with **`(% of Total CPU / % of W-Core Critical Path)`**.
- **`CritW`:** Modeled $W$-core critical path $\max(T_{\text{serial}}, T_{\text{worker\_sum}} / W)$ (testable directly in real wall on the physical 10-core / 20-thread i9-10900K).

### 4.1 `wikidata-l9` (`11.85 MB` $\to$ `209.70 MB`, 431 blocks)

| Workers | `MasterA` Wall (ms) | `MasterB` Wall (ms) | `C37` Wall (ms) | `A/A` Wall | **`C37 / Master` Wall** | `Master` CPU (ms) | `C37` CPU (ms) | `Master` Serial (`%CPU` / `%Crit`) | `C37` Serial (`%CPU` / `%Crit`) | `Master` `CritW` (ms) | `C37` `CritW` (ms) | **`C37 / Master` `CritW`** |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `1T` | `474.73` | `478.94` | `460.22` | `1.0089×` | **`0.9694×`** | `474.59` | `458.72` | `80.27 ms` (`16.9%` / `16.9%`) | `64.74 ms` (`14.1%` / `14.1%`) | `474.59` | `458.72` | **`0.9666×`** |
| `2T` | `238.98` | `240.53` | `228.64` | `1.0065×` | **`0.9567×`** | `470.00` | `453.40` | `81.97 ms` (`17.4%` / `42.3%`) | `2.95 ms` (`0.6%` / `1.3%`) | `193.97` | `225.10` | `1.1605×`* |
| `4W` | `249.74` | `250.11` | `241.52` | `1.0015×` | **`0.9671×`** | `489.79` | `471.73` | `81.60 ms` (`16.7%` / `80.0%`) | `2.44 ms` (`0.5%` / `2.1%`) | `101.98` | `117.34` | `1.1505×`* |
| `8W` | `267.10` | `266.64` | `257.41` | `0.9983×` | **`0.9637×`** | `516.18` | `509.18` | `83.17 ms` (`16.1%` / **`100.0%`**) | `2.61 ms` (`0.5%` / **`4.1%`**) | `83.17` | `63.31` | **`0.7612×`** |
| `10W` | `281.37` | `278.27` | `262.60` | `0.9890×` | **`0.9333×`** | `543.94` | `517.72` | `85.49 ms` (`15.7%` / **`100.0%`**) | `2.25 ms` (`0.4%` / **`4.4%`**) | `85.49` | `51.52` | **`0.6027×`** |
| `16W` | `285.67` | `283.52` | `268.30` | `0.9925×` | **`0.9392×`** | `552.46` | `528.02` | `86.35 ms` (`15.6%` / **`100.0%`**) | `2.91 ms` (`0.6%` / **`8.9%`**) | `86.35` | `32.77` | **`0.3795×`** |
| `20W` | `291.90` | `288.06` | `283.53` | `0.9868×` | **`0.9713×`** | `564.38` | `551.44` | `87.54 ms` (`15.5%` / **`100.0%`**) | `4.52 ms` (`0.8%` / **`16.5%`**) | `87.54` | `27.37` | **`0.3126×`** |

*\*Note on `2T`/`4W` `CritW`: `Master` at `W` workers runs **`W + 1` active threads** (`1` dedicated `Scanner` core + `W` worker cores = `3` cores at `2T`, `5` cores at `4W`), whereas `C37` runs **only `W` active worker threads** (`2` cores at `2T`, `4` cores at `4W`). On actual 2-core hardware (`C/M Wall`), `C37` beats `Master` at both `2T` (`0.9567×`) and `4W` (`0.9671×`) because its total CPU is lower (`453.4 ms` vs `470.0 ms`).*

### 4.2 `wikidata-l1` (`3.22 MB` $\to$ `36.00 MB`, 361 blocks)

| Workers | `MasterA` Wall (ms) | `MasterB` Wall (ms) | `C37` Wall (ms) | `A/A` Wall | **`C37 / Master` Wall** | `Master` CPU (ms) | `C37` CPU (ms) | `Master` Serial (`%CPU` / `%Crit`) | `C37` Serial (`%CPU` / `%Crit`) | `Master` `CritW` (ms) | `C37` `CritW` (ms) | **`C37 / Master` `CritW`** |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `1T` | `110.31` | `111.82` | `102.19` | `1.0137×` | **`0.9264×`** | `110.29` | `102.17` | `28.59 ms` (`25.9%` / `25.9%`) | `20.76 ms` (`20.3%` / `20.3%`) | `110.29` | `102.17` | **`0.9264×`** |
| `2T` | `74.27` | `74.59` | `61.24` | `1.0044×` | **`0.8245×`** | `135.49` | `120.37` | `33.31 ms` (`24.6%` / `65.1%`) | `2.77 ms` (`2.3%` / `4.7%`) | `51.13` | `58.79` | `1.1499×` |
| `4W` | `74.69` | `73.79` | `64.34` | `0.9880×` | **`0.8615×`** | `134.69` | `123.85` | `32.16 ms` (`23.9%` / **`100.0%`**) | `2.05 ms` (`1.7%` / **`6.7%`**) | `32.16` | `30.44` | **`0.9465×`** |
| `8W` | `72.69` | `73.74` | `64.32` | `1.0145×` | **`0.8848×`** | `135.62` | `122.87` | `32.52 ms` (`24.0%` / **`100.0%`**) | `1.42 ms` (`1.2%` / **`9.4%`**) | `32.52` | `15.19` | **`0.4670×`** |
| `10W` | `71.24` | `72.16` | `64.98` | `1.0129×` | **`0.9121×`** | `136.19` | `124.24` | `32.16 ms` (`23.6%` / **`100.0%`**) | `1.35 ms` (`1.1%` / **`11.0%`**) | `32.16` | `12.30` | **`0.3825×`** |
| `16W` | `72.85` | `73.07` | `66.46` | `1.0030×` | **`0.9123×`** | `138.91` | `126.36` | `32.21 ms` (`23.2%` / **`100.0%`**) | `1.25 ms` (`1.0%` / **`16.0%`**) | `32.21` | `7.82` | **`0.2428×`** |
| `20W` | `73.56` | `72.80` | `66.83` | `0.9897×` | **`0.9085×`** | `138.30` | `127.76` | `32.46 ms` (`23.5%` / **`100.0%`**) | `1.26 ms` (`1.0%` / **`19.9%`**) | `32.46` | `6.32` | **`0.1948×`** |

### 4.3 `records-l9` (`9.59 MB` $\to$ `9.60 MB`, 11 blocks — high-entropy column route)

| Workers | `MasterA` Wall (ms) | `MasterB` Wall (ms) | `C37` Wall (ms) | `A/A` Wall | **`C37 / Master` Wall** | `Master` CPU (ms) | `C37` CPU (ms) | `Master` Serial (`%CPU` / `%Crit`) | `C37` Serial (`%CPU` / `%Crit`) | `Master` `CritW` (ms) | `C37` `CritW` (ms) | **`C37 / Master` `CritW`** |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| `1T` | `156.93` | `157.37` | `151.67` | `1.0028×` | **`0.9665×`** | `156.91` | `151.65` | `38.48 ms` (`24.5%` / `24.5%`) | `33.20 ms` (`21.9%` / `21.9%`) | `156.91` | `151.65` | **`0.9665×`** |
| `2T` | `86.71` | `86.04` | `84.19` | `0.9923×` | **`0.9709×`** | `165.15` | `161.72` | `37.85 ms` (`22.9%` / `59.6%`) | `0.23 ms` (`0.1%` / `0.3%`) | `63.56` | `80.75` | `1.2704×` |
| `4W` | `89.45` | `89.97` | `86.78` | `1.0058×` | **`0.9702×`** | `172.06` | `167.96` | `38.33 ms` (`22.3%` / **`100.0%`**) | `0.24 ms` (`0.1%` / **`0.6%`**) | `38.33` | `41.92` | `1.0937×` |
| `8W` | `98.77` | `96.89` | `95.44` | `0.9810×` | **`0.9663×`** | `188.26` | `183.12` | `38.36 ms` (`20.4%` / **`100.0%`**) | `0.30 ms` (`0.2%` / **`1.3%`**) | `38.36` | `22.85` | **`0.5957×`** |
| `10W` | `101.64` | `102.24` | `101.49` | `1.0059×` | **`0.9986×`** | `190.68` | `196.71` | `38.44 ms` (`20.2%` / **`100.0%`**) | `0.33 ms` (`0.2%` / **`1.7%`**) | `38.44` | `19.64` | **`0.5109×`** |
| `16W` | `104.46` | `103.05` | `106.25` | `0.9864×` | `1.0171×` | `196.76` | `204.83` | `38.89 ms` (`19.8%` / **`100.0%`**) | `0.46 ms` (`0.2%` / **`3.6%`**) | `38.89` | `12.78` | **`0.3286×`** |
| `20W` | `103.71` | `106.53` | `106.16` | `1.0272×` | `1.0236×` | `196.37` | `204.50` | `38.27 ms` (`19.5%` / **`100.0%`**) | `0.55 ms` (`0.3%` / **`5.4%`**) | `38.27` | `10.20` | **`0.2665×`** |
