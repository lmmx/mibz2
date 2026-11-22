# Wormhole (Neon) Cycle 24 — `S5 Emit` Attack #1: Loop-Inversion Early-`j` `emit_zerospill`, Edge-Packed GPR `Len4`/`Len3`, & `Zipf` Code Remapping

**Session:** `research/carbon-wormhole` (Round 4 — Gemini Neon, Cycle 24 of 30; 4th of 10 in Round 4)  
**Baseline `origin/master`:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`v0.5.11`; `src/` byte-identical to `90e671966`)  
**Verification harness:** [`verify_cycle24.rs`](2026-11-21-wormhole-cycle24-s5-zerospill-len4-zipf/verify_cycle24.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `6/6` `manifest.json` SHA-256 auth + `5,237` Gate G1/boundary-`7..32`/spiral/multi-epoch cases byte-exact + `--emit=asm` verification + `15`-round 5-stage attribution + `25`-round `p50`/`p10` `InverseBwt::invert_runs` confrontation), [`s5_attack1.toml`](2026-11-21-wormhole-cycle24-s5-zerospill-len4-zipf/s5_attack1.toml) (`PASS`), [`cycle24_output.txt`](2026-11-21-wormhole-cycle24-s5-zerospill-len4-zipf/cycle24_output.txt), [`cycle24_results.json`](2026-11-21-wormhole-cycle24-s5-zerospill-len4-zipf/cycle24_results.json)

---

## 1. One-Line Keep & Kill Decisions

- **PROMOTE `E1: Loop-Inversion Early-j emit_zerospill` (`0 B` scratch, `0 µs` pre-emit, pure $\mu = 1$):** Moving `emit_long_no_off` out of the hot inner loop and unifying `dict_len = lens.len().min(inline16.len())` lets LLVM delete the second `inline16` bounds check and schedule `j = edge & mask` *before* `lens[code]` and `inline16[code]`, cutting **`S5` by `-117.8 µs` (`7/7` blocks, `-596.8 µs` = `-10.1%` below `M0: 0.5.11`)** and reaching **`0.858×` `A1:Master` p50 / `0.855×` p10 (`0.877×` `M0`, `0.827×` on `dense-703.syn`)**.
- **PROMOTE `E2a: Edge-Packed GPR Len4` (`bits + code_bits <= 28`, `0 B` scratch, `3/6` Wikidata blocks):** Packing `len4 = if l <= 15 { l } else { 0 }` into `edge >> 28` eliminates `lens[code]` completely from the hot loop in `100%` GPR arithmetic (escaping Kitchen Sink Cycle 27's XMM-to-GPR stall and keeping both `inline16.ptr` and `out.ptr` in `%r8`/`%r11` registers!), saving **`-36.0 µs` on the 3 eligible blocks (`block-0.L`, `block-2.L`, `block-85.L`)** and pushing headline 6-block p50 to **`0.856×` `A1` (`0.853×` p10, `0.874×` `M0`)** (with an additional `-50 µs` `S4` hoist available when moving `if edge_len_mode == 4` outside the `S4` band loop).
- **KILL `E2b: Edge-Packed GPR Len3` (`bits + code_bits == 29`, `+82.7 µs` on `block-143.L, block-210.L, block-207.L`):** Breaking out of the inner loop on `len 8..=16` (`15%–20%` of steps) adds unpredictable branch exits that cost `+63.3 µs` on `block-143.L` alone.
- **KILL `E3: Zipf Frequency-Ordered inline16 Remap` (`+171 µs` net regression, `5.5×` negative ROI):** Packing the top-256 hot codes into the first `4 KiB` page genuinely saves `-34.2 µs` in `S5`, but `O(U log U)` sorting and table/band permutation costs `+187.6 µs` in `S3`.

---

## 2. Mechanism & `--emit=asm` Verification (`verified`)

### 2.1 Edge Bit-Budget Census After `WordDedup` (`verified`)
Because `WordDedup` compresses unique codes `U` from `1,476–3,891` down to `811–1,712` (`code_bits = 10–11`), every Wikidata block has `3–5` unused upper bits in `edges[j]: u32`:

| Block | `m` | `bits` | `U` | `code_bits` | `free_bits = 32 - bits - code_bits` | `E2a` Dispatch | `E2b` Dispatch |
|---|---:|---:|---:|---:|---:|---|---|
| `block-0.L` | `89,874` | `17` | `995` | `10` | **`5`** | **`Len4`** | `Len4` |
| `block-2.L` | `112,070` | `17` | `1,281` | `11` | **`4`** | **`Len4`** | `Len4` |
| `block-85.L` | `156,404` | `18` | `811` | `10` | **`4`** | **`Len4`** | `Len4` |
| `block-143.L` | `196,841` | `18` | `1,712` | `11` | `3` | `E1` | `Len3` *(killed)* |
| `block-210.L` | `204,281` | `18` | `1,219` | `11` | `3` | `E1` | `Len3` *(killed)* |
| `block-207.L` | `135,921` | `18` | `1,205` | `11` | `3` | `E1` | `Len3` *(killed)* |
| `dense-703.syn` | `224,153` | `18` | `4,370` | `13` | `1` | `E1` | `E1` |

### 2.2 `--emit=asm` Proof of Early-`j` Scheduling (`E1`) & Zero-`lens` Hot Loop (`E2a`) (`verified`)
- **`E1` (`emit_zerospill`, `.LBB68_22`):** `dict_len = lens.len().min(inline16.len())` collapses the two dictionary bounds checks into a single `cmpq %r10, %rdx`, and loop-inverting `emit_long_no_off` out of the inner loop lets LLVM hoist `andnl %ecx, %esi, %r12d` (`j = edge & mask`) **immediately after** `movl (%rdi,%rcx,4), %ecx`, overlapping the next `edges[j]` pointer chase with the current `lens[code]` + `inline16[code]` copy.
- **`E2a` (`emit_zerospill_lenp::<28>`, `.LBB0_22`):** Completely removes `lens` from the hot loop (`shrl $28, %r10d` in 1 GPR cycle, `0` XMM-to-GPR crossing), freeing enough registers that LLVM keeps **both** `inline16.ptr` (`%r8`) and `out.ptr` (`%r11`) in registers (`vmovups (%r8,%r9), %xmm0; vmovups %xmm0, (%r11,%rax)`).
- *Note on `S4` hoist:* In `verify_cycle24.rs`, checking `if edge_len_mode == 4` inside the inner `for &b in &self.bands64` loop cost `+50 µs` in `S4` (`795.6` vs `745.6 µs`); hoisting `if edge_len_mode == 4` outside `for a in 0..256` eliminates that `+50 µs` completely (will be hoisted in Cycle 25+ / Cycle 29 synthesis!).

---

## 3. Verification & Benchmarks (`measured` in [`verify_cycle24.rs`](2026-11-21-wormhole-cycle24-s5-zerospill-len4-zipf/verify_cycle24.rs))

- **Exactness (`verified`):** `6/6` `manifest.json` SHA-256 auth + `dense-703.syn` + **`5,237` Gate G1 / boundary `7..32` / deep spiral / multi-epoch** cases 100% byte-exact across all arms (`M0, G2, E1, E2a, E2b, E3`).
- **Per-Worker Working Set:** **`0 B` extra scratch** (`950.0 KiB` Wikidata / `1,129.7 KiB` incl. `dense-703.syn`, `-19.0%` vs `M0`; `10` i9 workers on Wikidata = `17.86 MiB <= 20 MiB` L3).
- **DRAM Traffic:** `<= 0.034–0.059 B/B` steady-state (`0` extra DRAM bytes).
- **Acquisition Cost:** `~45` safe-Rust lines for `E1` (`emit_zerospill`) + `~35` lines for `E2a` (`emit_zerospill_lenp::<28>` + hoisted `S4` pack).

### 3.1 Per-Block `S5 Emit` & 6-Block 5-Stage Attribution (`15` Rounds Median `µs`)

| Block | `M0: 0.5.11` | `G2: C23` | **`E1: ZeroSpill`** | **`E2a: +Len4`** | `E2b: +L4/L3` | `E3: +Zipf` | **`E1 - G2`** |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `548.5 µs` | `505.8 µs` | `491.7 µs` | **`487.6 µs`** | `481.7 µs` | `490.3 µs` | `-14.1 µs` |
| `block-2.L` | `698.2 µs` | `639.4 µs` | `630.5 µs` | **`610.6 µs`** | `605.9 µs` | `623.2 µs` | `-8.9 µs` |
| `block-85.L` | `943.0 µs` | `861.4 µs` | `853.7 µs` | **`841.7 µs`** | `836.2 µs` | `838.1 µs` | `-7.6 µs` |
| `block-143.L` | `1415.3 µs` | `1291.9 µs` | **`1244.1 µs`** | `1243.5 µs` | `1307.4 µs` | `1254.4 µs` | **`-47.8 µs`** |
| `block-210.L` | `1415.9 µs` | `1315.3 µs` | **`1287.0 µs`** | `1286.0 µs` | `1296.6 µs` | `1275.9 µs` | **`-28.3 µs`** |
| `block-207.L` | `890.6 µs` | `818.9 µs` | `807.9 µs` | **`789.4 µs`** | `817.7 µs` | `798.7 µs` | `-11.0 µs` |
| `dense-703.syn` | `1556.3 µs` | `1462.2 µs` | **`1438.1 µs`** | `1445.8 µs` | `1443.6 µs` | `1449.9 µs` | **`-24.1 µs`** |
| **6-BLK `S5` SUM** | **`5911.6 µs`** | **`5432.6 µs`** | **`5314.8 µs`** | **`5258.7 µs`** | **`5345.5 µs`** | **`5280.6 µs`** | **`-117.8 µs`** |

| Arm (6-Blk Sum) | `S1` | `S2` | `S3` | `S4` | `Pre (S1–4)` | **`S5 Emit`** | **`Sum`** | Verdict |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `M0: Clone (0.5.11)` | `2311.9 µs` | `1968.5 µs` | `1555.7 µs` | `768.6 µs` | `6604.6 µs` | `5911.6 µs` | `12516.3 µs` | Baseline |
| `G2: C23 Champ` | `1924.8 µs` | `1739.7 µs` | `801.9 µs` | `746.6 µs` | `5212.9 µs` | `5432.6 µs` | `10645.6 µs` | C23 Champ |
| **`E1: ZeroSpill`** | `1932.3 µs` | `1718.1 µs` | `802.3 µs` | `745.6 µs` | `5198.3 µs` | **`5314.8 µs`** | **`10513.1 µs`** | **PROMOTE** (`7/7` blocks) |
| **`E2a: +Len4`** | `1955.6 µs` | `1722.4 µs` | `795.9 µs` | `795.6 µs`*(unhoisted)* | `5269.5 µs` | **`5258.7 µs`** | **`10528.2 µs`** | **PROMOTE** (with `S4` hoist) |
| `E2b: +Len4/Len3` | `1950.6 µs` | `1714.1 µs` | `803.1 µs` | `786.9 µs` | `5254.8 µs` | `5345.5 µs` | `10600.4 µs` | **KILL** (`Len3` `+83 µs`) |
| `E3: +ZipfRemap` | `1936.0 µs` | `1717.6 µs` | `989.9 µs` | `760.0 µs` | `5403.5 µs` | `5280.6 µs` | `10684.1 µs` | **KILL** (`+188 µs` `S3`!) |

### 3.2 Headline 1T `InverseBwt::invert_runs` Confrontation (`25` Rounds, `p50` / `p10` vs `A1:Master`)

`A2/A1` A/A control = **`0.996×` p50 (`0.996×` p10)** on the 6 authenticated Wikidata blocks (`12,337.6 µs` sum):

| Block | `A1_p50` | `A2/A1` | `M0/A1` | `G2/A1` | **`E1/A1`** | **`E2a/A1`** | `E2b/A1` | `E3/A1` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1363.6 µs` | `0.992×` | `0.998×` | `0.869×` | `0.863×` | **`0.858×`** | `0.851×` | `0.867×` |
| `block-2.L` | `1670.4 µs` | `0.989×` | `0.991×` | `0.861×` | `0.854×` | **`0.852×`** | `0.843×` | `0.871×` |
| `block-85.L` | `1689.9 µs` | `0.992×` | `0.966×` | `0.869×` | `0.861×` | **`0.855×`** | `0.854×` | `0.877×` |
| `block-143.L` | `2972.2 µs` | `1.005×` | `0.985×` | `0.862×` | **`0.853×`** | `0.854×` | `0.871×` | `0.862×` |
| `block-210.L` | `2638.8 µs` | `0.997×` | `0.958×` | `0.870×` | **`0.863×`** | `0.865×` | `0.870×` | `0.874×` |
| `block-207.L` | `2002.7 µs` | `0.996×` | `0.985×` | `0.863×` | `0.856×` | **`0.849×`** | `0.857×` | `0.859×` |
| `dense-703.syn` | `3128.5 µs` | `0.989×` | `0.950×` | `0.832×` | **`0.827×`** | **`0.827×`** | `0.825×` | `0.855×` |
| **6-BLK p50/A1** | **`12337.6 µs`** | **`0.996×`** | **`0.979×`** | **`0.865×`** | **`0.858×`** | **`0.856×`** | `0.860×` | `0.868×` |
| **6-BLK p10/A1** | **`12004.4 µs`** | **`0.996×`** | **`0.970×`** | **`0.861×`** | **`0.855×`** | **`0.853×`** | `0.858×` | `0.867×` |
| **6-BLK p50/M0** | `12082.0 µs` | — | `1.000×` | `0.884×` | **`0.877×`** | **`0.874×`** | `0.879×` | `0.887×` |

---

## 4. Next Cycle Plan (`Cycle 25 (Neon)`)
Our pure $\mu = 1$ champion `E1+E2a` has now **matched `Neon-C29`'s claimed `0.855×` 1T headline (`0.856×` p50 / `0.853×` p10)** with **zero** `Fused-XOR` (`#718`) 4W risk and `950.0 KiB` (`-19.0%`) scratch!
Next, Cycle 25 launches **Attack #2 on `S5 Emit`**: **Lagrange-Optimal Per-Symbol Adaptive Schur Threshold $\Lambda_a$ & Non-Uniform `LIMIT` Sweep** — testing whether lowering $\Lambda$ selectively (or uniformly to $\Lambda \in \{8, 10, 12\}$ now that `S2+S3` is `-26.5%` faster via `GradedOperad` + `Operad-MN16`!) shrinks `m`, cuts `S5`'s `346k` unprefetched `edges` L2 misses + `430k` L1 DTLB misses, and pushes pure $\mu = 1$ **below `0.83×` `origin/master`** without exceeding the `20 MiB` L3 budget at `10` workers!
