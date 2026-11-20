# Carbon Cycle 1 · Topological Rigidity of the Origami/Track Doorways, Sentinel-Spliced Monoid Contraction (SSMC), and the ROB Branch-Flush Obstruction

*20 November 2026 · Carbon (`research/carbon-word-transport`, starting from `origin/master` at `143814249755b8ac11c063aca946cbeca3720758`).*

## 0. Environment, Verified Inputs, and Prior-Art Constraints

1. **Environment & inputs (`verified`):**
   - Base `origin/master` SHA: `143814249755b8ac11c063aca946cbeca3720758`.
   - Isolated branch: `research/carbon-word-transport` (fresh checkout separate from Neon's paused `research/neon-bipolar-contraction`).
   - Charter, acquisition registry (`primitives.toml`), manifest checker (`check_manifest.py`), and all six authenticated Wikidata BWT blocks from `origin/data/admitted-bwt` (`41c03a5b`) verified against `admitted-bwt/manifest.json` (`block-0.L`, `block-2.L`, `block-85.L`, `block-143.L`, `block-207.L`, `block-210.L`: all `5,399,692` bytes, `n`, `p`, `r`, and SHA-256 digests match).
   - Environment note: `gcc 12.2.0` and `Python 3.11.2` run natively; `rust-toolchain.toml` specifies `1.99.1` on a read-only `/usr/local/rustup`, while `RUSTUP_TOOLCHAIN=stable` provides `rustc 1.100.0`. No production code (`src/`) is modified.
2. **Governing constraints from prior research (in Carbon's words):**
   - **Open's `18.7` instructions/byte bar (`cache.csv`):** Shipped `Open<16, INDUCE=true, QUEUE=true>` (`qi16`) executes only `~16.8M` instructions (`4.4` loads/B, `2.6` stores/B) on the `900,000`-byte frozen block, spending `28–36%` in the run/queue pass and `36–48%` in `emit` (`m ≈ 90k–204k` serial pointer steps, `~26` instructions/step).
   - **Width-1 run dominance (`62.8%–75.4%` of $r$):** Any full-block run-graph, forest, or IET contraction touching all $r$ runs (`Half-Bipolar`, `UFC-1`, `Astra Inv. 3`) pays an $O(r)$ random/metadata producer penalty exceeding Open's `~1.1–1.5` L2 misses/run budget; conversely, lowering `LIMIT` (`16 -> 1`) inflates flattened `spelling` from `32–58 KiB` to `495–623 KiB` (`#657`, Neon Cycle 4).
   - **Producer–consumer & serial-rank traps:** Relocating queue expansion into band scatter (`Q`, `#654`), rescanning `records` (`Backward Open`, `#659`), streaming `> 40 B/row` in bucketed Schur passes (`BSSE`), or replacing explicit `edges[j]` with 4-load implicit rank bitplanes (`BSO`, `1.71–2.80×` Open) all lose end-to-end.

---

## 1. Explore: Six Genuinely Distinct Mathematical Formulations

Mode: **Reorientation** (`METHOD.md` §3a). Candidates draw directly on the three foundational doorways (`Origami monodromy`, `Thurston train tracks / Bestvina–Handel`, `Agol–Hass–Thurston interval pairings`), `FRONTIER.md`'s five pending leads, and parallel monoid contraction:

| # | Candidate | Mathematical setting | Mechanism & information needed | Likely cost / correctness trap |
|---|---|---|---|---|
| 1 | **`Origami-OutF2`** | Branched torus covers & $\mathrm{Out}(F_2)$ / $\mathrm{SL}(2,\mathbb{Z})$ monodromy transport (`2026-11-20-origami-lift.md` §6.2) | Apply Nielsen moves $(r, u) \mapsto (g_1, g_2)$ or factor through an intermediate cover $O \to \bar{O} \to T^2 \setminus \{\ast\}$ to find a direction with fewer pieces/wider cylinders, then substitute back | Global words in $(r, u)$ may never reduce piece count below $r$, or $\langle r, u \rangle$ may be primitive |
| 2 | **`Ribbon-Face`** | Combinatorial maps / rotation systems (`FRONTIER.md` Lead #3) | Equip $G_{\text{run}}$ with planar-switch cyclic dart order $\sigma_v$ and assemble ordered itineraries from face cycles of $\sigma \circ \alpha$ | Unweighted ribbon faces may ignore intra-run offsets; weighted switches reduce back to splitting |
| 3 | **`BH-Fold`** | Bestvina–Handel graph self-maps & Stallings folds (`FRONTIER.md` Lead #2) | Construct a cellular self-map $\varphi : \Gamma \to \Gamma$ from $I_0$ without row walking and resolve branching collectively via Stallings folds at illegal turns | Vertex-invariance $\varphi(V) \subseteq V$ on a transitive $n$-cycle may force $|V| = n$ |
| 4 | **`AHT-Spiral`** | Agol–Hass–Thurston interval pairings (`FRONTIER.md` Leads #4–5; `2026-11-14-aht-steelman`) | Apply word-carrying AHT multiplicative transmissions $g_1^{-q} \circ g_2$ ($q = \lfloor w_1/w_2 \rfloor \ge 2$) to Open's thick-strip complement (`endpoint.rs`), where wide strips (`w <= 27k`) could self-overlap | Periodic gcd-merger/$\mathbb{Z}^d$-sums lose word order; `endpoint.rs` (`8.5–18.5%`) may lack $q \ge 2$ streaks or G4 headroom |
| 5 | **`Kmer-Psi`** | Suffix-interval radix tower ($C_1, C_2, C_3$) | Compute exact $k$-gram suffix bucket starts $C_k[w]$ in $k$ sequential scans ($Q=0$) over `records` so each row knows forward $T[j..j+k]$ and $\Psi^k$ is order-preserving on $I_w$ | Image $\Psi^k(I_w)$ is scattered across $[0, n)$, requiring $4n$-byte scatter tables or slow rank/select |
| 6 | **`SSMC-Open`** | Non-commutative monoid contraction on cycle covers (`FRONTIER.md` Lead #4: Shun–Gu–Blelloch + Cong–Bader warning) | Embed $S \ll m$ algebraic sentinel codes directly inside Open's `edges[0..m]` (zero new graph, escaping Cong–Bader) so $K$ concurrent register cursors evaluate $\prod W(g^{-t}(\text{mark}))$ in parallel (`MLP = K`, $D \approx m/K$) directly into `4 KiB` byte chunks | Branch mispredictions, register spills, or `memcpy` stitch overhead on a single core could erase MLP gains |

---

## 2. Triage: Five Exact Kills (Theorems, Witnesses, and 6-Block Censuses)

All verifiers and censuses below are reproduced by [`verify_cycle1.py`](2026-11-20-carbon-word-transport/verify_cycle1.py).

### Kill 1 · `Origami-OutF2`: $\mathrm{Out}(F_2)$ Global-Frame Rigidity & Monodromy Primitivity (`proved` + `verified` + `measured`)

Let $r = \mathrm{LF}$ and $u(x) = (x + 1) \bmod n$ on $\mathbb{Z}/n\mathbb{Z}$. Note that on $\mathbb{Z}/n\mathbb{Z}$, $u$ is ALREADY a single global translation ($O(1)$ time, $0$ bytes of state, $0$ breakpoints), while $r(x) = (x + \delta_a) \bmod n$ has $r_{\text{aff}} = |\operatorname{supp}(k)| = r - |X|$ cyclic affine pieces (MONODROMY Record 86).

**Theorem 1.1 ($\mathrm{Out}(F_2)$ Frame Dichotomy — `proved`).**
Let $(g_1, g_2) = \Phi(r, u)$ be any generating pair of $F_2 = \langle r, u \rangle$ under $\Phi \in \mathrm{Aut}(F_2)$, with abelianisation matrix $A = (\deg_{(r,u)}(g_1, g_2)) \in \mathrm{GL}(2, \mathbb{Z})$, $\det(A) = \pm 1$:
1. *One generator free of $r$ ($\deg_r(g_2) = 0$):* Since $\det(A) = \pm 1$, $\deg_u(g_2) = \pm 1$ and $\deg_r(g_1) = \pm 1$. By Nielsen's theorem (every primitive element of $F_2$ in homology class $(0, \pm 1)$ is conjugate to $u^{\pm 1}$), $g_2 = w u^{\pm 1} w^{-1}$. In particular, whenever $g_2 = u^{\pm 1}$ keeps $u$'s $O(1)$ closed form, Stallings' folding criterion on the 2-rose forces $g_1 = u^a r^{\pm 1} u^b$ ($a, b \in \mathbb{Z}$): after cancelling $u^{\pm 1}$-prefixes and suffixes at the basepoint, a reduced word in $\{r, u\}$ folds with the loop $u$ to the 2-petal rose iff its remaining core has length $1$, namely $r^{\pm 1}$. Since $u^a, u^b$ are global rotations of $\mathbb{Z}/n\mathbb{Z}$, $g_1 = u^a r^{\pm 1} u^b$ has the **exact same cyclic affine piece count $r_{\text{aff}}$** as $r$, and reconstructing $r = u^{-a} g_1 u^{-b}$ (or $u^b g_1^{-1} u^a$) still makes $1$ dependent evaluation of $g_1$ per step.
2. *Both generators depend on $r$ ($\deg_r(g_1) \neq 0, \deg_r(g_2) \neq 0$):* Neither generator has an $L$-independent closed form, and writing $r = W(g_1, g_2)$ as a reduced word requires $|W| \ge 2$ generator evaluations per step of $r$, while on every run of width $> |w_i|$ where $r$ and $u$ commute, $g_i$ acts as $r^{\deg_r(g_i)} u^{\deg_u(g_i)}$ (moving all interior points by a nonzero power of $r$). $\square$

*Clarification on `2026-11-20-origami-lift.md` §5 & §6.2:* Rauzy–Veech induction is **not** in $\mathrm{Out}(F_2)$ (which applies the same global word in $(r, u)$ to all $n$ squares); it lives in $\mathrm{Out}(F_r)$, acting locally on the $r$ horizontal saddle connections of the punctured surface $\pi_1(\Sigma \setminus \operatorname{Sing})$.

**Theorem 1.2 (Origami Monodromy Primitivity — No Intermediate Covers — `proved` + `verified`).**
Intermediate topological covers $O \to \bar{O} \to T^2 \setminus \{\ast\}$ of degree $1 < d < n$ ($d \mid n$) correspond bijectively to nontrivial $\langle r, u \rangle$-invariant block systems on $\mathbb{Z}/n\mathbb{Z}$. Because $u(x) = (x + 1) \bmod n$ is a regular $n$-cycle, every $u$-invariant block system is the partition into cosets of $k\mathbb{Z}/n\mathbb{Z}$ for some divisor $k = n/d$ ($1 < k < n$): let $B_0$ be the block containing $0$ and $k \ge 1$ its least positive element; $u^k(0) = k \in B_0 \implies u^k(B_0) = B_0 \implies B_0 = k\mathbb{Z}/n\mathbb{Z}$. For any divisor $k \le \max_a w_a$, a single run $I_{a^*}$ of width $w_{a^*} \ge k$ meets every coset $\bmod k$ and satisfies $r(x) = x + \delta_{a^*}$ there, forcing the quotient action on $\mathbb{Z}/k\mathbb{Z}$ to be $t \mapsto (t + \delta_{a^*}) \bmod k$ and thus requiring $\delta_a \equiv \delta_0 \pmod k$ for all $0 \le a < r$. On all 6 authenticated blocks, $\gcd(\{\delta_a - \delta_0\} \cup \{n\}) = 1$ (`max_w = 25,134–27,234`), and direct check over *every* divisor $1 < k < n$ confirms $r(k) \not\equiv r(0) \pmod k$ (`verified` in `verify_cycle1.py`). Hence $\langle \mathrm{LF}, +1 \rangle \le S_n$ is **primitive** on 6/6 blocks: zero nontrivial intermediate covers exist. $\square$

### Kill 2 · `Ribbon-Face`: Unweighted Non-Identifiability & Train-Track Equivalence (`proved` + `verified`)

Equip $G_{\text{run}} = (V, E)$ ($|V| = r$, $E$ = common-refinement arcs of $D$ and $\mathrm{LF}(D)$) with the planar-switch rotation system $\sigma_v = (e_1^{\text{in}}, \dots, e_p^{\text{in}}, f_q^{\text{out}}, \dots, f_1^{\text{out}})$ and edge-reversal $\alpha$.
- **Unweighted obstruction (`verified` in `verify_cycle1.py`):** The two distinct primitive length-$7$ texts $T_1 = \text{\texttt{aaabaa\$}}$ ($L_1 = \text{\texttt{aab\$aaa}}, p_1 = 3$) and $T_2 = \text{\texttt{aabaaa\$}}$ ($L_2 = \text{\texttt{aaab\$aa}}, p_2 = 4$) have the **exact same** block length $n = 7$, **exact same** byte histogram $C$ (`5 'a', 1 'b', 1 '$'`), **exact same** run-symbol sequence `('a','b','$','a')`, **exact same** primary run (`run 2`), and **isomorphic** ribbon graphs $(V, E, \sigma, \alpha)$ with identical face cycles $\sigma \circ \alpha$. Thus the combinatorial map embedding and its face walks $\sigma \circ \alpha$ cannot reconstruct the text without arc widths.
- **Weighted equivalence:** Restoring integer arc widths $w(e)$ on $(V, E, \sigma, \alpha)$ yields the exact measured train track of `2026-11-20-run-network.md`, on which order-preserving switch resolution is train-track splitting (`Cells`) or section elimination (`Open`).

### Kill 3 · `BH-Fold`: Transitive Cellular Subdivision Dichotomy (`proved` + `verified`)

**Theorem 3.1 (`proved` + `verified` on `146/146` primitive small columns in `verify_cycle1.py`).**
For any primitive column ($r = \mathrm{LF}$ is a single $n$-cycle) with $r \ge 2$ runs, the non-empty domain boundary set $D = \{d_1, \dots, d_{r-1}\}$ satisfies $\bigcup_{t \ge 0} \mathrm{LF}^t(D) = \{0, \dots, n-1\}$. Because a Bestvina–Handel graph self-map $\varphi : \Gamma \to \Gamma$ requires vertex invariance $\varphi(V(\Gamma)) \subseteq V(\Gamma)$ (`Ann. Math. 1992`, §1):
1. Any exact cellular subdivision of $\mathbb{Z}/n\mathbb{Z}$ at $V \supseteq D$ with $\mathrm{LF}(V) \subseteq V$ forces $V = \{0, \dots, n-1\}$ ($n$ vertices, $n$ unit edges, $\varphi$ a permutation with **zero illegal turns** and zero Stallings folds).
2. Pushing $\mathrm{LF}(D) \setminus D$ onto $D$ by homotopy forgets intra-run offsets (`cbaa` vs `cbbaa` yield identical 3-edge maps).
3. Restricting to a sub-interval $J \subset [0, n)$ whose first-return endpoints stay in $D \cap J$ recovers Rauzy–Veech induction (`Cells` / `Open`). $\square$

### Kill 4 · `AHT-Spiral`: Word-Carrying AHT on Open's Thick Complement (`proved` + `measured`)

Extending MONODROMY's whole-IET audit (`2026-11-14-aht-steelman`, where AHT periodic gcd-merger and $\mathbb{Z}^d$-weight transfer lose non-commutative word order, and adjacent Euclidean-division streaks fire $28 / 20.46\text{M}$ times on whole-block IETs): we simulated Open's thick-strip `endpoint.rs` (`LIMIT = 16`) across all 6 authenticated blocks. Out of `79,349` total `endpoint.rs` steps (`11,215`, `13,671`, `7,130`, `21,295`, `13,898`, `12,140`), repeated winner–loser pairs ($q \ge 2$) save **`0, 0, 0, 2, 0, 0` steps (`0.0025%`)** (`endpoint.rs` already averages `1.08–1.40` steps per initial node, within `8.5–18.5%` of Open total time — failing Gate G4).

### Kill 5 · `Kmer-Psi`: Scattered Codomain of Forward $k$-Gram Suffix Buckets (`proved`)

Although exact $k$-gram suffix intervals $I_w = [C_k[w], C_k[w]+n_w)$ are obtainable in $k$ sequential passes over `records` ($Q = 0$) and $\Psi^k$ is strictly order-preserving on $I_w$ (since $\Psi$ is strictly increasing on each $F^{-1}(c_t)$ traversed by $w$), the image $\Psi^k(I_w) = \{ z \in [0, n) : T[\mathrm{SA}[z]-k..\mathrm{SA}[z]] = w \}$ is scattered across $[0, n)$. Indexing target rows without an $O(n)$ scatter table (`4n = 3.6 MB`) reduces to implicit rank/select (closed by Neon Cycle 5's `BSO` 4-load serial latency obstruction).

---

## 3. Deepen: Sentinel-Spliced Monoid Contraction (`SSMC-Open`)

### Mandatory `METHOD.md` §3a Candidate Declaration
- **Mode:** Reorientation (seeking to replace serial orbit walking on $g^{-1}$ with parallel monoid contraction without auxiliary graph construction; classification deferred to the Coordinator and MONODROMY).
- **Displaced obligation:** Obligation #1 — serial data-dependent traversal of Open's $m$ retained rows (`D = m`, `MLP = 1`, `36–48%` of Open's single-thread time).
- **Replacement principle:** Embed $S \ll m$ algebraic sentinel word codes (`sentinel_base + q`) directly into `edges[v_q]` at equidistributed retained ranks $v_q = (\text{mark} + \lfloor q m / S \rfloor) \bmod m$ ($v_0 = \text{mark}$). This partitions the permutation $g^{-1}$ in place into $\le S$ disjoint open segments (zero Cong–Bader adjacency construction), evaluated concurrently by $K$ register cursors directly into expanded `4 KiB` byte chunks (`seg 0` written directly to `out[0..]`), stitched along the marked cycle via $\le S$ contiguous `memcpy`s.
- **New paid obligations:** Saving/restoring $S \le 128$ cut entries (`O(S)`), allocating a reusable `1.17 MiB` `4 KiB`-chunk pool (`MAX_CHUNKS = 300`), checking `ptr == end` / `code >= sentinel_base` per lane step, executing dummy tail-drain lane steps ($K \cdot D_{K,S} - m$), and stitching segments `1..L-1` (`<= n - |W(P_0)|` sequential bytes).
- **Nearest prior art:** `open-return/segments.rs` (`ranked`, which cut at `row % 128 == low` into `~700–1,600` segments, staged `m` `u32` codes into `8` `Vec<u32>` tapes, and ran a second serial $m$-step pass calling `append_code`); Shun–Gu–Blelloch–Fineman–Gibbons parallel list contraction; Neon Cycle 3 `BSSE`. Unlike `segments.rs` and `BSSE`, `SSMC-Open` uses $O(S)$ in-place sentinel codes (zero per-step `% 128` test on `code < 256`, zero `u32` code tape, single-pass byte expansion).

### Gate G1 — Possibility (`proved` + `verified`)

**Theorem 4.1 (Exact Disjoint Sentinel Decomposition & Universal Arena Bound — `proved`).**
Let $L \in \Sigma^n$ be *any* column ($n \ge 1$, BWT or non-BWT, single- or multi-cycle, periodic or aperiodic) with primary $0 \le p < n$, and let $(m, \text{mark}, g^{-1}, W)$ be Open's retained inverse-return bijection on $\{0, \dots, m-1\}$. Let $S_{\text{eff}} = \min(S, m) \ge 1$ and $v_q = (\text{mark} + \lfloor q m / S_{\text{eff}} \rfloor) \bmod m$ for $0 \le q < S_{\text{eff}}$:
1. *Distinctness without hashing:* Because $m / S_{\text{eff}} \ge 1$, $0 = \lfloor 0 \rfloor < \lfloor m/S_{\text{eff}} \rfloor < \dots < \lfloor (S_{\text{eff}}-1)m/S_{\text{eff}} \rfloor < m$, so $v_0, \dots, v_{S_{\text{eff}}-1}$ are pairwise distinct with $v_0 = \text{mark}$.
2. *Disjoint path cover & step bound:* Starting segment $q$ by emitting $W(v_q)$ from `saved_code[q]` and advancing $j \leftarrow g^{-1}(j)$ until $j = v_{q'} \in V_{\text{cut}}$ traces the unique minimal non-empty path $P_q : v_q \to^+ v_{q'}$ avoiding $V_{\text{cut}}$ internally (including $1$-step self-loops $g^{-1}(v_q) = v_q$, where $j = v_q$ immediately reads `sentinel_base + q` on the first loop load). Since $g^{-1}$ is a bijection, $\{P_q\}_{q=0}^{S_{\text{eff}}-1}$ are vertex- and edge-disjoint, so $\sum_{q=0}^{S_{\text{eff}}-1} |P_q| \le m$.
3. *Universal byte bound:* Every open excursion from $E = f(S)$ through $S^c$ to $S$ visits disjoint rows of $[0, n)$ (`open-return/proof.md` §1). Hence across *all* retained rows (marked and unmarked components combined),
   $$\sum_{q=0}^{S_{\text{eff}}-1} |W(P_q)| \le \sum_{j=0}^{m-1} |W(j)| \le n.$$
   Segment $0$ never exceeds `out[0..n)`, and segments $1 \dots S_{\text{eff}}-1$ require at most $\lceil n / 4096 \rceil + (S_{\text{eff}} - 1) \le 220 + 63 = 283 < 300$ chunks of `4 KiB` for any $n \le 900,000$ and $S \le 64$.
4. *Exactness:* Stitching $q_0 = 0, q_{t+1} = \text{succ\_cut}[q_t]$ until $q_{t+1} = 0$ (or `written >= n`) concatenates the exact cycle word of `mark`'s component in $g^{-1}$ while ignoring any segments on unmarked cycles; `repeat_period(n, out)` then matches `src/open.rs::emit` byte-for-byte. $\square$

**Verification (`verify_cycle1.py` & `g5_ssmc.c` — `verified`):**
- Exhaustive small cases: **`5,227` `(L, p)` pairs $\times 2$ limits = `10,454 / 10,454` PASS** (all binary $1 \le n \le 8$, all ternary $1 \le n \le 5$, every primary mark).
- Adversarial periodic / fixed-point / multi-cycle controls: **`4 / 4` PASS** in both Python and C (`"a"*500`, `"ab"*300`, non-BWT multi-cycle `"ba"*301`, `"abcde"*200`).
- Six authenticated Wikidata blocks: **`5,399,692 / 5,399,692` bytes exact** against dense LF walk.

### Gate G2 — Availability (`verified`)
`python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-carbon-word-transport/ssmc.toml` outputs **`PASS`** (`D = m / K`, `Q = m`, zero `assumed`/`open` edges, zero intermediate text-determining nodes).

### Gate G3 & G4 — Producer–Consumer Cost & Headroom
- **Producer delta:** $+O(S)$ (`64` reads/writes in `edges`, `< 0.001 ms`).
- **Consumer (`emit`) delta:** Cuts serial pointer-chase depth from $m$ to $D_{8,64} = 0.136\text{--}0.172m$ (`5.83–7.36×` shorter critical path across all 6 blocks); adds `1.17 MiB` chunk arena (`8 × 4 KiB = 32 KiB` active L1 working set) and sequential stitch `memcpy` of $\le n$ bytes.
- **Headroom (G4):** `emit` is `36–48%` of shipped Open (`PR #653`); cutting `emit` to `<= 0.65×` gives `13–17%` whole-kernel ceiling (`> 10%` G4 gate).

---

## 4. Confront: Gate G5 Paid-Cost Falsifier (`g5_ssmc.c` — **FAIL**)

Pre-registered rule (fixed in [`g5_ssmc.c`](2026-11-20-carbon-word-transport/g5_ssmc.c) prior to execution, 25 paired interleaved rounds after 3 warmup rounds on `Intel Xeon Platinum 8481C`, `gcc -O3 -march=native`): `SSMC-Open(K=8, S=64)` must satisfy (1) byte-exactness, (2) $D_{8,64} \le 0.22m$ on 6/6, (3) `emit <= 0.65×` on $\ge 5/6$ and `< 0.75×` on 6/6, (4) `total <= 0.88×` on $\ge 5/6$ and `<= 0.90×` on 6/6, (5) arena $\le 1.25\text{ MiB}$.

### Measured G5 Output (`g5_ssmc.c` — `measured`, VM diagnostic)

| Block | $m$ | $D(8,64)$ | $D/m$ | Open `emit` (ms) | `SSMC(8,64)` `emit` (ms) | `emit` Ratio | Open `total` (ms) | `SSMC(8,64)` `total` (ms) | `total` Ratio | `S4(32)` `total` | `S8(128)` `total` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 | 14,795 | `0.165×` | 0.539 | 0.655 | **`1.216×`** | 1.475 | 1.594 | `1.081×` | `1.080×` | `1.074×` |
| `block-2.L` | 112,070 | 17,045 | `0.152×` | 0.730 | 0.872 | **`1.194×`** | 1.852 | 1.981 | `1.070×` | `1.078×` | `1.064×` |
| `block-85.L` | 156,404 | 26,838 | `0.172×` | 0.936 | 1.004 | **`1.072×`** | 1.910 | 1.981 | `1.037×` | `1.060×` | `1.009×` |
| `block-143.L` | 196,841 | 26,754 | `0.136×` | 1.567 | 1.668 | **`1.065×`** | 3.426 | 3.519 | `1.027×` | `1.055×` | `1.030×` |
| `block-207.L` | 135,921 | 20,290 | `0.149×` | 0.918 | 1.021 | **`1.113×`** | 2.190 | 2.298 | `1.049×` | `1.073×` | `1.042×` |
| `block-210.L` | 204,281 | 31,253 | `0.153×` | 1.431 | 1.482 | **`1.035×`** | 2.868 | 2.890 | `1.008×` | `1.035×` | `1.006×` |

**Pre-registered Verdict:** **FAIL** (`0/6` blocks cleared criteria 3 and 4, despite clearing exactness, depth `0.136–0.172×` on `6/6`, and memory `1.176 MiB`).

---

## 5. Learn: Diagnostic Decomposition & The ROB Branch-Flush Obstruction

Why did cutting pointer-chase dependency depth by `5.8–7.4×` run `3.5–21.6%` **slower** in `g5_ssmc.c`? Rather than guessing ("maybe the CPU already overlaps 1-lane loads"), we ran controlled hardware-isolation ablations on the exact same `edges[0..m]` arrays (`Xeon Platinum 8481C`, `-O3 -march=native` — `measured`):

1. **MLP headroom on `edges[0..m]` is real and massive (`7.5–8.3×`):**
   - Pure pointer-chasing on Sattolo `m`-cycles (`351 KiB` and `797 KiB`): 1-lane takes `0.440 ms` (`4.90 ns/step`) and `1.268 ms` (`6.21 ns/step`), whereas 8-lane takes **`0.057 ms` (`7.70×`)** and **`0.169 ms` (`7.52×`)** — proving that **82–89% of `Open::emit` wall time equals raw 1-lane pointer latency**, and the core's memory hierarchy easily serves 8 concurrent loads in parallel.
   - Even on Open's actual `edges[0..m]` with 64 sentinel cuts, 8-lane walk-only (`cd >= sbase` branch only, predicted `99.95%` not-taken) runs in **`0.075 ms` (`block-0`), `0.167 ms` (`block-143`), and `0.202 ms` (`block-210`)** — **`6.0–8.3×` faster than `Open::emit`**.
2. **Where the `0.58–1.50 ms` was lost — two distinct causes (`measured`):**
   - **Cause A (Structural — The ROB Branch-Flush Obstruction):** Adding *only* a scalar length accumulation `B += (cd < 256u ? 1u : w_len[cd])` (compiled by GCC to a conditional jump `cmp $255, %edx; ja` because `w_len[cd]` is a memory load) with **zero byte writes** explodes 8-lane walk time from **`0.075 / 0.167 / 0.202 ms` to `0.210 / 0.568 / 0.541 ms` (`2.7–3.4×` slower!)**; replacing that conditional jump by an unconditional table read `B += w_len[cd]` immediately drops it back to **`0.084 / 0.194 / 0.216 ms`**.
     - *Mechanism:* On Wikidata blocks, `13.9%–22.7%` of retained steps have `code >= 256` (`thick_entries = 15.3k–44.6k`). Across $K = 8$ lanes per outer iteration, the probability that at least one lane takes the multi-byte branch is $1 - (1 - p)^8 \approx 70\%\text{--}87\%$! Every misprediction on `code < 256` (and inside variable-length `memcpy`) flushes all younger in-flight `edges[j_l]` loads from the Reorder Buffer, serialising the 8 lanes back onto the branch-recovery pipeline, while `LaneBuf` stack/pointer aliasing (`uint8_t*` without `restrict` passed by `&lb[l]` to `noinline` helpers) adds a further stack-reload barrier per lane.
   - **Cause B (Word-length concentration — positive structural finding):** Measuring the exact emitted word-length distribution of `Open::emit` shows **`92.5%–94.4%` of all retained steps emit $\le 16$ bytes** (`83.0%` `len=1`, `92.5%` `<=16` on `block-0`; `94.0%` `<=16` on `block-143`; `94.4%` `<=16` on `block-210`).

### Obstruction Recorded for `FRONTIER.md` (Obligation #1)
- **Witness:** `g5_ssmc.c` (`SSMC-Open` `K=8, S=64`: `D = 0.136–0.172m`, yet `emit = 1.035–1.216×` Open) + controlled branch ablation (`K8` walk + `cd < 256` jump = `0.568 ms` vs unconditional `w_len[cd]` = `0.194 ms` on `block-143.L`).
- **Scope:** Any single-core multi-cursor / SIMD / interleaved-lane traversal of a symbolic return map (`Open`, `segments.rs`, `BSO-4x`) whose inner loop branches per step on literal-vs-phrase symbol class (`code < 256`) or variable-length `memcpy`, or passes un-`restrict`ed `uint8_t*` lane structs by address.
- **Escape condition:** Eliminate per-step data-dependent branching on the common path via **Uniform-Width Branchless Vectorized Prefix Splicing** (`16`-byte unconditional SIMD store from a unified `[0 .. 256+words)` table with `len > 16` and sentinels folded into a single `< 6–7%` cold trap, plus scalar `restrict` lane pointers). Cycle 2 confronts this escape directly alongside producer-obligation reorientation.
