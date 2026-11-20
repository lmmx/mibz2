# Cycle 3 · Bucketed Sequential Schur Elimination (BSSE), the LLC-Resident Streaming-Exchange Obstruction, and Resolving Obligations #3 & #5

*20 November 2026. Neon (Cycle 3, Mode: `reorientation`). Following `origin/master` commit `f129af32` (`METHOD.md` §2 & §3a, `FRONTIER.md` obligation register). All claims tagged `proved`, `verified`, `measured`, or `conjecture`. No self-classification of conceptual/economic turns (left to MONODROMY and the Coordinator).*

## Summary

Cycle 3 explored six genuinely distinct mathematical formulations across all four lenses of `METHOD.md` §2 (Representation Import, Obstruction Inversion, Primitive Ablation, Small-Instance Exact Discovery). One foundational primitive — **Bucketed Sequential Schur Elimination with L2-Slabbed Output Emission (BSSE)** — was deepened through Gates G1–G5:

- **Gate G1 (`verify_bsse.py` — proved & verified):** Proved the **Conservation Theorem** (total routed edges $\le m - |B_{s-1}|$ regardless of bucket count $s$), the **Exact Chain-Head Decomposition Lemma** (zero memoization, single-visit topological self-chain recording, exact early cycle closure for multi-cycle/periodic columns), and reverse position propagation. Verified exact on **`97,976 / 97,976`** exhaustive small cases (all binary $n \le 8$, all ternary $n \le 5$, all primary marks, `LIMIT ∈ {1,2,4,16}`, `bsize ∈ {1,2,3,4,8}`, including `26,301` early-bucket closures and `83,016` short-period/multi-cycle cases) and **byte-exact on all 6 authenticated Wikidata blocks (`5,399,692 / 5,399,692` bytes)**.
- **Gate G2 (`bsse.toml` — verified):** Passed `docs/team/acquisition/check_manifest.py` cleanly from $I_0$ with zero `assumed`/`open` edges and no text-determining intermediate before final emission.
- **Gate G5 (`g5_bsse.c` — measured, pre-registered verdict: FAIL):** Despite eliminating **all** irregular L2/L3/DRAM pointer-chasing accesses outside L2 buckets (`Q_{>L2} = 0`), `BSSE-16K-slab` **FAILED** every pre-registered threshold on all 6 blocks: `1.265–1.656×` modelled L2 misses, `1.691–2.507×` modelled DRAM B/B (`8.17–18.18 B/B` vs `4.83–7.91 B/B`), `1.911–1.989×` work events, and `1.76–2.08×` single-thread `-O3` C diagnostic wall-clock time on the VM.
- **Structural discovery (Obstruction on Obligation #1):** Confronting BSSE exposed the **LLC-Resident Streaming-Exchange Obstruction** (`§4`): because Open's `edges[0..m]` (`360–817 KiB`) already fits inside a `2 MiB` LLC and incurs only `~0.50–0.61 m` L2 misses (`45k–121k`, mostly LLC hits), any streaming / bucket-elimination replacement moving $> 40$ B/retained-row sequentially (read+write) generates more L2 line transfers than Open's random walk and spills the `2 MiB` LLC into DRAM.
- **Calibration diagnostic on Cycle 2's Backward Open (`g5_bsse.c -DNO_CACHE_SIM` — measured):** Uninstrumented `-O3` C VM timing showed `Backward Open` running at `1.05–1.09×` Shipped Open (`1.65–3.74 ms` vs `1.61–3.77 ms`), despite beating Shipped Open on every 3-level LRU cache metric (`0.82–0.88×` L2m, `0.72–0.84×` DRAM). Phase timing revealed why: Open's `wir` is only `0.09–0.20 ms` (`5–7%`), while Backward Open's second record scan + branchy coalescing + active-band cache takes `0.59–1.21 ms` (and Backward Open's descending `emit` is `0.91–1.02×` Open, showing descending writes did *not* regress `emit`).
- **Triage & Obligation-Register settlements (`verify_obligations_3_5.py`):**
  1. Closed `OPEN-QUESTIONS.md` §1 (band-level second Open cut): although `90.4–96.2%` of $m$ lies in wide $P$-bands (`w > 16`), composing with $\mathrm{LF}_v$ shatters $B = 2{,}020\text{--}5{,}444$ bands into `47,544–126,226` pieces (mean width `1.62–2.22`).
  2. Settled `OPEN-QUESTIONS.md` §2 (Obligation #3, $\Phi$-overlap coordinates): `0.21–0.25r` raw $\Phi$-copies and `0.22–0.30r` overlaps are computable in $O(r \log r)$ purely from $(\mathrm{SA}[H], \mathrm{SA}[D])$ without run labels, and raw $\Phi$-intervals do *not* uniquely determine ternary texts (exact witnesses at $n = 4, 5$); however, `71.5–79.1%` of raw copies are locked in mutual dependency cycles across all 6 blocks.
  3. Bounded Obligation #5 (Pre-column bzip2 pipeline, previously `unexplored`): proved the **Pre-Column Causal Barrier** (no global $\mathrm{LF}(i)$ for $L[i] > \min(\Sigma)$ is determined before `EOB` because bzip2 omits $C[0..256]$ from block headers) and measured MTF locality on all 6 blocks (`MTF == 1` covers `13.6–15.7%` of run heads; `MTF <= 3` covers `28.9–31.2%`).

---

## 1. Explore: Six Distinct Mathematical Formulations (`METHOD.md` §2)

| # | Candidate | Generation lens | Mathematical principle | Displaced obligation | Likeliest cost trap |
|---|---|---|---|---|---|
| 1 | **Bucketed Sequential Schur Elimination + L2-Slabbed Emission (BSSE)** | Representation Import (Meyer block Gaussian elimination / Sibeyn bucketed list ranking) + Obstruction Inversion | Partition Open's retained set $S$ into $s$ power-of-two $L$-order slabs $B_0 \sqcup \dots \sqcup B_{s-1}$ (`<= 128 KiB` each); eliminate $B_0 \dots B_{s-2}$ in L2 via $\min(\text{bkt}(u),\text{bkt}(v))$ queue routing ($\le m$ routed edges total), compute exact byte offsets `pos` in reverse pass, emit into $C = 8$ L2-resident output slabs | Obligation #1 (serial L2/L3 walk on `edges`) + Obligation #2 (queues + scatter) | Sequential read+write traffic across forward/reverse/output queues may exceed `40 B/retained-row`, spilling LLC into DRAM and raising work $W$ |
| 2 | **1-Wave Directed-Cut Symbol Elimination** | Representation Import (asymmetric stochastic complementation on skewed alphabet: top-12 thin `F`-buckets cover `58–62%` of $m$) | Choose $E \subset \Sigma$ maximizing edges $E \to K = \Sigma \setminus E$; eliminate $U = \{y : F[y] \in E,\; F[\tilde{g}(y)] \in K\}$ in 1 sequential stream-merge pass ($U \cap \tilde{g}(U) = \emptyset$) | Part of Obligation #1 ($m \to m - |U|$) | Independent-set bound $|U| \le m/2$; greedy MaxDiCut on thin bigrams eliminates only `27–28%` of $m$, while doubling code width per surviving edge |
| 3 | **Band-Level Second Open Cut** (`OPEN-QUESTIONS.md` §1) | Representation Import (Durand return substitutions on Open's $B$ bands) | Cut $g = P \circ \mathrm{LF}_v$ or $\tilde{g} = \mathrm{LF}_v \circ P$ using only $P$'s wide bands (`w > 16`) without materialising $m$ rows | Obligation #1 ($m \to m_2 \ll m$) | $\mathrm{LF}_v$ shatters wide $P$-bands immediately into `~r` pieces of width `1–2` |
| 4 | **Direct $(\mathrm{SA}[H], \mathrm{SA}[D]) \to \Phi$-Overlap Graph** (`OPEN-QUESTIONS.md` §2) | Obstruction Inversion (bypassing Record 84's per-byte text-order run labels) | Build raw $\Phi$-copy intervals and overlap graph directly from the $2r$ run-boundary SA positions in $O(r \log r)$; test whether cycles dissolve or raw intervals determine the text | Obligation #3 ($\Phi$-overlap coordinates) | Raw copy intervals may be mostly cyclic; acquiring $(\mathrm{SA}[H], \mathrm{SA}[D])$ already pays Cells' RV (`1.68–1.87r`) |
| 5 | **Pre-Column Bzip2 MTF/RLE2 Local-Alphabet Contraction** | Primitive Ablation (forbid starting from flat/run column $L$; start from MTF+RLE2 stream) | Exploit small-MTF-rank slabs (`MTF <= 1`, `<= 3`) directly from Huffman/MTF before `runs.rs` | Obligation #5 (pre-column pipeline) | Bzip2 omits $C[0..256]$ before `EOB` (causal barrier); `src/runs.rs` already decodes MTF+RLE2 straight to capped runs in AVX2 registers |
| 6 | **Multi-Courier Parallel Open Walk** | Primitive Ablation (forbid 1-walker serial chain latency $D = m$) | Run $K$ walkers in lockstep from $K$ anchors on `edges[0..m]` to fill CPU Line Fill Buffers, then concatenate $K$ scratch buffers | Serial depth $D$ of Obligation #1 ($D = m \to m/K$) | Keeps all $m$ irregular loads $Q = m$ and adds an extra `1.0 n` output copy (`+1.8 MB` DRAM traffic) unless output offsets are known beforehand |

---

## 2. Triage: Cheap Kills & Obligation-Register Findings (`verify_obligations_3_5.py`)

### Triage 2.1 · 1-Wave Directed-Cut Symbol Elimination *(measured kill)*
- Although the top 16 thin `F`-buckets contain `64.5%–69.5%` of all thin rows (`block-0.L`: `69.5%`, `block-143.L`: `66.0%`, `block-210.L`: `64.5%`), most of their transitions stay *inside* the top 16 (digits $\to$ digits, quotes $\to$ digits/punctuation), so `top16 -> K` eliminates only `17.2%–18.0%` of $m$ (**measured**).
- Even an unconstrained greedy MaxDiCut over all symbols (`|E| = 28–46`) eliminates only `27.3%–28.0%` of $m$ (`24,528` on block 0; `53,889` on block 143; `57,121` on block 210), while forcing a second pass over $m$ rows and two band codes per surviving edge (doubling `edges` from `4 B` to `8 B` or adding an extra indirection). Killed.

### Triage 2.2 · Band-Level Second Open Cut (`OPEN-QUESTIONS.md` §1) *(proved + measured kill)*
- **Measured** (`verify_obligations_3_5.py`, all 6 blocks): while `90.4%–96.2%` of $m$ lives in wide bands of $P$ (`w > 16`, only $B = 2{,}020\text{--}5{,}444$ maximal bands), composing $P$ with $\mathrm{LF}_v$ in *either* order shatters those bands into `47,544–125,843` maximal constant-shift pieces for $g = P \circ \mathrm{LF}_v$ and `47,781–126,226` pieces for $\tilde{g} = \mathrm{LF}_v \circ P$ (mean piece width **`1.62–2.22`** rows).
- **Structural reason:** A band of $P$ is a contiguous $F$-interval whose image under $P$ is a contiguous $L$-interval containing many short thin runs of $v = L[S]$ (`63–75%` width 1). Thus $g$ and $\tilde{g}$ have `~r` pieces (not `O(B)`), returning directly to Cycle 1's Metadata-Density Obstruction and Astra Investigation 3's 15× forest overhead.

### Triage 2.3 · Obligation #3: Raw $\Phi$-Copy Overlap Graph & Uniqueness (`OPEN-QUESTIONS.md` §2) *(proved + verified + measured)*
- **Positive finding (verified):** Given only the $2r$ run-boundary SA positions $(\mathrm{SA}[H], \mathrm{SA}[D])$ (obtainable via weight-only RV, Record 81), the raw $\Phi$-copy intervals `(dst, src, len)` and their interval-overlap dependency graph are computable in $O(r \log r)$ with **zero** per-byte text-order run labels (`text_run[t]`). Across all 6 blocks, there are only `0.21–0.25 r` non-empty raw copies (`10,954–26,964`) and `0.22–0.30 r` raw overlap edges (`11,902–32,815`) with `0` self-overlaps (**measured**, `verify_obligations_3_5.py`).
- **Information-theoretic answer to `OPEN-QUESTIONS.md` §2 (verified exhaustively):**
  - On **ternary** alphabets ($\sigma = 3$), raw $\Phi$-copy intervals do **not** determine the text (even up to canonical alphabet isomorphism): at $n = 4$ (`18` primitive columns, `1` canonical collision: `raw_phi = ()` shared by `1100, 1200, 1210`), at $n = 5$ (`48` primitive columns, `3` canonical collisions: e.g. `raw_phi = ((3, 4, 1),)` shared by `11000, 12000, 22000`), at $n = 6$ (`116` primitive columns, `19` canonical collisions).
  - On **binary** alphabets ($\sigma = 2$, $n \le 9$, all `125` primitive columns), raw $\Phi$-copy intervals have **0** collisions because adjacent BWT runs strictly alternate `0/1` once `tau0` is fixed.
- **Exact obstruction keeping Obligation #3 blocked (measured):** Across all 6 blocks, **`71.5%–79.1%`** of raw $\Phi$-copy intervals lie in mutual dependency cycles (`topo_resolved = 20.9%–28.5%`, `verify_obligations_3_5.py`). Breaking those cycles into Record 82's `0.87–2.15r` acyclic schedule requires separating distinct run-offset depths $d(t) = \mathrm{ISA}[t] - h_{\text{run}(t)}$ within overlapping intervals — either via run labels (Record 84, which determine the text) or by peeling from the $r$ literal seeds (Round 5's `RV-Copy`, which shatters into `3.4–8.7r` fragments of `1.8 B`).

| Block | $r$ | $m$ | $B$ (`w>16` share) | Pieces of $g$ / $\tilde{g}$ (mean $w$) | Raw $\Phi$ copies ($/r$) | Raw $\Phi$ overlaps ($/r$) | Cyclic raw $\Phi$ share | MTF run heads `=1` / `<=3` / `<=7` |
|---|---:|---:|---|---:|---:|---:|---:|---|
| `block-0.L` | 51,184 | 89,874 | 2,150 (`91.8%`) | 47,544 / 47,781 (`1.89`) | 10,954 (`0.21r`) | 11,902 (`0.23r`) | `71.5%` | `14.2% / 29.7% / 52.2%` |
| `block-2.L` | 54,532 | 112,070 | 3,139 (`90.4%`) | 50,527 / 50,828 (`2.22`) | 13,223 (`0.24r`) | 16,138 (`0.30r`) | `78.5%` | `15.5% / 30.7% / 52.8%` |
| `block-85.L` | 79,375 | 156,404 | 2,020 (`96.2%`) | 77,733 / 77,971 (`2.01`) | 17,119 (`0.22r`) | 18,354 (`0.23r`) | `75.0%` | `15.1% / 30.3% / 52.8%` |
| `block-143.L` | 108,335 | 196,841 | 5,444 (`90.8%`) | 103,102 / 103,657 (`1.91`) | 26,964 (`0.25r`) | 32,815 (`0.30r`) | `79.1%` | `15.7% / 31.2% / 53.6%` |
| `block-207.L` | 87,111 | 135,921 | 3,548 (`91.8%`) | 83,448 / 83,862 (`1.63`) | 20,803 (`0.24r`) | 23,281 (`0.27r`) | `76.1%` | `15.5% / 30.3% / 52.4%` |
| `block-210.L` | 128,732 | 204,281 | 3,565 (`94.5%`) | 125,843 / 126,226 (`1.62`) | 26,748 (`0.21r`) | 28,360 (`0.22r`) | `71.7%` | `13.6% / 28.9% / 52.1%` |

*(Note: $B$ in `verify_obligations_3_5.py` counts maximal merged constant-$(\text{sym}, \delta, \text{word})$ bands, whereas Open's endpoint constructor emits `2,446–6,788` unmerged bands from endpoint node boundaries.)*

### Triage 2.4 · Obligation #5: Pre-Column Bzip2 Pipeline *(proved obstruction + measured locality)*
- **Theorem (Pre-Column Causal Barrier — proved):** In a standard bzip2 block bitstream (`block.rs`), the header contains `orig_ptr` and the alphabet bitmap `in_use` (`seq_to_unseq`), followed by 2–6 Huffman tables, 50-symbol selectors, and the Huffman-coded MTF+RLE2 symbol stream terminated by `EOB`. Symbol totals $C[0..256]$ are not stored in the header. For any strict prefix of the MTF+RLE2 stream before `EOB`, appending valid continuations shifts $C[c]$ for every $c > \min(\Sigma)$ by arbitrary non-negative integers without invalidating the prefix. Therefore:
  1. **No global $\mathrm{LF}(i)$ value** (for $L[i] > \min(\Sigma)$), no thick-strip landing coordinate `to`, and no global cycle/orbit order is functionally determined before `EOB` is decoded.
  2. The only structural information exposed prior to `EOB` is (a) exact maximal runs `(c, w)` without `runs.rs`'s `CAP = 127` split/re-coalesce, (b) intra-symbol relative ranks $\operatorname{rank}_c(L, i)$, and (c) local MTF recency ranks.
- **Measured MTF locality (`verify_obligations_3_5.py`):** Exact 2-symbol alternations (`MTF == 1`) account for only `13.6%–15.7%` of run heads (`16.8%–20.4%` of thin rows), and local 4-symbol regions (`MTF <= 3`) account for `28.9%–31.2%` of run heads (`31.9%–37.0%` of thin rows). Furthermore, `src/runs.rs` already decodes MTF+RLE2 directly into capped run records in AVX2 registers without materialising flat $L$.
- **Obstruction record (Obligation #5):**
  - *Witness:* Any two valid bzip2 blocks sharing a prefix of MTF+RLE2 symbols before `EOB` that differ in the count of $\min(\Sigma)$ in the suffix (e.g., appending `RUNA` vs another symbol before `EOB`), shifting $\mathrm{LF}(i)$ for every $L[i] > \min(\Sigma)$.
  - *Scope:* All exact single-block bzip2 decoders reading standard bzip2 bitstreams without modifying the wire format.
  - *Escape condition:* Only either (i) changing the compression format to store $C[0..256]$ or auxiliary order metadata (out of scope for standard bzip2 decoding), or (ii) fusing MTF+RLE2 decoding directly into a downstream run-pass kernel (traction fusion saving only `runs` record materialisation and `CAP = 127` re-coalescing, not the BWT inversion itself).

---

## 3. Deepen: Bucketed Sequential Schur Elimination + L2-Slabbed Emission (BSSE)

### Mandatory §3a Structured Summary
- **Mode:** `reorientation`
- **Displaced obligation:** Obligation #1 (dependent L2/L3/DRAM pointer-chasing walk over `edges[0..m]`, `36–48%` of Open's time) and Obligation #2 (Open's 256-bucket queues and scattered edge writes).
- **Replacement principle:** Block Gaussian elimination / Meyer stochastic complementation of Open's retained permutation $g = P \circ \mathrm{LF}_{L[S]}$ executed in streaming bucket-priority order ($\min(\text{bkt}(u), \text{bkt}(v))$ queue routing over power-of-two $L$-order slabs $B_k$ of size $\le 128$ KiB), paired with the **Exact Chain-Head Decomposition Lemma** (single-visit topological self-chain recording and exact early cycle closure) and **L2-Slabbed Output Emission** (`C = 8` output slabs of `112.5 KiB`).
- **New paid obligations:** Queue-free initial bucket partitioning (`orig_code` `2m` B + `fwd_init` `4 B/edge` + `in_q` `8 B/edge`); forward bucket-by-bucket Schur contraction ($R_{\text{rt}} \le m - |B_{s-1}|$ routed `8 B` edges + `2 B` `req_u` + `8 B` `self_chain`); reverse position propagation (`4 B` `resp_p` + re-reading `in_q` and `self_chain`); and output slab bucketing (`4m` B `out_q_slab`) + slab spelling into `out[0..n]`.
- **Nearest prior art:** Sibeyn (1997/2006) external-memory one-by-one bucketed list ranking (general linked lists); Open (`src/open.rs`, Astra 2026) thick-strip first-return exchange; Backward Open (Neon Cycle 2). BSSE adapts bucketed Schur elimination specifically to Open's compressed retained exchange $g = P \circ \mathrm{LF}_v$ using power-of-two $L$-order right-context locality (`bkt(u) = u >> bshift`), zero-memoization chain-head decomposition, exact short-period/multi-cycle early closure, and 4B slabbed output emission.

### Gate G1 · Mathematics & Proof (`proved` + `verified`)

Let $g = P \circ \mathrm{LF}_{L[S]}$ be Open's backward first-return bijection on $V = \{0, \dots, m-1\}$ with forward word lengths $\ell(u) = |W(u)| \ge 1$. Partition $V = B_0 \sqcup \dots \sqcup B_{s-1}$ into contiguous power-of-two slabs $B_k = [k \cdot 2^b, \min((k+1)\cdot 2^b, m))$ (`bkt(u) = u >> b`, `loc(u) = u & (2^b - 1)`).

**Theorem 1 (Priority-Routed Bucket Conservation — proved).**
Assign every active edge $(u \to w, \ell)$ of the current Schur complement $\pi_k$ on $\bigcup_{j \ge k} B_j$ to bucket $\min(\text{bkt}(u), \text{bkt}(w))$: into `fwd_q[j_u]` if $j_u \le j_w$ and into `in_q[j_w]` if $j_w < j_u$. At step $k$, every edge incident to $B_k$ resides in `fwd_q[k]` (all outgoing and internal edges of $B_k$) or `in_q[k]` (all external incoming edges to $B_k$). Contracting each path through $B_k$ creates exactly $|\text{in\_q}[k]| \le |B_k|$ new edges routed strictly to buckets $> k$. Hence across all $s - 1$ steps:
$$R_{\text{rt}} = \sum_{k=0}^{s-2} |\text{in\_q}[k]| \le \sum_{k=0}^{s-2} |B_k| = m - |B_{s-1}| < m,$$
independent of $s$. For a uniform random permutation on $s$ equal buckets, $\mathbb{E}[R_{\text{rt}}] = m(1 - H_s/s)$ (`0.479m` at $s=4$, `0.660m` at $s=8$, `0.789m` at $s=16$).

*Proof:* By induction on $k$, $\pi_k$ is a bijection on $V_k = \bigcup_{j \ge k} B_j$, so every vertex $v \in B_k$ has out-degree 1 and in-degree 1 in $\pi_k$. Every outgoing edge $(v \to w)$ of $B_k$ satisfies $k = j_v \le j_w$ and thus lies in `fwd_q[k]`; every external incoming edge $(u \to v)$ with $u \in V_{k+1}$ satisfies $j_v = k < j_u$ and thus lies in `in_q[k]`. Every vertex $v \in B_k$ either receives an external incoming edge (`|in_q[k]|` vertices) or an internal incoming edge from $B_k$, so $|\text{in\_q}[k]| = |B_k| - |E(B_k, B_k)| \le |B_k|$. Summing over $k < s-1$ yields $R_{\text{rt}} \le m - |B_{s-1}|$. In a uniform random permutation, the probability an element of $B_k$ has an internal incoming edge in $\pi_k$ is $1/(s - k)$, giving $\mathbb{E}[R_{\text{rt}}] = \frac{m}{s}\sum_{k=0}^{s-2}(1 - \frac{1}{s-k}) = m(1 - H_s/s)$. $\square$

**Lemma 2 (Exact Chain-Head Decomposition & Early Closure — proved).**
In $\pi_k$, the induced subgraph on $B_k$ is a disjoint union of directed paths $v_0 \to v_1 \to \dots \to v_t$ (exiting $B_k$ to $w_{\text{out}} \in V_{k+1}$) and zero or more closed cycles wholly inside $B_k$. The entry heads $v_0$ are in exact bijection with the records of `in_q[k]`. Therefore:
1. Tracing each record $(u, v_0, \ell_1) \in \text{in\_q}[k]$ directly through `nxt[0..|B_k|]` visits every non-cyclic vertex of $B_k$ **exactly once** (requiring zero memoization) and appends internal edges $(v_i, v_{i+1}, \ell)$ to `self_chain[k]` in exact topological order.
2. Tracking `(mark_anchor, mark_shift)` (initialized to `(mark, 0)` and updated to `(w_out, mark_shift + dist(mark_anchor -> w_out))` whenever `mark_anchor` is traversed on a non-cyclic chain) maintains the invariant $\text{bpos}[\text{mark\_anchor}] \equiv \text{mark\_shift} \pmod{L_{\text{cyc}}}$.
3. At the unique bucket $k^* \in \{0, \dots, s-1\}$ where `mark_anchor` lies in $B_{k^*}$ and remains unvisited after processing `in_q[k^*]`, the entire marked cycle in $\pi_{k^*}$ closes inside $B_{k^*}$. Walking that closed cycle yields $L_{\text{cyc}}$ and exact backward offsets $\text{bpos}[u] = (\text{rel}[u] + \text{mark\_shift}) \bmod L_{\text{cyc}}$ for all marked-cycle nodes in $B_{k^*}$.
4. The reverse pass ($j = k^*$ down to $0$) ignores any `in_q[j]` entry with $j_u > k^*$ (strictly belonging to unmarked cycles), answers `req_u[k][j]` in L2-resident `pos[0..|B_j|]`, propagates along `self_chain[j]` in one linear scan, and writes each marked-cycle word $W(z)$ to forward interval $[L_{\text{cyc}} - \text{bpos}[z] - |W(z)|,\; L_{\text{cyc}} - \text{bpos}[z]) \subseteq [0, L_{\text{cyc}})$ without wrap-around (repeating $L_{\text{cyc}}$ to $n$ bytes if $L_{\text{cyc}} < n$). $\square$

**Empirical right-context locality (`cycle3_probe2.py` & `verify_bsse.py` — measured):**
Contiguous power-of-two $L$-order slabs (`bkt(u) = u >> bshift`) outperform both $F$-order slabs and greedy bigram-affinity symbol clustering across all 6 blocks (`routed = 0.335–0.389m` at $s = 4$ vs random `0.479m`; `0.503–0.550m` at $s = 8$ vs random `0.660m`; `0.488–0.635m` at fixed `bsize = 16,384`, $s = 6\text{--}13$), because $L$-order groups right-contexts in contiguous ASCII/UTF-8 character classes automatically at zero cost.

**Exhaustive & 6-block verification (`verify_bsse.py` — verified):**
- Small exhaustive: **`97,976 / 97,976` PASS** (`26,301` early-bucket closures, `83,016` short-period/multi-cycle cases) in `2.61 s`.
- Six authenticated Wikidata blocks (`bsize = 16,384`, `n_slabs = 8`): **`5,399,692 / 5,399,692` bytes exact** against dense LF walk.

### Gate G2 · Acquisition Manifest (`bsse.toml` — verified)
`python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-neon-cycle3-bucketed-schur/bsse.toml` outputs **`PASS`**.

---

## 4. Confront: Gate G5 Paid-Cost Falsifier & VM Diagnostic Calibration (`g5_bsse.c`)

**Pre-registered PASS/FAIL rule** (fixed prior to compilation/execution in `g5_bsse.c`): `BSSE-16K-slab` (`bsize = 16,384`, `n_slabs = 8`) vs Shipped Forward Open (`QUEUE=true`), on all 6 blocks:
1. Byte-exact on all `5,399,692` bytes;
2. Total modelled 256 KiB L2 misses $\le 0.85\times$ Open;
3. Total modelled 2 MiB LLC DRAM B/B $\le 1.00\times$ Open;
4. Total modelled work events $\le 1.40\times$ Open;
5. Peak live memory $\le 1.25\times$ Open.

### Cache-Model Results (`g5_bsse.c`, 32 KiB 8w L1D / 256 KiB 4w L2 / 2 MiB 16w LLC, 64 B lines — measured)

All five implementations (`Shipped Open`, `Backward Open`, `BSSE-16K-slab`, `BSSE-8K-slab`, `BSSE-16K-direct`) matched `dense_reference` byte-for-byte on all `5,399,692` bytes (`0` ASan/UBSan errors; zero-warning `-O3 -Wall -Wextra -Werror` build).

| Block (`m`, `routed/m`) | Kernel | Total L2m (ratio) | Phase L2m breakdown | LLC misses | DRAM B/B (ratio) | Work events (ratio) | Peak MB (ratio) | G5 |
|---|---|---:|---|---:|---:|---:|---:|---|
| `block-0.L` (`89,874`, `0.488m`) | Shipped Open | `75,551` (`1.00×`) | `p1=8.4k ep=10.1k wir=12.2k emit=44.8k` | `34,862` | `4.83` (`1.00×`) | `335,225` (`1.00×`) | `2.19` (`1.00×`) | — |
| | Backward Open | `65,071` (`0.86×`) | `p1=2.6k ep=9.8k wir=8.0k emit=44.7k` | `29,199` | `4.03` (`0.83×`) | `245,351` (`0.73×`) | `1.83` (`0.84×`) | — |
| | **BSSE-16K-slab** | `110,402` (`1.46×`) | `p1=2.6k ep=9.8k qsc=13.0k fwd=37.8k rev=25.3k slb=21.9k` | `59,259` | `8.17` (`1.69×`) | `640,746` (`1.91×`) | `2.73` (`1.25×`) | **FAIL** |
| | *Diag 8K-slab / 16K-dir* | `1.36×` / `1.70×` | — | — | `1.70×` / `1.56×` | `1.95×` / `1.64×` | — | — |
| `block-2.L` (`112,070`, `0.492m`) | Shipped Open | `97,563` (`1.00×`) | `p1=10.0k ep=14.6k wir=15.5k emit=57.6k` | `39,060` | `5.42` (`1.00×`) | `408,657` (`1.00×`) | `2.44` (`1.00×`) | — |
| | Backward Open | `82,908` (`0.85×`) | `p1=2.8k ep=14.1k wir=9.6k emit=56.4k` | `31,927` | `4.41` (`0.81×`) | `296,587` (`0.73×`) | `1.99` (`0.82×`) | — |
| | **BSSE-16K-slab** | `134,606` (`1.38×`) | `p1=2.8k ep=14.1k qsc=15.9k fwd=45.9k rev=32.1k slb=23.7k` | `74,819` | `9.88` (`1.82×`) | `786,247` (`1.92×`) | `2.90` (`1.19×`) | **FAIL** |
| `block-85.L` (`156,404`, `0.587m`) | Shipped Open | `103,428` (`1.00×`) | `p1=13.0k ep=4.4k wir=20.5k emit=65.6k` | `40,816` | `5.61` (`1.00×`) | `558,163` (`1.00×`) | `2.58` (`1.00×`) | — |
| | Backward Open | `85,281` (`0.82×`) | `p1=3.0k ep=4.2k wir=12.8k emit=65.2k` | `30,828` | `4.20` (`0.75×`) | `401,759` (`0.72×`) | `1.95` (`0.76×`) | — |
| | **BSSE-16K-slab** | `171,271` (`1.66×`) | `p1=3.0k ep=4.2k qsc=22.0k fwd=69.1k rev=47.4k slb=25.5k` | `104,664` | `12.95` (`2.31×`) | `1,110,287` (`1.99×`) | `3.21` (`1.25×`) | **FAIL** |
| `block-143.L` (`196,841`, `0.635m`) | Shipped Open | `199,155` (`1.00×`) | `p1=17.3k ep=33.3k wir=27.5k emit=121.0k` | `59,757` | `7.91` (`1.00×`) | `726,941` (`1.00×`) | `3.44` (`1.00×`) | — |
| | Backward Open | `174,902` (`0.88×`) | `p1=4.8k ep=32.5k wir=17.1k emit=120.5k` | `42,644` | `5.81` (`0.73×`) | `530,100` (`0.73×`) | `2.66` (`0.77×`) | — |
| | **BSSE-16K-slab** | `251,947` (`1.27×`) | `p1=4.8k ep=32.5k qsc=29.2k fwd=89.7k rev=63.5k slb=32.4k` | `152,985` | `18.18` (`2.30×`) | `1,442,224` (`1.98×`) | `4.24` (`1.23×`) | **FAIL** |
| `block-207.L` (`135,921`, `0.553m`) | Shipped Open | `118,390` (`1.00×`) | `p1=12.4k ep=15.0k wir=18.6k emit=72.5k` | `43,184` | `5.92` (`1.00×`) | `513,368` (`1.00×`) | `2.69` (`1.00×`) | — |
| | Backward Open | `102,482` (`0.87×`) | `p1=3.7k ep=14.5k wir=12.2k emit=72.1k` | `34,310` | `4.68` (`0.79×`) | `377,447` (`0.74×`) | `2.15` (`0.80×`) | — |
| | **BSSE-16K-slab** | `164,462` (`1.39×`) | `p1=3.7k ep=14.5k qsc=20.1k fwd=59.1k rev=41.1k slb=25.9k` | `93,301` | `11.93` (`2.02×`) | `991,389` (`1.93×`) | `3.26` (`1.21×`) | **FAIL** |
| `block-210.L` (`204,281`, `0.631m`) | Shipped Open | `166,873` (`1.00×`) | `p1=17.9k ep=10.3k wir=27.4k emit=111.3k` | `53,429` | `7.13` (`1.00×`) | `757,934` (`1.00×`) | `3.22` (`1.00×`) | — |
| | Backward Open | `143,714` (`0.86×`) | `p1=4.9k ep=9.9k wir=17.7k emit=111.3k` | `38,256` | `5.14` (`0.72×`) | `553,653` (`0.73×`) | `2.41` (`0.75×`) | — |
| | **BSSE-16K-slab** | `230,813` (`1.38×`) | `p1=4.9k ep=9.9k qsc=30.0k fwd=91.7k rev=64.3k slb=30.1k` | `152,466` | `17.89` (`2.51×`) | `1,492,094` (`1.97×`) | `4.03` (`1.25×`) | **FAIL** |

**Pre-registered G5 verdict:** **FAIL (0/6 blocks passed).**

### Uninstrumented `-O3` C VM Diagnostic Timing (`g5_bsse.c -DNO_CACHE_SIM`, 15 interleaved rounds — measured; VM diagnostic only, NOT an i9 benchmark)

| Block | Shipped Open (phase split: `p1 / ep / wir / emit`) | Backward Open (`p1 / ep / wir / emit`, ratio) | BSSE-16K-slab (ratio) | BSSE-16K-dir (ratio) |
|---|---:|---:|---:|---:|
| `block-0.L` | `1.61 ms` (`0.56 / 0.42 / 0.09 / 0.55`) | `1.65–1.76 ms` (`0.15 / 0.42 / 0.59 / 0.56`, `1.05–1.09×`) | `2.98 ms` (`1.85×`) | `2.81 ms` (`1.74×`) |
| `block-2.L` | `1.99–2.05 ms` (`0.66 / 0.50 / 0.14 / 0.74`) | `2.03–2.16 ms` (`0.17 / 0.50 / 0.65 / 0.73`, `1.02–1.05×`) | `3.62 ms` (`1.77×`) | `3.31 ms` (`1.61×`) |
| `block-85.L` | `2.16 ms` (`0.85 / 0.23 / 0.13 / 0.93`) | `2.20–2.29 ms` (`0.19 / 0.24 / 0.81 / 0.92`, `1.02–1.06×`) | `4.48 ms` (`2.08×`) | `4.13 ms` (`1.91×`) |
| `block-143.L` | `3.77–3.82 ms` (`1.26 / 0.77 / 0.20 / 1.56`) | `3.74–4.06 ms` (`0.29 / 0.72 / 1.21 / 1.42`, `0.99–1.06×`) | `6.71 ms` (`1.76×`) | `6.11 ms` (`1.60×`) |
| `block-207.L` | `2.40–2.45 ms` (`0.82 / 0.49 / 0.14 / 0.92`) | `2.49–2.66 ms` (`0.23 / 0.48 / 0.85 / 0.90`, `1.04–1.09×`) | `4.61 ms` (`1.88×`) | `4.34 ms` (`1.77×`) |
| `block-210.L` | `3.30–3.35 ms` (`1.15 / 0.41 / 0.18 / 1.45`) | `3.38–3.47 ms` (`0.29 / 0.42 / 1.17 / 1.40`, `1.02–1.05×`) | `6.34 ms` (`1.92×`) | `5.97 ms` (`1.81×`) |

---

## 5. Learn: Two Crucial Structural Insights

### Insight 1 · The LLC-Resident Streaming-Exchange Obstruction (Obligation #1 — proved + measured)
- **Witness:** `BSSE-16K-slab` (`g5_bsse.c`) on all 6 Wikidata blocks (`m = 89,874–204,281`, `edges = 360–817 KiB < 2 MiB LLC`).
- **Quantitative law:** Open's `edges[0..m]` table (`4m` bytes = `360–817 KiB`) fits **entirely inside a 2 MiB LLC**. Consequently, Open's direct pointer-chasing walk pays `~0.50–0.61 m` L2 misses (almost all LLC hits, `~6–7 ns/hop` on the VM) and near-zero steady-state LLC misses. Any external-memory / multi-pass / bucket-elimination replacement moving $C_B$ sequential bytes (read + write) per retained row incurs at least $\frac{C_B}{64} m$ L2 line transfers; when $C_B > 40$ B/row (BSSE's forward + reverse + slab queues move `~46 B/row` read+write, totaling `2.5–6.5 MB > 2 MiB LLC`), the streaming queues both incur **more** L2 misses than Open's random walk (`1.27–1.66×`) **and** spill the 2 MiB LLC into DRAM (`1.69–2.51×` DRAM B/B, `1.76–2.08×` wall time).
- **Scope:** Every multi-pass streaming, radix-bucketed, or external-memory list-ranking / Schur-elimination scheme on Open's $m$ retained rows (with $4m + U < \text{LLC}$) whose combined queue read+write volume exceeds `~32–40 B/row` or whose live intermediate footprint spills LLC.
- **Escape condition:** A replacement for Obligation #1 must **either** (a) shrink the number of retained items $m$ substantially *before* constructing an $m$-scale map (without paying Cycle 1's per-run metadata density), **or** (b) break the serial latency chain $D = m$ *in-place inside the existing `4m`-byte LLC-resident representation* (zero `> 2 MiB` intermediate queues, e.g., $O(1)$-overhead multi-anchor / bidirectional / algebraic splitting of the walk without copying `n` bytes), **or** (c) operate in a regime where $4m \gg \text{LLC}$ (larger blocks / multi-thread LLC contention per core).

### Insight 2 · Why the 3-Level Cache Model Over-Predicted Cycle 2's Backward Open
On the VM in uninstrumented `-O3` C, Backward Open's `emit` (`0.56–1.42 ms`) matched or slightly beat Forward Open's `emit` (`0.55–1.56 ms`), disproving the fear that descending byte writes hurt `emit`. However, Forward Open's `wir` is already only `0.09–0.20 ms` (`5–7%`, a tight branchless loop over `qdata[a]`!), whereas Backward Open's `wir` (`0.59–1.21 ms`) re-scanned all `R` capped records with branchy run coalescing and per-run active-band state updates — eating all `0.41–0.97 ms` saved in `p1`. This directly validates `METHOD.md` §5's warning that cache-miss models alone omit instruction/branch throughput differences.

### Reproducibility
- `python3 docs/journal/2026-11-20-neon-cycle3-bucketed-schur/verify_bsse.py`
- `python3 docs/journal/2026-11-20-neon-cycle3-bucketed-schur/verify_obligations_3_5.py`
- `python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-neon-cycle3-bucketed-schur/bsse.toml`
- `gcc -O3 -std=c99 -Wall -Wextra -Werror docs/journal/2026-11-20-neon-cycle3-bucketed-schur/g5_bsse.c -o /tmp/g5_bsse && /tmp/g5_bsse <admitted-bwt-dir>`
- `gcc -O3 -std=c99 -Wall -Wextra -Werror -DNO_CACHE_SIM docs/journal/2026-11-20-neon-cycle3-bucketed-schur/g5_bsse.c -o /tmp/g5_bsse_bench && /tmp/g5_bsse_bench <admitted-bwt-dir>`
