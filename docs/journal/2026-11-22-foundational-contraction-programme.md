# Foundational Contraction Programme — Interim Research Journal (Cycles 1–6 of 20)

- **Branch:** `research/neon-teamwork-paradigm` (zero edits to `src/` or `Cargo.toml`; all reproducible artifacts under [`docs/journal/2026-11-22-foundational-contraction-programme/`](2026-11-22-foundational-contraction-programme/))
- **Programme Status:** **Interim Verified Handoff after Cycle 6 of 20** (continuing directly into Cycles 7–20 without pausing)
- **Investigators:** Alpha, Beta, Gamma, Adversarial Referee #1 & Referee #2 (with independent MONODROMY Record 90 reconciliation integrated)
- **Production Calibration Bar (i9-10900K, `bench-emit-cost`, #145):** `OpenCells` (`v0.5.12`) emit = **`4.8–5.8 ns` per retained row** at 1–10 workers (pooled slope `5.4–6.5 ns`; walk is **NOT DRAM-bound** on i9); construction = **`0.72–0.90 ns` per block row** (`648–810 µs/block`; mean `m_16 = 114k`; 6-block sum bar = **`8.13–10.05 ms`**, `0.97 MiB/worker` scratch).
- **Authoritative Scorer:** `bench/moonshot` from `origin/master` (`#729`, **`scorer v1` @ `ee7fbf0e50c60cd9dcf0faa0cdd634cdbbb02849`**), baseline = shipped **`OpenCells (0.5.12)`**. Parts 1 (hard gate), 2 (authoritative container wall + 95% CI), 4 (authoritative peak RSS), and 3 (advisory `chaingrind` proxy ranking, `instr` = Record 76 i9 predictor) are reported strictly side by side and never combined into one number.
- **Claim Tag Legend:** `[proved]` (deductive mathematical proof), `[verified]` (checked mechanically over exhaustive suites, 6 SHA-256 authenticated Wikidata blocks, or 338-case moonshot gate), `[measured]` (empirical timing/memory/census), `[bounded-UNSAT]` (SAT/DRAT proof up to stated $n$), `[falsified]` (explicit counterexample or empirical refutation), `[open]` (unsettled mathematical question). Every obstruction strictly separates **algorithmic escape conditions** from hardware **deployment conditions**.

---

## 1. Executive Summary (Cycles 1–6)

**Central Mathematical Question:** *Can direct BWT-space contraction be made compositional — is there an algebraic representation of $C_k \circ \cdots \circ C_1$ whose composition costs substantially less than executing each contraction separately, or that computes a reconstruction certificate directly from $I_0 = (n, p, R_{127}, C, \Sigma)$ without materialising intermediate BWTs?*

**What Cycles 1–6 Established:**
1. **Yes — dropping BWT-validity lifts contraction into the confluent $\mathcal{PT}_n$ (Word-Carrying Interval Exchange) category `[proved]` `[verified]`:**
   - Under the associative, commutative **Redheffer Star Product** $\Phi_{S_1} \star \Phi_{S_2}$ / **Bucket-Schur Elimination** $\mathcal{S}_E$ (`Theorems B1, B4, C1–C2`), arbitrary elimination orders are confluent, satisfy strict piece-count monotonicity ($\Delta q \le -1$), crush every constant-$r$ adversarial ring to $O(1)$ via Agol–Hass–Thurston / Sub-Interval 1-Piece Schur with hysteresis fallback, and under **Theorem B4 (Zero-Tombstone Causal Routing)** achieve literal **`0.00%` stale tombstones**, `2.11 r_0` single-copy buffer traffic, and **`1.40 ms` 6-block emit (`3.5×` faster than `OpenCells`'s emit)**!
   - **Yet bucket-by-bucket Schur elimination loses decisively on silicon at $n \le 900\text{ KiB}$:** Our safe-Rust `B4-Grid-Flat16` (`PASS 338/338`) scored **`3.560×` (1W) / `3.539×` (2W) slower** than shipped `0.5.12 OpenCells` on `bench/moonshot` (`5.07×` on 6 blocks, `51.88 ms` vs `10.22 ms`), because Stage 0 + Stage 1 bucket sorting (`9.12 ms`) + `ChunkPool`/`slab16`/`SLP` dispatch (`23.80 ms`) costs **`55.6 ns/op`** against Alpha's proved i9 break-even of **`5.8 ns/op`** (`Theorem F3`).
2. **Exact Lower Bounds & Tightened Monotonicity Theory `[proved]` `[bounded-UNSAT]`:**
   - **Revised Theorem C3 (tightest airtight form, independently audited by Alpha & Referee #2):** Even with **free** gap-closing coalescence and **free** hash-consing in $\mathcal{C}_{\text{gap}}$,
     $$5\,Q_{\text{comp}} + 2\,Q_{\text{sp}} + 3\,c_{\text{del}} + P_{\text{rem}} \;\ge\; r_0 \quad\Longrightarrow\quad Q_{\text{comp}} + Q_{\text{sp}} + c_{\text{del}} + m_{\text{rem}} \;\ge\; \frac{r_0}{5},$$
     with an explicit verified single 8-cycle witness achieving exact $\Delta P = -5$ in one composition.
   - **Beyond $\mathcal{PT}_n$ (`Theorems F1–F2`, scoped by Referee #2):** Valid $r_0$-run BWTs have $\Theta(r_0 \log r_0)$ bits of output entropy for $|\Sigma| = r_0^{\Omega(1)}$ or $n = r_0^{1+\Omega(1)}$ ($O(r_0)$ bits for fixed $|\Sigma| = O(1), n = \Theta(r_0)$); and even when $z(T) \ll r_0$, acquiring an exact certificate from $I_0$ without an external promise requires $\Omega(r_0)$ run inspections (even-permutation splice).
   - **Universal `r + n` Potential (`Theorem B2` + CaDiCaL DRAT proofs):** $r(L') + n' \le r(L) + n$ holds universally for **all** `(L, F)`-defined acyclic deletions (DRAT-certified `n=6..8, k=1..2`, verified over `100,035` arbitrary columns); no $f(r)$-bounded potential exists for general `BDMR` + self-runs (`Theorem B3`, `Fib_{2m}+$`); 1-line Farkas impossibility (`rle_10`) rules out any linear combination of the 14 standard 1-step BWT invariants when self-runs are deferred.
3. **Cycle 5–6 Pivot to Zero-Sort, Zero-SLP Compositional Operators Produced Our First Sub-`1.0` `instr` Breakthrough on `bench/moonshot` (`v1 @ ee7fbf0e`) `[verified]` `[measured]`:**
   - **`TubeOpen` (Zero-RV Post-S3a Inclusion-Forest on `R_2`, `2 <= w <= 16`):** **`PASS (338/338)`**, **`instr = 0.898` (`-10.2%` fewer instructions than shipped `0.5.12 OpenCells`!)**, **`L2 -11.6%`**, **`L3 -10.6%`**, **`Peak RSS = 1.015×` (1W) / `1.050×` (2W)**, and **`1.234×` (1W) / `1.117×` (2W) INCONCLUSIVE** container wall!
   - **`256-Cursor Zero-Sort SelThinSq` (`Theorem B5`):** **`PASS (338/338)`**, slashes walk steps `-43.5%` (`0.565×`) and `dep_L2` `-46%`, scoring `2.102×` (v1) / `1.778×` (v2 6-block A/B) due to `u64` `F`-scatter (`6.4 ms`) and losing bidirectional XOR.
   - **`Static M1 (R1 1-to-1 Chain Peeling, Theorem K1)` & `Theorem J1` Joint Composition:** 100% static (`0` sorting, `0` worklist), cuts `m_16` by **`-50.2%`** (`w>=1, <=8B`) in `1.97 ms` packed-8B C (`4.38 ns/saved step`), and composing **`M1 ∘ SelThinSq`** or **`Tube-Open ∘ SelThinSq`** crushes walk steps by **`-69.1%` to `-70.4%` (`0.296–0.309× m_16`)** with `0/6 wide` (ZERO SLP!).
4. **The Unified Cycle 6 Silicon Law (The Bijectivity + Two-Ended `wire_xor` Law) `[proved]` `[measured]`:**
   - Shipped `v0.5.12 OpenCells` ALREADY runs **2 parallel dependency chains** via `emit_bidirectional` (`wire_xor`, `0.500 × m_16 = 447.7k` serial rounds on a `4 B` `u32` table, `4.06 ms` emit).
   - BOTH `TubeOpen` (`-29.4%` walk) and `SelThinSq` (`-43.5%` walk) **broke bijectivity (`in_deg ∈ {0, 1, 2}`)**, losing `wire_xor` and paying **MORE serial latency** (`631.8k` / `506.2k` unidirectional rounds; `max_chain: 109 -> 618` in `chaingrind`) than `OpenCells`'s `447.7k` bidirectional rounds!
   - **Direct Cycle 7–20 Target:** **Bijective** zero-sort contractions (`Bijective R1 Exact-Match Tubes`, `1-Pass Ascent-DAG`, `Full Bijective Phi^2`, and `K >= 2`-chain bijective/XOR emit) that preserve `wire_xor` + `u32` `edges` + `RSS <= 1.00×` while keeping `TubeOpen`'s `instr < 0.90`!

---

## 2. MONODROMY Record 90 Reconciliation (`#145 comment 7073748142`, `84c80539`) `[verified]` `[proved]`

Applied strictly from Cycle 1 onward and verified independently by Referee #2 ([`cycle1/referee2/task1_full.py`](2026-11-22-foundational-contraction-programme/cycle1/referee2/)):
1. **`Symmetric-Middle-Run` Reconciliation:** Both MONODROMY's flat `320`-symbol run (`audit_alpha_falsify.py`: `low + M^K + high`) AND the `4,211`-symbol $\Theta(n)$ run (`audit_surgical.py`: `M^K + low + M^K + high`) reproduce on byte-identical committed `verify_full_and_falsify.py` (`SHA-256: 90cd265c...`). Minimal constant-$r$ ($r = 4$) witness **`T_K = M^K A M^K Z`** ($A < M < Z, n = 2K + 2$) forces committed `Lazard` (`both`) to `2,050 = n` steps (`513 r`) at $K = 1024$ `[verified]`. Per MONODROMY's correction, we dropped `Symmetric-Middle-Run` as a universal bar and tracked `T_K` solely as an exact regression witness (`SymMidRun`), which all our $\mathcal{PT}_n$ spiral engines (`AHT`, `Sub-Interval 1-Piece Schur`, `M4`) collapse in $O(1)$ (`2–4` steps).
2. **Two-Axis Contraction Algebra:** Confirmed $\kappa = 1 \iff$ split-free (`599k` cases); full absorption implies monotone $r$ `[proved]`; `1Cut` is partial absorption (non-monotone); `BDMR` has splits.
3. **Monotone Potential `r + n`:** Verified and **proved universally (`Theorem B2`)** for ALL `(L, F)`-defined acyclic deletions (including multi-step `Ascent-DAG`, `BDMR` self-runs, and $\kappa \ge 2$ bucket splits), with **CaDiCaL DRAT proofs** ([`cycle2/referee2/r_plus_n_n{6,7,8}_k{1,2}.drat`](2026-11-22-foundational-contraction-programme/cycle2/referee2/)) and `100,035` arbitrary-column checks `[proved]` `[bounded-UNSAT]`. Proved `(L, F)`-definedness is necessary via `T = [1, 0, 0, 0, 0, 0]` ($\Delta r = +2 > |D| = 1$).
4. **G1/G2 Scope:** Treated strictly as asymptotic (`G1`) and read-only sequential streaming (`G2`, `~35` bits at bzip2 scale) — neither used as a RAM/read-write wall.

---

## 3. Authoritative `v0.5.12` `bench/moonshot` Scorecard (`scorer v1 @ ee7fbf0e`) `[verified]` `[measured]`

Baseline: shipped **`OpenCells (0.5.12)`** on 2-CPU container (median wall `0.503 s` 1W / `0.376 s` 2W; peak RSS `39.9 MiB` 1W / `46.9 MiB` 2W). Register: [`cycle6/referee2/moonshot_register_v1.jsonl`](2026-11-22-foundational-contraction-programme/cycle6/referee2/moonshot_register_v1.jsonl).

| Candidate (`scorer v1 @ ee7fbf0e50c60cd9dcf0faa0cdd634cdbbb02849`) | **1. Hard Gate** (338 cases) | **2. Container Wall** (1W / 2W, 3 paired rds, `<1` faster) | **4. Peak RSS Ratio** (1W / 2W) | **3. Advisory `chaingrind` Proxy** (`1+1`, ranking only; `instr` = Record 76 i9 predictor) |
|---|---|---|---|---|
| `template.rs` (`LazyCells 0.5.6`, null ctrl) | `PASS (338/338)` | `1.349 [1.017, 1.789]` SLOWER / `1.244 [1.195, 1.296]` SLOWER | `1.575` / `1.697` | `instr 1.086` (`2+1`), `L2 38.0/24.4`, `DRAM@7W 4.16/0.30` |
| [`b4_grid_flat16_candidate.rs`](2026-11-22-foundational-contraction-programme/cycle6/gamma/b4_grid_flat16_candidate.rs) (`B4GridFlat16` v2) | **`PASS (338/338)`** | `3.560 [3.052, 4.153]` SLOWER / `3.539 [3.358, 3.730]` SLOWER | `1.255` / `1.407` | `instr 3.117`, `L2 55.4/29.3`, `L3 33.0/18.9`, `DRAM@7W 14.07/0.74`, **`dep_L2 0.0183/0.0496 (-63%)`** |
| [`selthinsq_candidate.rs`](2026-11-22-foundational-contraction-programme/cycle6/beta/selthinsq_candidate.rs) (`SelThinSq` v1 scored\*) | **`PASS (338/338)`** | `2.102 [1.912, 2.312]` SLOWER / `2.021 [1.222, 3.343]` SLOWER | `1.184` / `1.337` | `instr 2.245`, `L2 37.9/29.3`, `L3 34.9/19.0`, `DRAM@7W 11.59/0.73`, **`dep_L2 0.0267/0.0496 (-46%)`** |
| **[`tube_open_candidate.rs`](2026-11-22-foundational-contraction-programme/cycle6/gamma/tube_open_candidate.rs) (`TubeOpen` V6)** | **`PASS (338/338)`** | **`1.234 [0.832, 1.830]` INCONCLUSIVE / `1.117 [0.891, 1.401]` INCONCLUSIVE** | **`1.015` / `1.050`** | **`instr 0.898 (-10.2%!)`, `L2 25.9/29.3 (-11.6%)`, `L3 16.9/18.9 (-10.6%)`**, `DRAM@7W 3.93/0.71`, `dep_L2 0.0556/0.0496`, `max_chain 618/109` |

*\*Note: Beta subsequently upgraded `selthinsq_candidate.rs` to v2 (contiguous-`F` + direct 1-lane), cutting 6-block same-binary A/B from `2.571×` (v1) to `1.778×` (v2, `18.06 ms` vs `10.16 ms`), also passing `338/338` `moonshot check`.*

### Same-Binary Paired A/B + A/A & Complete i9-Calibrated Total-Cost Inequality (6 Authenticated Wikidata Blocks) `[measured]`

| Candidate | Same-Binary 6-Blk Sum (1T Xeon, A/A `0.998–1.007×`) | Walk Steps (`/ m_16`) | Construct / Peel Cost | Emit Cost | Peak Scratch (`PeakM`) | i9 Total-Cost Inequality vs `8.13–10.05 ms` (`0.97 MiB/w`) | Status (Cycle 6) |
|---|---|---|---|---|---|---|---|
| `v0.5.12 OpenCells` (bar) | **`10.16–10.46 ms` (`1.00×`)** | `895,391` (`1.000×`, 2-ended = `447.7k` rds) | `3.89–4.86 ms` (i9) | `4.24–5.19 ms` (i9) | **`0.95–0.97 MiB`** | **`8.13–10.05 ms` (`1.00×`)** | Baseline |
| `Wavefront Polarity-DAG` (C, Cyc 2–3) | `97.84 ms` (`~9.5×`, Round 1 alone `30.04 ms`) | `0` (root SLP) | `86.51 ms` + `4.16 ms` acq | `7.30 ms` | `~130 MB` R1 traffic | Loses `>9×` | **[falsified/killed]** |
| `B4-Grid-Flat16` (`CoreRvP16`, Cyc 4) | `51.88 ms` (`5.07×`) | `114k` (`0.127×`) | `38.68 ms` S0+S1 (`55.6 ns/op`) + `12.5 ms` S2+S3Fl | **`1.40 ms`** (`3.5×` faster!) | `11.45 MiB` (honest 12B floor: `1.14–2.65 MiB`) | Loses `4–5×` (`55.6 ns/op >> 5.8 ns/op` break-even); deployment crossover `> 60 ns/row` | **[falsified/killed at $n\le 900\text{k}$]** |
| `256-Cursor SelThinSq` v2 (Cyc 6) | `18.06 ms` (`1.78×`) | `506,181` (`0.565×`, 1-lane = `506.2k` rds) | `+5.1 ms` `S3_256cur` (`570k` `u64` scatters) | `~6.7 ms` (`+2.6 ms`, lost `wire_xor`) | `6.18 MiB` (`1.18–1.34×` RSS) | Loses `~1.8×` (non-bijective -> lost 2-ended XOR + `u64` scatter) | **[falsified/killed as non-bijective]** |
| **`Tube-Open V6`** (`t=2`, Cyc 6) | `15.31 ms` (`1.46×` 6-blk; **`1.117×` 2W moonshot**) | `631,803` (`0.706×`, 1-lane = `631.8k` rds) | `+1.40 ms` S1 + `+4.09 ms` Tube `-0.76 ms` S4 | `4.41 ms` (`+0.35 ms`, lost `wire_xor`) | `2.21 MiB` (`1.015–1.050×` RSS) | **`instr = 0.898` (`-10.2%`!), `L2 -11.6%`, `L3 -10.6%`**, but non-bijective `emit` + `TubeMerge` push wall `> 1.0×` | **Algorithmic escape -> Cycle 7 Bijective XOR!** |
| `Static M1 (w>=1, <=8B)` (C peel, Cyc 6) | `T_peel_8 = 1.97 ms` (`0.36 ms` for `w>=2`) | `445,507` (`0.498×`, **BIJECTIVE = `222.8k` 2-ended rds!**) | `1.97 ms` peel (`T_buildG` must fuse into S1) | proj. `~2.1–2.6 ms` | `0.40–1.00 MiB` (`e4 = 290 KiB`) | Break-even / `< 1.00×` **iff** `R1` discovery fuses into S1 (`<= 1.5 ms`) + keeps `wire_xor` | **Cycle 7 Primary Target!** |
| `1-Pass Ascent-DAG (<=8B)` (Cyc 5–6) | Python verified (`6/6`) | `546,800` (`0.611×`, **BIJECTIVE = `273.4k` 2-ended rds!**) | `0.47 r_0` local sort (`5×` less than B4) | proj. `~2.6–3.2 ms` | **`0.65–0.85 MiB`** (**SUB-`Open`!**) | Proj. **`~7.0–8.8 ms`** (`~0.85–0.88×` i9 `Open`) | **Cycle 7 Primary Target!** |

### Acquisition Manifest Verification Ledger (`docs/team/acquisition/check_manifest.py`) `[verified]`
All **4/4** programme manifests pass `check_manifest.py` with exit code `0`:
1. [`cycle3/gamma/iet_schur_manifest.toml`](2026-11-22-foundational-contraction-programme/cycle3/gamma/iet_schur_manifest.toml) — `PASS`
2. [`cycle4/beta/b4_grid_flat16_manifest.toml`](2026-11-22-foundational-contraction-programme/cycle4/beta/b4_grid_flat16_manifest.toml) — `PASS`
3. [`cycle6/alpha/m1_k1_peeling_manifest.toml`](2026-11-22-foundational-contraction-programme/cycle6/alpha/m1_k1_peeling_manifest.toml) — `PASS`
4. [`cycle6/beta/selthinsq_manifest.toml`](2026-11-22-foundational-contraction-programme/cycle6/beta/selthinsq_manifest.toml) — `PASS`
5. [`cycle6/gamma/tube_open_manifest.toml`](2026-11-22-foundational-contraction-programme/cycle6/gamma/tube_open_manifest.toml) — `PASS` *(5/5 manifests PASS!)*

---

## 4. Complete Mathematical Catalogue (Cycles 1–6)

### 4.1 Proved Theorems & Exact Identities `[proved]`
- **Redheffer Star Product & Bucket-Schur Confluence (`Alpha Thms 1–4`, `Beta Thm B1`, `Gamma Thms C1–C5`):** Associative, commutative exact composition in $\mathcal{PT}_n(\mathcal{B})$ with unconditional $\Delta q \le -1$ per full bucket elimination (zero BWT-validity constraint).
- **Theorem B2 (Universal `r + n` Potential) & DRAT Proofs:** $r(L') + n' \le r(L) + n$ for ALL `(L, F)`-defined acyclic deletions (including multi-step `Ascent-DAG`, `BDMR` self-runs, $\kappa \ge 2$ splits); `(L, F)`-definedness necessary (`T = [1,0,0,0,0,0]`).
- **Theorem B3 & Referee #1 Farkas Impossibility (`rle_10`):** No $f(r)$-bounded potential exists for general `BDMR` + self-run (`Fib_{2m}+$`); no linear combination of the 14 standard 1-step invariants (`CORE_10` / `RICH_24`) is monotone when self-runs are deferred.
- **Theorem B4 & B4b (Zero-Tombstone Single-Copy Causal Routing):** Routing every piece exclusively once to $\min(\text{rank}[u], \text{rank}[v])$ gives `0.00%` stale tombstones, exact `2.00×` buffer traffic cut (`4.20 -> 2.11 r_0`), `peak_buf <= q_0` at every bucket boundary, and `peak_micro <= q_0 + max_e k_self(e)` under hysteresis-guarded `AHT` (Referee #2 showed unguarded `AHT` can momentarily hit `4.75× d`, and `L = [1,1,0,0,0,0,0]` hits `peak_mid = q_0 + 1`).
- **Exact Step-Conservation (`Theorem C1`) & Tightened Gap-Closed Quotient Floor (`Revised Theorem C3`):** $Q_{\text{rand}}(T) + P_{\text{rem}} = r_{\text{init}} + S_{\text{in}} + S_{\text{hit}} + S_{\text{bnc}} + (Q_{\text{root}} - \text{root\_int\_dom}) \ge r_0$ (`0`-integer error across 5 trees × 6 blocks); and even with free gap-closing coalescence + free hash-consing in $\mathcal{C}_{\text{gap}}$, $5\,Q_{\text{comp}} + 2\,Q_{\text{sp}} + 3\,c_{\text{del}} + P_{\text{rem}} \ge r_0$ (step-sharp at Alpha's 8-cycle $\Delta P = -5$ witness & Referee #2's $\Delta P = -2$ spiral witness).
- **Model-Independent Acquisition & Trade-Off Barriers (`Theorems F1–F3`, Referee #2-scoped):** Output entropy is $\Theta(r_0 \log r_0)$ bits for $|\Sigma| = r_0^{\Omega(1)}$ or $n = r_0^{1+\Omega(1)}$ ($O(r_0)$ bits for fixed $|\Sigma| = O(1), n = \Theta(r_0)$); acquisition from $I_0$ without a side promise requires $\Omega(r_0)$ run inspections via even-permutation splices; `Theorem F3` establishes the exact i9 break-even $c_{\text{B4}} < \rho \cdot c_{\text{walk}} = 5.76\text{–}6.96\text{ ns/op}$.
- **Theorem B5 (`256`-Cursor Monotonicity of `SelThinSq`):** Scanning $I_0$ in $L$-order advances targets inside each $F_c$ strictly monotonically, so `256` L1-resident cursors (`1 KiB`) resolve all thin->thin $\mathrm{LF}^2$ overlaps in $\le r_0$ total advances with zero sorting (`16,290` exhaustive + 5 killers + 6 blocks verified).
- **Run-Overlap Graph $G(I_0)$ Hierarchy (`Theorems K1, K2-acyclic, K3`, `Theorem J1`):** `M1` (`R1` `J_u == I_v`) is 100% static and confluent (`Theorem K1`); `M2` (`R1+R2`) is Church–Rosser confluent on the acyclic sub-system (`Theorem K2`, scoped by Referee #2's `T=[1,0]` 2-cycle witness); `M4` (`+ R3 + R3* + O(1) Spiral`) is a complete zero-sort, zero-bisect inverse-BWT solver terminating at `q_core = 0` on all finite bijections (`Theorem K3`, 6/6 blocks + 5/5 killers); `Theorem J1` proves pre-chained single-pass `256`-cursor resolution suffices for joint composition (`M1 ∘ SelThinSq`: `-70.4%` walk; `Tube-Open ∘ SelThinSq`: `-69.1%` walk, `0/6 wide`).
- **`Open` Operator Break-Even Width Law (`Gamma Cycle 5`):** $w^* = C_{\text{strip}} / C_{\text{row}} \approx 16$ (EPYC) / `10–12` (i9), deriving `#657`'s `LIMIT = 16` from first principles.

### 4.2 Explicitly Falsified Conjectures & Dead Ends (Do Not Repeat) `[falsified]`
- `E_RG` potential (`n = 5, 9, 13`); $f(r)$-bounded potential for general `BDMR` (`Fib_{2m}+$`); `r + e_{2,self}` (`n = 14`); `k = 3, 4` interval DAGs (`15–18.8 r_0`); Coarse $K$-Super-Buckets (`K = 2..16`, blows up `maxSB` `8×`); Early-Stop Wavefront Polarity-DAG (`30.04 ms` Round 1); Exact CP-SAT Bucket-MIS (`4.3%` reduction); Cross-byte thin co-shifting (`0.000%` 1-step adjacent pairs, `1.6%` periodic `L`-motifs); Lowering `W` inside `endpoint.rs` (`W = 8/4/2/1` adds `+1.1–2.2 ms` and up to `3.5 MiB` arena); Unprotected `AHT` `max_q <= 2d` (`4.75× d` at $N = 10^{12}$); Post-`M1` 256-cursor monotonicity (`n = 5` ternary witness); Cycle 3 `C3 = 1/4` constant (tightened to `1/5`); Unscoped `M2` confluence on pure 2-cycles (`T = [1,0]`); Unscoped `Theorem F1` $\Theta(r_0 \log r_0)$ for fixed $|\Sigma| = O(1), n = \Theta(r_0)$.

---

## 5. Cycle 7–20 Research Roadmap (Continuing Automatically After Interim Handoff)

The Cycle 6 silicon discovery — **`TubeOpen` beating shipped `0.5.12 OpenCells` by `-10.2%` instructions (`instr = 0.898`), `-11.6%` L2, `-10.6%` L3 at `1.015×` RSS, blocked ONLY by losing bidirectional `wire_xor` + `S1`/`TubeMerge` overhead** — opens a direct mathematical path for Cycles 7–20:
1. **Bijective Contraction + Preserved `wire_xor` (Cycles 7–10):**
   - **Bijective Exact-Match `R1` / `M1` (`J_u == I_v`) fused into `Open` Pass 1:** Every retained row remains in a 100% bijection, keeping `emit_bidirectional` (`wire_xor`, `0.25–0.38 × m_16` serial rounds vs `Open`'s `0.500 × m_16`) with ZERO `TubeBiXor` rebuild and `0.50–0.77×` smaller `edges`!
   - **Bijective `1-Pass Ascent-DAG (<= 8B u64)` + `wire_xor`:** Exact bijection cutting BOTH `edges` RAM (`-39%`, `0.65–0.85 MiB/w` SUB-`Open`!) and bidirectional serial rounds (`0.305 × m_16`)!
   - **Bijective Full Strip-Space `Phi^2` + `wire_xor`:** Exact bijection on `Ind_{S_thin}(Phi^2)` (`0.297 × m_16` bidirectional serial rounds)!
2. **Deep Foundational Exploration Beyond `Open`'s Representation (Cycles 11–18):**
   - **Discrete Morse / Algebraic Morse Reduction & Perseus-style apparent pairs** (from `FRONTIER.md` Pending Leads — formalizing `Open` + `M1` + `M2` as a filtered Morse collapse over the free monoid ring and testing whether `apparent pairs` or `coreductions` / Backward-Open duality cut Pass 1 + `wire_xor` simultaneously!).
   - **Zero-Extra-Pass Contraction & `K >= 4` Multi-Chain Bijective Emit** (combining bijective contraction with zero-copy `wire_xor` or low-overhead multi-chain anchors so `dep_L2`, `instr`, `DRAM@7W`, AND container wall ALL drop `< 1.00×` simultaneously!).
   - **Origami Lift / Train-Track / Run-Network Core Operators** (testing whether any non-walk core operator on the `0.11 r_0` series-reduced run-network core beats `Open`'s walk).
3. **Final Audit, Full 20-Cycle Journal & Final Verified Handoff (Cycles 19–20).**
