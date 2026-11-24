# N-21: Balanced Multi-Flip Fibre Witnesses & Witnessed `U_𝒪` Bounds for `𝒪 = (H, A, A₂)` at `n ≈ 900,000`

*Cycle 5 constructive resolution of MONODROMY `ASK N (2)` (Record 108, `8e9b962`) & Coordinator Note #12 (`c4847b8`): exhibit two valid columns in the same `𝒪 = (H, A, A₂)` fibre (`ΔH = 0, ΔA = 0, ΔA₂ = 0`) whose position maps differ on an admitted block, and report `U_𝒪` for that block.*

**Script:** [`research/neon/scripts/search_n21_balanced_multiflip.py`](scripts/search_n21_balanced_multiflip.py) (reusing `measure_cycle2.py`; combines an exact `O(k)` compressed-arc monodromy evaluator with **full `O(n)` independent verification** of `LF'`, single `n`-cycle closure, `(H, A, A₂)`, all 256 per-symbol vector moments `(H[c], A^{(c)}, A₂^{(c)})`, and `pos_{L', p}` on every reported witness and every covering witness used to bound `U_𝒪`).

---

## 0. Pre-declared decision rule & book (METHOD.md §6a)

- **Decision rule (fixed before data in `CLAIMS.md` `79fb7f0`):** Fix `𝒪 = (H, A, A₂)` where `H` is the 256-byte histogram, `A = Σ_{r=0}^{n-1} r · L[r]`, and `A₂ = Σ_{r=0}^{n-1} r² · L[r]`. Classify `𝒪` as **fibre-ambiguous at real `n`** iff at least one byte-verified valid single-cycle column `L' ≠ L` exists on `block-0.L` (`n = 899,931, p = 264,045`) with `(H(L'), A(L'), A₂(L')) = (H(L), A(L), A₂(L))` and `pos_{L', p} ≠ pos_{L, p}`; report minimal-`k` / minimal-span witnesses and both lower and upper bounds on `|U_𝒪|` and `|J₁ ∩ U_𝒪|`.
- **Input features relied on:** Exact raw last column `block-0.L` (`n = 899,931, p = 264,045, h₁ = 35,116`) as primary target, plus confirmation across all 6 admitted blocks (`origin/data/admitted-bwt`).

---

## 1. Exact Multi-Flip Moment Algebra (`[proved]`)

### Lemma 21.1 — Minimum flip multiplicity `k` for `(H, A, A₂)` invariance (`[proved]`)

Let `f = (i, m, j)` (`i < m < j`, `L[i] ≠ L[j]`) be any transposition flip swapping `L[i]` and `L[j]`. Then `ΔH(f) = 0`, and:

$$\Delta A(f) = (i - j)(L[j] - L[i]) \neq 0, \qquad \Delta A_2(f) = (i^2 - j^2)(L[j] - L[i]) = (i + j)\,\Delta A(f).$$

Consequently:
1. **`k = 1` flip:** Never preserves `A` (`ΔA(f) ≠ 0`).
2. **`k = 2` flips `{f₁, f₂}`:** `ΔA(f₁) + ΔA(f₂) = 0` and `ΔA₂(f₁) + ΔA₂(f₂) = 0` hold **if and only if** `ΔA(f₂) = −ΔA(f₁)` and **`i₁ + j₁ = i₂ + j₂`** (concentric nested spans `i₁ < i₂ < j₂ < j₁` sharing the exact midpoint `(i₁ + j₁)/2`). In particular, **no two pairwise-disjoint flips (`j₁ < i₂`) can ever preserve `(H, A, A₂)`** (`i₁ + j₁ < 2j₁ < 2i₂ < i₂ + j₂`).
3. **`k = 3` pairwise-disjoint flips:** Solvable whenever `ΔA = (+d, −2d, +d)` and the midpoint sums `(u₁, u₂, u₃) = (i_r + j_r)` form a 3-term arithmetic progression `u₁ + u₃ = 2 u₂`.
4. **`k = 4` pairwise-disjoint flips on a single symbol pair `{a, b}` (equal span `s = j_r − i_r`, signs `(+1, −1, −1, +1)`):** Satisfying **`i₁ + i₄ = i₂ + i₃`** preserves **not only** scalar `(H, A, A₂)`, **but also all `256` per-symbol vector moments `(H[c], A^{(c)}, A₂^{(c)})_{c=0}^{255}` simultaneously** (`A^{(c)} = Σ_{L[r]=c} r`, `A₂^{(c)} = Σ_{L[r]=c} r²`), since `(b − a)` factors out identically!

---

## 2. Three Explicit `O(n)`-Verified Fibre Witnesses on `block-0.L` (`[proved]` + `[measured]`)

On `block-0.L` (`n = 899,931`, `p = 264,045 ∈ J₁`, `h₁ = 35,116`), the base moments are:
- **`A(L) = 34,337,628,836,230`**
- **`A₂(L) = 21,202,246,597,268,704,910`**

All **three** minimum-multiplicity regimes (`k = 2` concentric, `k = 3` disjoint, `k = 4` all-256-symbol-vector-moment-preserving) exist on `block-0.L` and pass independent `O(n)` single-cycle and moment verification:

### Witness 21A — Minimum-Hamming (`k = 2` concentric nested, Hamming distance `4` bytes!) `[proved]`
- **Flip 1:** `(i₁ = 870, m₁ = 879, j₁ = 885)`, swapping `L[870] = 'o'` (`111`) ↔ `L[885] = 'n'` (`110`): `ΔA = (−15)(−1) = +15`, `i₁ + j₁ = 1,755`, `ΔA₂ = +26,325`.
- **Flip 2:** `(i₂ = 875, m₂ = 877, j₂ = 880)`, swapping `L[875] = '.'` (`46`) ↔ `L[880] = '1'` (`49`): `ΔA = (−5)(+3) = −15`, `i₂ + j₂ = 1,755`, `ΔA₂ = −26,325`.
- **Verification:** `L'` is a valid single `899,931`-cycle (`ΔH = 0, ΔA = +15 − 15 = 0, ΔA₂ = +26,325 − 26,325 = 0`), moving **`663,594` rows (`73.74%` of `n`)**!

### Witness 21B — Minimum Disjoint Multiplicity (`k = 3` pairwise-disjoint, Hamming distance `6` bytes) `[proved]`
- **Flips:** `(102, 113, 114)` (`ΔA = +12, i+j = 216`), `(2507, 2508, 2509)` (`ΔA = −24, i+j = 5,016`), `(4907, 4908, 4909)` (`ΔA = +12, i+j = 9,816`).
- **Arithmetic progression check:** `ΔA = 12 − 24 + 12 = 0`; `ΔA₂ = 12(216 − 2·5016 + 9816) = 12 · 0 = 0`.
- **Verification:** Valid single `899,931`-cycle, moving **`678,134` rows (`75.35%` of `n`)**!

### Witness 21C — Minimal-Span (`s = 2` each) Hex Witness Preserving All `256` Per-Symbol Vector Moments (`k = 4`, Hamming distance `8` bytes) `[proved]`
Exchanging only hex digits `'3'` (`51`) and `'4'` (`52`) across four disjoint length-3 windows:
1. `(6381, 6382, 6383)`: `L = b"433" → b"334"` (Type B), `ΔA = +2`, `i + j = 12,764`
2. `(53428, 53429, 53430)`: `L = b"344" → b"443"` (Type B), `ΔA = −2`, `i + j = 106,858`
3. `(232917, 232918, 232919)`: `L = b"334" → b"433"` (Type A), `ΔA = −2`, `i + j = 465,836`
4. `(279964, 279965, 279966)`: `L = b"443" → b"344"` (Type A), `ΔA = +2`, `i + j = 559,930`
- **Hand-checkable moment balance:** `ΔA = 2 − 2 − 2 + 2 = 0`; `(i₁ + i₄) − (i₂ + i₃) = (6381 + 279964) − (53428 + 232917) = 286,345 − 286,345 = 0`, so **`ΔH = 0, ΔA = 0, ΔA₂ = 0`, AND `ΔA^{(c)} = ΔA₂^{(c)} = 0` for all `c ∈ [0, 256)`**!
- **Verification:** Valid single `899,931`-cycle, moving **`784,644` rows (`87.19%` of `n`)**! *(Packing local `s ≤ 32` quads also yields `91` pairwise column-disjoint balanced 4-flips on `block-0.L`.)*

---

## 3. Exact & Tightly Bracketed `U_𝒪` on `block-0.L` (and All 6 Admitted Blocks) (`[proved]` + `[measured]`)

Recall from `N-18` Lemma 18.1 (accepted in MONODROMY Record 108 §1) that `o = p` (`origPtr`) always satisfies `pos_{L', p}(p) = 0` across the entire fibre (`p ∈ U_𝒪(L, p)`), while any finite set `𝒲 ⊆ Fib_𝒪(L, p)` of **independently `O(n)`-verified** valid fibre members gives a rigorous upper bound `U_𝒪(L, p) ⊆ U_𝒲(L, p) = {x : ∀ L' ∈ 𝒲, pos_{L', p}(x) = pos_{L, p}(x)}`.

- **On `block-0.L` (`13` `O(n)`-verified balanced 4-flip witnesses `𝒲`):**
  - **`1 ≤ |U_{(H, A, A₂)}(block-0.L)| ≤ 354` rows** (`0.0001%–0.0393%` of `n = 899,931`)!
  - **`1 ≤ |J₁ ∩ U_{(H, A, A₂)}(block-0.L)| ≤ 2` width-1 rows** (`J₁ ∩ U_𝒲 = {20776, 264045}`, where `264045 = origPtr` is provably in `J₁ ∩ U_𝒪` — so `|J₁ ∩ U_𝒪|` is **either `1` or `2` out of `35,116` (`0.0028%–0.0057%` of `h₁`)**!).
  - **Exact Theorem N′ + Lemma 18.1 event floor on `block-0.L` under `𝒪 = (H, A, A₂)`:**
    $$c_1 \ge h_1 - |J_1 \cap U_{(H, A, A_2)}| \ge 35{,}114 \;\; (99.9943\% \text{ of } h_1)!$$

### Six-block confirmation summary (every covering witness 100% `O(n)` byte-verified)

| Block | `h₁` | `p ∈ J₁`? | `#𝒲` (`O(n)`-verified 4-flips) | **`\|U_{(H, A, A₂)}\|` bracket** | **`\|J₁ ∩ U_{(H, A, A₂)}\|` bracket** | **Theorem N′ floor `c₁ ≥ h₁ − \|J₁ ∩ U_𝒪\|`** |
|---|---:|---|---:|---|---|---:|
| `block-0.L` | 35,116 | Yes (`1`) | 13 | `1 ≤ \|U_𝒪\| ≤ 354` | **`1 ≤ \|J₁ ∩ U_𝒪\| ≤ 2`** | **`≥ 35,114` (`99.9943%` of `h₁`)** |
| `block-2.L` | 34,262 | Yes (`1`) | 16 | `1 ≤ \|U_𝒪\| ≤ 116` | **`\|J₁ ∩ U_𝒪\| = 1` (EXACT: `{p}`!)** | **`≥ 34,261` (`99.9971%` of `h₁`)** |
| `block-85.L` | 55,504 | Yes (`1`) | 19 | **`\|U_𝒪\| = 1` (EXACT: `{p}`!)** | **`\|J₁ ∩ U_𝒪\| = 1` (EXACT: `{p}`!)** | **`≥ 55,503` (`99.9982%` of `h₁`)** |
| `block-143.L` | 72,578 | Yes (`1`) | 17 | `1 ≤ \|U_𝒪\| ≤ 30` | **`\|J₁ ∩ U_𝒪\| = 1` (EXACT: `{p}`!)** | **`≥ 72,577` (`99.9986%` of `h₁`)** |
| `block-210.L` | 97,054 | Yes (`1`) | 18 | `1 ≤ \|U_𝒪\| ≤ 4` | **`\|J₁ ∩ U_𝒪\| = 1` (EXACT: `{p}`!)** | **`≥ 97,053` (`99.9990%` of `h₁`)** |
| `block-207.L` | 62,407 | Yes (`1`) | 13 | `1 ≤ \|U_𝒪\| ≤ 65` | **`\|J₁ ∩ U_𝒪\| = 1` (EXACT: `{p}`!)** | **`≥ 62,406` (`99.9984%` of `h₁`)** |

---

## 4. Obstruction Table Entry (All Six Required Fields)

| Field | Value |
|---|---|
| **Framework** | Order-sensitive polynomial-moment observations `𝒪 = (H, A, A₂)` (and all-256-symbol vector moments `(H[c], A^{(c)}, A₂^{(c)})`) for `P_𝒪` positional labelling in `Λ′` (`N-21`, MONODROMY `ASK N (2)`) |
| **Route class** | **R2** (free `O(1)`-word fold positional labels `P_𝒪` on start rows of rigid facts in `Λ′`) |
| **Gate failed** | **(a) Identifiability / Soundness (`|J₁ ∩ U_𝒪|` ceiling in Theorem N′ + Lemma 18.1):** on `block-0.L`, `|J₁ ∩ U_{(H, A, A₂)}| ≤ 2` (`0.0057%` of `h₁`), and on **all 5 remaining blocks (`block-2, 85, 143, 210, 207`)** **`|J₁ ∩ U_{(H, A, A₂)}| = 1` (`{origPtr}` alone!)**, forcing `c₁ ≥ h₁ − 2` (`≥ 99.994%` of `h₁`) across every decoder in `Λ′` using `(H, A, A₂)` |
| **Witness** | All `O(n)` byte-verified on `block-0.L` (`search_n21_balanced_multiflip.py`): `k = 2` concentric `[(870,879,885),(875,877,880)]` (Hamming 4, `73.7%` moved); `k = 3` disjoint `[(102,113,114),(2507,2508,2509),(4907,4908,4909)]` (Hamming 6, `75.4%` moved); `k = 4` hex `'3'↔'4'` `[(6381..6383),(53428..53430),(232917..232919),(279964..279966)]` (Hamming 8, preserves all 256 per-symbol moments, `87.2%` moved); plus `13–19` covering 4-flips per block across all 6 admitted blocks |
| **Scope** | All decoders in `Λ′` (Record 107 §2) whose `P_𝒪` observation is any deterministic function of `(H, A, A₂)` — or even of all `768` per-symbol moments `(H[c], A^{(c)}, A₂^{(c)})_{c=0}^{255}` — on the admitted blocks of `𝓕` |
| **Escape condition** | Either observe `Ω(n)` fine-grained cells / `Ω(n)` polynomial moments (since `d` moments vanish on `2^d`-flip finite-difference stencils), OR leave `Λ′`'s rigid-write model (whose `K = 2` limits are quantified in `N-19` and `N-20`) |
| **Reach** | **Exact (`[proved]` + `[measured]`)** on all 6 admitted blocks: `|J₁ ∩ U_𝒪|` is pinned to `{1, 2}` on `block-0.L` and to **exact `1` (`{origPtr}`)** on **5/6 blocks (`block-2, 85, 143, 210, 207`)** (with `|U_𝒪| = 1` exact out of `899,935` rows on `block-85.L` and `1 ≤ |U_𝒪| ≤ 4` on `block-210.L`!) |
| **Strength** | **Proved** (Lemma 21.1 + `100%` full-`O(n)` independent verification of every witness column `L'` in `𝒲`, zero reliance on unverified heuristics) |
| **Transferable insight** | **Finite-difference dipole cancellation (`2^d` equal-span flips for degree-`d` moments) scales small-`n` fibre ambiguity directly to `n = 900,000` with zero loss of orbit coverage:** pairing two `ΔA = 0` dipoles with equal spacing (`i₁ + i₄ = i₂ + i₃`) annihilates `(ΔH, ΔA, ΔA₂)` simultaneously across all 256 symbols and matches the single-flip envelope `U_all-flips` (`|J₁ ∩ U_𝒪| = 1` exact on 5/6 blocks, `{origPtr}` alone on `block-85.L`!) — closing Record 108 §1's large-`n` gap for polynomial moments completely |

---

## 5. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact raw `.L` columns from `origin/data/admitted-bwt`.
2. **Model assumptions:** Every witness `L'` used in any claim or bound is constructed explicitly in memory and verified in `O(n)` from scratch (`compute_lf`, single `n`-cycle check, exact 64/128-bit integer `A, A₂`, and exact `pos_{L', p}`).
3. **Method:** Meet-in-the-middle over `i_1 + i_4 = i_2 + i_3` guided by a `13`-vertex compressed-arc monodromy check, followed by full `O(n)` verification of the `13–19` covering witnesses per block.
4. **Outcome:** Answers MONODROMY `ASK N (2)` completely on `block-0.L` and across all 6 blocks, exhibiting `k = 2, 3, 4` witnesses and proving `|J₁ ∩ U_{(H, A, A₂)}| = 1` exact on 5/6 blocks (`≤ 2` on `block-0.L`, and `|U_𝒪| = 1` exact out of `899,935` rows on `block-85.L`!).
