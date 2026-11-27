# Pre-registration: Ideal permutation structures diagnostic

*Frozen on 26 November 2026 before collecting any timing data, under claim [`ideal-permutation-structures`](../../../../frontier/claims/ideal-permutation-structures.toml) and [`README.md`](README.md). No edits to `src/`, zero `unsafe`, zero changes to prior frozen gates (C2).*

---

## 1. Frozen stop test (verbatim from claim)

> Frozen before new data: bounded first-pass diagnostic under the linked note, no src changes. Pre-register candidate families, compute budget, train/held-out split and repeat design. Byte-exact `bwt::walk` on every tested case; preserve periodic/multicycle/error semantics as appropriate. Match warm/cold oracle conditions and charge the entire real consumer; report free selection, actual materialisation and consumer separately. A constructor follow-on requires a concrete candidate with measured downstream improvement over matched prebuilt baseline/forward control (or independently demonstrated cheaper materialisation), and a combined baseline construction-plus-consumer budget of at least 39.18 ms (3% of 1,306 ms) after charging the candidate consumer, with allowable constructor cost explicitly derived. Label projections, report consumer-only gain and scan-ideal 8w effects if measured; no new workstation spending. Otherwise STOP this candidate batch with scoped evidence. No broad no-go inferred from finite search and no alteration of prior gates.

---

## 2. Corpus, deterministic train / held-out split, and exactness suite

- **Primary corpus:** `corpora/wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 total blocks, **233 production-admitted `OpenCells` blocks** under `LIMIT = 16` and `Cap::dense`, 209,697,662 admitted rows).
- **Deterministic train / held-out split (zero adjacent-block leakage across windows):**
  - **Train set:** first 117 production-admitted blocks (`admitted_idx` in `0..117`, covering the first half of Wikidata windows). Used solely to sweep discrete operating parameters (`tau`, `D_max`) and freeze the winning parameter per family.
  - **Held-out set:** last 116 production-admitted blocks (`admitted_idx` in `117..233`, covering the second half of Wikidata windows). All verdict timings, ratios, and allowable constructor budgets are evaluated on the held-out set.
- **Non-admitted / forced controls (kept strictly separate from production-admitted metrics):**
  - All 198 non-admitted blocks of `corpora/wikidata.bz2` (runts and dense-refused blocks) verified byte-exact under forced Open construction (`Cap::NONE`), with non-trivial forced blocks (`n >= 1024`) also summarised separately.
- **Exactness stress suite (synthetic + small BWT/non-BWT instances):**
  1. `dense`: pseudo-random high-entropy valid BWT ($n = 4,096$, $r/n > 0.60$).
  2. `periodic`: exact periodic valid BWTs (`(abac)^256`, `(aaab)^512`, `(ab)^500`) where LF has multiple equal-length cycles and `repeat(n, out)` is exercised.
  3. `multicycle`: arbitrary non-BWT columns ($n = 64$ and $n = 4,096$ with multiple disjoint LF cycles of distinct lengths) verifying marked-cycle extraction and repetition matches `bwt::walk`.
  4. `run_heavy`: long-run valid BWT (`a^400 b^300 c^250 ...` plus structured template repeats, $n = 4,096$).
  5. `edge_cases`: trivial/tiny columns ($n = 1$, $n = 2$, constant byte $a^{64}$, $p = 0$, $p = n - 1$).

---

## 3. Part A — Exact finite search on small instances

Exhaustively enumerate **all $2^{|R_0|-1}$ legal first-return subsets** $R' \subseteq R_0$ containing the marked row $p$ ($p \in R'$, with $|R_0| \le 18 \implies \le 131,072$ subsets per instance) on 6 small instances spanning all structural regimes:
1. `small_json_rep`: structured repeated JSON-like valid BWT (`{"a":1,"a":2,"a":3}`, $n = 19$, starting from all runs or thin rows $|R_0| \le 18$).
2. `small_wikidata_micro`: Wikidata-style micro-BWT (`"P31":"Q5","P31":`, $n = 17$, $|R_0| = 17$).
3. `small_dense`: high-entropy / low-repeat valid BWT (`abcdefghijk12345`, $n = 16$, $|R_0| = 16$).
4. `small_periodic`: periodic valid BWT (`abacabacabacabac`, $n = 16$, 4 cycles of length 4, $|R_0| = 16$).
5. `small_multicycle`: arbitrary non-BWT column ($n = 16$, 3 disjoint cycles of lengths 7, 5, 4, $|R_0| = 16$).
6. `small_run_heavy`: run-heavy structured valid BWT (`aaaabbbbccccdddde`, $n = 17$, $|R_0| = 17$).

For every subset $R'$:
- Verify 100% byte-identical output to `bwt::walk` through the real first-return map $(\Psi_{R'}, w_{R'})$.
- Compute exact `(m = |R'|, visits = |R' ∩ cycle(p)|, distinct_words, dict_payload_bytes, rep_bytes = 4*m + dict_payload_bytes, max_word_len)`.
- Extract the exact Pareto frontiers of `(visits, dict_payload_bytes)` and `(visits, rep_bytes)` (both unconstrained and under `max_word_len <= 16`) and compare directly against Open's width-based `LIMIT` selections (`LIMIT in {1, 2, 4, 8, 16}`).

---

## 4. Part B — Candidate families, interfaces, and anti-transcript rules

### 4.1 Legality and anti-transcript invariants
- Every candidate is a **legal labelled first-return map** $(\Psi_{R'}, w_{R'})$ on a subset $R' \subseteq R$ with $p \in R'$, or a **legal conjugation** $\sigma \circ \Psi_R \circ \sigma^{-1}$.
- **Forbidden:** direct transcripts of decoded bytes (e.g., $R' = \{p\}$ with a single $n$-byte word in `spelling`), output-address tables without a charged walk+spelling consumer, or uncharged metadata conversion.
- All contraction families (Families 1, 3, 4) preserve the **original ascending BWT row-rank order** of surviving vertices $R'$ (non-sequential pointer chasing, identical locality model to `ctrl_shipped_fwd`). Only `F2b_conj_cycle_oracle` permutes rows into cycle order, and is explicitly labelled as an **offline conjugation upper bound**.
- Every candidate is verified **100% byte-identical to `bwt::walk`** on every block (all 233 admitted blocks, all forced controls, and the entire synthetic stress suite).

### 4.2 Matched prebuilt controls
1. **`ctrl_shipped_bidi`**: Open `LIMIT=16` XOR-linked bidirectional return map (`m >= 65_536 && !wide`, forward otherwise) consumed by exact production `emit_bidirectional` / `emit` (`src/open.rs`).
2. **`ctrl_shipped_fwd`**: Open `LIMIT=16` forward-only return map consumed by exact production forward `emit` (`src/open.rs`, matching `prod_forward_emit`). Every candidate is evaluated against both `ctrl_shipped_bidi` and `ctrl_shipped_fwd` so no candidate takes credit for forward-only emission.
3. **`ctrl_shipped_fwd_4lane`**: Exact same Open `LIMIT=16` forward return map augmented only with 4 precomputed equal-step cycle anchors `(start_vertex, byte_offset, step_limit)` (48 B metadata), consumed by a 4-lane interleaved forward walk+spell loop (falling back to 1-lane on short/periodic cycles or `m < 64`). Isolates pure MLP on the unchanged representation.

### 4.3 Candidate representation families
- **Family 1 — Frequency-Gated RePair / Digram Return-Map Contraction** *(exact `emit` consumer interface; strictly reused phrases with `freq >= tau`; zero 1-use contractions)*:
  - `F1a_repair_inl16`: greedy most-frequent adjacent pair contraction on $(\Psi_R, w_R)$ with combined word length cap `<= 16 B` (keeps 100% of new words inline; attacks the 81.6% literal storm). Train sweep: `tau in {2, 4, 8, 16}`. Freeze the fastest `tau` on train satisfying `rep_bytes <= ctrl_shipped_fwd rep_bytes`.
  - `F1b_repair_unb`: unbounded-length greedy most-frequent pair contraction (`freq >= tau`; collapses both literal streaks and repeated multi-word phrases while keeping dictionary compact via reuse). Train sweep: `tau in {2, 4, 8, 16}`. Freeze the fastest `tau` on train satisfying `rep_bytes <= ctrl_shipped_fwd rep_bytes`.
- **Family 2 — Conjugate Row-Renumbering** *(layout-only, identical $R$, words, and visit count; exact `emit` consumer)*:
  - `F2a_conj_code_order`: structure-derived permutation sorting vertices by `(code_freq_rank, code, old_rank)` (computable without knowing cycle order).
  - `F2b_conj_cycle_oracle`: offline traversal-order layout (`0 -> 1 -> ... -> m-1 -> 0`) through the exact indirect `emit` loop — explicitly labelled **upper bound on conjugation layout**.
- **Family 3 — Inline-Chain / Co-Located Edge Contraction**:
  - `F3a_dict_capped_fuse`: greedy `<= 16 B` chain contraction through the standard dictionary-indexed `emit` interface, capped at `D_max` total dictionary entries (`D_max in {4096, 16384}` swept on train, frozen for held-out).
  - `F3b_inline_edge12`: legal contracted first-return map (contracting consecutive short words up to `<= 12 B` in original row-rank order) encoded as co-located 16-byte edges `Edge16 { meta: u32 (len << 24 | next), payload: [u8; 12] }` (`len == 255` holds `(start, end)` into deduplicated `spelling`), eliminating `inline16`/`lens` dictionary indirection for all inline words. One-use inline edge share is explicitly audited and reported.
  - `F3c_repair_edge12`: frozen `F1b_repair_unb` followed by `<= 12 B` short-chain contraction into `Edge16` (combines reused phrase contraction with co-located inline edges).
- **Family 4 — Multi-Lane (4-Lane) + Contracted Return Maps** *(matching `ctrl_shipped_fwd_4lane`)*:
  - `F4a_repair_unb_4lane`: frozen `F1b_repair_unb` + 4-lane forward traversal.
  - `F4b_edge12_4lane`: `F3b_inline_edge12` + 4-lane forward traversal.
  - `F4c_repair_edge12_4lane`: `F3c_repair_edge12` + 4-lane forward traversal.

---

## 5. Measurement protocol, cache conditions, and compute budget

- **Machine:** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell, AVX2, 32 KiB L1d, 256 KiB L2, 45 MiB L3, no vPMU), pinned to CPU 0 (`taskset -c 0`), single-threaded, reusable `out` buffer.
- **Compilation matrix:** Run both default baseline x86-64 (`cargo build --release --example ideal_permutation_structures`) and `-C target-cpu=native` (`RUSTFLAGS="-C target-cpu=native"`), reporting both separately.
- **Repeats:** 7 interleaved repeats across all arms per condition; report median and `min..max` spread in `ms`, `ns/row`, `ns/visit`, and paired per-repeat ratio against `ctrl_shipped_bidi`, `ctrl_shipped_fwd`, and `ctrl_shipped_fwd_4lane`.
- **Matched cache conditions:**
  - **Warm (in-situ):** For each held-out block `b`, execute 1 untimed consumer call on `b` (warming `b`'s exact working set in L1/L2/L3 according to its true size, plus `out`), immediately followed by the timed consumer call on `b`.
  - **Cold (streaming / cold-store):** Before each arm's sequential pass over the 116 held-out blocks, run an untimed 64 MiB cache-eviction sweep (> 45 MiB L3) so smaller collections do not inherit cross-repeat L3 residency.
- **Separate cost accounting:** Measure and report separately:
  1. **Offline search / selection / build time** for each candidate family.
  2. **Actual materialisation time** on the held-out blocks for: (a) shipped `Open` bidi constructor (`A6a + A6b + A6c_xor`), (b) `Open` forward constructor (`A6a + A6b + A6c_fwd`), and (c) linear `O(m)` post-Open chain-contraction materialisation (`mat_f3b_post_open`).
  3. **Real consumer time** (warm and cold, baseline x86-64 and native).
- **Structural metrics tracked per arm:** total `rep_bytes`, `edges_bytes`, `dict_bytes` (`inline16 + lens + spelling`), `m` (retained rows / visits), dependent loads (`visits`), mean / p50 / max word length, `lit%`, `long%`, `wide_edges` block count, `out` write amplification (`bytes_written_to_out / n`), and peak consumer scratch bytes.
- **Compute budget:** Local VM only (target < 15 minutes total wall time); zero i9 workstation runs or external expenditure.

---

## 6. Anchored budget formulas and decision rule (C2)

Reference stage-1 one-worker i9-10900K budget (`wikidata ×4`, `character-stage1/report.txt` & `ideal-parts/results/floor/README.md`):
- $W_{1w} = 1,306.0\text{ ms}$; 3.0% gate $G = 39.18\text{ ms}$.
- Baseline construction $C_{\text{base}} = 577.7\text{ ms}$ ($C_{\text{A6a}} = 200.3$, $C_{\text{A6b}} = 216.0$, $C_{\text{A6c}} = 161.4$).
- Baseline emission $E_{\text{base}} = 490.3\text{ ms}$; combined $C_{\text{base}} + E_{\text{base}} = 1,068.0\text{ ms}$.
- Forward control construction projection (from measured VM `A6c_fwd / A6c_bidi` ratio on held-out blocks): $\hat{C}_{\text{fwd}} = 200.3 + 216.0 + 161.4 \cdot (t_{\text{A6c,fwd}} / t_{\text{A6c,bidi}})$.

For any candidate $X$ with held-out consumer median $t_X$ (transparently labelled as VM-to-i9 ratio projection):
- Projected i9 consumer time: $\hat{E}_X = 490.3 \times (t_X / t_{\text{ctrl\_bidi}})$.
- Consumer-only gain vs baseline: $\Delta E_{\text{vs bidi}} = 490.3 - \hat{E}_X$.
- Consumer-only gain vs matched forward control: $\Delta E_{\text{vs fwd}} = \hat{E}_{\text{fwd}} - \hat{E}_X$ (and vs 4-lane forward control: $\Delta E_{\text{vs 4lane}} = \hat{E}_{\text{4lane}} - \hat{E}_X$).
- Combined baseline budget remaining after charging candidate consumer: $B_{\text{vs base}} = 1,068.0 - \hat{E}_X$.
- Maximum allowable total constructor cost to clear the 39.18 ms gate:
  - Relative to shipped baseline: $C_{\max,\text{vs base}} = 1,068.0 - \hat{E}_X - 39.18 = 1,028.82 - \hat{E}_X$.
  - Relative to matched forward control: $C_{\max,\text{vs fwd}} = (\hat{C}_{\text{fwd}} + \hat{E}_{\text{fwd}}) - \hat{E}_X - 39.18$.
  - Incremental post-Open constructor allowance (if built on top of `Open` forward): $\Delta C_{\text{extra,max}} = \Delta E_{\text{vs fwd}} - 39.18$.
- **Scan-ideal 8-worker projection (model only, not measured on i9):** scaled via `character-stage1` $k=8$ emission share (`69.5 / 490.3 = 0.1417×` of 1w ms; worker gains gated behind serial scan `179.1 ms` until scan relief).

**Decision:** Admit a separate constructor follow-on claim at `status = "idea"` with a frozen net-cost gate **iff** a concrete, non-transcript, byte-exact candidate beats the matched prebuilt forward control (and baseline) on held-out blocks across both warm and cold conditions (or demonstrates independently cheaper materialisation), leaving $\ge 39.18\text{ ms}$ combined budget with a positive, realistically addressable allowable constructor cost. Otherwise record a **scoped STOP** for this candidate batch and append a scoped row to `frontier/dead-ends.tsv`.
