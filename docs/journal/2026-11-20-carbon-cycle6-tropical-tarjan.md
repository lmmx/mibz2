# Carbon Cycle 6 · Bijective Tube Exhaustion, Hierarchical Pinch-Point Tropical Tarjan Elimination (`HP-FTTE`), and the L1 LIFO Tail-Locality Obstruction

*20 November 2026 · Carbon (`research/carbon-word-transport`, Cycle 6).*

## 0. Cycle-Boundary Check (`origin/master` Fetch — `#665` & `9948e33`)

Fetching `origin/master` (`1af2fb33`) at the Cycle 5 → Cycle 6 boundary surfaced two important updates:
1. **`9948e33` (`docs/journal/2026-11-20-open-u16q.md` calibration):** Native Rust calibration (`calib.rs`) confirmed that our C `Open-qi16` baseline (`1.424–3.352 ms`) matches native safe-Rust `Open` (`1.426–3.415 ms`) within `0.1–6%`, while `U16Q` runs at `0.86–0.91×` on the isolated kernel (`−0.1%` in 1–4 thread whole-pipeline CI wall time; 10–20 thread i9 / stage attribution pending).
2. **`1af2fb3` (`#665`):** Adopted Cycle 4's binary-alphabet fix (`r - σ = 12` was a `σ = p = 1,024` artifact; fixed-alphabet `σ = 2` gives `r = 2,014`) onto `origin/master`.

With consumer traversal (`BVSMC-1P` / `BVSMC-2P`) and pipeline boundaries (`Cycle 5`) characterised, Cycle 6 attacks **Obligation #2 ("Building the retained map")** from first principles: `Open::construct` spends `55–71%` of its non-queue time (`0.54–0.96 ms`) inside `endpoint.rs + ep_append + bands[] + Wire` (building `EpNode` doubly-linked Rauzy–Veech lists, `EpRule` binary SLP trees, a DFS `stack`, `bottom[]`, `bands[]`, and scattered random writes into `edges[dst]`). Can reversing thick-complement elimination from backward `LF` to **forward $\Psi = \mathrm{LF}^{-1}$** over codomain tiles with **Tropical $(\min, +)$ path compression** eliminate all of those intermediate structures and write `edges[0..m]` strictly sequentially (`dst = 0 .. m-1`)?

---

## 1. Explore: Four Formulations Across Coxeter, Koopman, and Tropical Tarjan Worlds

Mode: **Reorientation** (`METHOD.md` §1 & §3a):

| # | Candidate / Framework | Mathematical setting | Mechanism & information needed | Likely mathematical or economic trap |
|---|---|---|---|---|
| 1 | **`Coxeter-Lehmer-LF`** | Symmetric-group Bruhat / Coxeter length $\ell(\mathrm{LF}) = \|\mathrm{Inv}(\mathrm{LF})\|$ & Lehmer inversion vectors | Compute Lehmer / inversion structure directly from run records in $O(R)$ without materialising `LF` | Global Coxeter invariants fail identifiability; row-wise Lehmer vector costs $\Theta(n)$ Fenwick/radix work |
| 2 | **`Koopman-Krylov`** | Shift-operator Krylov sequence $(w^\top P_{g^{-1}}^t e_{\text{mark}})_{t=0}^{m-1}$ over $(\mathbb{C}^m, P_{g^{-1}})$ | Replace step-by-step evaluation of $P_{g^{-1}}^t$ by spectral / FFT diagonalisation | Permutations are 1-sparse in the standard basis (`1` load/step); diagonalising $P_{g^{-1}}$ requires the cycle-order discrete-log bijection $\theta : [0, m) \to \mathbb{Z}/m\mathbb{Z}$ itself |
| 3 | **`FTTE-Term`** | Forward $\Psi = \mathrm{LF}^{-1}$ elimination from monotone domain gaps with terminal $(\min,+)$ certificates | Walk `dst = 0 .. m-1` sequentially; spell words forward (`0` reversals, `0` `bands[]`, `0` scattered `Wire`); memoise terminal intervals `[y - L, y + R)` | Bijective tube exhaustion (`Theorem 6.1`): consumed sub-tubes never recur, so terminal-only certificates re-walk shared thick prefixes (`45,285` steps on `block-0.L`) |
| 4 | **`HP-FTTE`** | **Hierarchical Pinch-Point Forward Tropical Tarjan Elimination** (Tarjan 1979 `EVAL-LINK` over $(\min,+)$ two-sided interval balls) | Store **non-terminal two-sided tropical pinch jumps** right before downstream tube narrowing (`unconsumed_width × steps_saved`), compressing shared thick prefixes into an implicit split DAG | Random-coordinate `tile_lut[cur >> 6] -> psi_hot[ti] -> psi_cert[ti]` lookups hit L2 (`476 KiB`), competing against `endpoint.rs`'s L1-resident LIFO tail pointers |

---

## 2. Triage & Foundational Theorems (`verify_cycle6.py`)

All proofs and censuses below are verified in [`verify_cycle6.py`](2026-11-20-carbon-cycle6/verify_cycle6.py) (`873/873` Coxeter columns + `10,454/10,454` exhaustive small instances + `7/7` adversarial controls including `50,000`-row periodic shear + `6/6` authenticated Wikidata blocks, `5,399,692` bytes exact).

### Result 2.1 · Coxeter Length Identity & Non-Identifiability (`proved` + `verified` on `873/873` columns)

**Proposition 6.1 (`proved` + `verified`).**
Because stable `LF` preserves relative order among equal symbols ($\mathrm{LF}(i) < \mathrm{LF}(j)$ whenever $i < j$ and $L[i] = L[j]$), a pair $i < j$ is a Coxeter inversion of $\mathrm{LF} \in S_n$ if and only if $L[i] > L[j]$:
$$\ell(\mathrm{LF}) = |\mathrm{Inv}(\mathrm{LF})| = \sum_{0 \le i < j < n} \mathbf{1}[L[i] > L[j]],$$
which is computable in $O(R)$ sequential time directly from run records. However, primitive single-cycle columns `L_1 = b"bc$aa"` and `L_2 = b"cb$aa"` at `p = 2` have identical symbol histograms $C$ and identical Coxeter lengths ($\ell(\mathrm{LF}_1) = \ell(\mathrm{LF}_2) = 5$) yet distinct texts. Reconstructing the full Lehmer code $(|\{j > i : L[j] < L[i]\}|)_{i=0}^{n-1}$ merely recovers $L$ itself. **Killed.**

### Result 2.2 · Bijective Tube Exhaustion & Hierarchical Pinch-Point Tarjan Compression (`proved` + `verified`)

**Theorem 6.1 (Bijective Tube Exhaustion & Two-Sided Tropical Pinch Compression — `proved` + `verified`).**
Partition $[0, n)$ in the domain into retained gaps $G$ (`|G| = m`) and thick strips $T$ (`|T| = n - m`), and in the codomain into Thin-Exit tiles $E_{\text{thin}} = \mathrm{LF}(G)$ and Thick $\Psi$-tiles $E_{\text{thick}} = \mathrm{LF}(T)$.
1. *Sequential Direct Wiring & Forward Spelling:* As $x$ sweeps $G$ in monotone row order (`dst = 0, 1, ..., m-1`), iterating $\Psi = \mathrm{LF}^{-1}$ from $x$ until the first entry $\Psi^{\tau(x)}(x) \in E_{\text{thin}}$ visits bytes in **exact forward text order** (`0` string reversals) and lands at the retained successor `queues[a][q_off]`, writing `edges[dst .. dst + span]` **strictly sequentially** (`0` `bands[]`, `0` scattered `Wire` writes).
2. *Two-Sided Tropical Tube Certificate:* Along any thick trajectory $(y_0, \dots, y_{K-1}) \in E_{\text{thick}}^K$ with local tile margins $l_u = y_u - \text{start}_{t_u}$ and $r_u = \text{end}_{t_u} - y_u$, the two-sided $(\min, +)$ ball around $y_s$ through step $j \le K$,
   $$B_{s \to j}(y_s) = \bigl[y_s - L_{s \to j},\; y_s + R_{s \to j}\bigr), \qquad L_{s \to j} = \min_{s \le u \le j} l_u,\quad R_{s \to j} = \min_{s \le u \le j} r_u,$$
   is the **exact maximal monochromatic affine tube** sharing the tile sequence $(t_s, \dots, t_j)$ and forward subword $w_s \cdots w_j$.
3. *Bijective Exhaustion Law:* Because $\Psi : [0, n) \to [0, n)$ is a **bijection**, wiring the domain span $B_0 = [x, x + \text{span}) \subseteq G$ **permanently exhausts** every point in $\Psi^t(B_0) \subset E_{\text{thick}}$ ($0 \le t < \tau(x)$): no future starting row $x' \in G \setminus B_0$ can ever intersect $\Psi^t(B_0)$. Therefore, whenever $\text{span} = R_{0 \to K}$ and $L_{s \to K} = 0$, a terminal certificate $B_{s \to K}(y_s)$ contains **zero unvisited future rows**. Selecting instead the pinch index $j^* \in (s, K]$ maximising unvisited saved work $\text{unc}(s, j) \cdot (\text{len}(s \to j) - 1)$ installs **non-terminal Tarjan jump certificates** (and automatically selects binary halving $j^* = \lfloor(s+K)/2\rfloor$ on self-shearing periodic strips), compressing thick discovery below `endpoint.rs`'s Rauzy–Veech step count! $\square$

| Block | Retained `m` | `PsiTile`s | `Open` (`endpoint.rs`) words / `spelling` | Terminal-only `FTTE` steps | **`HP-FTTE` Tarjan steps** | **`HP-FTTE` words / `spelling`** |
|---|---:|---:|---:|---:|---:|---|
| `block-0.L` | `89,874` | `5,605` | `1,326` / `50.8 KiB` (`11,215` RV steps) | `45,285` | **`9,253`** | **`1,302` / `50.1 KiB`** |
| `block-2.L` | `112,070` | `6,508` | `1,922` / `48.2 KiB` | `44,013` | **`11,072`** | **`1,904` / `48.1 KiB`** |
| `block-85.L` | `156,404` | `2,859` | `1,220` / `30.9 KiB` | `26,446` | **`5,424`** | **`1,182` / `30.0 KiB`** |
| `block-143.L` | `196,841` | `8,802` | `3,635` / `56.8 KiB` | `49,455` | **`17,291`** | **`3,576` / `56.4 KiB`** |
| `block-210.L` | `204,281` | `4,929` | `2,228` / `37.8 KiB` | `32,277` | **`9,338`** | **`2,181` / `37.3 KiB`** |
| `block-207.L` | `135,921` | `6,184` | `2,259` / `54.6 KiB` | `49,960` | **`12,330`** | **`2,227` / `54.2 KiB`** |

---

## 3. Deepen & Confront: Gate G5 Paid-Cost Falsifier (`g5_ftte.c` — **STRICT G5 VERDICT: FAIL**)

### Candidate Declaration & Acquisition Manifest
- **Manifest:** [`ftte.toml`](2026-11-20-carbon-cycle6/ftte.toml) (`python3 docs/team/acquisition/check_manifest.py` **PASS**).
- **Implementation (`g5_ftte.c`):** Split `16 B` `PsiTileHot` + `32 B` `PsiCert` (guarded by `TF_HAS_CERT` bit so uncertified tiles never touch `PsiCert`), `64`-row `O(1)` page directory `tile_lut[cur >> 6]` (`56 KiB`, built inline during the tile scan), zero `gaps[]` allocation (`strips[]` complement scanned directly), and `~1.0 MiB/worker` less scratch memory than `Open` (`EpNode`, `EpRule`, `order[2]`, `terminals[2]`, `bottom`, `stack`, and `bands` all eliminated).
- **Pre-Registered Rule (`g5_ftte.c`, 25 paired interleaved rounds after 3 warmup rounds):**
  - `Arm 1 (HP-FTTE-1L)` vs `Arm 0 (Open-qi16)`: `construct <= 0.85×` and `total <= 0.90×` on `>= 5/6` blocks;
  - `Arm 3 (HP-FTTE+BVSMC-2P)` vs `Arm 0` & `Arm 2 (Open+BVSMC-2P)`: `total <= 0.72×` `Open` and `<= 0.95×` `Open+BVSMC-2P` on `>= 5/6` blocks.
  - Post-hoc diagnostic arms (`Arm 5: HP-FTTE-Reg-1L*` and `Arm 6: HP-FTTE-Reg+BV2P*`) replace the monotonic-stack pinch scan with `O(1)` register-only `(Exit, K-1, K/2)` pinch tracking to isolate algorithmic pinch overhead from hardware cache latency.

### Paired 6-Block Measurements (`measured`, VM C diagnostic — `g5_ftte.c`, `-O3 -mavx2`)

| Block | `Open-qi16` Con / Tot (ms) | `HP-FTTE-1L` Con / Tot (`×Open`) | `HP-FTTE-Reg-1L*` Con / Tot (`×Open`) | `Open+BVSMC-2P` Tot (`×Open`) | `HP-FTTE-Reg+BV2P*` Tot (`×Open`) |
|---|---:|---:|---:|---:|---:|
| `block-0.L` | `0.889 / 1.375` | `1.179 / 1.681` (`1.326× / 1.223×`) | `0.975 / 1.471` (**`1.096× / 1.070×`**) | `1.153` (`0.839×`) | `1.250` (`0.909×`) |
| `block-2.L` | `1.089 / 1.766` | `1.412 / 2.069` (`1.296× / 1.171×`) | `1.161 / 1.833` (**`1.066× / 1.038×`**) | `1.355` (`0.767×`) | `1.540` (`0.872×`) |
| `block-85.L` | `0.919 / 1.797` | `1.105 / 2.000` (`1.202× / 1.113×`) | `0.996 / 1.863` (**`1.084× / 1.036×`**) | `1.306` (`0.727×`) | `1.347` (`0.749×`) |
| `block-143.L` | `1.815 / 3.196` | `2.302 / 3.711` (`1.269× / 1.161×`) | `1.935 / 3.347` (**`1.066× / 1.047×`**) | `2.217` (`0.694×`) | `2.401` (`0.751×`) |
| `block-210.L` | `1.367 / 2.667` | `1.605 / 2.910` (`1.174× / 1.091×`) | `1.454 / 2.765` (**`1.064× / 1.037×`**) | `1.876` (`0.703×`) | `1.957` (`0.734×`) |
| `block-207.L` | `1.211 / 2.071` | `1.625 / 2.462` (`1.342× / 1.189×`) | `1.370 / 2.242` (**`1.132× / 1.083×`**) | `1.545` (`0.746×`) | `1.674` (`0.808×`) |

- **Strict Gate G5 Verdict:** **FAIL** (`0/6` blocks meet `<= 0.85×` producer `construct`).

---

## 4. Learn: The L1 LIFO Tail-Locality vs L2 Codomain Tile-Jump Obstruction

Sub-phase profiling (`block-0.L`: `RunQ = 0.354 ms`, `TileLut = 0.070 ms`, `ThinWire = 0.111 ms`, `FwdTrace = 0.218 ms`, `Spell = 0.083 ms`, `CertInstall = 0.14–0.41 ms`) exposes a fundamental hardware-locality law governing **Obligation #2 ("Building the retained map")**:

- **Concrete Witness:** On `block-0.L`, `HP-FTTE-Reg` executes **fewer** elimination steps than `Open`'s `endpoint.rs` (`~9,300` vs `11,215`), writes `edges[0..m]` sequentially (`ThinWire = 0.111 ms` across `80%` of `m`), and eliminates `~1.0 MiB/worker` of scratch buffers, yet `construct` still runs `6.4–13.2%` (`+0.077` to `+0.159 ms`) slower than `Open-qi16`.
- **Exact Scope:** Binds all interval-lookup / tile-directory state-elimination constructors over $[0, n)$ (forward $\Psi$ `FTTE` / `HP-FTTE` as well as backward `LF` `q16`).
  1. `endpoint.rs` always splits `nodes[0].prev[0]` and `nodes[0].prev[1]` at the **LIFO rightmost tail** of two doubly-linked lists, keeping its active working set inside **L1D (`32 KiB`, ~4-cycle latency, ~11 ns/step)**, while `Wire` sweeps `queues[a]` sequentially in `256` byte-grouped outer iterations.
  2. By contrast, jumping across global `[0, n)` coordinates via `cur = tl.param + ll -> tile_lut[cur >> 6] -> psi_hot[ti] -> psi_cert[ti]` touches a **`476 KiB` L2-resident** working set with a 2-to-3-hop serial dependent L2 load chain plus random L2 RFO writes for certificate installation (`~35–40 ns/step`), and switches `queues[ex_sym]` across byte classes `1,226–3,153` times in domain-gap order.
- **Escape Condition:** Because `RunQ` (`each_run` + `queues[256]`) already accounts for `40–58%` of `Open::construct` and `endpoint.rs` executes in L1 cache in `0.12–0.25 ms`, **no alternative thick-complement solver over the same width-16 retained set $S$ can materially (`>= 15%`) accelerate `Open::construct`**. Escaping Obligation #2 + Obligation #1 simultaneously requires **changing the retained section $S$ itself** (non-run-union complements, `FRONTIER.md` line 132) — specifically testing whether $m = |S|$ can be collapsed toward the switch/core scale (`~15k` rows vs `90k–204k`) *without* incurring `switches.rs`'s `6.77 ms` narrow-word discovery blowup. That is the target of Cycle 7.
