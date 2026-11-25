# Ideal-Parts First Pass: Mathematics, Cost Bounds, and Pre-Run Matrix Predictions

- **Author / Role:** Antigravity (Neon, movement)
- **Date:** 25 November 2026
- **Prompt:** `origin/master:docs/journal/2026-11-24-algorithm-level/ideal-parts/antigravity/prompt-1.md` (at `origin/master` `9e500f89`)
- **Independence note:** Parts 1, 2, and 3 were written and frozen in full *before* fetching or reading MONODROMY's records 129–132 (`claude/sharp-meitner-cx0v78`) or journal §72. Part 4 was appended after reading them, without altering any prediction or claim in Parts 1–3.
- **Scope:** Mathematics and pre-registered predictions only. No `src/` edits, no engineering patches.

---

## Measured Baseline Anchor & Corpus Facts (All Numbers Traced)

Every empirical number used below comes from `origin/master` (`9e500f89`):

- **Workload (`wikidata ×4`):** 47.4 MB compressed, 838,800,784 bytes (838.8 MB) output, 1,724 blocks (431 distinct × 4; 932 Open blocks = 233 distinct × 4 [912 `open`, 20 `open_dense`], plus 792 tiny non-Open blocks [600 `skipped` median $n = 30$, 192 `column` median $n = 8$]). Sources: `docs/journal/2026-11-24-algorithm-level.md` §§57, 66; `math-targets/data/four-arm-report.txt` L3; `math-targets/data/character-stage1/report.txt` L361–364.
- **Symbol & retained-row volume:** 50,086,174 bytes of `u16` symbols across 431 distinct blocks = 25.04M symbols/copy ≈ **100.17M symbols** per decode (`four-arm-report.txt` L3; cf. §82's 24.98M symbols and 13.49M LUT lookups = **1.85 symbols/lookup** on 1× wikidata). Output-order codes: 26,567,581 `u32` codes = 106.27 MB across 233 distinct Open blocks → **106.27M retained rows** per decode (mean $m \approx 114{,}024$ per 900 KB Open block, $m/n \approx 0.127$). On the 6 admitted Wikidata blocks, $m_{16}/h_1 \in [2.105, 3.271]$ and Open IET steps $= 5{,}722\text{–}17{,}895$ ($0.101\text{–}0.339\,h_1$) (`research/neon-movement/T1-T2-LAZY-ACQ-AND-DEPTH2.md` §§1, 3).
- **Stage 1 per-activity breakdown (seconds/decode, unpinned, ledgers on; `math-targets/data/character-stage1/report.txt` L8–17, L32–41, L56–65, L166–175, L274–283):**

| $k$ | Wall (s) | Scan busy (`A2` / `A3` / `maint`) | Wkr busy | `M` (`A5`) | `C` (`A6a+b+c`) | `A7` (emit) | `R` (`A9`) | `D` (`copy+sink`) | `A0` (feed) |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1.2831 | 0.1728 (`0.0185 / 0.1445 / 0.0066`) | 1.2801 | 0.1324 (`10.3%`) | 0.5777 (`45.1%`: `0.2003+0.2160+0.1614`) | 0.4903 (`38.3%`) | 0.0760 (`5.9%`) | 0.0343 | 0.0100 |
| 6 | 0.2384 | 0.1832 (`0.0182 / 0.1537 / 0.0084`) | 0.2350 | 0.0242 (`10.3%`) | 0.1056 (`44.9%`: `0.0364+0.0393+0.0299`) | 0.0894 (`38.0%`) | 0.0151 (`6.4%`) | 0.0334 | 0.0110 |
| 8 | 0.1944 | 0.1850 (`0.0180 / 0.1541 / 0.0102`) | 0.1847 | 0.0193 (`10.4%`) | 0.0821 (`44.5%`: `0.0282+0.0304+0.0235`) | 0.0695 (`37.6%`) | 0.0133 (`7.2%`) | 0.0403 | 0.0123 |
| 10 | 0.1993 | 0.1903 (`0.0182 / 0.1567 / 0.0121`) | 0.1713 | 0.0174 (`10.2%`) | 0.0762 (`44.5%`: `0.0266+0.0276+0.0220`) | 0.0629 (`36.7%`) | 0.0140 (`8.2%`) | 0.0507 | 0.0140 |

- **Scan-profile untimed/timed (`measurements-scan-profile.tsv` L2, 7, 9, 11):** $k=1$: wall `1.3038 s` (`wkr 1.2965 s`); $k=6$: wall `0.2381 s` (`wkr 0.2339 s`, `scan 0.1853 s`); $k=8$: wall `0.1911 s` (`scan 0.1862 s`, `wkr 0.1786 s`, `starve 0.0119 s`); $k=10$: wall `0.2012 s` (`scan 0.1890 s`, `wkr 0.1670 s`, `starve 0.0341 s`).
- **Four-arm ratios (`measurements-four-arm.tsv` L2–13; unpinned, 2 s settle, old binary):** $k=1$: `B/A = 0.830`, `C/A = 0.996`, `D/A = 0.825`; $k=6$: `B/A = 0.850`, `C/A = 0.982`, `D/A = 0.830`; $k=10$: `B/A = 0.976`, `C/A = 0.806`, `D/A = 0.741` (`D/C = 0.920`). Stage 2a internal four-arm at $k=8$ (`character-stage2/report.txt` L80–82): `B/A = 0.967`, `C/A = 0.874`, `D/A = 0.747` (`D/C = 0.855`).
- **Matrix controls (`ideal-parts/design.md`):** 100 ms pre-spin on every CPU (`clock-mechanism/report.txt`: at $k=10$ pinned, `SPIN` median wall = **`0.196 s`** at 4.91 GHz, SD of log wall `1.12%`, `SPIN/S1 = 0.9493`) + scanner pinned to fastest core with sibling idle (`character-stage2b/analysis/link.tsv`: at $k=8$, `pinned/unpinned = 0.9598 [0.9306, 0.9899]`; stage 2b sibling sharing costs `+47.8%` at $k=8$, `+22.0%` at $k=4$).
- **Referee's currencies (`council/comparison.md` §1, Draft agreed formulation):**
  - $Q$: data-addressed (unpredictable / non-cursor) memory accesses;
  - $D$: dependent-chain depth (serial latency-carrying pointer or state chain);
  - $P$: streamed passes / streamed bytes normalised by block length $n$, under cursor budget $s_{\max} = |\Sigma| + O(1)$;
  - $S$: peak working state (bytes per thread).

---

## Part 1: Per Part, the Mathematics

### 1.1 Factor S — The Huffman Scan (`src/block.rs`, `src/huff.rs`, `src/bits.rs`; Target T4)

#### 1. Mathematical object
- **Input:** Bitstream $B \in \{0,1\}^L$ starting at stream offset $b_0$, encoding a sequence of variable-length blocks until `END_MAGIC` (`0x1772_4538_5090`). Each block $j$ begins with 48-bit `BLOCK_MAGIC` (`0x3141_5926_5359`), 32-bit CRC, 1-bit randomised flag, 24-bit `orig_ptr`, sparse 256-bit `seq_to_unseq` bitmap ($\alpha = n_{\text{in\_use}} + 2$), $n_g \in [2, 6]$, $n_{\text{sel}} \in [1, 18002]$, unary MTF-coded selector sequence $g_0, \dots, g_{n_{\text{sel}}-1} \in \{0,\dots,n_g-1\}$, delta-coded canonical code lengths $(\ell_{g,a})_{a < \alpha}$ defining $n_g$ prefix-free bijections $T_g$, and payload bits $B_{\text{pay}}^{(j)} = \operatorname{enc}_{T_{g_{\lfloor t/50 \rfloor}}}(y_t)_{t=0}^{M_j-1}$ with $y_{M_j - 1} = \text{EOB}$.
- **Output:** For each block $j$, the symbol stream `Symbols { crc, orig_ptr, seq_to_unseq, syms[0..M_j], eob, max_len }` and the exact end bit position $b_{\text{end}}^{(j)} = b_0^{(j+1)}$.
- **Dependency structure:** Two nested serial chains:
  1. **Cross-block boundary chain:** $b_0^{(j+1)} = b_{\text{end}}^{(j)}$. Without scanning for `BLOCK_MAGIC`, block $j+1$ cannot begin until block $j$ reaches `EOB`.
  2. **Within-block coupled bit/group chain:** For $t = 0, \dots, M_j - 1$, $b_{t+1} = b_t + \ell_{g_{\lfloor t/50 \rfloor}}(y_t)$ where $y_t = T_{g_{\lfloor t/50 \rfloor}}(B[b_t \dots])$. The next bit offset depends on the length of the previous codeword, **and** the active table $g_{\lfloor t/50 \rfloor}$ depends on the exact cumulative symbol index $t$ modulo `GROUP = 50`.

#### 2. Current cost (`src/block.rs`, `src/huff.rs`) and lower bounds
- **Current implementation (`src/huff.rs:36-443`, `src/block.rs:81-277`):**
  - 6 canonical tables in `ScanScratch` (`54 KiB` total: `18 KiB` selectors + `6 × (4 KiB root 10-bit LUT + 2 KiB SubLut)`), L1d/L2-resident. Root LUT encodes up to **2 symbols per entry** (`CNT_SH`, `SYM1`, `SYM2`); `decode_group_win` decodes 50-symbol groups in unrolled batches of `STEPS = 5` lookups from a 264-byte slice window.
  - $Q$: $Q_{\text{DRAM}} = 0$; L1d-resident LUT lookups $Q_{\text{L1}} = M / \bar{c} \approx 0.54 M$ ($\bar{c} = 1.85$ symbols/lookup on wikidata: 84% pairs, 15% singles, 1% slow path, §82). Plus table builds: $O(n_g \cdot (2^{10} + \alpha))$ per block (`A2` = 18.2 ms, 10% of scanner time).
  - $D$: Strictly serial chain of depth $D = \sum_j M_j / \bar{c} \approx 54.1\text{M}$ lookups across the entire wikidata ×4 stream. Measured decode cost: **1.41 ns/symbol** at $k=1$, **1.51 ns/symbol** at $k \ge 3$ (`~7.4 cycles/symbol`, `~13.7 cycles/lookup` at 4.91 GHz; §57, §82, `scan-profile-report.txt`).
  - $P$: $P = 1$ read pass over compressed input (`47.4 MB` = `0.056 B/out-byte`) + $1$ write pass of `syms: Vec<u16>` (`200.3 MB` = `0.239 B/out-byte`), plus `maint` (`0.0066–0.0121 s`, buffer-seam copying and `syms` zero-resizing in `block.rs:222`).
  - $S$: `54 KiB` `ScanScratch` + pooled `syms` buffers (up to `MAX_SYMS × 2 B = 1.8 MB` capacity each, avg `~116 KB` live).
- **Lower bounds:**
  - *Work bound `[PROVED]`*: Under any word-RAM decoder that emits the explicit symbol stream, reading the compressed bitstream and writing $M$ symbols requires $\Omega(|B|/w + M/c_{\max})$ operations, where $c_{\max}$ is symbols decoded per instruction/lookup.
  - *Single-thread serial LUT throughput bound `[ARGUED]`*: On wikidata, mean payload code length is $\bar{\ell} \approx (47.4\text{ MB} \times 8 - \text{headers}) / 100.2\text{M} \approx 3.5\text{–}3.7\text{ bits/sym}$. An L1d-resident root LUT ($\le 11$ bits, keeping 6 tables $\le 48\text{ KiB}$) holds at most $\lfloor 11 / 3.5 \rfloor \le 3$ symbols in expectation; `#870` (§82) proved empirically that 3-symbol 64-bit entries (8 KiB/table) cut lookups by only 12% while *regressing* wall time due to build cost, group-tail mispredicts, and spills, and simulated 12-bit tables yield only −16% lookups at 2–4× table size. With 1 load-to-use + shift + count chain costing $\ge 6\text{–}7$ cycles/lookup (`~1.2–1.4 ns/lookup`, `~0.7–0.8 ns/sym` hardware limit, ~1.2–1.3 ns/sym with group control/writes), **serial single-stream LUT decoding is within ~15–20% of its microarchitectural ceiling**.
  - *Cross-block parallel depth bound `[PROVED]`*: Because blocks are independent once located via the 48-bit `BLOCK_MAGIC` across all 8 bit alignments, locating all $J$ block starts takes one embarrassingly data-parallel SIMD pass of $O(|B|/W_{\text{SIMD}})$ work (`~1–2 ms` on 47.4 MB at ~25–40 GB/s), reducing serial depth $D$ from $\sum_{j=1}^J 0.54 M_j$ to $O(|B|/W_{\text{SIMD}} + \max_j 0.54 M_j)$ (`~150 µs` max block, a **~1000× reduction in $D$**). False-positive magic probability on uniform bits is $\le 8|B| \cdot 2^{-48} \approx 1.3 \times 10^{-6}$ on 47.4 MB, and determinism/exactness is certified by verifying $b_{\text{end}}^{(j)} = b_0^{(j+1)}$ on adjacent blocks (falling back to serial continuation from $b_{\text{end}}^{(j)}$ on any mismatch).
  - *Within-block parallel depth bound `[ARGUED]`*: Treating a block's payload as an FSM over state `(bit_offset_mod_codeword, sym_idx mod 50, group_idx)`, bit-level self-synchronisation collapses bit offsets within $O(\bar{\ell})$ bits, **but** the 50-symbol selector schedule breaks pure bit self-sync: a $\pm 1$ symbol-count error shifts every downstream group boundary by $\pm 1$ symbol, causing the wrong Huffman table to be invoked unless adjacent selectors are identical (in `scan-profile-report.txt`, 438 switches in 1,163 selectors/block ⇒ 62.3% of group boundaries keep the same table). Thus exact within-block speculative composition must carry up to 50 count-mod states (or speculate across constant-selector runs) and pays $\ge 2\text{–}4\times$ extra CPU work `[CONJECTURED]`.

#### 3. Candidate reformulations
1. **Cross-block magic-indexed parallel Huffman scan fused into the lazy worker pool**
   - *Levers pulled (`levers.md`):* **Lever 4** (dispatch/routing: workers pull raw bit-slices rather than decoded `Symbols`), **Lever 4b** (ownership: raw compressed window owned by bounded input chunk ring, eliminating the 200 MB `syms` inter-thread queue), **Lever 5** (state space: $k$ independent scanner states in parallel instead of 1 serial scanner).
   - *Predicted cost:* $Q_{\text{L1}}$ unchanged (`0.54 M`, distributed over $k$ cores); $D$ drops by **$k\times$** (`[PROVED]`); $P$ adds 1 SIMD magic-scan pass (`~0.056 B/out-byte`, ~`1–2 ms` on 47.4 MB) while **deleting** the `200 MB` inter-thread `syms` write/read handoff (each worker decodes `syms` hot into its own L2 cache and immediately runs `M`!); $S$ adds `54 KiB` `ScanScratch` per worker (negligible vs 58 MB RSS) plus a bounded compressed lookahead buffer (`~2–4 MB`). Expected wall at $k=8$–10: removes the serial scanner ceiling completely (shifting balance to total machine work / $k$: ~`1.58 s / 8 ≈ 0.198 s` before worker improvements, or with worker improvements scaling cleanly to 10 cores without starving!) `[ARGUED]`.
   - *Needs from neighbours:* Buffered/chunked lookahead from the feeder (`A0`, ~2–4 MB ahead of reader) + lightweight deterministic end-bit chain verifier (`b_end[j] == b_start[j+1]`) before publishing block $j+1$ as valid.
   - *Prior art:* `pbzip2`, `lbzip2`, `rapidgzip`, mibz2's `C-37` (achieved `0.883` with scheduling/memory regressions, journal §44), and MONODROMY record 122 (cross-block parallel parse). **Revisiting `C-37` because** stage 1/2b revealed why earlier parallel scans regressed: oversubscription (`20` threads on 10 cores) + eager buffers + no admission window. Running $k = 8\text{–}9$ lazy end-to-end workers (with no dedicated serial scanner thread stealing a physical core!) avoids both SMT contention and RSS blow-up.
2. **Within-block FSM transition-function composition (self-synchronising parallel Huffman)**
   - *Levers pulled:* **Lever 5** (state space: chunk transition functions composed by parallel prefix).
   - *Predicted cost:* `[CONJECTURED]` $2\text{–}4\times$ CPU work per symbol due to the 50-symbol selector-count obstruction; reduces $D$ within a single block by $p\times$, at significant CPU/energy cost. Only valuable on single-block inputs where cross-block parallelism has $J = 1$; strictly dominated by candidate (1) on multi-block corpora (`wikidata ×4` has $J = 1{,}724$).
   - *Needs from neighbours:* Whole block payload in memory before decode; offline corpus measurement of self-sync length and constant-selector run lengths (`leads/structure-and-function.md`).
   - *Prior art:* Klein & Wiseman (2003); Mytkowicz, Musuvathi & Schulte (ASPLOS 2014); `leads/structure-and-function.md`.
3. **Serial plumbing + batched bit count (PRs #846, #857, #879)**
   - *Levers pulled:* **Lever 1** (flow map $F$).
   - *Predicted cost:* Saves ~`10–15 ms` (`6–8%` of scanner time, §§79, 82); leaves serial bottleneck topology unchanged.

#### 4. Is S spent?
- **Single-thread serial LUT arithmetic: YES, SPENT.** At `1.85` symbols/lookup (`1.51 ns/sym`, §82), 3-symbol LUTs regressed (`#870`), and remaining serial plumbing is ≤ 8% (`~12–15 ms`).
- **Serial-scanner topology: NOT SPENT.** Removing the single-core serial constraint via cross-block parallel scanning is the single highest-leverage structural move above $k = 8$, both for S's own room and to unmask C, E, M, and R.

---

### 1.2 Factor E — Emission (`src/open.rs`: `emit`, `emit_bidirectional`; Target T1)

#### 1. Mathematical object
- **Input:** Open's constructed return system on $m$ retained rows:
  - Packed retained-edge array `edges[0..m]` (`u32`, storing `(code << bits) | (prev ^ next)` for the bidirectional XOR path when `m >= 65_536 && !wide`, or `(code << bits) | next` plus optional `extra[0..m]` otherwise);
  - Starting anchor `(mark, predecessor)` (and `bits`, target length $n$);
  - Word dictionary `(inline16[0..W], lens[0..W], spelling[0..S_sp])` where words of length $\le 16$ live inline (`~92–95%` of words on wikidata, `T1-T2-LAZY-ACQ-AND-DEPTH2.md` §1) and longer words store `(start, end)` offsets into `spelling`.
- **Output:** BWT-inverted byte array `out[0..n]` equal to $\prod_{t=0}^{\ell-1} W(\operatorname{code}(j_t))$ along the retained cycle $j_{t+1} = R(j_t)$ from $j_0 = \text{mark}$, repeated periodically to length $n$ if the cycle closes before $n$ bytes.
- **Dependency structure:** Two coupled sub-tasks inside one loop (`src/open.rs:919-986`):
  1. **Cycle traversal (code acquisition):** Two independent serial pointer-chasing chains (`front` forward from `mark`, `back` backward from `predecessor`), each of length $m/2$: $j_{t+1} = (\text{edges}[j_t] \mathbin{\&} \text{mask}) \oplus j_{t-1}$. Every load of `edges[j_{t+1}]` has a hard memory-latency data dependency on `edges[j_t]`.
  2. **Word spelling & output write:** Given `code = edges[j_t] >> bits`, fetch `len = lens[code]` and `inline16[code]` (or `spelling[start..end]`) and store 16 bytes at `out[lo..lo+16]` (forward) or shifted 16 bytes at `out[hi-16..hi]` (backward), advancing `lo += lf` / `hi -= lb`. The cursor update `lo += lf` is a 1-cycle ALU dependency, NOT a memory chase; `inline16` + `lens` is `~18–34 KB` per block (L1d/L2-resident).

#### 2. Current cost (`src/open.rs:869-986`) and lower bounds
- **Current implementation:**
  - $Q$: $m$ unpredictable random reads of `edges[0..m]` (`4m ≈ 456 KB/block`, spilling L2 into L3!) + $m$ L1d/L2 table lookups into `inline16`/`lens` (`~18–34 KB`). On the 6 admitted Wikidata blocks, $m = 2.105\text{–}3.271\,h_1$ (`T1-T2-LAZY-ACQ-AND-DEPTH2.md` §3).
  - $D$: Two parallel memory-dependent chains of depth $m/2$ each (`MLP = 2` via XOR links).
  - $P$: $1$ sequential write of `out[0..n]` (`838.8 MB` = `1.0 B/row`) + $m$ random 4-byte reads of `edges` (`~425 MB` across 932 Open blocks).
  - $S$: `edges` (`4m ≈ 456 KB`) + dictionary (`~35–60 KB`) + `out` (`900 KB`) $\approx 1.5\text{ MB}$/worker.
  - **Measured time decomposition (CRITICAL for Part 2!):** At $k=1$, `A7` totals `0.4903 s` (`38.3%` of worker time, `~4.61 ns` per retained row across 106.27M rows). Four-arm arm B (`supplied-codes`, `measurements-four-arm.tsv` L3) removes **only the `edges` pointer chase**, saving `0.218 s` (`17.0%` of wall = **`44.4%` of `A7`**, `~2.05 ns/row`). The remaining **`0.272 s` (`21.3%` of wall = `55.6%` of `A7`, `~2.56 ns/row`)** is **word lookup + branching + `u128` backward shifting + 16-byte stores to `out`**!
- **Lower bounds:**
  - *Output write bound `[PROVED]`*: Writing $n$ bytes requires $\ge n / W_{\text{store}}$ stores (at 16 B/store, $\ge n/16$; with mean Open word length $n/m \approx 7.9$ bytes, emitting one word per store requires $m$ stores).
  - *Rigid-interval calculus floor G1 (`[PROVED]` inside $\mathcal{R}$, M103; `council/comparison.md` §3)*: $c_1 \ge h_1$. Open pays $m \approx 2.1\text{–}3.3\,h_1$, leaving a $2\text{–}3\times$ gap to $h_1$ inside $\mathcal{R}$ (and no unconditional execution-model lower bound $\Omega(h_1)$ on $Q$ is proved, `council/comparison.md` §3).
  - *Chase-latency bound at MLP $= L$ `[ARGUED]`*: If an emission table of size $4m > \text{L2}$ (`256 KiB` on i9-10900K) is traversed along $L$ independent chains, memory latency alone floors chase time at $\frac{m}{L} \cdot t_{\text{L3}} \cdot p_{\text{L2-miss}}$. At $m = 106.3\text{M}$, $L = 2$, $t_{\text{L3}} \approx 8\text{–}10\text{ ns}$, $p_{\text{L2-miss}} \approx 0.45$, $\frac{m}{2}\cdot 9\text{ ns}\cdot 0.45 \approx 0.215\text{ s}$ — matching the measured `0.218 s` chase time almost to the millisecond!
  - *Word-spelling bound `[ARGUED]`*: At `106.3M` words, 1 `inline16` load + 1 unaligned 16-byte store + 1 cursor add is ~3–4 µops (~1.0–1.3 cycles/word at 4-wide retire = `~0.25 ns/word` = `~27 ms` at $k=1$, or `~80–100 ms` with L1/L2 `inline16` misses and `out` write bandwidth). Current `272 ms` (`~2.56 ns/word` = `~12.5 cycles/word`) is **~3× above** pure word-copy bandwidth because of (a) `u128` dynamic byte-shifting on every backward word (`open.rs:963`), (b) per-step `lf != LONG && hi - lo >= INLINE` branching, and (c) `inline16` + `lens` indirection.

#### 3. Candidate reformulations
1. **Closure-preserving retained-set contraction (`M-17` / `CL1` valley cut on $M_{16}$ with compact inline payloads)**
   - *Levers pulled:* **Lever 5** (state space: compose 1-step edges into multi-step valley-to-valley transitions on $m' \approx 0.35\text{–}0.45\,m \approx 0.71\text{–}1.17\,h_1$ vertices).
   - *Predicted cost:* Cuts emission $Q$ from `2.1–3.3 h₁` to `0.71–1.17 h₁` (`[PROVED]` on 6/6 Wikidata blocks, `T1-T2-LAZY-ACQ-AND-DEPTH2.md` §3). **Unresolved closure obstacle:** flat composed words inflate dictionary state $S_{\text{pay}}$ from `16–34 KB` to `2.4–2.6 MB` (blowing out L2!), whereas DAG payloads keep `S_pay = 34 KB` but re-incur `4.2–6.6 h₁` random payload reads. Viable only if **bounded-length inline fusion** (e.g., contracting *only* when combined word length $\le 16$ or $\le 32$ bytes, keeping the dictionary compact) captures a large fraction of `m -> m'` without blowing up `S_pay` `[CONJECTURED]`.
   - *Needs from neighbours:* Construction (`C`) builds the valley/fused edges directly in $P = 2$ streaming passes (`B_P` cursor merge).
   - *Prior art:* `CL1` (`research/neon-movement/CL1-MIRROR-FACTORED-CLOSURE.md`), `M-17` (`T1-T2-LAZY-ACQ-AND-DEPTH2.md`), SA-IS LMS valleys; lazy thin-run contraction `S*_1` was already killed (`T1-T2-LAZY-ACQ-AND-DEPTH2.md` §2: $B_P$ explodes $4.1\text{–}8.7\times$).
2. **Higher-MLP emission ($L \ge 4$ independent chains, or decoupling chase from copy)**
   - *Levers pulled:* **Lever 1** (flow map $F$: cuts chase depth $D$ from $m/2$ to $m/L$) + **Lever 5** (anchoring $L$ cycle entry points without a prior walk).
   - *Predicted cost:* If $L = 8$ chains can be started without paying a full walk, chase latency drops $4\times$ (`0.218 s → ~0.06 s` at $k=1$, `~12%` of worker time) `[ARGUED]`. **Obstacle:** Finding $L$ cycle anchors with known text offsets without walking the cycle is the cycle-offset obstruction (`S7`, `M108`). Wait: unlike raw LF where segment lengths are unknown, emitting $L$ *unpositioned* segments into $L$ small buffers and concatenating them in cycle order works **only if** the $L$ start vertices are guaranteed to partition the single cycle cleanly (easily done by randomly/regularly picking $L$ marked retained vertices, walking until hitting any marked vertex — `walk_back_room`'s exact multi-lane trick in `src/bwt.rs`, paying 1 extra segment-copy pass unless fused with RLE1!). **Wait:** fusing segment stitching with single-pass RLE1+CRC (`A9`) means the segment concatenation pass IS the RLE1+CRC pass (`ΔP = 0` extra passes!) `[CONJECTURED]`.
   - *Needs from neighbours:* `R` (`A9`) consumes $L$ ordered pre-RLE1 segments instead of 1 contiguous `out` buffer; `C` (`A6c`) can drop `wire_xor` (zero scattered RMWs!) if $L$-lane forward chasing replaces bidirectional XOR chasing!
   - *Prior art:* `bwt::walk_back_room` (`src/bwt.rs`), multi-lane BWT inversion (Kärkkäinen / `lbzip2` / mibz2's own `walk/lanes` branches). **Revisiting multi-lane on Open's retained graph (`edges[0..m]`) because** Open's $m \approx 0.127 n$ is $8\times$ smaller than $n$, AND if $L$-lane forward walk eliminates `wire_xor`'s `161 ms` scattered RMWs in `A6c` *simultaneously*, it pays back on BOTH `C` and `E`!
3. **Attacking the unremoved `55.6%` word-spelling half of `A7` (fast loop + pre-aligned backward `inline16` + 2-word fusion)**
   - *Levers pulled:* **Lever 1** (flow map $F$) + **Lever 5** (right-aligned backward table or forward-only $L$-lane emit eliminating `u128` shifts).
   - *Predicted cost:* `[ARGUED]` Removing dynamic `u128` shifts on the backward lane + hoisting `hi - lo >= 32` out of the hot loop + packing `(len, word)` cleanly can cut the `0.272 s` spelling half by ~`20–30%` (`~55–80 ms` at $k=1$, `~4–6%` of worker time), completely orthogonal to `edges` chasing.
   - *Needs from neighbours:* Nothing (or right-aligned copy of `inline16` from `C`, `~18 KB`).
   - *Prior art:* Carbon cycle 2 `U16` emit (`src/open.rs:863`).

#### 4. Is E spent?
- **NO.** `A7` is `38.3%` of worker time (`0.490 s` at $k=1$). Even `supplied-codes` leaves `21.3%` of worker time (`0.272 s`) untouched in word spelling, and the `edges` chase still pays `2.1–3.3 h₁` L3 reads at MLP $= 2$. Crucially, replacing bidirectional XOR chasing with $L$-lane forward chasing on `edges` attacks **both `E` (`A7`) and `C` (`A6c`) together**!

---

### 1.3 Factor C — Construction (`src/open.rs`, `src/open/endpoint.rs`, `src/open/outq.rs`; Target T2)

#### 1. Mathematical object
- **Input:** Capped run records `records: &[u16]` ($r_{\text{rec}}$ records of `(byte << 8 | len)`, $\text{len} \le 127$, $\sum \text{len} = n$) and marked row $p \in [0, n)$, with thin-run threshold `LIMIT = 16`.
- **Output:** Open's return system: `edges[0..m]` (XOR-linked retained permutation + packed word code), `(mark, predecessor, bits)`, and word dictionary `(inline16, lens, spelling)`.
- **Dependency structure:** Three strictly ordered sub-phases (`A6a → A6b → A6c`):
  - **`A6a` (Pass 1 — classification, counting & `OutQ16` queueing, `15.5%` of worker time, `0.200 s` at $k=1$):** Re-merges 127-capped records into maximal runs (`maximal()`), splits the thick run containing $p$, tallies `thin[256]`, `thick[256]`, `thick_rows[256]`, writes $n_s$ thick strips to `edges[0..3*ns]`, and pushes each thin run's retained rank interval $[m, m+w)$ into byte $c$'s 16-bit chunked linked queue stored inside `out` (`OutQ16`, `src/open/outq.rs:122-175`). Streaming fold over `records` with 256 byte-indexed output cursors.
  - **`A6b` (IET setup, elimination & dictionary, `16.7%` of worker time, `0.216 s` at $k=1$):** Computes exact F-bucket starts `first[256]` (requires `A6a` finished: Theorem T5.1!), counting-sorts thick strips into `image_order`, builds the doubly-linked interval exchange on $\le 3n_s + 3$ nodes inside `edges`, runs right-end endpoint elimination (`endpoint.rs`, with spiral contraction when $\max w > 32{,}768$) producing $O(n_s)$ rules, walks bottom intervals to emit `bands`, and hashes/deduplicates every band's expanded word (`Dictionary::code`, `endpoint.rs:516-590`) into `inline16`/`lens`/`spelling`.
  - **`A6c` (Retained-edge XOR wiring, `12.7%` of worker time, `0.161 s` at $k=1$):** Zeroes `edges[0..m]`, then iterates `a in 0..256` and each band of byte `a`, streaming retained ranks `v` from `OutQ16` and executing `edges[u] ^= pack | v; edges[v] ^= u` (`src/open/outq.rs:261-262`). Every retained row $u \in [0, m)$ executes **1 sequential RMW (`edges[u]`) and 1 scattered random RMW (`edges[v]`)**!

#### 2. Current cost (`src/open.rs`, `outq.rs`, `endpoint.rs`) and lower bounds
- **Current implementation:**
  - $Q$:
    - `A6a`: $0$ random DRAM accesses (`256` active `OutQ16` chunk write tails $\approx 16\text{ KB}$ of active cache lines, L1d/L2-resident).
    - `A6b`: `5.7k–17.9k` (`0.10–0.34 h₁`) local linked-list updates in `edges` (`Narrow` 16 B/node, ~160–480 KB) + `~3k–15k` `Dictionary::code` direct-mapped hash probes (`pairs` + `contents`) and recursive `inline_word` expansions. **Key diagnostic:** `932` Open blocks × `~11k` IET steps $\approx 10.3\text{M}$ steps, yet `A6b` takes **`216 ms` at $k=1$ (~1.06 billion cycles $\approx 103$ cycles per IET step!)**, showing that node setup, band emission, rule expansion, and double-hashing (`pairs` + `contents` + `spelling_hash`) dominate `A6b`, NOT the raw IET arithmetic!
    - `A6c`: **`m` (`2.1–3.3 h₁`, `106.3M` total across wikidata ×4) SCATTERED RANDOM RMWs (`edges[v] ^= u`)** across `4m ≈ 456 KB` (L3-resident), solely to support 2-lane XOR emission!
    - **Total $Q(C)$:** $\approx m + 0.25\,h_1 \approx 2.35\text{–}3.55\,h_1$ — **as many random L3 accesses as emission `A7` itself!**
  - $D$: `A6a`: sequential over $r_{\text{rec}}$; `A6b`: serial IET depth `6k–18k`/block; `A6c`: $D = O(1)$ per row (independent RMWs, ROB/MLP-limited).
  - $P$: $1$ read of `records` (`~180 MB`) + $1$ bucketed write of `OutQ16` (`2m ≈ 213 MB`) + $1$ read of `OutQ16` (`213 MB`) + `edges.fill(0)` (`425 MB`) + $1$ sequential RMW (`425 MB`) + $1$ scattered RMW (`425 MB`) = **`~1.88 GB` of memory traffic** (`~2.24 B/out-byte`)!
  - $S$: Multiplexed in `out` (`OutQ16`) and `edges` (`arena_words`), peak `~1.5 MB`/worker ($\le 40\text{ B/row}$ worst-case, `src/open/b5.rs:329`).
- **Lower bounds:**
  - *Histogram / pass bound `[PROVED]`*: By Theorem T5.1 (`T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md`), changing a single byte at the end of a block shifts `first[a]` and exact `LF(x)` on `15–50%` of rows; thus any exact construction requires $P \ge 1$ full pass over the run stream before any destination coordinate is finalised, and $\Omega(r)$ work.
  - *Strip elimination bound `[ARGUED]`*: Eliminating $n_s$ thick strips requires $\Omega(n_s)$ operations (`~2k–6k` strips/block). Open's IET takes `~2–3×` $n_s$ steps, near-optimal in step count, but **~5–10× above optimal in cycles per step** due to `A6b` setup/hashing overhead.
  - *Wiring bound `[PROVED]`*: Writing an explicit $m$-entry unidirectional `edges` array requires $m$ **sequential** writes (`Q = 0` random accesses! Look at `outq.rs:327-344`, `Reader::wire`, which is pure sequential!). Paying $m$ **scattered random RMWs** (`edges[v] ^= u`) is **NOT** a lower bound of construction — it is purely the bill for XOR bidirectional emission!

#### 3. Candidate reformulations
1. **Deforestation of `M` (`A5`: `runs::front`) + `C` Pass 1 (`A6a`: `Pass::take` / `OutQ16`)**
   - *Levers pulled:* **Lever 1** (flow map $F$) + **Lever 5** (state space: Bird–Meertens fusion eliminates the intermediate 127-capped `Vec<u16>` run stream!).
   - *Predicted cost:* Right now `src/runs.rs` computes each maximal run `(c, total)`, **chops** long runs into `CAP = 127` records into `f.runs` (~180 MB), and counts `counts[256]`; then `src/open.rs:119` (`maximal`) reads all 180 MB back, **re-merges** them into maximal runs, splits at `LIMIT = 16`, and recounts `thin`/`thick_rows`! Fusing `front_avx2` directly into `Pass::take` removes `1` full streaming pass (`-360 MB` traffic), all `CAP = 127` chop/merge loops, and duplicate histogramming. `[ARGUED]`: saves ~`45–60%` of `A6a` + ~`15–20%` of `A5` = **~`9–12%` of total worker time** (`~115–155 ms` at $k=1$) with $\Delta Q = 0$ and lower RSS.
   - *Needs from neighbours:* Early Open admission predicate on `syms.len()` (`takes_runs` + `Cap::dense`, falling back to `runs::front` only if `over` trips; 932/932 full wikidata blocks pass).
   - *Prior art:* `tracks/cross-stage.md` notes generic Huffman+MTF+construction fusion failed; **worker-side `A5+A6a` fusion** was identified in `T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md` §2 and remains unbuilt.
2. **Eliminating `A6c`'s $m$ scattered RMWs (paired with $L$-lane forward emit, OR software-prefetched/bucketed `wire_xor`)**
   - *Levers pulled:* **Lever 1** (flow map $F$) + **Lever 5** (representation of `edges`: unidirectional `u32` instead of XOR-linked `u32`, paired with $L$-lane forward emit).
   - *Predicted cost:*
     - *Path (2a) Unidirectional `wire` (`outq.rs:327`) + $L$-lane forward emit:* Eliminates `edges.fill(0)` (`425 MB`) AND all $m = 106.3\text{M}$ scattered RMWs (`edges[v] ^= u`), cutting `A6c` from `161 ms` (`12.7%`) to `~35–45 ms` (`~3%`) — saving **~`9–10%` of total worker time** (`[ARGUED]`), provided $L$-lane forward emit matches or beats 2-lane `emit_bidirectional`!
     - *Path (2b) Keeping `wire_xor`, adding lookahead prefetch on `OutQ16`'s `v` stream (or L2-tiled radix buffering):* Since `v` is read sequentially from `OutQ16`, `edges[v_{i+16}]` can be prefetched 16 steps ahead (`[ARGUED]`, cuts `A6c` by ~`30–45%` = `~4–6%` of worker time, zero change to `E`).
   - *Needs from neighbours:* Path (2a) needs $L$-lane forward emit in `E` (+ segment-aware `R`); Path (2b) needs nothing from neighbours.
   - *Prior art:* `Reader::wire` already exists in `src/open/outq.rs:327`; XOR bidirectional linking was added for 2-lane emit without re-measuring whether $L$-lane forward emit + zero-RMW `wire` dominates the joint `(A6c + A7)` sum.
3. **Streamlining `A6b` dictionary construction (skipping content-hash dedup / direct rule-id indexing)**
   - *Levers pulled:* **Lever 1 + 5** (avoid hashing every band's expanded spelling when `pairs` misses and word counts are small: `inline16` holds only `647–1,550` words per block = `16–34 KB`!).
   - *Predicted cost:* `[CONJECTURED]` Profiling `A6b` (`216 ms` at $k=1$, `16.7%` of worker time!) will split IET elimination vs `Dictionary::code` hashing + rule expansion; if hashing/expansion is $> 50\%$ of `A6b`, direct `(rule_id, byte)` indexing without `contents` string-hashing (allowing a few hundred extra duplicate `inline16` entries, still `< 48 KB`!) can save **~`5–8%` of worker time**.
   - *Needs from neighbours:* Slightly larger `inline16` table (`~1.5×`, still L1/L2-resident) in `E`.
   - *Prior art:* Direct-mapped `pairs` + `contents` cache in `src/open/endpoint.rs:506-590`.

#### 4. Is C spent?
- **NOT SPENT — C IS THE LARGEST UNTOUCHED WORKER OPPORTUNITY IN THE DECODER.** At **`45.1%` of worker time (`0.578 s` at $k=1$)**, C is larger than all of emission `A7` (`38.3%`) and **2.6× larger** than `supplied-codes` (`17.0%`). Unlike `E`'s chase (constrained by G1/closure), C's three sub-parts (`A6a = 15.5%`, `A6b = 16.7%`, `A6c = 12.7%`) each carry concrete, non-asymptotic structural waste (redundant 127-cap pass, ~100 cycles/IET-step dictionary overhead, and $m$ scattered RMWs).

---

### 1.4 Factor M — MTF/RLE2 and Run-Record Formation (`src/runs.rs`, `src/mtf.rs`; Target T3)

#### 1. Mathematical object
- **Input:** Decoded symbol stream `syms[0..M]` (`0 = RUNA`, `1 = RUNB`, $2 \le s < \text{eob}$ literals, $\text{eob} = \alpha-1$) and alphabet map `seq_to_unseq[0..256]`.
- **Output:** Capped run records `runs: Vec<u16>` (`byte << 8 | len`, $\text{len} \le 127$) of the BWT last column $L$, plus `counts[256]`, `per[256]`, and $n$ (or plain `L[0..n]` on tiny column-route blocks).
- **Dependency structure:**
  - *RLE2 decoding:* Runs of `RUNA`/`RUNB` between adjacent literals are purely local bijective base-2 integers ($\le 21$ bits). Once the `lit` and `one` bitmaps are built (embarrassingly parallel SIMD map), every run length is independent ($D_{\text{RLE2}} = O(1)$).
  - *MTF state:* Left-to-right serial action in the symmetric group $S_{|\Sigma|}$: for each literal $k = s - 1 \in [1, 255]$, emit $c = \pi(k)$ and update $\pi \leftarrow \pi \circ \text{cyc}_k$. Serial depth $D = r_{\text{lit}}$ ($\approx 55\text{–}60\text{M}$ literals across wikidata ×4).

#### 2. Current cost (`src/runs.rs:421-690`) and lower bounds
- **Current implementation (`front_avx2`):**
  - Pass 1 (AVX2, 32 symbols/iter): writes `lit` and `one` bitmaps (`m/64` `u64` words each).
  - Pass 2 (literal loop): iterates set bits of `lit`, extracts RLE2 count via `one` bit-extract, updates 2-level MTF list (`f0, f1` 32-byte XMM register front via 4 `pshufb` + 2 `por` using `STEPS[32]` (`2 KiB`); $k \ge 32$ calls AVX2 `second()` over 224-byte stack `tail`), updates `cnt[256]` (`u64` packed rows|recs), writes `u16` run records.
  - $Q$: $Q_{\text{DRAM}} = 0$ (all tables `STEPS`, `tail`, `cnt` total `< 4.5 KiB`, 100% L1d-resident!).
  - $D$: Serial MTF permutation chain of depth $r_{\text{lit}}$ (`~55–60M` literals). Measured `A5` = `0.1324 s` at $k=1$ (`10.3%` of worker time, `~1.32 ns/symbol` or `~2.2 ns/literal` = `~10.8 cycles/literal` at 4.91 GHz).
  - $P$: $2$ reads of `syms` (full AVX2 bitmap scan + literal gather `syms[q]`) + $1$ write of `runs` (`~180 MB`), total `~580 MB` traffic (`~0.69 B/out-byte`).
  - $S$: `~250 KB`/worker (`runs` + `lit`/`one` + `4.5 KiB` L1 scratch).
- **Lower bounds:**
  - *Work bound `[PROVED]`*: Must read all $M$ input symbols (`2M` bytes) and execute $r_{\text{lit}}$ MTF state transitions.
  - *Single-thread serial latency bound `[ARGUED]`*: Updating `(f0, f1)` for $k < 32$ has a minimum 2-cycle loop-carried dependency (`pshufb` + `por`), plus `lzcnt`/`blsr` + `syms[q]` load ($\ge 4\text{–}5$ cycles/literal floor = `~0.9–1.0 ns/literal` $\approx 55\text{–}60\text{ ms}$ at $k=1$, leaving ~`2×` headroom in current `132 ms`).
  - *Parallel depth bound `[PROVED]`*: Associative semidirect monoid $\mathcal{M}_{\text{MTF}} = (\mathbb{N}^\Sigma \times \mathbb{Z}_{\min}^\Sigma \times \mathbb{Z}_{\max}^\Sigma \times \mathbb{N}) \rtimes S_\Sigma$ (`E1-CROSS-STAGE-MTF-MONOID.md`, Theorem E1.1) reduces within-block depth to $O(\log G)$ at $O(K_g)$ composition work per chunk ($K_{50} \approx 24\text{–}67$). Useful only for within-block parallelism, not needed when 1,724 blocks saturate $k$ workers.

#### 3. Candidate reformulations
1. **Fuse `M` (`A5`) directly into `C` Pass 1 (`A6a`)** — *Levers 1 + 5* (see §1.3 Candidate 1). Eliminates `runs: Vec<u16>` write/read and `CAP = 127` splitting; saves ~`15–20%` of `A5` on top of ~`50%` of `A6a` (`[ARGUED]`).
2. **2-literal composed MTF transitions (`16×16` = `4 KiB` pair-shuffle table)** — *Levers 1 + 5*: On `(LIT, LIT)` pairs with $k_1, k_2 < 16$ (`34–52%` of adjacent pairs on wikidata, `T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md` §1), composing two 16-byte shuffles into one cuts the loop-carried `f0` dependency depth by ~30–40% (`[ARGUED]`, ~`15–20%` of `A5` = `~1.5–2%` of worker time).
3. **Selector-guided MTF fast-paths (zero-cost T5 rider: pass `scratch.selectors`, `1.8–3.7 KB`, already decoded in S)** — *Lever 5*: Table 3 groups have `50–72%` `nn = 1` swaps; hex groups stay in `nn < 16` (`T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md` §1). Specialising per-50-symbol group avoids `f1`/`second()` checks (`[CONJECTURED]`, ~`15–25%` of `A5` = `~1.5–2.5%` of worker time).
   - *Prior art:* `src/runs.rs` (zero-`L` run front), `E1-CROSS-STAGE-MTF-MONOID.md`, `T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md`, external reports 5–7 (`95b9cf9b`, `cc6c5831`), Carbon cycle 38 (`300a2997`).

#### 4. Is M spent?
- **Mostly spent as a standalone stage** (`10.3%` of worker time, zero DRAM misses, zero `L` materialised). Its main remaining leverage is **eliminating its interface boundary with `A6a`** (counted jointly with C).

---

### 1.5 Factor R — RLE1, CRC, and Output Finalisation (`src/rle.rs`, `src/crc.rs`; Target T6)

#### 1. Mathematical object
- **Input:** Pre-RLE1 inverted block `buf[0..n]` and expected header `crc: u32`.
- **Output:** Expanded final block `final[0..n']` (every 4 identical bytes `c,c,c,c` followed by count byte `extra ∈ 0..255` replaced by `4 + extra` copies of `c`) and exact verification `CRC-32/BZIP2(final) == crc` (polynomial `0x04C11DB7`, MSB-first, init/final XOR `!0`).
- **Dependency structure:**
  - *RLE1 run finding:* Purely local 4-byte equality predicate between runs, with greedy left-to-right disambiguation only at 5+ identical bytes (`aaaaa` = 4 `a`'s + count `b'a' = 97`).
  - *In-place expansion (`unrle_in_place`):* Requires global prefix maximum `shift = max_t (grown_t)` across all runs before shifting, forcing a 2-pass structure (`find_runs` → `copy_within` up by `shift` → forward expand + 64 KiB L2-tiled CRC).
  - *CRC-32:* Linear recurrence in $\mathbb{F}_2[x]/(P(x))$ — strictly an **associative homomorphism** (`M105`, `leads/structure-and-function.md`), parallelised via 4×128-bit `PCLMULQDQ` folding (`512` bits/step, `src/crc.rs:127-168`).

#### 2. Current cost (`src/rle.rs:103-236`, `src/crc.rs:127-168`) and lower bounds
- **Current implementation:**
  - $Q$: $0$ (100% sequential streaming).
  - $D$: $O(1)$ per 32 bytes (`find_runs_avx2`) and $O(1)$ per 64 bytes (4-lane `clmul_update`).
  - $P$: **High memory-traffic multiplier when `shift > 0`!**
    1. `find_runs_avx2` reads `n` bytes;
    2. `buf.copy_within(first..n, first + shift)` reads `~n` bytes and writes `~n` bytes;
    3. Forward expansion reads `~n` bytes and writes `~n'` bytes;
    4. 64 KiB tiled `crc::update_with` reads `n'` bytes (mostly L2-hot).
    Total traffic: up to **`~3n + 2n' ≈ 5n` bytes** (`~4.2 GB` across wikidata ×4!). That explains why `A9` inflates by **+86%** from $k=1$ (`44.0 µs/blk`, `5.9%` of worker time) to $k=10$ (`82.0 µs/blk`, `8.2%` of worker time; `character-stage1/report.txt` L139 vs L148): it is **L3/memory-traffic bound** at high $k$!
  - $S$: In-place in `buf` (`~900 KB–1 MB`) + tiny `runs: Vec<Run>`.
- **Lower bounds:**
  - *Streaming bound `[PROVED]`*: Must inspect every byte of `buf[0..n]` ($\ge n$ bytes read) and produce `final[0..n']` ($\ge n'$ bytes written) to detect runs and verify the exact CRC-32. Minimum separate-stage traffic is `n + n' ≈ 2n` bytes (1 pass: fused `unrle + crc` from `src` to `dst`), or `n'` bytes if fused into emission `A7` (zero pre-RLE1 `buf` materialised!).
  - *Compute bound `[ARGUED]`*: `PCLMULQDQ` (~`0.15 cyc/B`) + `AVX2` run-detect (~`0.10 cyc/B`) over 839 MB at 4.91 GHz is `~43 ms` at $k=1$ (`~5.4 ms` at $k=8$). Current `A9` (`76 ms` at $k=1$, `13.3 ms` at $k=8$, `14.0 ms` at $k=10$) is **`1.7–2.5×` above** its single-pass compute/bandwidth bound.

#### 3. Candidate reformulations
1. **Single-pass tiled out-of-place `unrle_and_crc(src, dst)` (eliminating `find_runs` + `copy_within`)**
   - *Levers pulled:* **Lever 1** (flow map: `5n → 2n` memory traffic) + **Lever 4b** (ownership: `src` is worker-private scratch reused every block, `dst` is the pooled buffer handed to `ready` — zero extra per-block `ready` RSS, only +`900 KB`/worker = +`7 MB` total at $k=8$!).
   - *Predicted cost:* `[ARGUED]` Cuts `A9` by `~40–50%` (`~3–4%` of worker time, and noticeably reduces L3/IMC traffic at $k \ge 8$!).
   - *Needs from neighbours:* Worker keeps one private 900 KB `pre_rle` buffer (actually `OutQ16` can live in `pre_rle`, and `dst` goes directly to `ready`).
   - *Prior art:* `src/rle.rs:9` (`unrle_avx2`) already does single-pass out-of-place `unrle`, just without L1-tiled inline `crc::update_with`.
2. **Fusing RLE1 + CRC into emission `A7` (tile-level online RLE1+CRC, or associative CRC across $L$ segments)**
   - *Levers pulled:* **Levers 1 + 5** (tupling / homomorphism: `CRC(A·B) = combine(CRC(A), CRC(B), |B|)`).
   - *Predicted cost:* `[ARGUED]` Eliminates pre-RLE1 `buf` write/read completely (`5n → 1n'`!), saving `~50–65%` of `A9` (`~3.5–5%` of worker time). Especially clean if paired with $L$-lane forward emission (§1.2 Candidate 2), where segment stitching and single-pass `unrle+crc` fuse into one pass!
   - *Prior art:* Record 105 (associative CRC); Rauzy–Veech 10 Nov / carbon-kitchen-sink C4 (word-grammar RLE1+CRC, `tracks/cross-stage.md`, rejected when it bloated the word dictionary — tile-level fusion avoids dictionary bloat).

#### 4. Is R spent?
- **Small ceiling (`5.9%` at $k=1$, `8.2%` of worker time at $k=10$), NOT at its traffic bound (`5n` vs `2n`),** so ~40–60% of R is capturable, but that buys only `~3–4%` of worker time (and 0% at $k \ge 8$ while S binds). Worth doing only as a clean fusion companion to `E` or `C`, never as a standalone priority.

---

### 1.6 Factor D — Delivery (`src/parallel.rs:841-968`, `A10`)

#### 1. Mathematical object
- **Input:** Out-of-order stream of completed `(place, Result<Vec<u8>, Error>)` entries inserted into `st.ready`, plus caller `read(&mut buf)` calls.
- **Output:** Contiguous in-order byte stream copied into caller's `buf` (or moved whole via `read_block`), returning spent `Vec<u8>` buffers to `st.free_out` and waking the scanner when `issued - taken < slots`.
- **Dependency structure:** Strict in-order resequencing (`taken = 0, 1, \dots, J-1`) running **concurrently on the caller thread** while scanner and workers process blocks `taken + 1 .. taken + W`.

#### 2. Current cost (`src/parallel.rs:841-968`) and lower bounds
- **Current implementation:**
  - $Q$: $0$ (sequential `memcpy` of `838.8 MB` + `1,724` mutex ops = `0.25 ms` total, `character-stage1/report.txt` L256).
  - $D$: $O(1)$ per cache line.
  - $P$: $1$ streaming copy (`838.8 MB` read + `838.8 MB` write = `1.68 GB` traffic). Measured `copy + sink` = `0.0343 s` ($k=1$), `0.0334 s` ($k=6$), `0.0403 s` ($k=8$), `0.0507 s` ($k=10$) (`character-stage1/report.txt` L274–283) — **always $\le 26\%$ of wall**!
  - $S$: Ready queue averages `0.03–2.02 MB` (peak `12.7 MB` at $k=10$, `character-stage1/report.txt` L80–89).
- **Lower bounds:**
  - *`std::io::Read` copy bound `[PROVED]`*: Any `Read::read(&mut self, buf: &mut [u8])` implementation must write $N$ bytes to `buf` ($\ge N / \text{BW}_{\text{mem}} \approx 25\text{–}35\text{ ms}$ on 839 MB).
  - *Zero-copy bound `[PROVED]`*: Moving buffer ownership (`read_block` / `BufRead`) cuts copy work to $0$ (leaving only `~0.25 ms` lock/resequencing overhead).
  - *Pipeline criticality `[PROVED]`*: Because D executes concurrently on its own thread, $T_D$ adds $0$ to steady-state wall whenever $T_D < \max(T_{\text{scan}}, T_{\text{wkr}})$, contributing only the final block's drain (`~20 µs`) plus minor shared-cache/core contention (`~0.5–1.5%`) at $k \ge 8$.

#### 3. Candidate reformulations
1. **Zero-copy ownership transfer (`read_block` / `BufRead::fill_buf`)** — *Lever 4b* (ownership). Already implemented in `src/parallel.rs:867`! Eliminates `30–47 ms` of caller CPU and `1.68 GB` of copy traffic, reducing contention by `~0.5–1.5%` at $k=8\text{–}10$, and removing the `47 ms` reader floor once all other parts are made $4\times$ faster.
2. **Direct worker write into caller buffer** — Impossible in bzip2 without pre-known expanded block lengths (`[PROVED]`, since RLE1 length is unknown until after BWT inversion).

#### 4. Is D spent?
- **YES, COMPLETELY SPENT for current operating points.** `D_read ≤ 0.26 × wall` at every $k$; `read_block` already achieves the zero-copy lower bound. D matters only at the extreme `all-ideal` floor at $k = 10$.

---

## Part 2: Pre-Registered Predictions for the Ideal-Parts Matrix

### 2.1 Calibration & Regime Arithmetic (Pinned Scanner + 100 ms Pre-Spin)

The matrix (`ideal-parts/design.md`) runs all arms in **one binary** on the i9-10900K (10 physical cores / 20 threads) with **(i) 100 ms pre-spin on every CPU** (`4.91 GHz`, removing the low-clock state that inflated four-arm and unspun runs, §81) and **(ii) the scanner pinned to its fastest core with its sibling idle** (stage 2b `cores` phase), leaving **9 physical cores (18 logical CPUs)** for the $k$ workers + reader + feeder.

From the traced baselines (`character-stage1/report.txt`, `measurements-scan-profile.tsv`, `measurements-four-arm.tsv`, `character-stage2b/analysis/link.tsv`, `clock-mechanism/report.txt`):

1. **All-real baseline under matrix controls (ledgers off, pinned, spun):**
   - **$k = 1$:** $T_0 \approx 1.283\text{ s}$ (stage 1 `1.2831 s`, scan-profile `1.3038 s`; workers bind cleanly: $T_{\text{wkr}} \approx 1.280\text{ s}$, $T_{\text{scan}} \approx 0.173\text{ s}$, $T_D \approx 0.034\text{ s}$).
   - **$k = 6$:** $T_0 \approx 0.238\text{ s}$ (stage 1 `0.2384 s`, scan-profile `0.2381 s`; already at 4.91 GHz, zero SMT sharing on 9 cores: $T_{\text{wkr}} \approx 0.235\text{ s}$, $T_{\text{scan}} \approx 0.183\text{ s}$, $T_D \approx 0.033\text{ s}$, startup+starve `4.9 ms`).
   - **$k = 8$:** Stage 1 unpinned `0.1944 s`, scan-profile `0.1911 s`, stage 2b `pinned/unpinned = 0.960`. Pinned + spun at 4.91 GHz: $T_{\text{scan}} \approx 0.1835\text{ s}$ (quiet core at 4.91 GHz, matching $k=6$'s `0.1832 s` + tiny L3 traffic), $T_{\text{wkr}} \approx 0.179\text{ s}$ (scan-profile `0.1786 s`, 8 workers on 9 free physical cores), knee/startup/drain overhead `~3.5%` → **$T_0 \approx 0.190\text{ s}$**. **Crucial consequence:** because pinning + spin already removes the ~10–13% unpinned/low-clock scanner inflation seen in four-arm and stage 2a (`C/A = 0.874`), $T_{\text{scan}}$ (`0.1835 s`) and $T_{\text{wkr}}$ (`0.179 s`) sit **within ~4.5 ms of each other** at $k=8$!
   - **$k = 10$:** **Directly measured in `clock-mechanism/report.txt` (`SPIN` arm, pinned at 4.91 GHz): $T_0 = 0.196\text{ s}$** (`S0 = 0.195 s`, SD `1.12%`). Here $T_{\text{scan}} \approx 0.1885\text{ s}$ (pinned at 4.91 GHz + ~5 ms L3/maint contention from 10 workers, cf. stage 1 `0.1903 s` / scan-profile `0.1890 s`), while $T_{\text{wkr}}$ in all-real is `0.167–0.171 s` (`13.3%` starved). When S becomes ideal (0% starvation), all **10 workers run continuously on the 9 non-scanner physical cores**, forcing 1–2 SMT sibling pairs (+48% per shared pair in stage 2b ⇒ ~3–5% mean inflation over `0.167 s`), giving $T_{\text{wkr}}(\text{S ideal}) \approx 0.173\text{–}0.176\text{ s}$.

2. **Two critical oracle-boundary subtleties (built into the predictions below):**
   - **Subtlety A — `E` (`supplied-codes`) removes only ~44% of `A7`, leaving ~56% (`0.272 s` at $k=1$) inside the clock:** From stage 1 (`A7 = 0.4903 s` = `38.3%` at $k=1$, `0.0894 s` = `38.0%` at $k=6$) vs four-arm (`B/A = 0.830` at $k=1$, `0.850` at $k=6$), `supplied-codes` removes `0.218 s` (`44.4%` of `A7`) at $k=1$ and `0.0352 s` (`39.4%` of `A7`) at $k=6$, because word lookup (`inline16`) + 16-byte stores to `out` still run! **This unremoved `55.6%` of `A7` (`~272 ms` at $k=1$, `~54 ms` at $k=6$, `~41–46 ms` at $k=8\text{–}10$) dominates the `all-ideal` floor!**
   - **Subtlety B — Oracle handover costs (`parts.md` L5, `prompt-build-oracles.md` L16, L40):**
     - `S`: `memcpy` of `200.3 MB` of `u16` symbols on 1 core ≈ `10–14 ms` (`0.010–0.014 s`) — negligible at $k=1..10$ except in the all-ideal floor.
     - `M`: `memcpy` of `~180 MB` of `Runs` records across $k$ workers ≈ `8 ms` at $k=1$, `4 ms` at $k=6..10$ (assuming M hands over `Runs` for Open blocks and `L` only for the 192 tiny column blocks; if M instead copied `839 MB` of `L` and re-ran `Runs::of_column`, it would add `~40 ms`!).
     - `C`: `~466 MB` of built structures (`edges`, `inline16`, `lens`, `spelling`; `116 MB` distinct > 20 MiB L3). Handover copy + cold-L3 miss penalty on `A7` (since `A6c` no longer warms `edges` in L2/L3) ≈ `~25 ms` at $k=1$, `~10–15 ms` at $k=6..10$ (less if `edges` is zero-copy borrowed `&[u32]`, slightly more if copied into `Kernel::edges`).
     - **`R` (HIGH-RISK HANDOVER WARNING!):** Final bytes = `838.8 MB` (`209.7 MB` distinct = `10.5×` L3!). At $k=6$, `A9` itself is only **`15.1 ms`/worker** (`90.6 ms` total CPU), whereas copying `838.8 MB` from a cold `210 MB` DRAM store across 6 workers (`1.68 GB` DRAM read+write at ~`40–50 GB/s` dual-channel DDR4) takes **`~22–35 ms` wall — SLOWER than real `A9`!** Below we predict **R (zero-copy `Arc<[u8]>` / slice handover)** as the primary algorithmic room, and register **R (1-copy `memcpy` handover)** explicitly alongside it so the matrix can distinguish true T6 room from copy handover!

### 2.2 Single-Part Room Predictions: `T(part alone ideal) / T(all real)`

Format: **point [range]** and expected `registration.md` label (`ROOM` iff upper bound $\le 0.95$; `NO ROOM` iff inside `[0.98, 1.02]`; `MASKED` iff `NO ROOM` alone + `INTERACTS` with another part Q + `ROOM` behind Q; `UNRESOLVED` otherwise).

| Part | $k = 1$ | $k = 6$ | $k = 8$ (pinned + spin) | $k = 10$ (pinned + spin) |
|---|---|---|---|---|
| **S** (Huffman scan, T4) | **0.996** `[0.990, 1.002]` (`NO ROOM`) | **0.982** `[0.975, 0.990]` (`NO ROOM` / edge) | **0.950** `[0.920, 0.975]` (`UNRESOLVED` / borderline `ROOM`) | **0.885** `[0.845, 0.925]` (`ROOM`) |
| **E** (supplied codes, T1) | **0.830** `[0.815, 0.845]` (`ROOM`) | **0.850** `[0.835, 0.865]` (`ROOM`) | **0.980** `[0.960, 1.000]` (`MASKED` by S) | **0.990** `[0.970, 1.010]` (`MASKED` by S) |
| **C** (construction, T2) | **0.570** `[0.545, 0.600]` (`ROOM`) | **0.785** `[0.760, 0.815]` (`ROOM`, **scanner-capped!**) | **0.973** `[0.950, 0.995]` (`MASKED` by S) | **0.980** `[0.955, 1.005]` (`MASKED` by S) |
| **M** (MTF/RLE2, T3) | **0.905** `[0.885, 0.925]` (`ROOM`) | **0.916** `[0.895, 0.938]` (`ROOM`) | **0.983** `[0.965, 1.002]` (`MASKED` by S) | **0.992** `[0.975, 1.010]` (`MASKED` by S) |
| **R** (RLE1/CRC, T6 — *zero-copy*) | **0.942** `[0.928, 0.956]` (`ROOM` / borderline) | **0.941** `[0.925, 0.958]` (`ROOM` / borderline) | **0.985** `[0.968, 1.002]` (`MASKED` by S) | **0.992** `[0.975, 1.010]` (`MASKED` by S) |
| *R (if 1-copy 839 MB `memcpy`)* | *0.968 `[0.952, 0.985]`* | *1.025 `[0.995, 1.060]` (SLOWER!)* | *1.015 `[0.985, 1.045]`* | *1.005 `[0.980, 1.030]`* |
| **D** (delivery, `A10`) | **0.999** `[0.993, 1.005]` (`NO ROOM`) | **0.995** `[0.985, 1.005]` (`NO ROOM`) | **0.990** `[0.975, 1.005]` (`NO ROOM`) | **0.985** `[0.965, 1.005]` (`NO ROOM`) |
| **Unpinned link at $k=8$** | — | — | **S alone unpinned: ~0.910** `[0.880, 0.940]` (vs 0.950 pinned) | — |

**Key arithmetic checks embedded in this table:**
- **Why `S` at $k=8$ pinned is `0.950` (NOT `0.874` as in stage 2a):** In stage 2a (unpinned, old four-arm binary), arm A was +10.9% slower (`~0.214 s`), so ideal S dropped to workers (`0.185 s`) giving `0.874`. With scanner pinned + 100 ms spin, $T_{\text{scan}} \approx 0.1835\text{ s}$ and $T_{\text{wkr}} \approx 0.179\text{ s}$, so ideal S hits the worker ceiling almost immediately (`0.181 / 0.190 = 0.953`, matching stage 2b's $e_{\text{up}} \approx 0.13\text{–}0.18$!).
- **Why `S` at $k=10$ pinned is `0.885` (NOT `0.806` as in four-arm):** Pinning + spin cuts all-real at $k=10$ from four-arm's `0.226 s` down to `0.196 s` (`clock-mechanism` SPIN), while unstarved 10 workers on 9 free physical cores run at `~0.173–0.176 s` → `0.174 / 0.196 = 0.888`.
- **Why `C` at $k=6$ is `0.785` (NOT `0.55`):** Removing `44.9%` of worker time (`0.1056 s` minus `~10 ms` handover) drops $T_{\text{wkr}}$ from `0.235 s` to `~0.140 s`, which **crosses below $T_{\text{scan}} = 0.183\text{ s}$**! Thus `C` alone at $k=6$ hits the scanner wall (`~0.187 / 0.238 = 0.786`).

### 2.3 The All-Ideal Floor (`S+E+C+M+R+D` over All Real) and D's Share of the Floor

Because `E` (`supplied-codes`) leaves the word-copying half of `A7` (`~55.6%` of `A7` = `0.272 s` at $k=1$, `0.054 s` at $k=6$, `0.041 s` at $k=8$, `0.046 s` at $k=10$) inside the clock, the `all-ideal` floor does **not** collapse near zero! Adding oracle handovers (`hM + hC + hR_zc ≈ 34 ms` at $k=1$, `~15 ms` at $k=6..10$; `hS ≈ 10–14 ms`):

| Arm | $k = 1$ | $k = 6$ | $k = 8$ (pinned + spin) | $k = 10$ (pinned + spin) |
|---|---|---|---|---|
| **All ideal (`SECMRD`) / All real** | **0.245** `[0.22, 0.28]` (`~314 ms`) | **0.300** `[0.26, 0.35]` (`~72 ms`) | **0.305** `[0.26, 0.36]` (`~58 ms`) | **0.330** `[0.28, 0.40]` (`~64 ms`) |
| **All ideal except D (`SECMR`) / All ideal** | **1.000** `[0.99, 1.01]` | **1.005** `[0.99, 1.03]` | **1.015** `[0.99, 1.06]` | **1.03–1.12** (D = `~47–51 ms` approaches/touches the `~55–64 ms` floor!) |

*(If C and R both perform full `memcpy` handovers rather than zero-copy borrows, the `all-ideal` floor shifts upward by `~20–30 ms` at $k=6..10$, to `~0.38–0.45`, and pushes D back below the floor even at $k=10$.)*

### 2.4 Predicted Pairwise Interactions

We report the **2×2 super-additivity contrast** $\text{SA}(A, B) = \frac{T(AB)\cdot T(0)}{T(A)\cdot T(B)}$ (where $< 1$ is super-additive / negative log interaction, $> 1$ is sub-additive / positive log interaction, matching `four-arm-report.txt` L11/21/31) and the conditional ratio $T(AB)/T(0)$.

| Pair | Mechanism | $k = 1$ | $k = 6$ | $k = 8$ (pinned + spin) | $k = 10$ (pinned + spin) |
|---|---|---|---|---|---|
| **`S × C`** | **Cross-station unmasking** (largest interaction in the matrix!) | `1.00` (`AB=0.569`, `ADDITIVE`) | **`0.76`** `[0.72, 0.81]` (`AB=0.594`, **INTERACTS** — C alone hits scanner wall at $k=6$!) | **`0.63`** `[0.58, 0.69]` (`AB=0.585`, **INTERACTS** — `C` behind `S` is `~0.61`!) | **`0.63`** `[0.57, 0.69]` (`AB=0.560`, **INTERACTS** — `C` behind `S` is `~0.63`!) |
| **`S × E`** | **Cross-station unmasking** (four-arm interaction) | `1.00` (`AB=0.829`, `ADDITIVE`) | `0.99` (`AB=0.846`, `ADDITIVE`) | **`0.87`** `[0.83, 0.91]` (`AB=0.811`, **INTERACTS** — `E` behind `S` is `~0.85`!) | **`0.91`** `[0.86, 0.96]` (`AB=0.815`, **INTERACTS** — `E` behind `S` is `~0.90–0.92`!) |
| **`S × M`** | **Cross-station unmasking** | `1.00` (`AB=0.902`, `ADDITIVE`) | `1.00` (`AB=0.909`, `ADDITIVE`) | **`0.93`** `[0.90, 0.97]` (`AB=0.874`, **INTERACTS**) | **`0.93`** `[0.89, 0.97]` (`AB=0.835`, **INTERACTS**) |
| **`S × R`** *(zero-copy)* | **Cross-station unmasking** | `1.00` (`AB=0.941`, `ADDITIVE`) | `1.00` (`AB=0.935`, `ADDITIVE`) | **`0.95`** `[0.92, 0.98]` (`AB=0.888`, **INTERACTS**) | **`0.93`** `[0.89, 0.98]` (`AB=0.837`, **INTERACTS**) |
| **`C × E`** | **Sign-flipping interaction!** (1) Same-station log-convexity + cache-warming at $k=1$; (2) Scanner-wall censoring at $k=6$! | **`0.84`** `[0.81, 0.87]` (`AB=0.400`, **INTERACTS** — algebraic log-convexity $\frac{1-\alpha-\beta}{(1-\alpha)(1-\beta)} = 0.845$ + ~1–2% cold-`edges` cache synergy!) | **`1.17`** `[1.12, 1.22]` (`AB=0.785`, **INTERACTS, POSITIVE!** — C alone saturates scanner wall, censoring E!) | `1.02` `[0.99, 1.05]` (`AB=0.969` when $S=-$; **note:** in the full $2^{6-1}$ sign table averaging over $S=\pm$, the $S=+$ half contributes `0.85` while $S=-$ contributes `1.02`, giving a net **negative** sign-table 2FI `~−0.07`!) | `1.00` `[0.98, 1.03]` (same sign-table split: net `~−0.05` across $S=\pm$) |
| **`C × M`** | Same-station log-convexity ($k=1$); scanner-wall censoring ($k=6$) | **`0.92`** `[0.90, 0.95]` (`AB=0.473`, **INTERACTS** — algebraic log-convexity!) | **`1.09`** `[1.05, 1.14]` (`AB=0.785`, **INTERACTS, POSITIVE!**) | `1.01` (at $S=-$); `~0.92` at $S=+$ | `1.00` (at $S=-$); `~0.92` at $S=+$ |
| **`C × R`** | Same-station log-convexity ($k=1$); scanner-wall censoring ($k=6$) | **`0.95`** `[0.93, 0.98]` (`AB=0.511`, **INTERACTS**) | **`1.06`** `[1.03, 1.10]` (`AB=0.785`, **INTERACTS, POSITIVE!**) | `1.01` (at $S=-$); `~0.95` at $S=+$ | `1.00` (at $S=-$); `~0.94` at $S=+$ |
| **`E × M`** | Mild log-convexity ($k=1$); barely touches scanner wall ($k=6$) | `0.98` `[0.96, 1.00]` (`AB=0.733`) | `1.02` `[0.99, 1.05]` (`AB=0.798`) | `1.01` (`ADDITIVE` at $S=-$) | `1.00` (`ADDITIVE` at $S=-$) |

**Diagnostic warning for Jain's log-wall analysis (`design.md` L48):**
At $k = 1$ (and inside the $S = +$ half at $k \ge 6$), any two large additive worker savings $\alpha, \beta$ produce a negative log-scale interaction $\log\frac{1-\alpha-\beta}{(1-\alpha)(1-\beta)} < 0$ purely from log-convexity (`C × E = 0.845`, `C × M = 0.919` at $k=1$). Do **not** misinterpret that algebraic term as a physical synergy; check additive (linear-time) interaction $T(AB) - T(A) - T(B) + T(0)$ alongside log-wall to isolate the true physical `C × E` cold-`edges` cache effect (~1–3%) from log-convexity!

---

## Part 3: Where to Look — Priority Ranking & Fastest Kill Experiments

Scoring each part by **effective room** (accounting for masking at production $k = 8\text{–}10$, i.e. S's gatekeeper role + each worker part's room behind S and at $k \le 6$) $\times$ **confidence that a reformulation captures a meaningful share** ($\ge 30\%$ of that part's room):

| Rank | Part | Room (standalone $k=1 / 6 / 8 / 10$; and behind S at $k=10$) | Capture confidence | Score (qualitative rank basis) | Primary reformulation | Fastest kill experiment (matrix cell or offline check) |
|---:|---|---|---:|---|---|---|
| **1** | **S** (Huffman scan, T4 — *topology*, NOT serial LUT) | `0.4% / 1.8% / 5% / 11.5%` pinned (`~19%` unpinned), **plus gatekeeps 40% in C & 10–15% in E!** | **0.80** (cross-block parallel scan); *0.05 for serial LUT* | **Highest (mandatory gatekeeper)** | Cross-block magic-indexed parallel Huffman inside lazy worker pool (workers pull raw bit-slices end-to-end, deleting serial scanner & 200 MB `syms` queue) | **Slice A `(S+, E-)` at $k = 8, 10$** (pinned + unpinned link): kills S's standalone urgency if pinned `S+` room at $k=10$ is $< 5\%$; offline SIMD magic-scan timing on wikidata ×4 kills parallel scan if scan overhead $> 10\text{ ms}$. |
| **2** | **C** (Construction, T2) | **`43% / 21.5% / 2.7% / 2.0%`** alone; **`~37–40%` behind S at $k = 6, 8, 10$** (`S+C ≈ 0.56–0.59`!) | **0.60** (`A5+A6a` deforestation + `A6c` zero-RMW/prefetch + `A6b` dict) | **#1 worker target (`~22–24%` expected capture behind S!)** | (a) Fuse `M` (`runs::front`) + `A6a` (`OutQ16`); (b) eliminate `A6c`'s $m$ scattered RMWs (paired with $L$-lane forward emit, or prefetched `wire_xor`); (c) strip `A6b` dictionary hash overhead | **Matrix `(S-, C+)` at $k = 1, 6$ and `(S+, C+)` vs `(S+, C-)` at $k = 8, 10$**: kills C's dominance immediately if `(S+, C+)/(S+, C-) > 0.75`. Locally at $k=1$, micro-profile `A6a` (`maximal` vs `OutQ16`) and `A6b` (IET vs `Dictionary::code`) before writing code. |
| **3** | **E** (Emission, T1 — *chase + spelling*) | `17% / 15% / 2% / 1%` alone (`supplied-codes`); `~10–15%` behind S, **plus `21%` unremoved word-copy half of `A7`** | **0.30** (chase/closure) / **0.50** (copy half & $L$-lane MLP) | **#2 worker target (`~8–12%` expected capture behind S)** | (a) $L$-lane forward emit on `edges` (cuts chase depth $D$ AND kills `A6c`'s XOR RMWs!); (b) eliminate backward `u128` shifts / fast loop / bounded 2-word inline fusion | **Slice A `(S+, E+)` vs `(S+, E-)` at $k = 8, 10$** (tests whether `D/C` improves from `0.92` toward `0.86` with spin+pin) + **all-ideal floor at $k = 1$** (measures exact size of the unremoved word-copy half of `A7`, testing our `~24.5%` floor prediction). |
| **4** | **M** (MTF/RLE2, T3) | `9.5% / 8.4% / 1.7% / 0.8%` alone; `~8%` behind S | **0.35** (mostly via `A5+A6a` fusion & 2-lit/selector hints) | Low standalone (~2–3% capture) | Fuse `A5` into `A6a` (counted with C) + 2-literal `16×16` shuffle table / selector-hinted fast paths | **Matrix `(S-, M+)` at $k = 1, 6$** and `(S+, M+)` at $k = 8, 10$: confirms `~9%` ceiling and tests whether `Runs` handover stays `< 2%`. |
| **5** | **R** (RLE1/CRC, T6) | `5.8% / 5.9% / 1.5% / 0.8%` gross (`~7–8%` behind S); *risks negative net room if 839 MB copied!* | **0.50** (single-pass tiled `unrle+crc`, `5n → 2n`) | Low (~3–4% capture) | Single-pass tiled `unrle+crc` (or fuse with $L$-lane segment stitching in E) | **Matrix `(S-, R+)` at $k = 1, 6$** with ledger check on handover time vs `A9`: immediately exposes whether R oracle copies 839 MB and confirms `~6%` gross ceiling. |
| **6** | **D** (Delivery, `A10`) | `0.1% / 0.5% / 1.0% / 1.5%` (`NO ROOM`) | **0.90** (`read_block` exists) | Negligible (~0–1%) | Use `read_block` / `BufRead` zero-copy API when caller supports it | **Matrix `(S-, D+)` at $k = 8, 10$** (expected `NO ROOM`) and `SECMR` vs `SECMRD` at $k = 10$ (checks floor emergence). |

### The Two Parts to Work on First
1. **S (cross-block parallel Huffman in the lazy worker pool):** Nothing in the workers moves the wall at $k = 8\text{–}10$ until S's serial thread is gone. Even better, letting the lazy workers pull raw compressed slices and decode end-to-end frees the scanner's physical core (turning 9 worker cores into 10) and eliminates the 200 MB `syms` inter-thread transfer.
2. **C (construction: `A5+A6a` fusion, `A6b` dictionary overhead, `A6c` zero-RMW wiring paired with $L$-lane forward emit in E):** Once S is unmasked (and immediately at $k \le 6$), **C holds `~40–43%` of whole-decoder wall time** (`S+C ≈ 0.56–0.59` vs `S+E ≈ 0.81`) — and unlike T1's hard closure barrier, C's sub-phases (`15.5% + 16.7% + 12.7%`) are directly attackable with concrete structural rewrites!

---

## Part 4: Comparison with MONODROMY's Records 129–132 (and Record 133)

*Written after Parts 1–3 were completed and frozen, upon reading `origin/claude/sharp-meitner-cx0v78:docs/journal/monodromy/` (`2026-11-25-course-mining-and-analysis-plan.md` [129], `2026-11-25-design-space-and-identification.md` [130], `2026-11-25-eup-prediction.md` [131], `2026-11-25-placement-and-the-max-model.md` [132], and `2026-11-25-corrections-129-132.md` [133]).*

### 4.1 Where We Agree (Independent Convergence)

1. **Construction (`C`, 45% of worker time) beats Emission (`E`, 38% total / 17% `supplied-codes`) as the #1 worker target.** MONODROMY Record 130 §6 (`A4`: MTF 0.10, construction 0.45 `[0.16, 0.17, 0.13]`, emission 0.38, RLE1/CRC 0.06) and our Parts 1–3 converge on the exact same ranking.
2. **Cross-block pooling (`P1`, Record 130 §1 = our S Candidate 1) strictly dominates within-block speculative Huffman (`P2`).** Both derive the same format-level principle: cut the serial chain where the format resets state (block boundaries via magic scan), whereas inside a block the 50-symbol selector-count obstruction breaks pure self-synchronisation and makes speculative FSM composition lose on CPU work for every $k < 50$. Pooling also removes Record 132 §5's structural placement condition `(N1)` by eliminating the distinguished serial scanner core.
3. **Single-thread serial Huffman LUT arithmetic is spent.** Record 133 §1 withdraws Record 129 §1.1's −13–17% multi-symbol price after §82 confirmed `src/huff.rs` already decodes 1.85 symbols/lookup and 3-symbol LUTs (`#870`) regressed — matching our independent audit of `src/huff.rs` in §1.1.
4. **Sharp maximum at pinned $k = 8$ with a ~3–4% gap (`W/S ≈ 0.966–0.975`).** Record 132 §2 derives $S = 1, W = 0.966$ (`g = 3.4%`) from stage 2b's elasticities alone, matching our independent stage 1 + scan-profile arithmetic ($T_{\text{scan}} \approx 0.1835\text{ s}, T_{\text{wkr}} \approx 0.179\text{ s} \Rightarrow W/S = 0.975$, predicting pinned `S alone` at $k=8$ saves only `~5%`, not stage 2a's `12.6%`).
5. **Algebra of programming & saturation law.** Record 130 §3's fusion / tupling / third-homomorphism taxonomy matches our Part 1 (MTF and CRC are associative homomorphisms; Huffman parse and LF walk are dependent chains whose composition inflates state). Record 129 §1.3's machine-repairman lane-saturation law $L^* = (\ell + c)/c$ (capped at ~10 LFBs) is an excellent quantitative refinement of our §1.2 Candidate 2: `emit_bidirectional` sits at $L = 2$, well below $L^* \approx 5\text{–}10$ for L3-served `edges`.
6. **Lazy thin-strip removal (`S*_1`) stays permanently closed** regardless of bottleneck migration (Record 129 §7, matching our `T1-T2-LAZY-ACQ-AND-DEPTH2.md` §2).

### 4.2 Where We Disagree (with the Arithmetic)

1. **Record 129 §1.2 & Record 130 §1 overprice pooled scanning (`P1`) at `90 µs/block` (−15%) by using low-$k$ worker demand and 10 uncontended cores:**
   - MONODROMY writes `(107 + 798) / 10 = 90.5 µs/block` (`0.156 s` on 1,724 blocks, −15% against `107 µs`), using `w = 798 µs` from $k = 1\text{–}4$ (`scan-profile`) and dividing by `c = 10` full physical cores.
   - **Check against Stage 1 (`character-stage1/report.txt` L32–41, L166–175):** At $k = 8$, `D_wkr = 8 × 0.1058 = 846.4 µs/block` (`1.478 s` total worker CPU), and at $k = 10$, `D_wkr` inflates to `990 µs/block` (`1.713 s`, +24% over `798 µs`) due to LLC capacity contention (`IMC reads 219 → 807 → 1562 MiB`) plus SMT sharing. Moreover, `reader + feeder` consume `~0.065 s / 0.196 s ≈ 0.33` of a physical core, leaving `~9.5–9.7` effective cores (not 10.0).
   - **Corrected pooled wall:** Using $k=8$ measured demands (`1.478 + 0.183 = 1.661 s`, less `~0.015 s` saved by deleting the 200 MB `syms` inter-thread queue/maint = `1.646 s`) over `~9.5` effective cores gives **`~0.173 s`** (`100 µs/block`), which is **`−9%`** against pinned $k=8$ (`0.190 s`), not `−15%`. (Running 10 pooled workers at $k=10$'s inflated `1.713 s` demand would give `~0.198 s` — zero gain unless capped at `8–9` workers to avoid SMT + L3 thrashing!) Pooling's decisive value is still what Record 129 §1.2 identifies: it shifts worker elasticity from `0` to `~0.88`, unlocking `C`'s 45% room (`S+C ≈ 0.56–0.59`).
2. **Record 132 §4's pinned worker capacity table (`7 + 2σ = 8.36` at $k=9$, `6 + 4σ = 8.72` at $k=10$) miscounts physical cores and conflates SMT sharing with L3 capacity contention:**
   - On a 10-core i9-10900K with 1 physical core reserved for the pinned scanner (sibling idle), **9 physical cores** remain for workers + reader + feeder (`reader + feeder` together use only `~0.33` of a core, not a full dedicated core). So at $k = 9$, all 9 workers get distinct physical cores (capacity `~8.7–8.8`), and at $k = 10$, only 2 workers share 1 core (`8 + 2σ − 0.3 ≈ 9.0–9.1`), NOT 4 workers sharing 2 cores (`6 + 4σ = 8.72`).
   - Why does Stage 1's actual effective worker throughput at $k=10$ (`8.0 × 0.1847 / 0.1713 = 8.63`) nevertheless match MONODROMY's `8.72`? Because **LLC capacity thrashing** (`IMC reads 807 → 1562 MiB`, nearly 2× from $k=8$ to $k=10$) inflates *every* worker's service time even on unshared cores! Record 130 §6 reduced worker inflation (`1.17 → 1.31 → 1.54 → 1.73`) solely to SMT core-sharing probability $z$, overlooking that pure-streaming `A9` (insensitive to SMT execution-port contention) inflates **+34%** from $k=8$ to $k=10$ (`61.2 → 82.0 µs`, `character-stage1/report.txt` L146 vs L148).
3. **Record 133 §2's claim that unpinned core-sharing averages out inside a decode ("a shift, not a jitter") is contradicted by §86:**
   - Record 133 §2's proof that common-mode clock noise leaves the knee sharp is **exact and valuable**.
   - However, its conclusion that unpinned placement noise averages over 1,724 blocks into a deterministic `1.04×` shift is refuted by §86 (`batched-bits`): with the 100 ms pre-spin holding the clock ratio at `1.0000` everywhere, unpinned `8–12` threads *still* showed a paired SD of **4–5%** (`±2–3%` intervals), versus **1.1%** when pinned + spun (`clock-mechanism` SPIN). Stage 1 (`character-stage1/report.txt` L220–239) explains why: the scanner sat on **1 CPU 100% of the time** in every decode, so whether a busy worker lands on its sibling persists across tens of milliseconds or entire decodes — producing genuine **decode-to-decode differential jitter** (and confirming why Record 132 §1's soft-max reading for unpinned runs should not be discarded completely!).
4. **Record 130 §2's within-block speculative factor (`≥ 50 × 1.5 ns`) slightly overstates the multiplier on real selector schedules:**
   - Because `scan-profile-report.txt` records only `438` table switches in `1,163` selectors/block (`62.3%` of adjacent 50-symbol groups keep the same table), a boundary inside a multi-group constant-selector run does not immediately diverge across all 50 count-mod hypotheses until the next table switch. The conclusion that `P2` loses badly to `P1` for all practical $k$ still holds cleanly, even with a smaller true multiplier (`~4–15×` rather than strictly `≥ 50×`).

### 4.3 What MONODROMY Missed

1. **The two-halves split of `A7` (emission) and its consequence for the matrix floor:**
   - MONODROMY treats `A7` (`38%` of worker time) and `supplied-codes` (`17%` of wall at $k=1$) without noticing the `21%` gap between them! Reading `src/open.rs:919-986` against `measurements-four-arm.tsv` reveals that `supplied-codes` removes only the `edges` pointer-chase (`44.4%` of `A7`, `0.218 s` at $k=1$), leaving **`55.6%` of `A7` (`0.272 s` at $k=1$, `~41–46 ms` at $k=8\text{–}10$)** in `inline16` word lookup + `u128` backward shifting + 16-byte output stores.
   - This single fact predicts that the ideal-parts `all-ideal` floor (`SECMRD`) will sit at **`~0.245–0.33`** (NOT near zero!), and opens an orthogonal non-G1 optimisation target inside `A7`.
2. **The code-level anatomy of `C` (`45%` of worker time) — especially the joint `(A6c + A7)` payoff of replacing XOR-bidirectional with $L$-lane forward emission:**
   - Record 130 §6 identified `C = 0.45` (`0.16 + 0.17 + 0.13`) from the ledgers, but did not inspect `src/open/` to diagnose the three sub-phases:
     - **`A6c` (`12.7%`, `161 ms` at $k=1$)** pays `m = 106.3M` scattered random L3 RMWs (`edges[v] ^= u`, `src/open/outq.rs:262`) + `edges.fill(0)` *solely* to enable 2-lane XOR emission. Combining MONODROMY's own Record 129 §1.3 saturation law ($L^* \approx 5\text{–}10$) with our `src/open/outq.rs` audit shows that **$L$-lane forward-only emission on unidirectional `edges` kills `A6c`'s $m$ scattered RMWs AND raises `A7`'s MLP from 2 toward $L^*$ in one move!**
     - **`A5 + A6a` (`25.8%`)** chops maximal runs at `CAP = 127` into 180 MB of `u16` records in `src/runs.rs` only for `src/open.rs:119` (`maximal()`) to immediately re-merge them — a direct Bird–Meertens deforestation win (`~9–12%` of worker time) that Record 130 §3's fusion table missed (it listed Huffman–MTF and walk–RLE1 fusion, but not `A5 + A6a`!).
     - **`A6b` (`16.7%`, `216 ms` at $k=1$)** spends **`~103 cycles/IET-step`** across only `~10.3M` IET steps, implicating `Dictionary::code` double-hashing and rule expansion (`src/open/endpoint.rs:516-590`) rather than IET itself.
3. **Two matrix-specific phenomena ahead of the run:**
   - **The `C × E` sign flip (`0.84` negative log-convexity at $k=1$ → `1.17` positive scanner-wall censoring at $k=6$) and `S × C` unmasking already at $k=6$ (`0.76`)**: Because `C` (`45%`) is large enough to push $T_{\text{wkr}}$ below $T_{\text{scan}}$ even at $k = 6$, `C` breaks the neat "workers bind at 6, scanner binds at 8–10" split! And at $k=1$, Jain's log-wall sign table will flag `C × E` and `C × M` as strong negative interactions purely from log-convexity of additive same-station cuts unless linear-time interactions are reported alongside log-wall.
   - **The `R` oracle's 839 MB copy-handover trap:** At $k \ge 6$, real `A9` is only `15.1 ms`/worker, whereas copying `838.8 MB` from a cold `210 MB` DRAM store across 6 workers takes `~22–35 ms` wall — making a 1-copy `ideal-rle1` oracle measure *slower* than real (`~1.02–1.05`) unless implemented with zero-copy slice/`Arc` handover!
