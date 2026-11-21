# Carbon Kitchen Sink — Cycle 11: Does `0.3%` Directed FVS Escape Record 84? Non-Planar Diamond Braiding, Collective DAG Propagation (`FVS-Topo`), and Confrontation Against `mibz2 0.5.8`

**Date:** 2026-11-21
**Branch:** `research/carbon-kitchen-sink`
**Inputs:** [`2026-11-20-carbon-kitchen-sink-cycle9.md`](2026-11-20-carbon-kitchen-sink-cycle9.md) (`d04880a`), [`2026-11-20-carbon-kitchen-sink-cycle10.md`](2026-11-20-carbon-kitchen-sink-cycle10.md) (`7a0c76c`), Round 6 & Record 84 (`docs/journal/2026-11-19-gemini-neon/{README.md,response-6.md,round6_attainability.py}`), and `mibz2 0.5.8` native `src/open.rs` (`origin/master` `bb98c99`, `#668`, `#675`).
**Artefacts:** [`probe_cycle11.py`](2026-11-20-carbon-kitchen-sink-cycle11/probe_cycle11.py), [`probe_cycle11_results.json`](2026-11-20-carbon-kitchen-sink-cycle11/probe_cycle11_results.json).

---

## 0. The Central Question

Cycle 9 proved that removing a tiny **Directed Feedback Vertex Set** $F \subseteq [0, r)$ of **only `171–508` runs (`0.29%–0.39%` of $r$)** breaks *every* directed cycle of the run-overlap graph $G_{\mathrm{ov}}$, leaving a pure DAG $D = G_{\mathrm{ov}} \setminus F$. Meanwhile, Round 6 (`round6_attainability.py`) and **Record 84** showed that direct $\Phi$-copy replay fails because `85.3%–93.8%` of `F_self_ns` copy fragments deadlock on dependency cycles, and resolving those cycles via run potentials requires text-order run labels (`which already determine the text`).

**Cycle 11 confronts the synthesis directly:**
1. *If we resolve the `0.3%` FVS runs first, does the acyclic remainder $D = G_{\mathrm{ov}} \setminus F$ become deadlock-free under $\Phi$-copy replay?*
2. *What if instead of $\Phi$-copying, we resolve the FVS runs and then propagate the acyclic remainder $D$ **collectively in topological order** — does an exact schedule exist that escapes Record 84, or does Cycle 9's phantom-path duality + per-worker working-set contention against `mibz2 0.5.8` forbid it?*

We settle **both** halves with exact proofs, a minimal $n = 15$ binary counterexample, byte-exact implementations verified across `3,016` small checks + all 6 Wikidata blocks, and an exact working-set + operation audit against `mibz2 0.5.8` (`origin/master:src/open.rs`).

---

## 1. Refutation of FVS-Cut Direct $\Phi$-Copy Replay: The Non-Planar Diamond-Braid Theorem

Suppose an **omniscient free oracle** pre-fills into `out[0..n)` **both**:
- all $r$ run-head literals $H = \{\tau(d_u) : u \in [0, r)\}$, **and**
- all $W(F) = \sum_{u \in F} w_u$ scattered text positions $T_F = \{\tau(x) : x \in U_F\}$ of a Directed Feedback Vertex Set $F$ of $G_{\mathrm{ov}}$.

Partition the remaining positions $[0, n) \setminus (H \cup T_F)$ into maximal contiguous text intervals $\mathcal{J}_F = \{J_0, \dots, J_{M-1}\}$ (`DAG-excursion fragments`), and form the $\Phi$-copy dependency graph $G_\Phi(\mathcal{J}_F)$ with edge $J_a \to J_b$ whenever $\Phi(J_b) \cap J_a \ne \emptyset$.

### Theorem 11.1 (Automatic Intra-Fragment Duplicate-Freeness vs Non-Planar Diamond-Braid Deadlock — *proved*; *verified* on $n = 15$ minimal witness + 6 Wikidata blocks)

1. **Zero Self-Dependencies & Zero Repeated Runs per Fragment:** Because every $J_m \in \mathcal{J}_F$ is a contiguous $\mathrm{LF}$-trajectory inside the acyclic graph $D = G_{\mathrm{ov}} \setminus F$, **no BWT run $v \in [0, r)$ can appear twice in any $J_m$**. Hence $\mathcal{J}_F$ automatically satisfies Neon Round 6's `F_self_ns` duplicate-freeness condition (`self_deps == 0` identically, without ever reading `text_run[0..n)`!).
2. **Refutation of Acyclic $\Phi$-Replay (Non-Planar Diamond Braiding):** Acyclicity of horizontal $\mathrm{LF}$ transitions in $D = G_{\mathrm{ov}} \setminus F$ does **not** imply acyclicity of vertical $\Phi$ dependencies ($x - 1 \xrightarrow{\Phi} x$) in $G_\Phi(\mathcal{J}_F)$! Whenever $D$ contains a **rank-inverting diamond braid** — two directed paths $\gamma_1, \gamma_2 : u \rightsquigarrow v$ in $D$ starting at $x_1 < x_2$ in $I_u$ (because $\gamma_1, \gamma_2$ branch into intermediate runs of opposite lexicographic byte order) and landing at $y_1 = \mathrm{LF}^{|\gamma_1|}(x_1) > y_2 = \mathrm{LF}^{|\gamma_2|}(x_2)$ in $I_v$ — the excursions $J_1 \ni \tau(x_1), \tau(y_1)$ and $J_2 \ni \tau(x_2), \tau(y_2)$ satisfy $x_1 < x_2 \implies J_1 \xrightarrow{\Phi^+} J_2$ at run $u$ and $y_2 < y_1 \implies J_2 \xrightarrow{\Phi^+} J_1$ at run $v$, creating a **mutual $\Phi$-deadlock inside an acyclic overlap DAG**.
3. **Minimal Binary Counterexample ($n = 15, r = 4$, exhaustive minimum):**
   - Primitive word $T = \mathtt{000110010010011}$, BWT $L = \mathtt{1111\,00000\,11\,0000}$ ($r = 4$ runs, $p = 0$), FVS $F = \{1\}$ (`1` run).
   - Removing run $1$ leaves $D = G_{\mathrm{ov}} \setminus \{1\}$ **100% cycle-free**, with $7$ remainder bytes in $4$ fragments (`self_deps = 0`), yet **`4 / 4 = 100.0%` of the fragments deadlock in a $\Phi$-cycle (`orderable = 0`)**!

### Empirical Confrontation: Oracle FVS-Cut $\Phi$-Attainability on All 6 Wikidata Blocks (`probe_cycle11.py`)

| Block | Round 6 `F_self_ns` Deadlocked | `FVS_card` Runs | Remainder Bytes | Oracle FVS-Cut Fragments $M$ | Self-Deps | Orderable | **Still Deadlocked in $\Phi$-Cycles** |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `92.5%` | 172 (`0.34% r`) | 756,218 | 99,548 (`7.60 B`) | **0** | 27,240 | **72,308 (`72.64%`)** |
| `block-2.L` | `93.8%` | 171 (`0.31% r`) | 761,490 | 91,640 (`8.31 B`) | **0** | 22,161 | **69,479 (`75.82%`)** |
| `block-85.L` | `90.0%` | 250 (`0.31% r`) | 726,482 | 105,496 (`6.89 B`) | **0** | 37,785 | **67,711 (`64.18%`)** |
| `block-143.L` | `90.5%` | 310 (`0.29% r`) | 688,810 | 114,813 (`6.00 B`) | **0** | 39,600 | **75,213 (`65.51%`)** |
| `block-210.L` | `85.3%` | 508 (`0.39% r`) | 676,062 | 108,188 (`6.25 B`) | **0** | 39,312 | **68,876 (`63.66%`)** |
| `block-207.L` | `91.5%` | 279 (`0.32% r`) | 709,579 | 114,572 (`6.19 B`) | **0** | 40,680 | **73,892 (`64.49%`)** |

**Verdict on Direct $\Phi$-Replay:** Even with a zero-cost oracle handing us all $r$ head literals *and* all `80K–104K` scattered text positions of the FVS runs, **`63.7%–75.8%` of the acyclic DAG remainder still deadlocks under $\Phi$-copying** because forward diamonds in $G_{\mathrm{ov}} \setminus F$ braid vertical run ranks non-monotonically.

---

## 2. Escaping Record 84 via Collective Topological DAG Resolution (`FVS-Topo` & `FVS-Open`) — And Why Phantom-Path Duality + Contention Refute It Against `mibz2 0.5.8`

What if we abandon vertical $\Phi$-copies altogether and resolve $D = G_{\mathrm{ov}} \setminus F$ **along the DAG's own topological order**?

### Theorem 11.2 (Exact Record-84-Escaping Collective DAG Decoder `FVS-Topo` — *proved*; *verified* byte-exact on `3,016` small checks + all 6 `900 KB` Wikidata blocks)

Let $F \subseteq [0, r)$ be any Directed Feedback Vertex Set of $G_{\mathrm{ov}}$ containing $\mathrm{run}(p)$ (constructible in strict $O(r)$ time via iterative DFS back-edges `DFS-FVS`, or via Levy–Low contraction `FVS_wt`), and let $\mathrm{topo}(D)$ be a topological ordering of the DAG $D = G_{\mathrm{ov}} \setminus F$. Then exact reconstruction from $I_0$ **without text-order run labels (escaping Record 84)** is achieved in three loop-free/bounded phases:

1. **Phase 1A (Single-Pass Reverse-Topological Phantom Pruning on $D$):** Because $U_F$'s interior positions $\tau|_{U_F}$ depend on return lengths through $D$, and $D$ has $2^{34.9}\text{--}2^{62.7}$ empty combinatorial paths (Cycle 9), sweep $v \in \mathrm{reversed}(\mathrm{topo}(D))$ **once** (`0` iterative cycle loops), pulling back exact `(width, target_uf_row, step_len)` sub-intervals across `out_overlaps[v]` to obtain the exact return map $(T_{U_F}, \ell_{U_F})$ (`7,001–19,500` affine bands).
2. **Phase 1B ($U_F$ Cycle Walk):** Walk ONLY $U_F$ from $p$ (`79,932–95,295` steps on `FVS_wt`; `109,586–123,808` on `DFS-FVS`), assigning $\tau[x]$ and `out[\tau[x]] = L[x]` for $x \in U_F$ via $\tau[T_{U_F}(x)] = (\tau[x] - \ell_{U_F}(x)) \bmod n$.
3. **Phase 2 (Pointer-Free Collective Forward-Topological Slice Propagation on $D$):** Push contiguous slices $\tau[c_f + t] = \tau[d_f + t] - 1$ for $f \in F$, then sweep $v \in \mathrm{topo}(D)$ in topological order. Because every predecessor of $v$ lies in $F$ or earlier in $\mathrm{topo}(D)$, **`\tau[d_v .. d_v + w_v)` is guaranteed 100% populated upon reaching $v$**, resolving $v$ collectively via `out[\tau[d_v + t]] = b_v` and contiguous slice shift `\tau[c_v + t] = \tau[d_v + t] - 1` (**zero pointer chasing in Phase 2**; alternatively, `FVS-Open` spells excursion words in Phase 1A and emits sequentially in Phase 1B).

### Costed Confrontation Against Native `mibz2 0.5.8` (`origin/master:src/open.rs`)

Measuring `mibz2 0.5.8`'s exact `Open<16, true, true>` buffer & step counts alongside `FVS-Topo` and `FVS-Open` (`probe_cycle11.py`):

| Block | **`0.5.8` Thick Strips $H$** (`≤3H+2` steps) | **`FVS-DAG` Propagated Splits** (`Phantom Pruning`) | `0.5.8` Walk $m$ | `FVS_wt` / `DFS` Walk $W$ | **`0.5.8` Emit Hot WS** | **`0.5.8` Total Kernel WS** | **`FVS-Open` WS** | **`FVS-Topo` WS** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | **3,448** (`≤10.3K`) | **85,105** (`8.2×`) | 89,874 | 95,295 / 123,494 | **377.3 KiB** | **1,258.2 KiB** | 4,503 KiB (`3.6×`) | **6,954 KiB (`5.5×`)** |
| `block-2.L` | **3,827** (`≤11.5K`) | **100,586** (`8.8×`) | 112,070 | 81,043 / 122,445 | **473.9 KiB** | **1,454.2 KiB** | 4,703 KiB (`3.2×`) | **7,182 KiB (`4.9×`)** |
| `block-85.L` | **1,565** (`≤4.7K`) | **142,717** (`30.4×`) | 156,404 | 82,572 / 117,650 | **635.5 KiB** | **1,766.3 KiB** | 6,112 KiB (`3.5×`) | **8,587 KiB (`4.9×`)** |
| `block-143.L` | **5,047** (`≤15.1K`) | **204,193** (`13.5×`) | 196,841 | 86,992 / 122,982 | **833.5 KiB** | **2,434.6 KiB** | 7,944 KiB (`3.3×`) | **10,410 KiB (`4.3×`)** |
| `block-210.L` | **2,809** (`≤8.4K`) | **223,486** (`26.5×`) | 204,281 | 79,932 / 109,586 | **839.2 KiB** | **2,358.0 KiB** | 8,849 KiB (`3.8×`) | **11,329 KiB (`4.8×`)** |
| `block-207.L` | **3,474** (`≤10.4K`) | **165,781** (`15.9×`) | 135,921 | 82,372 / 123,808 | **572.7 KiB** | **1,701.9 KiB** | 6,685 KiB (`3.9×`) | **9,161 KiB (`5.4×`)** |

**Three Decisive Reasons `FVS-Topo` / `FVS-Open` Lose to `mibz2 0.5.8`:**
1. **Phantom-Path Duality Taxes Phase 1A by `8.2×–30.4×` Over `0.5.8` Elimination:** Because $2^{35}..2^{63}$ combinatorial DAG paths are empty metric phantoms, Phase 1A cannot skip the `50,869–127,893` mostly width-1 thin runs of $D$: it must materialise all $r$ runs (`649–1,739 µs` in C, Cycle 1) and propagate `85K–223K` interval sub-pieces, whereas `0.5.8` keeps thin runs ephemeral in registers and eliminates only $H \le 5,047$ thick strips (`bands = 2,446..6,788`, `words = 1,220..3,635`!).
2. **Phase 2 Scatter Writes vs `0.5.8` Sequential 16B Writes:** `FVS-Topo` Phase 2 converts sequential `out` emission into `800,000+` random `out[\tau[x]] = b_v` cache-line scatter writes (`99.6%` L1 miss rate, Cycle 10) — `4×–9×` more random cache misses than `0.5.8`'s entire retained walk!
3. **`3.2×–5.5×` Per-Worker Working-Set Inflation (Violating the Contention Rule):** `mibz2 0.5.8`'s total per-worker `Kernel` footprint is remarkably lean (`1,258–2,435 KiB`, with `emit` hot tables only **`377–839 KiB`**!). Both `FVS-Open` (`4.5–8.8 MiB`) and `FVS-Topo` (`7.0–11.3 MiB`) multiply per-worker cache pressure by `3.2×–5.5×`, which would severely degrade 6–10 thread throughput on the `20 MiB`-L3 i9.

---

## 3. Crucial Bonus Discovery Inside `mibz2 0.5.8` (Seeding Cycle 12!)

Look closely at the exact `native_058_open16` profile in `probe_cycle11_results.json`:
- `emit` hot tables (`edges` + `inline16` + `lens`) are only **`377–839 KiB`** (`bands = 29–80 KiB`, `inline16 = 25–65 KiB`, `spelling = 35–71 KiB`),
- **WHEREAS `queues: Vec<Vec<u32>>` ALONE CONSUMES `720–1,368 KiB` (`55.0%–58.6%` OF `0.5.8`'S ENTIRE PER-WORKER WORKING SET!!)** because each active byte's `Vec<u32>` starts at `1,024` `u32`s (`4 KiB`) and doubles on power-of-2 boundaries (`leaving 35–55% unused capacity + 85–96 separate heap allocations`!), **AND** `inline16` has **`1.79×–2.50×` duplicate words** (`words = 1,220..3,635` vs `unique = 555..1,455`)!

In **Cycle 12**, we investigate **Bijective In-Place / Zero-Queue / Sub-Working-Set Algebraic Permutations & Exact Quotiented `Open` Constructions** — asking whether mathematics can **strictly shrink** `mibz2 0.5.8`'s `1.26–2.43 MiB` per-worker working set (eliminating or collapsing the `720–1,368 KiB` `queues` buffer *without* Neon Cycle 2's slow second record pass!) to attack the i9's 6–10 worker cache-contention bottleneck directly!
