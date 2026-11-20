# The research frontier

*A living record. Research sessions read it at the start of every cycle and update it at the end
(METHOD.md §2, Learn). It holds:*
- *the questions that challenge the current primitives;*
- *the obstructions already known;*
- *the connections not yet explored;*
- *a log of cycles.*

*Evidence lives in [CONTEXT.md](CONTEXT.md) and the journals. This file points to it.*

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
| Dependent traversal of retained rows | Open: about one dependent load per retained row (≈ one per ten output bytes); emission 36–48% of time (#653) | Can output-order relations be resolved collectively, without a producer as expensive as the walk? | open |
| Building the retained map | Open's run pass and queue fill 28–36%; queued wiring 23–28% (#653). Compression (Q) and fusion (F) lost (#654) | Can the map be produced in the direction its consumer needs, so no inversion scatter is required? Candidate: Queue-Free Backward Open (Neon cycle 2; audit pending) | candidate under audit |
| Acquiring the Φ-overlap coordinates | The overlap structure is small: 1.25–1.37r transitions on the six blocks (Neon cycle 1). The known route via run-boundary positions costs Cells' 1.68–1.87r steps (record 81) | Can those coordinates come out of a cheaper computation, or out of an earlier representation? | open; positive measurement |
| Full-block metadata | Grammar and interval-exchange machines carry 40–100 B per run across all r runs, mostly width-1 runs (Neon cycle 1; Astra D) | Can a primitive avoid per-run global state? | open |
| Starting from the decoded column | Every kernel begins from run records after Huffman, MTF and RLE2 decoding | Does an earlier representation carry ordering information more cheaply? | unexplored |
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
| Histogram non-identifiability: same run histogram, Θ(n)-different spelling cost | Predicting construction cost from run statistics alone | Neon (unverified by MONODROMY) |

## Connections not yet explored

These are pointers, not results:
- **The run network** ([journal](../../journal/2026-11-20-run-network.md)): LF as a graph on runs, built in O(r).
  - Its cycle rank, 0.21–0.25r, is fixed by record 63's identity E = 2r − 1 − C, and equals the copy count k plus one.
  - 88–89% of runs are pass-throughs, leaving a series-reduced core of about 0.11r runs.
  - With order-preserving switches it is a measured train track.
  - Open question: do small separators in the core give an Euler-tour composition cheaper than walking?
- **The origami lift** ([journal](../../journal/2026-11-20-origami-lift.md)): a BWT block is a branched cover of the torus whose monodromy is (LF, +1). Its cone points are the run boundaries: exactly r squares, in only 1–8 cycles on the six blocks, and block 207 sits in the minimal stratum. It is a candidate change of basis, with four open questions. It is not yet a cheaper decoder.
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
