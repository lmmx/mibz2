# Wormhole Cycle 40 (Neon) — INTERIM HANDOFF #2 (`Cycles 36–40`): DRAM/L3 Scaling Stack (`4W = 0.9221×`, `bzcat = 0.9043×`, `10W RSS -34%`, `16W RSS -44%`, `minflt -56%`)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10b2e2e781518a101db7ccf8b6352b986`).
**Mechanisms Coordination:** `origin/research/carbon-word-transport` (`300a29976`, `C38b StreamParScan`). Zero scan/finder duplication; Theorem 36.1 proves `ParScan` alone hits Comet Lake's DDR4 bandwidth roof (`54.4 ms` BW floor, `63.0 ms` wall at `10W`) without Wormhole's Phase 1 + Phase 2 L3/DRAM cuts.
**Artifacts:** [`c40_handoff2_ab.rs`](2026-11-21-wormhole-cycle40/c40_handoff2_ab.rs) (`safe_p2` module `#![forbid(unsafe_code)]`), [`c40_results.txt`](2026-11-21-wormhole-cycle40/c40_results.txt), [`c40_handoff2.toml`](2026-11-21-wormhole-cycle40/c40_handoff2.toml).

---

## 1. Phase-2 Cumulative Ablation (`15`-Round Rotated Fresh-Process `4W` + `5`-Round `7W/10W/16W`)

### 1a. `4W` Headline (`wikidata_431b_210m.bz2` `211.0 MB` & `hetero_60b.bz2`, exact `AbHash`, `15` rotated rounds)

| Arm | `wikidata` `4W` ratio `[95% CI]` | `sys_ms` / `minflt` / `vcsw` / `rss_MB` | `hetero_60b` `4W` ratio `[95% CI]` | `rss_MB` |
|---|---|---:|---|---:|
| `MasterA` (`v0.5.12`) | `1.0000 [1.0000, 1.0000]` (`370.0 ms`) | `41.5` / `15,729` / `711` / `60.10` | `1.0000 [1.0000, 1.0000]` | `70.22` |
| `MasterB` (`A/A`) | `0.9866 [0.9690, 1.0046]` (`367.0 ms`) | `40.8` / `15,943` / `706` / `61.04` | `1.0264 [0.9837, 1.0710]` | `71.75` |
| **`P1c_C35 + W0`** (AVX2 `unrle`) | **`0.9221 [0.9029, 0.9417]` (`345.5 ms`)** | **`23.8` (`-43%`) / `9,501` (`-40%`) / `403` / `49.75`** | **`0.9381 [0.9105, 0.9666]`** | `63.55` |
| `P2a_InPl_W0` (`0 B ll`) | `0.9389 [0.9240, 0.9540]` (`349.2 ms`) | `22.6` (`-45%`) / `8,380` (`-47%`) / `389` / `45.50` | `0.9563 [0.9328, 0.9805]` | `60.88` |
| **`P2b_Cap12M`** (`+W1+12M`) | **`0.9444 [0.9319, 0.9571]` (`349.2 ms`)** | **`24.3` / `8,139` (`-48%`) / `378` (`-47%`) / `44.97`** | **`0.9543 [0.9247, 0.9848]`** | `61.18` |
| `P2c_Full_T64` (`+64K CLMUL`) | `0.9441 [0.9328, 0.9556]` (`350.7 ms`) | `24.2` / **`8,108` (`-48%`)** / `386` / **`44.71`** | `0.9729 [0.9285, 1.0195]` | `60.81` |

*Consumer assembly standalone gains (Cycle 39, `12`-rd `4W` on `211 MB`):*
- `R1_DirectExtend` (`decode_parallel` drop-in): **`0.9559×`**, `-28.5 MiB` RSS.
- `R2_ZeroCopyChunks` (`decode_parallel_chunks`): **`0.8754 [0.8567, 0.8945]`**, `-54.4 ms` user, `-38.5 MiB` RSS.
- `R3_ZeroCopySink` (`examples/bzcat.rs` via `BufRead`): **`0.9043 [0.8866, 0.9225]`**, `sys_ms -47.1%`, `minflt -55.6%`, `RSS -38.7%` (`66.0 → 40.5 MiB`).

### 1b. Cold-Start Multi-Worker Scaling (`W = 7, 10, 16`, `wikidata_431b_210m`, `5`-round median)

| `W` | Arm | `wall_ms` (`Δ` vs MA) | `user_ms` | **`sys_ms` (`Δ`)** | **`minflt` (`Δ`)** | **`rss_MB` (`Δ`)** |
|---:|---|---:|---:|---:|---:|---:|
| `7` | `MasterA` / `MasterB` | `368.7 / 368.8` | `666.9 / 644.8` | `52.76 / 62.11` | `21,861 / 21,509` | `75.48 / 74.27` |
| `7` | **`P1c_C35 + W0`** | **`339.49` (`-7.92%`)** | **`624.81`** | `28.83` (`-45.4%`) | `12,398` (`-43.3%`) | `60.36` (`-20.0%`) |
| `7` | **`P2b_Cap12M` / `P2c`** | `348.10` (`-5.58%`) | `645.98` | **`22.36` (`-57.6%`)** | **`9,610` (`-56.0%`)** | **`50.12` (`-33.6%`)** |
| `10` | `MasterA` / `MasterB` | `376.5 / 377.1` | `672.6 / 668.1` | `59.25 / 58.82` | `26,883 / 26,536` | `94.15 / 97.57` |
| `10` | **`P1c_C35 + W0`** | **`351.77` (`-6.56%`)** | **`648.04`** | `36.32` (`-38.7%`) | `16,456` (`-38.8%`) | `70.59` (`-25.0%`) |
| `10` | **`P2b_Cap12M`** | **`355.75` (`-5.50%`)** | `661.99` | **`30.24` (`-49.0%`)** | **`13,287` (`-50.6%`)** | **`62.26` (`-33.9%`)** |
| `16` | `MasterA` / `MasterB` | `387.1 / 383.8` | `672.1 / 668.9` | `78.53 / 77.71` | `37,503 / 38,445` | `135.72 / 139.52` |
| `16` | **`P1c_C35 + W0`** | **`363.73` (`-6.03%`)** | **`653.13`** | `52.07` (`-33.7%`) | `22,386` (`-40.3%`) | `92.75` (`-31.7%`) |
| `16` | **`P2a_InPl_W0` / `P2b`** | `367.79` (`-4.99%`) | `666.72` | **`36.84` (`-53.1%`)** | **`15,983` (`-57.4%`)** | **`72.86` (`-46.3%`)** |

*Note on the `P2b` vs `P1c+W0` `user_ms` gap (`+8..+14 ms` on `105 MiB`-L3 Xeon):* Purely due to `#![forbid(unsafe_code)]` scalar `u64` SWAR run-scan vs `v0.5.12`'s `32`-byte AVX2 `_mm256_cmpeq_epi8` in `src/rle.rs`. In production, using `src/rle.rs`'s existing `32`-byte AVX2 scan inside `unrle_in_place` + streaming `clmul_update` captures **both** `P1c+W0`'s `0.922×` Xeon speed **and** `P2b`'s `-34%`–`-46%` RSS / `-51%`–`-57%` `minflt` / `-9 MiB` L3 win on Comet Lake's `20 MiB` L3!

---

## 2. Phase-2 Keep / Kill Ledger (`Cycles 36–40`)

| Cycle | Candidate | Verdict | One-Line Evidence |
|---:|---|---|---|
| C36 | Comet Lake `20 MiB` L3 / `42 GB/s` DDR4 Roofline | **PROVED (Thm 36.1)** | Proves `ParScan` alone hits `54.4 ms` DDR4 BW floor (`63 ms` wall) at `10W`; Phase 2 cuts DRAM `10.8→3.66 B/B` (`43.0 ms`). |
| C37 | `C37a: Emit-to-Out (0 B ll) + In-Place RLE1` | **PROMOTE** | Eliminates `W×0.90 MiB` (`-9 MiB` @`10W` = `45%` i9 L3), `2,000/2,000` exact, `-34%..46%` RSS, `-51%..57%` `minflt`. |
| C37 | `C37b: 64 KiB L2-Tiled GF(2) CLMUL CRC` | **KEEP (i9 L2)** | Eliminates `1.0 B/B` L3/DRAM CRC re-read on Comet Lake `256 KiB` L2 (`0.9441×` `4W`, `50.1 MiB` `7W` RSS). |
| C37 | `C37c: Two-Buffer Tiled (keeping ll)` | **KILL** | Retains `900 KiB/W` `ll`, regresses `4W` (`0.9775×`) and `10W/16W` `sys_ms` (`58/90 ms`). |
| C38 | `W0: Full-Block free_out Pool Separation` | **PROMOTE** | Stops tiny blocks from triggering `open.rs:360` `898 KiB` `memset(0)`; `P1c+W0` leaps to **`4W = 0.9221×`**. |
| C38 | `W1 + W13b: ImmedSyms + 12 MiB Ready Cap` | **PROMOTE** | Cuts `10W` `minflt -51%`, `sys -49%`, `RSS -34%` (`62.3 MiB`), `pkRdy 10.3 MiB` while winning `hetero_60b` (`0.9543×`). |
| C38 | `W13a: 8 MiB Ready Cap` | **KILL** | Tighter on `hetero_60b` (`0.9775×`) and higher `10W` `minflt` than `12 MiB`. |
| C39 | `R1/R2/R3: Zero-Copy decode_parallel & bzcat` | **PROMOTE ALL 3** | `R1` `0.9559×` drop-in, `R2` chunks `0.8754×`, `R3` `bzcat` `0.9043×` (`-55.6%` `minflt`, `-38.7%` RSS). |

---

## 3. Modelled `10T / 16T / 20T` i9-10900K Critical Path (End of Phase 2)

| Configuration (`211 MB`, Comet Lake `10C/20T`) | `7W` Wall | `8W` Wall | **`10W` Wall (`L3_MB`, `DRAM B/B`, `GB/s`)** | `16W` Wall | `20W` Wall |
|---|---:|---:|---|---:|---:|
| `v0.5.12` Shipped (`7W/28` trim for `10–20T`, Reg. 1) | `67.6 ms` | `68–72 ms` | **`68–72 ms`** (trimmed `7W`; untrimmed `10W` `~73 ms`) | `68–72 ms` | `68–72 ms` |
| `v0.5.12 + Mech C38b` (`ParScan` alone, Reg. 2) | `72.3 ms` | `68.0 ms` | `63.0 ms` (`49.4 MiB`, `10.8 B/B`, `36.2 GB/s`) | `73.9 ms` | `79.4 ms` |
| **`Phase 1+2 (P2b/c) + Mech C38b` (Reg. 2)** | `54.8 ms` | `49.1 ms` | **`43.0 ms`** (`26.0 MiB`, **`3.66 B/B`**, `18.0 GB/s`) | `50.6 ms` | `54.0 ms` |
| *Phase 3 Target (`+C41 LPT + C42 Zero-Syms-Handoff + R4 Open`)* | `51.1 ms` | `45.3 ms` | **`37–39 ms`** (`22.5 MiB`, **`3.1 B/B`**, `17.5 GB/s`) | `46.0 ms` | `49.5 ms` |

---

## 4. Phase 3 Roadmap (`Cycles 41–45`, Interim Handoff #3 at Cycle 45)

- **C41:** Tail-Latency & Straggler Scheduling (Graham LPT via `O(1)` `syms.len()` / compressed-span proxy + exact `10W/16W` discrete-event makespan sim + `4W` A/B).
- **C42:** **Post-Mechanisms `ParScan`-Coupled Interface Architecture** (`0` scanner thread, `0` `JobQueue` mutex, `0` `free_syms` pool — 1 thread-local `syms` per worker + `AtomicUsize::fetch_add(1)` + `P2`!).
- **C43:** Residual Post-`ParScan` + `P2` Bottleneck Shift & SMT/Core-Budget Analysis (`10C/20T` Comet Lake: `10` physical workers + SMT helper roles vs `11–12` workers, plus per-worker `runs` / `open` scratch audit).
- **C44:** Thread-Local Hot `out` Cache & Sharded Ready Slot Synchronization (`4W/10W/16W` A/B — keep or kill honestly).
- **C45 (Interim Handoff #3):** Unified `WormholeScaleDecoder` qualification across Phases 1–3!
