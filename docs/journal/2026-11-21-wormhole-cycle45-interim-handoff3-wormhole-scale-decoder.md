# Wormhole Cycle 45 (Neon) — INTERIM HANDOFF #3 (`Cycles 41–45`): Unified `WormholeScaleDecoder` Qualification (`4W = 0.8935×`, `10W sys -77%`, `10W RSS -49%`, `16W RSS -50%`)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10b2e2e781518a101db7ccf8b6352b986`), `MasterA` vs `MasterB` (`A/A = 1.0027 [0.9960, 1.0095]` across `15` rotated rounds!).
**Mechanisms Coordination:** `origin/research/carbon-word-transport` (`300a29976`, `C38b StreamParScan`). `R2_Par_*` uses `mibz2`'s existing public `BlockSpan`/`Bits::at`/`read_item` with zero scan/finder duplication.
**Artifacts:** [`c45_handoff3_ab.rs`](2026-11-21-wormhole-cycle45/c45_handoff3_ab.rs) (`wormhole_scale` module `#![forbid(unsafe_code)]`), [`c45_results.txt`](2026-11-21-wormhole-cycle45/c45_results.txt), [`c45_handoff3.toml`](2026-11-21-wormhole-cycle45/c45_handoff3.toml).

---

## 1. Unified Qualification Table (`wikidata_431b_210m.bz2`, `211.0 MB`, `431` blocks, exact `AbHash`)

### 1a. `15`-Round Rotated Fresh-Process `4W` Headline

| Arm | `4W` med (`ms`) | Ratio vs `MasterA` `[95% CI]` | `user_ms` | **`sys_ms` (`Δ`)** | **`minflt` (`Δ`)** | **`vcsw` (`Δ`)** | **`rss_MB` (`Δ`)** |
|---|---:|---|---:|---:|---:|---:|---:|
| `MasterA` (`v0.5.12`) | `373.00` | `1.0000 [1.0000, 1.0000]` | `660.71` | `37.94` | `15,925` | `704` | `59.78` |
| `MasterB` (`A/A`) | `374.07` | `1.0027 [0.9960, 1.0095]` | `658.02` | `38.48` | `15,713` | `723` | `59.99` |
| **`R1_Stream_P1W0`** (Drop-in stream) | **`351.60`** | **`0.9391 [0.9206, 0.9581]`** | `630.59` | `25.55` (`-32.7%`) | `9,606` (`-39.7%`) | `404` (`-42.6%`) | `49.98` (`-16.4%`) |
| **`R1_Stream_P2`** (Stream + `0 B ll` + `12M`) | `362.40` | `0.9658 [0.9536, 0.9781]` | `654.93` | `22.42` (`-40.9%`) | `8,023` (`-49.6%`) | `384` (`-45.5%`) | `44.90` (`-24.9%`) |
| **`R2_Par_P1W0`** (`ParScan` + AVX2 `unrle`) | **`332.70`** | **`0.8935 [0.8858, 0.9013]`** | **`619.63`** | `12.25` (`-67.7%`) | `4,841` (`-69.6%`) | `226` (`-67.9%`) | `35.23` (`-41.1%`) |
| **`R2_Par_P2`** (`ParScan` + `0 B ll` + `12M` + `L1`) | `341.65` | **`0.9145 [0.8991, 0.9302]`** | `642.01` | **`10.12` (`-73.3%`)** | **`3,749` (`-76.5%`)** | **`220` (`-68.8%`)** | **`31.11` (`-48.0%`)** |

### 1b. Cold-Start `W = 10` & `W = 16` Qualification (`5`-round median)

| `W` | Arm | `wall_ms` (`Δ`) | `user_ms` | **`sys_ms` (`Δ`)** | **`minflt` (`Δ`)** | **`vcsw` (`Δ`)** | **`rss_MB` (`Δ`)** |
|---:|---|---:|---:|---:|---:|---:|---:|
| `10` | `MasterA` / `MasterB` | `382.3 / 381.6` | `654.0 / 670.0` | `78.18 / 68.69` | `28,259 / 27,311` | `323 / 332` | `100.38 / 96.57` |
| `10` | `R1_Stream_P1W0` | `360.48` (`-5.7%`) | `662.34` | `38.85` (`-50.3%`) | `16,831` (`-40.4%`) | `370` | `77.59` (`-22.7%`) |
| `10` | `R1_Stream_P2` | `361.97` (`-5.3%`) | `676.23` | `27.69` (`-64.6%`) | `12,705` (`-55.0%`) | `476` | `61.55` (`-38.7%`) |
| `10` | **`R2_Par_P1W0`** | **`332.07` (`-13.1%`)** | **`620.69`** | `28.97` (`-62.9%`) | `11,640` (`-58.8%`) | **`85` (`-73.7%`)** | `61.84` (`-38.4%`) |
| `10` | **`R2_Par_P2`** | `338.34` (`-11.5%`) | `650.46` | **`17.92` (`-77.1%`)** | **`8,996` (`-68.2%`)** | **`97` (`-70.0%`)** | **`51.45` (`-48.7%`)** |
| `16` | `MasterA` / `MasterB` | `391.2 / 385.0` | `687.3 / 669.0` | `75.75 / 81.07` | `37,532 / 37,645` | `239 / 328` | `136.08 / 137.98` |
| `16` | `R1_Stream_P1W0` | `361.49` (`-7.6%`) | `657.43` | `45.14` (`-40.4%`) | `18,984` (`-49.4%`) | `558` | `82.25` (`-39.6%`) |
| `16` | `R1_Stream_P2` | `369.98` (`-5.4%`) | `680.01` | `37.39` (`-50.6%`) | `14,406` (`-61.6%`) | `556` | `68.61` (`-49.6%`) |
| `16` | **`R2_Par_P1W0`** | **`350.14` (`-10.5%`)** | **`644.03`** | `38.23` (`-49.5%`) | `18,106` (`-51.8%`) | **`152`** | `87.11` (`-36.0%`) |
| `16` | **`R2_Par_P2`** | `353.81` (`-9.6%`) | `657.47` | **`28.15` (`-62.8%`)** | **`13,180` (`-64.9%`)** | **`177`** | **`67.67` (`-50.3%`)** |

---

## 2. Phase-3 Keep / Kill Ledger (`Cycles 41–45`)

| Cycle | Candidate | Verdict | One-Line Evidence |
|---:|---|---|---|
| C41 | `Lemma 41.1` (`syms.len()` proxy `r=0.979–0.989`) + `ParScan` LPT | **KEEP (Regime 2 tail)** | Exact `10W` sim: `-2.17%` (`-1.0 ms`) Wikidata, `-27.6%` (`-6.8 ms`) `hetero_60b`. |
| C41 | `S1/S2` Mid-Stream Streaming `JobQueue` LPT | **KILL** | `1.0145×` on `hetero_60b` in serial-scan mode (mid-stream HOL delay). |
| C42 | `C42` ParScan-Coupled Interface (`TLS syms + Atomic Steal + P2`) | **PROMOTE** | `4W = 0.8935×`, `10W -13.1%` wall / `-77.1%` `sys_ms` / `-68.2%` `minflt` / `-48.7%` RSS. |
| C43 | Residual `95.1%` Parallel Profile + Comet Lake SMT Budget | **PROVED** | `Open 69.4% + Huff 16.8% + Front 8.9% = 95.1%`; `10` phys workers + `21%`-duty SMT reader optimal. |
| C43 | `M1_PreSyms` (`384 KiB` TLS) & `M2_SpanLPT` (16-blk mid-stream) | **KILL BOTH** | Neutral vs A/A (`1.0009×` / `0.9979×` at `10W`) while adding `+219..436` `minflt`. |
| C44 | `L1_AtomicFast` + Directed `cv_worker.notify_one()` | **PROMOTE (Thm 44.1)** | Cuts `16W` `vcsw` `1,078 → 177..217` (`-80%`), `10W` `vcsw` to `82..97`; sync closed (`~13 µs` total). |
| C44 | `L2_Sharded` (Per-Slot Mutexes) | **KILL** | Double-checked `wake_mu` multiplies `vcsw` `3.3×` (`262 → 870` at `16W`) with neutral wall. |

---

## 3. Final Phase Roadmap (`Cycles 46–50`, Final Handoff at Cycle 50, then STOP)

Every major wall (`W2` allocator/page-faults, `W3` post-BWT DRAM/RLE+CRC, `W4` window L3 pollution, `W5` hand-off/locks/reader) is now measured and broken. Final 5 cycles close the last remaining `>2%` opportunities and stress-verify everything:
- **C46:** Contiguous Slab/Arena Pool vs Separate `960 KiB` `Vec`s & Linux `2 MiB` THP (`madvise(MADV_HUGEPAGE)`) cold-start study.
- **C47:** Parallel `calloc` + `W`-Way Scatter Flatten for API-identical `decode_parallel(&[u8]) -> Result<Vec<u8>>`.
- **C48:** Multi-Regime Stress Verification (`wikidata-l9`, `wikidata-l1` 100 KiB small-block, `hetero_60b`, `dense`, `random`, corrupt/truncated/cancel) & i9-10900K `trimmed_here` Recalibration Table.
- **C49:** Final 6-Step Cumulative Ablation & Timeline Verification (`v0.5.12` vs `WormholeScaleDecoder` end-to-end).
- **C50 (FINAL ROUND 5 HANDOFF & STOP):** Complete `Cycles 31–50` keep/kill ledger, `4W` headline + `10T/16T/20T` Regime 1 & Regime 2 critical-path models, and ordered Coordinator PR roadmap.
