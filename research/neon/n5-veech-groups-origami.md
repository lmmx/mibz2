# N-5: E2 Screen 3 — Veech groups and the `SL(2, ℤ)` action on the `(LF, +1)` origami

*Directed kitchen-sink screen for Stream N (E2 open cell) under frozen parameters `K = 2` (also checked at `K = 3`), `δ = 1/2`, `S(c) ≤ openPeakScratch c`, and anti-cheat clauses 1–5 (`origin/claude/sharp-meitner-cx0v78` `4d7d83b`, `main-theorem.md` §1–1a).*

**Script:** [`research/neon/scripts/screen_cycle1_frameworks.py`](scripts/screen_cycle1_frameworks.py) (exact `SL(2, ℤ)` cylinder decompositions and piece/width-1 invariance checks across directions `(1,0)`, `(1,1)`, `(1,-1)`, `(2,1)`, `(1,2)` on all 6 admitted blocks of `origin/data/admitted-bwt`).

**Verdict:** Split into two sub-routes:
- **Route V1 (transporting cylinders from another rational slope `(a, c)` back to `(1, 0)`):** **CLOSED at (a) Escape** (`R1` inside `𝓡`; Theorem V1 proves every `a ≠ 0` slope has width-1 cylinder strips through every width-1 run, and piece/width-1 counts are invariant across `(1, 0), (1, 1), (1, -1)`).
- **Route V2 (nontrivial Veech-group / affine-automorphism symmetries `Aff⁺(O)`):** **PASSES (a) Escape** (`R2` global non-rigid symmetry), **CLOSED at (c) Viability** (`Aut(O) = {id}` proved for every primitive block; `T^k ∈ SL(O) ⟺ n | k` proved; `(1,1)` vs `(1,-1)` cylinder spectra asymmetric on 6/6 admitted blocks with generic random-permutation cylinder counts `14–71`).
Status: `tested failure within defined scope`.

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - The origami `O(L) = (r, u) ∈ S_n × S_n` with `r = LF` and `u(i) = i + 1 mod n`, verified on the 6 admitted blocks in MONODROMY Record 86 (`32d32f7`, `origami-verification.md`: `supp k = C̄ \ X` with `X = ∅`, single horizontal and vertical `n`-cylinders on all 6 blocks).
   - Exact cylinder decompositions in slopes `(1, 1)`, `(1, -1)`, `(2, 1)`, `(1, 2)` measured on the 6 admitted blocks (`screen_cycle1_frameworks.py`).
2. **Real code path:**
   - Baseline: `OpenCells` (`LIMIT = 16`, 1W i9 kernel time `276.23 ms` across 232 blocks; `n2-f-baseline-and-needs-m2.md`).
3. **Assumptions knowingly unrealistic:**
   - None: all permutation compositions and cycle decompositions are exact on the 6 admitted blocks.

## 1. Directed four-gate screen

### Route V1: Alternative rational directions `(a, c)` via `SL(2, ℤ)` Nielsen moves

#### (a) Escape: Does moving to direction `(a, c)` escape `𝓡`? — **NO** `[proved]`

**Theorem V1 (Singular-strip bottleneck for all `SL(2, ℤ)` directions) `[proved]`:**
Let `O(L) = (r, u)` be an origami with `r = LF`, `u(i) = i + 1 mod n`, and `X = ∅` (no two distinct maximal runs share a translation mod `n`, verified on all 6 admitted blocks by Record 86). Then:
1. For `A = T^k : (r, u) ↦ (r, u r^{-k})` and `u^{±1} ∘ r` (slopes `(1, ±1)`), adjacent squares `x, x + 1` satisfy `(u^{±1} ∘ LF)(x + 1) = (u^{±1} ∘ LF)(x) + 1` if and only if `LF(x + 1) = LF(x) + 1`, i.e., if and only if `x` and `x + 1` lie in the **same maximal run of `L`**. Thus the translation-piece decomposition and width-1 piece count `h₁` are **identical** in slopes `(1, 0)`, `(1, 1)`, and `(1, -1)`.
2. More generally, in **every** rational direction `(a, c)` with `a ≠ 0` (every direction not purely vertical `(0, ±1)`), any trajectory crossing a square `x` in a width-1 run `R = {x}` crosses a vertical edge whose top-right and bottom-right corners are both singular cone points (`x ∈ supp(k)` on both sides since `L[x - 1] ≠ L[x] ≠ L[x + 1]` and `X = ∅`). Because flat cylinders cannot contain cone points in their interior, every flat cylinder passing through `x` has **orthogonal width 1**.
3. Reading back to the horizontal orbit by jumps along any `SL(2, ℤ)` direction is already covered by MONODROMY Record 92 Lemma 2 (jump-walk universality), and transporting width-1 cylinders is a rigid width-1 operation in `𝓡` (`c₁ ≥ h₁` by G1). The purely vertical direction `(0, ±1)` (`u = +1 mod n`) never invokes `LF` and determines no orbit order.

**Empirical verification of Theorem V1(1) (`screen_cycle1_frameworks.py`) `[measured]`:** On all 6/6 admitted blocks, the exact translation-piece count and width-1 piece count of `LF`, `u ∘ LF`, and `u⁻¹ ∘ LF` are **bit-for-bit identical** (`(r, h₁)` unchanged to the last integer).

### Route V2: Nontrivial affine symmetries (`Aff⁺(O)` and Veech group `SL(O)`)

#### (a) Escape: Would a nontrivial affine automorphism `φ ∈ Aff⁺(O)` escape `𝓡`? — **YES** `[proved]`

A nontrivial affine automorphism `φ ∈ Aff⁺(O)` with derivative `Dφ = A ∈ SL(O)` conjugates the monodromy `(r, u)` to `A · (r, u)`, globally identifying distant squares `x ↦ φ(x)` across the surface. Such a symmetry is a multi-row non-rigid fact outside `𝓡`.

#### (b)–(c) Passes & Viability on `𝓕` — **NO** `[proved]` (for `Aut(O)` and parabolics) + `[measured]` (on 6 admitted blocks)

**Parameter cost scales with:** The index `[SL(2, ℤ) : SL(O)]` (or equivalently, the reciprocal of `|Aff⁺(O)|`, the symmetry order that could factor `h₁` into orbits of size `≥ 2`). For viability, `O(L)` must admit nontrivial deck or affine symmetries (`|Aff⁺(O)| ≥ 2` reducing `h₁` by a constant factor `≥ δ = 1/2`).

1. **Deck group `Aut(O)` is trivial on every primitive block `[proved]`:** Any permutation `φ ∈ S_n` commuting with the single `n`-cycle `u(i) = i + 1 mod n` is a rigid shift `φ(i) = i + s mod n`. Commutation with `LF` (and preserving `L`) forces `L` and `T` to be periodic of period `gcd(s, n) < n`. Every block in `𝓕` is aperiodic (single `LF` cycle of length `n`, `#805` census `cycles = 1` on 232/232 blocks and 6/6 admitted blocks), so **`Aut(O) = {id}` identically on all 238 blocks of `𝓕`**.
2. **Horizontal/vertical parabolic subgroup is minimal `[proved]`:** Because both `(1, 0)` and `(0, 1)` decompose into a single cylinder of circumference `n` and width `1`, a horizontal Dehn twist `T^k` lies in `SL(O)` iff `n | k` (similarly `S T^k S⁻¹ ∈ SL(O)` iff `n | k`), giving only trivial twists on `O`.
3. **Asymmetry and generic cylinder spectra on the 6 admitted blocks (`screen_cycle1_frameworks.py`) `[measured]`:**

| Block | `(r, h₁)` in `(1,0),(1,1),(1,-1)` | Cyls `(1,1)` (top 3 lengths) | Cyls `(1,-1)` (top 3 lengths) | Cyls `(2,1)` | Cyls `(1,2)` | `(1,1) == (1,-1)`? |
|---|---:|---:|---:|---:|---:|:---:|
| `block-0.L` | `(51184, 35116)` | `27` (`356388, 329516, 202667`) | `23` (`426467, 317820, 94550`) | `29` | `17` | **False** |
| `block-2.L` | `(54532, 34262)` | `44` (`588130, 130901, 128463`) | `22` (`716713, 83573, 75462`) | `65` | `27` | **False** |
| `block-85.L` | `(79375, 55504)` | `23` (`253294, 252255, 213310`) | `15` (`610155, 228008, 31859`) | `19` | `21` | **False** |
| `block-143.L` | `(108335, 72578)` | `53` (`282482, 281721, 207217`) | `53` (`368552, 280756, 199478`) | `43` | `17` | **False** |
| `block-210.L` | `(128732, 97054)` | `25` (`728218, 50019, 44613`) | `71` (`319347, 226399, 140668`) | `47` | `25` | **False** |
| `block-207.L` | `(87111, 62407)` | `14` (`880309, 14751, 1897`) | `28` (`540333, 246260, 99567`) | `25` | `19` | **False** |

- `S = [[0, -1], [1, 0]] ∉ SL(O)` on 6/6 blocks (since `S` maps slope `(1, 1)` to `(-1, 1) ∼ (1, -1)`, requiring identical cylinder-length multisets, which fails on 6/6 blocks).
- Cylinder counts (`14–71`) and top-cycle dominance match generic random permutations (`E[cycles] = H_n ≈ 14.3`), and computing the Veech group `SL(O)` itself requires enumerating the `SL(2, ℤ)`-orbit of the `n`-sheeted cover (cost `≫ n`, violating `K = 2` and `S ≤ openPeakScratch`).

## 2. Obstruction table entry

| Field | Value |
|---|---|
| **Framework** | Veech groups and `SL(2, ℤ)` action on `(LF, +1)` origami (`N-5`) |
| **Route class** | **R1** for Route V1 (cylinder transport in other slopes); **R2** (non-viable on `𝓕`) for Route V2 (`Aff⁺(O)` symmetries) |
| **Gate failed** | **Route V1:** **(a) Escape** (Theorem V1: width-1 bottleneck in all `a ≠ 0` slopes). **Route V2:** **(b) Passes** and **(c) Viability** (`Aut(O) = {id}` proved on all 238 primitive blocks; `S ∉ SL(O)` on 6/6 blocks; `T^k ∈ SL(O) ⟺ n \| k`) |
| **Witness** | All 6 admitted blocks (`block-0.L`..`block-210.L`): `(r, h₁)` invariant across `(1,0),(1,1),(1,-1)`; `(1,1)` vs `(1,-1)` cylinder multisets unequal on 6/6; plus `cycles = 1` on all 232 census blocks (`#805`) proving `Aut(O) = {id}` |
| **Scope** | All affine `SL(2, ℤ)` re-orientations, cylinder transports, and deck/Veech-group symmetry reductions of the `(LF, +1)` origami on primitive aperiodic blocks |
| **Escape condition** | Leave exact affine automorphisms of `(LF, +1)` (which require exact global symmetry of the entire block); seek *partial / local* non-rigid symmetries restricted to a subset of rows, or leave the square-tiled cover representation altogether |
| **Reach** | Global for `Aut(O)` on primitive blocks; bounded to `SL(2, ℤ)` flat-surface actions for V1/V2 |
| **Strength** | Proved (Theorem V1, `Aut(O) = {id}`, parabolic index `n`) and measured (exact cylinder decompositions on 6 admitted blocks) |

## 3. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Verified exact on all 6 admitted blocks (`screen_cycle1_frameworks.py`) and `#805`'s `cycles = 1` across 232 blocks.
2. **Model assumptions:** Directly extends MONODROMY Record 86 (`origami-verification.md`) and Record 92 Lemma 2 to non-jump cylinder decompositions and Veech symmetries.
3. **Method:** Proof of Theorem V1 + exact computation of `(1,1)`, `(1,-1)`, `(2,1)`, `(1,2)` cylinder spectra on all 6 admitted blocks.
4. **Outcome:** Closes the remaining open question of `origami-lift.md` §6.2 ("directions and transport" and Veech groups) for the major shot. No premise `P_N` survives.
