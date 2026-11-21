# Wormhole Cycle 11 — Flooring the Four Pre-`Emit` Stages of `origin/master` (`#692` + `#703`), Stage-by-Stage $\mathsf{BMCP}$ Upper Bounds, and Gap Ranking

**Date:** 2026-11-21
**Branch:** `research/carbon-wormhole` (Round 3, Cycle 1 of 10)
**Baseline `origin/master` SHA:** `90e6719663af0615fb417b038906ad2636193aa3` (`mibz2 0.5.10` + `#692` memory diet + `#703` dense admission / `EdgesMux` / `SPIRALS`, pre-`0.5.11`)
**Artefacts:**
- Native safe-Rust A/A + 4-stage timing & $\mathsf{BMCP}$ harness (`#![forbid(unsafe_code)]`): [`verify_cycle11.rs`](2026-11-21-wormhole-cycle11-pre-emit-stage-floors-and-gaps/verify_cycle11.rs)
- Gate G2 baseline stage manifest (`PASS` via `check_manifest.py`): [`master_pre_emit_stages.toml`](2026-11-21-wormhole-cycle11-pre-emit-stage-floors-and-gaps/master_pre_emit_stages.toml)
- Raw JSON & stage tables: [`cycle11_results.json`](2026-11-21-wormhole-cycle11-pre-emit-stage-floors-and-gaps/cycle11_results.json), [`cycle11_output.txt`](2026-11-21-wormhole-cycle11-pre-emit-stage-floors-and-gaps/cycle11_output.txt)

**One-Line Cycle Verdict:** Baseline diagnostic established (`A2/A1 = 0.989–1.010×`); pre-`emit` already owns **`42.3%–59.6%`** of `origin/master` (`697–1,547 µs`), led by **S1 First Pass (`227–529 µs`)**, **S2 Thick-Strip Elimination (`141–452 µs`)**, and **S3 Band/SLP Extraction (`134–417 µs`)**, while **S4 Wire (`90–200 µs`)** is already within `1.07–1.44×` of its compulsory L2 floor (`28–64` rows/band).

---

## 1. The Four Pre-`Emit` Stages of `origin/master` (`90e671966`)

After `#692` (`OutQ16` in `out` + `Endpoint` in `edges`) and `#703` (`DENSE` admission up to $r/n < 0.28$, `EdgesMux` `image_order` in `edges[..ns]`, and width-certified `SPIRALS` past `32,768`), `OpenCells` executes four distinct stages before `emit` (`S5`):

1. **Stage S1 (`First Pass + OutQ16`, `open.rs:88-197`, `outq.rs:122-175`):** Scans `records: &[u16]` ($R$ `CAP=127` records), coalescing maximal runs ($r$ runs), pushing `16`-element `u16` rank blocks into `256` active `1 KiB` chunks inside dead `out[0..n+16)`, and appending thick runs ($w > 16$, $H = H_{16}$ strips) to `strips: Vec<Strip>`.
2. **Stage S2 (`Thick-Strip Elimination`, `open.rs:386-393`, `endpoint.rs:297-547`):**
   - **S2a (`Image-Order Scatter + Node Build`):** Scatters `image_order` into `edges[..ns]`, then scans `strips` twice and gathers `strips[image_order[k]]` randomly to build $\le 3H + 3$ `Narrow` (`16 B`) doubly-linked nodes in `edges[ns..]`.
   - **S2b (`Rauzy–Veech–Morse Elimination`):** Peels `prev(0, 0)` against `prev(0, 1)`, splicing `u16`-packed doubly-linked lists and appending `16 B` SLP `Rule`s `[left, right, len, first]` at `edges[ns + base ..]`.
3. **Stage S3 (`Bottom Walk, Band/SLP Extraction & inline_words`, `endpoint.rs:553-607`, `open.rs:717-751`):**
   - **S3a:** Walks side-1 surviving gap nodes writing `bottom(id)`, then walks side-0 gap nodes per symbol $a$, DAG-expanding SLP words into `spelling: Vec<u8>` + `offsets: Vec<u32>` and pushing `Band { end, delta, code }` (`12 B`) into `bands: Vec<Band>`.
   - **S3b (`inline_words`):** Executed *after* S4 (`wire`), re-scanning `offsets` and `spelling` to populate `inline16: Vec<[u8; 16]>` and `lens: Vec<u8>`.
4. **Stage S4 (`OutQ16 Band Wire`, `open.rs:470-485`, `outq.rs:239-296`):** Reads `u16` ranks sequentially per symbol from `out`'s `1 KiB` chunks and writes `edges[dst .. dst + span]`.

---

## 2. Stage-by-Stage $\mathsf{BMCP}$ Lower Bounds (`proved`)

Keeping all six costs separate ($Q_{\mathrm{rand}}$, $D_{\mathrm{serial}}$ / $D_{\mathrm{past-}M_2}$, $C_{\mathrm{miss}}(M)$, $W_{\mathrm{prep}}/D_{\mathrm{prep}}$, $S_{\mathrm{mem}}$, $W_{\mathrm{par}}$) with $B = 64\text{ B}$:

### Theorem 11.1 (`proved`) — Stage S1 (`First Pass`) Lower Bound
Starting from `records: &[u16]` ($R$ records, $n$ rows, $m$ thin rows, $H$ thick strips) without upstream MTF/RLE2 metadata:
- **Adversary:** Flipping a single `CAP=127` record's width by $+1$ across $w = 16 \to 17$ changes $m$ and every subsequent thin rank's `u32` ID. Hence every exact classifier must inspect all $R$ records:
  $$W_{\mathrm{par}}(\mathrm{S1}) \;\ge\; R + H + (\text{if queued: } m), \qquad C_{\mathrm{miss}}^{\mathrm{cold}}(B) \;\ge\; \left\lceil \frac{2R}{B} \right\rceil + \left\lceil \frac{16H}{B} \right\rceil + \left\lceil \frac{2m}{B} \right\rceil.$$
- **Depth Floor:** Maximal-run coalescing and rank prefix sums over an array of length $R$ are associative ($\mathsf{NC}^1$ prefix scan), so intrinsic data-dependency depth is $D_{\mathrm{dep}}(\mu) = O(R / \mu)$ across $\mu$ independent sub-array cursors (`0` pointer chases, $D_{\mathrm{past-}M_2} = 0$).

### Theorem 11.2 (`proved`) — Stage S2 (`Thick-Strip Elimination`) Lower Bound
Given $H$ thick strips partitioning $n - m$ rows into character-monotone affine bijections:
- **Work & Cold Cache Floor (`proved`):** Every strip's `(from, to, width, byte)` determines the exact return target and spelled character of at least $17$ rows; omitting any single strip corrupts $\ge 17$ output bytes. Thus $W_{\mathrm{par}}(\mathrm{S2}) \ge H$ and $C_{\mathrm{miss}}^{\mathrm{cold}}(\mathrm{S2}) \ge \lceil 16H / B \rceil$.
- **Linked-List RV Depth vs Forest-Height Depth (`proved`):** Any sequential doubly-linked Rauzy–Veech–Morse tail-peeling algorithm executes $D_{\mathrm{serial}} \ge H$ pointer-dependent steps (Records 79–80; `1.79–4.55 H` including gap splits). However, the underlying mathematical object — the partial functional graph of $H$ thick intervals — is a **collection of disjoint directed paths** from the $B_{\mathrm{bands}}$ thin entry intervals of maximum path length $h_{\max} = \max_w |w|$ (`41–186` on Wikidata $\ll 21{,}295$ RV steps!), proving that $D_{\mathrm{serial}} = \Omega(H)$ binds **linked-list interval induction**, *not* path-doubling or interval-lookup collapse!

### Theorem 11.3 (`proved`) — Stage S3 (`Band & Dictionary Extraction`) Lower Bound
Any explicit band + `16 B` inline-prefix representation with $B_{\mathrm{bands}}$ maximal return bands and $V = n_{\mathrm{words}}$ multi-byte words ($V_{>16}$ of length $> 16$) requires:
$$W_{\mathrm{par}}(\mathrm{S3}) \;\ge\; B_{\mathrm{bands}} + V + \sum_{|w| > 16} |w|, \qquad C_{\mathrm{miss}}^{\mathrm{cold}}(\mathrm{S3}) \;\ge\; \left\lceil \frac{12 B_{\mathrm{bands}}}{B} \right\rceil + \left\lceil \frac{16(256 + V)}{B} \right\rceil + \left\lceil \frac{\sum_{|w|>16}|w|}{B} \right\rceil.$$
Crucially, the $V_{\le 16} = V - V_{>16}$ short words (**`63.6%–77.6%` of all words on the 6 blocks!**) are **never read from `spelling` during `emit`**, so materializing them into `spelling` and re-scanning them in `inline_words` is pure overhead above the lower bound.

### Theorem 11.4 (`proved`) — Stage S4 (`Wire`) Lower Bound
Writing all $m$ `u32` entries of `edges[0..m)` requires $W_{\mathrm{par}}(\mathrm{S4}) \ge m$, compulsory writes $\lceil 4m / B \rceil$ lines (`5,618–12,768` lines), and $D_{\mathrm{serial}} = 1$ (once band offsets are known, all $B_{\mathrm{bands}}$ spans are mutually disjoint and data-independent). Because mean band width on the 6 blocks is $m / B_{\mathrm{bands}} \in [28.1,\, 63.9]$ `u32`s (`112–256 B` $\approx 1.8\text{–}4.0$ cache lines per band!), contiguous band wiring incurs at most $+1$ partial-line RFO miss per band boundary.

---

## 3. Native Safe-Rust A/A Control & Stage-by-Stage Upper Bounds on `origin/master` (`90e671966`) (`measured`)

Measured in [`verify_cycle11.rs`](2026-11-21-wormhole-cycle11-pre-emit-stage-floors-and-gaps/verify_cycle11.rs) (`25` paired rotated rounds; `Mst_A1` and `Mst_A2` are two independent instances of compiled `origin/master` `mibz2::bwt::OpenCells` in the same binary; `dense-703.syn` is a `#703`-admitted dense block with $r/n = 0.146$, $m = 133{,}777$):

### 3.1 Wall-Clock Decomposition (`µs`) & A/A Noise Floor

| Block | `Mst_A1` (`µs`) | `Mst_A2` (`A2/A1`) | **Pre-`Emit` Sum** (`Pre%`) | **S1: First Pass** | **S2a: Nodes** | **S2b: RV Elim** | **S3a+b: Bands+Dict** | **S4: Wire** | `S5: Emit` (ref) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1,361.9` | `1,363.0` (`1.001×`) | **`799.7` (`59.6%`)** | **`227.3`** (`28%`) | `54.3` | `209.7` (`S2=33%`) | `196.8 + 13.6 = 214.6` (`27%`) | `93.9` (`12%`) | `542.3` |
| `block-2.L` | `1,668.7` | `1,665.4` (`0.998×`) | **`944.5` (`57.7%`)** | **`248.1`** (`26%`) | `60.3` | `252.6` (`S2=33%`) | `238.0 + 19.8 = 261.9` (`28%`) | `121.6` (`13%`) | `693.1` |
| `block-85.L` | `1,704.8` | `1,685.4` (`0.989×`) | **`696.8` (`42.3%`)** | **`329.2`** (`47%`) | `22.2` | `118.8` (`S2=20%`) | `121.4 + 12.2 = 133.6` (`19%`) | `93.1` (`13%`) | `949.6` |
| `block-143.L` | `3,003.9` | `3,028.3` (`1.008×`) | **`1,547.2` (`52.1%`)** | **`477.8`** (`31%`) | `76.0` | `375.6` (`S2=29%`) | `376.2 + 35.2 = 417.4` (`27%`) | `200.4` (`13%`) | `1,424.1` |
| `block-210.L` | `2,632.9` | `2,641.7` (`1.003×`) | **`1,147.5` (`44.5%`)** | **`528.7`** (`46%`) | `41.5` | `206.7` (`S2=22%`) | `207.1 + 21.4 = 227.4` (`20%`) | `143.3` (`12%`) | `1,430.9` |
| `block-207.L` | `1,992.7` | `1,997.2` (`1.002×`) | **`1,063.0` (`54.3%`)** | **`367.3`** (`35%`) | `50.5` | `247.3` (`S2=28%`) | `240.9 + 22.0 = 261.8` (`25%`) | `136.2` (`13%`) | `895.5` |
| `dense-703.syn` | `1,447.8` | `1,462.9` (`1.010×`) | **`489.8` (`34.9%`)** | **`458.5`** (`94%`) | `1.2` | `1.6` (`S2=0.6%`) | `1.2 + 0.1 = 1.3` (`0.3%`) | `27.2` (`6%`) | `915.2` |

### 3.2 Six-Dimensional $\mathsf{BMCP}$ Ledger Across Pre-`Emit` Stages (Representative Extremes: `block-85.L` Sparse-$H$ vs `block-143.L` Heavy-$H$)

| Stage | $Q_{\mathrm{rand}}$ (`b85` / `b143`) | $D_{\mathrm{serial}}$ (`b85` / `b143`) | $D_{\mathrm{past-}M_2}$ (`b85` / `b143`) | $C_{\mathrm{miss}}(32\text{K})$ (`b85` / `b143`) | $C_{\mathrm{miss}}(256\text{K})$ vs **Floor** (`b85` / `b143`) | $S_{\mathrm{mem}}$ Dedicated Scratch (`b143`) |
|---|---:|---:|---:|---:|---:|---:|
| **S1 (`First Pass + OutQ16`)** | `158.9k` / `216.9k` | `79,375` / `108,335` | `0` / `0` | `8,231` / `11,374` | `7,281` (`0.92×`) / `10,820` (`0.99×`) | `strips`: `128 KiB` + `spill`: `6 KiB` (`out` aliased) |
| **S2 (`Nodes + RV Elim`)** | `79.4k` / `245.1k` | `7,130` / `21,295` | `0` / **`3,223`** | `7,662` / **`37,416`** | `2,332` (**`5.95×`**) / `10,985` (**`8.70×`**) | Overlaid in `edges` (`769 KiB`) |
| **S3 (`Bands + Spell + Inline`)** | `21.0k` / `61.6k` | `4,818` / `13,500` | `0` / **`7,224`** | `6,221` / **`21,403`** | `1,533` (**`1.85×`**) / `11,138` (**`4.96×`**) | `bands`: `96 KiB` + `spelling`: `64 KiB` + `inline16`: `65 KiB` |
| **S4 (`OutQ16 Wire`)** | `7.7k` / `20.7k` | `1` / `1` | `0` / `0` | `12,685` / `19,356` | `10,493` (`1.07×`) / `17,750` (`1.44×`) | Writes `edges[0..m)` (`769 KiB`) |

---

## 4. Which Pre-`Emit` Stage Holds the Largest Gap After the Diet? (`Answer to Charge #1`)

Ranking the four pre-`emit` stages by their measured gap above their $\mathsf{BMCP}$ lower bounds reveals **three large, distinct gaps** and **one nearly-tight stage**:

1. **Largest Wall-Clock & Store-Amplification Gap — Stage S1 (`First Pass + OutQ16`, `227–529 µs`, `31%–48%` of pre-`emit`):**
   - Its `256 KiB` L2 miss count is already at the compulsory floor (`0.92–0.99×`), yet it burns `12–15` cycles per maximal run (`3.5–4.1 ns`/run) because:
     - **(a) `8–10×` Store Amplification & Byte-Pair Loop:** `outq.rs:128-131` writes `32 B` (`16` `u16`s via a byte-pair `chunks_exact_mut(2)` loop) for **every** thin run even though mean thin-run width is `1.6–2.0` rows (`1.6–4.1 MB` written vs `0.18–0.41 MB` live);
     - **(b) `256`-Stream LFB/Store-Buffer Contention:** Scattering `32 B` writes across up to `256` active `1 KiB` chunks in `out` generates `102k–258k` non-sequential cache-line transitions (`Q_rand`), colliding directly with Comet Lake's `10` LFBs;
     - **(c) Serial Coalescing & Width-Branch Mispredictions:** `maximal` scans `records` serially with data-dependent run-merge and `width <= 16` branches.
2. **Largest Serial Pointer-Chase & L1/L2 Miss-Inflation Gap — Stage S2 (`Thick-Strip Elimination`, `141–452 µs`, `5.95–8.70×` L2 miss inflation, up to `3,223` `dep_past_l2`):**
   - `S2a` touches `strips` **3 separate times** plus a random gather `strips[image_order[k]]`, and `S2b` executes `2.5–4.6× H` doubly-linked list pointer splices while chasing **two random `arena[rule + 2]` loads** on every `Elim::join` just to compute `len(left) + len(right)`.
3. **Largest `dep_past_l2` & Redundant-Buffer Gap — Stage S3 (`Band/SLP Extraction + inline_words`, `134–417 µs`, `1.85–4.96×` L2 miss inflation, up to `7,224` `dep_past_l2`):**
   - Walks two random linked lists (`side 1` and `side 0`), expands **100% of words** byte-by-byte into `spelling` even though **`63.6%–77.6%` of words (`len <= 16`) are never read from `spelling` by `emit`**, buffers `49–98 KiB` of `Vec<Band>`, and re-scans `offsets` + `spelling` cold after `S4` in `inline_words` (`12–35 µs`).
4. **Tightest Pre-`Emit` Stage — Stage S4 (`OutQ16 Wire`, `90–200 µs`, only `11%–13%` of pre-`emit`):**
   - Because mean band width is **`28.1–63.9` contiguous `u32`s (`112–256 B` = `1.8–4.0` cache lines per band)**, `S4` already operates within **`1.07–1.44×`** of its compulsory L2 write floor (`0` `dep_past_l2`).

Cycle 12 tightens $\mathsf{BMCP}$ on Comet Lake (`16` GPRs, `10` LFBs, hardware streamer/IP-stride prefetcher) so we can explain `S1`'s `256`-stream LFB stall and `S4`'s streamer advantage quantitatively before attacking **S1, S2, and S3** in Cycles 13–20.
