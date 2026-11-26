# Permutation patterns: exact structural census, consumer comparison, and scoped verdict

Claim [`permutation-patterns`](../../../../frontier/claims/permutation-patterns.toml) — smooth-zone application branch of [`ideal-permutation-structures`](../ideal-permutation-structures/results.md), following the source-checked [`literature.md`](literature.md) and the frozen [`preregistration.md`](preregistration.md) (committed before any census or timing data). Preserves [`chatgpt-response.md`](chatgpt-response.md) verbatim. Zero edits to `src/`, zero `unsafe` (`#![forbid(unsafe_code)]`), zero workstation spending, zero alteration of prior frozen gates (C2).

**Harness:** [`examples/permutation_patterns_probe.rs`](../../../../examples/permutation_patterns_probe.rs).  
**Corpus:** `corpora/wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 blocks: 233 production Open-admitted blocks + 198 non-admitted blocks).  
**Distinct-block split (identical to `ideal-permutation-structures`, frozen before data):** Train = first `117` admitted blocks (`105,298,817` rows); Held-out = last `116` admitted blocks (`104,398,845` rows).  
**Hardware:** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell, AVX2, hardware `popcnt`/`lzcnt` under `-C target-cpu=native`, 32 KiB L1d, 256 KiB L2, 45 MiB L3, no vPMU), pinned to CPU 0 (`taskset -c 0`), single-threaded, reusable `out` buffer, 7 interleaved repeats under both **warm** (in-situ) and **cold** (64 MiB L3 eviction flush) conditions, compiled for both **baseline x86-64** and **`-C target-cpu=native`**.

---

## Executive summary & verdict (strict C2 evaluation)

**Verdict: SCOPED FAMILY STOP — do NOT admit a constructor claim for permutation-pattern-compressed encodings of Open's unchanged `LIMIT=16` retained map $T$.**

1. **Two literature families are structurally excluded by exact proof & witnesses on `117/117` training blocks:**
   - **Family C — General Substitution-Decomposition / Separable Tree (`Brignall` strong intervals; `Chakraborty et al.` ISAAC 2024):** Exact $O(N \log N)$ strong-interval tree analysis (exhaustively verified on all `5,913` permutations of $S_1..S_7$) proves **0/117** blocks are separable on raw LF or on Open's retained map $T$ (`117/117` have explicit verified 4-tuple witnesses of both `2413` and `3142`). Both raw LF (`SIT depth = 3` on all 117 blocks; `max_prime_arity / n = 0.06441 == nSRuns / n`) and Open's $T$ (`max_prime_arity / m = 0.5511` vs `nSRuns / m = 0.5517`; median `2` prime nodes; `corr(h1/m, max_prime/m) = +0.989`) are **completely flat** above their strict $+1$ runs: once strict $+1$ runs (`99.98%` label-homogeneous) are deflated, the remainder collapses into a **single giant irreducible simple (prime) core** of median arity `~54,200` (`55.1%` of $m$) with `0.0000` label-homogeneous prime nodes.
   - **Family D — Short-Pattern Avoidance / Matrix-Division (`Kozma–Opler` ESA 2026):** While raw LF obeys proved $\text{LDS}(\operatorname{LF}) \le \sigma$ (`117/117`, median `64` vs $\sigma = 87$), Open's first-return elimination **breaks $D_{\sigma+1}$-avoidance on `112 / 117` (`95.7%`) real Wikidata blocks** — median $\text{LDS}(T) = 102 > \sigma = 87$ ($\text{LDS}/\sigma = 1.172$, max `129`), confirming empirically that the `bbaaa` counterexample is the rule rather than an exception. Every tested 4-pattern (`2413`, `3142`, `4321`, `1234`, `2143`) has explicit verified witnesses on `117/117` blocks, and all six 3-patterns have large exact shares (`12.7%–26.5%`).
2. **Two monotone-run families qualified strongly on training blocks, shrank navigation storage by `22%–40%`, and STILL LOST to matched prebuilt controls in EVERY cache condition and build:**
   - **Family A — Labelled Strict $+1$ Run / $\oplus$-Interval Rank-Indexed Map (`P1_strict_run`, Barbay–Navarro `nSRuns` + Brignall $\oplus$ intervals):** Shrinks `nav_bytes` by **−39.7%** (`455.5 → 274.7 KiB/blk`, total `rep` `0.65×`), yet loses to matched `ctrl_shipped_fwd` by **`2.322×` warm / `1.556×` cold** (baseline) and **`1.602×` warm / `1.177×` cold** (native hardware `popcnt`/`lzcnt`), and loses to matched `ctrl_shipped_fwd_4lane` by **`1.651×` warm / `1.362×` cold** (baseline) and **`1.249×` warm / `1.084×` cold** (native). Materialisation also adds `+74..+93 ms` over `mat_shipped_fwd`.
   - **Family B — Labelled Ascending-Run Monotone-Block Map (`P2_mono_B16`, Barbay–Navarro `nRuns` for $O(1)$ cache-line access):** Exploits the proved band-monotonicity theorem (`nLRuns(T)` median **`1,628` = `0.01581 m`**, mean labelled ascending run **`63` rows**, `33×` fewer runs than $T^{-1} = \text{LF}_R$), achieving `84.1%` pure single-label 16-row blocks and shrinking `nav_bytes` by **−21.6%** (`455.5 → 357.0 KiB/blk`, `rep` `0.81×`). Yet it loses to matched `ctrl_shipped_fwd` by **`1.435×` warm / `1.078×` cold** (baseline) and **`1.282×` warm / `1.022×` cold** (native), loses to matched `ctrl_shipped_fwd_4lane` by **`1.404×` warm / `1.135×` cold** (baseline) and **`1.402×` warm / `1.144×` cold** (native), and its direct `bands + queues` materialisation (`A6c = 33.8–34.5 ms`) is **`2.08–2.10×` slower** than `A6c_fwd` (`16.1–16.6 ms`).
   - Classic Barbay–Navarro (2013) Huffman-merge wavelet tree on held-out $T$ reaches `138.7 KiB/blk` permutation-only space (excluding codes/dictionary) at a prohibitive average merge-tree depth of **`9.48` dependent `select` ops per step**.
3. **Synthesis with `ideal-permutation-structures` (#992):** Compressing the *encoding* of Open's unchanged `13.53M`-visit permutation $T$ leaves visit count (`13.53M`) and literal fragmentation (`81.2%`) untouched while adding lookup latency/branches onto the pointer-chasing critical path — losing in every regime. By contrast, contracting the *labelled return map* itself (`F1a`/`F1b`/`F4a` in #992) simultaneously shrinks visits (`0.21–0.34×`), `nav_bytes` (`93.6–152.7 KiB/blk`, deep inside L2), and `lit%` (`81.2% → 6.5–8.3%`) through the unchanged 1-load `edges[j]` consumer, winning `2.8×–4.4×`.

---

## 1. Algorithmic self-verification & 100% byte-exactness

1. **Exact recognition self-check (`self_check`):**
   - `O(N log N)` strong-interval tree verified exhaustively against `O(N^3)` brute-force strong-interval enumeration on **5,913 / 5,913** permutations ($S_1..S_7$), reproducing exact large Schröder separable counts `[1, 2, 6, 22, 90, 394, 1806]` and matching $\operatorname{Av}(2413, 3142)$ on every permutation.
   - `O(N log N)` 6-pattern Fenwick-tree counter verified exhaustively against `O(N^3)` brute force on **873 / 873** permutations ($S_1..S_6$).
   - Exact `bbaaa` counterexample verified: raw LF $= [3, 4, 0, 1, 2]$ ($\text{LDS} = 2 \le \sigma = 2$, $\#321 = 0$), while first return to $\{0, 1, 2, 3\}$ gives $\text{LF}_R = [3, 2, 0, 1]$ and $\Psi_R = [2, 3, 1, 0]$, both with $\text{LDS} = 3$ and $\#321 = 2$.
2. **Consumer byte-exactness against `bwt::walk` (`exact`):**
   - **12 / 12** synthetic stress cases (dense, periodic `abac^256` / `aaab^512` / `ab^500`, arbitrary non-BWT multicycle `64` / `4k`, run-heavy `4k`, edge cases `n=1,2,const64`, `bbaaa`) × 14 arms = **168 / 168 exact** (including direct `bands+queues` materialisation of `MonoBlock` for $B \in \{16, 28, 32\}$).
   - **198 / 198** forced non-admitted `wikidata.bz2` blocks × 14 arms = **2,772 / 2,772 exact**.
   - **233 / 233** production-admitted `wikidata.bz2` blocks × 14 arms = **3,262 / 3,262 exact**.
   - **Total: 6,202 / 6,202 consumer runs 100% byte-identical to `bwt::walk`.**

---

## 2. Part 1 — Exact structural census on 117 training blocks (raw LF vs. Open $T$)

### 2.1 Block covariates (`117` train blocks, `105,298,817` rows)

| covariate | min | p10 | median (p50) | p90 | max |
|---|---:|---:|---:|---:|---:|
| $n$ | 899,809 | 899,973 | 900,000 | 900,000 | 900,000 |
| $m$ (retained rows) | 65,112 | 86,381 | 98,905 | 170,939 | 231,600 |
| $m / n$ | 0.0723 | 0.0960 | 0.1099 | 0.1899 | 0.2573 |
| $\sigma$ (alphabet size) | 83 | 86 | 87 | 91 | 102 |
| maximal runs $r$ | 47,958 | 52,097 | 57,968 | 96,178 | 136,178 |
| $h_1 / m$ | 0.3015 | 0.3546 | 0.4118 | 0.5100 | 0.6590 |
| $\text{strips} / m$ | 0.0099 | 0.0119 | 0.0352 | 0.0399 | 0.0532 |
| Open `bands` | 1,314 | 2,611 | 3,548 | 4,226 | 5,429 |

### 2.2 Raw LF ($\pi \in S_n$) vs. Open's retained map ($T = \Psi_R \in S_m$ and $T^{-1} = \text{LF}_R$)

| exact structural metric | Raw LF ($\pi \in S_n$) p50 `[min..max]` | Open $T = \Psi_R \in S_m$ p50 `[min..max]` | structural implication |
|---|---|---|---|
| **Contiguous ascending runs** `nRuns / N` | `0.03225` `[0.02677..0.07570]` | **`0.00835`** `[0.00401..0.01026]` (median `857` runs) | $T = \Psi_R$ has **120-row** mean ascending runs (`33×` fewer than $T^{-1} = \text{LF}_R$ at `0.27582`) |
| **Labelled ascending runs** `nLRuns / m` | n/a | **`0.01581`** `[0.00749..0.01991]` (median `1,628`) | Guaranteed $\le$ `bands/m` (`0.0359`); mean labelled ascending run = **63 rows**; `H(vLRuns) = 8.344` bits |
| **Strict $+1$ runs** `nSRuns / N` | `0.06441` `[0.05329..0.15132]` | **`0.5517`** `[0.4442..0.7637]` (median `54,253`) | Eliminating thick runs leaves ~54k strict $+1$ runs (`nLSRuns = 54,255`: **99.996% label-homogeneous**) |
| **Strict $\pm 1$ runs** `nSMRuns / m` | — | `0.5517` `[0.4442..0.7637]` | Almost zero strict $-1$ runs (`linear_dec_nodes` median `0`, max `4`) |
| **Exact `LIS`** | `171,916` `[158,137..180,538]` | `7,886` `[6,462..19,061]` | Long ascending subsequences (`1234` witnessed `117/117`) |
| **Exact `LDS` (= $\min \text{nSUS}$)** | `64` `[51..74]` (`LDS/σ = 0.725`, `0/117 > σ`) | **`102`** `[69..129]` (`LDS/σ = 1.172`, **`112/117 > σ`**) | **95.7% of real blocks violate $D_{\sigma+1}$-avoidance** after first-return elimination |
| **Exact `max_prime_arity / N`** | `0.06441` `[0.05329..0.15132]` | **`0.5511`** `[0.4439..0.7637]` | Equals `nSRuns / N` to 3–5 digits! Flat single giant prime core after strict $+1$ deflation |
| **Exact `SIT depth` & `prime_nodes`** | depth `3` `[3..3]` | depth `5` `[3..5]`, `prime_nodes = 2` `[1..4]` | Zero hierarchical substitution depth (`homog_prime_nodes = 0.0000`) |
| **Leaves in nontriv linear / N** | `0.9513` `[0.8898..0.9619]` | `0.5899` `[0.3419..0.7002]` (`99.98%` label-homog) | Linear intervals are almost purely single-code thin runs (`corr(h1/m, max_prime/m) = +0.989`) |
| **Exact separability** (`prime == 0`) | **`0 / 117`** | **`0 / 117`** | Verified witnesses of `2413`, `3142`, `4321`, `1234`, `2143` on **`117 / 117`** blocks |

**Exact sample witnesses on `train[0]` (`n = 899,931`, `m = 89,874`, $\sigma = 89$, `bands = 3,204`):**
- `2413`: indices `[0, 11, 16, 22]` $\to$ values `[4482, 12279, 53, 10961]`
- `3142`: indices `[0, 16, 22, 94]` $\to$ values `[4482, 53, 10961, 993]`
- `2143`: indices `[0, 16, 23, 91]` $\to$ values `[4482, 53, 17664, 15900]`
- `4321` (prefix of exact `LDS = 103 > sigma = 89` witness): indices `[444, 604, 683, 1045]` $\to$ values `[88715, 88498, 88328, 85367]`

### 2.3 Exact 3-pattern profile, local $k$-gram shapes, and covariate correlations (`117` train blocks)

- **Exact global 3-pattern shares of $T$** (vs uniform $1/6 = 0.16667$):
  - `123`: **`0.26489`** `[0.24982..0.29572]` (strongly enriched by ascending bands)
  - `213`: **`0.19474`** `[0.17365..0.21204]`
  - `132`: **`0.14236`** `[0.13230..0.17141]`
  - `231`: **`0.13673`** `[0.12510..0.14737]`
  - `312`: **`0.13370`** `[0.11399..0.14331]`
  - `321`: **`0.12728`** `[0.10621..0.14099]` (abundant, never avoided!)
- **Local contiguous $k$-gram relative-order shapes of $T$:**
  - Contiguous 3-gram `123` share: **`98.35%`** `[97.98%..99.20%]` (vs `16.67%` uniform), of which **`98.68%`** are label-homogeneous!
  - Contiguous 4-gram `1234` share: **`97.56%`** `[97.04%..98.81%]` (vs `4.17%` uniform; top 4-gram shape on 117/117 blocks), of which **`98.20%`** are label-homogeneous!
- **Pearson correlations with existing Wikidata block covariates:**
  - `corr(h1/m, nLSRuns/m) = +0.989` and `corr(h1/m, max_prime/m) = +0.989` — confirming that strict $+1$ run density and prime core size are almost completely determined by the width-1 thin-run share $h_1/m$.
  - `corr(strips/m, nLRuns/m) = +0.743`, `corr(m/n, nLRuns/m) = -0.532`, `corr(sigma, LDS(T)) = +0.582`, `corr(m, LDS(T)) = +0.599`.
- **Synthetic controls (strictly separate from train):** `synth_dense_4k` has `nLRuns/m = 0.006`, `nLSRuns/m = 0.955`, `maxPrime/m = 0.955`, `sep = false`; `synth_multicycle_4k` has `nLRuns/m = 0.004`, `nLSRuns/m = 0.875`, `maxPrime/m = 0.875`, `sep = false`; `synth_run_heavy_4k` has `nLRuns/m = 0.059`, `nLSRuns/m = 0.567`, `maxPrime/m = 0.564`, `sep = false`; trivial/periodic blocks collapsing to $m = 1$ have `maxPrime = 0, sep = true`.

### 2.4 Pre-registered train qualification decision

| candidate family | pre-registered train rule | measured train median | decision |
|---|---|---:|---|
| **Family A** (`P1_strict_run`, strict $+1$ runs) | `nLSRuns / m <= 0.75` | **`0.5517`** | **SELECTED (1/2)** |
| **Family B** (`P2_mono_block`, labelled ascending runs) | `nLRuns / m <= 0.15` | **`0.01581`** | **SELECTED (2/2)** |
| **Family C** (`P_subst_tree`, substitution tree / separable) | `sep = 117/117` OR `max_prime / m <= 0.50` | `sep = 0/117`, `0.5511` | **STRUCTURALLY EXCLUDED** |
| **Family D** (`P_pattern_avoid`, Kozma–Opler avoidance) | some $|\pi| \le 4$ avoided OR `LDS <= 8` | `0` avoided, `LDS = 102` | **STRUCTURALLY EXCLUDED** |

---

## 3. Part 2 — Train parameter sweep, held-out representation ledger, and consumer timings

### 3.1 Train sweep for Family B block size (`117` train blocks, 3 interleaved warm repeats)

Both baseline x86-64 and `-C target-cpu=native` select **`B = 16`** (`rep = 44.94 MiB <= 56.33 MiB`, fastest warm consumer, highest pure-block share `85.4%`):

| arm | `rep` MiB | `nav` KiB/b | pure% | baseline warm ms (ratio) | native warm ms (ratio) |
|---|---:|---:|---:|---:|---:|
| `train_ctrl_fwd` | 56.33 | 435.5 | — | 105.47 (`1.000×`) | 112.06 (`1.000×`) |
| `train_P_strict_run` | 38.00 | 275.1 | — | 237.22 (`2.249×`) | 175.92 (`1.570×`) |
| **`train_P_mono_B16`** *(frozen)* | **44.94** | **335.8** | **85.4%** | **142.89 (`1.355×`)** | **144.18 (`1.287×`)** |
| `train_P_mono_B28` (64B line) | 45.41 | 339.9 | 79.1% | 167.38 (`1.587×`) | 170.01 (`1.517×`) |
| `train_P_mono_B32` | 45.72 | 342.6 | 77.6% | 157.96 (`1.498×`) | 154.27 (`1.377×`) |

### 3.2 Held-out representation, working-set & materialisation ledger (`116` blocks, `104,398,845` bytes)

All arms traverse the **exact same `13,525,480` retained visits**, the **exact same `62.5 KiB/blk` dictionary** (`inline16 + lens + spelling`), identical `lit% = 81.2%`, identical write amplification `2.75×`, and `0 B` consumer scratch beyond `out` (`Lane4` adds 48 B/blk).

| arm | `nav` KiB/b | `dict` KiB/b | `rep` KiB/b (`vs fwd`) | dep loads / visit | coord conversion per visit | offline build ms (base / nat) |
|---|---:|---:|---:|---:|---|---:|
| `ctrl_shipped_bidi` | 455.5 | 62.5 | 518.0 (`1.00×`) | 1.00 (MLP=2) | `xor` + shift | 256.2 / 250.6 |
| `ctrl_shipped_fwd` | 455.5 | 62.5 | 518.0 (`1.00×`) | 1.00 (MLP=1) | mask + shift | 256.2 / 250.6 |
| `ctrl_shipped_fwd_4lane` | 455.5 | 62.5 | 518.0 (`1.00×`) | 1.00 (MLP=4) | mask + shift | 256.2 / 250.6 |
| `P1_strict_run_fwd` | **274.7** | 62.5 | **337.3 (`0.65×`)** | **2.00** (MLP=1) | `dir[j>>5]` + mask + `lzcnt` + `popcnt` + `head_edges[r]` + add (`heads/m = 0.541`) | 117.8 / 131.0 |
| `P1_strict_run_4lane` | **274.7** | 62.5 | **337.3 (`0.65×`)** | **2.00** (MLP=4) | same × 4 lanes | 117.8 / 131.0 |
| `P2_mono_B16_fwd` | **357.0** | 62.5 | **419.5 (`0.81×`)** | **1.16** (MLP=1) | `blocks[j>>4]` + branch (`84.1%` pure `base + offs[j&15]`, `15.9%` escape load) | 451.8 / 466.5 |
| `P2_mono_B16_4lane` | **357.0** | 62.5 | **419.5 (`0.81×`)** | **1.16** (MLP=4) | same × 4 lanes | 451.8 / 466.5 |
| *Barbay–Navarro (2013) wavelet ref* | *138.7 (perm only)* | *+codes+dict* | *—* | ***9.48*** | *Huffman-merge tree `select0`/`select1` chain* | *theoretical* |

**Actual materialisation cost on held-out `116` blocks (5 repeats, strictly separate from consumer):**

| materialisation arm | baseline x86-64 ms (`min..max`) | `-C target-cpu=native` ms (`min..max`) | `A6c` vs bidi / fwd | projected i9 `C` (ms) |
|---|---:|---:|---:|---:|
| `mat_shipped_bidi` (`A6a+b+c_xor`) | 168.04 (`163.91..183.45`), `A6c=46.65` | 161.84 (`160.64..171.18`), `A6c=44.98` | `1.000×` / — | 577.7 (anchor) |
| `mat_shipped_fwd` (`A6a+b+c_fwd`) | 140.93 (`135.17..154.77`), `A6c=16.62` | 135.79 (`131.08..137.75`), `A6c=16.08` | `0.356–0.357×` / `1.000×` | **473.8–474.0** |
| `mat_strict_run` (`fwd + O(m)` scan) | 233.65 (`+92.72`, `88.36..100.39`) | 210.02 (`+74.23`, `72.12..78.07`) | — | **740.3–794.6** |
| `mat_mono_direct` (`B=16`, no `edges`) | 157.14 (`153.01..175.58`), `A6c=34.48` | 149.83 (`146.46..152.44`), `A6c=33.80` | `0.739–0.751×` / **`2.075–2.102×`** | **535.6–537.6** |

Neither pattern-derived representation has cheaper materialisation than `mat_shipped_fwd` (direct `MonoBlock` construction from `bands + queues` avoids writing flat `edges[0..m]` and beats `A6c_bidi` by `0.74–0.75×`, but is still `2.08–2.10×` slower than `A6c_fwd` due to per-block span checking, escape allocation, and `u16` subtraction).

### 3.3 Held-out consumer timings & allowable constructor budgets — Baseline x86-64 (7 interleaved repeats)

All i9 columns (`E_i9`, `dE_*`, `Cmax_*`, `dC_ext`, `8w_dE`) are transparent VM-to-i9 ratio projections anchored at `E_base = 490.3 ms`, `C_base = 577.7 ms`, `C_fwd_proj = 473.8 ms`, `W_1w = 1,306.0 ms`, `3% gate = 39.18 ms`, and `8w_dE = dE_fwd × (69.5 / 490.3)` (model only, not measured on i9).

#### Warm (in-situ per-block) — baseline x86-64

| arm | ms med (`min..max`) | ns/row | ns/vis | vs bidi | vs fwd | vs 4ln | `E_i9` (proj) | `ΔE` vs bidi | `ΔE` vs fwd | `C_max` vs base | `C_max` vs fwd | `ΔC_ext` | `8w ΔE` (model) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 120.21 (`118.11..123.93`) | 1.151 | 8.89 | 1.000× | 1.132× | 1.375× | 490.3 | +0.0 | −55.8 | 538.5 | 378.8 | −95.0 | −7.9 |
| `ctrl_shipped_fwd` | 106.52 (`104.75..108.28`) | 1.020 | 7.88 | 0.883× | 1.000× | 1.223× | 434.5 | +55.8 | +0.0 | 594.3 | 434.6 | −39.2 | +0.0 |
| `ctrl_shipped_fwd_4lane` | 87.41 (`86.58..91.36`) | 0.837 | 6.46 | 0.727× | 0.818× | 1.000× | 356.5 | +133.8 | +77.9 | 672.3 | 512.6 | +38.8 | +11.0 |
| `P1_strict_run_fwd` | 245.92 (`242.24..255.37`) | 2.356 | 18.18 | 2.052× | **2.322×** | 2.819× | 1003.1 | −512.8 | −568.6 | 25.8 | −134.0 | −607.8 | −80.6 |
| `P1_strict_run_4lane` | 143.77 (`142.06..153.45`) | 1.377 | 10.63 | 1.196× | 1.354× | **1.651×** | 586.4 | −96.1 | −151.9 | 442.4 | 282.7 | −191.1 | −21.5 |
| `P2_mono_B16_fwd` | 153.12 (`149.16..159.43`) | 1.467 | 11.32 | 1.261× | **1.435×** | 1.730× | 624.5 | −134.2 | −190.1 | 404.3 | 244.6 | −229.2 | −26.9 |
| `P2_mono_B16_4lane` | 123.08 (`118.56..128.20`) | 1.179 | 9.10 | 1.006× | 1.153× | **1.404×** | 502.0 | −11.7 | −67.6 | 526.8 | 367.1 | −106.7 | −9.6 |

#### Cold (64 MiB flush before each arm pass) — baseline x86-64

| arm | ms med (`min..max`) | ns/row | ns/vis | vs bidi | vs fwd | vs 4ln | `E_i9` (proj) | `ΔE` vs bidi | `ΔE` vs fwd | `C_max` vs base | `C_max` vs fwd | `ΔC_ext` | `8w ΔE` (model) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 176.47 (`172.44..178.44`) | 1.690 | 13.05 | 1.000× | 0.954× | 1.366× | 490.3 | +0.0 | +23.6 | 538.5 | 458.2 | −15.6 | +3.3 |
| `ctrl_shipped_fwd` | 184.95 (`182.19..190.02`) | 1.772 | 13.67 | 1.048× | 1.000× | 1.435× | 513.9 | −23.6 | +0.0 | 515.0 | 434.6 | −39.2 | +0.0 |
| `ctrl_shipped_fwd_4lane` | 128.49 (`127.31..130.10`) | 1.231 | 9.50 | 0.732× | 0.697× | 1.000× | 357.0 | +133.3 | +156.9 | 671.8 | 591.5 | +117.7 | +22.2 |
| `P1_strict_run_fwd` | 288.19 (`283.33..305.58`) | 2.761 | 21.31 | 1.647× | **1.556×** | 2.243× | 800.7 | −310.4 | −286.9 | 228.1 | 147.7 | −326.0 | −40.7 |
| `P1_strict_run_4lane` | 175.91 (`171.27..180.10`) | 1.685 | 13.01 | 1.007× | 0.945× | **1.362×** | 488.8 | +1.5 | +25.1 | 540.1 | 459.7 | −14.1 | +3.6 |
| `P2_mono_B16_fwd` | 198.57 (`195.90..204.10`) | 1.902 | 14.68 | 1.133× | **1.078×** | 1.547× | 551.7 | −61.4 | −37.8 | 477.1 | 396.8 | −77.0 | −5.4 |
| `P2_mono_B16_4lane` | 146.54 (`143.95..148.84`) | 1.404 | 10.83 | 0.836× | 0.789× | **1.135×** | 407.2 | +83.1 | +106.7 | 621.7 | 541.3 | +67.5 | +15.1 |

### 3.4 Held-out consumer timings & allowable constructor budgets — `-C target-cpu=native` (Haswell/AVX2, 7 interleaved repeats)

(`C_fwd_proj = 474.0 ms`; hardware `popcnt` + `lzcnt` active for `P1_strict_run`.)

#### Warm (in-situ per-block) — `-C target-cpu=native`

| arm | ms med (`min..max`) | ns/row | ns/vis | vs bidi | vs fwd | vs 4ln | `E_i9` (proj) | `ΔE` vs bidi | `ΔE` vs fwd | `C_max` vs base | `C_max` vs fwd | `ΔC_ext` | `8w ΔE` (model) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 114.00 (`111.24..116.64`) | 1.092 | 8.43 | 1.000× | 0.994× | 1.285× | 490.3 | +0.0 | +3.0 | 538.5 | 437.8 | −36.2 | +0.4 |
| `ctrl_shipped_fwd` | 114.70 (`109.58..118.53`) | 1.099 | 8.48 | 1.006× | 1.000× | 1.323× | 493.3 | −3.0 | +0.0 | 535.5 | 434.8 | −39.2 | +0.0 |
| `ctrl_shipped_fwd_4lane` | 86.60 (`84.00..112.48`) | 0.829 | 6.40 | 0.778× | 0.756× | 1.000× | 372.4 | +117.9 | +120.9 | 656.4 | 555.7 | +81.7 | +17.1 |
| `P1_strict_run_fwd` | 183.04 (`177.11..204.77`) | 1.753 | 13.53 | 1.626× | **1.602×** | 2.119× | 787.2 | −296.9 | −293.9 | 241.6 | 140.9 | −333.1 | −41.7 |
| `P1_strict_run_4lane` | 110.07 (`106.99..116.76`) | 1.054 | 8.14 | 0.977× | 0.959× | **1.249×** | 473.4 | +16.9 | +19.9 | 555.4 | 454.7 | −19.3 | +2.8 |
| `P2_mono_B16_fwd` | 148.19 (`147.02..151.30`) | 1.419 | 10.96 | 1.296× | **1.282×** | 1.716× | 637.4 | −147.1 | −144.1 | 391.5 | 290.7 | −183.2 | −20.4 |
| `P2_mono_B16_4lane` | 120.71 (`118.16..124.10`) | 1.156 | 8.92 | 1.054× | 1.050× | **1.402×** | 519.2 | −28.9 | −25.9 | 509.7 | 408.9 | −65.0 | −3.7 |

#### Cold (64 MiB flush before each arm pass) — `-C target-cpu=native`

| arm | ms med (`min..max`) | ns/row | ns/vis | vs bidi | vs fwd | vs 4ln | `E_i9` (proj) | `ΔE` vs bidi | `ΔE` vs fwd | `C_max` vs base | `C_max` vs fwd | `ΔC_ext` | `8w ΔE` (model) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 169.79 (`166.73..174.80`) | 1.626 | 12.55 | 1.000× | 0.886× | 1.332× | 490.3 | +0.0 | +62.8 | 538.5 | 497.6 | +23.6 | +8.9 |
| `ctrl_shipped_fwd` | 191.54 (`188.33..196.26`) | 1.835 | 14.16 | 1.128× | 1.000× | 1.519× | 553.1 | −62.8 | +0.0 | 475.7 | 434.8 | −39.2 | +0.0 |
| `ctrl_shipped_fwd_4lane` | 127.87 (`122.68..129.22`) | 1.225 | 9.45 | 0.751× | 0.658× | 1.000× | 369.2 | +121.1 | +183.9 | 659.6 | 618.7 | +144.7 | +26.1 |
| `P1_strict_run_fwd` | 226.96 (`221.00..228.34`) | 2.174 | 16.78 | 1.330× | **1.177×** | 1.779× | 655.4 | −165.1 | −102.3 | 373.4 | 332.5 | −141.5 | −14.5 |
| `P1_strict_run_4lane` | 137.10 (`133.88..140.75`) | 1.313 | 10.14 | 0.812× | 0.713× | **1.084×** | 395.9 | +94.4 | +157.2 | 632.9 | 592.0 | +118.0 | +22.3 |
| `P2_mono_B16_fwd` | 195.06 (`192.35..200.85`) | 1.868 | 14.42 | 1.162× | **1.022×** | 1.551× | 563.3 | −73.0 | −10.2 | 465.5 | 424.6 | −49.4 | −1.4 |
| `P2_mono_B16_4lane` | 143.10 (`142.17..162.99`) | 1.371 | 10.58 | 0.837× | 0.742× | **1.144×** | 413.2 | +77.1 | +139.9 | 615.6 | 574.7 | +100.7 | +19.8 |

*(Note: when comparing 4-lane candidates against `ctrl_shipped_fwd` above, the positive `dE_fwd` in cold mode is 100% attributable to 4-lane MLP — against the **matched** 4-lane control `ctrl_shipped_fwd_4lane`, BOTH `P1_strict_run_4lane` (`1.084–1.651×`) and `P2_mono_B16_4lane` (`1.135–1.404×`) lose in every single cell!)*

---

## 4. Physical interpretation, scoped exclusion, and link to `ideal-permutation-structures`

1. **Why compressing Open's unchanged `T` loses even when strong monotone structure exists:**
   - Open's `edges[j]` already packs **both** `next` and `code` into **one 32-bit word** (`4 B/vertex`), decoded in 2 ALU ops (`&`, `>>`) with 1 dependent load per visit.
   - Even when `T = Psi_R` has massive ascending-run compressibility (`1,628` labelled ascending runs across `100k` vertices; `98.35%` local ascending 3-grams) or strict $+1$ runs (`45%` of entries absorbed), any compressed encoding of `T` on the **same `13.53M` vertices** must add either a second dependent memory access (`P1_strict_run`: `dir` → `head_edges`, `2.00` dep loads/visit) or a branch + escape lookup (`P2_mono_B16`: `15.9%` escape, `1.16` dep loads/visit) or a multi-level rank/select chain (Barbay–Navarro wavelet tree: `9.48` dep ops/visit; Kozma–Opler 2-level matrix division). Because visit count (`13.53M`) and literal spelling (`81.2%` 1-byte words) are untouched, the cache savings from shrinking `455.5 → 274.7–357.0 KiB/blk` never repay the per-visit lookup penalty.
   - Hardware `popcnt` + `lzcnt` (`-C target-cpu=native`) cuts `P1_strict_run_fwd` warm time by 25.6% (`245.9 → 183.0 ms`), showing the emulation tax on baseline x86-64, yet even with 1-cycle hardware instructions and 4-lane MLP in cold mode, `P1_strict_run_4lane` still trails matched `ctrl_shipped_fwd_4lane` (`137.1 vs 127.9 ms`, `1.084×`).
2. **Exact scope of this STOP (NOT a universal no-go):**
   - **Excluded:** Uncontracted permutation-pattern encodings of Open's `LIMIT=16` retained map $T$ on `corpora/wikidata.bz2` under the 39.18 ms (3%) combined budget and matched `forward-emission` (1-lane and 4-lane) controls — specifically: (i) separable / general strong-interval trees (structurally excluded: `0/117` separable, `55.1%` giant prime core), (ii) short-pattern-avoidance matrix divisions (structurally excluded: `0` short patterns avoided, `LDS(T) > sigma` on `112/117` blocks), (iii) strict $+1$ run rank-indexed tables (`P1_strict_run`), and (iv) fixed-width monotone-block delta tables (`P2_mono_B16/28/32`) and Barbay–Navarro wavelet-tree run encodings.
   - **Not excluded:** Labelled return-map **contraction** (`bwt-digram-return-map`, opened by [`ideal-permutation-structures`](../ideal-permutation-structures/results.md)), which changes the retained vertex set $R' \subset R$ to cut both visits (`0.21–0.34×`) and `lit%` (`81.2% → 6.5–8.3%`), or alternative BWT/bucket-space constructions that reduce total visits rather than re-encoding the same `13.53M` visits.
3. **Caveats:**
   - All timings were measured on a 2-vCPU Haswell Xeon VM (`256 KiB` L2, `45 MiB` L3, no vPMU); i9 millisecond projections are transparent ratio projections, and 8w scan-ideal numbers are model projections only.
   - One primary corpus (`corpora/wikidata.bz2`, 233 admitted blocks + 198 non-admitted controls) plus 12 synthetic stress cases.
   - No prior frozen STOP/KILL verdicts were altered.
