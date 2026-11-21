# Wormhole Cycle 1 — Directed Feedback Cuts, Metric-DAG Pushforward, and $\sigma$-Seed $\Phi$ Macro Schemes

*21 November 2026 · Branch `research/carbon-wormhole` · Base `origin/master` @ `a87b22d5446e1866f1ecd82452f0dc56d345d46d`*

---

## 0. Setup & Authentication Receipt

- **Baseline (`origin/master`):** `a87b22d5446e1866f1ecd82452f0dc56d345d46d` (`mibz2 0.5.8`, shipping `Open<16, INDUCE=true, QUEUE=true>` with fixed-16 `Vec16` queue writes and 16-byte `inline16` emission in `src/open.rs`).
- **Authenticated test blocks (`data/admitted-bwt` @ `41c03a5b`, `verified` against `manifest.json`):**
  - `block-0.L`: $n = 899{,}931$, $r = 51{,}184$, $p = 264{,}045$, SHA-256 `db8764224ac49801…`
  - `block-2.L`: $n = 900{,}000$, $r = 54{,}532$, $p = 320{,}807$, SHA-256 `8b47073232d1d9cf…`
  - `block-85.L`: $n = 899{,}935$, $r = 79{,}375$, $p = 325{,}088$, SHA-256 `5def63ec66c914e1…`
  - `block-143.L`: $n = 899{,}869$, $r = 108{,}335$, $p = 802{,}868$, SHA-256 `eb443421d4d7b459…`
  - `block-210.L`: $n = 899{,}959$, $r = 128{,}732$, $p = 791{,}429$, SHA-256 `dd4d0c7d03dd6bc7…`
  - `block-207.L`: $n = 899{,}998$, $r = 87{,}111$, $p = 205{,}180$, SHA-256 `7343ef019b44f0e8…`
- **Hardware framing (`bench/release-scaling/…-i9-10900k-…-0.5.8/env.json`):** The target workstation is a 10-core / 20-thread Intel Core i9-10900K (Comet Lake: `32 KiB` L1d/core, **`256 KiB` private L2/core**, **`20 MiB` shared L3** across all 10 cores). Unlike cloud Sapphire Rapids (`2 MiB` private L2), `Open 0.5.8`'s `edges[0..m)` alone (`359.5–817.1 KiB` at $m \in [89{,}874, 204{,}281]$) exceeds the i9's `256 KiB` L2 on all 6 blocks, explaining why multi-lane/wider-table designs (`#675`) won at 1 thread and lost at 8 threads under shared-L3 contention.

---

## 1. Explore & Triage: Three Chains from Cycle-Breaking to Acyclic Schedules

Cycle 1 investigates our first seed: **Kitchen Sink Cycle 9 discovered that a Directed Feedback Vertex Set (`FVS`) of `< 0.4%` of runs breaks every directed cycle of the run-overlap graph $G_{\mathrm{ov}}$, whereas Record 84 showed that $\Phi$-copying from completed output fails because `85–94%` of copies lie on cycles.** Can a chain of existing theorems turn a tiny FVS into an exact acyclic schedule cheaper than `Open 0.5.8`, or does Kitchen Sink's $2^{35}\text{–}2^{63}$ "phantom path" explosion forbid it?

We construct and triage three distinct chains from mature fields:

1. **Chain 1A (BWT-Space Train-Track FVS $\to$ Inverse-Monoid Metric-DAG Pushforward):**
   - *Link 1:* Run-length BWT $\to$ order-preserving weighted interval-overlap graph $G_{\mathrm{ov}}$ (Thurston 1988, *Bull. AMS* 19:417–431; Nishimoto & Tabei, *ICALP 2021*, LIPIcs 198:101).
   - *Link 2:* Exact degree/loop/bypass graph reduction $\to$ tiny Directed FVS $F \subset [0, r)$ (Levy & Low, *"A new algorithm for finding small cycle cutsets"*, *J. Algorithms* 9(4):470–493, 1988).
   - *Link 3:* Acyclic path-expression elimination (Tarjan, *J. ACM* 28(3):594–614, 1981) specialized to the Lawson inverse monoid of partial unit-slope interval bijections (Lawson, *Inverse Semigroups*, 1998) $\to$ exact Poincaré first-return map $T_{U_F} : U_F \to U_F$.
   - *Status:* **Deepened & confronted** (Theorems 1.1 & 1.3).

2. **Chain 1B (Text-Space $\Phi$-Cascade $\to$ $\sigma$-Seed Bidirectional Macro Scheme $\to$ Unsplit Copy-DAG Replay):**
   - *Link 1:* Weight-only accelerated Rauzy–Veech induction (`record 81`; Veech, *Ann. Math.* 115:201–242, 1982; Zorich, *Ann. Inst. Fourier* 46:325–370, 1996) $\to$ integer run-boundary $\mathrm{SA}$ samples in $1.013\text{–}1.019r$ steps.
   - *Link 2:* Permuted-LCP / $\Phi$ unit-slope theorem (Kärkkäinen, Manzini & Puglisi, *CPM 2009*, LNCS 5577:181–193; Gagie, Navarro & Prezza, *J. ACM* 67(1):2, 2020, Lemma 3.5) $\to$ Bidirectional Macro Scheme (`BMS`) in text space (Storer & Szymanski, *J. ACM* 29(4):928–951, 1982; Kempa & Prezza, *STOC 2018*, Theorem 5.4).
   - *Link 3:* Alphabet-bucket puncturing ($\le \sigma \le 96$ literal byte seeds, **zero run labels**) + weight-guarded Levy–Low FVS on $G_\Phi$ + Kahn topological `memcpy` replay directly inside `out[0..n)`.
   - *Status:* **Deepened & confronted** (Theorems 1.2 & 1.3).

3. **Chain 1C (Combinatorial Kleene Star / Symbolic Path Enumeration on $G_{\mathrm{ov}} \setminus F$):**
   - Enumerating unweighted paths on $D = G_{\mathrm{ov}} \setminus F$ before intersecting affine domains produces $2^{34.9}\text{–}2^{62.7}$ empty-domain paths (Kitchen Sink Cycle 9).
   - *Triage:* **Killed immediately** by Theorem 1.1 below — pushing *metric intervals* in Kahn order eliminates phantom paths completely without ever enumerating combinatorial paths.

### Candidate Profiles (`METHOD.md` §3a)

| Field | Chain 1A (`FVS-MetricDAG`) | Chain 1B (`Phi-SigmaBMS-DAG`) |
|---|---|---|
| **Mode** | Reorientation | Reorientation |
| **Displaced obligation** | `Open`'s thin-run queue fill (`Vec16`) and $m_{16}$-row walk (`89.9k–204.3k` hops) | Both orbit-following emission (`emit`) and Neon Round 6's `run_label_sequences` (`record 84`) |
| **Replacement principle** | Levy–Low FVS cuts $G_{\mathrm{ov}}$ to a DAG (`0.28–0.39%` runs); Kahn metric-interval pushforward computes $T_{U_F}$ in `< 2r` arrivals | Puncturing $\Phi$ at $\sigma \le 96$ bucket heads makes byte flow acyclic (`T[t] = T[Phi(t)]`); $F_\Phi$ breaks phrase cycles so $G_\Phi \setminus F_\Phi$ replays via 1 unsplit `memcpy`/phrase |
| **New paid obligations** | $O(r)$ explicit $G_{\mathrm{ov}}$ materialisation + Levy–Low heap FVS + Kahn pushforward + $|U_F|$-row walk | $1.015r$ weight-only RV steps + radix sort of $r$ boundary samples + $G_\Phi$ FVS + $|F_\Phi|$ byte evaluations + $K - |F_\Phi|$ random `out`-to-`out` `memcpy`s |
| **Nearest prior art** | Kitchen Sink Cycle 9 (which stopped at $2^{35}$ combinatorial paths without testing Kahn metric pushforward or weight-guarded FVS) | Neon Rounds 5–6 & `record 84` (which required $r$ literals + $3.4\text{–}8.7r$ peeled fragments or $O(n)$ run labels) |

---

## 2. Exact Theorems & Proofs

### Theorem 1.1 (Linear Metric-DAG Sub-Interval Law — `proved`; `verified` on `5,227` small `(L, p)` pairs + `6` blocks)

Let $(L, p)$ be any column of length $n \ge 1$ (primitive, periodic, or multi-cycle), let $G_{\mathrm{ov}} = ([0, r), E_{\mathrm{ov}})$ be its $O(r)$ run-overlap graph, and let $F \subseteq [0, r)$ be **any** Directed Feedback Vertex Set of $G_{\mathrm{ov}}$ with $r(p) \in F$. Set $U_F = \bigcup_{u \in F} D_u$ and $D = G_{\mathrm{ov}} \setminus F$. Then:
1. **Zero Phantom Paths & Exact Vertex Partition:** Pushing the outgoing intervals of $U_F$ through $D$ in Kahn topological order partitions every vertex $D_v$ ($v \in D$) into pairwise-disjoint non-empty sub-intervals covering $[0, w_v)$ exactly (`0` empty intersections, `0` gaps, on all cycles of $\mathrm{LF}$ simultaneously).
2. **Global Split Bound:** Across the entire topological sweep of $D$, the total number of interval splits is at most $|E_{\mathrm{ov}}| - r = r - 1 - |\overline{D} \cap \overline{C}|$, so the exact first-return map $T_{U_F} : U_F \to U_F$ has at most
   $$|T_{U_F}| \;\le\; |E_{\mathrm{ov}}(F, \cdot)| + (|E_{\mathrm{ov}}| - r) \;\le\; |E_{\mathrm{ov}}|$$
   unit-slope affine pieces, and total intermediate sub-interval arrivals satisfy $\sum_{v \in D} \#\mathrm{pieces}(v) \le n - |U_F|$ (measured at **`1.73r–1.95r`** across all 6 Wikidata blocks, refuting the $2^{35}\text{–}2^{63}$ phantom-path obstruction).

*Proof.* Because $\mathrm{LF} : [0, n) \to [0, n)$ is a bijection and every directed cycle of $\mathrm{LF}$ projects to a closed walk in $G_{\mathrm{ov}}$ intersecting $F$, every row $x \in [0, n) \setminus U_F$ has a unique backward origin in $U_F$ and visits each vertex $v \in D$ at most once before returning to $U_F$. By induction along Kahn topological order on $D$, incoming sub-intervals at $v \in D$ are the disjoint $\mathrm{LF}$-images of Partitioned subsets of $\mathrm{LF}^{-1}(D_v)$, hence partition $D_v$ with no empty pieces. Mapping $D_v$ onto $C_v$ splits a sub-interval at $y \in (c_v, c_v + w_v)$ only when $y \in \overline{D} \setminus \{0\}$. Since $\{C_u\}_{u=0}^{r-1}$ are pairwise disjoint on $[0, n)$, each interior split point of the common refinement $\overline{D} \vee \overline{C}$ (of which there are $|E_{\mathrm{ov}}| - r$) belongs to at most one $C_v$ and therefore splits at most one sub-interval during the entire sweep. $\square$

---

### Theorem 1.2 ($\sigma$-Seed Acyclic Byte Flow & Unsplit Copy-DAG Replay — `proved`; `verified` on `5,227` small `(L, p)` pairs + `6` blocks)

Let $(L, p)$ have marked $\Psi$-orbit $O_p = \{p, \Psi(p), \dots, \Psi^{d-1}(p)\}$ of minimal period $d \le n$ ($d = n$ on every primitive BWT), ordered by row index as $0 \le x_0 < x_1 < \dots < x_{d-1} < n$ with orbit positions $\tau(\Psi^t(p)) = t \in [0, d)$. Define $\Phi_p : [0, d) \to [0, d)$ by $\Phi_p(\tau(x_k)) = \tau(x_{(k-1)\bmod d})$, and let
$$\mathcal{S}_\sigma := \bigl\{\tau(x_k) : k = 0 \text{ or } F[x_k] \ne F[x_{k-1}]\bigr\}, \qquad |\mathcal{S}_\sigma| \le |\Sigma_{\mathrm{present}}| \le 256.$$
1. **Byte-Level Acyclicity with $\le \sigma$ Seeds (Zero Run Labels):** On $[0, d)$, $T[\tau(x_k)] = F[x_k]$ at every $t = \tau(x_k) \in \mathcal{S}_\sigma$, and $T[t] = T[\Phi_p(t)]$ for all $t \in [0, d) \setminus \mathcal{S}_\sigma$. Because $\Phi_p^{-1}$ is the single $d$-cycle $(\tau(x_0), \tau(x_1), \dots, \tau(x_{d-1}))$ and $\tau(x_0) \in \mathcal{S}_\sigma$, the punctured functional graph $([0, d) \setminus \mathcal{S}_\sigma,\, \Phi_p)$ is a **cycle-free forest of $|\mathcal{S}_\sigma|$ monochromatic directed paths** rooted at $\mathcal{S}_\sigma$.
2. **Unsplit Copy-DAG Replay:** Partition $[0, d) \setminus \mathcal{S}_\sigma$ into the $K \le r + |\mathcal{S}_\sigma|$ maximal unit-slope copy phrases $I_u = [a_u, a_u + w_u)$ ($\Phi_p(a_u + \delta) = b_u + \delta$), with dependency graph $G_\Phi$ having arc $v \to u$ iff $[b_u, b_u + w_u) \cap [a_v, a_v + w_v) \ne \emptyset$. For **any** Directed FVS $F_\Phi$ of $G_\Phi$, once $\mathcal{S}_\sigma$ and $\{I_f : f \in F_\Phi\}$ are written in `out[0..d)`, Kahn topological order on $G_\Phi \setminus F_\Phi$ reconstructs all remaining bytes via **exactly one unsplit `memcpy(&out[a_u], &out[b_u], w_u)` per phrase $u \notin F_\Phi$** (`0` fragment splits), and repeating `out[0..d)` to length $n$ yields the exact inverse BWT.

*Proof.* By `bwt::walk`, `out[t] = L[Psi^{t+1}(p)] = F[Psi^t(p)]`, so `out[\tau(x_k)] = F[x_k]`. Whenever $t = \tau(x_k) \notin \mathcal{S}_\sigma$, $k \ge 1$ and $F[x_k] = F[x_{k-1}] = \text{out}[\tau(x_{k-1})] = \text{out}[\Phi_p(t)]$. Iterating $\Phi_p$ decrements the orbit-sorted index $k \mapsto k - 1$, strictly decreasing $k$ until hitting the minimal index $k_a$ of the current $F$-symbol run, where $\tau(x_{k_a}) \in \mathcal{S}_\sigma$; hence no byte-level cycle exists. In $G_\Phi \setminus F_\Phi$, every source position $s \in [b_u, b_u + w_u)$ belongs either to $\mathcal{S}_\sigma$, to some $I_f$ ($f \in F_\Phi$), or to some $I_v$ ($v \notin F_\Phi$) with arc $v \to u$ in the DAG $G_\Phi \setminus F_\Phi$; thus when $u$ is popped in Kahn topological order, `out[b_u .. b_u + w_u)` is completely populated in physical RAM, so one contiguous `memcpy` writes `out[a_u .. a_u + w_u)` without splitting $I_u$ across predecessor boundaries. $\square$

---

### Theorem 1.3 (Hub–Spoke Cycle-Packing Conservation Law — `proved`; `verified` on `6 / 6` blocks)

Let $(G = (V, E), w)$ be any vertex-weighted directed graph and let $\mathcal{C}$ be any family of directed cycles with non-negative loads $\{\lambda_C\}_{C \in \mathcal{C}}$ satisfying $\sum_{C \ni v} \lambda_C \le w(v)$ for all $v \in V$. Then **every** Directed FVS $F \subseteq V$ satisfies
$$\sum_{v \in F} w(v) \;\ge\; \sum_{v \in F} \sum_{C \ni v} \lambda_C \;=\; \sum_{C \in \mathcal{C}} |F \cap C|\,\lambda_C \;\ge\; \sum_{C \in \mathcal{C}} \lambda_C.$$
Moreover, Levy–Low 1-in / 1-out vertex bypassing preserves minimum FVS weight **only** under the weight-dominance guard $w(v) \ge \min\bigl(\sum_{u \in N^-(v)} w(u),\, \sum_{z \in N^+(v)} w(z)\bigr)$; unguarded bypassing of a thin spoke $v$ ($w_v = 1$) between thick hubs $u \leftrightarrow v$ induces a false self-loop $u \to u$ costing $w_u \gg 1$.

---

## 3. Confrontation on Small Cases and the Six Authenticated Blocks

### 3.1 Gate G1 & Gate G2 Verification (`verified`)

- **Exhaustive small-case verification (`verify_cycle1.py`):** **PASS** on all **`5,227 / 5,227`** `(L, p)` pairs (`1 <= n <= 8` binary, `1 <= n <= 5` ternary, every mark $0 \le p < n$, covering primitive, periodic, and multi-cycle columns) for **both** Chain 1A and Chain 1B (`0` mismatches against `Plain::invert`, `0` invariant violations).
- **Authenticated 6-block byte-exact verification (`verify_cycle1.py`):** **PASS** across all `6 / 6` blocks (`5,399,692` bytes, `4` variants per block = `21,598,768` decoded bytes verified exact).
- **Gate G2 acquisition manifests (`check_manifest.py`):**
  - `chain1a_fvs_dag.toml`: **PASS** (`dependent/n-scale: fvs_return_walk: D = K, Q = K; levy_low_directed_fvs: D = r, Q = r`).
  - `chain1b_phi_bms.toml`: **PASS** (`dependent/n-scale: phi_fvs_and_seed_eval: D = r, Q = r + K; boundary_text_positions: D = S, Q = S`).

### 3.2 Exact Six-Block Census (`cycle1_results.json`, `measured` via `verify_cycle1.py`)

#### Table 1A — BWT-Space $G_{\mathrm{ov}}$ FVS, Metric-DAG Arrivals, and Cycle-Packing Lower Bounds

| Block | $r$ | `Open<16>` $m_{16}$ | `FVS_card` runs (`% r`) | `FVS_card` rows | `FVS_wt` runs | `FVS_wt` rows | Short-Cycle Pack LB (`rows`) | Metric-DAG Arrivals (`/r`) | Return Pieces $\|T_{U_F}\|$ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 51,184 | 89,874 | **151** (`0.295%`) | 92,281 | 8,111 | 80,684 | 21,436 (`1,445` cyc) | **`1.80r`** (`91,978`) | 6,905 (`0.135r`) |
| `block-2.L` | 54,532 | 112,070 | **152** (`0.279%`) | 81,213 | 10,334 | 88,900 | 25,603 (`2,311` cyc) | **`1.90r`** (`103,556`) | 8,710 (`0.160r`) |
| `block-85.L` | 79,375 | 156,404 | **221** (`0.278%`) | 89,483 | 12,760 | 83,755 | 17,690 (`2,286` cyc) | **`1.73r`** (`137,001`) | 10,937 (`0.138r`) |
| `block-143.L` | 108,335 | 196,841 | **255** (`0.235%`) | 100,326 | 21,438 | 101,968 | 28,445 (`5,707` cyc) | **`1.70r`** (`183,628`) | 18,470 (`0.170r`) |
| `block-210.L` | 128,732 | 204,281 | **405** (`0.315%`) | 96,591 | 19,258 | 88,234 | 18,531 (`3,664` cyc) | **`1.59r`** (`205,071`) | 17,143 (`0.133r`) |
| `block-207.L` | 87,111 | 135,921 | **253** (`0.290%`) | 96,911 | 15,957 | 102,222 | 29,522 (`3,644` cyc) | **`1.78r`** (`155,144`) | 13,876 (`0.159r`) |

#### Table 1B — Text-Space $G_\Phi$ $\sigma$-Seed Bidirectional Macro Scheme & Unsplit Copy-DAG Replay

| Block | Literal Seeds $\|\mathcal{S}_\sigma\|$ | Phrases $K$ | $\|E_\Phi\|$ | `FVS_card` phrases (`bytes`) | `FVS_wt` phrases | `FVS_wt` bytes | Unsplit DAG `memcpy`s |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | **89** | 51,109 | 67,094 | 588 (`103,891 B`) | 12,563 | **65,156 B** (`0.072n`) | 38,546 |
| `block-2.L` | **86** | 54,450 | 74,638 | 517 (`119,433 B`) | 16,912 | **82,321 B** (`0.091n`) | 37,538 |
| `block-85.L` | **94** | 79,288 | 103,066 | 832 (`106,438 B`) | 20,001 | **48,056 B** (`0.053n`) | 59,287 |
| `block-143.L` | **96** | 108,244 | 143,908 | 860 (`149,963 B`) | 30,170 | **109,106 B** (`0.121n`) | 78,074 |
| `block-210.L` | **96** | 128,640 | 160,223 | 1,276 (`105,351 B`) | 24,769 | **75,489 B** (`0.084n`) | 103,871 |
| `block-207.L` | **94** | 87,020 | 111,635 | 848 (`120,308 B`) | 19,951 | **74,061 B** (`0.082n`) | 67,069 |

### 3.3 Per-Worker Working Set, DRAM Traffic, and Economic Verdict vs `Open 0.5.8`

| Metric (6-block range) | `Open 0.5.8` (`master` baseline) | Chain 1A (`FVS-MetricDAG`) | Chain 1B (`Phi-SigmaBMS-DAG`) |
|---|---|---|---|
| **Per-worker working set** | `2.1–2.8 MiB` (`out` `900 KiB` + `edges` `360–817 KiB` + `queues` `~850 KiB` + `endpoint`/`inline16` `~350 KiB`) | **`2.8–5.4 MiB`** (`out` `900 KiB` + $G_{\mathrm{ov}}$ adjacency/heap `1.6–4.1 MiB` + `edges` `320–400 KiB`) | **`3.2–6.2 MiB`** (`out` `900 KiB` + RV nodes `1.2–3.1 MiB` + $G_\Phi$/FVS `1.1–2.2 MiB`) |
| **Modelled DRAM B/B (8–10 workers on `20 MiB` L3 i9)** | `~3.1–4.2 B/B` (L3 spill of `queues` + `edges` + `out`) | **`~6.4–9.8 B/B`** ($O(r)$ explicit graph + heap mutations + DAG buckets + walk) | **`~7.8–12.1 B/B`** ($1.015r$ RV + radix sort + $G_\Phi$ FVS + random `900 KiB` `out`-to-`out` reads) |
| **Acquisition cost before final stage** | $R$ stream + $2H$ RV (`1.6–5.0k` strips) + $m_{16}$ scatter | $O(r)$ merge + $O(r\log r)$ Levy–Low + `1.59–1.90r` DAG arrivals | $1.015r$ RV steps + $O(r)$ radix sort + $O(r\log r)$ Levy–Low + `48k–109k` seed hops |
| **Verdict** | Baseline to beat | **Killed economically** (mathematics survives) | **Killed economically** (mathematics survives) |

---

## 4. Learn: Where Chain 1 Breaks, and the Exact Missing Link for Cycle 2

Cycle 1 resolves Seed 1 completely:
1. **What works mathematically:** Neither combinatorial diamond paths ($2^{35}\text{–}2^{63}$, refuted by Theorem 1.1's `1.59–1.90r` metric pushforward) nor Neon Round 5–6's fragment shattering / run-label barrier (escaped by Theorem 1.2's $\le 96$-seed byte acyclicity + unsplit topological `memcpy`) blocks exact FVS-based schedules.
2. **Where the chain breaks (`Break 1`):**
   - **Witness:** Even with exact weight-guarded Levy–Low FVS, `FVS_card` (`151–405` runs) retains `81,213–100,326` rows inside thick JSON hubs, while `FVS_wt` (`8,111–21,438` runs on $G_{\mathrm{ov}}$; `12,563–30,170` phrases on $G_\Phi$) retains `80,684–102,222` rows (`48,056–109,106` bytes + `37,538–103,871` random `memcpy`s), bounded below by `17,690–29,522` rows of short ($\le 12$-step) packed cycles alone, **plus** $O(r \log r)$ explicit graph construction (`2.8–6.2 MiB` working set).
   - **Exact scope:** Any vertex-cutting feedback scheme on $G_{\mathrm{ov}}$ or $G_\Phi$ that treats runs/phrases as **indivisible vertices** of cost $w_u$.
   - **Escape condition $\implies$ Missing Link for Cycle 2:**
     A thick hub $D_u$ of width $w_u = 5{,}000$ has a self-loop in $G_{\mathrm{ov}}$ only as a *graph vertex*; as a *unit-slope affine bijection*, **Forman–Sköldberg Algebraic Discrete Morse Theory over $\mathbb{Z}\langle\Sigma^*\rangle$ / Agol–Hass–Thurston Interval Pairing** eliminates it with **`0` retained rows**!
     Yet `Open 0.5.8` stops that Morse collapse at `LIMIT = 16`, leaving `52.5%–69.4%` of its `89.9k–204.3k` retained rows in multi-row runs ($2 \le w \le 16$) and overflowing the i9-10900K's `256 KiB` L2 cache (`360–817 KiB` `edges`).
     **Cycle 2** therefore follows this exact break into **Filtered Algebraic Morse Theory (Sköldberg 2006; Mischaikow–Nanda 2013 / Perseus 6; Gel'fand–Retakh Quasideterminants 1995)**: what happens when we push the Morse filtration from $\Lambda = 16$ all the way down to $\Lambda \in \{1, 2, 3, 4\}$ so `edges` shrinks toward the i9's `256 KiB` L2 cache, and what exact algebraic obstruction appears in the Fox-word boundary dictionary?

---

## 5. Reproducibility

```bash
BLOCKS_DIR=/home/antigravity/admitted-bwt python3 -u docs/journal/2026-11-21-wormhole-cycle1-fvs-macro-duality/verify_cycle1.py
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-21-wormhole-cycle1-fvs-macro-duality/chain1a_fvs_dag.toml
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-21-wormhole-cycle1-fvs-macro-duality/chain1b_phi_bms.toml
```

