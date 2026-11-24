# N-20: Constructive Copy-Plan & Unimodal-Arch Acquisitions from Run Records, Plus `NEEDS M-17` on the 6 Admitted Blocks

*Cycle 5 constructive investigation answering MONODROMY `ASK N (1)` (Record 108, `8e9b962`), Coordinator Notes #12, #13, and #14 (`c4847b8`, `7e10769`, `8cc654c`), MONODROMY `NEEDS M-17` (Record 110, `f86a4d3`), and placing all results in MONODROMY Record 111's `(Q, D, P, S)` cursor-and-addressed machine model (`303b48c`).*

**Script:** [`research/neon/scripts/construct_n20_copy_and_sweeps.py`](scripts/construct_n20_copy_and_sweeps.py) (reusing `measure_cycle2.py` and `screen_cycle3_supplied.py`; every construction and sweep is executed and asserted **byte-exact** `decoded == T_true` across small witnesses, 60 KB novel random hex, and all 6 admitted blocks of `origin/data/admitted-bwt`).

---

## 0. Pre-declared decision rules & declared instruction sets (Record 108 §2 & Record 111 §1)

### 0.1 Pre-fixed decision rules (`CLAIMS.md` `8239a11`, fixed before data)

1. **N-20 (`K = 2` non-walking acquisition):** Under `ℐ_lenient` (one uncharged non-walking pass over the run records `R_0..R_{r-1}`, `K = 2`, `S ≤ openPeakScratch`), classify as a **count-yes** iff consumer + row-addressed steps `≤ h₁ / 2` on all 6 admitted blocks; otherwise report the **best constructive near misses with exact cost accounting** (where every pass, byte, and addressed access goes) plus a **6-field obstruction**.
2. **`NEEDS M-17` (pre-fixed by MONODROMY in Record 110 §4):** On Open's `m₁₆` retained map, if 1 ascending Gauss–Seidel sweep yields **`< 2.3` retained rows per link on `≥ 4` of the 6 admitted blocks**, Proposition S's order-statistics model does not transfer to the retained map and the near miss is withdrawn.

### 0.2 Declared instruction sets (in Record 108 & Record 111's exact sense)

By Record 108 §2 (`[proved]`), a count is well-defined only relative to a declared instruction set. We state three nested regimes and report Record 111's 4-coordinate cost vector **`(Q, D, P, S)`** (`Q` = addressed non-cursor accesses, `D` = dependent-chain depth of addressed accesses, `P` = streamed bytes `/ n`, `S` = peak working bytes, with `s ≤ |Σ| + O(1)` `±1`-monotone cursors):

1. **Consumer primitives (shared across all regimes, `1` instruction = `1` `Q`-step per invocation unless literal):**
   - `emit_macro(x → x_next, w)`: emit inline word `w` (`1 ≤ |w| ≤ cap`) backwards/forwards and jump to `x_next` (`ΔQ = 1, ΔD = 1`).
   - `copy_pos(dst, src, len)` (`0 ≤ src < dst`, `len ≥ 2`): copy `out[src .. src + len)` to `out[dst .. dst + len)` (`ΔQ = 1` random read from `out[src]`).
   - `copy_online(x, y, len, x_next)`: look up `src = emitted_pos[y]`, copy `len` bytes from `out[src]`, stamp `emitted_pos[x]`, and jump to `x_next` (`ΔQ = 3`: `emitted_pos[y]`, `out[src]`, `next`).
2. **Acquisition regimes over `R_0..R_{r-1}` (or `L[0..n)`):**
   - **`ℐ_strict` (Record 108 §4 operational Clause 2):** pure associative folds to `O(1)` words (`H, C[], A, A₂`) cost `0`; any pass reading run record `R_a` to route, compose, or match its rows pays `|R_a|` events (`1` per width-1 row read). Under `ℐ_strict`, every `T1`-exact acquisition+consumer pays `c₁ ≥ h₁ − max_c h₁(c) ≥ 0.9317 · h₁` on `𝓕` (`[proved]`, Record 108 §3 + `N-19`).
   - **`ℐ_lenient(K)` (Record 108 §3 lenient pass + Residual `(c)`):** up to `K − 1` non-walking build sweeps over `R_0..R_{r-1}` are uncharged in `c₁`, scoring consumer link/copy instructions `z` only.
   - **`ℐ_vec(K, s_max, S_0)` (Record 111 §1 cursor-vs-addressed vector):** cursor reads (`±1` step per symbol bucket, `s ≤ |Σ| + O(1)`) contribute to `P` only (`ΔQ = 0, ΔD = 0`), while data-addressed array lookups contribute to `Q` and `D`.

---

## 1. Constructive Copy-Plan Acquisitions (`Procedures C1–C3`) & Structural Theorems

### 1.1 Why offline positional copy plans `copy_pos(dst, src, len)` cannot be acquired without list-ranking `LF` (`[proved]`)

**Proposition 20.1 (Offline Copy Acyclicity Blindness, `[proved]`, machine-verified in `construct_n20_copy_and_sweeps.py`):**
There exist valid single-cycle columns `(L_1, p)` and `(L_2, p)` differing by a single distant valid flip `(i, m, j)` such that two occurrences `W_A, W_B` of a phrase `W` (`|W| = 3`) consist entirely of **width-1 rows** (`6/6` rows in `J_1`) with **bit-for-bit identical run+LF records `(s_x, e_x, L[x], LF(x) - x)` and 2-step `LF` neighbourhoods** in `L_1` and `L_2`, yet `pos(W_A) < pos(W_B)` in `T(L_1)` whereas `pos(W_B) < pos(W_A)` in `T(L_2)`.

- **Verified witness (`n = 31, p = 0`, flip `(1, 8, 10)`):**
  - `W_A` (`"abc"` inside `"1abc2"`, emitting rows `[18, 21, 3]`) and `W_B` (`"abc"` inside `"7abc9"`, emitting rows `[20, 25, 11]`) have all 6 rows width-1 and identical local run+LF neighbourhoods in `L_1` and `L_2`.
  - In `T(L_1)`, `W_A` is at offset `4` and `W_B` at `24` (`W_A → W_B` is acyclic, `dist = +20`); in `T(L_2)`, `W_B` is at `4` and `W_A` at `9` (`W_A → W_B` becomes **cyclic / forward-referencing**, violating `src < dst`!).
- **Consequence (`[proved]`):** Any non-walking copy acquisition MUST either resolve `src` *online* during emission (`copy_online`) or replace text-offset copies by position-free **macro-edges** `(x → LF^ℓ(x), w)`.

### 1.2 Lemma 20.2 — Macro-edge dominance over online copy (`[proved]`)

Suppose an online copy instruction `copy_online(x, y, ℓ, x_next)` skips `ℓ ≥ 2` walk steps at row `x` by copying `ℓ` bytes from an earlier-emitted row `y`.
1. At the boundary of any maximal `ℓ`-byte match (`T[p_1 - ℓ] ≠ T[p_2 - ℓ]` in backward walk, or `T[p_1 + ℓ] ≠ T[p_2 + ℓ]` in forward walk), **`LF^ℓ(x) ≠ LF^ℓ(y)`** (in forward walk `Ψ^ℓ(x)` and `Ψ^ℓ(y)` lie in distinct `F`-buckets!). Thus `y`'s earlier trajectory **never** supplies `x_next = LF^ℓ(x)`: the build pass must compute `x_next = LF^ℓ(x)` for `x` itself (depth-`ℓ` composition).
2. For every `ℓ ≤ 16`, once the build pass already computes `x_next = LF^ℓ(x)`, storing the `ℓ` literal bytes `w[x]` inline (`emit_macro(x → x_next, w[x])`) uses **strictly less state `S`** and **fewer addressed accesses `Q`** (`1` vs `3`: no `partner[x]`, no `emitted_pos[y]` table, no random read `out[src]`) than `copy_online`!

### 1.3 Six-block execution of `Procedure C1` (`Φ`-run sibling copy) & `Procedure C2` (Run-boundary attractors + LZ77 scatter) (`[measured]`, byte-exact)

| Block | `h₁` | `r` (`\|\Gamma\|`) | `J₁ ⊆ \Gamma` | **C1 (`Φ`-sibling copy)** total instr (`× h₁`) | C1 copies used | **C1 `J₁` saved** | **C2 (`min_len=2` LZ77)** `J₁` in `len=2` | **`J₁` in `len ≥ 3`** | Med. `\|run(x) - run(y)\|` (`≤ 1` run) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | 51,184 | **100%** | 348,944 (`9.94×`) | 4,497 | **`0` (`0.0%`)** | 12.0% | **87.2%** | **203** (`0.00%`) |
| `block-2.L` | 34,262 | 54,532 | **100%** | 345,196 (`10.08×`) | 5,394 | **`0` (`0.0%`)** | 13.3% | **85.8%** | **173** (`0.00%`) |
| `block-85.L` | 55,504 | 79,375 | **100%** | 342,409 (`6.17×`) | 7,670 | **`0` (`0.0%`)** | 9.2% | **90.0%** | **211** (`0.00%`) |
| `block-143.L` | 72,578 | 108,335 | **100%** | 386,250 (`5.32×`) | 10,528 | **`0` (`0.0%`)** | 9.4% | **89.8%** | **237** (`0.00%`) |
| `block-210.L` | 97,054 | 128,732 | **100%** | 365,974 (`3.77×`) | 10,474 | **`0` (`0.0%`)** | 7.3% | **92.2%** | **312** (`0.00%`) |
| `block-207.L` | 62,407 | 87,111 | **100%** | 357,198 (`5.72×`) | 8,495 | **`0` (`0.0%`)** | 11.0% | **88.3%** | **214** (`0.00%`) |

**Three structural takeaways answering Coordinator #13 (`[proved]` + `[measured]`):**
1. **`Φ`-runs (`Procedure C1`) save `0` width-1 rows (`[proved]` + `[measured]`):** Two adjacent same-run rows `x, x±1` co-travel only while `L[LF^k(x)] = L[LF^k(x) ± 1]`, meaning **every row along a `Φ`-copy has run width `≥ 2`**. It compresses `W`-rows only, never `J_1`.
2. **Kempa–Prezza run-boundary attractors (`\Gamma`, `Procedure C2`) are trivial on `J_1` (`[proved]`):** Every width-1 run `R_a = {x}` is its own run boundary, so **`J_1 ⊆ \Gamma` (`100%`)** and `|\Gamma| = r = 1.33–1.59 × h₁`.
3. **Why `min_len = 2` LZ77 (`z/h₁ = 0.48–0.64`) is invisible to 1-pass run windows (`[measured]`):** **`85.8–92.2%`** of `h₁` bytes in `min_len = 2` LZ77 sit in phrases of length `ℓ ≥ 3` (mostly `ℓ = 3`: `41.2–45.6%`, requiring depth `≥ 3`), and **`0.00%`** of copied `h₁` rows lie within `≤ 1` run of their copy source (`|run(x) - run(y)|` median **`173–312` runs** apart, because `x ∈ J_1` forces at least one intervening `≠ L[x]` run between `x` and `y`!).

---

## 2. Discharging MONODROMY `NEEDS M-17` — Gauss–Seidel Sweep Composition on the 6 Admitted Blocks (`[measured]`, byte-exact)

### 2.1 Part (i) — Sweep composition on `LF` over all `n` rows

- **Sequential-cursor verification (`[proved]` + `[measured]`):** Confirmed on all 6 blocks that Sweep 1 (`0 .. n-1`) reads `y = LF(x)` strictly via **`256` `+1`-advancing symbol cursors (`Q(\text{Sw1}) = 0`)**. By contrast, Sweep 2 (`n-1 .. 0`) reads `J_1[x] = LF^{2+d(x)}(x)` across variable depths and therefore requires **`Q(\text{Sw2}) = n` addressed accesses** (`[proved]`).

| Block | `n` | `h₁` | 1-Sweep `r/link` (`cap=8 / 16 / ∞`) | 1-Sw `links / h₁` | 2-Sw `r/link` (`cap=8`) | 2-Sw `r/link` (`cap=16`) | 2-Sw `r/link` (`∞`) | 2-Sw `links / h₁` (`cap=16`) |
|---|---:|---:|---|---:|---:|---:|---:|---:|
| `block-0.L` | 899,931 | 35,116 | `2.603 / 2.603 / 2.603` | `9.85×` | 5.610 | 7.256 | 7.271 | `3.53×` |
| `block-2.L` | 900,000 | 34,262 | `2.620 / 2.620 / 2.620` | `10.02×` | 5.653 | 7.303 | 7.321 | `3.60×` |
| `block-85.L` | 899,935 | 55,504 | `2.632 / 2.633 / 2.633` | `6.16×` | 5.687 | 7.289 | 7.300 | `2.22×` |
| `block-143.L` | 899,869 | 72,578 | `2.635 / 2.635 / 2.635` | `4.70×` | 5.778 | 7.277 | 7.291 | `1.70×` |
| `block-210.L` | 899,959 | 97,054 | `2.630 / 2.630 / 2.630` | `3.53×` | 5.680 | 7.214 | 7.226 | `1.29×` |
| `block-207.L` | 899,998 | 62,407 | `2.645 / 2.645 / 2.645` | `5.45×` | 5.809 | 7.395 | 7.406 | `1.95×` |

- **Edge-length distribution on raw `LF` (`[measured]`):**
  - *After 1 sweep (`cap=16`):* `med = 2, p90 = 4, max = 8–11`; `len=2`: `52.8–55.4%`, `len=3`: `30.7–32.9%`, `len=4..7`: `13.3–14.5%`, `len ≥ 8`: `0.0%`.
  - *After 2 sweeps (`cap=8`):* `med = 6, p90 = 8, max = 8`; `len=2`: `7.1–9.4%`, `len=3`: `4.1–4.8%`, `len=4..7`: `71.9–74.4%`, `len=8`: `13.6–16.3%`.
  - *After 2 sweeps (`cap=16`):* `med = 7, p90 = 11, max = 16`; `len=4..7`: `59.5–62.4%`, `len=8`: `9.7–12.3%`, `len=9..15`: `25.3–28.9%`, `len=16`: `0.3–0.7%`.

### 2.2 Part (ii) — Sweep composition on Open's `m₁₆` retained map (`LIMIT = 16`): Step-Cap Transfers (`2.73–2.78 > 2.3`), Literal 8-Byte Cap Splits!

On Open's `m₁₆` map, a retained edge `s → R(s)` already carries `b_0(s) ≥ 1` text bytes (`b_0(s) > 1` whenever `s` traverses thick strips `w > 16`). We therefore measure **both** MONODROMY `gs.py.txt`'s **retained-step cap** (`ell_ret ≤ cap`) and a **literal inline-byte cap** (`|word| ≤ cap` bytes):

| Block | `m₁₆` (`T² = m₁₆/2`) | `P(R(s) < s)` | **Step-cap `8/16` 1-Sw `ret/link`** | **1-Sw `links / h₁` (`vs T²`)** | **Step-cap `8` 2-Sw `ret/link` (`/ h₁`)** | **Step-cap `16` 2-Sw `ret/link` (`/ h₁`)** | **Byte-cap `8` 1-Sw / 2-Sw `ret/link`** | **Byte-cap `16` 1-Sw / 2-Sw `ret/link`** |
|---|---:|---:|---:|---|---:|---:|---|---|
| `block-0.L` | 89,874 (`44,937`) | 0.5026 | **`2.737 / 2.737`** | `0.935×` (`0.731× T²`) | `5.714` (`0.448×`) | `7.441` (`0.344×`) | `2.123 / 3.376` | `2.246 / 4.174` |
| `block-2.L` | 112,070 (`56,035`) | 0.5040 | **`2.732 / 2.732`** | `1.197×` (`0.732× T²`) | `5.671` (`0.577×`) | `7.425` (`0.441×`) | `2.108 / 3.188` | `2.253 / 4.059` |
| `block-85.L` | 156,404 (`78,202`) | 0.5126 | **`2.782 / 2.782`** | `1.013×` (`0.719× T²`) | `5.705` (`0.494×`) | `7.559` (`0.373×`) | `2.286 / 3.788` | `2.429 / 5.008` |
| `block-143.L` | 196,841 (`98,421`) | 0.5023 | **`2.744 / 2.745`** | `0.988×` (`0.729× T²`) | `5.702` (`0.476×`) | `7.441` (`0.364×`) | `2.056 / 3.021` | `2.286 / 4.066` |
| `block-210.L` | 204,281 (`102,141`) | 0.5028 | **`2.762 / 2.762`** | `0.762×` (`0.724× T²`) | `5.744` (`0.366×`) | `7.478` (`0.281×`) | `2.157 / 3.364` | `2.340 / 4.513` |
| `block-207.L` | 135,921 (`67,961`) | 0.5080 | **`2.733 / 2.733`** | `0.797×` (`0.732× T²`) | `5.708` (`0.382×`) | `7.380` (`0.295×`) | `2.143 / 3.302` | `2.334 / 4.331` |

- **Retained-step edge-length distribution (`Step-cap 16`, `[measured]`):**
  - *After 1 sweep:* `med = 2–3, p90 = 4, max = 9–11`; `len=2`: `49.0–50.7%`, `len=3`: `32.2–34.8%`, `len=4..7`: `16.1–18.5%`, `len=8`: `0.0–0.1%`.
  - *After 2 sweeps:* `med = 7, p90 = 11, max = 16`; `len=2`: `0.2–0.5%`, `len=4..7`: `55.5–59.6%`, `len=8`: `10.8–12.4%`, `len=9..15`: `28.2–31.6%`, `len=16`: `0.6–1.0%` (`Step-cap 8` 2-sweep: `med = 6, p90 = 8, max = 8`; `len=4..7`: `70.5–73.1%`, `len=8`: `15.4–17.8%`).
- **Verdict on MONODROMY's pre-fixed `2.3` rule (`[measured]` + `[argued]`):**
  1. **Order statistics (`Step-cap 8/16`): STANDS (`0 / 6` blocks below `2.3` — actually `2.732–2.782 > e = 2.718` on all 6 blocks!).** Proposition S transfers *better* than `e` to `R` because `P(R(s) < s) = 0.5023–0.5126 > 1/2`. One sweep cuts `T²`'s link count to `0.719–0.732 × T²` (`0.762–1.197 × h₁`, missing `δ = 1/2` at `K = 2`), while two sweeps (`K = 3`, `Step-cap 16`) reach **`0.281–0.441 × h₁ < 0.50 × h₁` on all 6 blocks**!
  2. **Critical representation catch (`Byte-cap`): literal `8`-byte inline buffers drop to `2.056–2.286` (`6 / 6` blocks `< 2.3`!), and `16`-byte buffers hit `2.246–2.429` (`3 / 6` blocks `< 2.3`).** Why? Whenever composition hits a retained row carrying a contracted thick strip (`> 8` bytes), literal inline concatenation refuses to compose across it. **Engineering corollary:** any macro-edge representation on `R` must store `(inline_thin_bytes, optional_thick_strip_id)` rather than flattening thick strips into the 8/16-byte inline word!
  3. **Alternative `O(r)` sweep keys `κ` (`block-0.L`, `Step-cap 16`):** `ascending_rank` (`2.737`), `descending_rank` (`2.740`), `symbol_freq_asc` (`2.717`), `symbol_freq_desc` (`2.745`), `bit_reversed` (`2.719`), and **`strip_len_desc` (`2.820`, `+3.0%` best)** — processing thick-strip carriers first lets thin rows inherit them in the same sweep, though all record-local keys stay bounded near `P(κ(R(s)) < κ(s)) ≈ 0.50`.
  4. **Access-kind note in Record 111's `(Q, D, P, S)` vector (`[proved]`):** Unlike raw `LF` Sweep 1 (`Q = 0`), on `R` (`m₁₆`), `R(s)` jumps across variable-length thick strips so it is **not** `±1` cursor-monotone; thus each sweep over `R` costs `Q = m₁₆` addressed accesses (`D ≈ log m₁₆` in-place, `S = 12–20 · m₁₆` bytes, exceeding `openPeakScratch = 4 · m₁₆`!).

---

## 3. Coordinator Note #14's New Post-Interval Basis — `Procedure C4`: Unimodal Orbit Arches (`𝒜`, Two-Cursor Boustrophedon)

Following Coordinator Note #14 and MONODROMY Record 111 §2 (`N1–N3`), we construct and test a new mathematical basis beyond points and intervals:

### 3.1 Definition & Theorem 20.3 (Unimodal Orbit Arch Decomposition, `[proved]`)

For any single-cycle map `g : [m] → [m]` (`m ≥ 3`), define the **input-determined valley set** (local minima of row index along the cycle, independent of any algorithm):

$$V(g) = \{ x \in [m] : x < g(x) \text{ and } x < g^{-1}(x) \}, \qquad \pi_{\mathrm{valley}}(g) = |V(g)|.$$

> **Project prior-art note (`R6` check per Coordinator Note #17):** Searching `origin/research/carbon-kitchen-sink` shows that cutting the `LF` cycle at local minima of row rank (= SA-IS LMS positions, Nong–Zhang–Chan 2009) and resolving inter-minimum segments in two ordered sweeps was **already proposed in Astra's Nov 18 investigation** (`docs/journal/2026-11-18-astra-investigation-1.md`, lines 575–685, *"Astra: cutting the LF cycle at local minima"*, alongside a 12-round move-structure version). Accordingly, **`Procedure C4` claims no `R6` novelty for the local-minimum / LMS arch object itself**; its contribution here is proving and measuring the exact `(Q, D, P, S)` cursor-vector trade-off (`Q(Acq) = 0` via `|Σ| + 1` `±1` boustrophedon cursors on raw `LF` vs `Q(Acq) ≥ h₁` once wide runs are contracted to `m₁₆` or `m₁`), explaining quantitatively why Astra's run-level variant required 12 rounds.

**Theorem 20.3 (`[proved]`):**
1. **Canonical unimodal tiling (`N1`):** Between any valley `v ∈ V(g)` and the next valley `v' ∈ V(g)` along `g`, the trajectory `v = x_0 → x_1 → … → p → y_1 → … → v'` is **strictly unimodal** — strictly ascending (`v < x_1 < … < p`) to a unique local maximum `p` (`p > g^{-1}(p)` and `p > g(p)`), then strictly descending (`p > y_1 > … > v'`). Every arch has length `≥ 2`, and the `|V(g)|` arches tile `[m]` bijectively.
   *Proof:* Any change from descending (`y > g(y)`) back to ascending (`z < g(z)`) before `v'` satisfies `w < g^{-1}(w)` and `w < g(w)` at the turning vertex `w`, placing `w ∈ V(g)`, contradiction. ∎
2. **Cost law on novel random content where runs fail (`N2`):** Under uniform random rank triples `(g^{-1}(x), x, g(x))`, `P(x ∈ V) = 1/3` (mean arch length **`3.000`**, strictly exceeding doubling's `2.000` and 1-sweep Gauss–Seidel's `e = 2.718`). On novel random hex (`σ = 16`, where BWT runs fail with `h₁ ≈ (15/16)² n = 0.879 n`), **`E[π_valley(LF)] ≈ n / 3 ≈ 0.379 · h₁ ≤ (1 − 1/2) · h₁`**!
3. **Pure-cursor acquisition with ZERO addressed reads (`N3` in Record 111's vector!):** On `(L, o)`, all `|V(LF)|` arches are acquired in **two boustrophedon cursor sweeps** with **`Q(Acq) = 0` and `D(Acq) = 0`** (`s = |Σ| + 1 ≤ 257` `±1`-monotone cursors):
   - *Sweep 1 (`x = 0 → n-1`, `|Σ|` `+1` forward cursors `y = LF(x)`):* contracts every descending leg `p → … → v'` (`y < x`).
   - *Sweep 2 (`x = n-1 → 0`, `|Σ|` `-1` backward cursors `y = LF(x)`):* contracts every ascending leg `v → … → p` (`y > x`) directly onto `p`'s already-built `(down_J[p], down_w[p])`!
   Unlike 2-sweep Gauss–Seidel (whose second sweep reads `J_1[J_1[x]]` at `Q = n` data-addressed locations), **both sweeps of `Procedure C4` read exclusively at `y = LF(x)` via `±1` sequential symbol cursors (`Q(Acq) = 0`)!**

### 3.2 Measured performance on novel random hex & on the 6 admitted blocks (`[measured]`, byte-exact)

- **Novel random hex (`n = 60,000`, `h₁ = 52,812 = 0.8802 n`):**
  - Two-Cursor Unimodal Arches: **`Q(Acq) = 0`**, **`Q(Run) = 19,306` links** (`3.108` rows/arch, **`0.366 × h₁ < 0.50 × h₁` even at `cap = 8` bytes!**). Meets `(N1)`, `(N2)`, and `(N3)` at `K = 3` on random content!
- **Six admitted Wikidata blocks (showing exact `(Q, D, P, S)` trade-off across raw `LF`, `m₁₆`, and `LIMIT = 1` `m₁ = h₁`):**

| Block | Raw `LF` 2-cursor arches (`Q(Acq)=0`, `/ h₁`) | `m₁₆` arches uncap (`ret/arch`, `/ h₁`) | `m₁₆` arches `cap=16B` (`/ h₁`) | **`LIMIT=1` (`m₁=h₁`) arches uncap (`/ h₁`)** | **`LIMIT=1` arches `cap=16B` (`/ h₁`)** | **`LIMIT=1` 1-Sw GS `step16` / `byte16` (`/ h₁`)** |
|---|---:|---|---:|---:|---:|---|
| `block-0.L` | 307,347 (`8.75×`) | `29,295` (`3.07`, `0.834×`) | `1.036×` | **`11,417` (`0.325×`)** | **`0.416×`** | **`0.361× / 0.461×`** |
| `block-2.L` | 309,494 (`9.03×`) | `37,021` (`3.03`, `1.081×`) | `1.337×` | **`11,128` (`0.325×`)** | **`0.436×`** | **`0.362× / 0.480×`** |
| `block-85.L` | 304,974 (`5.49×`) | `50,097` (`3.12`, `0.903×`) | `1.037×` | **`18,213` (`0.328×`)** | **`0.445×`** | **`0.363× / 0.482×`** |
| `block-143.L` | 313,735 (`4.32×`) | `63,997` (`3.08`, `0.882×`) | `1.091×` | **`23,695` (`0.326×`)** | **`0.429×`** | **`0.364× / 0.467×`** |
| `block-210.L` | 309,086 (`3.18×`) | `65,264` (`3.13`, `0.672×`) | `0.804×` | **`31,660` (`0.326×`)** | **`0.416×`** | **`0.363× / 0.453×`** |
| `block-207.L` | 316,820 (`5.08×`) | `44,401` (`3.06`, `0.711×`) | `0.852×` | **`20,395` (`0.327×`)** | **`0.437×`** | **`0.363× / 0.473×`** |

### 3.3 Exact accounting of where the cost goes (Record 111's non-relocation test `[proved]` + `[measured]`)

Why is neither `Procedure C3` nor `Procedure C4` a frozen-budget yes on Wikidata `𝓕`? Record 111's `(Q, D, P, S)` vector makes the exact trade-off transparent:
1. **On raw `L` (`Q(Acq) = 0` via `±1` cursors):** Both 1-sweep GS (`n/2.63 ≈ 342k`) and 2-cursor Unimodal Arches (`n/2.91 ≈ 310k`) beat `h₁` on pure random hex, **but lose to `h₁ = 34k–97k` on Wikidata** because `89–96%` of Wikidata lies in wide runs (`w ≥ 2`), which raw `L` sweeps do not contract first, AND storing per-row edges takes `S = Θ(n) ≫ openPeakScratch = 4 m₁₆`.
2. **After wide-run contraction (`m₁₆` or `LIMIT = 1` `m₁ = h₁`):**
   - Look at `LIMIT = 1` (`m₁ = h₁`, contracting *all* `w ≥ 2` runs into strips first): consumer links `Q(Run)` drop **below `0.50 · h₁` on all 6 blocks** — **`0.325–0.328 · h₁`** for Unimodal Arches (`0.416–0.445 · h₁` at `cap = 16B`) and **`0.361–0.364 · h₁`** (`0.453–0.482 · h₁` at `cap = 16B`) for **1-sweep Gauss–Seidel**!
   - **Where did the cost relocate?** Once wide runs are contracted to `[m₁]`, `R_{m₁}(s)` is **no longer `±1` cursor-monotone** per symbol: (a) contracting thin wide runs (`2 ≤ w ≤ 16`, `54k–124k` rows) into `R_{m₁}` requires Rauzy–Veech / strip chasing across `r - h₁ = 16k–36k` wide runs (`Q(Acq_RV) = Ω(h₁)` addressed steps), (b) each sweep on `[m₁]` pays **`Q(Sw) = m₁ = h₁` addressed reads** (`J[R_{m₁}(s)]`), making **`Q(Acq) + Q(Run) ≥ 1.36 · h₁ > h₁`**, and (c) storing word/strip descriptors alongside `J[]` inflates `S` above `4 · m₁₆`!

---

## 4. Obstruction & Near-Miss Table (All Six Required Fields)

| Field | Value |
|---|---|
| **Framework / Constructions** | One-pass non-walking copy-plan & macro-object acquisitions from run records: `C1` (`Φ`-sibling copy), `C2` (run-boundary attractors), `C3` (Gauss–Seidel sweep composition / `NEEDS M-17`), and `C4` (Two-Cursor Boustrophedon Unimodal Orbit Arches; local-min/LMS two-sweep object is prior art in Astra `2026-11-18-astra-investigation-1.md`, `R6` not claimed) (`N-20`) |
| **Route class** | Macro-edge / online-copy basis transformations over `ℐ_lenient(K)` and Record 111's `ℐ_vec(K, s_max, S_0)` |
| **Gate failed (where cost goes)** | **Copy plans (`C1, C2`):** `C1` saves `0%` of `J₁`; `C2` has `J₁ ⊆ \Gamma` (`100%`), `85.8–92.2%` of `h₁` in `len ≥ 3`, and `0.00%` within `≤ 1` run (`med = 173–312` runs). **Macro-edge / Arch sweeps (`C3, C4`):** on raw `L` (`Q(Acq)=0`), uncontracted wide runs inflate `Q(Run) = 3.18–10.02 × h₁` and `S = Θ(n)`; on contracted `m₁₆` (`K=2`), `Q(Run) = 0.762–1.197 × h₁ > 0.50 × h₁` and `R(s)` loses cursor monotonicity (`Q(Acq) = m₁₆`); on `LIMIT = 1` (`m₁ = h₁`), `Q(Run) = 0.325–0.482 × h₁ < 0.50 × h₁` succeeds on `Q(Run)` **by relocating `≥ h₁` addressed accesses into `Q(Acq)`** and extra passes/scratch `(P, S)` |
| **Witness** | Byte-exact verified in `construct_n20_copy_and_sweeps.py`: (1) Witness 20.1 (`n=31, p=0`, flip `(1,8,10)` reversing `"abc"` acyclicity with identical 2-step `J₁` neighbourhoods); (2) 60 KB novel random hex (`C4` 2-cursor arches: `Q(Acq)=0`, `Q(Run)=0.366·h₁` at `cap=8B`); (3) all 6 admitted blocks (`NEEDS M-17`: `Step-cap 16` 1-sweep `2.732–2.782 > 2.3` on `6/6` blocks, `Byte-cap 8` `2.056–2.286 < 2.3` on `6/6` blocks; `LIMIT=1` arches `0.325–0.328·h₁`) |
| **Scope** | Offline positional copy plans (global, `[proved]` by Witness 20.1); online `Φ`-run & run-boundary attractor copies (global on `J₁`, `[proved]`); 1- and 2-sweep Gauss–Seidel and Two-Cursor Unimodal Arch decompositions on raw `L`, `m₁₆`, and `m₁` across the 6 admitted blocks (`[measured]` + `[proved]` access counts) |
| **Escape condition** | Construct a contracted representation of wide runs (`size ≈ h₁`) whose induced successor map **preserves bucket/cursor monotonicity** (`Q(Acq) ≪ h₁` via `O(|Σ|)` cursors rather than `m₁` addressed reads), **and** separate thick-strip spelling IDs from inline thin bytes so word caps do not fracture macro-edges; at 10 workers, also address the non-emit pipeline ceiling of `bench-supplied-codes` (`0.988` whole-decoder) |
| **Reach** | **Exact (`[proved]`)** for offline copy blindness (Prop. 20.1), macro-edge dominance over online copy (Lemma 20.2), `Φ`/`\Gamma` zero-`J₁` saving, and `Q(Acq) = 0` two-cursor unimodal arch decomposition (Thm. 20.3); **measured exact (`[measured]`)** on all 6 admitted blocks for `C1–C4` and `NEEDS M-17` |
| **Strength** | **Proved** (Prop. 20.1, Lemma 20.2, Thm. 20.3) + **measured byte-exact** across all 6 admitted blocks and 60 KB synthetic random hex |
| **Transferable insight** | **Cursor monotonicity (`LF` is `+1` per symbol) and interval contraction (`R` collapses wide runs) are dual and mutually destructive:** raw `LF` permits `Q(Acq) = 0` pure-cursor composition (reaching `0.366 · h₁` via Two-Cursor Unimodal Arches on pure random hex!), whereas contracting wide runs (`Open`) destroys per-symbol monotonicity so subsequent composition sweeps pay `Q(Acq) = m` addressed accesses — and on `R`, macro-edges must keep thick-strip IDs out-of-line or an 8-byte cap loses `22–25%` of its chain length (`2.75 → 2.14` ret/link) |

---

## 5. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact raw `.L` columns on all 6 admitted blocks (`origin/data/admitted-bwt`) + 60 KB uniform hex validation.
2. **Model assumptions:** Evaluated simultaneously under Record 108's `ℐ_strict` / `ℐ_lenient(K)` and Record 111's `(Q, D, P, S)` cursor vector, plus both `Step-cap` and `Byte-cap` for `NEEDS M-17`.
3. **Method:** Constructive byte-exact implementations of `C1`–`C4` in `construct_n20_copy_and_sweeps.py` (`decoded == T_true` asserted on every run).
4. **Outcome:** Completes `N-20` constructively, discharges `NEEDS M-17` (confirming `Step-cap` transfers at `2.73–2.78 > 2.3` while exposing `Byte-cap 8`'s `< 2.3` thick-strip fracture), introduces **Two-Cursor Unimodal Orbit Arches** (`Q(Acq) = 0`, `3.00` rows/arch, `0.366 · h₁` on novel random hex; `0.325 · h₁` consumer links on `LIMIT = 1`), and pinpoints the exact cursor-monotonicity vs wide-contraction trade-off.
