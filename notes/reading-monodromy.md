# Reading notes on MONODROMY (`claude/sharp-meitner-cx0v78`)

## Cycle 1 read — 2026-11-24T10:38Z

Fetched `origin/claude/sharp-meitner-cx0v78` through HEAD `6737734cf49aec217f5c03a4fdc2bda578f65fa7`.

### Commits and files read

1. **`c191322` — Record 96** (`docs/journal/monodromy/2026-11-24-guid-wall.md`):
   - Proves Theorem G1 on rigid jump-walk routes `𝓡`: `D_width-1 ≥ h₁` ("for every column").
   - Proves G2 (`P(width-1) = (15/16)^2 = 0.879`) and G3 (expected `k`-key partners `≤ (N-1)/16^k`) under model I (independent uniform random nibbles). Measured on the six admitted blocks (`0.876–0.893`).
   - Introduces record-local hex proxy `h₁^hex`: recall `0.923–0.929`, precision `0.31–0.73` for GUID width-1 rows.

2. **`eae9ca9` — Record 98** (`docs/journal/monodromy/2026-11-24-fewer-than-h1.md`):
   - Formalises `𝓡` as a calculus of rigid facts `(I, h, t, w)` with rules `prim`, `mirror`, `restrict`, `compose`, `join`.
   - Defines `c₁(D)` as width-1 events anywhere in the program (rule applications with width-1 subject + width-1 output chain links).
   - Repairs "fewer than h₁" to a count (not depth), with streaming pass budget `K` and fixed family `𝓕`.
   - Identifies E2's global, non-rigid, non-O(1)-depth cell within `K` passes on non-random content as the sole open escape.

3. **`a7c2e4c` — Record 99 & `main-theorem.md` & `CLAIMS.md`** (`docs/journal/monodromy/2026-11-24-main-theorem.md`, `docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md`, `CLAIMS.md`):
   - Self-corrects Record 98: puts E3 (list ranking / pointer jumping / contraction) inside `𝓡` (`c₁ = Θ(n log n)` / `Θ(n)`), and closes E1 (class facts) as a non-escape because emitting order still requires a chain or E2.
   - Places Rauzy–Veech inside `𝓡` in one currency (`c₁`).
   - Proposes margin `δ` (`c₁ ≤ (1 - δ) · h₁`, provisional `δ = 1/2`), noting `origPtr` already saves 1 event.
   - Drafts `theorem main` in pseudo-Lean (`PROOF-PROGRAMME` §5.11 candidate) with `P0`, `P1` (additivity `c₁_split`), `PN`, `PR`, `Pglue`, `Pbudget`, `Pmargin`.
   - Claims `M-1` (main theorem, PUSHED), `M-2` (stream R, PUSHED), and posts `NEEDS M-2` requesting `h₁R / h₁` under `σ_hex` and census cost weights on `𝓕`.

4. **`6737734` — Record 100 (`M-3`, DONE)** (`docs/journal/monodromy/2026-11-24-positional-facts.md`):
   - Repairs `c₁` to charge single-row facts of *any* fact type in a declared language `Λ ⊇ 𝓡`, preventing trivial renaming ("pos(x) = k" one row at a time).
   - Defines an escape as a sound multi-row fact type `Φ` (subject size `≥ 2`) outside `𝓡`.
   - Closes locally keyed multi-row facts on model-I rows (combining G3 and 3-byte global sensitivity) and shows the full random-content conjecture is model-relative to a declared `Λ`.

### Immediate coordination & adversarial targets spotted

- **Do not overlap** `M-1` (main theorem ownership) or `M-2` (stream R theoretical lower bounds).
- **Support `NEEDS M-2`**: measure `h₁R / h₁` under `σ_hex` on the six admitted blocks (raw `.L` available on `data/admitted-bwt`) and compute cost-weighted `h₁` shares across all 232 census blocks (`#805` `census.tsv`/`features.tsv`), flagging that raw `.L` bytes for the 232 census blocks are not present in the repository (`corpora/wikidata.bz2` absent) so `h₁^hex` on the full 232 requires a corpus run.
- **Adversarial audit targets (`N-1`)**:
  1. `theorem G1` in `main-theorem.md` (`a7c2e4c` L49), Record 98 (`eae9ca9` L56), and Record 96 (`c191322` L48) fails on arbitrary multi-cycle `Column` (counterexample `L = "aab"`, `p = 0`: `h₁ = 1`, `c₁ = 0`).
  2. `theorem main` (`a7c2e4c`, `main-theorem.md` L56–65) is not provable from its stated hypotheses: `Pmargin` omits `c₁(Pglue)` (contradicting §4.5 L109), and no premise bounds `state` to prove `D.state c ∈ O(c.r)`.
  3. `c₁_split` (P1) and `c₁(Open)` (`eae9ca9` L52; `a7c2e4c` L52–53, L71–85): under `c₁` counting all width-1 events (including width-1 facts on rows of width `≥ 2` runs), `σ_hex` only partitions width-1 runs, leaving width-1 events on non-width-1 runs unattributed; and shipped Open expands every retained row `s ∈ S` (width `2..16`), so either `c₁(Open) ≥ m ≈ 2.56 h₁` or else (if chain links may cite width-`≥ 2` facts without restriction) the plain LF walk achieves `c₁(LF) = h₁ ≤ c₁(Open)`.
  4. Record 100 (`6737734` L40): `aR ≥ 0.996` is an expectation under model I for `k ≥ 6`, not a deterministic per-block bound on `𝓕`, and needs explicit handling of `k ≤ 5` chain links.

## Cycle 1 second read — 2026-11-24T10:45Z

Fetched `origin/claude/sharp-meitner-cx0v78` commits `4d7d83b`, `a96e969`, `dd1d6ea`:

1. **`a96e969` — `channel/coordinator-to-neon.md`:**
   - Requests `N-0`: measure `h₁R / h₁` under `σ_hex` across `𝓕` (per block + cost-weighted aggregate) and state whether `δ = 1/2` is reachable without stream R beating `aR = 1`.
   - Note: we already computed this in `N-2` (`656d5e7`): exact on the 6 admitted blocks (`0.7978–0.8514`, cost-weighted `0.8256`), with `#805` 232-block census weights computed and `corpora/wikidata.bz2` noted absent locally for per-byte `h₁R` on the remaining 226 blocks. Verdict: **`δ = 1/2` is unreachable without `aR < 1`** (even under an oracle GUID/non-GUID split, since `block-0.L` has 66.6% GUID width-1 runs).
   - Directs Neon & MONODROMY to fix `δ, K, 𝓕, S` under the stricter-wins rule; `Pglue` is fair game for Neon.

2. **`4d7d83b` — Frozen parameters in `main-theorem.md` §1 & §1a:**
   - `K = 2` (stricter than provisional `K = 3`), `δ = 1/2`, `S(c) ≤ openPeakScratch c`, plus anti-cheat clauses 1–5 (`c₁` charges every fact type; no bundling; no off-ledger cost; depth ≠ count; never relax under C2).
   - **Adopted under stricter-wins:** all Neon E2 screens (`N-3`..`N-7` and subsequent cycles) now screen against **`K = 2`** (while also recording `K = 3` where relevant so no result is ambiguous), **`δ = 1/2`**, **`S ≤ openPeakScratch`**, and anti-cheat clauses 1–5.

3. **`dd1d6ea` — `docs/journal/monodromy/HANDOFF-2026-11-24.md`:**
   - Notes MONODROMY's next item `M-4` (swap-invariance lemma for aggregate-only global rules on stream R) and confirms our `N-2` finding on `Pglue` (MONODROMY estimates ~10 switches per GUID ≈ 1/3 of `h₁R`; our `N-2` measured exact `all_sw / h₁ = 0.513–0.543` and `w1_switches / h₁N = 1.19–1.63` on all six admitted blocks!).
   - Also notes that anti-cheat clause 2 ("distinguished input") is informal and collapses for general global rules — consistent with our `N-1` audit.

## Cycle 2 read — 2026-11-24T10:54Z

Fetched `origin/claude/sharp-meitner-cx0v78` through HEAD `d0d0232` (`9e56347`, `908d292`, `d0d0232`):

1. **`9e56347` — Record 101 (`M-4a`, `M-5`):**
   - Proves the **Swap-Invariance Lemma** (`docs/journal/monodromy/2026-11-24-swap-invariance.md`): a valid count-preserving flip is a Hamming-distance-2 byte transposition `i < j` with one intervening `a`-or-`b` row `m`, giving `σ' = σ ∘ (3-cycle)` valid iff orbit order matches, exchanging two adjacent text blocks. Lemma 1: `pos'(x) = pos(x)` iff `x` lies in `origPtr`'s arc.
   - **Theorem 3 (`Λ_agg(𝒫, D)`):** any sound coarse-aggregate rule placing `x` needs `|𝒫| + 2|D| > ν′(x)`. Requests **`NEEDS M-4`** (measure valid flip coverage, greedy disjoint-span `ν′/n`, `|U|`, and orbit cut-point gaps on the 6 admitted blocks).
   - **M-5 (`Pglue` floor):** `c₁ ≥ seg_R(c) - |U|` over `𝓡 ∪ local ∪ Λ_agg`; predicts `aR ≥ ~0.29` on GUIDs from glue alone.

2. **`908d292` — `channel/coordinator-to-neon.md` entry #2 (interim outside review):**
   - Amends `N-0`: report `h₁` in 3 parts per block (**novel hex**, **repeated hex**, **non-hex**) with kernel-time-weighted aggregate.
   - Adds §1a clause 6 (adopted stricter by MONODROMY): a yes must state dependent loads per block `d(D, c) ≤ d(Open, c)`.
   - Flags two logical gaps in `main-theorem.md` (already caught in `N-1` and repaired in `M-6`).
   - Prioritises screening **positional copy schemes** (r-index `φ`/toeholds) and **compressed suffix coordinates** (sampled SA, r-index) before the remaining kitchen sink.

3. **`d0d0232` — Record 102 (`M-6`, `M-7`, repaired `main-theorem.md`):**
   - Accepts all 4 of Neon's `N-1` breaks (`8ec0f0b`) and repairs `main-theorem.md` (`singleCycle` in `G1`, `c₁(Pglue)` in `Pmargin`, `Pstate`, run-based attribution for `Mirror`). Withdraws `NEEDS M-5` as answered by `N-2` (`656d5e7`).
   - Proves **M-7 (repeats-are-wide lemma)** under model I: at most `log₁₆ N + 16/15 + 1 ≈ 6–7` width-1 rows per repeated hex occurrence. Argues positional copy schemes and r-index `φ` do not enter the open cell on novel content.

### Cycle 2 coordination & adversarial targets spotted

- **Adopt §1a clause 6** (`d(D, c) ≤ d(Open, c)` per block) under stricter-wins.
- **Do not duplicate `M-4a` or `M-7`:** fulfill `NEEDS M-4` directly (and test whether `ν′(x) = Θ(n)` extends `Λ_agg`'s closure to Stream N!), fulfill amended 3-part `N-0` + test `M-7`'s `≤ 7` prediction empirically, and audit Record 101 Theorem 3's run-record blindness condition (`L = "caab" ↔ "cbaa"`, `D = {2}` breaks `i, j ∉ D` when `D` exposes full run records rather than raw bytes).

## Cycle 3 read — 2026-11-24T11:03Z

Fetched `origin/claude/sharp-meitner-cx0v78` commits `2001f12` and `e48f904`:

1. **`2001f12` — Record 102 §3 & `main-theorem.md` (`c₁` convention fixed):**
   - Adopts Neon's stricter option (ii) from `N-1` break 3.3: `c₁` counts every 1-row-subject event in runs of *any* width, and links always count (`c₁(LF walk) = n`, `c₁(Open) ≥ m`, Record 98 L52 superseded).
   - Makes `σ` three-way `(R, N, W)` and adds `c₁W c` to `Pmargin`.
   - *Minor audit note:* `main-theorem.md` L82 introduces `c₁W` as an unbound symbol in `theorem main` (needs a `DW` / `W`-stream premise or definition) and L84 still prints `D.state c ∈ O(c.r)` instead of `≤ S c`.

2. **`e48f904` — `channel/coordinator-to-neon.md` entry #3 & `channel/outside-review-2026-11-24.md` (full ChatGPT Pro review):**
   - Recommends keeping `c₁` as rigid-calculus diagnostic while adding a dependent-miss conjunct `D_𝓗` (matching §1a clause 6), replacing finite `𝓕` with a uniform structural family (to exclude `𝓕`-indexed lookup advice), defining what `K` counts (working-array scans/sorts), and fixing the "no" to cover global E2.
   - Directs Stream N screening of **three specific supplied capabilities** (measuring descriptor/anchor/exception counts on expensive rows `h₁` and `m₁₆` and full-path payoff *before* acquisition work):
     1. Acyclic positional copy plans (LZ77 / Kempa–Prezza attractors and macro schemes, `q_copy`);
     2. Compressed `SA`/`ISA` differences (`DSA`) with paid anchors (`g_{Δ,N} + s_N`, Kempa Thm 5.1 audit);
     3. Ordered return-word hierarchies with paid exceptions (`(s, d, e)`, Durand / S-adic).
   - *Peer-claim check:* our `N-8` (`8756c55`) directly rebuttals the review's concern (§4 L134–136) that `σ_hex` mixes in large repeated SHA1 width-1 content: repeated 40-hex SHA1s contribute only `3 / 145,800 B` (`0.00–0.01%` of `h₁`), and novel hex alone is `61.3–68.2%` of `h₁`. Its Model-I run-length sampling bias catch (L141: size-biased `k(1-p)²p^{k-1}` vs geometric `(1-p)p^{k-1}`) is mathematically exact (Record 96 L74 mixed them for `w ≥ 2`, while `w = 1` was right).

### Cycle 3 second read — 2026-11-24T11:08Z

Fetched `origin/claude/sharp-meitner-cx0v78` commit `620eacc` (Record 103, `M-9`):
- Audited `M-9` scoped provenance proof of Theorem G1 (`docs/journal/monodromy/2026-11-24-g1-scoped.md`): **verified, 0 breaks found** (Lemma A, Corollary `|I| = 1`, DAG induction on `P(f, y, j)` is row-preserving and handles `prim`/`restrict`/`join`/`compose` cleanly).
- Adopted §1a clauses **6b** (`D_𝓗(D, c) ≤ D_𝓗(Open, c)`) and **7** (no advice; uniform family `𝓖(S, m, k)` of Record 103 §4, validated on `𝓕`) under stricter-wins.
- Noted `main-theorem.md` minor polish items (`c₁W` unbound in `theorem main` L82, `D.state c ∈ O(c.r)` in L84) and that `N-9` (`4ab193c`) already discharged `NEEDS M-4` referenced in Record 103 §4.

## Cycle 4 read — 2026-11-24T12:30Z

Fetched `origin/claude/sharp-meitner-cx0v78` through HEAD `66a05387` (commits `d67c22d`, `629cf60`, `c47a2a1`, `c03c7ab`, `aea0149`, `78abe2c`, `ad4f555`, `bc1429e`, `a91788e`, `66a0538`).

### Commits and files read

1. **Coordinator entries #4, #5, #6, #7, #8 (`channel/coordinator-to-neon.md`) & `channel/outside-review-round2-2026-11-24.md` (`78abe2c`, `a91788e`, `66a0538`):**
   - **#6 correction to #4:** `toy-transfer` L187–208 shows 8 lanes + branchless 16-byte stores cut Open's emit to `0.55–0.78×`. The blanket "work-bound, not latency-bound" claim is withdrawn; whole-decoder time decides utility.
   - **Native timing experiment (`bench-supplied-codes`):** owned by a separate workstation recipe, NOT Neon. Entry #5's work-per-event model fit is dropped.
   - **Round-2 audit breaks against Neon:**
     - **N-12 (`screen_cycle3_supplied.py`):** passes cyclic-rotation order to non-cyclic Kasai `LCP`/`LPF` (witness `L = baaaa`, `p = 2`: true `LPF = [0,1,0,2,1]`, script `[0,1,0,1,1]`). Assigned as **N-16**.
     - **N-13:** distinctness and `H₀` lower-bound literal-terminal representations only, not arithmetic descriptions (witness `T = [0,2,4,6,8,7,5,3,1]`, `SA[2j]=j`, `SA[2j+1]=8-j`). Assigned as **N-17**.
     - **N-14:** tested one comma split (`","`) and one 4-regex hole mask, not every return-word hierarchy. Assigned as **N-17**.
     - **N-9 / §III.6:** `U_all-flips ⊆ U(𝒫, D)` is a lower bound, not an upper bound, on observation-relative `U(𝒫, D)`; report threshold distribution `#{x ∈ J₁ : ν̂(x) ≤ q}` on the 6 blocks; and test complete-observation fibres for small `n` across raw bytes on `D`, `(H, A)` with `A = Σ i·code(L[i])`, and run records with LF translations. Assigned as **N-18**.
   - **#8 amendment:** MONODROMY's Record 107 withdrew Theorem N and `Coll_k`, replacing them with Theorem N′. N-19 is retargeted at MONODROMY's actual `ASK N` against Theorem N′'s output model.

2. **MONODROMY Records 104 (`c47a2a1`), 105 (`c03c7ab`), 106 (`aea0149`), and 107 (`ad4f555`, `bc1429e`):**
   - **Record 104 (`M-10`, `M-11`):** ordered-frontier scalar implementation is in `𝓡` (`c₁ = G·d` unchanged; packed/implicit transport unplaced); walk → Cells → Open is contraction along `compose` preserving `J₁`.
   - **Record 105 (`M-12`):** `G1` survives DAG abbreviation and batching; block CRC accept-on-match is inadmissible by exactness (`T1`).
   - **Record 106 (`M-13`, later withdrawn in 107):** proposed Theorem N over `Λ*` with `Coll_k` and `U(𝒫, D)`; adopted 4 process rules including **transferable insight** on every obstruction.
   - **Record 107 (`BREAK M-14`, `DONE M-14`):** accepted all five round-2 breaks (withdrew Theorem N and `Coll_k`; `NEEDS M-13` withdrawn; `NEEDS M-4` held until `𝒪` is fixed; Record 101 blindness updated to `D ∩ {i, m, j} = ∅`, `|𝒫| + 3|D| > ν′`; 104 price prediction and 105 proof-size criterion withdrawn). Replaced Theorem N with **Theorem N′** over `Λ′` (bytes written only by rigid facts of `𝓡`, plus positional labels `P₁` charged / `P_𝒪` uncharged on start rows; class facts E1 outside `Λ′`): `c₁ ≥ h₁ − |B|`. Notes `B` is unbounded by swap-invariance for order-sensitive `𝒪` such as `A = Σ i·code(L[i])`.
   - **Active MONODROMY claim (DO NOT TOUCH):** `STARTED M-15` — fix clause 2's observation language `𝒪`, then the admission form's lowering for a packed frontier.

## Cycle 5 read — 2026-11-24T16:42Z

Fetched `origin/claude/sharp-meitner-cx0v78` through HEAD `f86a4d3b` (commits `8e9b962`, `79591f0`, `cdeef65`, `18b674d`, `299a062`, `56d2c3a`, `c4847b8`, `7e10769`, `8cc654c`, `f86a4d3`).

### Commits and files read

1. **MONODROMY Record 108 (`8e9b962`, `M-15` DONE — `docs/journal/monodromy/2026-11-24-charge-convention.md`):**
   - Accepts Neon's Lemmas 18.1 (exact fibre floor `c₁ ≥ h₁ − |J₁ ∩ U_𝒪|` for every `𝒪`, correcting Record 107 §3), 18.2 (`1|D|` / `2|D|` sharp disjoint-span coefficients), Proposition 19.1, the 6-block `q₅₀ = 6,903–14,120` curve, and the three `Λ′`-neighbourhood decoders (`fill_remaining`, `copy`, `E1`).
   - Breaks own Record 102 §2.4 ("copy cannot touch novel random nibbles"): short copies (`min_len = 2`) save pointers (`z/h₁ = 0.54–0.64`), not entropy.
   - Proves "aggregate" has no fixed observation language; a charge exists only relative to a **declared instruction set**.
   - Establishes the pass-charging dichotomy: strict charging (`1` per row whose record a build pass reads for row routing) makes the frozen target "no" by definition; lenient charging (free streaming pass) makes `K` decide (`K ≥ 3` doubling gives a count-yes; `K = 2` gives `T²` at `m/2 ≥ 0.60·h₁`).
   - Operationalises §1a clause 2 under the strict reading (pure folds to `O(1)` words free; per-row record reads charged) and opens **Residual (c)** (*instruction count with supplied plans; charged cost of one-pass acquisition*) alongside `ASK N (1)` (one-pass non-walking copy-plan acquisition) and `ASK N (2)` (`U_𝒪` and a balanced multi-flip witness `ΔA = ΔA₂ = 0` for `𝒪 = (H, A, A₂)` on one admitted block).

2. **Coordinator Notes #9–#11 & MONODROMY Record 109 (`79591f0`, `cdeef65`, `18b674d`, `299a062`, `56d2c3a`):**
   - **#9 (`bench-supplied-codes` diagnostic, `main-theorem.md` §7):** supplying Open's word codes free in output order halves the emitter (`0.530` at 1W, `0.552` at 10W), giving `0.838` whole-decoder at 1W, **but `0.988 [0.957, 1.020]` at 10W (STOP)** — at 10 workers whole-decoder wall time is bounded outside the kernel (serial front end or shared memory bandwidth).
   - **#10 & Record 109 (`M-16` DONE — `docs/journal/monodromy/2026-11-24-supported-rows.md`):** answers the MST "supported edge" question: inversion's output is a bijection (`m = n` vs `m ≫ n`), so no width-1 row can be filtered out in `𝓡` (`G1`); existing certificates are wide-row adjacency (Open's own contraction, helping only `c₁W`), histogram forcing (`≤ 6.8%` of `h₁`), and observation labels (`≥ 10⁴` cells).
   - **#11 (`channel/packed-inversion-sweep-2026-11-24.md`):** no `o(n)` packed word-RAM BWT inversion algorithm or lower bound is known in the literature; KKP 2012 `Mtl-sa` and `libsais` 2.12 use `k = 2` bigram `LF²`.

3. **Coordinator Notes #12, #13, #14 & MONODROMY Record 110 (`c4847b8`, `7e10769`, `8cc654c`, `f86a4d3`):**
   - **#12:** assigns Cycle 5 items **N-20** (one-pass non-walking copy-plan acquisition, instruction set stated), **N-21** (`𝒪 = (H, A, A₂)` balanced multi-flip witness + `U_𝒪` on one admitted block), and **N-22** (self-contained `research/neon/scripts/census_extension.py` reproducing all published 6-block numbers for `N-0`, `M-4`, `M-8`, not run on corpus).
   - **#13 & #14 (re-prioritisation & Louis's 3-axiom basis framing):** directs executing **N-20 first** as a genuine *construction* attempt (deferring N-21 and N-22 until N-20 has a pushed result!), building concrete run-record procedures on the 6 admitted blocks and counting instructions against `h₁` and `T²` (`m/2`), **plus** proposing and testing at least one new post-interval basis object satisfying: (1) acts on a new object (neither rows nor row intervals), (2) cost law small on novel random content where runs fail, (3) tested for non-walking acquisition from run records.
   - **Record 110 (`M-17` DONE — `docs/journal/monodromy/2026-11-24-sweep-composition.md`):** constructs in-place ("Gauss–Seidel") row-order sweep composition yielding macro-edges of theoretical mean length `e ≈ 2.718` in 1 sweep (`e²` in 2 sweeps) under exchangeable ranks (`2.63–2.76` on synthetic texts), and posts **`NEEDS M-17`** on the 6 admitted blocks: run sweep composition `(i)` on `LF` over all rows and `(ii)` on Open's retained map (`1` ascending sweep then `1` descending sweep, word caps `8` and `16` bytes), with MONODROMY's pre-fixed withdrawal threshold `< 2.3` retained rows/link after 1 sweep on `≥ 4` blocks.
   - **Record 111 (`303b48c8f`, `M-18` DONE — `docs/journal/monodromy/2026-11-24-council-formulation.md`):** formalises a 4-coordinate machine cost vector `(Q, D, P, S)` separating **streamed accesses** (`±1` cursors, `s ≤ s_max`, recommended `|Σ| + O(1)`) from **addressed accesses** (`Q` count, `D` dependent-chain depth), with budget `𝔟 = (K, s_max, S₀)` and formal basis conditions `(N1)` new description class, `(N2)` input-defined parameter `π(L) ≤ (1 − δ)·h₁(L)` governing `Q(Acq) + Q(Run)`, and `(N3)` `Q(Acq), D(Acq) ≤ C·π(L)` (allowing cursor scans of `L`, which puts walk → Rauzy–Veech at `Q(Acq) = 0`). Proves `R3 ∧ R4` is unsatisfiable under Record 108's strict pass-charging convention and recommends the `(Q, D, P, S)` vector to the Coordinator.
   - **MONODROMY active `STARTED` check:** none (`M-15`..`M-18` all `DONE`).

### Cycle 5 second read — 2026-11-24T17:44Z (Coordinator Note #15, Carbon Cycle 1, Council)

Fetched `origin/claude/sharp-meitner-cx0v78` `a3cf329c`, `origin/research/carbon-new-basis` `aee62455`, `origin/research/council-neon` `4e56a735`, and `origin/docs/journal-algorithm-level` `6479feeb`:

1. **Coordinator Note #15 (`a3cf329c`):** No change of assignment; directs reading Carbon's Cycle 1 (`C-1`..`C-4`) and the Council formulations before our next result.
2. **Carbon Cycle 1 (`origin/research/carbon-new-basis` `aee62455`, `CLAIMS.md` & `research/carbon/c1..c4-*.md`):**
   - **`C-1` (Bipartite Boundary-Return Chords `B_L = LF(Thick_L) ∩ Thin_L`):** exact converged chords have `χ₂/h₁ = 0.229–0.320`, `χ₄/h₁ = 0.260–0.424`, `χ₁₆/h₁ = 0.367–0.652`. In 1 pure-cursor sweep (`Q_acq,thin = 0, L=2`), `Q_run/h₁ = 0.793–0.911` (`> 0.50`); reaching `< 0.44·h₁` requires addressed DAG sweeps (`(Q_acq,thin + Q_run)/h₁ = 1.34–1.46`) plus `Q_acq,thick` across `130k–223k` bands at `L = 2`.
   - **`C-2` (Radix-Fibre `k`-Cones):** uses `σ_R^k + 256` `±1` cursors (`Q_acq,thin = 0`) at `L = 16`, giving 2-sweep `Q_run/h₁ = 0.720–1.149` (`0.72–0.96·h₁` on 5/6 blocks, `> 0.50` on all 6).
   - **`C-3` (Polychromatic Quotient Slabs + SWAR nibble-rank):** merges `[0-9a-fA-F-]` runs into `r_π = 0.307–0.443·h₁` slabs with `Q_acq = 0`, and proves **Theorem QS-1**: height-`1` objects emit 1 byte per use, so `Q_run,thin = m₁₆ = 2.00·T² > h₁` (`h > 1` is necessary to lower `Q_run`).
   - **`C-4` (De Bruijn Unitig–Interval Duality `𝒰_8 ⊔ ℐ_8^{≥2}`):** synthesises sorted `8`-mers `u_8[0..n)` in `7` `256`-cursor passes (`Q = 0`). At `L = 1`, `u8_L1_cursor2sw` achieves `Q_run/h₁ = 0.522–0.555` with `Q_acq,unitig = 0` on the unitig side, **but** peer-audit note: (a) `7 + 2 = 9` cursor passes over `u_8` (`P ≈ 144 > K = 2`), and (b) `L = 1` incurs **`Q_acq,thick = 2.9M–5.2M` addressed probes** in `THICK_OVERLAP_PULL` across `250k+` fragmented bands (`C-4` §4.2) — independently confirming our `N-20` §3.3 duality wall between cursor monotonicity on `L` and interval contraction of wide runs!
3. **Council Formulations (`council/neon.md` `4e56a735`, `INDEX.md` `6479feeb`, alongside MONODROMY Record 111 `303b48c8`):**
   - Both MONODROMY (`(Q, D, P, S)` + `s ≤ |Σ| + O(1)` cursors) and `council/neon.md` (separating `W_seq` cursor-monotone work from `W_dep` data-addressed work, plus explicit `Q_ℋ, D_ℋ` under `N-15`'s i9 `𝓗`) resolve Record 108's pass-charging dichotomy identically: bulk cursor sweeps cost sequential bandwidth `P` / `W_seq` (`Q = 0`), while pointer-jumping/walks cost addressed `Q` / `W_dep`, making cost relocation across coordinates explicit.

