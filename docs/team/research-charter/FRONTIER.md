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
| **The origami lift:** a BWT block as a branched torus cover with monodromy (LF, +1) | Surface operations, and directions with a transport law, unavailable in the one-dimensional exchange. MONODROMY verified the construction (record 86); the cone points alone cannot decode. Carbon Kitchen Sink cycle 1 proved that cone-point cycles equal those of the width-1 reduced run-word w_runs (6/6 blocks), closing §6.1, and measured simple shears r u^{±1,±2} (13–71 cycles, 0–3 fixed points) | [journal](../../journal/2026-11-20-origami-lift.md); [cycle 1](../../journal/2026-11-20-carbon-kitchen-sink-cycle1.md) | §6.1 closed; general Veech directions open |
| **Illegal turns and folding** (Bestvina–Handel, Stallings) | If a graph self-map of the run dynamics can be built cheaply, its illegal-turn structure might isolate the few branching decisions that carry the ordering, resolved collectively by folds. Carbon Kitchen Sink cycle 4 measured `100%` same-byte outgoing switches and `99.87–99.99%` same-byte incoming switch adjacencies on $G_{\mathrm{ov}}$ (distinct bytes occur at $\le \alpha - 1$ bucket cuts: `3–14` per block), proving byte-labelled Stallings folds collapse $G_{\mathrm{ov}}$ toward the $\alpha$-vertex alphabet quotient unless planar metric widths are preserved (which reduces to RV splitting) | Bestvina–Handel (Ann. Math. 1992); [cycle 4](../../journal/2026-11-20-carbon-kitchen-sink-cycle4.md) | Stallings byte-folding obstructed (cycle 4) |
| **Ribbon graphs and rotation systems** | Restoring the cyclic order at each vertex encodes a surface embedding. Faces are cycles of σ∘α. Carbon Kitchen Sink cycle 4 proved $\mathrm{faces}(\sigma\circ\alpha) \cong \mathrm{cycles}(\Pi_{\mathrm{wall}})$ (`60–484` faces, genus $g_{\mathrm{rib}} = (k+2-F_{\mathrm{rib}})/2 = 5,448\text{--}13,336 \approx 0.21\,g_{\mathrm{orig}}$; 1,514 exact checks), and proved **Ribbon Metric-Blindness** via an equal-$(n,C,p)$ primitive witness at $n=10, r=4$ (`1000111100` vs `1001111000`) | Standard combinatorial-map literature; [origami](../../journal/2026-11-20-origami-lift.md); [cycle 4](../../journal/2026-11-20-carbon-kitchen-sink-cycle4.md) | pure ribbon embedding metric-blind (cycle 4) |
| **Splitting integer-weighted train tracks, and contraction** | Dunfield–Thurston count the components of weighted multicurves by splitting bands, which they identify with Rauzy induction, without tracing the curves. Carbon Kitchen Sink cycle 1 built exact series-reduced train-track RV (`TTSR-RV`: 0.407–0.497r winner-runs, m = 0) and Void-Deleted Exact-Chain Open (`DECO`: m' = 0.42–0.52m, emit 0.63–0.69× Open); both lost end-to-end in C (1.78–3.59× Open) due to O(r) run-array materialisation (`m/r ≈ 1.6`) | Dunfield–Thurston (2006), §7.3; Shun et al.; Cong–Bader; [cycle 1](../../journal/2026-11-20-carbon-kitchen-sink-cycle1.md) | tested (cycle 1); O(r)-materialised forms obstructed |
| **Agol–Hass–Thurston interval-pairing reduction** | Its input, integer intervals paired by translation, matches a block's runs, and the algorithm resolves global structure without walking points. It computes component counts, not order. Can a word-carrying version recover the marked orbit more cheaply than Cells or Open? | Agol, Hass and Thurston, *The computational complexity of knot genus and spanning area* (2006), the interval-pairing algorithm; Toby Hall's *Trains* software for Bestvina–Handel | partially tested via `TTSR-RV` (cycle 1); non-materialised pairings open |
| **Working backwards from the text, and an algorithm zoo** | Start from the text's structure and find which classes own the expensive rows. Measured: the JSON skeleton is already cheap (keys: 33–38% of bytes, 1–4% of thin rows). Unique GUIDs own 54–67% of width-1 rows; repeated sha1 reference hashes are thin, copyable rows. Questions: an operation on random segments that isn't a walk, or a scoped floor; a charged copy mechanism for repeated random strings; and one cheaply recognised primitive per class, oracle headroom first | [journal](../../journal/2026-11-20-working-backwards.md); Hutter Prize and STARLIT as provocation | pending |
| **Toy, missing ingredient, native surrogate** (the method of Alpöge's announcement note) | Re-derive inversion where it is already free, with ingredients chosen to transfer. Carbon Kitchen Sink cycle 1 proved the Koopman–Row-Variation Identity and verified that 2D Tribonacci Rauzy fractals cut into 1,024 pieces (mean width 2.42) preserve `|\{\Delta\tau\}|/r = 0.041` and low Koopman TV (`0.113`), whereas Wikidata blocks have `|\{\Delta\tau\}|/r = 0.88–0.93` with bimodal split: thick runs show local JSON periodicity (`tv = 0.18–0.54`), while width-1 GUIDs hit white noise (`tv = 0.83–0.90`) | [journal](../../journal/2026-11-20-toy-transfer.md); [cycle 1](../../journal/2026-11-20-carbon-kitchen-sink-cycle1.md) | torus coordinate obstructed on GUIDs; bimodal surrogate open |
| **The kitchen sink: use everything** (after the relayed overview of the claimed RH development; Kontorovich: "RH is proved by using *everything*!") | Active in `research/carbon-kitchen-sink`. Cycles 1–3 published (`LRB-Vec16` handed to Coordinator for native Rust validation); Cycle 4 proved Ribbon-Wall / Veech duality, Ribbon Metric-Blindness, Universal Schur-Complement Knapsack Optimality of `Open`, Tropical LCP duality, and Post-BWT `RLE1 ∘ CRC32` transducer lift | [journal](../../journal/2026-11-20-kitchen-sink.md); [cycle 1](../../journal/2026-11-20-carbon-kitchen-sink-cycle1.md); [cycle 2](../../journal/2026-11-20-carbon-kitchen-sink-cycle2.md); [cycle 3](../../journal/2026-11-20-carbon-kitchen-sink-cycle3.md); [cycle 4](../../journal/2026-11-20-carbon-kitchen-sink-cycle4.md) | active (cycles 1–4 published) |

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
| Dependent traversal of retained rows | Open: ≈ one dependent load per retained row; emission 36–48% of time (#653). `DECO` (cycle 1) cut emit to 0.63–0.69× Open via O(r) producer; `OIBS-H` (cycle 2, OPEN-QUESTIONS.md §4) cut hops 15–36% online but lost 1.13–1.27× on L2-resident `edges`. **Second section-cut domain-splitting census (cycle 3, OPEN-QUESTIONS.md §1):** forest section cut on `Open`'s map shrinks $m$ 9–12× (`m_2 = 7,543–20,432`), *but* domain $\sigma: J \to K$ splits across $V_1 = r_{\mathrm{thin}} = 0.93\text{--}0.98\,r$ thin runs (`54,575–144,171` arcs, not $O(B)$), hitting the Ephemeral-Run Obstruction (`649–1,739 µs` > `494–1,103 µs` emit ceiling). Unified Free-Monoid `Slot16` emit (`Vec16`, cycle 3) removes `code < 256` mispredicts | Can output-order relations be resolved collectively without materialising explicit O(r) run graphs or adding branches to L2-resident `emit`? | open; §1 and §4 bounded |
| Building the retained map | Open's run pass and queue fill 28–36%; queued wiring 23–28% (#653). Compression (Q) and fusion (F) lost (#654); Queue-Free Backward Open lost 1.10–1.15× (#659). Cycle 3 (`g5_cycle3_bench.c`, `C2` & `C4`) halved `pass1` (`422 → 219 µs` on block 0; `890 → 472 µs` on block 210) via coprime-stride non-4K-aliased queues + branchless $\mathbb{Z}^{16}$ AVX2 iota stores | Can the remaining `pass1` / `wire` work be reduced further without a second record pass? | active; `pass1` halved in C (`Vec16`) |
| Acquiring the Φ-overlap coordinates | Overlap structure is 1.21–1.25r arcs (`E_ov`). **Exact Cohomological Fragment Theorem (cycle 3, OPEN-QUESTIONS.md §2 & record 82):** proved $F_{\mathrm{cyc}}(\phi) = \|\omega + d\phi\|_{0,w} + |\ker(\omega + d\phi)| - r$ in $H^1(G_{\mathrm{ov}};\mathbb{Z})$ (`4,542/4,542` checks exact). On the 6 blocks, zero-holonomy cycles save only `25–665` rows, locking $F_{\mathrm{cyc}}(\phi^*) = 107,457\text{--}126,974$ (`0.119–0.141n`, exceeding `Open`'s $m$ on blocks 0 & 2), and locating chord text positions requires `1.015r` RV steps + `107K–127K` forest return steps before `copy_replay` | Can any non-potential representation bypass the $0.12n$ non-zero holonomy chord floor? | bounded by $H^1(G_{\mathrm{ov}};\mathbb{Z})$ holonomy |
| Full-block metadata | Grammar and interval-exchange machines carry 40–100 B per run across all r runs, mostly width-1 runs (Neon cycle 1; Astra D). **Ephemeral-run materialisation obstruction (Carbon Kitchen Sink cycle 1):** *witness:* `g5_kitchen_bench.c` `pass1` alone (649–1,739 µs to materialise `run_start, run_width, img_start, cod_order` + one 2r merge) exceeds Open's streaming `pass1` (415–879 µs) by more than the entire dependent-walk savings, because `m/r ≈ 1.56–1.97` and Open keeps runs ephemeral in registers (materialising only `H ≤ 5,047` thick strips); *scope:* single-thread decoders on `1/64 ≤ R/n < 0.15` blocks that materialise explicit O(r) domain/codomain run arrays + multi-pass merges; *escape:* keep runs ephemeral in registers during a single pass, operate at O(H) scale, or fuse with pre-record MTF/Huffman | Can a collective primitive keep non-thick runs completely ephemeral? | open; scoped obstruction recorded |
| Starting from the decoded column | Cycle 1 proved the Exact MTF–Run Bijection; Cycle 2 (`LRB-Open-AVX2`) reached 0.925–0.944× vs `2Stage-AVX2`. **Cycle 3 (`LRB-Vec16`, `g5_cycle3_bench.c`, 20,908 small cases + 6 blocks byte-exact, ASan/UBSan clean):** composing `LRB` pre-column stream fusion with coprime-stride $\mathbb{Z}^{16}$ vector queue fill + Free-Monoid `Slot16` emit achieves **0.773–0.808× vs shipped `2Stage-qi16` on all 6/6 blocks** (cutting `front+pass1` to **0.485–0.537×**, and beating unconfounded `2Stage-Vec16` at **0.917–0.957×** across all 6/6 blocks while saving `112–266 KB` of `Runs.runs`) | Does `LRB-Vec16` carry its 0.773–0.808× (`front+Open`) speedup into native Rust on `mibz2-workers-lazy` and 6–20 thread `OpenCells`? | **candidate ready for native port check (0.773–0.808× C)** |
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
| Materialising explicit O(r) run/partition arrays + multi-pass merges (`m/r ≈ 1.56–1.97`) costs more than Open's streaming register pass (649–1,739 µs vs 415–879 µs) | Multi-pass O(r) run-graph or thin-chain contractions (`TTSR-RV`, `DECO`) as single-thread replacements for Open on `1/64 ≤ R/n < 0.15` blocks | Carbon Kitchen Sink cycle 1 (`g5_kitchen_bench.c`, 21 paired rounds; not yet audited): `DECO-w1-fast16` cuts emit to 0.63–0.69× Open, yet loses 1.86–2.26× overall; `TTSR-RV` loses 2.55–3.59× |
| Online bundle/output shortcutting inside `Open::emit` (`OIBS-H`, OPEN-QUESTIONS.md §4) costs 1.35–1.62× in `emit` despite skipping 15.3–36.2% of hops | Single-walk online sheet/output shortcutting on L2-resident `edges[0..m]` (`m ≲ 2.5 × 10^5` on 2 MiB L2) | Carbon Kitchen Sink cycle 2 (`g5_cycle2_bench.c`, 21 paired rounds; 38,838 small cases + 6 blocks byte-exact; not yet audited): `Open-fast16` pays 5.5–7.0 ns/hop in L2, whereas 12.5% cache-line-boundary neighbour checks + cold-sheet branch mispredicts + dirty writebacks exceed the skipped-hop savings (1.127–1.269× overall) |
| Second section cut on `Open`'s retained map (`OPEN-QUESTIONS.md` §1) requires $\Theta(r_{\mathrm{thin}})$ pieces (`54,575–144,171` arcs, `93–98%` of $r$), not $O(B)$ | Building a second forest/section cut from `Open`'s bands alone in $O(B)$ time | Carbon Kitchen Sink cycle 3 (`probe_cycle3.py`, 6 blocks): although $m_2 = 0.084\text{--}0.112\,m$ (`7,543–20,432`), $\sigma: J \to K$ splits across $V_1 = r_{\mathrm{thin}}$ domain runs, hitting Cycle 1's Ephemeral-Run Obstruction (`649–1,739 µs` > `494–1,103 µs` emit ceiling) |
| $H^1(G_{\mathrm{ov}};\mathbb{Z})$ non-zero holonomy locks $\Phi$-potential fragments at $F_{\mathrm{cyc}}(\phi^*) = 107,457\text{--}126,974$ (`0.119–0.141n`) | Sub-$0.12n$ fragment counts in any run-potential $\Phi$-copy schedule $\tau = \theta + \phi(\mathrm{run})$ (`OPEN-QUESTIONS.md` §2, `FRONTIER.md:106`) | Carbon Kitchen Sink cycle 3 (`verify_cycle3.py`, 4,542 checks exact; `probe_cycle3.py`, 6 blocks): proved $F_{\mathrm{cyc}}(\phi) = \|\omega + d\phi\|_{0,w} + |\ker(\omega + d\phi)| - r$; zero-holonomy chords save $\le 665$ rows |
| Combinatorial ribbon embeddings $(G_{\mathrm{ov}}, \sigma, \alpha, b_u)$ are constant across the $\beta_1$-dimensional Veech chamber $\mathcal{C}(G_{\mathrm{ov}}, \sigma)$ (`10,955–26,965` DoF) | Decoding text order from pure ribbon-graph face walks or byte-labelled Stallings folds without metric width comparisons | Carbon Kitchen Sink cycle 4 (`verify_cycle4.py`): equal-$(n,C,p)$ primitive witness at $n=10, r=4, p=0$ (`1000111100` vs `1001111000`) has identical labelled ribbon graph yet decodes to 4 vs 6 text runs; `99.87–99.99%` of incoming switch adjacencies share the same byte |
| Word-homogeneous affine Schur complements satisfy $\|J\| \ge n - \sum_{j=1}^H w'_{(j)}$ universally | Alternative $H$-strip Schur complements (e.g. alphabet-subset complements) beating `Open`'s retained count $m$ at strip budget $H$ | Carbon Kitchen Sink cycle 4 (`verify_cycle4.py`, 72,816 subset checks; `probe_cycle4.py`, 6 blocks): `Open<16>` achieves `0` rows of slack on 6/6 blocks; alphabet-subset complements leave `3.3–4.9×` more retained rows |

## Connections not yet explored

These are pointers, not results:
- **The run network** ([journal](../../journal/2026-11-20-run-network.md)): LF as a graph on runs, built in O(r).
  - Its cycle rank, 0.21–0.25r, is fixed by record 63's identity E = 2r − 1 − C, and equals the copy count k plus one.
  - 88–89% of runs are pass-throughs, leaving a series-reduced core of about 0.11r runs.
  - With order-preserving switches it is a measured train track.
  - Census (bd781368): the core has no small separators, though it bisects 4–8× better than random. As a retained set for first returns it is worse than Open's.
  - Cycle 1 update (`TTSR-RV`): accelerating RV on the 0.33r-piece core drops winner-run steps to 0.407–0.497r (below the raw r floor) with m = 0, but building the O(r) network + SLP expansion loses 2.55–3.59× to Open.
- **The origami lift** ([journal](../../journal/2026-11-20-origami-lift.md)): a BWT block is a branched cover of the torus with monodromy (LF, +1). MONODROMY verified it (record 86). Cone cycles proved isomorphic to `w_runs` cone cycles (closing §6.1). Cycle 4 connected it to the Veech zippered-rectangle ribbon surface $\Sigma_{\mathrm{rib}}$ ($g_{\mathrm{rib}} = (k+2-F_{\mathrm{rib}})/2 \approx 0.21\,g_{\mathrm{orig}}$). Still open: general Veech-group / non-shear transport (§6.2).
- Kempa (SODA 2019) and Agol–Hass–Thurston: sublinear routes to run-boundary samples. Could an
  o(n) sample construction feed a reconstruction?
- LCP-chain structure (Kärkkäinen–Kempa–Piątkowski): Cycle 4 verified exact tropical duality $\mathrm{LCP}_F[y] = \min(n, 1 + \mathrm{LCP}_F[\Psi(y)])$ outside $\overline{C}$ (`4.89M` rows) and measured $\sum_{y\in\overline{C}} \mathrm{LCP}_F[y] = 0.96\text{--}1.39n$ (median `3` bytes = GUID hex collision depth).
- Stochastic complementation (Meyer 1989): **closed for affine strip complements in Cycle 4** — `Open` achieves the universal greedy knapsack lower bound $m^*(H)$ with `0` rows slack on 6/6 blocks.
- Derived sequences and return substitutions for the finite marked BWT (Durand), applied to the
  repeated-cut limit.

## Cycle log

Append one entry per research cycle: the date, the session, what was explored, what was killed and
why, and what was promoted.

| Date | Session | Explored | Killed (why) | Promoted |
|---|---|---|---|---|
| 2026-11-19–20 | Neon rounds 1–7 and follow-ups; the Coordinator | parallel induction; boundary samples; copy replay; NSPC; DOE; Open micro-optimisations; LIMIT | see CONTEXT.md §4 | none to production; Open shipped (0.5.7) from the Astra line |
| 2026-11-20 | Neon cycles 3–5 (branch `research/neon-bipolar-contraction`, 4598c5bf; not yet audited beyond BSO's timing) | bucketed Schur; unsplit forest; output-sourced anchors; bit-sliced implicit Open (BSO) | anchors (width-1 runs collapse interval certificates); BSO (serial rank latency, above) | none; Neon paused |
| 2026-11-20 | Carbon Kitchen Sink cycle 1 (`research/carbon-kitchen-sink`; unverified by MONODROMY) | 10 frameworks: Koopman/2D Rauzy fractals; origami cone-isomorphism & shears; quantum-torus few-nomials / Hensel / F_257 LFSR; left-regular-band MTF; RSK shapes; Wheeler k-grams; `TTSR-RV` (`0.41–0.50r` RV); `TTO` & `DECO` (`0.42–0.52m` walk) | global torus on GUIDs (Koopman TV 0.83–0.90); cone-cycle decoding (length-blind); Hensel/LFSR (nilpotency n, deg N/2); `TTO` (shatters thick strips); `TTSR-RV` & `DECO` in G5 C bench (1.78–3.59× Open: O(r) run-array materialisation exceeds walk savings at `m/r ≈ 1.6`) | none to native; theorems & Ephemeral-Run Obstruction recorded |
| 2026-11-20 | Carbon Kitchen Sink cycle 2 (`research/carbon-kitchen-sink`; unverified by MONODROMY) | Étale sheet coherence & Cyclic No-Wraparound Lemma (`OIBS-H4/H6`, OPEN-QUESTIONS.md §4; 38,838 small cases + 6 blocks byte-exact); Left-Regular-Band pre-column stream fusion (`LRB-Open-AVX2`, Obligation 5) | `OIBS-H4/H6` in G5 C bench (1.127–1.269× Open despite −15.3 to −36.2% hops: L2-resident `Open` pays 5.5–7.0 ns/hop, smaller than neighbour-split + cold-sheet branch + dirty-writeback tax); `LRB-OIBS` (1.058–1.159×) | `LRB-Open-AVX2` measured **0.925–0.944×** vs `2Stage-AVX2` on all 6/6 blocks (0.819–0.852× on `front+pass1`), held below strict ≤0.900× gate pending 256-queue bottleneck (`317–700 µs`) |
| 2026-11-20 | Carbon Kitchen Sink cycle 3 (`research/carbon-kitchen-sink`; unverified by MONODROMY) | $H^1(G_{\mathrm{ov}};\mathbb{Z})$ cohomology of $\Phi$-overlap schedules (`OPEN-QUESTIONS.md` §2; 4,542 exact checks); second section cut from `Open`'s bands (`OPEN-QUESTIONS.md` §1); `LRB-Vec16` synthesising Left-Regular-Band stream fusion + coprime-stride $\mathbb{Z}^{16}$ queue iota + Free-Monoid `Slot16` emit (20,908 small cases + 6 blocks byte-exact) | $O(B)$ second section cut ($\sigma$ splits across $V_1 = 0.93\text{--}0.98\,r$ thin runs, hitting Ephemeral-Run Obstruction); sub-$0.12n$ potential schedules (locked at `107K–127K` fragments by non-zero GUID holonomy) | **`LRB-Vec16` passed Gate G5 vs shipped `2Stage-qi16` on 6/6 blocks (`0.773–0.808×` total `Symbols`-to-`out`, `0.485–0.537×` on `front+pass1`, `0.917–0.957×` vs `2Stage-Vec16`)**; handed to Coordinator for native Rust validation |
| 2026-11-20 | Carbon Kitchen Sink cycle 4 (`research/carbon-kitchen-sink`; unverified by MONODROMY) | Grothendieck ribbon surface $\Sigma_{\mathrm{rib}}$ / Veech zippered rectangles (`1,514` wall-bijection checks); Bestvina–Handel / Stallings folds on $G_{\mathrm{ov}}$; Meyer Schur-complement knapsack optimality (`72,816` subset checks); tropical irreducible LCP duality (`4.89M` rows); post-BWT `RLE1 ∘ CRC32` Krohn–Rhodes / $\mathbb{F}_2[x]/(P_{32})$ transducer (`6/6` blocks) | Pure ribbon decoding (metric-blind on $\mathbb{Z}^{\beta_1}$ chamber; equal-$(n,C,p)$ witness at $n=10, r=4$); byte-labelled Stallings folds (`99.87–99.99%` same-byte collapse); alternative affine Schur complements (`Open` has `0` rows slack; alphabet subsets `3.3–4.9×` worse) | Proved 4 theorems (`faces == cycles(Pi_wall)` with $g_{\mathrm{rib}} \approx 0.21\,g_{\mathrm{orig}}$; Universal Schur Knapsack Optimality; Tropical LCP Duality; `RLE1 ∘ CRC32` SLP transducer lift; `99.71–99.93%` identity 32B windows) |
