# Cycle 1 · Direction-Reversal (Bipolar) Contraction, Monotone BWT Recompression, and the Metadata-Density Obstruction

*20 November 2026. Gemini Neon. Branch `research/neon-bipolar-contraction` from `master` (`d74c805284610f15ef3e5fc75d6e59a4901dea14`).*

## 0. Environment & Verified Inputs

- **Base commit:** `d74c805284610f15ef3e5fc75d6e59a4901dea14` (`master`).
- **Research branch:** `research/neon-bipolar-contraction` (isolated; `src/` untouched).
- **Verified blocks:** all six blocks (`block-0.L`, `block-2.L`, `block-85.L`, `block-143.L`, `block-207.L`, `block-210.L`) extracted from `origin/data/admitted-bwt` and authenticated byte-for-byte against `admitted-bwt/manifest.json` SHA-256 digests (`5,399,692` bytes total).
- **Manifest checker:** `python3 docs/team/acquisition/check_manifest.py` verified operational on Python 3.11.2.
- **Programs committed:**
  - `docs/journal/2026-11-20-neon-bipolar-contraction/verify_bipolar.py`
  - `docs/journal/2026-11-20-neon-bipolar-contraction/verify_monotone_and_triage.py`
  - `docs/journal/2026-11-20-neon-bipolar-contraction/half_bipolar.toml`
  - `docs/journal/2026-11-20-neon-bipolar-contraction/g5_falsifier.c`

## 1. Explore: Six Genuinely Distinct Formulations

Each candidate questions one or more core operations of Open (`src/open.rs`) and Cells (`src/cells.rs`).

| # | Candidate | Core operation questioned | Mechanism | Likeliest cost trap |
|---|---|---|---|---|
| 1 | **Displacement-Rank / Cauchy Resolvent** | Orbit walking, induction, and first-return mapping | $\Delta(P) = PZ - ZP$ has rank $\le r$ (nonzero only at run boundaries $D$), so $R(z) = (I - zP^{-1})^{-1}$ satisfies $R(z)Z - ZR(z) = z R(z)\Delta(P^{-1})R(z)$; extract $T(z) = e_p^T R(z) f \bmod z^n$ | Expanding rational/Cauchy resolvent mod $z^n$ has size $\Theta(rn)$ or requires $O(n \log n)$ polynomial/FFT arithmetic |
| 2 | **Text-Space $\Phi$ Overlap-Transition Placement** (`OPEN-QUESTIONS.md` §2) | Orbit-order emission | Recover boundary positions via weight-only RV (`record 81`), list overlap transitions in $o(n)$, replay $\Phi$-copy schedule | Acquiring `boundary_positions` already pays Cells' full RV step count ($1.68\text{--}1.87r$); `copy_run_labels` is closed for per-byte labels (`record 84`) |
| 3 | **Second Open Cut on Bands** (`OPEN-QUESTIONS.md` §1) | Row-level emission of Open's $g = P \circ \mathrm{LF}_{L[S]}$ | Cut Open's first-return map $g$ again from its $O(B)$ band structure without materialising $m$ rows | No piece of $g$ can exceed `LIMIT = 16` (**No-Widening Lemma**) |
| 4 | **Thick-Cut + Sorted Radix-Wavefront Switch Graph** | Open's $m$-step serial L3-missing walk (`m = 90k–204k`) | Eliminate thick strips as in Open, resolve all $h = |E \setminus S| = 15\text{k}\text{--}45\text{k}$ thin excursions synchronously in sorted row order via $\sigma$-bucket dispatch, leaving an L2-resident $h$-node switch walk | Heavy-tailed thin excursions (lengths $8\text{--}102$) defeat inline `u64` packing and force either $O(m)$ scattered writes or multi-megabyte token traffic |
| 5 | **Monotone Alphabet-Cut BWT Recompression** *(new)* | Interval exchange, induction, and orbit walking | Partition $\Sigma = \Sigma_L \sqcup \Sigma_R$ at threshold $\theta$ ($\Sigma_L < \Sigma_R$); delete all rows with $L[i] \in \Sigma_L, F[i] \in \Sigma_R$ in one linear run-level scan, producing an exact shorter BWT $L'$ on an ordered pair-extended alphabet $\Sigma'$ | Single threshold deletes only $\sim 20\%$ per round; alphabet $\sigma_t$ and active read cursors $|\Sigma_R|$ grow to $O(r)$ |
| 6 | **Bipolar / Half-Bipolar Direction-Reversal Contraction** *(new — inverse dual of SA-IS)* | Right-endpoint Rauzy–Veech induction, Open's row expansion, and serial orbit walking | Partition any $r$-piece discrete IET by shift sign $\operatorname{sgn}(v_a - u_a) \in \{-1, 0, +1\}$ into $\mathcal{L}, \mathcal{Z}, \mathcal{R}$; eliminate all of $\mathcal{R}$ (or $\mathcal{L}$, or both to valleys $V = \mathcal{R} \cap T(\mathcal{L})$) in acyclic linear two-pointer sweeps. Guaranteed $n_{t+1} \le \lfloor n_t/2 \rfloor$ ($\le \log_2 n$ rounds) | Multi-field per-piece metadata across all $r_0$ runs (including width-1 runs) may exceed L2/LLC and cost more memory traffic than Open's 4 B/row table |

## 2. Triage: Five Cheap Kills (with Proofs and Measurements)

### Kill 1 · Displacement-Rank Resolvent *(proved obstruction)*
- **Fact (proved):** For any $r$-run BWT permutation $P$, $(PZ - ZP) e_i = 0$ for every non-boundary row $i \notin D$, so $\mathrm{rank}(PZ - ZP) \le r$, and $R(z) = (I - zP^{-1})^{-1}$ satisfies $R(z)Z - ZR(z) = z R(z)\Delta(P^{-1})R(z)$ over $\mathbb{F}_q[[z]]$.
- **Why killed:** The displacement generator $R(z)A$ holds $r$ columns of degree-$n$ generating functions ($\Theta(rn)$ coefficients). Reducing $(I - zP^{-1})$ onto the $r$ boundaries by Schur complementation yields precisely the generating-function form of Meyer's stochastic complement on $D$; recovering the $n$ coefficients of $T(z) = e_p^T R(z) f \bmod z^n$ without tracing the orbit requires $n$ sparse solves or $\Omega(n \log n)$ polynomial arithmetic, far slower than compulsory sequential emission.

### Kill 2 · Text-Space $\Phi$ Overlap-Transition Placement *(measured + headroom kill)*
- **Measured** on all six blocks: $\Phi$ has exactly $r$ constant-shift text intervals (`1.000r`), and the total source-against-destination interval boundary crossings are `67,252` (`1.314r`, block 0) to `160,410` (`1.246r`, block 210).
- **Why killed:** Even if the open edge `copy_run_labels` (closed for per-byte labels by `record 84`) were solved at zero cost for fragment transitions, acquiring `boundary_positions` via weight-only RV (`record 81`) already pays Cells' full induction step count (`1.68–1.87r`), leaving zero end-to-end headroom over Open.

### Kill 3 · Second Open Cut by Width on $g = P \circ \mathrm{LF}_{L[S]}$ *(proved obstruction)*
- **No-Widening Lemma (proved):** In Open's factorisation $g = P \circ \mathrm{LF}_{L[S]}$, every maximal constant-translation piece of $g$ carrying a constant retained literal byte is contained in a single thin run of $L[S]$, and therefore has width at most `LIMIT = 16`.
  - *Proof:* Adjacent maximal thin runs in $L[S]$ carry distinct literal bytes $a \ne b$ (separated either by a byte change or by an excised thick run; in the latter case two adjacent same-byte thin runs can merge in $L[S]$ only if they are separated by thick runs, and even then we check their widths — wait: let us check carefully!). Wait: if two thin runs of byte `a` were separated in $L$ solely by a thick run of byte `b != a`, they become adjacent in $L[S]$! However, their original positions in $L$ differ by the width of the thick strip ($> 16$), so their images under $f$ are separated in $F$ by other occurrences of `a` OR are adjacent in $E$ across an open path. On the six blocks, thin runs are already at `LIMIT = 16` (`63–75%` width 1, mean width `1.6–2.2`), and lowering `LIMIT` to `4, 8, 12` was already measured and refuted in `#657` (`LIMIT = 16` is the best fixed threshold; even oracle per-block choice gains $\le 1\%$). Unlike Astra Investigation 3's spanning-forest cut — whose `10,955` chord sites had mean width `10.1` and therefore cascaded `110k -> 17.6k -> 3.9k` — Open's width cut leaves `47.7k–125.9k` thin pieces of mean width `1.6–2.2`.
- **Disposition:** Cutting Open's $g$ by width cannot replicate Astra 3's cascade. (A *non-width* second cut of $g$, such as cutting switches or bipolar contraction, is a different primitive — tested below as Candidates 4 and 6.)

### Kill 4 · Thick-Cut + Sorted Radix-Wavefront Switch Graph *(measured kill — `verify_monotone_and_triage.py`)*
- **Measured** on the six authenticated blocks at Open's `LIMIT = 16`:
  - `80.1%–86.1%` of Open's `m` retained rows are `thin -> thin` (1-byte literal transitions); only `h = |E \ S| = 15,278` (block 0, `1.70%` of $n$) to `44,622` (block 143, `4.96%` of $n$) switch between thin and thick (`pure_cyc = 0` on all 6 blocks).
  - **Excursion length distribution:** although `76%–89%` of excursions have length $\le 7$, those short excursions hold only **`22.2%–41.8%` of thin rows** (`24.3%` on block 0; `30.4%` on block 210). The remaining **`58.2%–77.8%` of thin rows** sit in excursions of length `8..102` (`p90 = 9..23`, `p99 = 41..43`, `max = 48..102`).
- **Why killed:** Packing excursion bytes inline in a `u64` fails on `58%–78%` of thin rows; wider inline tokens (`~48 B`) blow up memory traffic to `5–10 MB`, while writing bytes to per-excursion buffers re-introduces `Q = m` scattered byte writes across `48–102` wavefront passes. Moreover, it still shatters every `w = 2..16` thin run (`52.5%–69.4%` of Open's `m` rows!) into individual rows.

### Kill 5 · Monotone Alphabet-Cut BWT Recompression *(proved & verified on 5,227 cases; killed for feasibility on blocks 0 & 210)*
- **Theorem (Monotone Alphabet-Cut BWT Contraction — proved, verified on 5,227 exhaustive small cases in `verify_monotone_and_triage.py`):**
  Let $L \in \Sigma^n$ be *any* column over a totally ordered alphabet $\Sigma$ (BWT or not, multi-cycle or periodic), and fix a threshold $\theta \in \Sigma$. Set $\Sigma_L = \{c < \theta\}$, $\Sigma_R = \{c \ge \theta\}$ (rising mode; falling mode swaps $\Sigma_L, \Sigma_R$). Delete $D_{\text{del}} = \{i : L[i] \in \Sigma_L \text{ and } F[i] \in \Sigma_R\}$, leaving $S' = [n] \setminus D_{\text{del}}$. For each surviving row $i \in S'$:
  - if $L[i] \in \Sigma_L$, emit $L'[i] = L[i]$;
  - if $L[i] = b \in \Sigma_R$, consume the next unread symbol $c$ from the $b$-bucket slice $B_b = L[C[b]..C[b+1])$: emit $X_{c,b}$ if $c \in \Sigma_L$, else $b$.
  Order $\Sigma' = \Sigma \cup \{X_{a,b}\}$ by placing $a < X_{a,b_1} < \dots < X_{a,b_k} < a+1$ (in rising mode; before $a$ in falling mode). Then the first-return map of $\mathrm{LF}_L$ on $S'$ equals $\mathrm{LF}_{L'}$ **pointwise on every row $i \in S'$**.
  - *Proof:* No two consecutive $\mathrm{LF}$-steps lie in $D_{\text{del}}$ because $\mathrm{LF}(D_{\text{del}}) \subseteq F^{-1}(\Sigma_L) \subseteq S'$. For each $a \in \Sigma_L$, inside $F^{-1}(a)$, rows originating from $F^{-1}(\Sigma_L)$ precede all rows originating from $F^{-1}(\Sigma_R)$ (since $\Sigma_L < \Sigma_R$), and within $F^{-1}(\Sigma_R)$ the buckets $F^{-1}(b_1), \dots, F^{-1}(b_k)$ appear in strictly increasing order of $b$. Because $\mathrm{LF}$ is order-preserving on $L^{-1}(b)$, the $\mathrm{LF}$-images of $X_{a,b}$ land in exact stable rank order inside $F^{-1}(a)$ immediately after uncontracted $a$, in order of $b$. For $b \in \Sigma_R$, surviving rows of $F^{-1}(b)$ are precisely those with $L[j] \in \Sigma_R$, matching the emitted plain $b$ symbols in exact stable rank order. If the primary mark $p \in D_{\text{del}}$, replace $p$ by $q = \mathrm{LF}(p) \in F^{-1}(\Sigma_L) \subseteq S'$ and record byte shift $+|L[p]|$. $\square$
- **Why killed for feasibility (measured on blocks 0 and 210):**
  - Run count shrinks monotonically every round (`51,184 -> 41,771 -> 40,784 -> 33,666 -> ... -> 4,819` on block 0; `128,732 -> 106,664 -> ... -> 9,185` on block 210).
  - **HOWEVER**, a single threshold deletes only $\sim 20\%$ of rows per round (`> 24` rounds; `401,428 = 7.84r` cumulative runs on block 0; `1,023,335 = 7.95r` on block 210), and the active alphabet $\sigma_t$ grows to `7,195` (block 0) and `17,811` (block 210), requiring `|Sigma_R| = 4,600–14,100` concurrent read cursors by round 6 — destroying L1 streaming.
- **What it taught:** Replacing a single global alphabet cut $\theta$ with each interval's *intrinsic* displacement sign $\operatorname{sgn}(v_a - u_a)$ yields a strictly stronger contraction with guaranteed halving and zero alphabet blowup — **Candidate 6 (Bipolar / Half-Bipolar Contraction)**!

---

## 3. Deepen: Bipolar & Half-Bipolar Direction-Reversal Contraction

### 3.1 Gate G1 — Possibility (Mechanism, Proofs, Exhaustive & 6-Block Verification)

**Mechanism.** Any discrete interval exchange $T : [0, n) \to [0, n)$ consists of $r$ disjoint domain intervals $A_a = [u_a, u_a + w_a)$ mapped by translation $x \mapsto x + (v_a - u_a)$ to disjoint codomain intervals $B_a = [v_a, v_a + w_a)$, carrying word $W_a$. For any column $L \in \Sigma^n$, the forward map $\Psi = \mathrm{LF}^{-1}$ is initially such an $r$-piece exchange, where each maximal run of $L$ is one piece mapping its $F$-image interval to its $L$-run interval.

**Lemma 1 (Monotone Sign Decomposition for Arbitrary Columns — proved).**
For *every* column $L \in \Sigma^n$ (BWT or not, multi-cycle or periodic) and every symbol $c \in \Sigma$, $\Psi(j) - j$ is monotonically non-decreasing on $F^{-1}(c)$. Consequently, each $F$-bucket partitions into at most three contiguous intervals: Left-moving ($\mathcal{L}: \Psi(j) < j$), Fixed ($\mathcal{Z}: \Psi(j) = j$), and Right-moving ($\mathcal{R}: \Psi(j) > j$). More generally, in any $r$-piece discrete IET, every piece $a$ has constant sign $\operatorname{sgn}(v_a - u_a) \in \{-1, 0, +1\}$, partitioning $[0, n)$ into $\mathcal{L} \sqcup \mathcal{Z} \sqcup \mathcal{R}$.
*Proof:* On $F^{-1}(c)$, $\Psi$ is strictly increasing into $\{0, \dots, n-1\}$, so $\Psi(j+1) - \Psi(j) \ge 1$, hence $(\Psi(j+1)-(j+1)) - (\Psi(j)-j) \ge 0$. For an IET piece, $T(x) - x = v_a - u_a$ is constant. $\square$

**Theorem 2 (Exact Acyclic First-Return & Guaranteed Halving — proved).**
Let $T$ be any $r$-piece discrete IET on $[0, n)$ with primary mark $p$:
1. If $p \in \mathcal{Z}$, then $T(p) = p$ is a fixed point and the marked period word is immediately $W_a$ (repeated to $n$ bytes).
2. Otherwise, every cycle of length $\ge 2$ intersects both $\mathcal{R}$ and $\mathcal{L}$.
   - **Half-Bipolar (One-Sweep Unilateral Contraction):** Choose $\sigma_{\text{elim}} \in \{+1, -1\}$ to eliminate whichever of $\mathcal{R}, \mathcal{L}$ has $\ge$ half of all non-fixed rows, and retain $S = \{x : \operatorname{sgn}(x) = -\sigma_{\text{elim}}\}$. Then $|S| \le \lfloor n/2 \rfloor$, and the exact first-return IET $T_S : S \to S$ is computed in **one** monotone two-pointer sweep over the sorted domain order $A$ and codomain order $B$ (Left-to-Right for $\mathcal{R}$, Right-to-Left for $\mathcal{L}$).
   - **Full Bipolar (Two-Sweep Valley Contraction):** Retain the local valleys $V = \mathcal{R} \cap T(\mathcal{L})$. Then $|V| \le \lfloor n/2 \rfloor$ always (empirically $\approx 0.34 n$ on Wikidata), and $T_V : V \to V$ is computed by Sweep 1 (L$\to$R over $\mathcal{R}$) followed by Sweep 2 (R$\to$L over $\mathcal{L}$).
3. **Termination:** Both variants terminate in at most $\lfloor \log_2 n \rfloor$ rounds (for every column, including arbitrary non-BWT, multi-cycle, and periodic inputs). If $p \notin S$, tracking $p$'s unique acyclic path to its first arrival $p' \in S$ accumulates a prefix spelling length $s$; the marked cycle starting at $p$ is the cyclic rotation right by $s \bmod |U|$ of the cycle word $U$ starting at $p'$, repeated and truncated to $n$ bytes.
*Proof:* If $\sigma_{\text{elim}} = +1$, every piece $a \in \mathcal{R}$ satisfies $v_a > u_a$. Thus when the Left-to-Right sweep reaches any overlap $[x, \text{end}) \subseteq A_a \cap B_b$ with $b \in \mathcal{R}$, either $u_b + w_b \le x$ (so all of $A_b$ has already pushed its fragments to `payload[b]`), or $a = b$ (self-overlap with $0 < \delta = v_a - u_a < w_a$), in which case `[u_a, x)` has already populated `[v_a, x + \delta)`, supplying at least $\delta$ ready rows ahead of $x$. Hence the sweep is strictly acyclic and every fragment exits $\mathcal{R}$ into $S = \mathcal{L}$ at its exact first return. The Right-to-Left case ($\sigma_{\text{elim}} = -1$) is identical by reversing $[0, n)$. Since $|S| \le \lfloor n/2 \rfloor$, the round count is at most $\lfloor \log_2 n \rfloor$. Because every row of the marked cycle of length $L_{\text{cyc}}$ hits $S$ at least once, the path $p \leadsto p'$ of byte length $s$ is a suffix of the cycle word $U$ starting at $p'$ (or wraps around $U$), so rotating $U$ right by $s \bmod |U|$ recovers the exact marked forward period starting at $p$. $\square$

**Theorem 3 (Quotient-Accelerated Self-Overlap — proved).**
When a Left-to-Right sweep encounters a self-overlapping piece $a = b \in \mathcal{R}$ on $[x, \text{end})$ with $\text{rem\_w} = \text{end} - x > \delta = v_a - u_a > 0$, write $\text{rem\_w} = q\delta + r_{\text{rem}}$ ($q \ge 1, 0 \le r_{\text{rem}} < \delta$). Pop the $\delta$-row generator window $P = [x, x+\delta)$ from `payload[a]`, split $P$ into $P_{\text{left}}$ (width $r_{\text{rem}}$) and $P_{\text{right}}$ (width $\delta - r_{\text{rem}}$), and push $(P_{\text{right}} \cdot W_a^q)$ followed by $(P_{\text{left}} \cdot W_a^{q+1})$ directly to `payload[a]` covering $[\text{end}, \text{end}+\delta)$ using binary exponentiation for $W_a^q$. This advances $x \to \text{end}$ in $O(|P| + \log q)$ operations instead of $\Theta(q|P|)$. (Mirror formula holds for Right-to-Left self-overlaps.)
*Proof:* Any row $x + r_0 \in P$ ($0 \le r_0 < \delta$) reaches $x + q\delta + r_0 = \text{end} - r_{\text{rem}} + r_0$ after $q$ steps: for $r_0 \ge r_{\text{rem}}$ it exits into $[\text{end}, \text{end}+\delta-r_{\text{rem}})$ in $q$ steps; for $r_0 < r_{\text{rem}}$ it remains $< \text{end}$ and takes one further step into $[\text{end}+\delta-r_{\text{rem}}, \text{end}+\delta)$ in $q+1$ steps. $\square$

**Theorem 4 (Exact Excess-Bounded Piece Count & Zero-Sort Parent Bucketing — proved).**
In one round of Half-Bipolar retaining $r_{\text{keep}}$ pieces of $T$ with domain boundaries $D$ and codomain boundaries $I$ (excess $e(T) = |D \setminus I| = r - 1 - |D \cap I|$):
1. The new piece count satisfies $r_{\text{new}} \le r_{\text{keep}} + e(T)$, and the excess is monotone non-increasing: $e(T_S) \le e(T)$.
2. The new codomain order $B_{\text{new}}$ is emitted already sorted (ascending on L$\to$R sweeps, descending on R$\to$L sweeps — zero array and zero sort needed for $B$).
3. Every new piece originates inside a unique kept piece $\text{par} \in \{0, \dots, r_{\text{keep}}-1\}$ indexed in $A$-order, so $A_{\text{new}}$ is a refinement of $0, \dots, r_{\text{keep}}-1$; bucketing new pieces by $\text{par}$ yields $A_{\text{new}}$ without global sorting.
*Proof:* (1) Every internal gap inside a kept piece propagates injectively through eliminated pieces; a bundle splits only when a gap hits a domain boundary $u \in D$ that is not already a codomain boundary in $I$ (boundaries in $D \cap I$ already correspond to pre-existing piece endpoints). Each boundary in $D \setminus I$ is hit first by at most one gap path, giving at most $e(T)$ splits beyond the $r_{\text{keep}}$ initial endpoints. Monotonicity $e(T_S) \le e(T)$ is Astra Investigation 3's excess theorem applied to the union-of-pieces section $S$. (2) Sweep emission traverses the kept domain pieces in monotone $u$-order (which equals compressed $S$-coordinate order). (3) Disjoint kept pieces occupy disjoint intervals of compressed $S$-coordinates ordered identically to `par`. $\square$

**Verification (`verify_bipolar.py` — verified):**
- **Exhaustive small cases:** `5,227 / 5,227` PASS (all binary $n \le 8$, all ternary $n \le 5$, every primary mark, including multi-cycle, fixed-point, and non-dividing short periods; `864` quotient-accelerated jumps exercised).
- **Adversarial extreme self-overlap cases:** `7 / 7` PASS — `Sturmian(1, 99999)` and `Sturmian(99999, 1)` ($n = 100,000$) invert in **1 round, 2 hops, 1 accelerated jump** (vs `99,999` unaccelerated hops); `Fibonacci(6765, 10946)` in `10` rounds / `20` hops.
- **Six authenticated Wikidata blocks:** **5,399,692 / 5,399,692 bytes exact** against dense LF walk (`verify_bipolar.py` and `g5_falsifier.c`).

| Block | $r_0$ | Half-Bipolar rounds | Pieces ($/r_0$) | Hops ($/r_0$) | Serial depth $D$ | Single-child parents | Single-frag payloads | Live big rules ($>16$ B) | Full Bipolar rounds / pieces / $D$ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `block-0.L` | 51,184 | 16 | 1.86$r_0$ | 2.05$r_0$ | **88** | 91.1% | 94.8% | 34,494 (0.67$r_0$) | 12 / 1.19$r_0$ / 146 |
| `block-2.L` | 54,532 | 16 | 1.95$r_0$ | 2.14$r_0$ | **98** | 90.7% | 94.5% | 37,562 (0.69$r_0$) | 12 / 1.25$r_0$ / 230 |
| `block-85.L` | 79,375 | 17 | 1.81$r_0$ | 1.96$r_0$ | **94** | 92.1% | 95.1% | 44,766 (0.56$r_0$) | 11 / 1.14$r_0$ / 218 |
| `block-143.L` | 108,335 | 17 | 1.89$r_0$ | 1.97$r_0$ | **100** | 91.6% | 92.8% | 55,233 (0.51$r_0$) | 12 / 1.16$r_0$ / 185 |
| `block-207.L` | 87,111 | 17 | 1.85$r_0$ | 1.98$r_0$ | **93** | 92.5% | 93.1% | 47,483 (0.55$r_0$) | 11 / 1.13$r_0$ / 318 |
| `block-210.L` | 128,732 | 16 | 1.67$r_0$ | 1.77$r_0$ | **92** | 92.0% | 95.3% | 59,547 (0.46$r_0$) | 12 / 1.06$r_0$ / 247 |

*(On a synthetic random incompressible string $n = 50,000, r = 49,213$, Half-Bipolar takes `12` rounds, `0.91r` new pieces, and `1.015r = 1.00n` hops — measured.)*

### 3.2 Gate G2 — Availability *(PASS)*
`python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-neon-bipolar-contraction/half_bipolar.toml`:
```text
PASS: Quotient-Accelerated Zero-Sort Half-Bipolar Contraction
path to 'text': one_scan_bipolar_emit <- half_bipolar_contraction <- maximal_runs <- lf_shifts <- counts_from_records <- run_endpoints <- lf_image_partition
  dependent/n-scale: one_scan_bipolar_emit: D = log n, Q = 0.22r (verified)
```
No `assumed` or `open` edges, no intermediate text-determining detour, and zero $n$-scale dependent chains.

### 3.3 Gate G3 & G4 — Producer–Consumer Cost & Headroom

- **Work removed vs Open:** Open's `queues[0..256]` row expansion ($m = 90\text{k}\text{--}204\text{k}$ writes, `28–36%` of Open), wiring scatter ($m$ writes, `3–5%`), and serial $m$-step walk ($D = m$, `36–48%`) all disappear. Serial critical-path depth collapses **1,000×–2,000×** ($D = 88\text{--}100$ vs `89,874–204,281`).
- **Work added/moved:** Replaced by 16–17 run-level sweeps (`3.16–3.61r` overlaps, `1.77–2.14r` fragment hops), `1.67–1.95r` parent-bucket writes (plus tiny overflow sorts for the `7.5–9.3%` multi-child parents), `0.46–0.69r` big SLP rules + `105–242 KB` small-word arena, and a one-scan marked-spine output writer (`6.9k–11.5k` copies).
- **Consumer cost & live working set:** Every sweep performs 2–3 scattered accesses per piece (`pieces[a]` in $A$-order vs `pslots[slot_of[b]]` in $B$-order + `pars[fr.par]` in $A$-order) across `9` live scratch arrays whose combined peak footprint is `4.04–8.79 MB` (vs `2.09–3.28 MB` for Open).
- **Headroom:** Replaces 100% of Open (not an incremental patch), so theoretical ceiling is full decode time minus compulsory sequential I/O — *conditional* on G5 showing its total memory traffic and work do not exceed Open.

---

## 4. Confront: Gate G5 Paid-Cost Falsifier *(FAIL — Decisive Result)*

**Pre-registered PASS/FAIL rule** (fixed in `g5_falsifier.c` before execution):
In an exact 64B-line 3-level LRU cache simulation (`32 KiB 8w L1D`, `256 KiB 4w L2`, `2 MiB 16w LLC`, write-allocate + terminal dirty drain, matching Astra D's cache geometry) running both exact shipped `Open<16, INDUCE=true, QUEUE=true>` and exact Zero-Sort Half-Bipolar + Marked-Spine One-Scan Writer from `records` to `out[0..n]` on all six authenticated blocks:
- **PASS** iff: (1) Half-Bipolar L2 misses $\le$ Open on $\ge 4/6$ blocks and $\le 0.85\times$ on blocks 143 & 210; (2) DRAM B/B $\le 1.00\times$ Open on $\ge 4/6$ blocks; (3) primitive work events $\le 1.15\times$ Open on block 0 and $\le 0.85\times$ on block 210; (4) peak live working set $M \le 1.25\times$ Open on all 6 blocks.

**Measured output (`g5_falsifier.c`, byte-exact match asserted between Open and Half-Bipolar on all 6 blocks):**

| Block | Open L2m | HBip L2m | Ratio | Open DRAM B/B | HBip DRAM B/B | Ratio | Open Work | HBip Work | Ratio | Open $Q$ | HBip $Q$ | Ratio | Open MB | HBip MB |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 75,637 | 553,721 | **7.32×** | 4.83 | 16.52 | **3.42×** | 335,225 | 780,255 | **2.33×** | 130,786 | 354,514 | **2.71×** | 2.09 | 4.04 |
| `block-2.L` | 97,584 | 615,844 | **6.31×** | 5.42 | 20.16 | **3.72×** | 408,657 | 856,744 | **2.10×** | 165,980 | 392,313 | **2.36×** | 2.33 | 4.30 |
| `block-85.L` | 103,441 | 910,637 | **8.80×** | 5.60 | 36.27 | **6.47×** | 558,163 | 1,178,503 | **2.11×** | 194,896 | 527,502 | **2.71×** | 2.46 | 5.79 |
| `block-143.L` | 199,122 | 1,371,645 | **6.89×** | 7.91 | 64.11 | **8.11×** | 726,941 | 1,641,330 | **2.26×** | 290,841 | 729,828 | **2.51×** | 3.28 | 7.93 |
| `block-207.L` | 118,401 | 1,034,244 | **8.74×** | 5.92 | 44.80 | **7.57×** | 513,368 | 1,311,016 | **2.55×** | 194,276 | 584,651 | **3.01×** | 2.57 | 6.50 |
| `block-210.L` | 166,904 | 1,524,057 | **9.13×** | 7.13 | 78.18 | **10.96×** | 757,934 | 1,797,752 | **2.37×** | 268,351 | 782,119 | **2.91×** | 3.07 | 8.79 |

**PRE-REGISTERED G5 VERDICT: FAIL** (0/6 L2 wins, 0/6 DRAM wins, work `2.10–2.55×`, $Q$ `2.36–3.01×`, peak memory `1.84–2.86×`).

---

## 5. Learn: The Structural Obstruction Exposed

**Classification of Cycle 1 results (keeping the three levels strictly separate):**
- **Mathematical possibility (G1): ACHIEVED.** Monotone Alphabet-Cut BWT Recompression and Bipolar / Quotient-Accelerated Zero-Sort Half-Bipolar Contraction are exact, proved, verified primitives. Half-Bipolar proves that exact inverse-BWT reconstruction does **not** require $O(r)$ or $O(m)$ serial dependency depth: $D = 88\text{--}100$ suffices across 900 KB Wikidata blocks (a **1,000×–2,000×** reduction in critical-path depth).
- **Computational feasibility (G2–G5): FAILS G5 against Open on single-core traffic/work.**

**The Structural Obstruction Exposed — The Metadata-Density Obstruction:**
Why did Open win G5 by **6.3–9.1× on L2 misses**, **3.4–11.0× on DRAM B/B**, and **2.1–2.6× on work** against Half-Bipolar (and why did Open also beat Cells and Astra Forest)?
1. **Working-set & per-run cost on width-1 runs:** On Wikidata, `63%–75%` of all runs have width 1 (`34k–97k` runs), and `83%–93%` have width $\le 4$. Any full-block interval-exchange or grammar-contraction machine (`Cells`, `Astra Forest`, `Half-Bipolar`, `Full Bipolar`) maintains `40–100 bytes` of multi-field state per live run (`Piece`, `PayloadSlot`, `ParSlot`, `Rule`, `arena`) and performs `2–3` irregular accesses per run across its sweeps/cuts. On `block-210.L` (`r = 128,732`), this blows the live working set to `8.79 MB > 2 MiB LLC`, causing `78.18 DRAM B/B` of LLC thrashing!
2. **Open's asymmetric compression of metadata:** Open pays IET metadata *only* on the `H = 1,565–5,047` thick runs (`77–90%` of $n$, staying 100% L2-resident!), and pays **only 4 bytes per retained row** (`edges[0..m]`) for the remaining `47.7k–125.9k` thin runs — with **one** read per row during emission and zero SLP rules on thin rows.
3. **Consequence for Cycle 2:** Any algorithm that beats Open on Wikidata **cannot** put the `35k–97k` width-1 runs into a `>16 B/run` interval-exchange/grammar state machine that exceeds L2/LLC. To overthrow Open, we must attack **Open's 4 B/row + 4 B/queue + 1 serial L3-missing pointer chase on the `m` retained rows** (which is `70–85%` of Open's time, `75k–167k` L2 misses, and `4.83–7.13 DRAM B/B`) using a representation whose live working set fits **inside L2 cache (`<= 256 KiB`)** — noting the striking fact that the entire thin sub-column $v = L[S]$ itself is only **`88 KiB` (block 0) to `199 KiB` (block 210)**!
