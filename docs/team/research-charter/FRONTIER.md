# The research frontier

*A living record. Research sessions read it at the start of every cycle and update it at the end
(METHOD.md §2, Learn). It holds:*
- *the questions that challenge the current primitives;*
- *the obstructions already known;*
- *the connections not yet explored;*
- *a log of cycles.*

*Evidence lives in [CONTEXT.md](CONTEXT.md) and the journals. This file points to it.*

## Pending mathematical leads

This is an inbox for directions Louis introduces between cycles. A lead is an invitation, not an
assignment: Neon reads it at the next cycle boundary and decides how it should shape the next
cycle. Following one needs no approval. Neon updates the status.

| Mathematical doorway | Why it might expose a new representation or primitive | References | Status |
|---|---|---|---|
| **The origami lift:** a BWT block as a branched torus cover with monodromy (LF, +1) | Surface operations, and directions with a transport law, unavailable in the one-dimensional exchange. MONODROMY verified the construction (record 86); the cone points alone cannot decode | [journal](../../journal/2026-11-20-origami-lift.md) | pending |
| **Illegal turns and folding** (Bestvina–Handel, Stallings) | If a graph self-map of the run dynamics can be built cheaply, its illegal-turn structure might isolate the few branching decisions that carry the ordering, resolved collectively by folds. A self-map must be constructed first; without one, "illegal turn" is undefined | Bestvina–Handel, *Train tracks and automorphisms of free groups* (Ann. Math. 1992); Feighn–Handel on relative train-track maps; Rafiqi–Zeytin, arXiv:2610.13347 (survey) | pending |
| **Ribbon graphs and rotation systems** | Restoring the cyclic order at each vertex encodes a surface embedding. Faces are cycles of σ∘α. Could ordered paths be assembled collectively from the embedding? Caution: plain face traversal may just be LF in disguise | Standard combinatorial-map literature; the origami journal | pending |
| **Splitting integer-weighted train tracks, and contraction** | Dunfield–Thurston count the components of weighted multicurves by splitting bands, which they identify with Rauzy induction, without tracing the curves. Is there an exact analogue that yields the marked ordered itinerary? Also: parallel list and tree contraction carrying ordered values | Dunfield–Thurston, *A random tunnel number one 3-manifold does not fiber over the circle* (2006), §7.3; Shun, Gu, Blelloch, Fineman and Gibbons on list and tree contraction; Cong–Bader on how building the adjacency an Euler tour needs can cost as much as the tour (an acquisition warning) | pending |
| **Agol–Hass–Thurston interval-pairing reduction** | Its input, integer intervals paired by translation, matches a block's runs, and the algorithm resolves global structure without walking points. It computes component counts, not order. Can a word-carrying version recover the marked orbit more cheaply than Cells or Open? | Agol, Hass and Thurston, *The computational complexity of knot genus and spanning area* (2006), the interval-pairing algorithm; Toby Hall's *Trains* software for Bestvina–Handel | explored in Wormhole cycle 3 ([journal](../../journal/2026-11-21-wormhole-cycle3-slp-lz77-output-memoization.md)): bzip2's pre-BWT `RLE1` caps monochrome runs at 4+len, leaving `0–2` periodic-quotient steps on Wikidata |
| **Working backwards from the text, and an algorithm zoo** | Start from the text's structure and find which classes own the expensive rows. Measured: the JSON skeleton is already cheap (keys: 33–38% of bytes, 1–4% of thin rows). Unique GUIDs own 54–67% of width-1 rows; repeated sha1 reference hashes are thin, copyable rows. Questions: an operation on random segments that isn't a walk, or a scoped floor; a charged copy mechanism for repeated random strings; and one cheaply recognised primitive per class, oracle headroom first | [journal](../../journal/2026-11-20-working-backwards.md); Hutter Prize and STARLIT as provocation | pending |
| **Toy, missing ingredient, native surrogate** (the method of Alpöge's announcement note) | Re-derive inversion where it is already free, with ingredients chosen to transfer. In perfectly clustering BWTs (one run per letter, symmetric IETs) the text is mechanical and every position is computed, not reached. The capability to transfer is cheap global coordinate evaluation (or cheap composition of long orbit segments), not run width: the two separate (e.g. rotation codings by many-piece partitions keep the coordinate and lose the width). Seek a surrogate native to real blocks, with cheap acquisition. Independent lanes as the surrogate failed for Open's emit (not latency-bound) | [journal](../../journal/2026-11-20-toy-transfer.md); Ferenczi–Zamboni 2013; Mantaci–Restivo–Sciortino 2003 | pending |
| **The kitchen sink: use everything** (after the relayed overview of the claimed RH development; Kontorovich: "RH is proved by using *everything*!") | Each cycle attempts bridges from many frameworks, including unlikely ones; the named job is the output of an attempt, not its entry ticket; the register and native timing filter afterwards. About 25 frameworks are listed. Untouched here and closest to cheap global coordinates: Rauzy fractals and the Pisot conjecture (torus coordinates for substitutive words), and Veech groups (SL(2,ℤ) acting on our block's origami: other directions, computed from the two permutations). Also Krohn–Rhodes cascades, dimension groups, random-permutation laws for the GUID class | [journal](../../journal/2026-11-20-kitchen-sink.md) | pending |
| **Discrete Morse reduction** (Forman; algebraic Morse theory: Sköldberg, Jöllenbeck–Welker; filtered Morse theory: Mischaikow–Nanda 2013, the basis of Perseus) | Open is a discrete Morse collapse of LF's functional graph: thick-strip rows are matched away along an acyclic matching, retained rows are the critical cells, and each retained row's word is the label of its gradient path. Algebraic Morse theory computes the reduced differential as a sum over gradient paths, which is the same Schur complement as the stochastic complement already noted. Leads: (1) reductions versus coreductions (Mrozek–Batko), which change which cells survive (compare Backward Open); (2) filtered matchings that preserve persistence, read as reductions that preserve an order, which is our constraint; (3) the practitioners' lesson that reducing before algebra dominates cost. Prompted by Vidit Nanda's 2026 Perseus speed-ups. Perseus 6 (github.com/viditnanda/perseus, MIT, docs/TECHNIQUES.md) implements algebraic Morse reduction of unit pairs over Z and over Z[t, t⁻¹]. Weight each LF step by t and Open's reduction is exactly this: retained rows linked by t^(word length). Replace t by the free group ring to keep the words themselves (Fox calculus). Its apparent pairs, pairs settled locally with no reduction, are our free steps. Its graph-incidence reductions are totally unimodular, so pivots are ±1, which bears on Kitchen Sink's unimodular orbit-time system. Engineering worth comparing: candidates classified by one byte through 256-entry tables, prefetch two levels ahead, and pointer chains compressed in index order before the sorted pass | Forman 1998; Sköldberg 2006; Mischaikow–Nanda, *Discrete Comput. Geom.* 2013 | explored in Wormhole cycles 2–5 ([C2](../../journal/2026-11-21-wormhole-cycle2-morse-quasideterminant-cliff.md), [C3](../../journal/2026-11-21-wormhole-cycle3-slp-lz77-output-memoization.md), [C5](../../journal/2026-11-21-wormhole-cycle5-pigeonhole-chaitin-synthesis.md)): $\mathbb{Z}[t,t^{-1}]$ wordless decoding refuted at $\Lambda=1$ ($n=5$ twin `20011`/`21100`); $\Lambda<16$ & SLP output-memoization closed; coreduction (`Kakutani` conjugate stream) + `Node16` + Chaitin overlay yields `W2: CPC-Micro` |

*These references come from Louis's literature search and have not yet been checked against their
sources here.*

## Obligation register

These are the expensive classes of work that current decoders pay. Candidates come and go; an
obligation stays open until it is removed or bounded with a proof. Update a row only when a cycle
has learned something concrete (METHOD.md §3a).

When a failure exposes a significant obstruction, record three things under its row:
- the concrete **witness**;
- its exact **scope**: which algorithms, representations and cost model it binds;
- the **escape condition**: which assumption a different formulation would have to drop.

An obstruction with no stated scope is not established. Not every failure yields one.

| Obligation | Evidence | Open question | Status |
|---|---|---|---|
| Dependent traversal of retained rows | Open: about one dependent load per retained row (≈ one per ten output bytes); emission 36–48% of time (#653). Wormhole C1 Thm 1.3 (Hub–Spoke Cycle-Packing): **witness** `80.7k–102.2k` rows + $O(r\log r)$ Kahn tax on blocks 0–210; **scope** indivisible-vertex FVS DAGs on $G_{\mathrm{ov}}$ and $G_\Phi$; **escape** sub-interval collapse (`Open`). Wormhole C4 Thm 4.1 (`Bidi-Open`): zero-copy `at_f == at_b` escapes Depth Thm 6 staging (`emit` `-7..9%`), obstructed by dual `eb[sj]` scatter (**witness** `+101–318 µs` wire, `6.13 MiB` WS; **scope** explicit dual-direction edge tables on `m` rows; **escape** single-table reversible encoding) | Can output-order relations be resolved collectively, without a producer as expensive as the walk? | open |
| Building the retained map | Open's run pass and queue fill 28–36%; queued wiring 23–28% (#653). Compression (Q) and fusion (F) lost (#654). `#659` Backward Open lost (`1.10–1.15×`) only because it re-scanned raw `CAP=127` records with a scalar per-row loop: Wormhole C4–C5 `W2: CPC-Micro` ([journal](../../journal/2026-11-21-wormhole-cycle5-pigeonhole-chaitin-synthesis.md)) pairs a `79–147 KiB` `u16` thin-run stream (`0 B` queues) with `99.47–99.77%` branchless `Vec16` `iota` wiring + Direct-Linked `Node16` overlaid `100%` inside dead `edges` (`1.48 MiB` persistent scratch = `0.316×` `Open058`, fitting 7 i9 workers in `16.1 MiB <= 20 MiB` L3; `0.947–0.982×` 1t and `0.927–0.972×` 4t in C confrontation) | Does `W2: CPC-Micro` (and `W1: Chaitin-Open`) hold its win natively in safe Rust across 6–10 workers on the i9-10900K? | open; candidate ready for native Rust port |
| Acquiring the Φ-overlap coordinates | The overlap structure is small: 1.25–1.37r transitions on the six blocks (Neon cycle 1). Known route via run-boundary positions costs Cells' 1.68–1.87r steps (record 81). Wormhole C1 Thm 1.2: $\le 96$ byte seeds make punctured $\Phi_p$ 100% acyclic (`0` extra seeds on `6/6` blocks) + zero-split Kahn `memcpy` replay without run labels, obstructed on cost by **witness** `38k–104k` random `memcpy`s + $O(r\log r)$ interval lookup (`2.8–6.2 MiB` WS; **scope** explicit phrase-level $\Phi_p$ DAGs; **escape** implicit or coarse-grained word transport) | Can those coordinates come out of a cheaper computation, or out of an earlier representation? | open; positive measurement |
| Full-block metadata | Grammar and interval-exchange machines carry 40–100 B per run across all r runs, mostly width-1 runs (Neon cycle 1; Astra D). Wormhole C2–C3 confirmed lowering $\Lambda: 16\to 2$ inflates `Endpoint` scratch to `5.0–10.7 MiB` and time to `1.15–2.07×` even at `0 B` `spelling` (`OM-Fox`); at $\Lambda=16$, `W2: CPC-Micro` cuts persistent scratch to `1.48 MiB` (`1.68 B/byte` incl. `out`, `0 B` per thin row in queues) | Can a primitive avoid per-run global state? | open |
| Starting from the decoded column | Every kernel begins from run records after Huffman, MTF and RLE2 decoding. Wormhole C5 Thm 5.1 proved `Runs::counts` & `Runs::per` (`src/runs.rs:90-92`) bound $\mathrm{thin}[a] \le \min(C[a], 16\,\mathrm{per}[a]+1)$, with `CAP=127` thick chunks inflating `16*per[a]` by `3.4–4.4×` | Does an earlier representation (MTF/RLE2 before `CAP=127` splitting) carry ordering or exact thin counts more cheaply? | unexplored |
| Writing every output byte | Unavoidable; sets the floor | (bounds the others) | floor |

## Steering log

Each entry records a change of direction, written before the cycle it governs, and its observed
effect afterwards. Louis owns the decision at the bottom of each entry: hold heading, correct, or
reorient.

| Applied | Target | Intervention | Intended response | Observed response | Decision |
|---|---|---|---|---|---|
| f5961bfb, for Neon cycle 3 | The search kept returning to classical permutation inversion: induced sorting, pair recompression, the direction duality (audit, branch `audit/neon-cycles-1-2`) | Make computational obligations the unit of research; reorientation by default; the obligation register; the bzip2 pipeline before the decoded column made admissible | Investigations that replace ordering operations collectively, rather than reorganise them | pending cycle 3 | pending |
| 6d9acc87, for Neon cycle 3 onward | The obligation register was being used to generate ideas as well as judge them; agents self-classified; cost models were uncalibrated (0 of 6 promoted candidates passed natively) | Generation from mathematical structure, with obligations kept for evaluation; classification by the auditors; cost models treated as diagnostic; witness, scope and escape condition recorded for obstructions. Methodology frozen until evidence arrives | Mechanisms from structure, not from rephrased cost lists | pending | pending |

## Foundational questions (track 1)

Each questions whether a current core operation is necessary. None is settled.

1. **Must the decoder follow an orbit at all?**
   - Every production kernel ends by emitting in LF-orbit order. Rows are visited one after
     another, or retained rows are walked one after another.
   - Is there a formulation in which output positions are computed rather than reached? For
     example, a potential or ranking function on intervals that places whole runs of output
     directly.
   - **Known obstruction:** text-order run labels determine the text (record 84). Computing them
     *is* emission, so any "placement" must be cheaper than the labels it implies.
2. **Is the first return the right object?**
   - Open computes returns to a retained *set* of rows. Repeated cuts shrink that set fast:
     110,273 → 17,657 → 3,916 rows on block 0.
   - What is the limit object of repeated cutting? Can it be computed directly from the band
     structure, without materialising the intermediate levels?
   - See OPEN-QUESTIONS.md §1 for the feasibility version.
3. **Can geometry replace rows?**
   - Rauzy–Veech propagates information through interval endpoints; emission still touches
     every retained row.
   - Is there an exact reconstruction whose work scales with the number of *structural events*
     (run boundaries, overlaps, bands) and whose only n-scale work is sequential byte writing?
   - **Known obstructions:**
     - copy replay shatters into 1.8-byte fragments (round 5);
     - the in-place Φ schedule needs run labels (record 84);
     - the forest construction was 15× slower than Cells (Astra investigation 3).
4. **Is the inverse BWT the right problem?**
   - The decoder must produce bytes, not a permutation.
   - Could information already in the compressed stream let the decoder skip parts of the
     inversion? Examples: the MTF/Huffman symbols, the RLE2 structure, or the block's relation
     to its neighbours.
   - Not yet examined by anyone.
5. **Copies from completed output.**
   - Astra C: the marked output can supply its own sources.
   - What is the cheapest possible discovery of "which rows spell bytes already written"?

## Known structural obstructions

Do not rediscover these; build on them.

| Obstruction | What it rules out | Evidence |
|---|---|---|
| Accelerated induction floor r; Cells at 1.013–1.019r | Better monotone induction *schedules* | records 79–80 |
| Run labels determine the text | Any plan that needs text-order run labels before emitting | record 84 |
| Direct Φ-copy fragments are cyclic (85–94% on dependency cycles) | In-place write-once Φ replay at F_self_ns | round 6, attainability |
| Building precomputed jumps per capped record costs about one Open | NSPC-style per-record contraction | #652 |
| The expansion to one entry per retained row is required, and cheapest in a sequential fill | Moving or compressing Open's queue expansion | #654 |
| The CPU already issues the next edge load early | Software-pipelining Open's walk | #656 |
| LIMIT=16 is best; per-block choice gains ≤ 1% even with hindsight | Thin/thick threshold tuning | #657 |
| Computing LF from a compact rank structure instead of storing the edges costs about four chained dependent loads per step against Open's one | Implicit-rank (FM-index-style) successors as a replacement for Open's explicit edges, single-threaded. This is the known medium-space inverse BWT trade (Lauther–Lukovszki 2005; Kärkkäinen–Puglisi, ESA 2010) | Neon cycle 5, `research/neon-bipolar-contraction` 4598c5bf: BSO-512 2.48–2.80× Open in its C harness; the Coordinator reproduced 1.71–2.44× on another machine. Its memory and DRAM savings (0.36–0.75×) don't pay for it |
| Histogram non-identifiability: same run histogram, Θ(n)-different spelling cost | Predicting construction cost from run statistics alone | Neon (unverified by MONODROMY) |

## Connections not yet explored

These are pointers, not results:
- **The run network** ([journal](../../journal/2026-11-20-run-network.md)): LF as a graph on runs, built in O(r).
  - Its cycle rank, 0.21–0.25r, is fixed by record 63's identity E = 2r − 1 − C, and equals the copy count k plus one.
  - 88–89% of runs are pass-throughs, leaving a series-reduced core of about 0.11r runs.
  - With order-preserving switches it is a measured train track.
  - Census (bd781368): the core has no small separators, though it bisects 4–8× better than random. As a retained set for first returns it is worse than Open's.
  - Open question: is there an operation on the core that is not a first-return walk?
- **The origami lift** ([journal](../../journal/2026-11-20-origami-lift.md)): a BWT block is a branched cover of the torus with monodromy (LF, +1). MONODROMY verified it (record 86). The few cone points are generic, not special to Wikidata. The cone structure ignores run lengths, so it cannot decode alone. Still open: directions and transport (§6.2).
- Kempa (SODA 2019) and Agol–Hass–Thurston: sublinear routes to run-boundary samples. Could an
  o(n) sample construction feed a reconstruction?
- LCP-chain structure (Kärkkäinen–Kempa–Piątkowski): reducible and irreducible LCPs at run heads
  (deep-research-gap-chains).
- Stochastic complementation (Meyer 1989), the matrix form of Open's elimination: are there other
  complements, with other retained sets, with cheaper structure?
- Derived sequences and return substitutions for the finite marked BWT (Durand), applied to the
  repeated-cut limit.

## Cycle log

Append one entry per research cycle: the date, the session, what was explored, what was killed and
why, and what was promoted.

| Date | Session | Explored | Killed (why) | Promoted |
|---|---|---|---|---|
| 2026-11-19–20 | Neon rounds 1–7 and follow-ups; the Coordinator | parallel induction; boundary samples; copy replay; NSPC; DOE; Open micro-optimisations; LIMIT | see CONTEXT.md §4 | none to production; Open shipped (0.5.7) from the Astra line |
| 2026-11-20 | Neon cycles 3–5 (branch `research/neon-bipolar-contraction`, 4598c5bf; not yet audited beyond BSO's timing) | bucketed Schur; unsplit forest; output-sourced anchors; bit-sliced implicit Open (BSO) | anchors (width-1 runs collapse interval certificates); BSO (serial rank latency, above) | none; Neon paused |
| 2026-11-21 | Wormhole cycle 1 (`research/carbon-wormhole`; [journal](../../journal/2026-11-21-wormhole-cycle1-fvs-macro-duality.md)) | Levy–Low Directed FVS + Tarjan/Lawson Kahn metric-DAG pushforward on $G_{\mathrm{ov}}$; compressed-index $\Phi_p$ + $\sigma$-seed Storer–Szymanski/Kempa–Prezza BMS + unsplit Kahn `memcpy` replay on $G_\Phi$ (`5,227` small `(L,p)` + `6` blocks exact) | Indivisible-vertex FVS schedules on $G_{\mathrm{ov}}$ and $G_\Phi$ (Theorem 1.3 Hub–Spoke Cycle-Packing LB: `81k–102k` rows / `48k–109k` B + `38k–104k` random `memcpy`s + $O(r\log r)$ graph tax, `2.8–6.2 MiB` WS) | Proved Theorem 1.1 (`1.59–1.90r` metric-DAG arrivals, refuting $2^{35}\text{–}2^{63}$ phantom blowup) and Theorem 1.2 ($\le 96$-seed byte acyclicity + zero-split copy-DAG replay without run labels); break points to $\Lambda \to 1..4$ Fox–Morse interval collapse in Cycle 2 |
| 2026-11-21 | Wormhole cycle 2 (`research/carbon-wormhole`; [journal](../../journal/2026-11-21-wormhole-cycle2-morse-quasideterminant-cliff.md)) | Width-filtered Fox–Morse collapse $\mathcal{M}_\Lambda$ ($\Lambda \in \{1,2,4,8,16\}$) via Gel'fand–Retakh quasideterminants (`26,135` small + `30` block decodes); Milnor–Fox–Perseus $\mathbb{Z}[t, t^{-1}]$ augmentation (`5,227` small + `6` blocks); Perseus index-order local apparent pairs | Wordless $\mathbb{Z}[t, t^{-1}]$ decoding at $\Lambda=1$ ($n=5$ primitive twin `20011` vs `21100`, Theorem 2.2); index-order local chain compression (`0.04–0.07%` 64B hits, Theorem 2.3); eager SLP flattening at $\Lambda \to 1$ (`633–821 KiB` dict, `61–76%` single-use `width=1` strings, Theorem 2.1) | Isolated exact L2 vs flattening trade-off (`edges` drops to `134–379 KiB` and `>16B` steps fall `29–52%`, blocked only by `spelling` flattening + RV steps); break points directly to Rytter–Gasieniec output-memoized SLP (`0 B` `spelling`) in Cycle 3 |
| 2026-11-21 | Wormhole cycle 3 (`research/carbon-wormhole`; [journal](../../journal/2026-11-21-wormhole-cycle3-slp-lz77-output-memoization.md)) | Zorich (1996) / Agol–Hass–Thurston (2006) periodic interval quotienting + Rytter (2003) / Gasieniec et al. (2005) output-memoized SLP decoding (`OM-Fox`, `0 B` `spelling`, `17–47 KiB` L1d `inline16`; `26,495` small/random + `30` block decodes; 15-round C Gate G5 confrontation) | `OM-Fox<16>` (`1.041–1.077×` `Open058`: L2-hot `50K` flat `spelling` beats `900K` `out` self-copies); `OM-Fox<8,4,2>` (`1.15–2.07×` time, `5.0–10.7 MiB` scratch: $O(H_\Lambda)$ `Endpoint` cost dominates even at `0 B` `spelling`); Zorich/AHT on Wikidata (`0–2` steps saved due to pre-BWT `RLE1`) | Closed $\Lambda < 16$ and SLP-deferral cleanly; exposed `Open058`'s `4.73 MiB`/worker scratch (`38–55 MiB` at 7–10t > `20 MiB` i9 L3) + `MLP=1` walk, pointing directly to Cycle 4's zero-copy Time-Reversal Meet-in-the-Middle & L3-resident Conjugacy/Arena chains |
| 2026-11-21 | Wormhole cycle 4 (`research/carbon-wormhole`; [journal](../../journal/2026-11-21-wormhole-cycle4-dihedral-bidi-and-conjugate-stream.md)) | Birkhoff (1915) / Pohl (1971) zero-copy dihedral meet-in-the-middle walk (`Bidi-Open`, escaping Depth Thm 6 staging); Kakutani (1943) conjugate `u16`-stream `Vec16` wiring + Chaitin (1982) lifetime overlay (`CSW-Open`; `26,495` small/random + `6` blocks; 25-round C Gate G5 confrontation) | `Bidi-Open` (`emit` `-7..9%`, offset by `+101..318 µs` dual `eb[sj]` scatter wire & `6.13 MiB` scratch); `CSW-Bidi` (`1.007–1.109×`) | **`CSW-Open` wins `6/6` blocks (`0.973–0.991×` `Open058`) at `0.573×` scratch (`2.67 MiB` vs `4.66 MiB`, `0 B` queues, overturning `#659`'s `1.10–1.15×` loss!)**; stage attribution isolates band-span SIMD `wire` + `Endpoint` `u16` compaction + `I_0` Pigeonhole arena for Cycle 5 |
| 2026-11-21 | Wormhole cycle 5 (`research/carbon-wormhole`; [journal](../../journal/2026-11-21-wormhole-cycle5-pigeonhole-chaitin-synthesis.md)) | Dirichlet Pigeonhole on `Runs` $I_0$ (`counts`, `per`) + Direct-Linked 16B `u16` `Node16` (`0 B` `ord/term`) + 100% Chaitin (*POPL* 1982) `Endpoint`-in-`edges` overlay + Kakutani `u16` thin-stream (`W1: Chaitin-Open`, `W2: CPC-Micro`; `26,495` small/random + `6` blocks; 25-round 1t & 4t barrier C confrontation) | Flat `16*per[a]` queue arena (`CAP=127` thick records inflate `16*per[a]` to `3.4–4.4×` true $m$, `2.7 MiB`, Theorem 5.1) | **`W2: CPC-Micro` cuts persistent scratch to `1.48 MiB` (`0.316×` `Open058`, `559–1,173 KiB` active peak; 7 i9 workers = `16.1 MiB <= 20 MiB` L3!) and wins `6/6` blocks at 1t (`0.947–0.982×`) and 4t (`0.927–0.972×`)**; `W1: Chaitin-Open` (`0.476×` scratch, `0.967–0.985×` 1t, `0.947–0.982×` 4t) also ready for Coordinator Rust port |
| 2026-11-21 | Wormhole cycle 6 (`research/carbon-wormhole`; [journal](../../journal/2026-11-21-wormhole-cycle6-bmcp-floor-and-gap.md)) | Six-dimensional $\mathsf{BMCP}(M,B,\mu)$ lower & upper bounds in safe Rust against `origin/master` `d425d77`: Pătraşcu–Viola succinct vs 1-probe boundary (Thm 6.1), 4-Hamming text-splice adversary across $7!=5{,}040$ perms (Thm 6.2), exact Belady `OPT` vs full-trace `LRU` at `32K/256K/2048K` + Lever 2 prefix scaling `100k..900k` (`16,209` small/stress + `6` blocks) | Unrestricted cell-probe L2-miss lower bounds (`RLBWT` info is `56–133 KiB < 256 KiB` L2, escaped only at `>=3–4x` serial probe depth); 1-pass `900 KiB` staging at `W=10` (`exceeds ~270 KiB/worker L3 headroom`) | Pinpointed exact gap: Dieted `Open` meets `2048K` L3 floor (`479–831` misses/block) & $Q_{\mathrm{rand}}=m_{16}$, but chains `100%` of `25.5k–94.7k` L2 misses serially ($\mu=1$, `2.3–3.4×` Belady `OPT` and `4–8×` above $\mu$-MLP); points Cycle 7 to $\Phi$/toehold & Lyndon global-anchor constructions |
| 2026-11-21 | Wormhole cycle 7 (`research/carbon-wormhole`; [journal](../../journal/2026-11-21-wormhole-cycle7-phi-lyndon-sais-move-d.md)) | RH-Journal Move D global-order constructions in safe Rust (`#![forbid(unsafe_code)]`): $\Phi$/toehold sampling via weight-only RV (`14,204` small + `6` blocks), Duval Chen–Fox–Lyndon / BBWT oracle, and exact $O(r)$ BWT-domain SA-IS LMS formula $\mathrm{LF}(x)<x \land \mathrm{LF}^2(x)>\mathrm{LF}(x)$ (`2,012` small + `6` blocks) | $\Phi$/toehold RV bootstrap (`2.20–5.53 MiB` scratch > `2048K` L3, `93.9k–335.6k` L2 misses = `2.6–3.8×` Dieted `emit`, Thm 7.1); BBWT/Lyndon (`34.6–93.8%` single giant factor, Thm 7.2); Backward SA-IS (`305k–317k` LMS rows = `1.51–3.42×` $m_{16}$, Thm 7.3) | Proved `Open<16>`'s $m_{16}$ table is strictly smaller than $\Phi$-RV and SA-IS LMS; forces Cycle 8 to split `edges[0..m)` itself via zero-bit xor-mask cuts within the `2,048 KiB` L3 budget |
| 2026-11-21 | Wormhole cycle 8 (`research/carbon-wormhole`; [journal](../../journal/2026-11-21-wormhole-cycle8-zero-bit-cuts-and-2p-ws.md)) | Zero-bit xor-mask cuts `((j ^ mark) & cm) == 0` (`14,409` multi-cycle/multi-epoch cases + `6` blocks in safe Rust vs `origin/master` `OpenCells`), `1P-Stage256K` L3-safe bounded staging, and `2P-WS8` (`0 B` staging heap, `1.885 MiB`/w WS -> `10w = 18.85 MiB <= 20 MiB` L3) | `1P-Stage256K` (`256 KiB` L3-safe cap stalls aux lanes after `6.0–13.0%` of $m$, Thm 8.2(1)); `S = 17..27` cut count (Rényi $\frac{H_S}{S} > \frac{1}{8}$ straggler inflates `rounds_4a/4b` up to `2.27×` $m/8$, Thm 8.3) | **`2P-WS8` cuts critical-lane `dep_past_l2` to `0.34–0.59×` (`8.9k–52.9k` vs `25.5k–94.7k`) with `0` extra `2048K` L3 misses (`479–831`), `Pass 4b` beats `Diet` `emit` by `-8.8..32.8%` (`6/6`), and wins end-to-end on `block-143.L`**; points Cycle 9 to `S=65..128` Rényi de-straggling + `Relay-MLP` overlapped scouting + post-slop boundary restoration |
