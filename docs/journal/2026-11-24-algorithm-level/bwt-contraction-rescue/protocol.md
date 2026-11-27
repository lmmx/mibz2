# Frozen Protocol: Contracted Return-Map Constructor Rescue (`bwt-contraction-constructor-rescue`)

*Frozen on 27 November 2026 before running any held-out measurements under claim [`bwt-contraction-constructor-rescue`](../../../../frontier/claims/bwt-contraction-constructor-rescue.toml). Preserves `#1013` ([`bwt-digram-return-map`](../bwt-digram-return-map/results.md)) STOP verdict intact. Zero edits to `src/`, zero `unsafe` (`#![forbid(unsafe_code)]`), zero workstation spending, zero relaxation of frozen C2 gates.*

---

## 1. Frozen stop test (verbatim from claim)

> Preserve #1013's original verdict. Register distinct variants before new measurements, exact bytes on all original admitted/forced/guard and synthetic suites, charge full C+E on baseline and native against both matched single- and four-lane forward controls, seven interleaved repeats, same >=39.18 ms projected 1-worker improvement and guard no-regression conditions; report selection exposure and use independently registered fresh validation for confirmatory claims. No claimed PASS from free oracle or stage-only gain. First bounded session prices cost components and attempts at most two distinct justified improvements; further iterations only with a concrete new lever and plausible marginal budget, all attempts retained. Stop on scoped obstruction/exhausted feasible budget/no credible new lever after constructive consideration, not just a first losing implementation. Zero src edits, unsafe or new spending.

---

## 2. Cold Dissection & Cost Ledger of `#1013` (`fb_elim_75_L16`, Train `0..117` Only)

All structural measurements and component timings below were collected strictly on the **117 training blocks** (`admitted[0..117]`, `105,298,817` rows; zero held-out inspection).

### 2.1 Structural anatomy of `fb_elim_75_L16` on `train[0..117]`

- Retained Open vertices: $m = 13,042,101$ (`435.4 KiB/blk` at 4 B/edge).
- Kept vertices ($K$): $m_K = 3,210,984$ (`24.6%`, `107.2 KiB/blk` at 4 B/edge — fits in 256 KiB L2).
- Eliminated vertices ($E$): $m_E = 9,831,117$ (`75.4%`, `328.2 KiB/blk`).
- **Kept outgoing edge split:**
  - $K \to K$ direct edges (0 eliminated hops): `1,810,199` (**56.4%** of $K$).
  - $K \to E^+ \to K$ chains ($\ge 1$ eliminated hop): `1,400,785` (**43.6%** of $K$), traversing all `9,831,117` $E$-vertices (mean `7.02` hops/chain).
- **Prefix code `c0` class on `K -> E` chains (`1,400,785`):**
  - 1-byte literal (`c0 < 256`): `38.7%`.
  - Short inline (`lens[c0] in 2..=16`): `44.4%`.
  - Long spelling (`lens[c0] == LONG`, $>16$ B): `16.9%` (`236,277` chains).
- **Eliminated-hop code class (`9,831,117` hops):**
  - **96.89%** 1-byte literals (`cv < 256`), `2.92%` inline, `0.19%` `LONG`.
- **Chain hop-count distribution (`1,400,785` chains):**
  - `1` hop: `34.4%`, `2`: `14.8%`, `3`: `11.4%`, `4–8`: `24.2%` (`84.8%` are $\le 8$ hops, accounting for ~31% of $E$ hops).
  - `9–16` hops: `3.4%`, `>16` hops: `11.9%` (max `~219` hops; the `15.2%` of chains $>8$ hops account for **~69% of all $E$ hops**).
- **Contracted word length distribution:**
  - $\le 8$ B: `53.0%`, `9–16` B: `17.2%` (`70.2% <= 16` B), `17–32` B: `5.8%`, `>32` B: `24.0%`.
- **Dictionary and `spelling` bloat in `#1013`:**
  - Original dictionary: `1,014` codes/blk, `orig_spelling = 40.7 KiB/blk`.
  - Final `#1013` dictionary: `6,827` codes/blk (max `16,411`), `inline16 = 106.7 KiB/blk` (max `256.4 KiB`), `wide_new = 0/117`.
  - **Key contingent waste discovered:** `FINAL spelling = 370.0 KiB/blk` (max `611.7 KiB`, `37.6 MiB` written across 117 blocks), of which **`27.3 MiB` (72.4%) is redundant byte-for-byte re-copying of already-stored `c0 == LONG` prefix words** just to append a short literal suffix!
  - `dm_cache(1024)` hit rate: `73.4%` (`720,734` hits, `261,798` misses; `418,253` undeduped long chains).

### 2.2 Uncontended sub-phase cost ledger on `train[0..117]`

Isolated sub-phase timings on uncontended Haswell Xeon VM (`ctrl_fwd_1lane = 249.82 ms`, `A6a = 62.06`, `A6b = 61.03`, `A6c = 16.38`, `A7 = 110.35 ms`; `ctrl_fwd_4lane_ora ~ 219.5 ms`):

| Sub-phase of `#1013 fb_elim_75_L16` | Uncontended Train (ms) | Share of `contr` | Contingent vs Algorithmic |
|---|---:|---:|---|
| `prep_score` (F-bucket band scan + sort) | 3.39 | 3.3% | **Contingent:** fusable directly into `A6b` `induced_with` (`~0.5 ms`) |
| `prep_tag` (`edges` R/W tag + `kept_e0` copy) | 3.44 | 3.3% | **Algorithmic** if keeping pure SIMD `wire_forward` (`A6c = 16.4 ms`); fusing into `A6c` costs `+3.66 ms` |
| `pure_chase1` (1-lane dependent loads only) | 51.51 | 49.9% | **Algorithmic** for 1-lane chase over `13.04M` `edges[v]` loads (`435 KiB/blk > 256 KiB` L2) |
| `pure_chase4` (4-lane bare chase only, no bytes) | *27.16* | *26.3%* | **Hardware floor** for 4-lane MLP over `edges[v]` without byte assembly or commit calls |
| Non-chase assembly + dict + spelling + writes | 44.92 (vs `chase1`) / 69.27 (vs `chase4`) | 43.5% | **Partially contingent:** `72.4%` redundant `c0 == LONG` spelling copies, `u128` bit-shift assembly + `fill(0)`, 1024-slot `dm_cache`, `wide_new` checks |
| **Total `contr` (`#1013`)** | **103.26** | **100.0%** | vs `A7` saving: `-68.32 ms` (`110.35 -> 42.03 ms`) vs 1-lane, `-38 ms` vs 4-lane oracle |

**Budget arithmetic exposed by the ledger:**
Even at the **zero-assembly, zero-dict, zero-write** bare 4-lane chase floor (`prep_tag 3.44 + pure_chase4 27.16 = 30.60 ms`), `A6a (62.1) + A6b (61.0) + A6c (16.4) + 30.6 + A7 (42.0) = 212.1 ms` vs `ctrl_fwd_4lane_ora (~219.5 ms)` — leaving only `~7.4 ms` VM (`~28 ms` i9) *before paying any byte extraction, word assembly, dictionary, or edge-write cost*, whereas clearing the frozen `>= 39.18 ms` i9 gate requires `<= 209.1 ms` against `ctrl_fwd_4lane_ora` and `<= 239.4 ms` against `ctrl_fwd_1lane`.

---

## 3. Exploratory Train-Set Mechanisms Evaluated & Rejected (Retained per Rescue Brief)

Before selecting the two pre-registered redesigns below, the following distinct architectural mechanisms from `docs/journal/2026-11-27-contraction-rescue.md` were prototyped and timed strictly on `train[0..117]`:

1. **Dual-space `A6c` (`wire_dual_space`, monotonic `v` rank translation during `OutQ16` consumption):** Splitting `K` and `E` edges during `A6c` (`edges_K` + `edges_E`) measured **`32.52 ms`** vs `wire_forward (16.38) + prep_tag (3.44) = 19.82 ms` (`2.20x` slower `A6c` even with single-add fast-path) because per-band `u_seg` splitting and `v_seg` boundary checks break `Reader::wire`'s pure SIMD `pmovzxwd + por` loop, while `chase4_E` saved only `~3.5 ms` over `chase4`. **Rejected.**
2. **Fused `wire_forward_fused_elim` (`A6c` F-bucket tagging during wiring):** Splitting bands at `r_first` boundaries inside `A6c` added **`+3.66 ms`** (`20.32` vs `16.66 ms`), exceeding the `3.44 ms` `prep_tag` pass it replaced. **Rejected.**
3. **Wavefront batched chase (`wf8` / `wf16` / `wf32`, lockstep hop-by-hop drain):** Measured **`41–46 ms`** vs `pure_chase4 = 27.16 ms` due to straggler convoy drain (`11.9%` of chains have `>16` hops up to `219` hops). **Rejected.**
4. **4-lane hop-pool / 6-state machine / 2-lane 8-byte-evicting register chase:**
   - 6-state interleaved 4-lane (`184 ms` vs `111 ms`) and 2-pass `hop_pool` (`357 ms` vs `295 ms` contended) added per-hop heap writes, double passes, and register spilling on 16-GPR x86-64.
   - 2-lane `u64`-packed chase (`r2_reg2_bump_2pt`, `contr = 96.45 ms` uncontended) evicted chains at `>8` literal hops into serial Stage B2, which still contained **~69% of all `E` hops**, nullifying 2-lane MLP. **Upgraded to full-chain 4-lane `r2_mlp4_buf_bump` below.**
5. **Co-located 8-byte edges with zero dictionary growth (`Edge8`, `r2_edge8_nodict`):** Eliminating dict hashing cut `contr` `-21%` (`232.6` vs `295.2 ms` contended) but regressed `A7` by **`+22..+39%`** (`61` vs `44 ms` uncontended; `140–174` vs `89–115 ms` contended) due to 2x wider `edges8` (`214 KiB/blk`) and 4-way tag dispatch in `A7`. **Rejected.**
6. **Raised `LIMIT = 24` / `32` with fast `w <= 32` `Queues::push`:** Cut `A6a + A6b` by `-15..-23 ms` but expanded $m$ by `+17..+27%`, increasing `contr + A7` by more than `A6b` saved (net loss). **Rejected.**

---

## 4. Two Pre-Registered Constructor Redesigns (Frozen Before Held-Out Data)

Both redesigns keep pure SIMD `wire_forward` (`A6c`), fuse F-bucket literal scoring into `A6b`, add an $O(1)$ pre-construction profitability guard (`m >= 32,768` and `elim_frac >= 0.55`, rejecting unprofitable `runs`/`text` guard blocks at zero contraction cost), and use zero-copy 2-part prefix encoding (`TWO_PART_INL_BASE` / `TWO_PART_LONG`) so `c0 == LONG` prefixes are never re-copied into `spelling` (`-72.4%` spelling writes). They differ fundamentally in **chase concurrency** and **dictionary allocation**:

1. **Redesign 1 — `r1_clean1_2part` (Representation & Assembly Redesign, 1-Lane + L1d-Hot Hash Dedup + 2-Part Zero-Copy):**
   - **Dependency graph & costs eliminated:** Keeps the low-branch-misprediction 1-lane chase loop (`kept_e0 -> edges[v]`), removes `#1013`'s `27.3 MiB` redundant `c0 == LONG` `spelling` copies via 2-part prefix pointers, replaces `u128` shift-assembly + redundant `fill(0)` with direct stack `[u8; 16]` byte stores and fast `c0 < 256` initialization, and expands `dm_cache` from 1,024 to 4,096 slots (`16 KiB`, L1d-resident) to raise cache hit rate while preserving compact `inline16` (`~107 KiB/blk`) for `A7`.
2. **Redesign 2 — `r2_mlp4_buf_bump` (Execution & Allocation Redesign, 4-Lane Full-Chain L1d-Buffer MLP Chase + Zero-Hash Bump Compact 4B Dict):**
   - **Dependency graph & costs eliminated:** Attacks BOTH the `51.5 ms` serial 1-lane chase AND `#1013`'s `dm_cache` hashing/16B-comparison cost simultaneously. Runs 4 concurrent chase lanes across ALL $K \to K$ and $K \to E^+ \to K$ chains into `4 x 256 B` (`1 KiB`, L1d-resident) stack buffers (covering `99.9%+` of chain lengths including the `69%` of $E$ hops in chains $>8$ hops), isolates chain commits and slow non-literal hops in `#[inline(never)]` helpers to protect inner-loop GPR allocation (`ev0..3`, `len0..3`), and eliminates 100% of `dm_cache` hashing and word comparisons by bump-allocating directly into pre-reserved `inline16`/`lens` while retaining 4-byte `new_edges` (`107 KiB/blk` in L2) and 2-part `c0 == LONG` encoding.

---

## 5. Evaluation Matrix, Selection Exposure, and Frozen Decision Rule (C2)

- **Executable:** `examples/bwt_contraction_rescue.rs` (`exact` and `bench` subcommands).
- **Exactness gate (mandatory before verdict):** 100% byte-identical to `bwt::walk` across all `233` admitted Wikidata blocks, `198` forced non-admitted Wikidata blocks, `11` synthetic stress cases, `runs.bz2` (admitted), and `text.bz2` (forced), testing both default thresholds and forced-small thresholds (`min_m = 4, min_ef = 0.05`) on both baseline x86-64 and `-C target-cpu=native`.
- **Controls (7 block-interleaved repeats with rotating start offset `(k + blk + rep) % n_arms`):**
  - `ctrl_shipped_bidi` (production bidirectional `Open`, anchored to `1,068.0 ms` i9-10900K 1W `C + E`).
  - `ctrl_fwd_1lane` (matched single-lane forward `Open`).
  - `ctrl_fwd_4lane_onl` (paid online 4-lane forward `Open`, $K = 16$ anchors).
  - `ctrl_fwd_4lane_ora` (pre-anchored 4-lane forward lower bound).
  - `ref1013_fb75_L16` (exact `#1013` reference contractor for direct paired comparison).
- **Corpora:**
  - Held-out Wikidata: `admitted[117..233]` (`116` blocks, `104,398,845` rows).
  - Guard corpora (`ab gen`): `runs.bz2` (admitted) and `text.bz2` (forced).
- **Build matrix:** Both default baseline x86-64 and `RUSTFLAGS="-C target-cpu=native"`, reported separately.
- **Selection exposure disclosure:** This session did **not** inspect `admitted[117..233]` prior to freezing this protocol; however, `#1013` previously measured `admitted[117..233]` and informed the rescue brief's budget diagnosis. Accordingly, `admitted[117..233]` provides an apples-to-apples comparison against `#1013`, while any **confirmatory PASS claim** would require an independently registered fresh validation corpus not previously used for `bwt-digram-return-map`.
- **Verdict rule (frozen, C2):** PASS iff a concrete, non-oracle, byte-exact redesign clears $\Delta \ge +39.18\text{ ms}$ (3.0% of 1,306 ms 1W wall, computed via paired repeat ratios anchored at `1,068.0 ms` for `ctrl_shipped_bidi`) against BOTH `ctrl_fwd_1lane` and `ctrl_fwd_4lane_*` with non-overlapping 7-repeat spreads in both baseline x86-64 and `-C target-cpu=native` builds, without regressing guard corpora (`runs`, forced `text`). Otherwise STOP and record either a concrete justified next lever (if marginal feasible budget remains) or a defensible scoped obstruction.
