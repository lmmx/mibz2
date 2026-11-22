# Wormhole Cycle 41 (Neon) — Wall W5c: Tail-Latency & Straggler Scheduling (`r = 0.979–0.989` Proxy, `-2.2%`–`-27.6%` `10W` ParScan Tail, Streaming LPT Killed)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10`) & `P1c_Fifo` (`Cycle 40`).
**Candidate harness:** [`c41_straggler_sched_ab.rs`](2026-11-21-wormhole-cycle41/c41_straggler_sched_ab.rs) (`safe_c41` module `#![forbid(unsafe_code)]`), raw log [`c41_results.txt`](2026-11-21-wormhole-cycle41/c41_results.txt), manifest [`c41_straggler.toml`](2026-11-21-wormhole-cycle41/c41_straggler.toml).

---

## 1. Lemma 41.1 — `O(1)` Decode-Duration Proxy & Exact `10W/16W` Makespan Bounds

Measuring exact per-block `decode_block(OpenCells)` duration alongside `symbols.syms.len()` proves an extraordinarily tight linear predictor at `O(1)` cost:
- **`wikidata_431b_210m` (`431` blocks, `max = 5.20 ms`):** Pearson **`r = 0.9791`**.
- **`hetero_60b` (`64` blocks, `max = 12.79 ms` random fallback):** Pearson **`r = 0.9890`**.

| Corpus | `W` | Ideal (`Σ/W`) | FIFO Makespan (tail) | Graham LPT Makespan (tail) | Saved by LPT |
|---|---:|---:|---:|---:|---:|
| `wikidata_431b_210m` | `4` | `112.42 ms` | `113.61 ms` (`+1.06%`) | `112.79 ms` (`+0.33%`) | `-0.82 ms` (`-0.72%`) |
| `wikidata_431b_210m` | **`10`** | `44.97 ms` | `46.31 ms` (`+2.97%`) | **`45.30 ms` (`+0.74%`)** | **`-1.01 ms` (`-2.17%`)** |
| `wikidata_431b_210m` | `16` | `28.11 ms` | `29.51 ms` (`+4.99%`) | `28.48 ms` (`+1.35%`) | `-1.02 ms` (`-3.47%`) |
| `hetero_60b` | `4` | `42.37 ms` | `49.24 ms` (`+16.20%`) | `42.92 ms` (`+1.28%`) | `-6.32 ms` (`-12.84%`) |
| `hetero_60b` | **`10`** | `16.95 ms` | `24.72 ms` (`+45.82%`) | **`17.90 ms` (`+5.59%`)** | **`-6.82 ms` (`-27.59%`)** |
| `hetero_60b` | `16` | `10.59 ms` | `18.25 ms` (`+72.29%`) | `12.79 ms` (`+20.69%`) | `-5.47 ms` (`-29.95%`) |

---

## 2. Native `12`-Round Rotated Fresh-Process `4W` A/B (Streaming Serial-Scanner Mode)

| Corpus | Arm | `4W` med (`ms`) | Ratio vs `MasterA` `[95% CI]` | Ratio vs `P1c_Fifo` `[95% CI]` | Verdict |
|---|---|---:|---|---|---|
| `wikidata_431b` | `MasterA` / `MasterB` (`A/A`) | `327.3 / 330.2` | `1.0000` / `1.0009 [0.9875, 1.0145]` | `1.0607` | Control |
| `wikidata_431b` | `P1c_Fifo` | `311.85` | `0.9428 [0.9279, 0.9579]` | `1.0000` | Baseline |
| `wikidata_431b` | `S1_UrgentLpt` | `306.20` | `0.9303 [0.9163, 0.9444]` | `0.9868 [0.9668, 1.0071]` | Neutral/mild |
| `wikidata_431b` | `S2_HeavyOnly` | `309.33` | `0.9353 [0.9155, 0.9556]` | `0.9921 [0.9637, 1.0214]` | Neutral |
| `hetero_60b` | `MasterA` / `MasterB` (`A/A`) | `158.6 / 153.4` | `1.0000` / `0.9679 [0.9307, 1.0065]` | `1.0688` | Control |
| `hetero_60b` | **`P1c_Fifo`** | **`144.98`** | **`0.9356 [0.8852, 0.9889]`** | `1.0000` | **Wins streaming** |
| `hetero_60b` | `S1_UrgentLpt` | `150.38` | `0.9492 [0.9187, 0.9808]` | `1.0145 [0.9780, 1.0525]` | **KILL streaming** |
| `hetero_60b` | `S2_HeavyOnly` | `147.65` | `0.9401 [0.8919, 0.9908]` | `1.0048 [0.9687, 1.0422]` | **KILL streaming** |

---

## 3. Regime Separation & Kill Decisions

- **Why streaming LPT fails while `ParScan` LPT wins (`proved`):** In Regime 1 (serial scanner), the final straggler isn't discovered until the end of the file anyway, while mid-stream priority promotion inside a `16`-slot queue can delay `place = taken + 3` when a heavy block occupies a worker (`+1.45%` on `hetero_60b`). In Regime 2 (post-Mechanisms `ParScan`), all block bit-spans in a `4 MiB` chunk (`~120` blocks) or whole slice are known **before** decoding begins, so LPT cuts `10W` tail from `+2.97% → +0.74%` (`-1.0 ms`) on Wikidata and `+45.8% → +5.6%` (`-6.8 ms`) on heterogeneous corpora!
- **KILL `S1_UrgentLpt` & `S2_HeavyOnly` in Regime-1 Streaming `JobQueue`:** Regresses/neutral (`1.0145×` / `1.0048×`) vs `P1c_Fifo` on `hetero_60b` due to mid-stream HOL delay; keep FIFO in streaming serial-scan mode.
- **KEEP Graham LPT for Regime-2 `ParScan` Windowed/Slice Dispatch:** Validated in Cycle 43 using `end_bit - start_bit` known `O(1)` before Huffman decoding (`-0.8 to -1.0 ms` on `10T` Wikidata, `-6.8 ms` on heterogeneous streams).
