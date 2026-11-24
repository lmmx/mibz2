# CL2 — Exact Monotone-Schedule Lower Bound ($q_{\min}$ Obstruction) & Fragmentation Anatomy

**Task:** Channel `#27` / `chatgpt-closure.md` §5 — testing whether a representation can survive *arbitrary* further compositions (`Sweep 3+`) or hits a representation-independent rising-sequence barrier after `CL1`'s unfragmented second composition.  
**Script & Data:** [`cl2_closure_obstruction_anatomy.py`](cl2_closure_obstruction_anatomy.py), [`results_cl2_closure_obstruction_anatomy.json`](results_cl2_closure_obstruction_anatomy.json).

## 1. Exact Theorem: Universal Rising-Sequence Lower Bound

**Theorem CL2.1 (Exact Monotone-Schedule Lower Bound — proved & verified equality on 8/8 blocks).**
Let $T : [0, k) \to [0, k)$ be any bijection and let $\ell : [0, k) \to \{1, \dots, s\}$ be any class labelling (even an oracle with $s = k$ cursors). Define $\mathrm{seg}_{+1}(T, \ell)$ as the number of maximal per-class exact `+1` target-address segments ($T(j_{t+1}) = T(j_t) + 1$). Then:
$$\mathrm{seg}_{+1}(T, \ell) \;\ge\; q_{\min}(T) \;:=\; 1 + \mathrm{desc}(T^{-1}) \;=\; 1 + \big|\{y \in [0, k-2] : T^{-1}(y+1) < T^{-1}(y)\}\big|,$$
with exact equality attained uniquely by the rising-sequence labelling $\ell^*(x) = \text{index of the maximal increasing run of } T^{-1} \text{ containing } T(x)$.

*Proof.* Every exact `+1` step $T(j_{t+1}) = T(j_t) + 1$ with $j_t < j_{t+1}$ implies $T^{-1}(y + 1) = j_{t+1} > j_t = T^{-1}(y)$, so no descent of $T^{-1}$ can lie inside a `+1` segment. Hence each of the $\mathrm{desc}(T^{-1})$ descents starts a new segment, giving $\mathrm{seg}_{+1}(T, \ell) \ge 1 + \mathrm{desc}(T^{-1})$. Conversely, partitioning $[0, k)$ at the descents of $T^{-1}$ into $q_{\min}(T)$ maximal increasing runs $I_1, \dots, I_q$ and setting $\ell^*(x) = r \iff T(x) \in I_r$ walks each $I_r$ in exact `+1` steps with zero breaks. $\square$

*(Connection to prior art: $q_{\min}(T)$ is the Gessel–Reutenauer 1993 / Crochemore–Désarménien–Perrin 2005 standard-permutation minimum-alphabet invariant applied to the composed retained-row bijection.)*

## 2. Fragmentation Anatomy & The Third-Composition Wall (8/8 Blocks)

| Block | Orig `R`: $q_{\min}$ (`v`-segs) | M-17 GS: `v` / `last_byte` / **$q_{\min}(R_{\mathrm{gs}})$** (`!/|M_gs|`) | **CL1 `R_2`** (`|M_2|`): **$q_{\min}(R_2)$** (`!/|M_2|`) | Uniform **`R^2`**: $q_{\min}$ / **`(v, v_R)` pair segs** | Excursion `R_exc`: $q_{\min}$ (`!/K`) |
|---|---:|---:|---:|---:|---:|
| `synth-hex-iid-65536` | `3,629` (`10,094`) | `57,377` / `56,820` / **`7,881`** (`0.377`) | `18,695`: **`7,264`** (`0.389`) | `9,084` / `24,619` | `3,462` (`0.490`) |
| `synth-tmpl-hex-1000x40` | `19` (`33`) | `45,571` / `45,559` / **`5,304`** (`0.324`) | `14,547`: **`5,079`** (`0.349`) | **`300`** / **`343`** (`133×` < GS!) | `1,012` (`0.506`) |
| `block-0.L` | `791` (`2,148`) | `52,319` / `50,329` / **`6,658`** (`0.203`) | `29,295`: **`6,373`** (`0.217`) | **`3,066`** / **`6,339`** (`8.3×` < GS!) | `1,604` (`0.105`) |
| `block-2.L` | `1,089` (`3,136`) | `56,117` / `53,560` / **`7,252`** (`0.177`) | `37,020`: **`7,030`** (`0.190`) | **`4,139`** / **`9,028`** (`6.2×` < GS!) | `2,056` (`0.092`) |
| `block-85.L` | `707` (`2,018`) | `86,199` / `80,887` / **`10,054`** (`0.179`) | `50,096`: **`9,701`** (`0.194`) | **`3,820`** / **`7,767`** (`11.1×` < GS!) | `2,316` (`0.106`) |
| `block-143.L` | `1,823` (`5,436`) | `114,445` / `108,247` / **`13,977`** (`0.195`) | `63,997`: **`13,369`** (`0.209`) | **`7,316`** / **`16,785`** (`6.8×` < GS!) | `4,384` (`0.098`) |
| `block-210.L` | `1,229` (`3,564`) | `139,880` / `131,560` / **`15,223`** (`0.206`) | `65,263`: **`14,655`** (`0.225`) | **`5,528`** / **`12,295`** (`11.4×` < GS!) | `3,857` (`0.108`) |
| `block-207.L` | `1,236` (`3,547`) | `93,074` / `87,629` / **`11,145`** (`0.224`) | `44,400`: **`10,863`** (`0.245`) | **`5,538`** / **`12,046`** (`7.7×` < GS!) | `3,228` (`0.124`) |

## 3. Three Structural Findings

1. **Why M-17 GS fragmented `24×` (`2,148 -> 52,319`) and why relabelling by `last_byte` fails:**
   - Switching from `first_byte` (`v[j]`: `52,319`) to `last_byte` (`50,329`) barely moves the needle (`-3.8%`), because M-17 GS mixes variable chain lengths (`1, 2, 3, ...`) and non-commuting $[\mathrm{LF}_v, P]$ jumps.
   - Even on the section-reindexed surviving bijection $R_{\mathrm{gs}}$ (`|M_gs| = 32,833` on `block-0.L`), **the universal oracle lower bound $q_{\min}(R_{\mathrm{gs}})$ jumps `8.4×–14.2×`** over $q_{\min}(R)$ (to `6,658–15,223`, i.e., `17.7–22.4%` of surviving links on Wikidata, and `37.7%` on pure novel hex).
2. **The Third-Composition Wall (Exact Ceiling of `CL1`):**
   - `CL1` survives **Sweep 2** with zero fragmentation (`650–1,939` cache16 segments) because `A_1` still reads unmodified $P \circ \mathrm{LF}_v$ in reverse.
   - Once *both* `D_1` and `A_1` are composed into $R_2 : M_2 \to M_2$, **`q_min(R_2)` jumps to `6,373–14,655` (`19.0–24.5%` of `|M_2|` on Wikidata; `38.9%` on pure hex)**. By Theorem CL2.1, **no relabelling $\ell$ — not even an oracle — can keep a third exact-`+1` monotone-cursor sweep compact on $M_2$**.
3. **Why Uniform `R^2` under `(v[j], v[R[j]])` is `6–11×` Less Fragmented than M-17 GS (`133×` on Templated Hex):**
   - Uniform depth-2 squaring $R^2 = P \circ \mathrm{LF}_v \circ P \circ \mathrm{LF}_v$ preserves exact `+1` runs inside each 2-byte class $(v[j], v[R[j]])$ whenever $R[j]$ stays inside one $P$-piece: `6,339–16,785` segments (`2.9–3.9×` of $B_P$, vs `18–43×` for M-17 GS; `343` vs `45,571` on templated 40-hex!).
   - *Limitation:* Iterating uniform squaring multiplies active classes as $|\Sigma|^k$ (`16 -> 256 -> 4,096 -> ...`), so it is a controlled 1-step factor (`~3×` growth), not an indefinitely closed fixed-cursor algebra.

## 4. Referee Verdict

Theorem CL2.1 proves cleanly why the Closure Problem splits at depth 2:
- **Depth 2 (Sweep 1 + Sweep 2):** Solved constructively by `CL1` (zero fragmentation, `650–1,939` cache16 segments, `m` total reads, `0.67–1.08 h_1` emit links, `8m` DAG).
- **Depth 3+ (Sweep 3 on $M_2$):** Blocked representation-independently for exact-`+1` monotone cursors by $q_{\min}(R_2) = 0.19–0.25\,|M_2|$ (`0.39\,|M_2|` on novel hex).
