# Wormhole Cycle 20 — Whole-Kernel $\mathsf{BMCP}_{\mathrm{CML}}$ Bounds, Full Multi-Worker Confrontation & 10-Cycle Round 3 Handoff (`WormholePreEmitCells`)

**Session:** `research/carbon-wormhole` (Round 3, Cycle 20 of 20; 10th and final cycle of Round 3)  
**Baseline `origin/master`:** `90e6719663af0615fb417b038906ad2636193aa3` (`#692` memory diet + `#703` dense admission + `EdgesMux` + certified `SPIRALS`)  
**Verification harness:** [`verify_cycle20.rs`](2026-11-21-wormhole-cycle20-whole-kernel-bounds-and-handoff/verify_cycle20.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `manifest.json` SHA-256 auth + `14,466` exhaustive/stress/spiral/wide/dense cases + `21`-block `BzReader::parallel_workers_with` stream at `W = 1, 2, 4, 8` + `25`-round 1w & 2w barrier confrontation), [`wormhole_round3_final.toml`](2026-11-21-wormhole-cycle20-whole-kernel-bounds-and-handoff/wormhole_round3_final.toml) (`PASS`), [`cycle20_output.txt`](2026-11-21-wormhole-cycle20-whole-kernel-bounds-and-handoff/cycle20_output.txt), [`cycle20_results.json`](2026-11-21-wormhole-cycle20-whole-kernel-bounds-and-handoff/cycle20_results.json)

---

## 1. Complete Round 3 One-Line Keep & Kill Ledger (`Cycles 11–20`)

Every route tested in Cycles 11–20 against `origin/master` (`90e671966`, holding `S5 emit` **100% verbatim untouched** per non-duplication discipline):

### Promoted into `WormholePreEmitCells: InverseBwt` (`5` Orthogonal Links)
- **PROMOTE `L1: Constant-Sentinel pop_tail` (`Cycle 15`):** Replaces dynamic `L::next(id) == 0` loads with constant `0` in `Elim::eliminate`, eliminating sentinel-0 store-to-load alias stalls across all `7.1k–21.3k` RV steps (`-20%` to `-23%` on `S2b`, `0 B` scratch).
- **PROMOTE `L2: (len10 << 22) | id22` Graded Free-Monoid Semiring Handle (`Cycle 15`, Theorem 15.1):** Provably overflow-safe for all $n \le 900{,}000$ ($\mathrm{id} \le 1{,}800{,}256 < 2^{22}$, saturating `1023` fallback for deep spirals), eliminating both random `arena[rule+2]` loads per `Elim::join` (`-27.8%` to `-31.9%` total `S2b` cut, `0 B` arena growth).
- **PROMOTE `L3: Lazy-Prefix15-DirectInline16` (`Cycle 17`, Theorem 17.1):** Exploits non-erasing SLP tree bound ($\le 14$ internal nodes, fixed `[u32; 16]` stack for $|w| \le 15$) to unfold `64%–78%` of words directly into `inline16`, shrinking `spelling` by `-8%` to `-25%` and **deleting Stage `S3b` (`inline_words`) completely (`0.0 µs`)**.
- **PROMOTE `L4: Pohl (1971) 2-Way Bidirectional Side-1 Bottom Walk` (`Cycle 18`):** Walks circular doubly-linked `side-1` simultaneously from `0` forward and `m` backward (`D = H / 2`, `MLP = 2`, `0 B` scratch).
- **PROMOTE `L5: Chaitin [u32; 3] StripBandMux` (`Cycles 18–19`):** Packs thick `Strip` as `12 B` `[from, to, (width << 8) | byte]` multiplexed with `12 B` `Band` `[end, delta, code]` in `sb_buf: Vec<[u32; 3]>`, deleting `Vec<Band>` and cutting per-worker working set by **`-23` to `-125 KiB`** (`1.355–1.853 MiB`/worker on Wikidata).

### Killed Routes (`11` Honest Kills, One Line Each)
- **KILL `S4-First Bottleneck Hypothesis` (`Cycle 11`):** `S4 Wire` is only `11%–13%` of pre-`emit` (`90–185 µs`) because `28–64`-row band spans + Comet Lake's `4 KiB` streamer hide `85.2%–86.5%` of its L2 misses.
- **KILL `Pointer-Walk MU > 8` (`MU = 10, 12, 16`, `Cycle 12` Theorems 12.1–12.2):** Exceeds `15` usable x86-64 GPRs (triggering stack spills and `(%rsp,%r14,4)` SROA collapse for `+35%` to `+114%` slowdown) and saturates Comet Lake's `10` L1d LFBs (`effective cap 7.4–7.8`).
- **KILL `B3: Aggarwal–Vitter 8 KiB L1d Write-Combining` (`Cycle 13`):** Regresses `1.058×–1.116×` (`+163..335 µs`) because `wc_len`/`wc_buf` L1d bookkeeping exceeds `OutQ16` store latency.
- **KILL `B1/B2: Manual u64 SWAR8-Split4 / SWAR32 OutQ16 & Manual AVX2-Wire8` (`Cycle 13` Theorem 13.1):** `origin/master` already compiles `outq.rs:128` to a single 256-bit `vpaddw + vmovdqu %ymm0` (`1` store uop/run) and `outq.rs:245` to 4× `vpmovzxwd`; `w <= 4` branching loses `4.5%–10.8%` branch mispredicts (`0.992×–1.022×`).
- **KILL `B1: Unqueued (QUEUE=false)` (`Cycle 14` Theorem 14.1):** Regresses `1.261×–1.429×` (`S4` explodes `5.3–23×` from `33–184 µs` to `465–1,468 µs` due to `1.7`-row thin-run scatters replacing `28–64`-row `Band` SIMD stores).
- **KILL `B2/B3: Kakutani-1Stream (G=1) & Radix16-Kakutani (G=16)` (`Cycle 14` Theorem 14.1):** Regress `1.165×–1.285×` because any $G < 256$ sacrifices symbol-pure `Band` coalescing in `S4`.
- **KILL `B1: Tarjan Post-Order RULE2 Elimination` (`Cycle 15`):** Saves `2` words/rule in `S2` but loses DAG memoization `rule[3]`, regressing `S3a` by `+17..37 µs` vs `GradedRV`.
- **KILL `B2/B3: DietTileDir (INDUCE=false) & Tarjan Full-Tile Path Compression` (`Cycle 16` Theorems 16.1–16.2):** Regress `1.041×–1.204×` vs `M0` because `56 KiB` `O(n/64)` directory probes + `13%–16%` boundary-straddling strips cost more than `GradedRV` dynamic splitting.
- **KILL `B1: Eager-u128-Prefix16 in Elim::join (RULE8 = 32 B)` (`Cycle 17`):** Doubling rule stride to `32 B` (`+100..256 KiB` arena) and touching dead intermediate RV rules gives back `+35..96 µs` on `S2`, losing to demand-driven `Lazy15`.
- **KILL `B2: FusedS3S4-TailArena` (`Cycle 18` Theorem 18.1):** Holding `edges[0..m)` and `Elim` live simultaneously inflates per-worker working set by `+156..493 KiB` (`up to 2.37 MiB/w -> 23.7 MiB > 20 MiB L3 at 10w!`) and regresses `S3+S4` on `6/6` blocks via L1d/LFB interference.
- **KILL `U1/U2: Upstream runs::front CAP=255 (u16) & Unsplit-u32` (`Cycle 19` Theorem 19.1):** `CAP=127` adds only `+2.4%–10.2%` records on RLE1 BWT blocks (`CAP=255` saves `<0.5%` while breaking `cells_lazy` `len<=127`; `Unsplit-u32` costs `+89..345 KiB`/worker).

---

## 2. Charge 1 & Charge 2 — Tightened Whole-Kernel $\mathsf{BMCP}_{\mathrm{CML}}$ Bounds (`6` Costs Kept Separate)

### Tightened Comet Lake Microarchitectural Laws (`Cycle 12`, `proved` + `verified` via `--emit=asm`)
1. **16-Register Ceiling (`15` Usable GPRs, Theorem 12.1):** An unrolled walk holds `7` live loop invariants (`edges_ptr`, `out_ptr`, `inl_ptr`, `lens_ptr`, `mask`, `code_mask`, `cnt`), leaving **exact `8` GPRs** for parallel cursors `j[0..8]`. At `MU = 8`, `0` stack spills occur; at `MU = 10`, `j[8..10]` spill (`+35.4%`); at `MU = 12, 16`, LLVM abandons SROA (`(%rsp,%r14,4)`, `+49%` to `+114%`). Similarly, inlining 4 copies of `Queues::push` into `S1`'s `maximal()` triggers an `+80%` spill cliff (`Cycle 19`).
2. **Line-Fill-Buffer Contention (`10` L1d LFBs, Theorem 12.2):** Output RFO line allocations + `inline16`/`spelling` loads occupy `2.2–2.6` LFBs per step, capping pointer-walk memory-level parallelism at $\mu_{\mathrm{eff}}^{\mathrm{LFB}} = 10 - L_{\mathrm{aux}} \in [7.4, 7.8]$; in `S1`, `256` active `OutQ16` `1 KiB` chunks exceed the `64`-entry L1 DTLB (`107–223` DTLB misses).
3. **Hardware Prefetcher (`32`-Page `4 KiB` L2 Streamer, Theorem 12.3):** Tracks up to `32` concurrent `4 KiB` pages, hiding **`85.2%–86.5%`** of `S4 Wire` L2 misses, **`64.9%–71.9%`** of `S1`, **`42.4%–54.7%`** of large-arena `S2/S3`, and **only `3.3%–7.3%`** of `S5 Emit` (`random pointer jumps across 346–807 KiB`).

### Whole-Kernel 5-Stage Cost Table Across the `6` Authenticated Wikidata Blocks (`n ≈ 900,000`)

All **six costs kept strictly separate** (`LB` = $\mathsf{BMCP}_{\mathrm{CML}}$ lower bound within exact induced first-return class; `Master` = `origin/master` `90e671966`; `W3` = `WormholePreEmitCells` with `Master` `emit`; `[+R8]` = orthogonal Round 2 `DietRelay8Cells` bound on `S5`):

| Stage | Random-Access Count $Q_{\mathrm{rand}}$ (`LB` / `Master` / **`W3`**) | Serial Dep. Depth $D_{\mathrm{serial}}$ (`LB` / `Master` / **`W3`**) | Unprefetched L2 Misses $M_{\mathrm{L2}}^{\mathrm{eff}}$ & L3 (`Master` -> **`W3`**) | Preprocessing $W_{\mathrm{prep}}$ (`Master` -> **`W3`**) | Memory Footprint $M_{\mathrm{ws}}$ (`Master` -> **`W3`**) | Parallel Work $W_{\mathrm{par}}$ (`Master` -> **`W3`**) | Wikidata Wall Time (`Master` -> **`W3`**, `1w`) |
|---|---|---|---|---|---|---|---|
| **`S1: First Pass + OutQ16`** | `r_thin` (`28k–54k`) / `37k–84k` / **`37k–84k`** | `0` (data-dep) / `R_127` scan / **`R_127` scan** | `2.6k–5.4k` L2m (`65–72%` streamer-hidden), `~0` L3m | `0` -> **`0`** | `out` (`879 KiB`, `0 B` net) + **`49–98 KiB` `sb_buf`** (`-16..33 KiB` vs `Strip16`) | `O(R_127 + r_thin)` (`1` AVX2 `vmovdqu`/thin run) | `216–465 µs` -> **`207–463 µs`** *(optimal; Thms 13.1, 14.1, 19.1)* |
| **`S2: Strips + RV Elim`** | `ns + 2*S_RV` / `ns + 6*S_RV` / **`ns + 4*S_RV`** (`-2` rule loads/join!) | `S_RV` (`7.1k–21.3k`) / `S_RV` (+ sentinel alias) / **`S_RV` (`0` alias)** | `1.5k–11.8k` -> **`1.1k–8.8k`** L2m, `~0` L3m | `ns` radix sort (`4.1k–8.2k`) | `460–1,450 KiB` in dead `edges` (`0 B` extra) | `O(ns + S_RV)` | `143–473 µs` -> **`114–383 µs`** (**`-19%` to `-23%`**, `-28..32%` on `S2b`) |
| **`S3: Bottom + Bands + SLP`** | `H_crit + |T_long|` / `2*H_crit + |T_all|` / **`2*H_crit + |T_long|`** | `H_crit / 2` / `H_crit + |T_all|` / **`H_crit/2 + <=14`** (short) | `0.8k–4.7k` -> **`0.6k–3.5k`** L2m (`0.0 µs` cold `S3b`!) | `256` literal init (`once`) | `26–46 KiB` `spelling` (`-4..14 KiB`) + **`0 B` `Vec<Band>`** (`reuses sb_buf`!) | `O(n_bands + |spelling_long|)` (`skips 64–78%` short words!) | `120–398 µs` -> **`102–335 µs`** (**`-10%` to `-16%`**, `S3b = 0.0 µs`) |
| **`S4: OutQ16 Wire`** | `n_bands` (`3.3k–8.3k`) / `n_bands + cuts` / **`n_bands + cuts`** | `O(1)` / `O(1)` / **`O(1)`** | **`0.8k–1.6k`** unprefetched L2m (`85.2–86.5%` streamer-hidden!) | `0` -> **`0`** | `346–807 KiB` `edges[0..m)` over dead arena (`0 B` extra) | `O(m)` (`4× vpmovzxwd` AVX2) | `80–184 µs` -> **`80–185 µs`** *(optimal; Thms 12.3, 14.1)* |
| **`S5: Walk / Emit`** *(held fixed in `W3`)* | `m` (`86k–202k`) / `m` | `m / 7.6` (`CML` floor) / `m` (`Master` & `W3`) `[m/3.6 in Relay8]` | `23.8k–91.6k` unprefetched L2m (`93–97%` exposed!), **`479–831` L3m** | `0` (`Master`/`W3`) `[O(S) cuts in Relay8]` | `out + edges + inline16 + spelling` = **`1.355–1.853 MiB`/w** (`10w <= 18.53 MiB <= 20 MiB` L3!) | `O(m + n)` (`1` 16B copy/step) | `513–1,367 µs` *(100% untouched in `W3`; `-8..29%` in Round 2 `Relay8`)* |

---

## 3. Cycle 20 Full-Suite Verification & `1w` / `2w` Multi-Worker Confrontation (`measured` in [`verify_cycle20.rs`](2026-11-21-wormhole-cycle20-whole-kernel-bounds-and-handoff/verify_cycle20.rs))

- **Authentication (`manifest.json`):** `6/6` Wikidata blocks verified (`n`, `primary_index`, `SHA-256`).
- **Exhaustive & Stress Suite:** **`14,466` cases byte-exact** (`14,204` binary $n \le 9$ & ternary $n \le 6$ at every $p$ + `250` random multi-cycle/periodic/multi-epoch $m > 65{,}536$ + `SPIRALS` `widest = 50,000` & `len > 1,023` + forced `Wide` links & `wide` `extra` + `#703` `Cap::dense` admit/refuse + column `invert`/`invert_in_place`).
- **Multi-Worker `BzReader::parallel_workers_with::<WormholePreEmitCells>` (`W = 1, 2, 4, 8`):** **`21`-block multi-regime `.bz2` stream** (`15.49 MiB` unrled, `CRC32 = 0x26ecd15a`, spanning normal `Open`, `#703` dense `Open`, `#703` refused -> `LazyCells`, and high-entropy column fallback) **100% byte-exact at all worker counts**.
- **Per-Worker Working Set (`W3`):** **`1,388–1,897 KiB` (`1.355–1.853 MiB`)** on Wikidata (**`-23` to `-125 KiB`/worker** below `origin/master`; `10` i9 workers = **`18.53 MiB <= 20 MiB` L3**).
- **DRAM Traffic:** **`0.034–0.059 B/B`** (`479–831` cold `2048K` L3 misses/block, `0` extra DRAM bytes).
- **Acquisition Cost:** **~85 lines of `#![forbid(unsafe_code)]` safe Rust** confined to `src/open/endpoint.rs` and `src/open.rs`.

| Block | WS (`M0 -> W3 KiB`) | `1w A1:Master` | `1w A2/A1` | **`1w W3:PreEmit`** | **`1w W3 / M0`** | **`1w W3 / A1`** | `2w A1:Master` | `2w M0:Clone` | **`2w W3:PreEmit`** | **`2w W3 / M0`** | **`2w W3 / A1`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1445 -> 1388` (`-57`) | `1310.1 µs` | `0.998x` | **`1215.7 µs`** | **`0.933x`** | **`0.928x`** | `1320.0 µs` | `1305.5 µs` | **`1219.6 µs`** | **`0.934x`** | **`0.924x`** |
| `block-2.L` | `1590 -> 1558` (`-32`) | `1552.6 µs` | `0.993x` | **`1408.0 µs`** | **`0.913x`** | **`0.907x`** | `1549.7 µs` | `1544.2 µs` | **`1427.8 µs`** | **`0.925x`** | **`0.921x`** |
| `block-85.L` | `1640 -> 1617` (`-23`) | `1546.4 µs` | `0.989x` | **`1439.8 µs`** | **`0.961x`** | **`0.931x`** | `1543.7 µs` | `1492.9 µs` | **`1434.3 µs`** | **`0.961x`** | **`0.929x`** |
| `block-143.L` | `2022 -> 1897` (`-125`) | `2769.9 µs` | `0.987x` | **`2547.0 µs`** | **`0.943x`** | **`0.920x`** | `2757.2 µs` | `2666.4 µs` | **`2510.5 µs`** | **`0.942x`** | **`0.911x`** |
| `block-210.L` | `1964 -> 1894` (`-70`) | `2391.4 µs` | `1.000x` | **`2231.2 µs`** | **`0.962x`** | **`0.933x`** | `2394.3 µs` | `2320.9 µs` | **`2227.9 µs`** | **`0.960x`** | **`0.930x`** |
| `block-207.L` | `1697 -> 1659` (`-38`) | `1866.4 µs` | `0.975x` | **`1690.5 µs`** | **`0.942x`** | **`0.906x`** | `1839.9 µs` | `1861.7 µs` | **`1696.9 µs`** | **`0.911x`** | **`0.922x`** |
| `dense-703.syn` | `2026 -> 2028` (`+2`) | `2869.8 µs` | `1.049x` | **`2635.3 µs`** | **`0.940x`** | **`0.918x`** | `2966.8 µs` | `2793.4 µs` | **`2705.1 µs`** | **`0.968x`** | **`0.912x`** |

---

## 4. Clean Coordinator Port Specification (`mibz2 0.5.11+`)

`WormholePreEmitCells` ([`verify_cycle20.rs:1084-1410`](2026-11-21-wormhole-cycle20-whole-kernel-bounds-and-handoff/verify_cycle20.rs)) ports into production `src/open/endpoint.rs` + `src/open.rs` in four self-contained, orthogonal steps (and composes **100% additively** with Round 2's `DietRelay8Cells` or Mechanisms' `emit`, since `(edges[..m], extra, spelling, offsets, inline16, lens, mark, bits)` satisfies the identical post-condition):
1. **`src/open/endpoint.rs` (`Elim::pop_tail` + `Elim::join` Graded Handle):** Add `pop_tail(id, side)` (`2` constant-`0` link stores) and encode non-zero words as `(exact_len.min(1023) << 22) | id22` (`seed literals in induced_with as (1 << 22) | (u32::from(s_b) + 1)`).
2. **`src/open/endpoint.rs` (`compute_bottom_bidi`):** Replace the serial `side-1` `bottom` loop with the 20-line Pohl 2-way `f_id`/`b_id` walk (`reading f_next and b_prev before set_bottom`).
3. **`src/open/endpoint.rs` + `src/open.rs` (`emit_short_or_long` & Delete `inline_words`):** In `induced_with`'s `side-0` band loop, unfold `h_len <= 15` directly into `inline16` via `[u32; 16]` (`0`-load fast path for `h_len == 1`, `1`-rule fast path for `h_len == 2`), run `append` into `spelling` only when `h_len >= 16` (`lens.push(LONG)`), and remove the `inline_words(...)` call in `Kernel::walk`.
4. **`src/open.rs` (`sb_buf: Vec<[u32; 3]>` `StripBandMux`):** Replace `strips: Vec<Strip>` + `bands: Vec<Band>` with `sb_buf: Vec<[u32; 3]>` (`[from, to, (width << 8) | u32::from(byte)]` in `S1..S2a`, `.clear()` once `order[1]` is built, then `[end, delta, code]` in `S3..S4`, shrinking to `cap.strips` on `#703` refusal).
