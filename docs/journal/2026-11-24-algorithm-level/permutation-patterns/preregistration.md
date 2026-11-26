# Pre-registration: Permutation patterns census and consumer-value diagnostic

*Frozen on 26 November 2026 before collecting any structural census or timing data, under claim [`permutation-patterns`](../../../../frontier/claims/permutation-patterns.toml), [`README.md`](README.md), and [`literature.md`](literature.md). Interoperable with [`ideal-permutation-structures`](../ideal-permutation-structures/preregistration.md). Zero changes to `src/`, zero `unsafe`, zero workstation spending, zero alteration of prior frozen gates (C2).*

---

## 1. Frozen stop test (verbatim from claim)

> Freeze candidate families, bounded local compute budget, distinct-block train/held-out split and repeat design before data. Use exact recognition or proof for any assumed avoidance; sampled absence is not proof. Preserve byte-identical bwt::walk outputs, marked rotation and periodic/multicycle semantics; include arbitrary-column controls. Charge every consumer lookup, conversion, metadata, spelling and store; match warm/cold conditions, report prebuilt consumer gain and materialisation separately. Apply the parent diagnostic's 39.18 ms (3% of 1,306 ms at one worker) combined budget, with labelled measured or explicitly projected values and an actual downstream gain over the matched prebuilt baseline/forward control (or a separately demonstrated cheaper materialisation). Report allowable constructor cost C0+E0-E1-39.18 ms and scan-ideal eight-worker results only if measured. Admit a separate constructor claim only for a concrete candidate that passes; otherwise record the scoped family verdict, not a universal no-go. No new workstation spending or changes to frozen earlier verdicts.

---

## 2. Corpus, distinct-block split, and exactness suite

Aligned identically with [`ideal-permutation-structures`](../ideal-permutation-structures/preregistration.md) so every control and budget figure is directly comparable:

- **Primary corpus:** `corpora/wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 total blocks: **233 production-admitted `OpenCells` blocks** under `LIMIT = 16` and `Cap::dense`, `209,697,662` rows, plus **198 non-admitted blocks**).
- **Deterministic distinct-block split (zero adjacent-block leakage across the boundary):**
  - **Train set:** first `117` production-admitted blocks (`admitted_idx` in `0..117`, `105,298,817` rows). Used for Part 1 structural census, candidate-family qualification/selection, and discrete parameter selection.
  - **Held-out set:** last `116` production-admitted blocks (`admitted_idx` in `117..233`, `104,398,845` rows). All verdict timings, ratios, working-set figures, materialisation timings, and allowable constructor budgets are evaluated on the held-out set.
- **Forced non-admitted controls (kept strictly separate from admitted metrics):**
  - All `198` non-admitted blocks of `corpora/wikidata.bz2` verified 100% byte-identical to `bwt::walk` under forced `Cap::NONE` Open construction.
- **Synthetic exactness & pattern-theory stress suite (12 cases):**
  - The 11 synthetic cases from `ideal-permutation-structures` (`synth_dense_4k`, `synth_periodic_abac256`, `synth_periodic_aaab512`, `synth_periodic_ab500`, `synth_multicycle_64`, `synth_multicycle_4k`, `synth_run_heavy_4k`, `synth_edge_n1`, `synth_edge_n2_p0`, `synth_edge_n2_p1`, `synth_edge_const64`), PLUS:
  - Case 12: exact `bbaaa` counterexample (`L = b"bbaaa"`, retaining `{0, 1, 2, 3}` under `LIMIT = 1` vs raw LF) verifying that raw LF avoids `321` while its first-return map contains `321`.
- **Algorithmic self-verification (exactness of Part 1 recognition algorithms):**
  - Strong-interval tree (`O(N log N)` segment-tree permutation tree) exhaustively verified against `O(N^3)` brute-force strong-interval enumeration on all `5,913` permutations of lengths `1..=7` (`S_1..S_7`).
  - Exact 3-pattern counter (`O(N log N)` Fenwick-tree formula) exhaustively verified against `O(N^3)` brute-force counting on all `873` permutations of lengths `1..=6` (`S_1..S_6`).

---

## 3. Part 1 — Exact structural census (117 train blocks + controls)

Run exact (zero-sampling-avoidance) structural algorithms on **both** raw LF ($\pi \in S_n$) and Open's retained first-return map ($T = \Psi_R \in S_m$ and its inverse $T^{-1} = \text{LF}_R \in S_m$), reporting alongside block covariates (`n`, `m`, `m/n`, alphabet size $\sigma$, maximal runs `r`, `h1/m`, `strips/m`, `bands`):

1. **Exact strong-interval tree / substitution decomposition (Brignall):**
   - Exact strong-interval count, linear $\oplus$ and $\ominus$ node counts, prime (simple) node count, **largest prime core arity** (`max_prime_arity` and `max_prime_arity / N`), tree depth, total leaves inside nontrivial linear vs prime intervals, and exact **separability** (`prime_nodes == 0`).
   - On Open's labelled map $T$: exact **label-homogeneity** of nontrivial strong intervals (fraction of nontrivial linear and prime strong intervals whose vertices all share a single return-word `code`).
2. **Exact monotone / Barbay–Navarro decompositions (on raw LF, $T = \Psi_R$, and $T^{-1} = \text{LF}_R$):**
   - Contiguous ascending runs (`nRuns`), contiguous monotone runs (`nMRuns`), **labelled** contiguous ascending runs (`nLRuns`: `T(i) < T(i+1) && code(i) == code(i+1)`), empirical run-length entropies $H(\text{vRuns})$ and $H(\text{vLRuns})$;
   - Strict $+1$ runs (`nSRuns`), strict $\pm 1$ runs (`nSMRuns`), **labelled** strict $+1$ runs (`nLSRuns`: `T(i+1) == T(i)+1 && code(i+1) == code(i)`);
   - Exact `LIS` and `LDS` (= exact $\min \text{nSUS}$ by Dilworth's theorem), with explicit verified witnesses proving $D_{\text{LDS}} \le \pi$ and $D_{\text{LDS}+1} \not\le \pi$.
3. **Exact small-pattern profile & witnesses (Even-Zohar–Leng / separability):**
   - Exact counts of all six length-3 patterns (`123, 132, 213, 231, 312, 321`) on $T$ (as proportions of $\binom{m}{3}$ vs uniform $1/6$);
   - Explicit verified 4-tuple witnesses `(i1 < i2 < i3 < i4)` for `2413`, `3142`, `4321`, `1234`, and `2143` on $T$ (claiming avoidance ONLY when proved by `prime_nodes == 0`, `LDS < 4`, or `LIS < 4`);
   - Exact contiguous local $k$-gram relative-order shape histogram ($k = 3, 4$) on $T$ and label-homogeneity of ascending $k$-grams.

---

## 4. Part 2 — Pre-registered candidate-family menu & deterministic selection (at most 2 families)

Every candidate represents the **exact labelled Open retained map $(T, w)$** (preserving exact `bwt::walk` bytes, marked rotation, and periodic/multicycle semantics; zero direct byte transcripts; zero uncharged lookups). Four literature-grounded pattern families are pre-registered with explicit training-set qualification criteria:

1. **Family A — Labelled Strict $+1$ Run / $\oplus$-Interval Rank-Indexed Map (`P_strict_run`, Barbay–Navarro `nSRuns` + Brignall linear $\oplus$ intervals):**
   - *Mechanism:* Collapses every maximal labelled strict $+1$ run (`T(j+1) == T(j)+1 && code(j+1) == code(j)`, capped at length $\le 63$ so every offset fits in one `u64` word) into one 32-bit head edge `(code << bits) | head_dst` + a compact 64-bit `heads` bitvector with `u32` per-word prefix sums (`12 B` per 64 vertices = `0.1875 B/row`). Consumer computes `r = prefix[j>>6] + popcnt(word & mask) - 1`, `offset = j - prev_set_bit(word, j&63)` (no `head_src` array), reads `head_edges[r]`, adds `offset` to `dst`, and spells `word[code]` via the exact same `inline16`/`lens`/`spelling` dictionary.
   - *Train qualification rule:* Train median `nLSRuns / m <= 0.75`.
2. **Family B — Labelled Ascending-Run Monotone-Block Map (`P_mono_block`, Barbay–Navarro `nRuns` for $O(1)$ cache-line access):**
   - *Mechanism:* Exploits `nLRuns(T) <= #bands` by partitioning `0..m` into fixed-width blocks of $B$ vertices. Each block whose $B$ entries lie in a single labelled ascending run with value span `T(start+B-1) - T(start) <= 65,535` is stored inline as `(base: u32, code_and_tag: u32, offs: [u16; B])` (`2.25–2.50 B/row` vs `4.00 B/row`); boundary or wide-span blocks set an escape tag pointing into an exact `escape_edges: Vec<u32>` slice (`B` exact 32-bit entries). Consumer evaluates `(next, code)` in $O(1)$ arithmetic with zero binary search and zero wavelet-tree traversal.
   - *Train parameter sweep:* $B \in \{16\text{ (40 B, bitshift)}, 28\text{ (64 B aligned cache line)}, 32\text{ (72 B, bitshift)}\}$; freeze the fastest $B$ on train warm consumer subject to `rep_bytes <= ctrl_shipped_fwd rep_bytes`.
   - *Train qualification rule:* Train median `nLRuns / m <= 0.15`.
3. **Family C — General Substitution-Decomposition / Separable Tree Map (`P_subst_tree`, Brignall + Chakraborty et al.):**
   - *Mechanism:* Recursive strong-interval tree with compact prime-skeleton tables.
   - *Train qualification rule:* All train blocks separable (`prime_nodes == 0`) OR train median `max_prime_arity / m <= 0.50`. Otherwise structurally excluded by exact strong-interval census.
4. **Family D — Pattern-Avoidance Matrix-Division Map (`P_pattern_avoid`, Kozma–Opler):**
   - *Mechanism:* 2-level balanced matrix division for $\operatorname{Av}(\pi)$.
   - *Train qualification rule:* Exact avoidance of some pattern $|\pi| \le 4$ across all train blocks OR train median `LDS(T) <= 8`. Otherwise structurally excluded by exact pattern witnesses.

**Deterministic selection rule (strictly at most 2 families):**
Rank all qualifying families from `{A, B, C, D}` by lowest training total `rep_bytes` (ties broken by fewer dependent loads per step) and select **at most the top 2** families for held-out consumer benchmarking. For each selected family, evaluate both its **1-lane forward consumer** (matched against `ctrl_shipped_fwd`) and its **4-lane forward consumer** (matched against `ctrl_shipped_fwd_4lane`, using the exact same 48-byte `Lane4` anchor convention).

---

## 5. Controls, measurement protocol, and compute budget

- **Matched prebuilt controls (identical to `ideal-permutation-structures`):**
  1. `ctrl_shipped_bidi`: Open `LIMIT=16` bidirectional XOR-linked map consumed by exact production `emit_bidirectional` / `emit`.
  2. `ctrl_shipped_fwd`: Open `LIMIT=16` forward map consumed by exact production forward `emit`.
  3. `ctrl_shipped_fwd_4lane`: Open `LIMIT=16` forward map consumed by 4-lane interleaved forward walk+spell.
- **Hardware & compilation matrix:** 2-vCPU Haswell Xeon @ 2.30 GHz KVM VM (32 KiB L1d, 256 KiB L2, 45 MiB L3, no vPMU), pinned to CPU 0 (`taskset -c 0`), single-threaded, reusable `out` buffer; compiled and benchmarked on **both** default baseline x86-64 and `-C target-cpu=native`.
- **Repeats & matched cache conditions:** **7 interleaved repeats** across all arms per condition, reporting median and `min..max` in `ms`, `ns/row`, `ns/visit`, and paired ratios vs `ctrl_shipped_bidi`, `ctrl_shipped_fwd`, and `ctrl_shipped_fwd_4lane`:
  - **Warm (in-situ):** 1 untimed consumer call on block `b` immediately before the timed consumer call on `b`.
  - **Cold (streaming):** 64 MiB cache-eviction sweep (> 45 MiB L3) before each arm's sequential pass over the 116 held-out blocks.
- **Separate cost accounting:** Report separately (1) prebuilt consumer time (warm & cold), (2) offline build / conversion time, and (3) actual direct materialisation time (measured over 5 repeats on held-out blocks for `Open` bidi, `Open` fwd, and direct materialisation of any selected family that can be built directly from `bands + queues` without first materialising `edges[0..m]`).
- **Per-arm structural ledger:** `rep_bytes`, navigation structure bytes (`nav_bytes` = `edges` or `heads+head_edges` or `blocks+escapes`), `dict_bytes` (`inline16 + lens + spelling`), `m` / visits, dependent loads per visit, coordinate-conversion ops per visit, write amplification (`bytes_written_to_out / n`), escape/fallback rate, and peak consumer scratch bytes.
- **Local compute budget:** Target `< 15 minutes` total wall time on this VM; zero i9 workstation runs or external expenditure.

---

## 6. Anchored budget formulas & decision rule (C2)

Anchored identically to `ideal-permutation-structures` (stage-1 one-worker i9-10900K `wikidata ×4` reference):
- $W_{1w} = 1,306.0\text{ ms}$; 3.0% gate $G = 39.18\text{ ms}$.
- $C_{\text{base}} = 577.7\text{ ms}$ ($200.3 + 216.0 + 161.4$); $E_{\text{base}} = 490.3\text{ ms}$; $C_{\text{base}} + E_{\text{base}} = 1,068.0\text{ ms}$.
- Projected i9 forward constructor $\hat{C}_{\text{fwd}} = 200.3 + 216.0 + 161.4 \cdot (t_{\text{A6c,fwd}} / t_{\text{A6c,bidi}})$.
- For candidate $X$ with held-out consumer median $t_X$ (transparently labelled as VM-to-i9 ratio projection):
  - $\hat{E}_X = 490.3 \cdot (t_X / t_{\text{ctrl\_bidi}})$
  - Consumer-only gains: $\Delta E_{\text{vs bidi}} = 490.3 - \hat{E}_X$, $\Delta E_{\text{vs fwd}} = \hat{E}_{\text{fwd}} - \hat{E}_X$, $\Delta E_{\text{vs 4lane}} = \hat{E}_{\text{4lane}} - \hat{E}_X$
  - Allowable total constructor cost:
    - $C_{\max,\text{vs base}} = C_{\text{base}} + E_{\text{base}} - \hat{E}_X - 39.18 = 1,028.82 - \hat{E}_X$
    - $C_{\max,\text{vs fwd}} = (\hat{C}_{\text{fwd}} + \hat{E}_{\text{fwd}}) - \hat{E}_X - 39.18$
    - Incremental allowance above $\hat{C}_{\text{fwd}}$: $\Delta C_{\text{extra,max}} = \Delta E_{\text{vs fwd}} - 39.18$.
  - Scan-ideal 8w model projection (`69.5 / 490.3 = 0.1417×`) reported only as a labelled model projection (not measured on i9).

**Decision (strict C2):** Admit a separate bounded constructor claim at `status = "idea"` ONLY if a concrete, byte-exact, pattern-derived candidate beats the matched prebuilt forward control (and baseline) on held-out blocks across both warm and cold conditions (or independently demonstrates cheaper direct materialisation covering any consumer delta) AND clears the `39.18 ms` (3%) combined budget with a realistic positive allowable constructor cost. Otherwise record a **scoped family STOP** (naming the exact families, interfaces, corpus, cost model, and budget excluded — not a universal no-go) and append a scoped row to `frontier/dead-ends.tsv`.
