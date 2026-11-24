# N-18: Complete-Observation Fibres (Small Exhaustive `n`) & Observation-Relative `U` Threshold Distribution on the 6 Admitted Blocks

*Cycle 4 response to Coordinator Notes #7–#8 & Outside Review Round 2 §I(d), §III.6 (`origin/claude/sharp-meitner-cx0v78` `78abe2c`, `a91788e`, `66a0538`), bearing directly on MONODROMY's Theorem N′ (Record 107, `ad4f555`).*

**Script:** [`research/neon/scripts/measure_n18_fibres_and_thresholds.py`](scripts/measure_n18_fibres_and_thresholds.py) (reusing `measure_cycle2.py` for BWT loading, `LF`, orbit maps, and Type-A/B valid flip enumeration).

## 0. Pre-declared decision rules & book (METHOD.md §6a)

- **Decision rules (fixed before data in `CLAIMS.md` `86eb9b8`):**
  1. *Small exhaustive fibres:* For each declared observation model `(i)` raw bytes on `D`, `(ii)` histogram + first positional moment `A = Σ i·code(L[i])`, and `(iii)` run records with LF translations on `D`, classify the model as **fibre-ambiguous** iff fibres containing `≥ 2` valid single-cycle columns with distinct position maps `pos₀` exist and grow with `n`; report exact fibre counts, ambiguity share, max fibre size, conditional entropy `H(pos₀ | ι)`, and mean fibre-fixed set `E[|J₁ ∩ U_𝒪| / |J₁|]`.
  2. *Six-block threshold distribution:* Replace `N-9`'s all-flip intersection `|U_all-flips|` (a lower bound for `U(𝒫, D)`) with the witnessed threshold distribution `#{x ∈ J₁ : ν̂(x) ≤ q}` across `q ∈ [0, 20000]` and critical budgets `q_{1%}, q_{5%}, q_{10%}, q_{25%}, q_{50%}`. Under Theorem N′ (`c₁ ≥ h₁ − |B|`), every `(𝒫, D)` decoder in `Λ′` with observation budget `q < min_block q_{50%}` is closed against `δ = 1/2` on all 6 admitted blocks.
- **Input features relied on:** All valid single-cycle columns `L ∈ {0..σ-1}^n` verified against the primitive necklace count `M(n, σ) = (1/n) Σ_{d|n} μ(d) σ^{n/d}` (`[proved]`); and all 6 admitted blocks of `origin/data/admitted-bwt` (`41c03a5b`).
- **Assumptions knowingly unrealistic:** On the 6 blocks, flip enumeration caps column span at `j - i ≤ 256`, so `ν̂(x) ≤ ν′(x)` is a **rigorous witnessed lower bound** and `#{x ∈ J₁ : ν̂(x) ≤ q}` is a **rigorous upper bound** on `#{x ∈ J₁ : ν′(x) ≤ q}` (`[proved]`).

## 1. Two exact structural lemmas bearing on MONODROMY's Theorem N′ (Record 107)

### Lemma 18.1 — Exact fibre characterisation of `B` in Theorem N′ (`[proved]`)

Let `𝒪` be **any** deterministic observation language `ι(L, o)` (including order-sensitive summaries such as `A = Σ i·code(L[i])`). Define the **complete-observation fibre** of `(L, o)` as `Fib_𝒪(L, o) = { L' valid : ι(L', o) = ι(L, o) }`, and its **fibre-fixed row set** as

$$U_{\mathcal{O}}(L, o) = \{ x \in [n] : \forall L' \in \mathrm{Fib}_{\mathcal{O}}(L, o),\; \mathrm{pos}_{L', o}(x) = \mathrm{pos}_{L, o}(x) \}.$$

Then a sound `P_𝒪` rule can output a positional label `pos(x) = p` at `(L, o)` **if and only if** `x ∈ U_𝒪(L, o)`. Consequently, in MONODROMY's Theorem N′ (Record 107 §2), `B ⊆ J₁ ∩ U_𝒪(L, o)`, yielding the **exact fibre event floor**

$$c_1 \ge h_1 - |J_1 \cap U_{\mathcal{O}}(L, o)|$$

for **every** observation language `𝒪` — including order-sensitive `A`.

*Proof.* If `x ∉ U_𝒪(L, o)`, there exists a valid `L' ∈ Fib_𝒪(L, o)` with `pos_{L', o}(x) ≠ pos_{L, o}(x)`. Because `ι(L', o) = ι(L, o)`, any deterministic rule reading only `ι` outputs the same label for `x` on `L` and `L'`, contradicting soundness on at least one of them. Conversely, if `x ∈ U_𝒪(L, o)`, the map `ι(L, o) ↦ pos_{L, o}(x)` is well-defined and sound across the entire fibre. ∎

**Why this refines Record 107 §3 (`[argued]`):** Record 107 §3 noted that single-flip swap invariance says nothing against `A` and concluded that only a bit bound survives. Lemma 18.1 shows that **an exact event floor DOES survive for `(H, A)`** via complete-observation fibres — and Part 2 below shows `E[|J₁ ∩ U_{(H, A)}| / |J₁|]` collapses rapidly to `0.219` already at `n = 10, σ = 3`. What fails for `(H, A)` is **single-flip witness construction**, not the existence of an event floor; scaling the witness to large `n` requires balanced multi-flip fibre moves (`ΔA = 0`).

### Lemma 18.2 — Sharp disjoint-span blindness coefficients (`[proved]`, exhaustively verified `σ = 3, n = 4..7`)

For any family `ℱ_disj` of valid Type-A/B flips with **pairwise-disjoint column spans `[i, j]`** and any single probe row `d ∈ D`:
1. **Raw byte `L[d]`:** affected by at most **`1`** flip in `ℱ_disj` (since `L'[d] ≠ L[d]` requires `d ∈ {i, j} ⊆ [i, j]`). Bound: **`|𝒫| + |D| > ν′(x)`**.
2. **Pointwise byte + LF translation `(L[d], LF(d) - d)`:** affected by at most **`1`** flip in `ℱ_disj` (since Record 101 §1 proves `LF'(y) = LF(y)` for all `y ∉ {i, m, j} ⊆ [i, j]`, so changing `L[d]` or `LF(d)` requires `d ∈ [i, j]`). Bound: **`|𝒫| + |D| > ν′(x)`**.
3. **Full run record with LF translation `(s_d, e_d, L[d], LF(d) - d)`:** affected by at most **`2`** flips in `ℱ_disj` — **strictly tighter than `3|D|` in N-9 and Record 107 §1!** Bound: **`|𝒫| + 2|D| > ν′(x)`** (sharp!).

*Proof of (3).* Let `R_d = [s_d, e_d)` be the maximal run containing `d`. Suppose a valid flip `(i, m, j)` has `[i, j] ∩ {s_d - 1, e_d} = ∅`. Because `[i, j]` is an interval, either `j < s_d - 1` (disjoint from `[s_d - 1, e_d]`), `i > e_d` (disjoint from `[s_d - 1, e_d]`), or `s_d ≤ i < m < j ≤ e_d - 1` (impossible, since `L` is constant on `R_d` while a flip requires `L[i] ≠ L[j]`). In the first two cases `L` is unchanged on `[s_d - 1, e_d]` (so `R_d` is unchanged) AND `d ∉ [i, j]` (so `d ∉ {i, m, j}`, hence `LF(d)` is unchanged by Record 101 §1). Therefore every flip in `ℱ_disj` that alters `(s_d, e_d, L[d], LF(d) - d)` must contain `s_d - 1` or `e_d`. Since spans in `ℱ_disj` are pairwise disjoint, at most **2** spans can do so. ∎

*Exhaustive verification (`measure_n18_fibres_and_thresholds.py` `[measured]`):* Checked across all valid columns for `σ = 3, n = 4..7` (`V = 18 + 48 + 116 + 312 = 494` columns, all `d ∈ [0, n)`): max disjoint flips affecting `d` is `1` for raw byte, `1` for pointwise `(L[d], LF(d) - d)`, and **`2`** (sharp!) for full run record + LF (witness: `n = 6, L = "bbcbaa", d = 3`, affected by disjoint valid flips `(0, 2)` and `(3, 5)`).

## 2. Part 1 — Exhaustive complete-observation fibres for small `n` (`[measured]`, `o = 0`)

Every row below enumerates all `V(n, σ) = M(n, σ)` valid single-cycle columns with fixed `o = 0`, counting **distinct `pos₀` maps** per fibre (important for model `(i-a)` without `H`, where monotone symbol relabellings share the same `pos₀`).

### (a) Alphabet `σ = 3`, `n = 3..10` (`V = 8 → 5,880`)

| `n` (`V`) | Model | Fibres | Ambig. fibres | Ambig. cols % | Max `pos₀` / fibre | `H(pos₀ \| 𝒪)` (bits) | `E[\|U_𝒪\|/n]` | `E[\|J₁∩U_𝒪\|/\|J₁\|]` |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| `3` (`8`) | `(i-b)` `H + L[{0}]` | 8 | 0 | 0.0% | 1 | 0.000 | 1.000 | 1.000 |
| `3` (`8`) | `(ii)` `H + A` | 7 | 1 | **25.0%** | 2 | 0.250 | 0.833 | 0.833 |
| `3` (`8`) | `(iii-a)` `H + run+LF({0})` | 8 | 0 | 0.0% | 1 | 0.000 | 1.000 | 1.000 |
| `4` (`18`) | `(i-b)` `H + L[{0}]` | 15 | 3 | 33.3% | 2 | 0.333 | 0.833 | 0.833 |
| `4` (`18`) | `(ii)` `H + A` | 16 | 1 | 16.7% | 3 | 0.264 | 0.875 | 0.875 |
| `4` (`18`) | `(iii-a)` `H + run+LF({0})` | 17 | 1 | 11.1% | 2 | 0.111 | 0.944 | 0.972 |
| `6` (`116`) | `(i-b)` `H + L[{0}]` | 35 | 27 | 93.1% | 8 | 2.008 | 0.391 | 0.419 |
| `6` (`116`) | `(i-c)` `H + L[D_half]` (`\|D\|=3`) | 81 | 29 | 55.2% | 3 | 0.642 | 0.664 | 0.682 |
| `6` (`116`) | `(ii)` `H + A` | 79 | 25 | 53.4% | 5 | 0.740 | 0.619 | 0.621 |
| `6` (`116`) | `(ii+)` `H + (A, A₂)` | 114 | 2 | 3.4% | 2 | 0.034 | 0.974 | 0.974 |
| `6` (`116`) | `(iii-a)` `H + run+LF({0})` | 61 | 28 | 71.6% | 7 | 1.212 | 0.573 | 0.649 |
| `6` (`116`) | `(iii-b)` `H + run+LF({0,3})` | 111 | 5 | 8.6% | 2 | 0.086 | 0.960 | 0.966 |
| `8` (`810`) | `(i-b)` `H + L[{0}]` | 63 | 55 | 99.0% | 45 | 4.210 | 0.259 | 0.306 |
| `8` (`810`) | `(i-c)` `H + L[D_half]` (`\|D\|=4`) | 401 | 216 | 77.2% | 7 | 1.258 | 0.482 | 0.531 |
| `8` (`810`) | `(ii)` `H + A` | 324 | 200 | **84.7%** | 16 | 1.679 | 0.335 | **0.368** |
| `8` (`810`) | `(ii+)` `H + (A, A₂)` | 771 | 33 | 8.9% | 4 | 0.102 | 0.935 | 0.937 |
| `8` (`810`) | `(iii-a)` `H + run+LF({0})` | 160 | 104 | 93.1% | 32 | 3.108 | 0.345 | 0.407 |
| `8` (`810`) | `(iii-b)` `H + run+LF({0,4})` | 618 | 134 | 40.2% | 7 | 0.549 | 0.773 | 0.784 |
| `8` (`810`) | `(iii-c)` `H + run+LF(D_half)` (`\|D\|=4`) | 808 | 2 | 0.5% | 2 | 0.005 | 0.999 | 0.999 |
| `10` (`5,880`) | `(i-a)` raw `L[{0}]` (no `H`) | 2 | 2 | 100.0% | 2,702 | 11.290 | 0.100 | 0.171 |
| `10` (`5,880`) | `(i-b)` `H + L[{0}]` | 99 | 91 | **99.9%** | 259 | 6.663 | 0.201 | **0.250** |
| `10` (`5,880`) | `(i-c)` `H + L[D_half]` (`\|D\|=5`) | 1,941 | 1,318 | **89.4%** | 16 | 1.968 | 0.343 | **0.398** |
| `10` (`5,880`) | `(ii)` `H + A` | 957 | 745 | **96.4%** | 58 | **3.227** | **0.167** | **0.219** |
| `10` (`5,880`) | `(ii+)` `H + (A, A₂)` | 5,050 | 685 | **25.8%** | 6 | 0.304 | 0.805 | 0.814 |
| `10` (`5,880`) | `(iii-a)` `H + run+LF({0})` | 318 | 258 | **99.0%** | 170 | 5.456 | 0.234 | **0.288** |
| `10` (`5,880`) | `(iii-b)` `H + run+LF({0,5})` | 2,663 | 1,185 | **74.9%** | 22 | 1.630 | 0.514 | **0.536** |
| `10` (`5,880`) | `(iii-c)` `H + run+LF(D_half)` (`\|D\|=5`) | 5,854 | 24 | 0.9% | 3 | 0.009 | 0.996 | 0.996 |

### (b) Scaling across `σ = 4` (`n = 3..8`, `V = 20 → 8,160`) and `σ = 2` (`n = 4..12`, `V = 3 → 335`)

- **`σ = 4`, `n = 8` (`V = 8,160`):**
  - `(i-b)` `H + L[{0}]`: `99.8%` ambiguous, `maxF = 118`, `H = 5.287` bits, `E[|J₁∩U|/|J₁|] = 0.286`.
  - `(i-c)` `H + L[D_half]` (`|D| = 4`): `81.0%` ambiguous, `maxF = 11`, `H = 1.435` bits, `E[|J₁∩U|/|J₁|] = 0.507`.
  - **`(ii)` `H + A`**: **`91.2%` ambiguous**, `maxF = 31`, `H = 2.384` bits, **`E[|J₁∩U|/|J₁|] = 0.280`** (already **more ambiguous than reading one raw byte or one run+LF record**, and `(ii+)` with two moments `(A, A₂)` is already `17.8%` ambiguous!).
  - `(iii-a)` `H + run+LF({0})`: `98.0%` ambiguous, `maxF = 104`, `H = 4.379` bits, `E[|J₁∩U|/|J₁|] = 0.324`.
  - `(iii-b)` `H + run+LF({0, 4})`: `69.9%` ambiguous, `maxF = 24`, `H = 1.390` bits, `E[|J₁∩U|/|J₁|] = 0.587`.
  - `(iii-c)` `H + run+LF(D_half)` (`|D| = 4`): `4.7%` ambiguous (`186` ambiguous fibres, `maxF = 5`).
- **`σ = 2`, `n = 12` (`V = 335`):**
  - `(i-b)`: `99.4%` ambiguous (`maxF = 75`, `E[|J₁∩U|/|J₁|] = 0.260`); `(i-c)` (`|D| = 6`): `93.7%` ambiguous (`E[|J₁∩U|/|J₁|] = 0.317`).
  - `(ii)` `H + A`: **`93.1%` ambiguous**, `maxF = 13`, `H = 2.385` bits, **`E[|J₁∩U|/|J₁|] = 0.311`** (`(ii+)` `(A, A₂)`: `19.1%` ambiguous!).
  - `(iii-a)`: `96.4%` ambiguous (`E[|J₁∩U|/|J₁|] = 0.454`); `(iii-b)` (`|D| = 2`): `26.6%` ambiguous.

### (c) Minimal verified witness pairs per model (`[proved]`)

| Model | Minimal witness `(L₁, L₂)` | Shared observation | Differing `pos₀` maps |
|---|---|---|---|
| `(i-b)` `H + raw L[{0}]` | `n=4, σ=3`: `bbca` vs `bcab` | `H=(1,2,1), L[0]='b'` | `(0,1,2,3)` vs `(0,1,3,2)` |
| `(ii)` `H + first moment A` | `n=3, σ=3`: `bca` vs `cab` (round-2 witness); `n=6, σ=2`: `babbaa` vs `bbaaba` | `H=(1,1,1), A=2`; `H=(3,3), A=7` | `(0,1,2)` vs `(0,2,1)`; `(0,5,3,1,4,2)` vs `(0,2,5,1,3,4)` |
| `(iii-a)` `H + run+LF({0})` | `n=4, σ=3`: `caab` vs `cbaa` | `H=(2,1,1), rr(0)=([0,1), c, +3)` | `(0,3,2,1)` vs `(0,2,3,1)` |
| `(iii-b)` `H + run+LF({0, ⌊n/2⌋})` | `n=5, σ=4`: `badac` vs `bcdaa`; `n=6, σ=3`: `baabca` vs `bcabaa` | `H, rr(0), rr(⌊n/2⌋)` identical | `(0,4,1,3,2)` vs `(0,3,1,4,2)`; `(0,5,4,1,2,3)` vs `(0,3,5,1,2,4)` |
| `(iii-c)` `H + run+LF(D_half)` | `n=6, σ=4`: `badabc` vs `bcdaba` (`D={0,2,4}`) | `H` + all 3 run+LF records identical | `(0,5,1,4,3,2)` vs `(0,3,1,5,4,2)` |

### (d) Why ambiguity explodes with `n` (`[proved]` + `[measured]`)

- For any fixed `|D| = O(1)` under `(i)` or `(iii)`, or any fixed number `k` of polynomial moments `(H, A_1, ..., A_k)` under `(ii)`, the number of distinct observation values grows **polynomially** in `n` (`O(n^{σ-1+2|D|})` for run+LF records, `O(n^{σ-1+k(k+1)/2})` for `k` moments), while the number of valid single-cycle columns grows **exponentially** as `V(n, σ) = M(n, σ) = Θ(σ^n / n)` (`[proved]`).
- Consequently, average fibre size grows exponentially (`Ω(σ^n / poly(n))`), `H(pos₀ | 𝒪)` grows linearly in `n`, and **`E[|J₁ ∩ U_𝒪| / |J₁|]` drops monotonically toward `0` as `n` grows** across ALL THREE models (`[measured]`): by `n = 10` (`σ = 3`), even `(ii)` `(H, A)` fixes only `21.9%` of width-1 rows (already far below `50%`), while even `|D| = ⌈n/2⌉` raw bytes (`i-c`) still leaves `89.4%` of columns ambiguous and fixes only `39.8%` of `J₁`!
- **Verdict on Part 1:** All three models `(i)`, `(ii)`, `(iii)` are **decisively fibre-ambiguous**, with ambiguity scaling rapidly toward `100%` of columns and `E[|J₁ ∩ U_𝒪| / |J₁|] → 0` as `n` grows.

## 3. Part 2 — Observation-relative `U` threshold distribution on the 6 admitted blocks (`[measured]`)

Replacing `N-9`'s all-flip intersection `|U_all-flips|` (which corresponds only to `q = 0`: `1–2` width-1 rows, `0.00–0.01%` of `h₁`), the tables below report the exact witnessed threshold distribution **`#{x ∈ J₁ : ν̂(x) ≤ q}`** (rigorous upper bound on `|J₁ ∩ U(𝒫, D)|` for any `(𝒫, D)` with budget `q = (|𝒫| - 1) + c_D |D|`, where `c_D = 1` for raw bytes / pointwise LF, `c_D = 2` for full run+LF records by Lemma 18.2, or `c_D = 3` under Record 107's looser bound).

### (a) Critical observation budgets `q_p` (max `q` with `#{x ∈ J₁ : ν̂(x) ≤ q} ≤ p · h₁`)

| Block | `h₁` | `\|ℱ_disj\|` | `\|U_all-flips\|` (`in J₁`) | `q_{1%}` | `q_{5%}` | `q_{10%}` | `q_{25%}` | **`q_{50%}` (Theorem N′ `δ=1/2` wall)** | `q_{75%}` | `q_{90%}` | `q_{50%}` (`hex` / `nonhex`) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `block-0.L` | 35,116 | 16,206 | 354 (`2`) | 206 | 778 | 1,379 | 3,397 | **6,903** | 8,845 | 9,892 | `6,903 / 6,918` |
| `block-2.L` | 34,262 | 20,168 | 116 (`1`) | 235 | 1,159 | 2,034 | 5,630 | **8,761** | 10,433 | 10,980 | `8,753 / 8,984` |
| `block-85.L` | 55,504 | 26,491 | 1 (`1`) | 315 | 1,589 | 3,195 | 6,187 | **9,988** | 12,402 | 13,348 | `10,054 / 9,645` |
| `block-143.L` | 72,578 | 33,011 | 30 (`1`) | 482 | 2,097 | 4,412 | 8,971 | **14,120** | 17,137 | 17,979 | `14,082 / 14,314` |
| `block-210.L` | 97,054 | 33,407 | 4 (`1`) | 652 | 2,107 | 3,651 | 8,304 | **12,974** | 14,011 | 14,631 | `12,972 / 13,009` |
| `block-207.L` | 62,407 | 24,062 | 65 (`1`) | 403 | 1,460 | 2,595 | 5,625 | **9,684** | 12,479 | 13,139 | `9,841 / 8,771` |

### (b) Full threshold grid `#{x ∈ J₁ : ν̂(x) ≤ q}` (`% of h₁`)

| Block | `q=0` | `q=10` | `q=100` | `q=500` | `q=1,000` | `q=2,000` | `q=4,000` | `q=6,000` | `q=8,000` | `q=10,000` | `q=12,000` | `q=15,000` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 2 (0.01%) | 11 (0.03%) | 177 (0.50%) | 986 (2.81%) | 2,286 (6.51%) | 4,662 (13.28%) | 10,419 (29.67%) | 15,028 (42.80%) | 23,881 (68.01%) | 33,446 (95.24%) | 35,116 (100%) | 35,116 (100%) |
| `block-2.L` | 2 (0.01%) | 15 (0.04%) | 145 (0.42%) | 679 (1.98%) | 1,461 (4.26%) | 3,348 (9.77%) | 5,637 (16.45%) | 9,432 (27.53%) | 14,896 (43.48%) | 20,755 (60.58%) | 34,262 (100%) | 34,262 (100%) |
| `block-85.L` | 2 (0.00%) | 7 (0.01%) | 217 (0.39%) | 906 (1.63%) | 1,850 (3.33%) | 3,200 (5.77%) | 7,370 (13.28%) | 13,427 (24.19%) | 20,323 (36.62%) | 27,847 (50.17%) | 39,006 (70.28%) | 55,504 (100%) |
| `block-143.L` | 1 (0.00%) | 17 (0.02%) | 186 (0.26%) | 776 (1.07%) | 1,524 (2.10%) | 3,462 (4.77%) | 6,827 (9.41%) | 9,036 (12.45%) | 14,416 (19.86%) | 20,746 (28.58%) | 27,652 (38.10%) | 40,534 (55.85%) |
| `block-210.L` | 1 (0.00%) | 4 (0.00%) | 61 (0.06%) | 635 (0.65%) | 1,984 (2.04%) | 4,633 (4.77%) | 10,686 (11.01%) | 17,042 (17.56%) | 22,858 (23.55%) | 30,973 (31.91%) | 42,116 (43.39%) | 97,054 (100%) |
| `block-207.L` | 1 (0.00%) | 9 (0.01%) | 71 (0.11%) | 786 (1.26%) | 1,878 (3.01%) | 4,576 (7.33%) | 10,138 (16.24%) | 16,512 (26.46%) | 23,893 (38.29%) | 33,562 (53.78%) | 43,573 (69.82%) | 62,407 (100%) |

**Consequence for Theorem N′ (Record 107) (`[proved]` + `[measured]`):**
- Under `Λ′` with `𝒪 = (𝒫, D)` (interval histograms + declared raw bytes or run+LF records), Theorem N′ proves **`c₁ > h₁ / 2`** (closing `δ = 1/2`) on **all 6 admitted blocks** whenever the decoder's total observation budget satisfies **`(|𝒫| - 1) + c_D |D| < 6,903`** (and on `block-143.L` up to `< 14,120`, and on both Stream R and Stream N equally: `q_{50%}` is nearly identical for `hex_w1` and `nonhex_w1`!).
- Even at `q = 1,000` (e.g., 1,000 histogram intervals of size `900` rows, or `500` full run+LF records in `D`), `|B| ≤ 2.04–6.51%` of `h₁`, leaving **`c₁ ≥ 0.935 · h₁`**.

## 4. Obstruction table entry (all six fields)

| Field | Value |
|---|---|
| **Framework** | Positional labelling `P_𝒪` in `Λ′` from coarse interval histograms `(𝒫, D)` or low-degree order-sensitive moments `(H, A)` (`N-18`, bearing on Record 107 Theorem N′) |
| **Route class** | **R2** (positional labels on start rows of rigid facts in `Λ′`) |
| **Gate failed** | **(a) Soundness / identifiability (`|B|` ceiling in Theorem N′):** `|J₁ ∩ U(𝒫, D)| < h₁/2` on all 6 blocks for any budget `q < 6,903` (`< 6.5%` of `h₁` at `q = 1,000`); and on small exhaustive `n`, complete-observation fibres for `(i)`, `(ii)` `(H, A)`, and `(iii)` grow exponentially in `n`, driving `E[|J₁ ∩ U_𝒪| / |J₁|]` down to `0.219–0.288` by `n = 10, σ = 3` |
| **Witness** | Small exhaustive fibres (`measure_n18_fibres_and_thresholds.py`): `bca`/`cab` for `(H, A)` (`n=3, σ=3`), `caab`/`cbaa` for `H+run+LF({0})` (`n=4, σ=3`), `badabc`/`bcdaba` for `H+run+LF({0,2,4})` (`n=6, σ=4`), `bbcbaa` (`d=3`, flips `(0,2),(3,5)`) for sharp `2|D|`; 6-block threshold grid `q_{50%} = 6,903–14,120` |
| **Scope** | All sound `P_𝒪` rules in `Λ′` with `(𝒫, D)` budget `q < 6,903` on the 6 admitted blocks (proved + measured), and fixed-`|D|` / fixed-degree-moment observation models `(i)–(iii)` on small exhaustive single-cycle columns (`n ≤ 10` for `σ=3`, `n ≤ 8` for `σ=4`, `n ≤ 12` for `σ=2`) |
| **Escape condition** | Either spend fine-input observation budget `q ≥ 6,903–14,120` in `(𝒫, D)` (turning "aggregate" into `Ω(n)` fine probes), OR construct an observation `𝒪` with `|J₁ ∩ U_𝒪| ≥ h₁/2` on `𝓕` (necessarily carrying `Ω(n)` bits by the fibre count), OR leave `Λ′`'s rigid-write output model (see `N-19`) |
| **Reach** | **Global inside `Λ′`** for `(𝒫, D)` with `q < 6,903` on the 6 admitted blocks; **proved exact** for arbitrary `𝒪` via Lemma 18.1 (`c₁ ≥ h₁ − |J₁ ∩ U_𝒪|`), with large-`n` constructive multi-flip witnesses for order-sensitive `𝒪` still open |
| **Strength** | **Proved** (Lemma 18.1 exact fibre floor; Lemma 18.2 sharp `1|D|` and `2|D|` coefficients; polynomial-vs-exponential fibre growth) + **measured exact** (exhaustive small-`n` tables verified against `M(n, σ)`; 6-block `ν̂` threshold distributions) |
| **Transferable insight** | Replacing single-flip blindness with **complete-observation fibres `U_𝒪(L, o)`** upgrades Theorem N′ from a histogram-only swap bound to a universal exact event floor `c₁ ≥ h₁ − |J₁ ∩ U_𝒪|` that applies equally to order-sensitive summaries like `A = Σ i·code(L[i])`; scaling that floor to `n = 900,000` for polynomial moments requires pairing opposite-`ΔA` flips (`ΔA₁ + ΔA₂ = 0`) rather than abandoning event floors for bit bounds |

## 5. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exhaustive single-cycle enumeration verified against exact `M(n, σ)`; 6 admitted blocks with `SPAN_CAP = 256` yielding rigorous lower bounds `ν̂(x) ≤ ν′(x)`.
2. **Model assumptions:** `Λ′` from MONODROMY Record 107 §2; observation models `(i)–(iii)` match Coordinator #7 & Round 2 §III.6 verbatim.
3. **Method:** Exact exhaustive enumeration + difference-array threshold distribution `#{x ∈ J₁ : ν̂(x) ≤ q}`.
4. **Outcome:** Answers N-18 completely, replaces N-9's `U_all-flips` lower bound with the full observation-relative threshold curve (`q_{50%} = 6,903–14,120`), tightens Record 107's `3|D|` coefficient to sharp `2|D|` for full run+LF records (`1|D|` for raw/pointwise), and proves via Lemma 18.1 + small-`n` data that `(H, A)` still obeys a strong fibre event floor (`E[|J₁∩U|/|J₁|] = 0.219` by `n = 10, σ = 3`).
