# Wormhole (Neon) Cycle 23 — `S2` Synergy Attack: `GradedOperad` (`#714` × `Operad-MN16`) vs Tagless Tiered `u64/u128` & `set_links`

**Session:** `research/carbon-wormhole` (Round 4 — Gemini Neon, Cycle 23 of 30; 3rd of 10 in Round 4)  
**Baseline `origin/master`:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`v0.5.11`; `src/` byte-identical to `90e671966`)  
**Verification harness:** [`verify_cycle23.rs`](2026-11-21-wormhole-cycle23-s2-graded-operad-and-set-links/verify_cycle23.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `6/6` `manifest.json` SHA-256 auth + `5,237` Gate G1/boundary-`7..32`/spiral/multi-epoch cases byte-exact + `15`-round 5-stage attribution + `25`-round `p50`/`p10` `InverseBwt::invert_runs` confrontation), [`graded_operad.toml`](2026-11-21-wormhole-cycle23-s2-graded-operad-and-set-links/graded_operad.toml) (`PASS`), [`cycle23_output.txt`](2026-11-21-wormhole-cycle23-s2-graded-operad-and-set-links/cycle23_output.txt), [`cycle23_results.json`](2026-11-21-wormhole-cycle23-s2-graded-operad-and-set-links/cycle23_results.json)

---

## 1. One-Line Keep & Kill Decisions

- **PROMOTE `G2: GradedOperad` (`#714 (len10 << 22) | id22` Graded Handle × `Operad-MN16 RULE=4 u128`, `0 B` scratch):** Eliminates `71.5%` (`240,381` of `336,297`) of random `S2` rule loads on long joins (`tot >= 16`) and deletes `len_operad()` from `S3`, cutting **`S2` by `-114.7 µs` (`-6.2%`)**, **`S3` by `-49.3 µs` (`-5.9%`)**, and **`S2 + S3` to `2,527.2 µs` (`-282.8 µs` below `W1: #714+L4/L5`, `-913.1 µs` = `-26.5%` below `M0: 0.5.11`)**, reaching **`0.868×` p50 / `0.860×` p10 of `A1:Master`** (`7/7` blocks) in pure $\mu = 1$ safe Rust.
- **KILL `G1: Explicit Narrow::set_links` (`+0.8 µs` in `S2`):** LLVM already coalesces adjacent `set_prev` + `set_next` on `a[id * 4 + side]` into a single 32-bit store in `Narrow::insert`, leaving `0.0 µs` real `S2` speedup.
- **KILL `G3: Tagless Tiered GradedOperad (u64 <= 8 / u128 <= 16)` (`+60.4 µs` slower than `G2` in `S2`, `+92.1 µs` in `Pre(S1-4)`):** Splitting `tot <= 15` into `tot <= 8` vs `9..=16` (plus inner `ll <= 8` / `rl <= 8` checks) adds 3 unpredictable length-boundary branches inside `Elim::join`, whose misprediction penalty outweighs saving 2 sequential L1d words.

---

## 2. Wormhole Chain & Algebraic Proof (`GradedOperad`, Theorem 23.1)

Cycle 22's `CmlSim` proved that post-stack `S*` (`Operad-MN16`) still executes `336,297` random rule loads (`S2_RL`) across the 6 Wikidata blocks because `Elim::join_operad` loads `a[r + 3]` (and `a[r + 2]` on long words) for both operands merely to compute `ll + rl`, even though `71.5%` of joins satisfy `ll + rl >= 16` and never read byte payloads.

**Chain (`Graded Free Monoid` $\to$ `Truncated Operad-MN16` $\to$ `0-Load S3 Dispatch`):**
1. **Graded handle encoding (`proved` overflow-safe for $n \le 900{,}000$, Theorem 15.1):** Every non-empty word handle in `Elim` is `h = (exact_len.min(1023) << 22) | id22`, seeding literal strip `c` as `(1 << 22) | (c + 1)`. Since `id <= 1,800,256 < 2^{22}`, `id22` never overflows.
2. **Branch-minimal `Elim::join_m::<3>`:** Extract `ll = lh >> 22`, `rl = rh >> 22`, `tot = ll + rl` in registers (`0` loads).
   - If `tot <= 15` (`28.5%` of joins): read short `u128` payloads via `short_u128(lid)`/`short_u128(rid)`, merge `vl | (vr << (8 * ll)) | ((tot as u128) << 120)` into `RULE4` (`r3 >> 28 == 0`), return `(tot << 22) | id`.
   - If `tot >= 16` (`71.5%` of joins): **`0` random rule loads** (when `ll < 1023 && rl < 1023`, with saturating fallback `len_operad(lid) + len_operad(rid)` for deep spirals); write `[lid, rid, exact_len, u32::MAX]` sequentially and return `(exact_len.min(1023) << 22) | id`.
3. **Zero-load `S3` length dispatch:** In `induced`'s `side-0` band loop, `h_len = raw_word >> 22` directly dispatches `h_len <= 15` (`short_u128`) vs `h_len >= 16` (`append_operad`), eliminating `len_operad` probes on all `25,497` bands (`-49.3 µs` in `S3`).

*Why `G2` beats BOTH `#714` and `Operad-MN16` on `S2 + S3`:*
`W1 (#714)` achieves `S2 = 1,497 µs` (`0` eager merges) at the cost of `S3 = 1,313 µs` (`S2 + S3 = 2,810 µs`). `C22 (Operad-MN16)` achieves `S3 = 842 µs` at the cost of `S2 = 1,850 µs` (`S2 + S3 = 2,691 µs`). **`G2 (GradedOperad)`** pays only `+238 µs` over `W1` in `S2` (`1,735 µs`) to earn **`-521 µs`** in `S3` (`792 µs`), setting a new Pareto optimum at **`S2 + S3 = 2,527 µs`** (**`-913 µs` / `-26.5%` below `M0: 0.5.11`**).

---

## 3. Verification & Benchmarks (`measured` in [`verify_cycle23.rs`](2026-11-21-wormhole-cycle23-s2-graded-operad-and-set-links/verify_cycle23.rs))

- **Exactness (`verified`):** `6/6` `manifest.json` SHA-256 authenticated blocks + `dense-703.syn` + **`5,237` Gate G1 / length-boundary `7..32` / deep spiral (`widest = 65,000`, `len > 1,023`) / multi-epoch (`m > 65,536`)** cases 100% byte-exact across `M0, W1, C22, G1, G2, G3`.
- **Per-Worker Working Set:** `G2` adds **`0 B` scratch** (`950.0 KiB` Wikidata / `1,129.7 KiB` incl. `dense-703.syn` vs `M0`'s `1,172.7 / 1,275.5 KiB`; `10` i9 workers on Wikidata = `17.86 MiB <= 20 MiB` L3).
- **DRAM Traffic:** `<= 0.034–0.059 B/B` steady-state (`0` extra DRAM bytes).
- **Acquisition Cost:** `~28` safe-Rust lines in `Elim::join` + `induced` (`0` new structs or arrays).

### 3.1 5-Stage Attribution (`6`-Block Sum `µs`, `15` Rounds Median)

| Arm | `S1` | `S2` | `S3` | `S4` | **`Pre (S1–4)`** | `S5 Emit` | **`Sum`** | Verdict |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `M0: Clone (0.5.11)` | `2302.6 µs` | `1928.3 µs` | `1512.0 µs` | `733.5 µs` | `6476.4 µs` | `5903.7 µs` | `12380.1 µs` | Baseline |
| `W1: #714+L4+L5` | `2018.7 µs` | `1497.2 µs` | `1312.8 µs` | `745.1 µs` | `5573.8 µs` | `5926.4 µs` | `11500.3 µs` | Ref |
| `C22: S* + F4` | `1942.9 µs` | `1849.7 µs` | `841.5 µs` | `753.0 µs` | `5387.0 µs` | `5404.4 µs` | `10791.5 µs` | C22 Champ |
| `G1: +set_links` | `1962.8 µs` | `1850.5 µs` | `840.7 µs` | `749.3 µs` | `5403.4 µs` | `5365.9 µs` | `10769.3 µs` | **KILL** (`+0.8 µs` `S2`) |
| **`G2: +GradedOperad`** | **`1926.7 µs`** | **`1735.0 µs`** | **`792.2 µs`** | **`758.6 µs`** | **`5212.5 µs`** | **`5348.7 µs`** | **`10561.2 µs`** | **PROMOTE** (`-175 µs` Pre!) |
| `G3: Tiered+set_links` | `1951.0 µs` | `1795.4 µs` | `789.2 µs` | `769.1 µs` | `5304.6 µs` | `5408.5 µs` | `10713.1 µs` | **KILL** (`+60 µs` `S2` vs `G2`) |

### 3.2 Headline 1T `InverseBwt::invert_runs` Confrontation (`25` Rounds, `p50` / `p10` vs `A1:Master`)

`A2/A1` A/A control = **`0.992×` p50 (`0.989×` p10)** on the 6 authenticated Wikidata blocks (`12,366.9 µs` sum):

| Block | `A1_p50` | `A2/A1` | `M0/A1` | `C22/A1` | `G1/A1` | **`G2/A1`** | `G3/A1` | **`G2/M0`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1364.5 µs` | `0.985×` | `0.993×` | `0.889×` | `0.877×` | **`0.866×`** | `0.860×` | **`0.872×`** |
| `block-2.L` | `1662.7 µs` | `0.996×` | `0.998×` | `0.884×` | `0.873×` | **`0.864×`** | `0.869×` | **`0.866×`** |
| `block-85.L` | `1691.8 µs` | `0.988×` | `0.970×` | `0.887×` | `0.873×` | **`0.875×`** | `0.876×` | **`0.902×`** |
| `block-143.L` | `2999.9 µs` | `0.995×` | `0.984×` | `0.874×` | `0.870×` | **`0.863×`** | `0.861×` | **`0.877×`** |
| `block-210.L` | `2647.7 µs` | `0.994×` | `0.967×` | `0.880×` | `0.873×` | **`0.868×`** | `0.869×` | **`0.898×`** |
| `block-207.L` | `2000.4 µs` | `0.993×` | `0.984×` | `0.873×` | `0.874×` | **`0.873×`** | `0.868×` | **`0.887×`** |
| `dense-703.syn` | `3134.6 µs` | `1.001×` | `0.949×` | `0.839×` | `0.823×` | **`0.833×`** | `0.837×` | **`0.878×`** |
| **6-BLK p50/A1** | **`12366.9 µs`** | **`0.992×`** | **`0.981×`** | **`0.880×`** | **`0.873×`** | **`0.868×`** | **`0.867×`** | **`0.885×`** |
| **6-BLK p10/A1** | **`12099.2 µs`** | **`0.989×`** | **`0.972×`** | **`0.873×`** | **`0.870×`** | **`0.860×`** | **`0.862×`** | **`0.885×`** |

---

## 4. Next Cycle Plan (`Cycle 24 (Neon)`)
Now that `Pre(S1–4)` is down to **`5,212.5 µs`** (`-19.5%` below `M0`), `S5 Emit` is **`50.6%`** of total kernel time (`5,348.7 µs`) and **`90.2%`** of all unprefetched L2 misses. Cycle 24 launches **Attack #1 on `S5 Emit`**:
1. **Edge-Packed GPR `Len4` (`E1`):** With `WordDedup`, unique codes drop to `811–1,712` (`<= 11` bits!). Whenever `bits + 11 + 4 <= 32` (`bits <= 17`, covering `m <= 131,072` blocks like `block-0.L` and `block-2.L`; and `bits <= 18` when `codes <= 1,024` or via `len3`/`len4` split!), pack **`len4 = edge >> 28`** (`1..=15`, `0 = LONG/16`) directly into the 32-bit `edge` word in `100%` GPR arithmetic — **zero `lens[code]` load**, **zero XMM-to-GPR cross-domain stall** (escaping Kitchen Sink Cycle 27's `+465 µs` `Inline15` trap!), and **zero extra scratch**!
2. **True Zero-Spill `emit` Loop (`E2`):** Eliminate the remaining 4 stack reloads (`lens.ptr`, `inline16.ptr`, `out.ptr`, `mask`) that LLVM still kept in `F4` by hoisting the `LONG` exit cleanly (`loop` returning `Break(code, j, at)` vs in-loop `callq`!).
3. **Zipf / Frequency-Ordered `inline16` Code Remapping (`E3`):** Re-index hot deduped codes by band-span frequency so the top-256 hot words pack into a single `4 KiB` L1d/DTLB page!
