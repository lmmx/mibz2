# Wormhole Cycle 10 (Round 2 Handoff) — Multi-Worker `BzReader` Verification, Contention Audit, and the Complete Floor–Upper–Gap–Route Synthesis

**Date:** 2026-11-21
**Branch:** `research/carbon-wormhole` (Cycles 6–10, Round 2 complete)
**Baseline `origin/master` SHA:** `d425d772c0a0e6b82e9626b36bbfa4b28a6c9613` (`mibz2 0.5.9`; `src/` verified byte-identical to `0.5.8`)
**Authenticated Corpus:** `6/6` SHA-256 verified blocks from `/home/antigravity/admitted-bwt` against `manifest.json` (`block-0.L`, `block-2.L`, `block-85.L`, `block-143.L`, `block-210.L`, `block-207.L`)
**Cycle 10 Artefacts:**
- Production-drop-in `InverseBwt` multi-worker verification & 1w/2w confrontation (`#![forbid(unsafe_code)]`): [`verify_cycle10.rs`](2026-11-21-wormhole-cycle10-floor-squeeze-and-handoff/verify_cycle10.rs)
- Gate G2 acquisition manifest (`PASS` via `check_manifest.py`): [`diet_relay8_cells.toml`](2026-11-21-wormhole-cycle10-floor-squeeze-and-handoff/diet_relay8_cells.toml)
- Raw JSON & confrontation output: [`cycle10_results.json`](2026-11-21-wormhole-cycle10-floor-squeeze-and-handoff/cycle10_results.json), [`cycle10_output.txt`](2026-11-21-wormhole-cycle10-floor-squeeze-and-handoff/cycle10_output.txt)
- Preceding Round-2 Cycle Journals: [Cycle 6 (`BMCP` Floor & Gap)](2026-11-21-wormhole-cycle6-bmcp-floor-and-gap.md), [Cycle 7 (`\Phi`/Lyndon/SA-IS Obstructions)](2026-11-21-wormhole-cycle7-phi-lyndon-sais-move-d.md), [Cycle 8 (Zero-Bit Cuts & `2P-WS8`)](2026-11-21-wormhole-cycle8-zero-bit-cuts-and-2p-ws.md), [Cycle 9 (`Relay-MLP` & Rényi–Graham Sweep)](2026-11-21-wormhole-cycle9-relay-mlp-and-renyi-destraggling.md)

---

## Executive Answer (Stated Once)

1. **LOWER BOUND (`proved` + `verified`):** In the **Bounded-MLP Hierarchical Cell-Probe Model $\mathsf{BMCP}(M_2 = 256\text{ KiB}, B = 64\text{ B}, \mu)$**, unrestricted preprocessing admits **zero** `256 KiB`-L2 misses (`Theorem 6.1`, because RLBWT entropy is `56.1–133.0 KiB < 256 KiB` on all 6 blocks), escaped only by paying $\ge 3\text{–}4$ chained bit-rank probes per step (Pătraşcu–Viola 2010; Neon `BSO-512` `1.71–2.44×` slower). Once a decoder commits to $t_{\mathrm{probe}} = 1$ load/step ($S_{\mathrm{tab}} \ge 4 m_{16} \in [351, 798]\text{ KiB} > 256\text{ KiB}$) under non-jumping $O(r)$-work preprocessing (`Theorem 6.2`, proved via a 4-Hamming text-splice adversary over $7! = 5{,}040$ permutations), **every** exact decoder must pay:
   $$Q_{\mathrm{rand}} \;\ge\; m_{16}, \qquad D_{\mathrm{serial}}(\mu) \;\ge\; \left\lceil \frac{m_{16}}{\mu} \right\rceil \;\;(\text{universally } \ge \lceil r_1/\mu \rceil \text{ for all } \Lambda \ge 1), \qquad C_{\mathrm{miss}}(256\text{K}) \;\ge\; \mathrm{OPT}_{\mathrm{Belady}}(M_2) \in [7{,}437,\; 41{,}149].$$
2. **UPPER BOUND (`verified` + `measured`):** Combined Dieted `Open` ($\mu = 1$) already achieves the **exact floor** in random-access count ($Q_{\mathrm{rand}} = m_{16} \in [89{,}874,\, 204{,}281]$) and in `2,048 KiB` 10-worker L3 DRAM misses (`479–831` misses/block = `0.034–0.059 B/B`), **but chains 100% of its `25,533–94,739` `edges` L2 misses sequentially** ($D_{\mathrm{past-}M_2}^{\mathrm{crit}} = 25{,}533\text{–}94{,}739$, `2.3–3.4×` above Belady `OPT` and `3.1–4.0×` above $\mu = 8$ MLP).
3. **THE GAP (`proved` + `measured`):** Global-order bypasses outside `Open<16>` **fail** the three-bound squeeze ($\Phi$/toehold RV bootstrap costs `2.20–5.53 MiB` scratch and `2.6–3.8×` more L2 misses, `Theorem 7.1`; Lyndon/BBWT leaves a `34.6–93.8%` single giant factor, `Theorem 7.2`; BWT-SA-IS leaves `305k–317k` LMS rows = `1.51–3.42×` $m_{16}$, `Theorem 7.3`), and 1-pass L3-safe `256 KiB` staging stalls auxiliary lanes after **`6.0–13.0%` of $m$** (`Theorem 8.2(1)`). The **only** open corridor is zero-staging multi-lane traversal of the **already L3-resident** `edges[0..m)` table.
4. **THE BEST ROUTE BELOW IT (`DietRelay8Cells`, `verified` + `measured`):** Combining $k = 8$ **Zero-Bit Xor-Mask Cuts** `((j ^ mark) & cm) == 0` ($S \in [132, 219]$, eliminating Rényi cut stragglers to `1.06–1.07×` ideal, `Theorem 9.1`), **`Relay-MLP` (`1 Frontier Emitter + 7 Cut Scouts` $\to$ `8 Parallel Emitters`, $Q_{\mathrm{rand}} = \frac{15}{8}m$, `0 B` staging heap, `Theorem 9.2`)**, and **Commutative Post-Slop 16B Boundary Restoration** (`0` inner-loop bound branches, `Theorem 9.3`):
   - Cuts critical-lane `dep_past_l2_loads` to **`0.25–0.32×` (`6,507–28,264` vs `25,533–94,739`, `-68.3%` to `-75.1%`)**;
   - Adds **`0` extra `2,048 KiB` L3 misses** (`479–831`/block = `0.034–0.059 B/B` DRAM) and **`0 B` staging heap** (`1,051.1 KiB` scratch + `879.0 KiB` `out` = **`1.885 MiB`/worker $\implies$ `10w = 18.85 MiB <= 20.0 MiB` i9 L3**);
   - Wins **`6/6` blocks** against fair safe-Rust `DietMu1Cells` at both **1 worker (`0.865–0.984×` total; `-7.6%` to `-29.0%` `emit`)** and **2 concurrent barrier-synced workers (`0.868–0.990×`)**, already beats undieted `origin/master` `OpenCells` (`4.73 MiB`/w) on the 3 largest-$m$ blocks (`block-85/143/210`: `0.938–0.980×`) even on a `2 MiB`-L2 diagnostic Xeon, and passes **`14,461` Gate G1/edge cases + 18-block `BzReader::parallel_workers_with` (`15.45 MiB`, `W = 1, 2, 4, 8`)** in `#![forbid(unsafe_code)]` safe Rust.

---

## 1. Machine Model & Six Separated Cost Dimensions

Throughout Cycles 6–10 we work in **$\mathsf{BMCP}(M_1, M_2, M_3/P, B, \mu)$** (`Cycle 6 §2`), parameterized on the target i9-10900K ($P = 10$ workers) by $M_1 = 32\text{ KiB}$ L1d, $M_2 = 256\text{ KiB}$ private L2, $M_3/P = 2{,}048\text{ KiB}$ per-worker L3 share, $B = 64\text{ B}$ cache line, and $\mu \ge 1$ concurrent memory-request lanes per worker.

**Six costs strictly separated (never pooled into a scalar):**
1. **Random-access count ($Q_{\mathrm{rand}}$):** Non-sequential loads into data structures exceeding `32 KiB` L1d.
2. **Serial dependency depth ($D_{\mathrm{serial}}, D_{\mathrm{past-}M_2}^{\mathrm{crit}}$):** Total sequential dependent load rounds ($D_{\mathrm{serial}}$) and critical-lane `256 KiB`-L2 miss chain length ($D_{\mathrm{past-}M_2}^{\mathrm{crit}}$, modeling asynchronous OoO ROB execution of $\mu$ independent cursors).
3. **Cache misses ($C_{\mathrm{miss}}(256\text{K}), C_{\mathrm{miss}}(2048\text{K})$):** Total line transfers past private L2 (into L3) and past per-worker L3 slice (into DDR4 DRAM).
4. **Preprocessing ($W_{\mathrm{prep}}, D_{\mathrm{prep}}$):** Operations and serial depth before Phase 4 emission begins.
5. **Memory footprint ($S_{\mathrm{mem}}$):** Persistent scratch heap + output buffer per worker (`<= 2,048 KiB` required for 10-worker L3 residency).
6. **Parallel work ($W_{\mathrm{par}}$):** Total instructions/bytes moved across all $\mu$ lanes (exposing redundant scans or staging copies).

**Model limits (explicitly stated):**
- *What $\mathsf{BMCP}$ counts:* Exact pointer-chain dependencies, exact `64 B` line transfers under Belady `OPT` and full-trace `LRU` across `edges`, `lens`, `word_len`, `inline16`, `spelling`, and `out`, and exact per-worker L3 capacity thresholds.
- *What $\mathsf{BMCP}$ ignores:* (a) x86-64's **16-GPR register file ceiling** (spilling lane cursors when $\mu \ge 12$), (b) **10–12 L1d Line Fill Buffer (LFB) contention** between `edges` loads and `out` streaming RFOs when $\mu \ge 12$, (c) hardware spatial next-line prefetching, and (d) branch mispredictions. Cycle 9 measured limits (a)–(b) directly: `Relay12` lowered simulated $D_{\mathrm{past-}M_2}^{\mathrm{crit}}$ below `Relay8` (`4.3k–19.7k` vs `6.5k–28.3k`) yet lost `+24..100 µs` on `6/6` blocks, pinning $\mu = 8$ as the exact hardware ceiling.

---

## 2. Complete Bounds & Candidate Ledger Across the Six Authenticated Blocks

### 2.1 Master Comparison Table (`LOWER` vs `UPPER` (`Diet_mu1`) vs `Best Route` (`DietRelay8Cells`))

| Metric (Cost Dimension) | `LOWER` Bound (`Proved`, C6 Thms 6.1–6.3) | `UPPER`: `DietMu1Cells` ($\mu=1$, `Verified`) | **Promoted: `DietRelay8Cells` ($\mu=8, k=8$, `Verified`)** | Ratio (`Relay8 / DietMu1`) |
|---|---|---|---|---|
| **1. Random-access count $Q_{\mathrm{rand}}$** (adm. non-jumping prep) | $\ge m_{16} \in [89{,}874,\; 204{,}281]$ | `89,874 – 204,281` (`1.000×` $m_{16}$) | `169,748 – 383,042` ($\frac{15}{8}m_{16} = 1.875\times$) | `1.865–1.889×` (100% L3-resident) |
| **2a. Serial step depth $D_{\mathrm{serial}}$** | $\ge \lceil m_{16}/\mu \rceil \in [11{,}235,\; 25{,}536]$ ($\mu=8$) | `89,874 – 204,281` (`1.000×` $m_{16}$) | **`22,097 – 51,418`** (`~0.250×` $m_{16}$, vs $\frac{15}{64}m = 0.234 m$) | **`0.246–0.260×` (`~4.0×` shallower)** |
| **2b. Critical L2-miss depth $D_{\mathrm{past-}M_2}^{\mathrm{crit}}$** ($t_{\mathrm{probe}}=1$) | $\ge \lceil \mathrm{OPT}_{256\text{K}}/\mu \rceil \in [930,\; 5{,}144]$ ($\mu=8$; `0` if $t_{\mathrm{probe}} \ge 4$) | `25,533 – 94,739` (`40.4%` of cycles in C16!) | **`6,507 – 28,264`** | **`0.249–0.317×` (`-68.3%` to `-75.1%`)** |
| **3a. Total `256 KiB` L2 misses $C_{\mathrm{miss}}(256\text{K})$** | Belady `OPT` `edges`: `7,437 – 41,149` | `41,196 – 111,648` (all structs) | `62,951 – 224,888` (served at `>600 GB/s` L3) | `1.53–2.13×` (hidden under 8-way MLP) |
| **3b. Total `2,048 KiB` L3 misses $C_{\mathrm{miss}}(2048\text{K})$** | Cold compulsory floor: `479 – 831` | **`479 – 831`** (`0.034–0.059 B/B` DRAM) | **`479 – 831`** (`0.034–0.059 B/B` DRAM) | **`1.000×` (`0` extra DRAM misses!)** |
| **4. Preprocessing ($W_{\mathrm{prep}}, D_{\mathrm{prep}}$)** | $O(r)$ run scan + $O(H_{16})$ RV | `OutQ16` ($D=1$) + `Node16` ($D=H_{16}$) + `wire` | Identical + `u16` `word_len` push (`< 5 µs`) | `~1.00×` (`0` extra passes) |
| **5. Per-worker memory $S_{\mathrm{mem}}$** (`scratch + out`) | `out` alone = `878.9 KiB` | `1,044.1 + 879.0 = 1,923.1 KiB` (`1.878 MiB`) | **`1,051.1 + 879.0 = 1,930.1 KiB` (`1.885 MiB`)** | **`1.004×` (`10w = 18.85 MiB <= 20 MiB` L3)** |
| **6. Parallel work $W_{\mathrm{par}}$** (Phase 4) | $n$ output bytes | $m_{16}$ edges + $n$ output bytes | $\frac{15}{8}m_{16}$ edges + $n$ output bytes + $\le 16S$ fixup (`0 B` staged) | `0 B` staging `memcpy` |

---

### 2.2 Per-Block End-to-End Safe-Rust Confrontation (`1` Worker & `2` Barrier-Synced Workers, 21 Paired Rounds)

Measured via [`verify_cycle10.rs`](2026-11-21-wormhole-cycle10-floor-squeeze-and-handoff/verify_cycle10.rs) (`#![forbid(unsafe_code)]`) against compiled `origin/master` `OpenCells` (`4.73 MiB`/worker) on the 2-core Xeon 8481C VM (`48 KiB` L1d, `2 MiB` L2, `105 MiB` L3):

| Block | $r / n$ | $m_{16}$ | `1w` `Master` (`4.73M`) | `1w` `DietMu1` (`1.88M`) | **`1w` `DietRelay8` (`1.88M`)** | **`1w` `/Diet1`** | `2w` `Master` | `2w` `DietMu1` | **`2w` `DietRelay8`** | **`2w` `/Diet1`** | **`2w` `/Master`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `0.044` | 89,874 | `1,280.4 µs` | `1,465.0 µs` | **`1,441.5 µs`** | **`0.984×`** | `1,273.8 µs` | `1,452.4 µs` | **`1,437.6 µs`** | **`0.990×`** | `1.129×` |
| `block-2.L` | `0.056` | 112,070 | `1,551.1 µs` | `1,768.0 µs` | **`1,682.2 µs`** | **`0.951×`** | `1,571.7 µs` | `1,759.6 µs` | **`1,698.3 µs`** | **`0.965×`** | `1.081×` |
| `block-85.L` | `0.055` | 156,404 | `1,607.8 µs` | `1,713.7 µs` | **`1,540.4 µs`** | **`0.899×`** | `1,618.4 µs` | `1,754.4 µs` | **`1,572.8 µs`** | **`0.897×`** | **`0.972×`** |
| `block-143.L` | `0.097` | 196,841 | `2,850.2 µs` | `3,111.2 µs` | **`2,770.6 µs`** | **`0.891×`** | `2,805.4 µs` | `3,100.5 µs` | **`2,748.1 µs`** | **`0.886×`** | **`0.980×`** |
| `block-210.L` | `0.085` | 204,281 | `2,504.0 µs` | `2,695.1 µs` | **`2,330.5 µs`** | **`0.865×`** | `2,507.1 µs` | `2,710.9 µs` | **`2,351.9 µs`** | **`0.868×`** | **`0.938×`** |
| `block-207.L` | `0.056` | 135,921 | `1,877.1 µs` | `2,102.9 µs` | **`1,905.5 µs`** | **`0.906×`** | `1,886.9 µs` | `2,088.3 µs` | **`1,910.2 µs`** | **`0.915×`** | `1.012×` |

> **Diagnostic-to-i9 Fidelity Note (`CONTEXT.md` §5a):** On this diagnostic Xeon 8481C (`2 MiB` L2, `105 MiB` L3), `Master` (`4.73 MiB`/worker) never overflows L3 and `edges` (`351–798 KiB`) hits `2 MiB` L2 (~14 cycles) rather than overflowing Comet Lake's `256 KiB` L2 into ~42-cycle L3. **Even without L3 overflow or 42-cycle L2 misses**, `DietRelay8Cells` already beats `DietMu1Cells` on **`6/6` blocks (`-1.6%` to `-13.5%` total, `-7.6%` to `-29.0%` `emit`)** and already beats undieted `Master` on `block-85/143/210` (`-2.0%` to `-6.2%`). On the target **10-core i9-10900K (`256 KiB` L2, `20 MiB` shared L3)**, where `Master` overflows L3 (`47.3 MiB > 20 MiB`) and `DietMu1Cells` pays 42-cycle L3 latency on `25.5k–94.7k` `edges` misses, `DietRelay8Cells` captures **both** the L3-residency win (`18.85 MiB <= 20 MiB`) **and** the `3.1–4.0×` `dep_past_l2` latency reduction.

---

## 3. Summary of Closed Routes (With Concrete Witness, Scope, and Escape Condition)

| Route Tested | Cycle & Tag | Concrete Witness | Exact Scope Bound | Escape Condition |
|---|---|---|---|---|
| **Unrestricted L2-miss lower bound** | C6 Thm 6.1 (`proved` + `verified`) | `H_RLBWT = 56.1–133.0 KiB < 256 KiB` L2 on `6/6` blocks | Arbitrary cell-probe decoders with unbounded probes/step | Restrict to $t_{\mathrm{probe}} = 1$ load/step ($S_{\mathrm{tab}} \ge 4m > 256\text{ KiB}$) |
| **$\Phi$ / Toehold RV Bootstrap** | C7 Thm 7.1 (`proved` + `measured`) | `92.6k–216.8k` serial splices, `2.20–5.53 MiB` scratch, `93.9k–335.6k` L2 misses (`2.6–3.8×` `emit`), `19.2k–74.2k` L3 misses | All decoders acquiring run-boundary text positions $\tau(h_k)$ via uncollapsed RV induction | Sub-interval width-$\le 16$ collapse BEFORE induction (`Open<16>`) or upstream compressor metadata |
| **BBWT / Duval Lyndon Factorisation** | C7 Thm 7.2 (`proved` + `measured`) | `1` cycle on `6/6` `.bz2` blocks; oracle Lyndon max factor = `34.6%–93.8%` of $n$ (`75.9–93.8%` on 4/6 blocks) | Independent cycle/Lyndon-factor parallel inversion | Cut a single cycle artificially via row-index predicates (`Zero-Bit Cuts`, C8) |
| **BWT-Domain SA-IS LMS Induction** | C7 Thm 7.3 (`proved` + `verified` `2,012` + `6` blocks) | Exact $O(r)$ formula gives $n_{\mathrm{LMS}} = 304{,}973\text{–}316{,}819$ (`1.51–3.42×` $m_{16}$, `1.16–1.21 MiB` table) | Level-1 Nong–Zhang–Chan SA-IS / induced-sorting recursion on BWT | Width-threshold collapse ($\Lambda=16$) which strictly dominates valley-shape LMS selection on repetitive runs |
| **1-Pass L3-Safe Staging (`1P-Stage256K`)** | C8 Thm 8.2(1) (`proved` + `measured`) | `256 KiB` staging cap stalls aux lanes after **`6.03%–13.04%`** of $m$ (`87–94%` reverts to $\mu=1$) | Single-pass multi-lane walk staging into $\le 256\text{ KiB}$ L3 headroom | Zero-staging two-phase / overlapped `Relay-MLP` (`0 B` heap staging) |
| **Coarse Zero-Bit Cuts ($k = 5$, $S \le 32$)** | C8 Thm 8.3 & C9 Thm 9.1 (`proved` + `measured`) | `block-210.L` ($S=25$): single straggler = `50,888` edges, inflating rounds to `1.80–2.27×` ideal | Work-stealing cut walks with $\frac{(\mu-1)H_S}{S} \gtrsim 1$ | Fine zero-bit cuts $k = 8$ ($S \in [132, 219]$, $1 + 7H_S/S \le 1.21\times$, measured `1.06–1.07×`) |
| **Wide Register Lanes ($\mu = 12$, `Relay12`)** | C9 §3.3 (`measured`) | `+24..100 µs` slower than `Relay8` on `6/6` blocks despite lower simulated `dep_l2_crit` | Software-interleaved multi-lane loops on 16-GPR / 10–12-LFB x86-64 cores | Cap concurrency at $\mu = 8$ (`1` emitter + `7` scouts $\to$ `8` emitters) |

---

## 4. Why `DietRelay8Cells` Survives Multi-Worker i9 Contention (Unlike `#675` `8-lane BP4K`)

In `#675` (`CONTEXT.md` §5a), an 8-lane buffered walk won `+13%` at 1 thread and lost `-5%` at 8 threads on the i9-10900K because:
1. Undieted `Open` (`4.73 MiB`) + `900 KiB` staging = **`5.63 MiB`/worker $\times 8 = 45.0\text{ MiB} > 20\text{ MiB}$ L3**, turning 8-lane MLP into **64 concurrent DDR4 DRAM misses** across a single dual-channel controller (`~40 GB/s`), triggering Pollaczek–Khinchine `M/G/1` queueing blowup $\mathbb{E}[W_q] \propto \frac{\rho}{1 - \rho}$;
2. Staging wrote and re-read an extra `900 KiB` per block (`+1.8 MB` cache/memory traffic).

`DietRelay8Cells` eliminates **both** causes:
1. **Zero L3 overflow (`10w = 18.85 MiB <= 20.00 MiB`):** `edges[0..m)` (`351–798 KiB`) stays 100% inside Comet Lake's distributed L3 (`> 600 GB/s` aggregate slice bandwidth across 10 ring stops). At `7.9–13.9 GB/s`/worker of L2$\leftrightarrow$L3 traffic (`63k–225k` L2 misses over `508–1,037 µs`), 8–10 workers consume `< 20%` of L3 ring bandwidth while generating **`0` extra DDR4 DRAM misses** (`479–831`/block = `0.034–0.059 B/B`).
2. **Zero staging buffer (`0 B` heap):** Every output byte is written directly into its final address in `out[0..n)`.

---

## 5. Coordinator Integration Recipe (~180 Safe-Rust Lines)

On top of the Coordinator's native diet (`OutQ16` + `Node16`-in-`edges` + `[u32; 3]` metadata coloring):
1. **In Phase 2 (`inline_words`):** Alongside `inline16: Vec<[u8; 16]>` and `lens: Vec<u8>`, maintain `word_len: Vec<u16>` (`+2.6–8.0 KiB`, L1d-resident) storing exact `word.len() as u16`, and pad `inline16`, `lens`, and `word_len` to `cap = len.next_power_of_two()` (`code_mask = cap - 1`) so safe-Rust slice indexing is statically bounds-check-free.
2. **In Phase 4 (`emit`):** When `!wide && m >= 1024`, call `emit_relay8` ([`verify_cycle10.rs:895-1197`](2026-11-21-wormhole-cycle10-floor-squeeze-and-handoff/verify_cycle10.rs#L895-L1197)):
   - Compute zero-bit cuts `((j ^ mark) & cm) == 0` at `shift = bits.saturating_sub(8)` ($S \in [129, 256]$ on stack);
   - Run **Phase 4a** (`Lane 0` direct emitter from `mark` + `Lanes 1..7` length-only scouts over `edges` and `word_len`, with $O(1)$ hop-over on `SCOUTED` cuts and mid-segment split handoff on `SCOUTING` cuts);
   - Run **Phase 4b** (`8` work-stealing lanes executing unconditional `16 B` sloppy stores `out[at..at+16]` with `0` boundary branches);
   - Run **$O(S)$ post-slop restoration** (`<= 16` exact bytes along `edges` per segment boundary, `< 0.5 µs`) + `repeat(n, out)`.
3. **End-to-End Verification Status:** `14,461` small/periodic/tight-cut/multi-cycle/multi-epoch/`wide`/fallback cases + 18-block (`15.45 MiB`, `CRC32 = 0x3e6968b1`) `BzReader::parallel_workers_with` stream at `W = 1, 2, 4, 8` verified byte-exact.
