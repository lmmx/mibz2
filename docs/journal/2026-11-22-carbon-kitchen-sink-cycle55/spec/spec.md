# Cycle 55 Phase II Finale Specification: Classical Atlas Extension `C55-D6` (`CA-56`) & `#729` Drop-In Candidate `C55-DramUltra-v2`

**Date:** 2026-11-22
**Branch:** `research/carbon-kitchen-sink` (read-only baseline `90e671966`; `src/`, `Cargo.toml`, `benches/`, `examples/` untouched)
**Artifacts in this folder:**
- [`cand_c55_dramultra_v2_moonshot.rs`](cand_c55_dramultra_v2_moonshot.rs) — `#729` `bench/moonshot/candidates/` drop-in candidate (`C55DramUltraV2Moonshot`, `#![forbid(unsafe_code)]`, `-D warnings` clean, `11/11` unit tests pass)
- [`headroom_oracle_c55.rs`](headroom_oracle_c55.rs) — Unified `C55-D6` (`CA-56`) theorem verifier, `10,190+` exactness gate, deterministic WS attribution, `--proxy-arm` Cachegrind harness, and 8-arm Headroom Oracle v10 (`1T/2W/4W`)
- [`c55_verification_report.md`](c55_verification_report.md) — Complete forensic verification log and exact measurements

---

## 1. Classical Atlas Extension `C55-D6` (`CA-56`)

### Entry `CA-56` — Legendre (1787) Convex-Dual Rate Function & Markov (1906, 1913) 2nd-Order Chain Extension on `CA-53`

- **Primary Sources (Classical, Pre-1950):**
  1. **Adrien-Marie Legendre (1787)**, *"Mémoire sur l'intégration de quelques équations aux différences partielles"*, *Histoire de l'Académie Royale des Sciences*, Paris, 1787, pp. 309–351 — original introduction of the convex-conjugate / Legendre transformation $I_{\mathrm{Leg}}(E) = \sup_{\beta \in \mathbb{R}} \{ \beta E - \Psi(\beta) \}$ and biconjugate involution $\Psi(\beta^*) = \beta^* E - I_{\mathrm{Leg}}(E)$.
  2. **Andrei Andreyevich Markov (1906)**, *"Rasprostranenie zakona bol'shikh chisel na velichiny, zavisyashchie drug ot druga"* (*Izv. Fiz.-Mat. Obshch. Kazan Univ.*, Ser. 2, 15, pp. 135–156) & **A. A. Markov (1913)**, *"Primer statisticheskogo issledovaniya nad tekstom 'Evgeniya Onegina', illyustriruyushchii svyaz' ispytanii v tsep'"* (*Izv. Imp. Akad. Nauk*, Ser. 6, 7(3), pp. 153–162) — empirical transition chains on sequential symbol/run texts and higher-order ($k = 2$) conditional dependence.
- **Explicit Atlas Lineage (All Load-Bearing Ancestors Only, Respecting Cycle 54 Weiss Edge Tightening #1):**
  - **Uses:**
    - `CA-28` (Shannon 1948) — conditional entropy $H_2(P) = H(R_t \mid R_{t-1}, R_{t-2}) \le H_1(P) \le H_{\mathrm{marg}}(\pi)$, conditional mutual information $I_2 = H_1 - H_2 \ge 0$, and KL divergence $D_{\mathrm{KL}}(P_{\beta^*} \| P_0)$.
    - `CA-31` (MacMahon 1915 / de Moivre 1730) — `CAP = C` bounded-part run-record compositions and record-merging absorption structure.
    - `CA-42` (Perron 1907 / Frobenius 1908–1912) — 2nd-order pair-state transition graph stationarity $\pi^{(2)\top} P^{(2)} = \pi^{(2)\top}$ on active pairs $(u, v)$.
    - `CA-46` (Euler 1748) — geometric max-entropy ceiling $H_{\mathrm{Geom}}$ at top of the 8-level hierarchy.
    - `CA-50` (Hölder 1889 / Jensen 1906) — $H_{\mathrm{Joint}}(\mu, \mu_{\log}; C) \le \min(H_\zeta, H_{\mathrm{Geom}})$ and marginal-moment `m`-envelope baseline.
    - `CA-53` (Boltzmann 1877 / Gibbs 1902) — 1st-order canonical free energy $\Psi(\beta) = \ln Z(\beta)$, Gibbs ceiling $H_{\mathrm{Gibbs}}$, and 1st-order adjacency `m`-envelope dualized by Legendre (1787) and lifted to 2nd order by Markov (1906, 1913).
  - **Used by:** Cycle 55 `C55-DramUltra-v2` architectural justification (proving that no finite-order $k$ run-length or record-length tensor can predict `S2b` grammar depth or eliminate the remaining symbol-merging `m`-gap, justifying `O(1)` density-adaptive `nrec/n` + `64 -> 256` strided sampling over higher-order Markov predictors!).
- **Chain Length Records:**
  - **Chain D Extended to 10 Pure-`CA` Nodes / 9 Links:**
    `CA-26 → CA-27 → CA-31 → CA-37 → CA-41 → CA-44 → CA-47 → CA-50 → CA-53 → CA-56`
  - **Chain A+N+P+R+D4+D5 Extended to NEW 13-Node / 12-Link Pure-`CA` Record:**
    `CA-05 → CA-10 → CA-11 → CA-16 → CA-17 → CA-18 → CA-34 → CA-40 → CA-43 → CA-46 → CA-50 → CA-53 → CA-56`
- **Weiss Classification:** **`load-bearing`** (`0` relabels, `0` decorative links). All modern kernel optimizations (`L ∈ {8, 10, 12, 24, 32}`, `S5_Pow2Dict2048`, `S5_PackPrevNext`, `S4_8WayEpoch`, `S4_SlowEpoch8`, `TightSteps`) are strictly classified as **`2020s` modern CS** (never attributed to classical authors).

---

### 1.1 Theorem D6-1A (Legendre 1787 Convex-Dual Rate Function & Pythagorean Contraction)

Let $\pi$ be the empirical marginal distribution of `CAP = C` record lengths on $\{1, \dots, C\}$, $P_0(j \mid i) = \pi_j$ the product reference kernel, and $E(i, j) \in \{0, 1\}$ either the thick/thin boundary energy $E_{\mathrm{TT}}(i, j) = \mathbf{1}[\mathbf{1}(i > L) \neq \mathbf{1}(j > L)]$ or the `CAP`-to-thin absorption energy $E_{\mathrm{CT}}(i, j) = \mathbf{1}[i = C \land j \le L]$. Define `CA-53`'s scaled cumulant generating function (canonical free energy):

$$\Psi(\beta) = \sum_{i=1}^C \pi_i \ln\!\left( (1 - q_{1,i}) + q_{1,i} e^{\beta} \right), \qquad q_{1,i} = \sum_{j : E(i,j)=1} \pi_j.$$

Its **Legendre (1787) convex dual** $I_{\mathrm{Leg}}(E) = \sup_{\beta \in \mathbb{R}} \{ \beta E - \Psi(\beta) \}$ attains its supremum at $\Psi'(\beta^*) = \bar{E}$ and satisfies three exact identities for every empirical cyclic chain $P$:

1. **Exact Legendre–Gibbs Duality:**
   $$I_{\mathrm{Leg}}^{\mathrm{bits}}(\bar{E}) = \frac{\beta^* \bar{E} - \Psi(\beta^*)}{\ln 2} = D_{\mathrm{KL}}^{\mathrm{bits}}(P_{\beta^*} \| P_0) = H_{\mathrm{marg}}(\pi) - H_{\mathrm{Gibbs}}(\pi, \bar{E}) \ge 0.$$
2. **Exact Pythagorean Information Contraction:**
   $$I_1(P) = H_{\mathrm{marg}}(\pi) - H_1(P) = D_{\mathrm{KL}}^{\mathrm{bits}}(P \| P_{\beta^*}) + I_{\mathrm{Leg}}^{\mathrm{bits}}(\bar{E}), \qquad D_{\mathrm{KL}}^{\mathrm{bits}}(P \| P_{\beta^*}) \ge 0.$$
3. **Exact Legendre Biconjugate Involution (interior $\beta^* \in \mathbb{R}$):**
   $$\Psi(\beta^*) = \beta^* \bar{E} - I_{\mathrm{Leg}}(\bar{E}).$$

**Verified precision across `105,000` synthetic cyclic chains + `447/447` real blocks (`0` violations):**
- Max Legendre–Gibbs duality error: **`1.0e-15`**
- Max Pythagorean contraction error: **`5.3e-15`**
- Max Legendre biconjugate involution error: **`5.6e-17`**

### 1.2 Theorem D6-1B (Markov 1906/1913 2nd-Order Chain Extension & 8-Level Hierarchy)

Tabulating the $R$ cyclic 2nd-order record-length triples $(R_t, R_{t+1 \bmod R}, R_{t+2 \bmod R})$ yields counts $N_{u,v,w}^{\mathrm{cyc}}$ satisfying exact 2nd-order De Bruijn flow balance $\sum_w N_{u,v,w}^{\mathrm{cyc}} = N_{u,v}^{\mathrm{cyc}} = \sum_s N_{s,u,v}^{\mathrm{cyc}}$ and exact 2nd-order Perron–Frobenius (`CA-42`) stationarity $\pi^{(2)\top} P^{(2)} = \pi^{(2)\top}$ (verified max error **`1.7e-16`**). Defining the 2nd-order conditional entropy $H_2(P) = H(R_t \mid R_{t-1}, R_{t-2})$, 2nd-order conditional mutual information $I_2 = H_1(P) - H_2(P) \ge 0$, and 2nd-order Boltzmann–Markov multiplicity $W_2(\mathbf{N}^{(2)}) = \prod_{u,v} N_{u,v}! / \prod_{u,v,w} N_{u,v,w}!$, every block obeys the **strict 8-level entropy hierarchy**:

$$\frac{1}{R}\log_2 W_2(\mathbf{N}^{(2)}) \;\le\; H_2(P) \;\le\; H_1(P) \;\le\; H_{\mathrm{Gibbs}}(\pi, \bar{E}) \;\le\; H_{\mathrm{marg}}(\pi) \;\le\; H_{\mathrm{Joint}}(\mu, \mu_{\log}; C) \;\le\; \min(H_\zeta, H_{\mathrm{Geom}}).$$

**Key Empirical Finding across All `447/447` Real Blocks (`0` violations):**

| Corpus | $N$ | $W_2$ | $H_2$ | $H_1$ | $H_{\mathrm{Gibbs}}(\mathrm{TT})$ | $H_{\mathrm{marg}}$ | $H_{\mathrm{Joint}}$ | $H_\zeta$ | $H_{\mathrm{Geom}}$ | $I_1$ (b) | $I_2$ (b) | $I_{\mathrm{Leg}}^{\mathrm{TT}}$ | $I_{\mathrm{Leg}}^{\mathrm{CT}}$ | `WrapD` | LinGap `[L16/L12/L10]` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 6 Admitted | 6 | 1.494 | 1.584 | 1.806 | 2.046 | 2.193 | 2.497 | 2.778 | 4.765 | 0.387 | 0.222 | 0.1468 | 0.1783 | 0 | `0.68% / 0.44% / 0.35%` |
| 6 JSONL/Log | 6 | 1.407 | 1.482 | 1.714 | 1.963 | 2.281 | 2.767 | 3.306 | 5.416 | 0.567 | 0.233 | 0.3175 | 0.2693 | 0 | `1.93% / 0.80% / 0.68%` |
| 4 Code/Docs | 4 | 1.681 | 1.718 | 1.819 | 1.843 | 1.844 | 1.855 | 1.857 | 2.508 | 0.025 | **0.102** | 0.0011 | 0.0089 | 0 | `0.09% / 0.06% / 0.05%` |
| **233 FullWiki** | 233 | **1.405** | **1.507** | **1.763** | **2.042** | **2.232** | **2.631** | **3.021** | **5.118** | **0.468** | **0.257** | **0.1901** | **0.2109** | **0** | **`0.99% / 0.67% / 0.50%`** |
| 198 TinyWiki | 198 | 0.332 | 0.439 | 0.513 | 0.571 | 0.571 | 0.603 | 0.657 | 0.627 | 0.059 | 0.074 | 0.0000 | 0.0000 | 0 | `0.00% / 0.00% / 0.00%` |
| 431 AllWiki | 431 | 0.912 | 1.016 | 1.189 | 1.366 | 1.469 | 1.699 | 1.935 | 3.055 | 0.280 | 0.173 | 0.1028 | 0.1140 | 0 | `0.54% / 0.36% / 0.27%` |
| `dense-300k` | 1 | 0.304 | 0.305 | 0.305 | 0.305 | 0.305 | 0.305 | 0.308 | 0.306 | 0.000 | 0.000 | 0.0000 | 0.0000 | 0 | `0.00% / 0.00% / 0.00%` |

Notably, real BWT columns exhibit substantial **2nd-order Markov memory beyond 1st order**: $I_2 = 0.257$ bits on `233 FullWiki` (`55%` of $I_1$), and on `4 Code/Docs` $I_2 = 0.102$ bits is **`4.1×` larger than $I_1 = 0.025$ bits**!

### 1.3 Theorem D6-2A & D6-2B (Order-$k$ Record-Length Invariance & Cycle 54 Boundary Note #1 Resolution)

- **Linear vs Cyclic Wrap Discrepancy (Closing Cycle 54 Referee Boundary Note #1):**
  For any `CAP = C` record sequence $x_0, \dots, x_{R-1}$, the linear (non-wrapping) adjacency lower bound $m_{\min}^{\mathrm{adj,lin}}$ and the cyclic adjacency lower bound $m_{\min}^{\mathrm{adj,cyc}}$ satisfy the exact identity
  $$0 \;\le\; m_{\min}^{\mathrm{adj,lin}} - m_{\min}^{\mathrm{adj,cyc}} \;=\; x_0 \cdot \mathbf{1}[x_{R-1} = C \land x_0 \le L] \;\le\; L, \qquad m_{\min}^{\mathrm{adj,cyc}} \le m_{\min}^{\mathrm{adj,lin}} \le m_{\mathrm{act}} \le m_{\max}.$$
  Verified across all `105,000` synthetic chains (`max_wrap_delta = 126 <= L`) and all `447/447` real blocks (`wrap_hits = 0`).
- **Theorem D6-2A (Exact Order-$k$ Invariance of the Remaining `m`-Gap):**
  Even conditioning on the **entire exact sequence of `CAP = C` record lengths** $(x_0, \dots, x_{R-1})$ — hence bit-for-bit identical **linear AND cyclic** $k$-th order transition tensors $N^{(k)}_X$ for ALL $k \ge 1$ — cannot narrow $[m_{\min}^{\mathrm{adj,lin}}, m_{\max}]$ by a single row! Explicitly verified on `4` primitive single-cycle BWT pairs $(B_1, B_2)$ with `x1 == x2` element-for-element:
  - `(C=15, L=4, q=5, n=96)`: $m(B_1) = 22 = m_{\max}$ vs $m(B_2) = 1 = m_{\min}^{\mathrm{adj,lin}}$ (**`22.0×`, `100.0%` of gap!**)
  - `(C=31, L=8, q=8, n=313)`: $m(B_1) = 66$ vs $m(B_2) = 1$ (**`66.0×`, `100.0%` of gap!**)
  - `(C=127, L=16, q=20, n=2861)`: $m(B_1) = 322$ vs $m(B_2) = 1$ (**`322.0×`, `100.0%` of gap!**)
  - `(C=127, L=16, q=50, n=7151)`: $m(B_1) = 802$ vs $m(B_2) = 1$ (**`802.0×`, `100.0%` of gap!**)
- **Theorem D6-2B (Global Cycle-Topology Barrier Beyond ALL 2nd-Order Run & Record Triples):**
  Even conditioning on **element-for-element identical merged run-length sequences `wa == wb`** AND **identical `CAP = 127` record sequences `xa == xb`** (hence identical linear & cyclic order-$k$ tensors for all $k \ge 1$, identical $H_2, H_1, I_2, I_{\mathrm{Leg}}$, AND identical $m_{\mathrm{act}} = 2$), `S2b` grammar depth still diverges arbitrarily! Constructively verified on `4` primitive single-cycle pairs `(Clustered A = 0 · 1^(fq-1) · 2^fp · 3, Christoffel B)`:
  - `Fib(55, 89)` ($n = 145$): `wa == wb = [1, 1, 88, 55]` -> `145` vs `16` steps (**`9.1×`**), `418` vs `31` rule words (**`13.5×`**)
  - `Fib(89, 144)` ($n = 234$): `wa == wb = [1, 1, 143, 89]` -> `234` vs `17` steps (**`13.8×`**), `685` vs `34` rule words (**`20.1×`**)
  - `Fib(144, 233)` ($n = 378$): `wa == wb = [1, 1, 232, 144]` -> `378` vs `19` steps (**`19.9×`**), `1,239` vs `41` rule words (**`30.2×`**)
  - `Fib(233, 377)` ($n = 611$): `wa == wb = [1, 1, 376, 233]` -> `611` vs `20` steps (**`30.6×`**), `2,171` vs `45` rule words (**`48.2×`**)

---

## 2. Architecture of `#729` Drop-In Candidate `C55-DramUltra-v2`

`C55-DramUltra-v2` ([`cand_c55_dramultra_v2_moonshot.rs`](cand_c55_dramultra_v2_moonshot.rs)) fuses the orthogonal strengths of both Cycle 54 champions (`C54-DramLean-v1` + `C54-PipeLean-v2`) and introduces three new verified **`2020s` modern CS** mechanisms:

1. **4-Regime Density-Adaptive Retention Cutoff (`LIM ∈ {8, 10, 12, 24, 32}`, `CA-41`-certified):**
   - **`nrec/n > 0.24` (`nrec * 100 > n * 24`) -> `L = 12`**: protects ultra-high-entropy Code/Docs (`bits = 20`, `sentinel = 4,095`) so `num_codes <= 3,559 < 4,096` (`0` `wide` `self.extra` allocations, `0` `u32_fb` hits across all `447/447` real blocks, `PeakCap = 3,190.23 KiB < R5-Head 3,331.28 KiB`).
   - **`0.14 <= nrec/n <= 0.24` -> `L = 8`**: clamps the `9` densest LLC-thrashing Wikidata blocks (`blk 41, 56, 165, 270, 278, 387, ...`, where `nrec/n ∈ [0.14, 0.21]`), cutting `max(edges)` across all `431` Wiki blocks from `858.00 KiB` (`C54-DramLean-v1`) to **`769.47 KiB`** (`-88.53 KiB`!) and dropping **`PeakCap` to `886.73 KiB`** (`-73.27 KiB` vs `C54-DramLean-v1`, `-112.22 KiB` vs `C54-PipeLean-v2`!).
   - **`est_w1 <= 33,500` (`64 -> 256` strided sampler) -> `L = 24 / 32`**: preserves sparse repetitive JSONL/Syslog dominance (`0.8807×` vs `R5-Head`, `99.3%` of oracle headroom!).
   - **Default (`221` normal FullWiki blocks, including all `6` full `#729` proxy blocks) -> `L = 10`**: cuts `256 KiB` L2 misses by **`-15.9%`** vs `C54-PipeLean-v2` (`-5.4%` vs `C54-DramLean-v1`) and `1.0 MiB` effective LLC misses by **`-12.0%`** vs `C54-PipeLean-v2` (`-3.7%` vs `C54-DramLean-v1`)!
   - **Exact `CA-41` Pre-`S1` Sound Reject**: `nrec * 13 >= n * 10` (since `10/13 = 0.7692 > 0.24` routes to `L = 12 <= 63 = L*(127)`, guaranteeing `12*m(12) >= 13*nrec - n => m(12) >= 3n/4`, `0` false rejects!).
2. **Zero-Memset `S5_Pow2Dict2048` + `S5_PackPrevNext` + `S5_UnalignedBack_1Ptr` + `ZeroSpill_ConstBits`:**
   - Tracks dictionary length via persistent `num_codes` so `inline16` (`>= 2049` slots with `+16 B` leading guard at `inline16[0]`) and `lens` (`>= 2048` slots) are NEVER truncated or zero-filled between blocks (`0` wasted cache-line dirties!).
   - Whenever `num_codes <= 2048 && !wide && bits ∈ {17, 18, 19}` (`100%` of `#729` proxy blocks), dispatches `emit_bi_pow2_const::<BITS>` taking fixed `&[[u8; 16]; 2049]` and `&[u8; 2048]` with `(edge >> BITS) & 2047` and packing `(prev, next)` into `pn: u64`, eliminating `len_c`, ALL `lens` & `inline_flat` bounds checks, and `lens.ptr` stack reloads (`-5.65M` kernel `Ir` vs `C54-DramLean-v1`, recovering `85%` of `C54-PipeLean-v2`'s instruction lead while running at `L = 10`!).
3. **`S4_8WayEpoch` + `S4_SlowEpoch8` + `4 KiB` Stack `u16` Dict + `TightSteps`:**
   - Extends `S4_8WayEpoch` (`&mut [u32; 65_536]` epoch-slice bounds elimination) into `wire_xor_slow_nomark` (`-350K` kernel `Ir`).
   - Retains `4 KiB` stack `u16` `Dictionary::contents` + `2 KiB` stack `pairs` fused into `V3CodeMiss` (`0` `u32_fb` hits on `447/447` real blocks) + tighter growth steps (`SPELLING_STEP = 1024`, `INLINE_STEP = 64`, `MEMO_REC_CAP = 64`, `edges` step `2048`, `spill.next` `128`, `spill.ovf` `8`).

---

## 3. Honest Cycle 55 Ablation & Kill Ledger (Rule 5)

Every rejected mechanism was benchmarked in `valgrind --tool=cachegrind` (`x86-64-v3`) and native `Full233` timing and is retained below with its exact quantitative kill reason:

| Mechanism | Target Stage | Quantitative Kill Reason | Status |
|---|---|---|---|
| `Mini64-OutQ` / `Mini128-OutQ` 2-tier slabs (`CS-55a`) | `S1 / S4` | `+2.62M..+4.28M Ir`, `+5,526..+6,033` `256K` L2 misses, `+1,900` `W8/W10` misses from shared-chunk false locality & extra `push_slow_pt` | **KILLED** |
| `S2b_LocalRuleTop / as_chunks_mut::<4>()` (`CS-55b`) | `S2b` | `+1.51M Ir` in `eliminate_narrow_plain` (`10.32M -> 11.83M`) from register pressure | **KILLED** |
| `S1_ByteOff` (`pt` byte offset + `1023` sentinel) (`CS-55c`) | `S1` | `+786K Ir` in `try_run_const` (`16.62M -> 17.40M`) from extra masking & `at + 32` check | **KILLED** |
| `S1_HoistOutM` (`out_q` outside `Queues`) (`CS-55d`) | `S1` | `+1.86M Ir` from extra live parameter register pressure across `push_slow_pt` | **KILLED** |
| `S1_SplitMark` (2-phase `pos <= p` / `pos > p`) (`CS-55e`) | `S1` | `+658K Ir` & larger hot-loop code footprint | **KILLED** |
| `S1_W1Fast` (`if width == 1` 2B store) (`CS-55f`) | `S1` | `-606K Ir` in `push_slow_pt`, but `+13.0 ms` (`288.96 -> 301.96 ms`) native Xeon slowdown from `w == 1` branch mispredicts | **KILLED** |
| `assert!(dst_start + span <= edges.len())` in `S4` (`CS-55g`) | `S4` | `+104K Ir` in `wire_bi_tail` (`5.62M -> 5.72M`) | **KILLED** |
| `EmitPow2` cold-ctx struct (`CS-55h`) | `S5` | `+654K Ir` (`64.47M -> 65.13M`) vs direct `S5_Pow2Dict2048 + S5_PackPrevNext` | **KILLED** |
| Unbounded `nrec/n >= 0.14 -> L = 8` on Code/Docs (`CS-55i`) | `select_limit` | Pushed `4 Code/Docs` to `5,214 > 4,095` codes (`wide = true`, `+2,209 KiB` `self.extra`); fixed by `0.14 <= nrec/n <= 0.24 -> L = 8; > 0.24 -> L = 12` (`0` `u32_fb`, `0` `wide`, `max_codes_447 = 3,559`) | **FIXED** |
| **Per-block `resize(2048)` / `truncate` (`CS-55j`)** | `S5_Pow2` | `+3,387` L2 misses & `+2,022` `1.0M` LLC misses from zeroing unused `inline16` tail; fixed by Zero-Memset persistent `num_codes` | **FIXED** |

---

## 4. Summary of Verified Results (`C55-DramUltra-v2` vs Cycle 54 Dual Champions)

### 4.1 Exact Head-to-Head `#729` Drop-In Cachegrind (`x86-64-v3`, `10` Proxy Blocks + Warm Block 1)

Two-point linear calibration anchored directly on live `#729` i9-10900K measurements of `C54-PipeLean-v2` (`instr=0.757`, `L3=14.50`, `W=8=1.32`, `W=10=4.71`) and `C54-DramLean-v1` (`instr=0.807`, `L3=13.70`, `W=8=0.98`, `W=10=4.60`):

| `#729` Drop-In Candidate | `Cand_Ir` (exact) | `ΔIr` vs `PL2` | Calibrated `#729 instr` | `L2(256K)` misses | Calibrated `#729 L3` | `LLC(1.0M)` misses | Calibrated `#729 W=8 DRAM` | Calibrated `#729 W=10 DRAM` | `431` Wiki `PeakCap` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `C54-PipeLean-v2` (`L=16`) | `63,480,005` | `0` | `0.757` (live) | `465,125` | `14.50` (live) | `53,394` | `1.32` (live) | `4.71` (live) | `998.95 KiB` |
| `C54-DramLean-v1` (`L=12`) | `70,123,874` | `+6,643,869` | `0.807` (live) | `413,506` | `13.70` (live) | `48,772` | `0.98` (live) | `4.60` (live) | `960.00 KiB` |
| **`C55-DramUltra-v2`** (`L∈{8,10,12,24,32}`) | **`64,472,747`** | **`+992,742`** (`-5.65M` vs `DL1`!) | **`0.764`** | **`391,223`** (`-15.9%` vs `PL2`, `-5.4%` vs `DL1`!) | **`13.35`** | **`46,982`** (`-12.0%` vs `PL2`, `-3.7%` vs `DL1`!) | **`0.85`** | **`4.56`** (proxy; **`<= 4.54`** w/ `L=8` dense Wiki!) | **`886.73 KiB`** (`-73.3K` vs `DL1`, `-112.2K` vs `PL2`!) |

### 4.2 Headroom Oracle v10 (`441` Distinct Blocks = `431` Wiki + `6` JSONL/Log + `4` Code/Docs, Native Xeon)

| Arm | `1T` Total (`441` blks) | `2W` p50 (`441` blks) | `4W` p50 (`441` blks) | Beats `R5-Head` on ALL `1T/2W/4W`? |
|---|---:|---:|---:|:---:|
| `R5-Head-A1` | `324.14 ms` (`1.0000×`) | `536.70 ms` (`1.0000×`) | `768.86 ms` (`1.0000×`) | baseline |
| `C54-PipeLean-v2` | `290.36 ms` (`0.8958×`) | `456.26 ms` (`0.8501×`) | `788.32 ms` (`1.0253×`) | NO (`4W`) |
| `C54-DramLean-v1` | `314.51 ms` (`0.9703×`) | `549.60 ms` (`1.0240×`) | `854.82 ms` (`1.1118×`) | NO (`2W`, `4W`) |
| `Abl-Fused-L12` | `299.25 ms` (`0.9232×`) | `532.75 ms` (`0.9926×`) | `813.47 ms` (`1.0580×`) | NO (`4W`) |
| `Abl-Pow2-L12` | `298.23 ms` (`0.9201×`) | `513.03 ms` (`0.9559×`) | `789.36 ms` (`1.0267×`) | NO (`4W`) |
| `Abl-Pow2-L10` | `302.94 ms` (`0.9346×`) | `541.72 ms` (`1.0093×`) | `727.91 ms` (`0.9467×`) | NO (`2W`) |
| **`C55-DramUltra-v2`** | **`301.46 ms` (`0.9300×`)** | **`509.84 ms` (`0.9499×`)** | **`726.04 ms` (`0.9443×`, `#1`!)** | **YES (ONLY ALL-YES ARM!)** |

`C55-DramUltra-v2` is the **only** candidate in the 8-arm Headroom Oracle v10 suite to beat `R5-Head` across **all three concurrency levels (`1T`, `2W`, `4W`)** AND wins **`#1` overall at `4W` (`726.04 ms`, `-7.9%` faster than `C54-PipeLean-v2` and `-15.1%` faster than `C54-DramLean-v1`)**!
